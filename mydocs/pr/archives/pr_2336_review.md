# PR #2336 검토 — #2329 DEV 런타임 가드 제거, 소스 가드 일원화 (#2334)

- PR: https://github.com/edwardkim/rhwp/pull/2336 (lpaiu-cs, #2329 자기 정정 후속)
- 경위: 제거 커밋이 #2329 머지 레이스로 미반영 → 후속 PR

## 주장 검증 — 메인테이너 독립 확증

컨트리뷰터 주장(런타임 가드 오탐 net-negative)을 소스로 직접 확인:

- `input-handler-picture.ts` 드래그/nudge 는 `setObjectProperties` 로 뮤테이션을
  **직접 적용**(opDepth==0) 후 `executeOperation({kind:'record'})` →
  `recordWithoutExecute` 로 **사후 기록** — 정상 계약인데 가드가 경고
- `warnedMethods` 1회 dedup 이 고빈도 메서드명을 소진 → **진짜 미라우팅
  감지까지 침묵** (가드 자기 목적 훼손)
- #2329 검토 시 저희 e2e 배터리가 대화상자 흐름만 커버해 드래그 경로 오탐을
  못 봄 — 검증 커버리지의 맹점이었음을 인정

`allowUnrecordedMutation` 개별 래핑 대안은 옵트인 취약성 재도입이라는 논증도
타당 — 제거 + 소스 가드 일원화가 옳은 방향.

## 변경 본질

- 런타임 가드/escape/설치 전부 제거, IME 래핑 되돌림 (#2329 소스 원상)
- 권위 목록만 `mutation-method-registry.ts` 로 존치
- **소스 가드 강화**: 원장 스캔에 `engine/input-handler*`(종전 누락 — 최고밀도
  직접 뮤테이션 영역) 추가, MUTATING_VERB 확장 + 자기정합 단언

## 로컬 재실증 (merged tree)

| 게이트 | 결과 |
|--------|------|
| 가드 테스트 | 5/5 + 변조 검사(목록 1개 제거 → FAIL) 실효 유지 |
| 런타임 가드 잔재 | 소스 스캔 0건 (완전 제거) |
| studio 단위 / tsc | **317/317** / 0 |
| e2e | undo-contracts 24/0 · text-flow 0 FAIL |

## 판단

**merge 권고.** 머지 수시간 내 자기 code-review 로 결함을 확증하고 근거·대안
분석과 함께 되돌린 것 — #2329 의 가치(저작 시점 구조 차단)는 소스 가드
강화로 오히려 확대 유지. #2334 는 Closes 미선언 시 확인 필요.
