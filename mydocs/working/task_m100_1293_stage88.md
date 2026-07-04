# task 1293 stage88 - visible separator 미주 공통 경계 보정

## 목적

stage87 WIP와 sweep 결과를 기준으로 남은 문제를 개별 문항별 보정이 아니라 한컴 미주 모양의
공통 경계 모델로 다시 정리한다. 현재 남은 회귀는 특정 문제 번호가 아니라 `구분선 있음` 조합에서
본문 끝, 구분선 위, 구분선 아래, 미주 사이, 다음 미주 제목/본문 시작점이 서로 다른 곳에서 소비되는
문제이므로 이를 공통 로직으로 처리한다.

## 기준

- 브랜치: `local/task_m100_1293`
- 기준: `upstream/devel` (`2a176d09`)
- 현재 커밋 상태: upstream/devel 대비 2커밋 ahead
- 현재 작업 트리: stage87 WIP 존재
  - `src/renderer/typeset.rs`
  - `tests/issue_1139_inline_picture_duplicate.rs`
- 최신 targeted sweep:
  - `output/task1293_stage87_after_trim_revert_targeted/summary.json`

## 최신 sweep 관찰

| 대상 | 페이지 | flagged | 주요 후보 |
|---|---:|---:|---|
| `2024-09-between20` | `24/24/24` | `6/24` | tail `17,19`, question drift `16,17,18,19,21`, line drift `11,16,17,19,21` |
| `2024-11-practice-shape987` | `21/21/21` | `9/21` | tail `11,12,14,16,17,19`, question drift `11,16,17,18,19,20,21`, line drift `11,21` |

clean 기준으로 확인된 조합:

- `2024-11-practice-above0-between0-below0`: `flagged=0/21`
- `2024-11-practice-no-separator-above20-between20-below20`: `flagged=0/23`

## 이미 기각한 접근

`visible separator && non-default between_notes` 조합 전체에 대해 단순히 마지막 줄 trailing line spacing을
일괄 제거하는 실험은 기각한다. `2024-09-between20`에서 SVG/PDF 페이지 수가 `22/24`로 깨져서,
이 문제는 단순 trailing gap 제거가 아니라 미주 경계별 소비 위치를 분리해야 한다.

## 공통 원인 가설

1. `미주 사이` 자체 marker 간격은 일부 페이지에서 PDF와 거의 일치한다. 예를 들어
   `2024-11-practice-shape987` 11쪽은 marker gap delta가 최대 1px 수준인데도 우측 컬럼 문항 y drift가
   약 78px 발생한다. 따라서 문제는 번호 marker 사이 간격만이 아니라 새 미주 블록이 단 경계에서
   시작되는 기준이다.
2. `2024-09-between20` 17쪽은 좌/우 컬럼 모두 tail overflow가 있으며, PDF에는 왼쪽 컬럼 상단 미주가
   하나 더 남아 있다. 현재 로직은 일부 본문을 다음 컬럼으로 너무 일찍 넘기거나, 반대로 tail을
   프레임 아래까지 밀어 넣는다.
3. 공통 처리는 아래 경계를 분리해야 한다.
   - 본문 마지막 줄이 차지하는 높이
   - 같은 미주 내 문단 간 줄 간격
   - 이전 미주 본문 끝과 다음 미주 제목 사이의 `미주 사이`
   - 구분선 아래 첫 미주 시작 간격
   - 단/페이지 하단에서 새 미주 전체 또는 제목+첫 본문을 함께 보낼지 판단하는 fit 기준

## 처리 방향

- `EndnoteFlowProfile`에 흩어진 visible separator 판정을 유지하되, 실제 높이 소비는 공통 helper로
  모은다.
- render-y 기준으로 새 미주 제목/본문이 현재 단에 남아야 하는지 판단하고, 이 판단을 개별 문항 번호가
  아니라 `구분선 있음 + 미주 사이/구분선 위/아래 조합`에 적용한다.
- frame tail overflow 후보가 있는 페이지는 수치 보정 전에 render tree에서 어떤 pi가 현재 단에 남았어야
  하는지 확인한다.
- clean 조합인 0/0/0과 no-separator 20/20/20은 회귀 금지 기준으로 둔다.

## 검증 계획

사용자 승인 전 full CI는 수행하지 않는다.

1. `cargo build`
2. focused test:
   `cargo test --test issue_1139_inline_picture_duplicate issue_1293_ -- --nocapture`
3. targeted sweep:
   `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20 --target 2024-11-practice-shape987 --target 2024-11-practice-above0-between0-below0 --target 2024-11-practice-no-separator-above20-between20-below20 --out output/task1293_stage88_targeted`
