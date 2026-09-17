---
kind: snapshot
status: active
canonical: mydocs/pr/pr_7221_review_impl.md
last_verified: 2026-09-17
---

# PR #7221 통합 검토 처리 계획

[개별 review](archives/pr_7221_review.md)의 최종 판정은 **머지 보류**다.
통합 base `fcbd00e0f`, branch `codex/planet-review-20260917`.

| source SHA | cherry-pick SHA | 제목 |
| --- | --- | --- |
| `3ac1356dc46fab66994f55bba598f599da9c5e2e` | `502dfe7d9e8b951e53e01d7322741a100e2e06ec` | 수정: 쪼개진 자리차지 표 조각을 문단 상자 상단에 건다 (#7203) |

1. 완료: upstream/devel 동기화, reviewer 지정, exact source fetch, 중복 stack 제거 후 로컬 누적 체리픽.
2. 완료: 코드 경로·원문/한컴 PDF·실제 실행 검토. [공통 실행 기록](archives/pr_7210_review.md#통합-검토-공통-실행-기록)에서 전체 12개 SHA와 검증 범위를 확인한다.
3. 다음 조건: 같은 조각의 실제 top/점유 끝점을 공통 결과로 만들고 측정·예약·paint에서 소비한다. 원래 높이만 fit하는 경계 및 다음 표의 위치를 독립 PDF/정식 테스트로 확인한다.
4. 결과보고 후 검토 문서·대표 PNG·오늘할일을 로컬 commit한다. code/source cherry-pick의 작성자·provenance를 보존한다.
5. 원격 통합 PR 생성 지시 후 upstream의 임시 codex/ head로 push하고 devel 대상 PR을 만든다. owner를 자동 reviewer로 지정하지 않는다.
6. 최종 head의 required CI·mergeability·review gate를 확인한다. 승인된 merge 이후에만 원 PR 적용 결과 댓글과 source PR 처리, 실제 완료된 이슈의 종료를 수행한다.
7. 후속 처리: duration refresh만 실행됐는지 확인하고 devel을 동기화한다. 실행 중 Cargo/Rust가 없을 때 이 작업 전용 target/branch만 정리한다. contributor fork branch는 보존한다.

## 보정·rollback 범위

이번 회차에서 contributor code 위의 maintainer code 변경은 하지 않았다. #7221→#7228→#7215는 의존 묶음이며 뒤 변경을 남긴 채 앞 commit만 빼지 않는다.
제외를 지시받으면 깨끗한 최신 base에 선택한 source만 재적용해 검증하고, 현재 검토 branch와 증거를 보존한다. 다른 작업/branch를 reset하지 않는다.
결정이 필요한 다음 단계는 보류 사유의 보정 범위와 최종 통합 PR 게시다. 이번 로컬 검토를 원격 merge 승인으로 해석하지 않는다.
