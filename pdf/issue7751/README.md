# #7751 그림 색조의 독립 한컴 Print 기준

2026-10-17, 한컴2024 `13.0.0.3901`, `PrintMethod=0`,
`frame_print_to_pdf_ex_one_up`, 각1쪽. rhwp 출력으로 만든 기준이 아니다.

## 입력과 보존

RGB11패치 PNG를10개의 명시적 합성 그림 행에 배치한 HWPX를 한컴2024에서
실제로 HWP로 저장하고, **그 저장된 바이트**를 독립 Print PDF로 인쇄했다.
검증에 사용한 작은 HWP와 그에 대응하는 PDF를 모두 저장소에 보존한다.
[입력 설명](../../tests/fixtures/issue_7751_color_controls/README.md)을 따른다.
편람 원본의383쪽 피델리티는 이 합성 색조 대조군과 별도로 검증한다.

최초 수동 HWPX는 편람의 구역·스타일·그림 틀을 재사용했으며 사용하지 않는 자원과
저장 줄 가정이 남아 있었다. Native/fresh WASM의90% gate를 통과하지 못해 정식
fixture로 등록하지 않았다. 실패 입력·이전 PDF·SHA·실패 TSV는 로컬 검증 증거에
보존하고 실제 한컴 저장으로 만든 **새 입력**을 사용한다. 새 저장본의 통과를
이전 HWPX의 일치로 표시하지 않는다. 아래 기본/교차/경계3개 PDF의 내장 이미지
각10개는 이전 PDF와 RGB 바이트가 전부 같음을 직접 확인했다. 문서 배치·PDF SHA는
새 저장본의 것으로 바뀌었으며, RGB 관측값은 그대로 유효하다.

| 이름 | 입력 HWP / SHA-256 | Print PDF / SHA-256 | 한컴 저장 job | 한컴 인쇄 job |
| --- | --- | --- | --- | --- |
| `hancom-color-probe-2024` | [22016bytes](../../tests/fixtures/issue_7751_color_controls/hancom-color-probe-2024.hwp) / `e85d3a4b22ac9ce0515a2bf5702aafbc2259a3562e086fd523b7a05b005dd015` | [18660bytes](hancom-color-probe-2024.pdf) / `8b4420d8caa04c822f47e67fc9edff4b3e447d64c15cac487b4bbd26e1ea6269` | `24950aa3-8c7c-4569-88e4-c04c54efc8f8` | `2c887b54-deec-45c6-a44e-843cb998648d` |
| `hancom-color-probe-cross-2024` | [21504bytes](../../tests/fixtures/issue_7751_color_controls/hancom-color-probe-cross-2024.hwp) / `89a16e13e967b8ad9145cf99367f79af8c5014d4e3e09e9689f19c6ffba5655c` | [18030bytes](hancom-color-probe-cross-2024.pdf) / `c74563caae874002ceb0976db9af136f00bcbce8ba38a295113ce9d6d5f2b967` | `a9b9feab-09ca-467d-b7e2-8cbfe1b4de8b` | `6108035c-a499-4979-80d2-14cec9e7c7a1` |
| `hancom-color-probe-boundary-2024` | [16384bytes](../../tests/fixtures/issue_7751_color_controls/hancom-color-probe-boundary-2024.hwp) / `e40cfacea9220e44e1281e3a3841648f92d6b6098d20040cfce4497c3f369375` | [9854bytes](hancom-color-probe-boundary-2024.pdf) / `2ead7978b7f7210d8050d737f7ae318d9a74897b0395a4d162c5413e21c4af3e` | `8bb12e93-cda9-4e12-a886-a28457ed9130` | `b8a04582-e2fd-4b58-9ac3-1fa477295af0` |
| `hancom-blackwhite-zero-2024` | [22016bytes](../../tests/fixtures/issue_7751_color_controls/hancom-blackwhite-zero-2024.hwp) / `58c10239a1185abe20b3655c661b13f73977ede222bef918ac33c4ce331e166d` | [18686bytes](hancom-blackwhite-zero-2024.pdf) / `73d64173e0d8e106929adfc0fd9d3c8ab393ceea981872eed6329e63c513c69b` | `22f09ae7-1254-4d04-8902-f43c73b22a00` | `bca39cec-5c20-4eb1-bdb9-4f424fb59592` |
| `hancom-blackwhite-axis-probe-2024` | [16384bytes](../../tests/fixtures/issue_7751_color_controls/hancom-blackwhite-axis-probe-2024.hwp) / `e45bf1d4e57802a39496f3b993495bbb1400ac965df19250237d2c943b6ee464` | [9034bytes](hancom-blackwhite-axis-probe-2024.pdf) / `9c84d7ca1f199d213f71cb74420bd6017d1970e7fe769c27f5d71880c3c60a08` | `14aa270b-90e3-4834-901b-a1ddaa228b9d` | `47e0474b-693c-4dfa-83d9-9bad27242917` |

## 등록 시 검증

5개 입력은 각각1쪽이며 Native/fresh WASM의 전쪽 raw 최저가 모두100%였다.
독립 PDF에서 추출한 RGB 패치550개도 두 backend 모두 채널별±1 이내다.
실제 Canvas 브라우저 검사5건이 통과했고 축 대조군의 효과·B/C 저장값도 확인했다.
Native RGB 계약 검사2건도 실행했다.
흑백 계약은 수정 전 B10/원본100에서26을 출력해 실패했고 수정 후0으로 통과했다.
일반 회색조 Canvas는 수정 전 검정 패치를 워터마크212로 출력했으며 수정 후0이다.
실행 로그·TSV·중간 JSON은 ignored output에만 보존한다. 최종 PR code head의
편람 전쪽 검증과 CI는 별도로 판정한다.

