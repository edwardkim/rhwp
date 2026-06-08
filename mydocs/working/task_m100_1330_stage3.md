# Task M100-1330 Stage 3 구현 보고서

## 대상 이슈

- GitHub Issue: #1330 `rhwp-studio: 빈 글머리표 줄에 입력 시 marker와 caret 크기가 커짐`
- 브랜치: `issue-1330-bullet-marker-caret-size`
- 기준 브랜치: `upstream/devel`

## 구현 요약

빈 글머리표/번호 문단에서 본문 run이 없는 경우 렌더러 fallback이 기본 글자 모양
`charShapeId=0`을 사용하던 경로를 문단의 활성 글자 모양 기준으로 보정했다.

수정 대상:

- `src/renderer/layout/paragraph_layout.rs`
- `tests/issue_1330_bullet_marker_caret_size.rs`

## 변경 내용

### 1. 문단 활성 글자 모양 helper 추가

`paragraph_active_text_style()`을 추가해 문단과 문자 offset 기준의 활성 `char_shape_id`를
`TextStyle`로 변환하도록 했다.

빈 문단처럼 실제 본문 run이 없는 경우에도 `Paragraph::split_at()`이 보존한
`char_shapes[0]` 값을 사용할 수 있다.

### 2. list marker style fallback 보정

`numbering_marker_text_style()`을 추가해 marker style 기준을 다음처럼 통일했다.

1. 본문 run이 있으면 기존처럼 첫 run의 style 사용
2. 본문 run이 없으면 문단 활성 글자 모양 사용
3. 문단 정보가 없으면 마지막 fallback으로 기존 기본 style 사용

적용 경로:

- marker 폭 사전 계산
- marker TextRun 생성

### 3. 빈 TextRun fallback 보정

빈 문단 caret hit target으로 생성되는 empty TextRun도 문단 활성 글자 모양을 사용하도록
수정했다.

효과:

- Enter 직후 빈 list 문단의 caret height가 입력 예정 글자 크기와 일치한다.
- 텍스트 입력 후 첫 본문 run이 생겨도 marker/caret 크기가 불연속적으로 바뀌지 않는다.
- empty TextRun의 `char_shape_id`도 활성 값으로 채워져 서식 기준이 일관된다.

## 회귀 테스트

신규 테스트:

- `tests/issue_1330_bullet_marker_caret_size.rs`

테스트 흐름:

1. 빈 HWP 문서를 생성한다.
2. `fontSize=1800`을 적용한 글머리표 문단을 만든다.
3. 문단 끝에서 `split_paragraph_native()`로 새 빈 글머리표 문단을 만든다.
4. 빈 문단의 marker font size, empty anchor font size, caret height를 측정한다.
5. 새 문단에 `"가"`를 입력한다.
6. 입력 후 marker font size, body font size, caret height가 입력 전과 같은지 검증한다.

## 검증 결과

실행한 명령:

```bash
cargo fmt --all -- --check
cargo test --test issue_1330_bullet_marker_caret_size
```

결과:

- `cargo fmt --all -- --check`: 통과
- `cargo test --test issue_1330_bullet_marker_caret_size`: 통과, 1 passed

## 다음 단계

Stage 4에서 더 넓은 검증을 진행한다.

예상 검증:

- 관련 렌더링/커서 테스트
- `cargo test --lib`
- `wasm-pack build --target web`
- `rhwp-studio` 빌드 또는 로컬 dev server 수동 확인

전체 검증 범위는 작업지시자 승인 후 진행한다.
