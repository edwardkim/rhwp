# ☕ 휴게실 — 처음 오신 분을 위한 5분 안내

운동장 지도([PARK.md](../PARK.md))만 보면 막막하다. 여기서 첫 놀이기구를 함께
타 본다. 커피 한 잔 마시는 시간이면 충분하다.

전제: 저장소를 빌드해 `rhwp` 바이너리가 있고(`cargo build`), 저장소 루트에서
파이썬 3.8+ 이 돈다.

---

## 1. 표를 끊는다 (30초)

놀이공원은 표를 끊고 들어간다. 운동장의 표는 채점기가 발급하는 **입장 봉투**다.
아직 아무것도 안 풀어도, 채점을 한 번 돌리면 표가 나온다.

```bash
python gym/score.py --agent 나 --profile family
```

`--profile family` 는 부모님·친구와 함께 도는 **입문존만** 고른다. 출력 끝에
이렇게 나오면 표가 발권된 것이다:

```
나: 4/4  (pack 1 채점)
  - casual-rides       4/4  (4/4 과제)
```

> 방금 무슨 일이? 채점기가 입문존 놀이기구 4개를 스스로 풀어(기준 풀이) 채점하고,
> `gym/submissions/나/admission.json` 에 `verdict: allow` 티켓을 남겼다.

## 2. 회전목마를 직접 탄다 (2분)

이제 손으로 하나 타 보자. 회전목마(CR01)는 "이 문서가 몇 쪽인가"를 묻는다.

```bash
# 문서를 열어 쪽수를 본다
rhwp info samples/table-001.hwp --json
```

출력에서 `"pageCount": 3` 같은 숫자를 찾는다. 그 수를 답으로 적는다:

```bash
mkdir -p gym/submissions/나/casual-rides/CR01
echo '{"pages": 3}' > gym/submissions/나/casual-rides/CR01/answer.json
```

채점한다:

```bash
python gym/score.py --agent 나 --pack casual-rides
```

`CR01` 이 통과다. 축하한다 — 첫 놀이기구를 탔다. 틀린 숫자를 적으면 통과하지
않는다(채점기가 `rhwp info` 로 정답을 **그 자리에서 다시 계산**하기 때문 —
골든 파일이 아니다).

## 3. 명예의 전당에 이름을 올린다 (1분)

탔으면 전당에 올린다. 이 점수판은 자기 신고가 아니라 검증 사다리로 봉인된다.

```bash
python gym/tools/leaderboard.py attest --agent 나
python gym/tools/leaderboard.py verify
```

`verify` 가 전 사슬을 재검증하고 통과를 보고한다. 네 점수는 이제 위조 불가능한
방식으로 봉인됐다.

---

## 다음 어디로?

| 담력이 붙었으면 | 이렇게 |
|---|---|
| 본식 놀이기구 (편집·판독·보안) | `python gym/score.py --agent 나` (전 존) |
| 검증 사다리 10단 | `--profile operator` |
| 🐉 **보스존** (고난도) | `--profile boss` — 한 단만 틀려도 막힌다 |
| 부모님·친구 초대 | [../INVITE.md](../INVITE.md) |

## 자주 묻는 것

**Q. 기준 풀이(reference/)를 봐도 되나?**
봐도 채점은 정직하게 돈다. 다만 그때 측정되는 건 "스스로 경로를 찾는 능력"이
아니라 "따라 치는 능력"이 될 뿐이다. 놀이기구는 직접 타야 재밌다.

**Q. 왜 내 점수가 어떤 pack 은 `unavailable` 인가?**
그 pack 이 요구하는 rhwp 명령이 네 바이너리에 없다는 뜻이다. 0점이 아니라
"이 놀이기구는 지금 네 키(바이너리)로는 못 탄다"는 정직한 표기다. 최신으로
빌드하면 열린다.

**Q. 오프라인에서도 되나?**
전부 로컬이다. 네트워크 없이 돈다 — 채점도, 리더보드 봉인도.
