# Task #1285 Stage 2 - TAC 표 inline/셀 라벨 배치 보정

## 목적

`samples/21_언어_기출_편집가능본.hwp` 1페이지 머리말 영역에서 한 셀 안에 배치된
`성명` TAC 표와 `수험번호` TAC 표가 한 줄에 유지되면서, `수험번호` 표 내부 라벨이
한컴 편집기 출력처럼 오른쪽에 붙도록 보정했다.

## 수정 요약

### 1. 마지막 줄 문단 끝 TAC 폭 포함

`src/renderer/layout/paragraph_layout.rs`의 `tac_offsets_for_line`은 기본적으로
`start <= pos < end` 규칙을 사용한다. 이 규칙은 다음 줄 선두 TAC를 현재 줄 폭에 잘못
포함하지 않기 위한 #1219 방어선이다.

하지만 이번 샘플의 부모 셀 문단은 공백 4자 뒤, 즉 문단 마지막 위치(`pos == end`)에
두 번째 TAC 표(`수험번호`)가 걸려 있다. 렌더 경로는 마지막 run의 `run_char_end` 위치 TAC를
포함하지만, 줄 점유 폭 추정 경로의 `line_tac_offsets`는 이를 제외했다. 그 결과 오른쪽 정렬
기준 폭이 `성명 표 + 공백`까지만 계산되어, 두 번째 TAC 표의 시작점만 부모 셀 안에 걸치고
표 대부분이 셀 밖으로 나갔다.

수정 후에는 마지막 줄에 한정하여 `pos == end` TAC를 `line_tac_offsets`에 포함한다.

### 2. TAC 폭 중복 계상 방지

줄 점유 폭 추정 경로에서 run 내부 TAC 폭은 이미 `est_x += tac_w`로 반영된다. 이후 fallback에서
전체 `total_tac_width_in_line`을 다시 더하면, 한 줄에 TAC가 여러 개 있는 문단에서 줄 폭이
과대 계산될 수 있다.

수정 후에는 추정 경로에서 실제 포함한 TAC 폭을 `included_tac_width_in_est`로 추적하고,
fallback에서는 누락된 폭만 보정한다.

### 3. 셀 문단 자간 보정 후 실제 렌더 폭 기준 정렬

셀 문단에서 자간 보정이 들어간 경우 정렬 기준 폭은 자연 폭이 아니라 실제 렌더 폭이어야 한다.
기존에는 양수 자간 확장 일부만 고려했기 때문에, 음수 자간 압축이 들어간 오른쪽 정렬/좁은 셀
라벨에서 x 위치가 왼쪽으로 밀릴 수 있었다.

수정 후에는 셀 문맥, 비-justify/distribute, 탭 없음 조건에서 `extra_char_spacing`이 적용된
실제 측정 폭을 다시 계산해 정렬 기준으로 사용한다.

### 4. 좁은 Center 셀 라벨의 한컴식 오른쪽 packed 배치

이번 샘플의 `수험번호` TAC 표 첫 셀은 파일상 ParaShape가 Center로 해석되지만, 한컴 출력에서는
좁은 셀 라벨이 오른쪽에 붙어 보인다. 넓은 중앙 정렬 제목에는 영향을 주지 않도록 다음 조건을
모두 만족하는 경우에만 Center를 오른쪽 packed 배치처럼 처리했다.

- table-cell 문맥
- `Alignment::Center`
- 탭 없음
- 문자 수 4자 이상
- 라인 폭 110px 이하
- 실제 텍스트 폭이 라인 폭의 75% 이상

## 회귀 테스트

추가 파일:

- `tests/issue_1285_tac_sequence_right_align.rs`

테스트 내용:

1. 부모 셀에서 `성명` TAC 표, 공백 4자 TextRun, `수험번호` TAC 표가 같은 줄에서 연속 배치되는지 검증
2. `수험번호` TAC 표의 오른쪽 끝이 부모 셀 내부 TextLine 오른쪽 끝과 일치하는지 검증
3. `수험번호` TAC 표 첫 셀의 TextRun 오른쪽 끝이 TextLine 오른쪽 끝과 일치하는지 검증

## 산출물

- SVG:
  `output/poc/task1285/stage2_fixed/21_언어_기출_편집가능본_001.svg`
- render tree:
  `output/poc/task1285/stage2_fixed_tree/render_tree_001.json`

수정 후 render tree 기준 부모 셀 내부 배치:

| 항목 | x | w | right |
|---|---:|---:|---:|
| 부모 셀 내부 TextLine | 177.2 | 618.6 | 795.8 |
| `성명` TAC 표 | 296.7 | 189.0 | 485.7 |
| 공백 4자 TextRun | 485.7 | 25.0 | 510.7 |
| `수험번호` TAC 표 | 510.7 | 285.1 | 795.8 |

수정 후 render tree 기준 `수험번호` 라벨:

| 항목 | x | w | right |
|---|---:|---:|---:|
| TextLine | 512.6 | 92.8 | 605.4 |
| `수험번호` TextRun | 528.4 | 77.0 | 605.4 |

## 검증 명령

```bash
cargo fmt --check
cargo test --test issue_1285_tac_sequence_right_align -- --nocapture
cargo test --test issue_1219_equation_line_hangul_advance -- --nocapture
cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture

cargo run --bin rhwp -- export-svg \
  samples/21_언어_기출_편집가능본.hwp \
  -o output/poc/task1285/stage2_fixed --debug-overlay -p 0

cargo run --bin rhwp -- export-render-tree \
  samples/21_언어_기출_편집가능본.hwp \
  -o output/poc/task1285/stage2_fixed_tree -p 0
```

## 판정

로컬 회귀 테스트는 통과했다. 메인테이너 시각 판정은
`output/poc/task1285/stage2_fixed/21_언어_기출_편집가능본_001.svg`로 진행한다.
