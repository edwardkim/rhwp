---
name: rhwp-contributor
description: rhwp 저장소에 기여(이슈·코드 변경·문서·PR)할 때의 공식 절차를 안내합니다. 이슈 등록 → 분석 → 브랜치(upstream/devel) → 구현 → 로컬 검증 게이트 → 작업 증빙(캡슐) → 처리 결과 문서 → 한국어 PR 까지의 순서와, 변경 범위별 필수 검증(fmt/clippy/test·시각 검증)을 저장소 규약대로 밟습니다. 트리거 — 사용자가 "rhwp에 기여", "PR 올려", "이슈 만들고 수정", "버그 고쳐서 제출", "기여 절차" 등을 요청할 때. 규약 정본은 AGENTS.md 와 CONTRIBUTING.md.
---

# rhwp-contributor — 기여 절차 Skill

> **HARD GATE — `gh pr create` 직전 필수. 실패하면 PR/push 금지.**
>
> ```bash
> cargo fmt --all -- --check
> ```
>
> 이것이 CI Lint Format check 와 같은 명령이다.
> `cargo fmt --check` 는 **낡은 표기**다. 그 명령만 돌리고 통과했다고 쓰지 마라.
> 테스트만 고친 커밋도 다시 `cargo fmt --all -- --check` 를 통과해야 한다.
> rustfmt `newline_style = Unix`. Windows `core.autocrlf` 가 CRLF 를 넣으면 이 게이트가 실패한다.
> `crates/` 가 워크트리에 있으면(스파스 체크아웃이 꺼내지 않았다면) 반드시 통과해야 한다.

## 목적

기여 1건을 저장소 규약대로 **완주**한다. 이 스킬은 gym 과제가 아니다. Maker seat
만 — 실제 이슈를 닫는 PR 을 올린다.

정본: [AGENTS.md](../../../AGENTS.md) ·
[CONTRIBUTING.md](../../../CONTRIBUTING.md) ·
[PR 검토 절차](../../../mydocs/manual/pr_review_workflow.md) ·
[local_validation.md §4.3](../../../mydocs/manual/pr_review/local_validation.md) ·
[PR 템플릿](../../../.github/pull_request_template.md).

이 스킬은 **새 CLI 를 만들지 않는다.** DocumentCore 편집 로직을 발명하지 않는다.
다른 스킬 본문을 이 파동에서 고치지 않는다. `gym/` 을 건드리지 않는다.

## 자식 문서 (이 스킬의 본문)

SKILL.md 는 라우터다. 단계에 맞는 자식을 **읽고 나서** 명령을 실행한다.

| 단계 | 읽기 | 경로 |
|------|------|------|
| 필수 순서 전체 | 순서 | [references/procedure-order.md](references/procedure-order.md) |
| 1 이슈 | 선등록 | [references/issue-first.md](references/issue-first.md) |
| 2 분석 | 정본 | [references/analyze-canonical.md](references/analyze-canonical.md) |
| 3 브랜치 | 격리 | [references/branch-isolation.md](references/branch-isolation.md) |
| 3 워크트리 | 이름 | [references/isolation-worktree.md](references/isolation-worktree.md) |
| 4 구현 | 범위 | [references/implement-scope.md](references/implement-scope.md) |
| 4 스테이징 | 파일 | [references/staging-named-files.md](references/staging-named-files.md) |
| 5 fmt | 관문 | [references/fmt-hard-gate.md](references/fmt-hard-gate.md) |
| 5 rustfmt | Unix | [references/rustfmt-unix.md](references/rustfmt-unix.md) |
| 5 clippy·test | 검증 | [references/clippy-and-tests.md](references/clippy-and-tests.md) |
| 5 렌더 | 시각 | [references/visual-evidence.md](references/visual-evidence.md) |
| 6 영수증 | 포인터 | [references/work-receipt-pointers.md](references/work-receipt-pointers.md) |
| 7 처리 결과 | 문서 | [references/working-doc.md](references/working-doc.md) |
| 8 PR | 한국어 | [references/korean-pr.md](references/korean-pr.md) |
| 8 템플릿 | 첫 칸 | [references/pr-template-checkboxes.md](references/pr-template-checkboxes.md) |
| 예외 | 분기 | [references/exceptions.md](references/exceptions.md) |
| 함정 | 실록 | [references/pitfalls.md](references/pitfalls.md) |
| 요청 라우팅 | 트리 | [references/decision-tree.md](references/decision-tree.md) |
| 레시피 색인 | 색인 | [references/recipe-index.md](references/recipe-index.md) |
| 명령·필드 | 카탈로그 | [references/command-field-catalog.md](references/command-field-catalog.md) |

