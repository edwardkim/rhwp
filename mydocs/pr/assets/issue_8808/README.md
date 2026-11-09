# #8808 시각 검증 증거

- 검증 head: `df91ede30e` (native-skia release sha256 `a9d08e79…`, fresh WASM `rhwp_bg.wasm` sha256 `c0bb8ff8…`)
- base: upstream/devel `024d610633`
- 명령: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96` (TSV), 같은 명령에서 `--silhouette-only` 를 뺀 실행(review·overlay PNG). WASM 은 `--wasm-pkg pkg`.
- 판정값: `silhouette_raw_match_percent`(TSV 4열). 글꼴: 문서 face(맑은 고딕 보통·굵게·Semilight, HY견고딕, 휴먼명조, 한컴돋움)만 둔 디렉터리를 `RHWP_FONT_PATH` 로 공급.

## 입력과 기준 PDF

| 키 | 입력 | 기준 PDF | 쪽수 rhwp/한컴 |
|---|---|---|---|
| reduced | `samples/issue8808/36341965-float-band-gap-reduced.hwpx` (sha256 `18ea6268…`) | `pdf/issue8808/36341965-float-band-gap-reduced-2020.pdf` (engine 2020 Print, `681ba666…`) | 2 / 2 (base 3) |
| original | 공문서 `36341965_2026년새활용공간콘텐츠기획및운영제안서평가계획(재기안).hwpx` 원본(저장소 밖, sha256 `2fd2d841…`) | 한컴 engine 2020 Print PDF(Creator `Hwp 2022 12.0.0.4547`, `3c7601f1…`) | 8 / 8 (base 9) |

축소본은 원본에서 1쪽 결재 표 문단(`pi=0`)과 일정 표 뒤 문단을 지우고 구역 정의를 둘째 문단으로 옮겼다.
보안 마스킹 문단(저장 줄 없음)은 그대로 두어 로드 시 구역 vpos 재계산이 도는 조건을 유지한다.

## 결과

| 키 | Native | fresh WASM | 90% 미만 |
|---|---|---|---|
| reduced | p1 99.82 · p2 99.98 | p1 99.10 · p2 99.96 | 없음 |
| original | 최저 63.22 | 최저 62.78 | Native p1·p6·p7, WASM p1·p7 |

원본 쪽별(Native / WASM): p1 63.22/62.78 · p2 99.82/99.10 · p3 99.98/99.96 · p4 98.96/98.91 ·
p5 99.90/99.93 · p6 89.84/90.29 · p7 83.56/83.90 · p8 93.59/93.90.

- p1: 표 칸 안 제목 상자가 약 50px 위에 놓이는 기존 결함. base 와 같은 점수(63.22)로 이 변경과 무관하다
  (별도 추적: rhwp-issue-docs#1082).
- p6·p7: 줄 위치는 맞고, 이 환경에 없는 글꼴(신명 견고딕·HY신명조·신명 태명조) 문단의 글자 폭과 2~4px
  누적 차이다.
- base 원본은 9쪽이라 쪽 대응이 2쪽부터 어긋난다(`native/36341965-original_base_p1-8_silhouette.tsv`, p1~8 만).

## 파일

- `native/`·`wasm/`: 전쪽 TSV 원본
- `png/`: reduced p1·p2 의 review·overlay(Native·WASM)
