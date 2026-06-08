# Task M100-1330 구현 계획서 — 빈 list 문단 marker/caret 스타일 기준 보정

## 전제

- 수행계획서: `mydocs/plans/task_m100_1330.md`
- Stage 1 보고서: `mydocs/working/task_m100_1330_stage1.md`
- 브랜치: `issue-1330-bullet-marker-caret-size`
- base: `upstream/devel`

Stage 1 결론:

- Enter로 만들어진 새 빈 list 문단은 `Paragraph::split_at()`에서 활성 `char_shape_id`를 `char_shapes[0]`로 보존한다.
- 문제는 빈 list 문단 렌더링 fallback이 해당 문단의 활성 char shape를 쓰지 않고 `styles[0]`을 쓰는 데 있다.
- 텍스트 입력 후에는 첫 본문 run이 생기면서 실제 문단 char shape가 marker와 caret에 적용되어 크기가 바뀐다.

## 핵심 구현 원칙

1. 데이터 모델을 바꾸지 않는다.
   - `Paragraph::split_at()`과 `insert_text_at()`의 char shape 정책은 이미 입력 예정 style을 보존한다.

2. 빈 list 문단의 렌더링 fallback만 보정한다.
   - marker 폭 사전 계산
   - marker TextRun 생성
   - empty TextRun 생성

3. #1329의 caret x 좌표 보정과 섞지 않는다.
   - 이 브랜치는 #1329 PR 변경을 포함하지 않는다.
   - caret x 위치 문제는 건드리지 않고, style/font size/height 일관성만 다룬다.

4. 기존 본문 run이 있는 문단의 동작은 유지한다.
   - 본문 run이 있으면 기존처럼 첫 run style을 marker 기준으로 사용한다.
   - 본문 run이 없을 때만 paragraph active char style로 fallback한다.

## 구현 범위

### 수정 대상

- `src/renderer/layout/paragraph_layout.rs`

### 신규 테스트 대상

- `tests/issue_1330_bullet_marker_caret_size.rs`

### 수정하지 않을 대상

- `src/model/paragraph.rs`
- `src/document_core/queries/cursor_rect.rs`
- `rhwp-studio/src/engine/*`
- serializer/parser
- #1329에서 수정했던 caret x fallback 로직

## 구현 설계

### 1. active char style helper 추가

`src/renderer/layout/paragraph_layout.rs` 내부에 작은 helper를 추가한다.

예상 형태:

```rust
fn paragraph_active_text_style(
    styles: &ResolvedStyleSet,
    para: Option<&Paragraph>,
    char_offset: usize,
) -> (TextStyle, Option<u32>) {
    let char_shape_id = para
        .and_then(|p| p.char_shape_id_at(char_offset))
        .or_else(|| para.and_then(|p| p.char_shapes.first().map(|cs| cs.char_shape_id)));

    if let Some(id) = char_shape_id {
        (resolved_to_text_style(styles, id, 0), Some(id))
    } else {
        (resolved_to_text_style(styles, 0, 0), None)
    }
}
```

세부 결정:

- `char_offset`은 빈 문단에서는 `0`을 사용한다.
- language index는 빈 문단에서 실제 문자 스크립트를 아직 알 수 없으므로 `0`을 사용한다.
- 본문 run이 있는 경우에는 기존처럼 `run.lang_index`를 유지한다.
- 반환값에 `char_shape_id`를 포함해 empty TextRun에 `char_shape_id: Some(id)`를 넣을 수 있게 한다.

### 2. marker style 선택 helper 추가

marker 폭 사전 계산과 marker TextRun 생성이 같은 기준을 쓰도록 helper를 추가한다.

예상 형태:

```rust
fn numbering_marker_text_style(
    styles: &ResolvedStyleSet,
    para: Option<&Paragraph>,
    first_run: Option<&ComposedTextRun>,
) -> (TextStyle, Option<u32>) {
    if let Some(run) = first_run {
        (
            resolved_to_text_style(styles, run.char_style_id, run.lang_index),
            Some(run.char_style_id),
        )
    } else {
        paragraph_active_text_style(styles, para, 0)
    }
}
```

