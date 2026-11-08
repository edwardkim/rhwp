# 배포 문서의 저장 버전에 맞는 Print 기준

원본은 `samples/task1768/distribution_doc.hwpx`다. 확장자는 HWPX지만 실제 내용은 2018 저장 HWP5 CFB다. 첫 요청이 실패해 동일 바이트·동일 SHA의 `distribution_doc.hwp` 전송 별칭을 사용했다. 원본 내용이나 저장 줄을 바꾸지 않았고 service도 `input_preprocess: none`을 반환했다.

MCP `engine: 2020`, 한컴 `11.0.0.9136`의 PrintMethod 0·`hancom2020_pdf_driver_one_up`으로 전체 3쪽을 인쇄했다. 입력·PDF SHA와 job은 [출력 근거](print-provenance.json)에 있다. 기존 2024 PDF는 별도 버전 진단 자료로 남긴다.

PDF 확보만으로 시각 통과를 선언하지 않는다. Native/fresh WASM 전쪽 TSV와 직접 판독 후 해당 회귀의 소유·순서·포함 관계를 재검토한다.

## 위첨자 회귀의 독립 Print

`samples/issue5756/156732409_superscript_advance.hwp`는 2022 저장본이며 지침의2020 bucket을 사용했다. 원본 그대로 한컴11.0.0.9136의 PrintMethod0·한쪽씩 인쇄해 전체3쪽을 확보했다. [입력·출력 근거](superscript-print-provenance.json)를 따르며, 해당 원본의 전쪽 Native/freshWASM과 직접 판독 전에는 기존 px·sqrt 크기 검사를 바꾸지 않는다.
