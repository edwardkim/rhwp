---
kind: review-implementation
status: completed-local
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4265 통합·메인터너 보정 기록

## 통합 기준과 적용 순서

최신 `upstream/devel` `e919655a78d5928cdf7236152fce04d6aa6f6377` 위의
`review/humdrum00001010-20260808`에서 원 contributor 변경을 다음 순서로 누적했다.

| 순서 | 원 PR | 통합 code commit | 역할 |
| --- | --- | --- | --- |
| 1 | #4246 | `751796949` | 폰트 메트릭 색인 |
| 2 | #4247 | `e0a21d3c4` | 단일줄 과밀 memo |
| 3 | #4248 | `9dce61bb9` | 셀 캐럿 rect fast path |
| 4 | #4249 | `d30d60004` | units 지문 기반 eviction 생략 |
| 5 | #4250 | `dd372b805` | 셀 블록 서식 동기화 |
| 6 | #4251 | `abb5a1485` | IME 조합 재정박 |
| 7 | #4262 | `4f0fd7df2` | ResizeObserver 경고 격리 |
| 8 | #4258 | `bf803bf53` | split 셀 stale line_segs 재래핑 |
| 9 | #4259 | `7519a7061` | 표 호스트 문단 캐럿 질의 좁히기 |
| 10 | #4260 | `ebca79fb1` | 저장 시점 캐럿 메타데이터 |
| 11 | #4261 | `277b40222`, `3fadc9b42` | 거대 셀 Enter pagination 지연 제거 |

원 source의 stage 보고·계획 commit도 각 code commit 직후에 보존했다. 원 contributor branch는 rewrite하지
않았고, #4248/#4249의 단독 충돌은 서로 의존하는 stack을 하나의 통합 branch에서 유지해 해소했다.

## 메인터너 보정

`da2d9ae75`는 #4248의 부분 셀 compose가 뒤 문단 중첩 표의 border clip 확장을 누락하는 문제를 고친다.
KTX fixture에서 `cellBounds`가 legacy보다 좁아지는 것을 재현한 뒤, 대상 셀에 뒤 문단의 직접 표 control이
있을 때만 compose 종료 지점을 셀 끝으로 확장했다. 분할 셀의 기존 문단 창은 그대로 유지해 giant-cell
fast path가 전량 compose로 퇴행하지 않게 했다.

## 롤백 경계

- #4246~#4249는 동일한 셀 캐럿 성능 stack이므로 부분 롤백하지 않는다.
- #4258~#4261은 split 셀·캐럿·저장·Enter pagination 순서를 공유하므로 역순으로만 되돌린다.
- #4250, #4251, #4262는 Studio 입력 및 browser error surface 축이며 Rust stack과 독립적으로 롤백할 수 있다.
- `da2d9ae75`는 #4248 fast path와 함께 유지한다. 이를 단독으로 되돌리면 KTX 중첩 표 `cellBounds` parity가
  다시 깨진다.

## 원격 처리 전 조건

1. code head `da2d9ae75`와 첫 trailing review head `557bb8526`의 Full CI, CodeQL, Render Diff가
   모두 성공했음을 확인했다. Native Skia, slow shard, regular shard 1/3·2/3·3/3도 포함한다.
2. 이 CI 결과 기록을 반영한 마지막 문서 commit을 push한 뒤 최신 head의 docs-only gate를 다시 확인한다.
3. 작업지시자 승인 뒤에만 #4265를 merge하고, 원 PR·연결 이슈를 통합 반영 사실과 함께 후속 처리한다.
