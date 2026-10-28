# Task #1595 — 최종 결과보고서

**제목**: HWPX ClickHere 필드 타입 CLICKHERE→CLICK_HERE 수정 (페이지 붕괴 지배원인)
**마일스톤**: M100 · **이슈**: #1595 · **브랜치**: `local/task1595`

## 1. 근본원인
`serializer/hwpx/field.rs:180` 이 ClickHere 필드 타입을 `"CLICKHERE"`(언더스코어 누락)로 방출.
정답 `"CLICK_HERE"`(파서 4254·템플릿 2250/6695). 한글이 "CLICKHERE" 미인식 → ClickHere
placeholder 높이 변동 → 페이지 붕괴. 파서 관대(양형 수용)로 enum 동일 → IR diff=0(게이트 미검출).

## 2. 수정
`field.rs:180` `ClickHere => "CLICK_HERE"`. "CLICKHERE" 기대 테스트 2건 갱신.

## 3. 검증
| 검사 | 결과 |
|------|------|
| 단위 RED→GREEN | PASS (type="CLICK_HERE") |
| cargo test --lib | 1969/0 |
| baseline | 4/4 |
| IR 통제 비교(12042) | IR_DIFF 4(회귀 0) |
| 한글 오라클 붕괴 해소 | **37/40 (92.5%)** |
| 한글 오라클 악화 | 이전 OK 30/30 유지 (0) |

## 4. 영향
#1589 페이지 붕괴 군집(IR-invisible, PASS 파일 ~16%)의 **지배원인**. 붕괴파일 96% 가 ClickHere
보유, 본 수정으로 **92.5% 해소**. 시각 붕괴 갭 ~16% → ~1.3%(추정). 단일 1줄 수정.

## 5. 후속
잔여 붕괴 ~8%: holdAnchorAndSO(#1594, 일부) + 미상 소수. #1589 추가 좁히기 가능.

## 6. 산출물
소스: `serializer/hwpx/field.rs`(+테스트). 가드: `field_begin_emits_type_attr`(IR-invisible 라 단위 가드).
