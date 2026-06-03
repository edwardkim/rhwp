# Task #295 구현 계획서 — Paper-앵커 TopAndBottom 표의 본문 외부 배치 처리

## 핵심 진단

`src/renderer/layout.rs::layout_table_item` (≈1995행)에 다음 분기가 있다.

```rust
let renders_above_body = !is_tac
    && matches!(t.common.vert_rel_to, VertRelTo::Paper)
    && matches!(t.common.text_wrap, TextWrap::TopAndBottom)
    && { tbl_y < layout.body_area.y };  // 본문 위(머리말 자리)일 때만 true
if renders_above_body {
    // paper_images에 out-of-flow 렌더, y_offset 진행 안 함
} else {
    // col_node에 렌더, y_offset = layout_table 반환값(=표 하단)
}
```

이 분기는 머리말 영역의 페이퍼-앵커 표만 본문 흐름에서 분리한다. **꼬리말 영역(본문 아래) 페이퍼-앵커 표는 동일한 처리가 필요한데 누락**되어 있다.

`exam_math.hwp` 12쪽 `pi=22 ctrl[0]`은:
- wrap=TopAndBottom, vert=Paper/101954 HU(≈357mm)
- 본문 영역 하단(≈340mm) **아래**에 위치 → "확인 사항" 푸터 박스
- → `tbl_y >= body_area.y + body_area.height` (꼬리말 영역)
- → `renders_above_body=false` → else 분기로 진입 → `y_offset`이 푸터 위치로 jump

대칭적으로 본문 위(머리말)뿐 아니라 본문 아래(꼬리말)에 그려지는 페이퍼-앵커 TopAndBottom 표도 out-of-flow로 처리해야 한다.

또 동일 함수 하단 `is_above_body`(≈2090행)도 같은 조건으로 표 아래 간격 추가 여부를 결정하므로 동일하게 확장해야 한다.

## 수정 범위

| 파일 | 위치 | 변경 |
|------|------|------|
| `src/renderer/layout.rs` | `layout_table_item::renders_above_body` (≈1995) | `renders_outside_body`로 의미 확장 — `tbl_y < body_area.y` OR `tbl_y >= body_area.y + body_area.height` |
| `src/renderer/layout.rs` | `is_above_body` (≈2090) | 동일 확장 |

`Shape`(글상자/그림)도 동형 분기가 있을 가능성 — `layout_shape_item`(≈2326)도 점검 후 동일 패턴이면 확장.

## 단계 구성

### 3-1. 가설 확정 (디버그 임시 프린트)

확정을 위해 `layout_table_item` 진입/`renders_above_body` 분기/`y_offset` 변화를 1회 trace.
- 임시 `eprintln!`을 pi=22 처리 경로에 삽입 → 12쪽 export-svg 1회 실행 → 로그 확인 → 즉시 제거
- 확인 항목: pi=22 ctrl[0]의 `tbl_y`, `renders_above_body` 결과, layout_table 반환 y_offset

### 3-2. 본 수정

`renders_above_body` → `renders_outside_body`로 변경, 위/아래 양쪽 처리. 코멘트 갱신.
`is_above_body`도 동일 확장.

`layout_shape_item`도 동일 분기 있으면 확장.

### 3-3. 단위 검증

- `cargo build --release` 통과
- 12쪽 export-svg → debug-overlay y 좌표가 PDF 구조와 일치 (좌단 본문이 y≈147 부근에서 시작)
- LAYOUT_OVERFLOW 로그 사라짐
- inkscape PNG 변환 후 PDF와 시각적 비교

### 3-4. 인접/회귀 페이지 확인

영향 가능 페이지를 export-svg로 점검:
- `exam_math.hwp` 1, 4, 8, 12, 16, 20쪽 (다단 + 푸터 패턴)
- `exam_math_no.hwp` 동일 페이지
- 머리말 페이퍼-앵커 표를 가진 다른 샘플 (exam_math 1쪽이 해당) — `renders_above_body` 기존 동작 보존 검증

## 4단계 회귀 테스트

- `cargo test --release` 전체
- 주요 샘플 export-svg 단순 실행 (오류/패닉 없음):
  - `samples/equation-lim.hwp`, `samples/text-align-2.hwp` 등 푸터/머리말 표가 있는 문서

## 위험·트레이드오프

- `renders_above_body` 의미 확장 시 이름 변경하므로 grep 영향 작음 (호출부 없음, 함수 내 지역변수)
- 기존 머리말 처리(`above`) 동작은 `tbl_y < body_area.y` 조건 유지로 보존
- 꼬리말 페이퍼-앵커 표가 사실은 본문 흐름에 영향을 줘야 하는 케이스가 있는지 — 현재까지 발견된 사례 없음. 본문 흐름 영향이 필요한 표는 일반적으로 vert=Para 또는 본문 영역 내 vert=Paper로 작성됨.

## 범위 외 (후속 처리)

- 머리말 페이지번호 `4`↔`2` 불일치 — 별도 이슈 분리 (1단계 보고서에 명시)
- 다단 레이아웃 전반 재설계 — 본 픽스에 한함

## 결과물

- 수정: `src/renderer/layout.rs`
- 보고서: `mydocs/working/task_m100_295_stage3.md` (수정 + 단위 검증), `task_m100_295_stage4.md` (회귀)
- 최종: `mydocs/report/task_m100_295_report.md`
