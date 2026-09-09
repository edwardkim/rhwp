# task_m100_3355 처리결과 보고서 — build-from-ingest 텍스트 런 테두리 상자 제거

- **이슈**: [#3355](https://github.com/edwardkim/rhwp/issues/3355)
- **브랜치**: `pr/fix-issue-3355-ingest-borderfill` (upstream/devel `4a39f7cc0` 직분기)
- **범위**: `src/document_core/builders/exam_paper.rs` (borderFill 1개 생성부 + 회귀 단언)
- **분류**: 버그 수정 (빌더 — 렌더 결함)

## 1. 배경

`build-from-ingest` 산출물을 렌더하면(공식 예제 `sample_minimal.json` 포함) **모든 텍스트
런이 검정 실선 상자로 둘러싸인다.** 숫자·하이픈 구간은 런 경계마다 상자가 끊겨
`2026-07-26` → `2026-│07-│26` 처럼 벌어져 보인다. SVG·PDF 공통, v0.8.0 릴리스와 devel 동일.

산출 HWPX 의 XML 검증으로 원인을 좁혔다:

- ingest 산출: `borderFill id=1` 이 **4면 SOLID** + `charPr id=0 → borderFillIDRef="1"`
- 정상(파서 라운드트립 산출): 대응 borderFill 이 **4면 NONE**

## 2. 원인

`init_exam_doc_info` 는 `vec![BorderFill::default(), boxed_border_fill]` 로 "기본(무테두리) +
boxed(실선)" 를 의도했지만, `BorderLineType` 의 Rust `#[default]` 가 **Solid(1)** 라서
`BorderFill::default()` 는 4면 실선이다 (바이너리 규약은 0=선없음). 이를 기본 글자모양
(`border_fill_id: 1`)이 참조해 전 텍스트에 상자가 그려졌다.

## 3. 설계 결정

- **빌더 국소 수정** — 첫 borderFill 을 `BorderLineType::None` 4면으로 명시 생성.
  파서·렌더러·직렬화기 무변경.
- **`#[default]` 이동은 별도 조각** — `BorderFill::default()` 를 "빈값"으로 믿는 자리가
  hwp3 파서·hml adapter·table_ops 등에 더 있어(잠재 동종 결함), 전수 감사가 필요한
  근본 수정은 #3355 에 명단을 남기고 분리했다.

## 4. 검증

- **회귀 단언 추가** (기존 빌더 테스트 확장): 기본 borderFill 4면 None /
  boxed borderFill 4면 Solid 유지
- 산출 HWPX 재검증: `charPr → borderFillIDRef=1` 이 참조하는 fill 이 NONE 으로 방출됨
- **시각 전/후**: 동일 ingest JSON 렌더 — 전: 전 텍스트 런 상자 / 후: 상자 없음
  (재현 명령 2줄이 PR 본문에 있음)
- `cargo fmt` clean, clippy `-D warnings` 0건, 빌더 테스트 전부 green

## 5. 남긴 것

- `BorderLineType` `#[default]` 를 규약값(None=0)으로 옮기는 근본 수정 — 사용처 전수
  감사 후 별도 PR (후보 자리 목록은 #3355 본문).
- boxed 블록 배경(F7F7F7)·실선은 의도된 스타일로 유지했다.
