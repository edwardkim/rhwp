use crate::render::path;
use crate::util::allocate::RefAllocator;
use crate::util::context::Context;
use crate::util::helper::{deflate, ContentExt, TransformExt};
use crate::util::resources::ResourceContainer;
use crate::ConversionError::{self, InvalidFont, SubsetError};
use crate::Result;
use pdf_writer::types::{
    CidFontType, FontFlags, SystemInfo, TextRenderingMode, UnicodeCmap,
};
use pdf_writer::writers::WMode;
use pdf_writer::{Chunk, Content, Filter, Finish, Name, Ref, Str};
use siphasher::sip128::{Hasher128, SipHasher13};
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::sync::Arc;
use subsetter::GlyphRemapper;
use ttf_parser::{name_id, Face, GlyphId, PlatformId, Tag};
use usvg::{Fill, Group, ImageKind, Node, PaintOrder, Stroke, Transform};

const CFF: Tag = Tag::from_bytes(b"CFF ");
const CFF2: Tag = Tag::from_bytes(b"CFF2");

const SUBSET_TAG_LEN: usize = 6;
const IDENTITY_H: &str = "Identity-H";

const CMAP_NAME: Name = Name(b"Custom");
const SYSTEM_INFO: SystemInfo = SystemInfo {
    registry: Str(b"Adobe"),
    ordering: Str(b"Identity"),
    supplement: 0,
};

/// Write all font objects into the chunk.
pub fn write_font(
    chunk: &mut Chunk,
    alloc: &mut RefAllocator,
    font: &mut Font,
) -> Result<()> {
    // We've already parsed all fonts when creating the font objects, so each font
    // should be valid.
    let ttf = Face::parse(&font.face_data, font.face_index)
        .map_err(|_| InvalidFont(font.id))?;
    let units_per_em = ttf.units_per_em();

    let type0_ref = font.reference;
    let cid_ref = alloc.alloc_ref();
    let descriptor_ref = alloc.alloc_ref();
    let cmap_ref = alloc.alloc_ref();
    let data_ref = alloc.alloc_ref();

    let glyph_set = &mut font.glyph_set;
    let glyph_remapper = &font.glyph_remapper;

    // Do we have a TrueType or CFF font?
    //
    // FIXME: CFF2 must be handled differently and requires PDF 2.0
    // (or we have to convert it to CFF).
    let is_cff = ttf
        .raw_face()
        .table(CFF)
        .or_else(|| ttf.raw_face().table(CFF2))
        .is_some();

    let base_font = base_font_name(&ttf, glyph_set);
    let base_font_type0 =
        if is_cff { format!("{base_font}-{IDENTITY_H}") } else { base_font.clone() };

    chunk
        .type0_font(type0_ref)
        .base_font(Name(base_font_type0.as_bytes()))
        .encoding_predefined(Name(IDENTITY_H.as_bytes()))
        .descendant_font(cid_ref)
        .to_unicode(cmap_ref);

    // Write the CID font referencing the font descriptor.
    let mut cid = chunk.cid_font(cid_ref);
    cid.subtype(if is_cff { CidFontType::Type0 } else { CidFontType::Type2 });
    cid.base_font(Name(base_font.as_bytes()));
    cid.system_info(SYSTEM_INFO);
    cid.font_descriptor(descriptor_ref);
    cid.default_width(0.0);
    if !is_cff {
        cid.cid_to_gid_map_predefined(Name(b"Identity"));
    }

    let mut widths = vec![];
    for old_gid in glyph_remapper.remapped_gids() {
        let width = ttf.glyph_hor_advance(GlyphId(old_gid)).unwrap_or(0);
        let units = (width as f64 / units_per_em as f64) * 1000.0;
        widths.push(units as f32);
    }

    // Write all non-zero glyph widths.
    let mut first = 0;
    let mut width_writer = cid.widths();
    for (w, group) in widths.group_by_key(|&w| w) {
        let end = first + group.len();
        if w != 0.0 {
            let last = end - 1;
            width_writer.same(first as u16, last as u16, w);
        }
        first = end;
    }

    width_writer.finish();
    cid.finish();

    let mut flags = FontFlags::empty();
    flags.set(
        FontFlags::SERIF,
        find_name(&ttf, name_id::POST_SCRIPT_NAME)
            .is_some_and(|name| name.contains("Serif")),
    );
    flags.set(FontFlags::FIXED_PITCH, ttf.is_monospaced());
    flags.set(FontFlags::ITALIC, ttf.is_italic());
    flags.insert(FontFlags::SYMBOLIC);
    flags.insert(FontFlags::SMALL_CAP);

    let convert = |val| (val / units_per_em as f32) * 1000.0;

    let global_bbox = ttf.global_bounding_box();
    let bbox = pdf_writer::Rect::new(
        convert(global_bbox.x_min as f32),
        convert(global_bbox.y_min as f32),
        convert(global_bbox.x_max as f32),
        convert(global_bbox.y_max as f32),
    );

    let italic_angle = ttf.italic_angle();
    let ascender = convert(ttf.typographic_ascender().unwrap_or(ttf.ascender()) as f32);
    let descender =
        convert(ttf.typographic_descender().unwrap_or(ttf.descender()) as f32);
    let cap_height = ttf
        .capital_height()
        .filter(|&h| h > 0)
        .map(|h| convert(h as f32))
        .unwrap_or(ascender);
    let stem_v = 10.0 + 0.244 * (f32::from(ttf.weight().to_number()) - 50.0);

    // Write the font descriptor (contains metrics about the font).
    let mut font_descriptor = chunk.font_descriptor(descriptor_ref);
    font_descriptor
        .name(Name(base_font.as_bytes()))
        .flags(flags)
        .bbox(bbox)
        .italic_angle(italic_angle)
        .ascent(ascender)
        .descent(descender)
        .cap_height(cap_height)
        .stem_v(stem_v);

    if is_cff {
        font_descriptor.font_file3(data_ref);
    } else {
        font_descriptor.font_file2(data_ref);
    }

    font_descriptor.finish();

    let cmap = create_cmap(glyph_set, glyph_remapper).ok_or(SubsetError(font.id))?;
    chunk.cmap(cmap_ref, &cmap.finish()).writing_mode(WMode::Horizontal);

    // Subset and write the font's bytes.
    let data = subset_font(&font.face_data, font.face_index, glyph_remapper, font.id)?;

    let mut stream = chunk.stream(data_ref, &data);
    stream.filter(Filter::FlateDecode);
    if is_cff {
        stream.pair(Name(b"Subtype"), Name(b"CIDFontType0C"));
    }

    stream.finish();
    Ok(())
}

