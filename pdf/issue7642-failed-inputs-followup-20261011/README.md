# #7642 실패 회귀 원본의 독립 Print 출력

`issue_6172_pua_boxed_number_advance`와 `issue_7413_cell_padding_shrinks_only_when_wrapping_overflows`의
원본 HWP를 변경하지 않고 MCP의 start → status → download로 출력했다.

- 원본: `samples/issue6172/2599643_port_call_form.hwp`, `samples/issue5699/20099369_yeongwol_forms.hwp`
- 저장 제품: 첫 문서는 한컴 2020, 두 번째는 저장 제품 정보 없음. 둘 다 engine 2020을 명시했다.
- 실제 제품: 한컴 2020 11.0.0.9136, `pdf_print_method: 0`, `hancom2020_pdf_driver_one_up`.
- 원본·PDF SHA-256, 전체 쪽수, 실제 job과 Print 경로는 [print-provenance.json](print-provenance.json)에 있다.

출력 성공은 RHWP 시각 검증 통과와 구분한다. 새 head의 전체 Native/fresh WASM 대조·확대 판독 뒤에만
필요한 새 이름의 관계형 검사를 추가한다. 로그·TSV와 작업 상태 원장은 ignored output에 둔다.
