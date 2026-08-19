---
kind: working
status: active
---

# rhwp-agent 표 격자 실측

이 묶음은 `samples/` 안의 실제 HWP/HWPX 를 `rhwp-agent` 로 열어 얻은 조회 봉투다.
더미 행을 만들지 않았다. 값은 명령 stdout 에서 왔다.

## 한 줄

tables·table-inspect·table-csv·merged-tables 를 표본마다 실행하고, 나온 쪽·칸·누름틀·수확 항목을 검증 핀으로 남긴다.

## 계약

- 본 CLI(`src/main.rs`) 를 건드리지 않는다.
- 편집 로직을 만들지 않는다.
- `--json` 봉투의 schemaVersion·command·untrusted* 를 그대로 저장한다.
- 열리지 않는 파일은 exit 와 stderr 만 기록한다.

## 명령

tables·table-inspect·table-csv·merged-tables

## 표본

- 시도: 699
- 봉투를 얻은 파일: 692

대표 고정 표본: `samples/form-01.hwp`, `samples/hwp3-sample.hwp`, `samples/hwp_table_test.hwp`.

## 재실행

```
python tools/agent_harvest/tables/harvest_goal.py --pack tables
python tools/agent_harvest/tables/test_replay.py
```

재실행은 고정 표본 3개만 다시 열어 봉투 키가 비지 않는지 본다. 전 표본 수확은 이 디렉터리의 goldens/ 가 정본이다.
