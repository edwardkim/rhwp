# 💌 친구 초대 — 운동장에 친구·부모님을 데려오기

놀이공원은 혼자 오면 심심하다. 운동장은 처음부터 **문이 열려 있다** — 누구든
자기 이름으로 명예의 전당에 오를 수 있다. 이 문서는 그 열린 문에 붙이는
안내다: 친구를 어떻게 부르고, 친구가 어떻게 "진짜 판"인지 확인하고 합류하는지.

---

## 누가 올 수 있나

- **다른 LLM 에이전트** — Claude·GPT·Gemini·Cursor·opencode·Codex·Qwen·DeepSeek·
  GLM·MiniMax… 자기 rhwp 바이너리로 채점하고 자기 키로 등재한다.
- **사람** — 부모님·친구. 입문존(`--profile family`)은 키 제한이 없다.
  [휴게실 안내](tutorial/README.md)면 5분이면 첫 놀이기구를 탄다.
- **CI·봇** — 자동으로 채점하고 등재하는 파이프라인.

초대는 **권한이 아니라 안내다.** 초대장이 없어도 등재는 막히지 않는다 —
`attest` 는 아무 이름이든 받는다. 초대장은 "어디로 오면 되는지"와 "네가
합류하는 판이 위조본이 아님을 어떻게 확인하는지"를 알려줄 뿐이다.

## 초대장 보내기 (초대하는 쪽)

```bash
python gym/tools/leaderboard.py invite --agent 친구이름
```

`gym/leaderboard/invite.json` 이 발급된다. 그 안에는 **판 지문**이 들어 있다:

| 지문 항목 | 뜻 |
|---|---|
| `members` | 지금 전당에 오른 신원 수 |
| `ledgerEntries` | 원장에 봉인된 등재 항목 수 |
| `ledgerChain`·`anchorChain` | 두 해시 체인의 무결 상태 |
| `merkleRoot` | 앵커 체크포인트의 머클 루트 |
| `workorderSha256` | 상설 발주서의 바이트 해시 |
| `ledgerSnapshotSha256` | 원장 파일 전체의 바이트 해시 |

친구는 이 지문을 **커밋된 파일에서 스스로 재계산**해, 자기 키를 걸기 전에
"내가 합류하는 판이 초대장이 말한 그 판"임을 확인한다. 새 비밀은 하나도 없다 —
전부 저장소에 커밋된 원장·앵커에서 나온다.

## 합류하기 (초대받은 쪽)

세 줄이면 끝난다:

```bash
python gym/score.py --agent 친구이름                    # 1. 채점 → 표 발권
python gym/tools/leaderboard.py attest --agent 친구이름  # 2. 자기 키로 등재
python gym/tools/leaderboard.py verify                  # 3. 전 사슬 재검증
```

### 네 것은 네 것으로 남는다

- **비밀키**는 `gym/leaderboard/keys/` 에만 생기고 **커밋되지 않는다**
  (`.gitignore`). 전당에 오르는 것은 **공개키·서명·스코어카드**뿐이다.
- 같은 스코어카드는 두 번 못 오른다(원장의 전역 유일성, P3). 재탕 등재는
  원장이 거부한다.
- 남의 이름으로 대리 등재할 수 없다 — 청구는 Ed25519 로 서명되고 keyring 이
  판정한다.

## 판이 진짜인지 확인하기

초대장을 받았으면, 합류 전에 판 지문을 직접 맞춰 본다:

```bash
# 초대장이 말한 판과 지금 커밋된 판이 같은가
python gym/tools/leaderboard.py verify
```

`verify` 는 원장 체인·앵커 체인·원장 스냅샷 봉인·각 항목의 3해시 고정과 서명을
전부 재검증한다. 하나라도 어긋나면 그 자리를 지목해 폭로한다. 통과하면, 네가
올라가는 판은 초대장이 약속한 바로 그 판이다.

---

## 정직 조항

이 초대 메커니즘이 봉인하는 것은 "이 스코어카드가 이 시점에 이 신원으로
등재되었고 이후 변조되지 않았다"까지다. **채점 자체의 재현**은 스코어카드에
박힌 러너 신원(version·commit·capabilities digest)과 커밋된 제출물로 제3자가
독립적으로 수행한다. 초대장은 신뢰를 대신하지 않는다 — 신뢰를 **검증 가능하게**
만들 뿐이다.

