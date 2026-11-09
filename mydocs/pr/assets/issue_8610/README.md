# #8610 검증 증거

칸 안의 빈 host 문단이 비-TAC 글 위아래(TopAndBottom) 표를 품어 흐름을 표 바깥 상자 하단까지
밀었을 때, 뒤 문단의 문단 앞 간격을 표 띠에 흡수해(한/글 규칙
`max(host 글줄 끝 + 앞 간격, 표 점유 하단)`) 이중 가산을 없앤 수정의 증거다.

## 측정 조건

- 측정: 2026-11-05 07:20~08:30+09:00, `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`,
  열 `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣, `threshold_boundary_color_support_v1`,
  color tolerance 32).
- head: `task/8610-top-bottom-band-absorbs-spacing`(base stream/devel e96b500e3e + 이 수정의 `src/`).
- base: stream/devel e96b500e3e.
- Native: `cargo build --release --locked --features native-skia --bin rhwp`
  - head 바이너리 sha256 `a67acae397b3bf43…`, base `236a10d39c21412c…`
- fresh WASM: `scripts/wasm-pack-locked.sh --target web --out-dir pkg`, `--rasterizer svg --embed-fonts=full --font-path <face 폴더>`
  - head `rhwp_bg.wasm` sha256 `dc1dcd1d492b44fd…`, base `77612943385dc634…`
- 글꼴: `RHWP_FONT_PATH=<문서별 face 폴더>:ttfs/hwp:ttfs/windows`. 25128·36849 face 폴더 = 휴먼명조(HMKMM)·
  HY헤드라인M·HY견고딕·HY울릉도M(36849 는 KoPub 돋움 Light·Medium 추가). 한/글 Print 가 쓴 HY중고딕·
  Times New Roman 은 로컬에 없어 대체된다.

## 문서

기준 PDF 는 모두 한/글 Print(Creator `Hwp 2022 12.0.0.4547`, Producer `Hancom PDF 1.3.0.547`).

| 키 | 원본 | 한컴 쪽수 | rhwp 쪽수 |
|---|---|---|---|
| i318 | `samples/issue8610/25128-nars-press-release-20190517.hwp`(10k `nars_downloads/unknown-year/25128_(보도자료_20190517)…`, sha256 `090b0ce73b60f972…`), PDF `pdf/issue8610/25128-nars-press-release-20190517-2022.pdf`(`a0abb0f82eb6ac78…`) | 1 | 1 |
| i319 | `samples/issue8610/36849-nars-press-release-20211116.hwp`(10k `nars_downloads/unknown-year/36849_(보도자료_20211116)…`, `c41571e0bda978ce…`), PDF `pdf/issue8610/36849-nars-press-release-20211116-2022.pdf`(`4eb12aebb771a714…`) | 3 | 3 |
| i321 | 10k `nars_downloads/unknown-year/41449_(보도자료_20230220)…`(`d48a8c21824cd07d…`, 저장소 미포함) | 3 | 3 |
| i323 | 10k `nars_downloads/unknown-year/43944_(보도자료_20240117)…`(`aaf71b1625fb3ba0…`, 저장소 미포함) | 3 | 3 |
| i325 | 10k `nars_downloads/unknown-year/47288_(보도자료_20250417)…`(`91fd7ed3be074ab9…`, 저장소 미포함) | 2 | 2 |

## 결과 — 대상 문서 전쪽 raw

| 문서 | 쪽 | head Native | head WASM | base Native | base WASM |
|---|---|---|---|---|---|
| i318 | 1 | 99.95 | 99.43 | 55.42 | 54.09 |
| i319 | 1 | 98.51 | 98.62 | 52.73 | 52.09 |
| i319 | 2 | 97.76 | 97.97 | 97.76 | 97.97 |
| i319 | 3 | 96.37 | 96.78 | 96.37 | 96.78 |
| i321 | 전쪽 최저 | 97.65 | 97.32 | 57.08 | 54.63 |
| i323 | 전쪽 최저 | 98.79 | 98.79 | 57.53 | 53.80 |
| i325 | 전쪽 최저 | 99.38 | 99.40 | 70.75 | 68.01 |

쪽별 값은 각 TSV 원본에 있다. 다섯 문서 모두 전쪽 90% 이상, 쪽수 한컴과 같음.

## 결과 — A/B 에서 바뀐 다른 문서(바뀐 쪽만 측정)

881 표본·10k 코퍼스 전수 `dump-extents` A/B 에서 바뀐 문서는 위 다섯 건 외에 아래 여섯 건이다.
쪽별 diff 로 바뀐 쪽을 고르고 그 쪽만 `--pages N-N` 으로 측정했다(i737 520쪽 전쪽 WASM 은 중간 SVG 가 45G 를 넘어 디스크를 채웠다).

| 문서 | 바뀐 쪽 | head Native | head WASM | base Native | base WASM | 비고 |
|---|---|---|---|---|---|---|
| i322 (10k `nars_downloads/…/43496_…`) | 1 | 99.66 | 99.47 | 55.18 | 50.99 | 같은 서식. 쪽수 rhwp 3 / 한컴 2 는 base 와 같은 별도 원인(docs#322) |
| i375 (10k `opinion_downloads/행정안전부/65423_…`) | 2 | 43.11 | 45.82 | 42.36 | 45.11 | 중첩 표 → 「○」 줄 간격 한컴 13 px, base 18 px, head 12 px. 쪽 미달은 기존 |
| i683 (10k `prism_downloads/기후에너지환경부/1480000-201300260_…`) | 4 | 94.31 | 95.77 | 66.63 | 65.96 | |
| i683 | 56 | 15.48 | 15.00 | 15.49 | 15.01 | rhwp 99쪽 / 한컴 98쪽(기존) — 한컴 55쪽과 대응, 간격은 한컴과 같음 |
| i737 (10k `prism_downloads/법제처/1170000-201700002_…`) | 233 | 18.63 | 19.45 | 17.19 | 17.72 | rhwp 520쪽 / 한컴 519쪽(기존) — 한컴 232쪽과 대응, 간격은 한컴과 같음 |
| 156760012 고용노동부 보도자료(HWPX, PDF 없음) | — | — | — | — | — | 바뀐 문단 y − hu(vpos) 가 앞 문단과 같은 160.9 px(base 164.99) — 저장 사다리와 일치 |
| 1311000-201100031 KAF 회의자료(PDF 없음) | — | — | — | — | — | 중첩 표 뒤 문단이 표 하단 + 1.9 px(= 바깥여백 140 HU), base 3.9 px |

## 편람 게이트

`scripts/visual_sweep.py --handbook-gate --rhwp-bin <head Native>`: **passed**, 383쪽 최저 99.96%, 100.00% 370쪽, 회귀 없음.

## 파일

- `{head,base}/{native,wasm}/{i318,i319,i321,i323,i325}_silhouette.tsv`: 전쪽 sweep TSV 원본(바이트 그대로).
- `{head,base}/{native,wasm}/i<문서>_p<쪽>_silhouette.tsv`: 위 부수 문서의 바뀐 쪽 TSV.
- `png/{i318,i319}_p1_head_native_review.png`·`_head_native_overlay.png`·`_head_wasm_review.png`·`_base_native_review.png`.
