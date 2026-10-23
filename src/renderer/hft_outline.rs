//! 원본 HFT를 공급된 CFF의 보통 글자 윤곽으로 재생하는 공통 경계.
//! 줄 구성·전진폭은 기존 결과를 사용하며 이 모듈은 em 단위 윤곽만 돌려준다.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use sha2::{Digest, Sha256};
use ttf_parser::{cff, Face, GlyphId, OutlineBuilder};

use super::{PathCommand, TextStyle};

const MAX_SOURCE_BYTES: usize = 32 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 64 * 1024 * 1024;
const MAX_FAMILIES: usize = 256;
const MAX_COMMANDS: usize = 8192;
const MAX_CACHE_BYTES: usize = 8 * 1024 * 1024;
const MAX_CACHE_ENTRIES: usize = 512;

pub(crate) fn supports_style(style: &TextStyle) -> bool {
    style.source_font_is_hft
        && !style.bold
        && !style.italic
        && style.outline_type == 0
        && style.shadow_type == 0
        && !style.emboss
        && !style.engrave
}

pub(crate) fn first_family(family: &str) -> &str {
    family
        .split(',')
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches(['\'', '"'])
}

pub(crate) struct ContourCache<K> {
    entries: HashMap<K, Arc<[PathCommand]>>,
    bytes: usize,
}

impl<K> Default for ContourCache<K> {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            bytes: 0,
        }
    }
}

impl<K: Eq + Hash> ContourCache<K> {
    pub(crate) fn get_or_build(
        &mut self,
        key: K,
        build: impl FnOnce() -> Option<Vec<PathCommand>>,
    ) -> Option<Arc<[PathCommand]>> {
        if let Some(path) = self.entries.get(&key) {
            return Some(path.clone());
        }
        let commands = build()?;
        let bytes = commands
            .len()
            .checked_mul(std::mem::size_of::<PathCommand>())?;
        let commands: Arc<[PathCommand]> = commands.into();
        if self.entries.len() < MAX_CACHE_ENTRIES
            && self.bytes.saturating_add(bytes) <= MAX_CACHE_BYTES
        {
            self.bytes += bytes;
            self.entries.insert(key, commands.clone());
        }
        Some(commands)
    }
}

/// CFF1의 표준 em 행렬을 확인한다. CID의 별도 FD 행렬은 현재 지원하지 않는다.
pub(crate) fn contour(
    table: &cff::Table<'_>,
    glyph: GlyphId,
    units_per_em: u16,
) -> Option<Vec<PathCommand>> {
    if glyph.0 == 0 || units_per_em == 0 || table.glyph_name(glyph).is_none() {
        return None;
    }
    let matrix = table.matrix();
    let scale = 1.0 / f64::from(units_per_em);
    let expected = scale as f32;
    let tolerance = expected.abs() * f32::EPSILON * 4.0;
    if (matrix.sx - expected).abs() > tolerance
        || (matrix.sy - expected).abs() > tolerance
        || matrix.kx != 0.0
        || matrix.ky != 0.0
        || matrix.tx != 0.0
        || matrix.ty != 0.0
    {
        return None;
    }
    let mut builder = ContourBuilder {
        scale,
        current: (0.0, 0.0),
        commands: Vec::new(),
        invalid: false,
    };
    table.outline(glyph, &mut builder).ok()?;
    (!builder.invalid && !builder.commands.is_empty()).then_some(builder.commands)
}

struct ContourBuilder {
    scale: f64,
    current: (f32, f32),
    commands: Vec<PathCommand>,
    invalid: bool,
}

impl ContourBuilder {
    fn point(&mut self, x: f32, y: f32) -> (f64, f64) {
        if !x.is_finite() || !y.is_finite() {
            self.invalid = true;
        }
        (f64::from(x) * self.scale, -f64::from(y) * self.scale)
    }

    fn push(&mut self, command: PathCommand) {
        if self.commands.len() >= MAX_COMMANDS {
            self.invalid = true;
        }
        if !self.invalid {
            self.commands.push(command);
        }
    }
}

