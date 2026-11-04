# #8079 굵은 글자 보통 면 전진폭 — PR 시각 증적

- code head: `bba7229a1b` (branch `fix/8079-bold-regular-advance-devel`)
- base: `stream/devel` `f291d772a1`
- 규칙 1(`be034df9b6`·`7d78d7ea6b`·`bba7229a1b`): 굵은 TrueType 글자는 Bold 면으로 그리되 전진폭은
  같은 family 보통 면 값(선언 HFT 제외).
- 이전 head 의 규칙 2(줄 끝 공백 압축)·규칙 3(`고딕` HFT ASCII 반각)은 뺐다. devel 이 같은 결함을 이미
  고쳤고, 두 규칙의 검사는 devel `f291d772a1` 에서 이 PR 없이 통과한다.
  - 규칙 2: #8188 `17225375ff` 의 `paragraph_layout/trailing_space_fit.rs`(`tail_only_overflow`).
  - 규칙 3: #8191 `6097253468` 의 `font_metrics_overlays.rs::declared_original_glyph_advance_em`
    (`고딕`·`명조`·`산세리프` 선언 HFT 의 ASCII 0.5em).
- Native: `cargo build --release --locked --features native-skia --bin rhwp`
  (`--target-dir target/pr-review`), head binary sha256
  `ff5a59258798da8b756e42f66762cdaeaa4c8d32d3dab102a201b20ac1a39507`,
  devel binary(같은 명령, `f291d772a1`) `b70945001cc946c277a835ed1b0e4b944106f31644c4b48d4dea7e1a90600046`.
- fresh WASM: 저장소 루트에서 `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg`.
  head `pkg/rhwp_bg.wasm` = `rhwp-studio/public/rhwp_bg.wasm` sha256
  `0556961a30acb3ceea54a37db1469a271fa0fccede411cf5184ae085fb51d80a`,
  devel `4aab4720df9490e433e6c40e667c6bc90df1d8c128f78caeff824aec4b29f033`.

## 방법

`scripts/visual_sweep.py`, 96dpi, Print profile, `--rasterizer png`. compat는 기준 PDF 세대
(2020 또는 연도 없음 → `2022`, 2024 → `2024`). 판정값 `silhouette_raw_match_percent`
(내용 마스크 232 이진화, 2px 이웃 관용). 색상 경계 보정값은 진단값이며 판정에 쓰지 않았다.
글꼴: 문서 SVG 가 쓰는 face 의 파일을 모은 디렉터리(64MiB 미만, 글자 run 의 첫 family 우선)를
`RHWP_FONT_PATH` 첫 항목에 두고 기본 경로(`ttfs/hwp`·`ttfs/windows`·KoPub)를 뒤에 붙였다.
Native·WASM·devel 모두 같다.

```bash
RHWP_FONT_PATH=<문서별 글꼴>:<기본 경로> python3 scripts/visual_sweep.py --dpi 96 --rhwp-bin <rhwp> \
  --rasterizer png --compat <2022|2024> --hwp <입력> --pdf <기준 PDF> --pages <쪽> --key <key> --out <out>/native
RHWP_FONT_PATH=<같음> python3 scripts/visual_sweep.py --dpi 96 --rhwp-bin <head rhwp> --wasm-pkg pkg \
  --rasterizer png --compat <2022|2024> --hwp <입력> --pdf <기준 PDF> --pages <쪽> --key <key> --out <out>/wasm
```

## 대표 쪽 (raw 2px 실루엣, %)

| key | 입력 | 기준 PDF | 쪽 | devel N | head N | devel W | head W |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| oss | `samples/2026_oss_rst.hwp` | `pdf/hwpx/2026_oss_rst.pdf` | 1 | 98.70 | 99.93 | 98.73 | 100.00 |
| d1613 | `samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp` | `pdf/issue6764/1613000-202200037-air-traffic-controller-cbta-hwp-2020.pdf` | 42 | 96.09 | 98.23 | 95.97 | 98.21 |
| hs2 | `samples/hwpx_sample2.hwp` | `pdf/hwpx_sample2-hwp-2024.pdf` | 2 | 99.45 | 99.84 | 99.44 | 99.82 |
| k113 | `samples/issue6551/113424_evaluation_guideline.hwpx` | `pdf/113424_evaluation_guideline-2024.pdf` | 5 | 99.19 | 99.35 | 98.87 | 99.06 |

