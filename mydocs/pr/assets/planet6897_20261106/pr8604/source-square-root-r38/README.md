# 정책위원회 안건 11쪽 — R38 Native 실제 개선

제품 `09ada34bf1be3a3dcb60e36f2de016e8d35cfcd9`, 실제 검사 candidate `7f88708f08d14d47537ef34578c969dc057ee6e8`. Native 단계의 실제 결과다. 최종 fresh WASM 전쪽·전체 회귀·PR 승인은 별도로 완료해야 한다.

같은 원본과 기존 독립 한컴 Print를 96 DPI·Print profile·compat 2022·보정 전 2px 내용 실루엣으로 비교했다. 2020으로 끝나는 기존 PDF 파일명과 실제 한컴 2022 생성 메타데이터를 구분한다. PDF는 재산출하지 않았다.

| 범위 | 이전 Native | R38 Native |
| --- | ---: | ---: |
| 전체 14쪽 최저 | 96.52% | 98.18% |
| 11쪽 | 97.59% | 99.93% |

전체 쪽수 14 = PDF 14다. [전쪽 TSV](native.tsv) · [같은 11쪽 review](native-review-011.png) · [같은 11쪽 overlay](native-overlay-011.png). 두 U+221A 기호의 원본 전진, 뒤 닫는 괄호·문구 간격, 표 행과 후속 본문 순서를 실제 대조했다. 본문과 하단 주석의 서브픽셀 차이는 남으며 픽셀 동일로 표현하지 않는다.

- [원본](../../../../../../samples/issue8596/policy_committee_agenda.hwp), SHA-256 `b3b7c0d557a26d7080957fe5d8472153a6783b9dab029acae4e3b0c29186e4ad`
- [독립 Print](../../../../../../pdf/issue8596_policy_committee_agenda-2020.pdf), SHA-256 `b15da297882bf644e53e5b895fdb79d119dd03314e24f0fef660001904f4cd4f`
- Native producer SHA-256 `3941b00db042b95fdb348566f7e0dcabfdb25d40a21d7ebeca06f8dd480d2da0`
- 실제 TSV SHA-256 `87ad04b7e373aa567702fcb80ed2294e52483b2e3febf1c8acd3aded45571a84`

원인과 보정은 [R37 독립 메트릭 분석](../source-square-root-r37/README.md), [Git 복원·집중 검사 기록](../../r39-git-restoration-analysis.md)에 연결한다. 설치 OTF 프로그램은 새로 배포하지 않는다. 원본 메트릭의 1em 계약은 시스템 글꼴 없이도 관계형 회귀로 검사한다.