/// Create a /ToUnicode CMap.
fn create_cmap(
    glyph_set: &mut BTreeMap<u16, String>,
    glyph_remapper: &GlyphRemapper,
) -> Option<UnicodeCmap> {
    // Produce a reverse mapping from glyphs' CIDs to unicode strings.
    let mut cmap = UnicodeCmap::new(CMAP_NAME, SYSTEM_INFO);
    for (&g, text) in glyph_set.iter() {
        let new_gid = glyph_remapper.get(g)?;
        if !text.is_empty() {
            cmap.pair_with_multiple(new_gid, text.chars());
        }
    }

    Some(cmap)
}

fn subset_font(
    font_data: &[u8],
    index: u32,
    glyph_remapper: &GlyphRemapper,
    id: fontdb::ID,
) -> Result<Vec<u8>> {
    let data = font_data;
    let subsetted =
        subsetter::subset(data, index, glyph_remapper).map_err(|_| SubsetError(id))?;
    let mut data = subsetted.as_ref();

    // Extract the standalone CFF font program if applicable.
    let face = ttf_parser::RawFace::parse(data, 0).map_err(|_| SubsetError(id))?;
    if let Some(cff) = face.table(CFF) {
        data = cff;
    }

    Ok(deflate(data))
}

/// Render some text into a content stream.
pub fn render(
    text: &usvg::Text,
    chunk: &mut Chunk,
    content: &mut Content,
    ctx: &mut Context,
    rc: &mut ResourceContainer,
    accumulated_transform: Transform,
) -> Result<()> {
    let mut font_names = HashMap::new();

    // TODO: Don't clone here...
    let fonts = ctx.fonts.clone();

    for span in text.layouted() {
        for glyph in &span.positioned_glyphs {
            let Some(font) = ctx.font_ref(glyph.font) else { continue };
            font_names
                .entry(font.reference)
                .or_insert_with(|| rc.add_font(font.reference));
        }
    }

    for span in text.layouted() {
        if !span.visible {
            continue;
        }

        let operation = |content: &mut Content| -> Result<()> {
            for glyph in &span.positioned_glyphs {
                let Some(font) = fonts.get(&glyph.font).and_then(|f| f.as_ref()) else {
                    continue;
                };

                let name = font_names.get(&font.reference).unwrap();

                // TODO: Remove unwraps and switch to error-based handling.
                // NOTE(laurmaedje): If it can't happen, I think a panic is
                // better. There is no way to handle it as a consumer of
                // svg2pdf.
                let cid = font.glyph_remapper.get(glyph.id.0).unwrap();
                let ts = glyph
                    .outline_transform()
                    .pre_scale(font.units_per_em as f32, font.units_per_em as f32)
                    // The glyphs in usvg are already scaled according the font size, but
                    // we want to leverage the native PDF font size feature instead, so we downscale
                    // it to a font size of 1.
                    .pre_scale(1.0 / span.font_size.get(), 1.0 / span.font_size.get());
                content.save_state_checked()?;
                content.begin_text();
                content.set_text_matrix(ts.to_pdf_transform());
                content.set_font(Name(name.as_bytes()), span.font_size.get());
                content.show(Str(&[(cid >> 8) as u8, (cid & 0xff) as u8]));
                content.end_text();
                content.restore_state();
            }

            Ok(())
        };

        let stroke_operation = |content: &mut Content, _: &Stroke| -> Result<()> {
            content.set_text_rendering_mode(TextRenderingMode::Stroke);
            operation(content)
        };

        let fill_operation = |content: &mut Content, _: &Fill| -> Result<()> {
            content.set_text_rendering_mode(TextRenderingMode::Fill);
            operation(content)
        };

        if let Some(overline) = &span.overline {
            path::render(overline, chunk, content, ctx, rc, accumulated_transform)?;
        }

        if let Some(underline) = &span.underline {
            path::render(underline, chunk, content, ctx, rc, accumulated_transform)?;
        }

        content.save_state_checked()?;
        match (span.fill.as_ref(), span.stroke.as_ref()) {
            (Some(fill), Some(stroke)) => match span.paint_order {
                PaintOrder::FillAndStroke => {
                    path::fill(
                        fill,
                        chunk,
                        content,
                        ctx,
                        rc,
                        fill_operation,
                        accumulated_transform,
                        text.bounding_box(),
                    )?;
                    path::stroke(
                        stroke,
                        chunk,
                        content,
                        ctx,
                        rc,
                        stroke_operation,
                        accumulated_transform,
                        text.bounding_box(),
                    )?;
                }
                PaintOrder::StrokeAndFill => {
                    path::stroke(
                        stroke,
                        chunk,
                        content,
                        ctx,
                        rc,
                        stroke_operation,
                        accumulated_transform,
                        text.bounding_box(),
                    )?;
                    path::fill(
                        fill,
                        chunk,
                        content,
                        ctx,
                        rc,
                        fill_operation,
                        accumulated_transform,
                        text.bounding_box(),
                    )?;
                }
            },
            (None, Some(stroke)) => {
                path::stroke(
                    stroke,
                    chunk,
                    content,
                    ctx,
                    rc,
                    stroke_operation,
                    accumulated_transform,
                    text.bounding_box(),
                )?;
            }
            (Some(fill), None) => {
                path::fill(
                    fill,
                    chunk,
                    content,
                    ctx,
                    rc,
                    fill_operation,
                    accumulated_transform,
                    text.bounding_box(),
                )?;
            }
            (None, None) => {
                content.set_text_rendering_mode(TextRenderingMode::Invisible);
                operation(content)?;
            }
        };

        content.restore_state();

        if let Some(line_through) = &span.line_through {
            path::render(line_through, chunk, content, ctx, rc, accumulated_transform)?;
        }
    }

    Ok(())
}

