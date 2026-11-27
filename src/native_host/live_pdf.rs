//! Bounded print export of the worker's current in-memory document.

use std::io::{self, Write};

use super::render_snapshot::{NativeRenderSnapshotLimits, MAX_SNAPSHOT_PAYLOAD_BYTES};
use super::{NativeDocumentSession, NativeHostError};
use crate::paint::RenderProfile;
use crate::renderer::hyperlinks::{PdfLink, PdfNamedDestination};
use crate::renderer::pdf::{
    check_complete_pdf, write_layer_trees_pdf_with_annotations, DirectPdfExportOptions,
};
use crate::HwpError;

impl NativeDocumentSession {
    pub(super) fn export_pdf(
        &self,
        first_page: u32,
        page_count: u32,
        max_output_bytes: usize,
    ) -> Result<Box<[u8]>, NativeHostError> {
        let end = first_page
            .checked_add(page_count)
            .filter(|end| *end <= self.document().page_count())
            .ok_or_else(|| {
                NativeHostError::InvalidArgument("PDF page range is out of bounds".into())
            })?;

        let mut trees = Vec::with_capacity(page_count as usize);
        let mut links = Vec::with_capacity(page_count as usize);
        let mut destinations = Vec::with_capacity(page_count as usize);
        let mut remaining_payload = MAX_SNAPSHOT_PAYLOAD_BYTES;
        for page in first_page..end {
            let snapshot = self.render_snapshot_with_limits(
                page,
                RenderProfile::Print,
                NativeRenderSnapshotLimits {
                    max_payload_bytes: remaining_payload,
                    ..Default::default()
                },
            )?;
            let usage = snapshot.usage();
            // All page snapshots borrow the same explicit font bytes.
            remaining_payload -= usage.estimated_bytes - usage.explicit_font_bytes;
            let tree = snapshot.tree();
            let page_links: Vec<PdfLink> = self
                .document()
                .hyperlinks_in_layer_tree(tree)?
                .iter()
                .map(|link| link.pdf_link())
                .collect();
            let page_destinations = self.document().bookmarks_in_layer_tree(tree)?;
            remaining_payload = page_links
                .iter()
                .map(|link| std::mem::size_of::<PdfLink>() + link.uri.capacity())
                .chain(page_destinations.iter().map(|destination| {
                    std::mem::size_of::<PdfNamedDestination>() + destination.name.capacity()
                }))
                .try_fold(remaining_payload, usize::checked_sub)
                .ok_or_else(|| {
                    NativeHostError::LimitExceeded(
                        "PDF annotations exceed the retained print input limit".into(),
                    )
                })?;
            links.push(page_links);
            destinations.push(page_destinations);
            trees.push(tree.clone());
        }

        let renderer = crate::renderer::skia::SkiaLayerRenderer::new()
            .with_cache_limits(
                super::MAX_RENDER_CACHE_ENTRIES,
                super::MAX_RENDER_CACHE_BYTES,
            )
            .try_with_font_data(&self.font_data)?;
        let mut output = PdfOutput {
            max_bytes: max_output_bytes,
            bytes: Vec::new(),
            failed: false,
        };
        let result = write_layer_trees_pdf_with_annotations(
            &trees,
            &links,
            &destinations,
            &DirectPdfExportOptions::default(),
            &mut output,
            || renderer,
        );
        // Skia can close the document even if its stream rejected a write.
        // Never hand a caller the prefix produced before that failure.
        if output.failed {
            return Err(NativeHostError::LimitExceeded(
                "PDF output exceeds its reserved byte limit or could not be allocated".into(),
            ));
        }
        result.map_err(HwpError::RenderError)?;
        check_complete_pdf(&output.bytes).map_err(HwpError::RenderError)?;
        Ok(output.bytes.into_boxed_slice())
    }
}

struct PdfOutput {
    max_bytes: usize,
    bytes: Vec<u8>,
    failed: bool,
}

impl Write for PdfOutput {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if self.failed || data.len() > self.max_bytes - self.bytes.len() {
            self.failed = true;
            return Err(io::Error::other(
                "bounded PDF stream has no remaining capacity",
            ));
        }
        let needed = self.bytes.len() + data.len();
        if needed > self.bytes.capacity() {
            let capacity = needed
                .max(self.bytes.capacity().saturating_mul(2))
                .min(self.max_bytes);
            if self
                .bytes
                .try_reserve_exact(capacity - self.bytes.len())
                .is_err()
            {
                self.failed = true;
                return Err(io::Error::other(
                    "could not allocate the bounded PDF stream",
                ));
            }
        }
        self.bytes.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
