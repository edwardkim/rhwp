---
kind: report
status: active
canonical: mydocs/report/task_m100_table_csv/README.md
last_verified: 2026-08-02
---

# #3719 §6-7 처리 기록 — `table-to-csv` / `csv-to-table`

## 문제

데이터 보고서 자동화에는 **입구와 출구**가 둘 다 있어야 한다. rhwp 에는 지금까지
출구가 없었다.

- `export-tables` 는 병합을 `rowSpan`/`colSpan` 으로 **보존**한 격자 JSON 을 낸다.
  구조는 정확하지만, 엑셀·pandas 같은 표 계산기가 먹는 것은 직사각 격자뿐이라
  소비자가 매번 격자를 펴는 코드를 다시 쓴다.
- 그 펴는 코드가 틀리기 쉽다. 모델의 `Table.cells` 는 **앵커 셀만** 담고 병합으로
  덮인 좌표는 목록에 아예 없으므로(`table_extract.rs`), 앵커를 순서대로 이어 붙이면
  병합이 있는 행부터 **열이 통째로 밀린다**. 오류도 경고도 없이 값이 어긋난 표가
  나오고, 에이전트는 렌더를 보지 않으므로 알아채지 못한다.
- 되돌리는 축은 아예 없었다. 계산 결과를 원본 서식 그대로 표에 되넣으려면
  `edit set-cell` 을 칸 수만큼 호출해야 했다(왕복 비용 O(칸)).

## 구현

### 코어 — `src/document_core/queries/table_csv.rs` (신규)

파서·렌더 무변경의 순수 변환 모듈. `table_extract::TableGrid` 를 그대로 받는다.

- `grid_matrix(grid)` — 격자를 `rows`×`cols` 로 **채운다**. 앵커 위치에 값, 병합으로
  덮인 칸은 빈 문자열. 값을 병합 범위에 복제하지 **않는** 이유는 왕복 때문이다 —
  복제하면 되돌릴 때 "덮인 칸에 값이 있는 CSV" 가 되어 스스로 거부된다.
- `quote_field` / `matrix_to_csv` / `grid_to_csv` — RFC 4180 인용(`,`·`"`·CR·LF 가
  있으면 큰따옴표, 내부 `"` 는 `""`), 레코드 구분자는 CRLF(§2.1).
- `parse_csv` — RFC 4180 판독. LF 단독 구분자와 선두 BOM 도 받는다(유닉스 도구 산출
  호환). 마지막 레코드 뒤 구분자 하나는 빈 레코드를 만들지 않는다. 닫히지 않은
  따옴표·닫는 따옴표 뒤 잡문자는 위치(`record`/`field`)와 함께 오류로 낸다 —
  조용히 이어 붙이면 `"a"b` 가 `ab` 로 통과해 원본과 다른 값이 표에 들어간다.
- 모듈 단위 테스트 8건(인용 최소화·왕복·후행 구분자·빈 필드·BOM·빈 입력·오류 2종).

### `table-to-csv` (#3719 §6)

```
rhwp table-to-csv <파일> [--table N] [-o <경로>] [--bom] [--json]
```

- `--table` 생략 시 **본문 최상위 표 전부**. 표 번호는 `export-tables` 의 `index`
  이며 `edit set-cell --table` 과 같은 좌표계다(중첩·컨테이너 표는 v1 범위 밖).
- `-o` 의 뜻은 `--table` 유무로 갈린다: 한 표면 그 경로가 **파일**, 전부면 표별
  파일(`table<N>.csv`)을 담을 **폴더**(`export-svg` 의 `-o` 규약과 같은 이유).
- `--bom` 은 **파일 인코딩 표식**이라 파일에만 붙는다. 봉투의 `csv` 문자열에 섞으면
  JSON 을 그대로 쓰는 소비자가 U+FEFF 를 첫 셀 값의 일부로 읽는다.
- `-o` 도 `--json` 도 없으면 CSV 본문을 stdout 으로 흘린다(파이프용).

### `csv-to-table` (#3719 §7)

```
rhwp csv-to-table <파일> --csv <경로> --table N [-o <출력>] [--dry-run] [--verify] [--json]
```

