---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 1. 입장 티켓 — admission.json

놀이공원은 표를 끊고 들어간다. 운동장의 표는 채점기가 쓰는 **입장 봉투**
(`admission.json`)다. 이 페이지는 그 봉투를 처음 발급받는 절차만 안내한다.
입장 조건을 바꾸지 않는다. 조건의 정본은 `gym/score.py` 다.

돌아가기: [README.md](README.md) · 지도: [../PARK.md](../PARK.md)

## 왜 표가 따로 있나

채점(`scorecard.json`)과 입장(`admission.json`)은 다른 봉투다.

- 스코어카드는 **몇 점을 받았는가** 다.
- 입장 봉투는 **이 러너로 pack 을 하나라도 유효하게 채점했는가** 다.

만점이 입장 조건이 아니다. 낮은 점수도 순위이지 입장 거부 사유가 아니다.
이 구분은 [../PARK.md](../PARK.md) 의 입구 절과 [15-scoring-honesty.md](15-scoring-honesty.md)
가 같은 말을 한다.

## 표를 끊는 명령

저장소 루트에서:

```bash
python gym/score.py --agent 나 --profile family
```

`--profile family` 는 `gym/profiles/family.json` 을 읽어 pack 목록을
`casual-rides` 하나로 줄인다. 프로파일 이름이 `family` 가 아니면 파일이
없다. 시험이 이 이름을 잠근다.

같은 표를 프로파일 없이 입문 pack 만 지목해도 끊을 수 있다.

```bash
python gym/score.py --agent 나 --pack casual-rides
```

`--agent` 는 제출 폴더 이름이다. 한글이든 영문이든 된다. 나중에 전당에
오를 이름과 같게 두는 편이 덜 헷갈린다.

## 발급되는 파일

채점이 끝나면 `gym/submissions/나/` 아래에 적어도 세 파일이 생긴다.

| 파일 | 역할 |
|---|---|
| `scorecard.json` | pack 별 점수, 러너 신원, 과제 결과 |
| `report.md` | 사람이 읽는 같은 내용 |
| `admission.json` | 입장 판정 봉투 |

입장 봉투의 뼈대는 이렇다. 키 이름은 `gym/score.py` 가 쓰는 그대로다.

```json
{
  "schemaVersion": "1.0",
  "kind": "gymAdmission",
  "agent": "나",
  "verdict": "allow",
  "packsScored": 1,
  "packsUnavailable": 0,
  "score": 4,
  "max": 4,
  "runner": {
    "rhwpVersion": "…",
    "rhwpCommit": "…",
    "capabilitiesSha256": "…"
  }
}
```

- `verdict` 는 `packsScored >= 1` 이면 `allow`, 아니면 `deny` 다.
- `packsScored` 는 실제로 채점된 pack 수다. `unavailable` 은 여기에 안 든다.
- `runner` 는 **어느 바이너리로 채점했는가** 다. 점수는 바이너리마다 달라질
  수 있으므로 신원이 같이 붙는다.

이 안내가 `verdict` 계산식을 바꾸지 않는다. 읽기만 한다.

## 기준 풀이가 먼저 돈다

아직 `gym/submissions/나/casual-rides/` 에 답을 안 넣었는데 4/4 가 나올 수
있다. 채점기가 기준 풀이(`gym/packs/casual-rides/reference/`)를 써서 스스로
푼 것이다. 그건 "놀이기구가 고장 나지 않았다"는 뜻이지, "네가 탔다"는 뜻이
아니다.

직접 타려면 [02-cr01-carousel.md](02-cr01-carousel.md) 처럼 제출 폴더에
`answer.json` 을 놓고 다시 채점한다. 틀린 숫자를 넣으면 그 과제만 떨어진다.

## 표가 deny 일 때

`verdict: deny` 는 보통 이런 경우다.

1. `--profile` 오타. `familly` 나 `Family` 는 파일이 없다.
2. 바이너리를 못 찾아 채점이 시작도 못 함.
3. 고른 pack 이 전부 `unavailable` 이라 `packsScored` 가 0.

0점과 deny 는 다르다. 채점은 됐는데 답을 전부 틀린 사람은 `allow` 다.
점수가 낮을 뿐, 입장 거부 사유가 아니다. 부재는 [16-unavailable.md](16-unavailable.md).

## Windows

PowerShell 5.1 에서는 `mkdir -p` 가 없다. 표만 끊는 명령은 같다.

```powershell
python gym/score.py --agent 나 --profile family
```

폴더를 손으로 만들 때는 [19-windows.md](19-windows.md).

## 다음

표를 끊었으면 회전목마를 탄다 → [02-cr01-carousel.md](02-cr01-carousel.md).
프로파일 일곱 이름을 먼저 보고 싶으면 → [06-profiles.md](06-profiles.md).
