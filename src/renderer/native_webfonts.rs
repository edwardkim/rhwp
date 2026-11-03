//! Native 출력용 공개 글꼴 공급. 문서 이름은 URL을 만들지 않는다.
//!
//! 호출자가 지정한 글꼴과 설치된 글꼴 프로그램을 먼저 확인하며, 고정된 공급
//! 원장에 있는 글꼴만 사용자 캐시에 내려받는다.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::font_environment::FontEnvironment;
use crate::model::document::DocInfo;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const REGISTRY: &str = include_str!("../../assets/font-rules/font_rule_registry_v2.json");
const MAX_FONT_BYTES: usize = 32 * 1024 * 1024;
const GENERIC_WEBFONT: &str = "fonts/NotoSansKR-Regular.woff2";
const PROJECT_FONT_COMMIT: &str = "7d06d5c706f1d89f1662e739abf2b1b25d2a98e4";
const NOTO_SERIF: &str = "https://cdn.jsdelivr.net/gh/google/fonts@2fe1e8c6b137fb13041a7cfca6fdcd8a910ea4dc/ofl/notoserif/NotoSerif%5Bwdth,wght%5D.ttf";
const HANCOM_BATANG: &str =
    "https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff";

/// 실제 공급 결과. 미지원·네트워크 실패는 출력하지 못한 이름으로 구분한다.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeWebfontSupply {
    pub paths: Vec<PathBuf>,
    pub mappings: BTreeMap<String, String>,
    pub generic_fallbacks: Vec<String>,
    pub glyph_fallbacks: Vec<serde_json::Value>,
    pub unavailable: Vec<String>,
    pub web_fonts: BTreeMap<String, String>,
    pub web_font_styles: BTreeMap<String, NativeWebfontStyle>,
    pub resources: Vec<NativeWebfontResource>,
}

/// CSS에도 실제 프로그램의 굵기·기울기를 전달해 중복 합성을 피한다.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeWebfontStyle {
    pub weight: u16,
    pub italic: bool,
    pub weight_range: Option<(f32, f32)>,
}

/// 다운로드 원본과 Native 변환본의 신원을 구분한다.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeWebfontResource {
    pub url: String,
    pub source_sha256: String,
    pub native_sha256: String,
    pub native_bytes: usize,
}

impl NativeWebfontSupply {
    /// SVG를 여는 호스트에서도 같은 공급원을 읽도록 한다. 로컬 face가 우선한다.
    pub fn svg_style(&self) -> String {
        let mut css = String::new();
        for (family, url) in &self.web_fonts {
            let style = self.web_font_styles.get(family);
            let weight = style
                .map(|style| {
                    style
                        .weight_range
                        .map(|(min, max)| format!("{min} {max}"))
                        .unwrap_or_else(|| style.weight.to_string())
                })
                .unwrap_or_else(|| "400".into());
            let slant = if style.is_some_and(|style| style.italic) {
                "italic"
            } else {
                "normal"
            };
            let family = serde_json::to_string(family).unwrap();
            let url = serde_json::to_string(url).unwrap();
            css.push_str(&format!(
                "@font-face{{font-family:{family};font-weight:{weight};font-style:{slant};src:local({family}),url({url});}}\n"
            ));
        }
        if css.is_empty() {
            String::new()
        } else {
            super::svg::svg_font_style_element(&css)
        }
    }
}

fn normalized(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn cache_dir() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("RHWP_WEBFONT_CACHE").filter(|p| !p.is_empty()) {
        return Ok(PathBuf::from(path));
    }
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("LOCALAPPDATA").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".cache")))
        .ok_or("사용자 캐시 경로가 없습니다")?;
    Ok(base.join("rhwp").join("webfonts"))
}

