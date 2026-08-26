# M-tbl: 표 CSV 왕복 치수·병합 픽스처 고도화 (#5485)

## 무엇을

MEGA QUEUE M-tbl. devel 에 있는 `export-tables` · `table-to-csv` ·
`csv-to-table` 계약을 치수 거부 · `coveredCellNotEmpty` · `--dry-run` /
`--verify` 픽스처로 풀어 놓는다.

구현 위치: `tools/table_exchange/`.

## 왜

스킬 #5306 이 배선한 CLI 를 에이전트가 데이터로 읽으려면, 행/열 불일치·
덮인 칸·dry-run 의 `changedPages:null`·verify exit 3 산출 유지가
케이스마다 같은 단어로 고정돼 있어야 한다. 새 편집 명령을 만들지 않는다.

## 어떻게

Python 모델이 RFC 4180 과 점유 행렬을 독립 구현하고, 기존 종료 코드
0/1/2/3 과 `invalid[]` 이유 다섯 개를 수집한다. `fatten_catalog.py` 가
원장·JSONL·쇼케이스 봉투를 다시 쓴다. `rhwp` 바이너리는 부르지 않는다.

## 검증

```
python tools/table_exchange/fatten_catalog.py
python -m unittest discover -s tools/table_exchange/tests -t tools
cargo fmt --all -- --check
```

unittest 41 passed.

## 하지 않은 것

- DocumentCore / `src/` 미수정
- gym/ 미수정
- 다른 진행 석 (`fidelity_compare`, `hwp5_inventory`, form-fill, work-receipt) 미수정
- 병합 풀기·표 리사이즈 발명 없음
