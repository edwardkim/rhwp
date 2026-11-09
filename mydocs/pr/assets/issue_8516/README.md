# #8516 검증 증거

글자 없는 host 에 공동 앵커된, 가로로 나란한 양수 오프셋 자리차지 표의 배치 수정 증거다.

## 측정 조건

- 측정: 2026-11-04 16:30+09:00(stream/devel 1924d371d6 rebase 후 재측정), `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`,
  열 `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
- head: `task/8516-textless-coanchored-floats` 4132222e73 의 `src/`(base stream/devel 1924d371d6 위 rebase).
- base: stream/devel 1924d371d6.
- Native: `cargo build --release --features native-skia --bin rhwp`
  - head 바이너리 sha256 `8d4bccfdb23d683a…`, base `2d5d84f87c0695a7…`
- fresh WASM: `scripts/wasm-pack-locked.sh --target web --out-dir pkg`, `--rasterizer svg --embed-fonts=full`
  - head `rhwp_bg.wasm` sha256 `383b135b753b94e5…`, base `63466acb3d7a2987…`
- 글꼴: `RHWP_FONT_PATH=<문서별 face 디렉터리>:ttfs/hwp:ttfs/windows`(문서별 face 디렉터리는 입력 문서가 쓰는 face 만 모은 것).
- 기준 PDF: 한/글 2020 Print(`hancom_print_2020.pdf`, Creator `Hwp 2020`).

## 문서

| 키 | 원본 | 한컴 쪽수 | 비고 |
|---|---|---|---|
| d8516 | `samples/issue8516/1051000-202000124-evidence-policy-cover.hwp`(sha256 `c6165192bc6d…`) | 4 | 회귀 고정본. 기준 PDF `pdf/issue8516/1051000-202000124-evidence-policy-cover-2020.pdf`(sha256 `25cc0e909e74…`) |
| i654 | 10k 코퍼스 1051000-202000124 재정성과 향상을 위한 증거기반 정책평가 연구(docs#654) | 209 | 고정본의 원본 |
| i650 | 1051000-201200103 복권의 사행성 수준평가 및 대응정책의 방향성 연구(docs#650) | 139 | 같은 표지 구조 |
| i653 | 1051000-201800093 재정지출의 경제적 효과분석 및 재정 건전성 검정(docs#653) | 158 | 같은 표지 구조 |

## 결과(쪽 수 / 90% 미만 / 최저)

| 문서 | head Native | head WASM | base Native | base WASM |
|---|---|---|---|---|
| d8516 | 4 / 0 / 99.04 | 4 / 0 / 97.06 | 4 / 1 / 18.60 | 4 / 1 / 17.95 |
| i654 | 209 / 19 / 27.42 | 209 / 19 / 27.82 | 209 / 20 / 18.76 | 209 / 20 / 18.24 |
| i650 | 139 / 9 / 36.98 | 139 / 6 / 36.85 | 139 / 10 / 16.08 | 139 / 7 / 15.65 |
| i653 | 158 / 16 / 9.12 | 158 / 16 / 8.77 | 158 / 17 / 9.12 | 158 / 17 / 8.77 |

- 네 문서 모두 head 와 base 가 다른 쪽은 1쪽뿐이다(Native·WASM 전쪽 대조). 1쪽은 모두 97.68% 이상으로 개선:
  d8516 99.94/99.93, i654 99.92/99.85, i650 99.96/99.91, i653 99.62/97.68(Native/WASM).
- 원본 세 문서의 나머지 90% 미만 쪽은 base 와 같은 값의 기존 결함이며 이 수정의 범위 밖이다.
- rebase 전(base 7d06d5c706) 대비 Native 일부 쪽 값의 이동(i650 Native 90% 미만 6→9쪽 등)은 base 1924d371d6 에서도
  같은 값으로 나타난다 — stream/devel #8565(Native 웹폰트 자동 공급)의 글꼴 공급 변화이며 이 수정과 무관하다.
- rhwp 쪽수는 네 문서 모두 한컴과 같다.

## 파일

- `{head,base}/{native,wasm}/<키>_silhouette.tsv`: sweep TSV 원본(바이트 그대로).
- `png/d8516_p1_head_native_review.png`, `png/d8516_p1_head_native_overlay.png`, `png/d8516_p1_head_wasm_review.png`:
  head 1쪽.
- `png/d8516_p1_base_native_review.png`: base 1쪽(오른쪽 표와 뒤 본문이 아래로 밀림).
