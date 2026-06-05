# PR #1178 검토 — task 1139: 수식/미주 렌더링 정합 + 미주 입력 (보류: rebase 요청)

## 1. PR 정보

| 항목 | 내용 |
|------|------|
| 번호 | #1178 |
| 제목 | task 1139: 수식/미주 렌더링 정합 보정 및 미주 입력 지원 |
| 작성자 | @jangster77 (Taesup Jang) |
| base ← head | devel ← task_m100_1139 (cross-repo) |
| 상태 | OPEN, **CONFLICTING / DIRTY** |
| 변경 | +11172/-619, 99 파일 (.rs 36 + .ts 19, 샘플 9, 문서 68) |
| 라벨 | enhancement / v1.0.0 / Closes #1139 |
| CI | 6 pass (충돌 전 SHA 기준) |

## 2. 처리 결정: rebase 요청 (보류)

직전 머지 PR #1177 (Task #1151 표+picture, `cc8dee68`) 과 동일 picture/layout 코드를 각각 수정 → **레이아웃 로직 의미 충돌** (단순 텍스트 충돌 아님).

### 충돌 4파일 8 hunk
- `object_ops.rs`: #1177 tac 토글 migration vs 본 PR resolve_picture_control_mut resolver
- `picture-props-dialog.ts`: #1177 picture cellPath 분기 vs 본 PR 'group' 타입 추가
- `layout.rs` (3): #1177 v9 sibling TAC 가로 분배 vs 본 PR 미주 예약/label_extra (같은 picture y 계산 위치)
- `paragraph_layout.rs` (2): 동일

### 직접 해결 시도 → 중단
1-a(메인테이너 직접 결합)를 시도했으나, object_ops/dialog 는 정상 결합했으나 layout 충돌이 두 레이아웃 알고리즘의 깊은 의미 통합을 요구. 출력 환경 불안정으로 정확한 결합 신뢰성 부족 → **전체 abort (devel 무오염 f8970915 유지)**.

### rebase 요청 근거
layout 통합은 #1151 v9 가로 분배·sibling 표 y 보정과 미주/라벨 로직을 어떻게 합칠지 **작성자 판단이 정확**. 잘못 결합 시 #1151/#1139 레이아웃 조용히 깨짐 위험.

## 3. PR 자체 제한 사항 (작성자 명시)

- 미주는 두 시험문제 케이스 한정 제한적 보정. 다중 미주/편집 흐름/**저장 round-trip 미보장**.
- studio 미주 UI 는 기본 입력 경로 확인 수준.

## 4. 후속

- PR #1178 rebase 후 재검토 (issue-4585678997)
- @jangster77 #1180 (Rust 테스트 경고 정리) 도 열림 — 순서 조율
