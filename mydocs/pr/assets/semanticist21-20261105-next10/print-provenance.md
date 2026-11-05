# 독립 한컴 Print PDF 보완 자료

검증일: 2026-11-05 KST. 원 PR의 Mac 저장 입력은 변경하지 않았다. 파일로 된 Print 기준이 없던 미주·문단 머리·계산식 사례를 Windows 한컴 인쇄로 보완했다. 비동기 `start → status → download`를 사용했고, 다운로드 bytes와 SHA-256을 각각 대조했다. 서비스 주소·인증 정보·원시 응답은 공개 증적에 포함하지 않는다.

- 실제 실행 engine: **2020**, 한컴 **11.0.0.9136**.
- 실제 Print 계약: `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`.
- 원 HWPX `version.xml`의 `appVersion`은 `12.30.0.6544 MAC64LE Darwin_25.5.0`이다. 제품 `info`의 `lastSavedWith`는 null이어서 실제 사용한 Windows 2020 fallback을 명시한다. Mac 출력과 동일한 버전이라고 주장하지 않는다.
- 8개 모두 50 MiB 미만이며 검증 원본과 함께 정식 Git 경로에 보존한다. 공개된 기여 fixture만 서비스로 전달했다.

| 입력 | 입력 SHA-256 | 독립 Print PDF | bytes | PDF SHA-256 |
| --- | --- | --- | ---: | --- |
| `tests/fixtures/endnote_numbering/restart-5.hwpx` | `e88a8288407f7401cad44b7d403646e11cc2a6e4667b8c05e79b6a8cde20c3b9` | [endnote-restart5-2020.pdf](../../../../pdf/semanticist21-20261105-next10/endnote-restart5-2020.pdf) | 21220 | `3624a60d1926c7fc8e674856b050b3aa8beaaa1face0b008567746743936b3cf` |
| `tests/fixtures/endnote_numbering/continue.hwpx` | `467ae3882535aaddc4a4ced4c05ad0f154f27d8cb4bc6addb0cf69c9af76a2d8` | [endnote-continue-2020.pdf](../../../../pdf/semanticist21-20261105-next10/endnote-continue-2020.pdf) | 21108 | `2db2fb60d386267efcf357df85824416b788ee14ee444ac3f2935e2b24f5518b` |
| `tests/fixtures/paragraph_head/hancom-heads.hwpx` | `d73699eb4e0b536496d6d2d6d3e0e0f9f68816e38cb8891457c61c239baff808` | [paragraph-heads-2020.pdf](../../../../pdf/semanticist21-20261105-next10/paragraph-heads-2020.pdf) | 24547 | `d2f391d16174ee582bc80f45c04fa5966a984819cc58d0a8234fc945f64cf012` |
| `tests/fixtures/table_formula/easy.hwpx` | `b46636826c13cc09045a0097dd079580914244ddd8db4ce66f507a770583aed0` | [table-formula-easy-2020.pdf](../../../../pdf/semanticist21-20261105-next10/table-formula-easy-2020.pdf) | 11680 | `d46e01eebbf388dabd4346ad6513ab62fd3db5a2c24c3609eb75cc5b68bb5533` |
| `tests/fixtures/endnote_numbering/new-number-20.hwpx` | `437ef87552f9c637d4f328ec7bb665deac3bd0ebd544be621d1473804fd4a0c4` | [endnote-new20-2020.pdf](../../../../pdf/semanticist21-20261105-next10/endnote-new20-2020.pdf) | 20850 | `5d586112655432b70407a8f8633d5ec9958ad9b1681ed164bcb0e0c726ac31f3` |
| `tests/fixtures/table_formula/hancom.hwp` | `d8ac4dd6db6d4c73714c2c7519898f1398552bb840325f9782f59e10a94db903` | [table-formula-hancom-result-2020.pdf](../../../../pdf/semanticist21-20261105-next10/table-formula-hancom-result-2020.pdf) | 14308 | `f6b36077ebbfe982544a151bb49592707b8f589cc31db863e2acf0f9bb1111d4` |
| `tests/fixtures/table_formula/stale.hwpx` | `e11b4ee37a4ab016cbd7056e725ccf89e2a3448e68ba92da18c39c4442ad8177` | [table-formula-stale-2020.pdf](../../../../pdf/semanticist21-20261105-next10/table-formula-stale-2020.pdf) | 12047 | `a767229bca103363f79e0e0f8d80110401df0f98d868a2cd59f6b278780f796e` |
| `tests/fixtures/table_formula/cycles.hwpx` | `737c454944b3551d9cddcb759b2337093dd1d879eac525d7a218fa3e75704e65` | [table-formula-cycles-2020.pdf](../../../../pdf/semanticist21-20261105-next10/table-formula-cycles-2020.pdf) | 9782 | `a87e4ded81122fd791696765ae2f19c7e7726877780a9d9f6050432b1e75d951` |

전쪽 점수와 직접 판독은 별도 최종 검증 기록에 연결한다. PDF 확보·다운로드 확인만으로 RHWP 출력의 통과를 판정하지 않는다.
