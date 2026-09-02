---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# ☕ 휴게실 — 처음 오신 분을 위한 안내

운동장 지도([PARK.md](../PARK.md))만 보면 막막하다. 여기서 첫 놀이기구를 함께
타 본다. 5분이면 표를 끊고 회전목마를 한 바퀴 돌 수 있다. 담력이 붙으면 같은
휴게실에서 다음 존 입구까지 걸어간다.

전제: 저장소를 빌드해 `rhwp` 바이너리가 있고(`cargo build --bin rhwp`), 저장소
루트에서 파이썬 3.8+ 이 돈다. Windows 는 [19-windows.md](19-windows.md) 를
먼저 본다.

채점 논리·검사 연산자·pack 과제 JSON 은 이 휴게실이 **바꾸지 않는다**. 여기는
입구 안내다. 판정 정본은 `gym/core/checks.py` 와 각 `packs/*/tasks/*.json` 이다.

규약 정본(기계가 잠그는 계약)은 [../docs/tutorial.md](../docs/tutorial.md) 다.
작업 기록은 [../../mydocs/working/gym_tutorial.md](../../mydocs/working/gym_tutorial.md).

---

## 휴게실 지도

| 순서 | 문서 | 누구에게 |
|---|---|---|
| 0 | 이 파일 (5분 첫 방문) | 누구나 |
| 1 | [01-admission.md](01-admission.md) | 표를 어디서 끊는지 |
| 2 | [02-cr01-carousel.md](02-cr01-carousel.md) | 🎠 CR01 회전목마 |
| 3 | [03-cr02-ferris.md](03-cr02-ferris.md) | 🎡 CR02 관람차 |
| 4 | [04-cr03-circus.md](04-cr03-circus.md) | 🎪 CR03 서커스 텐트 |
| 5 | [05-cr04-ringtoss.md](05-cr04-ringtoss.md) | 🎯 CR04 링 던지기 |
| 6 | [06-profiles.md](06-profiles.md) | 일곱 프로파일 이름 |
| 7 | [07-starter-path.md](07-starter-path.md) | casual 다음, `starter` |
| 8 | [08-editor-path.md](08-editor-path.md) | `editor` 첫 편집 |
| 9 | [09-publisher-path.md](09-publisher-path.md) | `publisher` 변환·보안 |
| 10 | [10-operator-path.md](10-operator-path.md) | `operator` 사다리 |
| 11 | [11-boss-path.md](11-boss-path.md) | `boss` 자이로드롭 |
| 12 | [12-leaderboard.md](12-leaderboard.md) | 명예의 전당 |
| 13 | [13-invite.md](13-invite.md) | 친구 초대 |
| 14 | [14-submissions.md](14-submissions.md) | 제출 폴더 결 |
| 15 | [15-scoring-honesty.md](15-scoring-honesty.md) | 채점은 라이브다 |
| 16 | [16-unavailable.md](16-unavailable.md) | 0점이 아닌 부재 |
| 17 | [17-faq.md](17-faq.md) | 자주 묻는 것 |
| 18 | [18-troubleshooting.md](18-troubleshooting.md) | 막혔을 때 |
| 19 | [19-windows.md](19-windows.md) | PowerShell 명령 |
| 20 | [20-checklist.md](20-checklist.md) | 첫날 체크리스트 |

테마파크 한 장 지도는 [../PARK.md](../PARK.md), 초대장 정본은
[../INVITE.md](../INVITE.md).

---

## 1. 표를 끊는다 (30초)

놀이공원은 표를 끊고 들어간다. 운동장의 표는 채점기가 발급하는 **입장 봉투**다.
아직 아무것도 안 풀어도, 채점을 한 번 돌리면 표가 나온다.

```bash
python gym/score.py --agent 나 --profile family
```

`--profile family` 는 부모님·친구와 함께 도는 **입문존만** 고른다. 프로파일
이름은 `gym/profiles/family.json` 의 `id` 와 같다. 출력 끝에 이렇게 나오면
표가 발권된 것이다:

```
나: 4/4  (pack 1 채점)
  - casual-rides       4/4  (4/4 과제)
```

> 방금 무슨 일이? 채점기가 입문존 놀이기구 4개를 스스로 풀어(기준 풀이) 채점하고,
> `gym/submissions/나/admission.json` 에 `verdict: allow` 티켓을 남겼다.
> 만점이 입장 조건이 아니다. pack 을 하나라도 유효하게 채점하면 들어온다.

입장 봉투의 판정 기준은 `gym/score.py` 가 이미 하는 일이다. 이 안내가 그
조건을 바꾸지 않는다. 자세히 → [01-admission.md](01-admission.md).

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

나머지도 같은 결이다.

