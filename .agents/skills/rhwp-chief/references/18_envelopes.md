# 18. 봉투 필드

Chief 층에서 기계가 읽는 JSON 은 세 종류다.

## result.json (루프 산출)

| 키 | 형 | 의미 |
| --- | --- | --- |
| `schemaVersion` | `"1"` | 루프 산출 버전 |
| `generatedBy` | `tools/chief/service_loop.py` | 출처 |
| `goal` | 문자열 | `normalize_goal` 결과 |
| `route` | 문자열 또는 null | 트리아지 라우트 |
| `status` | 열거 | done/failed/needs-agent/escalated/invalid-input |
| `reason` | 문자열 | 실패·정지 사유 (있을 때) |
| `summary` | 문자열 | done 요약 |
| `artifacts` | 문자열 배열 | `out/` 안 파일 이름 |

픽스처 스냅샷은 `stop` 을 더 실을 수 있다. 루프 본체는 아직 `stop` 키를
쓰지 않는다. 문서 가독용이다.

## ticket.json (FDE 산출)

루프가 `-o` 로 받는다. 최소 `route`. 보통 `routeReason`, `steps`.
스키마의 정본은 FDE 층. 여기서 확장하지 않는다.

## 실행 봉투 (rhwp --json)

| goal | 읽는 것 |
| --- | --- |
| export-text | 전체 JSON. `pageCount` 만 요약에 |
| extract-tables | `tables[].index` |
| fill | `notFound`, `ambiguous`, `confusable`, `filledCount` |

export-pdf / export-hwpx / convert 는 JSON 봉투가 아니라 파일+exit+매직.

## capabilities --json

기동 시 한 번. `commands[].name` 집합. 핸들러 `needs:` 와 교집합.
조회 실패 시 실행 전면 정지 (C07).
