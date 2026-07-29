# kevin9327 PR #3457–#3541 통합 검토·구현 기록

## 라우팅과 범위

```text
base route: collaborator_external_pr
modifiers: intake_and_review, local_validation, multi_pr_update_branch,
  visual_fixture_evidence, rework_and_exceptions, post_merge
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_external_pr.md, intake_and_review.md, local_validation.md,
  multi_pr_update_branch.md, visual_fixture_evidence.md,
  verification/visual_verification_governance.md,
  verification/visual_sweep_guide.md, rework_and_exceptions.md, post_merge.md
```

기본 작업트리에 다른 작업의 미추적 파일이 있어 이를 변경하지 않는다. clean worktree
`/Users/tsjang/rhwp-review-kevin-20260729`의
`review/kevin9327-20260729`에서 최신 `upstream/devel`
`2f281d67f2df73ecc5f2a612f36bc1579b69b41f`를 통합 기준으로 삼는다.

원 contributor branch에는 push, rebase, amend, force-push를 하지 않는다. 오래된 base와
서로 충돌하는 CLI PR은 원 PR을 직접 merge하지 않고, 기능 commit만 `git cherry-pick -x`로
통합 후보에 적용한다. PR #3539에는 PR #3534의 세 commit이 포함돼 있으므로 #3534를 먼저
적용한 뒤 #3539 고유 commit 하나만 적용한다. PR #3478·#3457의 `Merge branch 'devel'`
commit도 제외한다.

| 적용 순서 | 원 PR | source head (작성 시점 참고값) | 체리픽 대상 |
| ---: | --- | --- | --- |
| 1 | #3457 | `6bded579e614` | `8614706cf023` |
| 2 | #3490 | `56812b7d5dd4` | `56812b7d5dd4` |
| 3 | #3533 | `8ae1a3cc8485` | `8ae1a3cc8485` |
| 4 | #3534 | `8d5a171d005d` | `c65d2001f2fe`, `b001d5123cfc`, `8d5a171d005d` |
| 5 | #3539 | `86b1cb173aff` | `86b1cb173aff` |
| 6 | #3537 | `c05c4a51b21a` | `9821cc3844a6`, `95315143f361`, `c05c4a51b21a` |
| 7 | #3540 | `fe1c2b181d51` | `17023c0457bc`, `fe1c2b181d51` |
| 8 | #3478 | `7676ddd77d5f` | `098ae52437dc` |
| 9 | #3482 | `5f7efa90c9c6` | `5f7efa90c9c6` |
| 10 | #3541 | `6be8efb58117` | `6be8efb58117` |

## 단계와 종료 조건

1. 원 PR source head, mergeable 및 CI를 재확인한다.
2. 위 순서로 15개 고유 contributor commit을 `-x` 체리픽했다. #3541의 `main.rs`·CLI 문서 conflict는
   #3478의 `ambiguous`, #3482의 `overflow`, #3541의 `outputFormat`을 모두 노출하도록 해소했다.
   결과 contributor 체리픽은 `1050903f8`부터 `5db6d24df`까지 저자와 source SHA를 보존한다.
   충돌 해소가 요구사항을 바꾸면 작업지시자에게 판단을 요청한다.

통합 validation에서 #3533의 새 test에 `Option::or_else(|| None)` no-op closure가 있어 `clippy -D
warnings`를 막는 것을 확인했다. 동작을 바꾸지 않고 그 체인을 제거한 메인터너 commit
`e0efa3ea3`으로 보정했다.
3. PR별 diff·관련 issue·기존 조사/트러블슈팅을 검토하고, CLI·parser·renderer 범위에 맞는 focused 및
   통합 검증을 순차 수행한다. renderer/HML 변경은 시각·fixture 증적 가이드를 추가 적용한다.
4. 원 PR별 archive review와 이 통합 기록에 체리픽 SHA·검증·판정을 기록한다. 필요한 review asset과
   오늘할일을 같은 review commit으로 추가한다.
5. 작업지시자 승인 범위에서 원본 저장소의 임시 head branch를 push하고 `devel` 대상 통합 PR을 만든다.
   최신 head CI 및 mergeable을 다시 확인한 뒤 merge한다.
6. merge SHA가 `upstream/devel`에 포함된 것을 확인한 뒤에만 원 PR·관련 issue에 결과를 남기고 close,
   devel sync 및 review worktree·전용 target 정리를 수행한다.

최종 merge 조건은 통합 PR 최신 head의 GitHub Actions 성공, mergeable 상태, 작업지시자 승인이다.

## 시각 증적

- #3540/#2771: `samples/aift.hwp`와 저장소의 공식 기준 `pdf/aift-2022.pdf`를 사용했다. 두 입력은
  모두 A4 74쪽이며, issue가 지목한 56·60·62쪽을 release binary로 SVG/render-tree/PDF sweep 했다.
  구조·flow heuristic은 **0/3 flagged**였다.
- overlay 보조 지표는 pixel match 평균 92.895%(최저 90.670%), ink match 평균 72.310%(최저
  57.167%)다. 서로 다른 글꼴·기존 조판 차이를 포함하므로 이 값은 첨자 회귀의 단독 pass/fail이 아니다.
  p60은 pixel 95.477%, ink 81.882%이고, 실제 3-way review panel은
  `pr_3540_kevin9327_aift_script_review_p060.png`(SHA-256
  `0bd5a9dac8d3631146cfae28b1dfd909196af3c098d6b61b9944edfb74958fe1`)에 보존했다.
- #3189 HML fixture는 공식 기준 PDF가 없다. `formatting_table.hml`의 parser/serializer 3개 회귀로
  기계적 계약을 확인하고, 외부 표준과의 최종 시각 수용을 주장하지 않는다.

## 로컬 검증

모든 Cargo 명령은 `CARGO_INCREMENTAL=0`과
`CARGO_TARGET_DIR=target/review-kevin9327-20260729`로 실행했다. 기존 공유 target은 수정하지 않았다.

- focused CLI/HWP3/HML 회귀 6 target: **43 passed / 0 failed**. clippy 보정 뒤 HWP3 target도
  **3 passed / 0 failed**로 재실행했다.
- `cargo build --release`: 통과.
- `cargo test --release --lib`: **3019 passed / 0 failed / 7 ignored**.
- `cargo test --profile release-test --tests`: 전체 integration target이 끝까지 실행됐고 failure 없이
  exit 0으로 완료했다. IR field sweep fixture도 이 gate에 포함된다.
- `cargo test --profile release-test --features native-skia --lib`:
  **3076 passed / 0 failed / 7 ignored**.
- `wasm-pack build --target web`: 검토 전용 `wasm-pkg` 출력으로 통과.
- `cargo fmt --all -- --check`, `git diff --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test --doc` (**4 passed / 2 ignored**) 모두 통과.

## CI 재검증 보정

최초 #3550 CI의 default-feature shard 8/8은 #2724 source guard에서
`set_field_value_by_name()`을 미분류 뮤테이터로 보고 실패했다. 이 public 메서드는 occurrence 0을
고르는 호환 래퍼이며, 실제 `set_field_value_by_name_at()`가 section `raw_stream`을 무효화한다.
무효화를 중복 추가하지 않고, guard의 검증되는 `DelegatesTo("set_field_value_by_name_at")` ledger에
근거와 함께 등록한 `d0b42ae18`로 보정했다. targeted guard 5/5, field occurrence 4/4, CLI JSON
22/22, fmt·clippy를 재통과한 뒤 CI를 다시 실행한다.
