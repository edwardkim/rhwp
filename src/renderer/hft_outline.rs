//! 원본 HFT를 공급된 CFF의 보통 글자 윤곽으로 재생하는 공통 경계.
//! 줄 구성·전진폭은 기존 결과를 사용하며 이 모듈은 em 단위 윤곽만 돌려준다.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use sha2::{Digest, Sha256};
use ttf_parser::{cff, Face, GlyphId, OutlineBuilder};

use super::{PathCommand, TextStyle};

pub(crate) const MAX_SOURCE_BYTES: usize = 32 * 1024 * 1024;
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

/// 공급된 SFNT 프로그램(glyf·CFF·CFF2)의 기본 인스턴스 윤곽을 em 단위로 돌려준다.
/// y축은 Canvas 기준선 좌표(아래가 +)로 뒤집는다. 빈 글리프와 비정상 윤곽은 None이다.
#[cfg(target_arch = "wasm32")]
pub(crate) fn face_contour(face: &Face<'_>, glyph: GlyphId) -> Option<Vec<PathCommand>> {
    let units_per_em = face.units_per_em();
    if glyph.0 == 0 || units_per_em == 0 {
        return None;
    }
    let mut builder = ContourBuilder {
        scale: 1.0 / f64::from(units_per_em),
        current: (0.0, 0.0),
        commands: Vec::new(),
        invalid: false,
    };
    face.outline_glyph(glyph, &mut builder)?;
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
    pub(crate) pua_coverage: super::pua_glyph::FontCoverage,
    faces: HashMap<String, Vec<Arc<SourceFace>>>,
    #[cfg(target_arch = "wasm32")]
    print_faces: HashMap<(String, u16), Arc<SourceFace>>,
    cache: Arc<Mutex<ContourCache<SourceGlyphKey>>>,
}

/// 실제 공급자가 선택한 보통 face의 윤곽과 프로그램 형식을 함께 보존한다.
#[cfg(target_arch = "wasm32")]
pub(crate) struct FauxBoldOutline {
    pub commands: Arc<[PathCommand]>,
    pub regular_cff: bool,
}

impl HftOutlineFonts {
    pub(crate) fn clear_family(&mut self, family: &str) {
        self.pua_coverage.clear_family(family);
        self.faces.remove(&first_family(family).to_lowercase());
        #[cfg(target_arch = "wasm32")]
        self.print_faces
            .retain(|(name, _), _| name != &first_family(family).to_lowercase());
    }

