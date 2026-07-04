# task 1293 stage89 - FootnoteShape attr와 미주 경계 모델 재검토

## 목적

stage88에서 확인한 미주 간격 보정은 일부 샘플을 개선했지만, 한컴 미주 모양을 공통 모델로 완성하기에는
아직 부족하다. stage89에서는 개별 문항 보정이 아니라 `FootnoteShape`의 공식 속성 비트와 저장
`lineSeg`/`vpos`가 미주 렌더링에서 어떻게 함께 쓰이는지 다시 검토한다.

## 시작 기준

- 브랜치: `local/task_m100_1293`
- 직전 커밋: `ec0d2a1b task 1293: 미주 공통 경계 보정 stage88`
- 기준 문서:
  - `mydocs/tech/한글문서파일형식_5.0_revision1.3.md`
  - `mydocs/tech/hwp_spec_errata.md`
  - `mydocs/troubleshootings/2010_01_06_footnote_line_spacing.md`

## stage88에서 남은 핵심 문제

1. `3-09월_교육_통합_2024-미주사이20.hwp` 계열은 raw spacing 값은 맞게 파싱되지만, 20mm large-gap
   조합에서 저장 `lineSeg`/`vpos`와 실제 렌더 경계 소비 위치가 어긋난다.
2. `3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp` 계열은 compact gap에서는 순차 fit과
   저장 vpos 우선순위를 잘못 잡으면 다른 페이지로 회귀한다.
3. `구분선 없음`, `구분선 위/아래`, `미주 사이`, `문서 끝/구역 끝`, `텍스트에 이어 출력`을 같은
   보정 수치로 처리하면 다른 샘플이 깨진다.

## 표 134 재검토 항목

`각주/미주 모양 속성`의 공식 비트 의미를 먼저 구현 모델과 대조한다.

| 비트 | 공식 의미 | 현재 검토 필요 |
|---|---|---|
| bit 0~7 | 번호 모양 | 기존 number format 매핑 유지 여부 확인 |
| bit 8~9 | 위치 | HWP5/HWPX parser와 renderer placement 정책 확인 |
| bit 10~11 | numbering | 현재 `(attr >> 8)`로 읽는 경로 수정 필요 |
| bit 12 | 번호 코드 위첨자 | `FootnoteShape` 모델 필드와 렌더 적용 여부 확인 |
| bit 13 | 텍스트에 이어 바로 출력 | `beneathText`/inline 출력 의미와 미주 배치 정책 확인 |

## stage89 처리 방향

1. `FootnoteShape` attr decode/encode를 공식 비트 기준으로 정리한다.
2. HWP5와 HWPX 경로가 같은 모델 필드를 채우도록 맞춘다.
3. 현재 20mm 샘플의 직접 원인은 attr가 아니라 `lineSeg`/`vpos` 경계이므로, attr 수정과 경계 보정은
   검증 항목을 분리한다.
4. sweep의 flagged 페이지를 문항별로 고치지 않고, 아래 공통 축으로 분류한다.
   - visible separator 여부
   - separator above/below 소비 위치
   - between notes 소비 위치
   - 새 미주 제목과 첫 본문을 같은 단/페이지에 묶어야 하는지
   - 저장 `lineSeg`가 이미 gap을 포함하는지

## 검증 계획

사용자 승인 전 full CI는 수행하지 않는다.

1. `cargo build`
2. `cargo test --test issue_1139_inline_picture_duplicate issue_1293_ -- --nocapture`
3. attr decode 단위 테스트 추가 후 해당 테스트만 먼저 수행
4. targeted sweep:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_targeted
```

## 승인 대기

stage89에서는 먼저 attr 모델 정리와 미주 경계 분류를 진행한다. 소스 수정은 이 문서 기준으로 사용자
승인을 받은 뒤 시작한다.

## 수정 1 - 표 134 attr decode/encode 정리

`FootnoteShape`가 표 134의 `attr` 비트를 직접 해석/인코딩하도록 공통 helper를 추가했다.

- `bit 0~7`: 번호 모양
  - `0~16` 범용 번호 모양
  - `0x80`: 4가지 문자 반복
  - `0x81`: 사용자 지정 문자 반복
- `bit 8~9`: 위치
- `bit 10~11`: 번호 매기기
- `bit 12`: 주석 내용 번호 코드 위첨자 여부
- `bit 13`: 텍스트에 이어 바로 출력 여부

수정 파일:

- `src/model/footnote.rs`
  - `apply_attr_fields_from_raw`
  - `encode_attr`
  - 번호 모양 코드/이름 변환 helper
  - `number_code_superscript`, `print_inline_after_text` 필드
- `src/parser/body_text.rs`
  - HWP5 `HWPTAG_FOOTNOTE_SHAPE` raw attr를 표 134 기준으로 모델 필드에 반영
- `src/parser/hwpx/section.rs`
  - HWPX `autoNumFormat type/supscript`, `numbering`, `placement place/beneathText`를 모델 필드에 반영
  - HWPX 경로에서도 최종 `shape.attr = shape.encode_attr()`로 HWP5 attr와 같은 의미 유지
- `src/document_core/commands/object_ops.rs`
  - API 적용 시 `shape.encode_attr()` 사용
  - `numberCodeSuperscript`, `printInlineAfterText` 조회/적용 추가
- `src/main.rs`
  - `dump-pages` note shape JSON에 bit12/bit13 모델 필드 노출

## 수정 1 검증

명령:

```bash
cargo fmt
cargo test --lib footnote_shape_attr -- --nocapture
cargo test --lib test_parse_endnote_ -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1139_stage31_endnote_shape_api_updates_section_shape -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1139_exam_2022_endnote_shape_matches_hancom_reference -- --nocapture
cargo test --test issue_1050_footnote_serialize issue_1050_hwpx_footnote_shape_contract -- --nocapture
```

결과:

- `FootnoteShape` 모델 attr bit 테스트 통과
- HWPX endnote shape attr bit 테스트 4개 통과
- 미주 모양 API 적용 테스트 통과
- `3-09월_교육_통합_2022.hwp` HWP5 endnote shape 기준 테스트 통과
- HWPX footnote shape contract 테스트 통과

판단:

- 표 134의 attr bit 해석 오류는 수정됐다.
- 이 수정은 `3-09월_교육_통합_2024-미주사이20.hwp`의 20mm lineSeg/vpos 경계 문제를 직접 해결하는
  변경은 아니다. 다음 확인은 이 attr 정리가 기존 sweep 결과에 회귀를 만들지 않는지와, 남은 p19/p21
  후보가 어느 공통 경계 축에 속하는지 분류하는 것이다.

## 수정 1 targeted sweep

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_targeted
```

