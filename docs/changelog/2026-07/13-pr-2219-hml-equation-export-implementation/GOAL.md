# GOAL - PR 2219 HML equation and export

Single source of done. Only the verifier checks criteria.

## Original Request

> subagent로 코드 수정 pr에 개선된 것도 포함 하기 내 repo 그리고 원저자 repo까지

## Spec

PR `edwardkim/rhwp#2219`의 실제 head `cskwork:feature/hml-document-open@9083e9d1`에서 후속 패치를 만든다. HML `<EQUATION>/<SCRIPT>`를 공용 `Control::Equation`으로 import하여 기존 renderer/editor를 사용하고, 의미 보존 HML serializer/preflight, embed `exportHml`, required `getHmlSaveState`, additive `hml-export` capability를 제공한다. 익명화한 repo-owned 수학 fixture로 load/edit/export/reparse를 검증한다. 미지원 equation semantics는 durable non-preserved metadata로 유지해 저장을 차단한다. 오브젝트 스토리지·ExamBank 제품 코드는 변경하지 않는다.

상세 요구사항은 구현 브랜치에 포함할 `docs/changelog/2026-07/13-pr-2219-hml-equation-export-spec/SPEC.md`를 따른다.

## Success Criteria

- [x] repo-owned HML fixture의 equation 4개와 ordered SCRIPT가 import되고 `EQUATION` 손실 경고가 없다 - verify: `cargo test --test hml_parser`
- [x] inline control offset, BaseUnit, TextColor, unknown semantics validation gates가 테스트로 고정된다 - verify: targeted parser/model tests
- [x] 기존 equation editor 경로로 imported equation script를 수정할 수 있다 - verify: Rust/WASM edit test + browser E2E
- [x] HML export/reparse가 수정 수식과 미수정 수식을 보존하고 unknown semantics는 typed blocker로 차단한다 - verify: `cargo test --test hml_serializer`
- [x] embed `exportHml`, `getHmlSaveState`, `hml-export` capability가 additive v1 계약으로 동작한다 - verify: `cd rhwp-studio && npm test`
- [x] 기존 HWP/HWPX/HML 관련 회귀 테스트와 frontend build가 통과한다 - verify: targeted Rust suites, `npm test`, `npm run build`, fmt/clippy
- [x] 실제 브라우저에서 수식 HML load/edit/export/reload가 visible equation assertions로 통과한다 - verify: playwright-cli evidence
- [x] 상세 SPEC과 provenance가 PR diff에 포함되고 모든 diff hunk가 요구사항에 추적된다 - verify: adversarial review + `Backward-trace: clean`
- [x] 검증 대상이 실제 PR head 계보에 있고 non-force push 대상 `cskwork/feature/hml-document-open`과 `edwardkim/rhwp#2219` 연결이 확인된다 - verify: git refs/GitHub PR metadata
- [x] 실제 canvas 수식 선택→기존 편집 dialog→submit→undo/redo→export/reload UI 흐름이 통과하고 네 수식 영역이 non-empty/non-overlap으로 보인다 - verify: browser E2E (surfaced: mandatory review found direct `__wasm` bypass)
- [x] unknown equation child의 entity-decoded attribute/text `name=value`가 durable blocker에 유지된다 - verify: parser + save-state tests (surfaced: finalized SPEC requires path/name/value)
- [x] fixture provenance가 실제 copy/transformation/checksum과 일치하고 disallowed identifier가 없다 - verify: fixture contract test + README audit (surfaced: mandatory review found inaccurate minimized claim)

## QA Cases

- [x] local parent harness에서 fixture를 iframe에 load하면 `x^2 +1`과 `3` 수식이 빈칸이 아닌 수식으로 보인다 - evidence: `qa/to-be-hml-equations.png`
- [x] 첫 수식을 `x^3 +2`로 수정하고 export/reload하면 변경·미변경 수식이 모두 보존된다 - evidence: `qa/to-be-hml-roundtrip.png`
- [x] as-is 실제 호스트에서는 같은 fixture의 EQUATION warning과 빈 수식 자리가 재현된다 - evidence: `qa/as-is-hml-equations.png`

## Decision Gates

| ID | Action | Status | Finding | Decision | Recheck |
|---|---|---|---|---|---|
| d1 | auto-fix | resolved | PR head는 local `pr/2219`이 아니라 `origin/feature/hml-document-open@9083e9d1` | 실제 PR head에서 worktree 생성 | `gh pr view 2219` |
| d2 | auto-fix | resolved | 원본 checkout에 사용자 POC/스펙 미추적 파일 존재 | 격리 worktree에서만 코드 수정 | `git worktree list` |
| d3 | no-op | resolved | object storage PUT은 rhwp 책임이 아님 | bytes-only embed 계약 유지 | review |
| d4 | auto-fix | resolved | commit gate 이전 성공조건에 실제 push를 넣으면 gate와 delivery가 순환 의존 | pre-commit 성공조건은 PR 계보·대상 연결 검증으로 한정하고 실제 commit/push는 gate 이후 delivery 단계에서 수행 | commit gate + final delivery evidence |
