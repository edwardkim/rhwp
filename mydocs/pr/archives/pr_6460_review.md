---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
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
- 검토 대상 base는 `devel` `3afbb066fe93724ab44309163a2e04efb954bf18`, 충돌 해소
  code candidate는 `5846201178a62d5019b0858c881bd3655eb1ac4b`이다.
- 기존 원격 head `a8081eccb36f4059c2a1250801753888d2dd131b`의
  [전체 CI](https://github.com/edwardkim/rhwp/actions/runs/33306117491)와
  [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/33306117409)는 성공했다.
  충돌 해소로 Rust source와 회귀 테스트가 바뀌었으므로 이 결과를 새 후보의 검증으로 대신하지 않는다.

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

## 기존 보정 후보 로컬 검증 (2026-08-30)

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

## 2026-08-31 충돌 해소

- 최신 base의 #4969 page-layer cache 도입과 이 PR의 preview cache 무효화가
  `commands/document.rs`, `commands/header_footer_ops.rs`, `queries/rendering.rs`에서 겹쳤다.
- 새 문서 생성은 공통 `invalidate_page_tree_cache()`를 사용하고, 한 페이지의 HF 숨김 변경은
  `invalidate_page_tree_cache_page()`를 사용하도록 최신 base의 공통 경로를 보존했다.
- 공통 page-local 무효화에 HF preview cache 초기화를 추가했다. focused-cell patch 경로에서는
  page-layer cache와 HF preview cache를 둘 다 비운다.
- 기존 #6452 regression test에 한 페이지의 머리말 숨김 변경 뒤 대표 꼬리말 preview가 정확히
  한 번 재빌드되고 이후 질의가 캐시를 재사용하는 assertion을 추가했다.
- #6461 원격 head `c0603b9f139266e3ca59afcdf8e6ac9e98a2d291`을 누적한 merge simulation도
  clean이다. 결과 tree `f14d5b8f820372cff958726dd38cf816f34ea910`에서 canonical HF resolver와
  preview cache 무효화가 함께 유지됨을 코드로 확인했다. 누적 tree 자체의 실행 검증은 아니다.

## 충돌 해소 후보 로컬 검증 (2026-08-31)

- 파생 suite 준비·manifest check: 통과
  (1,076 sources / 4,717 static test attrs / 28 suites + 20 exceptions / 최소 6,559 cases).
- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`: 통과.
- native Clippy, WASM Clippy, workspace build, workspace all-target Clippy: 순차 통과.
- #6452 focused integration test: 2/2 통과 (page-local 무효화 assertion 포함).
- 전체 integration nextest: 8,864/8,864 통과, 46 skipped (1 slow, 1 leaky 보고).
  macOS host의 12 logical CPU / 24 GiB RAM에서 test threads 6으로 실행했다.
- Native Skia 전체 library test: 4,128 passed, 13 ignored.
- Native Skia 필수 renderer 회귀: missing-picture 2/2, direct-PDF 4/4 통과.
- 최적화 WASM package: `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh
  --target web --out-dir pkg` 통과. Docker daemon 미실행으로 macOS native wrapper를 사용한
  host fallback이며 표준 Docker 검증을 통과했다고 간주하지 않는다. 최초 sandbox 실행은
  wasm-bindgen/wasm-opt 도구 캐시 권한에 막혔고, 같은 명령의 권한 상승 재실행으로 완료했다.
- 새 WASM과 실제 Google Chrome의 `npm run e2e:issue-4121`: 56/56 통과.
  다문단 선택·치환·복사·잘라내기·서식·IME·Undo/Redo·반복 페이지 투영·홀짝 전환을 확인했다.
- 검증 종료 시 원격 `devel`은 여전히 `3afbb066fe93724ab44309163a2e04efb954bf18`임을 확인했다.

## 시각 영향과 증적

- page tree 생성 횟수와 캐시 생명주기만 바꾸며 canvas, SVG, PDF의 geometry·색상·표시 계약은
  변경하지 않는다. 새 기준 이미지나 renderer fixture 갱신은 필요하지 않다.
- 보정 전 원격 head의 Canvas visual diff와 Native Skia checks가 성공했다.
- 보정 후보의 실제 Chrome E2E에서 다문단 선택, 반복 페이지 투영, 홀짝 전환과 편집 종료를 다시
  확인했다. E2E가 만든 ignored screenshot과 HTML report는 검증 산출물이며 stage하지 않았다.

## 최종 권고와 후속 조건

**승인.** 충돌 해소 후보의 필수 로컬 검증을 통과했고 두 PR을 함께 반영해도 코드 충돌이 없다.
batch 중간 질의와 not-found 보존량은 기존 비차단 후속 항목이다.
이 판정은 기술 검토 결과이며 원격 push·merge 실행 승인이 아니다.

- 갱신한 self-review를 trailing 문서 commit으로 같은 branch에 추가한다.
- 충돌 해소 후보는 아직 원격에 push하지 않았으므로 GitHub의 기존 `CONFLICTING` 표시는 남아 있다.
- push 후 새 trailing PR head의 required checks와 `MERGEABLE/CLEAN`을 확인한다. 기존 녹색 head의
  CI를 재사용해 새 source candidate를 병합하지 않는다.
- #6460을 먼저 병합한 뒤 #6461의 변경된 base와 최신 head 상태를 다시 확인하는 순서를 권고한다.
- merge는 작업지시자의 별도 승인 뒤 수행하며, #6452는 PR 본문의 `closes #6452`에 따라 merge 시 닫는다.