이유:

- 본문 run이 있으면 기존 동작과 동일하다.
- 빈 문단이면 문단의 `char_shapes[0]` 기준으로 marker를 렌더링한다.
- marker 폭 계산과 실제 marker TextRun이 같은 style을 사용한다.

### 3. marker 폭 사전 계산 보정

현재 경로:

- 첫 run이 있으면 first run style
- 없으면 `resolved_to_text_style(styles, 0, 0)`

변경:

- 첫 run이 없으면 `paragraph_active_text_style(styles, para, 0)` 사용

대상 위치:

- `src/renderer/layout/paragraph_layout.rs`의 `numbering_width` 계산부

효과:

- 빈 list 문단에서도 본문 시작 x 계산과 marker 실제 width가 입력 후와 같은 기준을 갖는다.

### 4. marker TextRun 생성 보정

현재 경로:

- 첫 run이 있으면 first run style
- 없으면 `resolved_to_text_style(styles, 0, 0)`
- marker `char_shape_id`는 항상 `None`

변경:

- 첫 run이 없으면 `paragraph_active_text_style(styles, para, 0)` 사용
- marker는 문서 좌표에 포함되지 않으므로 `char_start: None`은 유지
- `char_shape_id`는 marker가 본문 문자가 아니므로 `None` 유지가 안전하다.

대상 위치:

- `src/renderer/layout/paragraph_layout.rs`의 list marker TextRun 생성부

판단:

- marker TextRun은 실제 문서 문자 범위가 아니므로 toolbar selection용 `char_shape_id`를 넣지 않는다.
- style 자체만 active char shape 기준으로 맞춘다.

### 5. empty TextRun 생성 보정

현재 경로:

- 빈 문단 fallback에서 `resolved_to_text_style(styles, 0, 0)` 사용
- `char_shape_id: None`
- `bbox.height`는 `default_height = hwpunit_to_px(400, dpi)` 사용

변경:

- `paragraph_active_text_style(styles, para, char_offset)` 사용
- `char_shape_id: Some(active_id)`로 설정
- `TextRunNode.style.font_size`를 active style과 맞춘다.
- line bbox height는 1차에서는 기존 `default_height` 유지 여부를 구현 중 확인한다.

line bbox 판단:

- `get_cursor_rect_native()`는 먼저 TextRun hit를 찾으면 `text_run.style.font_size`를 height로 반환한다.
- 따라서 empty TextRun style만 맞춰도 caret height는 active style로 보정된다.
- TextLine bbox height를 곧바로 font size로 바꾸면 줄 배치/문단 간격 영향이 커질 수 있으므로 1차에서는 피한다.

대상 위치:

- `src/renderer/layout/paragraph_layout.rs`의 `composed.lines.is_empty()` fallback

## 테스트 설계

신규 통합 테스트 파일:

- `tests/issue_1330_bullet_marker_caret_size.rs`

테스트는 Rust core 레벨에서 작성한다.

### 공통 fixture

문서 구성:

- 빈 DocumentCore 또는 테스트용 Document를 구성한다.
- doc_info에 두 개 이상의 CharShape를 둔다.
  - style 0: 기본 크기
  - style N: 큰 크기, 예: 15pt 또는 18pt
- paragraph에는 `char_shapes[0].char_shape_id = N`을 설정한다.
- paragraph의 ParaShape는 `HeadType::Bullet` 또는 `HeadType::Number`로 설정한다.
- bullet/numbering 정의를 doc_info에 추가한다.

검증 helper:

- `get_page_text_layout_native(0)` JSON에서 특정 `paraIdx`의 marker TextRun을 찾는다.
  - marker는 `charStart`가 없고 `text`가 bullet/numbering marker와 일치한다.
