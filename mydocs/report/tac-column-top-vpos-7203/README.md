---
kind: report
status: active
last_verified: 2026-09-21
---

# [#7203] 단 맨 위 어울림(TAC) 표가 호스트의 저장 첫 줄 `vertical_pos` 를 버린다

- 대상 SHA: `upstream/devel` `a6dcbc138` 위의 한 갈래 추가
- 기준 PDF: `pdf/hwpctl_API_v2.4-hwp-2020.pdf` (engine 2020, 105쪽) ·
  `pdf/footnote-01-hwp-2020.pdf`
- 계측: 전폭 427.7px 표의 **윗변·아랫변을 표 높이로 짝지어** 잠근 뒤 정본 가로 괘선과
  `export-render-tree` 의 `Table` bbox 를 대조(165건 짝지음)

## 1. 무엇이 어긋났나

`LINE_SEG.vertical_pos` 는 문단 기준이 아니라 쪽(단) 상단 기준 절대값이다. 호스트 문단이
단 맨 위에 오면 흐름 커서가 곧 단 상단이므로 저장값이 그대로 위 여백이 된다. 본문 문단은
`paragraph_layout` 의 column-top 계약(Task #1811 — `vpos ≤ spacing_before` 면 가산)이 이
값을 싣는데, **빈 앵커 문단의 TAC 표**는 `PageItem::FullParagraph` 가 발행되지 않아 그
블록을 아예 타지 않는다(진단으로 확인: 이 문서의 32079HU 표 호스트 **158건 전부**
`layout_paragraph_lines` 의 해당 블록에 도달하지 않는다). 그래서 표가 단 상단 + 바깥 위여백에
붙었다.

## 2. 정본이 두 갈래를 가른다

단 맨 위(`y≈136`)에 놓인 표 **15건 전수**를 보면 어긋남이 저장 `vpos` 한 값에 걸려 있다.

| 저장 `vpos` | tac | n | 정본 윗변 | 종전 rhwp | 정본 − rhwp |
| ---: | --- | ---: | ---: | ---: | ---: |
| **500 HU** | true | **8** | 142.56 | 136.00 | **+6.56** |
| 0 | true | 1 | 136.01 | 136.00 | +0.01 |
| 사다리 절대(42697~63381) | false | 6 | 136.01 | 136.00 | +0.01 |

`500 HU = 6.67px` 이고 어긋남이 `+6.56px` 다. 8건의 dy 가 **전부 같은 값**이고, 같은 쪽 맨
위라도 `vpos=0` 인 1건과 자리차지 6건은 어긋나지 않는다.

상한은 Task #1811 과 같은 계약을 쓴다 — 쪽-상대 증거인 `vpos ≤ spacing_before` 만 싣고,
누적축 인코딩(`vpos ≫ spacing_before`)과 합성 사다리
(`TAG_IMPLEMENTATION_PROPERTY`)는 증거가 아니므로 종전대로 버린다.

> 이 문서만으로는 "저장 `vpos` 를 싣는다"와 "쪽 맨 위 문단 간격을 절반만 쓴다"를 가르지
> 못한다 — 8건이 전부 `spacing before=1000HU` 이기도 하다(반례 입력 없음, **미검증**).
> 다만 `vpos` 읽기는 값이 그대로 맞고, 같은 쪽 맨 위의 `vpos=0` 표는 움직이지 않는다.

## 3. 수정 전후 — 표 165건 전수

| | 자리차지(float) | 어울림(TAC) | 총쪽수 |
| --- | ---: | ---: | ---: |
| 수정 전 (`a6dcbc138`) | 36 / 37 (중앙값 +0.01) | 88 / 127 (−0.33) | 105 |
| 수정 후 | 36 / 37 (+0.01) | **109 / 127** (−0.43) | 105 |

1.5px 초과 이탈이 **40건 → 19건**으로 준다. +8 이 아니라 +21 인 이유는, 단 맨 위 표를 내리면
**같은 쪽의 뒤 표들도 함께 제자리로 간다**는 것이다(예: 33쪽 `pi=698`(+6.56)과
`pi=707`(+6.43)이 같은 쪽이다). 남은 19건은 이 축이 아니라 쪽 안 누적 드리프트(#4599 계열,
`dy −10.07 ~ +17.04`)와 자리차지 1건(#7203 코멘트의 잔여 ①)이다.

## 4. 시각 증적 — fresh WASM ↔ 정본 PDF, 같은 쪽·같은 영역

`scripts/visual_sweep.py --wasm-pkg`(wasm-pack `--target web` 새 빌드, Chrome
151.0.7922.34, 96dpi)로 16·33·61·90쪽을 수정 전후로 캡처했다. 16쪽 raster 에서 첫 코드
상자의 가로 괘선 행을 직접 읽으면 이렇다.

```text
  수정 전 rhwp   y = 135/136 · 206/207     (정본보다 6~7px 위)
  수정 후 rhwp   y = 142     · 213
  정본 PDF       y = 142     · 213
```

같은 쪽의 뒤 괘선(321/359 ↔ 정본 322/360)은 수정 전후 모두 1px 안이다.
`code-box-top-vs-oracle-p16.png` 가 세 출력을 같은 크롭으로 쌓은 것이다.
반례 90쪽(`vpos=0`)은 수정 전후 동일하고 정본과 일치한다.

## 5. 두 번째 문서 — `samples/footnote-01.hwp`

이 수정으로 1쪽 본문이 통째로 **20.0px** 내려간다. 그 자리가 정본이다.

```text
  정본 둘째 가로 괘선     244.69
  rhwp 표 윗변  수정 전   224.50   (−20.19)
                수정 후   244.50   (−0.19)
```

본문 줄의 잔차도 일정한 `+40.0px` 에서 `+20.0px` 로 준다(줄 상자 top ↔ glyph top 차이가
남는다). `tests/issue_598_footnote_marker_nav.rs` 의 고정 좌표 세 개는 이 이동을 따라
같은 폭으로 옮겼다 — 그 시험의 대상은 각주 마커 히트 테스트이지 줄 자리가 아니다.

## 6. 검증

- `cargo nextest run --release --no-fail-fast` — **10114 / 10114 통과**
  (좌표 갱신 전에는 `issue_598_footnote_marker_nav` 2건 실패, 갱신 후 0건)
- 래칫 4종(`body_overflow` · `text_overlap` · `off_canvas` · `overflow_cell` ·
  `oracle_page_count`) 127건 전부 통과 — 갱신한 기준값 **없음**
- Lint 묶음 전부 통과: `cargo fmt --all -- --check` · clippy native/wasm32/all-targets ·
  workspace build · `rust-test-suite-manifest --check --base-ref a6dcbc138` ·
  `rust-unit-test-tiers --check --base-ref a6dcbc138`
- red → green: `src/renderer/layout.rs` 만 되돌리면 새 핀이
  `16쪽 단 맨 위 표 윗변은 정본 142.56 근처여야 한다(수정 전 136) — got 136.04` 로 실패하고,
  반례 핀(`vpos=0`, 90쪽)은 그때도 통과한다.

## 7. 미검증

- 원인 두 후보(저장 `vpos` ↔ 쪽 맨 위 문단 간격 절반)를 가르는 입력을 찾지 못했다(§2).
- `samples/hwpctl_API_v2.4.hwpx` 쌍둥이는 재지 않았다.
- 남은 이탈 19건(쪽 안 드리프트 축, 자리차지 1건)은 이 수정의 범위가 아니다.
