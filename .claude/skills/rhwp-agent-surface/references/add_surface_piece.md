# 조각을 더하는 절차

정본: 플레이북 제1부 §2. 종류 카드: [`../fixtures/add_surface/`](../fixtures/add_surface/).

이 스킬은 절차를 안내한다. 이 PR 에서 새 CLI 를 만들지 않는다.

## 순서 고정

0. **잠금.** 이슈 assignee 와 같은 이슈를 가리키는 열린 PR 을 확인.
   비어 있으면 선점. 외부 기여자는 `gh issue edit --add-assignee` 가
   거부될 수 있다 — 그 경우 착수 코멘트가 잠금이다.
1. **이슈.** 공백을 실측으로 서술. 검증 계획을 적는다.
2. **red 계약 테스트.** `tests/*_contract.rs` **신설**.
   기존 테스트 파일 수정보다 신설 (병렬 PR 충돌 회피).
3. **구현.** 규칙 1~3. 실패 경로 stdout 순수성.
4. **검증.** 신규 green + 인접 계약 + `clippy -D warnings` + rustfmt.
5. **누적 머지 충돌검사.** `upstream/devel` 에 열린 PR 을 순차 merge.
6. **처리 문서 + 증적 2종.** 실행 원문 + 실제 렌더 화면.
7. **PR.** 한글 제목·본문, `closes #<이슈>`.

## 층별 최소 변경

### CLI JSON

1. 코어 함수가 이미 있는가? 없으면 코어 이슈를 먼저 (이 스킬 밖).
2. 명령 디스패치가 `*_json_value` 를 내게.
3. `capabilities_command_entries()` 에 `cmd_json(...)`.
4. `--json` 이면 `mcp_tool_definitions()` 한 줄도 같은 PR.
5. `cli_commands.md` + 지식 지도 행.

### MCP 무상태

1. 대응하는 `--json` CLI 가 이미 있어야 한다.
2. `tool()` 또는 `tool_with_optional_args()`.
3. `required` ↔ 자리표시자 1:1.
4. `outputFields` 에 판정 필드(`overflow` · `identical` · `notFound`)를 빠뜨리지 않는다.
5. `capabilities_mcp_covers_every_json_command` green.

### MCP 세션

1. 무상태 짝이 있어야 한다. 짝이 없는 동사를 `hwp_doc_*` 로 만들지 마라.
2. `ALL_SESSION_TOOLS` 에 이름 하나.
3. `served_tools()` 디스패치가 **같은 코어**를 부른다.
4. `docId` 필수. 닫힌 핸들 → `isError` + `nextCall.hwp_open`.
5. 디스크 기록은 `hwp_doc_save` 만. 편집은 인메모리.
6. 판정 어휘는 무상태 짝과 동형.
7. 프로필 `allows_session_tool` 경계 — 조회 전용 프로필에 save 를 열지 마라.

## 선등재

두 열린 PR 이 같은 허용목록에 항목을 더하고, 완전성 가드가 목록을
순회하면 머지 순서에 따라 패닉한다. 조건이 맞으면 상대 항목을
**죽은 값으로 미리 등재**한다. 조건·한계는
`mydocs/tech/autonomous_maintenance/pre_registration_pattern.md`.
여섯 조건이 하나라도 빠지면 적층(체인 3단 이하)으로 푼다.
