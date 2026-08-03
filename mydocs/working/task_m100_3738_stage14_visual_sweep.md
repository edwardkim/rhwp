---
kind: verification
status: completed
canonical: mydocs/manual/verification/visual_sweep_guide.md
last_verified: 2026-08-02
---

# Task #3738 Stage 14 — HWP p58–p59 existing-footnote reset-tail visual sweep

## 입력과 방법

- 입력: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 정답지: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- renderer: `target/review-planet6897-20260802/release-test/rhwp`
- 방법: `scripts/visual_sweep.py --pages 58-59 --dpi 144`

선택한 두 페이지는 모두 완료됐고 missing page와 structural flag는 없다. SVG/render-tree 전체 산출은
222쪽이다. 기준 PDF는 215쪽이므로 이 결과는 p58–p59 경계 회귀의 증거이며 전체 pagination 완료 증거는
아니다.

## 판정

| 쪽 | 확인 결과 |
| --- | --- |
| p58 | `캐나다의 …` 뒤 `호주 정부의 …`, `Medical Research Council … 치료와`까지의 stored reset 전 세 줄이 각주 70 위에 있다. 각주 separator·본문·footer가 겹치지 않는다. |
| p59 | `독립적이며 적절한 지식과 기술 …`부터 재개한다. p58에 남겨야 할 두 줄이 중복되지 않는다. |

overlay 평균 pixel match는 90.80747%, 최저는 90.44761%, 평균 ink match는 10.74416%다. 폰트 raster 차이는
별도이며, review PNG의 문장 경계와 footnote non-overlap으로 판정했다.

## 증적

compare/overlay/review PNG 여섯 장은
[`mydocs/pr/assets/pr_3740_issue3738_stage14/`](../pr/assets/pr_3740_issue3738_stage14/)에 보관한다. 원본
HWP/HWPX/PDF의 canonical 보관 위치는 [`pdf/pr3740/README.md`](../../pdf/pr3740/README.md)다.
