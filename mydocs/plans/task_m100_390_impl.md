# Task #390: 구현 계획서

## 1단계: `Paragraph::control_text_positions` 메서드 신설

### 목표

`src/model/paragraph.rs` 의 `impl Paragraph` 블록에 인라인 컨트롤의 character 위치를 반환하는 인스턴스 메서드를 추가한다.

### 작업 내용

1. `impl Paragraph` 블록 내 `char_shape_id_at` 다음 (line 716 부근) 에 메서드 삽입
2. 시그니처: `pub fn control_text_positions(&self) -> Vec<usize>`
3. 본체: 기존 `helpers::find_control_text_positions` 의 알고리즘 그대로 이식 (`char_offsets` 갭 분석, `total_controls == 0` / `offsets.is_empty()` / 일반 분기 3 케이스)
4. docstring — 사용 의도 (인라인 컨트롤의 character offset 복원), 8 UTF-16 코드 유닛 갭 명시, returns 문서화

### 산출물

- `cargo build` 성공
- 새로운 외부 가시 메서드 `Paragraph::control_text_positions` 노출

## 2단계: helpers thin wrapper 전환

### 목표

`src/document_core/helpers.rs` 의 `find_control_text_positions` 본체를 신설 메서드 위임으로 교체한다.

### 작업 내용

1. 함수 본체 (~60 줄) 를 `para.control_text_positions()` 한 줄로 교체
2. `pub(crate)` 가시성 유지 (외부 가시성 변화 없음)
3. docstring 유지 (의미 동일)
4. 모든 내부 caller (helpers.rs:21, 43, 86, 101 + 외부 22 라인) 변경 없음 — wrapper 가 그대로 동작

### 산출물

- 동일 동작 보장 (단위 알고리즘 변경 없음)
- 코드 중복 없음 (single source-of-truth = `Paragraph::control_text_positions`)

## 3단계: 단위 테스트 추가

### 목표

`src/model/paragraph/tests.rs` 에 `Paragraph::control_text_positions` 의 알고리즘 분기를 검증하는 테스트 케이스를 추가한다.

### 작업 내용

1. `test_control_text_positions_empty` — `controls.is_empty()` → 빈 벡터 분기
2. `test_control_text_positions_no_offsets_inline_sequential` — `char_offsets.is_empty()` 일 때 인라인 컨트롤 (Table) 순차 배치 분기
3. `test_control_text_positions_no_offsets_non_inline_skipped` — fallback 분기의 `else` 경로 cover (비인라인 `Bookmark` 는 pos 증가 안 함)
4. `test_control_text_positions_gap_between_chars` — 일반 분기, 'AB' 사이 인라인 Table 1개의 갭 분석 (offsets `[0, 9]` → position `1`)
5. `test_control_text_positions_gap_before` — 첫 문자 이전 갭, 'A' 앞에 인라인 Table 1개 (offsets `[8]` → position `0`)
6. `test_control_text_positions_surrogate_pair_char_width` — surrogate pair 문자 (`'🎉'`, U+1F389) 의 UTF-16 width=2 분기 검증 (boundary 테스트 — controls 2 개로 width=1 버그 시 결과 차이 발생)

### 산출물

- `cargo test --lib model::paragraph::tests` 통과 (신규 4 케이스)
- 전체 회귀 테스트 통과 (`cargo test`)

## 4단계: 품질 검증

### 작업 내용

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

### 통과 기준

- 변경 파일 (`src/model/paragraph.rs`, `src/document_core/helpers.rs`, `src/model/paragraph/tests.rs`) rustfmt 깨끗
- 변경 파일 clippy 경고 0건
- 1016 baseline + 6 신규 단위 테스트 모두 통과 (`cargo test --lib` 1022)

## 변경 파일 요약

| 파일 | 변경 형태 | 라인 수 (대략) |
|---|---|---|
| `src/model/paragraph.rs` | 새 메서드 추가 (`impl Paragraph` 내) | +85 |
| `src/document_core/helpers.rs` | 본체 → wrapper 위임 | -55 / +6 |
| `src/model/paragraph/tests.rs` | 신규 단위 테스트 6건 + 모듈 use Bookmark/Table | +95 |

순 알고리즘 라인 수 변동 거의 없음 (이동 + 위임).
