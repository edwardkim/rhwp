---
kind: canonical
status: active
canonical: mydocs/manual/agent_surface_playbook.md
last_verified: 2026-07-30
---

# 에이전트 표면 플레이북 — CLI `--json`·MCP 도구를 추가하는 공식 절차

rhwp 의 **에이전트 표면**(CLI 기계 계약 + MCP 도구 + 세션 도구)에 새 조각을 더하는
공식 절차와 수용 기준을 고정한다. 로드맵·잔여 목록의 권위는 #3608
(에이전트 표면 전면 커버리지, #2659 후속)이고, 본 문서는 그 로드맵의 조각을
**실제로 추가할 때 지켜야 하는 계약**이다. 절차를 어긴 표면 추가는 되돌린다.

## 1. 표면의 3층 구조 (어디에 더하는가)

| 층 | 무엇 | 단일 출처 |
|---|---|---|
| CLI `--json` | stdout 순수 JSON 봉투 + #2707 종료 코드 | 각 명령 구현 + 봉투 helper (`*_json_value`) |
| MCP 무상태 도구 | 선언(`capabilities --mcp`)과 실행(`mcp-serve`)이 공유하는 도구 | `mcp_tool_definitions()` (src/main.rs) |
| MCP 세션 도구 | 열린 핸들(`docId`) 위의 재파싱 없는 연산 | `mcp_serve.rs` 의 `served_tools()`+디스패치 |

**규칙 1 — 선언·실행·문서는 한 곳에서 갈라진다.** 무상태 도구는
`mcp_tool_definitions()` 에만 추가하면 선언과 서버가 함께 얻는다. 도구 목록을
다른 곳에 복제하지 않는다. 드리프트 가드
(`capabilities_mcp_covers_every_json_command`,
`tools_list_matches_capabilities_manifest`)가 어긋남을 잡는다.

**규칙 2 — 새 편집·조회 로직을 만들지 않는다.** MCP/세션 도구는 검증된 코어
함수(`set_field_value_by_name_at`, `replace_all_native`, `grep`,
`collect_field_records`, `extract_tables`, `edit_serialize` …)와 기존 봉투
helper 를 재사용한다. 서버 전용 경로를 새로 만들면 CLI 와 계약이 갈라진다.

**규칙 3 — 판정은 데이터다.** 차이 검출(`identical:false`)·치환 0건·`notFound` 는
오류가 아니라 봉투 필드다. `isError:true` 는 실행 실패(없는 파일, 닫힌 핸들)에만
쓴다. CLI 는 exit 3/4 로 판정을 신호하되 봉투를 먼저 낸다.

## 2. 추가 절차 (순서 고정)

1. **이슈 등록** — 공백을 실측으로 서술하고(#3608 매트릭스 갱신 포함) 검증 계획을 적는다.
2. **red 계약 테스트** — `tests/*_contract.rs` 신설. 구현 전 FAILED 를 확인한다.
   기존 테스트 파일 수정보다 신설을 우선한다(병렬 PR 충돌 회피).
3. **구현** — 규칙 1~3 준수. 실패 경로 stdout 순수성(부분 산출물 미출력) 포함.
4. **검증** — 신규 green + 인접 계약 스위트 무회귀 + `clippy -D warnings` 0 +
   rustfmt clean(변경 파일 기준).
5. **누적 머지 충돌검사** — `upstream/devel` 에서 임시 브랜치를 만들어 열린 PR
   브랜치 전부를 순차 merge, 충돌 0 확인. 겹치는 파일이 있으면 적층(베이스 PR 을
   본문에 명시)으로 전환한다.
6. **처리 문서 + 증적 2종** — `mydocs/report/task_m100_<이슈>/README.md` 에:
   ① 실행 원문(터미널 봉투) 캡처 ② **산출물을 실제 rhwp 로 열어 렌더한 화면**
   (`export-svg` → PNG 변환 → 합성). 편집 계열은 전/후 비교로.
7. **PR** — 한글 제목·본문, `closes #<이슈>`, 증적 이미지는 저장소에 커밋 후
   raw 링크로 본문 참조. 열린 PR 은 10건 이내를 유지한다.

## 3. 수용 기준 (Definition of Done — 조각 단위)

- [ ] stdout 순수성: `--json` 모드에서 stdout 에 JSON 하나(배치는 NDJSON)만.
      진단·진행 메시지는 stderr.
- [ ] 실패 경로: 런타임 실패 시 stdout 비움(부분 매니페스트 금지), exit 1.
      조립 오류는 exit 2 — **미지 옵션 침묵 무시 금지**.
- [ ] `schemaVersion` 필드 포함, 필드 추가는 허용·변경/삭제는 계약 테스트가 잡는 구조.
- [ ] 무상태 도구: `inputSchema.required` 와 `cli.args` 자리표시자가 1:1
      (선택 인자를 자리표시자로 쓰지 않는다 — 미치환 문자열이 CLI 로 새는 사고 방지).
- [ ] 세션 도구: 닫힌 핸들 `isError`, 디스크 기록은 `hwp_doc_save` 만, 판정 어휘는
      무상태 대응 도구와 동형.
- [ ] 문서: `cli_commands.md` 해당 절 현행화(+ 필요 시 `mcp_integration_guide.md`).

## 4. 증적 규약 (따라 하기 어려운 이유를 유지한다)

증적은 **재현 가능해야** 한다 — 이미지와 함께 재현 명령을 보고서에 남긴다.
가짜/합성 데이터로 만든 화면은 반드시 그렇게 표기하고, 실물 문서(인터넷 배포
서식 등)를 쓸 수 있으면 실물을 우선한다. 다쪽 문서 편집은 "건드리지 않은 쪽의
불변"(픽셀 대조)까지 포함한다.

## 5. 로드맵 연동

- 잔여 목록·우선순위·명시적 제외의 권위: **#3608** (§1 매트릭스, §6.5 백로그).
- 조각을 착수하면 #3608 의 해당 항목에 이슈 번호를 달고, 머지되면 체크한다.
- 매트릭스는 `capabilities` 교차 스크립트(#3608 §5)로 재생성해 감으로 갱신하지 않는다.
