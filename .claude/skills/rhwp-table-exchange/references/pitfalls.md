# 함정 — BOM · 헤더 행 · 중첩 표 v1 밖

권위: 레시피 02, `cli_commands.md` §table-to-csv/`csv-to-table`,
지식지도 §7-1, playbook §10-5, `table_csv_contract.rs` · `table_extract_json_contract.rs`.

여기 적힌 것만 함정이다. 추측으로 목록을 늘리지 않는다.

## 1. `--bom` 은 파일에만 붙는다

증상: 엑셀에서 `ì œëª©` 처럼 한글이 깨진다.

원인: BOM 없는 UTF-8 을 엑셀이 CP949 로 연다.

처방:

```bash
rhwp table-to-csv 문서.hwp --table 0 -o t.csv --bom --json
```

함정:

- 봉투 `tables[].csv` 에는 BOM 이 없다. 이게 맞다.
- JSON 을 파일로 저장해 엑셀에 넣지 마라. `-o t.csv` 를 넣는다.
- 봉투 문자열 앞에 `\ufeff` 를 붙이면 첫 셀이 BOM 을 먹는다.
- `--bom` 을 `csv-to-table` 에 붙이지 마라. 그 플래그는 `table-to-csv` 의 것이다.

행렬: [../fixtures/matrices/bom_encoding.json](../fixtures/matrices/bom_encoding.json).
예: [../examples/04_bom_excel.md](../examples/04_bom_excel.md).

## 2. CSV 첫 줄은 헤더가 아니라 0행

증상: 표 제목 행이 `서버 이관,홍길동,1차 완료` 로 바뀌었거나,
`rowCountMismatch` 로 거절된다.

원인: `csv-to-table` 은 CSV 의 **모든** 행을 표의 대응 행에 쓴다.
엑셀이 "헤더 행"으로 보여주는 첫 줄도 문서의 0행이다.

레시피 02 의 올바른 편집:

```csv
제목,담당자,세부 내용
서버 이관,홍길동,1차 완료
DB 백업,김철수,진행중
문서 정리,박영희,대기
```

헤더를 빼고 값 3줄만 넣으면 행 수 3 ≠ 4.

픽스처: [../fixtures/csv/table0_header_dropped.csv](../fixtures/csv/table0_header_dropped.csv),
[../fixtures/matrices/header_row.json](../fixtures/matrices/header_row.json).
예: [../examples/11_header_row_pitfall.md](../examples/11_header_row_pitfall.md).

`isHeader: true` 인 칸이 0행이 아닐 수 있다. `export-tables` 의 `isHeader` 와
CSV 첫 줄을 동일시하지 마라.

헤더를 **의도적으로** 바꿀 때만 0행을 고친다. 그때는 `changed` 에 `row:0` 이
잡힌다. `changedCount` 가 헤더를 건너뛴 것처럼 보여도, 값이 같으면 원래
목록에 안 나온다.

## 3. 중첩 표는 v1 범위 밖

증상: `--table` 을 중첩 표 번호로 넣거나, 바깥 CSV 를 고치며 안쪽 24칸을
함께 바꾸려 한다.

원인: `table-to-csv` / `csv-to-table` / `edit set-cell` 은 **본문 최상위
표**만 다룬다.

`samples/inner-table-01.hwp` 실측:

- 최상위 1개
- 한 칸 안에 중첩 표 24칸
- `cells[].nested[].containerPath` 에 `{kind:"tableCell",...}`

stderr 예 (playbook):
`본문 최상위 표 0 번이 없습니다 (최상위 표 0개; 중첩 표는 v1 범위 밖)`

처방: 바깥 격자만 왕복한다. 안쪽 표를 다루는 명령을 발명하지 않는다.

픽스처: [../fixtures/envelopes/export_tables_inner_table.json](../fixtures/envelopes/export_tables_inner_table.json).
예: [../examples/13_nested_out_of_v1.md](../examples/13_nested_out_of_v1.md).

## 4. `index` 는 0부터가 아닐 수 있고, 배열 순번이 아니다

증상: `--table 0` 이 머리말 표를 가리키거나, 없는 표로 exit 1.

