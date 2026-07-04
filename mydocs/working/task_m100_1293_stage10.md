# Task 1293 Stage 10: 미주 모양별 flow 과소/과다 분기 보정

## 목적

Stage9에서 2024년 11월 실전 미주 모양 샘플 8종을 다시 sweep한 결과, 쪽수 mismatch가 세 계열로
좁혀졌다.

| target | PDF | rhwp | 문제 |
|---|---:|---:|---|
| `2024-11-practice-shape987` | 21 | 22 | 기본 근방 9/8/7에서 1쪽 과다 |
| `2024-11-practice-above20-between0-below20` | 21 | 20 | 구분선 위/아래 20mm + 미주 사이 0mm에서 1쪽 과소 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23 | 21 | 구분선 없음 + 20/20/20에서 2쪽 과소 |

이번 스테이지는 이 세 계열을 같은 magic number로 당기지 않고, 한컴 미주 모양의 공식 항목을
다음 의미로 나누어 적용한다.

- `구분선 위`: 구분선이 있든 없든 첫 미주 본문 전에 소비되는 구분선 블록 상단 여백
- `구분선 아래`: 구분선이 있든 없든 첫 미주 본문 전에 소비되는 구분선 블록 하단 여백
- `미주 사이`: 서로 다른 미주 번호 묶음 사이에서만 소비되는 간격
- `구분선 없음`: 선분 높이는 0이지만 위/아래 여백은 유지

## 현재 단서

- `above20-between0-below20`은 첫 구분선 블록 20/20은 들어갔지만 후반부가 PDF보다 한 페이지
  빨리 끝난다. p19/p20 compare에서 rhwp가 문28~문30을 PDF보다 앞 페이지에 배치한다.
- `no-separator-above20-between20-below20`은 선분을 그리지 않는 것은 맞지만 23쪽 기준 PDF보다
  2쪽 적다. 여러 페이지에서 `LAYOUT_OVERFLOW`가 계속 발생한다.
- `shape987`은 반대로 22쪽으로 늘어나므로, 20mm 계열 보정을 기본 근방 8mm/9mm까지 적용하면 안
  된다.

## 샘플 확인

작업지시자가 추가한 2024년 11월 실전 샘플은 모두 HWP/HWPX/PDF 쌍으로 존재한다.

| target | 파일명 설정 | PDF | rhwp |
|---|---|---:|---:|
| `2024-11-practice-shape987` | 구분선 위 9 / 미주 사이 8 / 구분선 아래 7 | 21 | 22 |
| `2024-11-practice-above0-between0-below0` | 구분선 위 0 / 미주 사이 0 / 구분선 아래 0 | 21 | 21 |
| `2024-11-practice-above0-between7-below2` | 구분선 위 0 / 미주 사이 7 / 구분선 아래 2 | 21 | 21 |
| `2024-11-practice-above0-between7-below20` | 구분선 위 0 / 미주 사이 7 / 구분선 아래 20 | 21 | 21 |
| `2024-11-practice-above0-between20-below2` | 구분선 위 0 / 미주 사이 20 / 구분선 아래 2 | 22 | 22 |
| `2024-11-practice-above20-between0-below20` | 구분선 위 20 / 미주 사이 0 / 구분선 아래 20 | 21 | 20 |
| `2024-11-practice-above20-between7-below2` | 구분선 위 20 / 미주 사이 7 / 구분선 아래 2 | 21 | 21 |
| `2024-11-practice-no-separator-above20-between20-below20` | 구분선 없음 / 위 20 / 사이 20 / 아래 20 | 23 | 21 |

`dump-note-shape` 기준 raw 슬롯과 UI 의미값도 이름과 일치한다.

- `구분선 위`: `separatorMarginBottom`/UI `separatorAboveMm`
- `구분선 아래`: `noteSpacing`/UI `separatorBelowMm`
- `미주 사이`: `rawUnknown`/UI `betweenNotesMm`
- `구분선 없음`: line type/width/length가 모두 0, 위/아래/사이 여백은 유지

한컴 공식 도움말은 미주 모양 탭에서 번호/구분선/여백/번호 매기기/미주 위치를 설정한다고 설명하고,
공식 HWP 5.0 revision 1.3 스펙의 `HWPTAG_FOOTNOTE_SHAPE`는 `구분선 위 여백`,
`구분선 아래 여백`, `주석 사이 여백`을 별도 필드로 둔다.

## 추가 진단

`no-separator-above20-between20-below20` p11 두 번째 단에서 `pi=545` 이후가 실제 렌더 시 단
하단을 넘는다.