- 본문 run은 `charStart: 0`이 있고 입력 텍스트와 일치한다.
- `get_cursor_rect_native(sec, para, 0)`의 `height`를 읽는다.

### 테스트 1. bullet marker font size 입력 전/후 일치

흐름:

1. 큰 char shape를 가진 bullet 문단에 텍스트를 넣는다.
2. 문단 끝에서 `split_paragraph_native()`를 호출해 새 빈 bullet 문단을 만든다.
3. 빈 새 문단의 marker `fontSize`를 측정한다.
4. 같은 문단에 `insert_text_native(..., 0, "가")`를 호출한다.
5. 입력 후 marker `fontSize`와 본문 run `fontSize`를 측정한다.

기대:

- 입력 전 marker fontSize == 입력 후 marker fontSize
- 입력 후 marker fontSize == 본문 run fontSize
- 빈 상태 caret height == 입력 후 caret height 또는 허용 오차 내 일치

### 테스트 2. numbered marker font size 입력 전/후 일치

bullet과 동일한 구조를 numbering 문단에 적용한다.

기대:

- 번호 marker fontSize가 입력 전/후 유지된다.
- 본문 run fontSize와 일치한다.

### 테스트 3. 일반 빈 문단 caret height 회귀

list가 아닌 일반 문단에 큰 char shape를 적용하고 빈 문단 상태의 caret height를 확인한다.

기대:

- empty TextRun이 active char style을 사용한다.
- list가 아닌 빈 문단도 입력 예정 style 기준 caret height를 갖는다.
- 기존 일반 문단 cursor rect가 실패하지 않는다.

## 검증 계획

필수 자동 검증:

- `cargo fmt --all -- --check`
- `cargo test --test issue_1330_bullet_marker_caret_size`
- `cargo test --lib`
- `git diff --check`

프론트엔드/WASM 검증:

- `wasm-pack build --target web`
- `cd rhwp-studio && npm run build`

수동 검증:

- rhwp-studio dev server 실행
- 글머리표 문단 끝에서 Enter
- 새 빈 줄 marker와 caret 크기 확인
- 같은 줄에 텍스트 입력
- 입력 전/후 marker와 caret/입력 글자 크기가 불연속적으로 바뀌지 않는지 확인

주의:

- #1329가 아직 base에 없으면 빈 줄 caret x 위치는 여전히 marker 앞쪽일 수 있다.
- #1330 수동 판정에서는 x 위치보다 크기 변화 여부를 우선 본다.
- #1329 PR이 병합된 뒤에는 두 PR을 함께 적용해 x 위치와 크기를 동시에 재확인한다.

## 회귀 위험

- marker width 계산이 바뀌므로 list 문단의 첫 줄 가용폭과 본문 시작 x가 소폭 변할 수 있다.
- 번호/글머리표 정의 자체가 별도 marker char shape를 의도하는 문서가 있으면 본문 char shape 기준 보정이
  실제 한컴과 다를 가능성이 있다.
- empty TextRun에 active style을 적용하면 일반 빈 문단 caret height가 달라질 수 있다. 다만 사용자 입력
  예정 style과 맞추는 방향이므로 UX 기준으로는 더 자연스럽다.
- `para: Option<&Paragraph>`가 없는 fallback 경로에서는 기존 style 0 동작을 유지해야 한다.

## 완료 기준

1. 빈 bullet 문단 marker font size가 텍스트 입력 전/후 유지된다.
2. 빈 numbered 문단 marker font size가 텍스트 입력 전/후 유지된다.
3. 빈 문단 caret height가 입력 예정 char shape 기준으로 계산된다.
4. 본문 run이 있는 기존 list 문단의 marker style 선택은 기존과 동일하게 유지된다.
5. 자동 테스트와 rhwp-studio 빌드가 통과한다.
6. 작업지시자가 로컬 dev server에서 크기 변화가 사라졌다고 판정한다.

## 승인 요청

본 구현 계획서를 승인하면 Stage 3부터 소스 수정을 시작한다.
