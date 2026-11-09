# chrisryugj HWP3 PR 2건 원격 처리안

2026-11-09 KST 메인테이너가 “기존 차이 한정 예외·2건 원격 처리 승인”으로 아래 범위를 승인했다. 이는 실행 계획이며 실제 CI·merge·후속 처리 결과는 별도 기록한다. 코드/필수 로컬 검증 원장은 [README](README.md), 표시 차이는 [분리 대조](so-sueop-visual-hold.md)를 따른다.

## 승인된 수용 범위

- #8791의 새 셀 누락은 메인테이너가 수정·검증했다. 원 기여의 네 표 개선을 유지한다. 이 데이터 손실을 예외로 면제하지 않는다.
- #8789의 문자 8종은 실제 본문·각주와 독립 변환본이 일치한다. SO-SUEOP의 기존 본문 굵기·흐름에 의한 90% 미달과, **기존 독립 한컴 HWPX를 base renderer로 열어도 보이는 Native 긴 줄표 fallback 간격**에 한정한 수용 예외를 승인받았다. 새 renderer 코드가 도입된 오류로 기여자에게 재작업을 요구하지 않는다.
- 이 한정 예외를 알 수 없는 새 회귀·누락·쪽수 불일치·필수 검사 실패나 임의의 다른 문서로 확대하지 않는다.

## 승인 뒤 실행 순서

1. 예외 결정과 최종 판정을 문서에 반영한다. Rust 제품 코드·테스트 동작은 검증 완료 후보 그대로다. 최신 devel·원 두 head·본인 미처리 PR·충돌 simulation을 다시 대조한다. devel이 바뀌면 기존 작업 개선과 새 base 기능 보존을 먼저 검증한다.
2. `review/integration-20261109-chrisryugj-hwp3`를 upstream에 push하고 devel 대상 Open 통합 PR을 생성한다. 원 head `3c2c5fdf1bea13912b2eefaf4008ea974a2b4fd9`·`383f66acca8c966da9a198fd046c3da0ed94c3b6`와 원 기여 commit을 보존한다. 원 fork에는 push·force-push하지 않는다.
3. 새 head의 필수 GitHub CI·CodeQL·Render Diff·정책 감사 결과를 확인하고 self-review를 기록한다. CI가 실패하면 실제 원인을 조사하며 검사 기준을 낮추지 않는다. 수동 workflow 취소·재실행이나 CI 정의 수정은 이번 일반 실행 범위로 임의 실행하지 않는다.
4. 정확한 head·fresh-base merge simulation·최신 required checks·mergeable을 재확인한 뒤 merge commit으로 통합한다. squash로 원 기여 이력을 잃지 않는다.
5. maintainer 일반 후속 절차의 option M으로 실제 merge/CI/source·승인한 예외·오늘할일과 archive review 기록을 mydocs 운영 기록 commit으로 devel에 push한다. 이번 후보는 CI 정의를 바꾸지 않으므로 CI 변경 PR의 옵션 2 강제 대상은 아니다.
6. 최종 devel을 fast-forward 동기화한다. 원 PR 2건의 MERGED/간접 수용 상태를 확인하고, 자동 MERGED가 되지 않은 원 PR은 승인된 통합 수용 사실을 알린 뒤 완료 처리한다. closing issue는 없으며 다른 이미 해결된 이슈를 새로 닫거나 재분류하지 않는다.
7. 개별 PR의 [댓글 계획](merge-comment-drafts.md)을 실제 source·merge·운영 기록 SHA로 채운다. PNG가 devel에 존재함을 확인한 뒤 감사 댓글 2건을 게시하고 API 및 실제 렌더링으로 본문·한글·이미지·중복 여부를 확인한다.
8. root output 전체와 사용이 끝난 기본 작업공간의 target/pr-review/debug, 이 검토의 임시 ref·branch·base/검토 worktree를 정리한다. 사용자 source/WIP·기존 다른 worktree·local Docker 소비 환경·설치 글꼴·Studio 배포 JS/WASM·다른 target profile과 원 fork branch는 보존한다. 승인되지 않은 보안 분류 변경·배포·태그 조작은 실행하지 않는다.

## source와 승인 상태

- 검증 기준: devel `b768bee1c4c0023087687f79a0a95f12108743e9`.
- runtime 보정: `4fabb47ae28b99d371e2aac3560e2610e327c311`. 이후 테스트/증거 보강은 별도 기록한다.
- 최종 테스트 source: `581d22855f60af2de5fed3208949f7dd81258a0a`. 전체 회귀 13,013 PASS, Native 공식 4,077 PASS, 집중 8 PASS와 모든 적용 lint/build/suite 정책·편람 383쪽 gate가 통과했다. 최종 제출 head는 검토·증거 문서 commit 뒤 승인 요청에서 확정한다.
- 원격 승인: 2026-11-09 KST. 위 수용 예외·실행 범위를 승인받았다. 수동 CI 취소·재실행·CI 정의 수정·보안 분류 변경은 이 승인에 포함되지 않는다.