실측 워크스루는 [examples/](examples/README.md) 다.
기계가 읽는 픽스처는 [fixtures/catalog.json](fixtures/catalog.json) 다.

## 절차 (필수 순서 — 건너뛰지 않는다)

1. **이슈** — 무엇을 왜 바꾸는지, 판단 근거와 DoD 를 이슈로 먼저 남긴다.
   `gh issue list` · `gh pr list --search <키워드>` 로 같은 작업의 열린 PR 이
   없는지 본다. 이미 열린 중복 PR 이 있으면 새 PR 을 만들지 않는다.
2. **분석** — `mydocs/manual/README.md` 선택표와 기존 계약 테스트를 읽고
   원인·설계를 이슈에 기록한다. DocumentCore 편집 로직을 여기서 발명하지 않는다.
3. **브랜치** — `git fetch upstream devel` 후 최신 `upstream/devel` 에서 만든다.
   base 는 항상 `devel`. **isolation worktree**. 이미 있는 named worktree 를
   훔치지 않는다. 금지 경로: `rhwp`(본진), `rhwp-desk*`, `rhwp-handoff`,
   `rhwp-scaffold-final`, `rhwp-doc-repro`.
4. **구현** — 기존 결을 따른다. `git add -A` 금지. 경로를 지정해 stage 한다.
   새 rhwp CLI 명령을 추가하지 않는다.
5. **로컬 게이트** — 공통 최소:
   - HARD GATE: `cargo fmt --all -- --check` (`cargo fmt --check` 아님)
   - `cargo clippy -- -D warnings`
   - 관련 `cargo test`
   - 렌더링·레이아웃 변경은 시각 근거(PDF/SVG 전후)
   - 변경 집합에 `.claude/skills/` 또는 `.agents/skills/` 가 있으면
     **스킬 경로 게이트**(아래 절)를 PR 전에 명령마다 세 번
   `crates/` 가 있으면 fmt 는 반드시 통과해야 한다.
6. **작업 영수증** — 문서를 실제로 편집·생성했으면
   `rhwp replay --capsule` / `rhwp audit` / `rhwp lineage` 포인터를 따른다.
   그 스킬 본문을 이 파동에서 다시 쓰지 않는다.
7. **처리 결과 문서** — 규모 있는 변경은 `mydocs/working/` 에 무엇을·왜·어떻게·
   검증 실측을 남긴다.
8. **한국어 PR** — fmt 게이트(그리고 스킬 경로가 있으면 스킬 경로 게이트)
   통과 뒤에만 `gh pr create --base devel --body-file`.
   제목·본문 한국어. `closes #<이슈>`. PR 템플릿 **첫 체크박스 = fmt 게이트**.

## 판정 규약

- fmt 실패 = PR 금지. 고쳐서 다시 `cargo fmt --all -- --check`.
- clippy warning 은 `-D warnings` 아래 실패다. 허용하지 않는다.
- 관련 테스트만 돌렸으면 PR 본문에 **어떤 테스트를 왜** 적는다.
- 시각 근거가 필요한 변경에 스크린샷/SVG 가 없으면 게이트 미완이다.
- CI `noci`(문서 전용 paths-ignore · required check 미발행) 와
  CI `FAILURE`(실제 빨간 검사) 를 혼동하지 않는다.