## RGB 관측값

`pdfimages -png <PDF> <prefix>`로 PDF 내 이미지 픽셀을 직접 추출했다.
아래 기대값은 그 픽셀의 측정값이다. 구현의 색조식을 다시 계산한 값이 아니다.
각 행의 대표 입력 RGB는 `(80,100,120)`이다. ±1 차이는 한컴의 정수 양자화 범위다.

| Print | 밝기 | 대비 | 효과 | 관측 RGB |
| --- | ---: | ---: | --- | --- |
| 기본 | 0 | 0 | REAL_PIC | `(80, 100, 120)` |
| 기본 | 8 | 0 | REAL_PIC | `(100, 120, 140)` |
| 기본 | 0 | 8 | REAL_PIC | `(76, 97, 119)` |
| 기본 | 20 | 0 | REAL_PIC | `(131, 151, 171)` |
| 기본 | 0 | 20 | REAL_PIC | `(68, 93, 118)` |
| 기본 | 20 | 20 | REAL_PIC | `(125, 150, 175)` |
| 기본 | -20 | 0 | REAL_PIC | `(29, 49, 69)` |
| 기본 | 0 | -20 | REAL_PIC | `(89, 105, 121)` |
| 기본 | 0 | 50 | GRAY_SCALE | `(64, 64, 64)` |
| 기본 | 10 | 0 | BLACK_WHITE | `(0, 0, 0)` |
| -cross | -20 | -50 | REAL_PIC | `(65, 75, 85)` |
| -cross | 20 | -50 | REAL_PIC | `(142, 152, 162)` |
| -cross | -20 | -20 | REAL_PIC | `(43, 59, 75)` |
| -cross | 20 | -20 | REAL_PIC | `(135, 151, 167)` |
| -cross | -20 | 20 | REAL_PIC | `(10, 35, 60)` |
| -cross | 20 | 20 | REAL_PIC | `(125, 150, 175)` |
| -cross | -20 | 50 | REAL_PIC | `(0, 0, 35)` |
| -cross | 20 | 50 | REAL_PIC | `(108, 148, 188)` |
| -cross | 0 | 50 | GRAY_SCALE | `(64, 64, 64)` |
| -cross | 50 | 0 | BLACK_WHITE | `(255, 255, 255)` |
| -boundary | -100 | -100 | REAL_PIC | `(0, 0, 0)` |
| -boundary | -20 | -100 | REAL_PIC | `(0, 0, 0)` |
| -boundary | 0 | -100 | REAL_PIC | `(1, 1, 1)` |
| -boundary | 20 | -100 | REAL_PIC | `(52, 52, 52)` |
| -boundary | 100 | -100 | REAL_PIC | `(255, 255, 255)` |
| -boundary | -100 | 100 | REAL_PIC | `(0, 0, 0)` |
| -boundary | -20 | 100 | REAL_PIC | `(0, 0, 0)` |
| -boundary | 0 | 100 | REAL_PIC | `(0, 0, 0)` |
| -boundary | 20 | 100 | REAL_PIC | `(51, 51, 51)` |
| -boundary | 100 | 100 | REAL_PIC | `(255, 255, 255)` |

회색조·대비50의 원색 빨강 `(255,0,0)`은 `(24,24,24)`다.
회색조를 바이트로 먼저 확정한 다음 대비를 적용한 출력이며, 기존 SVG의
linearRGB·바깥 회색조 중첩과 다르다. 대비±100은 중간 범위의 극한과 달라
기본값1 또는 원본128 초과의 이산 양자화 뒤 밝기를 적용한다.

## 흑백 임계점과 보정 순서

네 번째 입력은 기본 대조군에서 마지막 흑백 행의 밝기만0으로 바꾼 뒤 한컴으로
다시 저장·인쇄했다. 독립 PDF의 원본 회색128은 검정,170은 흰색이다.
밝기10에서는 원본100은 검정,128은 흰색이다. 밝기50에서는 원본검정이 검정으로
남고50과 `(80,100,120)`은 흰색이다. 회색 바이트에 색조를 적용한 뒤128을 초과하는
경우만 흰색으로 이산화한다. 먼저 이산화하거나 마지막 밝기를 CSS 배율로 적용하면
이 결과와 다르다. 알파를 워터마크 투명도로 바꾸지 않는다.

추가 axis 입력은 흑백 밝기−100/−50/−20/0/10/50/100과 대비−100/−50/100을
서로 독립된10행에 저장·인쇄했다. 같은 순서와 이산 임계점을 유지하며 음수와
끝값에서도 두 backend의110개 패치가 각각 독립 PDF와±1 이내였다.

## 재확인 명령

```bash
pdfinfo pdf/issue7751/hancom-color-probe-2024.pdf
pdfimages -png pdf/issue7751/hancom-color-probe-2024.pdf output/pr-review/color-check
node --test scripts/tests/picture-tones.browser.mjs
node scripts/run-rust-test.mjs issue_7751_raster_picture_tones -- --cargo-profile release-test --features native-skia --target-dir target/pr-review
```

변환 호출은 [한컴2024 MCP](../../mydocs/manual/mcp_hwp2024Convert_usage.md)를 따른다.
인증 값은 기록하지 않는다. 이 RGB 대조는 Native PNG·Canvas·SVG·실제 Studio DOM의
색 계약이다. 13단계의 공통 producer에서 이미 PNG로 확정한 일반 그림 색조는
Direct PDF로도 실제 변환해 위5개 입력 모두 raw100%를 확인했다.
아직 bake되지 않은 이산 흑백 효과, 미지원 legacy 워터마크 조합과 패턴의 기존
명시적 거부는 유지한다. 이5개 대조군의 통과를 모든 그림 조합의 지원으로 확대하지 않는다.
