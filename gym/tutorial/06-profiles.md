---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 6. 프로파일 — pack 을 고르는 일곱 이름

프로파일은 점수를 뭉치지 않는다. pack 목록을 고를 뿐이다. 이름과 묶음은
`gym/profiles/<id>.json` 이 정본이다. 이 페이지는 그 파일을 읽기 쉽게
옮겨 적는다. 파일을 고치지 않고, 채점 논리도 고치지 않는다.

돌아가기: [README.md](README.md) · 지도: [../PARK.md](../PARK.md)

## 일곱 이름

시험이 아래 `id` 일곱 개가 `gym/profiles/` 에 있고, 문서가 그 철자를
그대로 쓰는지 잠근다. 대소문자·복수형·별명을 만들지 마라.

| id | 파일 | title | packs |
|---|---|---|---|
| `family` | `gym/profiles/family.json` | 가족 코스 | `casual-rides` |
| `starter` | `gym/profiles/starter.json` | 입문 | `core-cli`, `self-description` |
| `editor` | `gym/profiles/editor.json` | 편집자 | `core-cli`, `text-editing`, `table-editing`, `objects-media` |
| `publisher` | `gym/profiles/publisher.json` | 배포자 | `serialization`, `layout-rendering`, `security` |
| `operator` | `gym/profiles/operator.json` | 운영자 | `corpus-diagnostics`, `automation` |
| `boss` | `gym/profiles/boss.json` | 보스 코스 | `expert-challenges` |
| `maintainer` | `gym/profiles/maintainer.json` | 메인테이너 | 전 pack |

`maintainer` 의 packs 배열은 저장소에 있는 모든 pack id 와 같아야
한다. 그 계약은 이미 `scripts/tests/test_gym_packs.py` 의
`test_maintainer_profile_covers_every_pack` 이 잠근다. 휴게실은 그
시험을 복제하지 않고, 이름만 안내한다.

## 어떻게 고르나

```bash
python gym/score.py --agent 나 --profile family
python gym/score.py --agent 나 --profile starter
python gym/score.py --agent 나 --profile editor
python gym/score.py --agent 나 --profile publisher
python gym/score.py --agent 나 --profile operator
python gym/score.py --agent 나 --profile boss
python gym/score.py --agent 나 --profile maintainer
```

`--profile` 과 `--pack` 을 같이 주면 프로파일이 pack 목록을 덮어쓴다.
구현은 `gym/core/runner.py` 의 `score_all` 이다. 이 안내가 그 순서를
바꾸지 않는다.

없는 이름을 주면 `gym/profiles/<오타>.json` 을 열다가 실패한다.
`Family`, `casual`, `beginner`, `expert` 는 프로파일 id 가 아니다.

## 테마파크 존과의 대응

[../PARK.md](../PARK.md) 의 존은 은유다. 프로파일은 기계가 읽는 묶음이다.

| 존 (은유) | 가까운 프로파일 | 비고 |
|---|---|---|
| 🎠 입문존 | `family` | 키 제한 없는 네 놀이기구 |
| ☕ 휴게실 다음 | `starter` | casual 바깥 첫 입문 |
| ✏️ 편집존 | `editor` | 본문·표·개체 |
| 📖 판독존 + 🔐 보안존 | `publisher` | 변환·조판·보안 |
| ⚙️ 사다리존 + 판독 일부 | `operator` | 스윕과 10단 |
| 🐉 보스존 | `boss` | 한 단만 틀려도 막힘 |
| 공원 전체 | `maintainer` | 전 pack, 총점은 편의값 |

은유가 점수를 바꾸지 않는다. 정직 조항은 PARK 와
[15-scoring-honesty.md](15-scoring-honesty.md) 가 같이 지킨다.

## 고르는 순서 (추천)

처음 온 사람·에이전트는 이 순서가 덜 다친다.

1. `family` — 숫자 네 개를 옮겨 제출 폴더 결을 익힌다.
2. `starter` — `info`/`search` 와 `capabilities` 로 도구가 자기를
   설명하는 법을 본다.
3. `editor` 또는 `publisher` — 하고 싶은 일에 가까운 쪽.
4. `operator` — 폴더와 사다리.
5. `boss` — 담력이 붙은 뒤.
6. `maintainer` — 구멍 없이 한 바퀴.

순서는 권장이지 입장 조건이 아니다. `boss` 부터 타도 표는 끊긴다.
떨어질 뿐이다.

## 각 길로 가는 문

- `family` 실습: [02-cr01-carousel.md](02-cr01-carousel.md) ~ [05-cr04-ringtoss.md](05-cr04-ringtoss.md)
- `starter`: [07-starter-path.md](07-starter-path.md)
- `editor`: [08-editor-path.md](08-editor-path.md)
- `publisher`: [09-publisher-path.md](09-publisher-path.md)
- `operator`: [10-operator-path.md](10-operator-path.md)
- `boss`: [11-boss-path.md](11-boss-path.md)

## 프로파일이 하지 않는 것

- 점수를 가중하지 않는다. `family` 의 4점과 `boss` 의 23점은 다른
  pack 의 점수다. 총점은 편의값이다.
- 새 과제를 만들지 않는다. pack 에 있는 과제만 고른다.
- 채점 연산자를 바꾸지 않는다. `answer_eq` 는 여전히 `answer_eq` 다.
- 리더보드 순위를 바꾸지 않는다. 등재는
  [12-leaderboard.md](12-leaderboard.md) 의 `attest` 가 한다.
