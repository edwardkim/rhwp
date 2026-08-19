---
kind: pr-review
status: active
pr: 5569
---

# PR #5569 검토: 문서 에이전트 exact command 공개 브리지

## 접수

- PR: [#5569](https://github.com/edwardkim/rhwp/pull/5569) `기능: 문서 에이전트 공개 명령 브리지 추가`
- 작성자: `@coolwithyou` (first-time contributor)
- base: `devel` (`5305e994207b8b7a4eddac7600be3f804cfbf2f8`)
- code candidate: `364962caf60189b22dad244bb47f340dd2262261`
- 작성 시점 상태: `MERGEABLE`, `CLEAN`, non-draft. Reviewer `@jangster77`을 지정했다.
- base route: `collaborator_external_pr.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`, `multi_pr_update_branch.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md` 및 위 라우팅 문서

## 변경 범위와 계보

- contributor의 단일 기능 commit을 최신 `upstream/devel` 위 가시성 브랜치 `review/coolwithyou-5569-20260819`에 cherry-pick했다.
- `@rhwp/editor`에 strict v1 DTO 검증, document state/selection/apply/revert/focus/event 공개 표면을 추가한다.
- Studio에는 capability 협상, snapshot transaction, exact inverse revert, document-agent 변경 이벤트와 strict render 완료 경로를 연결한다.
- HWP/HWPX browser E2E와 SDK·transport·controller·history 회귀를 추가한다.
- 메인터너 코드·테스트 보정은 없다. 이 문서와 오늘할일만 contributor code candidate 뒤에 single-parent trailing commit으로 추가한다.

## 검토 결과

**차단 결함 없음.** public SDK의 strict 요청·응답 검증, capability fail-closed 동작, command fence, snapshot rollback, 일반 undo와 agent revert의 경계, event 구독 수명과 renderer commit 순서를 검토했다.

첫 기여자 처리로 최신 trailing head의 CI가 성공하면, 환영과 다음의 구체적 검증 결과를 포함한 approval comment를 게시한다. 검증 수준은 첫 기여 여부와 무관하게 일반 contributor PR과 동일하게 유지한다.

## 검증 증적

### 로컬

- `npm --prefix npm/editor test`: 32 passed.
- `npm --prefix rhwp-studio test`: 982 passed, 기존 skipped 1건.
- `npm --prefix rhwp-studio run build`: lockfile 기준 `npm ci` 뒤 통과.
- `(cd rhwp-studio && npx tsc --ignoreConfig --noEmit --skipLibCheck ../npm/editor/index.d.ts)`: 통과.
- `(cd npm/editor && npm pack --dry-run --json)`: `@rhwp/editor@0.8.5`, 배포 파일 6개 확인.
- `VITE_URL=http://127.0.0.1:7700 npm --prefix rhwp-studio run e2e:document-agent`: HWP/HWPX 각각 15개, 총 30개 browser assertion 통과. apply/revert, strict render 3초 상한, target 밖 manifest, focus/selection, native typing/undo, event 3회를 확인했다.
- `git diff --check upstream/devel...HEAD`: 통과.

### 기준선·환경 한계

- 표준 `docker compose --env-file .env.docker run --rm wasm`은 이 macOS host에서 Docker daemon 부재로 실행할 수 없었다.
- `node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs`와 일반 `e2e:embed`는 `pkg/rhwp.d.ts`에 없는 `getFontDecisionTrace` 때문에 중단됐다. 이 Rust export와 `pkg/`는 PR #5569 diff 밖의 최신 `devel` 기준선이며, document-agent E2E 자체는 통과했다. 이 사유로 PR 변경을 실패로 분류하지 않는다.

### GitHub Actions

- code candidate `364962caf60189b22dad244bb47f340dd2262261`의 CI preflight, Frontend package gate, Build & Test aggregate, CodeQL 세 언어, Proptest, Adapter inter-diff, Canvas visual diff가 성공했다.
- Rust·Native Skia·archive worker는 impact policy에 따라 skipped됐으며, Frontend package 변경 범위에 대한 gate가 실행된 것을 확인했다.

## 다음 게이트

이 문서와 오늘할일만 담은 single-parent trailing commit을 contributor source branch에 push한다. 이후 최신 head의 review-only fast-pass preflight와 Build & Test aggregate가 성공하고 mergeability가 유지될 때 first-time contributor approval comment를 게시한 뒤 merge한다.
