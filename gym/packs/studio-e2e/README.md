---
kind: guide
status: active
canonical: gym/packs/studio-e2e/README.md
last_verified: 2026-08-18
---

# studio-e2e — 스튜디오 e2e에서 파생한 CLI 검증 가능 문서 계약

## 왜 이 pack 인가 (온램프)

rhwp-studio 기여자는 편집 기능을 **브라우저 e2e 테스트**로 검증한다. 그런데 gym의
축은 **CLI 능력**이라, 그들의 e2e 검증은 gym에서 집계되지 않는다 — 두 검증 세계가
단절돼 있다. 이 pack은 그 다리다: **e2e가 브라우저에서 검증하는 문서-수준 계약 중
CLI로 재현 가능한 부분**을 gym 과제로 파생한다. 스튜디오 기여자의 편집 작업이
같은 코어를 CLI로도 두드리므로, 그 계약은 gym 과제로 **집계될 수 있다.**

ST01 한 줄만 있으면 에이전트는 "첫 칸을 91.7 로 바꾸면 축 전체"라고 학습한다.
같은 `chart-to-csv` / `csv-to-chart` 라도

- 지목 칸이 계열 0 인가 1 인가 2 인가, 첫 값인가 마지막 값인가
- 입력이 HWP 인가 HWPX 인가
- 읽는 자리가 `chartCount` 인가 `rowCount` 인가 `colCount` 인가
- 산출이 편집본인가 CSV 시트인가, BOM 을 붙이는가
- 분산형의 첫 칸이 빈 라벨인가 `X` 인가

가 다른 계약이다. 과제를 갈라 두면 자리를 다시 지목해야 한다.

새 CLI 는 없다. 기존 `chart-to-csv` · `csv-to-chart` 와 `samples/chart/` ·
`samples/issue2006/` 만 쓴다. `pack.json` 의 `requires.commands` 와 `runner`
신원은 그대로 둔다.

## ST01 — 차트 데이터 편집 (studio #4694 파생)

- **출처 e2e**: `rhwp-studio/e2e/issue-4694-chart-data-edit.test.mjs`
- **같은 코어**: e2e의 `window.__wasm.getChartDataByIndex`/`setChartDataByIndex`와
  CLI `chart-to-csv`/`csv-to-chart`는 **동일한** `get/set_chart_data_by_index_native`를
  구동한다(`src/main.rs:6362,7055` · `src/wasm_api.rs:3790`). 그래서 e2e의 데이터
  계약이 CLI로 충실히 왕복한다.
- **파생한 계약 (문서 데이터만)**: 샘플 `chart/세로막대형/묶은세로막대형.hwp`,
  차트 1, `series[0].values[0]`을 **4.3 → 91.7**로 편집(e2e의 `SENTINEL`과 동일).
- **채점**: `file_exists` + `differs_from_input`(무편집 복사 거부) + `value_eq`
  (같은 CSV 재적용 시 `changedCount==0` = 산출물이 이미 목표값을 담음). 전부
  CLI 봉투 재계산 — 라이브 오라클, 골든 파일 없음.

## 여정 지도

### J1. 칸을 지목해 되돌린다 (`csv-to-chart`)

같은 묶은세로막대 시트라도 칸이 다르면 다른 과제다. HWPX 쌍은 산출 확장자가
`.hwpx` 다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| ST01 | 계열 0 · 값 0 · 4.3→91.7 | `묶은세로막대형.hwp` | `changedCount==0` |
| ST02 | 계열 1 · 값 0 · 2.4→88.1 | 같은 HWP | 둘째 계열 첫 칸 |
| ST03 | 계열 2 · 값 0 · 2→77.3 | 같은 HWP | 셋째 계열 첫 칸 |
| ST04 | 계열 0 · 값 1 · 2.5→66.2 | 같은 HWP | 첫째 계열 둘째 행 |
| ST05 | 계열 1 · 값 3 · 2.8→55.9 | 같은 HWP | 둘째 계열 마지막 행 |
| ST06 | ST01 과 같은 칸, HWPX | `묶은세로막대형.hwpx` | `out.hwpx` |
| ST07 | ST02 와 같은 칸, HWPX | 같은 HWPX | `out.hwpx` |

**실패 모드**

