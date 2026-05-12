# Task #850 Stage 5 완료 보고서

## 단계 목표

웹서버에서 `samples/exam_social.hwp` 성명 칸 입력을 직접 검증하고, 네이티브 테스트와 브라우저 동작 사이의 차이를 확인한다.

## 추가 확인된 현상

초기 커밋 후 `rhwp-studio`를 실행했을 때 브라우저에서는 여전히 다음 오류가 재현되었다.

```text
Uncaught 렌더링 오류: 컨트롤 인덱스 0 범위 초과
```

원인은 코드 수정 누락이 아니라 `rhwp-studio`가 사용하는 `pkg/rhwp.js`, `pkg/rhwp_bg.wasm`이 2026-05-08 빌드본으로 남아 있었기 때문이다. 네이티브 테스트는 최신 Rust 코드를 사용했지만, 웹서버는 `rhwp-studio/vite.config.ts`의 `@wasm -> ../pkg` 별칭을 통해 기존 WASM 산출물을 계속 사용했다.

## WASM 재빌드

저장소 지침에 따라 WASM은 Docker 경로로 빌드했다.

현재 로컬 환경은 `docker compose` 플러그인이 없고 `docker-compose` 명령만 사용 가능했다. 또한 Colima가 정지되어 있어 먼저 Colima를 시작한 뒤 빌드했다.

```bash
colima start
docker-compose run --rm wasm
```

1차 WASM 재빌드 후 `pkg/` 산출물은 다음 시각으로 갱신되었다.

```text
pkg/rhwp.js       2026-05-12 18:14
pkg/rhwp_bg.wasm  2026-05-12 18:14
```

## 2차 결함

최신 WASM으로 브라우저에서 다시 확인하니 기존 `컨트롤 인덱스 0 범위 초과`는 사라졌다. 다만 입력 직후 다음 warning이 새로 확인되었다.

```text
[CursorState] updateRect 실패 → hitTest 폴백
렌더링 오류: 경로 기반 커서 위치를 찾을 수 없습니다
path=[{"controlIndex":4,"cellIndex":0,"cellParaIndex":3},{"controlIndex":0,"cellIndex":1,"cellParaIndex":0}]
```

원인은 `getCursorRectByPath`가 렌더 트리 `TextRun.cell_context`를 직접 비교했기 때문이다. `hitTest`는 Stage 2에서 traversal context로 전체 중첩 경로를 복원했지만, `getCursorRectByPath`는 동일 보정을 하지 않아 내부 표 TextRun의 로컬 경로만 보고 전체 path 매칭에 실패했다.

## 수정

수정 파일:

- `src/document_core/queries/cursor_rect.rs`
- `tests/issue_850_answer_sheet_name_hit_test.rs`

수정 내용:

- `get_cursor_rect_by_path_native()`에도 `hit_test_native()`와 같은 조상 표/셀 traversal context 보정을 적용했다.
- TextRun 자체가 로컬 `cell_context`만 가진 경우에도 traversal context가 더 깊으면 전체 `cellPath`를 기준으로 매칭한다.
- #850 회귀 테스트에 `insert_text_in_cell_by_path()` 이후 `get_cursor_rect_by_path()` 성공 검증을 추가했다.

## 검증

### 네이티브 회귀 테스트

```bash
cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture
```

결과:

```text
test result: ok. 2 passed; 0 failed
```

### 기존 회귀 테스트

```bash
cargo test --test issue_717_table_cell_hit_test -- --nocapture
cargo test --lib test_task105_nested_table_path_api -- --nocapture
```

결과:

```text
#717: 3 passed
task105: 1 passed
```

### WASM 재빌드

```bash
docker-compose run --rm wasm
```

결과:

```text
Finished `release` profile [optimized]
Your wasm pkg is ready to publish at /app/pkg.
```

### 브라우저 검증

URL:

```text
http://localhost:7700/?url=/samples/exam_social.hwp&filename=exam_social.hwp&t=8502
```

검증 플로우:

1. `exam_social.hwp` 로드
2. 1쪽 상단 답안지 `성명` 입력칸 클릭
3. 한글 1자 입력
4. 브라우저 console error/warn 신규 발생 확인

결과:

```json
{
  "newLogs": []
}
```

기존 탭에 남아 있던 warning 1건은 1차 WASM 빌드 전 검증 로그였고, 2차 수정 후 새 입력에서는 `컨트롤 인덱스 0 범위 초과`와 `getCursorRectByPath` warning 모두 새로 발생하지 않았다.

### 전체 테스트

```bash
cargo test
```

결과:

```text
test result: ok. 1232 passed; 0 failed; 2 ignored
```

통합 테스트와 doc-test까지 모두 통과했다.

## 결론

웹서버에서 남아 있던 최초 오류는 stale WASM 산출물 사용이 원인이었다. WASM 갱신 후 원래 오류는 사라졌고, 이어 드러난 path 기반 커서 좌표 조회 실패까지 같은 중첩 경로 보정 방식으로 수정했다.

최종적으로 네이티브 테스트, WASM 빌드, 브라우저 입력 검증, 전체 `cargo test`가 모두 통과했다.
