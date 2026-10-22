# #8079 굵은 글자 보통 면 전진폭 — PR 시각 증적

- code head: `b0405a2a845a93b985f4db2826c7a3c5b266d5b9` (branch `fix/8079-bold-regular-advance-devel`)
- base: `stream/devel` `2e087a5b4bc1515ad877e8020b30114639a59392`
- 규칙 1(`694bad7c1`·`48fc85794`): 굵은 TrueType 글자는 Bold 면으로 그리되 전진폭은 같은 family 보통 면 값(선언 HFT 제외).
- 규칙 2(`c842b9fa2`): 줄 끝 공백(문단 끝·명시 개행 포함)은 줄 맞춤 압축 판정에서 뺀다.
- 규칙 3(`b0405a2a8`): `고딕` HFT 프로그램의 ASCII는 반각(0.5em) 고정폭으로 잰다.
- Native: `cargo build --release --locked --features native-skia --bin rhwp`
  (`CARGO_TARGET_DIR=target/pr-review`), head binary sha256
  `b8c7828a781f742afe4b019a74247faadba26fcd41434b17422df5499e6fda02`;
  devel binary(같은 명령, `2e087a5b4`) `0b3d676b55d4f2b88568cd624e71ac1de5bdbb5c5606c0a3fcae882bbcc84cd2`.
- fresh WASM: 저장소 루트에서 `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg`,
  `pkg/rhwp_bg.wasm` = `rhwp-studio/public/rhwp_bg.wasm` sha256
  `b50e358cc5eedb495f4a1a9929ed15af2a9ef6d1d8861aa5e565f10b2f5424de`.

## 방법

`scripts/visual_sweep.py`, 96dpi, Print profile, `--rasterizer png`. compat는 기준 PDF 세대
(2020 또는 연도 없음 → `2022`, 2024 → `2024`). 판정값 `silhouette_raw_match_percent`
(내용 마스크 232 이진화, 2px 이웃 관용). 색상 경계 보정값은 진단값이며 판정에 쓰지 않았다.
글꼴: 문서별로 쓰는 face만 모은 디렉터리(각 6.2MiB 이하)를 `RHWP_FONT_PATH` 첫 항목에 두고
기본 경로(`ttfs/hwp`·`ttfs/windows`·KoPub)를 뒤에 붙였다. Native·WASM·devel 모두 같다.

```bash
RHWP_FONT_PATH=<문서별 글꼴>:<기본 경로> python3 scripts/visual_sweep.py --dpi 96 --rhwp-bin <rhwp> \
  --rasterizer png --compat <2022|2024> --hwp <입력> --pdf <기준 PDF> --pages <쪽> --key <key> --out <out>/native
RHWP_FONT_PATH=<같음> python3 scripts/visual_sweep.py --dpi 96 --rhwp-bin <head rhwp> --wasm-pkg pkg \
  --rasterizer png --compat <2022|2024> --hwp <입력> --pdf <기준 PDF> --pages <쪽> --key <key> --out <out>/wasm
```

## 대표 쪽 (raw 2px 실루엣, %)

| key | 입력 | 기준 PDF | 쪽 | devel Native | head Native | head WASM | 규칙 |
| --- | --- | --- | ---: | ---: | ---: | ---: | --- |
| oss | `samples/2026_oss_rst.hwp` | `pdf/hwpx/2026_oss_rst.pdf` | 1 | 98.71 | 99.91 | 100.00 | 1 |
| d1613 | `samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp` | `pdf/1613000-202200037-air-traffic-controller-cbta-2020.pdf` | 42 | 95.22 | 97.32 | 97.38 | 1 |
| hs2 | `samples/hwpx_sample2.hwp` | `pdf/hwpx_sample2-hwp-2024.pdf` | 2 | 99.85 | 99.98 | 99.99 | 1+2 |
| air | `samples/issue6793/1611000-201000141-small-air-transport-study.hwp` | `pdf/1611000-201000141-small-air-transport-study-2020.pdf` | 5 | 98.54 | 99.82 | 99.92 | 2 |
| h3s11 | `samples/hwp3-sample11-hwpx.hwpx` | `pdf/hwp3-sample11-hwpx-hwpx-2020.pdf` | 66 | 90.48 | 95.81 | 94.99 | 2+3 |
| synth | `samples/issue6882/synth_cell_enter_table_growth.hwp` | `pdf/synth_cell_enter_table_growth-2020.pdf` | 1 | 99.80 | 99.80 | 99.81 | 3 |
| k113 (내림) | `samples/issue6551/113424_evaluation_guideline.hwpx` | `pdf/113424_evaluation_guideline-2024.pdf` | 36 | 96.64 | 96.15 | 96.24 | 2 |
| so8 (내림) | `samples/SO-SUEOP.hwpx` | `pdf/SO-SUEOP-hwpx-2024.pdf` | 8 | 97.57 | 96.95 | 97.01 | 3 |

`pr_review_gate`([gate.json](gate.json)): 8쪽 × devel·head Native·head WASM 24회 모두 `passed`,
`page_count_mismatch=false`.

