//! [#4493] DocInfo raw 캐시 출처 봉인(provenance seal).
//!
//! ## 문제
//!
//! 공개 저수준 API 는 `parse_document → model::Document 직접 변경 →
//! serialize_document` 를 허용하는데, HWP 파싱은 `DocInfo.raw_stream` 과 레코드별
//! `raw_data` 를 채우고 `raw_stream_dirty=false` 로 남긴다. 공개 모델 필드를 직접
//! 바꾸면 dirty 표식이 서지 않으므로, 저장 시 원본 바이트 통과(스트림·레코드
//! 양쪽)가 그대로 발동해 **메모리에서 보이던 변경이 저장·재로드 뒤 조용히
//! 사라진다.**
//!
//! ## 처방 — 봉인과 검증
//!
//! 파싱과 모든 load fixup 이 끝난 시점(`parse_document_inner` 말미)에 봉인한다.
//!
//! - **스트림 봉인** — 원본 DocInfo 바이트와 모델 상태의 다이제스트 쌍. 저장 시
//!   둘 다 일치할 때만 `raw_stream` 전체 통과를 허용한다. 공개 `raw_stream` 을
//!   다른 바이트로 교체해도(raw digest 불일치) 봉인으로 승인되지 않는다.
//! - **레코드 봉인** — DocInfo 의 레코드별 `raw_data` 지름길(BIN_DATA·FACE_NAME·
//!   BORDER_FILL·CHAR_SHAPE·TAB_DEF·NUMBERING·BULLET·PARA_SHAPE·STYLE·
//!   DOCUMENT_PROPERTIES)에 대응하는 레코드 단위 다이제스트. 스트림이 재생성될
//!   때 **변경되지 않은 레코드만** 원본 바이트를 재사용하고, 변경된 레코드는
//!   모델 writer 로 다시 쓴다 — 하위 raw 를 무조건 폐기해 미모델링 바이트를
//!   잃는 방식은 쓰지 않는다(#4495 와 같은 원칙).
//!
//! 봉인은 모델 타입에 필드를 넣지 않고 이 모듈의 컨테이너(인덱스 정렬)로
//! 중앙 보관한다 — 목록 삽입·삭제로 인덱스가 어긋나면 다이제스트가 어긋나
//! 재생성으로 떨어지므로 안전한 방향으로 실패한다.
//!
//! ## 다이제스트 규약
//!
//! serde 직렬화(JSON 인코딩)를 blake3 로 해시한다 — `Debug`/`DefaultHasher`/
//! 재귀 `Clone` 스냅샷에 의존하지 않는다. serde_json 은 구조체 필드를 선언
//! 순서로, 숫자·float 를 결정적 표기(ryu/itoa)로 방출하므로 같은 프로세스에서
//! 항상 같은 바이트를 낸다. DocInfo 트리에는 순회 순서가 비결정적인 맵 타입이
//! 없다(생기면 이 모듈에서 정렬 직렬화로 감싸야 한다). 봉인 대상 필드는
//! [`doc_info_model_digest`] 가 **전수 구조분해**로 나열한다 — 필드 추가 시
//! 컴파일 오류로 목록 갱신을 강제한다.
//!
//! Section(`raw_stream`)·본문 컨트롤 raw 레코드의 같은 계약은 #4488·#4495 가
//! 별도로 다룬다 — 스트림·소유자가 다르다.

use serde::Serialize;

use super::document::{DocInfo, DocProperties};

/// 파싱 완료 시점의 DocInfo 봉인.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocInfoSeal {
    /// 봉인 시점 모델 상태의 다이제스트 ([`doc_info_model_digest`]).
    pub model_digest: [u8; 32],
    /// 봉인 시점 `raw_stream` 바이트의 다이제스트.
    pub raw_digest: [u8; 32],
    /// 레코드별 봉인 — 인덱스는 봉인 시점 목록 순서.
    pub record_seals: DocInfoRecordSeals,
}

/// 레코드별 `raw_data` 지름길에 대응하는 다이제스트 묶음.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocInfoRecordSeals {
    pub props: [u8; 32],
    pub bin_data: Vec<[u8; 32]>,
    /// 언어 축(바깥)×글꼴(안) — `DocInfo.font_faces` 와 같은 모양.
    pub fonts: Vec<Vec<[u8; 32]>>,
    pub border_fills: Vec<[u8; 32]>,
    pub char_shapes: Vec<[u8; 32]>,
    pub tab_defs: Vec<[u8; 32]>,
    pub numberings: Vec<[u8; 32]>,
    pub bullets: Vec<[u8; 32]>,
    pub para_shapes: Vec<[u8; 32]>,
    pub styles: Vec<[u8; 32]>,
}

/// 바이트 열의 다이제스트.
pub fn bytes_digest(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}