fn source_url(source: &str) -> Option<String> {
    // 일반 공개 웹폰트의 부분집합은 원본 글리프를 빠뜨릴 수 있다.
    // KoPub의 경우 실제 설치본과 바이트가 같은 공개 TTF를 쓴다.
    if let Some(name) = source.rsplit('/').next() {
        let family = if name.starts_with("KoPubBatang-") {
            "KoPubBatang"
        } else if name.starts_with("KoPubDotum-") {
            "KoPubDotum"
        } else {
            ""
        };
        if !family.is_empty() && name.ends_with(".woff") {
            let revision = if family == "KoPubBatang" {
                "7ef37ed4015cf0f638952e3d9937e4192f7cea97"
            } else {
                "295e0fc83931fda749f129b6690c0fa93f798007"
            };
            return Some(format!(
                "https://cdn.jsdelivr.net/gh/fonts-archive/{family}@{revision}/{}.ttf",
                name.trim_end_matches(".woff")
            ));
        }
    }
    if let Some(name) = source.strip_prefix("fonts/") {
        if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
            return None;
        }
        return Some(format!(
            "https://cdn.jsdelivr.net/gh/edwardkim/rhwp@{PROJECT_FONT_COMMIT}/assets/fonts/{name}"
        ));
    }
    if source.starts_with("https://cdn.jsdelivr.net/") && !source.contains(['?', '#', '\\']) {
        return Some(source.to_owned());
    }
    None
}

fn expected_kopub_hash(url: &str) -> Option<&'static str> {
    match url.rsplit('/').next()? {
        "KoPubBatang-Light.ttf" => {
            Some("b7438d61595fe9584706ab2cb3c9aa2acdc486d61c8299374a30b3e77bc2045f")
        }
        "KoPubBatang-Medium.ttf" => {
            Some("7a798592beb0d35bb557b010c2794cbcc988e89fcbcee0eb5edbb35161141ab4")
        }
        "KoPubBatang-Bold.ttf" => {
            Some("94de26a11aa25619526ac8183fde4f46e74e22c527acb42aea2e93f5313bbc5b")
        }
        "KoPubDotum-Light.ttf" => {
            Some("018a0f18613a98d446696ea3d36194e288d5d8004aa58327789c830f6179e23d")
        }
        "KoPubDotum-Medium.ttf" => {
            Some("d0adfdcb40b7e73389128cc0222b53a91e5aa0e653f89967485a1f8b46124eb7")
        }
        "KoPubDotum-Bold.ttf" => {
            Some("967efe18063dd971c63dde3e8fcb247e0fcf16486a24ba2e284a716e44b90275")
        }
        "NotoSerif%5Bwdth,wght%5D.ttf" => {
            Some("4d8e6761424656867019081a1a01336f3cb086982682698714054fc33f782713")
        }
        _ => None,
    }
}

/// WOFF/WOFF2 또는 SFNT를 크기 제한과 함께 Native SFNT로 읽는다.
pub fn decode_font(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.len() < 20 || bytes.len() > MAX_FONT_BYTES {
        return Err("글꼴 크기가 허용 범위 밖입니다".into());
    }
    let magic = &bytes[..4];
    let font = if magic == b"wOFF" || magic == b"wOF2" {
        let expected = u32::from_be_bytes(bytes[16..20].try_into().unwrap()) as usize;
        if expected == 0 || expected > MAX_FONT_BYTES {
            return Err("압축 해제 글꼴 크기가 허용 범위 밖입니다".into());
        }
        if magic == b"wOFF" {
            wuff::decompress_woff1(bytes)
        } else {
            wuff::decompress_woff2(bytes)
        }
        .map_err(|e| format!("웹폰트 변환 실패: {e:?}"))?
    } else {
        bytes.to_vec()
    };
    if font.len() > MAX_FONT_BYTES || ttf_parser::Face::parse(&font, 0).is_err() {
        return Err("유효한 Native 글꼴이 아닙니다".into());
    }
    Ok(font)
}

