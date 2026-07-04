# Task 1293 Stage 56: no-separator 잔여 qflow drift 분석

## 목적

Stage55에서 `2024-11-practice-no-separator-above20-between20-below20`의 renderer overflow는
0건이 되었고, `question_marker_flow_drift_pages`는 `[18, 20, 21, 22, 23]`에서 `[18, 22]`로
줄었다. 이번 단계에서는 남은 두 drift를 실제 한컴 PDF와 비교해 원인을 분리한다.

## 대상

- sample: `samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp`
- reference PDF: `pdf/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.pdf`
- latest sweep: `output/task1293_stage55_no_separator_final`

## 현재 관찰

- page count는 `23/23/23`으로 유지된다.
- `overflow_lines`는 0이다.
- page 18:
  - PDF는 문26 이어쓰기에서 시작하고 문27이 중간에서 시작한다.
  - rhwp는 문26 제목이 page 18에 남아 있어 한 문항 흐름이 늦다.
  - 직전 page 17~18 경계에서 문25 tail `pi=783`, `pi=784`와 문26 제목 `pi=785`가 핵심 후보이다.
- page 20:
  - Stage55에서 문24 제목 `pi=899`는 왼쪽 단 하단에 남게 되었지만, PDF처럼 문24 본문까지 남지는 않는다.
  - 이는 page 18 첫 drift가 누적되어 나타나는 결과일 가능성이 높다.
- page 22:
  - 문30 큰 그림 tail의 `LAYOUT_OVERFLOW`는 사라졌지만 PDF와 세부 문단 분배가 아직 다르다.

## 분석 계획

1. page 17~18의 `pi=778..786` 문25/문26 경계를 PDF compare와 render tree bbox로 다시 고정한다.
2. `pi=783`이 `cur=987.8`, `en_fit=16.4`에서 넘어가는 현상이 단순 bottom bleed 문제인지,
   앞선 `pi=778..782` 누적 높이 과다 문제인지 확인한다.
3. 문25 수식/빈문단의 `height_for_fit`, `total_height`, 저장 LINE_SEG vpos delta를 비교해 formatter가
   실제 renderer보다 과한 높이를 소비하는지 확인한다.
4. page 22 문30 tail은 page 18 보정 후 다시 sweep한 뒤 별도 보정이 필요한지 판단한다.

## 실험 결과

### 구분선 없음 separator block 0 처리

`endnote_separator_height_px`에서 보이는 구분선이 없으면 `0.0`을 반환하도록 임시 변경해
targeted sweep을 수행했다.

```bash
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage56_no_separator_no_sep_block \
  --rhwp-bin target/debug/rhwp
```

결과:

- SVG/PDF page count: `23/23`
- 기존 Stage55 qflow: `[18, 22]`
- 실험 qflow: `[10, 11, 18, 22]`
- `red_marker_drift`: `[10, 11, 13, 14, 16, 17, 18, 19, 20, 21, 22]`
- `line_band_drift`: `[9, 10, 15, 18, 19, 20, 21, 22]`

구분선 없음일 때 separator line 자체는 보이지 않지만, `구분선 위/아래` 값을 통째로 제거하면
후반부가 아니라 10~11쪽부터 문항 흐름 drift가 늘어난다. 따라서 현재 샘플의 남은 차이는
separator block 전체가 단순히 없어져야 하는 문제가 아니다. 이 임시 변경은 폐기하고 소스는
Stage55 상태로 되돌렸다.

다음 후보는 page 17~18의 문25 tail과 문26 제목 경계에서 formatter가 실제 renderer보다 문단 높이
또는 새 미주 번호 경계 예약을 과소/과대 판단하는지 확인하는 것이다.

## 검증 계획

- focused:
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- targeted visual:
  - `cargo build --bin rhwp`
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-no-separator-above20-between20-below20 --out output/task1293_stage56_no_separator --rhwp-bin target/debug/rhwp`
- 직접 확인:
  - `compare_017.png`
  - `compare_018.png`
  - `compare_020.png`
  - `compare_022.png`