- hs2 p2: 첫 줄 `입주자 모집공고일(2026.05.29.) 현재 …`가 PDF와 겹친다.
- air p5: `◦ 해외 소형 항공운송사업 운영 실태 및 발전 가능성 조사 ` 줄의 한글 전진이 20.00px로 PDF(20.00px)와 같다.
- h3s11 p66: `A.V.Aho, P.J. Weinberger, B. Kernigham` 영문이 PDF와 같은 0.5em 간격이다.
- synth p1: `*` 전진 6.03px(PDF 6.08px, 수정 전 7.00px).
- k113 p36: 압축을 없애자 글자 간격은 PDF와 같아졌지만, 내어쓰기 문단의 줄 시작이 PDF보다 5.2px 오른쪽이다
  (기존 차이). 종전 압축이 그 차이를 일부 가렸다.
- so8 p8: 굵은·기울인·밑줄 `고딕` HFT(장평 95%, 자간 −1%) `일본 헌병`의 한글에 HFT 굵기 추가 전진(0.05em)이
  붙는데, 한컴 PDF에는 없다(12.62px). 같은 `고딕` HFT 굵게가 hwp3-sample4-hwp5 3쪽에서는 1.055em이고,
  한양견명조 굵은·기울임(child_policy 5쪽)도 추가 전진이 있다. 이 반례를 가르는 조건은 찾지 못했다.

## 바뀐 쪽 전체

samples 1,167개 SVG md5 A/B (devel → head): 쪽수가 맞는 한컴 PDF가 있는 쪽을 Native로 채점했다
([tsv/changed-devel-vs-head-native.tsv](tsv/changed-devel-vs-head-native.tsv)).

| 경로 | 채점 쪽 | 0.1%p 이상 오름 | 0.1%p 이상 내림 | 새로 90% 미만 | 새로 90% 이상 | 90% 미만 devel → head |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Native devel→head | 1,890 | 647 | 57 | 0 | 185 | 386 → 201 |
| WASM devel→규칙 1 | 1,039 | 120 | 8 | 0 | 0 | |
| WASM 규칙 1→규칙 2 | 606 | 270 | 6 | 0 | 0 | |
| WASM 규칙 2→규칙 3 | 318 | 256 | 49 | 0 | 186 | |

- WASM은 규칙별 단계로 쟀다(`tsv/step1-*`, `tsv/step2-*`, `tsv/step3-*`).
- Native에서 0.3%p 이상 내린 쪽:
  - 90% 이상에 남은 쪽은 so8 p8, k113 p36, 1341000 연구보고서 p77(97.32→97.01) 셋이다.
  - 나머지 46쪽은 hwp3-sample11 HWP5·HWPX(같은 문서 두 저장본)의 쪽이다. devel에서도 90% 미만이었다.
    영문 폭이 PDF와 같아지면서, 다른 원인의 줄 시작 차이를 넓은 영문이 가리던 효과가 사라졌다.
  - hwp3-sample11-hwpx 전체로는 90% 미만이 137쪽 → 45쪽이다.
- WASM 측정 실패 문서(devel·head 공통): `issue4090/156492236_규제샌드박스_min.hwpx`(그림 decode),
  `issue6300/trade_report_forced_break_object.hwp`(부분 글꼴).

## 정식 검사 fixture 전쪽

- 2026_oss_rst HWP·HWPX(규칙 1): 6쪽 head 최저 Native 99.91 / WASM 100.00.
- issue5169(HFT 반례): 6쪽 head 최저 Native 98.23 / WASM 99.38.
- 1611000-201000141(규칙 2): 12쪽 head 최저 Native 98.39 / WASM 98.50.
- synth_cell_enter_table_growth(규칙 3): 2쪽 head 최저 Native 99.80 / WASM 99.81.

## 정본 전수 (규칙 1 근거)

[census-malgun-bold-digit-ascii.txt](census-malgun-bold-digit-ascii.txt): 굵은 맑은 고딕을 담은 한컴 PDF 222개를 대상으로 했다.
- 방법: 글자 원점(pdfium)으로 잰 전진을 같은 줄 이웃 한글 전진으로 나눈다. 장평·자간이 상쇄된다. 그 비를 보통/Bold hmtx와 비교했다.
- 결과: 숫자 보통 9,081 · Bold 76, 영문 보통 9,788 · Bold 146. 장평·자간과 관계없이 보통 면 폭이다.

## 규칙 3 근거

한컴 Print의 `고딕` HFT Type3 프로그램은 ASCII 95자의 glyph 폭이 모두 500/1000이다.
- 확인한 PDF: `pdf/hwp3-sample11-hwpx-hwpx-2020.pdf`, `pdf/hwp3-sample11-hwp-2020.pdf` 66쪽.
- 측정법(pdfium `FPDFFont_GetGlyphWidth`) 검증: 같은 PDF의 TrueType Haansoft Batang에서는 비례 폭이 나온다.
- HWP3 원본(비HFT `고딕`)은 이미 0.5em 반각으로 재고 있었다.
