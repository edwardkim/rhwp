//! 실제 paint 공급자가 준비한 SFNT에서만 원 PUA 표시와 전진폭을 선택한다.
//! 폰트 bytes와 윤곽은 보관하지 않으며, 문서 IR의 family와 문자도 변경하지 않는다.

use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
};

use sha2::{Digest, Sha256};

pub const BOOK_BRACKETS: [char; 2] = ['\u{F0854}', '\u{F0855}'];
pub fn is_book_bracket(character: char) -> bool {
    BOOK_BRACKETS.contains(&super::composer::hancom_symbol_display_key(character))
}
/// 실제 공급한 같은 SFNT의 원 PUA 표시와 전진을 보존할 기호다.
/// 이 기호는 Unicode 대체와 폭·윤곽이 달라 실제 공급한 원 glyph가 필요하다.
pub const SUPPORTED_SYMBOLS: [char; 10] = [
    '\u{F0854}',
    '\u{F0855}',
    '\u{F02FC}',
    '\u{F02EF}',
    '\u{F031C}',
    '\u{F03C5}',
    '\u{F03A7}',
    '\u{F03A8}',
    '\u{F0848}',
    '\u{F080F}',
];

pub fn is_supported_symbol(character: char) -> bool {
    SUPPORTED_SYMBOLS.contains(&super::composer::hancom_symbol_display_key(character))
}

const MAX_PROGRAMS: usize = 256;
const MAX_FONT_BYTES: usize = 32 * 1024 * 1024;
const MAX_OTF_BYTES: usize = 64 * 1024 * 1024;

