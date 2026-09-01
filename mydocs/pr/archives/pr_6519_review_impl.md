---
kind: pr-review-implementation
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6519
issue: 4969
---

# PR #6519 구현 검토 - #4969 W10-Q5/Q6 종료 인계

## 제출 계보

1. Q5-A `a8371d976`에서 PR #6493 이후 Q2/Q3/Q4 증적의 현재 merge-tree 재사용 자격을 감사했다.
2. Q5-B `0367ac865`에서 세 공개 face의 native·WASM canonical parity와 backend disposition을 고정했다.
3. Q5-C `7d9909791`에서 bounded·malformed·atomic rollback guard를 기존 integration source에 추가했다.
4. Q5-D `4d107fe09`에서 1/2/8 resource multiplicity와 성능·WASM ledger를 결산했다.
5. Q5-E `cf1536a9c`에서 Q2·Q4 `bounded-subset`, Q3 `qualified`, 전체 `bounded-subset`으로 판정했다.
6. Q6-A~C `8258f0284`~`27f037fd5`에서 최종 support matrix, tracker 게시 전 guard와 제출 검증을 고정했다.
7. 코드 후보 `27f037fd5`의 GitHub Full CI는 24 success, 5 expected skip, 실패·대기 0으로 완료됐다.

## 보호 불변식

- exact source·capability·지원 tuple이 모두 확인되기 전에는 GlyphRun/outline을 게시하지 않는다.
- reject 뒤 geometry, sidecar, measurement, cache와 font resource를 부분 변경하지 않는다.
- font 32 MiB, glyph·cluster·text 4,096, axis 16, feature 64와 registry 상한을 완화하지 않는다.
- backend가 증명하지 못한 payload는 기존 TextRun 또는 검증된 portable outline으로 닫는다.
- deferred surface, private corpus·설치 font·Hyper-V·한컴 결과를 현재 제품 지원으로 계산하지 않는다.
- tracker에는 실제 PR·merge SHA·최종 CI만 게시하며 placeholder가 남으면 중단한다.

## trailing review-only 처리

1. 이 review, 구현 검토와 오늘할일만 코드 후보 뒤 single-parent commit으로 추가한다.
2. `cargo fmt --all`, `cargo fmt --all -- --check`, 문서 링크·diff 검사를 통과한 뒤 같은 source branch에
   push한다.
3. preflight가 코드 후보 `27f037fd5`의 Full CI를 재사용하고 latest head의 required aggregate가 성공하는지
   확인한다. base 전진만을 이유로 update branch·merge·rebase하지 않는다.
4. latest head SHA, `MERGEABLE/CLEAN`, 실패·대기 0을 확인한 뒤 별도 merge 승인을 요청한다.

## merge와 tracker 후속

1. 승인 뒤 squash가 아닌 정상 merge commit 방식으로 PR #6519를 병합한다.
2. merge commit ancestry와 PR tree, #4969·#4960의 최신 body hash·상태를 다시 확인한다.
3. 별도 승인 뒤 로컬 tracker 초안의 placeholder를 실제 PR·merge·CI 값으로 치환하고 body drift가 없을 때만
   #4969 comment, #4960 최소 body patch와 comment를 게시한다.
4. #4969와 #4960은 실제 merge·tracker 감사 뒤에만 close한다.
5. post-merge 검증과 원격 `devel` 동기화를 끝낸 뒤 이 PR 전용 branch/worktree 정리를 수행한다.
