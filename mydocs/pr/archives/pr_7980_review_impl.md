# PR #7980 — self-review 후행 기록과 병합 순서

## 현재 단계

2026-10-17 작업지시자가 self-review 진행을 승인했다. 정확한 code head
`dcd70ea411a70fe7b45ddc89d8156dac51f8798f`의 Full CI·CodeQL·Render Diff·Adapter·Proptest·Controller는
모두 성공했다. [self-review](pr_7980_review.md)의 최종 판정은 승인이다.
검증 내용과 source/producer의 연결은 해당 문서와 [stage2](../../working/task_m100_7974_stage2.md)를
정본으로 사용하며 이 문서는 원격 반영 이후 실행 순서만 기록한다.

## 후행 문서 반영

1. 검증 code head 위에 self-review·이 실행 순서·오늘할일·stage2의 실제 완료 사실만 single-parent
   trailing commit으로 추가한다. 생성 `rhwp-studio/public/rhwp.js`는 로컬 Studio용 WIP로 보존한다.
2. 최신 `upstream/devel`은 `42dade44e3a603015cecb7e9731928e3c3c8bd05`다. 기본 활성 wasm-api의
   선택 소스 동등성과 #7981의 독립 기본 package/12쪽 동등성 근거를 연결한다. source를 반복 merge/rebase하지 않는다.
3. 오늘할일에는 source의 기존 항목을 유지하고 양쪽에서 바뀌지 않은 section 경계에 #7980만 추가한다.
   최신 base의 #7981 항목을 source에 복사하지 않는다.
4. 로컬 trailing commit과 최신 base의 `git merge-tree --write-tree`가 exit 0인지 확인하고,
   실제 merge tree의 공백·변경 Markdown 링크·최신 base 기록과 #7980의 보존·중복 없음을 검사한다.
   정확한 base/head/tree와 실행 결과는 ignored `output/pr-review/backlog-7949/validation-next/`에 기록한다.
5. push 직전에 원격 base와 원 PR head가 바뀌지 않았는지 다시 조회한다. 문서 외 변경이 없는지 확인한 뒤
   같은 upstream `task_m100_7974`에 push한다. PR 본문에는 review 링크와 실제 code-head CI 성공을 추가하되
   시각 이미지·수치의 검증 source를 바꾸지 않는다.
6. 새 trailing head의 required check·실제 review-only fast-pass 판정·mergeability를 조회한다.
   과거 code head 성공을 후행 CI 완료로 쓰지 않는다. 실패 시 원인과 영향 범위를 확인한다.

## 병합과 후속 처리 — 별도 승인 뒤 실행

- 작업지시자의 #7980 병합·후속 처리 승인을 받고 최신 head·required check·base merge tree를 재확인한다.
- 승인된 정상 merge 경로를 사용한다. `--admin` bypass를 사용하지 않는다.
- 실제 merge SHA와 devel 반영을 확인하고 duration 기록 절차의 실제 결과를 읽는다. 측정 없음은 그대로 기록한다.
- #7974의 해결 범위와 종료 상태를 확인하고 실제 결과·merge SHA 고정 대표 시각 증거를 댓글로 남긴다.
  부모 백로그와 나머지 미지원 범위 전체를 종료한 것으로 쓰지 않는다.
- 로컬·원격 branch와 worktree 정리는 [후속 처리](../../manual/pr_review/post_merge.md)의 소유·최종 SHA·
  병합·CI·활성 사용 조건을 충족한 이번 PR 전용 대상만 적용한다. 먼저 필요한 증거를 안정 경로에 보존한다.
  Studio 7795와 생성 glue는 아직 활성 사용 중이므로 자동으로 삭제하지 않는다. 기본 작업공간·7700·
  사용자 WIP·공유 `target/pr-review`와 다른 작업은 보호한다.

후행 head의 CI·병합·댓글·이슈 종료·정리는 위 순서의 미래 작업이며 이 문서에서 완료로 표시하지 않는다.
코드 보정이 발견되면 문서 commit에 섞지 않고 별도 code/test head와 필요한 검증으로 돌아간다.
