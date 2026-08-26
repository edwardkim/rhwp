---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 11. boss 길 — 한 단만 틀려도 막힌다

`boss` 프로파일은 `expert-challenges` 만 고른다. 정본은
`gym/profiles/boss.json`. 테마파크 지도의 🐉 보스존과 같은 pack 이다.

돌아가기: [README.md](README.md) · 지도: [../PARK.md](../PARK.md)

```bash
python gym/score.py --agent 나 --profile boss
```

## 왜 먼저 타지 않나

입문존은 숫자 하나다. 보스존은 서명·앵커·정산·계보·감사 표준을 한
제출로 묶는다. 키 제한(tier)이 4~5 다. 한 단계가 빠지면 최종 판정이
막힌다. 자이로드롭에서 안전바 하나만 안 걸려도 출발하지 않는 것과
같다. 은유는 [../PARK.md](../PARK.md) 보스존 절과 같다.

못 푸는 보스는 열지 않는다는 것이 정직 조항이다. 다섯 과제 모두
`gym/packs/expert-challenges/reference/` 기준 풀이 왕복을 통과한
뒤에야 등재됐다. 이 안내가 여섯 번째 보스를 슬쩍 끼워 넣지 않는다.

## XC01 — 사다리 완주 (적합성 L5)

정본: `gym/packs/expert-challenges/tasks/XC01.json`

제출 폴더에 다음을 둔다.

- `caps/work.capsule.json` (서명한 작업 캡슐)
- `keyring.json`
- `anchor.ndjson`
- `policy.json`
- `ledger.ndjson`

채점은 `rhwp conformance {file:caps} --level L5 --deep …` 의
`verdict` 가 `conformant` 인지 본다. 종료 코드 0 과 3 을 허용한다.
한 파일이 빠지면 L5 는 막힌다.

힌트 사슬: `keygen` → `replay --sign-key` → `anchor add` →
`settle propose/record` → `conformance --level L5 --deep`.

이 사다리가 [에이전트 작업 표준(AWS) 1.0](../../mydocs/tech/standards/agent_work_standard.md)
의 AW-L1~L5 를 과제로 소비한다. 운동장은 그 표준을 다시 정의하지
않는다.

## 나머지 보스 (이름만)

정본은 각 `tasks/XC0n.json` 이다. 여기서는 지도만 옮긴다.

| id | tier | 무엇을 완주하나 | 최종 판정 |
|---|---|---|---|
| XC01 | 5 | 서명→앵커→정산까지 전 사다리 | 적합성 L5 conformant |
| XC02 | 5 | 2세대 계보에서 오염 전파 | 오염원의 후손까지 리콜 |
| XC03 | 4 | 청구→원장→검증 | 캡슐·게이트·원장·워크오더 4관문 |
| XC04 | 4 | 끊기지 않는 3세대 사슬 | lineage valid · depth 3 |
| XC05 | 5 | 감사 리포트 + 서명 귀속 | 적합성 L3 conformant |

세부 힌트는 과제 `instructions` 가 유일 지시서다. 휴게실이 힌트를
늘려 채점을 쉽게 만들지 않는다.

## 보스존과 전당

보스 점수도 다른 pack 과 같이 스코어카드에 남는다. 전당 등재는
여전히 `attest` 다. 보스만 탔어도 표는 끊긴다(`packsScored >= 1`).
만점이어야 이름이 오르는 것이 아니다.

```bash
python gym/tools/leaderboard.py attest --agent 나
python gym/tools/leaderboard.py verify
```

→ [12-leaderboard.md](12-leaderboard.md)