/// 레코드(직렬화 가능한 모델 값) 하나의 다이제스트.
///
/// 레코드의 `raw_data` 필드도 다이제스트에 포함된다 — 모델 필드가 변하지 않는 한
/// `raw_data` 는 파싱 시점 그대로이므로 판정에 영향이 없고, `raw_data` 만 바꿔치기
/// 하는 변경도 불일치로 떨어져 승인되지 않는다.
pub fn record_digest<T: Serialize>(record: &T) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    serde_json::to_writer(&mut hasher, record)
        .expect("레코드 다이제스트 직렬화는 실패할 수 없다 (순수 인메모리 인코딩)");
    *hasher.finalize().as_bytes()
}

/// 봉인 컨테이너에서 `idx` 레코드의 raw 재사용 허용 여부.
///
/// - 봉인 자체가 없으면(`seals=None`, 파서를 거치지 않은 합성 IR) 종전 계약대로
///   허용한다.
/// - 봉인이 있으면 인덱스가 범위 안이고 다이제스트가 일치할 때만 허용한다 —
///   목록 삽입·삭제로 인덱스가 밀리면 불일치로 떨어져 모델 writer 로 재생성된다.
pub fn record_raw_permitted<T: Serialize>(
    seals: Option<&[[u8; 32]]>,
    idx: usize,
    record: &T,
) -> bool {
    match seals {
        None => true,
        Some(seals) => seals
            .get(idx)
            .is_some_and(|sealed| *sealed == record_digest(record)),
    }
}

/// DocInfo(+DocProperties) 모델 상태의 다이제스트.
///
/// 저장 산출물에 실리는 모델 필드 전부를 선언 순서로 직렬화해 해시한다.
/// 구조분해에 `..` 를 쓰지 않는 것이 계약이다 — 새 필드가 생기면 여기가
/// 컴파일 오류를 내서 "봉인 대상인가"의 판단을 강제한다. raw 캐시 자신
/// (`raw_stream`)과 그 관리 표식(`raw_stream_dirty`, `raw_provenance`), 통과
/// 경로가 스스로 처리하는 `distribute_doc_data_removed`(surgical remove)는 뺀다.
pub fn doc_info_model_digest(doc_info: &DocInfo, doc_props: &DocProperties) -> [u8; 32] {
    let DocInfo {
        bin_data_list,
        font_faces,
        border_fills,
        char_shapes,
        tab_defs,
        numberings,
        bullets,
        para_shapes,
        styles,
        extra_records,
        raw_stream: _,
        bullet_count,
        memo_shape_count,
        memo_properties_xml,
        distribute_doc_data_removed: _,
        raw_stream_dirty: _,
        hwpx_head_tail,
        hwpml_version,
        raw_provenance: _,
    } = doc_info;

    #[derive(Serialize)]
    struct Fingerprint<'a> {
        bin_data_list: &'a [crate::model::bin_data::BinData],
        font_faces: &'a [Vec<crate::model::style::Font>],
        border_fills: &'a [crate::model::style::BorderFill],
        char_shapes: &'a [crate::model::style::CharShape],
        tab_defs: &'a [crate::model::style::TabDef],
        numberings: &'a [crate::model::style::Numbering],
        bullets: &'a [crate::model::style::Bullet],
        para_shapes: &'a [crate::model::style::ParaShape],
        styles: &'a [crate::model::style::Style],
        extra_records: &'a [crate::model::document::RawRecord],
        bullet_count: u32,
        memo_shape_count: u32,
        memo_properties_xml: &'a Option<String>,
        hwpx_head_tail: &'a Option<String>,
        hwpml_version: &'a Option<String>,
        props: &'a DocProperties,
    }

    let fp = Fingerprint {
        bin_data_list,
        font_faces,
        border_fills,
        char_shapes,
        tab_defs,
        numberings,
        bullets,
        para_shapes,
        styles,
        extra_records,
        bullet_count: *bullet_count,
        memo_shape_count: *memo_shape_count,
        memo_properties_xml,
        hwpx_head_tail,
        hwpml_version,
        props: doc_props,
    };

    let mut hasher = blake3::Hasher::new();
    serde_json::to_writer(&mut hasher, &fp)
        .expect("모델 다이제스트 직렬화는 실패할 수 없다 (순수 인메모리 인코딩)");
    *hasher.finalize().as_bytes()
}

/// 레코드 봉인 묶음 계산 — 봉인 시점의 목록 순서 그대로.
fn compute_record_seals(doc_info: &DocInfo, doc_props: &DocProperties) -> DocInfoRecordSeals {
    DocInfoRecordSeals {
        props: record_digest(doc_props),
        bin_data: doc_info.bin_data_list.iter().map(record_digest).collect(),
        fonts: doc_info
            .font_faces
            .iter()
            .map(|lang| lang.iter().map(record_digest).collect())
            .collect(),
        border_fills: doc_info.border_fills.iter().map(record_digest).collect(),
        char_shapes: doc_info.char_shapes.iter().map(record_digest).collect(),
        tab_defs: doc_info.tab_defs.iter().map(record_digest).collect(),
        numberings: doc_info.numberings.iter().map(record_digest).collect(),
        bullets: doc_info.bullets.iter().map(record_digest).collect(),
        para_shapes: doc_info.para_shapes.iter().map(record_digest).collect(),
        styles: doc_info.styles.iter().map(record_digest).collect(),
    }
}

