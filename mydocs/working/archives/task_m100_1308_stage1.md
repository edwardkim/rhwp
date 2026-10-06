# Task #1308 Stage 1 - Shift+Enter 강제 줄바꿈 compose 분리

## 목적

Shift+Enter로 삽입된 강제 줄바꿈(`\n`) 뒤 텍스트가 같은 문단의 다음 visual line으로 배치되도록
composer 경로를 보정했다.

## 테스트 샘플

- `samples/eq-002.hwp`
- `samples/hwpx/eq-002.hwpx`
- `pdf-large/hwpx/eq-002.pdf`

## 조사 결과

rhwp-studio의 입력 경로는 정상적으로 같은 문단에 `\n`을 삽입한다.

- `rhwp-studio/src/engine/input-handler-keyboard.ts`
  - `Shift+Enter` → `InsertLineBreakCommand`
- `rhwp-studio/src/engine/command.ts`
  - `InsertLineBreakCommand` → `doInsertText(..., '\n')`

따라서 문단 분할 명령 문제가 아니라, Rust renderer/composer가 `\n`을 visual line으로 분리하는지의 문제다.

## 원인

`src/renderer/composer.rs`의 `compose_lines()` 일반 경로가 중간 `\n`을 별도 줄로 나누지 않았다.

기존 처리:

```rust
let has_line_break = line_text.ends_with('\n');
let line_text = if has_line_break {
    line_text.trim_end_matches('\n').to_string()
} else {
    line_text
};
```

이 방식은 `line_text` 끝의 `\n`만 처리한다. 단일 `LINE_SEG` 안에 `가나\n다라`처럼 중간 강제
줄바꿈이 있으면 `가나\n다라`가 하나의 `ComposedLine`으로 유지된다.

그 결과 renderer의 줄별 들여쓰기/내어쓰기 계산이 `다라`를 후속 줄로 볼 수 없다.

추가로, `samples/eq-002`의 마지막 줄에서는 TAC 수식, 일반 글자, 고정탭의 순서가 깨졌다.

문단 0.6 원본 구조:

```text
text = ", \t\t ㉠"
controls = [
  "{1} over {4} f(n)",
  "=2",
  "f(n)",
  "=8",
  "cdots cdots",
]
raw control_text_positions = [0, 0, 2, 2, 4]
```

즉 원본 `char_offsets`는 이미 다음 순서를 표현한다.

```text
수식0, 수식1, ", ", 수식2, 수식3, "\t\t", 수식4, " ㉠"
```

하지만 renderer 전용 `synthesize_marker_paragraph()`가 `\u{FFFC}` marker를 재합성하면서
TAC 위치를 `(9, 11, 11, 11, 11)`로 바꾸었다. 그 결과 첫 수식이 쉼표/탭 뒤로 밀리고,
한컴 편집자가 의도한 콘텐츠 스트림 순서가 깨졌다.

커서 이동 경로에서도 별도 문제가 확인되었다. 문단 0.4의 두 번째 visual line은 텍스트 없이
TAC 수식만 있는 줄이다. `src/document_core/queries/cursor_rect.rs`의 inline control cursor stop
생성 경로가 가까운 앞쪽 텍스트 stop의 y/height를 줄 metric으로 재사용하면서, 두 번째 줄 첫 수식
앞 커서(`charOffset=4`)가 x는 두 번째 줄 위치를 가리키지만 y는 첫 번째 줄에 남았다.
그 결과 첫 글자부터 오른쪽 방향키로 이동할 때 다음 줄 첫 번째 수식 앞 커서 위치가 보이지 않고
다음 글자로 건너뛰는 것처럼 동작했다.

문단 경계를 넘어 다음 문단으로 이동하는 경로에서도 별도 문제가 확인되었다. 문단 0.1은 수식으로
시작하지만 `navigate_to_para_start()`가 문단 시작 위치의 non-textbox inline control을 곧바로
1글자 소비한 것으로 처리했다. 따라서 문단 0.0 끝에서 오른쪽 방향키로 문단 0.1에 진입하면
첫 수식 앞(`charOffset=0`)이 아니라 첫 수식 뒤(`charOffset=1`)로 커서가 이동했다.

## 변경 내용

