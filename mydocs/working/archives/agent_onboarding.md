---
kind: working
status: active
canonical: mydocs/working/agent_onboarding.md
last_verified: 2026-08-18
---

# 작업 기록 — 에이전트 온보딩 닥터 고도화 (#5292)

gym 이 아니라 **실사용 에이전트**가 rhwp 에 붙는 경로를 키운다.
새 rhwp CLI 는 만들지 않았고, 편집 로직을 발명하지 않았다.

## 범위

| 포함 | 제외 |
|---|---|
| `tools/agent_onboarding/rhwp_doctor.py` | `gym/` 전체 |
| `tools/agent_onboarding/test_rhwp_doctor.py` | 다른 `.claude/skills/*` |
| `tools/agent_onboarding/fixtures/` | 열린 PR 문서 |
| `.claude/skills/rhwp-onboarding/` + `references/` | 새 `src/bin` / 새 하위명령 |
| 이 파일 | 한컴 호환·렌더 픽셀 |

## 한 일

1. 바이너리 탐색을 PATH/release 너머 debug·cargo bin·`RHWP_BIN` 까지 확장.
   기존 override→PATH→release 우선순위는 유지.
2. 샘플 매직 분류. 불량 입력은 파서에 넣지 않고 `bad_sample` 로 FAIL.
3. 네트워크 프로브를 비임계로 추가. `--offline` 지원. 오프라인은 실패가 아님.
4. 호스트별 MCP 스니펫 (`--host`, `--list-hosts`). `--write` 는 기존처럼 A형만.
5. 첫 5분 단계와 references/ 실존 검증 (`--list-recipes`).
6. 예외를 `exceptions[]` 로 방출. nextSteps 는 문서 경로를 가리킨다.
7. 비임계 자가검증 `explain` / `digest` / `inspect injection`.
8. 유닛 테스트가 세 예외 경로와 호스트 모양, 매직, CLI 를 가드.

## 종료 코드 계약 (유지)

| 코드 | 뜻 |
|---:|---|
| 0 | 임계 통과 |
| 1 | 임계 실패 (불량 샘플 포함) |
| 2 | 사용법 / `--write` 거부 / 잘못된 `--host` |
| 3 | 바이너리 없음 |

스키마는 `1.1` (필드 추가). 기존 `binary`/`checks`/`mcpJson`/`recipes` 는 유지.

## 검증

```bash
python -m unittest tools/agent_onboarding/test_rhwp_doctor.py
python tools/agent_onboarding/rhwp_doctor.py --list-hosts
python tools/agent_onboarding/rhwp_doctor.py --list-recipes
python tools/agent_onboarding/rhwp_doctor.py --offline --json --repo-root .
```

바이너리 없이 세 번째 호출은 exit 3 + `missing_binary` + `no_network`(정보).
불량 픽스처 + 스텁 버전 검사는 테스트가 커버한다.

Rust 소스 변경 없음. `cargo fmt --all -- --check` 는 포맷 게이트 유지용.

## 의도적으로 안 한 일

- `rhwp doctor` 같은 새 CLI 표면.
- `edit` 새 하위명령, 새 fill 문법.
- gym 팩 실행, 리더보드, certify.
- 다른 스킬 SKILL.md 수정.
- 네트워크에서 샘플을 받아 오는 기능.

## 남은 한계 (정직)

- `explain`/`digest` 는 구버전 바이너리에서 SKIP. 임계가 아니다.
- ZIP 매직은 DOCX 도 `hwpx` 후보로 분류한다. 최종 거절은 `info` 가 한다.
- 네트워크 프로브는 IPv4 리터럴이라 IPv6-only 환경을  offline 으로 볼 수 있다.
- `--write` 는 호스트 B/Zed/YAML 모양을 파일로 쓰지 않는다. 병합은 사람/에이전트.

## 파일 지도

