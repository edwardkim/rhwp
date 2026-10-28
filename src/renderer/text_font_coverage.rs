//! 실제 적재 프로그램의 문자 범위·스타일만 보관하는 paint용 face 선택.
//! 프로그램 바이트와 윤곽은 보존하지 않고, 문자 전진·조판 slot도 변경하지 않는다.
use resvg::usvg::fontdb::{
    Database, FaceInfo, Family, Language, Query, Source, Stretch, Style, Weight, ID,
};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

const MAX_FACES: usize = 4096;
const MAX_CODEPOINTS: usize = 262_144;
const MAX_RANGES: usize = 262_144;
type Ranges = Arc<[(u32, u32)]>;
type FaceKey = (String, u16, Style);

#[derive(Clone, Default)]
struct Catalog {
    db: Database,
    keys: HashMap<FaceKey, ID>,
    coverage: HashMap<ID, Ranges>,
    programs: HashMap<([u8; 32], u32), Ranges>,
    unknown: HashSet<String>,
    ranges: usize,
    disabled: bool,
}

#[derive(Clone, Default)]
pub(crate) struct TextFontCoverage(Arc<Catalog>);

pub(crate) struct PaintFace {
    pub family: String,
    pub slanted: bool,
}

impl TextFontCoverage {
    pub fn disable_family(&mut self, family: &str) {
        let catalog = Arc::make_mut(&mut self.0);
        if catalog.unknown.len() < MAX_FACES {
            catalog.unknown.insert(family.to_owned());
        } else if !catalog.unknown.contains(family) {
            catalog.disabled = true;
        }
    }

    pub fn register(
        &mut self,
        family: &str,
        bytes: &[u8],
        index: u32,
        face: &ttf_parser::Face<'_>,
    ) {
        let catalog = Arc::make_mut(&mut self.0);
        if catalog.disabled || catalog.unknown.contains(family) {
            return;
        }
        let style = if face.is_oblique() {
            Style::Oblique
        } else if face.is_italic() {
            Style::Italic
        } else {
            Style::Normal
        };
        let key = (family.to_owned(), face.weight().to_number(), style);
        if !catalog.keys.contains_key(&key) && catalog.keys.len() >= MAX_FACES {
            self.disable_family(family);
            return;
        }
        let program = (Sha256::digest(bytes).into(), index);
        let ranges = if let Some(ranges) = catalog.programs.get(&program) {
            ranges.clone()
        } else {
            let Some(ranges) = character_ranges(face) else {
                self.disable_family(family);
                return;
            };
            if catalog.ranges + ranges.len() > MAX_RANGES {
                self.disable_family(family);
                return;
            }
            catalog.ranges += ranges.len();
            let ranges: Ranges = ranges.into();
            catalog.programs.insert(program, ranges.clone());
            ranges
        };
        if let Some(id) = catalog.keys.get(&key).copied() {
            // 같은 family/style의 부분 face들은 브라우저의 같은 문자 후보군이다.
            // 내용이 다른 중복 프로그램은 확정하지 않고 기존 브라우저 경로에 맡긴다.
            if catalog.coverage.get(&id).is_some_and(|old| old != &ranges) {
                self.disable_family(family);
            }
            return;
        }
        let id = catalog.db.push_face_info(FaceInfo {
            id: ID::dummy(),
            source: Source::Binary(Arc::new(Vec::<u8>::new())),
            index: 0,
            families: vec![(family.to_owned(), Language::English_UnitedStates)],
            post_script_name: family.to_owned(),
            style,
            weight: Weight(key.1),
            stretch: Stretch::Normal,
            monospaced: face.is_monospaced(),
        });
        catalog.keys.insert(key, id);
        catalog.coverage.insert(id, ranges);
    }

    /// fontdb의 family/style/weight 선택과 실제 cmap를 함께 사용한다.
    /// 미수집·상한 초과 face는 정확한 선택이라고 주장하지 않는다.
    pub fn select(
        &self,
        families: &str,
        text: &str,
        bold: bool,
        italic: bool,
    ) -> Option<PaintFace> {
        if self.0.disabled {
            return None;
        }
        // Canvas/SVG에 전달한 CSS 체인의 인용·이스케이프는 family 이름이 아니다.
        // 내부의 비인용 목록 분리기를 재사용하면 적재된 face를 찾지 못한다.
        for family in svgtypes::parse_font_families(families).ok()? {
            let svgtypes::FontFamily::Named(family) = family else {
                // 플랫폼 generic의 실제 선택은 이 세션의 등록 정보만으로 확정할 수 없다.
                return None;
            };
            let family = family.to_lowercase();
            if self.0.unknown.contains(&family) {
                return None;
            }
            let list = [Family::Name(&family)];
            let id = self.0.db.query(&Query {
                families: &list,
                weight: if bold { Weight::BOLD } else { Weight::NORMAL },
                style: if italic { Style::Italic } else { Style::Normal },
                ..Default::default()
            });
            let Some(id) = id else {
                continue;
            };
            let ranges = self.0.coverage.get(&id)?;
            let covered = text.chars().all(|ch| {
                if ch.is_whitespace() || ch.is_control() {
                    return true;
                }
                let cp = ch as u32;
                let at = ranges.partition_point(|(_, end)| *end < cp);
                ranges
                    .get(at)
                    .is_some_and(|(start, end)| *start <= cp && cp <= *end)
            });
            if covered {
                return Some(PaintFace {
                    family,
                    slanted: self.0.db.face(id)?.style != Style::Normal,
                });
            }
        }
        None
    }
}

fn character_ranges(face: &ttf_parser::Face<'_>) -> Option<Vec<(u32, u32)>> {
    let mut points = Vec::new();
    let mut overflow = false;
    for table in face.tables().cmap?.subtables {
        if !table.is_unicode() {
            continue;
        }
        table.codepoints(|cp| {
            if points.len() == MAX_CODEPOINTS {
                overflow = true;
                return;
            }
            if char::from_u32(cp).is_some() && table.glyph_index(cp).is_some_and(|id| id.0 != 0) {
                points.push(cp);
            }
        });
        if overflow {
            return None;
        }
    }
    points.sort_unstable();
    points.dedup();
    let mut ranges: Vec<(u32, u32)> = Vec::new();
    for cp in points {
        if let Some(last) = ranges.last_mut().filter(|last| last.1 + 1 == cp) {
            last.1 = cp;
        } else {
            ranges.push((cp, cp));
        }
    }
    Some(ranges)
}
