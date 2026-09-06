# PR #6810 self-review — 공개 기여 검증 안내

## 접수와 경로

작성자 `postmelee`의 collaborator self-review다. reviewer 지정·GitHub approve event는 만들지 않았다.
`collaborator_self_merge.md`를 기본으로 intake, local_validation, review_only_fast_pass와 대형 PR 예외를
적용했다. 공개 안내·PR 템플릿이 포함되어 전체 PR의 review-only fast-pass는 적용하지 않는다.

| 항목 | 2026-09-06 작성 시점 참고값 |
| --- | --- |
| PR | [#6810](https://github.com/edwardkim/rhwp/pull/6810), OPEN, Draft 아님 |
| 작성자·담당자 | postmelee |
| base | devel, `016fe3ceed904633e74e70127a4cceaa1f18a756` |
| 최초 제출 head | `15f5a5574f1e9f989fdead277ccf2345e604e331` |
| branch·milestone | upstream `task_m100_6791`, v1.0.0 |
| 최초 제출 규모 | 8 files, +936/-106, 10 commits; 후속 review·계획·오늘할일 기록 전 수치 |
| mergeability·CI | MERGEABLE / BLOCKED, CI 시작·대기 중; 최신 head에서 재확인 필요 |

추가·삭제 합계 1,042줄로 대형 PR 경로를 적용했다. 대부분 단계별 승인·결과 기록이며 공개 변경은 두
파일이다. 이미 병합된 코드를 신규 diff로 포함하지 않았다. 최신 devel 병합은 충돌이 없었고 검증한 두
공개 파일 내용은 동일했다. 대형 PR 판정·사용자 merge 승인은 별도 cycle로 유지한다.

## 범위와 검토 결과

관련 이슈는 [#6791](https://github.com/edwardkim/rhwp/issues/6791)이다. 문제 출처 baba9811의
[PR #6786](https://github.com/edwardkim/rhwp/pull/6786)은 별도 작업이며 이 PR에서 변경·리뷰하지 않았다.

- CONTRIBUTING의 범위 표는 Rust source·helper·renderer·Studio 단독·혼합·package·문서를 구분한다.
  Studio 단독도 fresh WASM을 준비하며 Rust 입력을 함께 바꾸면 Rust 검증을 적용하도록 했다.
- source commit → 별도 worktree → prepare·fmt·세 Clippy·해당 회귀 → manifest → 동일 SHA 제출을
  연결했다. 누락 suite와 포맷 위반을 구별하고 보정 후에는 원본 새 commit을 다시 검증한다.
- PR 템플릿·빠른 시작·스타일·회귀 예제도 같은 절차를 가리킨다. 생성 harness·manifest 미제출,
  전체 integration·Native Skia 3종 등 범위별 기존 검증 요구와 실행 실패 중단 원칙을 확인했다.
- CI·Cargo·생성기·제품 코드·fixture 변경은 없다. report 이름·Issue 번호·계획 및 단계 기록이 일치한다.

## 완료한 검증과 한계

문서 검증 SHA는 `b5a33bfe3b85b9b0f4ebe119a1d1addeb8c1e43b`다. 새 clean source에서 공개 명령을 그대로
실행해 별도 review worktree를 만들고 prepare → fmt check → manifest check 및 SHA·clean 검사를 통과했다.
28 suites + 20 exceptions = 48/48 targets, tracked Rust·Cargo 2,206개 hash 불변을 확인했다.
생성 harness 28개·manifest는 review worktree의 ignored 파일이며 PR diff에 없다.
[Stage 3 증적](../../working/task_m100_6791_stage3.md)과 [최종 결과](../../report/task_m100_6791_report.md)에
경로·명령·결과를 기록했다. 변경 문서 링크·공백, anchor 21개, Rust bash 블록 10개를 확인했다.

문서 작업의 승인된 검증 범위에 따라 전체 Rust build·Clippy·nextest·Native Skia·Studio build는 반복하지
않았다. 렌더링·레이아웃·sample·fixture를 바꾸거나 시각 개선을 주장하지 않아 visual sweep은 해당 없다.
로컬 CLI 절차 실행은 완료했으나 모든 안내 명령을 실행한 것은 아니다.

classifier v7은 PR 템플릿을 workflow-contract로 판단해 전체 CI를 요구한다. 최초 PR Actions는 시작됐고
완료 여부를 아직 판정하지 않았다. review-only 후속 기록도 최신 head의 실제 checks를 별도로 확인한다.

## 문제·위험과 후속 조건

최초 self-review에서는 결함을 발견하지 못했으나 9월 7일 리뷰로 링크·범위 검사·셸 가드 결함을 확인했다.
아래 재검토 기록이 최초 판정을 갱신한다. 미래 suite·CI 명령 변경에 따른 문서 불일치 가능성은
남으며 이번 PR에서 생성기·정책을 바꾸지 않는다. 무관한 코드나 성능 개선 주장은 없다.
GitHub checks 미완료를 로컬 검증 성공으로 대체하지 않으며, merge와 #6791 close는 아직 승인되지 않았다.
실행 순서·승인 및 정리 경계는 [review implementation](pr_6810_review_impl.md)에 기록했다.

## 2026-09-07 재검토

[리뷰 11건](https://github.com/edwardkim/rhwp/pull/6810#issuecomment-5561356545)을 재현·검토한 뒤
사용자 보정·충돌 해소·댓글 게시 승인으로 공개 문서를 `25d1d1011`에서 수정했다.
최신 devel `07bc5e549`와의 충돌은 오늘할일 한 파일에서 양쪽 기록을 보존해 해소했다.
[Stage 4](../../working/task_m100_6791_stage4.md)에 항목별 판단과 실제 실행 결과를 기록했다.

공개 준비·fmt·manifest·정리, 실패 가드 48건, WASM 없는 Studio unit 및 fresh dev WASM 뒤 package 검사가
모두 통과했다. unit은 1,489 passed / 1 skipped이며 production bundle도 성공했다. tracked Rust·Cargo
2,215개 hash는 그대로였다. GitHub 렌더링 절대 링크 8개·anchor 26개와 문서 링크를 확인했다.
세 Clippy·전체 nextest·Native Skia·Windows native·Docker·브라우저 E2E는 이번 문서 보정에서는 미실행이다.
공개 문서 변경이므로 review-only tail만으로 취급하지 않고 새 head CI를 확인한다.

## 최종 판정

**승인** — 보정 문서 SHA `25d1d1011`의 공개 절차·범위 정합성과 새 실행 증적을 확인했다.
이는 문서 self-review 판정이다. 최종 merge 조건은 기록을 포함한 최신 head의 GitHub Actions 통과,
최신 base와의 정합성·mergeability 재확인, 대형 PR에 대한 작업지시자의 별도 merge 승인이다.
