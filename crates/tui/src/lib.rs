//! epher-tui — native full-screen terminal frontend (ADR-0001).
//!
//! The testable seam is [`App`] (input/result + the shared [`Session`]) plus
//! the pure [`render_ascii`] plot renderer (ADR-0006: the TUI renders ASCII).
//! [`run`] is the ratatui event loop — a thin shell over both — exposed as a
//! library function so the unified `epher` binary can host it (`epher tui`).

use epher_core::astro::SolarScene;
use epher_core::graph::{
    analyze, curve_frame, free_names, parse_graph_source, project_space_curve, project_surface,
    sample_data_plot, sample_space_curve, sample_spec, sample_surface, surface_frame, zoom_window,
    DataPlot, InterestKind, InterestPoint, SampledCurve, Segment3D, SpaceCurve, Surface, View3D,
};
use epher_core::Session;
use epher_i18n::Localizer;
use epher_shell::{classify, plain, run_command};
use epher_store::persist::{
    default_store_dir, load_exact, load_format, load_language, load_pois, load_separators,
    load_session, load_theme, save_exact, save_format, save_history, save_language, save_pois,
    save_separators, save_session, save_theme,
};
use epher_store::{DocStore, FsStore};
use unicode_width::UnicodeWidthStr;

/// The TUI's color theme (ADR-0017): dark is the terminal's natural
/// look; light forces a light canvas; night keeps long-wavelength reds on
/// near-black for dark-adapted eyes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    Night,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::Night => "night",
        }
    }

    pub fn from_str(name: &str) -> Option<Self> {
        match name {
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            "night" => Some(Theme::Night),
            _ => None,
        }
    }
}

/// What the file prompt under the menu bar is asking for (ADR-0017).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptKind {
    OpenHistory,
    OpenScript,
    SaveHistory,
    SaveScript,
}

/// One of the three 3D fine-control axes (ADR-0031): the Settings menu's
/// horizontal-rotation, vertical-rotation, and zoom rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewAxis {
    Horizontal,
    Vertical,
    Zoom,
}

/// The outcome of activating a menu item; `run` executes it with access
/// to the store and localizer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAction {
    OpenHistory,
    OpenScript,
    SaveHistory,
    SaveScript,
    Quit,
    Cut,
    Copy,
    Paste,
    SetTheme(&'static str),
    SetLanguage(&'static str),
    /// Show or hide the points-of-interest list in the graph panel
    /// (ADR-0019); the loop persists the choice.
    TogglePois,
    /// Toggle exact fraction display (ADR-0043); the loop persists.
    ToggleExact,
    /// Cycle Auto -> Scientific -> Engineering notation (ADR-0043).
    CycleNotation,
    /// Toggle thousands separators on results (ADR-0043).
    ToggleSeparators,
    /// Empty the graph pane (curves, points of interest, 3D surfaces).
    ClearGraph,
    /// Copy every listed point of interest to the terminal's clipboard
    /// via OSC 52 (the ADR-0038 amendment): the TUI spelling of the
    /// web heading's copy button.
    CopyPois,
    /// Open the in-app user guide (ADR-0018).
    OpenGuide,
    /// Open the keypad key-help overlay (ADR-0039): the current bank's
    /// keys with their meanings.
    OpenKeyHelp,
    /// Open the constants browser (ADR-0045): the grouped builtin
    /// constants; Enter inserts the selected name into the input line.
    BrowseConstants,
}

/// An active parameter animation: `name` steps by `step` within `lo..=hi`,
/// wrapping around (Desmos-style loop).
#[derive(Debug, Clone, PartialEq)]
pub struct Play {
    pub name: String,
    pub lo: f64,
    pub hi: f64,
    pub step: f64,
}

/// The TUI's application state — the testable seam. Rendering is thin.
#[derive(Default)]
pub struct App {
    input: String,
    /// The insertion point inside `input` (byte offset): Left/Right and
    /// a mouse click move it, typed characters insert there, Backspace
    /// deletes before it (ADR-0035 amendment, TUI). Kept at the end
    /// while the input is empty or after a submit.
    cursor: usize,
    result: String,
    session: Session,
    graph: Vec<SampledCurve>,
    pois: Vec<InterestPoint>,
    surface: Vec<Surface>,
    /// The 3D parametric curves (`graph3d param ...`, ADR-0054): the
    /// curve sibling of the surface set.
    curve3d: Vec<SpaceCurve>,
    /// The data plot (ADR-0044): a scatter, histogram, or boxplot owns
    /// the pane like the solar scene does — the newest command wins.
    data: Option<DataPlot>,
    /// The solar system scene (`solar3d`, ADR-0037) and the source of
    /// its time expression, for playback resampling.
    solar: Option<SolarScene>,
    solar_source: Option<String>,
    view: View3D,
    /// The 3D fine-control offsets (ADR-0031): horizontal rotation,
    /// vertical rotation, zoom — each −1..1, step 0.1, 0 = the orbit
    /// pose unchanged.
    view_h: f64,
    view_v: f64,
    view_z: f64,
    /// The 2D graph's viewport override (ADR-0034): mouse drags pan,
    /// the wheel zooms — `None` is the auto-fit around the samples.
    view2d: Option<(f64, f64, f64, f64)>,
    play: Option<Play>,
    /// Keypad focus mode (ADR-0016): Tab opens the button grid and
    /// switches its banks, arrows move the highlight, Enter appends
    /// the token, Esc closes.
    keypad: bool,
    /// The keypad's docked-away state (ADR-0060): true has slid the
    /// keypad out of view — the strip row where its top border sat
    /// remains as the grab area, and the history list grows into the
    /// freed space. Inverted so `App::default()` (the derived one)
    /// starts with the keypad shown. Session state; never persisted.
    keypad_docked: bool,
    /// The key-help overlay's scroll offset while open (ADR-0039).
    key_help: Option<usize>,
    /// The constants browser's selected row while open (ADR-0045): the
    /// index counts constant rows only; group headings are not
    /// selectable. Enter inserts the name into the input line.
    constants: Option<usize>,
    /// The browser's rows, snapshot at open: (name, value, hint) in
    /// the guide's group order — Math, Astronomy, Physics, Chemistry.
    constants_rows: Vec<(String, Option<f64>, String)>,
    kp_bank: usize,
    kp_row: usize,
    kp_col: usize,
    /// The color theme (ADR-0017).
    theme: Theme,
    /// Whether the graph panel lists the points of interest (ADR-0019).
    poi_list: bool,
    /// Result rendering (ADR-0043): exact fractions, notation, and
    /// thousands separators, applied to every submitted result.
    display: epher_core::DisplayPrefs,
    /// The open menu bar item and the highlighted row inside it
    /// (ADR-0017); `None` when the menus are closed.
    menu: Option<(usize, usize)>,
    /// An active file prompt: its kind and the path typed so far.
    prompt: Option<(PromptKind, String)>,
    /// History focus mode (ADR-0027): Tab reaches the history list, the
    /// arrows move the selection, and Enter loads the selected line into
    /// the input — the terminal spelling of the web's clickable history.
    /// `hist_sel` indexes the DISPLAYED list (0 = newest line on top).
    /// `hist_rows` maps each displayed ROW of the last frame to the
    /// displayed entry that owns it (ADR-0027 amendment: multi-line
    /// script entries occupy several rows, separated by rule lines).
    hist_focus: bool,
    hist_sel: usize,
    hist_rows: Vec<usize>,
    /// The in-app user guide view (ADR-0018): `Some(scroll)` when open,
    /// the offset counting wrapped rows from the top. `guide_chapters`
    /// holds the top-level chapter titles with their wrapped-row offsets
    /// in the CURRENT frame (ADR-0018 amendment): the table of contents
    /// at the top of the pager jumps to those rows on click or 1–9.
    guide: Option<usize>,
    guide_chapters: Vec<(String, usize)>,
    /// The guide text for the current locale, loaded from the installed
    /// files the first time the guide opens (ADR-0053): the markdown is
    /// never carried in the binary. The locale rides along so a language
    /// change reloads; `Err` says where it looked.
    guide_md: Option<(String, Result<String, epher_guide::GuideUnavailable>)>,
    /// The guide's search (ADR-0038 amendment): `/` starts a query, the
    /// typed text filters as you go, Enter jumps to the next hit. The
    /// hit rows are the wrapped rows of the matching lines in the
    /// CURRENT frame (written by the renderer, like `guide_chapters`).
    guide_searching: bool,
    guide_query: String,
    guide_hit_rows: Vec<usize>,
    /// The screen regions the last frame drew (ADR-0034): mouse events
    /// map their coordinates through these. Default until the first draw.
    areas: Areas,
}

/// The mouse-relevant screen regions of the last drawn frame (ADR-0034).
/// The menu bar's five label rects and the keypad's six bank-label rects
/// are stored rather than recomputed from label widths, so localized
/// labels stay clickable wherever their characters land.
#[derive(Debug, Clone, Copy, Default)]
pub struct Areas {
    pub menu_labels: [ratatui::layout::Rect; 5],
    pub input: ratatui::layout::Rect,
    pub result: ratatui::layout::Rect,
    pub history: ratatui::layout::Rect,
    pub graph: ratatui::layout::Rect,
    pub keypad: ratatui::layout::Rect,
    pub kp_bank_labels: [ratatui::layout::Rect; 7],
    /// The keypad's cell width and column count for the current bank,
    /// so clicks can map columns to cells with the same math as the draw.
    pub kp_cell_w: u16,
    pub kp_cols: usize,
    pub hints: ratatui::layout::Rect,
    /// The open menu popup: its menu index and the rect including border.
    pub popup: Option<(usize, ratatui::layout::Rect)>,
    /// The user guide's table-of-contents rows (ADR-0018 amendment): the
    /// rects the last guide frame drew, in chapter order. Clicks land on
    /// these to jump.
    pub guide_toc: [ratatui::layout::Rect; 12],
    pub guide_toc_len: usize,
    /// The history panel's scroll offset at draw time.
    pub history_scroll: u16,
    /// The column of the history title's trash glyph (ADR-0041): a mouse
    /// click there clears the history, like Ctrl+L.
    pub history_trash_col: u16,
    /// The input panel's scroll offset at draw time (the caret line is
    /// kept visible while a multi-line script overflows the pane).
    pub input_scroll: u16,
    /// Whether the user guide view covered the frame.
    pub guide: bool,
    /// The key-help overlay covers the screen (ADR-0039): wheel scrolls
    /// it and clicks are inert.
    pub key_help: bool,
    /// The constants browser covers the screen (ADR-0045): wheel scrolls
    /// the selection.
    pub constants: bool,
}

/// The TUI keypad (ADR-0016): a condensed grid of the most-used
/// tokens — the full set lives on the web keypad; the terminal stays
/// compact. (display, insert-at-end). The digits bank is a mirror of the
/// web keypad's `123` tab — same keys, same 5×5 arrangement — and its
/// three action keys (C, ⌫, =) are spelled by an empty insert string;
/// [`App::keypad_insert`] performs them (the "=" submit runs through
/// the entry's submit path in the caller, which owns the store).
/// The keypad's banks (ADR-0016): every function, constant, and command
/// the language supports, mirroring the web keypad's tabs in the same
/// order (digits first, like the web). Labels are the language tokens
/// themselves (ADR-0007 — the language is never localized). Rows may be
/// ragged; the widest row fixes the grid width.
const BANKS: &[(&str, &[&[(&str, &str)]])] = &[
    (
        "123",
        &[
            &[("C", ""), ("⌫", ""), ("(", "("), (")", ")"), ("÷", "/")],
            &[("7", "7"), ("8", "8"), ("9", "9"), ("×", "*"), ("−", "-")],
            &[("4", "4"), ("5", "5"), ("6", "6"), ("+", "+"), ("^", "^")],
            &[("1", "1"), ("2", "2"), ("3", "3"), (";", ";"), (",", ",")],
            // The newline key (ADR-0016 amendment): ans lives on the
            // var bank, and a real newline in the entry is how
            // multi-line scripts are composed at the keypad.
            &[("0", "0"), (".", "."), ("\u{23CE}", "\n"), ("=", "")],
        ],
    ),
    (
        "trig",
        &[
            &[
                ("sin", "sin("),
                ("cos", "cos("),
                ("tan", "tan("),
                ("asin", "asin("),
                ("acos", "acos("),
            ],
            &[
                ("atan", "atan("),
                ("sinh", "sinh("),
                ("cosh", "cosh("),
                ("tanh", "tanh("),
                ("asinh", "asinh("),
            ],
            &[
                ("acosh", "acosh("),
                ("atanh", "atanh("),
                ("deg", "deg("),
                ("rad", "rad("),
                ("atan2", "atan2("),
            ],
        ],
    ),
    (
        "fn",
        &[
            &[
                ("ln", "ln("),
                ("log", "log("),
                ("log2", "log2("),
                ("logb", "logb("),
                ("exp", "exp("),
            ],
            &[
                ("sqrt", "sqrt("),
                ("cbrt", "cbrt("),
                ("root", "root("),
                ("hypot", "hypot("),
                ("abs", "abs("),
            ],
            &[
                ("floor", "floor("),
                ("ceil", "ceil("),
                ("round", "round("),
                ("trunc", "trunc("),
                ("sign", "sign("),
            ],
            &[("min", "min("), ("max", "max(")],
        ],
    ),
    (
        "num",
        &[
            &[
                ("gcd", "gcd("),
                ("lcm", "lcm("),
                ("mod", "mod("),
                ("fact", "fact("),
            ],
            &[
                ("ncr", "ncr("),
                ("npr", "npr("),
                ("sum", "sum("),
                ("product", "product("),
            ],
            &[
                ("mean", "mean("),
                ("median", "median("),
                ("variance", "variance("),
                ("stdev", "stdev("),
            ],
            // The percent key (ADR-0042): the transparent /100 suffix.
            // It lives here, not on the digits bank: that bank is
            // exactly full at five rows, so any addition would push it
            // past the 80x24 frame (ADR-0042 amendment).
            &[("%", "%")],
        ],
    ),
    (
        "0x",
        &[
            &[("frac", "frac("), ("dec", "dec("), ("big", "big(")],
            &[
                ("bin", "bin("),
                ("oct", "oct("),
                ("hex", "hex("),
                ("!", "!"),
            ],
        ],
    ),
    (
        "astro",
        &[
            &[
                ("jd", "jd("),
                ("now", "now"),
                ("lst", "lst("),
                ("kepler", "kepler("),
                ("ra", "ra("),
            ],
            &[
                ("decl", "decl("),
                ("dist", "dist("),
                ("alt", "alt("),
                ("mag", "mag("),
                ("rise", "rise("),
            ],
            &[
                ("set", "set("),
                ("illum", "illum("),
                ("diam", "diam("),
                ("delta_t", "delta_t("),
                ("airmass", "airmass("),
            ],
            &[
                ("dawes", "dawes("),
                ("dist_mod", "dist_mod("),
                ("mag2jy", "mag2jy("),
                ("hms2deg", "hms2deg("),
                ("solar3d", "solar3d "),
            ],
            &[
                ("az", "az("),
                ("transit", "transit("),
                ("phase", "phase("),
                ("mjd", "mjd("),
                ("deg2hms", "deg2hms("),
            ],
        ],
    ),
    (
        "var",
        &[
            &[
                ("pi", "pi"),
                ("e", "e"),
                ("tau", "tau"),
                ("phi", "phi"),
                ("x", "x"),
            ],
            &[
                ("t", "t"),
                ("ans", "ans"),
                ("graph", "graph "),
                ("graph3d", "graph3d "),
                ("table", "table "),
            ],
        ],
    ),
];

/// The keypad banks, for tests and callers that need the grid.
pub fn banks() -> &'static [(
    &'static str,
    &'static [&'static [(&'static str, &'static str)]],
)] {
    BANKS
}

/// The FTL key of a keypad token's meaning (ADR-0039): the same
/// `key-hint-*` messages the web keypad's aria-labels and hint bar
/// speak. `None` for the self-evident digit keys, whose labels speak
/// for themselves. Shared by the key-help overlay and its tests, which
/// assert every non-digit token in every bank maps to something.
pub fn keypad_hint_key(disp: &str) -> Option<&'static str> {
    let named = |key: &'static str| Some(key);
    match disp {
        // digits bank: glyphs and actions
        "C" => named("key-hint-clear"),
        "%" => named("key-hint-percent"),
        "\u{232b}" => named("key-hint-backspace"),
        "(" => named("key-hint-lpar"),
        ")" => named("key-hint-rpar"),
        "\u{f7}" => named("key-hint-div"),
        "\u{d7}" => named("key-hint-mul"),
        "\u{2212}" => named("key-hint-sub"),
        "+" => named("key-hint-add"),
        "^" => named("key-hint-pow"),
        ";" => named("key-hint-semi"),
        "," => named("key-hint-comma"),
        "\u{23ce}" => named("key-hint-newline"),
        "=" => named("key-hint-equals"),
        // trig
        "sin" => named("key-hint-sin"),
        "cos" => named("key-hint-cos"),
        "tan" => named("key-hint-tan"),
        "asin" => named("key-hint-asin"),
        "acos" => named("key-hint-acos"),
        "atan" => named("key-hint-atan"),
        "sinh" => named("key-hint-sinh"),
        "cosh" => named("key-hint-cosh"),
        "tanh" => named("key-hint-tanh"),
        "asinh" => named("key-hint-asinh"),
        "acosh" => named("key-hint-acosh"),
        "atanh" => named("key-hint-atanh"),
        "deg" => named("key-hint-deg"),
        "rad" => named("key-hint-rad"),
        "atan2" => named("key-hint-atan2"),
        // functions
        "ln" => named("key-hint-ln"),
        "log" => named("key-hint-log"),
        "log2" => named("key-hint-log2"),
        "logb" => named("key-hint-logb"),
        "exp" => named("key-hint-exp"),
        "sqrt" => named("key-hint-sqrt"),
        "cbrt" => named("key-hint-cbrt"),
        "root" => named("key-hint-root"),
        "hypot" => named("key-hint-hypot"),
        "abs" => named("key-hint-abs"),
        "floor" => named("key-hint-floor"),
        "ceil" => named("key-hint-ceil"),
        "round" => named("key-hint-round"),
        "trunc" => named("key-hint-trunc"),
        "sign" => named("key-hint-sign"),
        "min" => named("key-hint-min"),
        "max" => named("key-hint-max"),
        // number theory and statistics
        "gcd" => named("key-hint-gcd"),
        "lcm" => named("key-hint-lcm"),
        "mod" => named("key-hint-mod"),
        "fact" => named("key-hint-fact"),
        "ncr" => named("key-hint-ncr"),
        "npr" => named("key-hint-npr"),
        "sum" => named("key-hint-sum"),
        "product" => named("key-hint-product"),
        "mean" => named("key-hint-mean"),
        "median" => named("key-hint-median"),
        "variance" => named("key-hint-variance"),
        "stdev" => named("key-hint-stdev"),
        // conversions and bases ("!" shares fact's meaning)
        "frac" => named("key-hint-frac"),
        "dec" => named("key-hint-dec"),
        "big" => named("key-hint-big"),
        "bin" => named("key-hint-bin"),
        "oct" => named("key-hint-oct"),
        "hex" => named("key-hint-hex"),
        "!" => named("key-hint-fact"),
        // var bank: constants and commands
        "pi" => named("key-hint-pi"),
        "e" => named("key-hint-e"),
        "tau" => named("key-hint-tau"),
        "phi" => named("key-hint-phi"),
        "x" => named("key-hint-x"),
        "t" => named("key-hint-t"),
        "ans" => named("key-hint-ans"),
        "graph" => named("key-hint-graph"),
        "graph3d" => named("key-hint-graph3d"),
        "solar3d" => named("key-hint-solar3d"),
        "table" => named("key-hint-table"),
        // astro
        "jd" => named("key-hint-jd"),
        "mjd" => named("key-hint-mjd"),
        "now" => named("key-hint-now"),
        "delta_t" => named("key-hint-delta_t"),
        "lst" => named("key-hint-lst"),
        "hms2deg" => named("key-hint-hms2deg"),
        "dms2deg" => named("key-hint-dms2deg"),
        "deg2hms" => named("key-hint-deg2hms"),
        "deg2dms" => named("key-hint-deg2dms"),
        "kepler" => named("key-hint-kepler"),
        "ra" => named("key-hint-ra"),
        "decl" => named("key-hint-decl"),
        "dist" => named("key-hint-dist"),
        "alt" => named("key-hint-alt"),
        "az" => named("key-hint-az"),
        "rise" => named("key-hint-rise"),
        "set" => named("key-hint-set"),
        "transit" => named("key-hint-transit"),
        "mag" => named("key-hint-mag"),
        "phase" => named("key-hint-phase"),
        "illum" => named("key-hint-illum"),
        "diam" => named("key-hint-diam"),
        "airmass" => named("key-hint-airmass"),
        "dawes" => named("key-hint-dawes"),
        "dist_mod" => named("key-hint-dist_mod"),
        "mag2jy" => named("key-hint-mag2jy"),
        "jy2mag" => named("key-hint-jy2mag"),
        "march_equinox" => named("key-hint-march_equinox"),
        "june_solstice" => named("key-hint-june_solstice"),
        "september_equinox" => named("key-hint-september_equinox"),
        "december_solstice" => named("key-hint-december_solstice"),
        "au" => named("key-hint-au"),
        "pc" => named("key-hint-pc"),
        "ly" => named("key-hint-ly"),
        "c" => named("key-hint-c"),
        "g" => named("key-hint-g"),
        "h" => named("key-hint-h"),
        "h_bar" => named("key-hint-h_bar"),
        "k_b" => named("key-hint-k_b"),
        "sigma_sb" => named("key-hint-sigma_sb"),
        "m_sun" => named("key-hint-m_sun"),
        "r_sun" => named("key-hint-r_sun"),
        "l_sun" => named("key-hint-l_sun"),
        "m_earth" => named("key-hint-m_earth"),
        "r_earth" => named("key-hint-r_earth"),
        _ => None,
    }
}

impl App {
    /// Replace the whole calculator state from a store reload
    /// (ADR-0010 amendment): history, functions, constants, scripts, and
    /// the bindings snapshot. The in-flight entry text and the cursor
    /// survive; a reload never writes, so it cannot loop with the store
    /// watcher.
    pub fn set_session(&mut self, session: Session) {
        let mut session = session;
        session.set_display(self.display);
        self.session = session;
    }

    /// The result display preferences (ADR-0043); a change applies to
    /// every later submit.
    pub fn set_display_prefs(&mut self, prefs: epher_core::DisplayPrefs) {
        self.display = prefs;
        self.session.set_display(prefs);
    }

    pub fn display_prefs(&self) -> epher_core::DisplayPrefs {
        self.display
    }

    /// Toggle exact fraction display (ADR-0043).
    pub fn toggle_exact(&mut self) {
        let mut prefs = self.display;
        prefs.exact_fractions = !prefs.exact_fractions;
        self.set_display_prefs(prefs);
    }

    /// Cycle the result notation Auto -> Scientific -> Engineering
    /// (ADR-0043).
    pub fn cycle_notation(&mut self) {
        use epher_core::Notation;
        let mut prefs = self.display;
        prefs.notation = match prefs.notation {
            Notation::Auto => Notation::Scientific,
            Notation::Scientific => Notation::Engineering,
            Notation::Engineering => Notation::Auto,
        };
        self.set_display_prefs(prefs);
    }

    /// Toggle thousands separators on results (ADR-0043).
    pub fn toggle_separators(&mut self) {
        let mut prefs = self.display;
        prefs.separators = !prefs.separators;
        self.set_display_prefs(prefs);
    }

