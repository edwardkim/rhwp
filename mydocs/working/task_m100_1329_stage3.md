# Task M100-1329 Stage 3 구현 보고서 — 글머리표 빈 줄 caret 위치

## 범위

- 대상 이슈: #1329 `rhwp-studio: 글머리표 Enter 직후 빈 줄 caret이 글머리표 앞에 표시됨`
- 브랜치: `issue-1329-bullet-caret`
- 구현 범위: 본문 문단의 `get_cursor_rect_native()` 보정과 회귀 테스트 추가
- 제외 범위: 셀/중첩 표 cursor rect API, list continuation 규칙, 글머리표/번호 UI

## 변경 파일

- `src/document_core/queries/cursor_rect.rs`
- `tests/issue_1329_bullet_caret.rs`
- `mydocs/orders/20260608.md`
- `mydocs/working/task_m100_1329_stage3.md`

## 구현 내용

### 1. 빈 list 문단 직접 hit 우회

`get_cursor_rect_native()`의 일반 TextRun hit 경로에서 list 문단의 빈 body anchor를 그대로 반환하지 않도록 했다.

조건:

- 문단 ParaShape `HeadType`이 `Outline`, `Number`, `Bullet` 중 하나
- `TextRun.char_start == Some(offset)`
- `effective_char_count == 0`
- `TextRun.text`가 비어 있음

이 경우 직접 hit을 반환하지 않고 fallback으로 넘긴다. 기존에는 이 빈 anchor가 marker 시작점에 있어 `charOffset: 0` caret도 marker 앞쪽에 표시될 수 있었다.

### 2. list 문단 fallback x 보정

마지막 fallback에서 첫 `TextLine`을 찾은 뒤, 해당 line 아래 TextRun을 수집하도록 보강했다.

수집 정보:

- `line_x`: 기존 fallback 좌표
- `first_body_x`: `char_start.is_some()`인 본문 TextRun의 x
- `marker_end_x`: `char_start.is_none()`이고 `FieldMarkerType::None`인 marker TextRun의 오른쪽 끝

list 문단의 `charOffset: 0`에서는 `marker_end_x`를 우선 사용한다. 일반 문단은 기존처럼 `line_x`를 유지한다.

### 3. marker 폭 재측정

빈 글머리표 문단에서 marker TextRun bbox 폭은 기본 스타일 기준으로 계산될 수 있었다. 이 값만 사용하면 입력 후 실제 첫 글자 시작점보다 왼쪽에 caret이 놓였다.

따라서 split 후 새 문단에 남아 있는 `char_shapes[0]`을 입력 스타일로 보고, 그 스타일로 marker 텍스트 폭을 다시 계산했다.

```text
marker_end_x = marker_x + estimate_text_width(marker_text, insertion_char_style)
```

이로써 입력 전 caret x와 실제 입력 후 첫 글자 시작 x가 일치한다.

## 테스트 추가

`tests/issue_1329_bullet_caret.rs`를 추가했다.

검증 항목:

1. `number-bullet.hwp`의 글머리표 문단을 끝에서 split한 뒤 새 빈 글머리표 문단의 caret x 확인
2. 새 빈 글머리표 문단에 실제 글자를 입력한 뒤 첫 글자 시작 x와 입력 전 caret x 비교
3. `para-head-num-2.hwp` 번호 문단에 같은 검증 적용
4. `blank2010.hwp` 일반 문단 split 후 일반 빈 문단 caret x 회귀 확인

초기 테스트에서 기존 증상이 재현됐다.

- 글머리표: 기대 body 시작 x `98.0`, 기존 빈 caret x `70.0`
- 번호: 기대 body 시작 x `164.1`, 기존 빈 caret x `140.1`

최종 구현 후 새 테스트는 통과했다.

## 실행한 명령

```bash
cargo fmt --all
cargo test --test issue_1329_bullet_caret
```

최종 `cargo test --test issue_1329_bullet_caret` 결과:

```text
running 3 tests
test issue_1329_plain_empty_paragraph_caret_keeps_original_start ... ok
test issue_1329_number_enter_empty_line_caret_stays_after_marker ... ok
test issue_1329_bullet_enter_empty_line_caret_stays_after_marker ... ok

test result: ok. 3 passed; 0 failed
```

## 미검증 항목

Stage 4에서 수행할 항목:

- `cargo fmt --all -- --check`
- `cargo test --test issue_1308_forced_break_hanging_indent`
- 가능하면 `cargo test --lib`
- 가능하면 `cd rhwp-studio && npm run build`
- 가능하면 브라우저에서 실제 rhwp-studio caret 시각 확인

## 남은 리스크

- 셀/중첩 표 cursor rect API는 이번 구현에서 수정하지 않았다. Stage 1에서 같은 계열 결함 가능성이 확인됐으므로 후속 이슈 후보로 남긴다.
- 빈 list 문단 marker 자체의 렌더 bbox는 여전히 기본 스타일 기준일 수 있다. 이번 수정은 cursor 위치를 실제 입력 시작점과 맞추는 범위에 한정했다.

## 다음 단계

Stage 4 검증을 진행한다. Stage 4 승인 전에는 추가 소스 수정을 진행하지 않는다.