/// Creates the base font name for a font with a specific glyph subset.
/// Consists of a subset tag and the PostScript name of the font.
///
/// Returns a string of length maximum 116, so that even with `-Identity-H`
/// added it does not exceed the maximum PDF/A name length of 127.
fn base_font_name<T: Hash>(ttf: &Face, glyphs: &T) -> String {
    const MAX_LEN: usize = 127 - REST_LEN;
    const REST_LEN: usize = SUBSET_TAG_LEN + 1 + 1 + IDENTITY_H.len();

    let postscript_name = find_name(ttf, name_id::POST_SCRIPT_NAME);
    let name = postscript_name.as_deref().unwrap_or("unknown");
    let trimmed = &name[..name.len().min(MAX_LEN)];

    // Hash the full name (we might have trimmed) and the glyphs to produce
    // a fairly unique subset tag.
    let subset_tag = subset_tag(&(name, glyphs));

    format!("{subset_tag}+{trimmed}")
}

/// Produce a unique 6 letter tag for a glyph set.
fn subset_tag<T: Hash>(glyphs: &T) -> String {
    const BASE: u128 = 26;
    let mut hash = hash128(&glyphs);
    let mut letter = [b'A'; SUBSET_TAG_LEN];
    for l in letter.iter_mut() {
        *l = b'A' + (hash % BASE) as u8;
        hash /= BASE;
    }
    std::str::from_utf8(&letter).unwrap().into()
}

