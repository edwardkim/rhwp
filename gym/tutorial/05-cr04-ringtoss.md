---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 5. 🎯 CR04 링 던지기 — '표' 글자가 몇 번?

입문존의 마지막 놀이기구. 검색 한 번으로 숫자를 옮긴다. 과제 정본은
`gym/packs/casual-rides/tasks/CR04.json`. 이 안내는 JSON 을 고치지
않는다.

돌아가기: [README.md](README.md) · 이전: [04-cr03-circus.md](04-cr03-circus.md)

## 과제가 묻는 것

| 항목 | 값 |
|---|---|
| id | `CR04` |
| tier | 1 |
| 제목 | '표' 글자가 몇 번? |
| 입력 | `samples/table-001.hwp` |
| 제출 | `answer.json` 의 `hits` |
| 라이브 오라클 | `rhwp search {input} --json -- 표` 의 `matchCount` |

`--` 는 검색어와 옵션을 가르는 관례다. 검색어가 옵션처럼 보이면
파서가 삼킨다. 과제 힌트가 `-- 표` 인 이유를 휴게실이 바꾸지 않는다.

## 손으로 타기

```bash
rhwp search samples/table-001.hwp --json -- 표
```

`matchCount` 를 찾는다. 그 수를 `hits` 에 넣는다.

```bash
mkdir -p gym/submissions/나/casual-rides/CR04
```

`gym/submissions/나/casual-rides/CR04/answer.json`:

```json
{"hits": 0}
```

`0` 은 자리 표시다. 네 `search` 출력을 적어라.

```bash
python gym/score.py --agent 나 --pack casual-rides
```

네 과제 모두 통과면 입문존을 한 바퀴 돈 것이다. 전당에 이름을 올리려면
[12-leaderboard.md](12-leaderboard.md). 다음 존은
[07-starter-path.md](07-starter-path.md).

## 표 개수와 '표' 글자 수는 다른 것

CR03 은 **표 개체** 수(`tableCount`)다. CR04 는 본문에 글자 **표** 가
몇 번 나오는지(`matchCount`)다. 표가 3개여도 본문에 '표' 가 더 많거나
적을 수 있다. 두 숫자를 같게 맞추려 하지 마라. 라이브 오라클이 각각
다른 명령을 돌린다.

## 검색어를 바꾸면

`표` 가 아닌 다른 글자를 세면 떨어진다. 채점기는 네가 어떤 검색어를
썼는지가 아니라, **과제에 적힌 명령으로 다시 센 값**과 네 답을
비교한다. 따라 치는 검색어를 바꿔도 오라클은 바뀌지 않는다.

이 성질이 [15-scoring-honesty.md](15-scoring-honesty.md) 의 "기대값을
박제하지 마라"다. 골든 `hits: 7` 같은 숫자는 저장소에 없다.

## 입문존을 닫은 뒤

1. 프로파일 이름을 한 번 본다 → [06-profiles.md](06-profiles.md)
2. casual 바깥 첫 입문 → [07-starter-path.md](07-starter-path.md)
3. 부모님을 부르려면 → [13-invite.md](13-invite.md) · [../INVITE.md](../INVITE.md)
4. 첫날 체크리스트 → [20-checklist.md](20-checklist.md)

`family` 프로파일은 `casual-rides` 만 고른다. 입문존을 닫아도
`--profile family` 는 계속 그 네 과제만 채점한다. 다음 존은
`--profile starter` 또는 `--pack core-cli` 로 고른다.
