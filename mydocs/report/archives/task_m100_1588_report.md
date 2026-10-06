# Task #1588 — 최종 결과보고서

**제목**: HWPX 저장 시 선 도형(`hp:line`) shapeComment 드롭 수정
**마일스톤**: M100 (v1.0.0) · **이슈**: edwardkim/rhwp#1588 · **브랜치**: `local/task1588`

---

## 1. 문제

HWPX 저장 시 선 도형의 설명(`shapeComment`, "선입니다.")이 드롭. fidelity 잔여 IR_DIFF
Class B 3건(36389418·36392900·36391302).

## 2. 근본원인 (`src/serializer/hwpx/shape.rs`)

`write_shape_comment(c)` 는 `c.description` 비어있지 않으면 `<hp:shapeComment>` 방출.
`write_rect`·`write_container_close` 는 호출하나 **`write_line` 만 미호출** → 선 도형 설명 드롭.
파서는 정상(원본 파싱이 description 캡처 — IR_DIFF expected 값이 증거). 순수 직렬화기 1줄 누락.

## 3. 해결

`write_line` 의 caption 방출 직후 `write_shape_comment(w, c)?;` 1줄 추가
(OWPML 순서 outMargin→caption→shapeComment, write_rect 동형).

## 4. 검증

| 검사 | 결과 |
|------|------|
| 단위 RED→GREEN (방출 + 빈설명 미방출) | PASS |
| `cargo test --lib` | 1963 passed, 0 failed |
| `hwpx_roundtrip_baseline` | 4/4 |
| opengov snapshot (36392900 가드) | PASS |
| **fidelity 통제 비교** | **개선 3 / 회귀 0 / 순효과 +3** (IR_DIFF 7→4) |

## 5. 산출물

- 소스: `src/serializer/hwpx/shape.rs`
- 테스트: `task1588_line_shape_comment_emitted` 외 1 + `samples/hwpx/opengov/36392900…` 가드
- 문서: 수행계획서, `_stage1~3`, 본 보고서

## 6. 후속 (잔존 IR_DIFF 4건)

- Class C — para0 char_shape 시프트 3건(36384689·36385445·36388711): secPr/colPr run charPr
  경계 오정렬, dedicated 조사 필요.
- Class D — spurious (0,0) 1건(36386761): 빈/공백 문단 기본 char_shape.
- 별도 — #1589 페이지 붕괴(시각 갭, 오라클 전수 군집 조사).
