# Task m100 #1891 Stage 9

## 목표

공식 PDF 기준으로 HWP5 원본과 HWPX, 그리고 HWP에서 export한 HWPX 재파스 결과의
쪽수를 모두 일치시킨다.

## 진행 원칙

- 특정 파일명, 페이지 번호, PR/issue 번호, 임의 계수로 분기하지 않는다.
- 보정은 입력 문서에서 읽을 수 있는 `LineSeg`, `ParaShape`, `CharShape`, 표/셀 속성,
  control 속성, section/page 속성 또는 공개 스펙 필드에 근거한다.
- #1906은 이번 검증 범위에서 제외한다.

## 분석 결과

- `80250`은 텍스트가 비어 있지만 `char_count`, `CharShape`, `ParaShape`가 남아 있는
  spacer 문단이 0 높이에 가깝게 측정되어 1쪽이 줄었다. 빈 텍스트/무 control/무 LineSeg
  문단이면서 문서에 남은 글자 모양이 의미 있는 크기일 때만 해당 `CharShape`와
  `ParaShape.line_spacing`으로 fallback line metric을 계산하도록 수정했다.
- `80168`은 단일 1행 RowBreak 표가 실제 조판 높이만 보면 남은 영역에 들어간다고
  판단되어 앞쪽에 붙었다. HWP CTRL_HEADER의 저장 object height가 남은 영역보다 크고,
  cell이 표 padding 기준 중앙 정렬을 쓰는 1행 표는 다음 쪽으로 넘기도록 했다.
- `80168` HWP export HWPX 재파스는 저장 행 높이보다 row unit content가 크게 초과하는
  RowBreak 행에서 저장 행 높이를 계속 사용해 1쪽이 줄었다. `row_span == 1` cell 내용
  기준 cut height가 저장 행 높이를 충분히 초과하면, 저장 LineSeg가 있어도 content cut
  height를 우선하도록 했다.
- `86712` HWP export HWPX 재파스는 HWP5-origin HWPX에 일반 HWPX RowBreak overflow
  tolerance를 적용해, 실제로 남은 영역보다 큰 partial row가 같은 쪽에 얹히며 1쪽이
  줄었다. HWP5-origin HWPX marker가 있는 문서는 HWP5 pagination 계열로 보고 일반 HWPX
  전용 overflow tolerance를 적용하지 않도록 했다.
- 기존 `issue_1133` HWP 샘플은 빈 host RowBreak 표의 선언 높이가 현재 쪽에 거의 정확히
  맞는 경우였다. 선언 높이 overflow 판정이 0.1px 수준의 부동소수 오차에도 다음 쪽으로
  이월되면서 HWP/HWPX debug marker 비교가 깨졌으므로, 저장 object height 기반 fit
  판정에 1px tolerance를 적용했다.

## 최종 쪽수 검증

| 샘플 | 기준 PDF | HWP | HWPX | HWP export HWPX |
| --- | ---: | ---: | ---: | ---: |
| `76076_regulatory_analysis` | 82 | 82 | 82 | 82 |
| `80168_regulatory_analysis` | 157 | 157 | 157 | 157 |
| `80250_regulatory_analysis` | 17 | 17 | 17 | 17 |
| `86712_regulatory_analysis` | 65 | 65 | 65 | 65 |

## 검증

- `env CARGO_INCREMENTAL=0 cargo build --bin rhwp`
- 공식 PDF/HWP/HWPX/HWP export HWPX 페이지 매트릭스 확인
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1891`
  - 3 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1133_nested_table_valign`
  - 3 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_rowbreak_chart_overlap`
  - 20 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --lib wasm_api::tests::test_reflow_linesegs_keeps_hwpx_sample2_page_count_for_textrun_warnings`
  - 1 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
  - passed
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`
  - passed
