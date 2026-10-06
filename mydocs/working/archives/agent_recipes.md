# #5331 실 에이전트 실무 레시피 라우터 — 작업 기록

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5331
브랜치: `feat/agent-recipes` (`upstream/devel` 기준 격리 worktree)
범위: `.claude/skills/rhwp-recipes/` ·
`scripts/tests/test_agent_recipes.py` ·
`tests/agent_recipes_skill_contract.rs` ·
capability 등록부 `CAP-5331` · 본 문서
비범위: `gym/` · 이웃 스킬 재작성 · 새 CLI · DocumentCore 편집 구현 ·
`rhwp-desk*` / `rhwp-handoff` / `rhwp-scaffold-final` / `rhwp-doc-repro`

## 무엇을

에이전트가 요청을 레시피 01(서식) · 02(표) · 03(마스킹) · 04(수신 점검) ·
05(메일머지) · 06(시각 회귀) · 09(대량 추출) · 10(송신 스윕) 중
**한 장**으로 고르게 한다. 이 스킬은 라우터다. form-fill /
table-exchange / security-sweep / bulk-pipeline / visual-regression 을
다시 쓰지 않는다.

07·08 은 존재하지 않는다. 레시피 09 머리말이 결번(#3905)이라고 이미
말했다. 이 작업은 그 파일을 만들지 않는다.

## 왜

이슈 본문: 어느 레시피로 들어갈지 와 빠진 번호 정직 표지.
gym 금지. 새 CLI / 편집 로직 발명 금지.

정본은 이미 `mydocs/manual/recipes/*.md` 에 있다. 에이전트가 필요한 것은
새 플레이북이 아니라 **요청 → 카드 → 첫 수 → 이웃 스킬** 과
예외 세 갈래(파일 없음 · last_verified stale · 두 장 충돌)다.

DoD: additions 5000–10000 (최소 5000). PR 전 `cargo fmt --all -- --check`.

## 어떻게

1. 격리 worktree `C:/Users/swsz9/rhwp-agent-recipes` 에
   `feat/agent-recipes` 를 `upstream/devel` 에서 분기.
   금지 디렉터리(`rhwp-desk*` 등)는 쓰지 않음. 디스크 부족으로
   sparse-checkout (mydocs/pr 자산·samples·gym 제외).
2. SKILL.md 를 대조표·결번·정지·인계 인덱스로 작성.
3. `references/` 25장: 나무, 대조표, 카드 8장, 결번, 예외,
   untrusted, 첫 수, 다음 스킬, 정지, 인계, 함정, 여정, 발화,
   stale, 파일 없음, 두 장, 발췌, 결정표.
4. `_gen_pack.py` 가 정본 레시피의 json/bash 블록을 **발췌**해
   `fixtures/transcripts/` 에 둔다. 살아 있는 CLI 를 돌리지 않음.
5. `scripts/tests/test_agent_recipes.py` 와
   `tests/agent_recipes_skill_contract.rs` 가 발명 명령·gym·결번·
   세 예외·발췌 계약을 바이너리 없이 검사.
6. capability 등록부 `CAP-5331` / `rhwp-recipes` 행 추가.

## 하지 않은 것

- 07·08 레시피 초안
- form-fill / table-exchange / security-sweep / bulk-pipeline /
  visual-regression 스킬 수정
- 새 recipe 하위명령
- DocumentCore / gym pack
- 살아 있는 봉투를 다시 실행해 지어내기

## 검증

```bash
python -m unittest scripts.tests.test_agent_recipes
cargo fmt --all -- --check
```

정본 레시피 여덟 장은 읽기만 했다.

## 예외

- 파일 없음 → R03
- last_verified 30일 초과 → R04 (2026-08-18 기준 여덟 장은 신선)
- 두 장 충돌 → R05
