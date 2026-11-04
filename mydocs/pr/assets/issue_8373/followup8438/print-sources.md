# 일괄 검토용 한컴 2024 Print 기준

두 PDF는 같은 실제 입력을 HWP 2024 MCP 비동기 `start → status → download`로 전송해 생성했다.
상태 결과의 한컴 버전은 13.0.0.3901, `pdf_print_method`는 0,
`pdf_output_mode`는 `frame_print_to_pdf_ex_one_up`이다. PDF raster는 96 DPI로 비교한다.
완료 상태와 다운로드 SHA-256을 확인했으며 endpoint·토큰·결과 capability는 기록하지 않는다.

| 입력 | 입력 SHA-256 | Print PDF | PDF SHA-256 | 작업 ID |
| --- | --- | --- | --- | --- |
| [PDF 편집 가져오기 저장 HWP](../../issue_8381/followup8438/editable-saved.hwp) | `edaf8172427572450503f3265fca95b1c1d2c9673c1e5b25528f5b55294a043f` | [16쪽](../../../../../pdf/issue8361/batch-editable-saved-2024.pdf) | `7ec4f3611e480160dfd1cf697671dd966e126b3ef7a6c4ac96cbf62bce758e65` | `c803d472-4513-49f6-ba12-329481840c5b` |
| [HWP에서 저장한 HWPX](sample2-saved.hwpx) | `c5149ebea0e34d78103e8b81def04e4a3cdc72aa9f9bb97dab222cad84dba9ca` | [29쪽](../../../../../pdf/issue8373/batch-8378-sample2-2024.pdf) | `0b017e9624ac4ac7fea1809ec27ec947a4497035785cd79620563bfd0dee7fc3` | `c7eb2334-d7fa-403b-b6bf-2680622e0b08` |

HWPX의 원본은 `samples/hwpx_sample2.hwp` SHA-256
`ca66d521d070ac8442e09fd3f283e17611eb9efb45eb4bf6c3a8f65e604f6b5f`이며,
통합 제품 코드 `d536455b4e`에서 `rhwp export-hwpx --verify-pages`로 저장했다.
이 HWPX는 이전 원 PR에서 인쇄한 HWPX와 해시가 달라 기존 PDF로 대체하지 않았다.
위 입력과 기준은 진행 중인 일괄 시각 검증의 자료이며, 파일이 존재한다는 사실만으로
검증 통과를 판정하지 않는다.
