# Task #850 구현 계획서

수행 계획서: [`task_m100_850.md`](./task_m100_850.md)

## 목표

`rhwp-studio`에서 `samples/exam_social.hwp`, `samples/exam_science.hwp` 1쪽 상단 답안지 `성명` 칸을 클릭한 뒤 이름을 입력할 때 발생하는 `컨트롤 인덱스 0 범위 초과` 회귀를 정정한다.

핵심 목표는 `hit_test_native()`가 중첩 표 내부 빈 입력칸에서도 문서 루트 기준의 외곽 표 컨텍스트와 전체 `cellPath`를 유지하도록 만드는 것이다.

## 변경 후보

### 옵션 A — `collect_runs()`의 table meta 전파 보정

메타가 없는 중첩 `RenderNodeType::Table`을 만났을 때 `current_table_meta`를 `None`으로 끊지 않고 유지한다.

- 장점: 변경량이 작고 회귀 지점과 직접 대응한다.
- 단점: 중첩 표 자체의 table_id와 외곽 표 meta가 섞일 수 있어 `cell_bboxes` 보정까지 함께 확인해야 한다.

### 옵션 B — `cell_bboxes` 보정 기준을 `CellContext` 전체 경로 기준으로 정밀화

현재 보정은 같은 table_id 안에서 `ctx.innermost().cell_index == cb.cell_index`만 확인한다. 중첩 표에서 같은 cell index가 반복될 수 있으므로 `table_id`, 외곽 path, innermost path를 함께 사용해 `CellBboxInfo.cell_context`를 채운다.

- 장점: `cellPath`를 public 반환 필드의 권위 데이터로 삼을 수 있다.
- 단점: 빈 셀처럼 TextRun이 없는 셀은 같은 표의 다른 run 템플릿에 의존하므로 보정 규칙을 신중히 유지해야 한다.

### 옵션 C — `format_hit()` 직전 public 필드 정규화 helper 추가

`CellContext`가 있으면 항상 `ctx.parent_para_index`와 `ctx.path[0]`을 public `parentParaIndex/controlIndex/cellIndex/cellParaIndex`로 반환하도록 helper를 만들고, TextRun hit와 cell bbox hit 양쪽에서 같은 helper를 사용한다.

- 장점: Studio 입력 분기와 WASM API가 기대하는 public 필드 의미를 한 곳에서 고정할 수 있다.
- 단점: 근본 원인인 잘못된 `CellContext` 생성은 별도 정정이 필요하다.

## 권장 방향

**옵션 A + B를 최소 범위로 적용하고, 옵션 C는 중복 문자열 정리 수준에서만 사용한다.**

이번 결함은 중첩 표의 실제 경로가 `CellContext.path`에 보존되어야 하는 문제다. 따라서 Studio TypeScript 입력 경로를 우회 수정하지 않고, Rust `hit_test_native()` 반환값을 문서 구조와 맞추는 방향이 맞다.

## 단계별 구현

### Stage 1 — RED 회귀 테스트 작성

신규 테스트 파일:

- `tests/issue_850_answer_sheet_name_hit_test.rs`

검증 항목:

1. `exam_social.hwp` 1쪽 상단 `성명` 입력칸 hit-test
   - 클릭 좌표: 상단 답안지 `성명` 오른쪽 빈칸 내부 좌표
   - 현재 기대 실패: `parentParaIndex=3`, `controlIndex=0`, `cellPath.length=1`
   - 최종 기대: `parentParaIndex=0`, `controlIndex=4`, `cellPath.length=2`
2. `exam_science.hwp` 1쪽 상단 `성명` 입력칸 hit-test
   - 클릭 좌표: 상단 답안지 `성명` 오른쪽 빈칸 내부 좌표
   - 현재 기대 실패: `parentParaIndex=3`, `controlIndex=0`, `cellPath.length=1`
   - 최종 기대: `parentParaIndex=0`, `controlIndex=6`, `cellPath.length=2`
3. 두 문서 모두 `insert_text_in_cell_by_path()`로 `"홍"` 삽입 성공

실행 명령:

```bash
cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture
```

완료 조건:

- 새 테스트가 현재 `upstream/devel` 기준으로 실패한다.
- 실패 메시지가 #850의 실제 결함인 잘못된 `parentParaIndex/controlIndex/cellPath` 또는 삽입 실패를 가리킨다.
- Stage 1 완료보고서 `mydocs/working/task_m100_850_stage1.md` 작성 후 승인 요청.

### Stage 2 — hit-test 경로 보존 로직 정정

대상 파일:

- `src/document_core/queries/cursor_rect.rs`

작업 내용:

1. `collect_runs()`에서 메타 없는 중첩 `Table` 진입 시 외곽 표의 루트 메타와 `table_id` 관계를 끊지 않도록 보정한다.
2. `cell_bboxes` 보정에서 같은 table_id의 TextRun 템플릿을 사용할 때, 내부 표 로컬 셀 인덱스만 public 필드로 승격하지 않도록 한다.
3. `CellContext.path[0]`이 public `controlIndex/cellIndex/cellParaIndex`의 권위가 되도록 `format_hit()` 및 빈 셀 bbox 반환 경로를 확인한다.
4. 기존 `issue_717_table_cell_hit_test`의 nested header 기대값 `[(0,0,0), (1,1,0)]`은 유지한다.

검증 명령:

```bash
cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture
cargo test --test issue_717_table_cell_hit_test -- --nocapture
```

완료 조건:

- #850 신규 테스트 GREEN
- #717 기존 테스트 GREEN
- Stage 2 완료보고서 `mydocs/working/task_m100_850_stage2.md` 작성 후 승인 요청.