- sparse 체크아웃에 `crates/` 가 없으면 fmt `--all` 이 워크스페이스 멤버를
  못 찾을 수 있다. 예외 경로: [exceptions.md](references/exceptions.md).
- 스킬 경로가 있을 때 `skills_have_valid_frontmatter_and_are_executable`
  실패(`rhwp <cmd>` 없음) = 기여 중단. 하드 페일이다.

## 스킬 경로 게이트 (PR 직전 필수)

변경 집합에 `.claude/skills/` 또는 `.agents/skills/` 가 있으면,
`gh pr create` / push 전에 아래를 **명령마다 세 번** 돌린다.
한 번이라도 실패하면 기여를 멈추고 PR/push 하지 않는다.

```bash
python tools/skill_router/gate_new_skill.py
python -m unittest tools/skill_router/test_route.py
cargo test --test regression_suite_015 skills_have_valid_frontmatter -- --nocapture
```

`cargo test --test regression_suite_015 skills_have_valid_frontmatter -- --nocapture` 의
`skills_have_valid_frontmatter_and_are_executable` 가 실패하면 기여를
멈춘다. 스킬 본문에 실행 가능한 `rhwp <cmd>` 가 없으면 하드 페일이다.
새 rhwp CLI 를 만들지 않는다. 기존 예(`rhwp replay --capsule` /
`rhwp audit` / `rhwp lineage`)를 지우지 않는다.

## 하지 않는 것

- 미병합 기능을 규약처럼 요구하지 않는다 — 증빙 명령은 devel 병합분만 안내한다.
- 다른 기여자의 변경을 임의로 되돌리지 않는다.
- 리뷰·머지 판단을 대신하지 않는다 — 메인테이너의 몫이다.
- `git add -A` 를 쓰지 않는다.
- named worktree 를 훔치지 않는다.
- DocumentCore 편집 로직을 발명하지 않는다.
- 새 rhwp CLI 명령을 만들지 않는다.
- `gym/` 과 다른 스킬 본문(영수증 스킬 포함)을 고치지 않는다.
- 열린 PR 의 파일을 가로채 고치지 않는다.

## 상세 레퍼런스

- 순서: [references/procedure-order.md](references/procedure-order.md)
- 이슈: [references/issue-first.md](references/issue-first.md)
- 분석: [references/analyze-canonical.md](references/analyze-canonical.md)
- 브랜치: [references/branch-isolation.md](references/branch-isolation.md)
- 워크트리: [references/isolation-worktree.md](references/isolation-worktree.md)
- 구현: [references/implement-scope.md](references/implement-scope.md)
- 스테이징: [references/staging-named-files.md](references/staging-named-files.md)
- fmt: [references/fmt-hard-gate.md](references/fmt-hard-gate.md)
- Unix: [references/rustfmt-unix.md](references/rustfmt-unix.md)
- clippy·test: [references/clippy-and-tests.md](references/clippy-and-tests.md)
- 시각: [references/visual-evidence.md](references/visual-evidence.md)
- 영수증 포인터: [references/work-receipt-pointers.md](references/work-receipt-pointers.md)
- 처리 결과: [references/working-doc.md](references/working-doc.md)
- PR: [references/korean-pr.md](references/korean-pr.md)
- 템플릿: [references/pr-template-checkboxes.md](references/pr-template-checkboxes.md)
- 예외: [references/exceptions.md](references/exceptions.md)
- 함정: [references/pitfalls.md](references/pitfalls.md)
- 판단 트리: [references/decision-tree.md](references/decision-tree.md)
- 레시피 색인: [references/recipe-index.md](references/recipe-index.md)
- 명령·필드: [references/command-field-catalog.md](references/command-field-catalog.md)
- 워크스루: [examples/README.md](examples/README.md)
- 픽스처: [fixtures/catalog.json](fixtures/catalog.json)
- 작업 기록: [`mydocs/working/agent_contributor.md`](../../../mydocs/working/archives/agent_contributor.md)
