# Task 1293 Stage 64: 0mm 미주 수식-only rewind base 보정

## 목적

Stage63에서 display 11쪽의 `pi=537` graph 자체가 PDF/Hancom보다 약 30px 아래로 내려간다는
점을 확인했다. 원인은 graph bbox/투명 여백이 아니라, `pi=516`에서 새로 만들어지는 lazy
vpos base가 `147097`로 낮아지는 흐름이다.

이번 단계는 `pi=514 → pi=516` 사이의 저장 vpos 되감김과 수식-only 문단을 공통 로직으로
분리해, 0/0/0 미주에서 graph/tail이 한컴보다 아래로 밀리는 원인을 수정한다.

## 기준 대상

- 샘플: `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`
- 기준 PDF: `pdf/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.pdf`
- target: `2024-11-practice-above0-between0-below0`
- page: display 11쪽, 내부 page index 10

## Stage63 확정값

- rhwp graph ink bbox: `(40, 899) - (248, 1049)`
- PDF graph ink bbox: `(40, 868) - (248, 1019)`
- `pi=537` Image render tree bbox: `x=34.9 y=898.0 w=220.6 h=155.9`
- `pi=537` Shape cursor: `y_in=896.7 y_out=1072.0`
- `pi=516` lazy base: `147097`
- PDF graph top에 해당하는 역산 base: 약 `149251`

## 추가 확인

`dump-pages -p 10`:

```text
pi=513 vpos=150253  "라 하면"
pi=514 vpos=148901  "에서 코사인법칙에 의해"
pi=515 vpos=150478..152157 "(빈)"
pi=516 vpos=153644 "에서 코사인법칙에 의해"
```

`RHWP_DEBUG_TAC_CURSOR=1`:

```text
FullPara pi=513 y_in=90.7 y_out=108.7 dy=18.0
FullPara pi=514 y_in=108.7 y_out=129.8 dy=21.0
FullPara pi=515 y_in=129.8 y_out=172.0 dy=42.2
FullPara pi=516 y_in=178.0 y_out=199.0 dy=21.0
```

render tree를 보면 `pi=515`는 텍스트가 비어 있지만 실제로는 수식 노드가 두 줄 있다. 따라서
이 구간은 단순 빈 spacer가 아니라, 저장 vpos가 되감긴 직후 이어지는 수식-only 문단 흐름이다.

## 가설

현재 `layout.rs`는 current paragraph의 first vpos가 직전 paragraph first vpos보다 작으면
`current_vpos_rewinds_from_prev`로 보고 `prev_layout_para`, `vpos_page_base`, `vpos_lazy_base`를
모두 끊는다.

display 11쪽에서는 `pi=514`가 이 조건에 걸린다.

- `pi=513 first = 150253`
- `pi=514 first = 148901`
- `pi=514 end = 150253`

즉 `pi=514`는 직전 first로 정확히 끝나는 얕은 rewind다. 이를 새 흐름 시작으로 끊으면, 후속
`pi=515` 수식-only 문단을 기준으로 lazy base를 다시 역산하고 그 결과 `pi=516` 이후 전체가
약 30px 내려간다.

## 수정 방향

문항 번호나 특정 paragraph 번호로 처리하지 않는다. 다음 공통 조건을 만족하는 경우만
vpos base reset을 완화하거나 lazy base seed를 보존한다.

- 미주 흐름(`col_content.endnote_flow`)
- 현재 미주 설정이 0/0/0 profile
- current paragraph가 직전 first보다 되감기지만, current last segment end가 직전 first에
  가깝게 닿는 얕은 rewind
- 후속에 수식-only/비텍스트 수식 문단이 이어지는 compact 풀이 흐름

## 검증 계획

- `cargo build --bin rhwp`
- focused:
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - `cargo test --test issue_1050_footnote_serialize -- --nocapture`
  - `cargo test --lib compact_endnote`
- visual sweep:
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0 --out output/task1293_stage64_zero_rewind_base --rhwp-bin target/debug/rhwp`
  - 회귀 확인 후 전체 task1293 sweep

## 실험 결과

### 폐기: 얕은 rewind seed 보존 + 제목 tail 완화

두 가지를 함께 실험했다.

1. `layout.rs`에서 0/0/0 미주이고 current paragraph가 직전 first vpos까지 얕게 되감겨 끝나는
   경우, vpos base reset 대신 `curr_first`를 lazy base seed로 유지했다.
2. `typeset.rs`에서 0/0/0 미주 제목 tail이 직전 단에 non-TAC Shape를 포함하면
   `line_advance(0)` fit 조건을 완화했다.

결과:

```text
dump-pages p10:
단 0 items=28, used=1003.8px
pi=537 Shape
pi=538 "따라서 "
pi=539 "문13）   22_11_실전 13) ③"
```

`pi=539`는 첫 단에 남았지만 renderer 위치는 여전히 한컴/PDF와 맞지 않았다.

```text
Shape pi=537 y_in=890.7 y_out=1065.9
FullPara pi=538 y_in=1065.9 y_out=1107.8
FullPara pi=539 y_in=1107.8 y_out=1125.9
```

baseline의 graph top `896.7px`에서 `890.7px`로 6px만 올라갔다. PDF/Hancom graph top
`868px`에는 여전히 약 23px 부족하다.

또한 sweep 결과가 나빠졌다.

```text
analysis: 2024-11-practice-above0-between0-below0
flagged=19/21
qflow=[11, 12, 13, 17, 19, 20]
order=[10]
```

`compare_010.png`, `compare_011.png`를 확인하면 `pi539`를 첫 단에 남기는 것만으로는 전체
page flow가 한컴/PDF와 맞지 않고, 앞쪽/다음쪽의 line band drift가 커진다.

### 폐기 판단

이 실험은 커밋하지 않는다.

- base seed 보존만으로는 `HeightCursor`의 backtrack 안전 조건 때문에 실제 y가 충분히 위로
  이동하지 않는다.
- 제목 tail fit 완화는 pagination만 바꾸어 다음 page flow를 흔든다.
- 따라서 Stage64의 “수식-only rewind base만 조정” 가설은 불충분하다.

## 다음 판단

Stage64 이후에는 특정 page tail을 맞추는 fit 완화가 아니라 미주 모양의 공식 의미를 먼저
renderer 모델에 반영해야 한다.

- 구분선 위/미주 사이/구분선 아래 값이 각각 어느 위치의 간격인지 확정한다.
- 구분선 없는 경우에도 “구분선 위” 값이 미주 블록 상단 gap으로 적용되는지 확인한다.
- 미주 separator block, 첫 미주 content top, 각 미주 사이 gap, 문서 끝/구역 끝 위치를 하나의
  common model로 분리한다.
- 그 뒤 `HeightCursor`의 vpos reset/lazy base 보정은 이 common model 안에서만 다시 실험한다.
