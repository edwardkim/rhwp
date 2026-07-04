# Task 1293 Stage 57: no-separator page 17~18 문항 흐름 drift 원인 추적

## 목적

Stage56에서 `2024-11-practice-no-separator-above20-between20-below20`의 separator block 제거
실험은 qflow를 `[18, 22]`에서 `[10, 11, 18, 22]`로 악화시켜 폐기했다. 이번 단계에서는
남은 첫 drift인 page 17~18 문25 tail/문26 제목 경계를 분석해, 미주 모양 값 자체가 아니라
문단 fit/advance 판단 중 어느 부분이 한컴 PDF와 달라지는지 분리한다.

## 대상

- sample: `samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp`
- reference PDF: `pdf/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.pdf`
- latest accepted sweep: `output/task1293_stage55_no_separator_final`

## 분석 계획

1. PDF page 17~18의 문항 marker 흐름을 텍스트와 이미지에서 확인한다.
2. render tree page 17~18의 `pi=778..786` bbox와 `dump-pages` 배치를 비교한다.
3. formatter가 page 17 오른쪽 단에서 문25 tail과 문26 제목을 넘기는 원인이
   - 저장 LINE_SEG vpos 기반 tail 높이 과대평가인지,
   - 새 미주 번호 경계의 `between_notes` 예약 위치 차이인지,
   - no-separator 전용 column bottom bleed 허용 범위 부족인지
   구분한다.

## 분석 결과

### 최초 drift 위치

처음에는 page 17~18의 문25 tail/문26 제목 경계를 의심했지만, compare PNG를 다시 열어보니
실제 최초 구조 차이는 page 15~16 경계였다.

- Stage55 `compare_015.png`
  - PDF는 문27 tail의 마지막 수식/문장까지 page 15 오른쪽 단 하단에 남긴다.
  - rhwp는 같은 tail을 page 16 왼쪽 단 첫머리로 넘긴다.
- 그 결과 Stage55 `compare_016.png`에서 PDF는 문28로 시작하지만, rhwp는 문27 tail 뒤에 문28이
시작한다.
- page 17~18의 문23~문26 drift는 이 page 15~16 tail 분배 차이가 누적된 결과다.

### trace 판정

임시 `RHWP_TRACE_ENDNOTE_FIT` trace로 page 15 오른쪽 단의 문27 tail을 확인했다.

핵심 지점:

```text
TRACE_PREFIT num=27 ep=6 col=1/2 cur=901.5 avail=1001.6
h4f=27.6 en_fit=27.6 adv_fit=true no_sep_saved=true
```

실제 formatter 높이로는 `901.5 + 27.6 < 1001.6`이므로 현재 단에 충분히 들어간다. 그러나
`no_separator_saved_vpos_tail_outside`가 저장 vpos 기준으로 stale tail outside로 판단해
강제 advance를 만들었다. 구분선이 없고 `구분선 위/아래`가 큰 샘플에서는 저장 vpos가 separator
block 기준으로 밀려 있어, 실제 formatter 높이가 현재 단에 들어가는 경우까지 이 guard가 적용되면
한컴보다 문항 흐름이 늦어진다.

## 수정

`no_separator_saved_vpos_tail_outside`가 실제 formatter 높이로 현재 단에 들어가는 tail에는 적용되지
않도록 조건을 좁혔다.

```rust
st.current_height + en_fit > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
```

즉, 저장 vpos가 바깥으로 보이더라도 실제 `en_fit`이 bottom tolerance 안에 들어가면 current column에
남긴다. 이는 공식 미주 모양 값 자체를 제거하는 것이 아니라, 구분선 없음 + 큰 separator block에서
저장 vpos가 stale할 때만 formatter 기준 fit 판단을 우선하는 보정이다.

## 실행 결과

targeted sweep:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage57_no_separator_tail_guard \
  --rhwp-bin target/debug/rhwp
```

결과:

- SVG/PDF/render tree page count: `23/23/23`
- renderer `LAYOUT_OVERFLOW`: 0
- frame overflow: 0
- title/order/equation overlap: 0
- Stage55 qflow: `[18, 22]`
- Stage57 qflow: `[18, 21, 22]`

qflow 후보 수만 보면 page 21이 추가됐지만, compare PNG 직접 확인 결과 page 21은 Stage55보다 PDF에
가깝다. Stage55 page 21은 큰 도형이 페이지 맨 위에 남아 PDF와 명백히 달랐고, Stage57은 문29와
도형 흐름이 PDF 위치에 가까워졌다. 자동 qflow는 marker 개수/짝짓기 조건이 민감하게 반응한 후보로
분류한다.

직접 확인:

- `output/task1293_stage57_no_separator_tail_guard/.../compare/compare_015.png`
  - 문27 tail이 PDF처럼 page 15 오른쪽 하단에 남는다.
- `output/task1293_stage57_no_separator_tail_guard/.../compare/compare_016.png`
  - page 16이 PDF처럼 문28부터 시작한다.
- `output/task1293_stage57_no_separator_tail_guard/.../compare/compare_018.png`
  - 문27/문28/문29 흐름이 Stage55보다 PDF에 가까워졌다.
- `output/task1293_stage57_no_separator_tail_guard/.../compare/compare_021.png`
  - Stage55의 큰 도형 top drift가 줄어들었다.

## 검증 결과

```bash
cargo build --bin rhwp
cargo fmt --all -- --check
cargo test --test issue_1139_inline_picture_duplicate -- --nocapture
```

- `cargo build --bin rhwp`: 통과
- `cargo fmt --all -- --check`: 통과
- `issue_1139_inline_picture_duplicate`: 52개 통과

## 남은 작업

이번 보정은 no-separator large block의 첫 누적 drift를 줄였지만, qflow 후보는 아직 `[18, 21, 22]`가
남아 있다. 다음 stage에서는 page 18과 page 22의 남은 후보가 실제 구조 차이인지, marker count 기반
오탐인지 구분하고 필요하면 sweep의 qflow 판단 기준도 함께 정교화한다.

## 검증 계획

- 분석 산출:
  - `output/task1293_stage55_no_separator_final/.../render_tree/render_tree_017.json`
  - `output/task1293_stage55_no_separator_final/.../render_tree/render_tree_018.json`
  - `dump-pages` page 16~17
  - `pdftotext -layout` page 17~18
- 수정 후:
  - `cargo build --bin rhwp`
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-no-separator-above20-between20-below20`
