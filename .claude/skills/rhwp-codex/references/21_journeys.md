# 항해 여정

기계 가독 정본: `fixtures/journeys.json` (40건).
각 여정은 기존 명령만 나열한다. gym 아님. 아래는 핵심 여정만 풀어 적는다.

## J001 — 처음 보는 공문 — 쪽수만

- 갈래: 파악
- 정지: X01
- 결과: 전문 덤프 없음
- 단계:
  1. `info --json`
  1. `pageCount·format 보고 정지`

## J002 — 처음 보는 서식 — 누름틀 조사

- 갈래: 파악
- 정지: X13
- 결과: 채움은 rhwp-form-fill
- 단계:
  1. `info --json`
  1. `explain --json`
  1. `fields --json`

## J003 — 긴 업무계획 — 발췌만

- 갈래: 파악
- 정지: X10
- 결과: export-text 전문 금지
- 단계:
  1. `info --json`
  1. `digest --json`
  1. `nextStep 따라 search 또는 export-text -p`

## J004 — 단어 위치만

- 갈래: 파악
- 정지: X10
- 결과: 주소 있는 검색
- 단계:
  1. `search --json`
  1. `matches[].page 로 쪽 지정 export-text`

## J005 — 표 CSV 왕복

- 갈래: 수확
- 정지: X13
- 결과: 치수 계약
- 단계:
  1. `export-tables --json`
  1. `table-to-csv --table N`
  1. `외부 편집`
  1. `csv-to-table --dry-run`
  1. `csv-to-table --verify`

## J006 — 날짜·금액 수확

- 갈래: 수확
- 정지: X12
- 결과: 값은 출처 표지
- 단계:
  1. `extract-data --json`
  1. `주소 필드로 근거 인용`

## J007 — 폴더 발견 후 일괄

- 갈래: 대량
- 정지: X13
- 결과: rhwp-bulk-pipeline
- 단계:
  1. `scan`
  1. `목록을 batch info --json`

## J008 — 문구 치환 안전

- 갈래: 편집
- 정지: X11
- 결과: 원본 불변
- 단계:
  1. `search 로 횟수 확인`
  1. `edit replace-text --dry-run --json`
  1. `edit replace-text -o --json`

## J009 — 표 칸 한 곳

- 갈래: 편집
- 정지: X11
- 결과: 눈에 보이는 표가 0번이 아닐 수 있음
- 단계:
  1. `export-tables 로 표 번호`
  1. `edit set-cell --dry-run`
  1. `edit set-cell -o --keep-style?`

## J010 — 누름틀 단건

- 갈래: 편집
- 정지: X13
- 결과: rhwp-form-fill
- 단계:
  1. `fields --json`
  1. `edit fill-fields --dry-run`
  1. `notFound/ambiguous 확인`
  1. `fill-fields -o --verify`

## J011 — 다단계 원자

- 갈래: 편집
- 정지: X11
- 결과: 선검증 후 한 번 저장
- 단계:
  1. `export-plan-schema`
  1. `run --dry-run`
  1. `run 계획.json`

## J012 — HWP→HWPX 검증

- 갈래: 변환
- 정지: X08
- 결과: exit 3 = 판정
- 단계:
  1. `export-hwpx --verify --json`
  1. `identical false 면 ir-diff`

## J013 — 배포용→HWP5

- 갈래: 변환
- 정지: X08
- 결과: 재파싱 게이트
- 단계:
  1. `convert --verify --json`
  1. `exit 3/4 를 데이터로`

## J014 — 레이아웃 회귀

- 갈래: 변환
- 정지: X13
- 결과: rhwp-visual-regression
- 단계:
  1. `render-diff --json`
  1. `STRUCT_MISMATCH 를 경로로 읽기`

## J015 — 쪽수 단언

- 갈래: 검증
- 정지: X08
- 결과: exit 3
- 단계:
  1. `verify --expect-pages N --json`
  1. `불일치 시 failed 필드`

## J016 — 작업 영수증

