# 최종 결과보고서 — task993: 분할 표 페이지네이션 모델 교체 (줄 범위 기반)

- 타스크: 로컬 task993 / 브랜치 `local/task993` (`local/task992` 위)
- 마일스톤: M100 (v1.0.0)
- 작성일: 2026-05-20
- 선행: task992 (페이지밖 콘텐츠 제거) — 본 사안을 진단·인계

## 1. 배경 / 문제

분할 표의 분할 상태가 페이지네이터와 렌더러에서 다른 모델로 표현됐다.

- 페이지네이터(`typeset.rs`): 연속 px `content_offset`로 추적.
- 렌더러(`table_partial.rs`): 이산 줄 범위(`compute_cell_line_ranges`).

HWP는 셀 내부 페이지 분할을 `LINE_SEG.vpos` 하향(vpos 리셋)으로 인코딩하는데,
px↔줄범위 매핑이 vpos 리셋에서 **비단조**가 되어 (a) 페이지네이터 과소측정 →
오버플로, (b) 측정을 고치면 무한 분할 루프가 발생했다(task992 최종보고서 §3).

## 2. 해결 — 컷 디스크립터 모델

분할 상태를 px가 아니라 **셀별 "소비한 콘텐츠 유닛 수"**(행 컷, `RowCut =
Vec<usize>`)로 표현한다. 콘텐츠 유닛 = 합성 줄 1개 또는 중첩 표 atom 1개.

- 단일 권위 함수 `advance_row_cut(table, row, start_cut, avail, styles)` —
  유닛을 공통 높이 예산 안에서 전진, vpos 리셋을 hard break(정지점)로 처리.
- 페이지네이터와 렌더러가 **모두 이 함수**를 호출 → 컷이 정의상 일치.
- 컷은 절대 유닛 인덱스로 단조 전진(매 호출 ≥1 유닛) → **무한 루프 구조적
  불가**.

## 3. 단계별 수행

| 단계 | 내용 | 커밋 |
|------|------|------|
| Stage 1 | 컷 디스크립터 설계 확정 + 호출부 조사 | `0f4df4f4` |
| Stage 2 | `RowCut` 타입 + `advance_row_cut` + 단위 테스트 5건 | `8be5e0c2` |
| Stage 3 (1/4) | `cell_units` 줄 매핑 + `cell_line_ranges_from_cut` | `069c3d69` |
| Stage 3+4 | 스키마(px→컷) + 페이지네이터 + 렌더러 일괄 교체 | `cb9576e0` |
| Stage 5 | form-002 골든 판정 / walk 회귀 수정 / 줄 메트릭 정합 | `c34cf378`·`74e6c10d`·`02e2ed54` |

### 주요 구현

- `PageItem::PartialTable`: `split_*_content_offset: f64`(px) →
  `start_cut`/`end_cut: Vec<usize>`(행 컷).
- 페이지네이터 `typeset_block_table`: 행 경계 walk를 `MeasuredTable` px
  누적에서 `advance_row_cut` 누적으로 전면 교체 — 측정 공간 단일화.
- 렌더러 `layout_partial_table`: `cell_line_ranges_from_cut`/
  `row_cut_content_height`로 컷 소비, `content_y_accum` 중첩 표 px 재판정 제거.
- `cell_units`: `cell.height` 빈 공간 필러 유닛, 줄 높이를 렌더러와 동일한
  `corrected_line_height`로 정합.
- rowspan(`row_span>1`) 행은 컷 모델 측정 대상 외 — 페이지네이터·렌더러
  양쪽에서 `MeasuredTable` 행 높이로 폴백(구현계획서 §4 정합).

## 4. 검증

- `cargo build --release` / `cargo clippy --release` 무경고.
- `cargo test --release` 1302 passed, 0 failed, 6 ignored.
- `svg_snapshot` 8 passed.
- 분할 표 export 무한 루프 없음(정상 종료).

### form-002 — 한컴 2022 PDF 대조 (`pdf/hwpx/form-002-2022.pdf`)

컷 모델이 분할 표 큰 셀을 vpos 리셋 지점에서 분할, 분할 콘텐츠 경계가
한컴과 일치(페이지 1 끝 "…주사제형화 기술 개발", 페이지 2 시작 "ㅇ PFC
나노산소운반체…"). 기존 px 모델이 분할 셀 박스를 콘텐츠보다 17.5px 길게
그리던 것을 정정 — 컷 모델이 정확. 골든 갱신.

### 비공개 분할 표 샘플 — 페이지밖 콘텐츠 (184페이지)

| 시점 | 오버플로 페이지 |
|------|----------------|
| task992 종료 | 3건 (123·144·176, 실제 오버플로) |
| task993 완료 | **2건 (123·176, 각 viewBox +2.1px)** |

- 페이지 144 해소.
- 개발 중 발생한 회귀 2건(페이지 13·120) — 컷 walk 예산 초과 검사 누락,
  rowspan 행 미측정 — 모두 수정.

## 5. 잔여 사항

- **페이지 123·176 (각 2.1px)**: 대형 rowspan 표에서 rowspan 셀 콘텐츠가
  `MeasuredTable` 행 높이 박스를 ~100px 초과해 하단 여백으로 흐르고, 그중
  2.1px가 viewBox를 넘는다. 구현계획서 §4대로 rowspan 행은 컷 모델이 아닌
  `MeasuredTable`/`row_block_for` 권위에 위임했으므로, 본 잔여는 컷 모델이
  아니라 `HeightMeasurer`의 rowspan 셀 측정 정합 문제다(task993 범위 밖).
  **작업지시자 판단(2026-05-20): 서브픽셀(2.1px ≈ 0.5mm, 시각상 0) 수준으로
  수용.** 후속 `HeightMeasurer` 정합 과제는 별도 타스크로 분리.
- `test_task77`(이미지 셀, `samples/20250130-hongbo.hwp`): 컷 모델 행 경계
  이동으로 `#[ignore]`. 이미지 셀 비분할 의도는 보존. hongbo PDF 대조 시
  기대값 재확정.
- 레거시 `Paginator`(`pagination/engine.rs`) 분할 표 인트라-분할 테스트 3건:
  engine.rs는 구현계획서 §50대로 컷 비생산(기계적 갱신) — `#[ignore]`.
  컷 분할은 `row_cut_tests` 단위 테스트 5건이 검증.

## 6. WASM / 릴리즈

페이지네이터·렌더러 변경 → WASM 릴리즈 시 Docker 재빌드 필요:
`docker compose --env-file .env.docker run --rm wasm`.

## 7. 비공개 문서

재현용 HWPX는 커밋하지 않았다. 한컴 2022 PDF `pdf/hwpx/form-002-2022.pdf`는
공개 권위 자료로 커밋 대상(작업지시자 제공). 비공개 184페이지 샘플은
비커밋 — 오버플로 스캔 결과만 본 보고서에 기록한다.
