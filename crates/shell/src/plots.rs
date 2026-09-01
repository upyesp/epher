//! The plot state of a CLI run (ADR-0020): `graph`/`graph3d` lines
//! accumulate curves and surfaces exactly as they do in the TUI's pane and
//! the web app's graph — and `graph save <file>` / `graph3d save <file>`
//! write the same self-contained SVG the desktop and PWA copy to the
//! clipboard, from the same renderer (`epher_core::graph_svg`). One
//! grammar, one picture, every frontend.
//!
//! The REPL, piped scripts, and one-shot strings all hold a [`Plots`]
//! across their lines; the TUI keeps its own pane state and reuses the
//! save helpers so the bytes match.

use epher_core::astro::SolarScene;
use epher_core::graph::{
    analyze, parse_graph_source, sample_data_plot, sample_spec, sample_surface, DataPlot,
    InterestPoint, SampledCurve, Surface, View3D,
};
use epher_core::graph_svg::{
    data_svg, graph3d_svg, graph_svg, solar3d_svg, Poi, DEFAULT_STROKE_WIDTH,
};
use epher_core::Env;
use epher_i18n::Localizer;

/// What a `graph`/`graph3d` line did, with the message to print. `error`
/// marks diagnostics (stderr, not data — ADR-0013).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlotOutcome {
    pub message: String,
    pub error: bool,
}

impl PlotOutcome {
    fn ok(message: String) -> Self {
        PlotOutcome {
            message,
            error: false,
        }
    }
    fn err(message: String) -> Self {
        PlotOutcome {
            message,
            error: true,
        }
    }
}

/// The curves, surfaces, data plots, and solar system plotted so far in
/// this run. A data plot (ADR-0044) is the pane's top priority, above
/// curves, like the solar system.
#[derive(Default)]
pub struct Plots {
    curves: Vec<SampledCurve>,
    surfaces: Vec<Surface>,
    data: Option<DataPlot>,
    solar: Option<SolarScene>,
}

/// The localized kind label for a point of interest — the same fluent
/// keys the web legend and the TUI list use.
fn poi_label(kind: epher_core::graph::InterestKind, localizer: &Localizer) -> String {
    use epher_core::graph::InterestKind;
    localizer.lookup(match kind {
        InterestKind::Root => "poi-root",
        InterestKind::Intersection => "poi-intersection",
        InterestKind::Maximum => "poi-maximum",
        InterestKind::Minimum => "poi-minimum",
    })
}

/// Interest points as the renderer sees them: localized kind labels with
/// coordinates.
pub fn labeled_pois(points: &[InterestPoint], localizer: &Localizer) -> Vec<Poi> {
    points
        .iter()
        .map(|p| Poi {
            kind: p.kind,
            label: poi_label(p.kind, localizer),
            x: p.x,
            y: p.y,
            curve: p.curve,
        })
        .collect()
}

impl Plots {
    pub fn new() -> Self {
        Plots::default()
    }

    /// A plot state carrying a data plot a frontend already holds — for
    /// saving without rebuilding.
    pub fn from_data(data: DataPlot) -> Self {
        Plots {
            curves: Vec::new(),
            surfaces: Vec::new(),
            data: Some(data),
            solar: None,
        }
    }

    /// A plot state carrying curves a frontend already holds (the TUI's
    /// pane) — for saving without re-plotting.
    pub fn from_curves(curves: Vec<SampledCurve>) -> Self {
        Plots {
            curves,
            surfaces: Vec::new(),
            data: None,
            solar: None,
        }
    }

    /// A plot state carrying surfaces a frontend already holds.
    pub fn from_surfaces(surfaces: Vec<Surface>) -> Self {
        Plots {
            curves: Vec::new(),
            surfaces,
            data: None,
            solar: None,
        }
    }