```text
LAYOUT_Y ... pi=537 y_after=612.8
LAYOUT_Y ... pi=537 Shape y_after=788.0
LAYOUT_Y ... pi=544 y_after=1164.4
LAYOUT_OVERFLOW ... pi=545 y=1176.4 bottom=1092.3 overflow=84.1px
```

`dump-pages`는 같은 단의 used height를 987.5px로 보고하지만, layout의 실제 y 진행은 약
1206px까지 내려간다. 즉 남은 mismatch의 한 축은 공식 값 파싱이 아니라, 미주 가상 문단의
typeset 높이와 layout 높이가 다르게 소비되는 문제다.

일반 본문 path에는 비-TAC `TopAndBottom + Para` Picture/Shape가 후속 본문을 밀어내는 경우
`pushdown_h`를 `current_height`에 더하는 공통 로직이 있다. 반면 미주 path는 같은 컨트롤을
`PageItem::Shape`로 발행하지만 `current_height`에는 paragraph vpos/formatter 기반 높이만
더한다. 다만 `shape987`은 현재도 1쪽 과다이므로, 이 보정은 기본 근방 샘플에 일괄 적용하지 않고
위/아래/사이 여백이 큰 샘플에서 실제 한컴 분기와 비교하며 좁게 적용해야 한다.

## 분석 계획

1. 세 mismatch target의 `dump-note-shape`, `dump-pages`, compare PNG를 같은 페이지 축으로 비교한다.
2. `구분선 아래 20mm`가 첫 미주 블록에만 소비되는지, 각 새 페이지/단의 미주 재개 시에도 다시
   소비되는지 확인한다.
3. `미주 사이 0mm`에서 번호 묶음 간 간격을 음수/0으로 압축하지 않도록 LINE_SEG spacing과
   formatter floor의 관계를 확인한다.
4. `구분선 없음 20/20/20`은 선분 없는 separator block의 높이와 새 단/새 페이지 미주 재개 높이
   예약을 분리해서 확인한다.

## 구현 결과

`typeset.rs`의 미주 가상 문단 높이 계측에서 구분선 위/아래 여백이 모두 기본 미주 사이 기준보다
큰 경우를 `large_separator_block`으로 분리했다.

- 큰 구분선 블록에서는 저장된 vpos를 stale 값으로 보지 않는다.
- 새 미주 묶음 advance cap도 큰 구분선 블록에서는 적용하지 않는다.
- 이 분기는 공식 미주 모양의 `구분선 위`/`구분선 아래` 값이 모두 큰 샘플에만 걸리므로,
  `shape987`처럼 기본 근방인 9/8/7 계열에는 적용되지 않는다.

이 보정으로 다음 두 mismatch가 쪽수 기준으로 해소됐다.

| target | 이전 PDF/rhwp | 이후 PDF/SVG/render tree | 결과 |
|---|---:|---:|---|
| `2024-11-practice-above20-between0-below20` | 21/20 | 21/21/21 | 해소 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23/21 | 23/23/23 | 해소 |

`2024-11-practice-shape987`은 여전히 PDF 21쪽, SVG/render tree 22쪽이다. 이번 보정은 큰
위/아래 여백에 대한 공식 필드 적용 범위이며, 9/8/7 계열의 1쪽 과다는 다음 스테이지에서 별도
원인으로 이어서 본다.

## 검증 결과

- `cargo fmt --all -- --check`
- `python3 -m py_compile scripts/task1274_visual_sweep.py`
- `cargo build --bin rhwp`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- 2024-11 샘플 8종 sweep

2024-11 신규 샘플 8종 쪽수 결과:

| target | PDF | SVG | render tree |
|---|---:|---:|---:|
| `2024-11-practice-shape987` | 21 | 22 | 22 |
| `2024-11-practice-above0-between0-below0` | 21 | 21 | 21 |
| `2024-11-practice-above0-between7-below2` | 21 | 21 | 21 |
| `2024-11-practice-above0-between7-below20` | 21 | 21 | 21 |
| `2024-11-practice-above0-between20-below2` | 22 | 22 | 22 |
| `2024-11-practice-above20-between0-below20` | 21 | 21 | 21 |
| `2024-11-practice-above20-between7-below2` | 21 | 21 | 21 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23 | 23 | 23 |

회귀 확인:

| target | PDF | SVG | render tree | 비고 |
|---|---:|---:|---:|---|
| `2024-09-between20` | 24 | 24 | 24 | 기존 쪽수 유지 |
| `2024-09-below20-above20` | 23 | 23 | 23 | 기존 쪽수 유지 |

산출물:

- `output/task1293_stage10_sample_check_all/summary.json`
- `output/task1293_stage10_2024_09_regression/summary.json`
