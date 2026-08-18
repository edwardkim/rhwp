---
kind: guide
status: active
canonical: gym/docs/coverage.md
last_verified: 2026-08-18
---

# batch-ops — 서식 1개와 데이터 N행으로 문서 N부를 만든다

이 pack은 **자동화** 축이다. `batch fill` 이 진짜 메일머지다. stdin 목록을
읽지 않는 유일한 batch 축이라 gym 러너가 그대로 구동한다.

`batch` 는 NDJSON(행당 레코드 1개)을 내므로 단일 봉투 모델로는 값을 길로
캐지 못한다. 채점은 산출물 축이다 — 기준풀이가 `batch fill` 로 out-dir 를
채우고, 채점은 산출 파일 존재와 각 문서에 그 행의 병합값이 들어갔는지를
단건 `search` 로 재검증한다.

T07 `edit fill-fields` 가 아니다. T07 은 서식 한 부에 값을 한 번 넣는다.
여기는 서식 1 + 데이터 N → 문서 N 부다.

## 조합 표

세 축을 갈라 놓았다. 한 조합만 있으면 나머지를 추측한다.

| ID | 서식 | 데이터 | 이름 | 확인 토큰 |
|---|---|---|---|---|
| BO01 | HWPX | CSV 3행 | 순번 | 계약서 / 신규 |
| BO02 | HWPX | CSV 2행 | `--name-field` | AlphaMerge / BetaMerge |
| BO03 | HWP5 | JSONL 2행 | 순번 | JsonlAlpha / JsonlBeta |
| BO04 | HWPX | JSONL 2행 | 순번 | HwpxJsonlOne / Two |
| BO05 | HWP5 | CSV 2행 | 순번 | Hwp5CsvAlpha / Beta |
| BO06 | HWP5 | CSV 2행 | `--name-field` | Hwp5NameA / B |
| BO07 | HWPX | CSV 3행 | `--name-field` | Gamma / Delta / Epsilon |
| BO08 | HWPX | CSV 4행 | 순번 | QuadMergeA / D |
| BO09 | HWP5 | JSONL 2행 | 순번 | Hwp5JsonlRed / Blue |
| BO10 | HWPX | CSV 3행 | 순번 | TripleHwpxOne / Three |
| BO11 | HWPX | CSV 2행 | `--name-field` | Zeta / Eta |
| BO12 | HWP5 | CSV 3행 | 순번 | Hwp5TripleA / C |

서식은 `samples/hwpx/form-01.hwpx` 또는 `samples/form-01.hwp` 다. 누름틀
이름은 `myMsg01` 하나다. 데이터 파일은 `gym/packs/batch-ops/assets/` 에
있다.

## 실패 모드

- `edit fill-fields` 로 한 부를 만들어 `0001` 만 제출한다. 2부·3부
  `file_exists` 가 떨어진다.
- 입력 서식을 `out/0001.hwpx` 로 복사한다. `differs_from_input` 이
  거절한다.
- `--name-field` 를 빼서 순번 파일로 제출한다. BO02·BO06·BO07·BO11 은
  필드 값 이름을 요구한다.
- `--name-field` 를 붙여 놓고 `0001.hwpx` 를 찾는다. 그 파일은 없다.
- 서식이 `.hwp` 인데 산출을 `.hwpx` 로 낸다. 확장자는 서식을 따른다.
- JSONL 을 CSV 로 읽는다. `.jsonl` 은 한 줄이 한 행, 객체 키는 누름틀
  이름이다.
- 같은 토큰을 두 부에 넣는다. `search` 는 그 부에 그 토큰이 있는지만
  본다. 토큰이 겹치면 판별이 안 된다.

## 힌트 한 줄

```bash
rhwp batch fill --form samples/hwpx/form-01.hwpx \
  --data gym/packs/batch-ops/assets/BO01-data.csv \
  --out-dir out --json
```

이름 필드:

```bash
rhwp batch fill --form samples/hwpx/form-01.hwpx \
  --data gym/packs/batch-ops/assets/BO02-data.csv \
  --out-dir out --name-field myMsg01 --json
```

## 러너 신원

`pack.json` 의 `runner` 는 이 확장이 만지지 않는다. 요구 명령은
`batch` · `search` 다.