fn prepare_resource(source: &str, cache: &Path) -> Result<(PathBuf, String), String> {
    let url = source_url(source).ok_or("허용되지 않은 글꼴 공급 주소")?;
    let folder = cache.join(hex(url.as_bytes()));
    let program = folder.join("font.otf");
    let receipt_path = folder.join("resource.json");
    let cached = (|| {
        let metadata = std::fs::symlink_metadata(&program).ok()?;
        if !metadata.is_file() || metadata.len() > MAX_FONT_BYTES as u64 {
            return None;
        }
        let receipt_metadata = std::fs::symlink_metadata(&receipt_path).ok()?;
        if !receipt_metadata.is_file() || receipt_metadata.len() > 64 * 1024 {
            return None;
        }
        let receipt: NativeWebfontResource =
            serde_json::from_slice(&std::fs::read(&receipt_path).ok()?).ok()?;
        let data = std::fs::read(&program).ok()?;
        (receipt.url == url
            && receipt.native_bytes == data.len()
            && receipt.native_sha256 == hex(&data)
            && expected_kopub_hash(&url).is_none_or(|expected| receipt.source_sha256 == expected)
            && decode_font(&data).is_ok())
        .then_some(data)
    })();
    let data = match cached {
        Some(data) => data,
        None => {
            if std::env::var_os("RHWP_WEBFONT_OFFLINE").is_some_and(|value| value == "1") {
                return Err("오프라인 모드이며 검증된 캐시가 없습니다".into());
            }
            let agent: ureq::Agent = ureq::Agent::config_builder()
                .timeout_global(Some(Duration::from_secs(15)))
                .max_redirects(0)
                .build()
                .into();
            let mut response = agent
                .get(&url)
                .call()
                .map_err(|e| format!("글꼴 다운로드 실패: {e}"))?;
            let bytes = response
                .body_mut()
                .with_config()
                .limit(MAX_FONT_BYTES as u64)
                .read_to_vec()
                .map_err(|e| format!("글꼴 읽기 실패: {e}"))?;
            if expected_kopub_hash(&url).is_some_and(|expected| hex(&bytes) != expected) {
                return Err("공개 원본 글꼴의 SHA-256이 다릅니다".into());
            }
            let data = decode_font(&bytes)?;
            std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
            atomic_write(&program, &data)?;
            let receipt = NativeWebfontResource {
                url: url.clone(),
                source_sha256: hex(&bytes),
                native_sha256: hex(&data),
                native_bytes: data.len(),
            };
            atomic_write(
                &receipt_path,
                &serde_json::to_vec(&receipt).map_err(|e| e.to_string())?,
            )?;
            data
        }
    };
    let mut database = usvg::fontdb::Database::new();
    database.load_font_data(data);
    let face = database.faces().next().ok_or("글꼴 이름이 없습니다")?;
    let name = if face.post_script_name.is_empty() {
        face.families
            .first()
            .map(|family| family.0.clone())
            .ok_or("글꼴 family가 없습니다")?
    } else {
        face.post_script_name.clone()
    };
    Ok((folder, name))
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    let mut nonce = [0u8; 16];
    getrandom::fill(&mut nonce).map_err(|e| e.to_string())?;
    let temp = path.with_file_name(format!(".font-{}-{}", std::process::id(), hex(&nonce)));
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|e| e.to_string())?;
    if let Err(error) = file.write_all(data) {
        let _ = std::fs::remove_file(&temp);
        return Err(error.to_string());
    }
    drop(file);
    if let Err(error) = std::fs::rename(&temp, path) {
        // Windows는 기존 파일을 rename으로 교체하지 못한다. 이 사용자 캐시만 교체한다.
        if path.is_file() {
            std::fs::remove_file(path).map_err(|e| e.to_string())?;
            if std::fs::rename(&temp, path).is_ok() {
                return Ok(());
            }
        }
        let _ = std::fs::remove_file(&temp);
        return Err(error.to_string());
    }
    Ok(())
}

