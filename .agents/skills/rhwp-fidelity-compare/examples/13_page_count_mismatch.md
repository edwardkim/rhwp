# 예제 — 쪽수 불일치 후보

이슈 #5329. 실 에이전트 경로. gym 아님.

## 카탈로그

```
source	pages	delta_vs_reference	scope	note
reference_pdf	35	0	full PDF	comparison baseline
rhwp_svg	37	2	full export	page-count difference is a candidate, not a global-break fix
rhwp_render_tree	37	2	full render tree	page-count difference is a candidate, not a global-break fix
```

전역 page-break 패치를 열지 않는다. owner 원장으로 창을 고른다.
맞춰찍기 축소 오라클인지도 본다.

관련: `references/15_page_count_mismatch.md`.
픽스처: `fixtures/tsv/page_count_drift.tsv`.
정지 F11.
