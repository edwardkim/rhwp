# task_m100_3349 처리결과 보고서 — `export-text` 위치 인자 파싱 통일

- **이슈**: [#3349](https://github.com/edwardkim/rhwp/issues/3349)
- **브랜치**: `pr/fix-issue-3349-export-text-option-order` (upstream/devel `4a39f7cc0` 직분기)
- **범위**: `src/main.rs`(`export_text` 파싱부만), `tests/issue_3349_export_text_option_order.rs`(신규),
  `mydocs/manual/cli_commands.md`(1항목)
- **분류**: 버그 수정 (CLI 파싱 — 에이전트 호출 실패)

## 1. 배경

`export-text` 는 옵션이 파일 **뒤**면 동작하고 **앞**이면 exit 2 로 죽는다.
v0.8.0 릴리스 바이너리 실측:

| 호출 순서 | 결과 |
|---|---|
| `export-text FILE --json -p 0` | exit 0 |
| `export-text --json FILE -p 0` | exit 0 |
| `export-text -p 0 --json FILE` | **exit 2 — `알 수 없는 옵션: 0`** |
| `export-text --json --page 0 FILE` | **exit 2 — `알 수 없는 옵션: 0`** |

원인은 `--json` 만 위치 무관으로 선추출한 뒤 **남은 첫 토큰을 무조건 파일로 가정**
(`let file_path = &args[0]`)하는 파싱이다. `-p` 가 파일 경로가 되고 `0` 이 옵션 자리에서
"알 수 없는 옵션"이 된다 — 오류 메시지가 실제 원인과 무관해 진단이 불가능하다.

같은 조회 축의 `export-structure`/`export-tables` 는 위치 무관 파싱이라 같은 순서가
동작한다. 자기서술(`capabilities`)과 JSON 파이프라인 가이드 어디에도 "파일 선행" 제약이
선언돼 있지 않으므로, 자기서술만 보고 호출을 생성하는 에이전트는 이 지뢰를 피할 수 없다.

## 2. 설계 결정

- **새 규약 발명 0** — `export-structure`(#3261)가 이미 쓰는 파싱(첫 비플래그 토큰 = 파일,
  옵션 위치 무관, 중복 positional 즉시 exit 2)을 그대로 옮겼다. 명령군 안에서 규약이
  하나로 수렴한다.
- **기존 호출 전부 보존** — 파일 선행 호출은 이전과 동일하게 동작한다. 순수 확장이다.
- **중복 positional 은 조용히 덮지 않는다** — `입력 파일은 하나만 지정할 수 있습니다` 로
  즉시 exit 2 (export-structure 와 동일 문구).
- **1차 조각은 `export-text` 하나만** — JSON 파이프라인 가이드가 첫 번째로 안내하는 추출
  축이다. 같은 패턴(파일 선행 강제)의 나머지 export 계열(svg/png/pdf/markdown/render-tree/
  doclang)은 같은 레시피의 후속 조각으로 남겼다(#3349 에 명단 기록).

## 3. 변경

- `export_text()` — 파싱 루프를 위치 무관으로 교체 (`args[0]` 가정 제거, `starts_with('-')`
  분기 + positional 수집). 파싱 이후 로직은 무변경.
- `cli_commands.md` — `export-text` 항목에 위치 무관 규약 1줄 명시.

## 4. 검증

- **회귀 테스트 5종** (`tests/issue_3349_export_text_option_order.rs`):
  - 버그 재현 형태 그대로 `--json -p 0 FILE` → exit 0 + 봉투 계약 (red→green)
  - `-p 0 --json FILE` — `--json` 특례가 아님을 고정
  - 순서 3종(파일 선행/플래그 선행/교차)의 **결과 JSON 완전 일치**
  - 중복 positional exit 2 + stdout 0바이트 + 오류 문구
  - 알 수 없는 플래그 exit 2 (기존 계약 유지)
- **무회귀**: `cli_json_contract`, `cli_exit_codes` 전부 green (release-test 프로필)
- `cargo fmt --all -- --check` clean
- **실측 전/후**: 전 = v0.8.0 릴리스 바이너리(공식 배포물), 후 = 본 브랜치 빌드 —
  PR 본문에 콘솔 증거 수록

## 5. 남긴 것

- 같은 패턴의 나머지 11곳(export 계열 6 + 내부 dump/diag 5)은 이 조각이 머지된 뒤
  같은 레시피로 후속 처리 가능 — 명단은 #3349 본문에 있다.
- `search` 의 검색어 positional(파일 + 검색어 2개)은 이 규약의 2-positional 확장이 필요해
  이번 범위에서 다루지 않았다.
