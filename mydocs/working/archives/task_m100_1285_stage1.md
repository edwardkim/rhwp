# Task #1285 Stage 1 - TAC 표 inline 폭 계산 조사

## 목적

`samples/21_언어_기출_편집가능본.hwp` 1페이지 머리말 영역에서 한 셀 안에 배치된
`성명` 표와 `수험번호` 표의 inline 배치/정렬 오류가 TAC 표 폭 계산 때문인지 확인했다.

## 재현 산출물

- SVG:
  `output/poc/task1285/stage1_debug/21_언어_기출_편집가능본_001.svg`
- render tree:
  `output/poc/task1285/stage1_tree/render_tree_001.json`

명령:

```bash
env RHWP_DEBUG_PARA_TAC=1 cargo run --bin rhwp -- export-svg \
  samples/21_언어_기출_편집가능본.hwp \
  -o output/poc/task1285/stage1_debug --debug-overlay -p 0

cargo run --bin rhwp -- export-render-tree \
  samples/21_언어_기출_편집가능본.hwp \
  -o output/poc/task1285/stage1_tree -p 0
```

## TAC line mapping 관찰

디버그 로그:

```text
TAC_LINE pi=0 line_idx=0 run_idx=0 run_char_pos=0 run_char_end=4
y=258.5 lh=40.2 ls=9.5 raw_lh=40.2 baseline=20.1
run_tacs=[(0, 188.98666666666668, 0), (4, 285.12, 1)]
```

해석:

- 부모 셀 문단 텍스트는 공백 4자다.
- 첫 번째 TAC 표(`성명`)는 문자 위치 0, 폭 약 188.99px다.
- 두 번째 TAC 표(`수험번호`)는 문자 위치 4, 폭 약 285.12px다.
- 두 TAC 표 모두 같은 line/run 안에 들어온다.

## dump/render tree 대조

`dump` 기준 내부 TAC 표 폭:

- `성명` 표: 4257 + 9917 = 14174 HU = 약 188.99px
- `수험번호` 표: 7244 + 2020 * 7 = 21384 HU = 약 285.12px

render tree 기준 부모 셀 내부 배치:

| 항목 | x | w | right |
|---|---:|---:|---:|
| `성명` TAC 표 | 581.8 | 189.0 | 770.8 |
| 공백 4자 TextRun | 770.8 | 25.0 | 795.8 |
| `수험번호` TAC 표 | 795.8 | 285.1 | 1080.9 |

따라서 실제 렌더 순서는 `성명 표 -> 공백 4자 -> 수험번호 표`로 끊기지 않고 이어진다.
이 샘플에서 TAC 표 자체의 column-sum 폭 추출은 맞다.

## 확인된 문제 지점

### 1. 일반 TAC line width 추정 경로의 중복 계상 위험

`src/renderer/layout/paragraph_layout.rs`의 `layout_composed_paragraph`는 줄 정렬용 점유 폭을
계산할 때 run 내부 TAC 위치를 순회하며 이미 `est_x += tac_w`를 수행한다.

그 뒤 다음 fallback을 다시 수행한다.

```rust
let total_tac_width_in_line: f64 = line_tac_offsets.iter().map(|(_, w, _)| w).sum();
if total_tac_width_in_line > 0.0 && total_text_width < total_tac_width_in_line {
    total_text_width += total_tac_width_in_line;
}
```

이 fallback은 "TAC 폭이 est_x에 미포함된 경우"를 보정하려는 의도지만,
현재 구조에서는 이미 포함된 TAC 폭까지 다시 더할 수 있다. 특히 한 줄 안에 TAC가 여러 개 있고
텍스트가 짧은 경우 같은 줄에 모두 들어가는 상황에서도 줄 점유 폭이 과대 계산될 수 있다.

단, 이번 샘플의 부모 셀 렌더 트리만 놓고 보면 두 TAC 표와 사이 공백은 실제로 연속 배치된다.
즉 `다음 TAC 표가 다음 줄로 넘어갈지` 판단 경로의 위험은 확인되지만, 현재 보이는 `수험번호`
텍스트 위치 오류의 직접 원인은 부모 sequence 순서가 깨지는 문제만으로 설명되지 않는다.

### 2. `수험번호` TAC 표 내부 오른쪽 정렬 오차

두 번째 TAC 표 첫 셀의 render tree:

| 항목 | x | w | right |
|---|---:|---:|---:|
| 첫 셀 bbox | 795.8 | 96.6 | 892.4 |
| 첫 셀 내부 TextLine | 797.7 | 92.8 | 890.5 |
| `수험번호` TextRun | 805.6 | 77.0 | 882.6 |

오른쪽 정렬이라면 `수험번호` TextRun의 기대 x는 대략 `890.5 - 77.0 = 813.5`다.
현재 x=805.6이므로 약 7.9px 왼쪽에 배치되어 있다.

이 오차는 부모 셀의 TAC sequence width만이 아니라, TAC 표 내부 셀 문단의 오른쪽 정렬/폭 계산
또는 cell underflow 보정 분기까지 함께 확인해야 한다.

## composer 쪽 확인

`src/renderer/composer.rs`의 TAC table width 수집은 `t.get_column_widths().iter().sum()`을 사용한다.
이번 샘플에서는 이 값이 dump/render tree와 일치한다.

다만 `recompose_for_cell_width`의 fallback 측정 함수 `estimate_composed_line_width`는 현재 text run 폭만
합산하며 TAC 폭을 포함하지 않는다. 이 함수는 `para.line_segs.is_empty()`일 때만 동작하므로
이번 HWP 샘플의 직접 경로는 아니다. 하지만 LineSeg가 없는 셀 문단에서 TAC가 포함될 경우에는
줄 분할 판단이 잘못될 수 있는 별도 위험이다.

## 결론

작업지시자의 가설은 부분적으로 맞다.

- TAC 표 폭 자체는 이번 샘플에서 올바르게 추출된다.
- 하지만 `layout_composed_paragraph`의 줄 점유 폭 추정에는 이미 반영한 TAC 폭을 fallback에서 다시
  더할 수 있는 구조가 있다.
- 같은 줄에 모두 배치되는 경우에도 이 fallback이 발화하면 width가 과대 계산될 수 있으므로 수정 대상이다.
- 현재 눈에 띄는 `수험번호` 위치 오류는 두 번째 TAC 표 내부 셀의 오른쪽 정렬 계산 문제까지 같이 고쳐야 한다.

## 다음 구현 권장안

1. 회귀 테스트를 먼저 추가한다.
   - 부모 셀에서 `성명` 표 right == 공백 TextRun x
   - 공백 TextRun right == `수험번호` 표 x
   - `수험번호` 첫 셀에서 오른쪽 정렬 TextRun x == inner_right - text_width
2. `layout_composed_paragraph`에서 `est_x`에 실제 포함된 TAC 폭을 추적한다.
   - 전체 `total_tac_width_in_line`을 무조건 더하지 않는다.
   - 누락된 TAC 폭만 보정한다.
3. TAC 표 내부 셀 문단의 오른쪽 정렬 계산을 점검한다.
   - cell context, `total_text_width`, `available_width`, underflow 보정 분기를 함께 검증한다.
4. #1219의 줄 경계 TAC 보정 회귀가 발생하지 않도록 기존 `issue_1219` 테스트를 같이 실행한다.

