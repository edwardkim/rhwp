---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6461
issue: 6453
author: postmelee
---

# PR #6461 review - 머리말/꼬리말 편집 API와 이벤트 계약 정리

## 라우팅과 metadata

- PR: [#6461](https://github.com/edwardkim/rhwp/pull/6461), 관련 이슈:
  [#6453](https://github.com/edwardkim/rhwp/issues/6453).
- base route: `collaborator_self_merge.md`; modifiers: `intake_and_review.md`,
  `local_validation.md`.
- loaded documents: `AGENTS.md`, `CLAUDE.md`, `mydocs/manual/codex/MEMORY.md`,
  `mydocs/README.md`, `mydocs/manual/pr_review_workflow.md`,
  `mydocs/manual/pr_review/README.md`, `mydocs/manual/pr_review/collaborator_self_merge.md`,
  `mydocs/manual/pr_review/intake_and_review.md`, `mydocs/manual/pr_review/local_validation.md`,
  `mydocs/manual/codex/docs_and_git_workflow.md`.
- 작성자·self-review: `postmelee`; collaborator 본인 PR이므로 reviewer request는 등록하지 않았다.
- 검토 대상 base는 `devel`, 로컬 code candidate는
  `4ab7d5f798277175b349f23f4593a6fbebadef02`이다. 보정 전 원격 head
  `575e7c88e97903a5693baae102460c996a57d279`의 required checks는 모두 성공했고,
  이 문서를 포함한 trailing head의 CI는 push 뒤 다시 확인해야 한다.

## 변경 범위와 판단

- `applyFrom`/`applyTo` 변환과 HF control 탐색을 canonical helper로 통합해 편집 command와
  renderer가 같은 정의를 선택한다.
- Studio의 대표 편집 페이지를 `previewPage` 계약으로 통일하고 공개 커서 API는 문단·문자 좌표와
  명시적 preview page를 받는다.
- HF 삽입·삭제·범위 치환을 `HeaderFooterTextReplaced` 원자 이벤트로 기록해 실제 clamp된 편집 전
  반열린 범위와 편집 후 `insertedEnd`를 제공한다.
- 저장 형식과 사용자 UI는 바꾸지 않으며 내부 resolver, bridge, 이벤트 payload의 모호성을 줄이는
  계약 정리로 범위가 통제되어 있다.

## 리뷰 보정

- 체크인한 `rhwp-studio/public/rhwp.d.ts`의 `preview_page_hint` 설명을 Rust 생성 원본과 축자
  일치하도록 `4ab7d5f79`에서 보정했다. 다음 WASM package 갱신 때 발생할 주석 churn을 제거했다.
- IME 조합 시작의 HF caret 계산도 `getRect()?.pageIndex` 우회 대신 `hfPreviewPage`를 직접 전달하도록
  같은 후보에서 보정했고, source guard test로 계약을 고정했다.
- HF 문단 split/merge가 범용 `ParagraphSplit`/`ParagraphMerged` 이벤트를 계속 사용하는 점은 유효한
  후속 설계 항목이다. 이번 이슈가 정리한 텍스트 replace 이벤트와 별개의 variant 확장이고, 현재
  Studio 소비자가 해당 범용 이벤트를 HF/본문 판별에 사용하지 않아 blocker는 아니다. 각주 등 다른
  편집 컨텍스트까지 포함한 이벤트 식별 계약으로 별도 이슈에서 다루는 것이 적절하다.

## 로컬 검증

- `git diff --check`: 통과.
- #6453 Studio source contract test: 3/3 통과.
- `npx tsc --noEmit`: 통과.
- Studio 전체 `npm test`: 1,312 passed, 1 skipped.
- Studio production build: 244 modules, 통과.
- 실제 Google Chrome `npm run e2e:issue-4121`: 56/56 통과.
- Rust source는 보정 전 원격 head와 동일하다. 해당 head에서 CI의 lint 묶음, 전체 Rust archive,
  Native Skia, Canvas visual diff, CodeQL과 package gate가 모두 성공했다.

## 시각 영향과 증적

- d.ts 문서와 IME 대표 페이지 전달 경로만 보정하며 UI 모양과 renderer 출력은 바꾸지 않는다.
  새 기준 이미지, PDF/SVG visual sweep과 screenshot 갱신은 필요하지 않다.
- 실제 Chrome E2E에서 HF 진입, 선택·치환·IME, 반복 페이지 투영, 홀짝 target 전환을 확인했다.
  E2E가 만든 ignored screenshot과 HTML report는 검증 산출물이며 stage하지 않았다.

## 최종 권고와 후속 조건

**조건부 수용.** 대표 페이지와 텍스트 변경 이벤트 계약을 자동 검증했고, 리뷰의 주석 동기화와 IME
우회 경로도 보정했다. 문단 split/merge 이벤트 식별은 비차단 후속 설계 항목이다.

- 이 self-review를 trailing 문서 commit으로 같은 branch에 추가한다.
- 최신 PR head의 required checks가 모두 성공하고 `MERGEABLE/CLEAN`인지 확인한다.
- merge는 작업지시자의 별도 승인 뒤 수행하며, #6453은 PR 본문의 `closes #6453`에 따라 merge 시 닫는다.
