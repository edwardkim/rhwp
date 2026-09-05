---
kind: working
status: active
issue: 5306
---

# 에이전트 표↔CSV 왕복 스킬 고도화 (#5306)

작업 브랜치: `feat/agent-table-exchange`
대상 스킬: `.agents/skills/rhwp-table-exchange/`
이슈: [agent: 표 CSV 왕복(table-exchange) 스킬 고도화](https://github.com/edwardkim/rhwp/issues/5306)

## 1. 한 줄

실사용 에이전트가 HWP/HWPX 표를 CSV 로 뽑아 스프레드시트에서 고친 뒤
같은 자리에 되돌리도록, 이미 devel 에 있는 `export-tables` ·
`table-to-csv` · `csv-to-table` · `edit set-cell` 을 문서·픽스처·시험으로
배선한다. 새 CLI 없음. DocumentCore 편집 로직 발명 없음. gym 없음.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- 치수 계약·병합 셀·dry-run/verify 실패 봉투·BOM/인코딩을 에이전트가
  **데이터로** 읽는 레시피·시험.
- `.agents/skills/rhwp-table-exchange/` SKILL.md + references/ +
  examples/ + fixtures.
- 기존 CLI 만: `export-tables`, `table-to-csv` (`--table` `--bom`),
  `csv-to-table` (`--dry-run` `--verify`).
- 병합 표는 기존 `edit set-cell` 만.
- `mydocs/working/agent_table_exchange.md` (이 파일).
- 계약 시험 (순수, 실픽스처).
- 추가 5000–10000줄, 최소 5000 (`git diff --shortstat upstream/devel`).
- PR 전 `cargo fmt --all -- --check`. rustfmt `newline_style=Unix`.
- isolation worktree, 브랜치 `feat/agent-table-exchange` from `upstream/devel`.
- 한국어 PR, base `devel`, `closes #5306`, `--body-file`.
- `git add -A` 금지. 이름 있는 파일만 stage.

금지:

- 새 CLI / DocumentCore 편집 로직 발명.
- gym/ 를 열지 않는다.
- 다른 스킬을 이 파동에서 고치지 않는다.
- 열려 있는 다른 PR 파일을 훔치지 않는다.
- 이름 있는 worktree 를 훔치지 않는다.

## 3. 왜 스킬만 키우나

K 트랙 표 왕복 스킬은 이미 있다. SKILL.md 한 장이 판단 트리와 함정
요약을 담고, 레시피 02 를 가리킨다.

부족한 것은 에이전트가 **기계로 따라갈 자식 문서**와 **같은 단어를
쓰는 픽스처**다.

- 좌표·병합 행렬이 `index`/`containerPath`/`rowSpan` 을 한 표로 모으지
  않으면 `--table 0` 습관이 머리말 표를 친다.
- `--bom` 이 파일에만 붙는다는 계약이 스킬 본문에 한 줄로만 있으면
  JSON `csv` 에 FEFF 를 다시 넣는 실수가 반복된다.
- exit 2/3 을 예외로 올리면 `invalid[]`·`verify.diffCount` 를 잃는다.
- 병합 표에 새 되돌리기 로직을 만들고 싶어 한다. 기존 `set-cell` 이면
  충분하다.

구현(`table_to_csv` / `csv_to_table` / `set_cell`)은 그대로다.
`tests/table_csv_contract.rs` · `tests/table_extract_json_contract.rs` 를
대체하지 않는다. 스킬 팩이 그 계약을 인용하는지만 본다.

## 4. 범위

만진 것:

| 경로 | 역할 |
|------|------|
| `.agents/skills/rhwp-table-exchange/SKILL.md` | 라우터. 30초 트리, 하지 않는 것 |
| `references/export_tables_matrix.md` | 좌표·병합·컨테이너·info 대비 |
| `references/table_to_csv_envelopes.md` | `--table`/`-o`/`--bom` 봉투 |
| `references/csv_to_table_contract.md` | 치수·covered·control·csvParse |
| `references/dry_run_verify.md` | dry-run / verify / exit 2·3 |
| `references/merged_table_fallback.md` | `edit set-cell` 만 |
| `references/pitfalls.md` | BOM·헤더·중첩·index |
| `references/failure_envelopes.md` | 종료 코드 분기 |
| `references/coordinate_index.md` | 좌표계 |
| `references/sample_transcripts.md` | 레시피 02·코덱스 20·playbook 실측 |
| `examples/01`–`18` + README | 워크스루 |
| `fixtures/**` | 봉투·CSV·행렬·루프·트랜스크립트 |
| `scripts/tests/test_agent_table_exchange.py` | 파일 계약 (바이너리 없음) |
| `mydocs/working/agent_table_exchange.md` | 이 기록 |

만지지 않은 것:

- `gym/`
- `.agents/skills/` 의 다른 스킬
- `src/document_core/`
- 새 `[[bin]]` / 새 CLI 하위명령
- 열려 있는 형제 PR 의 파일 (`rhwp-safe-edit` 등)

`tests/` 신규 integration 은 suite-policy 상 `tests/cases/` 만 허용이라
이 파동은 Python 시험을 정본으로 둔다. 엔진 계약을 다시 돌리지 않는다.

## 5. 표본과 정본

트랜스크립트는 다음에서 옮겼다. 이 파동이 바이너리를 재실행하지 않았다.

| 정본 | 쓰는 숫자 |
|------|-----------|
| `mydocs/manual/recipes/02_table_csv_roundtrip.md` | hwp_table_test 0번 3×4, changedCount 9 |
| `mydocs/manual/agent_codex/20_표와_데이터.md` | issue2007 tableCount 5, `--table 1` |
| `mydocs/manual/agent_surface_playbook.md` §10-5 | table-001 19×9, row/col mismatch |
| `mydocs/manual/agent_knowledge_map.md` §7-1 | 표본 특성 표 |
| `tests/table_csv_contract.rs` | BOM, RFC, invalid reason, dry-run |
| `tests/table_extract_json_contract.rs` | 병합 보존, nested, treatise 3 |

`csv-to-table` 의 `untrustedContent` 는 계약
(`changed[].oldText`)을 따른다. 레시피 02 원문의 `false` 판본보다
지식지도가 이긴다.

## 6. 시험

```bash
python -m unittest scripts.tests.test_agent_table_exchange
```

바이너리 없이 문서·픽스처만 가드한다.

- 레이아웃·frontmatter·형제 스킬 존재(본문 비결합)
- 레퍼런스 계약 토큰
- catalog ↔ 파일 1:1
- 봉투 `_skillMeta.exit` 와 reason
- dry-run `changedPages: null`
- verify 실패는 exit 3 + 산출 유지
- set-cell 덮인 칸은 stdout 0
- `rhwp` 머리 명령 allowlist
- `rhwp edit` 는 `set-cell` 만
- gym 경로에 산출물 없음

## 7. fmt 게이트

`cargo fmt --all -- --check` 를 PR 전에 돌린다. 이 파동은 Rust 소스를
추가하지 않으므로 rustfmt 가 고칠 파일이 없어야 한다.
`rustfmt.toml` 의 `newline_style = "Unix"` 를 따른다.

## 8. 줄 수

목표는 5000–10000 추가 줄, 최소 5000.
실레시피·픽스처·트랜스크립트·시험·문서다. 주석 패딩·lorem 복제가 아니다.

커밋 후 `git diff --shortstat upstream/devel...HEAD` 로 확인한다.
