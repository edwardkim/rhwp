# 구현계획서 — task_m100_3128

- **Issue**: #3128
- **브랜치**: `codex/issue-3128-continuation-geometry`
- **작성일**: 2026-08-18 KST
- **상태**: 소급 계획 승인, Stage 4 전체 검증 완료

> **소급 작성 고지.** 아래 내용은 구현 전에 승인받은 설계가 아니라, 이미 작성된 로컬 변경을
> 재검토해 파일별 의도와 위험 경계를 기록한 것이다. 승인 전에는 구현을 더 확장하지 않는다.

## 1. 원인 모델

34쪽 오차는 서로 독립적인 세 층이 누적된 결과다.

1. **재조판 폭**: 저장 `LINE_SEG`가 없는 셀 fallback이 첫 글자모양만 사용해 음수 tracking 구간과
   한컴의 반각 literal-space advance를 잃었다.
2. **content box**: `applyInnerMargin=false`인 1×1 child에도 510HU saved cell margin 호환 경로가
   적용돼 PDF보다 좁은 폭에서 한 줄이 추가됐다.
3. **terminal flow**: native terminal RowBreak child가 source cursor를 이미 소유하는데 parent mixed
   flow가 빈 host tail의 first unit을 다시 예약해 후속 표를 아래로 밀었다.

## 2. 파일별 변경

| 파일 | 변경 의도 |
| --- | --- |
| `src/renderer/composer.rs` | 일반 셀 계약은 보존하고, 구조 gate를 통과한 장문 child만 CharShape tracking·반각 공백·1px wrap tolerance를 사용하는 opt-in 재조판 추가 |
| `src/renderer/layout/table_layout.rs` | 1×1 content-box gate를 렌더·측정에 공통 적용하고 terminal native child의 중복 first-unit 예약 제거 |
| `src/renderer/layout/table_partial.rs` | partial/lazy cell compose도 동일 opt-in 계약을 사용하도록 전달 |
| `tests/issue_2308_render_normalized_derived_state.rs` | stale saved-margin 가정을 PDF 기반 content-box·370.9px continuation 계약으로 정정 |
| `tests/cases/issue_3128_terminal_nested_table_geometry.rs` | 82쪽, p34 외곽·후속 표 좌표, `연동시스템 등` wrap을 고정하고 generated suite 자동 배정 정책을 따름 |
| `tests/suites/unit-test-tiers.json` | 코드 삽입으로 이동한 기존 module 5곳·support item 2곳의 줄 번호만 재계산; 테스트 수·상한 불변 |

## 3. 구조 gate

새 tracking/content-box 경로는 다음 조건을 모두 만족할 때만 선택한다.

- native non-TAC 1×1 table, 단일 cell
- table left/right inMargin 0
- `applyInnerMargin=false`
- 작은 양수 saved cell margin
- 저장 `LINE_SEG` 없음
- 둘 이상의 literal ASCII 선행 공백
- 참조 글자모양의 font family, size, bold/italic, ratio, kerning이 동일
- 글자모양 구간 사이에 letter spacing 차이가 존재

legacy bullet의 regenerated-space 경로와는 상호 배타적으로 둔다. 일반
`recompose_for_cell_width` 호출자는 새 동작을 얻지 않는다.

## 4. 테스트 전략

### 수용 테스트

- `issue_3128_terminal_nested_table_geometry`
- `issue_2308_render_normalized_derived_state`
- `issue_1891`

### 인접 회귀

- #2430 cell rewrap threshold
- #3820 RowBreak/rowspan band
- #2007·#3637 nested table pagination
- overflow cell baseline 전 샘플

### 시각 검증

`scripts/visual_sweep.py`로 76076 HWP와 HWP 2024 PDF의 34쪽만 96dpi 비교한다. 전역 폰트 paint 차이로
전체 ink match를 완료 조건으로 쓰지 않고, issue-specific table border, continuation height, 후속 표
anchor와 줄바꿈을 render-tree 테스트와 함께 판정한다.

## 5. 위험과 방어

| 위험 | 방어 |
| --- | --- |
| 전역 tracking 복원으로 타 문서 페이지 수 변경 | 명시적 opt-in API와 구조 gate, #1891 페이지 수 회귀 |
| saved margin 호환 회귀 | #3128 content-box child만 기존 compat 경로에서 제외 |
| p81→p82 short child boundary 변경 | symbol-led child 제외, owner-content-box 회귀 유지 |
| terminal flow 보정의 일반화 | 기존 native terminal source-cursor eligibility를 재사용 |
| PDF raster의 글꼴 차이를 geometry 해결로 오판 | 좌표·line wrap 계약과 side-by-side 시각 판정을 함께 사용 |

## 6. 승인 후 남은 실행

1. 최신 `upstream/devel`을 fetch하고 branch divergence를 확인한다.
2. 충돌이 있으면 사용자 변경을 보존하며 최신 기준으로 재검증한다.
3. 별도 승인 후 renderer 범위의 전체 release·Clippy·WASM gate를 실행한다.
4. 최종보고서 수치와 오늘할일 상태를 갱신한다.
5. 단계별 커밋을 만든다.
6. 별도 승인 후 push하고 `devel` 대상 PR을 생성한다.