결과:

| 대상 | SVG/render/PDF | flagged | 판단 |
|---|---:|---:|---|
| `2024-09-between20` | `24/24/24` | `3/24` | stage88의 `4/24`에서 감소. 남은 후보는 p11/p18/p19. |
| `2024-11-practice-shape987` | `21/21/21` | `1/21` | stage88과 동일. p12 작은 tail 후보만 남음. |
| `2024-11-practice-above0-between0-below0` | `21/21/21` | `0/21` | clean 유지. |
| `2024-11-practice-no-separator-above20-between20-below20` | `23/23/23` | `0/23` | clean 유지. |

`2024-09-between20`의 note shape 덤프:

- `attr=0`
- `numberCodeSuperscript=false`
- `printInlineAfterText=false`
- `separatorAbove=0mm`
- `separatorBelow=2.032mm`
- `betweenNotes=19.999mm`

따라서 남은 후보는 표 134 attr 비트 문제로 발생한 것이 아니라, `visible separator + 20mm betweenNotes`
조합에서 저장 `lineSeg`/`vpos`와 실제 렌더 경계 소비가 어긋나는 문제다.

## 남은 후보 분류

| 대상 | 페이지 | 후보 | 공통 분류 |
|---|---:|---|---|
| `2024-09-between20` | 11 | `pi=573` 텍스트 `따라서` 뒤에 `pi=574` visual tail이 더 위 y로 렌더되어 `line_order_overlap` 발생 | large-gap 문서의 visual/tac tail 저장 vpos가 텍스트 흐름보다 앞서는 패턴 |
| `2024-09-between20` | 18 | `문29`의 `pi=922` 수식 tail이 frame 아래 24.5px 후보 | large-gap 문서의 equation-only tail이 하단 frame을 침범하는 패턴 |
| `2024-09-between20` | 19 | `문27` tail `pi=961`이 frame 아래 16.3px 후보이고, 다음 `문28`이 PDF보다 53.9px 위 | 이전 note 수식 tail/빈 수식 tail 높이와 다음 note title 사이 간격 소비가 어긋나는 패턴 |
| `2024-11-practice-shape987` | 12 | `문19` 표 뒤 설명 tail `pi=592`가 16.9px 후보 | compact-gap 문서의 표 뒤 text tail 작은 bleed 후보. 전체 advance는 p13 회귀를 만들므로 기각된 상태 |

다음 수정 후보는 특정 문항 번호가 아니라 `visual/tac/equation tail`이 저장 vpos상 현재 순서보다
위로 되감기거나 frame 하단을 넘는 경우를 공통으로 분류하는 것이다. 다만 p12 compact 후보는 이미
전체 advance가 회귀를 만든 전례가 있어, 20mm large-gap 전용 경계와 compact-gap 경계를 다시 섞으면 안 된다.

## 기각 실험 - large-gap continuation 단 시작 offset

가설:

- p19에서 `문28`이 PDF보다 약 53.9px 위에 있고, 오른쪽 단 continuation도 PDF보다 약 49px 위에서
  시작한다.
- 이 값이 `미주 사이 20mm - 기본 7mm` 초과분과 유사하므로, `large_between_equation_tail_starts_next_column`
  분기로 다음 단에 넘어갈 때 continuation 시작 높이에 `pagination_between_notes_margin()`을 더해 보았다.

결과:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --out output/task1293_stage89_continuation_gap_probe
```

- `flagged=7/24`
- 기존 `3/24`보다 악화
- 새로 p17/p21/p22의 red marker drift와 line/column drift가 발생

판단:

- continuation 단 시작에 초과 gap을 일괄 적용하면 `visible separator + large betweenNotes` 전체 흐름이
  한컴보다 늦어진다.
- p19는 "새 단 시작 offset"을 전역으로 더하는 문제가 아니라, 특정 endnote tail과 다음 note title/본문
  경계에서 저장 vpos와 실제 render gap을 선택하는 문제로 좁혀야 한다.
- 해당 실험 코드는 즉시 되돌렸다.

## 기각 실험 - tall equation tail 뒤 rewind title 경계 gap

가설:

- p19의 `문28`은 직전 `문27` equation-only tail 뒤에서 `endnote_has_vpos_rewind=true`인 새 미주 제목으로
  시작한다.
- 직전 tail의 마지막 line height가 3000HU 이상이고 현재 단의 45~75% 높이에서 새 제목이 시작하는 경우에만
  `pagination_between_notes_margin()`을 `st.current_height`와 render `line_spacing`에 함께 소비하면 p19의
  53px 상단 drift를 공통 조건으로 줄일 수 있을 것으로 보았다.

결과:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --out output/task1293_stage89_boundary_gap_probe
```

- `flagged=6/24`
- 기존 `3/24`에서 악화
- p21/p22에 새 `red_marker_drift`, `question_marker_drift`, `column_line_band_drift`가 발생

판단:

- 조건이 p19만 포착하지 못하고 같은 large-gap 문서 후반의 다른 미주 경계까지 함께 밀었다.
- p19의 `문28` drift는 "tall equation tail 뒤 새 note title"만으로는 공통 조건이 부족하다.
- 현재 보정 축은 `lineSeg/vpos`의 저장 위치 자체와 실제 한컴의 page/column advance 결정 지점을 더 직접
  비교해야 한다.
- 해당 실험 코드는 즉시 되돌렸다.

