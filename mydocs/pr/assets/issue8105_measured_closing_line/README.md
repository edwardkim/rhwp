# #8105 실측 쪽 끝 — PR 시각 증적

- code head: `8053bc6ea84036b8bdc50ae57dd7c9f7414e2a54` (branch `fix/8105-measured-closing-line`)
- base: `stream/devel` `2e087a5b4bc1515ad877e8020b30114639a59392`
- Native: `cargo build --release --locked --features native-skia --bin rhwp`
  (`CARGO_TARGET_DIR=target/pr-review`), binary sha256 `102aba138ac61037279a524197ef10c683ea9e92749dacc68a1c348555c8f6a3`
- fresh WASM: `scripts/wasm-pack-locked.sh --target web --out-dir pkg` (같은 head),
  `pkg/rhwp_bg.wasm` = `rhwp-studio/public/rhwp_bg.wasm` sha256
  `9cfa18d6c7fcffd9a5bf18494f1352e1225ab0190de5a692542ef7da9e9fb306`
- devel 기준: 같은 명령으로 `2e087a5b4`에서 만든 Native binary (Native 전쪽 TSV만; devel WASM TSV는 산출하지 않음)

## 방법

`scripts/visual_sweep.py`, 96dpi, Print profile, `--rasterizer png`. compat는 기준 PDF 세대
(2020 Print → `2022`, 2024 Print → `2024`). 판정값은 `silhouette_raw_match_percent`
(내용 마스크 232 이진화, 2px 이웃 관용). 색상 경계 보정 방법 `threshold_boundary_color_support_v1`
(흰 배경 244 이상 제외, RGB 차 32 이내)은 TSV 2열의 진단값이며 판정에 쓰지 않았다.

글꼴: Native는 문서별로 쓰는 face만 모은 디렉터리(각 1–7MB, 64MiB 적재 한도 아래)를
`RHWP_FONT_PATH`로 공급했다. devel 기준도 같은 디렉터리다. WASM은 기본 font catalog
(`ttfs/hwp`·`ttfs/windows`·KoPub 경로)를 썼다.

전쪽 TSV: `--silhouette-only`, 쪽 선택 없이 문서 전체. 대표 PNG: 일반 모드 `--pages`.

```bash
RHWP_FONT_PATH=<문서별 글꼴> python3 scripts/visual_sweep.py --dpi 96 --rhwp-bin <head rhwp> \
  --rasterizer png --compat <2022|2024> --hwp <입력> --pdf <기준 PDF> --pages <쪽> --key <key> --out <out>/native
python3 scripts/visual_sweep.py --dpi 96 --rhwp-bin <head rhwp> --wasm-pkg pkg \
  --compat <2022|2024> --hwp <입력> --pdf <기준 PDF> --pages <쪽> --key <key> --out <out>/wasm
```

## 대표 쪽 (raw 2px 실루엣, %)

| key | 입력 | 기준 PDF | 쪽 | devel Native | head Native | head WASM |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| k1130 | `samples/issue5966/1130000-202100008_franchise_review_report.hwp` | `pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue5966_franchise_review_report_1130000-202100008_franchise_review_report-2020.pdf` | 51 | 32.11 | 99.09 | 99.30 |
| k86712 | `samples/86712_regulatory_analysis.hwp` | `pdf/86712_regulatory_analysis-hwp-2024.pdf` | 21 | 60.57 | 99.90 | 99.95 |
| k86712 | 〃 | 〃 | 34 | 81.43 | 99.97 | 99.91 |
| d1613 | `samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp` | `pdf/1613000-202200037-air-traffic-controller-cbta-2020.pdf` | 190 | 37.94 | 99.47 | 99.48 |
| t1749x | `samples/task1749/saved_bounds_cumulative_page_break.hwpx` | `pdf/task1749/saved_bounds_cumulative_page_break-hwpx-2020.pdf` | 5 | 71.02 | 100.00 (99.998) | 99.97 |
| d5941 | `samples/issue5941/1480000-201900698-native-neartop-reset.hwp` | `pdf/issue5941-1480000-201900698-2020.pdf` | 11 | 46.16 | 100.00 | 100.00 |
| d1342 | `mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp` | `mydocs/pr/assets/issue7445/education-curriculum-map/existing-hwp-hancom2020-print.pdf` | 93 | 95.73 | 95.73 | 95.76 |

d1342는 변화 없음 대조군이다. 415쪽 전체에서 devel과 head의 Native 점수가 같고 SVG도
바이트 단위로 같다. p93은 이 문서의 최저 쪽이다.

각 쪽의 `native-<key>-pNNN-{review,overlay}.png`, `wasm-<key>-pNNN-{review,overlay}.png`를
직접 판독했다. 표 외곽·행 경계·쪽 끝 행 분할 위치·뒤 문단이 PDF와 겹친다.
`pr_review_gate`는 대표 쪽 14개 실행 모두 `passed`이고 전체 쪽수도 PDF와 같다(`page_count_mismatch=false`).
원문은 [gate.json](gate.json)이다.

## 전쪽 결과

[summary.tsv](summary.tsv)와 `tsv/<key>-{devel-native,head-native,head-wasm}.tsv`(원 TSV).
19개 문서 모두 Native·WASM 전체 쪽수가 기준 PDF와 같다.
devel 대비 0.1%p 이상 오른 쪽: d1613 12쪽, k1130 5쪽, k86712 4쪽, t1749x 2쪽, d5941 1쪽.
0.1%p 이상 내려간 쪽: d1613 p165 99.33→99.18, p173 99.40→99.22 (Native).
두 쪽의 표 가로 괘선은 devel이 PDF보다 약 1.5px 아래였고 head에서 PDF와 0.5px 안으로 맞았다.
점수가 내려간 원인은 이 측정으로 밝히지 못했다.

head에도 90% 미만인 쪽이 남아 있다. 모두 devel에서도 같은 값이거나 더 낮았다.
d5941 20쪽, d1613 4쪽(p1·p6·p32·p75), d1880 p10, hs2 4쪽, k86712 3쪽(p25·p27·p28), k1130 p68,
d80168x 104쪽(HWPX 경로 별도 문제). 이 PR은 이 쪽들을 해결했다고 주장하지 않는다.