    pub fn with_session(session: Session) -> Self {
        Self {
            input: String::new(),
            cursor: 0,
            result: String::new(),
            session,
            graph: Vec::new(),
            pois: Vec::new(),
            surface: Vec::new(),
            curve3d: Vec::new(),
            data: None,
            solar: None,
            solar_source: None,
            view: View3D::default(),
            view_h: 0.0,
            view_v: 0.0,
            view_z: 0.0,
            view2d: None,
            play: None,
            keypad: false,
            keypad_docked: false,
            key_help: None,
            constants: None,
            constants_rows: Vec::new(),
            kp_row: 0,
            kp_col: 0,
            kp_bank: 0,
            theme: Theme::default(),
            poi_list: true,
            display: epher_core::DisplayPrefs::default(),
            menu: None,
            prompt: None,
            hist_focus: false,
            hist_sel: 0,
            hist_rows: Vec::new(),
            guide: None,
            guide_chapters: Vec::new(),
            guide_md: None,
            guide_searching: false,
            guide_query: String::new(),
            guide_hit_rows: Vec::new(),
            areas: Areas::default(),
        }
    }

    // --- keypad mode (ADR-0016) ---

    pub fn keypad_focused(&self) -> bool {
        self.keypad
    }

    pub fn keypad_row(&self) -> usize {
        self.kp_row
    }

    pub fn keypad_col(&self) -> usize {
        self.kp_col
    }

    pub fn keypad_open(&mut self) {
        self.keypad = true;
        self.kp_bank = 0;
        self.kp_row = 0;
        self.kp_col = 0;
        self.menu = None;
    }

    /// The key-help overlay (ADR-0039): the current bank's keys with
    /// their meanings, scrollable, closed with q or Esc. The value is
    /// the scroll offset, mirroring the guide's pager.
    pub fn key_help_open(&mut self) {
        self.key_help = Some(0);
        self.menu = None;
    }

    pub fn key_help_close(&mut self) {
        self.key_help = None;
    }

    pub fn key_help_active(&self) -> bool {
        self.key_help.is_some()
    }

    pub fn key_help_scroll(&mut self, delta: isize) {
        if let Some(offset) = &mut self.key_help {
            *offset = offset.saturating_add_signed(delta);
        }
    }

    pub fn key_help_offset(&self) -> Option<usize> {
        self.key_help
    }

    // --- the constants browser (ADR-0045) ---

    /// Open the constants browser: Help -> Constants, a grouped list of
    /// the builtin constants; Enter inserts the selected name into the
    /// input line, Escape closes. The rows (name, value, hint) are
    /// snapshot here in the guide's group order.
    pub fn constants_open(&mut self, localizer: &Localizer) {
        self.constants_rows = Vec::new();
        for group in [
            epher_core::ConstGroup::Math,
            epher_core::ConstGroup::Astronomy,
            epher_core::ConstGroup::Physics,
            epher_core::ConstGroup::Chemistry,
        ] {
            for &(name, g) in epher_core::builtin_constant_groups() {
                if g != group {
                    continue;
                }
                let hint_key = format!("key-hint-{name}");
                let hint = localizer.lookup(&hint_key);
                let hint = if hint == hint_key {
                    String::new()
                } else {
                    hint
                };
                self.constants_rows.push((
                    name.to_string(),
                    epher_core::builtin_constant_value(name),
                    hint,
                ));
            }
        }
        self.constants = Some(0);
        self.menu = None;
        self.keypad = false;
    }

    pub fn constants_close(&mut self) {
        self.constants = None;
    }

    pub fn constants_active(&self) -> bool {
        self.constants.is_some()
    }

    /// The browser's selected row index (tests and the renderer).
    pub fn constants_selection(&self) -> Option<usize> {
        self.constants
    }

    /// The name of the constant row at `index` (tests).
    pub fn constants_row_name(&self, index: usize) -> Option<&str> {
        self.constants_rows.get(index).map(|(n, _, _)| n.as_str())
    }

    /// How many constant rows the browser lists (tests).
    pub fn constants_rows_len(&self) -> usize {
        self.constants_rows.len()
    }

    /// Move the selection over the constant rows (headings are not
    /// selectable); clamps at both ends.
    pub fn constants_select(&mut self, delta: isize) {
        let Some(sel) = &mut self.constants else {
            return;
        };
        let count = self.constants_rows.len();
        if count == 0 {
            *sel = 0;
            return;
        }
        *sel = if delta < 0 {
            sel.saturating_sub(delta.unsigned_abs())
        } else {
            (*sel + delta as usize).min(count - 1)
        };
    }

    /// Insert the selected constant's name at the input cursor and
    /// close the browser (SpeedCrunch's Ctrl+Space flow, ADR-0045).
    pub fn constants_insert(&mut self) {
        let Some(sel) = self.constants else {
            return;
        };
        let name = self.constants_rows.get(sel).map(|(n, _, _)| n.clone());
        if let Some(name) = name {
            self.insert_text(&name);
        }
        self.constants = None;
    }

    pub fn keypad_close(&mut self) {
        self.keypad = false;
    }

    /// The keypad's docked state (ADR-0060): false while the keypad
    /// shows.
    pub fn keypad_shown(&self) -> bool {
        !self.keypad_docked
    }

    /// Toggle the docked state (ADR-0060): the grab drag, the strip
    /// click, and Ctrl+K all land here. Hiding also drops keypad focus
    /// — a focused grid the user cannot see is a focus trap.
    pub fn keypad_toggle(&mut self) {
        self.keypad_docked = !self.keypad_docked;
        if self.keypad_docked {
            self.keypad = false;
        }
    }

