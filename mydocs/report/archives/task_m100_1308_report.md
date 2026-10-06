# Task #1308 완료 보고서

## 요약

Shift+Enter 강제 줄바꿈(`\n`)이 같은 문단 내부에서 다음 visual line으로 분리되지 않아,
강제 줄바꿈 뒤 줄에 문단 들여쓰기/내어쓰기 규칙이 적용되지 않는 문제를 보정했다.

추가로 TAC 수식, 일반 글자, 고정탭이 한 문단/한 줄에 섞인 경우 renderer 전용
marker 합성이 원본 콘텐츠 순서를 바꾸는 문제를 함께 보정했다.

후속 동작 테스트에서 확인된 커서 이동 회귀도 함께 보정했다. 강제 줄바꿈 뒤 TAC-only 줄의
첫 수식 앞 커서 위치가 첫 줄 y에 남아 오른쪽 방향키 이동 시 다음 줄 첫 수식 앞으로 진입하지
않는 것처럼 보이던 문제를 cursor rect 경로에서 해결했다.

추가로 문단 경계를 넘어 수식으로 시작하는 다음 문단에 진입할 때, 첫 수식 앞이 아니라 첫 수식
뒤로 커서가 이동하던 문제를 문서 트리 navigation 경로에서 함께 보정했다.

## 변경 내용

- 테스트 샘플
  - `samples/eq-002.hwp`
  - `samples/hwpx/eq-002.hwpx`
  - `pdf-large/hwpx/eq-002.pdf`

- `src/renderer/composer.rs`
  - 단일 `LINE_SEG` 범위 안에 중간 `\n`이 있는 경우 visual line을 분리하도록 변경
  - `\n` 앞 줄은 `has_line_break=true`로 유지
  - `\n` 뒤 줄은 `char_start`가 `\n` 다음 문자 위치를 가리키도록 생성
  - 끝의 `\n`은 빈 후속 줄을 중복 생성하지 않도록 처리
  - `synthesize_marker_paragraph()`가 원본 `control_text_positions` 순서를 덮어쓰지 않도록 적용 범위 축소
  - 원본 gap 분석이 TAC 위치를 `[0,0,2,2,4]`처럼 이미 분산한 문단은 marker 재합성을 건너뜀

- `src/renderer/composer/tests.rs`
  - 중간 강제 줄바꿈 분리 회귀 테스트 추가
  - trailing 강제 줄바꿈 중복 빈 줄 방지 테스트 추가

- `src/renderer/layout/paragraph_layout.rs`
  - 빈 text run의 TAC-only visual line도 일반 TextLine과 동일하게 문단 margin/indent 기준 x를 사용하도록 보정

- `src/document_core/queries/cursor_rect.rs`
  - inline control cursor stop의 y/height가 다른 visual line의 텍스트 stop을 잘못 재사용하지 않도록 보정
  - Shift+Enter 뒤 TAC-only 줄 첫 수식 앞 커서가 두 번째 visual line 위치에 표시되도록 처리

- `src/document_core/queries/doc_tree_nav.rs`
  - 문단 시작에 있는 non-textbox inline control을 문단 경계 진입 시 즉시 소비하지 않도록 변경
  - 문단 내부 오른쪽 이동에서는 기존처럼 inline control을 1글자 단위로 통과
  - 수식으로 시작하는 다음 문단에는 첫 수식 앞 `charOffset=0`으로 진입

- `tests/issue_1308_forced_break_hanging_indent.rs`
  - HWP/HWPX 강제 줄바꿈 뒤 TAC 수식 줄의 내어쓰기 적용 검증
  - HWP/HWPX TAC 수식/쉼표/고정탭/일반 글자 순서 보존 검증
  - HWP/HWPX 오른쪽 방향키 이동이 강제 줄바꿈 뒤 TAC-only 줄 첫 수식 앞으로 진입하는지 검증
  - HWP/HWPX 문단 경계 이동이 수식으로 시작하는 다음 문단의 첫 수식 앞에 진입하는지 검증

## 검증

| 항목 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --test issue_1308_forced_break_hanging_indent -- --nocapture` | 통과 |
| `cargo test test_compose_ --lib` | 통과 |
| `cargo test --lib` | 통과 |
| `cargo run --bin rhwp -- export-svg samples/eq-002.hwp -o output/poc/task1308/fixed-hwp --debug-overlay -p 0` | 통과 |
| `cargo run --bin rhwp -- export-svg samples/hwpx/eq-002.hwpx -o output/poc/task1308/fixed-hwpx --debug-overlay -p 0` | 통과 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과 |
| `pkg/` → `rhwp-studio/public/` WASM 동기화 | 완료 |
| rhwp-studio 메인테이너 동작 테스트 | 통과 |

회귀 테스트 결과:

```text
cargo test --test issue_1308_forced_break_hanging_indent -- --nocapture
8 passed; 0 failed

cargo test --lib
1599 passed; 0 failed; 6 ignored
```

SVG 확인:

- `output/poc/task1308/fixed-hwp/eq-002.svg`
- `output/poc/task1308/fixed-hwpx/eq-002.svg`
- 마지막 줄의 `{1} over {4} f(n)` 수식 시작 위치가 `x=113.386...`으로 복원
- 쉼표는 첫 두 TAC 수식 뒤 `x=167.586...`에 출력
- 강제 줄바꿈 뒤 첫 TAC 수식 앞 커서 위치가 `x=166.7`, `y=251.9`로 두 번째 visual line에 배치됨
- 문단 0.0 끝에서 오른쪽 이동 시 문단 0.1의 첫 수식 앞 `charOffset=0`으로 진입

WASM 동기화 확인:

```text
pkg/rhwp.js == rhwp-studio/public/rhwp.js
pkg/rhwp_bg.wasm == rhwp-studio/public/rhwp_bg.wasm
pkg/rhwp.d.ts == rhwp-studio/public/rhwp.d.ts
pkg/rhwp_bg.wasm.d.ts == rhwp-studio/public/rhwp_bg.wasm.d.ts
```

## 판정

Rust/SVG/WASM 검증 기준과 rhwp-studio 메인테이너 동작 테스트를 모두 통과했다.
