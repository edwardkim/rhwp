---
kind: guide
status: active
canonical: mydocs/manual/agent_preflight_guide.md
last_verified: 2026-08-02
---

# 에이전트 표면 선검사 가이드

`tools/agent_preflight.py`는 에이전트 표면([#3630](https://github.com/edwardkim/rhwp/issues/3630),
[#3719](https://github.com/edwardkim/rhwp/issues/3719))에 명령을 추가할 때 **드리프트 가드가
한 번에 하나씩만 드러나는 문제**를 없앤다.

## 왜 필요한가

드리프트 가드는 저마다 독립된 `assert`다. 그래서 실패도 하나씩만 드러난다.

```
코드 작성 → cargo build (6분) → MCP 도구 누락으로 실패     → 고침
         → cargo build (6분) → required 배열 누락으로 실패 → 고침
         → cargo build (6분) → 속성 배선 누락으로 실패     → 고침
```

셋 다 **같은 커밋의 같은 실수**인데 빌드를 세 번 태운다. 실제로 이 순서로 걸려
빌드에만 18분을 쓴 적이 있다.

선검사는 **모든 검사를 한 번에 돌리고 실패를 전부 모아서** 보고한다. 고칠 것을 한꺼번에
알고 빌드는 한 번만 하면 된다.

## 쓰는 법

코드를 쓰는 중에는 빌드 없이 (1초 안에 끝난다):

```bash
py tools/agent_preflight.py --static-only
```

커밋·푸시 직전에는 이미 빌드된 바이너리로 가드까지:

```bash
py tools/agent_preflight.py --bin target/release/rhwp
```

이 커밋이 건드려야 할 경로를 선언하면 오염까지 본다:

```bash
py tools/agent_preflight.py --scope src/main.rs --scope tests/my_contract.rs
```

종료 코드는 `0` 통과 / `1` 실패 있음 / `2` 사용법 오류다.

## 무엇을 보는가

| 검사 | 빌드 | 무엇을 막는가 |
| --- | --- | --- |
| 오염 | 불필요 | `git add -A` 로 워크트리·빌드산출물·남의 작업이 딸려 들어가는 것 |
| doc 주석 오배치 | 불필요 | 남의 doc 주석과 함수 사이에 새 함수를 끼워 넣는 것 |
| ReDoS | 불필요 | 지수형 `(…+)+` 와 다항형 `([A-Z]+)([A-Z]…)` |
| rustfmt | 불필요 | 미정렬, 그리고 **검사 자체가 실패한 것을 통과로 오독하는 것** |
| MCP inputSchema | 필요 | `type`·`properties`·`required` 누락 |
| 속성 ↔ CLI 배선 | 필요 | 선언만 하고 배선 안 해서 인자가 조용히 버려지는 것 |
| capabilities ↔ help | 필요 | 두 축이 따로 노는 것 |
| `--json` ↔ MCP 도구 | 필요 | `--json` 명령에 도구를 안 만드는 것 |
| 선언 flags 실재 | 필요 | 없는 플래그를 문서에 적는 것 |
| 실패 경로 stdout | 필요 | 실패하면서 반쪽 JSON 을 흘리는 것 |

## 허용목록은 베끼지 않는다

가드에는 정당한 예외가 있다. `paths`·`password`는 argv가 아니라 stdin으로 가고,
`core-pages` 같은 내부 프로브는 `--help`에 없어도 되며, `capabilities` 자신은 MCP 도구가
아니라 도구 목록의 원천이다.

선검사는 이 목록을 **계약 테스트에서 직접 읽는다** — `tests/mcp_server_contract.rs`의
`NON_ARGV_PROPERTIES`, `tests/cli_json_contract.rs`의 `HELP_HIDDEN`과
`capabilities_mcp_covers_every_json_command`의 인라인 제외.

베끼지 않는 이유는 단순하다. 베낀 목록은 언젠가 원본과 어긋나고, 어긋나면 선검사가 실제
가드와 **다른 말**을 한다. 헛울리는 검사기는 곧 무시당하고, 무시당하는 검사기는 없느니만
못하다 — 그게 정확히 이 도구가 없애려는 재작업이다.

## 선검사는 계약 테스트를 대신하지 않는다

권위는 `cargo test`다. 선검사는 **같은 실패를 더 싸게 먼저** 보여줄 뿐이다.

- 선검사가 통과해도 계약 테스트는 반드시 돌린다
- 선검사와 계약 테스트가 다른 말을 하면 **계약 테스트가 옳다**. 그때는 선검사를 고친다
- 새 가드를 계약 테스트에 추가하면, 가능하면 선검사에도 같은 검사를 넣는다

## `cargo fmt --all` 을 쓰지 않는 이유

이 저장소 규모에서 `cargo fmt --all`은 Windows 인자 길이 한계(32K)에 걸려
`os error 206`으로 **통째로 실패**한다. 그러면 출력에 `Diff in` 줄이 하나도 없어서
**통과처럼 보인다.** 이 저장소에서 실패를 통과로 오독한 사고가 두 번 있었다.

선검사는 파일을 10개씩 나눠 `rustfmt`를 직접 부르고, rustfmt 자체가 실패하면
그것을 **통과가 아니라 검사 불능**으로 보고한다.

## 큐 규율 검사 — 유일한 네트워크 검사, 경고 전용

병렬 세션 규약([parallel_session_protocol.md](../tech/autonomous_maintenance/parallel_session_protocol.md)
§8-6, 확정 이슈 #3914 수용 기준 2)의 구현. `gh` 가 있고 인증돼 있으면 자동으로 돌고,
없으면 **조용히 건너뛴다** — 네트워크 실패로 선검사 전체가 빨개지면 곧 우회당하고,
그러면 로컬 검사까지 함께 꺼지기 때문이다. `--no-network` 로 명시적으로 끌 수 있다.

세 가지를 본다. 전부 **경고 전용**이다 — 종료 코드에 영향이 없다.

| 검사 | 신호 | 왜 경고인가 |
|---|---|---|
| 잔량 | 열린 PR > 10건 | 캡은 "10건 내외"지 하드 리밋이 아니다 |
| 동일 이슈 중복 | 같은 이슈를 **대상으로 선언**한(제목 `#N`·본문 `closes/refs #N` 행) 열린 PR 2건 이상 | 중복 여부 판정은 사람 몫(protocol §6) |
| 미할당 착수 | 브랜치 이름이 `task/<n>-`·`wip/fix-<n>-` 인데 이슈 #n 에 assignee 도 착수 코멘트도 없음 | 선점하고 계속하면 된다 |

본문 전체의 `#N` 을 긁지 않는 이유: 조망 이슈(#3907 등) 참조가 어디에나 있어
헛울린다. 선언된 대상만 세면 오탐이 거의 없다 — 실사고 두 건(#3902/#3903,
#3897/#3904)은 둘 다 이 신호로 잡혔을 것이다.

착수 코멘트도 단어 `착수`의 포함 여부로 판정하지 않는다. 첫 줄이
`착수합니다 — <범위>` 형식일 때만 잠금으로 인정한다. 회고·인용·`아직 착수하지
않음`이 새 세션을 잘못 통과시키지 않도록 하는 경계다.

## 관련 문서

- [에이전트 표면 플레이북](agent_surface_playbook.md) — 표면 사용법
- [에이전트 실패 사전](agent_troubleshooting_guide.md) — 실패 진단
- [경량 에이전트 내성](../tech/weak_agent_proofing.md) — 가드의 설계 근거
- [로컬 검증 게이트](pr_review/local_validation.md) — 변경 범위별 필수 검증
- [병렬 세션 규약](../tech/autonomous_maintenance/parallel_session_protocol.md) — 큐 규율 검사의 설계 원천
