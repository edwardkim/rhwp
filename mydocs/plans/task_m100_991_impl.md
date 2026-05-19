# 구현계획서 — 쪽 분할된 글자처럼 취급 표의 셀 내용 렌더링 누락

- 타스크: 로컬 task991
- 브랜치: `local/task991`
- 수행계획서: `task_m100_991.md` (승인 완료)
- 작성일: 2026-05-19

## 추가 확인된 사실 (수행계획서 이후)

렌더러 코드 추적 + SVG 산출물 정밀 비교로 버그 범위가 좁혀졌다.

- 문제의 `treat_as_char=true` 표는 빈 문단에 단독 앵커되어 6→7쪽으로 **쪽 분할**된다(`PartialTable`).
- SVG 산출물 확인 결과 **표 테두리(외곽 박스 rect/line)는 6·7쪽 모두 정상 렌더링**된다.
- 누락된 것은 **셀 내부 5개 문단의 텍스트뿐**이다.
- 즉 버그는 `src/renderer/layout/table_partial.rs` 의 `layout_partial_table` → 셀 내용 렌더 경로(`compute_cell_line_ranges` / 분할 행 line_ranges 처리)에 국한된다.

가설: 쪽 분할 행(`is_in_split_row`)에서 셀 문단의 가시 줄 범위(`line_ranges`)가 빈 범위로 계산되거나, 분할 오프셋(`split_start_content_offset` / `split_end_content_limit`)과 셀 문단의 줄 좌표가 어긋나 모든 줄이 잘려나가는 것으로 보인다. 정확한 지점은 1단계에서 확정한다.

## 구현 단계 (3단계)

### 1단계 — 누락 지점 확정 (조사, 소스 수정 없음)

- `layout_partial_table` 의 셀 내용 렌더 경로를 추적한다:
  - `compute_cell_line_ranges` 가 이 셀(빈 문단 단독 앵커, 5개 셀 문단)에 대해 반환하는 `line_ranges` 값 확인.
  - `split_start_content_offset` / `split_end_content_limit` 와 셀 문단 줄 좌표(`compose_paragraph` 결과)의 정합성 확인.
  - 6쪽(split_end)·7쪽(split_start) 각각에서 어느 줄이 가시/비가시로 판정되는지 확인.
- 디버그 출력(임시 로그 또는 기존 `dump`/`dump-pages`)으로 가설을 검증한다.
- 산출물: `task_m100_991_stage1.md` — 누락 근본 원인과 정확한 수정 지점.

### 2단계 — 수정 구현

- 1단계에서 확정한 지점을 수정하여 분할된 TAC 표의 셀 문단이 정확한 줄 범위로 렌더링되도록 한다.
- 수정 범위는 `src/renderer/layout/table_partial.rs` (또는 1단계가 지목하는 관련 렌더러 파일)로 한정한다. 파서·문서 모델·페이지네이션 로직은 건드리지 않는다.
- 비-분할 TAC 표, 비-TAC 분할 표 등 인접 케이스에 회귀가 없도록 최소 침습으로 수정한다.
- 산출물: `task_m100_991_stage2.md` + 소스 커밋.

### 3단계 — 검증 및 보고

- `cargo build` / `cargo test` 전체 통과 확인.
- `cargo clippy` 경고 없음 확인.
- 골든 SVG 테스트(`tests/golden_svg/`) 회귀 없음 확인.
- 비공개 샘플 재현: `export-svg` 로 셀 5개 문단(불릿 목록)이 6·7쪽에 정상 표시되는지, 한컴 PDF와 시각 정합되는지 확인.
- 다른 공개 샘플로 분할 표/TAC 표 교차 회귀 확인.
- WASM 동작에 영향 있으면 Docker로 WASM 재빌드.
- 산출물: `task_m100_991_stage3.md` + `report/task_m100_991_report.md` + `orders/20260519.md` 갱신.

## 비공개 문서 / 테스트 픽스처 처리

- 재현용 HWPX/PDF는 커밋하지 않는다.
- 회귀 방지 테스트가 필요하면:
  - 우선 공개 가능한 샘플 중 동일 구조(빈 문단 단독 앵커 + 쪽 분할 TAC 표)를 찾아 골든 테스트로 추가한다.
  - 적합한 공개 샘플이 없으면 비공개 픽스처 기반 테스트는 비커밋 처리하고, 그 사실과 검증 결과를 3단계 보고서에 명시한다.
- 커밋 전 `git status` 로 비공개 hwp/pdf 가 스테이징되지 않았는지 확인한다.

## 범위 제외

- 페이지 수 ±1 누적 드리프트(별도 타스크).
- HWP3 경로.
- 파서·공통 문서 모델·페이지네이션 로직.
