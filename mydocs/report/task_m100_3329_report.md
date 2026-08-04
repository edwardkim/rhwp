# task_m100_3329 처리결과 보고서 — `edit fill-fields` (Stage 3 최소 조각)

- **이슈**: [#3329](https://github.com/edwardkim/rhwp/issues/3329)
- **브랜치**: `pr/task-edit-fill-fields` (upstream/devel `8bb8f277d` 직분기)
- **범위**: `src/main.rs`(명령 2개·디스패치 1행·help), `tests/edit_fill_fields_contract.rs`(신규),
  `mydocs/manual/cli_commands.md`
- **분류**: 기능 추가 (편집 — 로드맵 #2659 Stage 3)

## 1. 배경

Stage 1(계약)·Stage 2(조회 기계화)가 devel 에 반영되어, 에이전트는 문서를 발견·검색·이해할
수 있게 됐다. 특히 `fields --json` 으로 **서식이 무엇을 요구하는지** 읽을 수 있다.

그런데 값을 채워 넣으려면 여전히 Windows + 한컴 오피스 + COM 매크로가 필요했다. 코어에
`set_field_value_by_name` 이 이미 있고 rhwp-studio 가 쓰고 있는데 **CLI 출구가 없어서**
브라우저 밖 자동화가 불가능했다.

## 2. 설계 결정

- **로드맵 §7.3 의 3종 중 `fill-fields` 하나만** 놓았다. 편집 축에서 위험이 가장 작고
  (필드 값만 바꾸므로 레이아웃·구조 불변) 수요가 가장 크다(행정 서식 자동 작성).
- **새 편집 로직 0줄** — 검증된 `set_field_value_by_name` 을 그대로 부른다.
- **`--dry-run` 은 파일 생성 경로 자체를 타지 않는다.** 편집 명령의 안전장치는 "썼다가
  지운다"가 아니라 "쓰지 않는다"여야 한다. 계약 테스트가 파일 부재를 단언한다.
- **실패 시 원본 불변**: 필드 설정이 하나라도 실패하면 즉시 종료하고 출력을 쓰지 않는다.
- **없는 필드 이름을 조용히 무시하지 않는다** — `notFound` 로 보고해 에이전트가 오타를
  즉시 안다. 편집 전에 `collect_all_fields()` 로 실재 이름 집합을 먼저 모아 가려낸다.
- **`@파일` 입력** — 대량 메일머지에서 셸 인용 지옥을 피하기 위해 `--data @row.json` 지원.
- `edit` 을 명령군으로 열어 `replace-text`/`set-cell` 이 같은 규약으로 붙을 자리를 만들었다.

## 3. 변경

- `run_edit()` — `edit` 명령군 디스패처 (알 수 없는 하위 명령은 exit 2)
- `edit_fill_fields()` — 인자 파싱 / `@파일` 처리 / dry-run 분기 / 봉투 방출
- 디스패치 1행, help 등재, `cli_commands.md` 신설 항목

## 4. 검증

- **계약 테스트 7종 red→green**:
  - `--dry-run` 이 **파일을 만들지 않음**(핵심 안전 계약)
  - 저장 후 **산출물을 `fields --json` 으로 다시 읽어 값 반영 대조**(보고만 믿지 않음)
  - 없는 필드 이름 `notFound` 보고
  - 없는 파일 exit 1 + stdout 0바이트 + **출력 파일 미생성**
  - 잘못된 JSON exit 2 / `--data` 누락 exit 2 / 알 수 없는 하위 명령 exit 2
- `cargo clippy --release --bin rhwp -- -D warnings`: 0, `rustfmt` clean
- 무회귀: `cli_exit_codes`(10), `fields_json_contract`(8) 전부 green
- **실측 루프**: `samples/field-01.hwp` 로 서식 조사 → 3개 필드 채우기 → 산출물 재독으로
  `회사명`·`작성자`·`부서명` 값 반영 확인 (전 과정 CLI)

## 5. 남긴 것

- `edit replace-text` / `edit set-cell` — 같은 규약(`--dry-run`·결과 JSON·원본 불변)으로
  이 조각이 머지된 뒤 후속.
- 머리말/꼬리말·각주/미주 안의 필드는 `collect_fields_from_paragraph` 재귀 범위 밖이라
  채워지지 않는다(`fields` 와 같은 한계, 문서에 명시됨).
- HWPX 입력의 `edit fill-fields` 는 `export_hwp_native()` 가 HWP5 산출이라 이번 범위에서
  다루지 않았다 — HWPX 왕복 저장 경로 연결은 별도 이슈가 맞다.
