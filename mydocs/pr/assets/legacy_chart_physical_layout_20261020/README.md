# 구형 OLE 차트 배치와 쪽 배경 색조 증거

## 2026-10-25: 원본의 워터마크 검사 재검토

같은 원본 HWP/HWPX와 아래에 보존한 독립 Print를 현재 runtime
`8dc75860cec88ddf7c3f8aeee230b2ca7e2e5537`에서 다시 전쪽 비교했다.
`current-20261025/tsv/{native,wasm}/`는 현재 TSV 원문이고, 대표 HWP의 review·overlay는
`current-20261025/png/{native,wasm}/`에 있다. 제품 코드·입력·PDF·색조 기대 RGB는 바꾸지 않았다.

| 원본 | 전체 쪽수 | Native224 raw | fresh WASM063 raw |
| --- | ---: | ---: | ---: |
| `samples/143E433F503322BD33.hwp` | 1 | 99.28794% | 99.30167% |
| `samples/hwpx/143E433F503322BD33.hwpx` | 1 | 99.27978% | 99.29360% |

원 PR [#1019](https://github.com/edwardkim/rhwp/pull/1019)의 실제 merge는
`e26f159b609bcfb9b5106d246811b51c67dd064d`다. 원래 opacity 0.26/0.15는 화면의
근사 계약이었다. 이슈 #1156의 원 해결 `9ede2c589525f374661c8d200d2abdb1a3ce14c0`도
SVG에 opacity 그룹이 있어야 한다고 검사했다. 이슈 timeline은 이 커밋의 참조 뒤
2026-05-29에 닫힌 것을 기록한다. 해당 커밋과 연결된 #1294는 이후의 release PR이므로
이를 별도의 초기 해결 PR로 오인하지 않는다.

이 브랜치의 Print 보정 `4bdec5890848c063020cfeb14a3a0efd55a91714`는 원본 모델을
보존하면서 공통 `resolve_page_background_tone`에서 실제 RGB를 한 번 보정한다.
이후 render node의 밝기·대비를 소비 완료인 0/0으로 정규화해 backend의 이중 보정을 막는다.
이미 보정된 그림에 과거의 opacity를 다시 곱하는 것이 독립 Print와 다른 동작이다.

같은 Print PDF(SHA-256 `5bb1416b4850e9e1d4b9abaec3c68dd8322a7480a1c0f98d1b892a71ff44f18b`)의
이미지 객체를 다시 직접 디코딩했다. 예를 들어 xref 12와 14에 RGB197/255/255 표본이
있다. 원본 그림의 대응 RGB0/118/191 표본이 그 색으로 바뀌는 기존 독립 기대값을 유지한다.
RGB 채널의 ±2 바이트 오차는 JPEG에 대한 허용치이며 문서 위치의 px 오차가 아니다.

`issue_1019`의 쪽 배경 검사는 새 이름으로 실제 SVG의 배경 PNG를 디코딩하고,
대응 RGB·원래 bitmap 크기·불투명한 색 표본을 확인한다. 그림과 조상 노드에 추가
opacity/filter가 없는지도 검사한다. `issue_1156_chart_column_flow`의 새 이름 검사는
배경 소유 하나, 공통 보정의 소비 완료와 원본 채움 속성 보존을 확인한다.
고정 화면 영역이나 배치 좌표는 추가하지 않았다.

같은 `issue_1019` 모듈의 별도 `253E164F57A1BC6934-empty.hwp`는 기존 채움 그림의
0.26/0.15 opacity 조건을 그대로 유지하고, 실제 1·2쪽 검사를 다시 실행했다.
색조·차트·범례·배치의 기존 9개 검사를 포함한 세 모듈은 **13 PASS / 0 FAIL**이다.
정식 소스의 동결 라이브러리 실행이며 전체 nextest 완료가 아니다.

현재 Native/fresh 대표 review·overlay를 직접 판독했다. 배경 색·크기·중앙 배치는
Print와 맞으며, 차트와 뒤 본문의 순서도 유지한다. 아래 기존 기록처럼 본문 일부의
글자 가장자리·미세 간격 차이는 남는다. 이를 RGB나 페이지 점수로 면제하지 않는다.
전체 PR의 원 #8064 corpus·Rust·lint·Studio·고정 devel 대비 검증은 계속 진행한다.

---

Native019와 최적화 fresh WASM010의 33개 조건 전쪽 TSV 원문이다. 기준은 독립 한컴 Print PDF다.
각 조건은 1쪽이며 양쪽 33/33쪽이 90% 이상, 고정 devel 대비 0.1%p 이상 하락 0쪽이다.
이 단계는 차트 제목·축·범례·프레임과 쪽 배경 RealPic 색조의 검증이다. 통합 PR 전체의 완료 보고가 아니다.

## 생산·소비와 기준

- 제목·축·범례 예약과 실제 그리기가 같은 `ColumnLayout`·`LegendLayout` 결과를 소비한다.
- 저장 캐시의 글꼴과 현재 전역 범례 글꼴이 다르면 실제 메트릭으로 갱신한다.
- 축을 숨긴 경우에는 표시되지 않는 글꼴·눈금 길이로 플롯을 움직이지 않는다.
- 쪽 배경 RealPic은 원본 밝기·대비를 공통 생산 지점에서 한 번 적용하며 model/저장 원문은 바꾸지 않는다.
- 원본의 색조 70/-50은 실제 Print 이미지 9개와 대조했다. 기존 근사 alpha를 겹쳐 적용하지 않는다.
- base: `d976d638840892abceccd0c067512aa46b025f7f`. 제품 소스가 같은 보존 producer `d7d4d2ea6ed427f2f3bc8b59a936a5f99aca852b`를 사용했다. 이후 차이는 test/docs와 cfg(test) 모듈 제거뿐임을 별도로 확인했다.
- 후보 producer: `73b33fde72642faf71c7060bceba47358520262b` 위 아래 소스 지문. 단계 commit과도 같은 바이트인지 확인한다.
- Native `--profile print`, fresh WASM Canvas2D `print`, 96DPI. 표의 compat는 조판 모드이며 MCP engine 명칭과 구분한다.
- 2024 저장본인 분리 차트는 2024 Print PDF, 나머지는 MCP engine 2020의 PrintMethod 0 1-up이다.

## 직접 판독과 남은 차이

원본의 막대·축 숫자·제목과 범례, 좁은 범례의 한 열 세 행, 낮은 차트의 축 숨김, 두꺼운 프레임의 두 열 두 행을 같은 영역에서 대조했다. 최종 fresh WASM010의 원본 전쪽과 좁은·낮은 차트의 review·overlay도 직접 판독했다. 분리 차트는 Native/fresh WASM 둘 다 100%이며 축의 독립 좌표를 ±2px로 보호한다. 100%는 2px raw 실루엣이며 RGB 완전 일치를 뜻하지 않는다.

원본과 낮은 차트 대조군의 오른쪽 본문 줄 시작·행간, Print의 드롭캡과 글자 가장자리 잔차는 남아 있다. 이 TSV의 수치 통과로 그 본문의 완료를 선언하지 않는다. 통합 검토의 다른 미달 원본과 미주 접두사 실패도 별도 진행 중이다.

## 페이지별 원문

| 조건 | Native raw | fresh WASM raw | Native TSV | WASM TSV |
| --- | ---: | ---: | --- | --- |
| isolated-hwp | 100.00000% | 100.00000% | [원문](head/native/isolated-hwp.tsv) | [원문](head/wasm/isolated-hwp.tsv) |
| isolated-hwpx | 100.00000% | 100.00000% | [원문](head/native/isolated-hwpx.tsv) | [원문](head/wasm/isolated-hwpx.tsv) |
| original-hwp | 99.27590% | 99.29729% | [원문](head/native/original-hwp.tsv) | [원문](head/wasm/original-hwp.tsv) |
| original-hwpx | 99.26776% | 99.28923% | [원문](head/native/original-hwpx.tsv) | [원문](head/wasm/original-hwpx.tsv) |
| pension-category-font-20 | 99.26721% | 99.28979% | [원문](head/native/pension-category-font-20.tsv) | [원문](head/wasm/pension-category-font-20.tsv) |
| pension-frame-120 | 99.27545% | 99.29485% | [원문](head/native/pension-frame-120.tsv) | [원문](head/wasm/pension-frame-120.tsv) |
| pension-frame-127.5 | 99.27523% | 99.29539% | [원문](head/native/pension-frame-127.5.tsv) | [원문](head/wasm/pension-frame-127.5.tsv) |
| pension-frame-200 | 99.27889% | 99.29613% | [원문](head/native/pension-frame-200.tsv) | [원문](head/wasm/pension-frame-200.tsv) |
| pension-frame-40 | 99.26855% | 99.28700% | [원문](head/native/pension-frame-40.tsv) | [원문](head/wasm/pension-frame-40.tsv) |
| pension-frame-60 | 99.27069% | 99.29081% | [원문](head/native/pension-frame-60.tsv) | [원문](head/wasm/pension-frame-60.tsv) |
| pension-legend-08-font | 99.26736% | 99.28783% | [원문](head/native/pension-legend-08-font.tsv) | [원문](head/wasm/pension-legend-08-font.tsv) |
| pension-legend-09p5-font | 99.26761% | 99.28901% | [원문](head/native/pension-legend-09p5-font.tsv) | [원문](head/wasm/pension-legend-09p5-font.tsv) |
| pension-legend-12-font | 99.26796% | 99.28688% | [원문](head/native/pension-legend-12-font.tsv) | [원문](head/wasm/pension-legend-12-font.tsv) |
| pension-legend-17p5-font | 99.22612% | 99.26339% | [원문](head/native/pension-legend-17p5-font.tsv) | [원문](head/wasm/pension-legend-17p5-font.tsv) |
| pension-legend-20-font | 99.27769% | 99.29635% | [원문](head/native/pension-legend-20-font.tsv) | [원문](head/wasm/pension-legend-20-font.tsv) |
| pension-mid-height-19000 | 97.71545% | 97.80359% | [원문](head/native/pension-mid-height-19000.tsv) | [원문](head/wasm/pension-mid-height-19000.tsv) |
| pension-narrow | 99.26990% | 99.29043% | [원문](head/native/pension-narrow.tsv) | [원문](head/wasm/pension-narrow.tsv) |
| pension-original | 99.27606% | 99.29741% | [원문](head/native/pension-original.tsv) | [원문](head/wasm/pension-original.tsv) |
| pension-series-20-font | 99.26762% | 99.28913% | [원문](head/native/pension-series-20-font.tsv) | [원문](head/wasm/pension-series-20-font.tsv) |
| pension-short | 97.86430% | 97.95739% | [원문](head/native/pension-short.tsv) | [원문](head/wasm/pension-short.tsv) |
| pension-short-category-font-20 | 97.86417% | 97.95726% | [원문](head/native/pension-short-category-font-20.tsv) | [원문](head/wasm/pension-short-category-font-20.tsv) |
| pension-short-value-font-20 | 97.86417% | 97.95726% | [원문](head/native/pension-short-value-font-20.tsv) | [원문](head/wasm/pension-short-value-font-20.tsv) |
| pension-short-value-margin-0 | 97.86418% | 97.95728% | [원문](head/native/pension-short-value-margin-0.tsv) | [원문](head/wasm/pension-short-value-margin-0.tsv) |
| pension-short-value-margin-200 | 97.86408% | 97.95725% | [원문](head/native/pension-short-value-margin-200.tsv) | [원문](head/wasm/pension-short-value-margin-200.tsv) |
| pension-short-value-tick-0 | 97.86417% | 97.95726% | [원문](head/native/pension-short-value-tick-0.tsv) | [원문](head/wasm/pension-short-value-tick-0.tsv) |
| pension-short-value-tick-200 | 97.86417% | 97.95726% | [원문](head/native/pension-short-value-tick-200.tsv) | [원문](head/wasm/pension-short-value-tick-200.tsv) |
| pension-tall | 99.05445% | 99.07451% | [원문](head/native/pension-tall.tsv) | [원문](head/wasm/pension-tall.tsv) |
| pension-value-font-20 | 99.26308% | 99.22501% | [원문](head/native/pension-value-font-20.tsv) | [원문](head/wasm/pension-value-font-20.tsv) |
| pension-wide | 99.18075% | 99.20072% | [원문](head/native/pension-wide.tsv) | [원문](head/wasm/pension-wide.tsv) |
| title-ansi-only | 99.26776% | 99.28923% | [원문](head/native/title-ansi-only.tsv) | [원문](head/wasm/title-ansi-only.tsv) |
| title-font-12 | 99.26643% | 99.28835% | [원문](head/native/title-font-12.tsv) | [원문](head/wasm/title-font-12.tsv) |
| title-font-20 | 99.24001% | 99.25463% | [원문](head/native/title-font-20.tsv) | [원문](head/wasm/title-font-20.tsv) |
| title-font-8 | 99.26776% | 99.28918% | [원문](head/native/title-font-8.tsv) | [원문](head/wasm/title-font-8.tsv) |

`base/`에는 같은 입력의 기준 TSV를 보존했다. 실행 로그·중간 JSON은 ignored output에만 보관한다.

## 입력·Print SHA-256

| 조건 | 입력 | 입력 SHA-256 | Print PDF | PDF SHA-256 |
| --- | --- | --- | --- | --- |
| isolated-hwp | `samples/hwpx/143E433F503322BD33-ole.hwp` | `9f7975a341f9bf0be89e3d6dc1385b59de5e61e079eeb0048db0090c932ad14c` | `pdf/143E433F503322BD33-ole-2024.pdf` | `584902f845c4b0d88d4e827a8a7dde23bf3ee3d92bd843f4c1f925b7a7498b20` |
| isolated-hwpx | `samples/hwpx/143E433F503322BD33-ole.hwpx` | `83815ff764b8ab40e6e232e57bc20b6dcb6f0b2c824f7673148fe63d965e1255` | `pdf/143E433F503322BD33-ole-2024.pdf` | `584902f845c4b0d88d4e827a8a7dde23bf3ee3d92bd843f4c1f925b7a7498b20` |
| original-hwp | `samples/143E433F503322BD33.hwp` | `1bcdc7a2de38680cdb715556f40c8477f8242131d28de949b09e945022bf35fa` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/original-hwp.pdf` | `5bb1416b4850e9e1d4b9abaec3c68dd8322a7480a1c0f98d1b892a71ff44f18b` |
| original-hwpx | `samples/hwpx/143E433F503322BD33.hwpx` | `1ee1e9045be4334fb9f940d7226cfda2884df9fecac6fa7fca76fb81c08a4a81` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/original-hwpx.pdf` | `3eedafdf07e534af434c6dff58e6aefea0c2b1abe9d286419b410328e6aee44d` |
| pension-category-font-20 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-category-font-20.hwpx` | `06ece2e4d0756c695fd0342322aa2940d99afe4bc4330a5fc0c7fb44d916f6d3` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-category-font-20.pdf` | `3fcb9c7274fbb65a614d2b266e4c5811cb2cfbb24b05fbe48d18be58cf5f34ae` |
| pension-frame-120 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-frame-120.hwpx` | `59431cab52016eb680922feff3ba8b8095aa26ebc5f12a4ce71bc50adf7c4ade` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-frame-120.pdf` | `2185f5bdf36fd68a3925805e25ce942a5e4ca23494dc18b6157976eb4107fcc0` |
| pension-frame-127.5 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-frame-127.5.hwpx` | `e2508d4eeb5a9932ff2a9e70d9c84cc3cc1ce88a6b26bd55c9cfe8bd8acb3669` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-frame-127.5.pdf` | `03e66246b150bbbc9ac7142b1f8a2c76e11f8967fbd9a965115a47e796b23a7d` |
| pension-frame-200 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-frame-200.hwpx` | `3445fa513ef7f5325c8d4bd59eb1f1c1e1b72ea01012b352c20b7ffc1f68502a` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-frame-200.pdf` | `3878b47a593bb62a344a222c35893f4a48335d0bd988c0b3d8fe27ee3d4925c5` |
| pension-frame-40 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-frame-40.hwpx` | `fa41b4b89a4f1fd7f5451d044fea7210144d72d2284042f88f948607c763d515` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-frame-40.pdf` | `8c936e81d9fe5f1bfe066a87c4d06ed0064914127f6a68c89b2c68baf367c222` |
| pension-frame-60 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-frame-60.hwpx` | `e706a2cae78a82b8e583cde91662a35ca19112bc363e3cfefe715386fcabbc66` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-frame-60.pdf` | `667edf0648ced69205d6688906e7b44b57eb0e605a523567314e3877159e196e` |
| pension-legend-08-font | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-legend-08-font.hwpx` | `7def017e88da56c4cb6bae0ca6a4812b3d02b3a3031c92ec9d1963da69977325` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-legend-08-font.pdf` | `2463f8968e1861e1039d94fd165b5d02f80e4f5fb4791b326f73e29efecfa20c` |
| pension-legend-09p5-font | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-legend-09p5-font.hwpx` | `4a04d74053f100d0614b5c66e8d04b2a02735b59538d4ce15daa084dad7ac752` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-legend-09p5-font.pdf` | `6e26951aa4635d1c2950f0383a0eb877d28fba65c8154c4cf6b498819c39dd15` |
| pension-legend-12-font | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-legend-12-font.hwpx` | `27a46aa950146e26f6a001ba1db602690871c35d07d5c716122bfc50f849c445` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-legend-12-font.pdf` | `102c9bcd3e483e73804ee1cdd6a0676b43397833f629f9ee0ee5476a3263f52b` |
| pension-legend-17p5-font | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-legend-17p5-font.hwpx` | `45cc84e301e4e90113d52bccf9caf227666695a30479373f3721c7f89506fa9b` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-legend-17p5-font.pdf` | `3090d06affcef84cd692b86bcfcf4ceb3aebbd68d942b8f1a733160532d696c4` |
| pension-legend-20-font | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-legend-20-font.hwpx` | `4b04eca425288d6b3279f4a4a7ef5bd22ca7d6ffb7cced610b0d6d6be377ff12` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-legend-20-font.pdf` | `8f4e4d564d2a0478abed87cd0c52654dbbd314019bd277c33db86ba72ce3680d` |
| pension-mid-height-19000 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-mid-height-19000.hwpx` | `e2ab83947ab936035f463bc08e852ead37fc7a81c45627f7017cd3d69e698f1c` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-mid-height-19000.pdf` | `b3b12f610f39573d8b140012e5ddc4fd292962586ca8134c0f244001ecada740` |
| pension-narrow | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-narrow.hwpx` | `e8a414e282ea42a6374064f200215734699ac71730dd257e47c938f242e5e25b` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-narrow.pdf` | `80ff94f8d9bd250f1df247aeaf6b77af8bd8b0945a7f0da4176eea2c1a67e3bd` |
| pension-original | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-original.hwpx` | `6d152e8a18ebdaafc09e44ae36fb6af3932fbbeecbecf3b684ad8d9d49093f2d` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-original.pdf` | `a78fce8001e31fa4551565f930dbcd62e1943ba335ad5325eb16db66e9abfeb2` |
| pension-series-20-font | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-series-20-font.hwpx` | `90c8c8f2846c8f5267299552b8f1000b59763ecc508a7708a74a6960f971b26b` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-series-20-font.pdf` | `9f3437d2894706af4095ab383037aeca823b461397d3a7cbc505027268bc4544` |
| pension-short | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-short.hwpx` | `7cbf5c13873d9bb81b158bd1545bec658d03f4b24097d03936cb7b7083ceceba` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-short.pdf` | `ba060320556c39e1682d887b3ac3cd58c000c9b4848c5867b24f11792e410ad3` |
| pension-short-category-font-20 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-short-category-font-20.hwpx` | `94216f5c6954a80b73891ab2b853c8fd40610d3587d8d06543c8d4bdaca22e31` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-short-category-font-20.pdf` | `d216d886e8676a28175fe5a84dc332a239993f6ce377ebeab56547d30288711c` |
| pension-short-value-font-20 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-short-value-font-20.hwpx` | `b871b4fa1032bd9797cb71e471c2eb6d1997eaf371a5fa0ab8a61b53b885db68` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-short-value-font-20.pdf` | `f183b672c86d50e9b0bad55059ea7a13d8d43d56c28b50889f62fc80f9e09de5` |
| pension-short-value-margin-0 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-short-value-margin-0.hwpx` | `a155d57f65e4aa505e083d3232394083e459d5eec34035e0ad69492a27af3089` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-short-value-margin-0.pdf` | `ec56de01efcf30bd7eb5356db91e776b7e1d9445549219d716a6dbc5e66d4c22` |
| pension-short-value-margin-200 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-short-value-margin-200.hwpx` | `53f5bda1f3b7e3e124c93294718103162345ce1ca33abefcc564e8c37481cfd5` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-short-value-margin-200.pdf` | `1ef40b2dad1ff7502a48a23ebe1a0b587b2e18622c28a7f2033cf68a7ea284d5` |
| pension-short-value-tick-0 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-short-value-tick-0.hwpx` | `2cae51cbc887539d690b9ffca111412f4de92ecda9b3600846f3b1f5171ba4b7` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-short-value-tick-0.pdf` | `ea90ba58ee9ba4b9ecebf874d891b4bb245f1b8111f19e126cab627923d6c687` |
| pension-short-value-tick-200 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-short-value-tick-200.hwpx` | `1c5c7d05583b4240d1e69ce3258de3e4743fddb1dec635bc979e3f0b07a31281` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-short-value-tick-200.pdf` | `d56fe2a95359bd9362cdd3f7963f7a3e750d3180bb92f5d4e2233e5972bf9faf` |
| pension-tall | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-tall.hwpx` | `cb0a73c1a6e7548b5c47872da2ce6ecd31610e04ad0649db6abba97fef4ee7c9` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-tall.pdf` | `cbc03a3ebc875e615a4e4c056765b9d407cb0103c6e89cf254a037ec6a39be1f` |
| pension-value-font-20 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-value-font-20.hwpx` | `37e588ee171005f927eb5dc55236933f7d000d95267d44b3f87120087d8ce07f` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-value-font-20.pdf` | `9ecef28e3778fdc135b0123320bf1170a30a0e0da28299e4ab797b7067919f8b` |
| pension-wide | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-wide.hwpx` | `66c3819ff54493413207635f5e6fe8afe38738eb317a8f9b80fa38e44b4986c1` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/pension-wide.pdf` | `8bdbc207bdab165cc6b4f55b5088dce132fc7198ac68d1b3633181613710959a` |
| title-ansi-only | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-title-ansi-only.hwpx` | `c94a78437329110a9ae07d5f2900775ef2c21da93e4723178ea362f701f60a5e` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/title-ansi-only.pdf` | `84a927f68c04910473fa31b91de54af8e501e2908f4f5fb980906ddcb163b012` |
| title-font-12 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-title-font-12.hwpx` | `247a6fe21aec81b9e05865f45d5960afb3ecd2666500ee07536965a822b0e54b` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/title-font-12.pdf` | `ca3c46378dbd65e6f4eb2e263cda34f4bc10f6b41d43a3d04489b2230c1b0067` |
| title-font-20 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-title-font-20.hwpx` | `cae7716a248a7ce93493a69f04a83ba7da30b54863b86c39be12c5c37ed3ed2e` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/title-font-20.pdf` | `0ba75430b1948cc1aaa16ab7f788093d3058cb8a7457edc18ddc2feb37fdad0a` |
| title-font-8 | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs/pension-title-font-08.hwpx` | `378fc659fda3fc4e3bd501cc4d4718d3ac6369acd519e6c28b9af8a675058d55` | `mydocs/pr/assets/legacy_chart_physical_layout_20261020/pdf/title-font-8.pdf` | `f87fd29cf8356236e9b8b708b988f5c590e2ab493949a77ec1d54d87cd75bca2` |

## 제품 소스 지문

| 소스 | SHA-256 |
| --- | --- |
| `src/ole_chart/mod.rs` | `c22f7fb03a3976b91c63c00bfec0307aae5ef74bb28355eaecd9350983304fb3` |
| `src/ole_chart/legacy_column_renderer.rs` | `6febe921236637e9877d6ebfd3d36de9c981910ebf59b4de8d3f2e5b9a07cf30` |
| `src/ole_chart/legacy_column_layout.rs` | `71019cab3f3fa8f0fc52952c01d8ce45220b9c7661db176db077c2ae79abc882` |
| `src/ole_chart/legacy_legend_layout.rs` | `599321ae8aecb1154e142b62d379b8409cdaafaca343db10d9eb336e7273ee98` |
| `src/ole_chart/legacy_text_metrics.rs` | `ed3686728ae018ff2ff128ae7fdb9a7b031d1dc127cf6fd707496052a4165bd8` |
| `src/document_core/queries/cursor_rect.rs` | `795dc0928c4a9c2ebf4c2ab41c3464ff306cc6d9d8a4115ea67b47a8959c526a` |
| `src/document_core/queries/rendering.rs` | `85ed4458a2ae15d6ac61a4843cfe1a40a17ede9e4a1f37f248abecb92f3da900` |
| `src/renderer/image_resolver.rs` | `ee27f601956456a6574b3f9afa0741a8acee49d92d4d07d3fe2e5ad8cd02dc57` |
| `src/renderer/image_resolver/picture_adjustments.rs` | `1157edd4ced6299d28e706873877ea0cb3bcb4614ad5ac775c646596e70ebaf4` |
| `src/renderer/layout.rs` | `9175fa4038cf8f494504c7a77fbd2a51d2928bcb734acb9c29440f42c7f7c051` |

## 회귀의 독립 기대값

`isolated-hwp` p1: Print 원시 벡터의 세로축 x=222.72pt, 상단 y=166.737976pt, 하단 y=283.13pt를 96DPI 좌표로 바꿔 각각 296.96px, 222.3173px, 377.5067px를 ±2px로 검사한다. 입력·PDF SHA는 위 표, 양쪽 100% TSV와 PNG는 이 디렉터리에 있다. 제목/축/범례와 막대의 포함·순서 관계도 유지한다.

다른 크기·글꼴 변형은 절대 좌표 대신 예약 공간·같은 행/열 소속·순서·비표시 축 변화의 불변 관계로 검사한다. 새 검사의 수정 전후 실행 결과는 실행 뒤에 덧붙인다.

## 새 회귀의 수정 전후 실행

`tests/cases/legacy_chart_physical_layout.rs`의 새 이름 검사 9건을 같은 fixture로 실행했다.

| 동결 라이브러리 | 컴파일 | 실행 | 확인한 의미 |
| --- | --- | --- | --- |
| 물리 배치 보정 전 `native-008-before-band-fix` | PASS | 1 PASS / 8 FAIL | 제목·축·범례 메트릭의 변화와 배경 색조 누락을 검출 |
| 경계 보정 전 `native-017-before-plot-budget-fix` | PASS | 6 PASS / 3 FAIL | 좁은 범례의 행 구성, 비표시 축의 플롯 불변성, 배경 색조 누락을 검출 |
| 현재 Native019 | PASS | 9 PASS / 0 FAIL | 관계 검사, 양쪽 100% 페이지의 독립 ±2px 축 좌표, Print RGB와 원본 보존 |

분리 차트의 좌표 검사는 이전 제품에서도 통과하는 정상 대조군이며, 결함 검출 증거로 주장하지 않는다. 컴파일·입력 조회 오류는 위 실패 수에 포함하지 않는다.

정식 Cargo 자동 수집 경로에서도 `regression_suite_019`의 해당 모듈을 실행해 **9건 모두 통과**했다. 컴파일 잡은 8개, 테스트 스레드는 12개이며 공유 `target/pr-review`를 사용했다. 이 결과는 전체 Rust 회귀 완료를 뜻하지 않는다.

```bash
CARGO_BUILD_JOBS=8 cargo nextest run --locked \
  --cargo-profile release-test --target-dir target/pr-review \
  --features native-skia --test regression_suite_019 \
  -E 'test(legacy_chart_physical_layout::)' \
  --test-threads 12 --no-fail-fast
```