4. `annotated_*.png`, `metrics.json`, `question_flow.json`, `render_tree_*.json`로 visible separator 공통
   회귀 여부를 확인한다.

## stage88 원인 확인

`2024-11-practice-shape987` 11쪽에서 한컴/PDF는 `문12`의 `(ⅳ)` 문단이 왼쪽 단 하단에 남고,
오른쪽 단은 `한편, ...`으로 시작한다. 수정 전 rhwp는 `pi=530` `(ⅳ)`를 오른쪽 단 첫 문단으로
보내서 오른쪽 단 전체가 약 60px 내려갔다.

`RHWP_ENDNOTE_ADVANCE_DEBUG=1` 기준으로 `note=12 ep=3`까지는 왼쪽 단 `cur=848.79`,
`ep=4`는 문단 자체 높이 `total=54.08`이라 `848.79 + 54.08 <= 1001.56`으로 순차 fit이 가능하다.
그런데 `large_between_tail_render_overflows`가 저장 `lineSeg` vpos 직접 예측만 보고 `pi=530`을
다음 단으로 advance했다. 이 샘플은 `visible_sep=true`, `compact_gap=true`, `default_gap=false`인
보이는 구분선 + 비기본 compact 미주 사이 조합이므로, 문단 자체가 현재 단에 들어가면 저장 vpos 예측보다
순차 fit을 우선해야 한다.

## stage88 수정

`src/renderer/typeset.rs`에 `visible_compact_sequential_tail_fits_current_column` 공통 guard를 두고,
다음 두 advance 경로에서 같은 판단을 공유하게 했다.

- A2 overflow advance: 문단 자체가 현재 단에 들어가는 compact visible tail이면 advance하지 않는다.
- `large_between_tail_render_overflows`: 저장 vpos 예측이 frame 밖으로 보이더라도, 같은 compact visible
  tail이 순차 fit이면 다음 단으로 미리 넘기지 않는다.

이 변경은 특정 문항 번호가 아니라 `보이는 구분선 + 비기본 compact 미주 사이 + 현재 단 순차 fit` 조합에
적용된다.

## stage88 검증 결과

명령:

- `cargo build`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1293_ -- --nocapture`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20 --target 2024-11-practice-shape987 --target 2024-11-practice-above0-between0-below0 --target 2024-11-practice-no-separator-above20-between20-below20 --out output/task1293_stage88_targeted`

결과:

| 대상 | SVG/render/PDF | flagged | 주요 변화 |
|---|---:|---:|---|
| `2024-09-between20` | `24/24/24` | `6/24` | 기존 20mm large gap 후보 유지. 이번 compact visible guard로 신규 회귀 없음. |
| `2024-11-practice-shape987` | `21/21/21` | `7/21` | `9/21`에서 개선. line drift 후보가 사라지고 11쪽 `pi=530`이 왼쪽 단 하단에 남음. |
| `2024-11-practice-above0-between0-below0` | `21/21/21` | `0/21` | clean 유지. |
| `2024-11-practice-no-separator-above20-between20-below20` | `23/23/23` | `0/23` | clean 유지. |

직접 확인:

- `output/task1293_stage88_probe_shape987_p11_after2/render_tree_011.json`
- `pi=530` `(ⅳ)`가 왼쪽 단 `y=1042.0/1060.0/1078.0`에 남고, 오른쪽 단은 `pi=531` `y=90.7`로 시작한다.
- `문13` marker는 수정 전 `y=574.9`에서 `y=514.8`로 올라가 PDF 기준 흐름에 가까워졌다.

## A3 재적용 검토와 기각

`RHWP_EN_SSOT=A3`로 같은 4개 샘플을 다시 probe했다.

명령:

```bash
RHWP_EN_SSOT=A3 python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage88_targeted_a3_probe
```

결과:

| 대상 | flagged |
|---|---:|
| `2024-11-practice-shape987` | `9/21` |
| `2024-11-practice-above0-between0-below0` | `10/21` |
| `2024-11-practice-no-separator-above20-between20-below20` | `4/23` |

A3는 `shape987`를 stage88 수정 전 수준으로 되돌리고, clean 기준 샘플까지 깨뜨렸다. 따라서 A3는
stage88 공통 해결책이 아니며, 현재 단계에서는 기각한다.

## 추가 수정 1 - absorbed gap의 마지막 단 중복 소비 제거

