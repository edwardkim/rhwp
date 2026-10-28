# Task #1285 완료 보고서

## 요약

`samples/21_언어_기출_편집가능본.hwp` 1페이지 머리말 영역에서 한 셀 안에 있는
`성명` TAC 표와 `수험번호` TAC 표의 inline 배치와 `수험번호` 라벨 위치를 보정했다.

조사 결과, 이 샘플에서 TAC 표의 column-sum 폭 추출 자체는 맞았다. 실제 원인은 부모 문단의
마지막 위치(`pos == end`)에 걸린 두 번째 TAC 표가 줄 점유 폭 계산에서는 누락되고, 렌더 경로에서는
그려지는 불일치였다. 이 때문에 오른쪽 정렬 기준 폭이 `성명 표 + 공백`까지만 계산되어 두 번째
TAC 표의 시작점만 부모 셀 안에 걸쳤다.

추가로 줄 점유 폭 추정 경로에는 이미 포함한 TAC 폭을 fallback에서 다시 더할 수 있는 구조적
위험이 있어 함께 보정했다. 좁은 TAC 표 내부 셀 라벨의 정렬 기준 폭/packed 배치 차이도 같이
보정했다.

## 변경 내용

- `src/renderer/layout/paragraph_layout.rs`
  - 마지막 줄 문단 끝 위치(`pos == end`) TAC를 `line_tac_offsets`에 포함
  - 줄 점유 폭 추정에서 이미 포함한 TAC 폭을 추적하고, fallback에서는 누락된 TAC 폭만 보정
  - 셀 문단에서 자간 보정 후 실제 렌더 폭을 정렬 기준으로 사용
  - 좁은 Center table-cell 라벨을 한컴 출력과 유사하게 오른쪽 packed 배치로 처리
- `tests/issue_1285_tac_sequence_right_align.rs`
  - 답안지 머리말의 두 TAC 표 inline sequence 회귀 테스트 추가
  - 두 번째 TAC 표의 오른쪽 끝이 부모 셀 내부 TextLine 오른쪽 끝과 일치하는지 검증
  - `수험번호` 라벨의 오른쪽 packed 배치 회귀 테스트 추가

## 산출물

- `output/poc/task1285/stage2_fixed/21_언어_기출_편집가능본_001.svg`
- `output/poc/task1285/stage2_fixed_tree/render_tree_001.json`

## 검증

| 항목 | 결과 |
|---|---|
| `cargo fmt --check` | 통과 |
| `cargo test --test issue_1285_tac_sequence_right_align -- --nocapture` | 통과 |
| `cargo test --test issue_1219_equation_line_hangul_advance -- --nocapture` | 통과 |
| `cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture` | 통과 |
| SVG export | 통과 |
| render tree export | 통과 |
| 메인테이너 SVG 시각 판정 | 통과 |
| rhwp-studio 웹 시각 판정 | 통과 |
| WASM 빌드 | 통과 |

## 판정

TAC 표 2개가 한 줄에 유지되는 부모 셀 배치와 `수험번호` 라벨의 셀 내부 배치가 보정됐다.
메인테이너가 `output/poc/task1285/stage2_fixed/21_언어_기출_편집가능본_001.svg`와
rhwp-studio 웹 화면에서 모두 시각 판정 통과를 확인했다.

이번 타스크는 성공으로 완료 판정한다.
