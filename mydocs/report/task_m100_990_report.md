# Task #990 최종 결과보고서

**제목**: 빈 문단 위 treat-as-char 글상자 — `FullParagraph`/`Shape` PageItem advance 이중 가산 정정
**이슈**: [#990](https://github.com/edwardkim/rhwp/issues/990)
**브랜치**: `local/task990`
**마일스톤**: M100 (v1.0.0)

---

## 1. 증상

빈 문단(텍스트 없음) 위에 얹힌 `treat_as_char` 글상자(도형)가 세로로 연속
배치될 때, 박스 사이 세로 간격이 한컴 PDF 대비 **정확히 2배**로 벌어졌다.

## 2. Root cause — Task #974 회귀

bisect 결과 커밋 `c3e32151`("Fix textbox picture rendering", Task #974)가
`layout_shape_item()` 에 `Control::Shape` 분기를 **신규 추가**하면서 회귀 발생.

| 지점 | 박스 advance |
|------|-------------|
| `c3e32151~1` (Task #974 직전) | 66.44px (정상) |
| `c3e32151` 이후 / `devel` | 132.88px (2배) |

빈 문단 호스트는 `FullParagraph` PageItem 의 `layout_paragraph()` 가 이미
LINE_SEG advance(`lh+ls`)를 마친 상태인데, Task #974 가 추가한
`Shape` PageItem 분기가 `result_y = shape_y + line_advance.max(shape_h)` 로
**또 한 번 advance** → `4983 + 4983 = 9966 HU`.

## 3. 정정

`src/renderer/layout.rs` — `layout_shape_item()` 의 `!has_real_text` 분기:

- 해당 문단에 `PageItem::FullParagraph` 가 발행되었는지 확인
  (`has_full_para_item`).
- **있으면**(빈 문단 호스트 = RFP 형): 글상자를 호스트 문단 시작
  (`para_start`)에 배치하고 `result_y` 재진행을 생략 — 이중 가산 제거.
- **없으면**(선행 표 등에 이어 붙은 Shape, 예: `hy-001` pi=27):
  Task #974 동작(`shape_y=y_offset`, `result_y` 진행)을 유지.

Task #974 의 `set_inline_shape_position` 등록 의도는 보존.

> 초기안(Stage 2)은 FullParagraph 유무를 구분하지 않아 `hy-001` pi=27
> (선행 표 + Shape)의 글상자를 표 위로 올려 표를 가리는 회귀를 일으켰고,
> Stage 2 v2 에서 `has_full_para_item` 분기로 정밀화하여 해소.

## 4. 검증

- **회귀 진단(개발 단계)**: 비공개 문서(treat-as-char 글상자 3개)로
  RED 재현 — 박스 advance 132.88px → **66.44px**, PDF 비율 1.92 정합.
  해당 문서·임시 테스트는 비공개 자료라 커밋하지 않음(작업지시자 지시).
- **커밋된 회귀 가드**: `issue_table_vpos_01_page5_cell_hit_test` —
  `table-vpos-01.hwp` 5쪽 `pi=33`(빈 문단 + treat-as-char 도형)이 본 정정의
  동일 코드 경로를 거치므로, 이중 가산이 재발하면 셀 hit-test 가 실패한다.
- **전체 `cargo test`**: 1483 passed, 0 failed.
- **`cargo clippy` / `fmt`**: clean.
- **광범위 SVG sweep**(14 샘플): 123 byte-identical, 회귀 0.
  - `hy-001`: devel 과 동일(Task #974 동작 보존 확인).
  - `group-box`: 투명 rect 좌표 정정, PNG 렌더 바이트 동일(시각 변화 0).
- **`issue_table_vpos_01_page5_cell_hit_test`**: `pi=33` 이 동일 구조라
  `pi=34` 가 30.84px 상향 이동. 테스트 기댓값은 커밋 `c2d2157d` 가 #974
  버그 레이아웃에 박제한 stale 값 — `pi=34` inner 11x3 좌표 8건을
  정정 레이아웃에 맞춰 갱신(13/13 통과). 한컴 PDF 와 정합 확인.

## 5. 변경 파일

| 파일 | 내용 |
|------|------|
| `src/renderer/layout.rs` | `layout_shape_item` 빈 문단 분기 정정 |
| `tests/issue_table_vpos_01_page5_cell_hit_test.rs` | #974 박제 좌표 8건 정정 |

> 개발 중 작성한 임시 회귀 테스트(`tests/issue_990.rs`)와 재현 픽스처는
> 비공개 문서 기반이라 커밋하지 않는다.

## 6. WASM 빌드

Docker WASM 빌드 완료 — `pkg/rhwp_bg.wasm` (4.6 MB) 갱신.

## 7. 비고

- 본 task 브랜치는 `local/devel` 이 PR #538 시점(659+ 커밋) stale 하여
  `stream/devel` 로 정렬 후 진행(`local/devel` 직전 상태는
  `backup/local-devel-20260518` 보존).
- 이중 가산 제거 후에도 RFP 박스는 PDF 대비 ~1400 HU 잔차(matrix 스케일 +
  spacing) 가능 — 부차적, 별도 issue 후보.
