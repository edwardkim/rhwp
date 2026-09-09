# 완료 보고서 — Task M100-3028

- 이슈: #3028
- 제목: hh:bullet checkedChar 속성 파싱/방출 누락
- 작성일: 2026-07-22
- 브랜치: `task/m100-3025-hwpx-checkedchar-bullet`

## 1. 완료 내용

`src/parser/hwpx/header.rs`의 `parse_bullet_hwpx`가 `hh:bullet` 요소의 `char`,
`useImage` 속성만 읽고 `checkedChar` 속성을 무시하던 문제를 수정했다. 체크박스
글머리표(checkbox bullet)의 체크 문자가 항상 IR `check_bullet_char` 기본값
(`'\0'`)으로 고정되어 실제 문서에 지정된 체크 문자가 소실되고 있었다.

직렬화기(`src/serializer/hwpx/header.rs`의 `write_bullet`)도 대응 수정했다.
기존에는 `checkedChar` 속성을 방출하지 않고 `paraHead`의 `checkable` 속성을
항상 `"0"`으로 하드코딩하고 있었다. 같은 계열 버그(#2957, #3005, #3011)와
동일하게 리터럴 하드코딩이 실제 IR 필드를 반영하지 못한 사례다.

## 2. 주요 변경

- `src/parser/hwpx/header.rs`
  - `parse_bullet_hwpx`에 `checkedChar` 속성 분기 추가 → `bullet.check_bullet_char`
  - 회귀 테스트 `test_parse_bullet_hwpx_checked_char` 추가
- `src/serializer/hwpx/header.rs`
  - `write_bullet`: `check_bullet_char != '\0'`일 때만 `checkedChar` 속성 방출
  - `paraHead`의 `checkable` 값을 `has_checked_char` 여부에 따라 동적 설정
    (`"1"`/`"0"`)

## 3. 검증

- `cargo check --lib` 통과
- `cargo test --lib checked_char` 통과 (신규 테스트 1건)
- `cargo test --lib bullet` 통과 (관련 기존 테스트 3건 + 신규 1건, 전부 통과)
- `rustfmt --edition 2021` 적용 (추가 변경 없음)

## 4. 남은 이슈

없음.
