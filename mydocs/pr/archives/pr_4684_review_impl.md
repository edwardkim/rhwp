---
kind: pr-review-implementation
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4684 메인터너 보정 구현 기록

## 기준과 적용

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4684](https://github.com/edwardkim/rhwp/pull/4684) |
| 원 source head | `2c185af979f71bdf4ae80352a3db90c7cf6b0616` |
| 기준 devel | `6f70cd1b6` |
| 가시성 branch | `review/planet6897-4684-20260812` |
| 최종 local candidate | `3741441a28f5982bd167421cdbee0386a5596687` |

원 PR은 최신 `devel`과 `src/serializer/hwpx/section.rs` 테스트 블록에서 충돌했다. 가시성
branch에서 원 기능을 적용한 뒤, #4675의 기존 회귀와 #4676의 새 회귀를 모두 보존하도록 충돌을
해소했다. 원 contributor commit은 재작성하지 않았다.

| 순서 | local commit | 역할 |
| --- | --- | --- |
| 1 | `6e1a4e629` | 원 #4684의 curve `hp:seg` writer 변경을 최신 `devel` 위에 적용 |
| 2 | `278eee6ea` | #4675와 #4676 serializer 테스트 블록 충돌 해소 |
| 3 | `3741441a2` | HWPX `hp:seg` type의 HWP5 Bezier 오매핑 제거와 parser 회귀 추가 |

## 보정 이유

HWPX `hp:seg`는 각 구간의 시작점과 끝점만 제공한다. 반면 renderer가 사용하는 HWP5
`CurveShape.segment_types == 1`은 두 제어점과 끝점의 세 점을 요구한다. 원 PR처럼 HWPX의
`type="CURVE"`를 `1`로 넣으면 curve가 세 점씩 소비되어 공개 HWPX의 점 수와 경로가 달라진다.

보정은 `segment_types`를 비워 기존 LineTo 체인을 보존한다. serializer는 빈 HWP5 구간 타입에서
한글 호환 `type="CURVE"` segment를 계속 방출하므로, #4676의 crash 회피 XML 구조는 변하지 않는다.

## 완료한 단계

1. 원 source와 최신 `devel`의 충돌을 가시성 branch에서 해소했다.
2. HWPX CURVE와 LINE segment가 섞인 parser fixture에 빈 `segment_types` 회귀를 추가했다.
3. HWPX CURVE 점 체인의 XML→IR→XML 경계 회귀를 추가해, 재저장 후에도 HWP5 Bezier 타입이
   비어 있고 `hp:seg`만 방출됨을 고정했다.
4. parser 집중 테스트, serializer 집중 테스트, 전체 release-test nextest, fmt, clippy를 순차로
   실행해 통과했다. 마지막 경계 회귀 추가 뒤에는 #4676 집중 테스트와 fmt·diff 검사를 다시 통과했다.
5. 실제 비공개 `156550355` HWPX를 재저장하고 `--verify --json`, ZIP 무결성, curve XML 구조를
   확인했다. 보정 전후 산출물 SHA-256은 동일했다.

## 다음 단계

1. 작업지시자의 remote push 및 PR 생성 승인을 받는다.
2. 원 PR과 충돌하는 후보를 `devel` 대상 통합 PR로 push한다. code/test commit이 있으므로 Full CI와
   CodeQL을 최신 head에서 다시 확인한다.
3. 승인 뒤 통합 PR을 merge하고 #4684에 통합 반영과 메인터너 보정 이유를 코멘트한 뒤 close한다.
4. #4676은 한글 2022 오라클 재개방 범위가 실제 완료 조건을 만족하는지 확인한 뒤에만 close한다.