`visible separator + absorbed_between_notes_gap` 조합에서 이미 이전 문단의 render 위치가 `미주 사이`
간격을 소비했는데, 마지막 단의 이어지는 tail에 다시 `between_notes` line spacing을 주입하는 경우가
있었다. 이때 `문19`처럼 제목과 뒤 본문이 한 note gap만큼 아래로 밀린다.

`src/renderer/typeset.rs`의 render 준비 단계에서 다음 조건을 공통 guard로 분리했다.

- `visible_separator`
- `absorbed_between_notes_gap`
- `continued_endnote_tail_before_new_note`
- 마지막 단
- 현재 높이가 단 하단부에 있음

이 조건에서는 이전 문단 `last_seg.line_spacing = between_notes_hu` 재주입을 생략한다.

검증:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage88_absorbed_lastcol_gap_probe
```

결과:

| 대상 | flagged |
|---|---:|
| `2024-11-practice-shape987` | `5/21` |
| `2024-11-practice-above0-between0-below0` | `0/21` |
| `2024-11-practice-no-separator-above20-between20-below20` | `0/23` |

`shape987`의 p12/p14 후보가 빠졌고, clean 샘플은 유지됐다. 같은 보정을 마지막 단 조건 없이 넓히면
p10/p19 drift가 생겨서 마지막 단 조건을 유지한다.

## 추가 수정 2 - page-path 중하단 제목 stale-forward gap 접기

`shape987` p16은 render path에서 `prev_ls=2267`이 남은 상태로 `문25`, `문26` 제목 앞에 저장 vpos
stale-forward가 다시 적용됐다. 이 패턴은 특정 문항이 아니라 아래 형태다.

- page-path
- 보이는 구분선 + compact `미주 사이`
- 현재 문단이 새 미주 제목
- 저장 vpos가 순차 y보다 100px 이상 stale-forward
- 제목이 단 중하단 영역에 있음

`src/renderer/height_cursor.rs`에 `compact_endnote_page_title_mid_stale_gap` 공통 predicate를 추가하고,
이 경우 제목 y와 후속 vpos base를 `prev_line_spacing` 한 번만 접도록 했다.

같은 단계에서 `height_cursor` 단위 테스트의 두 기존 fixture도 현재 의도에 맞게 안정화했다.

- `strong_tall_tail_backtrack`은 렌더러가 실제 이전 콘텐츠 bottom을 제공할 때만 강한 backtrack을 쓴다.
- 제목 직후 다줄 tail은 측정된 이전 콘텐츠 bottom이 없으면 저장 vpos의 제한 backtrack을 허용한다.

## 기각한 추가 실험

`current_is_endnote_title && compact between_notes && stored vpos backtrack` 전체를 제목 y에서 직접 접는 실험은
기각했다. `shape987` p17의 `문28`은 올라가지만 `문29`와 p20의 `문29`가 과도하게 위로 이동했다.
남은 p17/p18 후보는 제목 자체 gap이 아니라 앞 문단의 렌더 높이와 후단 title gap 보존이 함께 얽힌
문제로 분리해서 봐야 한다.

## 최종 targeted 검증

명령:

```bash
cargo build
cargo test --lib height_cursor -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1293_ -- --nocapture
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage88_final_targeted_probe2
```

결과:

| 대상 | SVG/render/PDF | flagged | 판단 |
|---|---:|---:|---|
| `2024-09-between20` | `24/24/24` | `6/24` | 기존 large-gap 후보 유지. stage88 신규 회귀 없음. |
| `2024-11-practice-shape987` | `21/21/21` | `4/21` | stage87 `9/21`에서 개선. p16/p12/p14 후보 제거. |
| `2024-11-practice-above0-between0-below0` | `21/21/21` | `0/21` | clean 유지. |
| `2024-11-practice-no-separator-above20-between20-below20` | `23/23/23` | `0/23` | clean 유지. |

테스트:

- `cargo build`: 통과
- `cargo test --lib height_cursor -- --nocapture`: `49 passed`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1293_ -- --nocapture`: `9 passed`

## 남은 후보

stage88은 공통 visible separator compact 경계 중 “순차 fit인데 vpos 예측 때문에 앞당겨 advance되는 tail”과
“page-path 중하단 제목 stale-forward gap”을 줄였다. 남은 후보는 아래처럼 성격이 다르다.

- `2024-11-practice-shape987` p11: 문14 수식 tail의 하단 bleed 후보.
- `2024-11-practice-shape987` p17/p18: 문28/문30 전후의 앞 문단 렌더 높이와 title gap 보존이 결합된 흐름 후보.
- `2024-11-practice-shape987` p20: column line band drift 후보.
- `2024-09-between20`: 기존 20mm large-gap 후보 6쪽.

