//! HWP/HWPX → **LLM-ready RAG 출력** 축.
//!
//! 2025–2026 문서-AI 프런티어(Docling·LlamaParse·MarkItDown·Marker·Unstructured)는
//! PDF 를 청킹·표 선형화·출처 앵커가 붙은 RAG 입력으로 바꿔 준다. 그러나 이들 중
//! **어느 것도 HWP/HWPX 를 읽지 못한다.** rhwp 는 그 공백을 정조준한다 — 픽셀을
//! 추측하는 PDF 도구와 달리, rhwp 는 이미 **정확한 이진 구조**를 파싱한다: 읽기
//! 순서와 표 셀 경계(병합 span 포함)가 추측이 아니라 실측이다.
//!
//! 이 모듈은 **재파싱하지 않는다.** rhwp 가 이미 만든 IR 을 소비해 그 위에 LLM 패키징
//! 계층만 얹는다.
//! - 제목 계층 → [`crate::document_core::queries::structure::build_structure`]
//! - 표 격자(앵커 셀 + 병합 span) → [`crate::document_core::queries::table_extract::extract_tables`]
//!
//! # 산출 계약
//!
//! [`chunker::build_chunks`] 는 결정론적 RAG 청크 목록을 만든다. 각 청크는:
//! 1. **구조 인지 청킹** — 자연 경계(제목/문단/표)에서만 나뉘고, 설정 가능한 토큰
//!    예산([`chunker::ChunkOptions::max_tokens`])을 목표로 한다. 토큰 수는 실제
//!    토크나이저가 아니라 **추정치**이며 필드 이름도 `tokenEstimate` 다
//!    ([`chunker::TOKEN_ESTIMATOR`]).
//! 2. **자기완결 표** — 표는 머리 행을 보존하고 병합 셀을 주석해 Markdown 으로
//!    선형화한다. 큰 표는 **행 단위로만** 쪼개고(행 중간을 자르지 않는다) 파트마다
//!    머리 행을 되풀이한다.
//! 3. **출처 앵커** — 청크마다 `headingPath`(루트→소속 제목)와 소속 제목의
//!    `section`/`paragraph` 주소를 실어 다운스트림 에이전트가 인용할 수 있게 한다.
//! 4. **untrusted 표지** — RAG 청크는 프롬프트에 이어 붙는 **주입면 그 자체**다.
//!    봉투 출처 계약(`mydocs/tech/envelope_provenance.md`)대로 청크 텍스트를
//!    문서 파생(신뢰 불가)으로 표지한다.
//!
//! # 정직한 한계 (재파싱하지 않으므로 IR 이 모델하지 않는 것은 지어내지 않는다)
//!
//! - **본문 문단 주소**: 재사용하는 구조 IR([`build_structure`])은 제목의
//!   `(section, paragraph)` 만 남기고 본문 문단 각각의 주소는 접는다. 그래서 청크의
//!   인용 앵커는 **소속 제목의 주소**이지 본문 문단별 오프셋이 아니다.
//! - **문단↔표 정밀 인터리브**: 같은 이유로 세그먼트 안에서 본문 텍스트가 표보다
//!   앞서고, 표는 문서 위치 순으로 뒤에 온다. 문단과 표의 정확한 끼워넣기는 후속
//!   과제다.
//! - **페이지 번호**: 구조 IR 은 논리 구조를 주지 물리 페이지를 주지 않으므로 청크는
//!   페이지 번호를 싣지 않는다(지어내지 않는다).
//! - **다단(multi-column) 읽기 순서**: IR 이 주는 읽기 순서를 그대로 쓴다.
//!
//! [`build_structure`]: crate::document_core::queries::structure::build_structure

pub mod chunker;

pub use chunker::{
    build_chunks, estimate_tokens, ChunkKind, ChunkOptions, ChunkTableRef, LlmChunk,
    TOKEN_ESTIMATOR,
};
