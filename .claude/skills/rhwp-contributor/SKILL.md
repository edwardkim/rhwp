---
name: rhwp-contributor
description: rhwp 저장소에 기여(이슈·코드 변경·문서·PR)할 때의 공식 절차를 안내합니다. 이슈 등록 → 분석 → 구현 → 로컬 검증 게이트 → 작업 증빙(캡슐) → 처리 결과 문서 → PR 생성까지의 순서와, 변경 범위별 필수 검증(fmt/clippy/test·시각 검증)을 저장소 규약대로 밟습니다. 트리거 — 사용자가 "rhwp에 기여", "PR 올려", "이슈 만들고 수정", "버그 고쳐서 제출", "기여 절차" 등을 요청할 때. 규약 정본은 AGENTS.md 와 CONTRIBUTING.md.
---

# rhwp-contributor — 기여 절차 Skill

## 목적

기여 1건을 저장소 규약대로 완주한다. 절차의 정본은 [AGENTS.md](../../../AGENTS.md)·
[CONTRIBUTING.md](../../../CONTRIBUTING.md)·[PR 검토 절차](../../../mydocs/manual/pr_review_workflow.md)이고,
이 스킬은 그 순서를 실행 가능한 체크리스트로 옮긴 것이다.

## 절차 (필수 순서)

1. **이슈 선등록** — 무엇을 왜 바꾸는지, 판단 근거와 완료 기준(DoD)을 이슈로 먼저
   남긴다. 중복 확인: `gh pr list --search <키워드>` 로 같은 작업의 열린 PR 이 없는지 본다.
2. **분석** — 관련 canonical 문서(`mydocs/manual/README.md` 선택표)와 기존
   계약 테스트를 읽고, 원인·설계를 이슈에 기록한다.
3. **브랜치** — 최신 `upstream/devel` 기준으로 만든다. base 는 항상 `devel`.
4. **구현** — 기존 결(명명·주석 밀도·모듈 경계)을 따른다. 새 CLI/MCP 표면은
   [에이전트 표면 플레이북](../../../mydocs/manual/agent_surface_playbook.md)의 등재 절차를 따른다.
5. **로컬 검증 게이트** — 변경 범위별 기본 검증은
   [local_validation.md §4.3](../../../mydocs/manual/pr_review/local_validation.md) 이 정본.
   공통 최소: `cargo fmt --check` · `cargo clippy -- -D warnings` · 관련 `cargo test`.
   렌더링·레이아웃 변경은 시각 검증 근거(PDF/SVG 전후 비교)를 남긴다.
6. **작업 증빙 (권장)** — 문서를 실제로 편집·생성한 작업이면 영수증을 남긴다:

   ```bash
   rhwp replay --plan-json <계획JSON> --capsule work.capsule.json --json
   ```

   연속 작업은 `--parent 이전.capsule.json` 으로 계보를 잇고, 폴더 단위 재검증은
   `rhwp audit <폴더> --json`, 계보 검증은 `rhwp lineage <머리캡슐> --json`.
   봉투(JSON) 원문을 PR 본문에 붙이면 리뷰어가 재계산으로 검증할 수 있다.
7. **처리 결과 문서** — 규모 있는 변경은 `mydocs/working/` 에 무엇을·왜·어떻게·
   검증 실측을 남긴다(스테이지 문서 관례).
8. **PR** — [템플릿](../../../.github/pull_request_template.md) 체크리스트를 전부 채운다.
   제목·본문은 한국어. 관련 이슈를 `closes #` 로 연결한다.

## 하지 않는 것

- 미병합 기능을 규약처럼 요구하지 않는다 — 증빙 명령은 devel 병합분만 안내한다.
- 다른 기여자의 변경을 임의로 되돌리지 않는다.
- 리뷰·머지 판단을 대신하지 않는다 — 그것은 메인테이너의 몫이다.
