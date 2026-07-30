//! AEX: host-supplied widgets in the viewport tab bar.
//!
//! # Why this exists
//!
//! `egui_tiles` already provides the seam — `Behavior::top_bar_right_ui`,
//! documented upstream as "allow user to add buttons such as 'add new tab'".
//! Rerun implements it (maximize / minimize / view-count controls), but the
//! implementing type `TilesDelegate` is private to `viewport_ui`, so an
//! application embedding the viewer has no way to reach the strip.
//!
//! An embedder that wants its own controls there is left with two bad options:
//! draw a floating `egui::Area` on top of the tab bar (two owners on one
//! rectangle — input goes to whoever registered last), or spend a full row of
//! its own chrome directly underneath a mostly-empty 24 px strip.
//!
//! # Shape of the hook
//!
//! Additive and default-off: with no hook registered, `top_bar_right_ui`
//! behaves exactly as before, so this diff is a no-op for every existing user
//! and rebases cleanly onto upstream changes.
//!
//! The hook is a process-wide slot rather than a field threaded through
//! `ViewportUi` on purpose: the delegate is constructed deep inside
//! `viewport_ui`, and threading a callback down to it would change several
//! public signatures — precisely the kind of churn that makes a fork expensive
//! to carry.

use std::sync::{Arc, OnceLock, RwLock};

type TabBarExtraUi = Arc<dyn Fn(&mut egui::Ui) + Send + Sync>;

fn slot() -> &'static RwLock<Option<TabBarExtraUi>> {
    static SLOT: OnceLock<RwLock<Option<TabBarExtraUi>>> = OnceLock::new();
    SLOT.get_or_init(|| RwLock::new(None))
}

/// Register widgets to be drawn at the right edge of the viewport tab bar.
///
/// Called by the embedding application once at startup. Passing a new closure
/// replaces the previous one; the strip is drawn in a right-to-left layout, so
/// the first widget the closure adds ends up rightmost.
pub fn set_tab_bar_extra_ui(ui: impl Fn(&mut egui::Ui) + Send + Sync + 'static) {
    if let Ok(mut slot) = slot().write() {
        *slot = Some(Arc::new(ui));
    }
}

/// Remove any previously registered tab-bar widgets.
pub fn clear_tab_bar_extra_ui() {
    if let Ok(mut slot) = slot().write() {
        *slot = None;
    }
}

/// Draw the registered widgets, if any. The lock is released before the closure
/// runs so that a closure which re-registers cannot deadlock.
pub(crate) fn show_tab_bar_extra_ui(ui: &mut egui::Ui) {
    let hook = slot().read().ok().and_then(|slot| slot.clone());
    if let Some(hook) = hook {
        hook(ui);
    }
}