### Stage 3 — Studio 입력 경로 기준 검증

대상:

- Rust WASM API의 `insert_text_in_cell_by_path()`
- `rhwp-studio/src/engine/input-handler-text.ts`
- `rhwp-studio/src/engine/command.ts`

작업 내용:

1. Rust hit-test 반환값이 Studio의 기존 분기 조건 `(pos.cellPath?.length ?? 0) > 1`을 만족하는지 확인한다.
2. TypeScript 입력 경로 수정 없이 해결되는지 확인한다.
3. 필요 시에만 Studio 쪽 방어 로직을 추가한다. 단, 우선순위는 Rust hit-test 정정이다.

검증 명령:

```bash
cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture
cargo test --lib wasm_api::tests -- --nocapture
```

필요 시 rhwp-studio E2E:

```bash
cd rhwp-studio
npx vite --host 0.0.0.0 --port 7700
node e2e/text-flow.test.mjs
```

완료 조건:

- `insert_text_in_cell_by_path()` 삽입 성공이 테스트로 고정된다.
- Studio TypeScript 변경이 필요 없으면 변경하지 않는다.
- 변경이 필요한 경우 별도 근거를 완료보고서에 명시한다.
- Stage 3 완료보고서 `mydocs/working/task_m100_850_stage3.md` 작성 후 승인 요청.

### Stage 4 — 회귀 검증

검증 범위:

1. #850 신규 테스트
2. #717 기존 hit-test 테스트
3. 관련 WASM API 테스트
4. 전체 cargo 테스트

실행 명령:

```bash
cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture
cargo test --test issue_717_table_cell_hit_test -- --nocapture
cargo test
```

추가 확인:

- `samples/exam_social.hwp`의 본문 자료 표 빈 영역 hit-test가 기존 기대값을 유지한다.
- `samples/exam_social.hwp`, `samples/exam_science.hwp` 상단 답안지 영역의 public `controlIndex`가 각각 `4`, `6`으로 유지된다.

완료 조건:

- 관련 테스트 전체 GREEN
- 전체 `cargo test` GREEN 또는 실패 시 #850과 무관한 기존 실패로 분류
- Stage 4 완료보고서 `mydocs/working/task_m100_850_stage4.md` 작성 후 승인 요청.

### Stage 5 — 최종 정리

작업 내용:

1. 최종 결과보고서 작성: `mydocs/report/task_m100_850_report.md`
2. 오늘할일 문서 갱신: `mydocs/orders/20260512.md`
3. 최종 `git status` 확인
4. 필요 시 커밋 준비 범위 정리

최종 보고서 포함 내용:

- 회귀 원인
- 수정 방식
- 테스트 결과
- #717 보존 여부
- 남은 위험 또는 후속 권장 사항

완료 조건:

- 최종 보고서 승인 요청
- 작업 브랜치 `local/task850`에 커밋 가능한 상태 정리

### Stage 6 — 성능 후속 정리

작업 내용:

1. `getCursorRectByPath`가 입력마다 uncached page tree를 다시 만드는지 확인한다.
2. `rhwp-studio` 입력 렌더 루프가 매 입력마다 전체 `getPageLayerTree` JSON을 요청하는지 확인한다.
3. 불필요한 지연 재렌더 예약 또는 취소가 발생하지 않도록 `PageRenderer` 재시도 조건을 좁힌다.
4. WASM API와 Studio bridge에 입력 루프용 compact overlay image 조회 경로를 추가한다.

검증 명령:

```bash
cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture
cargo test --test issue_717_table_cell_hit_test -- --nocapture
cargo test --lib test_task105_nested_table_path_api -- --nocapture
cd rhwp-studio && npm run build
docker-compose run --rm wasm
```

완료 조건:

- `getCursorRectByPath`가 캐시된 page tree 경로를 사용한다.
- 입력 루프에서 `getPageLayerTree` 1.4MB JSON 대신 compact overlay JSON을 사용한다.
- 성명 칸 입력 브라우저 검증에서 `홍길동`이 반영되고 `컨트롤 인덱스 0 범위 초과`가 새로 발생하지 않는다.

## 예상 변경 파일

| 파일 | 변경 종류 | 목적 |
|------|----------|------|
| `src/document_core/queries/cursor_rect.rs` | 수정 | hit-test 중첩 표 경로 보존 |
| `tests/issue_850_answer_sheet_name_hit_test.rs` | 신규 | #850 RED/GREEN 회귀 테스트 |
| `mydocs/plans/task_m100_850.md` | 신규 | 수행 계획서 |
| `mydocs/plans/task_m100_850_impl.md` | 신규 | 구현 계획서 |
| `mydocs/working/task_m100_850_stage{1..4}.md` | 신규 | 단계별 완료보고서 |
| `mydocs/report/task_m100_850_report.md` | 신규 | 최종 결과보고서 |
| `mydocs/orders/20260512.md` | 수정 | 작업 상태 갱신 |

## 위험 관리

- `hit_test_native()`는 본문, 표, 글상자, inline shape 클릭을 모두 다루므로 변경 범위를 `CellContext`와 `cell_bboxes` 경로 보정에 한정한다.
- Studio TypeScript는 Rust 반환값이 올바르면 기존 `cellPath.length > 1` 경로로 동작하므로, TypeScript 수정은 최후 수단으로 둔다.
- #717 테스트를 Stage 2부터 매번 같이 실행해 기존 빈 셀 hit-test 정정을 되돌리지 않는다.

## 진행 조건

본 구현 계획서 승인 후 Stage 1 RED 회귀 테스트 작성을 시작한다.
