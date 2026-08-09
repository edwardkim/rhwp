---
kind: pr_review
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4271 검토 — #4162 글자 서식 no-op + #4270 문서 전환 예약 누수

> 이 문서는 contributor가 최초 PR 후보에서 작성한 self-review의 역사 기록이다. 후속
> 메인테이너 보정·최종 검증·merge 결과는 [PR #4271 메인테이너 검토](pr_4271_review.md)를
> 권위 기록으로 사용한다.

## 결론

**Open PR 생성, 로컬 TypeScript·전체 studio 테스트·`git diff --check` 통과, 실제 wasm
문서 왕복 행위 테스트와 `:7701` 실행 확인 완료.** 캐럿·F5 셀 블록 선택 상태에서 툴바
글자 색/글꼴/크기가 조용히 no-op이던 결함(#4162)을 캐럿 대기 글자 모양(pending char
shape) 예약 메커니즘으로 고쳤다. 구현 뒤 자체 리뷰(gestell skill)에서 그 예약이
`deactivate()`/`dispose()`에 리셋되지 않아 문서 전환 시 새어 들어갈 수 있는 결함을
찾아 별도 이슈(#4270)로 등록하고 같은 PR에서 함께 고쳤다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, intake_and_review.md,
                  local_validation.md
devel base: fc8eb90d6 (upstream/devel HEAD, #4266 merge 포함)
validated code head: a9a055ff9
```

원 후보는 로컬 `integration/all-works`(다른 이슈와 미커밋 무관 변경이 섞인 통합
branch) 대신, 최신 `upstream/devel` 위에 새 branch(`task_m100_4162`)로 처음부터
구현했다 — #4266 리뷰에서 겪은 것과 같은 이유(오래된 base·무관 이슈 혼입)를 사전에
피하기 위함이다. PR 생성 준비 중 `upstream/devel`이 `c391dbbdc` → `fc8eb90d6`(#4266
merge 포함)로 전진해 `git rebase devel`을 실행했다 — `#4266`이 건드린 `onInput`
조합 catch 블록과 이 PR이 그 바로 아래 줄에 추가한
`this.applyPendingCharShapeToRange?.(anchor, charCount(text));`가 서로 다른
hunk라 수동 conflict 없이 자동 3-way merge로 정리됐다. rebase 뒤 파일을 직접 읽어
두 수정이 올바르게 공존하는지 확인했다.

별도 `review_impl` 문서는 만들지 않았다. 단일 이슈 쌍(#4162 본 결함 + #4270 리뷰
중 발견)의 3-커밋 fix이고 커밋 메시지 자체가 실행 순서를 충분히 설명한다.

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4271](https://github.com/edwardkim/rhwp/pull/4271) |
| 관련 이슈 | [#4162](https://github.com/edwardkim/rhwp/issues/4162), [#4270](https://github.com/edwardkim/rhwp/issues/4270)(리뷰 중 발견해 새로 등록) |
| 작성자 | `humdrum00001010` (fork 기반 — `upstream` 직접 push 권한 없음) |
| reviewer | `jangster77` 지정 시도 — 권한 부족으로 `gh pr edit --add-reviewer` 실패, 작업지시자가 수동 지정 필요 |
| 대상 / head | `devel` / `humdrum00001010:task_m100_4162` (fork branch) |
| 생성 상태 | open, non-draft |
| 생성 시점 규모 | +419 / -46, 3 commits |
| 생성 시점 merge 상태 | mergeable, `BLOCKED` — CI 진행 중 참고값 |

## 렌더 영향 판정

시각·fixture 증적 보조 경로는 적용하지 않는다. `src/renderer`, `wasm_api.rs`,
golden/fixture, HWP/HWPX sample을 전혀 건드리지 않는 rhwp-studio 엔진·커맨드
레이어 변경이다.

## 로컬 검증

Rust/wasm 변경이 없는 rhwp-studio 전용 PR이라 [4.3 표](../../manual/pr_review/local_validation.md#43-변경-범위별-기본-검증)의
"rhwp-studio만 변경" 행을 따랐다.

| 검증 | 결과 |
| --- | --- |
| `npx tsc --noEmit` | PASS |
| `node --test tests/*.test.ts ../npm/editor/tests/*.test.mjs` | 818/818 pass |
| `git diff --check` (devel..HEAD) | PASS |
| 새 행위 테스트 (`pending-char-shape.runner.mjs`) | 실제 `pkg-node` wasm 문서에 대고 선택 범위 서식 적용+undo, 캐럿 대기 서식이 다음 삽입 런에만 적용, 서로 다른 예약 서식끼리 병합 금지, 빈 range(캐럿) no-op을 실제 커맨드 실행으로 검증 |
| `:7701` 실브라우저 확인 | 새 문서에서 캐럿만 두고 색 피커 → 연속 타이핑 2글자 모두 지정 색 반영을 `window.__wasm.doc.getCharPropertiesAt`으로 확인. IME 조합 합성 이벤트로 조합 텍스트 범위만 정확히 물드는 것도 확인(스코프 바깥 미영향) |
| #4270 회귀 재현·수정 확인 | `window.__inputHandler.deactivate()`를 직접 호출해 수정 전 `pendingCharShape`가 안 지워짐을 실행으로 재현, 수정 커밋 뒤 재실행해 `null`로 지워짐을 확인 |

wasm 타입 선언은 fresh `wasm-pack build`를 새로 돌리지 않고 원 checkout의 기존
`pkg/`를 재사용했다 — 이 PR이 `.rs` 파일을 바꾸지 않아 API 표면이 같다.

## 발견한 문제와 처리

- (해소) `deactivate()`/`dispose()`가 `pendingCharShape`/`pendingCharShapeAnchor`를
  리셋하지 않던 결함 — gestell 리뷰로 발견, #4270으로 등록, 같은 PR에서 수정.
- (범위 밖, 별도 정리) 구현 중 시도했던 `src/command/command-feedback.ts`(실행
  불가 커맨드를 조용히 사라지지 않게 하는 helper)는 실제 커맨드 호출부
  (`format.ts`/`edit.ts`의 `getInputHandler()?.`)를 하나도 바꾸지 않아 프로덕션
  경로 어디서도 쓰이지 않는 죽은 코드였다. `integration/all-works`에 같은 목적의
  더 완성된(showToast 연동) 구현이 이미 있어 이후 병합 충돌을 피하려고 이 PR에서는
  제거했다 — `format-command-visible-failure.test.ts`가 pin하는 그 별도 구현은
  이 PR의 범위가 아니다.

## 작성 시점 GitHub Actions와 남은 게이트

- 생성 직후 `mergeStateStatus: BLOCKED`는 CI 진행 중 참고값이며 최신 head의
  required check 성공을 merge 직전에 다시 확인해야 한다.
- reviewer 지정은 권한 부족으로 실패했다 — 작업지시자가 GitHub에서 직접
  `jangster77`(또는 다른 reviewer)을 지정해야 한다.
- `closes #4162`, `closes #4270` 모두 저장소 기본 branch(`main`)와 대상 branch(`devel`)가
  달라 자동 종료를 보장하지 않는다 — 두 이슈 모두에 이 PR을 링크하는 댓글을 남겼고,
  merge 뒤 수동으로 상태를 확인·종료한다.

## 작성 시점 최종 권고

로컬 TypeScript·전체 테스트·diff 검사·실브라우저 확인 모두 통과했다. 최신 head의
required CI 성공과 reviewer 지정·승인 확인 뒤 merge를 권고한다.