| 경로 | 역할 |
|---|---|
| `tools/agent_onboarding/rhwp_doctor.py` | 닥터 구현 |
| `tools/agent_onboarding/test_rhwp_doctor.py` | 닥터 구현 |
| `tools/agent_onboarding/fixtures/samples/` | 고의 불량 샘플 |
| `tools/agent_onboarding/fixtures/envelopes/` | 봉투 required 키 |
| `tools/agent_onboarding/fixtures/mcp/` | 호스트 스니펫 픽스처 |
| `tools/agent_onboarding/fixtures/reports/` | 리포트 형태 픽스처 |
| `tools/agent_onboarding/fixtures/recipes/` | 첫 5분 인덱스 JSON |
| `.claude/skills/rhwp-onboarding/SKILL.md` | 스킬 진입 |
| `.claude/skills/rhwp-onboarding/references/first-5-min.md` | 5분 지도 |
| `.claude/skills/rhwp-onboarding/references/first-5-min-triage.md` | 트리아지 레시피 |
| `.claude/skills/rhwp-onboarding/references/first-5-min-tables.md` | 표 레시피 |
| `.claude/skills/rhwp-onboarding/references/first-5-min-form-read.md` | 서식 조사 레시피 |
| `.claude/skills/rhwp-onboarding/references/first-5-min-security.md` | 보안 스윕 레시피 |
| `.claude/skills/rhwp-onboarding/references/mcp-json-paste.md` | 호스트 붙여넣기 |
| `.claude/skills/rhwp-onboarding/references/binary-discovery.md` | 바이너리 발견 |
| `.claude/skills/rhwp-onboarding/references/sample-selftest.md` | 자가검증 계약 |
| `.claude/skills/rhwp-onboarding/references/exception-missing-binary.md` | exit 3 플레이북 |
| `.claude/skills/rhwp-onboarding/references/exception-bad-sample.md` | 불량 샘플 플레이북 |
| `.claude/skills/rhwp-onboarding/references/exception-no-network.md` | 오프라인 플레이북 |
| `mydocs/working/agent_onboarding.md` | 이 기록 |

## 에이전트가 이 문서를 쓰는 법

1. `python tools/agent_onboarding/rhwp_doctor.py --json` 을 돌린다.
2. `exitCode` 와 `exceptions[].kind` 로 분기한다.
3. 0 이면 `first5Min[]` 의 실존 레시피를 순서대로 실행한다.
4. 3 이면 빌드만 하고 재실행한다.
5. 1 이면 불량 샘플을 바꾸고 재실행한다.
6. 네트워크 칸은 읽기만 하고 온보딩을 멈추지 않는다.

## 온보딩이 인용하는 기존 CLI (발명 없음)

| 명령 | 단계 | 읽기 전용 |
|---|---|---|
| `rhwp --version` | 바이너리 | 예 |
| `rhwp info --json` | 자가검증·트리아지 | 예 |
| `rhwp export-text --json --max-chars` | 자가검증 | 예 |
| `rhwp explain --json` | 트리아지 | 예 |
| `rhwp digest --json` | 트리아지 | 예 |
| `rhwp export-tables --json` | 표 | 예 |
| `rhwp table-to-csv --json` | 표 | 예 |
| `rhwp fields --json` | 서식 조사 | 예 |
| `rhwp inspect hidden-text --json` | 보안 | 예 |
| `rhwp inspect injection --json` | 보안 | 예 |
| `rhwp inspect unicode --json` | 보안 | 예 |
| `rhwp mcp-serve` | 부착 | 예 (서버) |
| `rhwp capabilities --mcp` | 부착 | 예 |
| `rhwp replay --plan-json --json` | 영수증 입구 | 예 (기존 스키마) |

쓰기 명령은 이 표에 없다. 필요하면 기존 스킬로 이동한다.

## 이슈 대응

- 이슈: https://github.com/edwardkim/rhwp/issues/5292
- 브랜치: `feat/agent-onboarding-doctor`
- base: `devel`
- 목표 additions: 5000–10000 (최소 5000). 주석 패딩이 아니라 레시피·픽스처·테스트.

## 검증 로그 (로컬)

유닛 테스트는 rhwp 바이너리 없이 돈다.

```text
python -m unittest tools/agent_onboarding/test_rhwp_doctor.py
```

잠그는 갈래:

- MCP 스니펫 복사본, 호스트 15종 모양, 포트 없음
- aggregate 0/1/3, 비임계 SKIP 이 건강을 안 뒤집음
- 샘플 매직: empty / too_small / not_document / OLE / ZIP / HWP3 / avoid
- 예외 플레이북 3종 (missing_binary / bad_sample / no_network)
- `main --json` 의 stdout 격리, exit 3/1/2, `--write` 보호
- 첫 5분 명령이 기존 CLI 접두사만 사용
- 워크트리 references/ 실존과 gym 점수 러너 비인용

Rust 소스는 바꾸지 않았다. `cargo fmt --all -- --check` 는 포맷 게이트 유지.

## 에이전트 운영 메모

실사용 에이전트가 이 브랜치를 체크아웃한 뒤의 최소 순서:

1. `python tools/agent_onboarding/rhwp_doctor.py --json`
2. exit 3 이면 `cargo build --release --bin rhwp` 후 1 반복
3. exit 1 이고 `bad_sample` 이면 `--sample samples/basic/english.hwp`
4. exit 0 이면 `mcpJson` 을 호스트 파일에 병합
5. `first5Min` 에서 `referenceExists` 인 단계만 실행
6. 편집이 필요하면 기존 스킬로 이동. 이 문서에서 fill/set-cell 을 발명하지 않음

오프라인 메모: `--offline` 은 프로브만 끈다. MCP 와 자가검증은 로컬이다.
