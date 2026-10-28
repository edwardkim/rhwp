# Task #1592 — 최종 결과보고서

**제목**: HWPX 저장 시 빈 문단(char_shapes=[])에 spurious (0,0) char_shape 추가 수정
**마일스톤**: M100 (v1.0.0) · **이슈**: edwardkim/rhwp#1592 · **브랜치**: `local/task1592`

## 1. 문제
run 이 없던 빈 문단에 직렬화기가 빈 `<hp:run charPrIDRef="0"><hp:t></hp:t></hp:run>` 추가 →
재파싱 시 char_shapes `[]`→`[(0,0)]`. fidelity 잔여 Class D(1건, 36386761 목록 para5).

## 2. 근본원인 (`src/serializer/hwpx/section.rs`)
- `RunSplitter::new`(298-300) 규칙3: char_shapes 비면 기본 (0,0) 세그먼트.
- `close_run`(333-335) 규칙5: 빈 run 도 `<hp:t></hp:t>` 방출.
- → char_shapes=[] 빈 문단에 charPrIDRef="0" run 생성. 원본은 run 없어 char_shapes=[].
  판별자 = `char_shapes.is_empty()`(빈 run 이면 파서가 [(0,0)] 산출하므로 [] 는 run 부재 의미).

## 3. 해결
`render_runs` 진입부: 완전 빈 문단(text·char_shapes·controls·field_ranges·orphan 전부 없음)이면
run 미방출(빈 문자열). char_shapes 있으면 종전 규칙3/5 유지. linesegarray 는 별도 경로라 보존.
`task1378_empty_paragraph_single_run_id_zero` 갱신(run 미방출이 정답).

## 4. 검증
| 검사 | 결과 |
|------|------|
| 단위 RED→GREEN | PASS |
| cargo test --lib | 1964 passed, 0 failed |
| hwpx_roundtrip_baseline | 4/4 |
| opengov snapshot (36386761 가드) | PASS |
| **fidelity 통제 비교** | **개선 1 / 회귀 0 / 순효과 +1** (IR_DIFF 4→3) |

빈 문단 광역 변경에도 공통 10581건 회귀 0(char_shapes=[] 조건이 좁음).

## 5. 산출물
- 소스: `src/serializer/hwpx/section.rs`
- 테스트: `task1592_empty_paragraph_no_spurious_charshape` + opengov 가드 + #1378 갱신
- 문서: 수행계획서, `_stage1~3`, 본 보고서

## 6. 후속 (잔여 IR_DIFF 3건)
- Class C1 (36384689·36385445): first-para mismatch-path char_shape (#1591 재범위, F3급).
- Class C2 (36388711): Field ClickHere −16/−8.
