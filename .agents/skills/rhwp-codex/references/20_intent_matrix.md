# 발화 → 갈래 → 장

기계 가독 정본: `fixtures/intent_matrix.json` (90행).
모든 command 는 기존 표면이다. 발명 명령 0.

| ID | 발화 | 갈래 | 명령 | 장 | 정지 |
|---|---|---|---|---|---|
| I001 | 이 문서 뭐야 | 파악 | `info` | 10_조회.md | X01 |
| I002 | 파일 형식과 쪽수 알려줘 | 파악 | `info` | 10_조회.md | X01 |
| I003 | 제목이 뭐로 들어 있어 | 파악 | `info` | 10_조회.md | X01 |
| I004 | recap 한 줄로 | 파악 | `explain` | 10_조회.md | X01 |
| I005 | 한 봉투로 요약해 | 파악 | `explain` | 10_조회.md | X01 |
| I006 | 표랑 누름틀 있어? | 파악 | `explain` | 10_조회.md | X01 |
| I007 | 암호 걸려 있어? | 파악 | `explain` | 10_조회.md | X01 |
| I008 | 목차만 보여줘 | 파악 | `export-structure` | 10_조회.md | X01 |
| I009 | 조문 구조 뽑아 | 파악 | `export-structure` | 10_조회.md | X01 |
| I010 | 개요 계층 | 파악 | `export-structure` | 10_조회.md | X01 |
| I011 | 긴 문서 발췌만 | 파악 | `digest` | 10_조회.md | X01 |
| I012 | RAG 청킹해 줘 | 파악 | `digest` | 10_조회.md | X01 |
| I013 | 이 단어가 어디에 있어 | 파악 | `search` | 10_조회.md | X01 |
| I014 | 국어 라는 말 찾아 | 파악 | `search` | 10_조회.md | X01 |
| I015 | 본문 원문 0쪽만 | 파악 | `export-text` | 10_조회.md | X01 |
| I016 | 전문 말고 한 쪽만 텍스트 | 파악 | `export-text` | 10_조회.md | X01 |
| I017 | 누름틀 이름이 뭐야 | 파악 | `fields` | 10_조회.md | X01 |
| I018 | 서식이야 일반 문서야 | 파악 | `fields` | 10_조회.md | X01 |
| I019 | fields 조사만 | 파악 | `fields` | 10_조회.md | X01 |
| I020 | 페이지 몇 장이야 | 파악 | `info` | 10_조회.md | X01 |
| I021 | 글꼴 목록 | 파악 | `info` | 10_조회.md | X01 |
| I022 | 이 HWP 신상 | 파악 | `info` | 10_조회.md | X01 |
| I023 | explain 돌려 | 파악 | `explain` | 10_조회.md | X01 |
| I024 | digest 로 줄여 읽어 | 파악 | `digest` | 10_조회.md | X01 |
| I025 | search 주소 포함해서 | 파악 | `search` | 10_조회.md | X01 |
| I026 | export-structure 목차 | 파악 | `export-structure` | 10_조회.md | X01 |
| I027 | 처음 보는 공문 파악 | 파악 | `info` | 10_조회.md | X10 |
| I028 | 문맥 아끼며 읽어 | 파악 | `info` | 10_조회.md | X10 |
| I029 | 덤프 하지 말고 좁혀 | 파악 | `info` | 10_조회.md | X10 |
| I030 | 쪽 주소로 찾아 | 파악 | `search` | 10_조회.md | X01 |
| I031 | 표 뽑아 줘 | 수확 | `export-tables` | 20_표와_데이터.md | X13 |
| I032 | 엑셀로 보고 싶어 | 수확 | `table-to-csv` | 20_표와_데이터.md | X13 |
| I033 | CSV 로 추출 | 수확 | `table-to-csv` | 20_표와_데이터.md | X13 |
| I034 | 병합 칸 유지해서 | 수확 | `export-tables` | 20_표와_데이터.md | X13 |
| I035 | 표 좌표 먼저 | 수확 | `export-tables` | 20_표와_데이터.md | X13 |
| I036 | 고친 CSV 다시 넣어 | 수확 | `csv-to-table` | 20_표와_데이터.md | X13 |
| I037 | 치수 계약 확인해 | 수확 | `csv-to-table` | 20_표와_데이터.md | X13 |
| I038 | 날짜랑 금액 추출 | 수확 | `extract-data` | 20_표와_데이터.md | X13 |
| I039 | 수량만 모아 | 수확 | `extract-data` | 20_표와_데이터.md | X13 |
| I040 | 폴더에서 hwp 찾아 | 수확 | `scan` | 20_표와_데이터.md | X13 |
| I041 | 확장자 안 맞는 파일 | 수확 | `scan` | 20_표와_데이터.md | X13 |
| I042 | 차트 숫자를 CSV 로 | 수확 | `chart-to-csv` | 20_표와_데이터.md | X13 |
| I043 | 차트에 CSV 되넣기 | 수확 | `csv-to-chart` | 20_표와_데이터.md | X13 |
| I044 | table-to-csv --table 1 | 수확 | `table-to-csv` | 20_표와_데이터.md | X13 |
| I045 | export-tables 전량 | 수확 | `export-tables` | 20_표와_데이터.md | X13 |
| I046 | extract-data 주소 포함 | 수확 | `extract-data` | 20_표와_데이터.md | X13 |
| I047 | scan 으로 발견 | 수확 | `scan` | 20_표와_데이터.md | X13 |
| I048 | pandas 에 넣을 표 | 수확 | `table-to-csv` | 20_표와_데이터.md | X13 |
| I049 | 표 몇 개야 | 수확 | `export-tables` | 20_표와_데이터.md | X13 |
| I050 | 중첩 표 구조 | 수확 | `export-tables` | 20_표와_데이터.md | X13 |
| I051 | 캡션 있는 표만 | 수확 | `export-tables` | 20_표와_데이터.md | X13 |
| I052 | 무역 통계 숫자 | 수확 | `extract-data` | 20_표와_데이터.md | X13 |
| I053 | 업무계획 표 | 수확 | `export-tables` | 20_표와_데이터.md | X13 |
| I054 | 셀 텍스트 원문 | 수확 | `export-tables` | 20_표와_데이터.md | X13 |
| I055 | 수확 후 편집은 별도 | 수확 | `export-tables` | 20_표와_데이터.md | X13 |
| I056 | 이 문구 바꿔 | 편집 | `edit replace-text` | 30_편집과_계획.md | X13 |
| I057 | 규제 를 다른 말로 | 편집 | `edit replace-text` | 30_편집과_계획.md | X13 |
| I058 | replace-text | 편집 | `edit replace-text` | 30_편집과_계획.md | X13 |
| I059 | 표 칸 값 고쳐 | 편집 | `edit set-cell` | 30_편집과_계획.md | X13 |
| I060 | set-cell 1행 1열 | 편집 | `edit set-cell` | 30_편집과_계획.md | X13 |
| I061 | 스타일 유지해서 칸 | 편집 | `edit set-cell` | 30_편집과_계획.md | X13 |
| I062 | 서식 채워 | 편집 | `edit fill-fields` | 30_편집과_계획.md | X13 |
| I063 | 누름틀에 회사명 | 편집 | `edit fill-fields` | 30_편집과_계획.md | X13 |
| I064 | fill-fields dry-run | 편집 | `edit fill-fields` | 30_편집과_계획.md | X13 |
| I065 | 도장 찍어 | 편집 | `edit insert-image` | 30_편집과_계획.md | X13 |
| I066 | 서명 이미지 | 편집 | `edit insert-image` | 30_편집과_계획.md | X13 |
| I067 | 개인정보 가려 | 편집 | `edit redact` | 30_편집과_계획.md | X13 |
| I068 | redact 먼저 탐지 | 편집 | `edit redact` | 30_편집과_계획.md | X13 |
| I069 | 작성자 지워 | 편집 | `edit sanitize` | 30_편집과_계획.md | X13 |
| I070 | sanitize 제출용 | 편집 | `edit sanitize` | 30_편집과_계획.md | X13 |
| I071 | 여러 단계 한 번에 | 편집 | `run` | 30_편집과_계획.md | X13 |
| I072 | run 계획서 | 편집 | `run` | 30_편집과_계획.md | X13 |
| I073 | 원자로 실행 | 편집 | `run` | 30_편집과_계획.md | X13 |
| I074 | --dry-run 만 | 편집 | `run` | 30_편집과_계획.md | X13 |
| I075 | 원본은 그대로 | 편집 | `edit replace-text` | 30_편집과_계획.md | X11 |
| I076 | -o 로 저장 | 편집 | `edit replace-text` | 30_편집과_계획.md | X11 |
| I077 | 덮어쓰지 마 | 편집 | `edit replace-text` | 30_편집과_계획.md | X11 |
| I078 | 조건부 치환 | 편집 | `edit replace-text` | 30_편집과_계획.md | X13 |
| I079 | occurrence 두 번째만 | 편집 | `edit replace-text` | 30_편집과_계획.md | X13 |
| I080 | keep-style | 편집 | `edit set-cell` | 30_편집과_계획.md | X13 |
| I081 | HWP 를 HWPX 로 | 변환 | `export-hwpx` | 40_변환과_렌더.md | X13 |
| I082 | 배포용을 편집 가능하게 | 변환 | `convert` | 40_변환과_렌더.md | X13 |
| I083 | convert --verify | 변환 | `convert` | 40_변환과_렌더.md | X13 |
| I084 | PDF 로 발행 | 변환 | `export-pdf` | 40_변환과_렌더.md | X13 |
| I085 | 마크다운으로 | 변환 | `export-markdown` | 40_변환과_렌더.md | X13 |
| I086 | 웹에서 읽게 | 변환 | `export-markdown` | 40_변환과_렌더.md | X13 |
| I087 | SVG 로 렌더 | 변환 | `export-svg` | 40_변환과_렌더.md | X13 |
| I088 | 쪽마다 그림 | 변환 | `export-svg` | 40_변환과_렌더.md | X13 |
| I089 | 썸네일 뽑기 | 변환 | `thumbnail` | 40_변환과_렌더.md | X13 |
| I090 | 전후 레이아웃 비교 | 변환 | `render-diff` | 40_변환과_렌더.md | X13 |
