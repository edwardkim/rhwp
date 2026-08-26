# hwp5_inventory — 저장 계약 인벤토리 픽스처 (M-hwp5)

`rhwp hwp5-inventory` / `hwp5-inventory-diff` / `hwp5-table-probe` 가 쓰는
레코드 언어를 Python 으로 다시 적어, oracle/generated 픽스처와 리포트 전사를
고정한다. 이슈 [#5469](https://github.com/edwardkim/rhwp/issues/5469).

시리얼라이저 페이지 수 로직은 범위 밖이다. `#4882` 석이 맡는다.

## 한 줄

```text
python tools/hwp5_inventory/fatten_catalog.py
python -m unittest tools.hwp5_inventory.tests.test_fatten_catalog tools.hwp5_inventory.tests.test_model tools.hwp5_inventory.tests.test_cases tools.hwp5_inventory.tests.test_transcripts
```

바이너리 HWP 를 열지 않는다. `rhwp` 빌드가 필요 없다.

## 명령 계약 (devel)

| 명령 | 역할 |
|---|---|
| `hwp5-inventory` | DocInfo/BodyText record 를 안정 행으로 푼다 |
| `hwp5-inventory-diff` | oracle/generated 를 index 또는 LCS 로 맞춘다 |
| `hwp5-table-probe` | TABLE 네 축을 이식한 probe HWP 를 만든다 |

`--report` : `diff` · `hints` · `bundles` · `table-fields` · `table-probe-plan`
`--focus` : `all` · `table` · `shape` · `ctrl` · `missing` · `docinfo`
`--align` : `index` · `lcs` (중간 삽입은 `lcs`)

종료 코드: 인자 없음 = 2, `--help` = 0, 파일 없음 = 1.
stdout 은 데이터, 사용법은 stderr.

## 이 패키지가 하는 일

1. 태그·컨트롤·테이블 필드·실패 유형 A–F 카탈로그를 JSONL 로 쓴다.
2. 50+ 계약 단위(표 여백/attr/tail, BinData, lineSeg, 필드 fourcc 등)를
   oracle/generated 인벤토리로 펼친다.
3. index 와 LCS diff, hints/bundles/table-fields/table-probe-plan 전사를
   CLI 제목과 같은 말로 남긴다.
4. table-probe 8 variant 의 이식 횟수를 픽스처로 계산한다. HWP 바이트는 쓰지 않는다.

## 하지 않는 일

- `src/serializer` 페이지 수 로직
- `canvaskit_policy` · `pdf` · `layout-anomaly` · `oracle_public` · `render_backend` · `proptest` · `fidelity_compare`
- `gym/`
- 새 rhwp CLI 명령
- DocumentCore 편집 로직

## 읽는 순서

1. `fixtures/cli_contract.json` — 옵션·종료 코드
2. `fixtures/failure_classes.json` — A–F
3. `reports/pair_index.md` — 케이스 표
4. `transcripts/inventory_diff/<id>.hints.md` — 다음 probe
5. `mydocs/working/hwp5_inventory_fatten.md`

정본 도구 설명은 `mydocs/manual/document_diagnostics_tool_manual.md` §6–10,
규칙 문서는 `mydocs/troubleshootings/hwpx2hwp-rule.md`.