    /// The highlighted bank's label.
    pub fn keypad_bank(&self) -> &'static str {
        BANKS[self.kp_bank].0
    }

    /// The highlighted bank's index.
    pub fn keypad_bank_index(&self) -> usize {
        self.kp_bank
    }

    /// Switch to the next (or previous) bank, wrapping.
    pub fn keypad_cycle(&mut self, dir: isize) {
        let banks = BANKS.len() as isize;
        self.kp_bank = (self.kp_bank as isize + dir).rem_euclid(banks) as usize;
        self.kp_row = 0;
        self.kp_col = 0;
    }

    /// Move the highlight, wrapping around the grid edges. Rows may be
    /// ragged, so vertical motion clamps the column to the new row.
    pub fn keypad_move(&mut self, dr: isize, dc: isize) {
        let rows = BANKS[self.kp_bank].1.len() as isize;
        let cols = BANKS[self.kp_bank]
            .1
            .iter()
            .map(|r| r.len())
            .max()
            .unwrap_or(1) as isize;
        self.kp_row = (self.kp_row as isize + dr).rem_euclid(rows) as usize;
        self.kp_col = (self.kp_col as isize + dc).rem_euclid(cols) as usize;
        let len = BANKS[self.kp_bank].1[self.kp_row].len();
        self.kp_col = self.kp_col.min(len.saturating_sub(1));
    }

    /// Apply the highlighted key: tokens insert at the cursor (the
    /// terminal cursor sits wherever the caret is); the digits bank's
    /// action keys clear ("C") and backspace ("⌫") at the caret. The
    /// "=" key only marks the highlight — [`Self::keypad_is_submit`] tells
    /// the caller to run the entry's submit path instead.
    pub fn keypad_insert(&mut self) {
        let row = &BANKS[self.kp_bank].1[self.kp_row];
        let (disp, token) = row[self.kp_col.min(row.len() - 1)];
        match (disp, token) {
            ("C", "") => self.clear_input(),
            ("⌫", "") => self.pop_char(),
            _ => {
                for c in token.chars() {
                    self.push_char(c);
                }
            }
        }
    }

    /// Whether the highlighted key is the digits bank's "=": the caller
    /// runs the entry's submit path (it owns the store and localizer).
    pub fn keypad_is_submit(&self) -> bool {
        let row = &BANKS[self.kp_bank].1[self.kp_row];
        row[self.kp_col.min(row.len() - 1)].0 == "="
    }

    /// Jump to an absolute bank (mouse click on a bank label, ADR-0034),
    /// resetting the highlight like [`Self::keypad_cycle`] does.
    pub fn keypad_select_bank(&mut self, bank: usize) {
        self.kp_bank = bank.min(BANKS.len() - 1);
        self.kp_row = 0;
        self.kp_col = 0;
    }

    /// Move the highlight to an absolute cell (mouse click, ADR-0034),
    /// clamping to the clicked bank's grid instead of wrapping.
    pub fn keypad_set(&mut self, row: usize, col: usize) {
        let bank = &BANKS[self.kp_bank].1;
        self.kp_row = row.min(bank.len().saturating_sub(1));
        let len = bank[self.kp_row].len();
        self.kp_col = col.min(len.saturating_sub(1));
    }

    // --- theme, menu bar, and file prompts (ADR-0017) ---

    pub fn theme(&self) -> Theme {
        self.theme
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Settings → Graph: the points-of-interest list toggle (ADR-0019).
    pub fn poi_list(&self) -> bool {
        self.poi_list
    }

    pub fn toggle_pois(&mut self) {
        self.poi_list = !self.poi_list;
    }

    pub fn set_pois(&mut self, pois: bool) {
        self.poi_list = pois;
    }

    /// The menu bar: File, Edit, Graph, Settings, Help.
    // Help sits above Settings (the ADR-0038 amendment), matching the
    // app's menu rail.
    pub const MENUS: [&'static str; 5] = ["file", "edit", "graph", "help", "settings"];

    /// How many items a menu has. The Settings menu grows three
    /// fine-control rows while 3D surfaces are displayed (ADR-0031).
    pub fn menu_len(&self, menu: usize) -> usize {
        match menu {
            0 => 5, // File: open history, open script, save history, save script, quit
            1 => 3, // Edit: cut, copy, paste
            2 => 2, // Graph: clear graph, copy points of interest
            3 => 3, // Help: the in-app guide, the keypad key help, constants
            4 => {
                if self.surface.is_empty() && self.solar.is_none() {
                    15 // POI, 3 themes, 8 languages, exact, notation, separators
                } else {
                    18 // …plus horizontal rotation, vertical rotation, zoom
                }
            }
            _ => 0,
        }
    }

    pub fn menu_active(&self) -> Option<(usize, usize)> {
        self.menu
    }

    pub fn menu_open(&mut self, menu: usize) {
        self.menu = Some((menu.min(4), 0));
        self.keypad = false;
    }

    pub fn menu_close(&mut self) {
        self.menu = None;
    }

    /// Move the highlight: vertical motion wraps within the open menu,
    /// horizontal motion switches menus (wrapping at the ends).
    pub fn menu_move(&mut self, dh: isize, dv: isize) {
        let Some((menu, item)) = self.menu else {
            return;
        };
        if dh != 0 {
            let menus = Self::MENUS.len() as isize;
            let next = (menu as isize + dh).rem_euclid(menus) as usize;
            self.menu = Some((next, 0));
            return;
        }
        let len = self.menu_len(menu) as isize;
        let next = (item as isize + dv).rem_euclid(len) as usize;
        self.menu = Some((menu, next));
    }

    /// Activate the highlighted item, returning the action for the event
    /// loop to execute (the loop owns the store and localizer).
    pub fn menu_activate(&mut self) -> Option<MenuAction> {
        let (menu, item) = self.menu?;
        let action = match menu {
            0 => match item {
                0 => MenuAction::OpenHistory,
                1 => MenuAction::OpenScript,
                2 => MenuAction::SaveHistory,
                3 => MenuAction::SaveScript,
                _ => MenuAction::Quit,
            },
            1 => match item {
                0 => MenuAction::Cut,
                1 => MenuAction::Copy,
                _ => MenuAction::Paste,
            },
            2 => match item {
                0 => MenuAction::ClearGraph,
                // The POI list leaves the pane the same way the web
                // heading's copy button does (ADR-0038 amendment).
                _ => MenuAction::CopyPois,
            },
            3 => match item {
                0 => MenuAction::OpenGuide,
                1 => MenuAction::OpenKeyHelp,
                _ => MenuAction::BrowseConstants,
            },
            4 => match item {
                0 => MenuAction::TogglePois,
                1 => MenuAction::SetTheme("light"),
                2 => MenuAction::SetTheme("dark"),
                3 => MenuAction::SetTheme("night"),
                4 => MenuAction::SetLanguage("en"),
                5 => MenuAction::SetLanguage("zh-CN"),
                6 => MenuAction::SetLanguage("hi"),
                7 => MenuAction::SetLanguage("es"),
                8 => MenuAction::SetLanguage("fr"),
                9 => MenuAction::SetLanguage("ar"),
                10 => MenuAction::SetLanguage("de"),
                11 => MenuAction::SetLanguage("pt"),
                // Fine-control rows (ADR-0031): adjusted with Left/Right
                // while highlighted; Enter leaves them be and keeps the
                // menu open.
                12 => MenuAction::ToggleExact,
                13 => MenuAction::CycleNotation,
                14 => MenuAction::ToggleSeparators,
                15..=17 => return None,
                _ => MenuAction::SetLanguage("pt"),
            },
            _ => MenuAction::OpenGuide,
        };
        self.menu = None;
        Some(action)
    }

    /// The in-app user guide view: open/closed and the scroll offset.
    pub fn guide_active(&self) -> bool {
        self.guide.is_some()
    }

    pub fn guide_open(&mut self) {
        self.guide = Some(0);
        self.menu = None;
        self.keypad = false;
    }

    /// Load the guide text from the installed files (ADR-0053), once
    /// per locale: a language change while the session lives reloads.
    pub fn guide_load(&mut self, locale: &str) {
        if self
            .guide_md
            .as_ref()
            .map(|(loaded, _)| loaded != locale)
            .unwrap_or(true)
        {
            self.guide_md = Some((locale.to_string(), epher_guide::load(locale)));
        }
    }

    /// The loaded guide text (ADR-0053): (locale, markdown or the miss).
    pub fn guide_text(&self) -> Option<&(String, Result<String, epher_guide::GuideUnavailable>)> {
        self.guide_md.as_ref()
    }

    pub fn guide_close(&mut self) {
        self.guide = None;
    }

    pub fn guide_scroll(&mut self, delta: isize) {
        if let Some(offset) = &mut self.guide {
            *offset = offset.saturating_add_signed(delta);
        }
    }

    pub fn guide_scroll_to(&mut self, offset: usize) {
        self.guide = Some(offset);
    }

    /// Jump the pager to a table-of-contents chapter (ADR-0018
    /// amendment): the wrapped row its heading starts at in the current
    /// frame.
    pub fn guide_jump(&mut self, chapter: usize) {
        if let Some((_, row)) = self.guide_chapters.get(chapter) {
            self.guide = Some(*row);
        }
    }

    /// Start a guide search (the ADR-0038 amendment's `/`): the pager's
    /// spelling of the web overlay's search box.
    pub fn guide_search_start(&mut self) {
        self.guide_searching = true;
    }

    pub fn guide_search_push(&mut self, c: char) {
        if !self.guide_searching {
            return;
        }
        self.guide_query.push(c);
    }

    pub fn guide_search_pop(&mut self) {
        self.guide_query.pop();
    }

    /// End the search: the query and its hits go, and the next Esc
    /// closes the guide again.
    pub fn guide_search_clear(&mut self) {
        self.guide_searching = false;
        self.guide_query.clear();
        self.guide_hit_rows.clear();
    }

    pub fn guide_searching(&self) -> bool {
        self.guide_searching
    }

    pub fn guide_query(&self) -> &str {
        &self.guide_query
    }

    pub fn guide_hit_rows(&self) -> &[usize] {
        &self.guide_hit_rows
    }

    /// Jump to the next hit after the current offset, wrapping to the
    /// first when the end is passed (the Enter spelling of the web's
    /// result click).
    pub fn guide_jump_next_hit(&mut self) {
        if self.guide_hit_rows.is_empty() {
            return;
        }
        let offset = self.guide.unwrap_or(0);
        let next = self
            .guide_hit_rows
            .iter()
            .copied()
            .find(|&r| r > offset)
            .unwrap_or(self.guide_hit_rows[0]);
        self.guide = Some(next);
    }

    /// The current scroll offset (clamped to the content at draw time).
    pub fn guide_offset(&self) -> Option<usize> {
        self.guide
    }

    /// Empty the graph pane: curves, points of interest, 3D surfaces —
    /// the menu spelling of `graph clear` + `graph3d clear` (ADR-0018).
    pub fn clear_graph(&mut self) {
        self.graph.clear();
        self.pois.clear();
        self.surface.clear();
        self.data = None;
        self.solar = None;
        self.solar_source = None;
        self.play = None;
        self.view2d = None;
        self.reset_view_offsets();
    }

    // --- file prompts ---

    pub fn prompt_active(&self) -> Option<(PromptKind, &str)> {
        self.prompt.as_ref().map(|(k, buf)| (*k, buf.as_str()))
    }

    /// Start a file prompt. Save prompts pre-fill the default file name
    /// (`epher-history.ehs` / `epher-script.esr`, ADR-0027) so Enter
    /// saves to the current directory; the buffer stays fully editable —
    /// any extension the user types wins. Open prompts start empty.
    pub fn prompt_start(&mut self, kind: PromptKind) {
        let default = match kind {
            PromptKind::SaveHistory => "epher-history.ehs".to_string(),
            PromptKind::SaveScript => "epher-script.esr".to_string(),
            PromptKind::OpenHistory | PromptKind::OpenScript => String::new(),
        };
        self.prompt = Some((kind, default));
        self.keypad = false;
        self.menu = None;
        self.hist_focus = false;
    }

    // --- history focus (ADR-0027) ---

    pub fn history_focused(&self) -> bool {
        self.hist_focus
    }

    /// The selected display row (0 = newest line on top).
    pub fn history_sel(&self) -> usize {
        self.hist_sel
    }

    pub fn history_open(&mut self) {
        self.hist_focus = true;
        self.hist_sel = 0;
        self.keypad = false;
        self.menu = None;
    }

    pub fn history_close(&mut self) {
        self.hist_focus = false;
    }

    /// Move the selection up (+1 = older) or down, wrapping.
    pub fn history_move(&mut self, dir: isize) {
        let len = self.session.history().len() as isize;
        if len == 0 {
            return;
        }
        self.hist_sel = (self.hist_sel as isize + dir).rem_euclid(len) as usize;
    }

    /// Load the selected line into the input (replacing whatever is
    /// there — it is not run) and leave history focus. `None` when the
    /// history is empty.
    pub fn history_pick(&mut self) -> Option<String> {
        self.history_pick_display(self.hist_sel)
    }

    /// Load the entry shown at `display_idx` (0 = newest on top) into the
    /// input — the mouse spelling of the web's clickable history
    /// (ADR-0034).
    pub fn history_pick_display(&mut self, display_idx: usize) -> Option<String> {
        let len = self.session.history().len();
        if len == 0 || display_idx >= len {
            self.hist_focus = false;
            return None;
        }
        let sel = display_idx;
        let entry = self.session.history()[len - 1 - sel].clone();
        // ADR-0031: the pick loads the expression — the recorded answer
        // suffix (`  = …`, `  error: …`, `  warning: …`) stays out of the
        // input so the user can edit and re-run it. Multi-line script
        // entries (ADR-0027 amendment) come back as one `; `-joined
        // line: the one-row input cannot hold newlines, and `;` is the
        // same separator, so the script re-runs exactly as recorded.
        let line = epher_core::history_expression(&entry).to_string();
        self.hist_focus = false;
        Some(line.replace('\n', "; "))
    }

    /// Open a menu with the highlight on a given item (mouse click on a
    /// popup row, ADR-0034).
    pub fn menu_select(&mut self, menu: usize, item: usize) {
        self.menu = Some((menu.min(4), item));
    }

    pub fn prompt_cancel(&mut self) {
        self.prompt = None;
    }

    pub fn prompt_push(&mut self, c: char) {
        if let Some((_, buf)) = &mut self.prompt {
            buf.push(c);
        }
    }

    pub fn prompt_pop(&mut self) {
        if let Some((_, buf)) = &mut self.prompt {
            buf.pop();
        }
    }

    /// Confirm the prompt: run the file operation and leave the prompt
    /// open on failure (so the path can be fixed) or closed on success.
    pub fn prompt_submit(&mut self, localizer: &Localizer) -> Option<PromptKind> {
        let (kind, path) = self.prompt.take()?;
        let outcome = match kind {
            PromptKind::OpenHistory => std::fs::read_to_string(&path)
                .map(|text| {
                    // Replace the history section with the file's lines —
                    // nothing executes, the lines display exactly as saved
                    // (ADR-0025). A `\n` escape becomes the entry's
                    // newline, so multi-line script entries (ADR-0027
                    // amendment) survive the one-line-per-entry format.
                    self.session.clear_history();
                    let mut loaded = 0;
                    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
                        self.session.record(&line.replace("\\n", "\n"));
                        loaded += 1;
                    }
                    let count = loaded.to_string();
                    self.result = localizer.lookup_args("history-loaded", &[("count", &count)]);
                })
                .map_err(|_| ()),
            PromptKind::OpenScript => std::fs::read_to_string(&path)
                .map(|text| {
                    self.input = text;
                    self.result = String::new();
                })
                .map_err(|_| ()),
            PromptKind::SaveHistory => std::fs::write(
                &path,
                self.history()
                    .iter()
                    .map(|h| h.replace('\n', "\\n"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
            .map_err(|_| ()),
            PromptKind::SaveScript => std::fs::write(&path, &self.input).map_err(|_| ()),
        };
        if outcome.is_ok() {
            None
        } else {
            Some(kind)
        }
    }

    pub fn set_result(&mut self, result: &str) {
        self.result = result.to_string();
    }

    /// Re-open a prompt with its previously typed path (after a failed
    /// operation, so the path can be corrected in place).
    pub fn prompt_restore(&mut self, kind: PromptKind, path: &str) {
        self.prompt = Some((kind, path.to_string()));
    }

    pub fn set_input(&mut self, input: &str) {
        self.input = input.to_string();
        self.cursor = self.input.len();
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn result(&self) -> &str {
        &self.result
    }

    pub fn history(&self) -> &[String] {
        self.session.history()
    }

    /// The session's variable bindings (user assignments plus `ans`), for
    /// the shared-store snapshot saved alongside the history (ADR-0010
    /// amendment).
    pub fn bindings(&self) -> &epher_core::ValueBindings {
        self.session.bindings()
    }

    /// The shared session (constants, history) — public so tests can read
    /// animation state.
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// The plotted curves of the current graph, if any (ADR-0014: the TUI
    /// overlays curves the way the web app does).
    pub fn graph(&self) -> &[SampledCurve] {
        &self.graph
    }

    // --- the 2D viewport (ADR-0034) ---

    /// The stored 2D viewport override, if any.
    pub fn graph2d_view(&self) -> Option<(f64, f64, f64, f64)> {
        self.view2d
    }

    /// The 2D ranges the plot uses: the stored override, else the
    /// auto-fit around the current samples.
    pub fn graph2d_effective(&self) -> Option<(f64, f64, f64, f64)> {
        self.view2d.or_else(|| curve_ranges(&self.graph))
    }

    /// Pan the 2D viewport by a mouse-drag delta in cells (ADR-0034):
    /// the plot follows the pointer — dragging right moves the window
    /// left through the data, dragging down moves it up.
    pub fn graph2d_pan(&mut self, dx_cells: f64, dy_cells: f64, width: usize, height: usize) {
        let Some((x_min, x_max, y_min, y_max)) = self.graph2d_effective() else {
            return;
        };
        let x_span = x_max - x_min;
        let y_span = y_max - y_min;
        let dx_data = -dx_cells / (width.max(1) as f64) * x_span;
        let dy_data = dy_cells / (height.max(1) as f64) * y_span;
        self.view2d = Some((
            x_min + dx_data,
            x_max + dx_data,
            y_min + dy_data,
            y_max + dy_data,
        ));
    }

    /// Zoom the 2D viewport by a factor around its center: `factor < 1`
    /// zooms in (narrower span), `> 1` out.
    pub fn graph2d_zoom(&mut self, factor: f64) {
        let Some((x_min, x_max, y_min, y_max)) = self.graph2d_effective() else {
            return;
        };
        let cx = (x_min + x_max) / 2.0;
        let cy = (y_min + y_max) / 2.0;
        let hx = (x_max - x_min) / 2.0 * factor;
        let hy = (y_max - y_min) / 2.0 * factor;
        self.view2d = Some((cx - hx, cx + hx, cy - hy, cy + hy));
    }

    /// Drop the 2D viewport override — the plot re-fits its samples.
    pub fn graph2d_reset(&mut self) {
        self.view2d = None;
    }

    /// Reset the 3D camera to the default pose (mouse double-click,
    /// ADR-0034). The fine-control offsets are untouched — they belong
    /// to the sliders.
    pub fn view_reset_pose(&mut self) {
        self.view = View3D::default();
    }

    /// Set the 3D camera distance directly (mouse-wheel zoom, ADR-0034).
    pub fn view_set_camera(&mut self, camera: f64) {
        self.view = self.view.with_camera(camera);
    }

    /// The points of interest of the current graph (roots, intersections,
    /// extrema), recomputed after every graph command.
    pub fn pois(&self) -> &[InterestPoint] {
        &self.pois
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor = 0;
    }

    /// Empty the history list (Ctrl+L); definitions and constants stay.
    pub fn clear_history(&mut self) {
        self.session.clear_history();
    }

    /// The insertion point (byte offset) inside the input text.
    pub fn cursor(&self) -> usize {
        self.cursor.min(self.input.len())
    }

    /// Insert a whole token (a constant name from the browser,
    /// ADR-0045) at the cursor, moving past it — the multi-character
    /// counterpart of [`push_char`].
    pub fn insert_text(&mut self, text: &str) {
        let at = self.cursor();
        self.input.insert_str(at, text);
        self.cursor = at + text.len();
    }

    /// Insert a pasted text at the cursor as one unit (ADR-0059): the
    /// clipboard can carry a whole multi-line script, newlines and all,
    /// and the next Enter runs it exactly like the desktop and web
    /// entries run theirs. Line endings normalize to `\n` (Windows
    /// clipboards carry `\r\n`). Unlike typing, no `ans` is injected
    /// for a leading operator — pasted text is verbatim.
    pub fn paste_text(&mut self, text: &str) {
        let text = text.replace("\r\n", "\n").replace('\r', "\n");
        self.insert_text(&text);
    }

    /// Insert a character at the cursor (ADR-0035 amendment): typing,
    /// the keypad's token insert, and Shift+Enter's newline all land at
    /// the insertion point, which moves past what was inserted.
    pub fn push_char(&mut self, c: char) {
        // ADR-0042 auto-ans: an operator typed into an empty entry means
        // "continue from the previous answer" - `ans` goes in first.
        if self.input.is_empty() && matches!(c, '+' | '-' | '*' | '/' | '^' | '%' | '!') {
            self.input.push_str("ans");
            self.cursor = self.input.len();
        }
        let at = self.cursor();
        self.input.insert(at, c);
        self.cursor = at + c.len_utf8();
    }

    /// The word ending at the caret, for F1 help (ADR-0042).
    pub fn word_before_cursor(&self) -> &str {
        let at = self.cursor().min(self.input.len());
        let head = &self.input[..at];
        if !head
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            return "";
        }
        let start = head
            .char_indices()
            .rev()
            .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
            .last()
            .map(|(i, _)| i)
            .unwrap_or(at);
        &head[start..]
    }

    /// Delete the character before the cursor; a selected range is not
    /// a TUI concept, so this is just Backspace at the caret.
    pub fn pop_char(&mut self) {
        let at = self.cursor();
        if at == 0 {
            return;
        }
        let start = self.input[..at]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.input.replace_range(start..at, "");
        self.cursor = start;
    }

    /// Move the cursor one character left/right (byte-aware).
    pub fn cursor_move(&mut self, dir: isize) {
        let at = self.cursor();
        let next = if dir < 0 {
            self.input[..at]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0)
        } else {
            let next = self.input[at..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| at + i)
                .unwrap_or(self.input.len());
            next
        };
        self.cursor = next;
    }

    /// Move the cursor to the start/end of the line it is on
    /// (`dir` < 0 home, > 0 end).
    pub fn cursor_line_edge(&mut self, dir: isize) {
        let at = self.cursor();
        if dir < 0 {
            let start = self.input[..at].rfind('\n').map(|i| i + 1).unwrap_or(0);
            self.cursor = start;
        } else {
            let end = self.input[at..]
                .find('\n')
                .map(|i| at + i)
                .unwrap_or(self.input.len());
            self.cursor = end;
        }
    }

    /// Move the cursor one line up/down, keeping its column (clamped to
    /// the target line's length).
    pub fn cursor_line(&mut self, dir: isize) {
        let at = self.cursor();
        let line_start = self.input[..at].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_end = self.input[at..]
            .find('\n')
            .map(|i| at + i)
            .unwrap_or(self.input.len());
        let col = self.input[line_start..at].chars().count();
        let target = if dir < 0 {
            // line above: from line_start back one more newline
            if line_start == 0 {
                return;
            }
            let prev_start = self.input[..line_start - 1]
                .rfind('\n')
                .map(|i| i + 1)
                .unwrap_or(0);
            (prev_start, line_start - 1)
        } else {
            if line_end == self.input.len() {
                return;
            }
            let next_end = self.input[line_end + 1..]
                .find('\n')
                .map(|i| line_end + 1 + i)
                .unwrap_or(self.input.len());
            (line_end + 1, next_end)
        };
        let (start, end) = target;
        let pos = self.input[start..end]
            .char_indices()
            .nth(col)
            .map(|(i, _)| start + i)
            .unwrap_or(end);
        self.cursor = pos;
    }

    /// The number of the line the cursor sits on (0-based), for the
    /// renderer's scroll and the mouse's row mapping.
    pub fn cursor_line_index(&self) -> usize {
        self.input[..self.cursor()]
            .chars()
            .filter(|&c| c == '\n')
            .count()
    }

    /// Place the cursor at the byte offset for the given (line, column)
    /// in character columns — the mouse's spelling of a caret move
    /// (ADR-0035 amendment, TUI).
    pub fn cursor_to(&mut self, line: usize, col: usize) {
        let mut start = 0;
        for _ in 0..line {
            match self.input[start..].find('\n') {
                Some(i) => start += i + 1,
                None => {
                    self.cursor = self.input.len();
                    return;
                }
            }
        }
        let end = self.input[start..]
            .find('\n')
            .map(|i| start + i)
            .unwrap_or(self.input.len());
        let pos = self.input[start..end]
            .char_indices()
            .nth(col)
            .map(|(i, _)| start + i)
            .unwrap_or(end);
        self.cursor = pos;
    }

    /// Evaluate the current input via the shared [`Session`]. A script
    /// line's every answer shows, in order (ADR-0052): the session
    /// returns the whole transcript, newline-joined.
    pub fn submit(&mut self) {
        self.result = self.session.submit_all(&self.input);
        self.input.clear();
        self.cursor = 0;
    }

    /// Handle one submitted line the way the event loop does: shell commands
    /// dispatch through the shared kernel (epher-shell), `graph ` samples,
    /// `graph3d ` samples a surface, anything else evaluates — and history
    /// persists. A line may join several statements with `;` or newlines
    /// (the same separator, ADR-0001 — Shift+Enter composes them in the
    /// entry, ADR-0035 amendment): each statement dispatches in order,
    /// exactly as if typed one by one, but the history keeps the script
    /// the way the user entered it — one entry per line, newlines and
    /// semicolons intact, with the last answer appended when the final
    /// statement is an evaluation. Returns the new language preference
    /// when a `language` command changed it, so the caller can re-resolve
    /// its Localizer.
    pub fn submit_line(
        &mut self,
        line: &str,
        store: &DocStore<FsStore>,
        localizer: &Localizer,
    ) -> Option<String> {
        let mut language = None;
        // `;` separates statements only where the tokenizer sees it:
        // inside a string literal or a comment it is text (the shared
        // epher-shell splitter mirrors the tokenizer, ADR-0040
        // amendment). Newlines in the text split the same way a file
        // does, line by line.
        let pieces = epher_shell::split_statements(line);
        if pieces.is_empty() {
            return None;
        }
        let single = pieces.len() == 1;
        // The output of the last evaluation, for the combined history
        // entry of a multi-statement script, and every evaluation's
        // output, for the result area (ADR-0052: a script shows its
        // whole transcript in order, not only its final value).
        let mut last_eval_output: Option<String> = None;
        let mut eval_outputs: Vec<String> = Vec::new();
        let mut last_was_eval = false;
        for piece in &pieces {
            let (changed, was_eval) = self.submit_statement(piece, store, localizer, !single);
            if language.is_none() {
                language = changed;
            }
            last_was_eval = was_eval;
            if was_eval {
                last_eval_output = Some(self.result.clone());
                if !self.result.is_empty() {
                    eval_outputs.push(self.result.clone());
                }
            } else {
                last_eval_output = None;
            }
        }
        if !single {
            // The result area shows every answer the script produced, in
            // order; a command or plot finishing the script keeps its own
            // message, exactly as before.
            if last_was_eval && !eval_outputs.is_empty() {
                self.result = eval_outputs.join("\n");
            }
            // One history entry for the whole script: the line as typed,
            // with the last answer appended exactly as single statements
            // record theirs.
            let entry = match &last_eval_output {
                Some(out) if !out.is_empty() => format!("{}  {out}", line.trim()),
                _ => line.trim().to_string(),
            };
            self.session.record(&entry);
            // `save script` persists the whole script the user entered,
            // not just its last statement.
            self.session.set_last_line(line.trim());
            let _ = save_history(store, self.history());
            let _ = save_session(store, self.bindings());
        }
        language
    }

    /// Dispatch one statement (no `;`, no newline) the way submit_line used
    /// to handle a whole line. `quiet` skips the history recording (a
    /// multi-statement line records once, for the whole line, in
    /// submit_line). Returns the new language preference (when a
    /// `language` command changed it) and whether the statement was a
    /// plain evaluation.
    fn submit_statement(
        &mut self,
        piece: &str,
        store: &DocStore<FsStore>,
        localizer: &Localizer,
        quiet: bool,
    ) -> (Option<String>, bool) {
        // The keypad's command keys (ADR-0038): `clear` empties the plot
        // like the menu's Clear graph; `history` opens the history list.
        // Both previously fell through to the evaluator and errored as
        // unknown names.
        if piece == "clear" {
            self.clear_graph();
            self.result.clear();
            return (None, false);
        }
        if piece == "history" {
            self.history_open();
            return (None, false);
        }
        if let Some(source) = piece.strip_prefix("graph ") {
            // The command joins the session history like every other
            // submitted line; the plot is the output.
            if !quiet {
                self.session.record(piece);
                let _ = save_history(store, self.history());
                let _ = save_session(store, self.bindings());
            }
            if let Some(path) = source.trim().strip_prefix("save ") {
                let path = path.trim();
                if path.is_empty() {
                    self.result = localizer.lookup("graph-no-path");
                } else {
                    self.result = self.save_graph_svg(path, localizer);
                }
                return (None, false);
            }
            if source.trim() == "save" {
                self.result = localizer.lookup("graph-no-path");
                return (None, false);
            }
            let _ = self.submit_graph(source);
            return (None, false);
        }
        if let Some(source) = piece.strip_prefix("graph3d ") {
            if !quiet {
                self.session.record(piece);
                let _ = save_history(store, self.history());
                let _ = save_session(store, self.bindings());
            }
            if let Some(path) = source.trim().strip_prefix("save ") {
                let path = path.trim();
                if path.is_empty() {
                    self.result = localizer.lookup("graph-no-path");
                } else {
                    self.result = self.save_graph3d_svg(path, localizer);
                }
                return (None, false);
            }
            if source.trim() == "save" {
                self.result = localizer.lookup("graph-no-path");
                return (None, false);
            }
            let _ = self.submit_surface(source);
            return (None, false);
        }
        if let Some(source) = piece.strip_prefix("solar3d ") {
            if !quiet {
                self.session.record(piece);
                let _ = save_history(store, self.history());
                let _ = save_session(store, self.bindings());
            }
            if let Some(path) = source.trim().strip_prefix("save ") {
                let path = path.trim();
                if path.is_empty() {
                    self.result = localizer.lookup("graph-no-path");
                } else {
                    self.result = self.save_solar_svg(path, localizer);
                }
                return (None, false);
            }
            if source.trim() == "save" {
                self.result = localizer.lookup("graph-no-path");
                return (None, false);
            }
            let _ = self.submit_solar3d(source);
            return (None, false);
        }
        if let Some(cmd) = classify(piece) {
            let handled = run_command(&cmd, &mut self.session, store, localizer);
            self.result = plain(handled.message);
            // A table is a computation: the command joins the session
            // history like every other submitted line (the graph
            // precedent, ADR-0027) — picking it loads the command, and
            // re-running it regenerates the table. The other shell
            // commands persist their effect and leave no entry.
            if matches!(cmd, epher_shell::Command::Table { .. }) && !quiet {
                self.session.record(piece);
                let _ = save_history(store, self.history());
                let _ = save_session(store, self.bindings());
            }
            self.input.clear();
            // The `theme` command persists through run_command (the shell
            // kernel saved it); the App re-applies its palette right here,
            // so the TUI needs no extra plumbing (ADR-0017).
            if let Some(name) = handled.theme {
                if let Some(theme) = Theme::from_str(&name) {
                    self.theme = theme;
                }
            }
            return (handled.language, false);
        }
        if quiet {
            self.result = self.session.submit_quiet(piece);
            self.input.clear();
        } else {
            self.input = piece.to_string();
            self.submit();
            let _ = save_history(store, self.history());
            let _ = save_session(store, self.bindings());
        }
        (None, true)
    }

    /// Parse `source` as a graph command (ADR-0014 grammar: cartesian,
    /// `param`, `polar`, domain bounds, `y <`/`y >` fills) and overlay it on
    /// the current plot; `graph clear` empties the plot. Returns an error
    /// string on failure; points of interest are recomputed for the whole
    /// set.
    /// Write the current 2D plot as the same self-contained SVG the web
    /// app's copy button yields (ADR-0020) — from the same renderer, so
    /// the bytes match. The result line carries the localized outcome.
    pub fn save_graph_svg(&self, path: &str, localizer: &Localizer) -> String {
        let plots = match self.data.as_ref() {
            Some(data) => epher_shell::plots::Plots::from_data(data.clone()),
            None => epher_shell::plots::Plots::from_curves(self.graph.clone()),
        };
        let out = plots.save_svg(path, self.poi_list, self.session.env(), localizer);
        out.message
    }

    /// Write the current 3D surfaces as a self-contained SVG (ADR-0020),
    /// from the current orbit pose.
    pub fn save_graph3d_svg(&self, path: &str, localizer: &Localizer) -> String {
        let plots = epher_shell::plots::Plots::from_surfaces(self.surface.clone());
        let out = plots.save_3d_svg_with_view(path, &self.effective_view(), localizer);
        out.message
    }

    /// Write the current solar system scene as a self-contained SVG
    /// (ADR-0020), from the current orbit pose.
    pub fn save_solar_svg(&self, path: &str, localizer: &Localizer) -> String {
        match self.solar.as_ref() {
            Some(scene) => {
                let plots = epher_shell::plots::Plots::from_scene(scene.clone());
                let out = plots.save_solar_svg(path, &self.effective_view(), localizer);
                out.message
            }
            None => localizer.lookup("graph-empty"),
        }
    }

    /// Parse `source` as a `solar3d` time expression (any expression that
    /// evaluates to a Julian Date), build the scene, and show it - the
    /// pane shows one kind at a time, so curves and surfaces yield
    /// (ADR-0037).
    pub fn submit_solar3d(&mut self, source: &str) -> Result<(), String> {
        if source.trim() == "clear" {
            self.solar = None;
            self.solar_source = None;
            self.result.clear();
            return Ok(());
        }
        let jd = match epher_core::astro::eval_jd(source.trim(), self.session.env()) {
            Ok(jd) => jd,
            Err(e) => {
                self.result = format!("error: {e}");
                return Err(e.to_string());
            }
        };
        let scene = match epher_core::astro::solar_scene(jd).map_err(|e| e.to_string()) {
            Ok(scene) => scene,
            Err(e) => {
                self.result = format!("error: {e}");
                return Err(e);
            }
        };
        self.graph.clear();
        self.pois.clear();
        self.surface.clear();
        self.data = None;
        self.solar = Some(scene);
        self.solar_source = Some(source.trim().to_string());
        self.view = self.solar.as_ref().expect("just set").default_view();
        self.reset_view_offsets();
        self.result.clear();
        Ok(())
    }

    pub fn submit_graph(&mut self, source: &str) -> Result<(), String> {
        if source.trim() == "clear" {
            self.graph.clear();
            self.pois.clear();
            self.data = None;
            self.view2d = None;
            self.result.clear();
            return Ok(());
        }
        // Data plots (ADR-0044): a scatter, histogram, or boxplot owns
        // the pane like a solar scene does.
        if epher_core::graph::is_data_plot_source(source) {
            match sample_data_plot(source, self.session.env()).map_err(|e| e.to_string()) {
                Ok(plot) => {
                    self.graph.clear();
                    self.pois.clear();
                    self.surface.clear();
                    self.data = Some(plot);
                    self.view2d = None;
                    self.result.clear();
                    Ok(())
                }
                Err(e) => {
                    self.result = format!("error: {e}");
                    Err(e)
                }
            }
        } else {
            self.submit_curve(source)
        }
    }

    /// Parse `source` as a plain graph command (ADR-0014 grammar:
    /// cartesian, `param`, `polar`, domain bounds, `y <`/`y >` fills) and
    /// overlay it on the current plot.
    fn submit_curve(&mut self, source: &str) -> Result<(), String> {
        let spec = match parse_graph_source(source).map_err(|e| e.to_string()) {
            Ok(s) => s,
            Err(e) => {
                self.result = format!("error: {e}");
                return Err(e);
            }
        };
        let samples = match sample_spec(&spec, 120, self.session.env()).map_err(|e| e.to_string()) {
            Ok(samples) => samples,
            Err(e) => {
                self.result = format!("error: {e}");
                return Err(e);
            }
        };
        // The pane shows one kind at a time (ADR-0015 amendment): drawing
        // a 2D curve clears any 3D surfaces, so the two never share the
        // pane and each plot keeps its full size.
        self.surface.clear();
        self.data = None;
        self.graph.push(SampledCurve {
            source: source.to_string(),
            kind: spec.kind,
            domain: spec.domain,
            samples,
            fill: spec.fill,
        });
        self.pois = analyze(&self.graph, self.session.env());
        // A new plot re-fits the viewport: the previous pan/zoom was
        // about the curves that were there (ADR-0034).
        self.view2d = None;
        // Graphing prints nothing to the answer line (ADR-0027): the
        // command joins the history list, and the plot is the result.
        self.result.clear();
        Ok(())
    }

    /// Parse `source` as a `graph3d` command (ADR-0015 grammar:
    /// `z = f(x, y)` over an optional square domain) and overlay it on the
    /// current surface set; `graph3d clear` empties it.
    pub fn submit_surface(&mut self, source: &str) -> Result<(), String> {
        if source.trim() == "clear" {
            self.surface.clear();
            self.curve3d.clear();
            self.result.clear();
            return Ok(());
        }
        // A `param` body is a space curve (ADR-0054); anything else is
        // a surface. Both own the pane alone.
        if source.trim_start().starts_with("param ") {
            let first = self.curve3d.is_empty();
            let curve = match sample_space_curve(source, 240, self.session.env())
                .map_err(|e| e.to_string())
            {
                Ok(c) => c,
                Err(e) => {
                    self.result = format!("error: {e}");
                    return Err(e);
                }
            };
            self.graph.clear();
            self.pois.clear();
            self.data = None;
            self.view2d = None;
            self.surface.clear();
            if first {
                self.reset_view_offsets();
            }
            self.result.clear();
            self.curve3d.push(curve);
            return Ok(());
        }
        let first = self.surface.is_empty();
        let surface =
            match sample_surface(source, 40, self.session.env()).map_err(|e| e.to_string()) {
                Ok(s) => s,
                Err(e) => {
                    self.result = format!("error: {e}");
                    return Err(e);
                }
            };
        // The pane shows one kind at a time (ADR-0015 amendment): drawing
        // a surface clears any 2D curves and their points of interest.
        self.graph.clear();
        self.pois.clear();
        self.data = None;
        self.view2d = None;
        self.curve3d.clear();
        // A 3D graph drawn into an empty pane brings fresh fine controls
        // at their default 0 (ADR-0031); overlays keep the current pose.
        if first {
            self.reset_view_offsets();
        }
        self.result.clear();
        self.surface.push(surface);
        Ok(())
    }

    /// The plotted 3D parametric curves, if any (ADR-0054).
    pub fn curve3ds(&self) -> &[SpaceCurve] {
        &self.curve3d
    }

    /// The plotted surfaces, if any.
    /// The plotted data plot, if any (ADR-0044).
    pub fn data(&self) -> Option<&DataPlot> {
        self.data.as_ref()
    }

    /// The plotted solar system scene, if any (ADR-0037).
    pub fn solar(&self) -> Option<&SolarScene> {
        self.solar.as_ref()
    }

    pub fn surfaces(&self) -> &[Surface] {
        &self.surface
    }

    /// The 3D camera pose.
    pub fn view(&self) -> &View3D {
        &self.view
    }

    /// The 3D fine-control offsets: (horizontal, vertical, zoom).
    pub fn view_offsets(&self) -> (f64, f64, f64) {
        (self.view_h, self.view_v, self.view_z)
    }

    /// The effective pose: the orbit base with the fine-control offsets
    /// applied (ADR-0031).
    pub fn effective_view(&self) -> View3D {
        self.view
            .with_offsets(self.view_h, self.view_v, self.view_z)
    }

    /// Nudge one fine-control offset by ±0.1, clamped to −1..1 (the
    /// slider's range and step).
    pub fn nudge_view_offset(&mut self, axis: ViewAxis, delta: f64) {
        let slot = match axis {
            ViewAxis::Horizontal => &mut self.view_h,
            ViewAxis::Vertical => &mut self.view_v,
            ViewAxis::Zoom => &mut self.view_z,
        };
        *slot = (*slot + delta).clamp(-1.0, 1.0);
    }

    /// A freshly drawn 3D graph starts with the controls at 0 (their
    /// default), like the web sliders.
    pub fn reset_view_offsets(&mut self) {
        self.view_h = 0.0;
        self.view_v = 0.0;
        self.view_z = 0.0;
    }

    /// The Settings-menu row index when the highlight sits on one of the
    /// three fine-control rows (they exist only while 3D surfaces do).
    pub fn menu_view_item(&self) -> Option<usize> {
        match self.menu {
            Some((4, item @ 15..=17)) if !self.surface.is_empty() || self.solar.is_some() => {
                Some(item)
            }
            _ => None,
        }
    }

    /// Orbit the 3D view by the given yaw/pitch deltas (radians).
    pub fn rotate_view(&mut self, dyaw: f64, dpitch: f64) {
        self.view = self
            .view
            .with_pitch(self.view.pitch + dpitch)
            .with_yaw(self.view.yaw + dyaw);
    }

    /// The active animation, if any.
    pub fn play(&self) -> Option<&Play> {
        self.play.as_ref()
    }

    /// Start or stop the parameter animation. Playing animates the first
    /// constant referenced by any plotted surface (or curve) within its
    /// current value ±2, stepping 0.1 per tick and wrapping around — the
    /// TUI's counterpart of the web sliders' play button (ADR-0015).
    pub fn toggle_play(&mut self) -> bool {
        if self.play.is_some() {
            self.play = None;
            return false;
        }
        let name = self.animated_constant();
        let Some(v) = name.as_ref().and_then(|n| self.session.env().constant(n)) else {
            return false;
        };
        let v = match v {
            epher_core::Value::Float(f) => *f,
            _ => return false,
        };
        self.play = Some(Play {
            name: name.unwrap(),
            lo: v - 2.0,
            hi: v + 2.0,
            step: 0.1,
        });
        true
    }

    /// The first constant referenced by a plotted surface, else a plotted
    /// curve — the parameter animation steps it.
    fn animated_constant(&self) -> Option<String> {
        let mut names = std::collections::BTreeSet::new();
        for s in &self.surface {
            if let Ok((expr, _)) = epher_core::graph::parse_surface_source(&s.source) {
                free_names(&expr, &mut names);
            }
        }
        for c in &self.curve3d {
            if let Ok((x, y, z, _)) = epher_core::graph::parse_space_curve_source(&c.source) {
                free_names(&x, &mut names);
                free_names(&y, &mut names);
                free_names(&z, &mut names);
            }
        }
        if let Some(source) = &self.solar_source {
            if let Ok(expr) = epher_core::parse(source) {
                free_names(&expr, &mut names);
            }
        }
        if let Some(n) = names
            .into_iter()
            .find(|n| self.session.env().constant(n.as_str()).is_some())
        {
            return Some(n);
        }
        self.curve_animated_constant()
    }

    /// The first constant referenced by a plotted CURVE — the "space
    /// animates" hint shows only while one exists (ADR-0035 amendment),
    /// because the play button's web counterpart only appears next to a
    /// constant a curve uses.
    fn curve_animated_constant(&self) -> Option<String> {
        let mut names = std::collections::BTreeSet::new();
        for c in &self.graph {
            if let Ok(spec) = parse_graph_source(&c.source) {
                match &spec.kind {
                    epher_core::graph::CurveKind::Cartesian(e) => free_names(e, &mut names),
                    epher_core::graph::CurveKind::Parametric { x, y } => {
                        free_names(x, &mut names);
                        free_names(y, &mut names);
                    }
                    epher_core::graph::CurveKind::Polar(e) => free_names(e, &mut names),
                    epher_core::graph::CurveKind::Implicit(e) => free_names(e, &mut names),
                }
            }
        }
        names
            .into_iter()
            .find(|n| self.session.env().constant(n.as_str()).is_some())
    }

    /// Advance the animation by one tick: step the constant, wrapping at the
    /// bounds, and re-sample everything that references it.
    pub fn tick(&mut self) {
        let Some(play) = self.play.clone() else {
            return;
        };
        let Some(v) = self.session.env().constant(&play.name) else {
            self.play = None;
            return;
        };
        let v = match v {
            epher_core::Value::Float(f) => *f,
            _ => {
                self.play = None;
                return;
            }
        };
        let mut next = v + play.step;
        if next > play.hi {
            next = play.lo;
        }
        self.session.set_constant(
            play.name.clone(),
            epher_core::Value::float(next),
            String::new(),
        );
        self.resample_all();
    }

    /// Re-sample every plot against the current environment (after an
    /// animation tick moved a constant).
    fn resample_all(&mut self) {
        let env = self.session.env().clone();
        for c in &mut self.graph {
            if let Ok(spec) = parse_graph_source(&c.source) {
                if let Ok(samples) = sample_spec(&spec, 120, &env) {
                    c.samples = samples;
                }
            }
        }
        self.pois = analyze(&self.graph, &env);
        for s in &mut self.surface {
            if let Ok(fresh) = sample_surface(&s.source, 40, &env) {
                *s = fresh;
            }
        }
        for c in &mut self.curve3d {
            if let Ok(fresh) = sample_space_curve(&c.source, 240, &env) {
                *c = fresh;
            }
        }
        if let (Some(_scene), Some(source)) = (self.solar.as_ref(), self.solar_source.as_deref()) {
            if source_references_any_constant(source, &env) {
                if let Ok(jd) = epher_core::astro::eval_jd(source, &env) {
                    if let Ok(fresh) = epher_core::astro::solar_scene(jd) {
                        self.solar = Some(fresh);
                    }
                }
            }
        }
    }
}

/// Whether the solar scene's time expression depends on any session
/// constant - the resample gate (its expression is ordinary code, so
/// `const t = now(); solar3d t` replays through the existing
/// transport, ADR-0037).
fn source_references_any_constant(source: &str, env: &epher_core::Env) -> bool {
    let mut names = std::collections::BTreeSet::new();
    if let Ok(expr) = epher_core::parse(source) {
        free_names(&expr, &mut names);
    }
    names.iter().any(|n| env.constant(n).is_some())
}

/// Render the projected 3D mesh as an ASCII wireframe (ADR-0015): depth-
/// shaded Bresenham lines on a uniform grid — near segments `*`, middle
/// `+`, far `.` — with the ground square and axes (`o`) drawn on top. The
/// painter-sorted segments overpaint in draw order, so nearer mesh lines
/// stay visible over farther ones.
pub fn render_ascii3d(surfaces: &[Surface], view: &View3D, width: usize, height: usize) -> String {
    if surfaces.is_empty() || width == 0 || height == 0 {
        return String::new();
    }
    let mut all = Vec::new();
    for (i, s) in surfaces.iter().enumerate() {
        all.extend(project_surface(s, view));
        if i == 0 {
            all.extend(surface_frame(s, view));
        }
    }
    let frame: Vec<Segment3D> = surfaces
        .first()
        .map(|s| surface_frame(s, view))
        .unwrap_or_default();
    render_ascii3d_scene(all, &frame, view, width, height)
}

/// Render a set of 3D parametric curves as an ASCII wireframe
/// (ADR-0054): the same depth-shaded treatment as the mesh, with the
/// curve's bounding-box frame on top.
pub fn render_ascii3d_curves(
    curves: &[SpaceCurve],
    view: &View3D,
    width: usize,
    height: usize,
) -> String {
    if curves.is_empty() || width == 0 || height == 0 {
        return String::new();
    }
    let mut all = Vec::new();
    for c in curves {
        for path in project_space_curve(&c.points, view) {
            for pair in path.points.windows(2) {
                all.push(Segment3D {
                    x1: pair[0].0,
                    y1: pair[0].1,
                    x2: pair[1].0,
                    y2: pair[1].1,
                    depth: path.depth,
                });
            }
        }
    }
    let frame: Vec<Segment3D> = curves
        .first()
        .map(|c| curve_frame(c, view))
        .unwrap_or_default();
    render_ascii3d_scene(all, &frame, view, width, height)
}

/// Stamp one segment list far-to-near (painter's algorithm), the frame
/// on top: the shared core of the surface and curve wireframes.
fn render_ascii3d_scene(
    all: Vec<Segment3D>,
    frame: &[Segment3D],
    view: &View3D,
    width: usize,
    height: usize,
) -> String {
    if all.is_empty() {
        return String::new();
    }
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for seg in &all {
        x_min = x_min.min(seg.x1).min(seg.x2);
        x_max = x_max.max(seg.x1).max(seg.x2);
        y_min = y_min.min(seg.y1).min(seg.y2);
        y_max = y_max.max(seg.y1).max(seg.y2);
    }
    let (x_min, x_max, y_min, y_max) = (x_min, x_max, y_min, y_max);
    if !x_min.is_finite() || x_max - x_min < 1e-9 || y_max - y_min < 1e-9 {
        return String::new();
    }
    // The zoom window (ADR-0015 amendment): the projection is affine, so
    // shrinking the window scales the scene without moving anything
    // relative to anything else.
    let (wx, wy, ww, wh) = zoom_window(x_min, x_max, y_min, y_max, view);
    let depth_min = all.iter().map(|s| s.depth).fold(f64::INFINITY, f64::min);
    let depth_max = all
        .iter()
        .map(|s| s.depth)
        .fold(f64::NEG_INFINITY, f64::max);
    let span = depth_max - depth_min;
    let gw = width - 2;
    let gh = height;
    let scale = ((gw as f64) / ww).min((gh as f64) / wh);
    let ox = (gw as f64 - ww * scale) / 2.0;
    let oy = (gh as f64 - wh * scale) / 2.0;
    let to_grid = |x: f64, y: f64| {
        let c = (x - wx) * scale + ox;
        let r = (wy + wh - y) * scale + oy;
        (r as isize, c as isize)
    };
    let mut grid = vec![vec![' '; width]; height];
    let mut stamp = |x1: f64, y1: f64, x2: f64, y2: f64, depth: f64, frame: bool| {
        let (r1, c1) = to_grid(x1, y1);
        let (r2, c2) = to_grid(x2, y2);
        let glyph = if frame {
            'o'
        } else if span < 1e-9 {
            '*'
        } else {
            let t = ((depth - depth_min) / span * 2.0).clamp(0.0, 2.0);
            ['*', '+', '.'][t.floor() as usize]
        };
        // Bresenham
        let (dr, dc) = (r2 - r1, c2 - c1);
        let steps = dr.abs().max(dc.abs());
        if steps == 0 {
            if r1 >= 0 && r1 < height as isize && c1 >= 0 && c1 < width as isize {
                grid[r1 as usize][c1 as usize] = glyph;
            }
            return;
        }
        for k in 0..=steps {
            let r = r1 + (dr * k) / steps;
            let c = c1 + (dc * k) / steps;
            if r >= 0 && r < height as isize && c >= 0 && c < width as isize {
                let cell = &mut grid[r as usize][c as usize];
                // Nearer mesh overpaints farther mesh; the frame (drawn
                // last) overpaints everything.
                if frame || *cell != 'o' {
                    *cell = glyph;
                }
            }
        }
    };
    // Far to near: nearer (drawn later) overpaints.
    let mut order = all.iter().collect::<Vec<_>>();
    order.sort_by(|a, b| a.depth.total_cmp(&b.depth));
    for seg in order {
        stamp(seg.x1, seg.y1, seg.x2, seg.y2, seg.depth, false);
    }
    // Frame last, on top.
    for seg in frame {
        stamp(seg.x1, seg.y1, seg.x2, seg.y2, seg.depth, true);
    }
    grid.into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Render the solar system scene as ASCII (ADR-0037 + the ADR-0015
/// amendment): orbit and trail polylines as depth-shaded Bresenham runs
/// (the same glyphs as the mesh), each positioned dot stamped `O` on top
/// with its body's first letter beside it - the legend row above the
/// pane names the bodies in the same order.
pub fn render_solar_ascii(
    scene: &SolarScene,
    view: &View3D,
    width: usize,
    height: usize,
) -> String {
    use epher_core::graph::{project_space_curve, project_world_dot};
    if width == 0 || height == 0 {
        return String::new();
    }
    let mut segments = Vec::new();
    for path in scene.orbits.iter().chain(scene.trails.iter()) {
        for run in project_space_curve(&path.points, view) {
            for pair in run.points.windows(2) {
                segments.push(Segment3D {
                    x1: pair[0].0,
                    y1: pair[0].1,
                    x2: pair[1].0,
                    y2: pair[1].1,
                    depth: run.depth,
                });
            }
        }
    }
    if segments.is_empty() {
        return String::new();
    }
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for seg in &segments {
        x_min = x_min.min(seg.x1).min(seg.x2);
        x_max = x_max.max(seg.x1).max(seg.x2);
        y_min = y_min.min(seg.y1).min(seg.y2);
        y_max = y_max.max(seg.y1).max(seg.y2);
    }
    let mut dots: Vec<(f64, f64, f64, i64)> = scene
        .dots
        .iter()
        .filter_map(|d| {
            project_world_dot(d.xyz[0], d.xyz[1], d.xyz[2], view)
                .map(|(x, y, zp)| (x, y, zp, d.body))
        })
        .collect();
    for (x, y, _, _) in &dots {
        x_min = x_min.min(*x);
        x_max = x_max.max(*x);
        y_min = y_min.min(*y);
        y_max = y_max.max(*y);
    }
    if !x_min.is_finite() || x_max - x_min < 1e-9 || y_max - y_min < 1e-9 {
        return String::new();
    }
    // The zoom window (ADR-0015 amendment): the projection is affine, so
    // shrinking the window scales the scene without moving anything
    // relative to anything else.
    let (wx, wy, ww, wh) = zoom_window(x_min, x_max, y_min, y_max, view);
    let depth_min = segments
        .iter()
        .map(|s| s.depth)
        .fold(f64::INFINITY, f64::min);
    let depth_max = segments
        .iter()
        .map(|s| s.depth)
        .fold(f64::NEG_INFINITY, f64::max);
    let span = depth_max - depth_min;
    let gw = width - 2;
    let gh = height;
    let scale = ((gw as f64) / ww).min((gh as f64) / wh);
    let ox = (gw as f64 - ww * scale) / 2.0;
    let oy = (gh as f64 - wh * scale) / 2.0;
    let to_grid = |x: f64, y: f64| {
        let c = (x - wx) * scale + ox;
        let r = (wy + wh - y) * scale + oy;
        (r as isize, c as isize)
    };
    let mut grid = vec![vec![' '; width]; height];
    let mut order = segments.iter().collect::<Vec<_>>();
    order.sort_by(|a, b| a.depth.total_cmp(&b.depth));
    for seg in order {
        let (r1, c1) = to_grid(seg.x1, seg.y1);
        let (r2, c2) = to_grid(seg.x2, seg.y2);
        let glyph = if span < 1e-9 {
            '*'
        } else {
            let t = ((seg.depth - depth_min) / span * 2.0).clamp(0.0, 2.0);
            ['*', '+', '.'][t.floor() as usize]
        };
        let (dr, dc) = (r2 - r1, c2 - c1);
        let steps = dr.abs().max(dc.abs());
        if steps == 0 {
            if r1 >= 0 && r1 < height as isize && c1 >= 0 && c1 < width as isize {
                grid[r1 as usize][c1 as usize] = glyph;
            }
            continue;
        }
        for k in 0..=steps {
            let r = r1 + (dr * k) / steps;
            let c = c1 + (dc * k) / steps;
            if r >= 0 && r < height as isize && c >= 0 && c < width as isize {
                grid[r as usize][c as usize] = glyph;
            }
        }
    }
    // Dots on top, painted far-to-near so a nearer body overpaints a
    // farther one (depth grows toward the camera).
    dots.sort_by(|a, b| a.2.total_cmp(&b.2));
    for (x, y, _, body) in dots {
        let (r, c) = to_grid(x, y);
        if r >= 0 && r < height as isize && c >= 0 && c < width as isize {
            grid[r as usize][c as usize] = 'O';
            let label = epher_core::astro::body_name(body);
            if let Some(first) = label.chars().next() {
                let c2 = c + 1;
                if c2 >= 0 && c2 < width as isize {
                    let cell = &mut grid[r as usize][c2 as usize];
                    if *cell == ' ' {
                        *cell = first;
                    }
                }
            }
        }
    }
    grid.into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Render the plotted curves as an ASCII plot — the TUI's renderer
/// (ADR-0006/0014). The x and y ranges are scaled to the grid; each curve
/// plots with its own glyph (`o`, `x`, `+`, `*`); region fills shade with
/// `.`; axes draw as `|`/`-` when zero lies strictly inside the range
/// (edge-zero plots stay clean); non-finite points are skipped.
/// The auto-fit ranges around a curve set's finite samples: the plot's
/// default viewport (ADR-0034), shared by the renderer and the mouse's
/// pan/zoom math.
pub fn curve_ranges(curves: &[SampledCurve]) -> Option<(f64, f64, f64, f64)> {
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for c in curves {
        for s in &c.samples {
            if s.x.is_finite() && s.y.is_finite() {
                x_min = x_min.min(s.x);
                x_max = x_max.max(s.x);
                y_min = y_min.min(s.y);
                y_max = y_max.max(s.y);
            }
        }
    }
    if !x_min.is_finite() {
        return None;
    }
    Some((x_min, x_max, y_min, y_max))
}

/// Render the sampled curves as an ASCII plot. A `view` override (the
/// mouse's pan/zoom viewport, ADR-0034) replaces the auto-fit ranges
/// when present.
pub fn render_ascii(
    curves: &[SampledCurve],
    width: usize,
    height: usize,
    view: Option<(f64, f64, f64, f64)>,
) -> String {
    if curves.is_empty() || curves.iter().all(|c| c.samples.is_empty()) || width == 0 || height == 0
    {
        return String::new();
    }
    let Some((x_min, x_max, y_min, y_max)) = view.or_else(|| curve_ranges(curves)) else {
        return String::new();
    };
    let x_span = (x_max - x_min).max(1e-12);
    let y_span = (y_max - y_min).max(1e-12);

    let mut grid = vec![vec!['·'; width]; height];

    // Region fills under/above each curve.
    for c in curves {
        let Some(fill) = c.fill else { continue };
        let below = matches!(fill, epher_core::graph::Fill::Below);
        for s in &c.samples {
            if !s.x.is_finite() || !s.y.is_finite() {
                continue;
            }
            // The pan/zoom viewport (ADR-0034) can leave samples outside
            // the window: they draw nothing, here and for the glyphs.
            let col_f = ((s.x - x_min) / x_span) * (width - 1) as f64;
            let row_f = ((y_max - s.y) / y_span) * (height - 1) as f64;
            if !(0.0..=(width - 1) as f64).contains(&col_f)
                || !(0.0..=(height - 1) as f64).contains(&row_f)
            {
                continue;
            }
            let col = col_f.round() as usize;
            let row = row_f.round() as usize;
            if below {
                for cell_row in grid[(row + 1)..].iter_mut() {
                    if cell_row[col] == '·' {
                        cell_row[col] = '.';
                    }
                }
            } else {
                for cell_row in grid[..row].iter_mut() {
                    if cell_row[col] == '·' {
                        cell_row[col] = '.';
                    }
                }
            }
        }
    }

    // Axes, only when zero is strictly inside the range.
    let eps_x = x_span * 1e-9;
    let eps_y = y_span * 1e-9;
    if x_min + eps_x < 0.0 && 0.0 < x_max - eps_x {
        let col = ((-x_min) / x_span * (width - 1) as f64).round() as usize;
        let col = col.min(width - 1);
        for row in grid.iter_mut() {
            if row[col] == '·' {
                row[col] = '|';
            }
        }
    }
    if y_min + eps_y < 0.0 && 0.0 < y_max - eps_y {
        let row = height as f64 - 1.0 - ((-y_min) / y_span) * (height - 1) as f64;
        let row = (row.round() as usize).min(height - 1);
        for cell in grid[row].iter_mut() {
            if *cell == '·' {
                *cell = '-';
            }
        }
    }

    // Curves, glyph per curve index. Samples outside the pan/zoom
    // viewport (ADR-0034) are clipped, like the fills above.
    const GLYPHS: [char; 4] = ['o', 'x', '+', '*'];
    for (i, c) in curves.iter().enumerate() {
        let glyph = GLYPHS[i % GLYPHS.len()];
        for s in &c.samples {
            if !s.x.is_finite() || !s.y.is_finite() {
                continue;
            }
            let col_f = ((s.x - x_min) / x_span) * (width - 1) as f64;
            let row_f = ((y_max - s.y) / y_span) * (height - 1) as f64;
            if !(0.0..=(width - 1) as f64).contains(&col_f)
                || !(0.0..=(height - 1) as f64).contains(&row_f)
            {
                continue;
            }
            let col = col_f.round() as usize;
            let row = row_f.round() as usize;
            grid[row][col] = glyph;
        }
    }
    grid.into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Render a data plot (ADR-0044) as ASCII on the same canvas the curve
/// renderer uses: scatter points with the fitted line, histogram bars,
/// or a box-and-whisker row.
pub fn render_ascii_data(data: &DataPlot, width: usize, height: usize) -> String {
    if width == 0 || height == 0 {
        return String::new();
    }
    let (x_min, x_max, y_min, y_max) = epher_core::graph::data_ranges(data);
    let x_span = (x_max - x_min).max(1e-12);
    let y_span = (y_max - y_min).max(1e-12);
    let mut grid = vec![vec!['·'; width]; height];
    let col_of =
        |x: f64| -> usize { (((x - x_min) / x_span) * (width - 1) as f64).round() as usize };
    let row_of =
        |y: f64| -> usize { (((y_max - y) / y_span) * (height - 1) as f64).round() as usize };
    match data.kind {
        epher_core::graph::DataPlotKind::Scatter => {
            // The fitted model first, so points drawn over it stay legible.
            if let Some(f) = data.fit {
                for (col, cell_row) in grid.iter_mut().enumerate() {
                    let x = x_min + col as f64 / (width - 1) as f64 * x_span;
                    let y = f.fit.eval(x);
                    if y.is_finite() && y >= y_min && y <= y_max {
                        let row = row_of(y);
                        if cell_row[row] == '·' {
                            cell_row[row] = '-';
                        }
                    }
                }
            }
            for (x, y) in &data.points {
                if x.is_finite() && y.is_finite() {
                    let (col, row) = (col_of(*x), row_of(*y));
                    grid[row][col] = 'o';
                }
            }
        }
        epher_core::graph::DataPlotKind::Histogram => {
            // Bars from the count-zero baseline to each bin's count.
            for (lo, hi, count) in &data.bins {
                let (c0, c1) = (col_of(*lo), col_of(*hi).max(col_of(*lo) + 1));
                let top = if *count > 0.0 {
                    row_of(*count)
                } else {
                    height - 1
                };
                let base = row_of(0.0);
                for col in c0..c1.min(width) {
                    if *count > 0.0 {
                        for cell in grid[top..=base.min(height - 1)].iter_mut() {
                            cell[col] = '█';
                        }
                    } else if base < height {
                        // a zero-count bin still marks its baseline
                        grid[base][col] = '█';
                    }
                }
            }
        }
        epher_core::graph::DataPlotKind::BoxPlot => {
            if let Some(b) = data.boxplot {
                let row = (height / 2).min(height - 1);
                let (c0, c1, cm, c2, c3) = (
                    col_of(b[0]),
                    col_of(b[1]),
                    col_of(b[2]),
                    col_of(b[3]),
                    col_of(b[4]),
                );
                grid[row][c0..=c1].fill('─');
                grid[row][c2..=c3].fill('─');
                grid[row][c1..=c2].fill('█');
                grid[row][cm] = '┼';
                grid[row][c0] = '╫';
                grid[row][c3] = '╫';
            }
        }
    }
    grid.into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The localized kind label for a point of interest (the same fluent keys
/// the web legend uses).
fn poi_label(kind: InterestKind, localizer: &Localizer) -> String {
    localizer.lookup(match kind {
        InterestKind::Root => "poi-root",
        InterestKind::Intersection => "poi-intersection",
        InterestKind::Maximum => "poi-maximum",
        InterestKind::Minimum => "poi-minimum",
    })
}

/// Run the interactive terminal UI (ratatui event loop). Blocks until the
/// user quits (Ctrl+C, or `q` with empty input). The loop itself is a thin
/// shell over [`App`]: it loads the shared store (ADR-0002), resolves the
/// UI language (ADR-0008), and forwards keys; all state transitions go
/// through the tested [`App`] seam.
pub fn run() -> std::io::Result<()> {
    use crossterm::execute;
    let mut terminal = ratatui::init();
    // Mouse capture (ADR-0034): the menu bar, history, keypad, and the
    // graph panels all respond to the pointer; released on restore.
    // Bracketed paste (ADR-0059): the terminal wraps a paste between
    // marker sequences, so the whole clipboard arrives as one
    // Event::Paste instead of a burst of keystrokes; terminals without
    // support keep delivering keystrokes, which the key arm still
    // accepts (the Ctrl+J convention below).
    let _ = execute!(
        std::io::stdout(),
        crossterm::event::EnableMouseCapture,
        crossterm::event::EnableBracketedPaste
    );
    let result = run_loop(&mut terminal);
    let _ = execute!(
        std::io::stdout(),
        crossterm::event::DisableMouseCapture,
        crossterm::event::DisableBracketedPaste
    );
    ratatui::restore();
    result
}

fn run_loop(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

    let store_dir = default_store_dir();
    let store = DocStore::new(FsStore::new(&store_dir));
    // Publish/subscribe (ADR-0010 amendment): the TUI writes its state
    // immediately at every submit, and this watcher delivers a signal
    // whenever another frontend (desktop app, CLI, another TUI) changes
    // the shared store, so the open TUI refreshes live.
    let store_rx = epher_store::watch::spawn_store_watcher(store_dir);
    let session = match load_session(&store) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("warning: could not load saved data ({e}); starting fresh");
            Session::new()
        }
    };
    let preference = load_language(&store).unwrap_or(None);
    let detected: Vec<String> = sys_locale::get_locales().collect();
    let mut localizer = Localizer::resolve(preference.as_deref(), &detected);
    let mut app = App::with_session(session);
    // The stored theme (ADR-0017) wins over the default dark palette;
    // the stored points-of-interest choice (ADR-0019) over "shown".
    if let Some(name) = load_theme(&store).unwrap_or(None) {
        if let Some(theme) = Theme::from_str(&name) {
            app.set_theme(theme);
        }
    }
    if let Some(pois) = load_pois(&store).unwrap_or(None) {
        app.set_pois(pois);
    }
    // Result rendering (ADR-0043): the stored display preferences shape
    // every result line; the defaults are exact fractions on, Auto
    // notation, no separators.
    {
        use epher_core::{DisplayPrefs, Notation};
        let mut prefs = DisplayPrefs::default();
        if let Some(exact) = load_exact(&store).unwrap_or(None) {
            prefs.exact_fractions = exact;
        }
        if let Some(format) = load_format(&store).unwrap_or(None) {
            prefs.notation = match format.as_str() {
                "scientific" => Notation::Scientific,
                "engineering" => Notation::Engineering,
                _ => Notation::Auto,
            };
        }
        if let Some(separators) = load_separators(&store).unwrap_or(None) {
            prefs.separators = separators;
        }
        app.set_display_prefs(prefs);
    }
    // One step per 120 ms while playing — the same rate as the web
    // sliders' play button (ADR-0015). The poll below wakes at 50 ms so
    // key presses stay responsive; the step itself is paced here.
    let mut last_tick = std::time::Instant::now();
    // Mouse state (ADR-0034): the graph drag in flight and the previous
    // click for double-click detection.
    let mut drag: Option<(u16, u16, MouseDrag)> = None;
    let mut last_click: Option<(std::time::Instant, u16, u16)> = None;
    loop {
        terminal.draw(|frame| draw(frame, &mut app, &localizer))?;
        // The store watcher and the animation both need a bounded wait:
        // the poll wakes every 50 ms, so key presses stay responsive and
        // external store changes are noticed within one poll.
        let event = match event::poll(std::time::Duration::from_millis(50)) {
            Ok(true) => Some(event::read()?),
            Ok(false) => None,
            Err(e) => return Err(e),
        };
        // Another frontend wrote to the shared store (ADR-0010
        // amendment): reload the session — history, functions,
        // constants, scripts, and the bindings snapshot — keeping the
        // in-flight entry text and the plot state untouched. Definitions
        // the user created in THIS session but has not `save`d yet are
        // replayed over the store's state, so a foreign write (or our
        // own echo) cannot erase live work; the store's bindings win.
        // Reloading never writes, so no loop is possible.
        if epher_store::watch::drain_signal(&store_rx) {
            if let Ok(mut fresh) = load_session(&store) {
                for source in app.session().def_sources().values() {
                    fresh.submit_quiet(source);
                }
                for source in app.session().const_sources().values() {
                    fresh.submit_quiet(source);
                }
                app.set_session(fresh);
            }
            if let Some(pref) = load_language(&store).unwrap_or(None) {
                if localizer.locale() != pref {
                    localizer = Localizer::resolve(Some(&pref), &[]);
                }
            }
            if let Some(name) = load_theme(&store).unwrap_or(None) {
                if let Some(theme) = Theme::from_str(&name) {
                    app.set_theme(theme);
                }
            }
            if let Some(pois) = load_pois(&store).unwrap_or(None) {
                app.set_pois(pois);
            }
        }
        if app.play().is_some() && last_tick.elapsed() >= std::time::Duration::from_millis(120) {
            app.tick();
            last_tick = std::time::Instant::now();
        }
        match event {
            Some(Event::Mouse(me)) => {
                // The pointer works against the regions the last frame
                // drew (ADR-0034): menus, history, keypad, and the
                // graph panels.
                if handle_mouse(
                    &mut app,
                    &mut localizer,
                    &store,
                    me,
                    &mut drag,
                    &mut last_click,
                ) {
                    return Ok(());
                }
            }
            Some(Event::Key(key)) => {
                if key.kind == KeyEventKind::Press {
                    // The user guide view (ADR-0018) is modal: only scrolling
                    // and closing keys act; nothing reaches the calculator.
                    // Number keys jump the table of contents (ADR-0018
                    // amendment) — the keyboard spelling of the ToC clicks.
                    if app.guide_active() {
                        // While a search is being typed (the ADR-0038
                        // amendment), the keys feed the query and Enter
                        // jumps; Esc clears the query before it closes.
                        if app.guide_searching() {
                            match key.code {
                                KeyCode::Char('c')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    return Ok(());
                                }
                                KeyCode::Esc => app.guide_search_clear(),
                                KeyCode::Enter => app.guide_jump_next_hit(),
                                KeyCode::Backspace => app.guide_search_pop(),
                                KeyCode::Char(c) => app.guide_search_push(c),
                                _ => {}
                            }
                            continue;
                        }
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                return Ok(());
                            }
                            KeyCode::Esc | KeyCode::Char('q') => app.guide_close(),
                            KeyCode::Char('/') => app.guide_search_start(),
                            KeyCode::Up => app.guide_scroll(-1),
                            KeyCode::Down => app.guide_scroll(1),
                            KeyCode::PageUp => app.guide_scroll(-12),
                            KeyCode::PageDown => app.guide_scroll(12),
                            KeyCode::Home => app.guide_scroll_to(0),
                            KeyCode::End => app.guide_scroll_to(usize::MAX),
                            KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                                app.guide_jump((c as u8 - b'1') as usize)
                            }
                            _ => {}
                        }
                        continue;
                    }
                    // The key-help overlay (ADR-0039) is modal like the
                    // guide: scrolling and closing keys act; nothing
                    // reaches the calculator.
                    if app.key_help_active() {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                return Ok(());
                            }
                            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                                app.key_help_close()
                            }
                            KeyCode::Up => app.key_help_scroll(-1),
                            KeyCode::Down => app.key_help_scroll(1),
                            KeyCode::PageUp => app.key_help_scroll(-12),
                            KeyCode::PageDown => app.key_help_scroll(12),
                            KeyCode::Home => {
                                app.key_help = Some(0);
                            }
                            KeyCode::End => {
                                app.key_help = Some(usize::MAX);
                            }
                            _ => {}
                        }
                        continue;
                    }
                    // The constants browser (ADR-0045) is modal like the
                    // key help: arrows move the selection, Enter inserts
                    // the name into the input line, Escape closes.
                    if app.constants_active() {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                return Ok(());
                            }
                            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                                app.constants_close()
                            }
                            KeyCode::Up => app.constants_select(-1),
                            KeyCode::Down => app.constants_select(1),
                            KeyCode::PageUp => app.constants_select(-12),
                            KeyCode::PageDown => app.constants_select(12),
                            KeyCode::Home => {
                                app.constants = Some(0);
                            }
                            KeyCode::End => {
                                app.constants = Some(usize::MAX);
                            }
                            KeyCode::Enter => app.constants_insert(),
                            _ => {}
                        }
                        continue;
                    }
                    // With bracketed paste off — the terminal does not
                    // support it, or a frontend sends keys directly — a
                    // pasted newline arrives as LF, which crossterm parses
                    // as Ctrl+J (the terminal convention for line feed).
                    // Treat it as Enter so the paste still submits, line
                    // by line. With bracketed paste on, the whole
                    // clipboard lands as one Event::Paste (ADR-0059).
                    let is_enter = key.code == KeyCode::Enter
                        || (key.code == KeyCode::Char('j')
                            && key.modifiers.contains(KeyModifiers::CONTROL));
                    match key.code {
                        // Guarded arms must precede the generic `Char` arm — the
                        // catch-all would swallow Ctrl+C and type a 'c' instead.
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(());
                        }
                        // File prompt mode (ADR-0017): almost every key goes to
                        // the path buffer.
                        KeyCode::Char(c) if app.prompt_active().is_some() && !is_enter => {
                            app.prompt_push(c);
                        }
                        KeyCode::Backspace if app.prompt_active().is_some() => {
                            app.prompt_pop();
                        }
                        KeyCode::Esc if app.prompt_active().is_some() => {
                            app.prompt_cancel();
                        }
                        // Menu bar mode (ADR-0017): F10 opens/closes; arrows
                        // move; Enter activates; Esc closes.
                        KeyCode::F(10) if app.prompt_active().is_none() => {
                            if app.menu_active().is_some() {
                                app.menu_close();
                            } else {
                                app.menu_open(0);
                            }
                        }
                        KeyCode::Left if app.menu_view_item().is_some() => {
                            if let Some(item) = app.menu_view_item() {
                                let axis = view_axis_of(item);
                                app.nudge_view_offset(axis, -0.1);
                            }
                        }
                        KeyCode::Right if app.menu_view_item().is_some() => {
                            if let Some(item) = app.menu_view_item() {
                                let axis = view_axis_of(item);
                                app.nudge_view_offset(axis, 0.1);
                            }
                        }
                        KeyCode::Left if app.menu_active().is_some() => app.menu_move(-1, 0),
                        KeyCode::Right if app.menu_active().is_some() => app.menu_move(1, 0),
                        KeyCode::Up if app.menu_active().is_some() => app.menu_move(0, -1),
                        KeyCode::Down if app.menu_active().is_some() => app.menu_move(0, 1),
                        KeyCode::Esc if app.menu_active().is_some() => app.menu_close(),
                        // The keypad's docked state (ADR-0060): Ctrl+K
                        // slides the keypad away and back — the keyboard
                        // spelling of the grab bar's drag. Works while
                        // the keypad has focus too (hiding drops the
                        // focus, so the grid is never an invisible trap).
                        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.keypad_toggle();
                        }
                        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.clear_history();
                            let _ = save_history(&store, app.history());
                            let _ = save_session(&store, app.bindings());
                        }
                        KeyCode::Char('q') if app.input().is_empty() && !app.keypad_focused() => {
                            return Ok(());
                        }
                        // The key-help overlay (ADR-0039): ? opens it when
                        // nothing else owns the key — the entry is empty
                        // (the same spelling rule as `q`) or the keypad has
                        // focus, where ? types nothing. The Help menu opens
                        // it too, for discovery.
                        KeyCode::Char('?')
                            if app.input().is_empty()
                                && app.prompt_active().is_none()
                                && app.menu_active().is_none() =>
                        {
                            app.key_help_open();
                        }
                        // Keypad mode (ADR-0016): Tab opens the button grid
                        // and cycles its banks (Shift+Tab cycles back);
                        // inside it, arrows move, Enter inserts, Esc closes.
                        KeyCode::Tab => {
                            // Focus cycle: input → keypad → history → input
                            // (ADR-0027: the history list is the last stop so
                            // a picked line drops you back on the input).
                            if app.keypad_focused() {
                                app.keypad_close();
                                app.history_open();
                            } else if app.history_focused() {
                                app.history_close();
                            } else {
                                app.keypad_open();
                            }
                        }
                        KeyCode::BackTab => {
                            if app.keypad_focused() {
                                app.keypad_cycle(-1);
                            } else if app.history_focused() {
                                app.history_close();
                                app.keypad_open();
                            }
                        }
                        KeyCode::Left if app.keypad_focused() => app.keypad_move(0, -1),
                        KeyCode::Right if app.keypad_focused() => app.keypad_move(0, 1),
                        KeyCode::Up if app.keypad_focused() => app.keypad_move(-1, 0),
                        KeyCode::Down if app.keypad_focused() => app.keypad_move(1, 0),
                        KeyCode::Esc if app.keypad_focused() => app.keypad_close(),
                        // History focus (ADR-0027): arrows move the selection
                        // (up = older), Esc steps back to the input.
                        KeyCode::Up if app.history_focused() => app.history_move(1),
                        KeyCode::Down if app.history_focused() => app.history_move(-1),
                        KeyCode::Esc if app.history_focused() => app.history_close(),
                        // The entry owns the arrow keys while it holds text
                        // (ADR-0035 amendment): Left/Right move the caret,
                        // Up/Down move between lines of a multi-line script,
                        // Home/End jump to the line's edges. The graph's
                        // rotation arms below only take the arrows the entry
                        // does not need (its input is empty), so typing never
                        // loses an arrow key to the plot.
                        KeyCode::Left
                            if app.prompt_active().is_none()
                                && app.menu_active().is_none()
                                && !app.keypad_focused()
                                && !app.history_focused()
                                && !app.input().is_empty() =>
                        {
                            app.cursor_move(-1)
                        }
                        KeyCode::Right
                            if app.prompt_active().is_none()
                                && app.menu_active().is_none()
                                && !app.keypad_focused()
                                && !app.history_focused()
                                && !app.input().is_empty() =>
                        {
                            app.cursor_move(1)
                        }
                        KeyCode::Up
                            if app.prompt_active().is_none()
                                && app.menu_active().is_none()
                                && !app.keypad_focused()
                                && !app.history_focused()
                                && !app.input().is_empty() =>
                        {
                            app.cursor_line(-1)
                        }
                        KeyCode::Down
                            if app.prompt_active().is_none()
                                && app.menu_active().is_none()
                                && !app.keypad_focused()
                                && !app.history_focused()
                                && !app.input().is_empty() =>
                        {
                            app.cursor_line(1)
                        }
                        KeyCode::Home
                            if app.prompt_active().is_none()
                                && !app.keypad_focused()
                                && !app.history_focused()
                                && app.menu_active().is_none() =>
                        {
                            app.cursor_line_edge(-1)
                        }
                        KeyCode::End
                            if app.prompt_active().is_none()
                                && !app.keypad_focused()
                                && !app.history_focused()
                                && app.menu_active().is_none() =>
                        {
                            app.cursor_line_edge(1)
                        }
                        // Shift+Enter starts a new line in the entry
                        // (ADR-0035 amendment): the same gesture as the
                        // desktop app and the web app. Plain Enter still
                        // runs the whole multi-line script as one history
                        // item.
                        KeyCode::Enter
                            if key.modifiers.contains(KeyModifiers::SHIFT)
                                && app.prompt_active().is_none() =>
                        {
                            app.push_char('\n')
                        }
                        // 3D orbit (ADR-0015): arrows rotate when the input line
                        // is empty, so typing never loses an arrow key.
                        KeyCode::Left if app.input().is_empty() => app.rotate_view(-0.15, 0.0),
                        KeyCode::Right if app.input().is_empty() => app.rotate_view(0.15, 0.0),
                        KeyCode::Up if app.input().is_empty() => app.rotate_view(0.0, 0.15),
                        KeyCode::Down if app.input().is_empty() => app.rotate_view(0.0, -0.15),
                        // Space starts/stops the parameter animation (ADR-0015).
                        KeyCode::Char(' ') if app.input().is_empty() => {
                            app.toggle_play();
                        }
                        // Any typed character leaves keypad/history focus
                        // first — typing is the other spelling of the same
                        // input.
                        KeyCode::Char(c) if !is_enter => {
                            app.keypad_close();
                            app.history_close();
                            app.menu_close();
                            app.push_char(c);
                        }
                        KeyCode::Backspace => {
                            app.history_close();
                            app.pop_char();
                        }
                        // F1 (ADR-0042): function help for the word before
                        // the caret, into the answer line - the next
                        // submission replaces it.
                        KeyCode::F(1)
                            if app.prompt_active().is_none()
                                && app.menu_active().is_none()
                                && !app.keypad_focused()
                                && !app.history_focused() =>
                        {
                            let word = app.word_before_cursor().to_string();
                            let hint_key = format!("key-hint-{word}");
                            let hint = localizer.lookup(&hint_key);
                            if word.is_empty() || hint == hint_key {
                                app.set_result(&localizer.lookup("help-no-description"));
                            } else {
                                app.set_result(&format!("{word}: {hint}"));
                            }
                        }
                        KeyCode::Esc => app.clear_input(),
                        _ => {}
                    }
                    // Shift+Enter is handled above (it inserts a newline);
                    // every other Enter press — including pasted Ctrl+J —
                    // runs the submit chain below.
                    let shift_enter =
                        key.code == KeyCode::Enter && key.modifiers.contains(KeyModifiers::SHIFT);
                    if is_enter && !shift_enter && app.prompt_active().is_some() {
                        // Confirm the file prompt: execute, and either close
                        // with a success message or reopen with the path kept.
                        let kind = app.prompt_active().map(|(k, _)| k);
                        let path = app
                            .prompt_active()
                            .map(|(_, buf)| buf.to_string())
                            .unwrap_or_default();
                        match app.prompt_submit(&localizer) {
                            Some(failed) => {
                                app.prompt_restore(failed, &path);
                                let msg = if matches!(
                                    failed,
                                    PromptKind::OpenHistory | PromptKind::OpenScript
                                ) {
                                    localizer.lookup("tui-open-failed")
                                } else {
                                    localizer.lookup("tui-save-failed")
                                };
                                app.set_result(&msg);
                            }
                            None => {
                                // OpenHistory already reported the loaded line
                                // count; the other outcomes report here.
                                let msg = match kind {
                                    Some(PromptKind::OpenScript) => {
                                        Some(localizer.lookup("menu-loaded"))
                                    }
                                    Some(PromptKind::OpenHistory) => None,
                                    _ => Some(localizer.lookup_args("saved", &[("name", &path)])),
                                };
                                if let Some(msg) = msg {
                                    app.set_result(&msg);
                                }
                            }
                        }
                    } else if is_enter && !shift_enter && app.menu_active().is_some() {
                        if let Some(action) = app.menu_activate() {
                            if perform_menu_action(&mut app, &store, &mut localizer, action) {
                                return Ok(());
                            }
                        }
                    } else if is_enter && !shift_enter && app.history_focused() {
                        // Pick the highlighted history line into the input
                        // (ADR-0027) — the user edits and re-runs it.
                        if let Some(line) = app.history_pick() {
                            app.set_input(&line);
                        }
                    } else if is_enter
                        && !shift_enter
                        && (!app.keypad_focused() || app.keypad_is_submit())
                    {
                        // The entry's Enter and the keypad's "=" key run the
                        // same submit path (ADR-0016).
                        let line = app.input().trim().to_string();
                        if let Some(code) = app.submit_line(&line, &store, &localizer) {
                            localizer = Localizer::resolve(Some(&code), &[]);
                        }
                        // Every submit empties the line — including graph
                        // commands, whose path doesn't clear it itself — so a
                        // multi-line paste leaves a clean slate for the next
                        // line instead of appending to the leftover.
                        app.clear_input();
                    } else if is_enter && !shift_enter {
                        app.keypad_insert();
                    }
                }
            }
            Some(Event::Paste(text)) => {
                // Bracketed paste (ADR-0059): the clipboard arrives as
                // one event. Land it in the entry as one unit — newlines
                // and all — so a script pasted from the website runs on
                // the next Enter, exactly like the desktop and web
                // entries. The old keystroke-by-keystroke delivery
                // submitted each line on its own, and a script's opening
                // `/* banner` line died as an unterminated comment. The
                // guide stays modal; a path prompt takes the text
                // without its control characters.
                if app.guide_active() {
                    // Modal: pasted text is swallowed, like every key
                    // but scrolling and closing.
                } else if app.prompt_active().is_some() {
                    for c in text.chars().filter(|c| !c.is_control()) {
                        app.prompt_push(c);
                    }
                } else {
                    app.keypad_close();
                    app.history_close();
                    app.menu_close();
                    app.constants_close();
                    app.paste_text(&text);
                }
            }
            None => {}
            Some(_) => {}
        }
    }
}

