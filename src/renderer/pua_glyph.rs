//! 표준 옛한글 자소의 호환 PUA 글리프를 실제 공급 프로그램의 cmap로만 선택한다.
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

const FIRST: u32 = 0xe000;
const LAST: u32 = 0xf8ff;
const HANGUL_FIRST: u32 = 0xac00;
const HANGUL_LAST: u32 = 0xd7a3;
const PUA_BITS: u32 = LAST - FIRST + 1;
const RANGES: [(u32, u32); 5] = [
    (FIRST, LAST),
    (HANGUL_FIRST, HANGUL_LAST),
    (0x1100, 0x11ff),
    (0xa960, 0xa97f),
    (0xd7b0, 0xd7ff),
];
const WORDS: usize =
    ((PUA_BITS + HANGUL_LAST - HANGUL_FIRST + 1 + 256 + 32 + 80).div_ceil(64)) as usize;
// 보조 cmap 비트만 보관한다. 실제 공급 catalog의 별칭·스타일도 유한하게 수용한다.
const MAX_FACES: usize = 4096;
const MAX_SOURCE_BYTES: usize = 32 * 1024 * 1024;

/// 모호한 역매핑은 선택하지 않는다. 원문·표시 문자열·문자 주소는 변경하지 않는다.
pub(crate) fn candidate(cluster: &str) -> Option<char> {
    if let Some(syllable) = modern_syllable(cluster) {
        return Some(syllable);
    }
    if super::contains_old_hangul_jamo(cluster) && cluster.chars().count() == 1 {
        return cluster.chars().next();
    }
    pua_candidate(cluster)
}

/// 단독 자소의 원 cmap가 없을 때도 유일한 원 PUA가 있으면 사용한다.
pub(crate) fn fallback_candidate(cluster: &str) -> Option<char> {
    pua_candidate(cluster).filter(|glyph| Some(*glyph) != candidate(cluster))
}

fn pua_candidate(cluster: &str) -> Option<char> {
    if !super::contains_old_hangul_jamo(cluster) {
        return None;
    }
    static REVERSE: OnceLock<HashMap<String, Option<char>>> = OnceLock::new();
    REVERSE
        .get_or_init(|| {
            let mut map = HashMap::new();
            for cp in FIRST..=LAST {
                let ch = char::from_u32(cp).unwrap();
                if let Some(chars) = super::pua_oldhangul::map_pua_old_hangul(ch) {
                    let key: String = chars.iter().collect();
                    map.entry(key)
                        .and_modify(|value| *value = None)
                        .or_insert(Some(ch));
                }
            }
            map
        })
        .get(cluster)
        .copied()
        .flatten()
}

/// Unicode Hangul L/V/T의 정규 합성. paint 후보만 반환하며 원문을 정규화하지 않는다.
pub(crate) fn modern_syllable(cluster: &str) -> Option<char> {
    let mut chars = cluster.chars();
    let leading = (chars.next()? as u32)
        .checked_sub(0x1100)
        .filter(|n| *n < 19)?;
    let vowel = (chars.next()? as u32)
        .checked_sub(0x1161)
        .filter(|n| *n < 21)?;
    let trailing = match chars.next() {
        Some(ch) => (ch as u32)
            .checked_sub(0x11a7)
            .filter(|n| (1..28).contains(n))?,
        None => 0,
    };
    if chars.next().is_some() {
        return None;
    }
    char::from_u32(HANGUL_FIRST + (leading * 21 + vowel) * 28 + trailing)
}

fn bit_index(ch: char) -> Option<usize> {
    let cp = ch as u32;
    let mut offset = 0;
    for (first, last) in RANGES {
        if (first..=last).contains(&cp) {
            return Some(offset + (cp - first) as usize);
        }
        offset += (last - first + 1) as usize;
    }
    None
}

type FaceKey = (String, u16, bool);

/// 글꼴 바이트나 윤곽선을 복제하지 않고 공급 face별 가시 PUA 비트만 보관한다.
#[derive(Clone, Default)]
pub(crate) struct FontCoverage {
    faces: HashMap<FaceKey, Arc<[u64; WORDS]>>,
    pub(crate) text_faces: super::text_font_coverage::TextFontCoverage,
}

fn primary(family: &str) -> String {
    family
        .split(',')
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches(['\'', '"'])
        .to_lowercase()
}

