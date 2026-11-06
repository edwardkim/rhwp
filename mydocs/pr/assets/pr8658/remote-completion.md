# PR #8658 수용·원격 검증 완료 기록

- [수용 PR #8694](https://github.com/edwardkim/rhwp/pull/8694), 검증 head `d440ddc370026ba89ac79a6f6bc2385b24195973`, base `08963f6825956984d38b0d9392f44777db7107ab`.
- merge [`67adf83672eedd0672a4a1b0cb81db7ef7e85574`](https://github.com/edwardkim/rhwp/commit/67adf83672eedd0672a4a1b0cb81db7ef7e85574), 2026-11-06 20:42:13 KST. 원 기여 SHA `218c8856ad0a338fd1820c43e62b40dbcdcb86bb`를 그대로 보존했고 [원 PR #8658](https://github.com/edwardkim/rhwp/pull/8658)의 간접 MERGED를 확인했다.
- [merge·이슈 상태 영수증](merge-and-issue-state.json), [직전 exact-head·최신 base·merge-tree](premerge-exact-head-check.json), [전체 check·status](github-final-validation.json)를 보존했다. fork 권한 제한으로 upstream 수용 PR을 사용했으며 원 fork branch에는 push·삭제하지 않았다.

## 정확한 head CI와 self-review

| 검증 | 실제 성공 실행 |
| --- | --- |
| CI·Native·Lint·Frontend·Archive A–E | [42207470188](https://github.com/edwardkim/rhwp/actions/runs/42207470188) |
| CodeQL 4개 언어 | [42207470398](https://github.com/edwardkim/rhwp/actions/runs/42207470398) |
| 별도 CodeQL 보안 check | [127859213805](https://github.com/edwardkim/rhwp/runs/127859213805) — 승인된 분류 뒤 success |
| Render Diff | [42207470067](https://github.com/edwardkim/rhwp/actions/runs/42207470067) |
| Adapter / Proptest | [42207470286](https://github.com/edwardkim/rhwp/actions/runs/42207470286) / [42207470307](https://github.com/edwardkim/rhwp/actions/runs/42207470307) |
| 최종 CI Impact Policy | [42211252503](https://github.com/edwardkim/rhwp/actions/runs/42211252503) |

[GitHub Archive 실제 로그 집계](ci-archive-actual-results.json)는 A/B/C/D/E 각각 3,801/2,538/2,073/2,071/2,083 PASS, 합계 **12,566 PASS·기존 45 skipped**다. [Native 실제 성공 요약](ci-native-actual-results.json)도 확인했다. 로컬 **12,760 PASS**, Native 공식 **4,064 PASS**, 집중 **8 PASS**는 [로컬 최종 검증](final/validation.md)과 구분한다. 제품 workflow 취소·재실행은 없었다.

[작성자 self-review](https://github.com/edwardkim/rhwp/pull/8694#pullrequestreview-5799585117)에서 공통 그림 점유→측정→선언 수용→최종 배치, 분할 유닛 소유·예약·종료, 원본 저장 계약·관계형 회귀와 실제 Studio 소비를 검토했다. [기록](self-review-final.json)에 근거를 남겼으며 새 구현 blocker는 없다. 별도 속도 벤치마크를 수행했다고 주장하지 않는다.

## CodeQL 보안 check의 별도 확인

workflow success를 보안 check success와 혼동한 초기 보고를 정정했다. 새 #500 때문에 별도 check는 failure였으며, 상태를 확인하기 전에는 merge하지 않았다. #500의 sink·guard는 최신 base와 동일한 `RHWP_6313_DBG` 숫자 좌표 진단이다. [source/sink 도입·현재 SARIF 4개 flow와 기존 3건 검토](security-bot-review.json), [전체 flow location·도구 정보](alert500-normalized-flows.json)를 보존했다.

작업지시자의 명시 승인으로 [#500만 false positive 분류](alert500-classification-result.json)했다. [같은 보안 check가 자동 failure→success로 갱신](codeql-security-check-resolution.json)돼 조건부로 승인된 Rust 분석 재실행은 사용하지 않았다. 기존 #478·#483·#484는 OPEN 유지, 탐지 query·severity·workflow는 바꾸지 않았다.

## 입력·시각 증거와 공개 확인

원 사례·경계 8입력 10쪽 Native/fresh optimized WASM 각각 raw 100.00%, 전체 36입력 77쪽은 최저 91.16%/91.83%·90% 미달/누락 0이다. 실제 base 대조 67쪽씩은 각각 PNG 바이트가 같다. [최종 시각 대장](final/validation.md)에 독립 Desktop Print·전쪽 TSV·확대 판독·실제 Studio를 연결했다.

[head 고정 공개 파일 252개](raw-publication-proof.json)는 실제 다운로드 바이트·SHA-256이 같았고, [PR 화면 이미지 23개](github-pr-body-display.json)도 모두 정상 decode됐다. 감사 댓글의 원 주장 3입력·전 4쪽 Native/WASM review·overlay 16개는 [실제 merge SHA의 이미지 형식·해시](merge-image-publication-proof.json)를 확인했다.

## 병합 후 duration 메타데이터

[Refresh nextest target duration data](https://github.com/edwardkim/rhwp/actions/runs/42212204387)는 success다. 이 head의 B–E 측정 artifact 4개를 수집했고 `ci-metrics/nextest-target-durations`에 commit `15e11fd94db4a42895612473caca3189d5d1d115`를 게시했다. [실제 수집·게시 결과](duration-refresh-result.json)를 확인했다. 병합 후 제품 CI·CodeQL·Adapter·Proptest·Oracle 검증을 시작하거나 재실행하지 않았다.

## 후속 기록과 남은 실행 순서

후속 기록은 **maintainer 직접 반영 옵션 M**이다. archive review/계획·이 assets·오늘할일만 반영하고 source·test·workflow·golden·기존 sample은 바꾸지 않는다. 운영 기록 push·최종 devel fast-forward 뒤 이슈 근거·원 PR 감사 댓글·종료 정리를 순서대로 수행한다.

이 기록 시점 [#8576](https://github.com/edwardkim/rhwp/issues/8576)은 devel push 자동화로 CLOSED(2026-11-06T11:42:40Z)이며, 수동 close를 반복하지 않는다. 해당 merge·검증 근거의 메인테이너 댓글은 아직 게시 전이다. [#8579](https://github.com/edwardkim/rhwp/issues/8579)는 OPEN을 유지한다.

이후 contributor 댓글은 [archive의 페이지별 계획](../../archives/pr_8658_review.md#merge-후-contributor-pr-comment-계획)에 따라 첫 게시부터 4쪽의 대응 이미지를 넣고, API 본문과 실제 화면을 검증한다. 작업 전용 local/remote branch·worktree를 제거하고 기본 작업공간 `output/` 전체·사용 종료된 `target/pr-review/debug`를 정리한다. 원 fork branch와 다른 작업 공간은 보존한다. 종료 정리 로그는 `mydocs/privacy/pr8658-cleanup-20261106.json`에 남긴다.