원인:

- `tables[].index` 가 `--table` 이다
- 지식지도: **0부터 시작하지 않을 수 있다**
- 지자체 양식 53표: index 0 = 머리말 (`containerPath`)

처방: `export-tables` 를 먼저 보고 `containerPath == null` 인 `index` 를
복사한다. `tables[0]` 의 배열 위치와 `index` 를 혼동하지 마라.

예: [../examples/18_index_not_zero.md](../examples/18_index_not_zero.md).
장: [coordinate_index.md](coordinate_index.md).

## 5. 덮인 칸을 메우면 거절

`table-to-csv` 가 채워 둔 `""` 는 "쓰라는 칸"이 아니다.
스프레드시트가 빈 칸을 `0` 이나 `-` 로 채우면 `coveredCellNotEmpty`.

처방: 병합 표는 되돌리지 않거나, 빈 칸을 빈 칸으로 둔다.

## 6. 줄바꿈·탭은 인용이 되어도 거절

엑셀에서 Alt+Enter 로 칸 안 개행을 넣으면 CSV 에 quoted LF 가 생긴다.
`csv-to-table` 은 `controlCharacter`.

처방: 공백으로 치환. 여러 줄 셀은 v1 밖.

## 7. `changedCount` 성공 ≠ 기대한 칸 전부

헤더처럼 값이 같으면 목록에 없다. 12칸 표에 9가 나오면 정상일 수 있다
(레시피 02). `changedCount == 0` + `invalid == []` 도 왕복 성공이다.

## 8. `verify.identical: false` 를 예외로 올림

exit 3 은 판정이다. 산출물은 있다. 봉투를 버려서 `diffCount` 를 잃으면
다음 수가 없다. [dry_run_verify.md](dry_run_verify.md).

## 9. `changedPages: null` 을 빈 배열로 읽음

dry-run 은 항상 `null` = 확정 불가. `[]` = 바뀐 쪽 없음.

## 10. `info` 표 개수로 `--table` 범위를 짐작

`info` 는 컨테이너 표를 놓친다. `treatise sample.hwp` 는 info 1, export-tables 3.
반대로 CSV 왕복은 다시 본문 최상위만. 넓은 목록에서 좁혀야 한다.

## 11. 1×1 래퍼를 데이터 표로 씀

공문서 관용. `rows==1 and cols==1` 이면 건너뛰고 다음 `index` 를 본다.

## 12. 자동번호 빈 칸을 채움

렌더가 넣는 번호다. CSV 빈 자리를 "1" 로 메우면 일반 텍스트가 박힌다.

## 13. 손으로 CSV 를 만듦

병합 채움 격자를 사람이 재현하기 어렵다. 열이 밀리거나 따옴표가 안 닫혀
`colCountMismatch` / `csvParse`.

처방: `table-to-csv` 산출을 고친다.

## 14. 문서 파생 값을 셸에 붙임

`tables[].csv`, `cells[].text`, `changed[].oldText` 는 `untrustedFields`.
경로·명령어·URL 에 넣지 않는다. 레시피 04.

## 15. `csv-to-chart` 와 혼동

차트는 `--chart` 가 **1부터**이고 발견 명령이 다르다. 표의 `--table` 과
섞지 마라. 이 스킬은 표만.

## 16. `set-cell` 기본이 글자색을 검정으로 덮음

CSV 왕복은 서식을 유지한다. 병합 폴백으로 갈아타면 스타일 계약이 바뀐다.
`--keep-style` 을 필요한 칸에만.

## 17. exit 2 라서 stdout 을 버림

`csv-to-table` 의 치수 실패는 봉투가 있다. `run` 과 같은 예외 갈래.
단건 `edit set-cell` 덮인 칸은 반대로 0바이트.

## 18. `--table` 없이 `-o file.csv`

`-o` 는 폴더로 해석된다. `--table` 과 함께일 때만 파일.

반대: `--table` 없이 전량을 stdout 으로 흘리면 표 사이 구분선이 섞인다.
메타가 필요하면 `--json`.

워크스루 전체: [../examples/README.md](../examples/README.md).