impl DocInfo {
    /// 파싱(+로드 픽스업) 완료 시점에 호출 — raw 캐시가 있으면 현재 모델 상태와
    /// 함께 봉인한다. raw 가 없으면(HWPX/HWP3/합성 IR) 봉인도 없다.
    pub fn seal_raw_provenance(&mut self, doc_props: &DocProperties) {
        self.raw_provenance = self.raw_stream.as_ref().map(|raw| DocInfoSeal {
            model_digest: doc_info_model_digest(self, doc_props),
            raw_digest: bytes_digest(raw),
            record_seals: compute_record_seals(self, doc_props),
        });
    }

    /// 저장 시점 스트림 통과 검증 — raw 바이트와 모델 상태가 둘 다 봉인과 같은가.
    ///
    /// 봉인이 없는 raw(`raw_provenance=None`, 파서를 거치지 않은 합성 IR 등)는
    /// 종전 계약대로 통과를 허용한다 — 봉인은 공개 parse→mutate→serialize
    /// 경로의 손실을 막기 위한 것이고, 파서가 만든 문서에는 항상 봉인이 있다.
    pub fn raw_provenance_permits_reuse(&self, doc_props: &DocProperties) -> bool {
        let Some(raw) = self.raw_stream.as_ref() else {
            return false;
        };
        match &self.raw_provenance {
            None => true,
            Some(seal) => {
                seal.raw_digest == bytes_digest(raw)
                    && seal.model_digest == doc_info_model_digest(self, doc_props)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_changes_when_model_changes() {
        let mut di = DocInfo::default();
        let props = DocProperties::default();
        let before = doc_info_model_digest(&di, &props);
        di.char_shapes
            .push(crate::model::style::CharShape::default());
        let after = doc_info_model_digest(&di, &props);
        assert_ne!(before, after, "char_shapes 변경이 다이제스트에 실려야 한다");
    }

    #[test]
    fn digest_ignores_raw_cache_fields() {
        let mut di = DocInfo::default();
        let props = DocProperties::default();
        let before = doc_info_model_digest(&di, &props);
        di.raw_stream = Some(vec![1, 2, 3]);
        di.raw_stream_dirty = true;
        di.distribute_doc_data_removed = true;
        let after = doc_info_model_digest(&di, &props);
        assert_eq!(before, after, "raw 캐시 표식은 모델 다이제스트 밖이다");
    }

    #[test]
    fn sealed_raw_reuse_denied_after_model_mutation() {
        let mut di = DocInfo::default();
        let props = DocProperties::default();
        di.raw_stream = Some(vec![9, 9, 9]);
        di.seal_raw_provenance(&props);
        assert!(di.raw_provenance_permits_reuse(&props), "무변경이면 재사용");

        di.para_shapes
            .push(crate::model::style::ParaShape::default());
        assert!(
            !di.raw_provenance_permits_reuse(&props),
            "공개 모델 직접 변경 뒤에는 raw 재사용이 거부돼야 한다 (#4493)"
        );
    }

    #[test]
    fn sealed_raw_reuse_denied_after_raw_swap() {
        let mut di = DocInfo::default();
        let props = DocProperties::default();
        di.raw_stream = Some(vec![9, 9, 9]);
        di.seal_raw_provenance(&props);

        di.raw_stream = Some(vec![8, 8, 8]);
        assert!(
            !di.raw_provenance_permits_reuse(&props),
            "raw 바이트 교체는 기존 봉인으로 승인되지 않는다 (#4493)"
        );
    }

    #[test]
    fn unsealed_raw_keeps_legacy_passthrough() {
        let mut di = DocInfo::default();
        let props = DocProperties::default();
        di.raw_stream = Some(vec![7]);
        assert!(
            di.raw_provenance_permits_reuse(&props),
            "봉인 이전(합성 IR) 경로는 종전 계약 유지"
        );
    }

    #[test]
    fn record_raw_permission_follows_seal() {
        let cs = crate::model::style::CharShape::default();
        let sealed = [record_digest(&cs)];
        assert!(record_raw_permitted(Some(&sealed), 0, &cs), "무변경 허용");
        assert!(
            !record_raw_permitted(Some(&sealed), 1, &cs),
            "범위 밖(새 레코드)은 모델 writer"
        );
        let mut changed = cs;
        changed.base_size = 2000;
        assert!(
            !record_raw_permitted(Some(&sealed), 0, &changed),
            "변경 레코드는 raw 재사용 거부"
        );
        assert!(
            record_raw_permitted(None, 0, &changed),
            "봉인 없는 합성 IR 은 종전 계약"
        );
    }
}