- ST01 의 91.7 만 외워 모든 편집 과제에 넣는다.
- HWPX 입력에 `out.hwp` 를 낸다. ST06·ST07 은 확장자가 계약이다.
- 계열 이름·카테고리 라벨을 바꾼다. 크기·이름은 범위 밖이고 한 칸도 쓰이지 않는다.
- 원본을 복사한다. `differs_from_input` 이 거절한다.

### J2. 봉투를 읽는다 (`chart-to-csv --json`)

숫자를 파일 이름에서 추측하지 않는다. 채점은 라이브 오라클이다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| ST08 | 한 장 지목 차트 수 | `묶은세로막대형.hwp` | `chartCount` (`--chart 1`) |
| ST09 | 행 수 | 같은 HWP | `charts[0].rowCount` |
| ST10 | 열 수 | 같은 HWP | `charts[0].colCount` |
| ST11 | HWPX 차트 수 | `묶은세로막대형.hwpx` | `chartCount` |
| ST12 | HWPX 행 수 | 같은 HWPX | `rowCount` |
| ST13 | 가로막대 차트 수 | `묶은가로막대형.hwp` | `chartCount` |
| ST14 | 꺾은선 차트 수 | `꺽은선형.hwp` | `chartCount` |
| ST15 | 원형 차트 수 | `2차원원형.hwp` | `chartCount` |
| ST16 | 분산형 차트 수 | `직선이있는분산형.hwpx` | `chartCount` |
| ST17 | 실사용 보고서 전부 | `1790387_prep_final_report.hwpx` | `chartCount` (지목 없음) |
| ST18 | 보고서 첫째 행 수 | 같은 보고서 | `--chart 1` `rowCount` |
| ST19 | 보고서 둘째 열 수 | 같은 보고서 | `--chart 2` `colCount` |
| ST20 | 누적세로 차트 수 | `누적세로막대형.hwp` | `chartCount` |
| ST21 | 3D 묶은세로 차트 수 | `3차원묶은세로막대형.hwp` | `chartCount` |
| ST31 | 첫 차트 번호 | `묶은세로막대형.hwp` | `charts[0].chart` |
| ST32 | 전부 추출 차트 수 | 같은 HWP | `chartCount` (지목 없음) |
| ST33 | 누적가로 HWPX | `누적가로막대형.hwpx` | `chartCount` |
| ST34 | 표식 꺾은선 | `표식이있는꺽은선형.hwp` | `chartCount` |
| ST35 | 쪼개진 원형 | `쪼개진원형.hwp` | `chartCount` |
| ST36 | 3D 원형 HWPX | `3차원원형.hwpx` | `chartCount` |

**실패 모드**

- 차트 번호를 0 부터 센다. 이 명령은 **1부터**다. `--chart 0` 은 없다.
- 행 수를 카테고리 라벨 수로 센다. 값이 행을 정한다. ST18 이 그 회귀다.
- 열 수에 라벨 열을 포함한다. `colCount` 는 계열 수다.
- ST08 답을 ST11·ST13·ST17 에 복사한다. 표본이 바뀌면 다시 읽어야 한다.
- 조각 수·3D 여부·표식 유무를 `chartCount` 로 적는다. 스타일이 아니라 장 수다.

### J3. 시트를 파일로 남긴다 (`chart-to-csv -o`)

stdout 만 보고 제출을 빼먹으면 `file_exists` 가 떨어진다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| ST22 | 계열 머리 | `묶은세로막대형.hwp` | `csv_cell_eq` (0,1)=`계열 1` |
| ST23 | 항목 라벨 | 같은 HWP | `csv_cell_eq` (1,0)=`항목 1` |
| ST24 | HWPX 시트 | `묶은세로막대형.hwpx` | 파일 존재 |
| ST25 | 분산형 X 열 | `직선이있는분산형.hwpx` | `csv_cell_eq` (0,0)=`X` |
| ST26 | BOM | `묶은세로막대형.hwp` | `utf8_bom` |
| ST27 | 꺾은선 시트 | `꺽은선형.hwp` | 파일 존재 |
| ST28 | 원형 시트 | `2차원원형.hwp` | 파일 존재 |
| ST29 | 가로막대 시트 | `묶은가로막대형.hwp` | 파일 존재 |
| ST30 | 보고서 첫째 + 행 수 | `1790387_…hwpx` | 파일 + `rowCount` |
| ST37 | 백분율 누적 시트 | `백프로기준누적세로막대형.hwp` | 파일 존재 |
| ST38 | 주식형 시트 | `고가저가종가.hwp` | 파일 존재 |

**실패 모드**

