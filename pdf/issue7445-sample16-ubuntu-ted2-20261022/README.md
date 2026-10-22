# ubuntu-ted2 제공 sample16 Print PDF

사용자가 2026-10-22에 지정한 `ubuntu-ted2:/home/tsjang/rhwp/pdf/`의 파일을 복사했다. 전송 전 원격 SHA-256과 복사 후 로컬 SHA-256이 일치한다. 이전 `pdf/`의 동명 PDF는 보존했다. 세 파일 모두 Hancom PDF producer, 64쪽이다. 사용자 제공 Print 원본이며 파일 메타데이터만으로 MCP job·Print method·저장 환경 전체를 추정하지 않는다.

| PDF | 대응 입력 | PDF SHA-256 |
| --- | --- | --- |
| [hwp3-sample16-hwp5-2022-hwp-2020.pdf](hwp3-sample16-hwp5-2022-hwp-2020.pdf) | `samples/hwp3-sample16-hwp5-2022.hwp` | `c4e05635726869d737f1640db4e4f29c868df9e1c31e350b0b06e115fd210b33` |
| [hwp3-sample16-hwp5-2024-hwp-2024.pdf](hwp3-sample16-hwp5-2024-hwp-2024.pdf) | `samples/hwp3-sample16-hwp5-2024.hwp` | `62745aebf90af452bf278f8cfb97c4d523e854496a998279d0e88c7a9a3d9c9b` |
| [hwp3-sample16-hwp5-hwpx-2024.pdf](hwp3-sample16-hwp5-hwpx-2024.pdf) | `samples/hwp3-sample16-hwp5.hwpx` | `be4a0286d3ad5209a1a0787fd4ff691e110ddbcf7bde3a52a4547958e452a1ee` |

## 입력 구분

- 2022 HWP: `fec3671c9a83cb8388cbdaab38754cb17a8da54d0314ca7e3f56860e871506fe`.
- 2024 HWP: `f737ee22888b0a11171622b3e1c12d71d0f42cf066f729d3c98dde9ca6cc122d`.
- HWPX: `49e3e809eb41e22b2c059383db32b0cf038787269b5c523d1ff59d1a52b4340c`.
- 원격 호스트의 대응 입력과 로컬 입력도 각각 같은 SHA-256이다.
- 기존 #8064의 별도 `hwp3-sample16-hwp5.hwp`는 `6a3cdf2c148bf39f40ab06e847767e7f722543fce59e6ee4328ef44b835f3f45`이며, 위 2024 HWP로 대체하지 않는다. CFB의 DocInfo·BodyText/Section0 바이트도 서로 다르다.

## 기존 동명 기준과의 전쪽 대조

현재 canonical PDFium 153.0.7999.0, 96DPI로 양쪽 PDF를 새로 래스터화했다. 2024 HWP와 HWPX는 각각 전64쪽 RGB 픽셀이 동일하다. 2022 HWP는59쪽이 달라졌고 PDF 대 PDF raw2px 최저는98.92725%(14쪽)다. 파일 SHA의 차이만으로 같거나 다른 화면이라고 판단하지 않았다.

## 현재 후보의 전쪽 정량 결과

`e236f80262eb6521dced7f705b62b66d1486fc74`와 기록된 source patch를 검증했고 동일 source는 로컬 커밋 `8c8a59ec9e`에 보존됐다. Native/fresh WASM 각3조건192쪽, 각64/64쪽·90% 미만0이다. 전쪽 TSV 원문은 `mydocs/pr/assets/issue7445/completed-20261009/rebase-605ccfd9-20261022/consumer-resolution/ubuntu-ted2-reference/`에 있다.

| 경로 | 입력 | 최저 raw2px | 최저 쪽 |
| --- | --- | ---: | ---: |
| native | hwp3-sample16-hwp5-2022-hwp | 96.25320% | 40 |
| native | hwp3-sample16-hwp5-2024-hwp | 97.09573% | 2 |
| native | hwp3-sample16-hwp5-hwpx | 95.13828% | 18 |
| wasm | hwp3-sample16-hwp5-2022-hwp | 96.16423% | 40 |
| wasm | hwp3-sample16-hwp5-2024-hwp | 97.03480% | 2 |
| wasm | hwp3-sample16-hwp5-hwpx | 95.14423% | 18 |

Mac WASM은 locked web/no-opt 로컬 대체 빌드다. Docker 최적화 통과가 아니다. 이 결과는 전쪽 정량 결과이며 확대 판독·새 base 대비 보존·남은 전체 회귀·실제 Studio 소비 경로 및 전체 PR 검증 완료를 대신하지 않는다.
