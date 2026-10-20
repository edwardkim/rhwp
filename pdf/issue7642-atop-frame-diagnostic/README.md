# #7642 상하 항·정렬 경계·빈 줄의 독립 Print 대조군

Issue: #7642

## 입력/출력과 재현

한컴 정상 문서의 구역·스타일을 재사용한 26쪽 HWPX에서 저장 수식 크기·기준선·LineSeg를 비운다. 별도 이름 문단 → 수식 문단 → 후속 이름 문단 순서이며 첫 구역 정의 문단은 유지한다. 같은 입력을 MCP engine 2020에서 정상 HWP 저장한 뒤 Print로 출력한다. RHWP 재저장/렌더는 기준 PDF 생성에 쓰지 않았다. 원 카탈로그/기존 실패 문서를 이 대조군으로 대체하지 않는다.

| 파일 | SHA-256 |
| --- | --- |
| `frame-input.hwpx` | `adf0aa1484113e7c28c725e6c08b48d36a5b0c573c5997689e54718a393de577` |
| `frame-hancom-2020.hwp` | `0b3c01aa39609e25bc852ad9b59d1faa60b6c0784ee4db02ee6ef1026b4cb764` |
| `frame-hancom-2020.pdf` | `9d39939a4aa16b79b3a698bf875ec2037246dfe261b1f434fda7e827406d20c1` |

정상 저장 job: `5119fdd6-748e-4ca2-bdad-ab551a923878`; Print job: `fd38ef32-47da-481f-acbd-d5a1b047d27a`.
실제 한컴 버전: `11.0.0.9136`; `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 전체 26쪽. MCP CLI의 start/status/download와 비공개 env-file 절차를 사용했다. 영수증·실행 로그·생성/분석 스크립트는 ignored output에 보존하며 접근 정보는 기록하지 않는다.

## 독립 관측

- 일반 ATOP과 CHOOSE/BINOM 및 괄호 ATOP은 서로 다른 논리 크기를 발행한다. 상하 항·명시적 빈 항·분수 항을 포함해 비교한다.
- EQALIGN의 근호·괄호·첨자·색 범위·인접 연산자 경계를 일반 글줄 간격 계약과 대조한다.
- 2열/3열의 불균등 MATRIX, 직사각형, 명시적 빈 칸/빈 행으로 실제 셀 순서와 남은 물리 예약을 비교한다.
- 인자 없는 PILE/LPILE/RPILE과 CASES 및 빈 CASES 템플릿의 개체 점유와 후속 문단 원점을 구분한다.

## 구현 검증 상태

새 소스의 전쪽 Native/fresh WASM TSV와 확대 판독은 진행 중이다. 통과한 회귀 fixture나 PR 완료 증거로 간주하지 않는다.
