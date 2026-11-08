//! The backend trait — the layer below the pipeline.
//!
//! Every concrete backend (composite RGBA, separation per-plate, future
//! preflight measure-only) implements [`PaintBackend`]. The pipeline produces
//! [`ResolvedPaintCmd`]s; the backend translates them into draw calls against
//! its surface.
//!
//! `Surface` is an associated type because the surface representation is
//! deeply backend-specific (`&mut tiny_skia::Pixmap` for composite, `&mut
//! [tiny_skia::Pixmap]` for separation, `&mut PaintLedger` for preflight).
//! Generic over `Self::Surface` lets each backend pick the natural shape
//! without runtime indirection.

use crate::error::Result;

use super::resolved::ResolvedPaintCmd;

/// A pluggable paint backend.
///
/// The pipeline calls [`PaintBackend::begin_page`] once before any paint
/// operation, calls [`PaintBackend::paint`] once per intent the operator
/// dispatcher produces, and calls [`PaintBackend::end_page`] once at the end.
///
/// `Surface` is the backend-specific surface type. The pipeline owns nothing
/// about it; the operator walker hands the backend its surface as a mutable
/// borrow on each call. This keeps the pipeline free of backend-specific
/// allocation concerns.
pub(crate) trait PaintBackend {
    /// Backend-specific draw target. Composite uses `&mut tiny_skia::Pixmap`;
    /// separation uses `&mut [tiny_skia::Pixmap]`.
    type Surface<'s>
    where
        Self: 's;

    /// Called once at the start of a page. Backends use this to fill a
    /// background, allocate per-plate buffers, or compile shared state.
    ///
    /// Default implementation is a no-op so dummy backends (test surfaces,
    /// preflight collectors that don't paint) can skip it.
    fn begin_page(&mut self, _page_width_px: u32, _page_height_px: u32) -> Result<()> {
        Ok(())
    }

    /// Paint a single resolved command into the surface.
    fn paint(&mut self, cmd: &ResolvedPaintCmd, surface: Self::Surface<'_>) -> Result<()>;

    /// Called once at the end of a page. Backends use this to finalise
    /// per-page state (encoding, ink-coverage summaries).
    ///
    /// Default implementation is a no-op.
    fn end_page(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::graphics_state::Matrix;
    use smallvec::SmallVec;

    use super::super::intent::{PaintKind, PaintSide};
    use super::super::resolved::{BlendPlan, ClipPlan, InkSelector, OverprintPlan, ResolvedColor};

    /// A minimal in-memory backend used to exercise the trait surface.
    /// Captures the command sequence so tests can assert on what was
    /// painted without dragging in tiny-skia.
    struct RecordingBackend {
        begin_calls: Vec<(u32, u32)>,
        paint_calls: usize,
        end_calls: usize,
    }

    impl PaintBackend for RecordingBackend {
        type Surface<'s> = &'s mut ();

        fn begin_page(&mut self, w: u32, h: u32) -> Result<()> {
            self.begin_calls.push((w, h));
            Ok(())
        }

        fn paint(&mut self, _cmd: &ResolvedPaintCmd, _surface: Self::Surface<'_>) -> Result<()> {
            self.paint_calls += 1;
            Ok(())
        }

        fn end_page(&mut self) -> Result<()> {
            self.end_calls += 1;
            Ok(())
        }
    }

    fn dummy_cmd() -> ResolvedPaintCmd<'static> {
        // Construct a placeholder cmd. Path is None-equivalent (we use a
        // shading kind whose name is a 'static borrow so the cmd's
        // lifetime is 'static).
        ResolvedPaintCmd {
            kind: PaintKind::Shading {
                shading_name: "DummyShading",
            },
            side: PaintSide::Fill,
            color: ResolvedColor::Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            overprint: OverprintPlan {
                enabled: false,
                mode: 0,
                participating: SmallVec::new(),
                selector: InkSelector::Listed,
                all_tint: 0.0,
                spot_source: None,
                alt_cmyk_fallback: None,
            },
            blend: BlendPlan::Native(tiny_skia::BlendMode::SourceOver),
            clip: ClipPlan::None,
            ctm: Matrix::identity(),
        }
    }

    #[test]
    fn trait_lifecycle_is_begin_paint_end() {
        let mut backend = RecordingBackend {
            begin_calls: Vec::new(),
            paint_calls: 0,
            end_calls: 0,
        };
        backend.begin_page(100, 200).unwrap();
        let cmd = dummy_cmd();
        let mut surface = ();
        backend.paint(&cmd, &mut surface).unwrap();
        backend.paint(&cmd, &mut surface).unwrap();
        backend.end_page().unwrap();

        assert_eq!(backend.begin_calls, vec![(100, 200)]);
        assert_eq!(backend.paint_calls, 2);
        assert_eq!(backend.end_calls, 1);
    }

    /// A backend that does nothing — exists purely to prove the default
    /// `begin_page` and `end_page` implementations are sufficient for the
    /// minimal case.
    struct NoOpBackend;

    impl PaintBackend for NoOpBackend {
        type Surface<'s> = &'s mut ();

        fn paint(&mut self, _cmd: &ResolvedPaintCmd, _surface: Self::Surface<'_>) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn default_begin_and_end_are_no_ops() {
        let mut backend = NoOpBackend;
        // Default begin_page / end_page must succeed without overriding.
        backend.begin_page(0, 0).unwrap();
        let cmd = dummy_cmd();
        let mut surface = ();
        backend.paint(&cmd, &mut surface).unwrap();
        backend.end_page().unwrap();
    }
}
