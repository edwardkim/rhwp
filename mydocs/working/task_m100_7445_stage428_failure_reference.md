# #7445 회귀 실패 문서의 독립 기준 보완

Issue: #7445. 전체 회귀 27개 실패 중 `#7413` 검사 입력의 대응 기준 PDF를 확보했다.

- 원본: `samples/issue5699/20099369_yeongwol_forms.hwp`, SHA-256 `6d723789148699479983fbcb6fa6368a252c1187fe0a7b3c922b583f6727ec09`.
- 독립 기준: `pdf/20099369_yeongwol_forms-2020.pdf`, 4쪽, 91,696 bytes, SHA-256 `01682a87a130c0f1d713afa74c4c54b74ef2c0200395b5184d4c31a9f635335b`.
- 저장 제품 메타데이터는 미상이다. 지침에 따라 engine 2020을 명시했고, 한컴 `11.0.0.9136`의
  `pdf_print_method=0`, `hancom2020_pdf_driver_one_up` 인쇄 경로와 다운로드 success를 확인했다.
- 이 기준을 Native/fresh WASM으로 대조하기 전 좌표 선택·줄 수 기대값을 변경하지 않는다.
- 위첨자 3쪽 원본은 새 job에서도 `HwpEditorFrame.SaveAsDocument returned false for HWP`로 실패했다.
  대응 Print PDF는 미확보이며 시각 미달이나 검사 오류로 확정하지 않는다.

기존 출력 원본과 회귀 검사는 보존했다. 이 단계는 기준 자료 확보이며 피델리티 통과를 의미하지 않는다.
