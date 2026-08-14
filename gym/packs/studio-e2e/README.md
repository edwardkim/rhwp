---
kind: guide
status: active
canonical: gym/packs/studio-e2e/README.md
last_verified: 2026-08-14
---

# studio-e2e — 스튜디오 e2e에서 파생한 CLI 검증 가능 문서 계약

## 왜 이 pack 인가 (온램프)

rhwp-studio 기여자는 편집 기능을 **브라우저 e2e 테스트**로 검증한다. 그런데 gym의
축은 **CLI 능력**이라, 그들의 e2e 검증은 gym에서 집계되지 않는다 — 두 검증 세계가
단절돼 있다. 이 pack은 그 다리다: **e2e가 브라우저에서 검증하는 문서-수준 계약 중
CLI로 재현 가능한 부분**을 gym 과제로 파생한다. 스튜디오 기여자의 편집 작업이
같은 코어를 CLI로도 두드리므로, 그 계약은 gym 과제로 **집계될 수 있다.**

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

## 정직한 경계 — e2e에만 남는 것

이 pack은 **문서 데이터 계약만** 채점한다. e2e의 나머지 계약 —
컨텍스트 메뉴 노출·더블클릭 다이얼로그·Ctrl+Z 스냅샷 undo·무편집 무흔적·비-차트
OLE 음성계약 — 은 **CLI로 표현할 수 없고 gym의 축(능력=CLI)이 아니라서** 파생하지
않는다. 그 부분은 계속 브라우저 e2e에서만 검증된다. 이 과제를 "e2e 전체를 공짜로"가
아니라 "**e2e의 데이터 계약을** CLI로"라고 읽어야 정확하다.

## 재현 (기준풀이 왕복 — 이 pack의 admission)

```bash
python gym/tools/build_baseline.py --agent baseline --pack studio-e2e --bin target/debug/rhwp
python gym/score.py               --agent baseline --pack studio-e2e --bin target/debug/rhwp
# → baseline: 3/3 (studio-e2e 3/3, 1/1 과제)
```

`assets/ST01-edit.csv`는 손으로 쓰지 않았다 — `chart-to-csv`로 실제 차트를 뽑아
(계열명·라벨·타 값이 정확히 맞는 CSV) 첫 데이터 칸만 91.7로 바꾼 것이다. `runner`
블록은 이 왕복을 검증한 바이너리 신원(v0.8.4)이다.

## 파생 자동화 — 어댑터 `gym/tools/from_e2e.mjs`

이 pack의 과제는 손으로 안 만든다. e2e에 계약 3줄만 있으면 어댑터가 기계 생성한다:

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

이것이 온램프의 핵심이다 — 스튜디오 기여자가 e2e를 쓰면 그 데이터 계약이 gym
과제를 **거의 공짜로** 낳는다. 강제가 아니라 유틸리티: 안 쓰면 손해라서 쓴다.
관련: 이슈 #4756(자연 온램프).
