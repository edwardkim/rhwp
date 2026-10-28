---
kind: working
status: active
issue: 5706
---

# 에이전트 스킬 라우터 — 작업 기록 (CAP-5706)

날짜: 2026-08-20
이슈: https://github.com/edwardkim/rhwp/issues/5706
브랜치: `feat/skill-router`
capability: `CAP-5706`
이 기록의 소유 경로만: `tools/skill_router/test_route.py`,
`mydocs/manual/agent_skill_router.md`, 본 파일.

비범위(다른 에이전트 소유): `catalog.json`, `route.py`, `intents.py`,
`graph.py`, `.claude/skills/rhwp-skill-router/SKILL.md`, capability 등록부,
`mydocs/report/task_m100_5706/` PNG.

## 무엇을

사용자 요청을 `request → intent → requiredCapabilities → skillSelection →
executionGraph` 한 장으로 고르는 라우터의 **계약 시험**과 **권위 문서**를
남긴다. 라우터 구현·카탈로그·스킬 진입점·렌더 PNG 는 쓰지 않는다.

고정한 두 발화:

- `"이 서식 채워줘"` → intent `fill-form`, capability `rhwp-form-fill`,
  그래프는 `fields` 다음 fill
- `"PR 올려"` → intent `contribute`, capability `rhwp-contributor`,
  그래프는 issue 와 pr

봉투 필수 키: `schemaVersion`, `request`, `intent`, `requiredCapabilities`,
`skillSelection`, `executionGraph`. stdout 은 JSON 하나.

## 왜

Issue #5706: 스킬이 많아도 요청을 바로 투입할 선택 경로가 없으면 리뷰어가
검증 여부를 묻는다. 에이전트가 PR 을 올릴 때도 같은 파이프로 contributor
그래프가 나와야 한다.

이 파동(F tests-docs)은 구현을 추측해 카탈로그를 채우지 않는다. README 의
실 CLI 두 줄을 그대로 돌리고, 그 stdout 을 권위 문서 예시로 옮긴다.
`route.py` 가 없으면 시험은 hang 하지 않고 skip 한다.

터미널 로그는 증적이 아니다. 스킬 화면 증적은 다른 에이전트가
`mydocs/report/task_m100_5706/` 에 두는 rhwp 렌더 페이지 PNG 다.

## 어떻게

1. `tools/skill_router/README.md` 와 이슈 본문의 5단 파이프를 읽었다.
   권위 문서 예시는 README 의 CLI 두 줄을 실행한 stdout 이다.
2. `test_route.py` — `unittest`, 표준 라이브러리. CLI 는
   `python tools/skill_router/route.py "<요청>" --json` (타임아웃 20초).
   `import tools.skill_router.route` 는 별도 서브프로세스(타임아웃 15초).
   `route()` 가 있으면 fill-form 도 함수로 본다. 더미 JSON 코퍼스 없음.
3. `mydocs/manual/agent_skill_router.md` — canonical, last_verified
   2026-08-20. 5단 설명, CONTRIBUTING·capability 등록부 링크.
4. 본 기록.

만지지 않은 것: 라우터 본문, 카탈로그, 등록부, SKILL.md, report PNG,
`gym/`, `src/`.

## 검증

```bash
python -m unittest tools/skill_router/test_route.py
```

2026-08-20 이 워크트리에서 8 tests OK. 같은 날 실 CLI:

```bash
python tools/skill_router/route.py "이 서식 채워줘" --json
python tools/skill_router/route.py "PR 올려" --json
```

각각 `fill-form`/`rhwp-form-fill`/`fields`→fill, `contribute`/`rhwp-contributor`/
issue·pr. stdout 은 JSON 객체 하나.

시각 PNG 는 `mydocs/report/task_m100_5706/` (verify-A/B 소유)에 산다. 이
기록은 터미널 창을 찍었다고 주장하지 않는다.

`.claude/skills/<name>/SKILL.md` 를 새로 쓰거나 고치면 PR 전에 세 번 더
돌린다.

```bash
python tools/skill_router/gate_new_skill.py
python -m unittest tools/skill_router/test_route.py
cargo test --test regression_suite_015 skills_have_valid_frontmatter -- --nocapture
```

계약:

- frontmatter `name` == 폴더명, `description` ≥ 20자
- 본문에 실재 `rhwp <command>` (ASCII 소문자 토큰) 하나 이상.
  `rhwp <명령>` 플레이스홀더는 `tests/skills_contract.rs` 가 세지 않는다.

실측 실패: PR #5707 shard 3. `rhwp-skill-router` 에 `rhwp <cmd>` 가 없어
`skills_have_valid_frontmatter_and_are_executable` 가 깨졌다.

## 새 스킬 만들어 — rhwp-skill-author

`"새 스킬 만들어"` 도 `request → intent → requiredCapabilities →
skillSelection → executionGraph` 를 탄다. 경로 스킬은 `rhwp-skill-author`
(`.claude/skills/rhwp-skill-author/SKILL.md`). CAP ID 를 발명하지 않는다.

카탈로그 스킬에 `PROBES` 가 없으면 `gate_new_skill.py` 가 실패한다.

명령마다 3회:

```bash
python tools/skill_router/gate_new_skill.py
python -m unittest tools.skill_router.test_route
cargo test --test regression_suite_015 skills_have_valid_frontmatter -- --nocapture
```

CI: `.github/workflows/skill-router-gate.yml`.