/// Apply a menu action — Enter on the highlighted row, or a mouse click
/// on it (ADR-0034). Returns true when the action quits the TUI.
fn perform_menu_action(
    app: &mut App,
    store: &DocStore<FsStore>,
    localizer: &mut Localizer,
    action: MenuAction,
) -> bool {
    match action {
        MenuAction::OpenHistory => app.prompt_start(PromptKind::OpenHistory),
        MenuAction::OpenScript => app.prompt_start(PromptKind::OpenScript),
        MenuAction::SaveHistory => app.prompt_start(PromptKind::SaveHistory),
        MenuAction::SaveScript => app.prompt_start(PromptKind::SaveScript),
        MenuAction::Quit => return true,
        MenuAction::Cut => {
            if !app.input().is_empty() {
                osc52_copy(app.input());
                app.clear_input();
            }
        }
        MenuAction::Copy => {
            let text = if app.result().is_empty() {
                app.input().to_string()
            } else {
                app.result().to_string()
            };
            if !text.is_empty() {
                osc52_copy(&text);
            }
        }
        // Terminals have no read-side clipboard API: paste stays with
        // the terminal itself.
        MenuAction::Paste => {
            let hint = localizer.lookup("tui-paste-hint");
            app.set_result(&hint);
        }
        MenuAction::SetTheme(name) => {
            app.set_theme(Theme::from_str(name).unwrap_or(Theme::Dark));
            let _ = save_theme(store, name);
        }
        MenuAction::SetLanguage(code) => {
            *localizer = Localizer::resolve(Some(code), &[]);
            let _ = save_language(store, code);
        }
        MenuAction::TogglePois => {
            app.toggle_pois();
            let _ = save_pois(store, app.poi_list());
        }
        MenuAction::ToggleExact => {
            app.toggle_exact();
            let _ = save_exact(store, app.display_prefs().exact_fractions);
        }
        MenuAction::CycleNotation => {
            app.cycle_notation();
            let notation = match app.display_prefs().notation {
                epher_core::Notation::Auto => "auto",
                epher_core::Notation::Scientific => "scientific",
                epher_core::Notation::Engineering => "engineering",
            };
            let _ = save_format(store, notation);
        }
        MenuAction::ToggleSeparators => {
            app.toggle_separators();
            let _ = save_separators(store, app.display_prefs().separators);
        }
        MenuAction::ClearGraph => {
            app.clear_graph();
            let msg = localizer.lookup("graph-cleared");
            app.set_result(&msg);
        }
        MenuAction::CopyPois => {
            let text = app
                .pois()
                .iter()
                .map(|p| format!("{} ({:.3}, {:.3})", poi_label(p.kind, localizer), p.x, p.y))
                .collect::<Vec<_>>()
                .join("\n");
            if text.is_empty() {
                let msg = localizer.lookup("guide-no-results");
                app.set_result(&msg);
            } else {
                osc52_copy(&text);
                let msg = localizer.lookup("poi-copied");
                app.set_result(&msg);
            }
        }
        MenuAction::OpenGuide => {
            app.guide_open();
            // The guide text loads from disk on demand (ADR-0053); the
            // binary carries none of it.
            app.guide_load(localizer.locale());
        }
        MenuAction::OpenKeyHelp => app.key_help_open(),
        MenuAction::BrowseConstants => app.constants_open(localizer),
    }
    false
}