- **표 크기를 바꾸지 않는다.** 선검증 → 인메모리 적용 → 단 한 번 저장은 `run`(#3703)
  의 원자 실행과 같은 규약이다. 선검증에 걸리면 한 칸도 쓰지 않고 `invalid[]` +
  exit 2 — 조용히 잘라내면 "표는 그럴듯한데 뒤쪽 데이터가 통째로 사라진" 보고서가
  남는다. `invalid[].reason` 4종:
  - `csvParse` — CSV 문법 오류(위치 포함)
  - `rowCountMismatch` / `colCountMismatch` — 행·열 수 불일치
  - `coveredCellNotEmpty` — 병합으로 덮인 칸에 값이 있음(쓸 수 없는 칸이다)
  - `controlCharacter` — 셀 안 줄바꿈·탭(`set_cell_control_char_rejection` 공유 판정)
- 좌표 해석은 `resolve_table_cell` 재사용, 쓰기는 `edit set-cell` 과 같은
  `delete_text_in_cell`/`insert_text_in_cell` 경로. **값이 실제로 달라지는 앵커
  칸만** 다시 쓴다.
- `--verify` 는 `edit_verify_report` 재사용(저장 직후 재파싱 IR 대조, 차이 시 exit 3).
- `changedPages` 는 `pages_covering_paragraphs` 재사용(표 호스트 문단이 걸친 쪽 전부).
- 산출 형식은 `edit_output_format` 재사용 — 입력 형식 보존(HWPX → HWPX).

#### set-cell 과 **다른** 결정 하나: 글자색을 덮지 않는다

`edit set-cell` 은 기본으로 셀 글자를 검정·비이탤릭으로 덮는다(#3391) — 빈 서식의
파란 이탤릭 안내문을 지우고 제출용 실값을 쓰는 축이기 때문이다. `csv-to-table` 은
반대로 **이미 서식이 잡힌 보고서의 값을 갱신**하는 축이라, 표 머리·강조 스타일을
일괄로 지우면 그 자체가 눈에 보이는 회귀다. 그래서 스타일을 보존한다. 빈 양식
채우기는 계속 `set-cell`/`fill-fields` 가 담당한다.

### 배선 (드리프트 가드 동시 충족)

- 명령 2개 dispatch + `--help` 절 2개
- `capabilities` 등재 2건(`table-to-csv`=export, `csv-to-table`=edit) — 선언한
  flags 는 전부 실제 파서가 받는다
- MCP 도구 2개 `hwp_table_to_csv` / `hwp_csv_to_table` — `inputSchema` 에
  `type`/`properties`/`required` 배열, 선언한 입력 속성은 전부 `cli.args` 자리표시자
  또는 `cli.optionalArgs.when` 에 배선(값 없는 `--bom`/`--dry-run`/`--verify` 는
  presence 플래그 형태)

## 검증

실행 원문은 [`evidence.txt`](evidence.txt). 요약:

| 게이트 | 결과 |
| --- | --- |
| `cargo build --release --bin rhwp` | Finished in 8m 06s, exit 0 |
| `cargo test --release --test table_csv_contract` | **14 passed; 0 failed** |
| `cargo test --release --test cli_json_contract` | 26 passed; 0 failed (무회귀) |
| `cargo test --release --test mcp_server_contract` | 22 passed; 0 failed (무회귀) |
| `cargo test --release --lib table_csv` | 8 passed; 0 failed (코어 단위) |
| `cargo clippy -- -D warnings` | Finished in 53.15s, exit 0 (경고 0) |
| `rustfmt --check` (변경 4파일) | 차이 없음 |

무회귀 중 이 변경이 직접 건드리는 가드 4종이 green 이다 —
`capabilities_covers_every_help_command` · `help_covers_every_capabilities_command` ·
`capabilities_mcp_covers_every_json_command` ·
`every_declared_input_property_is_wired_to_the_cli`.

### 실측 (evidence.txt 발췌)

1. **표 번호는 0 에서 시작하지 않는다** — 샘플은 표 53개 중 최상위 52개이고 첫
   최상위 `index` 는 **1**(0 번은 머리말 안의 표). 테스트도 상수 대신
   `export-tables` 가 보고한 실제 `index` 를 쓴다.
2. **병합 격자 채움** — 병합 있는 첫 최상위 표(index 2, 3×3, 앵커 6개)의 CSV 는
   레코드 3개, 레코드별 필드 수 `[3, 3, 3]`. 앵커만 이어 붙였다면 `[3, 2, 1]` 이
   되어 2행부터 열이 밀었을 자리다.
3. **왕복 무변경** — 뽑은 CSV 를 그대로 되넣으면 `changedCount: 0`,
   `verify: {"identical": true, "diffCount": 0}`, `changedPages: [1]`.
4. **한 칸 갱신** — `changedCount: 1`,
   `changed: [{"row":0,"col":0,"oldText":"","newText":"표값-2026"}]`,
   `changedPages: [1, 2]`. 저장본을 다시 CSV 로 뽑아 값이 실제로 들어갔음을 확인.
5. **조용한 절삭 없음** — 행 1개 부족 → `rowCountMismatch`, 덮인 칸에 값 →
   `coveredCellNotEmpty`, 따옴표 미종결 → `csvParse`. 셋 다 exit 2 이고 **출력
   파일이 생기지 않는다**.
6. **실패 경로 stdout 0바이트** — 인자 없음(exit 2)·없는 표 번호(exit 1) 모두
   stdout 0바이트.
7. **BOM** — `--bom` 이 파일 선두에 `EF BB BF` 를 넣지만 봉투의 `csv` 문자열에는
   붙지 않는다. `-o <폴더>` 로 최상위 표 52개가 표별 파일로 나온다.

## 남은 것

- 셀 안 줄바꿈: 내보내기는 RFC 4180 대로 인용해 그대로 내지만, 되넣기는 거부한다
  (`controlCharacter`). 다문단 셀에 문단 골격을 만들어 넣는 쓰기는 v1 범위 밖 —
  왕복 무손실을 원하면 이 축을 따로 승격해야 한다.
- 중첩 표·컨테이너(글상자·머리말) 안의 표는 두 명령 모두 대상이 아니다
  (`set-cell` 과 같은 v1 경계).
- 표 크기 변경(행 추가·삭제)은 이 명령의 계약이 아니다. 필요하면 별도 축이다.