## 기각 실험 - A2 render-column bottom 전체 적용

## stage89 추가 분석 - p19 수식 tail advance 실험

`2024-09-between20` p19의 직접 후보는 다음 구조다.

- `문27` 좌측 단 하단에서 `pi=960` 수식-only tail이 y=1064.9, h=29.4로 배치된다.
- 바로 다음 `pi=961` 텍스트 `그러므로`가 y=1100.3으로 frame bottom 1096을 16.3px 넘는다.
- PDF/Hancom 기준으로는 같은 수식 tail이 우측 단 상단으로 넘어가고, 그 뒤에 `그러므로`, 다음 수식,
  `한편...`이 이어진다.
- 이 경우 `문28` 제목은 PDF y≈803.5이고, 기존 rhwp는 y=749.6으로 약 53.9px 높다.

실험 1:

- 짧은 텍스트 tail(`그러므로`)이 render-y 기준으로 frame을 넘으면 advance하도록
  `large_between_short_text_before_equation_tail_bleeds_previous_column` 허용 조건을 보정했다.
- 결과: `2024-09-between20 flagged=4/24`로 악화.
- 원인: p19는 해당 조건의 필수 축인 `later_endnote_vpos_rewinds_after_current`가 아니어서 잡히지 않았고,
  p10의 별도 되감기 패턴만 잘못 advance되었다.

실험 2:

- 수식-only tail 자체(`pi=960`)와 다음 짧은 텍스트 한 줄을 하나의 그룹으로 보고,
  `predict_current_column_para_y` 기준으로 frame을 넘으면 수식 tail부터 다음 단으로 advance하도록 했다.
- 결과: p19 자체는 `문28` y=803.1로 PDF y≈803.5와 거의 맞았지만,
  `2024-09-between20 flagged=5/24`로 후속 p20~p22 흐름이 크게 회귀했다.
- `eq_group_over=true`는 `문27 ep=4` 한 곳에서만 발생했으므로 조건 과검출은 아니었다.
- 판단: p19의 시각 위치는 "수식 tail부터 다음 단"이 맞지만, 이를 실제 pagination advance로 처리하면
  이후 페이지의 누적 흐름 높이가 한컴보다 늦어진다. 따라서 이 케이스는 단순 advance가 아니라
  저장 `lineSeg/vpos` 기반 시각 배치와 pagination 소비 높이를 분리해서 다뤄야 한다.

두 실험 코드는 모두 되돌렸다.

현재 기준 재확인:

```bash
cargo fmt --check
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --out output/task1293_stage89_reverted_probe
```

결과:

- `cargo fmt --check`: 통과
- `cargo build --bin rhwp`: 통과
- `2024-09-between20`: `flagged=3/24`
  - `line=[11, 19]`
  - `column=[11, 19]`
  - `tail=[18, 19]`
  - `question=[19]`
  - `large=[11, 18, 19]`

다음 접근:

- p19는 개별 문항 보정으로는 해결하지 않는다.
- 공통 원인은 `large betweenNotes + visible separator + 수식-only tail + 다음 텍스트/수식 그룹`에서
  저장 `vpos`가 시각 배치를 요구하지만, 같은 값을 pagination advance로 소비하면 후속 페이지가 늦어지는
  점이다.
- 다음 수정은 "단/쪽 전환을 결정하는 fit 상태"와 "render tree에서 lineSeg vpos를 적용하는 시각 y"를
  같은 scalar height로 묶지 않는 방향이어야 한다.

가설:

- `compute_en_metrics`의 저장 vpos delta 근사가 p18/p19 하단 tail에서 실제 renderer bottom보다 낮게 잡힌다.
- 이미 존재하는 `RHWP_EN_SSOT=A2`의 `simulate_endnote_column_bottom_y`는 단 bottom을 renderer 경로로 다시
  계산하므로, 이 값이 p18/p19의 공통 overflow를 잡을 수 있는지 확인했다.

명령:

```bash
RHWP_EN_SSOT=A2 python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_a2_probe
```

결과:

| 대상 | baseline | A2 probe |
|---|---:|---:|
| `2024-09-between20` | `3/24` | `7/24` |
| `2024-11-practice-shape987` | `1/21` | `12/21` |
| `2024-11-practice-above0-between0-below0` | `0/21` | `2/21` |
| `2024-11-practice-no-separator-above20-between20-below20` | `0/23` | `8/23` |

판단:

- A2 전체 승격은 기존 compact/zero/no-separator 샘플을 크게 흔들어 PR 수준 해법이 아니다.
- 다만 p18/p19처럼 `current_height + en_fit`은 frame 안쪽으로 판단하지만 render-tree bbox가 frame을 넘는
  후보가 있으므로, A2 전체 적용이 아니라 특정 하단 tail의 실제 render y를 판정하는 보조 수단만 검토한다.

## 기각 실험 - HeightCursor 예측 y 기반 large-gap tail split/advance

가설:

- `large_between_tail_render_overflows`가 저장 vpos delta를 직접 계산해 y를 예측하는데, p18/p19에서는
  `HeightCursor`의 lazy base/vpos 조정 결과와 달라 실제 render y를 낮게 예측한다.
- `predict_current_column_para_y`로 한 줄 tail은 다음 단으로 넘기고, 다줄 tail은 frame 안에 들어가는 줄까지만
  split하면 p18/p19를 공통으로 잡을 수 있을 것으로 보았다.