- 분산형 첫 칸을 비운다. ST25 는 `X` 다.
- `--bom` 없이 BOM 과제를 낸다. 봉투의 `csv` 문자열에는 BOM 이 없다.
- 백분율 누적을 100 으로 환산한다. CSV 는 원본 숫자다.
- 원본 HWP 를 `chart.csv` 자리에 둔다. 최소 바이트와 셀 검사가 거절한다.

### J4. 하한만 본다 (`value_ge` · `len_ge`)

정확한 개수를 박제하지 않는 입문 과제다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| ST39 | 차트 수 하한 | `묶은세로막대형.hwp` | `chartCount >= 1` |
| ST40 | 배열 길이 하한 | 같은 HWP | `len(charts) >= 1` |

**실패 모드**

- `{"ok": true}` 를 손으로 적는다. 하한은 라이브 봉투다.
- ST39 의 숫자 자리와 ST40 의 배열 자리를 같은 검사로 취급한다.

## 정직한 경계 — e2e에만 남는 것

이 pack은 **문서 데이터 계약만** 채점한다. e2e의 나머지 계약 —
컨텍스트 메뉴 노출·더블클릭 다이얼로그·Ctrl+Z 스냅샷 undo·무편집 무흔적·비-차트
OLE 음성계약 — 은 **CLI로 표현할 수 없고 gym의 축(능력=CLI)이 아니라서** 파생하지
않는다. 그 부분은 계속 브라우저 e2e에서만 검증된다. 이 과제를 "e2e 전체를 공짜로"가
아니라 "**e2e의 데이터 계약을** CLI로"라고 읽어야 정확하다.

차트 번호는 문서 순서 **1부터**다. `export-tables --table` 의 0 기준과 섞지 마라.

## 재현 (기준풀이 왕복 — 이 pack의 admission)

```bash
python gym/tools/build_baseline.py --agent baseline --pack studio-e2e --bin target/debug/rhwp
python gym/score.py               --agent baseline --pack studio-e2e --bin target/debug/rhwp
```

`assets/ST01-edit.csv` 와 ST02–ST05·ST07 자산은 손으로 창작하지 않았다.
알려진 묶은세로막대 시트(계열 3 × 값 4)에서 계약이 지목한 한 칸만 바꿨다.
ST06 은 ST01 자산을 HWPX 입력에 재사용한다. `runner` 블록은 기존 왕복을
검증한 바이너리 신원(v0.8.4)이며 이 확장에서 갱신하지 않는다.

## 파생 자동화 — 어댑터 `gym/tools/from_e2e.mjs`

이 pack의 **첫 과제(ST01)** 는 손으로 안 만든다. e2e에 계약 3줄만 있으면
어댑터가 기계 생성한다:

```js
// rhwp-studio/e2e/issue-4694-chart-data-edit.test.mjs 안의 단일 출처
export const gymContract = {
  sample: 'chart/세로막대형/묶은세로막대형.hwp',
  chart: 1,
  edit: { series: 0, point: 0, from: '4.3', to: '91.7' }, // series[0].values[0]
};
```

```bash
node gym/tools/from_e2e.mjs \
  --e2e rhwp-studio/e2e/issue-4694-chart-data-edit.test.mjs \
  --pack studio-e2e --id ST01 --bin target/debug/rhwp
# → assets/ST01-edit.csv · tasks/ST01.json · reference/ST01.json 을 생성
```

어댑터는 편집 CSV를 손으로 쓰지 않는다 — `chart-to-csv`로 실제 차트를 뽑아 계약이
지정한 한 칸만 바꾼다(형태 맞추기를 rhwp에 시킨다 = gym 라이브 오라클과 같은 원리).
설계 함정 둘을 실측으로 회피했다: ① e2e는 top-level에서 `runTest`를 돌리므로
`import`가 아니라 **무실행 정적 parser**로 계약 리터럴만 읽는다(브라우저 기동과 임의 코드 실행 방지),
② `chart-to-csv --json`은 순수 JSON이라 머리줄 strip 없이 `charts[0].csv`를 쓴다.

ST02 이후는 같은 코어·같은 표본 가족을 **자리만 갈라** 늘린 것이다. 새 e2e
`gymContract` 가 생기면 어댑터로 또 한 줄을 낳으면 된다. 강제가 아니라 유틸리티:
안 쓰면 손해라서 쓴다. 관련: 이슈 #4756(자연 온램프), #5262(이번 확장).
