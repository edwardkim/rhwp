# Task M100-2439 Stage 7 — 표 헤더 열 폭·fragment 여백·행내 컷 정합

- 이슈: [#2439](https://github.com/edwardkim/rhwp/issues/2439)
- 브랜치: `fix/2439-split-table-flow`
- 작성일: 2026-07-20
- 선행 기록: [Stage 6](task_m100_2439_stage6.md)

## 1. 추가 분석

Stage 6 이후 정답 PDF의 2~3쪽을 확대 비교해 다음 잔여 문제를 확인했다.

- 2쪽의 선행 정보 표와 본 표 사이가 정답보다 좁았다.
- 본 표 첫 헤더 행의 극단적으로 작은 저장 폭이 전체 열 격자의 기준으로 채택되어
  `일자`/`점검 항목`이 깨지고 `비고` 열이 비정상적으로 넓어졌다.
- 셀 padding을 제외한 높이로 orphan guard를 판단해 2쪽 마지막 행이 3쪽으로 넘어갔다.
- 3쪽 연속 fragment의 바깥 여백과 뒤따르는 서명문 flow가 서로 다른 기준을 사용해
  표와 텍스트가 겹쳤다.

원본 PDF와 HWP IR을 함께 확인한 결과, 2쪽 헤더에는 별도의 대각선을 합성할 근거가
없었다. 따라서 대각선을 임의로 추가하지 않고 저장 셀 폭과 텍스트 배치만 복원했다.

## 2. 구현

### 열 격자와 행별 폭 역할 분리

`Table`의 행 폭 패턴을 두 역할로 나눴다.

- `base_grid_outlier_rows`: 반복되는 지배적 폭 벡터와 다른 소수 행은 기본 열 폭 산출에서
  제외한다.
- `inferred_local_resize_rows`: 행 합계가 지배적 행 또는 선언 표 폭과 맞는 경우에만
  보상 resize 행으로 인정한다.

이로써 2쪽의 축퇴된 헤더 행은 기본 격자를 오염시키지 않지만, 실제 보상 resize 행은
기존처럼 독립 경계를 유지한다. 추론 행의 양수 residual은 마지막 셀을 임의로 늘리지
않고 기본 열 경계로 fallback한다.

### RowBreak fragment flow

- orphan guard의 visible fragment 높이에 셀 top/bottom padding을 포함했다.
- native HWP5의 구조 증거가 일치하는 단일 empty-host `TopAndBottom`/`RowBreak` 표에서만
  저장 LineSeg, 양수 offset, fragment outer margin 계약을 사용한다.
- 첫 fragment는 `이전 outer-bottom + 현재 outer-top + vertical-offset`, 연속 fragment는
  현재 outer-top을 사용한다. vertical offset은 첫 fragment에만 적용한다.
- full `PageItem::Table`과 첫 `PartialTable`이 같은 `outer-top + vertical-offset` 상단
  좌표를 사용하도록 맞췄다.
- trailing outer-bottom은 painted content가 아니라 다음 flow의 여백으로 처리했다.

구조 판별은 `float_placement.rs`의 공용 helper로 중앙화했다. native HWP5, 비-TAC,
빈 host, Para 기준 TopAndBottom, RowBreak, 양수 offset, 단일 표, 비합성 저장 LineSeg,
뒤따르는 일반 텍스트 문단이 모두 성립해야 한다. #2097에서 반증된 광범위한 outer-margin
가산은 적용하지 않는다.

## 3. 2~3쪽 좌표 확인

최종 render tree 기준:

- 2쪽 선행 정보 표 하단: 약 `130.8px`
- 2쪽 본 표 상단: 약 `143.6px`
- 두 표 사이: 약 `12.8px` = `965HU`
  (`283HU` 이전 bottom + `283HU` 현재 top + `399HU` offset)
- 2쪽 본 표 마지막 행: `드롭센서 기능 체크`까지 같은 쪽에 존재
- 3쪽 연속 fragment: 상단 약 `41.6px`, 하단 약 `57.9px`
- 3쪽 서명문 상단: 약 `61.1px`; 표와 약 `3.2px` 간격, 겹침 없음

2쪽의 `일자`/`점검 항목`은 각 영역에 읽을 수 있게 배치되고, `비고` 열은 정답 PDF와
같은 좁은 폭으로 돌아왔다. 3쪽 상단에는 의도된 마지막 행의 연속 부분만 남는다.

## 4. 회귀 검증

- `cargo test --lib`: 2,347 passed, 0 failed, 7 ignored
- 관련 통합 테스트 10개 target: 36 passed, 0 failed
  - `issue_1510`, `issue_1535`, `issue_1549`, `issue_1663`
  - `issue_1772_table_outer_margin_sync`
  - `issue_1789_exclusion_probe_line_spacing`
  - `issue_2097_band_fill`
  - `issue_2322_fullpage_form_table_pair`
  - `issue_2439`, `issue_493_cell_attrs`
- `cargo fmt --all`: 통과
- `git diff --check`: 통과
- 최종 read-only 코드 재검토: 잔여 P1/P2 없음
- `wasm-pack build --target web --out-dir pkg`: 통과

추가 회귀 테스트는 다음 경계를 고정한다.

- 축퇴 폭 행이 기본 열 격자를 늘리지 않음
- 합계가 보존된 보상 resize 행은 그대로 유지
- padding을 포함한 orphan guard가 마지막 행을 불필요하게 넘기지 않음
- native empty-host evidence의 true/false 경계
- full 표와 첫 partial fragment의 상단 좌표 동일성
- 실제 full `PageItem::Table`의 outer-top 증가량이 렌더 좌표에 그대로 반영됨

## 5. 최종 PDF visual sweep

- 결과 루트: `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720`
- rhwp/PDF: 10/10쪽
- compare/overlay/review: 각 10쪽
- 자동 이상 후보: 0/10
- `LAYOUT_OVERFLOW`: 없음
- 평균 `pixel_match_percent`: 89.60574%
- 평균 `visual_accuracy_proxy_percent`: 6.80340%
- 최저 `visual_accuracy_proxy_percent`: 4.60630% (2쪽)

| 쪽 | compare | overlay | review | visual accuracy proxy |
|---:|---|---|---|---:|
| 1 | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/compare/compare_001.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/overlay/overlay_001.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/review/review_001.png` | 4.75947% |
| 2 | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/compare/compare_002.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/overlay/overlay_002.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/review/review_002.png` | 4.60630% |
| 3 | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/compare/compare_003.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/overlay/overlay_003.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/review/review_003.png` | 5.13746% |
| 4 | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/compare/compare_004.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/overlay/overlay_004.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/review/review_004.png` | 5.26445% |
| 5 | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/compare/compare_005.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/overlay/overlay_005.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/review/review_005.png` | 5.84580% |
| 6 | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/compare/compare_006.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/overlay/overlay_006.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/review/review_006.png` | 5.62240% |
| 7 | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/compare/compare_007.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/overlay/overlay_007.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/review/review_007.png` | 5.57155% |
| 8 | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/compare/compare_008.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/overlay/overlay_008.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/review/review_008.png` | 5.59558% |
| 9 | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/compare/compare_009.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/overlay/overlay_009.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/review/review_009.png` | 5.34630% |
| 10 | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/compare/compare_010.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/overlay/overlay_010.png` | `/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720/issue2439-answer/review/review_010.png` | 20.28470% |

`review_001.png`부터 `review_010.png`까지 직접 확인했다. 자동 지표는 폰트 fallback의
영향을 크게 받으므로 호환성 점수가 아니라 내용 픽셀 중심의 보조값으로만 사용한다.

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 평균 약 6.80%.
높을수록 좋음: 기준 PDF와 rhwp PNG의 잉크 위치가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다

## 6. Studio와 범위

- WASM 산출물: `pkg/rhwp_bg.wasm`, 2026-07-20 재생성
- Studio PID: `81399`
- cwd: `rhwp-studio`
- 주소: `http://127.0.0.1:7700`
- HTTP 응답: 200

로컬 구현, 전체/관련 회귀, 10쪽 PDF 비교, WASM과 Studio 확인까지 완료했다. remote
push, PR 생성, 이슈 코멘트는 수행하지 않았다.