- `src/renderer/composer.rs`
  - `compose_lines()` 일반 경로에서 `line_text`를 `\n` 기준으로 visual line 조각으로 분리
  - `\n` 앞 조각은 `has_line_break=true`
  - `\n` 뒤 조각은 새 `ComposedLine`으로 생성
  - 각 조각의 `char_start`는 원래 paragraph char index를 유지
  - trailing `\n`은 빈 후속 줄을 중복 생성하지 않음
  - `synthesize_marker_paragraph()` 적용 범위를 축소
  - 원본 `control_text_positions`가 이미 컨트롤을 텍스트 중간/뒤 위치로 분산해 주는 경우
    marker를 재합성하지 않도록 처리
  - TAC 수식 + 일반 글자 + 고정탭 혼합 문단의 원본 순서를 보존

- `src/renderer/composer/tests.rs`
  - `test_compose_internal_forced_line_break_splits_visual_lines`
  - `test_compose_trailing_forced_line_break_keeps_single_marked_line`

- `src/document_core/queries/cursor_rect.rs`
  - inline control cursor stop의 y/height를 계산할 때 가까운 텍스트 stop이 해당 control bbox의
    세로 범위에 포함되는 경우에만 재사용
  - 텍스트 stop과 control bbox가 다른 visual line에 있으면 control bbox 자체의 y/height를 사용
  - Shift+Enter 뒤 TAC-only visual line의 첫 수식 앞 커서가 실제 두 번째 줄 y에 표시되도록 보정

- `src/document_core/queries/doc_tree_nav.rs`
  - 문단 시작으로 진입할 때 수식/그림/표 같은 inline control을 즉시 소비하지 않도록 보정
  - non-textbox inline control은 문단 내부 오른쪽 이동에서 1글자처럼 소비하되, 문단 경계 진입 시에는
    먼저 `charOffset=0`에 멈추도록 처리
  - 수식으로 시작하는 문단에 이전 문단 끝에서 진입할 때 첫 수식 앞 커서 위치를 보존

- `tests/issue_1308_forced_break_hanging_indent.rs`
  - HWP/HWPX 강제 줄바꿈 뒤 TAC 수식 줄의 내어쓰기 적용 회귀 테스트 추가
  - HWP/HWPX TAC 수식/쉼표/고정탭/일반 글자 순서 보존 회귀 테스트 추가
  - HWP/HWPX 오른쪽 방향키 이동이 강제 줄바꿈 뒤 TAC-only 줄 첫 수식 앞으로 진입하는지 검증
  - HWP/HWPX 문단 경계를 넘어 수식으로 시작하는 다음 문단에 진입할 때 첫 수식 앞에 멈추는지 검증

## 검증

```bash
cargo fmt --all -- --check
cargo test --test issue_1308_forced_break_hanging_indent -- --nocapture
cargo test test_compose_ --lib
cargo test --lib
```

결과:

- `cargo fmt --all -- --check`: 통과
- `cargo test --test issue_1308_forced_break_hanging_indent -- --nocapture`: 8 passed
- `cargo test test_compose_ --lib`: 10 passed
- `cargo test --lib`: 1599 passed, 0 failed, 6 ignored

SVG 산출:

```bash
cargo run --bin rhwp -- export-svg samples/eq-002.hwp -o output/poc/task1308/fixed-hwp --debug-overlay -p 0
cargo run --bin rhwp -- export-svg samples/hwpx/eq-002.hwpx -o output/poc/task1308/fixed-hwpx --debug-overlay -p 0
```

확인:

- HWP: `output/poc/task1308/fixed-hwp/eq-002.svg`
- HWPX: `output/poc/task1308/fixed-hwpx/eq-002.svg`
- 마지막 줄의 `{1} over {4} f(n)` 수식 시작 위치가 `x=113.386...`으로 복원
- 쉼표는 첫 두 TAC 수식 뒤 `x=167.586...`에 출력
- rhwp-studio 메인테이너 동작 테스트: 통과
  - 강제 줄바꿈 뒤 TAC-only 줄 첫 수식 앞 커서 이동 정상
  - 문단 경계에서 수식으로 시작하는 다음 문단의 첫 수식 앞 커서 이동 정상

## 판정

컴포저 단계에서 Shift+Enter 후속 줄이 별도 `ComposedLine`으로 구성되도록 보정했다.
이제 renderer의 기존 줄별 문단 들여쓰기/내어쓰기 산식이 강제 줄바꿈 뒤 줄에도 적용될 수 있다.
커서 이동 경로도 강제 줄바꿈 내부 이동과 문단 경계 이동 모두 메인테이너 동작 테스트를 통과했다.
