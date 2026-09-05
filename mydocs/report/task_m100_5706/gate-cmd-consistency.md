# Gate command consistency — contributor skill-path-gate vs author 3-pass

날짜: 2026-08-20
이슈: #5706
소스 수정 없음. 커밋 없음. push 없음. SKILL.md 미수정.

대조 대상:

- `.agents/skills/rhwp-contributor/SKILL.md` — `## 스킬 경로 게이트 (PR 직전 필수)`
- `.agents/skills/rhwp-skill-author/SKILL.md` — `## 3-pass 게이트 — 끝내기 전에 명령마다 세 번`

기대 세 명령: `gate_new_skill`, `test_route`, `regression_suite_015 --nocapture`.

## 판정

**MATCH**

두 절의 bash 펜스 본문이 바이트 단위로 같다. 세 줄 모두 동일하다.

```bash
python tools/skill_router/gate_new_skill.py
python -m unittest tools/skill_router/test_route.py
cargo test --test regression_suite_015 skills_have_valid_frontmatter -- --nocapture
```

## 줄 위치

| 파일 | 절 | 펜스 |
|------|-----|------|
| `.agents/skills/rhwp-contributor/SKILL.md` | 스킬 경로 게이트 | L113–117 |
| `.agents/skills/rhwp-skill-author/SKILL.md` | 3-pass 게이트 | L44–48 |

DIFF 가 아니므로 본문 인용은 위 공통 블록 한 번만 둔다.