/// What a left-button drag over the graph panel manipulates (ADR-0034).
#[derive(Clone, Copy)]
enum MouseDrag {
    Pan2D,
    Rotate3D,
    /// A grab-area drag (ADR-0060): the pointer went down on the
    /// keypad's top border (or the docked strip row) — drag down two
    /// rows to dock the keypad away, up two rows to restore, release
    /// on the bar without crossing either threshold to toggle.
    Keypad {
        start_row: u16,
        shown_at_start: bool,
    },
}

/// Handle one mouse event against the areas the last frame drew
/// (ADR-0034): menu bar clicks and popup item clicks, history picks,
/// keypad bank/cell clicks, and graph drags/wheel/double-clicks for 2D
/// pan-zoom and 3D orbit. Returns true when the event quits the TUI.
#[allow(clippy::too_many_lines)]
fn handle_mouse(
    app: &mut App,
    localizer: &mut Localizer,
    store: &DocStore<FsStore>,
    event: crossterm::event::MouseEvent,
    drag: &mut Option<(u16, u16, MouseDrag)>,
    last_click: &mut Option<(std::time::Instant, u16, u16)>,
) -> bool {
    use crossterm::event::MouseEventKind;
    let areas = app.areas;
    let inside = |r: ratatui::layout::Rect, col: u16, row: u16| {
        col >= r.x
            && col < r.x.saturating_add(r.width)
            && row >= r.y
            && row < r.y.saturating_add(r.height)
    };
    match event.kind {
        // The wheel zooms the graph (3D camera / 2D viewport) and
        // scrolls the guide pager.
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
            if areas.guide {
                app.guide_scroll(if matches!(event.kind, MouseEventKind::ScrollUp) {
                    -3
                } else {
                    3
                });
            } else if areas.key_help {
                app.key_help_scroll(if matches!(event.kind, MouseEventKind::ScrollUp) {
                    -3
                } else {
                    3
                });
            } else if areas.constants {
                app.constants_select(if matches!(event.kind, MouseEventKind::ScrollUp) {
                    -3
                } else {
                    3
                });
            } else if inside(areas.graph, event.column, event.row) && !app.surfaces().is_empty() {
                let factor = if matches!(event.kind, MouseEventKind::ScrollUp) {
                    0.9
                } else {
                    1.1
                };
                let camera = app.view().camera * factor;
                app.view_set_camera(camera);
            } else if inside(areas.graph, event.column, event.row) && !app.graph().is_empty() {
                let factor = if matches!(event.kind, MouseEventKind::ScrollUp) {
                    0.8
                } else {
                    1.25
                };
                app.graph2d_zoom(factor);
            }
            false
        }
        MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
            let (col, row) = (event.column, event.row);
            // 0. The user guide pager (ADR-0018) covers the screen: only
            // the table-of-contents rows act (ADR-0018 amendment) —
            // everything else scrolls by wheel and is inert to clicks.
            if areas.guide {
                for i in 0..areas.guide_toc_len {
                    if inside(areas.guide_toc[i], col, row) {
                        app.guide_jump(i);
                    }
                }
                return false;
            }
            // The key-help overlay (ADR-0039) covers the screen too; its
            // rows are informational, so a click is simply swallowed.
            if areas.key_help {
                return false;
            }
            // 1. The open popup wins: a click on an item activates it,
            //    a click on a rule is ignored, a click anywhere else
            //    closes the menu without acting (browser convention).
            if let Some((menu, popup)) = areas.popup {
                if inside(popup, col, row) {
                    let row_idx = row.saturating_sub(popup.y.saturating_add(1)) as usize;
                    if let Some(PopupRow::Item(i, _)) = menu_rows(app, localizer).get(row_idx) {
                        app.menu_select(menu, *i);
                        if let Some(action) = app.menu_activate() {
                            return perform_menu_action(app, store, localizer, action);
                        }
                    }
                    return false;
                }
                app.menu_close();
                return false;
            }
            // 2. The menu bar: click a label to open/close its menu.
            if row == areas.menu_labels[0].y {
                if let Some(i) = areas.menu_labels.iter().position(|r| inside(*r, col, row)) {
                    if app.menu_active().map(|(m, _)| m) == Some(i) {
                        app.menu_close();
                    } else {
                        app.menu_open(i);
                    }
                    return false;
                }
            }
            // 3. Panels. (Clicking outside the menu bar with a menu
            //    open is already handled above — menus were closed.)
            if inside(areas.input, col, row) {
                app.keypad_close();
                app.history_close();
                // A click inside the entry moves the caret to the
                // clicked (line, column) — the mouse spelling of the
                // Left/Right keys (ADR-0035 amendment). The click row is
                // mapped through the pane's scroll to the text line, and
                // the column through the pane's left border.
                let line = (row.saturating_sub(areas.input.y.saturating_add(1)) as usize)
                    + areas.input_scroll as usize;
                let col = col.saturating_sub(areas.input.x.saturating_add(1)) as usize;
                app.cursor_to(line, col);
                return false;
            }
            if inside(areas.history, col, row) {
                // The trash glyph in the title row (ADR-0041): clicking it
                // clears the history, the mouse spelling of Ctrl+L. The
                // glyph is two columns wide.
                if row == areas.history.y
                    && col >= areas.history_trash_col
                    && col < areas.history_trash_col + 2
                {
                    app.clear_history();
                    return false;
                }
                let content_row = row.saturating_sub(areas.history.y.saturating_add(1)) as usize;
                let display_row = content_row + areas.history_scroll as usize;
                if let Some(&display_idx) = app.hist_rows.get(display_row) {
                    if display_idx != usize::MAX {
                        app.hist_sel = display_idx;
                        if let Some(line) = app.history_pick_display(display_idx) {
                            app.set_input(&line);
                        }
                    }
                }
                return false;
            }
            if inside(areas.keypad, col, row) {
                // The grab area (ADR-0060): the panel's top border row
                // when shown, the whole strip row when docked away.
                if row == areas.keypad.y {
                    *drag = Some((
                        col,
                        row,
                        MouseDrag::Keypad {
                            start_row: row,
                            shown_at_start: app.keypad_shown(),
                        },
                    ));
                    return false;
                }
                // The bank label row.
                if row == areas.keypad.y.saturating_add(1) {
                    if let Some(b) = areas
                        .kp_bank_labels
                        .iter()
                        .position(|r| inside(*r, col, row))
                    {
                        app.keypad_select_bank(b);
                    }
                    return false;
                }
                // The grid rows: move the highlight there and apply
                // the key — the mouse spelling of the web's buttons.
                // "=" submits like the entry's Enter; every other cell
                // inserts its token.
                let grid_row = row.saturating_sub(areas.keypad.y.saturating_add(2)) as usize;
                if grid_row < 5 {
                    let cell_col = (col.saturating_sub(areas.keypad.x.saturating_add(1))
                        / areas.kp_cell_w.max(1)) as usize;
                    if cell_col < areas.kp_cols {
                        app.keypad_set(grid_row, cell_col);
                        if app.keypad_is_submit() {
                            let line = app.input().trim().to_string();
                            if let Some(code) = app.submit_line(&line, store, localizer) {
                                *localizer = Localizer::resolve(Some(&code), &[]);
                            }
                            app.clear_input();
                        } else {
                            app.keypad_insert();
                        }
                    }
                }
                return false;
            }
            if inside(areas.graph, col, row)
                && (!app.surfaces().is_empty() || !app.graph().is_empty())
            {
                // A double-click resets the view: 2D re-fits the samples,
                // 3D returns to the default pose.
                let now = std::time::Instant::now();
                let is_double = last_click
                    .map(|(t, lc, lr)| {
                        now.duration_since(t) < std::time::Duration::from_millis(500)
                            && lc.abs_diff(col) <= 1
                            && lr.abs_diff(row) <= 1
                    })
                    .unwrap_or(false);
                *last_click = Some((now, col, row));
                if is_double {
                    if !app.surfaces().is_empty() {
                        app.view_reset_pose();
                    } else {
                        app.graph2d_reset();
                    }
                    *drag = None;
                    return false;
                }
                *drag = Some((
                    col,
                    row,
                    if app.surfaces().is_empty() {
                        MouseDrag::Pan2D
                    } else {
                        MouseDrag::Rotate3D
                    },
                ));
                return false;
            }
            false
        }
        // Dragging the graph: 3D orbits with the pointer (the web's
        // sensitivity, 0.01 rad/cell), 2D pans so the plot follows it.
        MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
            let Some((last_col, last_row, kind)) = *drag else {
                return false;
            };
            let dx = event.column as f64 - last_col as f64;
            let dy = event.row as f64 - last_row as f64;
            match kind {
                // The grab drag (ADR-0060) measures from its start row,
                // not from the last event, so the drag position itself
                // is never updated. Two rows down docks the keypad away,
                // two rows up restores; the drag ends on the toggle so
                // continuing the motion never toggles twice — the
                // release arm owns the click case.
                MouseDrag::Keypad {
                    start_row,
                    shown_at_start,
                } => {
                    let dy = event.row as i32 - start_row as i32;
                    // Still in the state the gesture started from — a
                    // finished (already-toggled) drag is inert. The
                    // return skips the shared tail that re-arms the
                    // drag with the new position: a grab drag measures
                    // from its start row and ends when it fires.
                    let crossed = (shown_at_start && dy >= 2) || (!shown_at_start && dy <= -2);
                    if crossed && app.keypad_shown() == shown_at_start {
                        app.keypad_toggle();
                        *drag = None;
                    }
                    return false;
                }
                MouseDrag::Rotate3D => {
                    let v = *app.view();
                    app.view = v
                        .with_yaw(v.yaw + dx * 0.01)
                        .with_pitch(v.pitch + dy * 0.01);
                }
                MouseDrag::Pan2D => {
                    let (w, h) = graph_dims(areas.graph);
                    app.graph2d_pan(dx, dy, w, h);
                }
            }
            *drag = Some((event.column, event.row, kind));
            false
        }
        MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
            // Releasing a grab drag back on the grab row (no threshold
            // crossed) is a click: toggle (ADR-0060) — the mouse
            // spelling of the web bar's tap.
            if let Some((
                _,
                _,
                MouseDrag::Keypad {
                    start_row,
                    shown_at_start,
                },
            )) = *drag
            {
                if event.row == start_row && app.keypad_shown() == shown_at_start {
                    app.keypad_toggle();
                }
            }
            *drag = None;
            false
        }
        _ => false,
    }
}

