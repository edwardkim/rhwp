# Task M100 #1470 stage3 착수 문서

- 이슈: #1470 `스타일 적용/편집 불일치: 왼쪽 여백 배율, 줄간격 미반영, 표 캡션/생성 위치 문제`
- 기준 커밋: `aa5d3dde task 1470: TAC 표 중복 렌더 방지`
- 작업 브랜치: `task_m100_1470`
- 작성일: 2026-06-22
- 상태: 구현 및 focused 검증 완료.

## 1. Stage 3 배경

Stage 1은 스타일 여백/줄간격 반영과 표 캡션 생성 보정을 처리했다.
Stage 2는 `createTableEx(... treatAsChar: true)` 표가 같은 `paraIdx/controlIdx`로 두 번 렌더되는 문제를 처리했다.

이후 한컴 도움말 기준으로 스타일 구현을 재검토한 결과, 제보된 "왼쪽 여백 15pt가 30pt로 바뀜"과 "줄간격 미반영"은 현재 focused 테스트로 막고 있으나, 한컴 스타일 동작 전체 기준에서는 아직 부족한 부분이 남아 있다.

참고한 한컴 도움말:

- `format/style/style.htm` 스타일
- `format/style/style(apply).htm` 스타일 적용하기
- `format/style/style(edit).htm` 스타일 편집하기

핵심 기준:

- 문단 스타일은 문단 단위로 적용된다.
- 글자 스타일은 현재 커서 위치/낱말/선택 범위의 글자 모양에 적용된다.
- 스타일 편집 시 그 스타일이 적용된 모든 문단/글자 모양이 바뀐다.
- 단, 사용자가 직접 바꾼 글자 모양이나 문단 모양은 그대로 둔다.
- 같은 문단에 같은 스타일을 다시 적용하면 덮어쓰기 동작으로 직접 바꾼 모양까지 스타일 내용으로 덮을 수 있다.

## 2. 확인된 현재 구현 문제

### 2.1 직접 글자 모양 보존 누락

`src/document_core/commands/formatting.rs`의 `apply_style_native`와 `apply_cell_style_native`는 스타일 적용 시 `char_shapes.clear()` 후 스타일의 `char_shape_id` 하나만 남긴다.

`src/wasm_api.rs`의 `updateStyleShapes`도 스타일을 사용하는 모든 본문/셀 문단에 대해 `char_shapes.clear()` 후 새 스타일 `char_shape_id`를 단일 적용한다.

이는 한컴 도움말의 "사용자가 직접 바꾼 글자 모양은 그대로 놓아둔다" 기준과 맞지 않는다.

### 2.2 직접 문단 모양 보존 누락

`updateStyleShapes`는 해당 스타일을 사용하는 모든 문단의 `para_shape_id`를 새 스타일 `para_shape_id`로 일괄 교체한다.

현재 구조에서는 "스타일을 기반으로 한 문단이지만 사용자가 직접 바꾼 문단 모양"인지 구분하는 메타데이터가 없다.
따라서 스타일 편집 시 직접 문단 서식까지 모두 새 스타일로 덮을 수 있다.

### 2.3 글자 스타일 적용 단위 불일치

Studio `InputHandler.applyStyle`는 선택한 스타일의 `type`을 보지 않고 문단 target에 `applyStyle/applyCellStyle`을 호출한다.
Rust `apply_style_native`도 `style_type`별 분기 없이 문단 `style_id`, `para_shape_id`, 전체 `char_shapes`를 바꾼다.

한컴 도움말은 글자 스타일과 문단 스타일의 적용 단위를 분리한다.

### 2.4 같은 스타일 재적용 덮어쓰기 모드 부재

한컴은 같은 문단에 같은 스타일을 다시 적용하면 직접 바꾼 모양까지 덮어쓸 수 있다.
현재 API에는 보존 적용과 덮어쓰기 적용을 구분하는 옵션이 없다.

