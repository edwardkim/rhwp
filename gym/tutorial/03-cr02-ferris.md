---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 3. 🎡 CR02 관람차 — 문단이 몇 개인가요?

회전목마와 같은 문서, 다른 창문. 쪽수 대신 문단 수를 읽는다. 과제 정본은
`gym/packs/casual-rides/tasks/CR02.json` 이다. 이 안내는 그 JSON 을
고치지 않는다.

돌아가기: [README.md](README.md) · 이전: [02-cr01-carousel.md](02-cr01-carousel.md)

## 과제가 묻는 것

| 항목 | 값 |
|---|---|
| id | `CR02` |
| tier | 1 |
| 제목 | 문단이 몇 개인가요? |
| 입력 | `samples/table-001.hwp` |
| 제출 | `answer.json` 의 `paragraphs` |
| 라이브 오라클 | `rhwp explain {input} --json` 의 `paragraphCount` |

명령이 `info` 가 아니라 `explain` 이다. 입문존은 "한 번 실행하고 숫자
하나를 옮긴다"는 결은 같고, **어느 창문을 여는가** 만 달라진다.

## 손으로 타기

```bash
rhwp explain samples/table-001.hwp --json
```

`paragraphCount` 를 찾는다. 그 수를 `paragraphs` 에 넣는다.

```bash
mkdir -p gym/submissions/나/casual-rides/CR02
```

`gym/submissions/나/casual-rides/CR02/answer.json`:

```json
{"paragraphs": 0}
```

`0` 은 자리 표시다. 네 `explain` 출력을 적어라. 채점기가 그 자리에서
다시 센다.

```bash
python gym/score.py --agent 나 --pack casual-rides
```

## 자주 하는 실수

1. **CR01 의 키를 다시 쓴다.** `pages` 는 CR01 전용이다. CR02 는
   `paragraphs` 다. 키가 틀리면 `answer_eq` 가 그 칸을 못 읽는다.
2. **명령을 `info` 로 연다.** `info` 의 `pageCount` 는 쪽수다. 문단
   수가 아니다. 과제 힌트가 `explain` 인 이유가 있다.
3. **같은 `answer.json` 을 CR01 폴더에 덮어쓴다.** 과제마다 폴더가
   갈린다. `CR01/` 과 `CR02/` 는 다른 제출이다.

## 왜 같은 문서인가

입문존 네 놀이기구는 모두 `samples/table-001.hwp` 를 본다. 문서를 바꾸는
게 목적이 아니다. **같은 입력을 네 명령으로 네 숫자로 읽는 것**이
목적이다. `info` · `explain` · `export-tables` · `search` 가
`casual-rides` pack 의 `requires.commands` 다. 하나라도 바이너리에
없으면 이 pack 전체는 `unavailable` 이다. 0점이 아니다.
→ [16-unavailable.md](16-unavailable.md)

## 다음

서커스 텐트 → [04-cr03-circus.md](04-cr03-circus.md). 같은 문서의 표
개수를 센다.
