//! The visual portion of the zombiesplit user interface.
pub mod config;
pub mod gfx;
mod widget;

use crate::ui::view::widget::{IndexLayout, LayoutContext, Widget};

use self::gfx::render::Renderer;

use super::{presenter, Result};

use crate::model::time::position::Index;
pub use config::Config;

/// The top-level view structure.
pub struct View<R> {
    /// The renderer to use for the view.
    renderer: R,
    /// The root widget of the user interface.
    root: widget::Root,
}

impl<R: Renderer> View<R> {
    /// Creates a new graphics core.
    #[must_use]
    pub fn new(renderer: R, wmetrics: gfx::metrics::Window) -> Self {
        let mut root = widget::Root::default();
        root.layout(root_layout_context(&renderer, wmetrics));
        Self { renderer, root }
    }

    /// Redraws the user interface.
    ///
    /// # Errors
    ///
    /// Returns an error if SDL fails to redraw the screen.
    pub fn redraw(&mut self, state: &presenter::State) -> Result<()> {
        self.renderer.clear();
        self.root.render(&mut self.renderer, state)?;
        self.renderer.present();

        Ok(())
    }
}

/// Creates the root layout context.
fn root_layout_context<R: Renderer>(renderer: &R, wmetrics: gfx::metrics::Window) -> LayoutContext {
    let bounds = wmetrics.win_rect();
    let font_metrics = renderer.font_metrics();

    widget::LayoutContext {
        wmetrics,
        bounds,
        font_metrics,
        time_positions: &TIME_POSITIONS,
    }
}

// TODO(@MattWindsor91): don't hardcode this
const TIME_POSITIONS: [IndexLayout; 3] = [
    IndexLayout {
        index: Index::Minutes,
        num_digits: 2,
    },
    IndexLayout {
        index: Index::Seconds,
        num_digits: 2,
    },
    IndexLayout {
        index: Index::Milliseconds,
        num_digits: 3,
    },
];