## 3. Stage 3 목표

Stage 3는 한컴 스타일 동작 전체를 완성하기보다, #1470 피드백과 직접 연결되는 스타일 적용/편집 정합성을 안전하게 개선한다.

1. 문단 스타일 적용 시 기본적으로 직접 글자 모양 범위를 보존한다.
2. 스타일 편집 시 해당 스타일을 사용하는 문단의 직접 글자 모양 범위를 보존한다.
3. 문단 모양 직접 서식 보존은 기존 데이터 모델의 한계 때문에 우선 탐지 가능한 범위로 제한하거나, 별도 덮어쓰기 API 설계를 문서화한다.
4. 글자 스타일은 문단 스타일과 구분하여 최소한 문단 `para_shape_id/style_id`를 바꾸지 않도록 한다.
5. 같은 스타일 재적용 덮어쓰기 모드는 이번 Stage에서 API 옵션까지 설계하되, UI 확인 대화상자까지 한 번에 넣을지는 구현 중 리스크를 보고 결정한다.

## 4. 구현 방향

### 4.1 직접 글자 모양 보존

문단 스타일 적용/편집 전후에 다음 기준으로 직접 글자 모양을 보존한다.

- 기존 문단 `char_shapes`가 단일 `[start_pos=0, char_shape_id=old_style.char_shape_id]`이면 직접 글자 모양이 없다고 보고 새 스타일 `char_shape_id` 단일 적용.
- 기존 문단 `char_shapes`에 여러 range가 있거나, 단일 range라도 기존 스타일 `char_shape_id`와 다르면 직접 글자 모양이 있다고 보고 기존 range를 유지한다.
- 단, `start_pos=0`의 기본 range가 기존 스타일과 같으면 새 스타일 `char_shape_id`로 교체하고, 나머지 직접 range는 유지하는 방안을 우선 검토한다.

이 방식은 "스타일 기반 기본 글자 모양은 새 스타일로 바꾸되, 사용자가 직접 지정한 글자 모양 범위는 유지"하는 최소 구현이다.

### 4.2 직접 문단 모양 보존

현재 문단에는 "스타일 para shape에서 직접 수정된 값"을 알 수 있는 명시적 diff 메타데이터가 없다.

우선 다음 보수적 전략을 검토한다.

- 문단의 현재 `para_shape_id`가 스타일의 이전 `para_shape_id`와 같으면 새 스타일 `para_shape_id`로 교체한다.
- 다르면 직접 문단 서식이 있다고 보고 `para_shape_id`를 유지한다.
- 단, Stage 1의 "스타일 편집 후 줄간격이 실제 적용 문단에 반영되어야 한다" 요구와 충돌할 수 있으므로, 이 전략은 테스트로 검증한 뒤 적용한다.

### 4.3 글자 스타일 분기

`style.style_type != 0`인 경우:

- 본문 문단의 `style_id`/`para_shape_id`는 변경하지 않는다.
- 선택 범위 API가 없는 단순 `applyStyle(sec, para, styleId)` 호출에서는 현재 문단 전체 char range에 글자 스타일 `char_shape_id`를 적용하는 fallback을 검토한다.
- Studio의 실제 선택 범위가 있으면 추후 선택 범위에만 적용하도록 확장한다.

### 4.4 덮어쓰기 모드

API 설계 후보:

- `applyStyleEx(sec, para, styleId, optionsJson)` 또는 기존 `applyStyle` 내부 옵션 확장
- `overwriteDirectFormatting: true`이면 기존 Stage 1/2처럼 직접 글자/문단 모양까지 새 스타일로 덮는다.
- 기본값은 한컴 도움말 기준 보존 적용으로 둔다.

## 5. 테스트 계획

Focused Rust 테스트:

- `issue_1470_style_apply_preserves_direct_char_shape`
  - 문단에 스타일 적용 후 일부 범위에 직접 CharShape를 지정
  - 다른 문단 스타일 적용 또는 스타일 편집 후 직접 CharShape 범위가 유지되는지 확인

