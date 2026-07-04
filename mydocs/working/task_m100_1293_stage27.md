# Task 1293 Stage 27: 구분선 없음 첫 단 미주 overflow 정합

## 목적

Stage26에서 `구분선 없음 + 구분선위20 + 미주사이20 + 구분선아래20` 샘플의
`문4` 제목/본문이 좌측 단 하단에 남아 renderer overflow가 발생하는 것을 확인했다. 이번
단계에서는 `PageItem::EndnoteSeparator` 뒤 첫 미주 단에서 pagination 누적 높이와 실제 render
y가 왜 달라지는지 추적하고, 문서별 수치가 아닌 미주 공통 flow 로직으로 보정한다.

## 검토 기준

- 한컴 공식 도움말은 `구분선 위`를 본문과 미주 구분선 사이 간격, `구분선 아래`를 미주 구분선과
  미주 내용 사이 간격, `미주 사이`를 앞 번호 미주 내용과 다음 번호 미주 내용 사이 간격으로
  설명한다.
- `구분선 넣기`가 꺼진 샘플에서도 `구분선 위/아래` 값은 파일에 남아 있으나, 실제 한컴/PDF의
  단 분기와 renderer overflow를 기준으로 소비 방식을 검증한다.

## 현재 관찰

- `dump-pages -p 9` 비교:
  - `above0-between20-below2`: `EndnoteSeparator len=14173 above=0 below=566`,
    단 0 `used=908.8px`, `hwp_used≈911.8px`, diff `-3.1px`
  - `no-separator-above20-between20-below20`: `EndnoteSeparator len=0 above=5669 below=5669`,
    단 0 `used=982.6px`, `hwp_used≈914.2px`, diff `+68.4px`
- `RHWP_VPOS_DEBUG=1 export-render-tree -p 9` 결과:
  - 첫 미주 `pi=451`부터 단 하단 `pi=466`까지 모두 `VPOS_CORR_SKIP`이다.
  - separator 뒤 첫 미주가 직전 본문 `pi=450`과 이어진 것으로 계산되어 lazy base가 음수가 된다.
  - 두 번째 단은 첫 항목이 미주이므로 page base가 정상 잡혀 `VPOS_CORR`가 적용된다.
- render tree 기준 no-separator page 10:
  - `pi=464` y=1087.2, h=12.0
  - `pi=465` y=1105.3, h=15.5
  - `pi=466` y=1126.8, h=15.0
  - body column bottom은 약 1092.3px이므로 `pi=465` 이후는 실제 overflow다.

## 가설

첫 단의 미주는 separator block 이후에도 body 문단과 같은 HeightCursor 흐름으로 들어가며,
저장 vpos 보정이 skip된다. 이때 pagination의 endnote 문단별 누적 높이가 실제 draw bottom보다
작아져 `used_height`는 frame 안쪽으로 보이지만 render tree는 frame 밖으로 나간다.

Stage27 수정 후보:

1. separator 뒤 첫 미주 문단부터 body 이전 문단과의 vpos 관계를 끊고, 미주 전용 기준으로
   fit/advance를 평가한다.
2. 구분선이 없고 위/아래 margin이 큰 block에서는 새 번호 제목이 하단에 남는 조건을 실제
   draw advance 기준으로 더 엄격하게 본다.
3. 수정은 `separator_length == 0 && line_type == 0 && line_width == 0` 같은 공식 미주 모양
   조건과 compact endnote flow에 한정한다.

## 검증 계획

- `cargo fmt --all -- --check`
- `cargo test --lib compact_endnote -- --nocapture`
- `cargo build --bin rhwp`
- target sweep:
  - `2024-11-practice-no-separator-above20-between20-below20`
  - `2024-11-practice-above0-between20-below2`
  - `2024-11-practice-above20-between0-below20`
- 필요하면 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`로 기존 미주 회귀를 확인한다.

## 폐기한 가설 1: 보이지 않는 separator의 첫 단 제목 tail 금지

- 실험 내용:
  - `endnote_has_visible_separator()` helper를 추가해 `separator_line_type == 0`,
    `separator_line_width == 0`, `separator_length == 0`이면 보이는 separator가 없다고 판단했다.
  - `large_separator_block`이라도 보이는 separator가 없으면 `allow_large_separator_first_column_tail`
    대상에서 제외했다.
- 검증:
  - `cargo fmt --all -- --check`: 통과
  - `cargo test --lib compact_endnote -- --nocapture`: 28개 통과
  - `cargo build --bin rhwp`: 통과
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-no-separator-above20-between20-below20 --target 2024-11-practice-above0-between20-below2 --target 2024-11-practice-above20-between0-below20 --out output/task1293_stage27_no_separator_target --rhwp-bin target/debug/rhwp`
- 결과:
  - `no-separator` page 10의 `dump-pages -p 9` 배치가 그대로였다.
  - `pi=464~466` overflow도 그대로 남았다.
