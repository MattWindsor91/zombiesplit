//! Logic for drawing splits.

mod row;

use super::super::{
    super::presenter::state::State,
    gfx::{
        metrics::{conv::sat_i32, Anchor, Rect, Size},
        Renderer, Result,
    },
    layout::{self, Layoutable},
};

/// The split viewer widget.
#[derive(Default)]
pub struct Widget {
    /// The bounding box used for the widget.
    rect: Rect,
    /// The split drawer set, containing enough drawers for one layout.
    rows: Vec<row::Row>,
}

impl Layoutable for Widget {
    fn layout(&mut self, ctx: layout::Context) {
        self.rect = ctx.bounds;
        self.rows = rows(ctx);
    }
}

impl<R: Renderer> super::Widget<R> for Widget {
    type State = State;

    fn render(&self, r: &mut R, s: &Self::State) -> Result<()> {
        for (i, row) in self.rows.iter().enumerate() {
            // TODO(@MattWindsor91): calculate scroll point
            if let Some(split) = s.splits.get(i) {
                row.render(r, split)?;
            }
        }
        Ok(())
    }
}

/// Constructs a vector of row widgets according to `ctx`.
fn rows(ctx: layout::Context) -> Vec<row::Row> {
    // TODO(@MattWindsor91): padding
    let n_splits = usize::try_from(ctx.bounds.size.h / ctx.wmetrics.split_h).unwrap_or_default();
    (0..n_splits).map(|n| row(ctx, n)).collect()
}

fn row(ctx: layout::Context, index: usize) -> row::Row {
    let mut r = row::Row::default();
    r.layout(ctx.with_bounds(row_bounds(ctx, index)));
    r
}

fn row_bounds(ctx: layout::Context, index: usize) -> Rect {
    Rect {
        top_left: ctx.bounds.point(
            0,
            sat_i32(index) * sat_i32(ctx.wmetrics.split_h),
            Anchor::TOP_LEFT,
        ),
        size: Size {
            w: ctx.bounds.size.w,
            h: ctx.wmetrics.split_h,
        },
    }
}
