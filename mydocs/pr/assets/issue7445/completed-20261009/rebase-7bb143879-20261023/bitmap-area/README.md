# Print 비트맵 면적 적분 검증

Issue: #7445. 기존 대상의 로컬 보정 체크포인트이며 **전체 PR 준비 완료가 아니다**.

## 조건과 출처

- 부모 HEAD: `65fdd7926795f09d3b5b388a75d40e9186bbc242`.
- base: `7bb143879dbde80a02fb4a63692556ac28fe46ac`.
- 96 DPI / Print / PDFium 153.0.7999.0 / pypdfium2 5.13.0 / `pdfium_smooth_rgb_v1` / 색상 경계 보정 전 2px 실루엣 90%.
- Native Skia와 실제 fresh WASM Canvas 출력을 각각 산출했다. 캡처 총 8, Cargo 순차, jobs 8, test threads 8을 유지했다.
- `bitmap_raster.rs`: `a71dd337d46e3f4d22add3fea175384e5125ff9a83aaeb9147e797f80948f750`.
- 로드맵 검사 소스: `5c45ba5babc77875cbbcc9d20187771384543f1cd856ff64f2fd733d664f4d30`.
- Native: `bc9830a28add5bdf3bd7e0d2e953e3f2667067544dc0726822d35c00b6ad061f`.
- JS: `aea5f6639676442e0628b640b75d0f70eb300481421112d23b661a082f695a68`.
- WASM: `d1f6dbc9749881213348fce936063932fac6941f5386817491eedf6a769bcae1`.
- root pkg / Studio public / frozen package hash가 같다. 실제 Studio 초기화 응답과 actual36 검증은 아직 미완료다.
- Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화 결과가 아니다.

## 전쪽 결과

| 기존 대상 | 쪽수 | Native 최저 | fresh WASM 최저 | 90% 미달 |
| --- | ---: | ---: | ---: | ---: |
| 로드맵 | 302 | 92.63744% | 92.63744% | 양쪽 0 |
| PrEP | 140 | 94.75150% | 94.22354% | 양쪽 0 |
| RowBreak HWP | 18 | 93.06876% | 92.99291% | 양쪽 0 |
| RowBreak HWPX | 18 | 92.85921% | 92.82532% | 양쪽 0 |

입력은 각각 `samples/issue5941/1490000-201600081_roadmap_research.hwp`, `samples/issue2006/1790387_prep_final_report.hwpx`, `samples/rowbreak-problem-pages.hwp`, `samples/rowbreak-problem-pages.hwpx`다. 기존 독립 Print PDF를 변경하지 않았다. 로드맵/HWPX는 compat 2022, PrEP/RowBreak HWP는 compat 2024다.

로드맵 153쪽은 양쪽 83.55427% → 92.63744%다. 직전 Stage68 후보 대비 최대 하락은 Native 1.55230%p, WASM 1.55246%p로 2%p 이상 하락은 없다. 이 비교를 upstream/devel 비교로 대신하지 않는다. 고정 base의 같은 로드맵은 299쪽이어서 302쪽 후보와의 엄격한 전쪽 대응 검증은 미완료다.

두 backend 각각 302개 RenderTree가 Stage68과 byte 동일해 이 변경으로 쪽·줄·셀·개체 소속이나 순서가 바뀌지 않았다. 할당을 줄인 최종 v2도 초기 면적 적분 후보와 각각 302개 PNG가 byte 동일하며, 이 문서의 TSV는 v2에서 전쪽 새로 산출한 원본이다.

## 회귀와 판독

기존 그림·crop·palette·로드맵 검사 15건이 통과했다. 이후 완료된 296·297쪽(양쪽 각 100%)의 독립 PDF 외곽 괘선에 ±2CSSpx 검사를 기존 행 소속 함수 안에 추가하고 로드맵 3건을 다시 실행해 3 PASS를 확인했다. 3건은 앞 15건과 겹치며 18개 고유 검사라는 뜻이 아니다. 기존 셀 소속·국가 행·머리행 반복·누락/중복·마지막 출처 검사는 유지했다.

153쪽의 점·추세선·축·격자와 캡션, 최대 하락한 152쪽의 막대·평균선·국가 이름, 296·297쪽 표의 Estonia 종료/Finland 시작과 하단 괘선을 review/overlay로 판독했다. 153쪽은 JPEG 기준과 원본 BMP의 농도·색상 차이가 남는다. 152쪽도 막대 가장자리와 회색 격자의 차이가 남으며 RGB 일치로 보고하지 않는다. 296·297쪽의 raw 100%도 테두리 농도까지 동일하다는 뜻은 아니다.

