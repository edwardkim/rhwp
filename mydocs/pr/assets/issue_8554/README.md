# #8554 검증 증거

쪽·용지 기준 자리차지 표 띠에 밀린 빈 host 줄의 전진(줄높이 + 줄간격) 수정 증거다.

## 측정 조건

- 측정: 2026-11-04 16:40+09:00(stream/devel 1924d371d6 rebase 후 재측정), `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`,
  열 `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
- head: `task/8554-paper-band-host-line`(base stream/devel 1924d371d6 + 이 수정의 `src/`, 516173a85e 의 `src/`).
- base: stream/devel 1924d371d6.
- Native: `cargo build --release --features native-skia --bin rhwp`
  - head 바이너리 sha256 `e69dbfdcd8532b7f…`, base `2d5d84f87c0695a7…`
- fresh WASM: `scripts/wasm-pack-locked.sh --target web --out-dir pkg`, `--rasterizer svg --embed-fonts=full`
  - head `rhwp_bg.wasm` sha256 `ae0904550a937369…`, base `63466acb3d7a2987…`
- 글꼴: `RHWP_FONT_PATH=<문서별 face 디렉터리>:ttfs/hwp:ttfs/windows`.
- 기준 PDF: 한/글 Print(`pdf/issue8554/2913671-appendix1-family-register-2022.pdf`, Creator `Hwp 2022 12.0.0.4547`).

## 문서

| 키 | 원본 | 한컴 쪽수 |
|---|---|---|
| i39 | `samples/issue8554/2913671-appendix1-family-register.hwp`(10k 코퍼스 `2913671_[별지 1] 별지 1(법정분가된 직계비속장남자의 직계혈족입적에 관한 사무처리지침).hwp`, docs#39, sha256 `7f62b9fea6135cbf…`) | 2 |

## 결과(쪽별 raw)

| 쪽 | head Native | head WASM | base Native | base WASM |
|---|---|---|---|---|
| 1 | 95.98 | 99.44 | 47.45 | 47.26 |
| 2 | 100.00 | 100.00 | 100.00 | 100.00 |

- 1쪽 「2.」 줄 위끝: base 315.3 px → head 332.6 px(한컴 Print ≈ 332.4 px). 쪽수는 한컴과 같다.
- rebase 전(base 7d06d5c706) head Native 1쪽은 99.51% 였다. rebase 후 95.98% 로 내려간 차이는 이 수정과 무관한
  stream/devel #8565(Native 웹폰트 자동 공급) 이후 한양신명조 슬롯이 한글 전용 subset 웹폰트(NotoSansKR-Regular,
  CJK 통합 한자 0자)에 매핑되고 글리프 대체(`glyphFallbacks`)가 비어 「李乙女」·「金丙女」 한자가 대체 없이
  그려지지 않는 차이다(#8590). base 1924d371d6 에서도 같은 한자가 빠지며(`RHWP_FONT_PATH` 유무와 무관),
  줄·표 괘선 위치는 rebase 전 head 와 같다. fresh WASM 은 99.44% 그대로다.

## 파일

- `{head,base}/{native,wasm}/i39_silhouette.tsv`: sweep TSV 원본(바이트 그대로).
- `png/i39_p1_head_native_review.png`, `png/i39_p1_head_native_overlay.png`, `png/i39_p1_head_wasm_review.png`: head 1쪽.
- `png/i39_p1_base_native_review.png`: base 1쪽(「2.」 이하가 17.3 px 위).
