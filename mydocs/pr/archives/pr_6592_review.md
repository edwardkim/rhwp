# PR #6592 검토 기록 — v0.8.6 devel → main 릴리스 승격

- PR: [#6592](https://github.com/edwardkim/rhwp/pull/6592)
- 이슈: [#6584](https://github.com/edwardkim/rhwp/issues/6584)
- 작성자·검토자: `edwardkim` collaborator self-review
- base: `main@496333b27d21ddb9114ba9ae340bcb895870c9a7`
- 검토한 code head: `devel@7dcf162bef223dc2bcd426da74f5a394de52c959`
- 규모: 2,236 commits, 17,404 files, +5,710,849 / -92,051
- 검토일: 2026-09-02 KST

## 1. 리뷰 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`,
  `visual_fixture_evidence.md`, `review_only_fast_pass.md`, `rework_and_exceptions.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
  `pr_review/collaborator_self_merge.md`, `pr_review/intake_and_review.md`,
  `pr_review/local_validation.md`, `pr_review/visual_fixture_evidence.md`,
  `pr_review/review_only_fast_pass.md`, `pr_review/rework_and_exceptions.md`
- 작성자 본인의 self-review이므로 reviewer를 지정하지 않았다.

## 2. 범위와 계보

이 PR은 `main`의 v0.8.4 기준선을 최신 검증된 `devel`로 승격하는 누적 릴리스 PR이다. 신규 기능이나
신규 배포 채널을 추가하지 않는다. 공개 기능·기여자 계측 범위는
`v0.8.4..063041a2ced54085b5cf94c2e646ac7aa0e1960d`의 2,214 commits / 262 PR provenance / 사람
20명으로 고정돼 있다. 그 뒤 현재 head까지의 22개 commit은 #6584 릴리스 준비·검증·기록,
#6588 계측 문구 정정과 #6594 canonical promotion CI 정정이며 새 제품 기능이나 사람 기여자를 더하지 않는다.

기여자 ledger, `CHANGELOG.md`, `CHANGELOG_EN.md`와
[릴리스 노트 초안](../../working/task_m100_6584_release_notes.md)의 사람 20명 집합은 release record
계약 검사에서 일치했다. #6584는 배포 채널 정산 전, #5949는 실제 Linux AArch64 release asset 확인 전,
#6243은 실제 post-release Render Diff canary 전까지 OPEN으로 유지한다.

## 3. 검증

### 3.1 릴리스 후보와 배포물

- [Stage R4 결과](../../working/task_m100_6584_stage_r4.md): 전체 nextest 8,925 pass / 0 fail,
  Rust native·WASM·workspace lint, Native Skia, Docker WASM, Studio 1,362 pass / 0 fail,
  확장·VS Code build와 Chrome CDP E2E가 통과했다.
- PR #6585 exact product candidate `4280831d1f25a189416c2fcec14e0d252dfb90c3`의 Full CI
  [#33568191662](https://github.com/edwardkim/rhwp/actions/runs/33568191662), CodeQL
  [#33568191630](https://github.com/edwardkim/rhwp/actions/runs/33568191630), Render Diff
  [#33568191576](https://github.com/edwardkim/rhwp/actions/runs/33568191576), Proptest와 Adapter가 성공했다.
- Release Binary dry-run
  [#33569503350](https://github.com/edwardkim/rhwp/actions/runs/33569503350)에서 Windows x86_64,
  Linux x86_64/AArch64, macOS x86_64/AArch64 5개 target이 성공했다. Linux AArch64 native runner에서
  ELF AArch64, mode 0755와 `rhwp v0.8.6`을 확인했다.
- 위 product candidate 뒤 현재 head까지 제품 source·renderer·fixture 변경은 없다. 변경은 릴리스 기록,
  review 문서, canonical promotion workflow와 그 계약 test뿐이다. 따라서 제품 후보의 Render Diff·CDP
  증적을 현재 승격의 시각 근거로 재사용하고 새 visual sweep을 중복 실행하지 않았다.

### 3.2 canonical promotion exact head

#6594 병합 뒤 갱신된 exact head `7dcf162bef223dc2bcd426da74f5a394de52c959`에서 실제
`devel → main` Full 경로가 성공했다.

- CI [#33579621944](https://github.com/edwardkim/rhwp/actions/runs/33579621944): promotion manifest,
  unit-tier, native/WASM/workspace Clippy, Native Skia, frontend package, archive A/B/C/D와
  `Build & Test` 성공.
- CodeQL [#33579621943](https://github.com/edwardkim/rhwp/actions/runs/33579621943): Rust,
  JavaScript/TypeScript, Python 분석 성공.
- Proptest [#33579621950](https://github.com/edwardkim/rhwp/actions/runs/33579621950), Adapter
  [#33579621952](https://github.com/edwardkim/rhwp/actions/runs/33579621952), Skill router가 성공했다.
- PR check 집계는 56 success / 5 policy skip / 0 failure / 0 pending이었다.
- 같은 merge commit의 `devel` push CI [#33579615378](https://github.com/edwardkim/rhwp/actions/runs/33579615378)와
  CodeQL [#33579615259](https://github.com/edwardkim/rhwp/actions/runs/33579615259)도 성공했다.

focused 재검증은 release channel·contributor record 19/19, font trace 12/12, manifest 계약 21/21,
CI 영향 계약 33/33이 통과했다. `main`은 code head의 조상이고 merge-tree는 head tree
`16f73b5d1c416a3b77db27ae3af49438fe0e472f`와 일치했다.

전체 `main..devel`의 `git diff --check`는 과거 `devel`에 이미 병합된 CRLF 문서·CSV의 공백 경고를
보고한다. 승격 단계에서 이력 전체의 byte를 재작성하지 않는다. 이번 review-only 후행 commit 자체는
별도로 `git diff --check`와 Markdown 링크 검사를 통과시킨다.

## 4. 발견 사항과 잔여 조건

- 차단되는 코드·배포물·CI 결함은 발견하지 못했다.
- PR 본문은 이전 head `bd8cdd0a1`과 2,232 commits, 이전 merge-tree를 적고 있다. review-only head를
  push할 때 검증한 code head `7dcf162b`, 현재 2,236-commit 범위와 tree를 구분해 현행화해야 한다.
- 실제 tag, GitHub Release, 다섯 최종 archive와 `SHA256SUMS.txt`, npm·VS Code/Open VSX·브라우저
  스토어 게시 결과는 아직 존재하지 않는다. 이는 main 병합 뒤 별도 승인으로 진행한다.
- 이 review와 오늘할일은 `mydocs/`만 바꾸는 single-parent trailing commit이다. push 뒤 최신 head의
  review-only fast-pass aggregate와 mergeability를 다시 확인한다.

## 5. 최종 판정

- 판정: 승인
- 검증한 code head: `7dcf162bef223dc2bcd426da74f5a394de52c959`.
- merge 전 조건: review-only 기록을 포함한 최신 head의 required checks 성공, PR 본문 현행화,
  `MERGEABLE / CLEAN` 재확인과 메인테이너의 별도 정상 merge commit 승인.
- merge 후 조건: exact `main` merge commit의 필수 check를 확인한 뒤 별도 승인으로 annotated
  `v0.8.6` tag와 GitHub Release를 생성한다.
- 이 기록 자체는 원격 push, GitHub review event, merge, tag 또는 Release를 수행하지 않는다.
