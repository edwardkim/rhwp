# PR #4774 구현 검토 - 앵커 그림 스택 maintainer 보정

## 목적

contributor 원 변경이 #4770의 저장 LineSeg 기반 그림 스택을 앵커 쪽에 유지하는 동안, 기존 #1995
낱장 분할 억제를 비대칭 조건으로 넓게 비활성화하지 않도록 보정한다. contributor 원 commit은 재작성하지
않고, 해당 source head 위에 collaborator commit을 추가한다.

## 커밋 계보

| 순서 | SHA | 구분 | 내용 |
| --- | --- | --- | --- |
| 1 | `c92fe4fbe88d6a8b5a0f78b8db5ec77b5eeeff63` | contributor | 저장 LineSeg가 그림 스택의 앵커 쪽 유지 조건을 보이면 #1995/#2004 재분류를 억제 |
| 2 | `74cd60a9067dd9e0ecee041d50a8f69439a2469f` | collaborator 보정 | 두 렌더 경로의 구조 술어와 절대 본문 하단 좌표 비교를 통일, `TopAndBottom` 음성 회귀 추가 |
| 3 | `85cafced1cdd44f54f601aa9821f671b80159a67` | collaborator 보정 | 공통 `Option` helper의 clippy 계약과 포맷 정리 |

## 보정 내용

1. `floating_image_stack_extents`가 그림 스택의 최소 폭과 최대 높이를 반환하고, 재분류와 typeset이
   같은 구조 조건을 사용한다.
2. `body_pile_stays_on_anchor_page`가 첫 LineSeg의 가로 시작점과 그림 하단을 판정한다. 본문 하단은
   `PageAreas::body_area.bottom`의 페이지 절대 좌표를 사용한다.
3. `TextWrap::TopAndBottom`을 `Square`로 오인하지 않는 회귀를 `typeset` 단위 테스트로 추가한다.
4. 보정 코드와 stage 기록은 contributor head 위의 별도 commit으로 분리했고, contributor commit에는
   rebase, amend, reset 또는 force push를 사용하지 않았다.

## 검증과 통합 단계

- 보정 head `85cafced1`에서 rustfmt, native-skia clippy, 새 unit test 및 release-test 전체 integration
  test를 완료했다.
- 같은 head의 GitHub CI, Rust CodeQL, Canvas visual diff가 모두 성공했다.
- 다음 trailing commit은 review와 오늘할일만 포함한다. 허용 범위의 single-parent 문서 commit이므로
  code candidate `85cafced1`의 녹색 Full CI를 review-only fast-pass 후보로 사용한다.
- trailing head의 aggregate와 mergeability를 재확인한 뒤 일반 squash merge를 수행한다. merge SHA,
  #4770 상태, contributor 감사 댓글, fork branch 보존 여부는 merge 후속 처리 단계에서 확인한다.
