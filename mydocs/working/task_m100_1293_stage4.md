# Task 1293 Stage 4: 공식 미주 모양 기반 잔여 flags 분석

## 목적

Stage3에서 미주 모양 raw 슬롯과 공식 UI 의미값을 sweep 산출물에 노출했다. 이번 단계에서는
`구분선 위`, `구분선 아래`, `미주 사이` 값을 실제 layout 흐름에 적용하는 위치를 검토하고,
잔여 frame/red/line flags가 어떤 공식 의미를 잘못 소비해서 생기는지 분리한다.

## 공식 기준

한컴 도움말의 미주 모양 설명은 다음 의미를 기준으로 한다.

- `구분선 위`: 미주와 본문이 만나는 경우, 본문과 미주 구분선 사이의 간격
- `구분선 아래`: 미주 구분선과 미주 내용 사이의 간격
- `미주 사이`: 앞 번호 미주 내용과 다음 번호 미주 내용 사이의 간격

따라서 `구분선 위`와 `구분선 아래`는 미주 블록의 시작부에 한 번씩 쓰이고, `미주 사이`는
번호가 달라지는 미주 내용 사이에서만 쓰여야 한다. 문항 본문 줄 사이 gap이나 페이지/단 이월
backtrack 보정값으로 임의 소비하면 overwrap 또는 drift가 생긴다.

## 입력 산출물

- `output/task1293_stage3_below20/summary.json`
- `output/task1293_stage3_between20/summary.json`
- `output/task1293_stage3/summary.json`

## 분석 계획

1. 세 target의 `metrics.json`에서 공통으로 잡히는 flags를 정렬한다.
2. `note_shape` 값과 frame/marker/line drift의 위치를 비교한다.
3. layout 코드에서 `separator_above_margin_hu`, `separator_below_margin_hu`,
   `between_notes_margin_hu`를 소비하는 위치를 전수 확인한다.
4. 첫 수정은 공식 의미와 명확히 어긋난 소비 위치에 한정한다.

## 참고한 트러블슈팅

- `mydocs/troubleshootings/typeset_layout_drift_analysis.md`
  - typeset fit 판단과 layout 실제 y 진행이 달라지면 하단 라인 piling/overlap이 발생한다.
  - vpos correction 한쪽만 고치면 collapse 회귀가 생길 수 있으므로 작은 단위로 검증해야 한다.
- `mydocs/troubleshootings/typeset_fit_accumulation_drift.md`
  - fit 판정과 누적 갱신은 다른 의미다. 마지막 자리 trailing spacing은 fit에서 제외될 수 있지만,
    다음 항목 시작 위치 계산에는 full advance가 필요하다.
- `mydocs/troubleshootings/2010_01_06_footnote_line_spacing.md`
  - 각주/미주 내부 paragraph transition에서 line_spacing 누락이 줄간격 불일치로 이어진다.
- `mydocs/troubleshootings/hwpx_lineseg_reflow_trap.md`
  - LINE_SEG와 TAC 높이를 재계산할 때 HWP/HWPX 표현 차이를 그대로 믿으면 안 된다.

## Stage4 수정

`src/renderer/typeset.rs`에서 `EndnoteSeparator`를 current item에 넣은 뒤 separator 높이를
항상 `current_height`에 누적하도록 수정했다.

기존 코드는 `구분선 아래`가 작으면 `EndnoteSeparator`가 렌더 단계에서는 y를 증가시키지만
typeset 누적 높이에는 반영하지 않았다. 공식 미주 모양에서는 `구분선 위`, 선, `구분선 아래`가
미주 블록 시작부의 실제 높이이므로, fit 이후 다음 미주 문단 시작 위치 계산에도 포함되어야 한다.

## 검증 결과

- `cargo fmt --all -- --check` — 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1139_endnote_spacing_reference_files_match_hancom_page_counts -- --nocapture` — 1 passed
- `cargo build --bin rhwp` — 통과
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-below20-above20 --out output/task1293_stage4 --rhwp-bin target/debug/rhwp`
  - 23/23/23
  - flags: `frame=[9,17,18,19]`, `red=[9,10,13,17,18,19,22]`, `line=[4,7,10,11,15,17,18,21,22]`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20 --out output/task1293_stage4_between20 --rhwp-bin target/debug/rhwp`
  - 24/24/24
  - flags: `frame=[12]`, `red=[11,13,14,17,18,20,21,22,23]`, `line=[4,7,8,10,12,14,15,16,20,21,23]`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` — 51 passed

## 판단

separator 높이 누적 정합은 공식 모델상 맞는 수정이지만, 조합 샘플과 `미주사이20`의 visual flags를
줄이지는 못했다. 남은 문제는 `EndnoteSeparator` 자체가 아니라 미주 내부 LINE_SEG/vpos 흐름과
단 이월 판단의 불일치로 보인다.

다음 단계에서는 `dump-pages`의 `used`와 `hwp_used` 차이가 큰 페이지를 중심으로, 미주 문단의
fit 판정과 누적 갱신이 공식 `미주 사이`와 LINE_SEG를 중복/누락 소비하는지 확인한다.
