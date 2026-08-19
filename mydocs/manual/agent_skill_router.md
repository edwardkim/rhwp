---
kind: canonical
status: active
canonical: mydocs/manual/agent_skill_router.md
last_verified: 2026-08-20
---

# 에이전트 스킬 라우터

사용자 요청을 스킬 본문 추측이 아니라 **한 장의 JSON 봉투**로 고른다. 파이프는
다섯 단이다.

`request → intent → requiredCapabilities → skillSelection → executionGraph`

구현은 표준 라이브러리만 쓰는 `tools/skill_router/` 다. 새 rhwp 하위명령이
아니고, 문서 편집 로직도 발명하지 않는다. capability 이름은
[에이전트 capability 카탈로그](agent_capability_registry.md)가 권위다. 기여
요청이 이 파이프를 타면 [CONTRIBUTING.md](../../CONTRIBUTING.md) 절차를 가리키는
`rhwp-contributor` 그래프가 나온다.

등록 식별번호는 Issue [#5706](https://github.com/edwardkim/rhwp/issues/5706)의
`CAP-5706` 이다.

## 5단 파이프

| 단 | 봉투 키 | 하는 일 |
| --- | --- | --- |
| 1 요청 | `request` | 사용자가 한 말을 그대로 싣는다. 명령을 발명하지 않는다. |
| 2 의도 | `intent` | 요청을 의도 슬러그로 분류한다. 서식 채움은 `fill-form`, PR·기여는 `contribute`. |
| 3 capability | `requiredCapabilities` | 카탈로그의 capability ID. 서식은 `rhwp-form-fill` (`CAP-5300`), 기여는 `rhwp-contributor` (`CAP-4561`). |
| 4 스킬 | `skillSelection` | 그 capability 의 Claude/Codex 진입점. 보통 `.claude/skills/<id>/SKILL.md`. |
| 5 그래프 | `executionGraph` | 실행 순서. `{nodes, edges}`. 노드는 `id`, `skill`, `action`, `command`. 가장자리는 `from` → `to`. |

봉투 최상위는 항상 다음 키를 가진다.

`schemaVersion`, `request`, `intent`, `requiredCapabilities`, `skillSelection`,
`executionGraph`

패키지 README 는 rhwp `--json` 출처 표지와 맞춰 `untrustedContent`,
`untrustedFields` 도 적는다. 라우터는 문서를 열지 않으므로 표지가 있다면
`untrustedContent` 는 false, `untrustedFields` 는 빈 목록이다. 출처 계약은
[봉투 출처](../tech/envelope_provenance.md)다.

stdout 규약은 [CLI JSON 파이프라인 가이드](cli_json_pipeline_guide.md) 와 같다.
`--json` 이면 stdout 은 JSON 객체 **하나**다. 진단은 stderr.

## CLI — 실측 진입점

저장소 루트에서 돌린다. 의존성 0, Python 3 표준 라이브러리. 아래 두 줄과
JSON 은 2026-08-20 에 이 워크트리에서 그대로 실행한 결과다.

```bash
python tools/skill_router/route.py "이 서식 채워줘" --json
python tools/skill_router/route.py "PR 올려" --json
```

`--json` 이 없거나 요청이 비면 exit 2, 사용법은 stderr.

### 이 서식 채워줘

```console
$ python tools/skill_router/route.py "이 서식 채워줘" --json
{
  "schemaVersion": "1.0",
  "request": "이 서식 채워줘",
  "intent": {
    "id": "fill-form",
    "label": "서식 채움",
    "confidence": 0.99
  },
  "requiredCapabilities": [
    "rhwp-form-fill"
  ],
  "skillSelection": [
    {
      "id": "rhwp-form-fill",
      "path": ".claude/skills/rhwp-form-fill/SKILL.md",
      "reason": "서식 채움 요청이므로 rhwp-form-fill 을(를) 선택한다 (겹치면 더 구체적인 스킬)"
    }
  ],
  "executionGraph": {
    "nodes": [
      {
        "id": "fields",
        "skill": "rhwp-form-fill",
        "action": "fields",
        "command": "rhwp fields <서식> --json"
      },
      {
        "id": "dry-run-fill",
        "skill": "rhwp-form-fill",
        "action": "dry-run fill",
        "command": "rhwp edit fill-fields <서식> --data <JSON> --dry-run --json"
      },
      {
        "id": "fill-verify",
        "skill": "rhwp-form-fill",
        "action": "fill --verify",
        "command": "rhwp edit fill-fields <서식> --data <JSON> -o <출력> --verify --json"
      },
      {
        "id": "sanitize",
        "skill": "rhwp-form-fill",
        "action": "sanitize",
        "command": "rhwp edit sanitize <산출> -o <제출본> --json"
      }
    ],
    "edges": [
      {
        "from": "fields",
        "to": "dry-run-fill"
      },
      {
        "from": "dry-run-fill",
        "to": "fill-verify"
      },
      {
        "from": "fill-verify",
        "to": "sanitize"
      }
    ]
  },
  "untrustedContent": false,
  "untrustedFields": []
}
```

그래프는 `fields` 다음에 fill 이다. 서식 스킬 권위는
[서식 자동화 가이드](form_filling_guide.md) 와
[rhwp-form-fill Skill](../../.claude/skills/rhwp-form-fill/SKILL.md).

### PR 올려

```console
$ python tools/skill_router/route.py "PR 올려" --json
{
  "schemaVersion": "1.0",
  "request": "PR 올려",
  "intent": {
    "id": "contribute",
    "label": "기여·PR",
    "confidence": 0.9
  },
  "requiredCapabilities": [
    "rhwp-contributor"
  ],
  "skillSelection": [
    {
      "id": "rhwp-contributor",
      "path": ".claude/skills/rhwp-contributor/SKILL.md",
      "reason": "기여·PR 요청이므로 rhwp-contributor 을(를) 선택한다 (겹치면 더 구체적인 스킬)"
    }
  ],
  "executionGraph": {
    "nodes": [
      {
        "id": "issue",
        "skill": "rhwp-contributor",
        "action": "issue",
        "command": "gh issue list; gh pr list --search <키워드>; 없으면 gh issue create (DoD·판단 근거)"
      },
      {
        "id": "analyze",
        "skill": "rhwp-contributor",
        "action": "analyze",
        "command": "mydocs/manual/README.md 선택표와 기존 계약 테스트를 읽고 이슈에 기록"
      },
      {
        "id": "branch",
        "skill": "rhwp-contributor",
        "action": "branch(upstream/devel)",
        "command": "git fetch upstream devel; isolation worktree from upstream/devel"
      },
      {
        "id": "implement",
        "skill": "rhwp-contributor",
        "action": "implement",
        "command": "기존 결을 따라 구현. git add <경로> (git add -A 금지)"
      },
      {
        "id": "fmt-clippy-test",
        "skill": "rhwp-contributor",
        "action": "fmt/clippy/test",
        "command": "cargo fmt --all -- --check; cargo clippy -- -D warnings; cargo test"
      },
      {
        "id": "working-doc",
        "skill": "rhwp-contributor",
        "action": "working-doc",
        "command": "mydocs/working/<이름>.md 에 무엇·왜·어떻게·검증 실측"
      },
      {
        "id": "pr",
        "skill": "rhwp-contributor",
        "action": "pr(devel, Korean template, closes #)",
        "command": "gh pr create --base devel --body-file <한국어 템플릿> (closes #)"
      }
    ],
    "edges": [
      {
        "from": "issue",
        "to": "analyze"
      },
      {
        "from": "analyze",
        "to": "branch"
      },
      {
        "from": "branch",
        "to": "implement"
      },
      {
        "from": "implement",
        "to": "fmt-clippy-test"
      },
      {
        "from": "fmt-clippy-test",
        "to": "working-doc"
      },
      {
        "from": "working-doc",
        "to": "pr"
      }
    ]
  },
  "untrustedContent": false,
  "untrustedFields": []
}
```

그래프는 issue 노드와 pr 노드를 포함한다. 기여 절차 정본은
[CONTRIBUTING.md](../../CONTRIBUTING.md) 와
[rhwp-contributor Skill](../../.claude/skills/rhwp-contributor/SKILL.md).
HARD GATE 는 `cargo fmt --all -- --check`. 라우터는 그 절차를 그래프에 올리고
리뷰·머지 판단은 하지 않는다.

## 검증

```bash
python -m unittest tools/skill_router/test_route.py
```

위 두 발화를 CLI 와 `import tools.skill_router.route` 로 넣고, stdout 이 JSON
하나인지와 5단 키·그래프를 본다. `route.py` 가 없으면 hang 하지 않고 skip
한다. 더미 JSON 코퍼스는 쓰지 않는다.

스킬별 화면 증적(터미널이 아니라 `rhwp export-svg` 페이지 PNG)은
`mydocs/report/task_m100_5706/` 에 둔다. 그 폴더는 이 문서의 소유가 아니다.

## 새·변경 SKILL.md

`.claude/skills/<name>/SKILL.md` 를 새로 만들거나 바꾸면 아래를 만족해야
한다. 라우터 스킬도 예외가 아니다.

1. YAML frontmatter 의 `name` 은 폴더명과 같다. `description` 은 20자 이상.
2. 본문에 실행 가능한 `rhwp <command>` 가 **하나 이상** 있어야 한다. 명령
   토큰은 ASCII 소문자(`[a-z0-9-]+`, 소문자로 시작). `rhwp <명령>`
   플레이스홀더는 참조가 아니다 — CI `tests/skills_contract.rs` 의
   `skills_have_valid_frontmatter_and_are_executable` 가 실패한다.

PR 전에 저장소 루트에서 세 번 돌린다.

```bash
python tools/skill_router/gate_new_skill.py
python -m unittest tools/skill_router/test_route.py
cargo test --test regression_suite_015 skills_have_valid_frontmatter -- --nocapture
```

실패 예: PR [#5707](https://github.com/edwardkim/rhwp/pull/5707) shard 3 에서
`rhwp-skill-router` 본문에 `rhwp <cmd>` 실참조가 없어
`skills_have_valid_frontmatter_and_are_executable` 가 깨졌다.

## 하지 않는 것

- 새 rhwp CLI 명령, 새 edit 로직, `src/` 의 `#[cfg(test)]`
- `gym/`
- 카탈로그에 없는 capability 이름 발명
- 터미널 창 스크린샷을 검증 증적으로 제출
