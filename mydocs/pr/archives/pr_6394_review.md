---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6394
issue: 4121
author: postmelee
---

# PR #6394 review - 머리말/꼬리말 텍스트 선택

## 라우팅과 metadata

- PR: [#6394](https://github.com/edwardkim/rhwp/pull/6394), 관련 이슈:
  [#4121](https://github.com/edwardkim/rhwp/issues/4121).
- base route: `collaborator_self_merge.md`; modifiers: `intake_and_review.md`,
  `local_validation.md`, `review_only_fast_pass.md`.
- loaded documents: `AGENTS.md`, `mydocs/manual/pr_review_workflow.md`,
  `mydocs/manual/pr_review/README.md`, `mydocs/manual/pr_review/collaborator_self_merge.md`,
  `mydocs/manual/pr_review/intake_and_review.md`, `mydocs/manual/pr_review/local_validation.md`,
  `mydocs/manual/pr_review/review_only_fast_pass.md`.
- 작성자·self-review: `postmelee`; collaborator 본인 PR이므로 reviewer request는 등록하지 않았다.
- 작성 시점 참고값: base `devel`, code candidate
  `3a83540cd0194f99594dd92e6ca4fd3f84b76926`, Open non-draft, `MERGEABLE/BLOCKED`,
  49 files, 18 commits, `+5,569/-136`. `BLOCKED`는 진행 중인 required check가 남은 상태다.

## 변경 범위와 판단

- 머리말/꼬리말 텍스트를 본문과 분리된 범위로 선택하고 마우스·키보드 선택, 다문단 범위,
  교체·삭제·복사·잘라내기·부분 서식과 Undo/Redo를 지원한다.
- 같은 Both/Odd/Even 정의가 적용되는 반복 페이지에 선택과 편집 결과를 투영하고, 홀수·짝수 정의도
  구역 첫 페이지에서 대표 편집한다. 현재 정의는 도구 상자와 canvas 라벨로 표시한다.
- 새 머리말/꼬리말의 기본 문단 정렬은 왼쪽으로 설정해 공백 입력이 짧은 마지막 줄을 과도하게
  벌리는 현상을 피한다.
- 편집 영역 전체를 칠하던 초기 표시는 제거했다. 기존 페이지 여백 안내선과 같은 꺾쇠, 배율에 대응하는
  텍스트 라벨만 사용해 실제 문서 내용을 가리지 않는다.
- 머리말/꼬리말 내부 표·그림 개체 편집과 한컴 전용 대화상자·리본 전체 복제는 의도적으로 제외했다.

## 로컬 검증

- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`: 통과.
- `node scripts/rust-unit-test-tiers.mjs --check`: 4,221 tests / 299 modules, 통과.
- #4121 focused Rust integration test: 7/7 통과.
- 관련 Studio Node test 6개 파일: 43/43 통과.
- Studio production build: 241 modules, 통과.
- 실제 Google Chrome `npm run e2e:issue-4121`: 56/56 통과.
- 사용자가 요청한 최소 검증 범위에 따라 최종 rebase 후 전체 `cargo test`/nextest/clippy와 전체 최적화
  WASM rebuild는 다시 실행하지 않았다. 동일한 논리 변경 tree의 이전 단계에서 전체 nextest,
  Studio 전체 test, Clippy와 최적화 WASM 검증을 완료했으며, 최종 후보에는 focused Rust·Studio·Chrome
  E2E를 다시 적용했다.

## 시각 영향과 증적

- Studio canvas의 머리말/꼬리말 편집 라벨, 공용 페이지 여백 꺾쇠와 선택 overlay가 바뀐다.
- HWP/PDF fixture, renderer 출력 또는 기준 PDF는 변경하지 않아 PDF/SVG visual sweep 대상은 아니다.
- 실제 브라우저 E2E에서 다문단 선택·반복 페이지 투영·홀짝 전환을 확인했고 다음 ignored 로컬 화면을
  생성했다. 소스 PR에는 stage하지 않았으며 작업지시자가 PR 본문에 직접 첨부할 수 있다.

  - `issue4121-stage4-both-header-multiline-selection.png`
  - `issue4121-stage4-odd-even-footer-switch.png`

## 최종 권고와 후속 조건

**조건부 수용.** #4121의 선택·편집·반복 정의 계약과 대표 편집 UI를 focused 자동 검증 및 실제 Chrome
E2E로 확인했다.

- 이 self-review와 오늘할일을 포함한 trailing 문서 commit을 같은 branch에 추가한다.
- 최신 PR head의 required checks가 모두 성공하고 `MERGEABLE/CLEAN`인지 확인한다.
- merge는 작업지시자의 별도 승인 뒤 수행하며, #4121은 PR 본문의 `Closes #4121`에 따라 merge 시 닫는다.