    /// A plot state carrying a solar system scene a frontend already
    /// holds (the TUI's pane) - for saving without rebuilding.
    pub fn from_scene(scene: SolarScene) -> Self {
        Plots {
            curves: Vec::new(),
            surfaces: Vec::new(),
            data: None,
            solar: Some(scene),
        }
    }

    /// The plotted solar system scene, if any.
    pub fn solar(&self) -> Option<&SolarScene> {
        self.solar.as_ref()
    }

    /// The plotted curves.
    pub fn curves(&self) -> &[SampledCurve] {
        &self.curves
    }

    /// The plotted data plot, if any (ADR-0044).
    pub fn data(&self) -> Option<&DataPlot> {
        self.data.as_ref()
    }

    /// The plotted surfaces.
    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }

    /// Handle the text after `graph ` (ADR-0014 grammar, plus `clear` and
    /// `save <file>`): add a curve to the plot, empty it, or write the SVG
    /// document.
    pub fn submit_graph(&mut self, source: &str, env: &Env, localizer: &Localizer) -> PlotOutcome {
        let source = source.trim();
        if source == "clear" {
            self.curves.clear();
            self.data = None;
            return PlotOutcome::ok(localizer.lookup("graph-cleared"));
        }
        if source == "save" {
            return PlotOutcome::err(localizer.lookup("graph-no-path"));
        }
        if let Some(path) = source.strip_prefix("save ") {
            let path = path.trim();
            if path.is_empty() {
                return PlotOutcome::err(localizer.lookup("graph-no-path"));
            }
            return self.save_svg(path, true, env, localizer);
        }
        // Data plots (ADR-0044): a scatter, histogram, or boxplot owns
        // the pane like a solar scene does — the newest command wins.
        if epher_core::graph::is_data_plot_source(source) {
            match sample_data_plot(source, env) {
                Ok(plot) => {
                    self.data = Some(plot);
                    self.curves.clear();
                    self.surfaces.clear();
                    self.solar = None;
                    PlotOutcome::ok(format!("graph: {source}"))
                }
                Err(e) => PlotOutcome::err(e.to_string()),
            }
        } else {
            // a plain curve command displaces any data plot
            self.data = None;
            match parse_graph_source(source) {
                Ok(spec) => match sample_spec(&spec, 120, env) {
                    Ok(samples) => {
                        self.curves.push(SampledCurve {
                            source: source.to_string(),
                            kind: spec.kind,
                            domain: spec.domain,
                            samples,
                            fill: spec.fill,
                        });
                        PlotOutcome::ok(format!("graph: {source}"))
                    }
                    Err(e) => PlotOutcome::err(e.to_string()),
                },
                Err(e) => PlotOutcome::err(e.to_string()),
            }
        }
    }

    /// Handle the text after `graph3d `: add a surface, clear, or save.
    pub fn submit_surface(
        &mut self,
        source: &str,
        env: &Env,
        localizer: &Localizer,
    ) -> PlotOutcome {
        let source = source.trim();
        if source == "clear" {
            self.surfaces.clear();
            self.data = None;
            return PlotOutcome::ok(localizer.lookup("graph-cleared"));
        }
        if source == "save" {
            return PlotOutcome::err(localizer.lookup("graph-no-path"));
        }
        if let Some(path) = source.strip_prefix("save ") {
            let path = path.trim();
            if path.is_empty() {
                return PlotOutcome::err(localizer.lookup("graph-no-path"));
            }
            return self.save_3d_svg(path, localizer);
        }
        match sample_surface(source, 40, env) {
            Ok(surface) => {
                // the newest command owns the pane (ADR-0044: data plots
                // are displaced like curves are)
                self.data = None;
                self.surfaces.push(surface);
                PlotOutcome::ok(format!("graph3d: {source}"))
            }
            Err(e) => PlotOutcome::err(e.to_string()),
        }
    }

    /// Handle the text after `solar3d `: build the scene at the evaluated
    /// time expression (a Julian Date in any form the language can
    /// produce - `now()`, `t`, `jd(2020, 1, 1)`), clear it, or save the
    /// SVG document (ADR-0037).
    pub fn submit_solar3d(
        &mut self,
        source: &str,
        env: &Env,
        localizer: &Localizer,
    ) -> PlotOutcome {
        let source = source.trim();
        if source == "clear" {
            self.solar = None;
            return PlotOutcome::ok(localizer.lookup("graph-cleared"));
        }
        if source == "save" {
            return PlotOutcome::err(localizer.lookup("graph-no-path"));
        }
        if let Some(path) = source.strip_prefix("save ") {
            let path = path.trim();
            if path.is_empty() {
                return PlotOutcome::err(localizer.lookup("graph-no-path"));
            }
            return self.save_solar_svg(path, &View3D::default(), localizer);
        }
        let jd = match epher_core::astro::eval_jd(source, env) {
            Ok(jd) => jd,
            Err(e) => return PlotOutcome::err(e.to_string()),
        };
        match epher_core::astro::solar_scene(jd) {
            Ok(scene) => {
                self.data = None;
                self.solar = Some(scene);
                PlotOutcome::ok(format!("solar3d: {source}"))
            }
            Err(e) => PlotOutcome::err(e.to_string()),
        }
    }

    /// Write the current solar system scene as a self-contained SVG
    /// document from an explicit camera pose.
    pub fn save_solar_svg(&self, path: &str, view: &View3D, localizer: &Localizer) -> PlotOutcome {
        match self.solar.as_ref() {
            Some(scene) => match solar3d_svg(scene, view, DEFAULT_STROKE_WIDTH) {
                Some(doc) => write_document(path, doc, localizer),
                None => PlotOutcome::err(localizer.lookup("graph-empty")),
            },
            None => PlotOutcome::err(localizer.lookup("graph-empty")),
        }
    }

    /// Write the current 2D plot as a self-contained SVG document.
    /// `markers` decides whether the points of interest are drawn —
    /// callers pass what their plot currently shows.
    pub fn save_svg(
        &self,
        path: &str,
        markers: bool,
        env: &Env,
        localizer: &Localizer,
    ) -> PlotOutcome {
        if self.curves.is_empty() && self.data.is_none() {
            return PlotOutcome::err(localizer.lookup("graph-empty"));
        }
        if let Some(data) = &self.data {
            return write_document(path, data_svg(data, DEFAULT_STROKE_WIDTH), localizer);
        }
        let pois = labeled_pois(&analyze(&self.curves, env), localizer);
        write_document(
            path,
            graph_svg(&self.curves, &pois, None, markers, DEFAULT_STROKE_WIDTH),
            localizer,
        )
    }

    /// Write the current 3D surfaces as a self-contained SVG document.
    pub fn save_3d_svg(&self, path: &str, localizer: &Localizer) -> PlotOutcome {
        self.save_3d_svg_with_view(path, &View3D::default(), localizer)
    }

    /// The same save from an explicit camera pose (the TUI's orbit state).
    pub fn save_3d_svg_with_view(
        &self,
        path: &str,
        view: &View3D,
        localizer: &Localizer,
    ) -> PlotOutcome {
        if self.surfaces.is_empty() {
            return PlotOutcome::err(localizer.lookup("graph-empty"));
        }
        match graph3d_svg(&self.surfaces, view, DEFAULT_STROKE_WIDTH) {
            Some(doc) => write_document(path, doc, localizer),
            None => PlotOutcome::err(localizer.lookup("graph-empty")),
        }
    }
}

fn write_document(path: &str, doc: String, localizer: &Localizer) -> PlotOutcome {
    if doc.is_empty() {
        return PlotOutcome::err(localizer.lookup("graph-empty"));
    }
    match std::fs::write(path, doc) {
        Ok(()) => PlotOutcome::ok(localizer.lookup_args("saved", &[("name", path)])),
        Err(e) => PlotOutcome::err(e.to_string()),
    }
}
