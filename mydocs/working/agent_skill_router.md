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