/// TTC/OTC는 컨테이너 표식이므로 선택한 face의 실제 윤곽 형식을 확인한다.
pub(crate) fn is_cff_program(bytes: &[u8], face_index: u32) -> bool {
    ttf_parser::Face::parse(bytes, face_index)
        .is_ok_and(|face| face.tables().cff.is_some() || face.tables().cff2.is_some())
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativePuaMetrics {
    digest: [u8; 32],
    face_index: u32,
    advances: [Option<f64>; SUPPORTED_SYMBOLS.len()],
}

/// 실제 paint 후보 순서에 따른 CFF 전진과 앞선 TTF의 선택 경계다.
#[derive(Debug)]
pub struct NativeOtfMetrics {
    programs: Vec<NativeOtfProgram>,
    advances: std::sync::Mutex<BTreeMap<char, Option<(f64, bool)>>>,
}

#[derive(Debug, Clone)]
struct NativeOtfProgram {
    bytes: Arc<[u8]>,
    face_index: u32,
    digest: [u8; 32],
    faux_bold: bool,
    cff: bool,
}

impl PartialEq for NativeOtfMetrics {
    fn eq(&self, other: &Self) -> bool {
        self.programs.len() == other.programs.len()
            && self.programs.iter().zip(&other.programs).all(|(a, b)| {
                a.digest == b.digest && a.face_index == b.face_index && a.faux_bold == b.faux_bold
            })
    }
}

impl NativeOtfMetrics {
    pub fn needs_faux_bold(&self, character: char) -> bool {
        self.glyph_metric(character).is_some_and(|(_, faux)| faux)
    }
    pub fn advance_em(&self, character: char) -> Option<f64> {
        self.glyph_metric(character).map(|(advance, _)| advance)
    }
    fn glyph_metric(&self, character: char) -> Option<(f64, bool)> {
        let mut cache = self.advances.lock().ok()?;
        if let Some(advance) = cache.get(&character) {
            return *advance;
        }
        let mut value = None;
        for program in &self.programs {
            let Ok(face) = ttf_parser::Face::parse(&program.bytes, program.face_index) else {
                continue;
            };
            let Some(glyph) = face.glyph_index(character).filter(|id| id.0 != 0) else {
                continue;
            };
            // 앞선 실제 TTF가 glyph를 그리면 뒤의 CFF 폭으로 대체하지 않는다.
            // 원본 HFT 폭 계약을 유지하도록 None을 반환한다.
            if program.cff {
                value = face.glyph_hor_advance(glyph).map(|width| {
                    (
                        f64::from(width) / f64::from(face.units_per_em()),
                        program.faux_bold,
                    )
                });
            }
            break;
        }
        cache.insert(character, value);
        value
    }
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
    otf_programs: BTreeMap<String, BTreeMap<(bool, bool), Arc<NativeOtfMetrics>>>,
}

impl NativePuaFonts {
    pub fn clear_otf(&mut self) {
        self.otf_programs.clear();
    }
    pub fn register_otf(
        &mut self,
        family: &str,
        bold: bool,
        italic: bool,
        bytes: &[u8],
        face_index: u32,
    ) -> Result<(), &'static str> {
        if family.is_empty()
            || family.len() > 4096
            || bytes.len() > MAX_FONT_BYTES
            || !matches!(
                bytes.get(..4),
                Some(b"OTTO" | b"\0\x01\0\0" | b"true" | b"ttcf")
            )
        {
            return Err("invalid native font program");
        }
        let face =
            ttf_parser::Face::parse(bytes, face_index).map_err(|_| "invalid native font face")?;
        let digest: [u8; 32] = Sha256::digest(bytes).into();
        let old = self
            .otf_programs
            .get(family)
            .and_then(|v| v.get(&(bold, italic)));
        if old.is_some_and(|font| {
            font.programs
                .iter()
                .any(|p| p.digest == digest && p.face_index == face_index)
        }) {
            return Ok(());
        }
        let count = self
            .otf_programs
            .values()
            .flat_map(BTreeMap::values)
            .map(|font| font.programs.len())
            .sum::<usize>();
        if count >= MAX_PROGRAMS {
            return Err("native OpenType program limit exceeded");
        }
        let mut seen = HashSet::new();
        let mut shared = None;
        let mut total = 0usize;
        for program in self
            .otf_programs
            .values()
            .flat_map(BTreeMap::values)
            .flat_map(|font| &font.programs)
        {
            if seen.insert(program.digest) {
                total = total.saturating_add(program.bytes.len());
            }
            if program.digest == digest {
                shared = Some(Arc::clone(&program.bytes));
            }
        }
        if shared.is_none() && total.saturating_add(bytes.len()) > MAX_OTF_BYTES {
            return Err("native OpenType byte limit exceeded");
        }
        let mut programs = old.map_or_else(Vec::new, |font| font.programs.clone());
        programs.push(NativeOtfProgram {
            bytes: shared.unwrap_or_else(|| Arc::from(bytes)),
            face_index,
            digest,
            faux_bold: bold && !face.is_bold(),
            cff: is_cff_program(bytes, face_index),
        });
        self.otf_programs
            .entry(family.to_owned())
            .or_default()
            .insert(
                (bold, italic),
                Arc::new(NativeOtfMetrics {
                    programs,
                    advances: Default::default(),
                }),
            );
        Ok(())
    }

    pub fn get_otf(&self, family: &str, bold: bool, italic: bool) -> Option<Arc<NativeOtfMetrics>> {
        self.otf_programs.get(family)?.get(&(bold, italic)).cloned()
    }

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

#[cfg(test)]
mod otf_program_tests {
    use super::NativePuaFonts;

    fn collection(first: &[u8], second: &[u8]) -> Vec<u8> {
        let mut result = b"ttcf\0\x01\0\0\0\0\0\x02\0\0\0\0\0\0\0\0".to_vec();
        for (index, source) in [first, second].into_iter().enumerate() {
            while result.len() % 4 != 0 {
                result.push(0);
            }
            let start = u32::try_from(result.len()).unwrap();
            result[12 + index * 4..16 + index * 4].copy_from_slice(&start.to_be_bytes());
            let mut face = source.to_vec();
            let tables = u16::from_be_bytes([face[4], face[5]]) as usize;
            for table in 0..tables {
                let pos = 12 + table * 16 + 8;
                let offset = u32::from_be_bytes(face[pos..pos + 4].try_into().unwrap());
                face[pos..pos + 4].copy_from_slice(&(offset + start).to_be_bytes());
            }
            result.extend(face);
        }
        result
    }

    #[test]
    fn collection_uses_each_faces_actual_outline_and_shares_bytes() {
        let cff = include_bytes!("../../tests/fixtures/fonts/RHWPHftOutline.otf");
        let ttf = include_bytes!("../../tests/fixtures/fonts/RHWPHostFixture-Regular.ttf");
        let data = collection(cff, ttf);
        assert!(super::is_cff_program(&data, 0));
        assert!(!super::is_cff_program(&data, 1));
        assert!(!super::is_cff_program(&data, 2));
        let mut fonts = NativePuaFonts::default();
        fonts
            .register_otf("collection", false, false, &data, 0)
            .unwrap();
        fonts
            .register_otf("collection", false, false, &data, 1)
            .unwrap();
        let metrics = fonts.get_otf("collection", false, false).unwrap();
        assert!(metrics.advance_em('가').is_some());
        assert_eq!(metrics.advance_em('A'), None);
        assert!(std::sync::Arc::ptr_eq(
            &metrics.programs[0].bytes,
            &metrics.programs[1].bytes
        ));
    }

    #[test]
    fn split_family_uses_the_program_that_contains_each_character() {
        // 저장소의 합성 CFF 글꼴은 가만 지원한다. 같은 글꼴의 cmap을 나로
        // 바꿔 서로 다른 문자를 공급하는 두 번째 프로그램을 메모리에서 만든다.
        let first = include_bytes!("../../tests/fixtures/fonts/RHWPHftOutline.otf");
        let mut second = first.to_vec();
        let table_count = u16::from_be_bytes([second[4], second[5]]) as usize;
        let cmap = (0..table_count)
            .find_map(|index| {
                let pos = 12 + index * 16;
                (second.get(pos..pos + 4) == Some(b"cmap")).then(|| {
                    u32::from_be_bytes(second[pos + 8..pos + 12].try_into().unwrap()) as usize
                })
            })
            .unwrap();
        let sub =
            cmap + u32::from_be_bytes(second[cmap + 8..cmap + 12].try_into().unwrap()) as usize;
        assert_eq!(u16::from_be_bytes([second[sub], second[sub + 1]]), 4);
        // 첫 세그먼트의 end/start와 idDelta를 가에서 나로 옮긴다.
        for offset in [sub + 14, sub + 20] {
            assert_eq!(&second[offset..offset + 2], &[0xac, 0x00]);
            second[offset..offset + 2].copy_from_slice(&[0xb0, 0x98]);
        }
        let delta = 1u16.wrapping_sub('나' as u16).to_be_bytes();
        second[sub + 24..sub + 26].copy_from_slice(&delta);
        let mut fonts = NativePuaFonts::default();
        fonts
            .register_otf("synthetic", false, false, first, 0)
            .unwrap();
        fonts
            .register_otf("synthetic", false, false, &second, 0)
            .unwrap();
        let metrics = fonts.get_otf("synthetic", false, false).unwrap();
        assert!(metrics.advance_em('가').is_some());
        assert!(metrics.advance_em('나').is_some());
        assert!(metrics.advance_em('다').is_none());
    }

    #[test]
    fn earlier_true_type_glyph_blocks_a_later_cff_advance() {
        let ttf = include_bytes!("../../tests/fixtures/fonts/RHWPHostFixture-Regular.ttf");
        let mut cff = include_bytes!("../../tests/fixtures/fonts/RHWPHftOutline.otf").to_vec();
        let count = u16::from_be_bytes([cff[4], cff[5]]) as usize;
        let cmap = (0..count)
            .find_map(|index| {
                let pos = 12 + index * 16;
                (cff.get(pos..pos + 4) == Some(b"cmap")).then(|| {
                    u32::from_be_bytes(cff[pos + 8..pos + 12].try_into().unwrap()) as usize
                })
            })
            .unwrap();
        let sub = cmap + u32::from_be_bytes(cff[cmap + 8..cmap + 12].try_into().unwrap()) as usize;
        assert_eq!(u16::from_be_bytes([cff[sub], cff[sub + 1]]), 4);
        for offset in [sub + 14, sub + 20] {
            cff[offset..offset + 2].copy_from_slice(&('A' as u16).to_be_bytes());
        }
        cff[sub + 24..sub + 26].copy_from_slice(&1u16.wrapping_sub('A' as u16).to_be_bytes());

        let mut before = super::NativePuaFonts::default();
        before.register_otf("same", false, false, ttf, 0).unwrap();
        before.register_otf("same", false, false, &cff, 0).unwrap();
        assert_eq!(
            before
                .get_otf("same", false, false)
                .unwrap()
                .advance_em('A'),
            None
        );

        let mut after = super::NativePuaFonts::default();
        after.register_otf("same", false, false, &cff, 0).unwrap();
        after.register_otf("same", false, false, ttf, 0).unwrap();
        assert!(after
            .get_otf("same", false, false)
            .unwrap()
            .advance_em('A')
            .is_some());
    }
}
