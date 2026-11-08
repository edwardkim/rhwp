//! 실제 선택된 HFT 대체 CFF face의 윤곽을 기존 글자 원점에 재생한다.

use std::cell::RefCell;

use skia_safe::{Canvas, Data, Font, Paint, PathBuilder, Typeface};

use crate::renderer::hft_outline::{contour, ContourCache};
use crate::renderer::PathCommand;

pub(super) struct HftOutlineReplay {
    typeface_id: u32,
    data: Data,
    units_per_em: u16,
    cache: RefCell<ContourCache<u16>>,
}

impl HftOutlineReplay {
    pub(super) fn new(typeface: &Typeface) -> Option<Self> {
        if typeface.is_bold() || typeface.is_italic() {
            return None;
        }
        let tags = typeface.read_table_tags()?;
        if [*b"COLR", *b"SVG ", *b"fvar"]
            .iter()
            .any(|tag| tags.contains(&u32::from_be_bytes(*tag)))
        {
            return None;
        }
        let data = typeface.copy_table_data(u32::from_be_bytes(*b"CFF "))?;
        if data.len() > 32 * 1024 * 1024 {
            return None;
        }
        let units_per_em = u16::try_from(typeface.units_per_em()?).ok()?;
        Some(Self {
            typeface_id: typeface.unique_id(),
            data,
            units_per_em,
            cache: RefCell::new(ContourCache::default()),
        })
    }

    pub(super) fn draw(
        &self,
        canvas: &Canvas,
        text: &str,
        font: &Font,
        origin: (f32, f32),
        paint: &Paint,
    ) -> bool {
        let mut chars = text.chars();
        let Some(character) = chars.next() else {
            return false;
        };
        if chars.next().is_some() || font.typeface().unique_id() != self.typeface_id {
            return false;
        }
        let glyph = font.unichar_to_glyph(character as i32);
        let commands = self.cache.borrow_mut().get_or_build(glyph, || {
            let table = ttf_parser::cff::Table::parse(self.data.as_bytes())?;
            contour(&table, ttf_parser::GlyphId(glyph), self.units_per_em)
        });
        let Some(commands) = commands else {
            return false;
        };
        let mut path = PathBuilder::new();
        for command in commands.iter() {
            match *command {
                PathCommand::MoveTo(x, y) => {
                    path.move_to((x as f32, y as f32));
                }
                PathCommand::LineTo(x, y) => {
                    path.line_to((x as f32, y as f32));
                }
                PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                    path.cubic_to(
                        (x1 as f32, y1 as f32),
                        (x2 as f32, y2 as f32),
                        (x as f32, y as f32),
                    );
                }
                PathCommand::ClosePath => {
                    path.close();
                }
                _ => return false,
            }
        }
        canvas.save();
        canvas.translate(origin);
        canvas.scale((font.size(), font.size()));
        canvas.draw_path(&path.detach(), paint);
        canvas.restore();
        true
    }
}
