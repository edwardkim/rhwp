# Task #850 최종 결과 보고서

**이슈**: [#850 — rhwp-studio v0.7.11 회귀: exam_social/exam_science 성명 칸 입력 시 컨트롤 인덱스 0 범위 초과](https://github.com/edwardkim/rhwp/issues/850)  
**브랜치**: `local/task850` (`upstream/devel` 기준)  
**마일스톤**: v1.0.0 (M100)

## 1. 결함 요약

`rhwp-studio`에서 수능형 샘플 문서 1쪽 상단 답안지 영역의 `성명` 입력칸에 이름을 입력하면 다음 오류가 발생하고 입력이 반영되지 않았다.

```text
Uncaught 렌더링 오류: 컨트롤 인덱스 0 범위 초과
```

확인 문서:

- `samples/exam_social.hwp`
- `samples/exam_science.hwp`

사용자 확인 기준으로 `v0.7.10`에서는 정상 입력되었고 `v0.7.11`부터 실패한 회귀다.

## 2. 원인

상단 답안지의 `성명` 칸은 문서 루트 문단 3의 표가 아니라 첫 문단의 상단 표 내부에 있는 중첩 표다.

`exam_social.hwp` 구조:

- 외곽 상단 표: `section 0 / paragraph 0 / control 4`
- 성명 입력칸: 외곽 표 `cellIndex=0`, `cellParaIndex=3` 내부 1x2 중첩 표의 두 번째 셀
- 기대 경로: `[(4, 0, 3), (0, 1, 0)]`

`exam_science.hwp` 구조:

- 외곽 상단 표: `section 0 / paragraph 0 / control 6`
- 성명 입력칸: 외곽 표 `cellIndex=0`, `cellParaIndex=3` 내부 1x2 중첩 표의 두 번째 셀
- 기대 경로: `[(6, 0, 3), (0, 1, 0)]`

하지만 기존 `hit_test_native()`는 내부 TAC 표의 로컬 메타를 문서 루트 기준처럼 반환했다.

```json
{
  "parentParaIndex": 3,
  "controlIndex": 0,
  "cellPath": [
    { "controlIndex": 0, "cellIndex": 1, "cellParaIndex": 0 }
  ]
}
```

이 값이 Studio 입력 경로로 전달되면 `insertTextInCell(0, 3, 0, 1, 0, 0, text)`가 호출되어 루트 문단 3의 컨트롤 0을 찾다가 `컨트롤 인덱스 0 범위 초과`가 발생했다.

## 3. 수정

수정 파일:

- `src/document_core/queries/cursor_rect.rs`

`hit_test_native()`와 `get_cursor_rect_by_path_native()`의 렌더 트리 수집 단계에서 조상 표/셀 컨텍스트를 함께 전파하도록 보정했다.

핵심 보정:

1. `table_ctx_from_node()`
   - Table 노드가 셀 내부에 있고 `para_index/control_index`를 가진 경우, 이를 문서 루트 메타가 아니라 현재 셀의 `cellParaIndex`와 내부 표 `controlIndex`로 해석한다.
2. `cell_ctx_for_table_cell()`
   - TableCell 진입 시 현재 표 경로에 `cellIndex`, `cellParaIndex`, `textDirection`을 반영한다.
3. `effective_cell_context()`
   - TextRun 자체가 내부 표 로컬 경로만 가진 경우, 조상 traversal context가 더 깊으면 전체 경로를 보존한 traversal context를 우선 사용한다.
4. `get_cursor_rect_by_path_native()` 후속 보정
   - path 기반 삽입 후 커서 좌표 조회도 동일한 전체 `cellPath` 기준으로 TextRun을 찾도록 수정했다.

Studio TypeScript는 수정하지 않았다. Studio는 이미 `cellPath.length > 1`이면 `insertTextInCellByPath`를 호출하므로, Rust hit-test 반환값만 정상화하면 기존 경로로 해결된다.

## 4. 회귀 테스트

신규 테스트:

- `tests/issue_850_answer_sheet_name_hit_test.rs`

검증 항목:

- `exam_social.hwp` 성명 칸 hit-test가 `parentParaIndex=0`, `controlIndex=4`, `cellPath=[(4,0,3),(0,1,0)]`를 반환
- `exam_science.hwp` 성명 칸 hit-test가 `parentParaIndex=0`, `controlIndex=6`, `cellPath=[(6,0,3),(0,1,0)]`를 반환
- 두 문서 모두 `insert_text_in_cell_by_path()`로 `"홍"` 삽입 후 `get_text_in_cell_by_path()`로 확인
- 삽입 후 `get_cursor_rect_by_path()`가 전체 `cellPath`로 정상 좌표를 반환하는지 확인
- `exam_social.hwp` page 0에서 입력 루프용 `getPageOverlayImages()` JSON이 전체 `getPageLayerTree()` JSON으로 회귀하지 않도록 compact size 가드

## 5. 검증

### #850 신규 테스트

```bash
cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture
```

```text
test result: ok. 3 passed; 0 failed
```

### #717 기존 테스트 보존

```bash
cargo test --test issue_717_table_cell_hit_test -- --nocapture
```

```text
test result: ok. 3 passed; 0 failed
```

### 기존 중첩 표 path API

```bash
cargo test --lib test_task105_nested_table_path_api -- --nocapture
```

```text
test result: ok. 1 passed; 0 failed
```

### 전체 테스트

```bash
cargo test
```

첫 실행은 sandbox 네트워크 제한으로 `static.crates.io` DNS 조회에 실패했다. 승인 후 `web-sys v0.3.95`를 다운로드하여 재실행했고 전체 테스트가 통과했다. Stage 5 후속 수정 뒤에도 다시 전체 테스트를 실행했다.

```text
test result: ok. 1232 passed; 0 failed; 2 ignored
```

통합 테스트와 doc-test까지 모두 통과했다.

### WASM/브라우저 검증

`rhwp-studio`는 `../pkg/rhwp.js`, `../pkg/rhwp_bg.wasm`을 사용하므로 Rust 수정 후 WASM 산출물 재빌드가 필요했다. 기존 `pkg/`는 2026-05-08 빌드본이었다.

```bash
colima start
docker-compose run --rm wasm
```

최종 브라우저 검증 URL:

```text
http://localhost:7700/?url=/samples/exam_social.hwp&filename=exam_social.hwp&t=8502
```

검증 결과:

```json
{ "newLogs": [] }
```

2차 수정 후 `성명` 칸 입력에서 `컨트롤 인덱스 0 범위 초과`와 `getCursorRectByPath` warning이 새로 발생하지 않았다.

## 6. 기존 경고

전체 테스트 중 기존 warning 6건이 출력되었다.

- `src/renderer/equation/parser.rs`: duplicated attribute
- `src/renderer/layout/integration_tests.rs`: unnecessary parentheses
- `src/serializer/hwpx/field.rs`: non-snake-case test name
- `src/wasm_api/tests.rs`: non-snake-case test name 1건
- `src/wasm_api/tests.rs`: unused Result 2건

#850 수정과 직접 관련된 실패는 없다.

## 7. 산출물

| 영역 | 파일 |
|------|------|
| 수행 계획서 | `mydocs/plans/task_m100_850.md` |
| 구현 계획서 | `mydocs/plans/task_m100_850_impl.md` |
| Stage 1 보고서 | `mydocs/working/task_m100_850_stage1.md` |
| Stage 2 보고서 | `mydocs/working/task_m100_850_stage2.md` |
| Stage 3 보고서 | `mydocs/working/task_m100_850_stage3.md` |
| Stage 4 보고서 | `mydocs/working/task_m100_850_stage4.md` |
| Stage 5 보고서 | `mydocs/working/task_m100_850_stage5.md` |
| Stage 6 보고서 | `mydocs/working/task_m100_850_stage6.md` |
| 최종 보고서 | `mydocs/report/task_m100_850_report.md` |
| 본질 정정 | `src/document_core/queries/cursor_rect.rs` |
| 회귀 가드 | `tests/issue_850_answer_sheet_name_hit_test.rs` |

## 8. 성능 후속

Stage 5 이후 사용자 검증에서 입력 지연이 확인되어 Stage 6에서 별도 분석했다.

결론은 #850 정확성 수정이 새 비용을 직접 만든 것이 아니라, `v0.7.11`의 #717 이후 성명 칸이 nested path 입력 경로를 타게 되면서 기존 느린 경로가 노출된 것이다. `v0.7.10`에서는 해당 칸이 이 경로로 진입하지 않아 같은 성능 문제가 체감되지 않았다.

성능 병목:

- `getCursorRectByPath`가 매 입력마다 uncached page tree를 구성했다.
- Studio `PageRenderer`가 입력 렌더마다 `getPageLayerTree()`로 약 1.4MB JSON을 생성/파싱했다.
- 이미지 지연 재렌더 조건이 넓어 같은 image count에서도 타이머 상태를 불필요하게 건드렸다.

추가 수정:

- `getCursorRectByPath`와 `buildPageLayerTree`를 cached page tree 경로로 전환했다.
- 입력 루프용 `getPageOverlayImages(pageNum)` WASM API를 추가해 overlay image만 compact JSON으로 반환한다.
- Studio `PageRenderer`가 새 API를 우선 사용하고, 구버전 WASM에서만 `getPageLayerTree()` fallback을 사용하도록 했다.
- flow 렌더와 동일 image count에서는 지연 재렌더를 새로 예약하지 않도록 조건을 좁혔다.

측정 결과:

```text
getPageOverlayImages(0): 0.54~0.91ms, len=39
getPageLayerTree(0):     16.81ms, len=1,402,745
getCursorRectByPath:     cached 이후 0.15~0.18ms
insertTextInCellByPath:  0.12~0.35ms
```

브라우저 검증에서 `samples/exam_social.hwp` 성명 칸에 `홍길동`이 실제 셀 텍스트로 반영되었고, `컨트롤 인덱스 0 범위 초과`는 재현되지 않았다.

## 9. 결론

#850 회귀는 Studio 입력 라우터 문제가 아니라 Rust `hit_test_native()`가 상단 답안지 내부 TAC 표의 로컬 메타를 문서 루트 컨텍스트처럼 반환한 문제였다.

조상 표/셀 컨텍스트를 수집 단계에서 전파해 외곽 표 기준 `parentParaIndex/controlIndex`와 전체 `cellPath`를 복원했다. 또한 path 기반 삽입 후 커서 좌표 조회도 같은 경로 기준으로 동작하도록 보정했다. 이로써 `exam_social.hwp`, `exam_science.hwp`의 `성명` 입력칸이 기존 Studio path 기반 입력 API로 정상 처리된다.

후속 성능 정리로 nested path 입력 루프의 cached cursor 조회와 compact overlay 조회도 적용했다. 기존 #717 hit-test 회귀 테스트, WASM 빌드, 브라우저 입력 검증, 전체 `cargo test` 모두 통과했다.
