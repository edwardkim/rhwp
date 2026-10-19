# 개체 앞 공백과 명시적 개행의 독립 Print 계약

세 진단 HWPX와 각각을 인쇄한 독립 PDF를 함께 보존한다. 호스트의 저장 LineSeg를 수용하는
근거가 아니라 **저장 줄이 없는 재조판**의 계약이다. PDF는 RHWP 출력이나 HWPX 저장본을
PDF처럼 취급한 자료가 아니다. 각 진단 HWPX 자체를 한컴의 Print 경로로 인쇄했다.

## 생성과 대조 범위

공개 `samples/task1768/distribution_doc.hwpx`는 확장자와 달리 HWP5 배포 문서다.
그 문서를 한컴에서 진단용 HWPX로 별도 저장한 뒤 용지·스타일과 단위 표시 호스트·예산 표를
복제했다. 표는 한 셀로 줄이고 내용은 OBJECT/BREAK 표식으로 바꿨다. 일반 저장본의 캐시
유효성을 주장하거나 이 대조군 통과를 원본 배포 문서의 통과로 대신하지 않는다.
원본 3쪽도 같은 독립 Print와 별도로 검증했다.

- `print-object-space-overflow.hwpx` (9쪽): 선행 공백 82개와 단위 표시 11pt, 개체 앞 공백
  14pt/굵게/자간 -10을 사용한다. 1~6쪽은 후행 공백 0~5개, 7~8쪽은 선행 81/83개와
  후행 3개, 9쪽은 선행 82개·명시적 개행·후행 3개다.
- `print-object-space-boundaries.hwpx` (20쪽): 위 4쪽의 호스트를 복제한다. 1~12쪽은 선행
  공백 76~87개·후행 3개·자간 -10, 13~17쪽은 선행 82개에서 자간 -20/-5/0/+5/+10,
  18~20쪽은 선행 81/82/83개에서 동일 스타일의 공백 런을 나눈 대조다.
- `print-authored-break-metrics.hwpx` (6쪽): 차례로 같은 10pt 개행 1개/2개, 본문 11pt와
  개행 14pt, 본문 14pt와 개행 10pt, 본문 없이 14pt 개행 2개, 선행 82개와 같은 10pt
  개행이다. 개체를 담은 런에 hp:lineBreak를 넣고 호스트 문단마다 한 쪽을 사용한다.

MCP client의 `start → status → download`, engine **2020**, 실제 한컴
**11.0.0.9136**, PrintMethod **0**, `hancom2020_pdf_driver_one_up`을 확인했다.
원본 용지·방향과 100% 한 쪽 인쇄를 유지했다. 새 fixture로 복사할 때 파일명을 정리했으며
입력/PDF 바이트는 검증한 진단 원본과 같다.

| 입력/PDF 쌍 | Print job |
| --- | --- |
| object-space-overflow | `d7db1b61-0cf4-4fb1-8726-8f75705f28fc` |
| object-space-boundaries | `e86fdc88-fe15-4d33-9f82-bd6d84e68c1f` |
| authored-break-metrics | `d51ba289-d734-4bff-8548-9f341ef3fa28` |

## 파일 고정

| 파일 | 바이트 수 | SHA-256 |
| --- | ---: | --- |
| `print-object-space-boundaries.hwpx` | 9271 | `06f03325fe2105628e39051aea75fc12a98afd3cdde5807bea4cdc5468b51461` |
| `print-object-space-boundaries-2020.pdf` | 30739 | `c6e4ce7b012a8aa28a228ae8ad6625009713b48cd0894563ab70f01ab951451f` |
| `print-object-space-overflow.hwpx` | 8870 | `57f713b32a55c7450014d82fcca2ae833836414fb07d2b14d17d36008a9fcef3` |
| `print-object-space-overflow-2020.pdf` | 22255 | `ed00457e3d38c36832f2deb5e4c69f67610397ce4928cc4e4a0a77bc8a497d6e` |
| `print-authored-break-metrics.hwpx` | 8804 | `d892ea2642fdce3f732e43accd9dfd6cd9eaf3aee99f6234445bf5c2ee17f76b` |
| `print-authored-break-metrics-2020.pdf` | 20758 | `633a381004838cc52905d02c6d5d810954396ede75bb8810fab0081729f0767a` |

## 검증과 회귀

등록 전 35쪽 모두 Native/fresh WASM 일치율이 각각 **100.00000%**이며 전체 쪽수가 같다.
96dpi·Poppler none·동일 글꼴 경로를 사용했다. 수식이 없는 단위 표시·표 경계·실제 빈 줄을
같은 배율로 직접 읽었다. 원래 배포 문서는 Native 최저 98.17563%, fresh WASM 최저
98.04595%이고 보호하는 3쪽은 양쪽 100%다.

새 `tests/cases/verified_inline_print_rows.rs`는 원본 소유로 표를 선택하고 호스트/셀 내용·
작성 공백·문자 원점 배열·줄 순서·연속 개행·원본 문서의 제목과 후속 주석을 보호한다.
완료한 각 페이지의 표 경계는 독립 Print 벡터를 CSS 96dpi 좌상단 좌표로 환산해 ±2px로
함께 검사한다. 좌표로 검사 대상을 찾지 않으며 관계 검사도 유지한다.

생산 소스와 해시·전쪽 TSV·대표 review/overlay·before FAIL/after PASS는
`mydocs/working/task_m100_7642_stage1507.md`와 그 문서가 가리키는 ignored output에 있다.
TSV·실행 로그·중간 JSON은 커밋하지 않는다. 얇은 괘선의 래스터 농도 차이는 남으며
실루엣 100%를 원시 픽셀 일치로 해석하지 않는다. 이후에도 고정 페이지는 양쪽 98% 이상과
독립 Print의 직접 판독 조건을 유지해야 한다.
