# Task #3604 Stage 3: 보호 문서 MCP 암호 전달 계약

Issue: #3604

## 목표

`rhwp mcp-serve`가 암호 HWP5, 압축 HWP3, ODF 암호 HWPX를 세션과 무상태 도구에서
열 수 있게 한다. 암호는 CLI 인자, MCP 응답, 세션 상태에 보존하지 않는다.

## 시작 상태

- CLI는 `--password`와 `--password-stdin`으로 세 종류의 보호 문서를 연다.
- 세션 `hwp_open`은 `HwpDocument::from_bytes`만 호출하므로 보호 문서를 열 수 없다.
- 무상태 MCP 도구는 `cli.args`만 조립하며 자식 프로세스 stdin에 암호를 보낼 경로가 없다.
- `hwp_batch`와 `hwp_batch_search`는 이미 stdin을 경로 목록에 사용한다.

## 구현 계획

1. `hwp_open` inputSchema에 선택 `password`를 `writeOnly`로 선언하고, 존재할 때
   `from_bytes_with_password`를 호출한다.
2. 문서를 실제로 파싱하는 무상태 도구에 `password` 입력과 `cli.passwordStdin` 메타를
   선언한다. `cli.args`에는 암호 자리표시자를 넣지 않는다.
3. `mcp-serve`는 `password`가 있을 때만 자식 CLI에 `--password-stdin`을 붙이고,
   자식 stdin 첫 줄로 값을 쓴다. 경로 목록 stdin을 쓰는 batch 계열은 암호를 거부한다.
4. 실제 HWP5/HWP3/HWPX fixture로 누락·오류·정상 암호 세션 계약과 무상태 stdin
   계약을 검사한다. 오류·응답 텍스트에 테스트 암호가 에코되지 않는지도 검사한다.
5. MCP 통합 가이드에 입력·보안 경계·batch 제약을 기록한다.

## 성공 기준

- 올바른 암호로 `hwp_open` → `hwp_doc_text` → `hwp_close`가 세 fixture 모두 성공한다.
- 누락 또는 틀린 암호는 `isError: true`이고 세션 핸들을 만들지 않는다.
- 무상태 `hwp_info`는 암호를 argv가 아닌 `--password-stdin` 경로로 전달해 성공한다.
- MCP 응답과 오류에 전달한 비밀번호 문자열이 포함되지 않는다.
- `hwp_batch`/`hwp_batch_search`는 password와 paths stdin을 혼용하지 않는다.

## 테스트 결과

실행 시각: 2026-08-01 KST

| 검증 | 결과 |
| --- | --- |
| 신규 `mcp_password_contract` | 4건 성공. HWP5/HWP3/HWPX 세션의 누락·오류·정상 암호, 무상태 stdin 전달, `passwordStdin` 선언과 `cli.args` 비노출, 비밀값 비에코, batch stdin 충돌 거부를 확인했다. |
| 기존 `mcp_server_contract` | 6건 성공. initialize, 선언-실행 목록, 평문 세션 왕복 등 기존 계약에 회귀가 없다. |
| 기존 HWP3/HWP5/HWPX 암호 fixture | 각각 11건·2건·3건 성공. CLI 복호화 계약이 유지된다. |
| `cli_json_contract` | 22건 성공. `capabilities --mcp` 선언과 CLI JSON 계약의 드리프트가 없다. |
| `cargo fmt --check`, `git diff --check` | 성공. |
| 실제 시각 증적 | 암호 HWPX를 `--password-stdin`으로 열어 1쪽 SVG→PNG 렌더를 확인했다. `mydocs/report/task_m100_3604/password_hwpx_page001.png`에 보존했다. |

release test target은 `target/task_m100_3604`를 사용했으며, 공유 `target/release`는 건드리지 않았다.
