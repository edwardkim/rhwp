---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 10. operator 길 — 폴더를 훑고 사다리를 오른다

`operator` 는 `corpus-diagnostics` 와 `automation` 을 고른다. 정본은
`gym/profiles/operator.json`. 한 문서의 숫자에서 폴더와 계획으로
시야를 넓힌다.

돌아가기: [README.md](README.md) · 프로파일: [06-profiles.md](06-profiles.md)

```bash
python gym/score.py --agent 나 --profile operator
```

## CD01 — 폴더 스윕 계수

정본: `gym/packs/corpus-diagnostics/tasks/CD01.json`

입력 필드는 `samples/table-001.hwp` 이지만, 실제로 세는 대상은
`samples/hml` 폴더다. 과제가 `rhwp scan` 을 그 폴더에 돌리라고 적었다.

```bash
rhwp scan samples/hml --json
mkdir -p gym/submissions/나/corpus-diagnostics/CD01
```

답 키는 `files`. 연산자는 `len_answer_eq` 이고 오라클 경로는
`files` 배열이다. 폴더에 파일이 늘면 수가 바뀐다. 그래서 골든을
박제하지 않는다.

## AU01 — 계획서 원자 실행

정본: `gym/packs/automation/tasks/AU01.json`

첫 표 (0, 0) 을 '계획실행' 으로 바꾸는 `run` 계획서를 실행하고
`out.hwp` 를 낸다.

채점 두 칸:

1. `cell_text_eq` — `export-tables` 로 본 (0, 0) 이 `계획실행`
2. `differs_from_input` — 원본 복사가 아님

`cell_text_eq` 는 표 좌표를 지목한다. `cells[0]` 순서 가정이 아니다.
그 이유는 `gym/core/checks.py` 의 `find_cell` 주석(#4600)에 있다.
휴게실이 그 함수를 고치지 않는다.

힌트: `rhwp run --plan-json`. 계획서 한 장이 검증을 포함하고, 실행은
원자여야 한다. 에이전트 작업 표준의 영수증 축(AW-L1)을 운동장이
과제로 소비하는 입구다.

```bash
mkdir -p gym/submissions/나/automation/AU01
```

## 사다리 은유와 실제 pack

[../PARK.md](../PARK.md) 는 automation 을 "검증 사다리 10단" 으로
그린다. 그 은유가 점수를 바꾸지 않는다. 과제는 AU01 부터 AU13 까지
이미 등재된 것들이다. 이 안내가 14번째 과제를 끼워 넣지 않는다.
다른 열린 PR 이 automation 과제를 늘리고 있으면 그 파일은 그 PR 의
것이다.

## operator 다음에

보스존은 사다리를 **한 체인으로** 묶는다. 부분 점수가 없다.
[11-boss-path.md](11-boss-path.md).
