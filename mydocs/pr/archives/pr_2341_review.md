# PR #2341 검토 — history-jumped 파생 상태 무효화 일반화 (#2339, 연작 2)

- PR: https://github.com/edwardkim/rhwp/pull/2341 (lpaiu-cs) — Closes #2339
- #2303(개체 선택만 인라인 해제)의 계급 2 일반화

## 변경 본질

히스토리 점프 시 파생 상태 무효화가 개별 옵트인이던 것을 —

1. `resetDerivedStateAfterHistoryJump` 로 일반화: 개체/표 선택(기존) +
   텍스트 선택 clear(유령 범위 → WASM 예외/무언 삭제 차단) + F5 셀 블록
   해제 + **`history-jumped` emit** (확장점)
2. find-dialog 가 이벤트 구독으로 currentHit 무효화 (열림 동안만 — 누수 방지)
3. 선택 = CLEAR 안전 최소 (한컴식 원 선택 복원은 별도 enhancement 명시)

## 충돌 해소 (maintainer edit push 5927490e)

#2338(HF/FN 컨텍스트 복원)과 handleUndo/Redo 겹침 — 통합 방침:
reset(일반화)이 exitObjectSelection 을 대체하고, 커서 이동은 #2338 의
restoreEditContextAfterHistory 가 담당(본문 분기가 moveTo 포함). 구 #2303
잔재 참조 0 확인. 해소 후 fork 브랜치 push → MERGEABLE.

## 로컬 재실증 (통합 트리)

| 게이트 | 결과 |
|--------|------|
| 신규 테스트 | 4/4 (구 헬퍼명 부재 단언 포함) |
| studio 단위 / tsc | **338/338** / 0 |
| e2e | undo-contracts 24/0 · undo-object-selection 0 FAIL |

## 판단

**merge 권고.** 계급 2(동형 stale 재발) 근절 + 이후 파생 상태는 구독만으로
합류하는 확장점 — #2303 계보의 올바른 일반화.
