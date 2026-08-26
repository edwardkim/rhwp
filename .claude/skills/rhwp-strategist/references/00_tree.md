# 00 스킬 나무

이 장은 rhwp-strategist 스킬이 **무엇이고 무엇이 아닌지** 한 장에 고정한다.
정본은 playbook 이다. 이 문서는 스킬 라우팅용 축소판이다.

## 한 문장

고객 목표와 문서 코퍼스를 받아, 엔진이 전수 지도와 좌표 대장을 만들고,
에이전트가 대장 안에서만 주장한 뒤 §5 게이트를 통과한 산출물을 납품한다.

## 세 보장 (엔진)

1. 전수성 — 모든 `.hwp`/`.hwpx` 가 `corpus_map.json` 에 나타난다.
2. 좌표 — 봉투가 준 `section`/`paragraph`/`page`/`charOffset` 만 옮긴다.
3. 게이트 — 대장 밖 주장은 `--validate` 가 거부한다.

엔진은 전략을 만들지 않는다. "무엇을 주장할지"는 에이전트, "그 주장이
실좌표에 붙었는지"는 엔진이다.

## 네 산출

| 파일 | 단계 | 내용 |
| --- | --- | --- |
| `corpus_map.json` | A | 문서별 `info --json` (+`explain`). 실패는 `status:failed` |
| `evidence.json` | B | EV-n 목록. search 매치 + extract-data 날짜·금액 |
| `spec.json` | C | CLAIM 플레이스홀더 + 근거 연결표 |
| 판정 봉투 | D | `verdict` / `violations` / 선택적 `swsAudit` |

## 명령 사다리 (발명 금지)

```
capabilities          광고 확인. 미광고 명령을 추측 실행하지 않는다
info --json           문서 1건 지도. 실패해도 행을 남긴다
explain --json        광고된 경우에만
search --json         질문 키워드. --limit 절단은 truncated 로 남긴다
extract-data --json   광고된 경우 date/amount
scaffold              광고된 경우에만 spec → hwpx
```

허용 표면: `info`, `search`, `extract-data`, `explain`, `scaffold`,
`capabilities`. 엔진 엔트리: `python3 tools/strategist/engagement.py`.

금지 발명: `rhwp strategy`, `rhwp claim-check`, `rhwp forecast`,
`rhwp evidence-ledger`, `rhwp claim-gate`.

## gym · 이웃 층

- gym 과제·채점·admission·리더보드는 이 스킬의 경로가 아니다.
- FDE 는 라이브 증상, Chief 는 요청 큐, 이 스킬은 목표+코퍼스.
- bug-hunter 는 정답지 대조 결함 발굴. 전략 산출물이 아니다.

## 트리거 문장

- "이 문서들로 전략 보고서/제안서 만들어줘"
- "정부과제 수주 근거를 쪽 번호로 남겨줘"
- "엔게이지먼트 수행 / 근거 대장"
- "주장마다 원문 좌표가 필요해"
- chief 큐의 목표형 요청 ("~하고 싶다", "~전략이 필요하다")

증상("표가 깨져")이나 큐 운영("오늘 요청 처리해")은 이 나무가 아니다.

## 정지 (이 장에서 바로)

| id | 언제 | 동작 |
| --- | --- | --- |
| ST-FORECAST | 출처 없는 전망을 쓰라는 요청 | 거부. 질문 보강 후 엔진 재실행만 허용 |
| ST-INVENT-PAGE | page 없는 매치에 쪽을 추정 | 거부. 키 생략 |
| ST-DROP-FAILED | 실패 문서를 코퍼스에서 삭제 | 거부. failed 행 유지 |
| ST-SKIP-ENGINE | 즉흥 search 로 주장 초안 | 거부. engagement.py 부터 |
| ST-GATE-FAIL | validate exit 3 | 납품 금지. 위반 수정 |

다음: [01_playbook_authority.md](01_playbook_authority.md).