- 갈래: 검증
- 정지: X13
- 결과: rhwp-work-receipt
- 단계:
  1. `replay --plan-json --json`
  1. `3해시 전달`

## J017 — 폴더 감사

- 갈래: 검증
- 정지: X08
- 결과: 1건 실패도 exit 3
- 단계:
  1. `audit <폴더> --json`
  1. `reproducedRate`

## J018 — 계보

- 갈래: 검증
- 정지: X08
- 결과: 부모 산출=자식 입력
- 단계:
  1. `lineage --deep --json`
  1. `brokenAt`

## J019 — 수신 문서 3축

- 갈래: 보안
- 정지: X13
- 결과: rhwp-security-sweep
- 단계:
  1. `inspect injection --json`
  1. `inspect hidden-text --json`
  1. `inspect unicode --json`

## J020 — LLM 투입 전 armor

- 갈래: 보안
- 정지: X12
- 결과: 지시에 읽지 말 것
- 단계:
  1. `inspect injection`
  1. `armor --json`
  1. `격벽 사이만 본문으로`

## J021 — 폴더 수백 텍스트

- 갈래: 대량
- 정지: X13
- 결과: N=성공+실패
- 단계:
  1. `목록 stdin`
  1. `batch export-text --json`
  1. `실패 행 jq`

## J022 — 세션 반복 조회

- 갈래: 대량
- 정지: X13
- 결과: rhwp-mcp-session
- 단계:
  1. `capabilities --mcp`
  1. `mcp-serve`
  1. `hwp_open → hwp_doc_* → hwp_close`

## J023 — 명령 미지 — 검색 폴백

- 갈래: 자기서술
- 정지: X03
- 결과: 발명 금지
- 단계:
  1. `capabilities --search 키워드`
  1. `장 번호로 이동`
  1. `0건이면 거절`

## J024 — 대전 신선도

- 갈래: 유지보수
- 정지: X05
- 결과: DATA = stale
- 단계:
  1. `python tools/gen_agent_codex.py --check`
  1. `exit 3 이면 재생성`
  1. `생성 장 수기 금지`

## J025 — 필드 사전 조회

- 갈래: 자기서술
- 정지: X06
- 결과: 경계
- 단계:
  1. `지식지도 §2-2 를 연다`
  1. `스킬에 사전 복제 금지`

## J026 — 개발자가 렌더 버그

- 갈래: 진단
- 정지: X07
- 결과: 통상 작업 금지
- 단계:
  1. `85장은 개발자 전용임을 고지`
  1. `rhwp-cli 디버깅 순서로만`

## J027 — info 신상 field-01.hwp

- 갈래: 파악
- 정지: X01
- 결과: 쪽수·형식 · fixture 0
- 단계:
  1. `info samples/field-01.hwp --json`

## J028 — explain 한 봉투 field-01.hwp

- 갈래: 파악
- 정지: X01
- 결과: 표·필드 유무 · fixture 0
- 단계:
  1. `explain samples/field-01.hwp --json`

## J029 — fields field-01.hwp

- 갈래: 파악
- 정지: X13
- 결과: fieldCount · fixture 0
- 단계:
  1. `fields samples/field-01.hwp --json`

## J030 — export-tables field-01.hwp

- 갈래: 수확
- 정지: X13
- 결과: 표 번호 · fixture 0
- 단계:
  1. `export-tables samples/field-01.hwp --json`

## J031 — replace-text dry-run field-01.hwp

- 갈래: 편집
- 정지: X11
- 결과: 디스크 무변경 · fixture 0
- 단계:
  1. `edit replace-text samples/field-01.hwp --find x --replace y --dry-run --json`

## J032 — ir-diff 자기 field-01.hwp

- 갈래: 검증
- 정지: X08
- 결과: identical · fixture 0
- 단계:
  1. `ir-diff samples/field-01.hwp samples/field-01.hwp --json`

나머지 8건은 `fixtures/journeys.json` 만 본다.
