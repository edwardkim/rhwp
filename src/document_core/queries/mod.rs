mod bookmark_query;
mod cursor_nav;
mod cursor_rect;
pub(crate) mod doc_tree_nav;
/// [#3828] `explain` 명령 전용 집계(각주/미주 개수) — 다른 조회가 채우지 못하는 구멍.
pub mod explain;
// [#3281] `fields` CLI 가 필드 위치(NestedEntry)를 읽어야 하므로 공개한다.
// 읽기 전용 질의 모듈이며 `structure`·`rendering` 과 같은 가시성이다.
pub mod field_query;
mod form_query;
pub mod hwpctrl_sets;
pub mod rendering;
// [#3283] `grep` 이 같은 매칭 규칙(find_matches)을 쓰도록 크레이트 내부 공개.
/// [프롬프트 주입 방패] 문서 본문을 nonce 격벽으로 감싸 LLM 에 안전하게 넘긴다 — 읽기 전용.
pub mod armor;
/// 주소(구역·문단·페이지)를 가진 검색 — 조판 엔진이 있어야만 가능한 질의.
pub mod changed_pages;
/// 날짜·금액·수량을 주소와 함께 뽑는 추출 코어 — `grep` 과 같은 페이지 인덱스를 쓴다.
pub mod extract_data;
pub mod grep;
/// [#3787 S3] 은닉 텍스트 판정 — 흰 글씨/0pt/쪽 밖 텍스트를 읽기 전용으로 보고한다.
pub mod hidden_text;
/// [#3787 S2] 프롬프트 주입 신호 탐지 — 읽기 전용, 문서 무변경.
pub mod injection_scan;
/// 개요 번호 전용 탐색 메타데이터. 문단 모양의 Outline만 대상으로 한다.
pub mod navigation;
/// [#3719 §6-11] 공개 전 개인정보 탐지 — 읽기 전용 판정(마스킹은 CLI 의 치환 경로).
pub mod pii_scan;
pub(crate) mod search_query;
/// 숨은 마크(제로폭·호모글리프·공백 스테가노) 탐지 + 방어적 정화 코어 — 읽기 전용 판정.
pub mod stego_scan;
pub mod structure;
/// 무기화 문서 구조 위협 탐지 — 읽기 전용 안전 에어락(컨테이너·레코드 구조 층).
pub mod threat_scan;
// [#3719 §6-7] 표 ↔ CSV 변환 — `table_extract` 격자를 재사용하는 순수 변환 코어.
/// [#4100] 차트 데이터 ↔ CSV 행렬 (행=카테고리, 열=계열).
pub mod chart_csv;
/// [#4100] 문서 안 차트 열거와 ①② 슬롯 해석.
pub mod chart_extract;
pub mod table_csv;
pub mod table_extract;
