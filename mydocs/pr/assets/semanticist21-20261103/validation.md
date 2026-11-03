# 누적 검토의 로컬 검증

현재는 최종 전체 검증 전이다. 이 커밋은 중간 제품 head `25607fc28d`에서 실제 측정한 입력·Print·전쪽 TSV·PNG를 보존한다.
중첩 표 저장 높이의 마지막 보정 이후 최종 Native/fresh WASM을 다시 검증하며, 그 결과와 이 중간 측정을 구분한다.

- 30개 입력·70쪽을 Native/fresh optimized WASM 각각 전쪽 비교했다. 최저값은 분할 행 문서의 92.88%다.
- notice의 실제 53개 API 호출은 live 24쪽에서 Native 97.71%, WASM 97.46% 이상이다. 실제 두 backend의 저장본을 교차 재열기한 24쪽은 Native 97.74%, WASM 97.46% 이상이다.
- form-01은 HBATANG 프로그램을 명시 공급했다. 같은 공급의 base 양쪽 100.00%에서 후보 Native 99.60%, WASM 99.58%다. 잘못된 빈 2022 PDF와 광범위 글꼴 경로의 카탈로그 누락은 판정에서 제외한 진단이다.
- 머리말, 회전 도형, grow3, 중첩 표, 공개 편집 상태의 독립 Print를 실제 비동기 start/status/download로 확보했다. 원본별 engine·실제 Print 방법·입력/PDF SHA는 [출처 원장](../../../../pdf/semanticist21-20261103/print-provenance.json)에 있다. 변환 자격 증명은 포함하지 않았다.
- [전쪽 측정](visual-results.json), [직접 확대 판독](direct-visual-review.json), [공급 글꼴](font-supply.json)은 점수와 판독을 구분한다. 실루엣 100%도 동일 픽셀이라는 뜻은 아니다.
- 최소 글꼴 환경의 정규 집중 회귀 77건에서 74건이 통과했다. 새 경계 검사 세 실패는 중첩 저장 출처 누락, 머리말 테스트의 JSON 키, 회전 외곽 폭을 원래 local 폭으로 오인한 테스트였다. 마지막 작은 경계 진단 8건은 보정 후 모두 통과했다. 진단용 opt-level=0 실행은 최종 정규 release-test를 대신하지 않는다.
- 같은 공개 계약 검사를 고정 base의 보존 Native rlib로 직접 컴파일한 음성 대조는 36건 중 13건 통과·23건 실패였다. 원 기여의 과거 CI와 누적 보정 후보의 검증을 구분한다.

새 원본·독립 Print·TSV는 공식 경로에 보존하며, 글꼴 파일이나 사적 corpus를 넣지 않는다. 원격 push·공개 댓글·merge는 아직 수행하지 않았다.