/// One row of an open menu popup (ADR-0033): a plain selectable item
/// with its item-space index, or a labeled section rule.
enum PopupRow {
    Item(usize, String),
    Rule(String),
}

/// The open menu's rows: plain items plus labeled section rules.
/// Settings marks its subsections — Theme, Language, and the 3D View
/// controls — with dim "─ Label ────" dividers; the highlight and the
/// activation index stay in item space, so the rules never intercept
/// arrow movement, Enter, or a mouse click (ADR-0034 resolves clicks
/// through the same list).
fn menu_rows(app: &App, localizer: &Localizer) -> Vec<PopupRow> {
    let menu = app.menu_active().map(|(m, _)| m).unwrap_or(0);
    let mut rows: Vec<PopupRow> = Vec::new();
    match menu {
        0 => {
            for (i, label) in [
                localizer.lookup("menu-open-history"),
                localizer.lookup("menu-open-script"),
                localizer.lookup("menu-save-history"),
                localizer.lookup("menu-save-script"),
                localizer.lookup("menu-quit"),
            ]
            .into_iter()
            .enumerate()
            {
                rows.push(PopupRow::Item(i, label));
            }
        }
        1 => {
            for (i, label) in [
                localizer.lookup("menu-cut"),
                localizer.lookup("menu-copy"),
                localizer.lookup("menu-paste"),
            ]
            .into_iter()
            .enumerate()
            {
                rows.push(PopupRow::Item(i, label));
            }
        }
        2 => {
            rows.push(PopupRow::Item(0, localizer.lookup("graph-clear")));
            rows.push(PopupRow::Item(1, localizer.lookup("poi-copy")));
        }
        3 => {
            rows.push(PopupRow::Item(0, localizer.lookup("menu-guide")));
            rows.push(PopupRow::Item(1, localizer.lookup("menu-key-help")));
            rows.push(PopupRow::Item(2, localizer.lookup("menu-constants")));
        }
        _ => {
            rows.push(PopupRow::Item(0, localizer.lookup("graph-points")));
            rows.push(PopupRow::Rule(localizer.lookup("tui-settings-theme")));
            rows.push(PopupRow::Item(1, localizer.lookup("theme-light")));
            rows.push(PopupRow::Item(2, localizer.lookup("theme-dark")));
            rows.push(PopupRow::Item(3, localizer.lookup("theme-night")));
            rows.push(PopupRow::Rule(localizer.lookup("tui-settings-language")));
            for (i, code) in epher_i18n::SUPPORTED_LOCALES.iter().enumerate() {
                rows.push(PopupRow::Item(
                    4 + i,
                    native_language_name(code).to_string(),
                ));
            }
            // Result rendering (ADR-0043): exact fractions, the notation,
            // and thousands separators. The rows show the live choice.
            rows.push(PopupRow::Rule(localizer.lookup("tui-settings-results")));
            let on_off = |on: bool| {
                if on {
                    localizer.lookup("results-on")
                } else {
                    localizer.lookup("results-off")
                }
            };
            rows.push(PopupRow::Item(
                12,
                format!(
                    "{}: {}",
                    localizer.lookup("tui-settings-exact"),
                    on_off(app.display.exact_fractions)
                ),
            ));
            let notation = match app.display.notation {
                epher_core::Notation::Auto => localizer.lookup("results-auto"),
                epher_core::Notation::Scientific => localizer.lookup("results-scientific"),
                epher_core::Notation::Engineering => localizer.lookup("results-engineering"),
            };
            rows.push(PopupRow::Item(
                13,
                format!("{}: {notation}", localizer.lookup("tui-settings-notation")),
            ));
            rows.push(PopupRow::Item(
                14,
                format!(
                    "{}: {}",
                    localizer.lookup("tui-settings-separators"),
                    on_off(app.display.separators)
                ),
            ));
            // The 3D fine controls (ADR-0031) join the menu only while
            // surfaces are displayed, mirroring the web's sliders; their
            // rows show the live value.
            if !app.surfaces().is_empty() {
                rows.push(PopupRow::Rule(localizer.lookup("tui-settings-view")));
                let (h, vv, z) = app.view_offsets();
                rows.push(PopupRow::Item(
                    12,
                    format!("{}  {h:+.1}", localizer.lookup("view-horizontal")),
                ));
                rows.push(PopupRow::Item(
                    13,
                    format!("{}  {vv:+.1}", localizer.lookup("view-vertical")),
                ));
                rows.push(PopupRow::Item(
                    14,
                    format!("{}  {z:+.1}", localizer.lookup("view-zoom")),
                ));
            }
        }
    }
    rows
}