- `issue_1470_style_update_preserves_direct_char_shape`
  - 스타일을 사용하는 문단에 직접 글자 모양 range를 둔다
  - `updateStyleShapes`로 스타일 글자 모양을 바꾼다
  - 기본 range는 새 스타일로 바뀌고 직접 range는 유지되는지 확인

- `issue_1470_style_update_reflows_and_keeps_margin_unit`
  - Stage 1 기존 테스트 유지
  - 직접 문단 모양 보존 전략이 줄간격 전파를 깨지 않는지 확인

- `issue_1470_character_style_does_not_replace_para_style`
  - 글자 스타일 적용 시 문단 `para_shape_id`와 문단 스타일이 바뀌지 않는지 확인

브라우저 검증:

- 7700 Studio에서 스타일 편집 대화상자 기준:
  - 왼쪽 여백 15pt 입력 후 다시 열어도 15pt로 보이는지
  - 줄간격 변경 후 문단 LineInfo가 바뀌는지
  - 직접 글자 모양이 있는 문단에 스타일 편집을 해도 직접 글자 모양이 유지되는지

검증 명령:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --release issue_1470_style --lib`
- 필요 시 `wasm-pack build --target web --out-dir pkg`
- 필요 시 `cd rhwp-studio && npm run build`

`cargo clippy --all-targets -- -D warnings`는 이전 작업지시자의 중지 지시가 있었으므로 별도 지시가 있을 때만 실행한다.

## 6. 제외 범위

- 스타일 가져오기/내보내기 파일 포맷 완성
- 스타일마당 UI
- 스타일 자동 적용 전체 구현
- 붙여넣기/문서 끼워 넣기 시 스타일 병합 정책
- 글자 스타일의 낱말 단위 자동 선택 UX 완성

## 7. 승인 게이트

이 문서 승인 전에는 소스 파일을 수정하지 않는다.
승인 후 Stage 3 구현은 스타일 직접 서식 보존과 글자/문단 스타일 적용 단위 분리의 focused 회귀 테스트 추가로 제한한다.

## 8. Stage 3 구현 결과

작업지시자 승인 후 다음 구현을 반영했다.

- `Paragraph`에 스타일 기본 글자 모양 run만 치환하고 직접 지정 run은 유지하는 헬퍼를 추가했다.
- 본문/셀 문단 스타일 적용 시 이전 스타일의 기본 CharShape/ParaShape와 같은 부분만 새 스타일 정의로 바꾸고, 직접 글자/문단 서식으로 판단되는 부분은 유지하도록 바꿨다.
- 글자 스타일(`style_type == 1`) 적용 시 문단 `style_id`와 `para_shape_id`를 바꾸지 않고 글자 모양에만 적용하도록 분기했다.
- `updateStyleShapes` 전파 시 직접 CharShape range를 지우지 않고, 이전 스타일 기본 CharShape run만 새 스타일 CharShape로 치환하도록 바꿨다.
- 직접 문단 모양은 현재 문단의 `para_shape_id`가 이전 스타일의 `para_shape_id`와 같을 때만 새 스타일 `para_shape_id`로 갱신한다. 다르면 사용자가 직접 바꾼 문단 모양으로 보고 유지한다.

추가한 focused 회귀 테스트:

- `issue_1470_style_apply_preserves_direct_char_shape`
- `issue_1470_style_update_preserves_direct_char_shape`
- `issue_1470_character_style_does_not_replace_para_style`

검증 결과:

- `cargo fmt --check` 통과
- `cargo test --release issue_1470_style --lib` 통과
- `cargo test --release issue_1470_character_style --lib` 통과
- `cargo test --release issue_1470 --lib` 통과

`cargo clippy --all-targets -- -D warnings`는 이전 작업지시자의 중지 지시에 따라 실행하지 않았다.
