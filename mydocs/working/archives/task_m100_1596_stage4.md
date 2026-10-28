# Task #1596 — Stage 4 완료보고서 (통제 비교)

**브랜치**: `local/task1596`

## 1. IR 통제 비교 (fidelity17, 11874 파일)
IR_DIFF **4** (불변) → 회귀 0. (도형 지오메트리 드롭은 본래 IR-invisible — diff_documents 가
shape points/lineShape 미비교.)

## 2. 한글 오라클 — 붕괴 해소 + 악화
- 36396457(polygon, 11→4 붕괴) → 수정 후 **11→11 해소**. 지오메트리 보존(hc:pt 32·lineShape 4·shadow 4 = orig).
- 이전 붕괴 표본 40(누적 ClickHere+shape): OK 38/COLLAPSE 2 (#1595 단독 37 대비 +1 = polygon 해소).
- 이전 OK 표본 40: **40/40 유지 (악화 0)**.

## 3. 판정 — 채택
generic-shape 지오메트리 방출로 도형 충실도 복원 + 잔여 shape 붕괴 해소, 악화 0, IR/baseline/lib 회귀 0.

## 4. 가드
단위 `task1596_polygon_geometry_serialized`. 지오메트리는 IR-invisible(diff_documents 미비교)이라
단위 가드가 유일·충분. (게이트 IR-visible化는 후속 검토.)