결과:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_tail_predict_probe
```

- `2024-09-between20`: `3/24` -> `6/24`로 악화
  - p19 후보는 일부 줄었지만 p10/p20/p21/p22에 새 drift/overflow가 생겼다.
- `2024-11-practice-shape987`: `1/21` 유지
- `2024-11-practice-above0-between0-below0`: `0/21` 유지
- `2024-11-practice-no-separator-above20-between20-below20`: `0/23` 유지

판단:

- HeightCursor 예측 y 자체는 p18/p19 후보를 설명하지만, 이를 pagination split/advance에 바로 연결하면
  `2024-09-between20` 후반 문항 흐름이 연쇄적으로 밀린다.
- 따라서 p18/p19 후보를 곧바로 layout 수정 대상으로 확정하지 않고, 먼저 PDF/PNG 기준으로 실제 한컴과 다른
  overflow인지, sweep의 bbox 과검출인지 분리한다.
- 해당 실험 코드는 즉시 되돌렸고, 되돌림 확인 targeted sweep은 baseline(`3/24`, `1/21`, `0/21`, `0/23`)으로
  복귀했다.

## 기각 실험 - 큰 TAC 그림 뒤 다음 payload orphan 방지

가설:

- p19 `문28`의 첫 TAC 그림은 그림 자체의 column-relative bottom이 `990.07px / available 1001.56px`로
  frame 안에 들어가므로 기존 `visible_separator_large_tac_tail_overflows_frame`가 켜지지 않는다.
- 하지만 PDF/한컴은 이 그림을 p19에 시작하지 않으므로, `visible separator + large betweenNotes`의 마지막
  단에서 큰 TAC 그림 뒤 다음 payload 문단까지 들어가지 못하면 그림을 다음 page/column으로 넘기는 orphan
  방지 조건을 추가해 보았다.

디버그 확인:

```text
note=28 ep=5 col=2/2 cur=769.41 avail=1001.56
tac_candidate=true tac_render_y=795.87 tac_bottom=990.07
visible_large_tac_tail=false
```

결과:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_tac_orphan_probe
```

- `2024-09-between20`이 `flagged=6/24`로 악화
- p17/p18/p20/p21에 새 red/line/column/question drift가 발생
- 다른 11월 샘플은 기존 수준이었지만, 핵심 2024-09 between20 문서가 악화되어 기각

판단:

- "큰 TAC 그림 뒤 다음 payload가 못 들어가면 그림을 넘긴다"는 조건은 p19를 설명하는 단서이지만,
  단독 공통 조건으로는 너무 넓다.
- p19를 해결하려면 TAC 그림만 보지 말고, 직전 `문27` tail부터 `문28` title/body/TAC head까지 이어지는
  저장 vpos 구간과 한컴/PDF의 실제 question-flow 경계를 같이 비교해야 한다.
- 해당 동작 변경 코드는 즉시 되돌렸고, 디버그 로그용 `tac_candidate/tac_render_y/tac_bottom` 값만 남겨
  다음 분석에서 사용할 수 있게 했다.

## 기각 실험 - 큰 TAC 그림 뒤 다음 첫 줄 orphan 방지

가설:

- 전체 다음 문단 높이를 기준으로 하면 너무 넓으므로, TAC 그림 뒤에 오는 다음 visible text/equation의
  첫 줄 advance만 현재 단 남은 공간과 비교하면 p19만 좁게 포착할 수 있을 것으로 보았다.
- p19 `문28` TAC 후보는 남은 공간이 약 `11.49px`라 다음 첫 줄도 들어가지 못한다.

결과:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_tac_orphan_line_probe
```

- `2024-09-between20`이 `flagged=6/24`로 악화
- p21/p22에 새 red/question/column drift가 발생
- 11월 샘플은 baseline 수준이지만 핵심 between20 문서에서 회귀가 크므로 기각

판단:

- "TAC 그림 뒤 다음 첫 줄" 조건도 p19만 격리하지 못한다.
- p19에서 한컴/PDF와 다른 지점은 `문28` 첫 그림 자체라기보다, `문27` tail 종료 후 `문28` head 그룹이
  어느 정도까지 p19에 남을 수 있는지를 결정하는 group 단위 pagination이다.
- 해당 동작 변경 코드는 즉시 되돌렸다.

## 기각 실험 코드 정리 후 재검증

`visible separator + large betweenNotes` 마지막 단에서 TAC 그림 뒤 다음 첫 줄이 들어가지 않으면
TAC 그림을 다음 쪽으로 넘기는 실험 분기가 코드에 남아 있어 제거했다. 제거 후 같은 targeted sweep을
다시 수행해 stage89 기준치로 돌아왔는지 확인했다.

명령:

```bash
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_after_revert_targeted
```

결과:

| 대상 | SVG/render/PDF | flagged | 판단 |
|---|---:|---:|---|
| `2024-09-between20` | `24/24/24` | `3/24` | p11, p18, p19 후보만 남음. |
| `2024-11-practice-shape987` | `21/21/21` | `1/21` | p12 작은 tail 후보만 남음. |
| `2024-11-practice-above0-between0-below0` | `21/21/21` | `0/21` | clean 유지. |
| `2024-11-practice-no-separator-above20-between20-below20` | `23/23/23` | `0/23` | clean 유지. |

## 남은 20mm large-gap 후보 재분류

`question_flow.json`과 render tree를 다시 비교한 결과, p19는 그림만의 문제가 아니다. PDF/한컴도 p19
오른쪽 단에 `문28` 제목과 첫 본문 일부를 남기지만, rhwp는 `문28` 제목을 약 `53.9px` 위에서 시작한다.
이 때문에 `문28` 첫 TAC 그림이 p19 하단에 끼어 들어가고, 그림만 다음 쪽으로 넘기면 p20 이후 문항
흐름이 다시 어긋난다.

남은 후보는 다음 공통 축으로 다시 본다.

| 페이지 | 현재 후보 | 공통 해석 |
|---:|---|---|
| 11 | `pi=573` 텍스트 뒤 `pi=574` visual tail이 더 위 y로 렌더됨 | 한 미주 내부 visual/TAC tail의 저장 vpos가 순차 텍스트 흐름보다 앞서는 경우 |
| 18 | `문29`의 `pi=922` equation-only tail이 frame 아래 약 `24.5px` | 마지막 단 하단 equation-only tail의 saved-vpos/fit 높이 불일치 |
| 19 | `문27` tail `pi=961` frame 아래 약 `16.3px`, 다음 `문28` title이 PDF보다 `53.9px` 빠름 | 이전 note tail 종료와 다음 note head group 시작 사이에서 `미주 사이` 소비 위치가 달라지는 경우 |

다음 수정은 TAC 그림 단독 advance가 아니라 `이전 note tail + 다음 note title/body/head visual`을 하나의
head group으로 보아, 저장 vpos와 순차 fit 중 어느 값을 적용할지 결정하는 공통 규칙이어야 한다.

## lineSeg 단위 TAC head/tail 감지 재검증

기존 `para_is_treat_as_char_picture_only` 기반 감지는 문단에 탭/빈 text run이 섞인 큰 TAC 그림 줄을
놓친다. p19 `문28`의 큰 그래프도 render tree에서는 `text=""` + TAC 그림 lineSeg 형태라 paragraph-only
판정으로는 잡히지 않았다. 따라서 lineSeg별로 visible text가 없고 treat-as-char Picture/Shape 높이가
큰 줄을 찾는 방식으로 경계 감지를 보강했다.

명령:

```bash
cargo fmt
cargo build --bin rhwp
RHWP_ENDNOTE_BOUNDARY_DEBUG=1 RHWP_ENDNOTE_ADVANCE_DEBUG=1 \
  target/debug/rhwp dump-pages samples/3-09월_교육_통합_2024-미주사이20.hwp -p 19 \
  > output/task1293_stage89_line_tac_boundary_debug_p19.log 2>&1
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_line_tac_boundary_probe
```

결과:

| 대상 | flagged | 판단 |
|---|---:|---|
| `2024-09-between20` | `3/24` | p11, p18, p19 그대로. p19 title drift `-53.9px` 유지. |
| `2024-11-practice-shape987` | `1/21` | baseline 수준 유지. |
| `2024-11-practice-above0-between0-below0` | `0/21` | clean 유지. |
| `2024-11-practice-no-separator-above20-between20-below20` | `0/23` | clean 유지. |

디버그에서 `s0:p334:ci0`의 `note=28` 경계는 `large_tac_head=true`로 잡힌다. 그러나 `문28` 제목
좌표는 `rhwp y=749.6`, PDF `y=803.5`로 그대로였다. 즉 lineSeg 감지는 맞지만, 현재 보정은
`vpos_offset`에만 반영되어 실제 다음 head group의 시작 높이/fit 판단에는 영향을 주지 못한다.

## 기각 실험 - TAC head 경계에서 current_height 직접 예약

가설:

- `large_tac_head=true`인 경계에서 `vpos_offset`뿐 아니라 `st.current_height`에도 `미주 사이` 초과분을
  더하면 `문28` title/body/head visual 그룹이 한컴처럼 아래에서 시작할 수 있다고 보았다.

결과:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_boundary_height_probe
```

