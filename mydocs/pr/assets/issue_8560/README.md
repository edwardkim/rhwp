# #8560 검증 증거

글자 없는 빈 host 의 문단 기준 자리차지 표 뒤 저장 host 줄이, 다음 문단도 글자 없는 자리차지 표
host 일 때 줄높이와 줄간격을 차지하도록 한 수정의 증거다.

## 측정 조건

- 측정: 2026-11-04 07:30+09:00, `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`,
  열 `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣, `threshold_boundary_color_support_v1`).
- head: `task/8560-textless-float-host-successor`(base stream/devel 7d06d5c706 + 이 수정의 `src/`, f3148d5247).
- base: stream/devel 7d06d5c706.
- Native: `cargo build --release --features native-skia --bin rhwp`
  - head 바이너리 sha256 `d2151f1809b52f06…`, base `f7f203ce21ca05ef…`
- fresh WASM: `scripts/wasm-pack-locked.sh --target web --out-dir pkg`, `--rasterizer svg --embed-fonts=full`
  - head `rhwp_bg.wasm` sha256 `573c471712e86e2e…`, base `0ebcb9230aa1fbc4…`
- 글꼴: `RHWP_FONT_PATH=<문서별 face 디렉터리>:ttfs/hwp:ttfs/windows`.
- 기준 PDF: 한/글 Print(`pdf/issue8560/21138757-agenda-cover-form-2020.pdf`, Creator `Hwp 2020 0.0.0.0`,
  Producer `Hancom PDF 1.3.0.550`, sha256 `e1a8e2d86382e572…`).

## 문서

| 키 | 원본 | 한컴 쪽수 |
|---|---|---|
| i470 | `samples/issue8560/21138757-agenda-cover-form.hwp`(10k 코퍼스 `21138757_[별지 제1호서식] 안건표지 표기방법(아산시 조례ㆍ규칙심의회 운영 규칙).hwp`, docs#470, sha256 `b3a068a6ee73a14f…`) | 1 |

## 결과(쪽별 raw)

| 쪽 | head Native | head WASM | base Native | base WASM |
|---|---|---|---|---|
| 1 | 93.75 | 93.40 | 49.72 | 49.90 |

- 문단 0.9 표(「・○○○ 조례…」) 윗변: base 는 문단 0.8 띠 아래끝 + 26.25 px, head 는 + 49.72 px
  (host 줄 1100 + 줄간격 660 + 오프셋 1685 + 바깥 여백 284 HU). 뒤 두 표도 같이 23.5 px 내려가
  한컴 Print 의 괘선과 겹친다. 쪽수는 한컴과 같다.
- 남은 차이: 「○○○」·「・」 글자 폭(문서 face 의 기호 글리프 폭). 표 괘선·문단 시작 위치는 일치.

## 파일

- `{head,base}/{native,wasm}/i470_silhouette.tsv`: sweep TSV 원본(바이트 그대로).
- `png/i470_p1_head_native_review.png`, `png/i470_p1_head_native_overlay.png`, `png/i470_p1_head_wasm_review.png`: head 1쪽.
- `png/i470_p1_base_native_review.png`: base 1쪽(문단 0.9 이하 표가 23.5 px 위).
