# R30 실제 원본 5쪽 Native 선행 검증

원본은 `samples/basic/sungeo.hwp`, 기준은 기존 한컴 Print
`pdf/basic/sungeo-hwp-2020.pdf`다. 원본·PDF·공급 글꼴 환경·96 DPI·보정 전 2px raw 기준을
바꾸지 않았다. 제품은 리베이스 이전 `d0826a8a4b116da16396f0894b59ec8bc9a181c9`이며,
컴파일 소스 해시 원장과 실제 Native producer를 검토 output에 별도로 고정했다.

| 같은 원본 5쪽 | Native raw |
| --- | ---: |
| 고정 base `a7feb849e1` | 99.95% |
| R29 보정 전 | 98.93% |
| R30 유효한 0번 참조 보정 후 | 100.00% |

[선택 5쪽 TSV](p005-native.tsv)·[review](review_005.png)·[overlay](overlay_005.png)·
[동일 원점 확대](heading-detail.png)를 함께 보존한다. 마커와 본문 시작, 한자 뒤 괄호 및 한글의
원점 관계가 독립 Print와 맞게 이동했다. raw 100.00%는 픽셀의 완전 동일성을 뜻하지 않는다.
확대 영상에 보이는 획·자소 모양의 렌더 차이는 최종 확대 판독에서 별도로 확인한다.

최소 공개 글꼴 5종의 집중 회귀는 57건 모두 통과했다. 새 검사 3건의 수정 전 2 FAIL/
정상 대조 1 PASS 기록도 보존했다. **이 자료는 Native 한 쪽의 선행 결과이며 전 94쪽이나
수정 후 fresh WASM의 최종 통과 증거가 아니다.** 이후 새 `upstream/devel`을 리베이스해
그림 편집·기준선·그림 점유 높이 보존과 이 원본의 전쪽 검증을 계속한다.