impl FontCoverage {
    /// 미검증 스타일이 같은 family의 이전 양수 coverage를 재사용하지 않게 한다.
    /// 선택적 보조 정보의 상한 때문에 브라우저의 정상 글꼴 적재를 실패시키지 않는다.
    fn disable_family(&mut self, family: String) -> bool {
        self.text_faces.disable_family(&family);
        let key = (family.clone(), 0, false);
        if self
            .faces
            .get(&key)
            .is_some_and(|mask| mask.iter().all(|word| *word == 0))
        {
            return false;
        }
        self.faces.retain(|(name, _, _), _| name != &family);
        if self.faces.len() < MAX_FACES {
            self.faces.insert(key, Arc::new([0; WORDS]));
        }
        false
    }

    pub(crate) fn register(
        &mut self,
        family: &str,
        bytes: &[u8],
        index: u32,
    ) -> Result<bool, &'static str> {
        if family.is_empty() || family.len() > 256 || family.chars().any(char::is_control) {
            return Err("invalid text font family");
        }
        let family = primary(family);
        if bytes.is_empty() {
            return Err("empty text font source");
        }
        if bytes.len() > MAX_SOURCE_BYTES {
            return Ok(self.disable_family(family));
        }
        let Ok(face) = ttf_parser::Face::parse(bytes, index) else {
            return Ok(self.disable_family(family));
        };
        self.text_faces.register(&family, bytes, index, &face);
        let key = (family, face.weight().to_number(), face.is_italic());
        if !self.faces.contains_key(&key) && self.faces.len() >= MAX_FACES {
            return Ok(self.disable_family(key.0));
        }
        let mut coverage = [0u64; WORDS];
        for cp in RANGES.into_iter().flat_map(|(first, last)| first..=last) {
            let Some(glyph) = face.glyph_index(char::from_u32(cp).unwrap()) else {
                continue;
            };
            // 이름·glyph ID 존재만으로 빈 GSUB 대상을 가시 글리프로 인정하지 않는다.
            if glyph.0 == 0 || face.glyph_bounding_box(glyph).is_none() {
                continue;
            }
            let bit = bit_index(char::from_u32(cp).unwrap()).unwrap();
            coverage[bit / 64] |= 1 << (bit % 64);
        }
        if self
            .faces
            .get(&key)
            .is_some_and(|old| old.as_ref() == &coverage)
        {
            return Ok(false);
        }
        self.faces.insert(key, Arc::new(coverage));
        Ok(true)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn has_style(&self, family: &str, bold: bool, italic: bool) -> bool {
        let family = primary(family);
        self.faces.keys().any(|(name, weight, slanted)| {
            name == &family && (*weight >= 600) == bold && *slanted == italic
        })
    }

    pub(crate) fn glyph(&self, family: &str, cluster: &str) -> Option<char> {
        candidate(cluster)
            .into_iter()
            .chain(fallback_candidate(cluster))
            .find(|&glyph| self.covers(family, glyph))
    }

    fn covers(&self, family: &str, glyph: char) -> bool {
        let family = primary(family);
        let Some(bit) = bit_index(glyph) else {
            return false;
        };
        let mut found = false;
        for ((name, _, _), coverage) in &self.faces {
            if name != &family {
                continue;
            }
            found = true;
            // 실제 공급한 스타일들이 모두 같은 호환 글리프를 가질 때만 고른다.
            if coverage[bit / 64] & (1 << (bit % 64)) == 0 {
                return false;
            }
        }
        found
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Candidate {
    pub start: usize,
    pub end: usize,
    pub glyph: char,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_glyph: Option<char>,
}

pub(crate) fn candidates(text: &str) -> (Vec<Candidate>, bool) {
    let candidates = super::layout::split_into_clusters(text)
        .into_iter()
        .filter_map(|(start, cluster)| {
            candidate(&cluster).map(|glyph| Candidate {
                start,
                end: start + cluster.chars().count(),
                glyph,
                fallback_glyph: fallback_candidate(&cluster),
            })
        })
        .take(crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN + 1)
        .collect::<Vec<_>>();
    if candidates.len() > crate::paint::MAX_POSITIONED_CONTROL_MARKS_PER_RUN {
        (Vec::new(), false)
    } else {
        (candidates, true)
    }
}
