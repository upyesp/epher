//! epher-i18n — shared UI localization (ADR-0008).
//!
//! Fluent catalogs are embedded at build time; locale negotiation picks the
//! best match for the device's languages with English as the always-complete
//! fallback. The scripting language is never localized (ADR-0007).

use fluent::{FluentArgs, FluentBundle, FluentResource};
use fluent_langneg::{
    negotiate_languages, parse_accepted_languages, LanguageIdentifier as NegLangId,
    NegotiationStrategy,
};
use unic_langid::LanguageIdentifier as BundleLangId;

/// Locales shipped in v1 (ADR-0008): English, Mandarin Chinese, Hindi,
/// Spanish, French, Arabic (right-to-left), German, and Portuguese.
pub const SUPPORTED_LOCALES: &[&str] = &["en", "zh-CN", "hi", "es", "fr", "ar", "de", "pt"];

/// The default and always-complete fallback locale.
pub const DEFAULT_LOCALE: &str = "en";

/// A loaded localizer for one negotiated locale, with English as the
/// always-complete fallback (ADR-0008).
pub struct Localizer {
    locale: String,
    primary: FluentBundle<FluentResource>,
    fallback: FluentBundle<FluentResource>,
}

impl Localizer {
    /// Resolve and load the best locale: an explicit preference (e.g. the
    /// store's `language` setting) wins; otherwise negotiate the detected
    /// locales; always fall back to English.
    pub fn resolve(preference: Option<&str>, detected: &[String]) -> Localizer {
        let chosen = match preference {
            Some(p) if SUPPORTED_LOCALES.contains(&p) => p.to_string(),
            _ => negotiate(detected),
        };
        Self::for_language(&chosen)
    }

    /// The locale in use (one of [`SUPPORTED_LOCALES`]).
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// Look up a message by key, falling back to English, then to the key
    /// itself.
    pub fn lookup(&self, key: &str) -> String {
        self.lookup_args(key, &[])
    }

    /// Look up a message with placeholder arguments.
    pub fn lookup_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut fluent_args = FluentArgs::new();
        for (k, v) in args {
            fluent_args.set((*k).to_string(), (*v).to_string());
        }
        let args = if args.is_empty() {
            None
        } else {
            Some(&fluent_args)
        };
        if let Some(value) = lookup_in(&self.primary, key, args) {
            return value;
        }
        if let Some(value) = lookup_in(&self.fallback, key, args) {
            return value;
        }
        key.to_string()
    }

    fn for_language(locale: &str) -> Localizer {
        let primary = bundle_for(locale)
            .unwrap_or_else(|| bundle_for(DEFAULT_LOCALE).expect("english catalog must parse"));
        let fallback = bundle_for(DEFAULT_LOCALE).expect("english catalog must parse");
        Localizer {
            locale: locale.to_string(),
            primary,
            fallback,
        }
    }
}

/// Negotiate the detected locales against the supported set.
fn negotiate(detected: &[String]) -> String {
    let requested = parse_accepted_languages(&detected.join(","));
    if requested.is_empty() {
        return DEFAULT_LOCALE.to_string();
    }
    let available: Vec<NegLangId> = SUPPORTED_LOCALES
        .iter()
        .filter_map(|l| l.parse().ok())
        .collect();
    let default: NegLangId = DEFAULT_LOCALE.parse().expect("default locale parses");
    let negotiated = negotiate_languages(
        &requested,
        &available,
        Some(&default),
        NegotiationStrategy::Filtering,
    );
    negotiated
        .first()
        .map(|l| l.to_string())
        .unwrap_or_else(|| DEFAULT_LOCALE.to_string())
}

/// Build a bundle for one of the shipped catalogs (embedded at compile time).
fn bundle_for(locale: &str) -> Option<FluentBundle<FluentResource>> {
    let source = match locale {
        "en" => include_str!("../locales/en.ftl"),
        "zh-CN" => include_str!("../locales/zh-CN.ftl"),
        "hi" => include_str!("../locales/hi.ftl"),
        "es" => include_str!("../locales/es.ftl"),
        "fr" => include_str!("../locales/fr.ftl"),
        "ar" => include_str!("../locales/ar.ftl"),
        "de" => include_str!("../locales/de.ftl"),
        "pt" => include_str!("../locales/pt.ftl"),
        _ => return None,
    };
    let resource = FluentResource::try_new(source.to_string()).ok()?;
    let langid: BundleLangId = locale.parse().ok()?;
    let mut bundle = FluentBundle::new(vec![langid]);
    bundle.add_resource(resource).ok()?;
    Some(bundle)
}

fn lookup_in(
    bundle: &FluentBundle<FluentResource>,
    key: &str,
    args: Option<&FluentArgs>,
) -> Option<String> {
    let message = bundle.get_message(key)?;
    let pattern = message.value()?;
    let mut errors = Vec::new();
    Some(
        bundle
            .format_pattern(pattern, args, &mut errors)
            .into_owned(),
    )
}