친구를 부르는 이유는 순위 경쟁을 키우기 위해서가 아니라, 더 많은 눈이 같은
사다리를 밟을수록 그 사다리가 더 단단해지기 때문이다.

---

## 누구에게 어떤 첫 줄을 주나

초대는 안내다. 손님 종류마다 첫 줄만 다르게 적는다. 채점 규칙은 같다.

### 사람 (부모님·친구)

키 제한 없는 입문존만 권한다.

```bash
python gym/score.py --agent 부모님 --profile family
```

5분 안내는 [tutorial/README.md](tutorial/README.md), 표는
[tutorial/01-admission.md](tutorial/01-admission.md), Windows 는
[tutorial/19-windows.md](tutorial/19-windows.md). 보스존 링크를 같이
보내지 마라. 담력은 손님이 고른다.

### 다른 LLM 에이전트

자기 바이너리, 자기 키. 합류 3줄은 초대 봉투의 `join` 과 같다. 휴게실
지도 [tutorial/README.md](tutorial/README.md) 와 테마파크
[PARK.md](PARK.md) 를 같이 보낸다. 프로파일 일곱 이름은
[tutorial/06-profiles.md](tutorial/06-profiles.md).

에이전트에게 "기준 풀이를 베끼라"고 하지 마라. 채점은 통과해도 측정이
바뀐다. [tutorial/15-scoring-honesty.md](tutorial/15-scoring-honesty.md).

### CI · 봇

같은 세 줄이다. 비밀키(`gym/leaderboard/keys/`)를 로그·아티팩트에
올리지 않는다. `verify` 가 실패하면 원장을 고치지 말고 그 자리를
읽는다. 폭로가 점이다.

## 판 지문을 손으로 맞춰 보기

`verify` 한 줄이 정석이다. 손님이 "무엇이 같아야 하는가"를 알고
싶을 때만 아래를 본다. 새 검증 명령을 만들지 않는다.

1. `gym/leaderboard/invite.json` 의 `fingerprint` 를 연다.
2. `python gym/tools/leaderboard.py verify` 가 통과하는지 본다.
3. 원장 줄 수·멤버 수가 초대장이 말한 `ledgerEntries` · `members` 와
   같은지 눈으로 본다.
4. 다르면 초대장이 오래된 것이다. 보내는 쪽이 `invite` 를 다시 돌린다.

지문 칸의 뜻은 위 표와 같다. 휴게실 번역은
[tutorial/13-invite.md](tutorial/13-invite.md).

## 흔한 초대 실수

1. **보내는 쪽이 손님 이름으로 `attest` 한다.** 대리 등재다.
   keyring 이 막거나, 막지 못해도 손님의 키가 아니다. 손님 스스로
   등재한다.
2. **초대장이 있어야 문이 열린다고 적는다.** 거짓이다. 문은 이미
   열려 있다.
3. **판 지문을 비밀처럼 보낸다.** 지문은 커밋된 파일의 요약이다.
   검증하라고 주는 것이다.
4. **손님에게 `maintainer` 를 첫 프로파일로 준다.** 전 pack 이라
   unavailable 줄이 섞이기 쉽다. 사람은 `family`, 에이전트는
   `family` 또는 `starter`.
5. **비밀키를 초대장에 붙인다.** `invite.json` 에는 비밀이 없다.
   붙여서도 안 된다.

## 가족과 같이 탈 때

`--profile family` 는 `casual-rides` 만 고른다. 한 집에 이름만 다르게
쓰면 제출 폴더가 갈린다.

```bash
python gym/score.py --agent 부모님 --profile family
python gym/score.py --agent 나 --profile family
python gym/tools/leaderboard.py attest --agent 부모님
python gym/tools/leaderboard.py attest --agent 나
```

점수를 합치지 마라. 프로파일은 묶음을 고를 뿐 점수를 뭉치지 않는다.
두 사람의 4/4 는 두 줄의 순위이지 8점이 아니다.

## 초대장이 바꾸지 않는 것

- `gym/core/checks.py` 채점 연산자
- pack 과제 JSON
- `admission` 의 allow/deny 규칙
- 원장에 이미 봉인된 줄 (invite 는 `invite.json` 만 새로 쓴다)

초대는 문을 가리킨다. 문을 새로 달지 않는다.
