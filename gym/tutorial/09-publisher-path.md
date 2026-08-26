---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 9. publisher 길 — 내보내고 배포하기 전에

`publisher` 는 `serialization`, `layout-rendering`, `security` 를
고른다. 정본은 `gym/profiles/publisher.json`. 변환이 진짜인지, 쪽수가
맞는지, 배포 전 오염이 있는지를 먼저 묻는다.

돌아가기: [README.md](README.md) · 프로파일: [06-profiles.md](06-profiles.md)

```bash
python gym/score.py --agent 나 --profile publisher
```

## SR01 — HWPX 변환 자기검증

정본: `gym/packs/serialization/tasks/SR01.json`

입력을 HWPX 로 보내 `conv.hwpx` 를 내고, IR 이 같은지 `answer.json` 의
`identical` 에 true/false 로 적는다.

채점 두 칸:

1. `value_eq` — `rhwp info conv.hwpx --json` 의 `format` 이 `"hwpx"`
2. `answer_eq` — `rhwp ir-diff {input} conv.hwpx --json` 의 `identical`

`ir-diff` 는 다르면 종료 코드 3 을 낼 수 있다. 과제가
`expect_exits: [0, 3]` 을 허용하는 이유다. 종료 코드 3 을 실패로
버리면 정직한 "다르다" 판정이 사라진다. 이 계약은
`scripts/tests/test_gym_score.py` 가 이미 잠근다. 휴게실이 그 시험을
바꾸지 않는다.

힌트: `rhwp export-hwpx --verify`. 변환 명령을 `convert` 로 추측하지
마라. 과제가 가리키는 명령을 쓴다.

```bash
mkdir -p gym/submissions/나/serialization/SR01
```

## LR01 — 쪽수 판정

정본: `gym/packs/layout-rendering/tasks/LR01.json`

CR01·T01 과 같은 창문이다. 입력만
`samples/basic/issue2007_nested_cell_pagination_42065.hwp` 다.
조판 pack 의 첫 과제가 쪽수인 이유는, 렌더 산출을 보기 전에 "몇
쪽으로 읽히는가"가 맞아야 해서다.

```bash
rhwp info samples/basic/issue2007_nested_cell_pagination_42065.hwp --json
mkdir -p gym/submissions/나/layout-rendering/LR01
```

`answer.json` 키는 `pages`.

## SE01 — PII 탐지 (읽기 전용)

정본: `gym/packs/security/tasks/SE01.json`

입력 `samples/task2097/75544_pii_bunseok.hwpx` 에서 개인정보 후보가
몇 건인지 **읽기 전용으로** 센다.

```bash
rhwp edit redact samples/task2097/75544_pii_bunseok.hwpx --dry-run --json
mkdir -p gym/submissions/나/security/SE01
```

답 키는 `findings`, 오라클 경로는 `findingCount`. `--dry-run` 이
빠지면 원본을 고칠 수 있다. 과제가 읽기 전용을 명시한 이유를 지킨다.

보안존의 뒤 과제들은 은닉 텍스트·주입 신호·유니코드·서명으로 올라간다.
첫 과제는 "배포 전에 숫자를 읽는다"다. 입문존과 같은 결을 오염 축에
적용한 것이다.

## publisher 가 묶지 않는 것

`render-tree` 와 `studio-e2e` 는 저장소에 있지만 `publisher` 목록에
없다. 필요하면 `--pack` 으로 고른다. 프로파일 JSON 을 이 안내가
고쳐 넣지 않는다. 다른 열린 PR 이 pack 을 늘리고 있을 수 있다.

## 다음

폴더 스윕과 사다리 → [10-operator-path.md](10-operator-path.md).
보스존은 그 다음 → [11-boss-path.md](11-boss-path.md).
