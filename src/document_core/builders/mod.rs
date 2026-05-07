//! 외부 입력 변환 IR 빌더 (Neumann 작업, Task #660~).
//!
//! 외부 형식(이미지/PDF/MD/DOCX)을 [`Document`](crate::model::document::Document) IR로
//! 변환하는 도메인 빌더 모듈. Claude Code Skill (`rhwp-exam-ingest`)이 작성한
//! `ingest_schema_v1.json` 을 입력으로 받아 시험문제 표준 layout으로 IR을 구성한다.
//!
//! 본 모듈의 책임:
//! - JSON 중간 표현 → Document IR 변환 (텍스트, 선택지, 이미지 placement)
//! - 시험지 표준 ParaShape/CharShape 풀 (`exam_styles`)
//! - placement 4모드 IR 매핑 (`exam_layout`, Task #661)
//!
//! 본 모듈의 비책임:
//! - 외부 형식 파싱 (Skill 측 책임, helpers/*.{sh,py})
//! - Vision/OCR 분석 (Claude Code Skill의 멀티모달 능력)
//! - HWPX/HWP5 직렬화 (`crate::serializer`의 책임)

pub mod exam_paper;
