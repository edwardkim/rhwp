---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 113 — 정책연구 p160–p165 장대 표 fragment 감사

## 목적

Stage 98 원장에서 exact PDF-owner 회귀 없이 남은 두 장대 1×1 표를 최신 renderer와
한컴 PDF로 다시 판정한다.

- p160–p162: source `(pi=1737, ci=0)`
- p163–p165: source `(pi=1758, ci=0)`
- 시작 commit: `0f4c14988`

## 전수 원장

전체 215쪽 SVG와 render tree를 새로 생성하고 p160–p165 여섯 쪽을 raster 대조했다.

- PDF / SVG / render tree: `215/215/215`쪽
- 요청 / 완료 / 누락: `6/6/0`
- 여섯 쪽 PDF-only / SVG-only 문자: 모두 0
- owner-shift / owner-sequence / page-boundary: 모두 0건
- visible text excess: 0건
- table-cell text overlap / boundary: 0건
- table outside frame / table-footer: 0건

같은 `(pi, ci)`가 인접 페이지에 이어지는 네 행은 모두 의도된 장대 1×1 표 fragment다.

| 경계 | source | 현재 fragment |
|---|---|---|
| p160→p161 | `1737/0` | `h=910.9px` → `937.6px` |
| p161→p162 | `1737/0` | `h=937.6px` → `97.6px` |
| p163→p164 | `1758/0` | `h=84.3px` → `937.6px` |
| p164→p165 | `1758/0` | `h=937.6px` → `844.3px` |

## PDF 직접 판정

- p160의 표 39 시작과 p161의 continuation 첫 줄은 PDF와 같다.
- p161 마지막 `임신 검사(필요하다면)`와 p162의 `PSA(필요하다면)` 시작은 같은
  페이지를 소유하며 소실·중복이 없다.
- p162의 표 39 종료선, 후속 `(다) 전파성 질환...` 본문 시작과 하단 본문 흐름은
  PDF와 같다.
- p163의 표 40·41·42 시작, p164의 표 42 continuation, p165의 continuation 끝은
  PDF와 같은 순서와 페이지를 소유한다.
- 각 페이지의 좌·우 외곽선, 마지막 가시 행과 footer 사이 여백이 PDF와 일치한다.

가로선 후보 원장은 p161·p162·p164·p165의 상단선을 `visible_height_ratio=0.5`로
올렸다. `0.5px` stroke가 body clip 경계에 중심 정렬돼 절반이 clip 안에 놓이는
정상 continuation 형상이다. 실제 raster에서 한컴 PDF와 rhwp 양쪽 모두 같은 상단선이
보이므로 결함이 아니다.

픽셀 diff는 p161 최저 `5.28%`, p162 최고 `16.32%`다. p162의 높은 값은 본문 명조
획과 antialiasing이 많은 영역에서 커졌지만 text owner와 fragment bbox는 PDF와
일치한다.

## 결론

p160–p165의 두 장대 표는 최신 renderer에서 정상이다. 실제 결함 없이 candidate만
남은 범위이므로 코드를 변경하지 않는다. 다음 stage는 Stage 98에서 설명되지 않은
p72의 SVG-only 29자를 최신 visible owner와 PDF로 감사한다.

## 증적

- 페이지별 비교:
  `mydocs/pr/assets/task_m100_3820_stage113_policy_p160_p165_fragment_audit/compare_p160.png`
  부터 `compare_p165.png`
- [fragment 원장](../pr/assets/task_m100_3820_stage113_policy_p160_p165_fragment_audit/table-fragment-candidates.tsv)
- [문자 원장](../pr/assets/task_m100_3820_stage113_policy_p160_p165_fragment_audit/text-report.tsv)
- [가로선 clip 후보](../pr/assets/task_m100_3820_stage113_policy_p160_p165_fragment_audit/svg-table-horizontal-border-clip-candidates.tsv)
