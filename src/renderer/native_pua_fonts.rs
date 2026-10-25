//! 실제 paint 공급자가 준비한 SFNT에서만 원 PUA 표시와 전진폭을 선택한다.
//! 폰트 bytes와 윤곽은 보관하지 않으며, 문서 IR의 family와 문자도 변경하지 않는다.

use std::{collections::BTreeMap, sync::Arc};

use sha2::{Digest, Sha256};

pub const BOOK_BRACKETS: [char; 2] = ['\u{F0854}', '\u{F0855}'];
pub fn is_book_bracket(character: char) -> bool {
    BOOK_BRACKETS.contains(&super::composer::hancom_symbol_display_key(character))
}
/// 실제 공급한 같은 SFNT의 원 PUA 표시와 전진을 보존할 기호다.
/// F02FC는 Unicode ►와 폭이 다르므로 원 glyph가 있는 프로그램만 사용한다.
pub const SUPPORTED_SYMBOLS: [char; 3] = ['\u{F0854}', '\u{F0855}', '\u{F02FC}'];

pub fn is_supported_symbol(character: char) -> bool {
    SUPPORTED_SYMBOLS.contains(&super::composer::hancom_symbol_display_key(character))
}

const MAX_PROGRAMS: usize = 256;
const MAX_FONT_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct NativePuaMetrics {
    digest: [u8; 32],
    face_index: u32,
    advances: [Option<f64>; 3],
}

impl NativePuaMetrics {
    pub fn advance_em(&self, character: char) -> Option<f64> {
        let key = super::composer::hancom_symbol_display_key(character);
        self.advances[SUPPORTED_SYMBOLS.iter().position(|ch| *ch == key)?]
    }
}

#[derive(Debug, Clone, PartialEq)]
struct LiteralSpaceMetric {
    digest: [u8; 32],
    face_index: u32,
    advance_em: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NativePuaFonts {
    programs: BTreeMap<String, BTreeMap<(bool, bool), Arc<NativePuaMetrics>>>,
    literal_spaces: BTreeMap<String, BTreeMap<(bool, bool), LiteralSpaceMetric>>,
}

impl NativePuaFonts {
    /// 실제 표시 face의 선택은 공급자의 책임이다. 이름만으로 호출하지 않는다.
    /// 동일 프로그램은 멱등이며 글꼴 세대 교체의 다른 프로그램은 새 snapshot이 된다.
    pub fn register(
        &mut self,
        family: &str,
        bold: bool,
        italic: bool,
        bytes: &[u8],
        face_index: u32,
    ) -> Result<bool, &'static str> {
        if family.is_empty() || family.len() > 4096 || family.chars().any(char::is_control) {
            return Err("invalid native PUA font family");
        }
        if bytes.is_empty() || bytes.len() > MAX_FONT_BYTES {
            return Err("native PUA font exceeds 32 MiB");
        }
        let face = ttf_parser::Face::parse(bytes, face_index)
            .map_err(|_| "invalid native PUA SFNT face")?;
        let advances = SUPPORTED_SYMBOLS.map(|character| {
            let glyph = face.glyph_index(character).filter(|glyph| glyph.0 != 0)?;
            let width = face.glyph_hor_advance(glyph)?;
            (width > 0).then(|| f64::from(width) / f64::from(face.units_per_em()))
        });
        if advances.iter().all(Option::is_none) {
            let removed = self
                .programs
                .get_mut(family)
                .is_some_and(|programs| programs.remove(&(bold, italic)).is_some());
            if self.programs.get(family).is_some_and(BTreeMap::is_empty) {
                self.programs.remove(family);
            }
            return Ok(removed);
        }
        let metric = NativePuaMetrics {
            digest: Sha256::digest(bytes).into(),
            face_index,
            advances,
        };
        if self
            .get(family, bold, italic)
            .is_some_and(|old| *old == metric)
        {
            return Ok(false);
        }
        if self.get(family, bold, italic).is_none()
            && self.programs.values().map(BTreeMap::len).sum::<usize>() >= MAX_PROGRAMS
        {
            return Err("native PUA font program limit exceeded");
        }
        self.programs
            .entry(family.to_owned())
            .or_default()
            .insert((bold, italic), Arc::new(metric));
        Ok(true)
    }

    /// 기호 fallback 프로그램과 공백 프로그램은 서로 덮어쓰지 않는다.
    pub fn register_literal_space(
        &mut self,
        family: &str,
        bold: bool,
        italic: bool,
        bytes: &[u8],
        face_index: u32,
    ) -> Result<bool, &'static str> {
        if family.is_empty() || family.len() > 4096 || family.chars().any(char::is_control) {
            return Err("invalid literal space font family");
        }
        if bytes.is_empty() || bytes.len() > MAX_FONT_BYTES {
            return Err("literal space font exceeds 32 MiB");
        }
        let face = ttf_parser::Face::parse(bytes, face_index)
            .map_err(|_| "invalid literal space SFNT face")?;
        let glyph = face
            .glyph_index('\u{2007}')
            .or_else(|| face.glyph_index('0'))
            .filter(|glyph| glyph.0 != 0);
        let Some(advance) = glyph
            .and_then(|glyph| face.glyph_hor_advance(glyph))
            .filter(|width| *width > 0)
        else {
            let removed = self
                .literal_spaces
                .get_mut(family)
                .is_some_and(|entries| entries.remove(&(bold, italic)).is_some());
            if self
                .literal_spaces
                .get(family)
                .is_some_and(BTreeMap::is_empty)
            {
                self.literal_spaces.remove(family);
            }
            return Ok(removed);
        };
        let metric = LiteralSpaceMetric {
            digest: Sha256::digest(bytes).into(),
            face_index,
            advance_em: f64::from(advance) / f64::from(face.units_per_em()),
        };
        let existing = self
            .literal_spaces
            .get(family)
            .and_then(|entries| entries.get(&(bold, italic)));
        if existing == Some(&metric) {
            return Ok(false);
        }
        if existing.is_none()
            && self
                .literal_spaces
                .values()
                .map(BTreeMap::len)
                .sum::<usize>()
                >= MAX_PROGRAMS
        {
            return Err("literal space font program limit exceeded");
        }
        self.literal_spaces
            .entry(family.to_owned())
            .or_default()
            .insert((bold, italic), metric);
        Ok(true)
    }

    pub fn literal_space_em(&self, family: &str, bold: bool, italic: bool) -> Option<f64> {
        Some(
            self.literal_spaces
                .get(family)?
                .get(&(bold, italic))?
                .advance_em,
        )
    }

    pub fn get(&self, family: &str, bold: bool, italic: bool) -> Option<Arc<NativePuaMetrics>> {
        self.programs.get(family)?.get(&(bold, italic)).cloned()
    }
}
