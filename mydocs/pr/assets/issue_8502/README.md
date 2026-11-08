# #8502 저장 TAC 표 줄의 후행 간격 닫힘 — 전쪽 silhouette 증거

- 지표: `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣), `scripts/visual_sweep.py --silhouette-only --dpi 96`.
  셀은 `측정 쪽 수 / 90% 미만 쪽 수 / 최저값`.
- head: `upstream/devel` 4e2cc6bb36 + 이 수정(`src/` = fix 커밋), Native native-skia, fresh WASM `rhwp_bg.wasm` sha256 `ad42b69c878b7183…`.
- base: `upstream/devel` 4e2cc6bb36, fresh WASM sha256 `7e2aac2a753852b4…`.
- 기준 PDF: 각 원본의 한컴 2022 12.0.0.4547 Print(compat 2022). 글꼴: 문서별 face 디렉터리 + `ttfs/hwp`, `ttfs/windows`.
- 대상: 881 표본 dump-extents A/B 에서 바뀐 3문서 전부(쪽수 변화 없음). 문서 번호 = planet6897/rhwp-issue-docs 이슈 번호.

| 문서 | 한컴 쪽수 | head Native | head fresh WASM | base Native | base fresh WASM |
|---|---|---|---|---|---|
| i1071 | 5 | 5 / 0 / 99.68 | 5 / 0 / 99.68 | 5 / 2 / 37.27 | 5 / 2 / 38.36 |
| i1004 | 3 | 3 / 0 / 92.93 | 3 / 0 / 92.11 | 3 / 1 / 54.80 | 3 / 1 / 54.36 |
| i923 | 2 | 2 / 2 / 50.55 | 2 / 2 / 51.91 | 2 / 2 / 50.55 | 2 / 2 / 51.91 |

i923 1쪽(50.55%)은 base 와 같은 기존 결함(표지 두 줄 제목 표가 한 줄로 눌림, #8506)이다. 2쪽은 68.82 → 75.45(Native)로 개선됐으나 90% 미만이 남아 #8506 에 함께 기록했다.

## 대표 PNG(`png/`)
- `i1071_p3_*`: 2쪽 문단 10 TAC 표 뒤 간격이 맞아 3쪽 첫 줄부터 한컴과 같다. `i1071_p3_base_native_review.png` 는 수정 전(앞 쪽에서 9px 당겨진 결과 한 줄이 2쪽에 남아 37.27%).
