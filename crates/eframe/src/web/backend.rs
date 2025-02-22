use std::collections::BTreeMap;

use egui::mutex::Mutex;

use crate::epi;

use super::percent_decode;

// ----------------------------------------------------------------------------

/// Data gathered between frames.
#[derive(Default)]
pub(crate) struct WebInput {
    /// Required because we don't get a position on touched
    pub primary_touch: Option<egui::TouchId>,

    /// The raw input to `egui`.
    pub raw: egui::RawInput,
}

impl WebInput {
    pub fn new_frame(&mut self, canvas_size: egui::Vec2, pixels_per_point: f32) -> egui::RawInput {
        let mut raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Default::default(), canvas_size)),
            time: Some(super::now_sec()),
            ..self.raw.take()
        };
        raw_input
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(pixels_per_point);
        raw_input
    }

    /// On alt-tab, or user clicking another HTML element.
    pub fn set_focus(&mut self, focused: bool) {
        if self.raw.focused == focused {
            return;
        }

        // log::debug!("on_web_page_focus_change: {focused}");
        self.raw.modifiers = egui::Modifiers::default(); // Avoid sticky modifier keys on alt-tab:
        self.raw.focused = focused;
        self.raw.events.push(egui::Event::WindowFocused(focused));
        self.primary_touch = None;
    }
}

// ----------------------------------------------------------------------------

/// Stores when to do the next repaint.
pub(crate) struct NeedRepaint(portable_atomic::AtomicF64);

impl Default for NeedRepaint {
    fn default() -> Self {
        Self(portable_atomic::AtomicF64::new(f64::NEG_INFINITY)) // start with a repaint
    }
}

impl NeedRepaint {
    /// Returns the time (in [`now_sec`] scale) when
    /// we should next repaint.
    pub fn when_to_repaint(&self) -> f64 {
        self.0.load(portable_atomic::Ordering::Relaxed)
    }

    /// Unschedule repainting.
    pub fn clear(&self) {
        self.0
            .store(f64::INFINITY, portable_atomic::Ordering::Relaxed);
    }

    pub fn repaint_after(&self, num_seconds: f64) {
        self.0.fetch_min(
            super::now_sec() + num_seconds,
            portable_atomic::Ordering::Relaxed,
        );
    }

    pub fn needs_repaint(&self) -> bool {
        self.when_to_repaint() <= super::now_sec()
    }

    pub fn repaint_asap(&self) {
        self.0
            .store(f64::NEG_INFINITY, portable_atomic::Ordering::Relaxed);
    }
}

// ----------------------------------------------------------------------------

/// The User-Agent of the user's browser.
pub fn user_agent() -> Option<String> {
    web_sys::window()?.navigator().user_agent().ok()
}

/// Get the [`epi::Location`] from the browser.
pub fn web_location() -> epi::Location {
    let location = web_sys::window().unwrap().location();

    let hash = percent_decode(&location.hash().unwrap_or_default());

    let query = location
        .search()
        .unwrap_or_default()
        .strip_prefix('?')
        .unwrap_or_default()
        .to_owned();

    epi::Location {
        // TODO(emilk): should we really percent-decode the url? 🤷‍♂️
        url: percent_decode(&location.href().unwrap_or_default()),
        protocol: percent_decode(&location.protocol().unwrap_or_default()),
        host: percent_decode(&location.host().unwrap_or_default()),
        hostname: percent_decode(&location.hostname().unwrap_or_default()),
        port: percent_decode(&location.port().unwrap_or_default()),
        hash,
        query_map: parse_query_map(&query),
        query,
        origin: percent_decode(&location.origin().unwrap_or_default()),
    }
}

/// query is percent-encoded
fn parse_query_map(query: &str) -> BTreeMap<String, Vec<String>> {
    let mut map: BTreeMap<String, Vec<String>> = Default::default();

    for pair in query.split('&') {
        if !pair.is_empty() {
            if let Some((key, value)) = pair.split_once('=') {
                map.entry(percent_decode(key))
                    .or_default()
                    .push(percent_decode(value));
            } else {
                map.entry(percent_decode(pair))
                    .or_default()
                    .push(String::new());
            }
        }
    }

    map
}

/*
// TODO(emilk): this test is never acgtually run, because this whole module is wasm32 only 🤦‍♂️
#[test]
fn test_parse_query() {
    assert_eq!(parse_query_map(""), BTreeMap::default());
    assert_eq!(parse_query_map("foo"), BTreeMap::from_iter([("foo", "")]));
    assert_eq!(
        parse_query_map("foo=bar"),
        BTreeMap::from_iter([("foo", "bar")])
    );
    assert_eq!(
        parse_query_map("foo=bar&baz=42"),
        BTreeMap::from_iter([("foo", "bar"), ("baz", "42")])
    );
    assert_eq!(
        parse_query_map("foo&baz=42"),
        BTreeMap::from_iter([("foo", ""), ("baz", "42")])
    );
    assert_eq!(
        parse_query_map("foo&baz&&"),
        BTreeMap::from_iter([("foo", ""), ("baz", "")])
    );
    assert_eq!(
        parse_query_map("badger=data.rrd%3Fparam1%3Dfoo%26param2%3Dbar&mushroom=snake"),
        BTreeMap::from_iter([
            ("badger", "data.rrd?param1=foo&param2=bar"),
            ("mushroom", "snake")
        ])
    );
}
*/
