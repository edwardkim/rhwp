# planet6897 연작 7건 통합 구현 기록 (2026-07-28)

Issue: #3466 #3413 #3403 #3380 #3375 #3492 #3385
원 PR: #3472 #3475 #3479 #3487 #3488 #3493 #3499 (author: planet6897)
통합 branch: `review/planet6897-20260728` (기준 `origin/devel` = `ead4300d2`)

## 1. 범위 확정 경과

작업지시자 지시로 planet6897 열린 PR 전체를 연작 처리한다. reviewer=edwardkim,
assignee=planet6897, milestone=v1.0.0 을 전 건에 지정했다.

- 착수 시점 8건이었으나 접수 중 **#3491·#3497 을 컨트리뷰터가 자진 close** 했다 — 각각
  kevin9327 의 선행 PR #3478(#3476)·#3482(#3480)와 중복임을 스스로 확인하고 양보. #3470/
  #3471 supersede 선례가 컨트리뷰터 간 규범으로 작동한 사례다.
- 그 사이 신규 #3499 가 열려 최종 범위는 **7건**이다. `scripts/pr_triage.sh` 7건과 대조 일치
  (축: doccore 4 / render 1 / model 1 / hwp3 1, CONFLICTING 0).

## 2. 체리픽 누적

PR당 기능 커밋 1개, 오래된 번호순, `-x` provenance, **충돌 0**.

| 순서 | 원 PR | 원 커밋 | 누적 SHA | 축 |
|---|---|---|---|---|
| 1 | #3472 | `009f26bef` | `91fd2e8be` | model — 자동번호 컨트롤 갭 어순 |
| 2 | #3475 | `4badc9b21` | `7f5b89f14` | query — structure 수식 보존 |
| 3 | #3479 | `f3b16fea8` | `ab6e28aa8` | query — 분할 표 셀 실제 쪽 |
| 4 | #3487 | `2851576f7` | `45510c468` | core — 안내문 동일 실값 보존 |
| 5 | #3488 | `278794900` | `9920b5453` | render — 안내문 인쇄 프로필 억제 |
| 6 | #3493 | `2f8121a67` | `a7f0bbeda` | hwp3 — 개요번호 마커 IR 제거 |
| 7 | #3499 | `5c67929c6` | `b74b50983` | text — PUA 텍스트 표면 매핑 |

선행 의존성 없음 — 7건이 서로 다른 파일 축이라 순서는 번호순 규약일 뿐이다. 유일한 인접
지점은 #3475/#3499 가 같은 `queries/rendering.rs` 를 만지는 것인데 서로 다른 함수라 자동
병합됐다.

## 3. 누적 검증 (공유 게이트)

| 항목 | 결과 |
|---|---|
| focused 7 스위트 | **20 passed / 0 failed** |
| red-check (#3472) | `paragraph.rs` 만 devel 로 되돌리면 수정 대상 2건만 FAILED — 실증 |
| `cargo test --profile release-test --tests` | **4253 passed / 0 failed** (기준 4233 + 신규 20 정합) |
| svg_snapshot golden | **무변화** — #3488 screen·#3499 렌더 불변 주장 교차 확인 |
| IR field sweep baseline | 통과 — #3493 파서 변경의 왕복 발산 없음 |
| `cargo fmt --check` / `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 경고 0 |

시각 판정: #3488 만 렌더 변경이며 신규 sweep 생략 근거 4가지를
[pr_3488_review.md](pr_3488_review.md)에 기록했다(확립 정책 동형·screen 무변화·흐름 불변식
테스트·print 의도 변화 계약 고정). 나머지 6건은 렌더 출력 무변경.

## 4. merge 계획 (승인 게이트)

1. `review/planet6897-20260728` 을 upstream 임시 head 로 push — **승인 필요**
2. `devel` 대상 통합 PR 생성 (본문에 7건 표·검증·supersede 목록) — **승인 필요**
3. 통합 PR 최신 head CI 성공 확인 → **merge commit** 으로 병합 — **승인 필요**
4. 원 PR 7건 supersede close + 반영 위치·감사 코멘트, 관련 이슈 7건 close 상태 확인
5. branch·검토 전용 target 정리

각 원 PR 의 review 문서는 이 디렉터리에 PR 번호별로 있다. merge 전 각 PR 의 최신 head
변동을 개별 재확인한다(작성 시점 참고값 원칙).
