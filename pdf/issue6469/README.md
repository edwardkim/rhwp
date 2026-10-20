# WMF 구조 보존 원본의 한컴 2020 Print

Issue: #7642

입력: `samples/issue6469/wmf_fill_shapes.hwpx`.
입력 SHA-256: `3834193d41c41a0bc71c68cae51194e3fc40b913b2de9461253653d42afb7648`.
구조 보존 슬라이스의 전체 15쪽을 내용 변경 없이 직접 Print했다.

- engine: Hancom Office 2020, 실제 `11.0.0.9136`
- Print method: `0`, `hancom2020_pdf_driver_one_up`
- job: `3b42e66e-654c-4e9e-b638-8d10cc6df1f3`
- 원본 용지·방향·1-up 유지
- [wmf_fill_shapes-2020.pdf](wmf_fill_shapes-2020.pdf): SHA-256 `4270352ba27bbf885a8aca6289f6e147ff419665dd2bbf2f7eee77f3ce548e4f`

MCP 실행 기록은 ignored output의 `stage173-wmf6469-auto-print.json`에 보존한다.
[현재 Native/fresh WASM 결과와 보류 영역](../../mydocs/pr/assets/issue7642-wmf-independent-print-20261012/README.md).

## 같은 원본의 한컴 2024 교차 진단

기본 기준인 위 2020 Print는 유지한다. 별도 버전의 출력 차이를 확인하려고
같은 입력을 내용 변경 없이 한컴 2024로 Print했다.

- engine: Hancom Office 2024, 실제 `13.0.0.3901`
- Print method: `0`, `frame_print_to_pdf_ex_one_up`
- job: `b73087a1-9b65-43b8-97cf-4d0568663b8d`
- [wmf_fill_shapes-2024.pdf](wmf_fill_shapes-2024.pdf): SHA-256 `ec87b7138a501b5d8f1beb5ae5c84d2225cccd17a8480579c7f992f42d65a198`

Stage 175 Native 전체 15쪽 최저 92.56896%, 미달·누락 없음이다.
이 교차 진단의 WASM 대조는 실행하지 않았다. 기본 2020 Print 대조는
Native/fresh WASM 각각 전체 15쪽 최저 92.83091%이며,
[새 내용 보존 검사와 확대 판독](../../mydocs/pr/assets/issue7642-wmfc-content-20261012/README.md)에 연결한다.
