---
kind: implementation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 85 — native short-child content-box 조판 폭

## 독립 기준과 HWPCTRL 경계

- 대상 HWP: `samples/76076_regulatory_analysis.hwp`
- 독립 Oracle: `samples/issue1891/76076_regulatory_analysis-2024.pdf`의 p81--p82
- HWPCTRL 문서: `mydocs/manual/webhwpctrl_compat_development.md`

HWPCTRL API ledger와 Windows COM Oracle은 HWPCTRL fixture/API 호환성에만 사용한다.
이번 변경은 기존 HWP의 PDF line-wrap 및 table-fragment owner 보정이므로 한컴 2024 PDF를
유일한 시각 Oracle으로 사용한다. macOS WASM 결과나 HWPCTRL fixture를 PDF 정답으로 바꾸지 않는다.

## Stage 84 결과와 이번 범위

Stage 84의 native HWP5 `TopAndBottom + Para + RowBreak` 마지막 1×1 child 경로는 p81→p82의
첫 줄 중복을 제거했다. 하지만 직접 `fidelity_compare --text-only` 대조에서 p81은 `… 등의 사`,
p82는 `고를 예방…`으로 한 글자씩 갈렸다. 기준 PDF는 p81 `… 등의 사고`, p82 `를 예방…`이다.

같은 child의 조판 진단은 다음을 확정한다.

| horizontal content width | 첫 줄 | 판정 |
| ---: | --- | --- |
| 487.63px | `… 등의 사` | 현재의 오답 |
| 509.93px | `… 등의 사고` | 한컴 PDF 줄 경계와 일치 |

487.63px는 저장 child width를 parent width로 투영한 뒤에도 #2308 p34 보호용
`cell.padding=(510,510,141,141)`의 좌우 510HU를 계속 적용해 생긴다. p81의 child table은
`padding=(0,0,141,141)`, `aim=false`이므로 native short-child owner contract에서는 그 saved
small margin이 아닌 table content box를 써야 한다.

이번 구현은 `RenderNormalizationOverlay`가 이미 판정한 short-child projection에만
`use_owner_content_box` flag를 보관한다. 그 flag가 있는 table은 `layout_table_cells()`와
`nested_table_mixed_fragment_heights()`에서 saved small cell margin compatibility를 끈다.
일반 non-TAC nested table, 특히 76076 p34는 flag가 없으므로 기존 510HU 여백과 우측 border
안쪽 text contract를 유지한다. 전역 한양중고딕 U+0020 metric, 일반 table padding, page budget,
orphan threshold는 변경하지 않는다.

## 검증 순서

1. `issue_2430_cell_rewrap_threshold`를 첫 Cargo gate로 재실행한다.
2. p81/p82 text-owner direct PDF 대조에서 `사고`가 p81에, `를 예방`이 p82에 남는지 확인한다.
3. #2308 p34 saved-margin focused regression과 Stage 84 rowbreak regression을 실행한다.
4. 통과 후에만 targeted fixture assertion·PDF 증적·fmt/clippy 및 전체 release-test gate를 수행한다.

## 관측 기반 scope 축소와 현재 결과

처음의 “short parent” 가설은 parent/child height 비율만으로는 충분하지 않았다. overlay
candidate 진단에서 p33의 `pi=511`도 같은 5×2, 3문단, near-fit 구조로 폭 투영돼, #2308 PDF
geometry pin을 `y=351.1px`에서 `377.8px`로 옮기는 반례가 나왔다. `pi=842`와의 실제 차이는
stored table viewport다.

| owner | parent common height | child common height | 투영 |
| --- | ---: | ---: | --- |
| p33 `pi=511` | 24,456HU | 14,406HU | 금지 |
| p34 `pi=336` | 19,400HU | 9,350HU | 금지 |
| p81 `pi=842` | 8,304HU | 12,846HU | 허용 |

따라서 render-only projection의 최종 source gate는 `child.common.height >
owner.common.height`도 요구한다. `pi=842`만 parent cell width 및 owner content box를 사용하고,
p33/p34 일반 non-TAC nested table은 stored width·510HU horizontal margin을 계속 유지한다.

현재 direct oracle 재대조는 다음과 같다.

```text
fidelity_compare p81--p82, 한컴 2024 기준 PDF
p81 text owner: reference_only=0, svg_only=0
p82 text owner: reference_only=0, svg_only=0
p81 raster diff=17.34%, p82 raster diff=3.91%
```

즉 이번 change는 목표 source owner (`p81 … 등의 사고` / `p82 를 예방…`)를 회복했지만,
raster fidelity 전체를 완료로 선언하지 않는다. p81의 17.34%에는 이 short-child owner와 별개인
기존 font/vertical table geometry 차이가 포함되며 다음 stage에서 분리한다.

또한 #2308 p33 geometry test는 projection을 환경상으로 억제해도 같은 `y=377.8px`으로
실패했다. 이는 현재 worktree의 선행 height/layout 변경 baseline이며, 이번 projection의 원인이
아니다. assertion을 갱신하지 않았고 전체 regression은 아직 통과 상태가 아니다.