fn draw(frame: &mut ratatui::Frame, app: &mut App, localizer: &Localizer) {
    use ratatui::layout::{Constraint, Layout, Position, Rect};
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span, Text};
    use ratatui::widgets::{Block, Borders, Clear, Paragraph};
    // The mouse map (ADR-0034): every widget records where it drew so
    // clicks land on what the user saw.
    let mut areas = Areas::default();

    // The theme palette (ADR-0017): dark is the terminal's natural look,
    // light forces a light canvas, night stays in long-wavelength reds on
    // near-black. Contrasts: night text #ffb3a8 on #0d0000 = 12.1:1,
    // hints #d98878 = 7.6:1, selection 7.2:1; light black on white
    // 15.9:1, result #006e3c 5.9:1, selection white on #006e6e 4.9:1.
    let (screen_bg, fg, result_style, hints_style, sel_bg, sel_fg, border_fg) = match app.theme() {
        Theme::Light => (
            Some(Color::White),
            Color::Black,
            Style::default()
                .fg(Color::Rgb(0, 110, 60))
                .add_modifier(Modifier::BOLD),
            Style::default().fg(Color::DarkGray),
            Color::Rgb(0, 110, 110),
            Color::White,
            Color::Black,
        ),
        Theme::Night => (
            Some(Color::Rgb(13, 0, 0)),
            Color::Rgb(255, 179, 168),
            Style::default()
                .fg(Color::Rgb(255, 110, 96))
                .add_modifier(Modifier::BOLD),
            Style::default().fg(Color::Rgb(217, 136, 120)),
            Color::Rgb(255, 107, 90),
            Color::Rgb(26, 0, 0),
            Color::Rgb(170, 64, 51),
        ),
        Theme::Dark => (
            None,
            Color::Reset,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            Style::default().fg(Color::DarkGray),
            Color::Cyan,
            Color::Black,
            Color::Reset,
        ),
    };
    if let Some(bg) = screen_bg {
        frame.render_widget(
            Block::default().style(Style::default().bg(bg)),
            frame.area(),
        );
    }
    let border_style = Style::default().fg(border_fg);
    let block = |title: String| {
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title_style(border_style)
            .title(title)
    };

    // The user guide view (ADR-0018): a full-screen pager over the same
    // markdown the website guide pages and the web overlay are built
    // from, rendered in the current interface language. The table of
    // contents (ADR-0018 amendment) pins the top-level chapters above
    // the content: one row per chapter, clickable, and the number keys
    // 1–9 jump to the same rows.
    // The key-help overlay (ADR-0039): the current bank's keys with
    // their meanings, modal like the guide - it paints over the whole
    // screen and the draw returns, so nothing else leaks into the mouse
    // map this frame.
    if let Some(offset) = app.key_help_offset() {
        let rows = Layout::vertical([
            Constraint::Length(1), // title
            Constraint::Min(0),    // the key rows
            Constraint::Length(1), // scroll hint
        ])
        .split(frame.area());
        let title = format!(
            " {} — {} ",
            localizer.lookup("menu-key-help"),
            app.keypad_bank()
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                title,
                Style::default()
                    .bg(sel_bg)
                    .fg(sel_fg)
                    .add_modifier(Modifier::BOLD),
            ))),
            rows[0],
        );
        let bank = &BANKS[app.keypad_bank_index()].1;
        let lines: Vec<Line> = bank
            .iter()
            .flat_map(|row| row.iter())
            .map(|(disp, _)| {
                let text = match keypad_hint_key(disp) {
                    Some(key) => format!("{}  {}", disp, localizer.lookup(key)),
                    None => disp.to_string(),
                };
                Line::from(Span::styled(format!(" {}", text), Style::default().fg(fg)))
            })
            .collect::<Vec<_>>();
        // Clamp to the last page.
        let content_rows = rows[1].height as usize;
        let max = lines.len().saturating_sub(content_rows);
        let offset = offset.min(max);
        frame.render_widget(
            Paragraph::new(Text::from(lines))
                .style(Style::default().fg(fg))
                .scroll((offset as u16, 0)),
            rows[1],
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                localizer.lookup("tui-key-help-hint"),
                hints_style,
            ))),
            rows[2],
        );
        areas.key_help = true;
        app.key_help = Some(offset);
        app.areas = areas;
        return;
    }

    // The constants browser (ADR-0045): Help -> Constants. Grouped
    // rows (name, value, hint); the selection highlights and Enter
    // inserts the name into the input line. Renders like the key-help
    // overlay: full screen, title row, scrollable list, hint row.
    if app.constants_active() {
        let rows = Layout::vertical([
            Constraint::Length(1), // title
            Constraint::Min(0),    // the constant rows
            Constraint::Length(1), // scroll hint
        ])
        .split(frame.area());
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!(" {} ", localizer.lookup("menu-constants")),
                Style::default()
                    .bg(sel_bg)
                    .fg(sel_fg)
                    .add_modifier(Modifier::BOLD),
            ))),
            rows[0],
        );
        let content_rows = rows[1].height as usize;
        // The visible lines: group headings (bold, not selectable) and
        // constant rows (name, value, hint). Selection indexes into the
        // constant rows only, so the highlight needs the row offset of
        // the selected constant inside the drawn lines.
        let group_key = |g: epher_core::ConstGroup| match g {
            epher_core::ConstGroup::Math => "constants-group-math",
            epher_core::ConstGroup::Astronomy => "constants-group-astronomy",
            epher_core::ConstGroup::Physics => "constants-group-physics",
            epher_core::ConstGroup::Chemistry => "constants-group-chemistry",
        };
        let mut lines: Vec<(Line<'static>, bool)> = Vec::new();
        let mut const_index = 0usize;
        let mut sel_line = 0usize;
        for group in [
            epher_core::ConstGroup::Math,
            epher_core::ConstGroup::Astronomy,
            epher_core::ConstGroup::Physics,
            epher_core::ConstGroup::Chemistry,
        ] {
            let mut heading_drawn = false;
            for (name, value, _) in &app.constants_rows {
                let g = epher_core::builtin_constant_groups()
                    .iter()
                    .find(|(n, _)| n == name)
                    .map(|(_, g)| *g)
                    .unwrap_or(epher_core::ConstGroup::Math);
                if g != group {
                    continue;
                }
                if !heading_drawn {
                    lines.push((
                        Line::from(Span::styled(
                            format!(" {}", localizer.lookup(group_key(group))),
                            Style::default().fg(fg).add_modifier(Modifier::BOLD),
                        )),
                        false,
                    ));
                    heading_drawn = true;
                }
                let shown = match value {
                    Some(v) => {
                        epher_core::format_value(&epher_core::Value::float(*v), &app.display)
                    }
                    None => String::new(),
                };
                let text = if shown.is_empty() {
                    format!(" {name}")
                } else {
                    format!(" {name}   {shown}")
                };
                let is_sel = app.constants == Some(const_index);
                if is_sel {
                    sel_line = lines.len();
                }
                const_index += 1;
                let style = if is_sel {
                    Style::default().bg(sel_bg).fg(sel_fg)
                } else {
                    Style::default().fg(fg)
                };
                lines.push((Line::from(Span::styled(text, style)), true));
            }
        }
        // Clamp the scroll so the selection stays visible.
        let offset = {
            let max = lines.len().saturating_sub(content_rows);
            let want = sel_line.saturating_sub(content_rows / 2);
            want.min(max)
        };
        let mut text = Text::default();
        for (line, _) in lines.iter().skip(offset) {
            text.push_line(line.clone());
        }
        frame.render_widget(
            Paragraph::new(text)
                .style(Style::default().fg(fg))
                .scroll((0, 0)),
            rows[1],
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                localizer.lookup("tui-constants-hint"),
                hints_style,
            ))),
            rows[2],
        );
        areas.constants = true;
        app.areas = areas;
        return;
    }

    if let Some(offset) = app.guide_offset() {
        // The guide text (ADR-0053): loaded from the installed files at
        // open; when the install has none, the pager says where it
        // looked instead of a blank page.
        let (chapters, guide_lines) = match app.guide_md.as_ref().map(|(_, md)| md) {
            Some(Ok(md)) => (epher_guide::chapters(md), epher_guide::render_text(md)),
            _ => {
                let tried = app
                    .guide_md
                    .as_ref()
                    .and_then(|(_, r)| r.as_ref().err())
                    .map(|e| e.tried.clone())
                    .unwrap_or_default();
                let mut lines = vec![
                    epher_guide::TLine::Text(localizer.lookup("guide-unavailable")),
                    epher_guide::TLine::Blank,
                ];
                for dir in tried {
                    lines.push(epher_guide::TLine::Code(dir));
                }
                lines.push(epher_guide::TLine::Blank);
                lines.push(epher_guide::TLine::Text("https://epher.org/guide/".into()));
                (Vec::new(), lines)
            }
        };
        let toc_len = chapters.len().min(12);
        let rows = Layout::vertical([
            Constraint::Length(1),              // title
            Constraint::Length(toc_len as u16), // table of contents
            Constraint::Min(0),                 // content
            Constraint::Length(1),              // scroll hint
        ])
        .split(frame.area());
        let title = localizer.lookup("menu-guide");
        // While a search is typed, the title row becomes the query
        // strip: what is typed, how many hits, how to leave (the
        // ADR-0038 amendment's TUI spelling of the web search box).
        let title_text = if app.guide_searching() {
            let hits = app.guide_hit_rows().len();
            format!(
                " {title}  /{}  {hits}  Enter=next  Esc=back ",
                app.guide_query()
            )
        } else {
            format!(" {title}  (/ searches) ")
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                title_text,
                Style::default()
                    .bg(sel_bg)
                    .fg(sel_fg)
                    .add_modifier(Modifier::BOLD),
            ))),
            rows[0],
        );
        // The ToC rows: numbered chapter titles; their rects feed the
        // mouse (areas.guide_toc) and their jump targets come from the
        // wrapped-row offsets computed below.
        for (i, chapter) in chapters.iter().enumerate().take(toc_len) {
            areas.guide_toc[i] = rows[1];
            areas.guide_toc[i].y = rows[1].y + i as u16;
            areas.guide_toc[i].height = 1;
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!(" {} {}", i + 1, chapter),
                    Style::default().fg(fg),
                ))),
                areas.guide_toc[i],
            );
        }
        areas.guide_toc_len = toc_len;
        let mut lines = Vec::new();
        let mut chapters_found: Vec<(String, usize)> = Vec::new();
        for t in guide_lines {
            match t {
                epher_guide::TLine::Heading(level, text) => {
                    let style = if level == 1 {
                        Style::default()
                            .fg(fg)
                            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(fg).add_modifier(Modifier::BOLD)
                    };
                    lines.push((Line::from(Span::styled(text.clone(), style)), level));
                    if level == 2 {
                        chapters_found.push((text, 0));
                    }
                }
                epher_guide::TLine::Text(text) => {
                    lines.push((Line::from(Span::styled(text, Style::default().fg(fg))), 0));
                }
                epher_guide::TLine::Code(text) => {
                    lines.push((Line::from(Span::styled(text, hints_style)), 0));
                }
                epher_guide::TLine::Quote(text) => {
                    lines.push((Line::from(Span::styled(text, hints_style)), 0));
                }
                epher_guide::TLine::Blank => lines.push((Line::from(""), 0)),
            }
        }
        // The offset counts wrapped rows; chapter targets are the
        // wrapped rows their headings start at, for the ToC jumps.
        let content_width = rows[2].width as usize;
        let mut wrapped = 0usize;
        let mut found = 0usize;
        for (line, level) in &lines {
            if *level == 2 {
                if let Some(target) = chapters_found.get_mut(found) {
                    target.1 = wrapped;
                }
                found += 1;
            }
            wrapped += line.width().div_ceil(content_width.max(1)).max(1);
        }
        app.guide_chapters = chapters_found;
        // The search hits (ADR-0038 amendment): the wrapped row each
        // matching line starts at, for Enter-to-jump.
        app.guide_hit_rows = if app.guide_searching() && !app.guide_query().is_empty() {
            let query = app.guide_query().to_lowercase();
            let mut hit_rows = Vec::new();
            let mut at = 0usize;
            for (line, _) in &lines {
                let text: String = line.spans.iter().map(|s| s.content.to_string()).collect();
                if text.to_lowercase().contains(&query) {
                    hit_rows.push(at);
                }
                at += line.width().div_ceil(content_width.max(1)).max(1);
            }
            hit_rows
        } else {
            Vec::new()
        };
        let rendered: Vec<Line> = lines.into_iter().map(|(l, _)| l).collect();
        // Clamp to the last page.
        let content_rows = rows[2].height as usize;
        let max = wrapped.saturating_sub(content_rows);
        let offset = offset.min(max);
        frame.render_widget(
            Paragraph::new(Text::from(rendered))
                .style(Style::default().fg(fg))
                .wrap(ratatui::widgets::Wrap { trim: false })
                .scroll((offset as u16, 0)),
            rows[2],
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                localizer.lookup("guide-hint"),
                hints_style,
            ))),
            rows[3],
        );
        areas.guide = true;
        app.areas = areas;
        return;
    }

    // The menu bar row (ADR-0017): the labels render in [`App::MENUS`]
    // order - Help above Settings (the ADR-0038 amendment) - so the
    // clickable rects, the popup indices, and the visible bar can never
    // drift apart.
    let base = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(frame.area());
    let (menu_area, body) = (base[0], base[1]);
    areas.menu_labels = [menu_area; 5];

    let menu_labels: Vec<String> = App::MENUS
        .iter()
        .map(|m| localizer.lookup(&format!("menu-{m}")))
        .collect();
    let menu_labels = &menu_labels;
    let mut bar = Vec::new();
    let mut x = menu_area.x;
    for (i, label) in menu_labels.iter().enumerate() {
        let label = label.as_str();
        let open = app.menu_active().map(|(m, _)| m) == Some(i);
        let text = format!(" {} ", label);
        let style = if open {
            Style::default()
                .bg(sel_bg)
                .fg(sel_fg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(fg)
        };
        // The clickable label rect (ADR-0034): exactly the characters
        // this entry paints, in the localized menu order.
        areas.menu_labels[i] = Rect {
            x,
            y: menu_area.y,
            width: text.chars().count() as u16,
            height: 1,
        };
        x += text.chars().count() as u16 + 1;
        bar.push(Span::styled(text, style));
        bar.push(Span::raw(" "));
    }
    frame.render_widget(Paragraph::new(Line::from(bar)), menu_area);

    // The open menu's items drop down below their title. They are drawn
    // LAST, over every panel, with a Clear underneath: ratatui paints
    // widgets into one shared buffer in call order, so a menu rendered
    // before the history/graph would be painted over wherever those
    // panels overlap it — its items would vanish behind the screen's
    // existing text.
    // Wide terminals get the desktop layout (ADR-0017): the calculator
    // column on the left, the graph panel in its own section on the
    // right, and the key hints spanning the full width underneath both
    // (ADR-0019) — the panel used to run down over the hints row and
    // clip the key guide at the column edge. Below 72 columns only, the
    // vertical stack from ADR-0016 remains — one split, no overlapping
    // regions. The threshold moved from 104 to 72 (ADR-0025) so a
    // standard 80×24 terminal gets the same layout every platform does:
    // history below the answer, graph on the right.
    // The keypad is always part of the screen now (ADR-0033) — it used
    // to appear only while focused, which read as "lost". Tab just
    // moves the highlight onto it. The hint strip wraps to as many rows
    // as its text needs (two on a standard 80-column terminal) so the
    // whole key guide is visible, and the keypad's cell width comes
    // from its real pane so narrower terminals shrink the grid instead
    // of clipping it. Everything fits 80×24.
    let wide = body.width >= 72;
    // The hint strip is composed from parts (ADR-0035 amendment): the
    // arrow-key hint only names rotation while a 3D surface is displayed
    // and the space hint only names animation while an animatable 2D
    // graph is displayed — a hint must not advertise an affordance the
    // current plot does not offer.
    let has_surface = !app.surfaces().is_empty();
    let animatable_2d = !app.graph().is_empty() && app.curve_animated_constant().is_some();
    let hint_text = format!(
        "{}{}{}{}",
        localizer.lookup("tui-hint-base-a"),
        if has_surface {
            localizer.lookup("tui-hint-rotate")
        } else {
            String::new()
        },
        if animatable_2d {
            localizer.lookup("tui-hint-play")
        } else {
            String::new()
        },
        localizer.lookup("tui-hint-base-b")
    );
    let hint_rows = (hint_text.chars().count() as u16)
        .div_ceil(body.width.max(1))
        .clamp(1, 3);
    // The entry grows with the script being composed (ADR-0035
    // amendment): Shift+Enter starts new lines, and the pane shows up to
    // four content rows, scrolling to keep the caret line visible. The
    // file prompt stays one line (paths cannot contain newlines).
    let input_lines = if app.prompt_active().is_some() {
        1
    } else {
        app.input().chars().filter(|&c| c == '\n').count() + 1
    };
    let input_rows = input_lines.min(4);
    let input_h = 2 + input_rows as u16;
    // The answer line keeps only what reads well on one line (ADR-0056):
    // a short single answer. Anything longer — a script's transcript, a
    // table, a long number — renders in the result pane, one answer per
    // line, by the same rule the web app routes with. The answer area
    // keeps one row while its text is away in the pane, so the layout
    // does not jump when a long answer arrives; the pane now carries
    // ADR-0052's every-answer-visible contract for transcripts.
    let long_answer = !answer_fits(app.result(), wide);
    let result_rows = if long_answer {
        1
    } else {
        (app.result().chars().filter(|&c| c == '\n').count() + 1).min(6)
    };
    let result_h = result_rows as u16;
    // The keypad's docked state (ADR-0060): shown, the panel is the
    // bank row plus the digits bank's five key rows; docked away, one
    // grab strip remains where its top border sat, and the history —
    // the Min(0) sibling — grows into the freed rows.
    let keypad_h = if app.keypad_shown() { 8 } else { 1 };
    let (input_area, result_area, history_area, graph_area, keypad_area, hints_area) = if wide {
        let split =
            Layout::vertical([Constraint::Min(0), Constraint::Length(hint_rows)]).split(body);
        let (content, hints) = (split[0], split[1]);
        let split = Layout::horizontal([Constraint::Length(46), Constraint::Min(0)]).split(content);
        let (calc_col, graph_col) = (split[0], split[1]);
        // Input, answer, history, then the keypad — the calculator
        // column reads top to bottom exactly like the app and the PWA
        // (entry, result, history, keypad).
        let calc_rows = Layout::vertical([
            Constraint::Length(input_h),  // input (grows with the script)
            Constraint::Length(result_h), // result (grows with the transcript)
            Constraint::Min(0),           // history
            Constraint::Length(keypad_h), // keypad, or its grab strip (ADR-0060)
        ])
        .split(calc_col);
        (
            calc_rows[0],
            calc_rows[1],
            calc_rows[2],
            graph_col,
            Some(calc_rows[3]),
            hints,
        )
    } else {
        // The narrow stack (ADR-0016): input, answer, then history and
        // graph sharing what is left, then the always-visible keypad
        // (or its grab strip, ADR-0060) and the wrapped hints.
        let rows = Layout::vertical([
            Constraint::Length(input_h),   // input (grows with the script)
            Constraint::Length(result_h),  // result (grows with the transcript)
            Constraint::Min(0),            // history
            Constraint::Min(0),            // graph
            Constraint::Length(keypad_h),  // keypad, or its grab strip (ADR-0060)
            Constraint::Length(hint_rows), // hints
        ])
        .split(body);
        (rows[0], rows[1], rows[2], rows[3], Some(rows[4]), rows[5])
    };

    // The input row doubles as the file prompt (ADR-0017).
    let (input_title, input_text) = match app.prompt_active() {
        Some((PromptKind::OpenHistory, buf)) => {
            (localizer.lookup("tui-open-prompt"), buf.to_string())
        }
        Some((PromptKind::OpenScript, buf)) => {
            (localizer.lookup("tui-open-prompt"), buf.to_string())
        }
        Some((PromptKind::SaveHistory, buf)) => {
            (localizer.lookup("tui-save-prompt"), buf.to_string())
        }
        Some((PromptKind::SaveScript, buf)) => {
            (localizer.lookup("tui-save-prompt"), buf.to_string())
        }
        None => (localizer.lookup("tui-expression"), app.input().to_string()),
    };
    // Record every panel's rect for the mouse (ADR-0034). The input's
    // scroll (caret-line keep-in-view) is recorded too, so a click can
    // map its row back to the text's line.
    let cursor_line = app.cursor_line_index();
    let input_scroll = cursor_line.saturating_sub(input_rows.saturating_sub(1)) as u16;
    areas.input = input_area;
    areas.input_scroll = input_scroll;
    areas.result = result_area;
    areas.history = history_area;
    areas.graph = graph_area;
    areas.hints = hints_area;
    if let Some(kp) = keypad_area {
        areas.keypad = kp;
    }
    let input = Paragraph::new(input_text.clone())
        .style(Style::default().fg(fg))
        .scroll((input_scroll, 0))
        .block(block(input_title));
    frame.render_widget(input, input_area);

    // A long answer empties the answer line — its transcript renders in
    // the result pane instead (ADR-0056).
    let answer_line = if long_answer {
        String::new()
    } else {
        app.result().to_string()
    };
    let result = Paragraph::new(answer_line).style(result_style);
    frame.render_widget(result, result_area);

    // History (ADR-0027): entries render newest first, one row per line
    // of the entry, with a full-width rule between entries — the visible
    // boundary that marks where one item ends and the next begins
    // (ADR-0027 amendment: a multi-line script is one item occupying
    // several rows between two rules). `hist_rows` maps each displayed
    // row to the displayed entry that owns it, for the mouse.
    let mut history_lines: Vec<Line> = Vec::new();
    let mut hist_rows: Vec<usize> = Vec::new();
    let rule = "─".repeat(history_area.width.saturating_sub(2) as usize);
    let rule_style = Style::default().fg(border_fg);
    for (display_idx, h) in app.history().iter().rev().enumerate() {
        if display_idx > 0 {
            history_lines.push(Line::from(Span::styled(rule.clone(), rule_style)));
            hist_rows.push(usize::MAX);
        }
        let selected = app.history_focused() && display_idx == app.history_sel();
        for row in h.split('\n') {
            let line = Line::from(row.to_string());
            // History focus (ADR-0027): every row of the selected entry
            // is highlighted with the theme's selection colors.
            history_lines.push(if selected {
                line.style(Style::default().fg(sel_fg).bg(sel_bg))
            } else {
                line
            });
            hist_rows.push(display_idx);
        }
    }
    app.hist_rows = hist_rows;
    // Keep the selection in view while the history has focus: the
    // paragraph scrolls (in rows) so the selected entry is visible — its
    // last row sits at the bottom edge when it fits, its first row at
    // the top when it is taller than the viewport.
    let history_scroll = if app.history_focused() {
        let visible = history_area.height.saturating_sub(2) as usize;
        let total_rows = app.hist_rows.len();
        let sel_top = app
            .hist_rows
            .iter()
            .take_while(|&&i| i != usize::MAX)
            .position(|&i| i == app.history_sel())
            .unwrap_or(0);
        let sel_height = app
            .hist_rows
            .iter()
            .filter(|&&i| i == app.history_sel())
            .count()
            .max(1);
        let scroll = sel_top.saturating_sub(visible.saturating_sub(sel_height.min(visible)));
        scroll.min(total_rows.saturating_sub(visible)) as u16
    } else {
        0
    };
    areas.history_scroll = history_scroll;
    // The trash glyph rides in the title, right after the name
    // (ADR-0041): the web pane's heading-side icon, spelled for a
    // terminal. Its columns are recorded so a mouse click on it clears
    // the history, like Ctrl+L.
    let history_title = format!("{} \u{1f5d1}", localizer.lookup("tui-history"));
    // The glyph sits after the localized name and one space; unicode
    // width decides the columns (the emoji is double-width).
    let trash_col = areas.history.x
        + 1
        + <str as UnicodeWidthStr>::width(localizer.lookup("tui-history").as_ref()) as u16
        + 1;
    areas.history_trash_col = trash_col;
    let history = Paragraph::new(history_lines)
        .style(Style::default().fg(fg))
        .scroll((history_scroll, 0))
        .block(block(history_title));
    frame.render_widget(history, history_area);

    // The keypad grid (ADR-0016): bank tabs on the first row — Tab
    // cycles them — and the highlighted cell inserts its token.
    if let Some(kp_area) = keypad_area {
        if !app.keypad_shown() {
            // Docked away (ADR-0060): the strip row is the whole grab
            // area — the same three bold dots the shown border carries,
            // at the same columns, so the handle never shifts.
            let buf = frame.buffer_mut();
            let (cy, cx) = (kp_area.y, kp_area.x + kp_area.width / 2);
            for dx in [-2i16, -1, 0] {
                let col = cx as i16 + dx;
                if col > kp_area.x as i16 && col < (kp_area.x + kp_area.width - 1) as i16 {
                    if let Some(cell) = buf.cell_mut((col as u16, cy)) {
                        cell.set_char('·')
                            .set_style(Style::default().fg(border_fg).add_modifier(Modifier::BOLD));
                    }
                }
            }
        } else {
            let bank = &BANKS[app.keypad_bank_index()].1;
            let cols = bank.iter().map(|r| r.len()).max().unwrap_or(1);
            // Cell width from the keypad's actual pane width: in the 46-wide
            // calc pane 5 columns → 8-wide cells, 4 columns → 11 (enough for
            // `variance`); narrower terminals shrink cells down to 6 so the
            // grid always fits its pane.
            let cell = ((kp_area.width.saturating_sub(2)) as usize / cols).clamp(6, 11);
            areas.kp_cell_w = cell as u16;
            areas.kp_cols = cols;
            let mut bx = kp_area.x + 1;
            for (b, (label, _)) in BANKS.iter().enumerate() {
                let text = format!(" {label} ");
                areas.kp_bank_labels[b] = Rect {
                    x: bx,
                    y: kp_area.y + 1,
                    width: text.chars().count() as u16,
                    height: 1,
                };
                bx += text.chars().count() as u16;
            }
            let bank_line = Line::from(
                BANKS
                    .iter()
                    .enumerate()
                    .map(|(b, (label, _))| {
                        let selected = b == app.keypad_bank_index();
                        let style = if selected {
                            Style::default()
                                .bg(sel_bg)
                                .fg(sel_fg)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(fg)
                        };
                        Span::styled(format!(" {label} "), style)
                    })
                    .collect::<Vec<Span>>(),
            );
            let rows: Vec<Line> = bank
                .iter()
                .enumerate()
                .map(|(r, row)| {
                    let cells: Vec<Span> = row
                        .iter()
                        .enumerate()
                        .map(|(c, (disp, _))| {
                            let selected = r == app.keypad_row() && c == app.keypad_col();
                            let style = if selected {
                                Style::default()
                                    .bg(sel_bg)
                                    .fg(sel_fg)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(fg)
                            };
                            Span::styled(format!(" {:<width$}", disp, width = cell - 1), style)
                        })
                        .collect();
                    Line::from(cells)
                })
                .collect();
            let mut grid = vec![bank_line];
            grid.extend(rows);
            let keypad = Paragraph::new(Text::from(grid))
                .style(Style::default().fg(fg))
                .block(block(if app.keypad_focused() {
                    localizer.lookup("tui-keypad-active")
                } else {
                    localizer.lookup("tui-keypad")
                }));
            frame.render_widget(keypad, kp_area);
            // The grab area (ADR-0060): three bold dots replacing the top
            // border at the panel's center — the handle the mouse drags
            // down to dock the keypad away. Same glyph the docked strip
            // shows, so the affordance reads as the same thing.
            let buf = frame.buffer_mut();
            let cy = kp_area.y;
            let cx = kp_area.x + kp_area.width / 2;
            for dx in [-2i16, -1, 0] {
                let col = cx as i16 + dx;
                if col > kp_area.x as i16 && col < (kp_area.x + kp_area.width - 1) as i16 {
                    if let Some(cell) = buf.cell_mut((col as u16, cy)) {
                        cell.set_char('·')
                            .set_style(Style::default().fg(border_fg).add_modifier(Modifier::BOLD));
                    }
                }
            }
        }
    }

    let hints = Paragraph::new(hint_text)
        .style(hints_style)
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(hints, hints_area);

    // A long answer renders in the result pane (ADR-0056), one answer
    // per line, above any curves — the pane gives the transcript the
    // room the answer line cannot.
    let mut graph_text = String::new();
    let mut transcript_rows = 0u16;
    if long_answer {
        graph_text.push_str(app.result());
        graph_text.push('\n');
        transcript_rows = app.result().lines().count() as u16 + 1;
    }
    // The plot shares the pane with the transcript: it draws below it,
    // sized to the rows the transcript leaves.
    let plot_dims_area = Rect {
        height: graph_area.height.saturating_sub(transcript_rows),
        ..graph_area
    };
    let curves = app.graph();
    if let Some(scene) = app.solar() {
        let _ = curves;
        let legend: Vec<String> = scene
            .dots
            .iter()
            .map(|d| epher_core::astro::body_name(d.body).to_string())
            .collect();
        graph_text.push_str(&legend.join("   "));
        graph_text.push('\n');
        let (w, h) = graph_dims(plot_dims_area);
        graph_text.push_str(&render_solar_ascii(scene, &app.effective_view(), w, h));
    } else if let Some(data) = app.data() {
        let _ = curves;
        graph_text.push_str(data.source.trim());
        graph_text.push('\n');
        let (w, h) = graph_dims(plot_dims_area);
        graph_text.push_str(&render_ascii_data(data, w, h));
    } else if !app.curve3ds().is_empty() {
        // 3D parametric curves (ADR-0054): the surface pane's pose,
        // legend row, and depth-shaded wireframe treatment.
        let legend: Vec<String> = app
            .curve3ds()
            .iter()
            .map(|c| format!("param {}", c.source.trim()))
            .collect();
        graph_text.push_str(&legend.join("   "));
        graph_text.push('\n');
        let (w, h) = graph_dims(plot_dims_area);
        graph_text.push_str(&render_ascii3d_curves(
            app.curve3ds(),
            &app.effective_view(),
            w,
            h,
        ));
    } else if !app.surfaces().is_empty() {
        let legend: Vec<String> = app
            .surfaces()
            .iter()
            .map(|s| format!("z = {}", s.source.trim()))
            .collect();
        graph_text.push_str(&legend.join("   "));
        graph_text.push('\n');
        let (w, h) = graph_dims(plot_dims_area);
        graph_text.push_str(&render_ascii3d(app.surfaces(), &app.effective_view(), w, h));
    } else if !curves.is_empty() {
        let legend: Vec<String> = curves
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let glyph = ['o', 'x', '+', '*'][i % 4];
                let caption = match &c.kind {
                    epher_core::graph::CurveKind::Cartesian(_) => {
                        format!("y = {}", c.source.trim())
                    }
                    _ => c.source.trim().to_string(),
                };
                format!("{glyph} {caption}")
            })
            .collect();
        graph_text.push_str(&legend.join("   "));
        graph_text.push('\n');
        let (w, h) = graph_dims(plot_dims_area);
        graph_text.push_str(&render_ascii(curves, w, h, app.graph2d_effective()));
        let poi_lines: Vec<String> = app
            .pois()
            .iter()
            .take(2)
            .map(|p| format!("{} ({:.3}, {:.3})", poi_label(p.kind, localizer), p.x, p.y))
            .collect();
        if !poi_lines.is_empty() && app.poi_list() {
            graph_text.push('\n');
            graph_text.push_str(&poi_lines.join("   "));
        }
    }
    // The plot pane shares the web app's pane name (ADR-0056): the
    // localized "Result" — graphs and long answers live in the same
    // pane there, and the terminal calls it the same thing.
    let graph = Paragraph::new(graph_text)
        .style(Style::default().fg(fg))
        .block(block(localizer.lookup("result-pane")));
    frame.render_widget(graph, graph_area);

    // The open menu popup goes on top of everything else: draw it last,
    // after all panels, and clear its region first so whatever sat
    // underneath (history lines, graph text) cannot bleed through.
    if let Some((menu, item)) = app.menu_active() {
        let rows = menu_rows(app, localizer);
        let x = 11 * menu as u16 + 1;
        // +5: two for the border, two for the Settings check mark, one
        // spare — the fine-control rows' labels and values (ADR-0031)
        // must never clip at the right border.
        let w = 26u16.max(
            rows.iter()
                .map(|r| match r {
                    PopupRow::Item(_, s) | PopupRow::Rule(s) => s.chars().count() as u16 + 5,
                })
                .max()
                .unwrap_or(10),
        );
        let mut lines: Vec<Line> = Vec::new();
        for row in &rows {
            match row {
                PopupRow::Rule(label) => {
                    // A section divider: "─ Label " plus a dim rule
                    // filling the popup's width.
                    let fill = w
                        .saturating_sub(2)
                        .saturating_sub(label.chars().count() as u16 + 3);
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("─ {label} "),
                            hints_style.add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("─".repeat(fill as usize), hints_style),
                    ]));
                }
                PopupRow::Item(i, label) => {
                    let checked = menu == 3 && item_checked(app, *i, localizer.locale());
                    let mark = match (checked, menu) {
                        (true, _) => "\u{2713} ",
                        (false, 3) => "  ",
                        _ => "",
                    };
                    let text = format!("{mark}{label}");
                    if *i == item {
                        lines.push(Line::from(Span::styled(
                            text,
                            Style::default().bg(sel_bg).fg(sel_fg),
                        )));
                    } else {
                        lines.push(Line::from(Span::styled(text, Style::default().fg(fg))));
                    }
                }
            }
        }
        let h = lines.len() as u16 + 2;
        let popup = Rect {
            x: menu_area.x + x,
            y: menu_area.y + 1,
            width: w.min(frame.area().right().saturating_sub(menu_area.x + x)),
            height: h.min(frame.area().bottom().saturating_sub(menu_area.y + 1)),
        };
        frame.render_widget(Clear, popup);
        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            ),
            popup,
        );
        // The mouse map (ADR-0034): clicks inside the popup resolve
        // through the same row list the popup drew.
        areas.popup = Some((menu, popup));
    }
    app.areas = areas;

    // Focus visible: the terminal cursor sits at the caret — the
    // insertion point (ADR-0035 amendment), which arrow keys and mouse
    // clicks move — inside the visible (scrolled) part of the entry.
    let caret_line = cursor_line.saturating_sub(input_scroll as usize);
    let at = app.cursor();
    let line_start = input_text[..at].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let caret_col = UnicodeWidthStr::width(&input_text[line_start..at]) as u16;
    let x = input_area
        .x
        .saturating_add(1)
        .saturating_add(caret_col)
        .min(input_area.right().saturating_sub(2));
    let y = input_area
        .y
        .saturating_add(1)
        .saturating_add(caret_line as u16);
    frame.set_cursor_position(Position::new(x, y));
}

