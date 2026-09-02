# 실사용 여정

이슈: #5331. 라우터 장 `18_journeys.md`.
정본 디렉터리: `mydocs/manual/recipes/`.
gym 이 아니고 새 CLI 도 없다. 07·08 을 만들지 않는다.

여정 40개. 전체는 `fixtures/journeys.json`. 아래는 입구 20개.

### J01 — 단건 서식 제출

- 레시피: `01`
- 정지: `R01`
- 단계:
  - fields --json
  - fill-fields --dry-run
  - fill-fields --verify
  - sanitize

### J02 — 표 CSV 왕복

- 레시피: `02`
- 정지: `R01`
- 단계:
  - export-tables --json
  - table-to-csv
  - 외부 편집
  - csv-to-table --verify

### J03 — 배포 전 마스킹

- 레시피: `03`
- 정지: `R01`
- 단계:
  - redact --dry-run --no-raw
  - redact -o --verify --no-raw
  - search 원문 0
  - sanitize
  - 재검사 0

### J04 — 낯선 첨부 수신

- 레시피: `04`
- 정지: `R01`
- 단계:
  - info --json
  - digest --json
  - fields --json
  - 필요 시 search

### J05 — 메일머지 명단

- 레시피: `05`
- 정지: `R01`
- 단계:
  - fields --json
  - batch fill --dry-run
  - batch fill --verify

### J06 — 편집 전후 회귀

- 레시피: `06`
- 정지: `R01`
- 단계:
  - render-diff --via hwpx
  - render-diff before after
  - 노드 경로 대조

### J07 — 폴더 일괄 추출

- 레시피: `09`
- 정지: `R01`
- 단계:
  - 목록 stdin
  - batch info
  - batch export-text
  - 실패 행 재시도
  - N=성공+실패

### J08 — 송신 스윕 게이트

- 레시피: `10`
- 정지: `R01`
- 단계:
  - hidden-text
  - injection
  - unicode
  - redact --dry-run --no-raw
  - 처리
  - 재스윕

### J09 — 07 요청 거절

- 레시피: `07`
- 정지: `R02`
- 단계:
  - 결번 고지
  - 파일을 만들지 않음

### J10 — 08 요청 거절

- 레시피: `08`
- 정지: `R02`
- 단계:
  - 결번 고지
  - 협업 계약을 발명하지 않음

### J11 — 서식 채워줘 모호

- 레시피: `01+05`
- 정지: `R05`
- 단계:
  - 후보 01 과 05 를 보여 줌
  - 건수를 물음

### J12 — 보내도 돼 모호

- 레시피: `03+10`
- 정지: `R05`
- 단계:
  - 후보 03 과 10 을 보여 줌
  - 방향·깊이를 물음

### J13 — 안전한가 모호

- 레시피: `04+10`
- 정지: `R05`
- 단계:
  - 수신인지 송신인지 물음

### J14 — 한꺼번에 모호

- 레시피: `05+09`
- 정지: `R05`
- 단계:
  - 쓰기 명단인지 읽기 폴더인지 물음

### J15 — 표 뽑아줘 모호

- 레시피: `02+09`
- 정지: `R05`
- 단계:
  - 단건인지 폴더인지 물음

### J16 — stale last_verified

- 레시피: `stale`
- 정지: `R04`
- 단계:
  - 날짜를 보여 줌
  - 순서를 추측하지 않음

### J17 — 레시피 파일 없음

- 레시피: `missing`
- 정지: `R03`
- 단계:
  - 경로를 보여 줌
  - 대체본을 쓰지 않음

### J18 — 낯선 첨부인데 채움 요청

- 레시피: `04`
- 정지: `R06`
- 단계:
  - 04 의 info 부터
  - 01 로 바로 가지 않음

### J19 — 명단인데 단건 fill

- 레시피: `05`
- 정지: `R08`
- 단계:
  - 05 로 보냄
  - stdin 목록을 fill 에 넣지 않음

### J20 — 폴더인데 batch fill

- 레시피: `09`
- 정지: `R09`
- 단계:
  - 09 의 batch info
  - fill 은 --data 행

나머지 여정(카드 첫 정지·두 장 충돌·게이트)은 JSON 을 연다.