## TSV 원본

- [roadmap-before-native.tsv](roadmap-before-native.tsv) — `4fbdb3d3ec348a5b5d4e7b02bf15a6c94cd11762e498560d76b9e24ea7383ecc`
- [roadmap-before-wasm.tsv](roadmap-before-wasm.tsv) — `e4570b4981c98838c6fc8b99f4bffdf28c0b6d20d553b064c9f17dfac5edd84c`
- [roadmap-after-native.tsv](roadmap-after-native.tsv) — `801b94ec1704d6d489b694ba0bca7fb75cbb3cff8ef56f6d1f83f4a682dfb142`
- [roadmap-after-wasm.tsv](roadmap-after-wasm.tsv) — `28d804cd20c5a82e444d2eebc7ec8561415f7ec269e9715095d18350a55b3152`
- [prep-native.tsv](prep-native.tsv) — `185dbb84d9acffc7186a3d6b38f29d1a023b09e8d4d703e4c13472a8c0261600`
- [prep-wasm.tsv](prep-wasm.tsv) — `e6fd58eb2d6233ec569524c4fa17c00678bdc37a28f79bf5d6146d5e49b389f3`
- [rowbreak-hwp-native.tsv](rowbreak-hwp-native.tsv) — `e9a17e77c4cc2147d83010e6c0d464118a390356a35fffacc4c406e6fc7e12ef`
- [rowbreak-hwp-wasm.tsv](rowbreak-hwp-wasm.tsv) — `0d1ed147069b340e14acabc5087909f6897d8561307963ac53fb020add833847`
- [rowbreak-hwpx-native.tsv](rowbreak-hwpx-native.tsv) — `c01f3f43330a0c3c9136e57375a7f7ca530bd4daf650710fc97d221bc786c47a`
- [rowbreak-hwpx-wasm.tsv](rowbreak-hwpx-wasm.tsv) — `bb4ae9441bef5f27b7bd391a76457a42be9bbda65fd8c6cc7b584c1c62f1c8d4`

## 대표 review / overlay

- before native 153쪽: [review](before-native-p153-review.png) / [overlay](before-native-p153-overlay.png).
- after native 153쪽: [review](after-native-p153-review.png) / [overlay](after-native-p153-overlay.png).
- after native 152쪽: [review](after-native-p152-review.png) / [overlay](after-native-p152-overlay.png).
- after native 141쪽: [review](after-native-p141-review.png) / [overlay](after-native-p141-overlay.png).
- after native 296쪽: [review](after-native-p296-review.png) / [overlay](after-native-p296-overlay.png).
- after native 297쪽: [review](after-native-p297-review.png) / [overlay](after-native-p297-overlay.png).
- before wasm 153쪽: [review](before-wasm-p153-review.png) / [overlay](before-wasm-p153-overlay.png).
- after wasm 153쪽: [review](after-wasm-p153-review.png) / [overlay](after-wasm-p153-overlay.png).
- after wasm 152쪽: [review](after-wasm-p152-review.png) / [overlay](after-wasm-p152-overlay.png).
- after wasm 141쪽: [review](after-wasm-p141-review.png) / [overlay](after-wasm-p141-overlay.png).
- after wasm 296쪽: [review](after-wasm-p296-review.png) / [overlay](after-wasm-p296-overlay.png).
- after wasm 297쪽: [review](after-wasm-p297-review.png) / [overlay](after-wasm-p297-overlay.png).

## 미완료 범위

이 전쪽 결과는 위 기존 4개 입력의 현재 출력 검증이다. 브랜치 전체 nextest·필수 lint/build/Skia/manifest/unit tier·실제 소비 경로 및 최종 devel 대응 비교는 미완료다. 직전 집중 검사의 task1725·overlap partition12·7379 캡션·7470 첫 띠 4개 실패를 해결했다고 쓰지 않는다. C354는 고정 devel Native 98.85995% 대비 후보 94.57727%로 4.28268%p 하락하여 재검토 대상이다. C356 합성 격자도 42.89522%로 미달이다. 이 독립 진단 잔여를 로드맵 개선으로 면제하지 않으며 원인 검토를 계속한다. 지침·기대값·기준 PDF·threshold를 완화하지 않았고 원격 PR/push/merge를 실행하지 않았다.
