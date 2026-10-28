---
kind: snapshot
status: active
canonical: mydocs/pr/pr_7214_review_impl.md
last_verified: 2026-09-17
---

# PR #7214 통합 검토 처리 계획

[개별 review](archives/pr_7214_review.md)의 최종 판정은 **승인(보정된 변경 범위)**이다.
통합 base `fcbd00e0f`, branch `codex/planet-review-20260917`.

| source SHA | cherry-pick SHA | 제목 |
| --- | --- | --- |
| `fc55c1a138500007cabeffff3d1146621ca45161` | `542f2442036a71b1033deb9f78b94fd001f572d1` | 수정: 중첩 표의 셀 크기를 셀 경로로 조절한다 (#7189) |

1. 완료: upstream/devel 동기화, reviewer 지정, exact source fetch, 중복 stack 제거 후 로컬 누적 체리픽.
2. 완료: 코드 경로·원문/한컴 PDF·실제 실행 검토. [공통 실행 기록](archives/pr_7210_review.md#통합-검토-공통-실행-기록)에서 전체 12개 SHA와 검증 범위를 확인한다.
3. 완료: 중첩 표 제한값 조회와 크기 변경이 같은 CellPath를 사용함을 확인했다. 최종 전체 회귀·Skia·lint 및 Native/fresh WASM 증거를 review에 기록했다.
4. 결과보고 후 검토 문서·대표 PNG·오늘할일을 로컬 commit한다. code/source cherry-pick의 작성자·provenance를 보존한다.
5. 원격 통합 PR 생성 지시 후 upstream의 임시 codex/ head로 push하고 devel 대상 PR을 만든다. owner를 자동 reviewer로 지정하지 않는다.
6. 최종 head의 required CI·mergeability·review gate를 확인한다. 승인된 merge 이후에만 원 PR 적용 결과 댓글과 source PR 처리, 실제 완료된 이슈의 종료를 수행한다.
7. 후속 처리: duration refresh만 실행됐는지 확인하고 devel을 동기화한다. 실행 중 Cargo/Rust가 없을 때 이 작업 전용 target/branch만 정리한다. contributor fork branch는 보존한다.

## 보정·rollback 범위

메인터너 보정 `c1c9e2047`(중첩 셀 경로), `2a9810642`·`95eed7197`(표 조각 예약/paint), `54c24ebdd`(3→4쪽 회귀)를 누적했다. 집중 41개·전체 9,974개 PASS와 최종 fresh WASM/Skia/lint 증거를 review에 반영했다. 후속 `f94dece59`는 테스트 lint 정리이며 해당 2개를 재검증했다. #7221→#7228→#7215는 의존 묶음이며 뒤 변경을 남긴 채 앞 commit만 빼지 않는다.
제외를 지시받으면 깨끗한 최신 base에 선택한 source만 재적용해 검증하고, 현재 검토 branch와 증거를 보존한다. 다른 작업/branch를 reset하지 않는다.
보정 범위는 사용자가 승인했다. 남은 단계는 별도 지시를 받은 뒤 통합 PR 게시와 최종 원격 CI 확인이다. 이번 로컬 검토를 원격 merge 승인으로 해석하지 않는다.
