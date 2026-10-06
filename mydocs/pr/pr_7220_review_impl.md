---
kind: snapshot
status: active
canonical: mydocs/pr/pr_7220_review_impl.md
last_verified: 2026-09-17
---

# PR #7220 통합 검토 처리 계획

[개별 review](archives/pr_7220_review.md)의 최종 판정은 **승인**다.
통합 base `fcbd00e0f`, branch `codex/planet-review-20260917`.

| source SHA | cherry-pick SHA | 제목 |
| --- | --- | --- |
| `06ceed2db48d56b53d63f1b64a7796a87cc1b9eb` | `b472bf515a5339068baac41e6d73c920646e74c3` | 수정: 저장 textpos 가 이미 HWP5 축인 HWPX 구역 첫 문단을 올려 읽지 않는다 (#7190) |
| `82225af0edf8204d3fa78aeccee7e648f816682e` | `9d94231187d11d1ccd5b495d00bf8155eb11a26b` | 정리: 슬롯 경계 판정의 나머지 연산을 is_multiple_of 로 바꾼다 (#7190) |

1. 완료: upstream/devel 동기화, reviewer 지정, exact source fetch, 중복 stack 제거 후 로컬 누적 체리픽.
2. 완료: 코드 경로·원문/한컴 PDF·실제 실행 검토. [공통 실행 기록](archives/pr_7210_review.md#통합-검토-공통-실행-기록)에서 전체 12개 SHA와 검증 범위를 확인한다.
3. 다음 조건: 최종 통합 CI 확인 및 #7190의 남은 진단 범위를 본문에 구분한 뒤 merge한다.
4. 결과보고 후 검토 문서·대표 PNG·오늘할일을 로컬 commit한다. code/source cherry-pick의 작성자·provenance를 보존한다.
5. 원격 통합 PR 생성 지시 후 upstream의 임시 codex/ head로 push하고 devel 대상 PR을 만든다. owner를 자동 reviewer로 지정하지 않는다.
6. 최종 head의 required CI·mergeability·review gate를 확인한다. 승인된 merge 이후에만 원 PR 적용 결과 댓글과 source PR 처리, 실제 완료된 이슈의 종료를 수행한다.
7. 후속 처리: duration refresh만 실행됐는지 확인하고 devel을 동기화한다. 실행 중 Cargo/Rust가 없을 때 이 작업 전용 target/branch만 정리한다. contributor fork branch는 보존한다.

## 보정·rollback 범위

이번 회차에서 contributor code 위의 maintainer code 변경은 하지 않았다. #7221→#7228→#7215는 의존 묶음이며 뒤 변경을 남긴 채 앞 commit만 빼지 않는다.
제외를 지시받으면 깨끗한 최신 base에 선택한 source만 재적용해 검증하고, 현재 검토 branch와 증거를 보존한다. 다른 작업/branch를 reset하지 않는다.
결정이 필요한 다음 단계는 보류 사유의 보정 범위와 최종 통합 PR 게시다. 이번 로컬 검토를 원격 merge 승인으로 해석하지 않는다.
