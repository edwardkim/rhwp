//! 메타파일 paint가 실제 공급된 SFNT와 font 세대를 공유한다. 문서 측정 slot과 분리한다.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex};

use resvg::usvg::fontdb::{Database, Family, Source, Style, Weight, ID};
use serde::{Deserialize, Serialize};

use super::metafile_surface::SurfaceCache;

const MAX_FONT_BYTES: usize = 32 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 64 * 1024 * 1024;
const MAX_ALIASES: usize = 256;

#[derive(Clone, Default)]
pub(crate) struct MetafileFonts {
    database: Option<Arc<Database>>,
    sources: HashMap<[u8; 32], Arc<Vec<u8>>>,
    aliases: HashMap<(String, u16, u8), ([u8; 32], u32, ID)>,
    pub(super) fingerprint: [u8; 32],
    pub(super) surfaces: Arc<Mutex<SurfaceCache>>,
    requests: Arc<Mutex<Option<(u32, String)>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct FontRequest {
    pub families: Vec<String>,
    pub weight: u16,
    pub italic: bool,
    pub text: String,
}

pub(super) fn family(name: &str) -> Family<'_> {
    match name {
        "serif" => Family::Serif,
        "sans-serif" => Family::SansSerif,
        "monospace" => Family::Monospace,
        "cursive" => Family::Cursive,
        "fantasy" => Family::Fantasy,
        _ => Family::Name(name),
    }
}

pub(crate) fn requests(svg: &[u8]) -> Option<Vec<FontRequest>> {
    let xml = roxmltree::Document::parse(std::str::from_utf8(svg).ok()?).ok()?;
    let mut groups = BTreeMap::<(Vec<String>, u16, bool), HashSet<char>>::new();
    for node in xml.descendants().filter(|n| n.has_tag_name("text")) {
        let names =
            svgtypes::parse_font_families(node.attribute("font-family").unwrap_or("sans-serif"))
                .ok()?
                .into_iter()
                .map(|name| match name {
                    svgtypes::FontFamily::Named(value) => value,
                    other => other.to_string(),
                })
                .collect::<Vec<_>>();
        let weight = match node.attribute("font-weight").unwrap_or("400") {
            "bold" => 700,
            value => value.parse().ok()?,
        };
        let italic = matches!(node.attribute("font-style"), Some("italic" | "oblique"));
        let chars = groups.entry((names, weight, italic)).or_default();
        for text in node
            .descendants()
            .filter_map(|n| n.is_text().then(|| n.text()).flatten())
        {
            chars.extend(
                text.chars()
                    .filter(|c| !c.is_whitespace() && !c.is_control()),
            );
        }
    }
    Some(
        groups
            .into_iter()
            .map(|((families, weight, italic), chars)| {
                let mut chars = chars.into_iter().collect::<Vec<_>>();
                chars.sort_unstable();
                FontRequest {
                    families,
                    weight,
                    italic,
                    text: chars.into_iter().collect(),
                }
            })
            .collect(),
    )
}

fn configure_generics(database: &mut Database) {
    let first = |names: &[&str]| {
        names.iter().find_map(|name| {
            database
                .faces()
                .any(|f| f.families.iter().any(|(family, _)| family == name))
                .then_some((*name).to_owned())
        })
    };
    let sans = first(&[
        "Noto Sans CJK KR",
        "Noto Sans KR",
        "HCR Dotum",
        "Malgun Gothic",
        "DejaVu Sans",
    ]);
    let serif = first(&[
        "Noto Serif CJK KR",
        "Noto Serif KR",
        "HCR Batang",
        "DejaVu Serif",
    ]);
    let mono = first(&["GulimChe", "D2Coding", "DejaVu Sans Mono"]);
    if let Some(name) = sans {
        database.set_sans_serif_family(name);
    }
    if let Some(name) = serif {
        database.set_serif_family(name);
    }
    if let Some(name) = mono {
        database.set_monospace_family(name);
    }
}

fn default_database() -> Arc<Database> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::OnceLock;
        static DATABASE: OnceLock<Arc<Database>> = OnceLock::new();
        DATABASE
            .get_or_init(|| {
                let mut database = Database::new();
                database.load_system_fonts();
                crate::renderer::font_paths::load_into_fontdb(&mut database, &[]);
                configure_generics(&mut database);
                Arc::new(database)
            })
            .clone()
    }
    #[cfg(target_arch = "wasm32")]
    {
        Arc::new(Database::new())
    }
}

impl MetafileFonts {
    /// 세션에 명시적으로 공급한 프로그램만 새 텍스트 효과의 근거로 쓴다.
    pub(super) fn is_supplied_face(&self, id: ID) -> bool {
        self.aliases
            .values()
            .any(|(_, _, supplied)| *supplied == id)
    }

    pub(crate) fn cached_requests(&self, epoch: u32) -> Option<String> {
        self.requests
            .lock()
            .ok()?
            .as_ref()
            .filter(|(value, _)| *value == epoch)
            .map(|(_, json)| json.clone())
    }

    pub(crate) fn cache_requests(&self, epoch: u32, json: &str) {
        if let Ok(mut cache) = self.requests.lock() {
            *cache = Some((epoch, json.to_owned()));
        }
    }

