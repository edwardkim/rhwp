# Task #1594 — 최종 결과보고서

**제목**: HWPX 직렬화 시 holdAnchorAndSO 드롭(1→0) 수정
**마일스톤**: M100 · **이슈**: #1594 · **브랜치**: `local/task1594`

## 1. 문제·근본원인
HWPX 직렬화기가 개체 `<hp:pos holdAnchorAndSO>` 를 IR 값 무시하고 "0" 하드코딩
(table/picture/shape/equation). 페이지 하단 앵커 개체에서 1→0 드롭 시 한글 페이지 붕괴.
IR diff=0(게이트 미검사)라 시각-only. #1589 군집 단락 이진탐색으로 36383351 의 단일 원인 확정.

## 2. 수정
1. 직렬화 4지점이 `holdAnchorAndSO` 를 `c.prevent_page_break != 0` 로 방출(하드코딩 제거).
2. `diff_documents` 에 `ObjectHoldAnchor` 비교 추가(Table/Picture/Equation) → 게이트 봉인.

## 3. 검증
| 검사 | 결과 |
|------|------|
| 단위 RED→GREEN (table) | PASS |
| cargo test --lib | 1969/0 |
| hwpx_roundtrip_baseline | 4/4 |
| IR 통제 비교(11855) | IR_DIFF 4(회귀 0), holdAnchor 게이트 0 mismatch |
| 한글 오라클 | 36383351 붕괴 해소(2→2), 이전 OK 30/30 유지(악화 0) |

## 4. 한계 (정직)
#1589 페이지 붕괴 **군집은 이질적**. holdAnchorAndSO 는 36383351 의 deciding 요인이나,
붕괴 표본의 22/30 은 holdAnchorAndSO 보존됐는데도 붕괴(다른 systematic 드롭 deciding).
→ 본 수정은 holdAnchorAndSO-deciding 부분집합만 해소. 군집 대다수는 후속(아래).

## 5. 후속
다른 IR-invisible 직렬화 드롭 후보(holdAnchorAndSO 와 동형): outlineShapeIDRef(0→1),
noteSpacing(미세 감소), noteLine(NONE→SOLID), curSz(0→5669). 각 별 조사 권장.

## 6. 산출물
소스: table/picture/shape/section(equation)/roundtrip.rs. 테스트: task1594_* + opengov 36383351.