- `2024-09-between20`이 `flagged=6/24`로 악화
- p20~p22에 red/question/column drift와 tail 후보가 새로 발생
- `current_height`를 직접 미는 방식은 p19 단일 현상에는 맞아 보일 수 있지만, 뒤쪽 문항 흐름을
  과하게 늦추므로 공통 규칙으로 사용할 수 없다.

판단:

- lineSeg TAC 감지는 유지할 가치가 있지만, `current_height` 직접 가산은 기각하고 되돌렸다.
- 다음 분석은 “다음 head group 자체를 얼마나 현 단에 남길 수 있는가”를 현재 높이 가산이 아니라
  group fit/overflow 판정으로 결정해야 한다.

## 기각 실험 - 큰 TAC head 뒤 follow-up 블록 fit 판정

가설:

- p19 `문28`은 큰 TAC 그림 자체는 frame 안에 들어가지만, 그 뒤의 설명 블록까지는 같은 단에 들어가지
  않는다.
- 따라서 `visible separator + large betweenNotes` 마지막 단에서 head 초반의 큰 TAC 그림 뒤 follow-up
  payload 2개가 같이 들어가지 못하면 큰 TAC부터 다음 단/쪽으로 넘기면 한컴/PDF와 맞을 것으로 보았다.

결과:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_tac_head_follow_probe
```

- `2024-09-between20`이 다시 `flagged=6/24`로 악화
- p19는 의도한 방향으로 움직일 수 있으나 p20~p22 red/question/column drift가 함께 발생
- "큰 TAC + 뒤따르는 payload"만으로는 문서 뒤쪽의 같은 패턴을 구분하지 못한다.

판단:

- p19만 보면 큰 TAC를 넘기는 것이 맞지만, 그 판단을 로컬 paragraph fit에 직접 넣으면 뒤쪽 문항 전체
  흐름이 밀린다.
- 다음 시도는 paragraph 단위 advance가 아니라 한컴/PDF가 실제로 보존하는 저장 vpos 구간을 기준으로
  `문28` head group의 split anchor를 계산해야 한다. 즉 `lineSegArray.vertical_pos`의 단/쪽 경계 신호와
  render tree question marker drift를 결합해, 특정 문항 head group이 앞 단에 남겨질 최대 범위를 정해야 한다.

## 기각 실험 - 큰 TAC head boundary 신호를 paragraph fit에 전달

가설:

- `lineSeg` 기준으로 `large_tac_head=true`인 다음 미주 head를 감지하고, 해당 boundary 신호를 paragraph
  fit 단계에 전달하면 p19 `문28`처럼 큰 TAC head가 앞선 단의 남은 공간을 잘못 점유하는 상황을 공통으로
  줄일 수 있을 것으로 보았다.

결과:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_tac_head_boundary_group_probe
```

- `2024-09-between20`이 `flagged=6/24`로 악화
- p20~p22에 새 red/question/column drift가 발생
- p19 `문28`의 title drift도 `y_delta=-53.9px`로 그대로였다.

판단:

- 현재 `layout.rs`는 column 첫 item의 `vpos`를 `vpos_page_base_init` anchor로 사용한다. 이 구조에서는
  같은 group에 균일하게 들어간 `vpos_offset`이 첫 title 기준으로 상쇄되어 제목 시작 y를 움직이지 못한다.
