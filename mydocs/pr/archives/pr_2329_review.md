# PR #2329 검토 — undo 뮤테이션 라우팅 가드 세트 (#2327)

- PR: https://github.com/edwardkim/rhwp/pull/2329 (lpaiu-cs)
- 이슈: #2327 (undo 연작 후속) — 소스/DEV 전용, 런타임 동작 무변경
- base=devel, MERGEABLE/BEHIND, 4파일 +334

## 변경 본질

undo 기록이 옵트인이라 미라우팅 뮤테이션(3중 위험: undo 불가·redo 오염·
스냅샷 동반 파괴)을 구조적으로 잡는 2중 가드:

1. **단일 권위 목록** `wasm-mutation-guard.ts`: MUTATING_METHODS 123 +
   EXCLUDED_NON_DOCUMENT — 드리프트 테스트가 신규 브리지 메서드의 분류
   누락을 강제.
2. **DEV 런타임 가드**: CommandHistory 실행 창(opDepth) 밖 뮤테이터 호출을
   메서드당 1회 warn. IME 조합 2단 계약은 `allowUnrecordedMutation` 명시
   escape (유일 사용처 확인).
3. **원장 트립와이어**: ui/+command/ 뮤테이션 표면(23파일 호출 수) 동결 —
   증가 시 FAIL 로 의식적 baseline 갱신+리뷰 강제. "라우팅 여부"가 아니라
   "표면 증가"의 트립와이어라는 한계를 스스로 명시 (정직한 계약).

## 로컬 재실증 (merged tree)

| 게이트 | 결과 |
|--------|------|
| 신규 가드 테스트 | 4/4 |
| **변조 검사** (MUTATING_METHODS 1개 제거) | 드리프트 **FAIL 재현** → 원복 4/4 — 트립와이어 실효 확인 |
| studio 단위 / tsc | **311/311** / 0 |
| e2e (DEV 가드 활성 상태) | undo-contracts 24/0 · unsaved-guard 6/0 · text-flow 0 FAIL — warn 과 오류 수집기 무충돌 |

## 판단

**merge 권고.** 재발 계급(#2027/#2037/#2053/#2077)을 버그당 사후 가드가
아니라 구조로 차단. 후속 연작(미라우팅 ~45곳 이관, 원장 감소가 진행 게이지)
의 기반 PR. #2332 와 연작 관계 — 순서 처리.
