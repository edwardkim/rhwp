# Task #484: 구현 계획서

## 1단계: `Paragraph::utf16_pos_to_char_idx` 메서드 신설

### 목표

`src/model/paragraph.rs` 의 `impl Paragraph` 블록에 UTF-16 단위 위치를 codepoint 인덱스로 정규화하는 인스턴스 메서드를 추가한다.

### 작업 내용

1. `impl Paragraph` 블록 내 `control_text_positions` (PR #390 으로 추가, line 802 닫는 brace) 다음에 메서드 삽입
2. 시그니처: `pub fn utf16_pos_to_char_idx(&self, utf16_pos: u32) -> usize`
3. 본체: `self.char_offsets.iter().position(|&off| off >= utf16_pos).unwrap_or(self.char_offsets.len())` — helpers 의 free function 과 동일 알고리즘
4. docstring — 사용 의도 (`char_shapes[i].start_pos` / `line_segs[i].text_start` 정규화), 알고리즘 본체 자체 보유 사유 (시그니처 호환성), `# Returns` 섹션

### 산출물

- `cargo build --tests` 성공
- 새로운 외부 가시 메서드 `Paragraph::utf16_pos_to_char_idx` 노출

## 2단계: 단위 테스트 추가

### 목표

`src/model/paragraph/tests.rs` 에 `Paragraph::utf16_pos_to_char_idx` 의 boundary 케이스를 검증하는 테스트 6건을 추가한다.

### 작업 내용

1. `test_utf16_pos_to_char_idx_empty_offsets` — `char_offsets.is_empty()` → `unwrap_or(0)` 분기
2. `test_utf16_pos_to_char_idx_zero_returns_first` — `utf16_pos = 0`, `offsets[0] = 0 >= 0` → 인덱스 0
3. `test_utf16_pos_to_char_idx_exact_match` — `offsets` 안의 정확한 값일 때 해당 인덱스
4. `test_utf16_pos_to_char_idx_between_offsets` — `offsets` 사이 값일 때 첫 entry >= 인덱스 (SMP `🎉` 가 들어간 `[0, 1, 3]` 케이스)
5. `test_utf16_pos_to_char_idx_beyond_end_returns_len` — `utf16_pos > 모든 entry` → `char_offsets.len()` fallback
6. `test_utf16_pos_to_char_idx_surrogate_pair_midpoint` — surrogate pair 의 low half 위치를 다음 codepoint 시작 위치로 정규화 (`"🎉A"`, offsets `[0, 2]`)

### 산출물

- `cargo test --lib model::paragraph::tests` 통과 (PR #390 6건 + 본 PR 6건 = 신규 12건 누적, 회귀 39건 + 신규 6건 = 45건 통과)

## 3단계: 품질 검증

### 작업 내용

```bash
cargo fmt -- --check src/model/paragraph.rs src/model/paragraph/tests.rs    # 변경 파일만
cargo test --lib    # 회귀 + 신규
cargo clippy --all-targets 2>&1 | grep "src/model/paragraph"    # 우리 변경 파일만
```

### 통과 기준

- 변경 파일 (`src/model/paragraph.rs`, `src/model/paragraph/tests.rs`) rustfmt / clippy 깨끗
- 신규 6건 테스트 모두 통과 + 회귀 통과
- base branch (`upstream/devel`) 의 기존 lint / fmt warning (총 43건, `emf/tests.rs` / `serializer/hwpx/field.rs` / `wasm_api/tests.rs`) 은 별도 — 우리 변경 무관

## 변경 파일 요약

| 파일 | 변경 형태 | 라인 수 (대략) |
|---|---|---|
| `src/model/paragraph.rs` | 새 메서드 추가 (`impl Paragraph` 내) | +24 |
| `src/model/paragraph/tests.rs` | 신규 단위 테스트 6건 | +75 |
| `src/document_core/helpers.rs` | 변경 없음 | 0 |

순 알고리즘 라인 수 변동: +1 (1줄짜리 알고리즘이 model 메서드로 추가, helpers free function 그대로). 코드 중복은 1줄 — silent drift 위험 trivial.
