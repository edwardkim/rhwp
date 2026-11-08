# 양식 개체의 혼합 글줄·캐럿 재현

기존 #8093 helper가 공개 문서 API로 빈 문서를 만들고 `가나` 사이의 수식 슬롯을
누름 단추로 바꾼 결과다. 개체 폭·높이는 7087·1984HU다. 한컴 저장본이라고 부르지
않는다. 같은 바이트를 한컴 2020 MCP Print로 출력해 독립 기준을 확보했다.

| 파일 | SHA-256 |
| --- | --- |
| `form-slot.hwp` | `1c7e40474066cfc7c4892ab8d3b2e6ae7dde7055335ae2330c772e1cdc159b6c` |
| `form-slot.hwpx` | `4c9a67733132501dc47b303285dc4051ecbd9cb1223f964f77927e266900666d` |
| `pdf/pr8093_review/form-slot-hwp-2020.pdf` | `fd36f88c6e8766ee36bf6dbfd36dea9defe33e6efa11aea18c0c53caa3f87817` |
| `pdf/pr8093_review/form-slot-hwpx-2020.pdf` | `d99caafedcd41355ebfaeee1fdf746bdb6da62fb096cd1bd03f270a1c10179e9` |

Print는 engine 2020·Hancom 11.0.0.9136·`pdf_print_method=0`·
`hancom2020_pdf_driver_one_up` 경로다. PDF는 각각 1쪽이다. Native255/fresh
WASM073의 같은 원본 전 1쪽 raw는 모두 100%다. 96dpi·print 프로필·`--compat 2022`로
비교했고 review·overlay에서 옆 글자의 기준선·개체 폭·앞뒤 위치를 확인했다.

원본 HWP는 이전 수식의 작은 합성 줄을 담고 HWPX는 줄이 없다. 두 경로를 함께
보호한다. 독립 PDF 첫 글자의 원점은 (85.033912659, 116.033195496)pt,
뒤 글자는 (165.630233765, 116.033195496)pt다. 이를 96/72로 환산한 ±2px 검사와
캐럿 순서·원본 폭·입력 위치·문단 분할 소유 관계를 함께 검사한다.

기존 재현의 정식 보존 경로이며 다른 문서의 동적 corpus에는 추가하지 않는다.
출처·생성·검증의 상세 기록은
`mydocs/working/task_semanticist21_20261026_inline_wrap_contracts.md`에 있다.