    /// 실제 FontFace의 우선순위를 원자적으로 교체한다. 첫 cmap 소유 face만 소비한다.
    pub(crate) fn register_candidates(
        &mut self,
        family: &str,
        candidates: &[(&[u8], u32)],
    ) -> Result<bool, &'static str> {
        if family.is_empty() || family.len() > 256 || family.chars().any(char::is_control) {
            return Err("invalid outline font family");
        }
        if candidates.is_empty() || candidates.len() > 64 {
            return Err("outline font candidate limit exceeded");
        }
        let family = first_family(family).to_lowercase();
        if family.is_empty() {
            return Err("invalid outline font family");
        }
        let mut prepared = Vec::new();
        let mut programs = HashSet::new();
        let mut total = 0usize;
        for &(bytes, face_index) in candidates {
            if bytes.is_empty() || bytes.len() > MAX_SOURCE_BYTES {
                return Err("outline font byte limit exceeded");
            }
            // 혼합 TTF/WOFF·가변·스타일 face는 실제 사다리를 아는 기존 painter에 맡긴다.
            let supported = Face::parse(bytes, face_index).is_ok_and(|face| {
                face.tables().cff.is_some()
                    && !face.is_variable()
                    && !face.is_bold()
                    && !face.is_italic()
                    && face.tables().colr.is_none()
                    && face.tables().svg.is_none()
            });
            if !supported {
                return Ok(self.faces.remove(&family).is_some());
            }
            let digest: [u8; 32] = Sha256::digest(bytes).into();
            if programs.insert(digest) {
                total = total.saturating_add(bytes.len());
            }
            if total > MAX_TOTAL_BYTES {
                return Err("outline font session byte limit exceeded");
            }
            prepared.push((digest, bytes, face_index));
        }
        if self.faces.get(&family).is_some_and(|old| {
            old.len() == prepared.len()
                && old
                    .iter()
                    .zip(&prepared)
                    .all(|(source, (digest, _, index))| {
                        source.digest == *digest && source.face_index == *index
                    })
        }) {
            return Ok(false);
        }
        if !self.faces.contains_key(&family) && self.faces.len() >= MAX_FAMILIES {
            return Err("outline font family limit exceeded");
        }
        for (name, sources) in &self.faces {
            if name == &family {
                continue;
            }
            for source in sources {
                if programs.insert(source.digest) {
                    total = total.saturating_add(source.bytes.len());
                }
            }
        }
        if total > MAX_TOTAL_BYTES {
            return Err("outline font session byte limit exceeded");
        }
        let mut sources: Vec<Arc<SourceFace>> = Vec::new();
        for (digest, bytes, face_index) in prepared {
            let source = self
                .faces
                .values()
                .flatten()
                .chain(sources.iter())
                .find(|source| source.digest == digest && source.face_index == face_index)
                .cloned()
                .unwrap_or_else(|| {
                    Arc::new(SourceFace {
                        digest,
                        bytes: Arc::from(bytes),
                        face_index,
                    })
                });
            sources.push(source);
        }
        self.faces.insert(family, sources);
        Ok(true)
    }

    pub(crate) fn glyph(&self, family: &str, character: char) -> Option<Arc<[PathCommand]>> {
        let sources = self.faces.get(&first_family(family).to_lowercase())?;
        for source in sources {
            let face = Face::parse(&source.bytes, source.face_index).ok()?;
            let Some(glyph) = face.glyph_index(character) else {
                continue;
            };
            let key = (source.digest, source.face_index, glyph.0);
            // 소유 face의 윤곽을 지원하지 않으면 다음 face로 바꾸지 않고 기존 painter에 맡긴다.
            return self.cache.lock().ok()?.get_or_build(key, || {
                contour(face.tables().cff.as_ref()?, glyph, face.units_per_em())
            });
        }
        None
    }

    /// 실제 공급 목록이 선택한 정적 TTF 대체 face의 unhinted 윤곽이다.
    /// 원 face의 정상 글리프·조판 전진·문맥 shaping은 바꾸지 않는다.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn print_fallback_glyph(
        &self,
        style: &TextStyle,
        character: char,
    ) -> Option<Arc<[PathCommand]>> {
        if !style.source_font_is_ttf
            || style.is_visually_bold()
            || style.italic
            || style.outline_type != 0
            || style.shadow_type != 0
            || style.emboss
            || style.engrave
        {
            return None;
        }
        let families = super::canvas_text_font::family_chain(style);
        let selected =
            self.pua_coverage
                .text_faces
                .select(&families, &character.to_string(), false, false)?;
        // 앞 후보의 coverage가 없으면 그 글자의 실제 공급자를 확정할 수 없다.
        // faux bold와 같이 실제 CSS 사다리를 아는 Canvas painter에 맡긴다.
        if !selected.predecessors_known
            || selected.family == first_family(&style.font_family).to_lowercase()
        {
            return None;
        }
        let source = self.print_faces.get(&(selected.family, selected.weight))?;
        let face = Face::parse(&source.bytes, source.face_index).ok()?;
        face.tables().glyf.as_ref()?;
        let glyph = face.glyph_index(character)?;
        if glyph.0 == 0 {
            return None;
        }
        self.cache
            .lock()
            .ok()?
            .get_or_build((source.digest, source.face_index, glyph.0), || {
                face_contour(&face, glyph)
            })
    }

    /// 굵은 face가 없는 합성 굵기 글자의 윤곽이다(#8430). Canvas가 그 글자에 고른 것과 같은
    /// 실제 적재 face(공급 목록의 보통 굵기 선택)를 [`Self::print_fallback_glyph`]와 같은 Print TTF
    /// 보관소에서 찾는다. 선택이 기울임 face이거나 보관하지 않은 face면 None이다.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn faux_bold_glyph(
        &self,
        style: &TextStyle,
        character: char,
        allow_hft_outline: bool,
    ) -> Option<FauxBoldOutline> {
        let families = super::canvas_text_font::family_chain(style);
        let selected = self.pua_coverage.text_faces.select(
            &families,
            &character.to_string(),
            false,
            style.italic,
        );
        // 선언 또는 HWP5의 미정의 유형 비트와 실제 공급 프로그램은 다를 수 있다.
        // 실제 원 TTF 별칭이 이 글자를 소유하면 유지하고, 그렇지 않으면 등록된
        // 같은 family의 CFF를 소비한다. 원본 유형·측정 프로그램을 다시 추정하지 않는다.
        let source_ttf_selected = selected
            .as_ref()
            .is_some_and(|face| face.family.starts_with("__rhwp_ttf__"));
        // Print는 등록된 원 HFT CFF도 같은 fill+stroke 경로에서 소비한다.
        // 윤곽 재생 허용과 primary의 결측 소유 판단은 별개다.
        // 실제 기울임·옛한글 전용 face의 선택은 바꾸지 않는다.
        if !source_ttf_selected
            && !style.italic
            && !super::contains_old_hangul_jamo(&character.to_string())
            && self
                .faces
                .contains_key(&first_family(&style.font_family).to_lowercase())
        {
            // 분리 CFF가 빠뜨린 글자는 같은 CSS family의 TTF 동반 face가
            // 소유할 수 있다. 불완전한 coverage로 선택한 뒤 family의 윤곽을
            // 대신 그리지 않고 실제 FontFace 사다리를 아는 Canvas에 맡긴다.
            let primary = self.glyph(&style.font_family, character);
            if allow_hft_outline || primary.is_none() {
                return primary.map(|commands| FauxBoldOutline {
                    commands,
                    regular_cff: true,
                });
            }
        }
        let selected = selected?;
        // 복수 FontFace의 공급 정보가 생략된 앞 family를 건너뛰고 선택한
        // 뒤 face는 실제 Canvas 선택의 증거가 아니다. 원 primary의 등록 CFF는
        // 위에서 이미 해소했으며, 나머지는 실제 사다리를 아는 글자 painter에 맡긴다.
        if !selected.predecessors_known {
            return None;
        }
        // 실제 기울임 face는 보관하지 않는다. 같은 family·굵기의 정체 윤곽으로 대신하지 않는다.
        if selected.slanted {
            return None;
        }
        let source = self.print_faces.get(&(selected.family, selected.weight))?;
        let face = Face::parse(&source.bytes, source.face_index).ok()?;
        let glyph = face.glyph_index(character)?;
        let commands = self
            .cache
            .lock()
            .ok()?
            .get_or_build((source.digest, source.face_index, glyph.0), || {
                face_contour(&face, glyph)
            })?;
        Some(FauxBoldOutline {
            commands,
            regular_cff: false,
        })
    }

    /// 호스트가 실제 적재한 TTF만 보조 캐시에 둔다. 한도를 넘으면 원 painter를 유지한다.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn register_print_font(&mut self, family: &str, bytes: &[u8], index: u32) {
        let family = first_family(family).to_lowercase();
        let usable = || {
            if bytes.len() > MAX_SOURCE_BYTES {
                return None;
            }
            let face = Face::parse(bytes, index).ok()?;
            (face.tables().glyf.is_some()
                && !face.is_variable()
                && !face.is_bold()
                && !face.is_italic()
                && face.tables().colr.is_none()
                && face.tables().svg.is_none())
            .then_some(face.weight().to_number())
        };
        let Some(weight) = usable() else {
            self.print_faces.retain(|(name, _), _| name != &family);
            return;
        };
        let key = (family, weight);
        let digest: [u8; 32] = Sha256::digest(bytes).into();
        if self
            .print_faces
            .get(&key)
            .is_some_and(|old| old.digest == digest && old.face_index == index)
        {
            return;
        }
        let mut programs = HashSet::from([digest]);
        let mut total = bytes.len();
        for (name, source) in &self.print_faces {
            if name != &key && programs.insert(source.digest) {
                total += source.bytes.len();
            }
        }
        if total > MAX_TOTAL_BYTES
            || (!self.print_faces.contains_key(&key) && self.print_faces.len() >= MAX_FAMILIES)
        {
            self.print_faces.remove(&key);
            return;
        }
        let source = self
            .print_faces
            .values()
            .find(|source| source.digest == digest && source.face_index == index)
            .cloned()
            .unwrap_or_else(|| {
                Arc::new(SourceFace {
                    digest,
                    bytes: Arc::from(bytes),
                    face_index: index,
                })
            });
        self.print_faces.insert(key, source);
    }
}
