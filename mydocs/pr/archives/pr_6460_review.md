---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6460
issue: 6452
author: postmelee
---

# PR #6460 review - 머리말/꼬리말 선택 렌더 트리 캐시 재사용

## 라우팅과 metadata

- PR: [#6460](https://github.com/edwardkim/rhwp/pull/6460), 관련 이슈:
  [#6452](https://github.com/edwardkim/rhwp/issues/6452).
- base route: `collaborator_self_merge.md`; modifiers: `intake_and_review.md`,
  `local_validation.md`.
- loaded documents: `AGENTS.md`, `CLAUDE.md`, `mydocs/manual/codex/MEMORY.md`,
  `mydocs/README.md`, `mydocs/manual/pr_review_workflow.md`,
  `mydocs/manual/pr_review/README.md`, `mydocs/manual/pr_review/collaborator_self_merge.md`,
  `mydocs/manual/pr_review/intake_and_review.md`, `mydocs/manual/pr_review/local_validation.md`,
  `mydocs/manual/codex/docs_and_git_workflow.md`.
- 작성자·self-review: `postmelee`; collaborator 본인 PR이므로 reviewer request는 등록하지 않았다.
- 검토 대상 base는 `devel`, 로컬 code candidate는
  `764a207de48ec9fb370d38bfcd32c84069f2983f`이다. 보정 전 원격 head
  `cbeccf7260681a0707f8ca8a5c305193d67d3c89`의 required checks는 모두 성공했고,
  이 문서를 포함한 trailing head의 CI는 push 뒤 다시 확인해야 한다.

## 변경 범위와 판단

- 실제 적용 페이지의 page tree cache를 머리말/꼬리말 선택·커서·hit-test 질의에서도 재사용한다.
- 구역 첫 페이지에 임시 투영하는 홀수/짝수 정의는 별도의 단일 preview cache로 재사용한다.
- 문서 geometry를 바꾸는 편집 경로에서 실제 페이지와 preview cache를 함께 무효화한다.
- 시계 기반 수치 대신 page tree build counter 회귀 테스트로 반복 프레임 재사용과 편집 후 1회
  재빌드를 결정적으로 고정한다.
- 공개 API, 저장 형식과 렌더 출력은 바꾸지 않는 내부 성능 변경으로 범위가 통제되어 있다.

## 리뷰 보정

- 리뷰에서 확인된 `try_patch_cached_focused_cell_tail_line`의 preview cache 무효화 누락을
  `764a207de`에서 보정했다. layer tree JSON cache를 비우는 시점에
  `header_footer_preview_tree_cache`도 함께 비워 캐시 무효화 불변식을 맞췄다.
- batch mode에서 중간 질의의 신선도 의미가 달라질 가능성은 현재 생산 호출부가 표·셀 묶음 작업에
  한정되고 배치 중 HF rect를 조회하지 않아 이 PR의 blocker가 아니다. 계약을 넓힐 때 별도 회귀
  테스트와 함께 다루는 후속 항목으로 남긴다.
- cursor rect not-found 탐색이 방문한 페이지 tree를 캐시에 남기는 보존량 증가는 드문 실패 경로의
  메모리 특성이다. 캐시 build 횟수 개선과 정확성에는 영향이 없으므로 큰 이미지 문서 측정과 보존
  정책을 별도 성능 후속으로 다루는 것이 적절하다.

## 로컬 검증

- `node scripts/rust-test-suite-manifest.mjs --prepare` 후 `--check`: 통과
  (1,041 sources / 4,572 static test attrs / 32 suites + 16 exceptions / 최소 6,559 cases).
- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`: 통과.
- native Clippy, WASM Clippy, workspace build, workspace all-target Clippy의 `-D warnings` 묶음: 통과.
- #6452 focused integration test: 2/2 통과.
- 전체 integration nextest: 8,723/8,723 통과, 43 skipped.
- Native Skia 전체 library test: 통과.
- Native Skia 필수 renderer 회귀: missing-picture 2/2, direct-PDF 4/4 통과.
- `scripts/wasm-pack-locked.sh --target web --out-dir pkg`: 최적화 WASM package 생성 통과.
- 실제 Google Chrome `npm run e2e:issue-4121`: 56/56 통과.

## 시각 영향과 증적

- page tree 생성 횟수와 캐시 생명주기만 바꾸며 canvas, SVG, PDF의 geometry·색상·표시 계약은
  변경하지 않는다. 새 기준 이미지나 renderer fixture 갱신은 필요하지 않다.
- 보정 전 원격 head의 Canvas visual diff와 Native Skia checks가 성공했다.
- 보정 후보의 실제 Chrome E2E에서 다문단 선택, 반복 페이지 투영, 홀짝 전환과 편집 종료를 다시
  확인했다. E2E가 만든 ignored screenshot과 HTML report는 검증 산출물이며 stage하지 않았다.

## 최종 권고와 후속 조건

**조건부 수용.** 캐시 재사용의 정확성, 편집 후 무효화와 실제 브라우저 동작을 로컬 검증했으며,
리뷰의 직접 보정 요청도 반영했다. batch 중간 질의와 not-found 보존량은 비차단 후속 항목이다.

- 이 self-review를 trailing 문서 commit으로 같은 branch에 추가한다.
- 최신 PR head의 required checks가 모두 성공하고 `MERGEABLE/CLEAN`인지 확인한다.
- merge는 작업지시자의 별도 승인 뒤 수행하며, #6452는 PR 본문의 `closes #6452`에 따라 merge 시 닫는다.