- TAC/head payload만 별도로 밀면 title anchor와 payload가 분리되어 다음 페이지들에서 drift가 커진다.
- 따라서 stage89의 다음 분석은 `FootnoteShape` 수치 보정이나 paragraph fit 개별 가산이 아니라,
  한컴이 보존하는 "미주 제목 + 본문 head group"의 anchor를 어디에 두는지부터 다시 확인해야 한다.

## 기각 실험 - large betweenNotes saved-vpos text tail overflow 일괄 advance

가설:

- p19 `문27`의 `pi=961`은 누적 `current_height` 기준으로는 fit이지만, 저장 `lineSeg.vertical_pos`를
  반영한 render y 기준으로는 왼쪽 단 frame 아래에 `그러므로`가 남는다.
- 따라서 `visible separator + large betweenNotes`에서 render 예측 y가 frame 아래로 나가는 한 줄 text
  tail을 다음 단으로 넘기면 p19 `문28` drift도 함께 해소될 것으로 보았다.

결과:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_large_between_vpos_tail_probe
```

- `2024-09-between20`이 `flagged=6/24`로 악화
- p19만 아니라 p20~p22에 red/question/column drift와 tail 후보가 새로 발생
- 나머지 3개 target은 baseline 수준을 유지했다.

판단:

- render y 기반 tail overflow 판정 자체는 p19 원인을 설명하지만, large betweenNotes 전체에 일괄 적용하면
  뒤쪽 문항 흐름을 과하게 다음 단/쪽으로 밀어 회귀한다.
- p19의 공통 조건은 "render y overflow"만으로 부족하고, 다음 단에 이미 같은 note의 수식 continuation이
  시작하는 `lineSeg` 경계 또는 현재 tail이 PDF/한컴에서 실제로 다음 단 첫 블록 앞에 재배치되는 신호를
  함께 봐야 한다.
- 해당 실험 코드는 즉시 되돌렸다.

## 기각 실험 - 수식-only + 짧은 텍스트 tail group advance

가설:

- p19의 `문27` tail은 `수식-only 한 줄 -> 짧은 텍스트 한 줄 -> 다음 수식 continuation` 형태다.
- 한컴/PDF는 이 group을 한 단락씩이 아니라 묶음으로 다음 단에 배치하는 것으로 보였으므로, render
  예측 y 기준 group bottom이 frame을 넘으면 첫 수식 단락부터 advance하면 공통 해결될 것으로 보았다.

실험 1:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_equation_text_tail_group_probe
```

- `2024-09-between20` 결과가 baseline과 동일한 `flagged=3/24`였다.
- 원인: `later_endnote_vpos_rewinds_after_current` 조건이 p19의 실제 후보에서 false였다. 이 문서는
  `lineSeg.vertical_pos` rewind가 아니라, layout 단계의 advance 때문에 다음 단으로 넘어가는 형태였다.

실험 2:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_equation_text_tail_group_probe2
```

- `2024-09-between20`이 `flagged=6/24`로 악화
- p19의 `문28` marker drift는 `54px -> 42px`로 일부 줄었지만, p20~p22 red/question/column drift가 새로 발생
- debug에서 실제로 켜진 후보는 의도한 note27 ep4가 아니라 note27 ep2였다.

판단:

- `수식 -> 텍스트 -> 수식` 패턴만으로는 같은 문항 내부의 더 앞쪽 풀이 묶음까지 잘라서 뒤 페이지 회귀를 만든다.
- p19를 해결하려면 "묶음 형태"보다 더 강한 신호가 필요하다. 후보는 PDF/한컴 기준 문항 marker y drift,
  해당 note의 마지막 단 tail 여부, 또는 뒤 문항 title과의 간격을 함께 보는 방식이다.
- 해당 실험 코드는 즉시 되돌렸다.

## 기각 실험 - 큰 수식 tail 뒤 title gap 적용 게이트 완화

가설:

- p19 `문28` title은 `HeightCursor` 내부에서 후보 `result=803.64px`가 계산되지만,
  `injected_between_notes` 단일 줄 prev 보호 분기가 `y_offset=749.64px`를 반환해 실제 렌더에는
  반영되지 않았다.
- 직전 p974는 보이는 텍스트가 없는 수식 3개짜리 큰 lineSeg이고, p975 PDF 기준 y는 `803.5px`이므로
  `title_after_equation_tail_extra_gap`이 생긴 경우에는 `injected_between_notes` 보호 분기를 우회하면
  p19가 맞을 것으로 보았다.

확인:

```text
VPOS_CORR pi=975 y_in=749.64 result=803.64 applied=false
LAYOUT_VPOS pi=975 y_before=749.64 y_after=803.64
render_tree p975 y=803.6, PDF 문28 y=803.5
```

결과:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_large_gap_tall_equation_fixed_probe
```

| 대상 | baseline | 실험 결과 | 판단 |
|---|---:|---:|---|
| `2024-09-between20` | `3/24` | `7/24` | p19는 맞지만 p14, p20~p22 red/question/column drift가 새로 발생 |
| `2024-11-practice-shape987` | `1/21` | `2/21` | p16 red/question drift 회귀 |
| `2024-11-practice-above0-between0-below0` | `0/21` | `0/21` | 영향 없음 |
| `2024-11-practice-no-separator-above20-between20-below20` | `0/23` | `1/23` | p17 tail 후보 회귀 |

판단:

- `HeightCursor`의 p975 후보값은 국소적으로 한컴/PDF와 맞지만, 이를 단순히 적용하면 뒤쪽 note 경계가
  과하게 밀린다.
- `injected_between_notes` 보호 분기는 p10/p12류 overflow 회귀를 막는 역할이 있고, 큰 수식 tail 뒤
  title만으로는 이를 우회할 공통 조건이 부족하다.
- p19 해결에는 "직전 수식 tail + 새 title"뿐 아니라 해당 title/head group이 문서 전체 question flow에서
  어떤 page/column anchor를 가져야 하는지까지 판단해야 한다.
- 실험 코드는 되돌렸고, 되돌린 뒤 targeted sweep이 baseline(`3/24`, `1/21`, `0/21`, `0/23`)으로 복귀함을
  `output/task1293_stage89_after_tall_gap_revert_targeted`에서 확인했다.

