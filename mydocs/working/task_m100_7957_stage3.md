# Task #7957 3단계 — F2 입력·쪽 선택 계약

- Issue: #7957·#7962, 공동 범위 #7958·#7959·#7960·#7961·#7963
- 승인된 공동 계획의 F2. Native 계약 검증 완료, 실제 새 WASM은 F5 통합 게이트.

입력 상한은 frontend/backend 공통 64 MiB다. 복사 전 검사를 유지하며 생성한 정상 PDF의
16 MiB −1/정확/+1, 64 MiB −1/정확/+1을 실행했다. 앞의 5개는 1쪽을 가져오고 마지막은
constructor/resource snapshot 없이 input bytes used 0/requested 67,108,865/limit 67,108,864로
거절했다. qpdf syntax/stream 검사와 정확한 파일 길이·hash는 ignored
`output/pr-review/issue7957/f2/boundary-inputs.json`에 있다. 50 MiB 이상 생성본도 Git에 넣지 않았다.
기존 Docker WASM은 정상 16 MiB +1만 거절했으므로 기존 오류와 수정 후 Native를 구분한다.

원본 traversal은 backend 참조 방문 예산의 source 범위, 선택량은 최대 1,024개 구역으로 분리한다.
자원 ID·문단·그림·연산 등 다음 한도는 유지한다. CLI 범위 확장은 선택 상한을, WASM 숫자는
선택 개수와 원본 인덱스를 각각 검증한다. 실제 원본 쪽 범위 검증은 공통 parser에서 한다.
신뢰 호스트용 `parse_pdf_with_policy`도 유한한 값/구역 u16 범위를 확인하고 같은 경로를 소비한다.

합성 162쪽 전체와 129·162번째 선택, 선택량 초과·범위 밖·역순·중복 및 입력 정확 경계 계약을
검사했다. resource 114 PASS, frontend 14 PASS다. F1 실행에 frontend suite가 포함되지 않았던
기록을 바로잡았다. 두 suite는 각각 001·021이며 기존 003은 이 대상이 아니었다.
이미 성공한 resource 검사를 재실행하지 않고 같은 source의 frontend 검사 결과를 합쳤다.

실물 162쪽은 쪽수 gate를 넘지만 1쪽 text의 operators 250,000에서 다음 거절이 발생한다.
따라서 #7957 전체 문서 성공으로 판정하지 않는다. source/실행 파일 hash와 로그는
`f2/source-evidence.json`, `native-boundaries.jsonl`, `focused-tests*.log`에 보존했다.

F3 선행 관측에서 별도 신뢰 Native 측정 정책으로 operators/decoded/path만 넓혀 작업량을 관측했다.
이 정책은 제품 기본값·새 WASM·배포 결과가 아니다. 원본 연산은 독립 pikepdf에서 274,718(12쪽),
421,337(107쪽)이며 Form은 없다. 기존 요청은 각각 549,436/421,337 연산과 누적
99,265,975/1,163,489,907 decoded bytes를 사용한다. JPEG 14쪽은 누적 799,328,649 bytes다.
위치 가져오기에서 사용하지 않는 문서 전체 머리말 분류와 반복 해석·그림 변환의 실제 소비를
F3에서 먼저 줄인 뒤 유한한 요청 작업 정책을 정한다. 한도를 없애거나 실패 카운터를 초기화하지 않는다.
