# semanticist21 누적 검토의 독립 Print PDF

원본 입력별 한컴 Print 경로를 `start → status → download`로 실행했다. 입력과 PDF의
SHA-256·바이트 수·한컴 버전·실제 Print 모드·전체 쪽수·글꼴 공급 관측은
[print-provenance.json](print-provenance.json)에 기록한다. 모두 50 MiB 미만이다.

머리말 HWP/HWPX는 저장 metadata에 따라 engine 2020을 사용했다. 나머지는 기여자의
한컴 생성 대조군 설명에 따라 engine 2024로 출력했다. 원본 저장 제품이 미상인 범위는
metadata가 확인됐다고 주장하지 않는다. 이 PDF 확보는 후보의 시각 검증 완료를 뜻하지 않는다.

[검토 계획](../../mydocs/pr/semanticist21_20261103_review_plan.md)의 기준 devel·후보를
같은 원본·PDF·글꼴·DPI로 각각 비교한다. 편집 결과는 별도 저장본과 Print를 연결한다.
