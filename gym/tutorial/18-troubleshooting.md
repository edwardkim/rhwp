---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 18. 막혔을 때

돌아가기: [README.md](README.md) · FAQ: [17-faq.md](17-faq.md)
· Windows: [19-windows.md](19-windows.md)

이 페이지는 입문 방문자가 실제로 걸리는 자리만 적는다. 새 진단 명령을
만들지 않는다. 있는 CLI 와 채점기 출력으로 좁힌다.

## 1. `rhwp` 를 못 찾는다

증상: `rhwp: command not found` 또는 PowerShell 의
`용어 'rhwp'이(가) cmdlet … 인식되지 않습니다`.

1. 저장소 루트인지 확인한다.
2. `cargo build --bin rhwp` 를 돌린다.
3. 채점기에 바이너리를 직접 넘긴다.

```bash
python gym/score.py --agent 나 --profile family --bin target/debug/rhwp
```

Windows 는 `target\debug\rhwp.exe`.

## 2. 프로파일 파일을 못 연다

증상: `FileNotFoundError` 가 `gym/profiles/…json` 을 가리킨다.

철자를 [06-profiles.md](06-profiles.md) 의 일곱 이름과 대조한다.
`--profile casual` 은 없다. `--pack casual-rides` 또는
`--profile family`.

## 3. 4/4 가 나왔는데 내가 탄 것 같지 않다

기준 풀이가 먼저 돈 것이다. 제출 폴더에 `answer.json` 을 놓고 틀린
숫자를 넣으면 그 과제만 떨어진다. 그게 라이브 오라클이다.
[02-cr01-carousel.md](02-cr01-carousel.md) 의 "일부러 틀려 보기".

## 4. 과제가 제출 없음으로 떨어진다

1. 폴더가 `gym/submissions/<같은이름>/<pack>/<과제id>/` 인가.
2. `--agent 나` 인데 폴더는 `gym/submissions/me/` 인가.
3. 파일 이름이 `answer.json` / `edited.hwp` 처럼 과제가 적은 그대로인가.
4. JSON 이 BOM 이나 작은따옴표로 깨졌는가.

자리 결은 [14-submissions.md](14-submissions.md).

## 5. `unavailable`

[16-unavailable.md](16-unavailable.md). 먼저
`rhwp capabilities` 가 JSON 을 내는지 본다. 안 나오면 바이너리가
너무 오래됐다.

## 6. 한글 경로에서 search 가 실패한다

T02 입력 `samples/2022년 국립국어원 업무계획.hwp` 는 따옴표가
필요하다. PowerShell 은 작은따옴표가 리터럴이다.

```powershell
rhwp search 'samples/2022년 국립국어원 업무계획.hwp' 국어 --json
```

콘솔이 한글을 `??` 로 바꾸면 코드 페이지 문제다.
[19-windows.md](19-windows.md).

## 7. `attest` 가 스코어카드를 못 찾는다

채점을 먼저 한다. `gym/submissions/<이름>/scorecard.json` 과
`admission.json` 이 있어야 한다. 입장 거절(`deny`) 상태면 등재
사슬의 게이트가 막힐 수 있다. 표를 다시 끊는다.
[01-admission.md](01-admission.md).

## 8. `verify` 가 한 줄을 폭로한다

원장이나 스코어카드를 손으로 고친 흔적이다. 로컬 실험을 되돌려
커밋된 리더보드 파일과 맞춘 뒤, 자기 제출만 다시 등재한다.
커밋된 `gym/leaderboard/ledger.ndjson` 을 고쳐서 점수를 올리는
길은 없다. `verify` 가 그 자리를 가리킨다.

## 9. 이 안내와 과제 JSON 이 다르다

과제 파일이 유일 지시서다 (`gym/README.md` 규칙 1). 휴게실은
요약이다. 충돌하면 `packs/<id>/tasks/<과제>.json` 의
`instructions` 를 따른다. 이 문서를 고쳐서 채점을 쉽게 만들지
말고, 과제가 바뀐 것이면 그 pack 의 PR 을 본다. 다른 열린 PR 의
과제 JSON 을 이 가지에서 고치지 않는다.

## 10. 그래도 막히면

- 지도: [../PARK.md](../PARK.md)
- 정직 조항: [15-scoring-honesty.md](15-scoring-honesty.md)
- 첫날 한 장: [20-checklist.md](20-checklist.md)
- 저장소 기여 절차는 휴게실 밖이다. `rhwp-contributor` 스킬과
  `CONTRIBUTING.md`.