/// The ASCII plot size: the graph panel's own dimensions (the renderer
/// scales to them) — wide, narrow, and keypad variants all share it.
/// True when the answer line keeps a result (ADR-0056): exactly one
/// answer, no line breaks, and short enough to read without scrolling.
/// Anything longer — a pasted script's transcript, a table, a long
/// number — renders in the result pane instead, one answer per line.
/// The terminal routes by the same rule as the web; the TUI joins a
/// script's answers with newlines (the web joins with a private
/// separator), so the newline test covers both multi-answer
/// transcripts and multiline tables.
pub fn answer_fits(text: &str, wide: bool) -> bool {
    if text.is_empty() || text.contains('\n') {
        return false;
    }
    // The web's conservative caps (ADR-0056): about 44 monospace
    // characters fit the wide layout's calculator column (46 columns
    // minus its borders), about 24 fit the narrow stack's phone-like
    // answer line. A borderline answer moving to the result pane is a
    // smaller fault than one the answer line clips.
    let cap = if wide { 44 } else { 24 };
    text.chars().count() <= cap
}

fn graph_dims(graph_area: ratatui::layout::Rect) -> (usize, usize) {
    let w = graph_area.width.saturating_sub(2) as usize;
    let h = graph_area.height.saturating_sub(4) as usize;
    (w.max(20), h.max(3))
}

/// Which Settings radio is checked: the theme item index (0 light,
/// 1 dark, 2 night); languages are items 3..10 and map through
/// SUPPORTED_LOCALES.
/// Which Settings item is checked, if any: item 0 is the POI-list
/// checkbox (ADR-0019), items 1–3 the theme radios, items 4–11 the
/// language radios.
fn item_checked(app: &App, item: usize, locale: &str) -> bool {
    match item {
        0 => app.poi_list(),
        1..=3 => match app.theme() {
            Theme::Light => item == 1,
            Theme::Dark => item == 2,
            Theme::Night => item == 3,
        },
        _ => {
            let index = item - 4; // SUPPORTED_LOCALES order, as in menu_activate
            epher_i18n::SUPPORTED_LOCALES
                .get(index)
                .map(|code| *code == locale)
                .unwrap_or(false)
        }
    }
}

/// Which fine-control axis a Settings-menu row highlights (ADR-0031):
/// rows 12–14 are horizontal rotation, vertical rotation, zoom.
fn view_axis_of(item: usize) -> ViewAxis {
    match item {
        13 => ViewAxis::Vertical,
        14 => ViewAxis::Zoom,
        _ => ViewAxis::Horizontal,
    }
}

/// The name of a language in itself — the TUI menu lists languages the
/// way their speakers write them, independent of the UI language.
fn native_language_name(code: &str) -> &str {
    match code {
        "en" => "English",
        "zh-CN" => "\u{7b80}\u{4f53}\u{4e2d}\u{6587}",
        "hi" => "\u{939}\u{93f}\u{928}\u{94d}\u{926}\u{940}",
        "es" => "Espa\u{f1}ol",
        "fr" => "Fran\u{e7}ais",
        "ar" => "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629}",
        "de" => "Deutsch",
        "pt" => "Portugu\u{ea}s",
        _ => code,
    }
}

/// Copy text to the terminal's clipboard via OSC 52 (ADR-0017): the
/// escape sequence every mainstream terminal honors, remote sessions
/// included. Written raw between ratatui frames.
fn osc52_copy(text: &str) {
    use std::io::Write;
    let encoded = base64(text.as_bytes());
    print!("\x1b]52;c;{encoded}\x07");
    let _ = std::io::stdout().flush();
}

/// Minimal RFC 4648 base64 encoder (with padding): OSC 52 payloads must
/// be base64, and this avoids a dependency for twenty lines of math.
fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// The base64 payload for an OSC 52 copy (ADR-0017) — public for tests.
pub fn base64_for_osc52(bytes: &[u8]) -> String {
    base64(bytes)
}

#[cfg(test)]
mod draw_tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;

    /// ADR-0056 (amended): a long answer — one line, but over the
    /// answer line's calm cap — renders in the result pane on the
    /// right, and the answer line under the entry empties. Exactly the
    /// web app's routing.
    #[test]
    fn a_long_answer_moves_to_the_result_pane() {
        let mut app = App::default();
        let long = format!("= {}", "3".repeat(60));
        app.set_result(&long);
        let localizer = Localizer::resolve(None, &[]);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| draw(frame, &mut app, &localizer))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();

        let mut in_answer_column = false;
        let mut in_pane = false;
        for y in 1..24 {
            let mut calc_row = String::new();
            let mut pane_row = String::new();
            for x in 0..46 {
                calc_row.push_str(buffer.cell((x, y)).unwrap().symbol());
            }
            for x in 46..80 {
                pane_row.push_str(buffer.cell((x, y)).unwrap().symbol());
            }
            // The pane is 32 columns wide inside its borders: a 60-digit
            // answer clips, so look for its head.
            if pane_row.contains(&"3".repeat(20)) {
                in_pane = true;
            }
            if calc_row.contains(&"3".repeat(10)) {
                in_answer_column = true;
            }
        }
        assert!(in_pane, "the long answer must render in the result pane");
        assert!(
            !in_answer_column,
            "the answer line must not clip a long answer"
        );
    }

    /// ADR-0056: a short single answer stays where it has always been,
    /// in the answer line under the entry — and out of the result pane.
    #[test]
    fn a_short_answer_stays_on_the_answer_line() {
        let mut app = App::default();
        app.set_result("= 42");
        let localizer = Localizer::resolve(None, &[]);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| draw(frame, &mut app, &localizer))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();

        let mut answer_line = false;
        let mut pane_has_it = false;
        for y in 1..24 {
            let mut calc_row = String::new();
            let mut pane_row = String::new();
            for x in 0..46 {
                calc_row.push_str(buffer.cell((x, y)).unwrap().symbol());
            }
            for x in 46..80 {
                pane_row.push_str(buffer.cell((x, y)).unwrap().symbol());
            }
            if calc_row.contains("= 42") {
                answer_line = true;
            }
            if pane_row.contains("= 42") {
                pane_has_it = true;
            }
        }
        assert!(answer_line, "the short answer must stay on the answer line");
        assert!(
            !pane_has_it,
            "the result pane must stay out of short answers"
        );
    }

    /// ADR-0056: a multi-answer transcript renders in the result pane,
    /// one answer per line, never joined on the answer line.
    #[test]
    fn a_transcript_renders_in_the_pane_one_answer_per_line() {
        let mut app = App::default();
        app.set_result(
            "= 111
= 222",
        );
        let localizer = Localizer::resolve(None, &[]);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| draw(frame, &mut app, &localizer))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();

        let (mut first, mut second) = (None, None);
        for y in 1..24 {
            let mut pane_row = String::new();
            for x in 46..80 {
                pane_row.push_str(buffer.cell((x, y)).unwrap().symbol());
            }
            if first.is_none() && pane_row.contains("= 111") {
                first = Some(y);
            }
            if pane_row.contains("= 222") {
                second = Some(y);
            }
        }
        let (first, second) = (
            first.expect("the first answer must render in the pane"),
            second.expect("the second answer must render in the pane"),
        );
        assert!(
            second > first,
            "the transcript reads one answer per line, in order"
        );
    }

    /// ADR-0056: answers and plots share the pane without ceremony — a
    /// transcript renders above any curves.
    #[test]
    fn a_transcript_renders_above_the_curves() {
        let mut app = App::default();
        app.submit_graph("x").unwrap();
        let long = format!("= {}", "7".repeat(60));
        app.set_result(&long);
        let localizer = Localizer::resolve(None, &[]);

        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| draw(frame, &mut app, &localizer))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();

        let (mut marker_y, mut legend_y) = (None, None);
        for y in 1..30 {
            let mut pane_row = String::new();
            for x in 46..80 {
                pane_row.push_str(buffer.cell((x, y)).unwrap().symbol());
            }
            if marker_y.is_none() && pane_row.contains(&"7".repeat(20)) {
                marker_y = Some(y);
            }
            if legend_y.is_none() && pane_row.contains("o y = x") {
                legend_y = Some(y);
            }
        }
        let (marker_y, legend_y) = (
            marker_y.expect("the transcript must render in the pane"),
            legend_y.expect("the curve legend must still render in the pane"),
        );
        assert!(
            marker_y < legend_y,
            "the transcript renders above the curves"
        );
    }

    /// The menu popup must paint over the screen's existing content, not
    /// behind it: ratatui renders into one buffer in call order, and the
    /// popup used to be drawn before the history panel — wherever the two
    /// overlapped, history lines bled through and hid the menu items.
    #[test]
    fn open_menu_covers_existing_history_text() {
        let history: Vec<String> = (0..30)
            .map(|i| format!("HISTORY-MARKER-{i:02} padded line"))
            .collect();
        let mut app = App::with_session(Session::with_history(history));
        app.menu_open(0);
        let localizer = Localizer::resolve(None, &[]);

        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| draw(frame, &mut app, &localizer))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();

        // The File menu drops down at column 1, one row under the menu
        // bar, and is 26 wide × (5 items + borders) tall.
        let popup = Rect {
            x: 1,
            y: 1,
            width: 26,
            height: 7,
        };
        let mut saw_quit = false;
        let mut leaked = false;
        for y in popup.y..popup.y + popup.height {
            let mut row = String::new();
            for x in popup.x..popup.x + popup.width {
                row.push_str(buffer.cell((x, y)).unwrap().symbol());
            }
            if row.contains("Quit") {
                saw_quit = true;
            }
            if row.contains("HISTORY-MARKER") {
                leaked = true;
            }
        }
        assert!(
            saw_quit,
            "the File menu's Quit item must be visible in the popup"
        );
        assert!(!leaked, "history text must not bleed into the open menu");
    }

    /// ADR-0025: a standard 80×24 terminal gets the same two-column layout
    /// as any wider one — history below the answer, graph pane on the
    /// right. The old 104-column threshold (and the stacked layout's fixed
    /// 20-row graph) hid the history section entirely at this size.
    #[test]
    fn eighty_column_terminal_shows_history_and_graph_on_the_right() {
        let mut app = App::with_session(Session::with_history(vec!["2 + 2  = 4".to_string()]));
        app.set_result("= 4");
        app.submit_graph("x").unwrap();
        let localizer = Localizer::resolve(None, &[]);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| draw(frame, &mut app, &localizer))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();

        // The graph pane sits to the right of the 46-column calculator
        // column: its title cell starts at column 47 on row 1 (menu row 0).
        let mut saw_graph_title = false;
        for x in 46..80 {
            let cell = buffer.cell((x, 1)).unwrap().symbol();
            if !cell.trim().is_empty() {
                saw_graph_title = true;
            }
        }
        assert!(saw_graph_title, "the graph pane must occupy the right side");

        // History renders below the answer: the seeded line appears in the
        // calculator column below the result row.
        let mut saw_history = false;
        for y in 5..24 {
            let mut row = String::new();
            for x in 0..46 {
                row.push_str(buffer.cell((x, y)).unwrap().symbol());
            }
            if row.contains("2 + 2  = 4") {
                saw_history = true;
            }
        }
        assert!(saw_history, "history must be visible below the answer");
    }

    /// ADR-0045: the constants browser paints a full-screen grouped
    /// list with the selection highlighted and the hint row at the
    /// bottom, like the key-help overlay.
    #[test]
    fn constants_browser_renders_groups_and_selection() {
        let mut app = App::default();
        let localizer = Localizer::resolve(Some("en"), &[]);
        app.constants_open(&localizer);
        app.constants_select(1); // move off the first row

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| draw(frame, &mut app, &localizer))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();

        // The title row names the browser.
        let title: String = (0..30)
            .map(|x| buffer.cell((x, 0)).unwrap().symbol().to_string())
            .collect();
        assert!(title.contains("Constants"), "title row: {title}");
        // The four group headings appear and the first constant row is
        // e (Euler's number) with its value column.
        let mut text = String::new();
        for y in 1..24 {
            for x in 0..80 {
                text.push_str(buffer.cell((x, y)).unwrap().symbol());
            }
            text.push('\n');
        }
        // The first two groups fit the screen; the selection's group
        // scrolls the later ones into view.
        assert!(text.contains("Math"), "first group: {text}");
        assert!(text.contains("Astronomy"), "second group: {text}");
        assert!(text.contains(" e "), "first constant row: {text}");
        assert!(text.contains("2.71828"), "the value column shows e");
        assert!(text.contains("choose"), "the hint row mentions choosing");

        // Jump near the end: the tail groups appear and the selection
        // moves with the scroll.
        app.constants_select(100);
        terminal
            .draw(|frame| draw(frame, &mut app, &localizer))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        let mut text = String::new();
        for y in 1..24 {
            for x in 0..80 {
                text.push_str(buffer.cell((x, y)).unwrap().symbol());
            }
            text.push('\n');
        }
        assert!(text.contains("Chemistry"), "tail groups: {text}");
        assert!(text.contains("n_a"), "Avogadro row near the end");
    }
}

/// The keypad's docked state (ADR-0060): the grab-bar toggle, the
/// space it hands to history, the strip that remains, and the mouse
/// drag that docks and restores.
#[cfg(test)]
mod keypad_dock_tests {
    use super::*;
    use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn tui_store() -> DocStore<FsStore> {
        let dir = std::env::temp_dir().join(format!("epher-tui-kp-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        DocStore::new(FsStore::new(dir))
    }

    fn draw_once(app: &mut App, localizer: &Localizer, w: u16, h: u16) -> ratatui::buffer::Buffer {
        let backend = TestBackend::new(w, h);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, app, localizer)).unwrap();
        terminal.backend().buffer().clone()
    }

    fn mouse(kind: MouseEventKind, col: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::empty(),
        }
    }

    #[test]
    fn toggle_docks_the_keypad_and_hands_its_rows_to_history() {
        let mut app = App::default();
        let localizer = Localizer::resolve(None, &[]);
        draw_once(&mut app, &localizer, 80, 24);
        let shown_h = app.areas.history.height;
        let keypad_y = app.areas.keypad.y;
        assert!(app.areas.keypad.height >= 8, "the keypad panel shows");

        app.keypad_toggle();
        assert!(!app.keypad_shown());
        draw_once(&mut app, &localizer, 80, 24);
        assert_eq!(app.areas.keypad.height, 1, "the grab strip remains");
        assert_eq!(
            app.areas.keypad.y,
            keypad_y + 7,
            "the strip pins to the pane's bottom edge — the row the keypad's bottom border held"
        );
        assert_eq!(
            app.areas.history.height,
            shown_h + 7,
            "history grows by the seven rows the keypad gave up"
        );

        app.keypad_toggle();
        assert!(app.keypad_shown());
    }

    #[test]
    fn hiding_drops_keypad_focus_so_the_grid_is_never_an_invisible_trap() {
        let mut app = App::default();
        app.keypad_open();
        assert!(app.keypad_focused());
        app.keypad_toggle();
        assert!(!app.keypad_focused());
    }

    #[test]
    fn the_grab_dots_render_on_the_border_and_on_the_docked_strip() {
        let mut app = App::default();
        let localizer = Localizer::resolve(None, &[]);
        let buffer = draw_once(&mut app, &localizer, 80, 24);
        let kp = app.areas.keypad;
        let cx = kp.x + kp.width / 2;
        for dx in [-2i32, -1, 0] {
            assert_eq!(
                buffer
                    .cell(((cx as i32 + dx) as u16, kp.y))
                    .unwrap()
                    .symbol(),
                "·",
                "shown: the grab dots ride the top border, centered"
            );
        }

        app.keypad_toggle();
        let buffer = draw_once(&mut app, &localizer, 80, 24);
        let strip = app.areas.keypad;
        let cx = strip.x + strip.width / 2;
        for dx in [-2i32, -1, 0] {
            assert_eq!(
                buffer
                    .cell(((cx as i32 + dx) as u16, strip.y))
                    .unwrap()
                    .symbol(),
                "·",
                "docked: the strip row carries the same dots"
            );
        }
    }

    #[test]
    fn a_grab_drag_docks_and_an_upward_drag_restores() {
        let mut app = App::default();
        let mut localizer = Localizer::resolve(None, &[]);
        let store = tui_store();
        let mut drag: Option<(u16, u16, MouseDrag)> = None;
        let mut last_click: Option<(std::time::Instant, u16, u16)> = None;

        draw_once(&mut app, &localizer, 80, 24);
        let kp = app.areas.keypad;
        let col = kp.x + kp.width / 2;

        // Press on the border row, drag two rows down: docked away.
        handle_mouse(
            &mut app,
            &mut localizer,
            &store,
            mouse(MouseEventKind::Down(MouseButton::Left), col, kp.y),
            &mut drag,
            &mut last_click,
        );
        handle_mouse(
            &mut app,
            &mut localizer,
            &store,
            mouse(MouseEventKind::Drag(MouseButton::Left), col, kp.y + 2),
            &mut drag,
            &mut last_click,
        );
        assert!(!app.keypad_shown(), "two rows down docks the keypad");
        assert!(drag.is_none(), "the finished drag is inert");

        // A fresh frame puts the strip where the border was; drag two
        // rows up: restored.
        draw_once(&mut app, &localizer, 80, 24);
        let strip = app.areas.keypad;
        handle_mouse(
            &mut app,
            &mut localizer,
            &store,
            mouse(MouseEventKind::Down(MouseButton::Left), col, strip.y),
            &mut drag,
            &mut last_click,
        );
        handle_mouse(
            &mut app,
            &mut localizer,
            &store,
            mouse(MouseEventKind::Drag(MouseButton::Left), col, strip.y - 2),
            &mut drag,
            &mut last_click,
        );
        assert!(app.keypad_shown(), "two rows up brings the keypad back");
    }

    #[test]
    fn a_click_on_the_grab_area_toggles_without_dragging() {
        let mut app = App::default();
        let mut localizer = Localizer::resolve(None, &[]);
        let store = tui_store();
        let mut drag: Option<(u16, u16, MouseDrag)> = None;
        let mut last_click: Option<(std::time::Instant, u16, u16)> = None;

        draw_once(&mut app, &localizer, 80, 24);
        let kp = app.areas.keypad;
        let col = kp.x + kp.width / 2;
        handle_mouse(
            &mut app,
            &mut localizer,
            &store,
            mouse(MouseEventKind::Down(MouseButton::Left), col, kp.y),
            &mut drag,
            &mut last_click,
        );
        handle_mouse(
            &mut app,
            &mut localizer,
            &store,
            mouse(MouseEventKind::Up(MouseButton::Left), col, kp.y),
            &mut drag,
            &mut last_click,
        );
        assert!(!app.keypad_shown(), "press and release on the bar toggles");
    }

    #[test]
    fn a_small_wiggle_that_ends_elsewhere_does_not_toggle() {
        let mut app = App::default();
        let mut localizer = Localizer::resolve(None, &[]);
        let store = tui_store();
        let mut drag: Option<(u16, u16, MouseDrag)> = None;
        let mut last_click: Option<(std::time::Instant, u16, u16)> = None;

        draw_once(&mut app, &localizer, 80, 24);
        let kp = app.areas.keypad;
        let col = kp.x + kp.width / 2;
        // Down one row (under the threshold) and release off the bar:
        // not a click, not a dock — nothing changes.
        handle_mouse(
            &mut app,
            &mut localizer,
            &store,
            mouse(MouseEventKind::Down(MouseButton::Left), col, kp.y),
            &mut drag,
            &mut last_click,
        );
        handle_mouse(
            &mut app,
            &mut localizer,
            &store,
            mouse(MouseEventKind::Drag(MouseButton::Left), col, kp.y + 1),
            &mut drag,
            &mut last_click,
        );
        handle_mouse(
            &mut app,
            &mut localizer,
            &store,
            mouse(MouseEventKind::Up(MouseButton::Left), col, kp.y + 1),
            &mut drag,
            &mut last_click,
        );
        assert!(app.keypad_shown(), "an unfinished drag changes nothing");
    }
}
