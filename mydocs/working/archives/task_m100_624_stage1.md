# Task #624 Stage 1 보고서

## 목적

회귀 재현 TDD 통합 테스트 추가 → RED 확인.

## 적용 변경

### `src/renderer/layout/integration_tests.rs`

`test_624_textbox_inline_shape_y_on_line2_p2_q7` 추가 (+58 LOC).

**테스트 로직**:
1. `samples/exam_science.hwp` page 2 (index 1) SVG 렌더
2. ㉠ 사각형 식별: `<rect width≈63 height≈22.88 fill="#ffffff" stroke="#000000"/>`
3. y 좌표 검증: 정상 범위 [230, 240] (Line 2 영역)
   - 회귀 시: y≈213.95 (Line 1 영역)
   - 정상 시: y≈235.65 (Line 2 영역)

## RED 확인

```
test test_624_textbox_inline_shape_y_on_line2_p2_q7 ... FAILED

Task #624: ㉠ 사각형 y=213.95 가 Line 2 영역 [230, 240] 에 있어야 함.
회귀 (Task #520 부분 회귀): y≈213.95 (Line 1 영역, 본문 '분자당 구성' 위 겹침).
정정 (3 line fix): y≈235.65 (Line 2 영역, ' 이다.' 앞).
```

측정값 y=213.95 가 회귀 범위와 정확히 일치. 회귀 재현 완료.

## 베이스 브랜치 정정

수행계획서/구현계획서 작성 중 base 브랜치 오설정 (`local/devel`) 발견:
- `local/devel`: Task #520 fix **이미 보존** (회귀 없음)
- `pr-task618` / `devel`: Task #520 회귀 존재 (PR #561 cherry-pick `3de0505`)

`local/task624` 를 `pr-task618` 에서 재분기. Stage 0 plan commit 은 그대로 유지 (`git am` 으로 재적용).

## 다음 단계

Stage 2: `src/renderer/layout/table_layout.rs` 3 라인 정정 적용 → RED → GREEN 전환 + 광범위 cargo test --lib 회귀 확인.