    pub(super) fn database(&self) -> Arc<Database> {
        self.database.clone().unwrap_or_else(default_database)
    }

    pub(crate) fn register(
        &mut self,
        name: &str,
        bytes: &[u8],
        face_index: u32,
    ) -> Result<bool, &'static str> {
        let name = name.trim();
        if name.is_empty() || name.len() > 1024 || name.chars().any(char::is_control) {
            return Err("메타파일 글꼴 family가 유효하지 않습니다");
        }
        if bytes.len() > MAX_FONT_BYTES {
            return Err("메타파일 글꼴의 바이트 상한을 초과했습니다");
        }
        let parsed = ttf_parser::Face::parse(bytes, face_index)
            .map_err(|_| "메타파일 글꼴 face를 읽을 수 없습니다")?;
        let key = (
            name.to_lowercase(),
            parsed.weight().to_number(),
            if parsed.is_italic() {
                1
            } else if parsed.is_oblique() {
                2
            } else {
                0
            },
        );
        let digest = *blake3::hash(bytes).as_bytes();
        if self
            .aliases
            .get(&key)
            .is_some_and(|(hash, index, _)| *hash == digest && *index == face_index)
        {
            return Ok(false);
        }
        if self.aliases.len() >= MAX_ALIASES && !self.aliases.contains_key(&key) {
            return Err("메타파일 글꼴 alias 상한을 초과했습니다");
        }
        let source = if let Some(data) = self.sources.get(&digest) {
            data.clone()
        } else {
            let used: usize = self.sources.values().map(|v| v.len()).sum();
            if used.saturating_add(bytes.len()) > MAX_TOTAL_BYTES {
                return Err("메타파일 글꼴의 총 바이트 상한을 초과했습니다");
            }
            Arc::new(bytes.to_vec())
        };
        let mut temporary = Database::new();
        temporary.load_font_source(Source::Binary(source.clone()));
        let mut face = temporary
            .faces()
            .find(|f| f.index == face_index)
            .cloned()
            .ok_or("메타파일 글꼴의 명명 face를 읽을 수 없습니다")?;
        // 이 등록은 family 하나에 대한 명시 binding이다. 원본의 다른 alias를
        // 함께 노출하면 후속 alias 등록이 앞서 등록한 다른 binding까지 지울 수 있다.
        // 원래 family 이름과 프로그램 자체는 공급된 SFNT 바이트에 보존된다.
        let language = face
            .families
            .first()
            .ok_or("메타파일 글꼴의 family가 없습니다")?
            .1;
        face.families = vec![(name.to_owned(), language)];
        let db = self.database.get_or_insert_with(default_database);
        let db = Arc::make_mut(db);
        // 명시적으로 공급한 실제 face가 같은 family/style의 오래된 프로그램을 대체한다.
        let old = db
            .faces()
            .filter(|old| {
                old.weight == face.weight
                    && old.style == face.style
                    && old
                        .families
                        .iter()
                        .any(|(family, _)| family.eq_ignore_ascii_case(name))
            })
            .map(|f| f.id)
            .collect::<Vec<_>>();
        for id in old {
            db.remove_face(id);
        }
        let id = db.push_face_info(face);
        configure_generics(db);
        self.sources.insert(digest, source);
        self.aliases.insert(key, (digest, face_index, id));
        let mut hash = blake3::Hasher::new();
        hash.update(&self.fingerprint);
        hash.update(name.as_bytes());
        hash.update(&digest);
        hash.update(&face_index.to_le_bytes());
        self.fingerprint = *hash.finalize().as_bytes();
        Ok(true)
    }

    /// 없는 글자를 투명한 출력으로 확정하기 전에 실제 선택 후보의 cmap을 대조한다.
    pub(super) fn coverage_fingerprint(&self, requests: &[FontRequest]) -> Option<[u8; 32]> {
        let database = self.database();
        let mut sources = blake3::Hasher::new();
        for request in requests {
            let mut missing = request.text.chars().collect::<HashSet<_>>();
            for name in &request.families {
                let families = [family(name)];
                let Some(id) = database.query(&resvg::usvg::fontdb::Query {
                    families: &families,
                    weight: Weight(request.weight),
                    style: if request.italic {
                        Style::Italic
                    } else {
                        Style::Normal
                    },
                    ..Default::default()
                }) else {
                    continue;
                };
                database.with_face_data(id, |data, index| {
                    if let Ok(face) = ttf_parser::Face::parse(data, index) {
                        // 시스템 파일도 실제 사용한 bytes로 식별한다. 같은 경로의
                        // 프로그램 교체가 이전 surface의 재사용으로 이어지면 안 된다.
                        sources.update(blake3::hash(data).as_bytes());
                        sources.update(&index.to_le_bytes());
                        missing.retain(|ch| !face.glyph_index(*ch).is_some_and(|g| g.0 != 0));
                    }
                });
                if missing.is_empty() {
                    break;
                }
            }
            if !missing.is_empty() {
                return None;
            }
        }
        Some(*sources.finalize().as_bytes())
    }
}