/// Calculate a 128-bit siphash of a value.
pub fn hash128<T: Hash + ?Sized>(value: &T) -> u128 {
    let mut state = SipHasher13::new();
    value.hash(&mut state);
    state.finish128().as_u128()
}

/// Try to find and decode the name with the given id.
pub(super) fn find_name(ttf: &Face, name_id: u16) -> Option<String> {
    ttf.names().into_iter().find_map(|entry| {
        if entry.name_id == name_id {
            if let Some(string) = entry.to_string() {
                return Some(string);
            }

            if entry.platform_id == PlatformId::Macintosh && entry.encoding_id == 0 {
                return Some(decode_mac_roman(entry.name));
            }
        }

        None
    })
}

/// Decode mac roman encoded bytes into a string.
fn decode_mac_roman(coded: &[u8]) -> String {
    #[rustfmt::skip]
    const TABLE: [char; 128] = [
        'Ä', 'Å', 'Ç', 'É', 'Ñ', 'Ö', 'Ü', 'á', 'à', 'â', 'ä', 'ã', 'å', 'ç', 'é', 'è',
        'ê', 'ë', 'í', 'ì', 'î', 'ï', 'ñ', 'ó', 'ò', 'ô', 'ö', 'õ', 'ú', 'ù', 'û', 'ü',
        '†', '°', '¢', '£', '§', '•', '¶', 'ß', '®', '©', '™', '´', '¨', '≠', 'Æ', 'Ø',
        '∞', '±', '≤', '≥', '¥', 'µ', '∂', '∑', '∏', 'π', '∫', 'ª', 'º', 'Ω', 'æ', 'ø',
        '¿', '¡', '¬', '√', 'ƒ', '≈', '∆', '«', '»', '…', '\u{a0}', 'À', 'Ã', 'Õ', 'Œ', 'œ',
        '–', '—', '“', '”', '‘', '’', '÷', '◊', 'ÿ', 'Ÿ', '⁄', '€', '‹', '›', 'ﬁ', 'ﬂ',
        '‡', '·', '‚', '„', '‰', 'Â', 'Ê', 'Á', 'Ë', 'È', 'Í', 'Î', 'Ï', 'Ì', 'Ó', 'Ô',
        '\u{f8ff}', 'Ò', 'Ú', 'Û', 'Ù', 'ı', 'ˆ', '˜', '¯', '˘', '˙', '˚', '¸', '˝', '˛', 'ˇ',
    ];

    fn char_from_mac_roman(code: u8) -> char {
        if code < 128 {
            code as char
        } else {
            TABLE[(code - 128) as usize]
        }
    }

    coded.iter().copied().map(char_from_mac_roman).collect()
}

