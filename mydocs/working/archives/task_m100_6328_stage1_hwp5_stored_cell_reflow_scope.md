# Stage 1: #6328 HWP5 저장 셀 재래핑 범위 분석

## 목적

통합 PR #6328의 CI가 `text_overlap` baseline을 넘긴 원인을 고치되, #5952의
실제 HWP5 유의사항 상자 overflow와 #6175 square-band 회귀를 유지한다. baseline
파일을 갱신해 실패를 허용하지 않는다.

## Stage 1.1: 도입 범위와 baseline 비교

- #5952 도입 전 `9ac5f9b3`에서 overlap sweep은 샘플 946건, nonzero 152종,
  총 4371건으로 통과했다.
- #6328의 최신 integration head는 #5952의 다중 저장 행 재래핑을 포함한다.
- 원래 #5952 보정은 저장 행이 둘 이상인 native HWP5 셀에서 다음 중 하나면
  fresh reflow를 허용했다: 합성 행 수 감소, 한 합성 행이 글자 대부분을 차지,
  또는 저장폭 대비 과폭.
- 이 범위는 실제 #5952 SVG assertion과 #6175 fixture를 통과했지만, overlap
  gate에서는 946건 중 nonzero 153종, 총 4408건으로 실패했다.

## Stage 1.2: 실패한 중간 후보

아래 후보들은 모두 baseline을 갱신하지 않고 실행했다.

| 후보 | #5952 실제 SVG | #6175 fixture | overlap 결과 | 판정 |
| --- | --- | --- | --- | --- |
| 저장폭 일치·2~3행으로 제한, 폭 과폭 포함 | 통과 | 통과 | 총 4414, 행정 편람 15→26 등 증가 | 범위 과다 |
| 정확히 한 합성 행으로 제한 | 실패 | 미실행 | 미실행 | 실제 #5952 대상 제외 |
| 실제 텍스트 시작점만 허용 | 실패 | 미실행 | 미실행 | 실제 HWP5 저장 행 표현과 불일치 |

## Stage 1.3: 실제 #5952 대상 메타데이터

동일한 release 바이너리로 `samples/2025 행정업무운영 편람(최종).hwp`의 69쪽
SVG를 직접 export하고 저장 행을 관찰했다. 문제가 되는 유의사항 행은 다음 공통
형태를 가진다.

- 셀 내폭은 약 `500.8px`, 저장 `segment_width`는 각 행 `37560HU`다.
- 저장 행은 2개이며 첫 합성 행은 49~54자, 561~587px으로 셀 폭의 약
  1.12~1.17배다.
- 둘째 합성 행은 짧고, 본문은 `※` 또는 `☞`로 시작하는 유의사항 bullet이다.
- 일반 다중행 본문 셀도 저장폭 일치와 약한 과폭을 흔히 가진다. 따라서 폭만으로
  fresh reflow를 허용하면 overlap baseline을 악화시킨다.

관찰 로그는 추적 대상이 아닌
`target/pr-6328-lib-unit-20260829/issue-5952-direct-diagnostic.stderr`에만 둔다.

## Stage 2 계획

1. 임시 진단 출력을 제거한다.
2. #5952의 실제 유의사항 bullet 구조(`※`/`☞`)와 저장폭/과폭을 함께 확인하는 최소
   predicate를 구현한다. 일반 다중행 본문, 끝의 빈 저장 행, 폭만 다른 행은
   재래핑하지 않는다.
3. #5952 실제 SVG 4건, #6175 fixture 1건, `text_overlaps_do_not_grow` 순으로
   재검증한다.
4. 세 gate가 모두 통과할 때만 maintainer correction commit과 PR push를 진행한다.

## 현재 상태

Stage 1 분석 완료. Stage 2 후보는 구현됐으며, 임시 진단 출력은 제거했다.

### Stage 2 검증 기록

- `cargo fmt --all -- --check`: 통과.
- `cargo test --release --test regression_suite_006 issue_5952_cell_note_overflow -- --nocapture`:
  4 passed. 실제 69쪽 SVG의 사이드바 비겹침과 상자 하단 분리를 포함한다.
- `cargo test --release --test regression_suite_025
  issue_6175_uniformly_narrowed_ladder_keeps_square_band -- --nocapture`: 1 passed.
- `cargo test --release --test regression_suite_005 text_overlaps_do_not_grow -- --nocapture`:
  통과. 샘플 946건, nonzero 152종, 총 4371건으로 baseline과 동일하다.

### Stage 2 결론

- repair predicate는 `※`/`☞` 유의사항 bullet, 저장 2~3행, 각 저장폭과 셀 내폭의
  1px 이내 일치, 합성 행의 1.10배 초과를 모두 요구한다.
- #5952 실제 문서의 overflow를 해소하면서 일반 HWP5 다중행 셀 재래핑은 막았다.
- Stage 3에서는 이 세 파일만 maintainer correction commit으로 기존 integration PR에
  push하고, 새 head의 CI를 다시 확인한다.
