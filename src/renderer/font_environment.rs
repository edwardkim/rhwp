//! Caller-declared font availability for one rendering session, never document IR.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::HwpError;

/// Explicit final face substitutions shared by layout and paint.
///
/// Global keys are exact document font names. Conflicting document declarations
/// retain their language/font-record address. Targets are single faces, not CSS
/// fallback chains. Mappings are applied once, not recursively.
/// This declaration does not prove that the backend has the target font.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FontEnvironment {
    id: String,
    substitutions: BTreeMap<String, String>,
    /// 실제 공급 snapshot에서 조건을 충족한 canonical 선택의 출처다.
    /// 글꼴 설치·바이트 공급의 증명과 구분하며 명시적 환경에는 생략할 수 있다.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    selection_rules: BTreeMap<String, String>,
    /// 호출자가 실제 출력 프로그램을 확인한 TrueType 최종 face.
    /// 설치 증명은 아니며, 생략하면 기존 HFT 호환 메트릭을 유지한다.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    true_type_faces: Vec<String>,
    /// 자동 공급 선택은 표시 프로그램만 보완한다. 검증된 원본 HFT의
    /// 조판 폭까지 대체 프로그램의 폭으로 바꾸는 명시적 요청과 구분한다.
    #[serde(default, skip_serializing_if = "is_false")]
    preserve_source_metrics: bool,
    /// 호출자가 공급을 확인한 글리프 폴백. 키는 치환 후의 최종 face이며,
    /// 기본 face가 없는 글자만 이 순서로 측정·출력한다. IR에는 저장하지 않는다.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    glyph_fallbacks: BTreeMap<String, Vec<String>>,
    /// 같은 원본 이름이라도 언어 슬롯/글꼴 레코드가 지정한 프로그램은 다를 수 있다.
    /// 전역 치환으로 합칠 수 없는 선택만 원본 주소와 함께 보존한다.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    document_substitutions: Vec<DocumentFontSubstitution>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DocumentFontSubstitution {
    language_slot: usize,
    font_id: u16,
    source: String,
    target: String,
    program_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    selection_rule: Option<String>,
}

/// 실제 프로그램 조회용 이름이다. generic·호환 대체는 설치 별칭에 포함하지 않는다.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentFontSelectionRequest {
    pub source: String,
    pub primary: Vec<String>,
    pub substitute: Vec<String>,
    /// 원본·문서 지정 프로그램이 모두 없을 때만 시도할 공개 프로그램.
    /// 선언 이름을 무조건 정규화하는 alias와 구분한다.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub public_fallback: Vec<String>,
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

/// 실제 프로그램 공급과 존재 확인은 서로 다른 증거다. 이름만 확인된
/// source는 공개 fallback을 막을 수 있지만 TrueType/메트릭 공급자가 되지 않는다.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AvailableFontProgramSnapshot {
    programs: Vec<AvailableFontProgram>,
    #[serde(default)]
    public_fallback_blocked_sources: Vec<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AvailableFontProgramInput {
    Snapshot(AvailableFontProgramSnapshot),
    Legacy(Vec<AvailableFontProgram>),
}

fn is_false(value: &bool) -> bool {
    !value
}

fn program_names(name: &str) -> Vec<String> {
    std::iter::once(name)
        .chain(super::installed_font_program_aliases(name).iter().copied())
        .filter(|name| FontEnvironment::valid_name(name))
        .map(str::to_owned)
        .collect()
}

impl FontEnvironment {
    pub(crate) fn with_observed_substitutions(
        base: Option<&Self>,
        replacements: BTreeMap<String, String>,
    ) -> Result<Option<Self>, HwpError> {
        if replacements.is_empty() {
            return Ok(None);
        }
        let mut result = base.cloned().unwrap_or(Self {
            id: "observed-host-fonts".to_owned(),
            substitutions: BTreeMap::new(),
            selection_rules: BTreeMap::new(),
            true_type_faces: Vec::new(),
            preserve_source_metrics: false,
            document_substitutions: Vec::new(),
            glyph_fallbacks: BTreeMap::new(),
        });
        for (family, target) in replacements {
            if result.has_source_selection(&family) {
                continue;
            }
            result.substitutions.insert(family, target.clone());
            if !result.true_type_faces.contains(&target) {
                result.true_type_faces.push(target);
            }
        }
        result.validate()?;
        Ok(Some(result))
    }

