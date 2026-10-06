use pdf_writer::{Chunk, Content, Filter, Ref};
use usvg::Tree;

#[cfg(feature = "text")]
use {
    crate::render::text,
    crate::render::text::{write_font, Font},
    std::collections::HashMap,
    usvg::fontdb::ID,
};

use super::helper::deflate;
use crate::util::allocate::RefAllocator;
use crate::Result;
use crate::{ConversionOptions, GRAY_ICC_DEFLATED, SRGB_ICC_DEFLATED};

/// Holds all of the necessary information for the conversion process.
pub struct Context {
    /// Options that where passed by the user.
    pub options: ConversionOptions,
    /// The refs of the fonts
    #[cfg(feature = "text")]
    pub fonts: HashMap<ID, Option<Font>>,
    srgb_ref: Option<Ref>,
    sgray_ref: Option<Ref>,
    pub ref_allocator: RefAllocator,
}

impl Context {
    pub fn new(
        #[allow(unused_variables)] tree: &Tree,
        options: ConversionOptions,
    ) -> Result<Self> {
        #[allow(unused_mut)]
        let mut ctx = Self {
            ref_allocator: RefAllocator::new(),
            options,
            #[cfg(feature = "text")]
            fonts: HashMap::new(),
            srgb_ref: None,
            sgray_ref: None,
        };

        #[cfg(feature = "text")]
        if options.embed_text {
            text::fill_fonts(tree.root(), &mut ctx, tree.fontdb().as_ref())?;
        }

        Ok(ctx)
    }

    /// Allocate a new reference.
    pub fn alloc_ref(&mut self) -> Ref {
        self.ref_allocator.alloc_ref()
    }

    pub fn srgb_ref(&mut self) -> Ref {
        let alloc = &mut self.ref_allocator;
        let srgb_ref = &mut self.srgb_ref;

        *srgb_ref.get_or_insert_with(|| alloc.alloc_ref())
    }

    pub fn sgray_ref(&mut self) -> Ref {
        let alloc = &mut self.ref_allocator;
        let sgray_ref = &mut self.sgray_ref;

        *sgray_ref.get_or_insert_with(|| alloc.alloc_ref())
    }

    #[cfg(feature = "text")]
    pub fn font_ref(&self, id: ID) -> Option<&Font> {
        self.fonts.get(&id).and_then(|f| f.as_ref())
    }

    pub fn write_global_objects(&mut self, pdf: &mut Chunk) -> Result<()> {
        #[cfg(feature = "text")]
        {
            let allocator = &mut self.ref_allocator;

            // Sort fonts by their object reference (assigned deterministically
            // in document order by `fill_fonts`) so that the refs allocated
            // inside `write_font` and the emission order of font objects do
            // not depend on `HashMap` iteration order. This keeps the PDF
            // output byte-for-byte reproducible across runs.
            let mut fonts: Vec<_> = self.fonts.values_mut().flatten().collect();
            fonts.sort_by_key(|font| font.reference);
            for font in fonts {
                write_font(pdf, allocator, font)?
            }
        }

        if let Some(srgb_ref) = self.srgb_ref {
            pdf.icc_profile(srgb_ref, &SRGB_ICC_DEFLATED)
                .n(3)
                .range([0.0, 1.0, 0.0, 1.0, 0.0, 1.0])
                .filter(Filter::FlateDecode);
        }

        if let Some(sgray_ref) = self.sgray_ref {
            pdf.icc_profile(sgray_ref, &GRAY_ICC_DEFLATED)
                .n(1)
                .range([0.0, 1.0])
                .filter(Filter::FlateDecode);
        }

        Ok(())
    }

    /// Just a helper method so that we don't have to manually compress the content if this was
    /// set in the [ConversionOptions] struct.
    pub fn finish_content(&self, content: Content) -> Vec<u8> {
        if self.options.compress {
            deflate(&content.finish())
        } else {
            content.finish()
        }
    }
}
