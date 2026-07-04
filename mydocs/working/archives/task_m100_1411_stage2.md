# Task 1411 Stage 2 — `2022-10` p14 수식/bbox 후보 분류

## 목적

Stage 1 baseline에서 남은 `2022-10` p14의 `equation_text_overlap`,
`line_band_drift`, `large_ink_region_drift` 후보가 공식 미주 모양 계산 잔여인지,
검출기 후보인지, 별도 layout 결함인지 분리한다.

## 입력

- 기준 커밋: `1660aa5a` (`task 1411: baseline 잔여 후보 재현`)
- baseline 산출물: `output/task1411_stage1_baseline/2022-10`
- 추가 디버그 산출물:
  - `output/task1411_stage2_vpos_debug/2022-10/vpos_debug.log`
  - `output/task1411_stage2_vpos_debug/2022-10/vpos_ssot_debug.log`
  - `output/task1411_stage2_vpos_debug/2022-10/render_tree_ssot/render_tree_014.json`

## 확인 명령

```bash
RHWP_VPOS_DEBUG=1 target/debug/rhwp export-render-tree \
  samples/3-10월_교육_통합_2022.hwp \
  -o output/task1411_stage2_vpos_debug/2022-10/render_tree \
  2> output/task1411_stage2_vpos_debug/2022-10/vpos_debug.log

RHWP_VPOS_DEBUG=1 RHWP_EN_SSOT_DEBUG=1 target/debug/rhwp export-render-tree \
  samples/3-10월_교육_통합_2022.hwp \
  -o output/task1411_stage2_vpos_debug/2022-10/render_tree_ssot \
  2> output/task1411_stage2_vpos_debug/2022-10/vpos_ssot_debug.log
```

## 관찰

### sweep 후보

`flagged_pages.json`의 p14 후보는 다음과 같다.

- `equation_text_overlap_candidates`: 문24 마지막 수식 line의 식 bbox와 문25 첫 본문 쉼표 bbox가
  `9px x 9px` 교차한다.
  - 문24 tail: `pi=775`, `equation_line_text="[EQ]에서 [EQ]"`
  - 문25 첫 본문: `pi=777`, `text=", "`, `text_line_text="[EQ], [EQ]이므로"`
- `question_marker_drift_candidates`와 tail overflow 후보는 없다.
- `content_bottom_delta_px=9.0`으로 쪽 하단은 크게 어긋나지 않는다.

### render tree 위치

`render_tree_014.json` 기준 주요 위치는 다음과 같다.

| pi | 내용 | y | h | 비고 |
| --- | --- | ---: | ---: | --- |
| 770 | 문23 제목 | 691.0 | 12.0 | PDF 문23 y≈676.0 |
| 772 | 문23 마지막 큰 수식 | 729.4 | 58.8 | visual bbox bottom≈788.2 |
| 773 | 문24 제목 | 841.1 | 12.0 | PDF 문24 y≈799.8 |
| 774 | 문24 큰 수식 | 859.1 | 58.8 |  |
| 775 | 문24 tail `에서` | 939.3 | 30.6 | 다음 제목과 수직 교차 |
| 776 | 문25 제목 | 940.3 | 12.0 | PDF 문25 y≈939.7 |
| 777 | 문25 첫 본문 | 958.4 | 27.6 | 쉼표 bbox가 문24 tail 수식과 교차 |

문25 제목은 PDF와 거의 같은 위치에 있다. 겹침은 문25가 과도하게 올라온 문제가 아니라,
문24 제목과 본문 tail이 이전 문항 경계에서 내려와 문25 영역을 침범한 문제다.

### vpos/layout 로그

`vpos_ssot_debug.log`의 p14 주변 로그 요약:

| pi | `vpos_adjust` 입력 | `vpos_adjust` 결과 | 실제 render 시작 | 판단 |
| --- | ---: | ---: | ---: | --- |
| 773 | 814.65 | 822.65 | 841.1 | layout 단계가 문23 tail 뒤 note gap을 추가 보존 |
| 776 | 996.35 | 940.35 | 940.3 | HeightCursor가 문25 제목을 PDF 위치에 가깝게 당김 |

`pi=773`에서 `HeightCursor` 결과는 문23 tail 뒤에 약 34px visual gap을 이미 확보한다.
하지만 layout의 `should_preserve_endnote_title_gap` 경로가 직전 `line_spacing=1984HU`
약 26.45px을 `y_before_vpos` 기준으로 다시 보존하면서 문24 제목을 `822.65 → 841.1`로
내린다.

직전 문단 `pi=772`는 textless equation tail이고, 저장 vpos의 문23 tail bottom→문24
head gap은 약 `452HU`다. 이 경우 실제 기준은 paragraph logical bottom이 아니라
visible equation bbox bottom이어야 한다. 현재 보존 로직은 visible bottom 기준으로 이미
충분한 gap이 있는 케이스에도 full note gap을 한 번 더 얹는다.

## 분류 결론

`2022-10` p14 후보는 공식 미주 모양 모델 계산 잔여도, 단순 detector 오탐도 아니다.

실제 layout 결함으로 분류한다. 원인은 compact endnote flow에서 textless tall equation tail
뒤의 새 문항 제목을 배치할 때, `HeightCursor` 보정이 visible content 기준 gap을 이미 만든
상태인데도 layout 단계가 직전 `line_spacing`을 note gap으로 한 번 더 보존하는 것이다.

## Stage 3 작업 방향

- `src/renderer/layout.rs`의 미주 제목 gap 보존 직전에서, 직전 문단이 보이는 텍스트 없는
  equation tail이고 현재 제목의 위치가 `prev_item_content_bottom_y + prev_endnote_title_gap_px`
  이상이면 추가 gap 보존을 생략한다.
- 이 보정은 `prev_endnote_title_gap_px` 자체를 없애지 않고, visible bottom 기준으로 이미
  충분한 gap을 확보한 케이스만 제외한다.
- 최소 검증:
  - `cargo fmt --check`
  - `cargo build --bin rhwp`
  - targeted sweep 3종 재실행
  - `git diff --check`