## 기각 실험 - 큰 TAC 그림과 후속 설명 group advance

가설:

- p19의 실제 시각 차이는 `문28` title만이 아니라, `문28` 내부 큰 TAC 그림이 p19 오른쪽 단 하단에 남는
  점에서 시작된다.
- 한컴/PDF는 p19 오른쪽 단에 title과 몇 줄만 남기고, 큰 도형은 p20 왼쪽 단 상단으로 넘긴다.
- 따라서 `visible separator + large betweenNotes + 마지막 단 하단 + 큰 TAC 그림 + 뒤 설명 group까지는
  들어가지 않음` 조건에서 TAC 그림 문단부터 advance하면 문항 번호를 쓰지 않고 공통 해결될 것으로 보았다.

결과:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_tac_group_probe
```

| 대상 | baseline | 실험 결과 | 판단 |
|---|---:|---:|---|
| `2024-09-between20` | `3/24` | `6/24` | p21/p22 red/question/column drift와 tail 후보가 새로 발생 |
| `2024-11-practice-shape987` | `1/21` | `1/21` | 변화 없음 |
| `2024-11-practice-above0-between0-below0` | `0/21` | `0/21` | 변화 없음 |
| `2024-11-practice-no-separator-above20-between20-below20` | `0/23` | `0/23` | 변화 없음 |

판단:

- TAC 그림 group advance는 p19 원인 형태를 더 잘 설명하지만, 해당 문단을 실제 pagination에서 넘기면
  뒤쪽 p21/p22 흐름이 연쇄적으로 늦어져 전체 문서 정합이 악화된다.
- 한컴/PDF의 p19 차이는 "그림이 단독으로 들어가느냐"보다 더 높은 수준의 page/column anchor 재해석 문제다.
  특히 `HeightCursor`의 국소 y 후보와 pagination 누적 높이를 한쪽으로만 맞추면 다른 페이지가 회귀한다.
- 해당 실험 코드는 즉시 되돌렸다.

## stage89 재시작 기준 sweep

사용자가 지적한 `FootnoteShape` 표 133/134와 문서화된 실제 28바이트 레코드 해석을 기준으로 다시
점검했다.

확인한 사실:

- `구분선 위`, `구분선 아래`, `미주 사이` 값 자체는 `dump-note-shape`와 HWPX round-trip 경로에서
  분리되어 읽힌다.
- 표 134의 속성 비트도 `placement`, `numbering`, `번호 코드 위첨자`, `텍스트에 이어 출력`으로
  명시적으로 분리했다.
- 그럼에도 `2024-09-between20` p19/p11/p18 후보가 남는 이유는 저장 값 자체가 틀린 것이 아니라,
  같은 미주 값을 pagination/layout 단계에서 어디에 적용해야 하는지의 경계 모델이 아직 충분하지 않기
  때문이다.

기준 확인:

```bash
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_after_failed_pad_revert_targeted
```

결과:

| 대상 | 결과 | 남은 후보 |
|---|---:|---|
| `2024-09-between20` | `3/24` | p11 line/column/order, p18 large tail, p19 line/column/tail/question/large |
| `2024-11-practice-shape987` | `1/21` | p12 column/tail |
| `2024-11-practice-above0-between0-below0` | `0/21` | 없음 |
| `2024-11-practice-no-separator-above20-between20-below20` | `0/23` | 없음 |

## 기각 실험 - 다음 단 수식 continuation start padding

가설:

- p19는 note27 마지막 수식 tail이 왼쪽 단 하단에서 다음 단으로 넘어갈 때, 한컴/PDF가 오른쪽 단
  첫 줄을 rhwp보다 약 49px 낮게 시작하는 형태로 보였다.
- 따라서 `visible separator + large betweenNotes + 수식-only TAC textRun + 다음 줄 visible text` 조건에서
  다음 단 시작부에 `미주 사이` 초과분을 padding으로 주면 공통 해결될 수 있다고 보았다.

확인 로그:

```text
ENDNOTE_PRE_ADV note=27 ep=6 col=1/2 cur=923.25 pad=49.13 eq_tac_only=true next_visible_text=true local_rewind_cond=false
ENDNOTE_ADV phase=fit note=27 ep=6 col=2/2 cur=0.00 large_eq_tail_next_col=true advance_fit=false
EN_RENDER pi=962 y_in_rel=0.0 y_out_rel=35.4
```

판단:

- 실제 단 전환은 `advance_for_fit`이 아니라 더 앞쪽의
  `large_between_equation_tail_starts_next_column` advance 블록에서 발생한다.
- `advance_for_fit` 뒤에 padding을 넣는 실험은 p19에 효과가 없었다.
- 실제 advance 지점에 padding을 넣는 실험은 `2024-09-between20`을 `3/24 -> 7/24`로 악화시켰다.
  p17/p21/p22 red/question/column drift가 새로 생겼다.

결론:

- "다음 단으로 넘어간 수식 continuation은 top padding을 준다"는 규칙은 너무 넓다.
- p19의 핵심은 단순 top padding이 아니라 `lineSeg.vertical_pos` 기반 rewind, pagination advance,
  render-tree vpos 보정이 서로 다른 좌표계를 쓰는 상황에서 note 경계 anchor를 하나로 정규화하지 못하는
  것이다.
- 해당 실험 코드는 되돌렸고, 위 기준 sweep으로 baseline 복귀를 확인했다.

## 수정 2 - 수식 TAC 판정을 composer까지 일관 적용

사용자가 제공한 반례 `samples/수식-문자처럼취급-아님.hwp`는 수식 컨트롤이 문단 안에 있어도
`eq.common.treat_as_char=false`이다. stage77에서 pagination/layout helper는 이미
`Control::Equation` 타입만으로 TAC라고 보지 않도록 정리했지만, `composer.rs`에는 아직
수식을 무조건 `tac_controls`에 넣는 경로가 남아 있었다.

이 상태에서는 pagination은 비TAC 수식을 별도 `Shape` item으로 라우팅하지만, paragraph layout은
같은 수식을 `TextLine` 안의 inline `Equation`으로 다시 렌더할 수 있다. 즉 "수식은 무조건
글자처럼 취급"하는 잔여 경로이며, 미주 문단의 `lineSeg -> ComposedLine -> TAC control`
소속 판단을 공통 모델로 만들려면 먼저 제거해야 한다.

수정:

- `src/renderer/composer.rs`
  - `tac_controls` 수집에서 `Control::Equation(eq)`도 `eq.common.treat_as_char`일 때만 포함한다.
  - `identify_inline_controls`에서도 비TAC 수식은 inline shape로 분류하지 않는다.
  - HWP5 누락 marker 보정의 `controls_are_tac_objects` 판정도 TAC 수식만 수식 TAC로 본다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - `issue_1293_equation_control_is_not_always_treat_as_char`가 render tree에서 비TAC 수식이
    `TextLine` 내부 inline `Equation`으로 남지 않는지 추가 검증한다.

검증:

```bash
cargo fmt
cargo test --test issue_1139_inline_picture_duplicate \
  issue_1293_equation_control_is_not_always_treat_as_char -- --nocapture
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage89_equation_not_tac_composer_probe
```

결과:

| 대상 | 결과 | 판단 |
|---|---:|---|
| `수식-문자처럼취급-아님.hwp` | focused test 통과 | 비TAC 수식이 `Shape` item으로 남고, `TextLine` 안 inline Equation은 0개 |
| `2024-09-between20` | `3/24` | baseline 유지. p11/p18/p19 잔여 후보는 그대로 |
| `2024-11-practice-shape987` | `1/21` | baseline 유지 |
| `2024-11-practice-above0-between0-below0` | `0/21` | clean 유지 |
| `2024-11-practice-no-separator-above20-between20-below20` | `0/23` | clean 유지 |

판단:

- `Control::Equation`의 TAC 판정 잔여 경로는 정리됐다.
- 이 수정은 남은 p11/p18/p19/p12 후보를 직접 줄이지는 않지만, 앞으로 남은 문제를
  `lineSeg` 단위로 좁힐 때 비TAC 수식이 TAC textRun으로 섞여 들어가는 오판을 막는 공통 전제다.
- 다음 분석은 계속 `lineSeg.vertical_pos`, `HeightCursor` 보정 결과, 실제 render tree anchor 사이의
  불일치에 집중한다.

## stage89 현재 기준 재확인

현재 WIP 기준으로 `2024-09-between20`만 다시 sweep했다.

```bash
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --out output/task1293_stage89_current_probe
```

결과:

- `flagged=3/24`
- 남은 페이지: p11, p18, p19
- p19 flags: `render_tree_frame_tail_overflow`, `question_marker_drift`, `line_band_drift`,
  `column_line_band_drift`, `large_ink_region_drift`
- p19 `문28` marker는 rhwp `y=749.6`, PDF `y≈803.5`로 약 `54px` 위에 있다.
- p19 `문28` 첫 TAC 그림은 rhwp p19 오른쪽 단 `y=875.7`, `h=194.2`로 하단에 남는다.

디버그:

```text
ENDNOTE_BOUNDARY note=28 src=s0:p334:ci0 emitted=35 col=2/2 cur=594.17 avail=1001.56 between=5669 prev_spacing=452 extra=5217 large_rewind=false large_tac_head=true continued=true visible_sep=true
```

판단:

- `FootnoteShape` 설정값과 표 133/134 attr 해석은 맞다. `between=5669HU(약 20mm)`,
  `separatorBelow≈2mm`, `separatorAbove=0`도 덤프에서 분리되어 나온다.
- 실패 원인은 값 파싱이 아니라, 새 미주 경계에서 저장 `lineSeg.vertical_pos`와 sequential pagination
  높이 중 어느 쪽을 head group anchor로 삼을지 결정하는 모델에 있다.
- `large_tac_head=true` 신호는 p19를 포착하지만, vpos offset에 전체 `between_notes`를 더해도
  `layout.rs`의 column/page anchor에서 상쇄되어 `문28` 제목 y는 변하지 않았다. 해당 가산 분기는
  코드에서 제거했고, 판정 신호는 디버그용으로만 유지했다.

다음 분석:

- `미주 사이` 수치 자체를 더하거나 빼는 방식은 p20~p22 회귀를 만든다.
- 공통 해법은 `이전 note tail + 다음 note title/body/head visual`을 head group으로 보고,
  group anchor가 저장 vpos에서 오는지, 현재 column sequential flow에서 오는지, 또는 page/column split에서
  새로 잡혀야 하는지를 `lineSeg` 단위로 판정해야 한다.

## stage89 종료 판단

stage89에서는 표 133/134 기준 `FootnoteShape` attr 해석과 HWP5/HWPX 모델 동기화, 그리고
수식 컨트롤의 TAC 판정 잔여 경로 정리를 완료 범위로 삼는다. 현재 기준 검증에서 `2024-09-between20`
은 `flagged=3/24`로 남았고, 잔여 후보는 p11/p18/p19다.

남은 문제는 다음 stage로 넘긴다.

- `visible separator + 20mm betweenNotes` 조합에서 저장 `lineSeg.vertical_pos`와 sequential pagination
  높이 중 어느 값을 note head group anchor로 삼을지 결정하는 모델
- p19의 `문27` tail과 `문28` title/body/head visual 경계에서 `미주 사이` 소비 위치가 PDF/한컴과
  약 54px 어긋나는 문제
- p11/p18의 visual/TAC/equation tail 저장 vpos와 실제 render-tree bbox 불일치 분류

다음 stage는 이번 stage89 변경분을 커밋한 뒤 새 단계 문서로 시작한다. stage90에서는 `미주 사이`
값을 단순 가감하는 실험을 반복하지 않고, `이전 note tail + 다음 note head group`의 page/column
anchor 판정을 별도 모델로 분리하는 방향에서 진행한다.
