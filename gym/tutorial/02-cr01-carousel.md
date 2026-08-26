---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 2. 🎠 CR01 회전목마 — 몇 쪽인가요?

입문존의 첫 놀이기구. 문서를 한 번 열고, 쪽수 하나를 `answer.json` 에
옮기면 된다. 과제 JSON 은 이 안내가 고치지 않는다. 정본은
`gym/packs/casual-rides/tasks/CR01.json` 이다.

돌아가기: [README.md](README.md) · 입장: [01-admission.md](01-admission.md)

## 과제가 묻는 것

| 항목 | 값 |
|---|---|
| id | `CR01` |
| tier | 1 (키 제한 없음) |
| 제목 | 몇 쪽인가요? |
| 입력 | `samples/table-001.hwp` |
| 제출 | `answer.json` 의 `pages` |
| 라이브 오라클 | `rhwp info {input} --json` 의 `pageCount` |

채점기는 골든 숫자를 가지고 있지 않다. 채점 시점에 `rhwp info` 를 다시
돌려 `pageCount` 와 네 `pages` 를 비교한다. 연산자는 `answer_eq` 다.
이 연산자의 정의는 `gym/core/checks.py` 에 있고, 휴게실이 그 정의를
바꾸지 않는다.

## 손으로 타기

저장소 루트에서 문서를 연다.

```bash
rhwp info samples/table-001.hwp --json
```

출력 JSON 에서 `pageCount` 를 찾는다. 그 숫자를 그대로 적는다. 예시는
숫자가 3 일 때다. **네 바이너리가 말한 수를 적어라.** 이 문서의 3 은
설명용이다. 라이브 오라클이 다른 수를 내면 그 수가 정답이다.

```bash
mkdir -p gym/submissions/나/casual-rides/CR01
```

`gym/submissions/나/casual-rides/CR01/answer.json`:

```json
{"pages": 3}
```

채점:

```bash
python gym/score.py --agent 나 --pack casual-rides
```

`CR01` 줄이 통과면 첫 바퀴를 돈 것이다.

## 제출 자리

채점기는 먼저 `gym/submissions/<이름>/casual-rides/CR01/` 을 본다.
pack 폴더가 없으면 예전 평면 배치 `gym/submissions/<이름>/CR01/` 로
되돌아간다. 새로 탈 때는 pack 아래가 맞다. 자리는
[14-submissions.md](14-submissions.md) 가 한 장으로 모은다.

## 일부러 틀려 보기

라이브 오라클인지 확인하고 싶으면 `{"pages": 0}` 을 넣고 다시 채점한다.
통과하면 안 된다. 채점기가 골든을 안 보고 `rhwp info` 를 다시 계산하기
때문이다. 확인이 끝나면 올바른 숫자로 되돌린다.

음성 대조(일 안 한 제출)를 전 pack 에 자동으로 넣는 도구는
`gym/tools/discriminate.py` 다. 휴게실 범위 밖이고, 채점 논리도 아니다.

## Windows

```powershell
New-Item -ItemType Directory -Force -Path gym/submissions/나/casual-rides/CR01 | Out-Null
Set-Content -Encoding utf8 gym/submissions/나/casual-rides/CR01/answer.json '{"pages": 3}'
python gym/score.py --agent 나 --pack casual-rides
```

BOM 이 섞이면 JSON 파싱이 깨질 수 있다. [19-windows.md](19-windows.md)
의 UTF-8 without BOM 절차를 본다.

##  copilot 이 아니라 네가 센다

`reference/CR01.json` 은 기준 풀이다. 채점 재현용이지 치트 시트가
아니다. 봐도 채점은 정직하게 돈다. 다만 그때 측정되는 것은 "쪽수를
스스로 읽었는가"가 아니라 "따라 쳤는가"다.

## 다음

관람차 → [03-cr02-ferris.md](03-cr02-ferris.md). 같은 문서
`samples/table-001.hwp` 를 문단 수로 다시 읽는다.
