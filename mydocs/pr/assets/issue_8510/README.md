# #8510 공백만 있는 데코레이션 표 host 문단 — 전쪽 silhouette 증거

- 지표: `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣), `scripts/visual_sweep.py --silhouette-only --dpi 96`.
  셀은 `측정 쪽 수 / 90% 미만 쪽 수 / 최저값`. rhwp 쪽수가 한컴보다 많은 문서는 앞 한컴 쪽수만큼만 쟀다.
- head: `upstream/devel` 4e2cc6bb36 + 이 수정, Native native-skia, fresh WASM sha256 `c4753594ce421911…`.
- base: `upstream/devel` 4e2cc6bb36, fresh WASM sha256 `f464214177567492…`.
- 기준 PDF: 각 원본의 한컴 2022 12.0.0.4547 Print(compat 2022). 글꼴: 문서별 face 디렉터리 + `ttfs/hwp`, `ttfs/windows`.
- 대상: 881 표본 dump-extents A/B 에서 바뀐 5문서 전부. 문서 번호 = planet6897/rhwp-issue-docs 이슈 번호.

| 문서 | 한컴 쪽수 | rhwp base→head | head Native | head fresh WASM | base Native | base fresh WASM |
|---|---|---|---|---|---|---|
| i222 | 5 | 5→5 | 5 / 0 / 99.85 | 5 / 0 / 99.89 | 5 / 1 / 72.54 | 5 / 1 / 72.45 |
| i701 | 33 | 33→33 | 33 / 3 / 20.45 | 33 / 3 / 20.75 | 33 / 4 / 20.45 | 33 / 4 / 20.75 |
| i477 | 29 | 29→29 | 29 / 2 / 50.22 | 29 / 2 / 50.27 | 29 / 2 / 50.22 | 29 / 2 / 50.27 |
| i566 | 332 | 336→335 | 332 / 110 / 0.00 | 332 / 110 / 0.00 | 332 / 110 / 0.00 | 332 / 110 / 0.00 |
| i844 | 168 | 183→183 | 168 / 159 / 2.17 | 168 / 159 / 2.16 | 168 / 159 / 2.17 | 168 / 159 / 2.16 |

- head 의 어느 쪽도 base 보다 0.5 점 넘게 낮지 않다(5문서 × Native·WASM 전쪽 대조). 나아진 쪽: i222 4쪽, i701 1쪽, i566 6쪽.
- i701 의 9·19·26쪽, i477 의 1·20쪽, i566·i844 의 대량 미달은 base 와 같은 값(이 변경과 무관한 기존 결함). i566·i844 는 쪽수 자체가 한컴보다 많다.

## 대표 PNG(`png/`)
- `i222_p4_*`: 공백 host 문단 0.74 의 줄이 흐름을 차지해 「□ 추진 절차」 아래 절차 표가 제자리에 놓인다. `i222_p4_base_native_review.png` 는 수정 전(표가 제목 위로 겹침, 72.54%).