impl OutlineBuilder for ContourBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let (px, py) = self.point(x, y);
        self.push(PathCommand::MoveTo(px, py));
        self.current = (x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let (px, py) = self.point(x, y);
        self.push(PathCommand::LineTo(px, py));
        self.current = (x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (x0, y0) = self.current;
        self.curve_to(
            x0 + (x1 - x0) * (2.0 / 3.0),
            y0 + (y1 - y0) * (2.0 / 3.0),
            x + (x1 - x) * (2.0 / 3.0),
            y + (y1 - y) * (2.0 / 3.0),
            x,
            y,
        );
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let a = self.point(x1, y1);
        let b = self.point(x2, y2);
        let c = self.point(x, y);
        self.push(PathCommand::CurveTo(a.0, a.1, b.0, b.1, c.0, c.1));
        self.current = (x, y);
    }
    fn close(&mut self) {
        self.push(PathCommand::ClosePath);
    }
}

struct SourceFace {
    digest: [u8; 32],
    bytes: Arc<[u8]>,
    face_index: u32,
}

type SourceGlyphKey = ([u8; 32], u32, u16);

/// 호스트가 실제 FontFace에 적재한 프로그램만 보관한다. 파일 이름으로 다시 찾지 않는다.
#[derive(Clone, Default)]
pub(crate) struct HftOutlineFonts {
    faces: HashMap<String, Arc<SourceFace>>,
    cache: Arc<Mutex<ContourCache<SourceGlyphKey>>>,
}

impl HftOutlineFonts {
    pub(crate) fn register(
        &mut self,
        family: &str,
        bytes: &[u8],
        face_index: u32,
    ) -> Result<bool, &'static str> {
        if family.is_empty() || family.len() > 256 || family.chars().any(char::is_control) {
            return Err("invalid outline font family");
        }
        if bytes.is_empty() || bytes.len() > MAX_SOURCE_BYTES {
            return Err("outline font byte limit exceeded");
        }
        let family = first_family(family).to_lowercase();
        if family.is_empty() {
            return Err("invalid outline font family");
        }
        // FontFace는 WOFF 등도 받는다. SFNT로 읽지 못하면 이전 바인딩을 지우고
        // 호스트의 원래 글꼴 painter에 맡긴다.
        let Ok(face) = Face::parse(bytes, face_index) else {
            return Ok(self.faces.remove(&family).is_some());
        };
        if face.tables().cff.is_none()
            || face.is_variable()
            || face.is_bold()
            || face.is_italic()
            || face.tables().colr.is_some()
            || face.tables().svg.is_some()
        {
            return Ok(self.faces.remove(&family).is_some());
        }
        let digest: [u8; 32] = Sha256::digest(bytes).into();
        if self
            .faces
            .get(&family)
            .is_some_and(|old| old.digest == digest && old.face_index == face_index)
        {
            return Ok(false);
        }
        if !self.faces.contains_key(&family) && self.faces.len() >= MAX_FAMILIES {
            return Err("outline font family limit exceeded");
        }
        let mut programs = HashSet::new();
        let mut total = bytes.len();
        programs.insert(digest);
        for (name, source) in &self.faces {
            if name != &family && programs.insert(source.digest) {
                total += source.bytes.len();
            }
        }
        if total > MAX_TOTAL_BYTES {
            return Err("outline font session byte limit exceeded");
        }
        let source = self
            .faces
            .values()
            .find(|source| source.digest == digest && source.face_index == face_index)
            .cloned()
            .unwrap_or_else(|| {
                Arc::new(SourceFace {
                    digest,
                    bytes: Arc::from(bytes),
                    face_index,
                })
            });
        self.faces.insert(family, source);
        Ok(true)
    }

    pub(crate) fn glyph(&self, family: &str, character: char) -> Option<Arc<[PathCommand]>> {
        let source = self.faces.get(&first_family(family).to_lowercase())?;
        let face = Face::parse(&source.bytes, source.face_index).ok()?;
        let glyph = face.glyph_index(character)?;
        let key = (source.digest, source.face_index, glyph.0);
        self.cache.lock().ok()?.get_or_build(key, || {
            contour(face.tables().cff.as_ref()?, glyph, face.units_per_em())
        })
    }
}
