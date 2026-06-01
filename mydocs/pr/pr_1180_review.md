# PR #1180 검토 — task 1179: Rust 테스트 경고 정리

- **작성일**: 2026-05-31
- **PR**: #1180 (OPEN)
- **컨트리뷰터**: @jangster77 (반복 기여자 — #1177/#1178/#1180 누적)
- **연결 이슈**: Closes #1179
- **base/head**: `devel` ← `task_m100_1179`
- **mergeable**: MERGEABLE / mergeStateStatus: **BEHIND** (충돌 없음, head 가 devel 보다 뒤처짐)
- **규모**: 6 파일, +78 / −6 (코드 4 파일, 문서 2 파일)
- **마일스톤**: v1.0.0 / 라벨: enhancement

## 1. PR 정보 확인

`cargo test renderer::height_cursor::tests::compact_endnote -- --nocapture` 실행 중
출력되던 Rust warning 6개를 정리. 런타임 동작 변경 없음(테스트/진단 코드 한정).

### 변경 파일 (merge-base `53df8f5c` 기준 PR 고유 변경)

| 파일 | 변경 내용 | 대응 경고 |
|------|----------|----------|
| `src/renderer/equation/parser.rs` | 중복 `#[test]` attribute 1개 제거 (`test_hwpeq_inf_remains_symbol`) | 중복 attribute |
| `src/renderer/layout/integration_tests.rs` | `let expected_gap = (… )` 불필요한 괄호 제거 | `unused_parens` |
| `src/serializer/hwpx/field.rs` | `footnote_emits_autoNum` → `footnote_emits_auto_num` | `non_snake_case` |
| `src/wasm_api/tests.rs` | ① `test_merge_then_control_layout_has_colSpan` → `…_has_col_span` (non_snake_case)<br>② `insert_text_native(...)` → `.unwrap()` (unused `Result`)<br>③ `convert_to_editable_native()` → `.unwrap()` (unused `Result`) | `non_snake_case` ×1, `unused_must_use` ×2 |

> 비고: `git diff --name-only origin/devel..pr` 는 65 파일로 보이나, 이는 PR head 가 devel 보다
> BEHIND 여서 그 사이 머지된 #1177(Task #1151) 변경분이 역방향 차이로 나타나기 때문이다.
> merge-base 기준 PR 순수 변경분은 위 6 파일이 전부이며 충돌 없이 머지된다.

## 2. 검토 항목

### 정합성
- [x] 변경이 본문 설명과 1:1 일치 (warning 6개 ↔ 변경 6건)
- [x] 함수명 변경 대상이 **테스트 함수**이며 외부 호출 참조 없음 (grep 확인: 정의부만 존재)
- [x] `.unwrap()` 추가가 테스트 의미를 바꾸지 않음 (실패 시 panic = 테스트 실패, 기존 묵시 무시보다 엄격)
- [x] 런타임/직렬화/렌더 동작 경로 변경 없음

### 위험
- 없음. 테스트 코드 한정, non-public 함수명만 변경.

### 검증 계획 (4단계 — 빌드/테스트/clippy/fmt)
1. `cargo build` (네이티브)
2. `cargo test --tests` (통합 테스트 회귀 — feedback_push_full_test_required 준수)
3. `cargo clippy` (대상 warning 소거 확인)
4. `cargo fmt --all --check`

## 3. 판단 (예정)

검증 통과 시 **머지** 권고. 결과는 `pr_1180_report.md` 에 기록.
