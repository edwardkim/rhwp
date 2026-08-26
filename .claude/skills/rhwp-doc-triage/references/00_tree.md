# 판단 트리 — info → explain → export-structure → digest → search → extract-data

이 문서는 에이전트가 긴 HWP/HWPX 를 **전문 덤프 없이** 좁혀 읽는 트리의 정본이다.
명령 계약의 권위는 `cli_commands.md` 다. 여기는 순서와 정지다.

## 한 줄

싼 질의부터 내려가고, 질문이 답이면 멈춘다. 6단은 메뉴이지 체크리스트가 아니다.

## 순서

1. `info` — 열림·형식·쪽수·암호
2. `explain` — 결정론 한 줄 (표/누름틀/각주)
3. `export-structure` — 개요 또는 조문 뼈대
4. `digest` — 예산 내 발췌 (기본 0~2쪽)
5. `search` — 사실 + 쪽 주소
6. `extract-data` — 날짜·금액·수량 + 쪽 주소

점프는 허용된다. 예: "어디에 X?" 는 info 직후 search.
되돌아가 덤프하는 것은 금지다.

## 쪽수 밴드

### 밴드 `tiny` — 1~3쪽 (pageCount 1..3)

- 첫 명령: `export-text --json`
- 정지: export-text --json 전문이 컨텍스트에 들어가면 여기서 멈춘다
- 이유: 전문이 한 화면보다 짧아 사다리를 타면 왕복만 늘어난다
- 금지: unlimited export-png, batch of one file
### 밴드 `small` — 4~8쪽 (pageCount 4..8)

- 첫 명령: `info --json 다음 explain --json`
- 정지: explain 한 줄로 종류가 밝혀지고 질문이 종류뿐이면 멈춘다
- 이유: 짧지만 표·누름틀이 있으면 전문보다 구조가 싸다
- 금지: export-text without --max-chars when only a fact is needed
### 밴드 `medium` — 9~30쪽 (pageCount 9..30)

- 첫 명령: `info --json → explain --json → digest --json --max-chars`
- 정지: digest excerpt+outline 으로 질문에 답하면 멈춘다
- 이유: 전문 덤프가 컨텍스트를 밀어내기 시작하는 구간
- 금지: export-text 무제한, 전 쪽 export-png
### 밴드 `large` — 31~100쪽 (pageCount 31..100)

- 첫 명령: `info --json → digest --json --max-chars 800`
- 정지: search/extract-data 가 주소를 주면 그 쪽만 후속
- 이유: 편람·업무계획 규모. 첫 3쪽 발췌로 뒤를 판단하면 틀린다
- 금지: export-text 무제한, digest excerpt 를 문서 전체로 읽기
### 밴드 `huge` — 101쪽 이상 (pageCount 101..∞)

- 첫 명령: `info --json → digest --json --max-chars 600 → search --limit 20`
- 정지: 질문에 답하는 매치/항목이 나오면 즉시 멈춘다
- 이유: 컨텍스트 예산의 적. 좁히지 않으면 실패가 기본값이다
- 금지: export-text 무제한, digest --pages 0..last 한 방에, 전 쪽 PNG


## 사용자 발화 → 진입 단

| 여정 | 발화 | 진입 | 정지 |
| --- | --- | --- | --- |
| J01 | 이 hwp 뭐야? | info → explain | S04 |
| J02 | 몇 쪽짜리야? 암호 걸려 있어? | info | S03 |
| J03 | 목차만 뽑아줘 | info → export-structure | S05 |
| J04 | 긴 문서인데 다 읽지 말고 훑어줘 | info → digest | S06 |
| J05 | 어디에 위임전결이 나와? | info → search | S07 |
| J06 | 금액만 쪽과 함께 뽑아줘 | info → extract-data | S09 |
| J07 | 이 서식 채울 수 있어? | info → explain | S11 |
| J08 | 표가 많은 문서야? | info → explain | S11 |
| J09 | 짧은 메모 전문 보여줘 | info → export-text | tiny band |
| J10 | 뒷부분도 더 읽어줘 | digest --pages | S10 |
| J11 | 절 단위로 나눠서 요약해 | digest --sections | S06 |
| J12 | 폴더에서 위임전결 있는 파일만 | batch search | S14 |
| J13 | 암호 걸린 문서 열어봐 | info | S02 |
| J14 | 이 문서 보내도 돼? | info → handoff security-sweep | S12 |
| J15 | 날짜가 언제로 되어 있어? | extract-data --kind date | S09 |

## 트리 운용 규칙 T01~T80

1. 질문이 종류면 explain 에서 끊고, 목치면 structure, 사실이면 search 로 점프한다 (트리 규칙 T01).
2. info 없이 pageCount 를 추측해 export-text 를 고르지 않는다 (트리 규칙 T02).
3. 같은 파일을 같은 명령으로 세 번 이상 돌리지 않는다. 결과가 같으면 멈춘다 (트리 규칙 T03).
4. digest nextStep 문자열을 고쳐 쓰지 않는다. 받아 적는다 (트리 규칙 T04).
5. search --limit 없이 수백 쪽을 치면 컨텍스트가 먼저 죽는다 (트리 규칙 T05).
6. extract-data --kind all 은 날짜·금액·수량을 한 번에 준다. 질문 축만 고른다 (트리 규칙 T06).
7. 폴더는 단건 사다리보다 batch 선별이 먼저다 (트리 규칙 T07).
8. 암호 재시도는 비밀번호가 있을 때만 한다 (트리 규칙 T08).
9. 시각 확인은 매치 쪽만. 미리보기 전 쪽은 트리아지가 아니다 (트리 규칙 T09).
10. 사다리는 아래로만 간다. 답을 얻은 뒤 위로 돌아가 덤프하지 않는다 (트리 규칙 T10).

## 의사코드

```
doc = path
intent = classify(user)
meta = rhwp info doc --json
if meta failed: handle_open_failure(); stop
band = band_of(meta.pageCount)
if intent == kind_only: explain; maybe_handoff; stop
if intent == toc: export-structure; stop
if intent == skim: digest --max-chars; announce truncated; stop
if intent == fact: search --limit; answer pages; stop
if intent == numbers: extract-data --kind; answer; stop
if band == tiny: export-text --json; stop
# default long-doc
digest --max-chars
if still_needed: search or extract-data or digest --pages
never export-text unlimited
never render all pages
```

## 실패 갈래

| 증상 | exit | 다음 |
| --- | --- | --- |
| 파일 없음·파싱 실패 | 1 | 중단. 다른 명령으로 우회 금지 |
| 암호, 비밀번호 없음 | 2 | 비밀번호를 묻는다 |
| 암호 틀림 | 1 | 한 번만 재확인. 무한 추측 금지 |
| 사용법 (`--limit 0`) | 2 | 명령을 고친다. 덤프로 우회 금지 |
| 매치 0건 | 0 | 어휘 변경 1~2회. 그다음 "없다" |

## 관련

- 정지: [07_when_to_stop.md](07_when_to_stop.md)
- 예산: [10_context_budget.md](10_context_budget.md)
- 덤프 금지: [15_anti_dump.md](15_anti_dump.md)
