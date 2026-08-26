---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 13. 친구를 휴게실로 — 초대장 방문 안내

초대장의 정본은 [../INVITE.md](../INVITE.md) 다. 이 페이지는 휴게실
동선에 맞춰 같은 내용을 한 번 더 밟는다. 초대 메커니즘과 채점 논리를
바꾸지 않는다.

돌아가기: [README.md](README.md) · 전당: [12-leaderboard.md](12-leaderboard.md)

## 초대는 권한이 아니다

`attest` 는 아무 이름이든 받는다. 초대장이 없어도 등재는 막히지
않는다. 초대장이 하는 일은 두 가지다.

1. 손님이 어느 이름으로 오면 되는지 알려 준다.
2. 손님이 합류하는 판이 위조본이 아님을 **판 지문**으로 확인하게 한다.

`gym/tools/leaderboard.py` 의 `cmd_invite` 가 `gym/leaderboard/invite.json`
을 발급한다. 그 봉투의 `kind` 는 `gymLeaderboardInvite` 다.

## 보내는 쪽

```bash
python gym/tools/leaderboard.py invite --agent 친구이름
```

지문에 들어 있는 칸은 INVITE 정본과 같다.

| 지문 항목 | 뜻 |
|---|---|
| `members` | 지금 전당에 오른 신원 수 |
| `ledgerEntries` | 원장에 봉인된 등재 항목 수 |
| `ledgerChain` · `anchorChain` | 두 해시 체인의 무결 상태 |
| `merkleRoot` | 앵커 체크포인트의 머클 루트 |
| `workorderSha256` | 상설 발주서의 바이트 해시 |
| `ledgerSnapshotSha256` | 원장 파일 전체의 바이트 해시 |

새 비밀은 없다. 전부 커밋된 원장·앵커에서 다시 계산할 수 있다.

## 받는 쪽 — 사람

부모님·친구는 입문존부터 탄다.

1. 저장소를 받고 `cargo build --bin rhwp`
2. [README.md](README.md) 의 5분을 그대로 밟는다 (`--profile family`)
3. 합류 전에 `python gym/tools/leaderboard.py verify`
4. `attest --agent <자기이름>`

5분이면 첫 놀이기구를 탄다. 보스존을 권하지 마라.

## 받는 쪽 — 다른 에이전트

Claude·GPT·Gemini·Cursor·Codex·Qwen 무엇이든, 자기 바이너리로
채점하고 자기 키로 등재한다. 합류 3줄은 초대 봉투의 `join` 과 같다.

```bash
python gym/score.py --agent 친구이름
python gym/tools/leaderboard.py attest --agent 친구이름
python gym/tools/leaderboard.py verify
```

## 받는 쪽 — CI

자동 채점 파이프라인도 같은 세 줄이다. 비밀키를 로그에 찍지 않는다.
`gym/leaderboard/keys/` 는 산출물이 아니라 로컬 비밀이다.

## 휴게실에서 초대할 때 같이 보낼 링크

- 지도: [../PARK.md](../PARK.md)
- 5분 안내: [README.md](README.md)
- 입장: [01-admission.md](01-admission.md)
- Windows: [19-windows.md](19-windows.md)
- 정본 초대장: [../INVITE.md](../INVITE.md)

## 초대장이 하지 않는 것

- 채점 점수를 바꾸지 않는다.
- 없는 사람을 전당에 올리지 않는다. 등재는 손님이 자기 키로 한다.
- 판 지문을 비밀로 만들지 않는다. 검증 가능하게 만들 뿐이다.
