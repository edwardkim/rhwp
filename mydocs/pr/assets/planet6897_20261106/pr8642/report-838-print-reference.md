# report-838 보고서의 원본과 한컴 Print 기준

[원 PR #8642](https://github.com/edwardkim/rhwp/pull/8642)의 실문서 각주·표 이어받기 대조에 쓰는 원본이다.

- [원본](report-838-original.hwp): 228864바이트, SHA-256 `686146869f9eae267c1f2ac7d82c9fa2bc0615c36a727687a2e2cf0985b6d207`.
- 사용자 지정 `korea_downloads`의 `1430000-200600021_D0150014-2-002_2차 중간보고서.hwp`을 내용 변경 없이 복사했다. 원 PR에 공개된 원본 SHA 접두사와 일치한다.
- 저장 앱 메타데이터: `5.7.6.3030`, product `null`. 원 PR이 한컴 2020 Print 대상으로 지목한 구형 저장본으로 engine 2020을 명시했다. 이를 2024 저장본이라고 추정하지 않았다.
- [새 독립 Print PDF](../../../../../pdf/planet6897-20261106/pr8642/report-838-original-2020.pdf): 전체 109쪽, 1066495바이트, SHA-256 `c4fa3f7de6e276850daa648216346d5c4564a93dbcd4dbb10494b18c6ee71ed9`.
- MCP `start → status → download`, job `70f651a3-93ff-42a3-a3d7-35f2263aa859`, 요청/결과 형식 PDF. 실제 한컴 `11.0.0.9136`, engine `2020`, `pdf_print_method: 0`, `pdf_output_mode: hancom2020_pdf_driver_one_up`를 확인했다.
- 새 기준 확보일: 2026-11-06T09:52:55.743795+00:00. 원본 용지·배율을 재지정하지 않은 1-up 전체 문서 인쇄다.

기존 원 PR의 과거 PDF와 이 새 PDF는 SHA가 다르다. 과거 TSV를 이 새 기준의 통과 자료로 재표기하지 않고, 같은 새 기준으로 변경 전후 실제 출력을 비교한다. Native/fresh WASM 전쪽 TSV와 확대 판독은 아직 완료하지 않았다.