    /// 원본과 문서 지정 내장 프로그램을 같은 이름 집합으로 보존한다.
    pub(crate) fn embedded_program_names(
        doc_info: &crate::model::document::DocInfo,
    ) -> std::collections::BTreeSet<String> {
        let mut names = std::collections::BTreeSet::new();
        for font in doc_info.font_faces.iter().flatten() {
            let substitute = font
                .subst_font
                .as_ref()
                .filter(|substitute| substitute.is_embedded);
            if font.is_embedded || substitute.is_some() {
                names.insert(font.name.clone());
            }
            if let Some(substitute) = substitute {
                names.insert(substitute.face.clone());
            }
        }
        names
    }

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
        Self::document_requests_with_slots(doc_info)
            .into_iter()
            .map(|(_, _, request)| request)
            .collect()
    }

    fn document_requests_with_slots(
        doc_info: &crate::model::document::DocInfo,
    ) -> Vec<(usize, u16, DocumentFontSelectionRequest)> {
        let mut requests = Vec::new();
        // 같은 이름에 내장 프로그램이 있으면 전역 CSS 별칭으로 덮어쓰지 않는다.
        let embedded = Self::embedded_program_names(doc_info);
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
                let Ok(font_id) = u16::try_from(index) else {
                    continue;
                };
                requests.push((
                    language,
                    font_id,
                    DocumentFontSelectionRequest {
                        source: font.name.clone(),
                        primary: program_names(primary),
                        public_fallback:
                            super::font_rule_layout_name_projection::find_font_rule_layout_name(
                                "rust-style-resolution.missing-program",
                                &font.name,
                                language,
                            )
                            .filter(|rule| {
                                rule.conditions.availability
                                    == Some("primary-and-document-substitute-missing")
                            })
                            .map(|rule| program_names(rule.target_face_or_policy))
                            .unwrap_or_default(),
                        substitute: decision
                            .subst_font
                            .as_deref()
                            .map(program_names)
                            .unwrap_or_default(),
                    },
                ));
            }
        }
        requests
    }

    fn from_document_lookup(
        doc_info: &crate::model::document::DocInfo,
        mut lookup: impl FnMut(&str) -> Option<AvailableFontProgram>,
        public_fallback_blocked_sources: &[String],
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
        let mut candidates =
            BTreeMap::<String, Option<(AvailableFontProgram, Option<&'static str>)>>::new();
        let embedded = Self::embedded_program_names(doc_info);
        let mut slot_candidates = Vec::new();
        for (language_slot, font_id, request) in Self::document_requests_with_slots(doc_info) {
            let primary = request.primary.iter().find_map(|name| resolve(name));
            let candidate = if let Some(program) = primary {
                // 원본 선언은 HFT지만 실제로 준비된 동일 주 프로그램은 TTF일 수 있다.
                // 존재 확인으로 끝내지 않고 프로그램 신원을 공통 스타일 해소에 전달한다.
                // 치환 family와 원 HFT 프로그램 출처는 이 주 프로그램 인증이 아니다.
                let source = doc_info
                    .font_faces
                    .get(language_slot)
                    .and_then(|faces| faces.get(font_id as usize));
                (program.true_type
                    && program.query == request.source
                    && source
                        .is_some_and(|font| font.alt_type == 2 && font.native_program.is_none()))
                .then_some((program, None))
            } else {
                request
                    .substitute
                    .iter()
                    .find_map(|name| resolve(name))
                    .filter(|program| !embedded.contains(&program.family))
                    .map(|program| (program, None))
                    .or_else(|| {
                        // local() 이름 존재는 공개 대체를 막는 데만 사용한다.
                        // 프로그램이 실제 공급됐다는 증명으로 바꾸지 않는다.
                        if public_fallback_blocked_sources.contains(&request.source) {
                            return None;
                        }
                        request
                            .public_fallback
                            .iter()
                            .find_map(|name| resolve(name))
                            .filter(|program| !embedded.contains(&program.family))
                            .map(|program| {
                                let rule = super::font_rule_layout_name_projection::find_font_rule_layout_name(
                                    "rust-style-resolution.missing-program", &request.source, 0,
                                ).map(|rule| rule.rule_id);
                                (program, rule)
                            })
                    })
            };
            slot_candidates.push((
                language_slot,
                font_id,
                request.source.clone(),
                candidate.clone(),
            ));
            candidates
                .entry(request.source)
                .and_modify(|previous| {
                    // 한 번 충돌한 선택을 뒤 슬롯에서 되살리지 않는다.
                    match previous.as_mut().zip(candidate.as_ref()) {
                        Some((left, right))
                            if left.0.family == right.0.family
                                && left.0.program_id == right.0.program_id
                                && left.0.true_type == right.0.true_type =>
                        {
                            // 같은 프로그램이라도 선택 사유가 다른 슬롯은 전역 조건의
                            // 증거가 아니다. 공급 선택은 유지하고 조건 출처만 제외한다.
                            if left.1 != right.1 {
                                left.1 = None;
                            }
                        }
                        _ => *previous = None,
                    }
                })
                .or_insert(candidate);
        }
        let conflicted: std::collections::BTreeSet<_> = candidates
            .iter()
            .filter(|(_, selection)| selection.is_none())
            .map(|(source, _)| source.clone())
            .collect();
        // 다른 프로그램이 같은 최종 family를 주장하면 슬롯 주소만으로 paint를
        // 구분할 수 없다. 그런 모호성은 기존처럼 수용하지 않는다.
        let mut family_programs: BTreeMap<(String, String), Option<(String, bool)>> =
            BTreeMap::new();
        for (_, _, source, candidate) in &slot_candidates {
            if let Some((program, _)) = candidate {
                let identity = (program.program_id.clone(), program.true_type);
                family_programs
                    .entry((source.clone(), program.family.clone()))
                    .and_modify(|previous| {
                        if previous.as_ref() != Some(&identity) {
                            *previous = None;
                        }
                    })
                    .or_insert(Some(identity));
            }
        }
        let mut substitutions = BTreeMap::new();
        let mut selection_rules = BTreeMap::new();
        let mut true_type_faces = std::collections::BTreeSet::new();
        let mut digest = Sha256::new();
        digest.update(b"preserve-source-hft-metrics\0");
        for (source, selected) in candidates {
            let Some((selected, rule_id)) = selected else {
                continue;
            };
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
            if let Some(rule_id) = rule_id {
                digest.update(rule_id.as_bytes());
                digest.update([0]);
                selection_rules.insert(source.clone(), rule_id.to_owned());
            }
            substitutions.insert(source, selected.family);
        }
        let mut document_substitutions = Vec::new();
        for (language_slot, font_id, source, candidate) in slot_candidates {
            let Some((program, rule)) = candidate.filter(|_| conflicted.contains(&source)) else {
                continue;
            };
            if family_programs
                .get(&(source.clone(), program.family.clone()))
                .is_none_or(Option::is_none)
            {
                continue;
            }
            digest.update(b"document-font-slot\0");
            digest.update((language_slot as u64).to_le_bytes());
            digest.update(font_id.to_le_bytes());
            for value in [&source, &program.family, &program.program_id] {
                digest.update(value.as_bytes());
                digest.update([0]);
            }
            digest.update([u8::from(program.true_type)]);
            if let Some(rule) = rule {
                digest.update(rule.as_bytes());
                digest.update([0]);
            }
            if program.true_type {
                true_type_faces.insert(program.family.clone());
            }
            document_substitutions.push(DocumentFontSubstitution {
                language_slot,
                font_id,
                source,
                target: program.family,
                program_id: program.program_id,
                selection_rule: rule.map(str::to_owned),
            });
        }
        if (substitutions.is_empty() && document_substitutions.is_empty())
            || substitutions.len() > 256
            || document_substitutions.len() > 256
        {
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
            selection_rules,
            true_type_faces: true_type_faces.into_iter().collect(),
            preserve_source_metrics: true,
            glyph_fallbacks: BTreeMap::new(),
            document_substitutions,
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
        let input: AvailableFontProgramInput = serde_json::from_str(json)
            .map_err(|e| HwpError::RenderError(format!("invalid font program snapshot: {e}")))?;
        let (records, public_fallback_blocked_sources) = match input {
            AvailableFontProgramInput::Snapshot(snapshot) => {
                (snapshot.programs, snapshot.public_fallback_blocked_sources)
            }
            AvailableFontProgramInput::Legacy(records) => (records, Vec::new()),
        };
        if public_fallback_blocked_sources.len() > 1024
            || public_fallback_blocked_sources
                .iter()
                .any(|name| !Self::valid_name(name))
        {
            return Err(HwpError::RenderError(
                "invalid font presence snapshot".into(),
            ));
        }
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
        Self::from_document_lookup(
            doc_info,
            |query| programs.get(query).cloned(),
            &public_fallback_blocked_sources,
        )
    }

    /// Native는 실제 family/PostScript와 프로그램 바이트로 같은 선택을 수행한다.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_native_document_substitutions(
        doc_info: &crate::model::document::DocInfo,
        font_paths: &[std::path::PathBuf],
    ) -> Result<Option<Self>, HwpError> {
        use sha2::{Digest, Sha256};
        let needs_primary_program_identity =
            doc_info.font_faces.iter().flatten().any(|font| {
                !font.is_embedded && font.alt_type == 2 && font.native_program.is_none()
            });
        if !needs_primary_program_identity
            && !Self::document_requests(doc_info).iter().any(|request| {
                !request.substitute.is_empty() || !request.public_fallback.is_empty()
            })
        {
            return Ok(None);
        }
        let mut database = usvg::fontdb::Database::new();
        for directory in super::font_paths::search_dirs(font_paths) {
            if directory.exists() {
                database.load_fonts_dir(directory);
            }
        }
        Self::from_document_lookup(
            doc_info,
            |query| {
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
            },
            &[],
        )
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
            || self.selection_rules.len() > 256
            || self.selection_rules.iter().any(|(source, rule_id)| {
                self.canonical_selection_rule(source) != Some(rule_id.as_str())
            })
            || self.substitutions.len() > 256
            || self.document_substitutions.len() > 256
            || self.document_substitutions.iter().any(|selection| {
                selection.language_slot >= super::style_resolver::LANG_COUNT
                    || !Self::valid_name(&selection.source)
                    || !Self::valid_name(&selection.target)
                    || !Self::valid_name(&selection.program_id)
                    || self.substitutions.contains_key(&selection.source)
                    || selection.selection_rule.as_deref().is_some_and(|rule| {
                        Self::canonical_rule(&selection.source, &selection.target, rule).is_none()
                    })
            })
            || self
                .document_substitutions
                .iter()
                .map(|selection| (selection.language_slot, selection.font_id))
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.document_substitutions.len()
            || self.glyph_fallbacks.len() > 256
            || self.glyph_fallbacks.iter().any(|(from, faces)| {
                !Self::valid_name(from)
                    || faces.is_empty()
                    || faces.len() > 16
                    || faces
                        .iter()
                        .any(|face| !Self::valid_name(face) || face == from)
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

    /// JSON 전달 뒤에도 canonical 조건·선택 대상과 일치하는 출처만 반환한다.
    pub(crate) fn canonical_selection_rule(&self, source: &str) -> Option<&'static str> {
        let recorded = self.selection_rules.get(source)?;
        let target = self.substitutions.get(source)?;
        Self::canonical_rule(source, target, recorded)
    }

    fn canonical_rule(source: &str, target: &str, recorded: &str) -> Option<&'static str> {
        super::font_rule_layout_name_projection::find_font_rule_layout_name(
            "rust-style-resolution.missing-program",
            source,
            0,
        )
        .filter(|rule| {
            rule.rule_id == recorded
                && rule.target_face_or_policy == target
                && rule.conditions.availability == Some("primary-and-document-substitute-missing")
        })
        .map(|rule| rule.rule_id)
    }

    pub(crate) fn selects_true_type(&self, face: &str) -> bool {
        self.true_type_faces.iter().any(|selected| selected == face)
    }

    pub(crate) fn preserves_source_metrics(&self) -> bool {
        self.preserve_source_metrics
    }

    pub(crate) fn replacement(&self, face: &str) -> Option<&str> {
        self.substitutions.get(face).map(String::as_str)
    }

    /// family 단위 관측은 원본 주소별 프로그램 선택을 덮어쓸 수 없다.
    pub(crate) fn has_source_selection(&self, face: &str) -> bool {
        self.substitutions.contains_key(face)
            || self
                .document_substitutions
                .iter()
                .any(|selection| selection.source == face)
    }

    fn document_substitution(
        &self,
        face: &str,
        language_slot: usize,
        font_id: u16,
    ) -> Option<&DocumentFontSubstitution> {
        self.document_substitutions.iter().find(|selection| {
            selection.language_slot == language_slot
                && selection.font_id == font_id
                && selection.source == face
        })
    }

    pub(crate) fn replacement_for(
        &self,
        face: &str,
        language_slot: usize,
        font_id: u16,
    ) -> Option<&str> {
        self.document_substitution(face, language_slot, font_id)
            .map(|selection| selection.target.as_str())
            .or_else(|| self.replacement(face))
    }

    pub(crate) fn canonical_selection_rule_for(
        &self,
        source: &str,
        language_slot: usize,
        font_id: u16,
    ) -> Option<&'static str> {
        if let Some(selection) = self.document_substitution(source, language_slot, font_id) {
            return Self::canonical_rule(
                source,
                &selection.target,
                selection.selection_rule.as_deref()?,
            );
        }
        self.canonical_selection_rule(source)
    }

    pub(crate) fn glyph_fallbacks(&self, face: &str) -> Option<&[String]> {
        self.glyph_fallbacks.get(face).map(Vec::as_slice)
    }
}