| 놀이기구 | 명령 | 읽는 칸 | 답 키 |
|---|---|---|---|
| 🎠 CR01 | `rhwp info samples/table-001.hwp --json` | `pageCount` | `pages` |
| 🎡 CR02 | `rhwp explain samples/table-001.hwp --json` | `paragraphCount` | `paragraphs` |
| 🎪 CR03 | `rhwp export-tables samples/table-001.hwp --json` | `tableCount` | `tables` |
| 🎯 CR04 | `rhwp search samples/table-001.hwp --json -- 표` | `matchCount` | `hits` |

한 대씩 타려면 [02-cr01-carousel.md](02-cr01-carousel.md) 부터 순서대로.

## 3. 명예의 전당에 이름을 올린다 (1분)

탔으면 전당에 올린다. 이 점수판은 자기 신고가 아니라 검증 사다리로 봉인된다.

```bash
python gym/tools/leaderboard.py attest --agent 나
python gym/tools/leaderboard.py verify
```

`verify` 가 전 사슬을 재검증하고 통과를 보고한다. 네 점수는 이제 위조 불가능한
방식으로 봉인됐다. 자세히 → [12-leaderboard.md](12-leaderboard.md).

---

## 일곱 프로파일 — 이름을 외우지 말고 파일을 본다

프로파일은 pack 을 **고르는** 도구이지 점수를 뭉치는 도구가 아니다. 이름은
`gym/profiles/<id>.json` 의 `id` 필드와 같아야 한다. 시험이 이 일곱 이름을
잠근다.

| id | 묶는 pack | 언제 |
|---|---|---|
| `family` | `casual-rides` | 부모님·친구, 키 제한 없음 |
| `starter` | `core-cli`, `self-description` | 도구의 결을 익힐 때 |
| `editor` | `core-cli`, `text-editing`, `table-editing`, `objects-media` | 문서를 고칠 때 |
| `publisher` | `serialization`, `layout-rendering`, `security` | 내보내고 배포하기 전 |
| `operator` | `corpus-diagnostics`, `automation` | 폴더와 사다리 |
| `boss` | `expert-challenges` | 한 단만 틀려도 막히는 곳 |
| `maintainer` | 전 pack | 운동장 전체를 돌 때 |

```bash
python gym/score.py --agent 나 --profile family
python gym/score.py --agent 나 --profile starter
python gym/score.py --agent 나 --profile editor
python gym/score.py --agent 나 --profile publisher
python gym/score.py --agent 나 --profile operator
python gym/score.py --agent 나 --profile boss
python gym/score.py --agent 나 --profile maintainer
```

표의 정본은 [06-profiles.md](06-profiles.md). casual 바깥 입문은
[07-starter-path.md](07-starter-path.md) 부터다.

## 다음 어디로?

| 담력이 붙었으면 | 이렇게 |
|---|---|
| 본식 놀이기구 (편집·판독·보안) | `python gym/score.py --agent 나` (전 존) |
| 검증 사다리 10단 | `--profile operator` |
| 🐉 **보스존** (고난도) | `--profile boss` — 한 단만 틀려도 막힌다 |
| 부모님·친구 초대 | [../INVITE.md](../INVITE.md) · [13-invite.md](13-invite.md) |
| 첫날 한 장 | [20-checklist.md](20-checklist.md) |

## 자주 묻는 것

**Q. 기준 풀이(reference/)를 봐도 되나?**
봐도 채점은 정직하게 돈다. 다만 그때 측정되는 건 "스스로 경로를 찾는 능력"이
아니라 "따라 치는 능력"이 될 뿐이다. 놀이기구는 직접 타야 재밌다.

**Q. 왜 내 점수가 어떤 pack 은 `unavailable` 인가?**
그 pack 이 요구하는 rhwp 명령이 네 바이너리에 없다는 뜻이다. 0점이 아니라
"이 놀이기구는 지금 네 키(바이너리)로는 못 탄다"는 정직한 표기다. 최신으로
빌드하면 열린다. → [16-unavailable.md](16-unavailable.md)

**Q. 오프라인에서도 되나?**
전부 로컬이다. 네트워크 없이 돈다 — 채점도, 리더보드 봉인도.

**Q. 이 안내가 채점 점수를 바꾸나?**
바꾸지 않는다. `gym/core/checks.py` 의 연산자 등록부와 pack 과제 JSON 은
휴게실 문서의 범위 밖이다. 점수는 여전히 pack 별로 보존되고, 기대값은 채점
시점에 rhwp 로 재계산된다. → [15-scoring-honesty.md](15-scoring-honesty.md)

**Q. Windows 인데 `mkdir -p` 가 안 된다.**
PowerShell 명령은 [19-windows.md](19-windows.md) 에 모아 두었다.

더 많은 질문 → [17-faq.md](17-faq.md). 막히면 → [18-troubleshooting.md](18-troubleshooting.md).
