# Task M100 #717 Stage 1 완료보고서

## 단계 목표

`samples/exam_social.hwp` 1/4쪽 자료 표 제목 행 빈 영역 클릭 좌표가 현재 `hit_test_native()`에서 어떤 결과를 반환하는지 코드 레벨에서 고정한다.

## 수행 내용

- `tests/issue_717_table_cell_hit_test.rs` 신규 추가.
- `HwpDocument::from_bytes()`로 `samples/exam_social.hwp`를 로드.
- 이슈 #717 기준 클릭 좌표를 native `hit_test_native()`로 직접 호출.
  - page: `0`
  - x: `191.0`
  - y: `356.0`
- 기대 컨텍스트를 테스트로 고정.
  - `sectionIndex=0`
  - `parentParaIndex=1`
  - `controlIndex=0`
  - `cellIndex` 존재
  - `cellPath` 존재

## 재현 결과

실행 명령:

```bash
cargo test --test issue_717_table_cell_hit_test -- --nocapture
```

결과: 실패. RED 재현 성공.

실제 반환값:

```json
{
  "sectionIndex": 0,
  "paragraphIndex": 0,
  "charOffset": 0,
  "parentParaIndex": 0,
  "controlIndex": 1,
  "cellIndex": 0,
  "cellParaIndex": 0,
  "cellPath": [
    { "controlIndex": 1, "cellIndex": 0, "cellParaIndex": 0 }
  ],
  "cursorRect": {
    "pageIndex": 0,
    "x": 486.8,
    "y": 1393.7,
    "height": 15.3
  }
}
```

## 분석

이슈 좌표는 SVG 디버그 오버레이 기준 `s0:pi=1 ci=0` 자료 표 bbox 내부이다.

하지만 실제 native hitTest 결과는 다음처럼 반환됐다.

- `parentParaIndex=0`
- `controlIndex=1`
- `cursorRect.x=486.8`
- `cursorRect.y=1393.7`

이는 이슈 본문의 "페이지 하단 작은 번호 표(`32`) 쪽으로 이동" 관찰과 일치한다. 즉 브라우저 이벤트나 TypeScript 경로 이전에, Rust/WASM `hit_test_native()` 자체가 클릭한 표가 아닌 다른 표 컨텍스트를 반환한다.

원인 후보는 수행 계획서의 가설과 일치한다.

- `cell_bboxes` 보완 로직이 `cell_index`만으로 첫 TextRun을 찾는다.
- 여러 표와 바탕쪽 표가 같은 `cell_index=0`을 반복 사용한다.
- 그 결과 클릭 bbox는 대상 표 내부여도, 컨텍스트가 `parentParaIndex=0/controlIndex=1`로 오염된다.

## 변경 파일

- `tests/issue_717_table_cell_hit_test.rs`
- `mydocs/working/task_m100_717_stage1.md`

## 검증 상태

| 항목 | 결과 | 비고 |
|------|------|------|
| `cargo test --test issue_717_table_cell_hit_test -- --nocapture` | 실패 | RED 재현 성공 |

## 다음 단계 요청

Stage 2 진행 승인을 요청한다.

Stage 2에서는 `src/document_core/queries/cursor_rect.rs::hit_test_native()`의 `cell_bboxes` 메타 보완 로직을 정정한다. 핵심은 `cell_index` 단독 매칭을 제거하고, `parent_para_index/control_index/cellPath` 기준으로 클릭한 셀 컨텍스트를 고정하는 것이다.
