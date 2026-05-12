# Task #850 Stage 1 완료보고서

## 단계

Stage 1 — RED 회귀 테스트 작성 및 실패 확인

## 작업 내용

신규 회귀 테스트를 추가했다.

- `tests/issue_850_answer_sheet_name_hit_test.rs`

테스트 대상:

- `samples/exam_social.hwp` 1쪽 상단 답안지 `성명` 오른쪽 빈 입력칸
- `samples/exam_science.hwp` 1쪽 상단 답안지 `성명` 오른쪽 빈 입력칸

검증 의도:

- hit-test 결과가 문서 루트 기준 외곽 표 컨텍스트를 유지해야 한다.
- `cellPath`가 외곽 상단 표와 내부 1x2 중첩 표를 모두 포함해야 한다.
- `insert_text_in_cell_by_path()`로 실제 이름 입력이 가능해야 한다.

## 기대값

`exam_social.hwp`:

- `parentParaIndex == 0`
- `controlIndex == 4`
- `cellPath == [(4, 0, 3), (0, 1, 0)]`

`exam_science.hwp`:

- `parentParaIndex == 0`
- `controlIndex == 6`
- `cellPath == [(6, 0, 3), (0, 1, 0)]`

## 실행 결과

명령:

```bash
cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture
```

결과:

```text
test result: FAILED. 0 passed; 2 failed; 0 ignored
```

`exam_social.hwp` 실제 hit-test 결과:

```json
{
  "sectionIndex": 0,
  "paragraphIndex": 0,
  "charOffset": 0,
  "parentParaIndex": 3,
  "controlIndex": 0,
  "cellIndex": 1,
  "cellParaIndex": 0,
  "cellPath": [
    { "controlIndex": 0, "cellIndex": 1, "cellParaIndex": 0 }
  ],
  "cursorRect": { "pageIndex": 0, "x": 212.7, "y": 211.8, "height": 15.3 }
}
```

`exam_science.hwp` 실제 hit-test 결과:

```json
{
  "sectionIndex": 0,
  "paragraphIndex": 0,
  "charOffset": 0,
  "parentParaIndex": 3,
  "controlIndex": 0,
  "cellIndex": 1,
  "cellParaIndex": 0,
  "cellPath": [
    { "controlIndex": 0, "cellIndex": 1, "cellParaIndex": 0 }
  ],
  "cursorRect": { "pageIndex": 0, "x": 206.7, "y": 212.8, "height": 13.3 }
}
```

## 판정

RED 확인 완료.

두 문서 모두 현재 `upstream/devel` 기준으로 외곽 상단 표 경로를 잃고, 내부 중첩 표의 로컬 컨텍스트만 문서 루트 컨텍스트처럼 반환한다.

이는 이슈 #850의 재현 증상인 `insertTextInCell(0, 3, 0, 1, 0, 0, text)` 잘못된 입력 경로와 일치한다. 따라서 테스트 실패는 좌표 오차가 아니라 실제 회귀를 정확히 포착한 실패다.

## 다음 단계

Stage 2에서 `src/document_core/queries/cursor_rect.rs`의 hit-test 경로 보존 로직을 정정한다.

우선 확인할 영역:

- `collect_runs()`의 메타 없는 중첩 `Table` 처리
- `cell_bboxes` 보정의 TextRun 템플릿 선택 조건
- `CellContext.path[0]` 기반 public `parentParaIndex/controlIndex` 반환 유지

## 승인 요청

Stage 2 진행 승인 요청.

