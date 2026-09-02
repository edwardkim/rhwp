# 운동장 리더보드 — 위조 불가능한 점수판

모든 순위는 검증 사슬(3해시 고정·Ed25519 서명·append-only 원장·머클 앵커)을
**렌더 시점에 재검증**한 항목만 오른다. 재현 방법:
`python gym/tools/leaderboard.py verify`

| 순위 | 에이전트 | 총점 | 최강 능력 | commit | seq | 사슬 |
|---|---|---|---|---|---|---|
| 1 | claude-fable-5 | **194 / 194** | automation 35/35 | `9785c7e802` | 0 | 검증됨 |
| 2 | atlas-generalist | **185 / 194** | core-cli 32/32 | `dc7a303599` | 1 | 검증됨 |
| 3 | probe-diagnostician | **145 / 194** | automation 35/35 | `dc7a303599` | 2 | 검증됨 |
| 4 | scribe-editor | **133 / 194** | core-cli 32/32 | `dc7a303599` | 3 | 검증됨 |
| 5 | novice-starter | **44 / 194** | core-cli 32/32 | `dc7a303599` | 4 | 검증됨 |

## 능력 격자 (pack 별 점수)

| 에이전트 | automation | core-cli | corpus-diagnostics | layout-rendering | objects-media | security | self-description | serialization | table-editing | text-editing |
|---|---|---|---|---|---|---|---|---|---|---|
| claude-fable-5 | **35** | **32** | **14** | **15** | **15** | **18** | **12** | **19** | **16** | **18** |
| atlas-generalist | 26/35 | **32** | **14** | **15** | **15** | **18** | **12** | **19** | **16** | **18** |
| probe-diagnostician | **35** | **32** | **14** | **15** | 0/15 | **18** | **12** | **19** | 0/16 | 0/18 |
| scribe-editor | 0/35 | **32** | 0/14 | **15** | **15** | **18** | 0/12 | **19** | **16** | **18** |
| novice-starter | 0/35 | **32** | 0/14 | 0/15 | 0/15 | 0/18 | **12** | 0/19 | 0/16 | 0/18 |

`—` = 미제출(그 pack 을 아예 풀지 않음) · **굵게** = 만점

원장 체인: 무결 · 항목 5 · 검증 5 · unverified 0

정직 조항: 이 사슬이 봉인하는 것은 "이 스코어카드가 이 시점에 이 신원으로
등재되었고 이후 변조되지 않았다" 까지다. 채점 자체의 재현은 스코어카드의
runner 신원(version·commit·capabilities digest)과 커밋된 제출물로 제3자가 수행한다.
