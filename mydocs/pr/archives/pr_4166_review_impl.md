---
kind: review-implementation
status: completed-local
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR #4166 통합·메인터너 보정 기록

## 적용 순서와 commit 경계

| 순서 | SHA | 역할 |
| --- | --- | --- |
| 1 | `bb88ddce834beb2a67d4db9d8b36a58d956e9b76` | 머리말/꼬리말 문단 reader를 툴바·눈금자에 연결 |
| 2 | `1e7be43ab1c274d33458daab9a614bad33472d32` | Percent가 아닌 줄 간격에서 툴바의 이전 선택값 제거 |
| 3 | `aeee39734a9a129d9fb4d715873880a6a1c514dc` | 표 경계 좌표 병합과 hit-test 대표 인덱스 map 추가 |
| 4 | `0f0bcce831dd254ccfb726610e244ba24406f788` | 머리말/꼬리말·각주 문단 서식 적용 경로 연결 |
| 5 | `d2a0f2e6f2ff0b1f444cadbfb376fc2288d900ee` | HF/FN 문단 서식 Undo/Redo 편집 문맥 보존 메인터너 보정 |

통합은 최신 `upstream/devel` `98acdd9a` 위에서 #4120 → #4133 → #4134 순서로 수행했다.
#4134의 `mutation-routing-guard` 기준선 충돌은 현재 `devel`의 #4119 호출 한 건과 #4134 호출
두 건을 합쳐 `30`으로 해소했다.

## 보정 범위

1. `OperationDescriptor`의 snapshot 경로에 선택적 `EditContext`를 선언했다.
2. 일반 `SnapshotCommand`에는 문맥 복원을 추가하지 않았다. 이 클래스는 기존 구조 편집의
   Undo 뒤 본문 복귀 계약을 그대로 유지한다.
3. `SubmodeSnapshotCommand`가 일반 snapshot 동작을 상속하고, 명시적으로 전달된 HF/FN 문맥만
   history 복원기에 노출한다.
4. 머리말/꼬리말과 각주의 문단 서식 호출부가 각각 현재 커서 좌표와 서브모드 식별자를
   descriptor에 전달한다.
5. source guard를 추가해 두 경로의 전용 명령 선택과 문맥 노출이 빠지면 테스트가 실패하게 했다.

## 완료한 검증과 판정

`npx tsc --noEmit`, focused Studio 35건, `npm test` 802건, `npm run build`를 순차로 통과했다.
headless Chromium에서 머리말·각주의 줄 간격을 각각 적용하고 Undo·Redo한 뒤에도 해당 편집
서브모드에 남는 것을 확인했다. 일반 snapshot의 문맥 비노출 정책을 검사하는
`undo-submode-insert.test.ts`도 함께 통과해 기존 구조 편집 계약이 바뀌지 않았음을 확인했다.

원격 `d2a0f2e6f` code head에서는 CI, CodeQL, Render Diff가 모두 성공했다. 이 문서와 오늘할일은
후속 documentation commit으로만 push한다. push 직후 최신 head의 review-only fast-pass 또는
workflow fallback 결과를 확인하고, 작업지시자의 merge 승인 뒤 #4166 병합과 원 PR 후속 처리를
수행한다.
