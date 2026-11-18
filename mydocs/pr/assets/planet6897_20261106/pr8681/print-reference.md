# PR #8681 관련 결재문서 36478532의 독립 Print 기준

[원 PR #8681](https://github.com/edwardkim/rhwp/pull/8681)과 [이슈 #8686](https://github.com/edwardkim/rhwp/issues/8686)의 칸 안 자리차지 표·뒤 본문 잘림을 검토하기 위한 원본이다. 원 PR의 주 입력 36385506과 구분한다.

- 원본: [36478532-original.hwpx](36478532-original.hwpx). 사용자 지정 `korea_downloads/samples/강북소방서 미아119안전센터/36478532_결재문서본문.hwpx`를 내용 변경 없이 복사했다.
- 원본 SHA-256: `7a4cedd8e891e9be6b95c9dab4f7236fea6521b08ccb24d67e177e6a0975773b`, 27465바이트.
- 저장 제품: `hancom-office-2020`, 저장 버전 `11.0.0.8227`(`rhwp info --json`의 메타데이터).
- 기준: [36478532-original-2020.pdf](../../../../../pdf/planet6897-20261106/pr8681/36478532-original-2020.pdf), 2쪽, 81627바이트.
- PDF SHA-256: `c1909506b49c5ce201454e23024bf32e02ae3248a904937955e1962bb494c9ca`.
- 생성일: 2026-11-06T17:53:53.453249+09:00. MCP `start → status → download`, 입력 형식 HWPX, **결과 형식 PDF**.
- 실제 engine `2020`, 한컴 버전 `11.0.0.9136`, `pdf_print_method: 0`, `pdf_output_mode: hancom2020_pdf_driver_one_up`. 한 쪽씩 인쇄하며 용지·배율을 재지정하지 않았다.
- job ID: `3939478d-b18a-4c6f-ab02-4d950e1ca0da`. 실제 결과 바이트의 SHA를 client와 다시 대조했다.

원본과 독립 기준을 확보한 단계다. 이 PDF와 같은 원본의 Native/fresh WASM 전쪽 TSV·쪽수·확대 판독은 아직 완료하지 않았다. 기존 원 PR의 측정값을 현재 통합 head의 검증 결과로 사용하지 않는다.