/// Extra methods for [`[T]`](slice).
pub trait SliceExt<T> {
    /// Split a slice into consecutive runs with the same key and yield for
    /// each such run the key and the slice of elements with that key.
    fn group_by_key<K, F>(&self, f: F) -> GroupByKey<'_, T, F>
    where
        F: FnMut(&T) -> K,
        K: PartialEq;
}

impl<T> SliceExt<T> for [T] {
    fn group_by_key<K, F>(&self, f: F) -> GroupByKey<'_, T, F> {
        GroupByKey { slice: self, f }
    }
}

/// This struct is created by [`SliceExt::group_by_key`].
pub struct GroupByKey<'a, T, F> {
    slice: &'a [T],
    f: F,
}

impl<'a, T, K, F> Iterator for GroupByKey<'a, T, F>
where
    F: FnMut(&T) -> K,
    K: PartialEq,
{
    type Item = (K, &'a [T]);

    fn next(&mut self) -> Option<Self::Item> {
        let mut iter = self.slice.iter();
        let key = (self.f)(iter.next()?);
        let count = 1 + iter.take_while(|t| (self.f)(t) == key).count();
        let (head, tail) = self.slice.split_at(count);
        self.slice = tail;
        Some((key, head))
    }
}

#[derive(Clone)]
pub struct Font {
    pub id: fontdb::ID,
    pub glyph_set: BTreeMap<u16, String>,
    pub glyph_remapper: GlyphRemapper,
    pub reference: Ref,
    pub face_data: Arc<Vec<u8>>,
    pub units_per_em: u16,
    pub face_index: u32,
}

pub fn fill_fonts(
    group: &Group,
    ctx: &mut Context,
    fontdb: &fontdb::Database,
) -> Result<()> {
    for child in group.children() {
        match child {
            Node::Text(t) => {
                let allocator = &mut ctx.ref_allocator;
                for span in t.layouted() {
                    for g in &span.positioned_glyphs {
                        let font = ctx.fonts.entry(g.font).or_insert_with(|| {
                            fontdb
                                .with_face_data(g.font, |data, face_index| {
                                    // TODO: Currently, we are parsing each font twice, once here
                                    // and once again when writing the fonts. We should probably
                                    // improve on that...
                                    if let Ok(ttf) =
                                        ttf_parser::Face::parse(data, face_index)
                                    {
                                        let reference = allocator.alloc_ref();
                                        let glyph_set = BTreeMap::new();
                                        let glyph_remapper = GlyphRemapper::new();
                                        return Some(Font {
                                            id: g.font,
                                            reference,
                                            face_data: Arc::new(Vec::from(data)),
                                            units_per_em: ttf.units_per_em(),
                                            glyph_set,
                                            glyph_remapper,
                                            face_index,
                                        });
                                    }

                                    None
                                })
                                .flatten()
                        });

                        if let Some(ref mut font) = font {
                            font.glyph_set.insert(g.id.0, g.text.clone());
                            font.glyph_remapper.remap(g.id.0);
                        }

                        if ctx.options.pdfa && g.id.0 == 0 {
                            return Err(ConversionError::MissingGlyphs);
                        }
                    }
                }
            }
            Node::Group(group) => fill_fonts(group, ctx, fontdb)?,
            Node::Image(image) => {
                if let ImageKind::SVG(svg) = image.kind() {
                    fill_fonts(svg.root(), ctx, fontdb)?;
                }
            }
            _ => {}
        }

        let mut result = Ok(());
        child.subroots(|subroot| {
            result = result.and(fill_fonts(subroot, ctx, fontdb));
        });
        result?;
    }

    Ok(())
}
