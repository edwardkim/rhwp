# task_m100_3359 처리결과 보고서 — export 계열 위치 인자 파싱 통일 (2차 조각)

- **이슈**: [#3359](https://github.com/edwardkim/rhwp/issues/3359) (#3349 후속)
- **브랜치**: `pr/fix-issue-3359-export-family-option-order` (upstream/devel `4a39f7cc0` 직분기)
- **범위**: `src/main.rs`(export_svg/export_render_tree/export_png/export_pdf/
  export_markdown/export_doclang 파싱부만), `tests/issue_3359_export_family_option_order.rs`(신규),
  `mydocs/manual/cli_commands.md`(공통 옵션 1항목)
- **분류**: 버그 수정 (CLI 파싱 — #3349 명단의 export 계열 잔여분)

## 1. 배경

#3349 에서 확인한 파일-선행 강제 파싱(`args[0]` 무조건 파일 가정)의 export 계열 잔여
6곳이다. 실측(v0.8.0): `rhwp export-svg -p 0 FILE` → `알 수 없는 옵션: 0` exit 2.
1차 조각(#3352, export-text)과 같은 레시피를 기계적으로 적용했다.

## 2. 설계 결정

- **1차 조각과 동일 규약** — 첫 비플래그 토큰 = 파일, 옵션 위치 무관, 중복 positional
  즉시 exit 2 (`입력 파일은 하나만 지정할 수 있습니다`), 파일 미지정 시 기존 사용법
  메시지 그대로 exit 2. 각 명령의 옵션 처리 암은 무변경 — 기본 암(`_`)만
  `starts_with('-')` 분기 + positional 수집으로 교체했다.
- **export-pdf 의 `--help` 특례 보존** — 첫 인자 `--help|-h` 는 종전대로 사용법 출력
  exit 0. (다른 위치의 `--help` 는 종전에도 지금도 알 수 없는 옵션 exit 2.)
- **export-png 도 같은 레시피 적용** — 단 native-skia feature 빌드에서만 실행되므로
  통합 테스트는 non-feature 빌드에서 가능한 5종만 다룬다. 내부 도구(dump-*, diag)는
  '일반 사용자 대상 아님' 구분에 따라 범위 제외(#3359 본문에 기록).

## 3. 변경

- 6개 함수 파싱부: `args[0]` 가정 제거 → 위치 무관 수집. 파싱 이후 로직 무변경.
- `cli_commands.md` 공통 옵션에 위치 무관 규약 1항목.

## 4. 검증

- **회귀 테스트 8종** (`tests/issue_3359_export_family_option_order.rs`, red→green):
  svg/markdown/render-tree/pdf/doclang 옵션 선행 성공(산출물 확인 포함) /
  중복 positional exit 2 / 옵션만 주고 파일 없음 exit 2 (5종 일괄) /
  알 수 없는 플래그 즉시 exit 2 유지
- **무회귀**: `cli_exit_codes`(export-svg `--fontpath` 오타 즉시 exit 2 + 산출물 미생성
  계약 포함), `cli_json_contract` 전부 green (release-test 프로필)
- `cargo fmt --all -- --check` clean, clippy `--bin rhwp -- -D warnings` 0건
- **실측 전/후**: 전 = v0.8.0 릴리스(export-svg `-p 0` 선행 exit 2), 후 = 본 브랜치
  빌드(동작) — PR 본문 수록

## 5. 남긴 것

- 내부 도구(dump-pages/dump-note-shape/dump-endnote-lines/dump/diag)의 같은 패턴 5곳 —
  필요해지면 같은 레시피로.
- `search` 의 2-positional(파일+검색어) 확장은 #3349 때와 같은 이유로 범위 밖.