`pr_review_gate`([gate.json](gate.json)): 4쪽 × devel Native·head Native·head WASM 12회 모두 `passed`,
`page_count_mismatch=false`. 그림 이름은 `devel-*`(devel Native), `native-*`, `wasm-*`(head)이다.

- oss p1: 본문 속 굵은 맑은 고딕 숫자·영문(`~2026. 8. 27.(목) 18:00`, `HWP(HWPX)`, `10pt`)과 그 뒤 글자가
  PDF 와 같은 위치다(devel 은 Bold 폭으로 뒤가 밀린다).
- d1613 p42: 굵은 머리말 `제2장 ICAO가 제시하는 CBTA의 …` 줄 길이가 PDF 와 같아졌다(devel 은 겹침 윤곽).
- hs2 p2: 첫 줄 `입주자 모집공고일(2026.05.29.) 현재 …`의 줄 끝이 PDF 와 같다.
- 본문 명조류의 굵기·글자 모양 차이는 devel 과 같은 기존 글꼴 차이다.

## 바뀐 쪽 전체

samples 1,378개 SVG md5 A/B (devel `f291d772a1` → head): 163문서 1,199쪽이 바뀌었고 쪽수가 바뀐 문서는 없다.
쪽수가 맞는 한컴 PDF 가 있는 115문서 1,042쪽을 Native·WASM 의 devel·head 로 채점했다
([tsv/changed-devel-vs-head.tsv](tsv/changed-devel-vs-head.tsv)). 48문서는 쪽수가 맞는 PDF 가 없다.

| 경로 | 채점 쪽 | 0.1%p 이상 오름 | 0.1%p 이상 내림 | 새로 90% 미만 | 90% 미만 devel → head |
| --- | ---: | ---: | ---: | ---: | --- |
| Native | 1,042 | 140 | 1 | 0 | 39 → 39 |
| fresh WASM | 1,042 | 136 | 3 | 0 | 38 → 38 |

- 0.1%p 이상 내린 쪽은 모두 devel 부터 90% 미만인 쪽이고 폭은 −0.14%p 이하다:
  `issue6879/156767332-broadcast-revenue-attachment.hwp` p8(N 43.01→42.87, W 43.03→42.91)·p4(W 72.74→72.62),
  `issue6044/156513948.hwpx` p30(W 85.62→85.50).
- 이전 head 에서 규칙 2·3 으로 바뀌던 1611000-201000141, synth_cell_enter_table_growth, hwp3-sample11,
  SO-SUEOP 은 이제 SVG 가 devel 과 같다.
- 80168, 1342000 은 SVG 가 devel 과 같다. 1130000 은 p3 만 바뀐다(N 94.55→94.55, W 94.63→94.63).

## 정식 검사 fixture 전쪽

`tsv/*full-{native,wasm}.tsv`, head.
- 2026_oss_rst HWP·HWPX: 6쪽씩, 최저 Native 99.92 / WASM 100.00.
- issue5169(HFT 반례, `pdf/issue7674/issue5169_viewtext_changetracking-2020.pdf`): 6쪽, 최저 Native 99.39 / WASM 99.48.

## 정본 전수 (규칙 1 근거)

[census-malgun-bold-digit-ascii.txt](census-malgun-bold-digit-ascii.txt): 굵은 맑은 고딕을 담은 한컴 PDF 222개를 대상으로 했다.
- 방법: 글자 원점(pdfium)으로 잰 전진을 같은 줄 이웃 한글 전진으로 나눈다. 장평·자간이 상쇄된다. 그 비를 보통/Bold hmtx와 비교했다.
- 결과: 숫자 보통 9,081 · Bold 76, 영문 보통 9,788 · Bold 146. 장평·자간과 관계없이 보통 면 폭이다.
