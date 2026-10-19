# #7642 독립 진단 입력과 한컴 출력 보존

수동 합성 진단 원문과 이를 한컴에서 정상 저장한 HWP, 저장본의 Print PDF를 함께 보존한다.
합성 원문과 한컴 저장본은 서로 다른 입력이며 저장본의 통과를 원문의 통과로 대신하지 않는다.
파일별 byte 수와 SHA-256은 [file-inventory.json](file-inventory.json)에 있다.

| 경로 | 출처와 기존 분석 |
| --- | --- |
| `pdf/issue7642-negation-contract/` | 한컴 2024 13.0.0.3901, Print method 0. Stage 40·81의 저장/Print 증거 |
| `pdf/issue7642-not-operand-diagnostic/` | 같은 한컴 2024. Stage 82의 정상 저장·Print job 및 20개 수식 소비 규칙 |
| `pdf/issue7642-not-glyph-frame/` | 같은 한컴 2024. Stage 83의 20개 글자 크기·사선 장평 진단과 Print job |
| `pdf/issue7642-hft-char-style-diagnostic/` | 한컴 2020 11.0.0.9136, Print method 0. Stage 92·93의 Type3 폭·굵기·자간 근거 |

이번 보존 커밋은 기존 출력의 출처를 고정한다. 현재 head의 새 시각 검증 완료나 새 회귀 추가를 뜻하지 않는다.
TSV, 실행 로그, 중간 JSON과 nextrun은 ignored `output/`에 유지한다.