이들은 stage88의 직접 제목 y 보정으로 풀면 다른 문항이 과보정되는 것을 확인했으므로, 다음 단계에서는
문단 렌더 높이와 `prev_endnote_title_gap_px` 후단 보존 로직을 기준으로 다시 공통화해야 한다.

## 추가 수정 3 - 마지막 텍스트+TAC 수식 tail keep

`shape987` p21은 `문30` 제목이 PDF보다 약 60px 위에서 시작했다. 직접 비교해 보니 원인은
새 미주 제목 gap이 아니라 이전 미주 29의 마지막 tail 분리였다.

- rhwp: p20 오른쪽 단 하단에 `pi=974` 텍스트 문단을 남기고, p21에는 `pi=975` TAC-only 수식만 넘김.
- PDF/한컴: `pi=974` 텍스트 문단과 `pi=975` TAC-only 수식 문단을 함께 p21로 넘김.

이 패턴은 특정 문항 번호가 아니라 아래 공통 형태다.

- 보이는 구분선 + 비기본 compact `미주 사이`
- 현재 미주의 끝에서 두 번째 문단
- 현재 문단의 마지막 줄에 문자처럼 취급되는 TAC 수식이 있음
- 다음 문단이 TAC-only 수식 문단
- 현재 문단만 마지막 단에 남기면 들어가지만, 다음 TAC 수식까지는 들어가지 않음

`src/renderer/typeset.rs`에
`large_between_last_column_final_lead_tac_tail_starts_next_page` guard를 추가해 이 경우 현재 문단도
다음 페이지에서 시작하게 했다. 이로써 p21은 PDF처럼 이전 tail 두 줄이 먼저 나오고 그 아래 `문30`이
시작한다.

## 기각한 추가 실험 2 - 표 뒤 텍스트 tail 전체 넘김

`shape987` p12에는 `문19` 표 뒤 설명 문단(`pi=592`)이 render tree 기준으로 frame 아래 16.9px
후보로 잡힌다. 이를 없애려고 “마지막 단 + 표 바로 뒤 다줄 텍스트 tail”을 render-y overflow 기준으로
다음 페이지에 보내는 guard를 실험했지만 기각했다.

- p12 overflow 후보는 사라졌으나, p13이 새 `question_marker_drift`, `line_band_drift` 후보가 됐다.
- PDF p13은 그래프 이미지부터 시작하지만, 실험안은 p12에서 넘긴 `pi=592` 텍스트를 p13 맨 위에
  올려 전체 흐름을 깨뜨렸다.

따라서 p12는 “문단 전체 advance”가 아니라 render-y 미세 조정 또는 sweep 허용 후보로 따로 분리해야 한다.

## 최신 targeted 검증 갱신

명령:

```bash
cargo build
cargo fmt --check
cargo test --test issue_1139_inline_picture_duplicate issue_1293_ -- --nocapture
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage88_final_common_tail_probe
```

결과:

| 대상 | SVG/render/PDF | flagged | 판단 |
|---|---:|---:|---|
| `2024-09-between20` | `24/24/24` | `6/24` | 기존 large-gap 후보 유지. 이번 tail keep guard로 신규 회귀 없음. |
| `2024-11-practice-shape987` | `21/21/21` | `1/21` | p17/p18/p21 question drift 제거. p12의 작은 tail 후보만 남음. |
| `2024-11-practice-above0-between0-below0` | `21/21/21` | `0/21` | clean 유지. |
| `2024-11-practice-no-separator-above20-between20-below20` | `23/23/23` | `0/23` | clean 유지. |

테스트:

- `cargo build`: 통과
- `cargo fmt --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1293_ -- --nocapture`: `9 passed`

최신 남은 후보:

- `2024-11-practice-shape987` p12: `pi=592` 표 뒤 설명 문단의 render tree frame tail 후보.
  전체 advance는 p13 흐름을 깨뜨리므로 기각.
- `2024-09-between20`: 기존 20mm large-gap 후보 6쪽.

## 추가 수정 4 - compact gap과 large gap advance 경계 분리

`2024-09-between20`의 20mm large gap 후보를 줄이기 위해 마지막 단에서 아래 공통 조건을 추가했다.

- 보이는 구분선 + 비기본 `미주 사이`
- 마지막 단 하단
- 저장 vpos 기준으로 새 미주 제목/본문 head group이 frame 밖으로 밀리는 경우
- 이전 미주 tail의 마지막 줄만 현재 단에 남고 나머지는 다음 쪽으로 이어져야 하는 경우

