# 10 — 컨텍스트 예산

에이전트 실패의 대부분은 모델이 못 읽어서가 아니라 **너무 많이 넣어서**다.

## 예산 단위

| 장치 | 기본 | 긴 문서에서 |
| --- | --- | --- |
| digest --max-chars | 2000 (excerpt) | 600~1000 |
| digest --sections 절별 | 240 | 240 유지, 필요한 절만 |
| search --limit | 무제한 | 20 |
| extract-data --limit | 무제한 | 질문 축 + 50 |
| export-text --max-chars | 무제한 (위험) | 쪽을 고른 뒤 2000~4000 |
| export-png | 쪽당 이미지 | 매치 1~3쪽만 |

## 규칙

1. 예산을 걸 수 있는 명령에 예산을 안 거는 것은 버그로 본다.
2. `truncated:true` 는 실패가 아니다. 총량을 보고 멈출지 창을 옮길지 고른다.
3. 같은 창을 한도만 키워 반복하지 않는다. 창을 옮긴다 (`--pages a..b`).
4. 프롬프트에 넣는 것은 주소+짧은 발췌다. 봉투 원문 전체를 붙이지 않는다.

## 밴드별 반복 B01~

1. 1~3쪽 문서는 `export-text --json` 로 시작하고 한 호출 예산을 넘기면 `export-text --json 전문이 컨텍스트에 들어가면 여기서 멈춘다`. 금지: unlimited export-png, batch of one file (B01).
2. 4~8쪽 문서는 `info --json 다음 explain --json` 로 시작하고 한 호출 예산을 넘기면 `explain 한 줄로 종류가 밝혀지고 질문이 종류뿐이면 멈춘다`. 금지: export-text without --max-chars when only a fact is needed (B02).
3. 9~30쪽 문서는 `info --json → explain --json → digest --json --max-chars` 로 시작하고 한 호출 예산을 넘기면 `digest excerpt+outline 으로 질문에 답하면 멈춘다`. 금지: export-text 무제한, 전 쪽 export-png (B03).
4. 31~100쪽 문서는 `info --json → digest --json --max-chars 800` 로 시작하고 한 호출 예산을 넘기면 `search/extract-data 가 주소를 주면 그 쪽만 후속`. 금지: export-text 무제한, digest excerpt 를 문서 전체로 읽기 (B04).
5. 101쪽 이상 문서는 `info --json → digest --json --max-chars 600 → search --limit 20` 로 시작하고 한 호출 예산을 넘기면 `질문에 답하는 매치/항목이 나오면 즉시 멈춘다`. 금지: export-text 무제한, digest --pages 0..last 한 방에, 전 쪽 PNG (B05).
6. 1~3쪽 문서는 `export-text --json` 로 시작하고 한 호출 예산을 넘기면 `export-text --json 전문이 컨텍스트에 들어가면 여기서 멈춘다`. 금지: unlimited export-png, batch of one file (B06).
7. 4~8쪽 문서는 `info --json 다음 explain --json` 로 시작하고 한 호출 예산을 넘기면 `explain 한 줄로 종류가 밝혀지고 질문이 종류뿐이면 멈춘다`. 금지: export-text without --max-chars when only a fact is needed (B07).
8. 9~30쪽 문서는 `info --json → explain --json → digest --json --max-chars` 로 시작하고 한 호출 예산을 넘기면 `digest excerpt+outline 으로 질문에 답하면 멈춘다`. 금지: export-text 무제한, 전 쪽 export-png (B08).
9. 31~100쪽 문서는 `info --json → digest --json --max-chars 800` 로 시작하고 한 호출 예산을 넘기면 `search/extract-data 가 주소를 주면 그 쪽만 후속`. 금지: export-text 무제한, digest excerpt 를 문서 전체로 읽기 (B09).
10. 101쪽 이상 문서는 `info --json → digest --json --max-chars 600 → search --limit 20` 로 시작하고 한 호출 예산을 넘기면 `질문에 답하는 매치/항목이 나오면 즉시 멈춘다`. 금지: export-text 무제한, digest --pages 0..last 한 방에, 전 쪽 PNG (B10).
11. 1~3쪽 문서는 `export-text --json` 로 시작하고 한 호출 예산을 넘기면 `export-text --json 전문이 컨텍스트에 들어가면 여기서 멈춘다`. 금지: unlimited export-png, batch of one file (B11).
12. 4~8쪽 문서는 `info --json 다음 explain --json` 로 시작하고 한 호출 예산을 넘기면 `explain 한 줄로 종류가 밝혀지고 질문이 종류뿐이면 멈춘다`. 금지: export-text without --max-chars when only a fact is needed (B12).
13. 9~30쪽 문서는 `info --json → explain --json → digest --json --max-chars` 로 시작하고 한 호출 예산을 넘기면 `digest excerpt+outline 으로 질문에 답하면 멈춘다`. 금지: export-text 무제한, 전 쪽 export-png (B13).
14. 31~100쪽 문서는 `info --json → digest --json --max-chars 800` 로 시작하고 한 호출 예산을 넘기면 `search/extract-data 가 주소를 주면 그 쪽만 후속`. 금지: export-text 무제한, digest excerpt 를 문서 전체로 읽기 (B14).
15. 101쪽 이상 문서는 `info --json → digest --json --max-chars 600 → search --limit 20` 로 시작하고 한 호출 예산을 넘기면 `질문에 답하는 매치/항목이 나오면 즉시 멈춘다`. 금지: export-text 무제한, digest --pages 0..last 한 방에, 전 쪽 PNG (B15).
## 쪽수별 예산 표

상세 1..220은 [18_pagecount_routing.md](18_pagecount_routing.md).
