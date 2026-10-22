//! 실제 공급자가 확인한 원본 대체 TTF 선택을 조판·paint에 함께 전달한다.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::DocumentCore;
use crate::{error::HwpError, renderer::font_environment::FontEnvironment};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeclaredFontFallbackRequest {
    pub family: String,
    pub fallback: String,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObservedFontSource {
    pub family: String,
    pub sha256: String,
    pub face_index: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObservedFontFallback {
    pub request: DeclaredFontFallbackRequest,
    pub source: ObservedFontSource,
}

#[derive(Clone)]
pub(crate) struct HostFontEnvironment {
    requests: Vec<DeclaredFontFallbackRequest>,
    environment: FontEnvironment,
    selections: Vec<ObservedFontFallback>,
}

impl DocumentCore {
    /// 원본에 명시한 TTF 대체만 요청한다. 메트릭 표의 존재로 설치 여부를 추측하지 않는다.
    pub fn declared_font_fallback_requests(&self) -> Vec<DeclaredFontFallbackRequest> {
        let info = &self.document.doc_info;
        let mut requests = BTreeSet::new();
        for shape in &info.char_shapes {
            for (language, &font_id) in shape.font_ids.iter().enumerate() {
                let Some(font) = info
                    .font_faces
                    .get(language)
                    .and_then(|f| f.get(font_id as usize))
                else {
                    continue;
                };
                if font.is_embedded
                    || font.alt_type != 1
                    || self
                        .font_environment
                        .as_ref()
                        .is_some_and(|env| env.replacement(&font.name).is_some())
                {
                    continue;
                }
                let decision = crate::renderer::style_resolver::lookup_font_name_decision(
                    info, language, font_id,
                );
                if decision.substitution_boundary.is_some() {
                    continue;
                }
                let fallback = match &font.subst_font {
                    Some(f) if f.font_type == 1 && !f.is_embedded => Some(f.face.as_str()),
                    None => font.alt_name.as_deref(),
                    _ => None,
                };
                let Some(fallback) = fallback
                    .map(str::trim)
                    .filter(|f| !f.is_empty() && *f != font.name)
                else {
                    continue;
                };
                requests.insert(DeclaredFontFallbackRequest {
                    family: font.name.clone(),
                    fallback: fallback.to_owned(),
                    bold: shape.bold,
                    italic: shape.italic,
                });
                if requests.len() > 256 {
                    return Vec::new();
                }
            }
        }
        requests.into_iter().collect()
    }

    /// 공급자가 원 face 부재와 원본 대체 프로그램 선택을 확인한 경우에만 Some을 반환한다.
    /// None은 원 face 존재 또는 관측 불명이다. 같은 family의 모든 사용 style이 일치해야 한다.
    pub fn prepare_declared_font_fallbacks_with(
        &mut self,
        mut observe: impl FnMut(&DeclaredFontFallbackRequest) -> Option<ObservedFontSource>,
    ) -> Result<bool, HwpError> {
        if self.batch_mode {
            return Err(HwpError::RenderError(
                "font supply cannot change during an edit batch".into(),
            ));
        }
        let requests = self.declared_font_fallback_requests();
        let mut choices = BTreeMap::<String, Option<String>>::new();
        let mut selections = Vec::new();
        for request in &requests {
            let observed = observe(request);
            if observed.as_ref().is_some_and(|source| {
                source.sha256.len() != 64 || !source.sha256.bytes().all(|b| b.is_ascii_hexdigit())
            }) {
                return Err(HwpError::RenderError(
                    "observed font SHA-256 is invalid".into(),
                ));
            }
            let next = observed.as_ref().map(|s| s.family.clone());
            if let Some(source) = observed {
                selections.push(ObservedFontFallback {
                    request: request.clone(),
                    source,
                });
            }
            choices
                .entry(request.family.clone())
                .and_modify(|previous| {
                    if *previous != next {
                        *previous = None;
                    }
                })
                .or_insert(next);
        }
        let replacements: BTreeMap<_, _> = choices
            .into_iter()
            .filter_map(|(family, target)| target.map(|t| (family, t)))
            .collect();
        selections.retain(|s| replacements.get(&s.request.family) == Some(&s.source.family));
        let environment = FontEnvironment::with_observed_substitutions(
            self.font_environment.as_ref(),
            replacements,
        )?;
        if self
            .host_font_environment
            .as_ref()
            .map(|e| (&e.requests, &e.environment, &e.selections))
            == environment.as_ref().map(|e| (&requests, e, &selections))
        {
            return Ok(false);
        }
        self.host_font_environment = environment.map(|environment| HostFontEnvironment {
            requests,
            environment,
            selections,
        });
        self.canvas_metrics = None;
        self.pending_pagination_job = None;
        self.deferred_pagination_descriptor = None;
        self.rebuild_derived_state();
        Ok(true)
    }

    /// 별도 backend는 같은 입력·실제 공급 프로그램을 대조한 관측만 전달한다.
    /// 부분 관측이나 style 불일치는 공통 prepare 경로에서 해당 family 전체를 보류한다.
    pub fn set_observed_font_fallbacks_native(&mut self, json: &str) -> Result<bool, HwpError> {
        if json.len() > 65_536 {
            return Err(HwpError::RenderError(
                "observed font selection exceeds 64 KiB".into(),
            ));
        }
        let records: Vec<ObservedFontFallback> = serde_json::from_str(json)
            .map_err(|e| HwpError::RenderError(format!("observed font selection: {e}")))?;
        let requests = self.declared_font_fallback_requests();
        let mut selected = BTreeMap::new();
        if records.len() > 256 {
            return Err(HwpError::RenderError(
                "observed font selection exceeds 256 entries".into(),
            ));
        }
        for record in records {
            if !requests.contains(&record.request)
                || selected.insert(record.request, record.source).is_some()
            {
                return Err(HwpError::RenderError(
                    "observed font request is unknown or duplicated".into(),
                ));
            }
        }
        self.prepare_declared_font_fallbacks_with(|request| selected.get(request).cloned())
    }

    pub fn observed_font_fallbacks(&self) -> &[ObservedFontFallback] {
        self.host_font_environment
            .as_ref()
            .filter(|host| host.requests == self.declared_font_fallback_requests())
            .map_or(&[], |host| host.selections.as_slice())
    }

    /// 사용자 선언은 별도로 보존한다. 원본 요청이 달라지면 옛 공급 관측을 재사용하지 않는다.
    pub fn resolved_font_environment(&self) -> Option<&FontEnvironment> {
        self.host_font_environment
            .as_ref()
            .filter(|host| host.requests == self.declared_font_fallback_requests())
            .map(|host| &host.environment)
            .or(self.font_environment.as_ref())
    }
}