초기 실험에서는 이 guard가 `2024-11-practice-shape987`의 8mm compact gap에도 적용되어
p16/p17에서 `문26`, `문30`의 `advance_new=true`가 다시 발생했다. 로그상 문제 문항은 모두
`compact_gap=true`, `gap=Some(30.226...)`인데도 `large_head_outside=true`가 켜졌다.

따라서 stage88의 최종 형태에서는 아래처럼 공통 경계를 분리했다.

- 20mm급 large gap 전용:
  - `large_between_last_column_visual_split`
  - `large_between_last_column_flow_tail_split`
  - `large_between_last_column_question_title_tail_fits`의 head group fit
  - `large_between_last_column_vpos_head_group_outside`
  - `large_between_notes_head_near_bottom`
  - `large_between_notes_vpos_head_outside`의 마지막 saved-vpos fallback
- 8mm compact gap 전용:
  - 기존 `allow_compact_question_title_tail`
  - 순차 fit 우선 guard
  - page-path 중하단 stale gap 접기

핵심은 `visible_large_between_notes_gap`만으로는 충분하지 않고, 한컴 미주 모양의 8mm compact gap은
large-gap advance 정책에서 명시적으로 제외해야 한다는 점이다.

## 최신 targeted 검증 갱신 2

명령:

```bash
cargo fmt --check
cargo build
cargo test --test issue_1139_inline_picture_duplicate issue_1293_ -- --nocapture
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage88_compact_large_head_split_probe
```

결과:

| 대상 | SVG/render/PDF | flagged | 판단 |
|---|---:|---:|---|
| `2024-09-between20` | `24/24/24` | `4/24` | 20mm large-gap 공통 guard 유지. p18 question drift는 제거됐고 p19/p21 등 기존 후보만 남음. |
| `2024-11-practice-shape987` | `21/21/21` | `1/21` | compact 8mm 회귀 제거. red/question/line/large drift가 사라지고 p12 tail 후보만 남음. |
| `2024-11-practice-above0-between0-below0` | `21/21/21` | `0/21` | clean 유지. |
| `2024-11-practice-no-separator-above20-between20-below20` | `23/23/23` | `0/23` | clean 유지. |

테스트:

- `cargo fmt --check`: 통과
- `cargo build`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1293_ -- --nocapture`: `9 passed`

최신 남은 후보:

- `2024-11-practice-shape987` p12: `pi=592` 표 뒤 설명 문단의 6px bleed/tail 후보. 전체 문단 advance는 p13 흐름을 깨뜨려 기각.
- `2024-09-between20` p11/p18/p19/p21: 20mm large-gap 계열 후보. p18 question drift는 제거됐지만 p19/p21의 large ink/line 후보는 추가 공통화 필요.

## 표 134 속성 비트 재검토

`mydocs/tech/한글문서파일형식_5.0_revision1.3.md` 표 134를 기준으로 보면 현재 구현은
각주/미주 모양의 간격 값뿐 아니라 `attr` 비트 의미도 완전하게 반영하지 못한다.

- bit 8~9: 다단/미주 위치
- bit 10~11: 번호 매기기
- bit 12: 주석 내용 번호 코드의 위첨자 여부
- bit 13: 텍스트에 이어 바로 출력할지 여부

확인된 구현 차이:

- `src/parser/body_text.rs`의 `parse_footnote_shape_record`는 `numbering`과 `placement`를 모두
  `(attr >> 8) & 0x03`에서 읽는다. 표 134 기준으로 `numbering`은 `(attr >> 10) & 0x03`이어야 한다.
- `src/parser/hwpx/section.rs`도 HWPX `numbering`을 attr bit 8~9에 반영한다. 표 134 기준으로
  `numbering`은 bit 10~11에 반영해야 하며, bit 8~9는 `placement` 전용이다.
- `FootnoteShape` 모델에는 bit 12, bit 13을 표현하는 필드가 없다. HWPX의
  `autoNumFormat supscript`와 `placement beneathText`도 현재 공통 모델/렌더링 판단에 충분히
  연결되어 있지 않다.

단, 현재 주로 보고 있는 `3-09월_교육_통합_2024-미주사이20.hwp`의 `endnoteShape.attr`는 `0`으로
덤프되므로, p19/p21 20mm 후보의 직접 원인은 raw 간격 파싱 오류가 아니라 `lineSeg`/저장 vpos와
미주 경계 높이 소비 위치의 불일치다. 하지만 표 134 비트 누락 때문에 다른 샘플에서는 위치/번호
정책부터 잘못 해석될 수 있으므로, stage88 이후 공통 모델 정리에는 이 attr 해석 수정이 포함되어야 한다.
