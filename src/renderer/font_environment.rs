//! Caller-declared font availability for one rendering session, never document IR.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::HwpError;

/// Explicit final face substitutions shared by layout and paint.
///
/// Keys are exact document font names. Targets are single installed/renderable
/// faces, not CSS fallback chains. Mappings are applied once, not recursively.
/// This declaration does not prove that the backend has the target font.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FontEnvironment {
    id: String,
    substitutions: BTreeMap<String, String>,
    /// 호출자가 실제 출력 프로그램을 확인한 TrueType 최종 face.
    /// 설치 증명은 아니며, 생략하면 기존 HFT 호환 메트릭을 유지한다.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    true_type_faces: Vec<String>,
    /// 호출자가 공급을 확인한 글리프 폴백. 키는 치환 후의 최종 face이며,
    /// 기본 face가 없는 글자만 이 순서로 측정·출력한다. IR에는 저장하지 않는다.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    glyph_fallbacks: BTreeMap<String, Vec<String>>,
}

/// 실제 프로그램 조회용 이름이다. generic·호환 대체는 설치 별칭에 포함하지 않는다.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentFontSelectionRequest {
    pub source: String,
    pub primary: Vec<String>,
    pub substitute: Vec<String>,
}

/// 출력 호스트가 실제로 로드한 프로그램. 형식 미확인 시 HFT 메트릭을 유지한다.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AvailableFontProgram {
    pub query: String,
    pub family: String,
    pub program_id: String,
    #[serde(default)]
    pub true_type: bool,
}

fn program_names(name: &str) -> Vec<String> {
    std::iter::once(name)
        .chain(super::installed_font_program_aliases(name).iter().copied())
        .filter(|name| FontEnvironment::valid_name(name))
        .map(str::to_owned)
        .collect()
}

impl FontEnvironment {
    fn valid_name(name: &str) -> bool {
        !name.is_empty()
            && name.trim() == name
            && name.len() <= 256
            && !name.chars().any(|character| {
                character.is_control()
                    || matches!(character, ',' | '\'' | '"' | '\\' | ';' | '{' | '}')
            })
    }

    /// 문서 선언과 프로그램 별칭을 한 번 생산해 Native와 브라우저에 전달한다.
    pub fn document_requests(
        doc_info: &crate::model::document::DocInfo,
    ) -> Vec<DocumentFontSelectionRequest> {
        let mut requests = Vec::new();
        // 같은 이름에 내장 프로그램이 있으면 전역 CSS 별칭으로 덮어쓰지 않는다.
        let embedded: std::collections::BTreeSet<_> = doc_info
            .font_faces
            .iter()
            .flatten()
            .filter(|font| font.is_embedded)
            .map(|font| &font.name)
            .collect();
        for (language, fonts) in doc_info.font_faces.iter().enumerate() {
            for (index, font) in fonts.iter().enumerate() {
                if embedded.contains(&font.name) || !Self::valid_name(&font.name) {
                    continue;
                }
                let decision = super::style_resolver::lookup_font_name_decision(
                    doc_info,
                    language,
                    index as u16,
                );
                let Some(primary) = decision.normalized_face.as_deref() else {
                    continue;
                };
                requests.push(DocumentFontSelectionRequest {
                    source: font.name.clone(),
                    primary: program_names(primary),
                    substitute: decision
                        .subst_font
                        .as_deref()
                        .map(program_names)
                        .unwrap_or_default(),
                });
            }
        }
        requests
    }

    fn from_document_lookup(
        doc_info: &crate::model::document::DocInfo,
        mut lookup: impl FnMut(&str) -> Option<AvailableFontProgram>,
    ) -> Result<Option<Self>, HwpError> {
        use sha2::{Digest, Sha256};
        // 같은 프로그램을 언어 슬롯마다 다시 읽거나 해시하지 않는다.
        let mut queried = BTreeMap::<String, Option<AvailableFontProgram>>::new();
        let mut resolve = |name: &str| {
            queried
                .entry(name.to_owned())
                .or_insert_with(|| lookup(name))
                .clone()
        };
        let mut candidates = BTreeMap::<String, Option<AvailableFontProgram>>::new();
        let embedded: std::collections::BTreeSet<_> = doc_info
            .font_faces
            .iter()
            .flatten()
            .filter(|font| font.is_embedded)
            .map(|font| &font.name)
            .collect();
        for request in Self::document_requests(doc_info) {
            let candidate = if request.primary.iter().any(|name| resolve(name).is_some()) {
                None
            } else {
                request
                    .substitute
                    .iter()
                    .find_map(|name| resolve(name))
                    .filter(|program| !embedded.contains(&program.family))
            };
            candidates
                .entry(request.source)
                .and_modify(|previous| {
                    // 한 번 충돌한 선택을 뒤 슬롯에서 되살리지 않는다.
                    if !previous
                        .as_ref()
                        .zip(candidate.as_ref())
                        .is_some_and(|(left, right)| {
                            left.family == right.family
                                && left.program_id == right.program_id
                                && left.true_type == right.true_type
                        })
                    {
                        *previous = None;
                    }
                })
                .or_insert(candidate);
        }
        let mut substitutions = BTreeMap::new();
        let mut true_type_faces = std::collections::BTreeSet::new();
        let mut digest = Sha256::new();
        for (source, selected) in candidates {
            let Some(selected) = selected else { continue };
            if !Self::valid_name(&selected.family) {
                continue;
            }
            for value in [&source, &selected.family, &selected.program_id] {
                digest.update(value.as_bytes());
                digest.update([0]);
            }
            digest.update([u8::from(selected.true_type)]);
            if selected.true_type {
                true_type_faces.insert(selected.family.clone());
            }
            substitutions.insert(source, selected.family);
        }
        if substitutions.is_empty() || substitutions.len() > 256 {
            return Ok(None);
        }
        let environment = Self {
            id: format!(
                "declared-program-{}",
                digest
                    .finalize()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            ),
            substitutions,
            true_type_faces: true_type_faces.into_iter().collect(),
        };
        environment.validate()?;
        Ok(Some(environment))
    }

