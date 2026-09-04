# Task #1595 — Stage 3 완료보고서 (통제 비교)

**브랜치**: `local/task1595` · **바이너리**: local/task1595 HEAD

## 1. IR 통제 비교 (fidelity16, 12042 파일)
IR_DIFF **4** (불변) → 수정의 IR 회귀 0 (IR-invisible 수정; 파서가 CLICKHERE·CLICK_HERE 양형
수용해 enum 동일 → IR 비교 불변).

## 2. 한글 오라클 — 붕괴 해소율
- 이전 붕괴 표본 40 재측정: **OK 37 / COLLAPSE 3 → 해소율 92.5%**.
  (예: 36391546 8쪽→1쪽 붕괴 → 수정 후 8쪽.)
- 이전 OK 표본 30: **30/30 OK 유지 (악화 0)**.

→ **#1589 페이지 붕괴 군집(~16%)의 대다수 해소**. 잔여 붕괴 ~8%(다른 원인: holdAnchorAndSO#1594
+ 미상). 시각 붕괴 갭 ~16% → ~1.3%(추정).

## 3. 회귀 가드
단위 테스트 `field_begin_emits_type_attr`(type="CLICK_HERE" 단언)가 직렬화 회귀 봉인.
필드 타입 버그는 파서 정규화로 **본질적 IR-invisible**(enum 동일) → diff_documents 추가 불가,
단위 테스트가 유일·충분한 가드.

## 4. 판정 — 채택
지배원인 단일 수정으로 붕괴 92.5% 해소, 악화 0, IR/baseline/lib 회귀 0. 채택.