- 판단:
  - `allow_large_separator_first_column_tail`만 끊어도 `advance_for_new_endnote`가 발동하지 않았다.
  - 문제는 새 제목 tail 허용 분기 하나가 아니라, 해당 구간의 `endnote_has_vpos_rewind`,
    `large_between_notes_vpos_head_outside`, `current_height` 누적 조건 조합이다.
- 조치:
  - 실익 없는 코드 변경은 되돌렸다. Stage27에는 분석 결과만 남긴다.

## 채택한 수정: 보이지 않는 구분선의 미주 사이 전체 gap 반영

### 원인

`RHWP_ENDNOTE_DEBUG=1`로 page 10 표시분의 `pi=451..466` fit 판정을 확인했다.

- `pi=464`(`문4`)에서 `large_between_out=true`였으므로 새 미주 제목을 다음 단으로 넘겨야 했다.
- 그러나 `allow_compact_tail=true`가 먼저 켜져 `문4` 제목만 좌측 단 하단에 남고,
  `pi=465` 본문부터 우측 단으로 넘어갔다.
- 이 상태가 한컴/PDF와 달랐다. 한컴은 구분선이 보이지 않는 큰 미주 block에서 새 번호 제목과
  그 본문을 같은 우측 단에서 시작한다.

구체적으로 두 계산이 서로 어긋났다.

1. pagination은 `미주 사이 20mm` 중 기본 7mm를 뺀 초과분만 `vpos_offset`에 반영했다.
2. renderer는 이전 미주 문단의 마지막 `line_spacing`을 전체 `미주 사이 20mm`로 반영했다.

보이는 구분선이 없는 샘플에서는 separator line이 기준점을 잡아 주지 않으므로, 이 차이가 첫 단
하단에서 누적되어 pagination은 들어간다고 판단하지만 renderer는 frame 밖으로 그렸다.

### 수정

- `endnote_has_visible_separator()` helper를 추가했다.
  - `separator_line_type`, `separator_line_width`, `separator_length`가 모두 0이면 보이는 구분선이
    없는 것으로 본다.
- 보이는 구분선이 없고 `미주 사이`가 기본값보다 큰 경우에는 pagination도 전체 `미주 사이`
  값을 번호 경계 gap으로 반영한다.
- `large_separator_block`인데 보이는 구분선이 없으면 compact question title tail 허용 대상에서
  제외했다.
  - 이로써 `문4` 제목만 좌측 단 하단에 남고 본문이 우측 단으로 분리되는 흐름을 막는다.

## 검증 결과

- `cargo fmt --all -- --check`: 통과
- `cargo test --lib compact_endnote -- --nocapture`: 28개 통과
- `cargo build --bin rhwp`: 통과
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-no-separator-above20-between20-below20 --target 2024-11-practice-above0-between20-below2 --target 2024-11-practice-above20-between0-below20 --out output/task1293_stage27_tail_fix_target --rhwp-bin target/debug/rhwp`

| target | page count | overflow | 판단 |
|---|---:|---:|---|
| `2024-11-practice-no-separator-above20-between20-below20` | 23/23/23 | 37건 -> 4건 | page 10의 `문4`가 PDF처럼 우측 단 시작으로 이동 |
| `2024-11-practice-above0-between20-below2` | 22/22/22 | 38건 유지 | 보이는 구분선 target 회귀 없음 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 0건 유지 | Stage23 보정 유지 |

`dump-pages -p 9` 기준 no-separator page 10은 다음처럼 바뀌었다.

- 단 0: `pi=451..463`까지 배치
- 단 1: `pi=464`(`문4`)가 첫 항목으로 시작

`output/task1293_stage27_tail_fix_target/2024-11-practice-no-separator-above20-between20-below20/compare/compare_010.png`
기준으로도 `문4` 제목/본문 흐름이 PDF와 같은 우측 단 시작으로 보인다.

## 남은 문제

이번 수정은 Stage26에서 지목한 `pi=464~466` chain을 해결했지만, 전체 미주 기능은 아직 완료가 아니다.

- `no-separator` 잔여 overflow:
  - page 12, `pi=593`, Shape overflow 14.1px
  - page 13, `pi=613`, FullParagraph/Shape overflow 65~71px
- `above0-between20-below2`는 page 12, 14, 18, 21의 후반 chain이 남아 있다.

다음 stage에서는 잔여 `pi=593` 계열의 TAC/shape overflow가 미주 사이/구분선 block과 어떤 관계인지
분리해 분석한다.
