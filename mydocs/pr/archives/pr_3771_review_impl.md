---
kind: review-plan
status: active
canonical: mydocs/pr/archives/pr_3771_review_impl.md
last_verified: 2026-08-03
---

# PR #3771 메인터너 보정 반영 기록

## Commit 경계

| 순서 | commit | 역할 |
| --- | --- | --- |
| 기준 | `efd25b455686847ac27140d2183988859f6ca63c` | 보정 시작 시 contributor PR 원격 head |
| 1 | `43f7c54df` | Alt의 한컴형 인접 열/행 조절 보정 |
| 2 | `b22822f67` | collaborator 외부 PR 가시성 branch 연속 사용 절차 정정 |
| 3 | `22e3595c2` | 병합 셀 Alt 조절 범위 보정 |
| 4 | `7a5785856` | F5 이동의 내부 문서 위치 동기화 |
| 5 | `85e87a044` | F5 셀 선택 중 화면 캐럿 숨김 |
| 6 | `e7fb4f5be` | F5 셀 선택 Escape의 마지막 셀 캐럿 복원 |
| 기록 | review·구현 기록·오늘할일 commit | 검증 결과와 merge gate 보존 |

## 수행 결과

1. 완료: Alt 조절을 한컴의 표 크기 유지 규칙에 맞춰 선택 축과 즉시 이웃 축의 반대 delta로 제한했다.
2. 완료: 병합 셀의 실제 `rowSpan`/`colSpan` 범위를 Alt 조절 helper와 회귀 테스트에 반영했다.
3. 완료: F5 phase 1 이동 시 입력 대상이 마지막 선택 셀과 일치하도록 `CursorState` 문서 위치를 갱신했다.
4. 완료: 셀 선택 중에는 텍스트 캐럿을 숨기고, `Escape`에서는 표 개체 선택을 만들지 않고 그 위치의
   편집 캐럿만 복원했다.
5. 완료: focused 6건, Studio 전체 749건, TypeScript 검사, production build와 작업지시자 브라우저
   상호작용 검증을 마쳤다.
6. 대기: contributor head branch로 보정과 review 기록을 push하고 최신 head full CI를 확인한다.
7. 대기: 작업지시자 승인 범위에 따라 merge, contributor PR comment, `devel` fast-forward와 정확한
   review branch 정리를 수행한다.

## 롤백 경계

- Alt 또는 병합 셀 조절 회귀는 `43f7c54df`와 `22e3595c2`의 table resize helper·테스트만 독립적으로
  되돌릴 수 있다.
- F5 선택 표현 회귀는 `7a5785856`, `85e87a044`, `e7fb4f5be`의 cursor/keyboard/test 묶음만
  독립적으로 되돌릴 수 있다.
- workflow와 review 기록은 코드 동작에 영향을 주지 않으며 사실 관계만 정정한다.

## 원격 상태 변경 gate

작업지시자는 source branch remote push, 최신 CI 완료 뒤 merge, contributor PR comment와 merge 후
정리를 승인했다. push 직전 원격 head SHA와 LFS 대상 여부를 다시 확인한다. 코드·테스트 commit이
포함되어 있으므로 review-only fast-pass를 적용하지 않고 최신 head full CI 성공을 기다린다.
