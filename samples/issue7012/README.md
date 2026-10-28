# #7012 세로 가운데 정렬 중첩 표 재현 문서

- 출처: https://github.com/edwardkim/rhwp/issues/7012
- 고정 원본: https://github.com/kidsnote/rhwp/blob/4d1f1c56e1588b127995848bbd7e5a32c603af1f/samples/synth_cell_nested_float_lead_center.hwp
- 원 작성자는 실문서의 한글 음절 치환, 이미지 더미화, 기하 보존으로 익명화한 fixture라고 설명했다.
  해당 공개 파일을 바이트 변경 없이 보존했다.
- SHA-256: `3fcd280bf793e4205c76fcf15bddcbe1337a09e05db1989d9ee9cd9fe39557c1`
- Git blob: `49b76c162ecccc6e81a75958cb7e51ba576b0767`, 79,872 bytes.
- 저장 metadata: HWP5, Hancom Office 2020 `11.0.0.8808`, 2쪽.
- 검증 영역: 실제 2쪽의 `valign=Center` 칸, 마지막 문단의 문단 기준 자리차지 2x5 중첩 표.
- [한컴 기준 PDF](../../pdf/synth_cell_nested_float_lead_center-2020.pdf)는 MCP engine 2020으로 생성했으며
  PDF의 Creator는 Hwp 2022, Producer는 Hancom PDF, PDF version은 1.6이다.
- 검토 기록과 오늘할일은 최초 Full CI 성공 후 같은 PR의 Markdown-only trailing commit으로 추가한다.