    /// 실제 로드에 성공한 프로그램만 받는다. 존재하지 않는 후보는 넣지 않는다.
    pub fn from_available_programs(
        doc_info: &crate::model::document::DocInfo,
        json: &str,
    ) -> Result<Option<Self>, HwpError> {
        if json.len() > 65_536 {
            return Err(HwpError::RenderError(
                "font program snapshot exceeds 64 KiB".into(),
            ));
        }
        let records: Vec<AvailableFontProgram> = serde_json::from_str(json)
            .map_err(|e| HwpError::RenderError(format!("invalid font program snapshot: {e}")))?;
        if records.len() > 1024
            || records.iter().any(|record| {
                !Self::valid_name(&record.query)
                    || !Self::valid_name(&record.family)
                    || !Self::valid_name(&record.program_id)
            })
        {
            return Err(HwpError::RenderError(
                "invalid font program identity".into(),
            ));
        }
        let mut programs = BTreeMap::new();
        for record in records {
            if let Some(previous) = programs.insert(record.query.clone(), record.clone()) {
                if previous != record {
                    return Err(HwpError::RenderError(
                        "conflicting font program identity".into(),
                    ));
                }
            }
        }
        Self::from_document_lookup(doc_info, |query| programs.get(query).cloned())
    }

    /// Native는 실제 family/PostScript와 프로그램 바이트로 같은 선택을 수행한다.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_native_document_substitutions(
        doc_info: &crate::model::document::DocInfo,
        font_paths: &[std::path::PathBuf],
    ) -> Result<Option<Self>, HwpError> {
        use sha2::{Digest, Sha256};
        if !Self::document_requests(doc_info)
            .iter()
            .any(|request| !request.substitute.is_empty())
        {
            return Ok(None);
        }
        let mut database = usvg::fontdb::Database::new();
        for directory in super::font_paths::search_dirs(font_paths) {
            if directory.exists() {
                database.load_fonts_dir(directory);
            }
        }
        Self::from_document_lookup(doc_info, |query| {
            let face = database
                .query(&usvg::fontdb::Query {
                    families: &[usvg::fontdb::Family::Name(query)],
                    ..usvg::fontdb::Query::default()
                })
                .and_then(|id| database.face(id))
                .or_else(|| database.faces().find(|face| face.post_script_name == query))?;
            let family = face.families.first()?.0.clone();
            database.with_face_data(face.id, |bytes, index| {
                let offset = if bytes.get(..4) == Some(b"ttcf") {
                    let record = 12usize.checked_add((index as usize).checked_mul(4)?)?;
                    u32::from_be_bytes(bytes.get(record..record + 4)?.try_into().ok()?) as usize
                } else {
                    0
                };
                let true_type = matches!(
                    bytes.get(offset..offset.checked_add(4)?),
                    Some(b"\0\x01\0\0" | b"true")
                );
                let mut digest = Sha256::new();
                digest.update(bytes);
                digest.update(index.to_be_bytes());
                Some(AvailableFontProgram {
                    query: query.into(),
                    family,
                    true_type,
                    program_id: digest
                        .finalize()
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect(),
                })
            })?
        })
    }

    pub fn from_json(json: &str) -> Result<Self, HwpError> {
        if json.len() > 65_536 {
            return Err(HwpError::RenderError(
                "font environment exceeds 64 KiB".into(),
            ));
        }
        let environment: Self = serde_json::from_str(json)
            .map_err(|e| HwpError::RenderError(format!("invalid font environment: {e}")))?;
        environment.validate()?;
        Ok(environment)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn validate(&self) -> Result<(), HwpError> {
        if !Self::valid_name(&self.id)
            || self.true_type_faces.len() > 256
            || self
                .true_type_faces
                .iter()
                .any(|face| !Self::valid_name(face))
            || self
                .true_type_faces
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.true_type_faces.len()
            || self.substitutions.len() > 256
            || self.glyph_fallbacks.len() > 256
            || self.glyph_fallbacks.iter().any(|(from, faces)| {
                !valid_name(from)
                    || faces.is_empty()
                    || faces.len() > 16
                    || faces.iter().any(|face| !valid_name(face) || face == from)
                    || faces
                        .iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        != faces.len()
            })
            || self
                .substitutions
                .iter()
                .any(|(from, to)| !Self::valid_name(from) || !Self::valid_name(to))
        {
            return Err(HwpError::RenderError(
                "font environment requires a valid id, single-face substitutions, and bounded distinct glyph fallback faces".into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn selects_true_type(&self, face: &str) -> bool {
        self.true_type_faces.iter().any(|selected| selected == face)
    }

    pub(crate) fn replacement(&self, face: &str) -> Option<&str> {
        self.substitutions.get(face).map(String::as_str)
    }

    pub(crate) fn glyph_fallbacks(&self, face: &str) -> Option<&[String]> {
        self.glyph_fallbacks.get(face).map(Vec::as_slice)
    }
}
