# 정상 저장된 칸 내부 어울림 표

기존 #8045의 수동 재현을 한컴 2020에서 HWP로 저장하고 다시 HWPX로 저장했다.
그 HWPX를 같은 한컴의 Print로 출력한 PDF와 비교한다. 수동 LineSeg 원본과
같은 바이트라고 보고하지 않는다. 그 원본과 차이는
`mydocs/pr/assets/source_pr8045/manual_cache_reference/README.md`에 남겼다.

| 파일 | SHA-256 |
| --- | --- |
| `square-hancom2020.hwpx` | `a85aa1a67089027180c9772d8285b5d39d2eb345a4cf706ab2bd380616f5c361` |
| `pdf/pr8045_review/square-hancom2020-2020.pdf` | `3a429a93a009f44c275cc0394caef62296e706fc6e224b2a63ce2333fd1e3843` |

Print는 engine 2020·Hancom 11.0.0.9136·`pdf_print_method=0`·
`hancom2020_pdf_driver_one_up` 경로다. 전체 1쪽을 96dpi·print 프로필·
`--compat 2022`로 비교했다. Native255/fresh WASM073 raw는 각각 100%다.
review·overlay에서 부모 표 높이·메모 원점·옆 표와의 배제를 직접 판독했다.
얇은 괘선의 래스터 농도 차이는 남으며 raw 100%를 픽셀 완전 일치로 부르지 않는다.

한컴 저장 메모 줄의 `column_start=20100`, `vertical_pos=1600`과 부모 셀의
원본 높이를 관계 검사에 사용한다. 독립 PDF 메모 첫 글자 원점은
(257.500427246, 111.238433838)pt이며 96/72 환산 뒤 ±2px로 보호한다.
내용 단일 소유·부모 포함·어울림 표 오른쪽 배제도 함께 검사한다.

기존 재현의 정식 보존 경로이며 동적 corpus에 추가하지 않는다. 생성·수정 전후
검증은 `mydocs/working/task_semanticist21_20261026_inline_wrap_contracts.md`에 있다.
