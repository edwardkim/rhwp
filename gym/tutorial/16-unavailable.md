---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 16. unavailable — 0점이 아닌 부재

어떤 pack 줄이 `unavailable` 로 나오면, 그것은 실패가 아니다. 요구
명령이 지금 바이너리에 없다는 정직한 표기다. 이 페이지는
`gym/core/runner.py` 의 `score_pack` 이 이미 하는 일을 방문어로
옮긴다. 규칙을 바꾸지 않는다.

돌아가기: [README.md](README.md) · 정직: [15-scoring-honesty.md](15-scoring-honesty.md)

## 한 줄로

오래된 바이너리에게 "너는 0점"이라고 말하는 것은 거짓말이다. 그
놀이기구의 키가 없을 뿐이다. 최신으로 빌드하면 열린다.

```bash
cargo build --bin rhwp
python gym/score.py --agent 나 --profile family
```

## 어디서 오나

각 pack 의 `pack.json` 에 `requires.commands` 가 있다.
`casual-rides` 는 `info`, `explain`, `export-tables`, `search` 다.
하나라도 `rhwp capabilities` 에 없으면 그 pack 은 채점하지 않고
`status: unavailable` 과 `missingCommands` 를 남긴다. `score` 는
`null` 이다. 0 이 아니다.

입장 봉투의 `packsScored` 는 unavailable pack 을 세지 않는다. 고른
pack 이 전부 부재면 `verdict` 는 `deny` 일 수 있다. 그것은 "점수가
낮다"가 아니라 "유효 채점이 하나도 없다"다.

## 출력에서 읽는 법

```
나: 4/4  (pack 1 채점, 1 unavailable)
  - casual-rides       4/4  (4/4 과제)
  - self-description   unavailable (없는 명령: export-ontology)
```

위는 설명용이다. 실제 빠진 명령 이름은 네 바이너리에 따라 다르다.

## 자주 있는 원인

1. **빌드하지 않았다.** `cargo build --bin rhwp` 가 없다.
2. **PATH 의 다른 rhwp.** 예전 설치본이 먼저 잡힌다. `gym/score.py --bin target/debug/rhwp` 로 지목한다.
3. **부분 빌드.** 일부 명령만 있는 오래된 커밋을 돌린다.
4. **프로파일을 넓게 골랐다.** `maintainer` 는 전 pack 이다. 새 pack
   이 요구하는 명령이 없으면 그 줄만 unavailable 이다. 다른 줄 점수와
   섞지 마라.

## 0점과 어떻게 다른가

| 상태 | 의미 | 입장 |
|---|---|---|
| `scored` 0/N | 명령은 있고 답이 틀렸다 | allow (채점은 됨) |
| `unavailable` | 명령이 없어 채점하지 않았다 | 그 pack 은 packsScored 에 안 듦 |

둘을 같은 색으로 그리면 정직 조항이 깨진다. 리포트와 리더보드도
이 구분을 지킨다. 휴게실이 색을 섞지 않는다.
