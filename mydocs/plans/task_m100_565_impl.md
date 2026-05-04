# Task #565 구현계획서

## 1. 변경 위치

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/integration_tests.rs` | #565 재현 테스트 추가 |
| `src/renderer/layout/paragraph_layout.rs` | `layout_inline_table_paragraph` 에 TAC 수식 배치 추가 |

## 2. TDD 순서

1. 실패 테스트 추가
   `samples/exam_science.hwp` 12/15/18/19번 대상 문단에서 #565 대상 수식들이 같은 `translate(x,y)` 좌표에 겹치지 않는지 검증한다.

2. 실패 확인
   현재 구현에서는 `rmX`, `rmA`, `rmB`, `rmC`, `rmD`, `m-4`, `m-2`, `m+2`, `m+4` 가 같은 좌표에 렌더되어 테스트가 실패해야 한다.

3. 최소 수정
   `layout_inline_table_paragraph` 가 `inline_tables` 만 순회하던 부분을 `composed.tac_controls` 기반 `inline_items` 순회로 바꾼다. 표는 기존 table width/measure 경로를 유지하고, 수식은 인라인 `EquationNode` 로 직접 렌더링한 뒤 `inline_shape_position` 을 등록한다. 같은 text position 에 TAC 개체가 여러 개 있으면 모두 같은 gap 에서 순서대로 배치한다.

4. 통과 확인
   신규 테스트가 12번 `pi=61`, 15번 `pi=79`, 18번 `pi=110`, 19번 `pi=118` 을 모두 검증하고, 기존 layout integration 테스트를 통과시킨다.

5. 시각 검증
   수정 전/후 SVG를 PNG로 캡처하여 2쪽 12번 본문 빈자리와 겹침 해소를 확인한다.

## 3. 최소 수정 원칙

- 수식 파서, 토크나이저, Equation Version 60 매핑은 변경하지 않는다.
- TAC picture/shape 기존 경로는 변경하지 않는다.
- 표 폭 계산과 표 렌더링은 기존 경로를 그대로 사용한다.

## 4. 회귀 위험

- `inline_items` 는 `tac_controls` 의 text position 기준으로 정렬하여 control index 순서와 저장 위치가 어긋난 문서도 방어한다.
- 같은 text position 에 수식과 표가 연속 저장된 경우, 두 번째 개체가 다음 텍스트 뒤로 밀리지 않도록 현재 gap 의 TAC 개체를 모두 처리한다.
- `total_width` 에 수식 폭을 포함하여 Center/Right 정렬 문단에서 시작 x 가 어긋나지 않게 한다.
- 인라인 수식 렌더링 후 `inline_shape_position` 을 등록하여 fallback shape/layout 중복 렌더링을 막는다.
- 빈 paragraph TAC 수식(Task #287/#490)은 별도 `comp_line.runs.is_empty()` 경로라 영향 범위 밖이다.

## 5. 승인 후 실행할 명령

```bash
cargo test --lib test_565_exam_science_inline_tac_equations_are_not_collapsed
cargo test --lib renderer::layout::integration_tests -- --nocapture
cargo run --bin rhwp -- export-svg samples/exam_science.hwp -p 1 -o /private/tmp/rhwp_task565_after
```

## 6. 승인 상태

Stage 1 진행 중.