/// 출력 시작 전 실행한다. 문서 글꼴 프로그램이 있으면 외부 공급을 건너뛴다.
pub fn prepare(doc_info: &DocInfo, paths: &[PathBuf]) -> Result<NativeWebfontSupply, String> {
    let registry: serde_json::Value = serde_json::from_str(REGISTRY).map_err(|e| e.to_string())?;
    let rules = registry["rules"]
        .as_array()
        .ok_or("글꼴 원장이 올바르지 않습니다")?;
    let mut database = usvg::fontdb::Database::new();
    for path in super::font_paths::search_dirs(paths) {
        if path.is_dir() {
            database.load_fonts_dir(path);
        } else if path.is_file() {
            let _ = database.load_font_file(path);
        }
    }
    let present = |name: &str| {
        let target = normalized(name);
        if database.faces().any(|face| {
            normalized(&face.post_script_name) == target
                || face
                    .families
                    .iter()
                    .any(|(family, _)| normalized(family) == target)
        }) {
            return true;
        }
        // PostScript 하이픈 차이는 단일 실제 face로 확인되는 경우에만 허용한다.
        let compact = target.replace('-', "");
        let matches: BTreeSet<_> = database
            .faces()
            .filter(|face| normalized(&face.post_script_name).replace('-', "") == compact)
            .map(|face| face.post_script_name.clone())
            .collect();
        matches.len() == 1
    };
    let embedded: BTreeSet<_> = doc_info
        .font_faces
        .iter()
        .flatten()
        .filter(|face| face.is_embedded)
        .map(|face| normalized(&face.name))
        .collect();
    let mut grouped = BTreeMap::<String, (Vec<String>, Vec<String>)>::new();
    for request in FontEnvironment::document_requests(doc_info) {
        let entry = grouped.entry(request.source).or_default();
        entry.0.extend(request.primary);
        entry.1.extend(request.substitute);
    }
    let cache = cache_dir().ok();
    let mut result = NativeWebfontSupply::default();
    let mut original_programs = BTreeSet::new();
    let mut priority_paths = Vec::new();
    let mut face_paths = BTreeMap::<String, PathBuf>::new();
    for (source, (primary, substitute)) in grouped {
        if embedded.contains(&normalized(&source)) || primary.iter().any(|face| present(face)) {
            continue;
        }
        let rule = [source.as_str()]
            .into_iter()
            .chain(primary.iter().map(String::as_str))
            .find_map(|name| {
                rules.iter().find(|r| {
                    r["status"] == "active"
                        && r["supply"]["kind"] == "canvas2d-webfont"
                        && r["supply"]["unicodeRange"].is_null()
                        && r["sourceFace"]
                            .as_str()
                            .is_some_and(|face| normalized(face) == normalized(name))
                })
            });
        let generic = rule.is_none();
        let url = if generic {
            // 원 프로그램의 공급과 일반 문자 대체는 구분한다. 문서가 선언한
            // 설치 대체 글꼴은 유지하고, 그것도 없을 때 공개 Noto를 사용한다.
            result.unavailable.push(source.clone());
            if substitute.iter().any(|face| present(face)) {
                continue;
            }
            if present("NotoSansKR-Regular") {
                result.generic_fallbacks.push(source.clone());
                result.mappings.insert(source, "NotoSansKR-Regular".into());
                continue;
            }
            GENERIC_WEBFONT
        } else {
            rule.and_then(|rule| rule["supply"]["sourceUrl"].as_str())
                .ok_or("등록된 공개 글꼴의 공급 주소가 없습니다")?
        };
        let supplied = cache
            .as_deref()
            .ok_or("사용자 캐시 경로가 없습니다".to_string())
            .and_then(|cache| prepare_resource(url, cache));
        match supplied {
            Ok((folder, face)) => {
                let resolved_url = source_url(url).ok_or("공개 글꼴 주소가 올바르지 않습니다")?;
                face_paths.insert(face.clone(), folder.clone());
                result.web_fonts.insert(face.clone(), resolved_url.clone());
                // KoPub의 고정 원본 TTF는 같은 프로그램의 공급이다. 전역 치환으로
                // 표시하면 원본 조판 메트릭의 소유가 대체 글꼴로 바뀐다.
                let original_kopub = source_url(url).as_deref().is_some_and(|url| {
                    url.contains("/fonts-archive/KoPub") && expected_kopub_hash(url).is_some()
                });
                if original_kopub {
                    for alias in std::iter::once(&source).chain(primary.iter()) {
                        face_paths.insert(alias.clone(), folder.clone());
                        result.web_fonts.insert(alias.clone(), resolved_url.clone());
                    }
                    priority_paths.push(folder);
                    original_programs.insert(source);
                } else {
                    result.paths.push(folder);
                    if generic {
                        result.generic_fallbacks.push(source.clone());
                    }
                    result.mappings.insert(source, face);
                }
            }
            Err(error) => result.unavailable.push(format!("{source}: {error}")),
        }
    }
    // 원본이 공급돼도 일부 문자는 문서가 선언한 보조 프로그램이 필요하다.
    // 언어 슬롯과 글꼴 레코드 주소를 보존하며 전역 치환으로 합치지 않는다.
    let mut resources = BTreeMap::<String, (PathBuf, String)>::new();
    for (slot, font_id, request) in FontEnvironment::document_requests_with_slots(doc_info) {
        if embedded.contains(&normalized(&request.source))
            || !(request.primary.iter().any(|face| present(face))
                || result.mappings.contains_key(&request.source)
                || original_programs.contains(&request.source))
            || request.substitute.is_empty()
            || request.substitute.iter().any(|face| present(face))
        {
            continue;
        }
        let Some(rule) = request.substitute.iter().find_map(|name| {
            rules.iter().find(|r| {
                r["status"] == "active"
                    && r["supply"]["kind"] == "canvas2d-webfont"
                    && r["supply"]["unicodeRange"].is_null()
                    && r["sourceFace"]
                        .as_str()
                        .is_some_and(|face| normalized(face) == normalized(name))
            })
        }) else {
            continue;
        };
        let Some(url) = rule["supply"]["sourceUrl"].as_str() else {
            continue;
        };
        if !resources.contains_key(url) {
            let supplied = cache
                .as_deref()
                .ok_or("사용자 캐시 경로가 없습니다".to_string())
                .and_then(|cache| prepare_resource(url, cache));
            match supplied {
                Ok(font) => {
                    resources.insert(url.to_owned(), font);
                }
                Err(error) => {
                    result
                        .unavailable
                        .push(format!("{}: {error}", request.source));
                    continue;
                }
            }
        }
        let (path, face) = resources.get(url).unwrap();
        priority_paths.push(path.clone());
        let mut faces = vec![face.clone()];
        if url == HANCOM_BATANG
            && slot == 5
            && matches!(
                normalized(&request.source).as_str(),
                "kopub바탕체light" | "kopubbatanglight"
            )
        {
            if !present("Noto Serif") {
                if !resources.contains_key(NOTO_SERIF) {
                    if let Some(cache) = cache.as_deref() {
                        match prepare_resource(NOTO_SERIF, cache) {
                            Ok(font) => {
                                resources.insert(NOTO_SERIF.into(), font);
                            }
                            Err(error) => result.unavailable.push(format!("Noto Serif: {error}")),
                        }
                    }
                }
                if let Some((path, _)) = resources.get(NOTO_SERIF) {
                    priority_paths.insert(0, path.clone());
                }
            }
            if present("Noto Serif") || resources.contains_key(NOTO_SERIF) {
                faces.insert(0, "Noto Serif".into());
            }
        } else if url == HANCOM_BATANG
            && slot == 0
            && matches!(
                normalized(&request.source).as_str(),
                "kopub돋움체light" | "kopubdotumlight"
            )
        {
            const NOTO_SANS: &str = "fonts/NotoSansKR-ExtraLight.woff2";
            if !present("Noto Sans KR ExtraLight") && !resources.contains_key(NOTO_SANS) {
                if let Some(cache) = cache.as_deref() {
                    match prepare_resource(NOTO_SANS, cache) {
                        Ok(font) => {
                            resources.insert(NOTO_SANS.into(), font);
                        }
                        Err(error) => result
                            .unavailable
                            .push(format!("Noto Sans KR ExtraLight: {error}")),
                    }
                }
            }
            if let Some((path, face)) = resources.get(NOTO_SANS) {
                priority_paths.insert(0, path.clone());
                faces.insert(0, face.clone());
            } else if present("Noto Sans KR ExtraLight") {
                faces.insert(0, "NotoSansKRExtraLight".into());
            }
        }
        result.glyph_fallbacks.push(serde_json::json!({"languageSlot":slot,"fontId":font_id,"source":request.source,"faces":faces}));
    }
    for (url, (path, face)) in &resources {
        let resolved = source_url(url).ok_or("문자 대체 글꼴 주소가 올바르지 않습니다")?;
        result.web_fonts.insert(face.clone(), resolved.clone());
        face_paths.insert(face.clone(), path.clone());
        if url == NOTO_SERIF {
            result.web_fonts.insert("Noto Serif".into(), resolved);
            face_paths.insert("Noto Serif".into(), path.clone());
        }
    }
    priority_paths.append(&mut result.paths);
    result.paths = priority_paths;
    let mut seen = BTreeSet::new();
    result.paths.retain(|path| seen.insert(path.clone()));
    // Skia의 기존 공급 한도와 같은 상수를 사용한다. 제한 밖 프로그램을
    // layout에만 넘겨 실제 paint도 공급됐다고 오인하지 않는다.
    let limit = super::font_paths::MAX_NATIVE_CUSTOM_FONT_BYTES;
    let mut used = 0u64;
    let mut retained_bytes = 0u64;
    let mut file_count = 0usize;
    let mut source_files = BTreeSet::new();
    for source in super::font_paths::custom_font_dirs(paths) {
        let mut files = if source.is_file() {
            vec![source]
        } else {
            std::fs::read_dir(source)
                .into_iter()
                .flatten()
                .flatten()
                .map(|entry| entry.path())
                .collect()
        };
        files.sort();
        for file in files {
            if !source_files.insert(file.clone())
                || !file.extension().and_then(|x| x.to_str()).is_some_and(|x| {
                    matches!(x.to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc")
                })
            {
                continue;
            }
            if let Ok(meta) = file.metadata() {
                if file_count < 256
                    && meta.is_file()
                    && meta.len() <= MAX_FONT_BYTES as u64
                    && used.saturating_add(meta.len()) <= limit
                {
                    used += meta.len();
                    file_count += 1;
                    if let Ok(data) = std::fs::read(&file) {
                        // 컬렉션은 Skia가 face마다 원본의 복사본을 보유한다.
                        // 파싱 가능한 face까지 보수적으로 예약해 공급을 과장하지 않는다.
                        let faces = ttf_parser::fonts_in_collection(&data).unwrap_or(1).min(64);
                        for index in 0..faces {
                            if retained_bytes.saturating_add(data.len() as u64) > limit {
                                break;
                            }
                            if ttf_parser::Face::parse(&data, index).is_ok() {
                                retained_bytes += data.len() as u64;
                            }
                        }
                    }
                }
            }
        }
    }
    result.paths.retain(|path| {
        let length = path
            .join("font.otf")
            .metadata()
            .map(|meta| meta.len())
            .unwrap_or(u64::MAX);
        if file_count < 256
            && used.saturating_add(length) <= limit
            && retained_bytes.saturating_add(length) <= limit
        {
            used += length;
            retained_bytes += length;
            file_count += 1;
            true
        } else {
            result
                .unavailable
                .push(format!("{}: Native 글꼴 공급 한도 초과", path.display()));
            false
        }
    });
    let retained: BTreeSet<_> = result.paths.iter().cloned().collect();
    let available_face = |face: &str| {
        present(face)
            || face_paths
                .get(face)
                .is_none_or(|path| retained.contains(path))
    };
    result.mappings.retain(|_, face| available_face(face));
    result
        .generic_fallbacks
        .retain(|source| result.mappings.contains_key(source));
    result.web_fonts.retain(|face, _| available_face(face));
    for (face_name, path) in &face_paths {
        if !result.web_fonts.contains_key(face_name) {
            continue;
        }
        if let Ok(data) = std::fs::read(path.join("font.otf")) {
            if let Ok(face) = ttf_parser::Face::parse(&data, 0) {
                let range = face
                    .variation_axes()
                    .into_iter()
                    .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wght"))
                    .map(|axis| (axis.min_value, axis.max_value));
                result.web_font_styles.insert(
                    face_name.clone(),
                    NativeWebfontStyle {
                        weight: face.weight().to_number(),
                        italic: face.is_italic(),
                        weight_range: range,
                    },
                );
            }
        }
    }
    for fallback in &mut result.glyph_fallbacks {
        if let Some(faces) = fallback["faces"].as_array_mut() {
            faces.retain(|face| face.as_str().is_some_and(available_face));
        }
    }
    result.glyph_fallbacks.retain(|entry| {
        entry["faces"]
            .as_array()
            .is_some_and(|faces| !faces.is_empty())
    });
    for path in &result.paths {
        if let Ok(data) = std::fs::read(path.join("resource.json")) {
            if let Ok(receipt) = serde_json::from_slice(&data) {
                result.resources.push(receipt);
            }
        }
    }
    result.unavailable.sort();
    result.unavailable.dedup();
    Ok(result)
}
