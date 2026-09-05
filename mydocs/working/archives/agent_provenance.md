# 에이전트 출처 표지 소비 스킬 고도화

- **Issue**: [#5295](https://github.com/edwardkim/rhwp/issues/5295)
- **브랜치**: `feat/agent-provenance`
- **기준**: `upstream/devel`
- **날짜**: 2026-08-18
- **범위**: 실사용 에이전트가 문서 파생 값을 미신뢰 데이터로 다루게 한다. gym 아님.

## 1. 문제

rhwp `--json` 봉투에는 엔진 값과 문서 작성자가 정한 값이 한 객체로 섞인다.
경량 에이전트는 `pages[].text`·`title`·`fields[].guide` 를 도구가 내려준
지시처럼 읽는다. 표지(`untrustedContent`/`untrustedFields`)와
`export-provenance-map` 은 이미 있다. 부족한 것은 **소비 쪽 규약**이다.

이슈 DoD: additions 5000–10000, PR 전 `cargo fmt --all -- --check`. gym 금지.
새 CLI 없음.

## 2. 한 일

`.agents/skills/rhwp-provenance/` 를 스킬 + 레퍼런스 + 픽스처로 키웠다.

| 경로 | 역할 |
| --- | --- |
| `SKILL.md` | 운영 진입점. 요청→명령, 표지 읽기, 금지 자리, B1~B5 |
| `references/export-provenance-map.md` | 지도 명령을 호출 전 정책으로 쓰는 법 |
| `references/untrusted-content-fields.md` | true/false/미표기, 경로 문법, D/R/C |
| `references/injection-boundaries.md` | B1~B5 와 inspect 연계 |
| `references/forbidden-prompt-slots.md` | 금지 자리 전수 |
| `references/command-field-catalog.md` | MAP 전 명령 소비 해설 |
| `references/consumption-playbook.md` | 호출 순서 체크리스트 |
| `references/anti-patterns.md` | 실측된 나쁜 소비 |
| `references/privilege-reduction.md` | 호스트 권한 축소 |
| `fixtures/*.json` | 테스트가 긁는 기계 목록 |
| `tests/cases/agent_provenance_skill_contract.rs` | MAP·픽스처·스킬 드리프트 가드 |

`mydocs/manual/agent_capability_registry.md` 에 `CAP-5295` /
`rhwp-provenance` 행을 추가했다.

## 3. 하지 않은 일

- 새 CLI / MCP 도구를 만들지 않았다. 지도와 `inspect`/`armor` 가 있다.
- `gym/` 을 만지지 않았다.
- `rhwp-onboarding`·`rhwp-mcp-session`·`rhwp-safe-edit`·`rhwp-doc-triage`
  스킬을 만지지 않았다.
- `crates/rhwp-contracts/src/provenance.rs` 의 MAP 을 바꾸지 않았다.
  소비 해설만 추가했다. (`charts` 중복 항목도 엔진 쪽 수정이 아니라
  첫 항목을  consum 한다.)

## 4. 검증

로컬:

```bash
cargo fmt --all -- --check
# 스킬 계약 (generated suite 에 배정)
node scripts/run-rust-test.mjs agent_provenance_skill_contract
```

가드가 지키는 것:

- 픽스처 명령·경로 = `MAP` (유일 키)
- 카탈로그에 전 명령 절
- 금지 자리 12개 + 허용 2개
- B1~B5
- 체크리스트 id 가 플레이북에 존재
- 자리 사례가 MAP 필드를 가리킴
- 봉투 예가 true/false/미표기를 가르침
- Cargo.toml `[[bin]]` 이 2개로 유지
- 레지스트리 `CAP-5295`
- 바이너리가 있으면 라이브 `export-provenance-map` 과 픽스처 대조

## 5. 소비 에이전트가 가져갈 것

1. 문서를 열기 전에 지도를 캐시한다.
2. 표지 키 부재는 false 가 아니라 미표기다.
3. D 값은 화면 또는 nonce 격벽만. 시스템 프롬프트·경로·도구 이름·URL·
   계획서·승인 근거에 넣지 않는다.
4. 읽기 턴에 쓰기 도구를 치운다. 신호가 있으면 멈춘다.
5. 산출 파일 본문은 봉투 표지 밖의 새로운 미신뢰 입력이다.

## 6. 남은 일 (이 PR 밖)

- 옛 실측 미표기 표면이 현재 바이너리에서 닫혔는지 주기적 재실측.
- `charts` MAP 중복은 엔진 쪽 별도 정리.
- 호스트(도구 등록기)의 프로필 전환은 각 제품의 구현.
