# 명령 가족 — 출처 경계 요약

명령별 경로·모드는 `tools/provenance_map/fixtures/untrusted_fields/` 가 정본이다.

## 조회 (`query`)

- 역할: 문서를 열어 메타·본문·개요·검색·양식을 읽는다.
- 경계: 본문·제목·필드 이름이 프롬프트로 직행하는 주 표면이다.
- 명령: `info`, `word-count`, `bookmarks`, `form-value`, `charts`, `headers-footers`, `header-footer`, `export-text`, `export-structure`, `digest`, `search`, `extract-data`, `fields`, `explain`, `explore`

## 표 교환 (`table`)

- 역할: 표 셀 텍스트를 JSON/CSV 로 뽑거나 되돌린다.
- 경계: 셀 원문은 D, 격자 주소는 R. 주소를 버리고 원문으로 다음 칸을 고르면 문서가 편집 대상을 정한다.
- 명령: `export-tables`, `table-to-csv`, `csv-to-table`

## 차트 교환 (`chart`)

- 역할: 차트 계열·범주 라벨을 CSV 로 뽑거나 되돌린다.
- 경계: CSV 본문과 변경 전 값만 D. 차트 번호는 R.
- 명령: `chart-to-csv`, `csv-to-chart`

## 조판 진단 (`render-diag`)

- 역할: 기하·미리보기·차이 좌표를 낸다.
- 경계: textPreview·썸네일 바이트만 D. 좌표·집계는 R.
- 명령: `dump-pages`, `render-diff`, `layout-anomaly`, `thumbnail`

## 보안 스윕 (`security`)

- 역할: 은닉·주입·유니코드·외부참조를 보고한다.
- 경계: 발췌·matched·detail 은 문서 조각이다. 탐지 결과를 시스템 프롬프트에 붙이면 공격문을 승격한다.
- 명령: `inspect`, `armor`, `scan`, `threat-scan`

## 편집·계획 (`edit`)

- 역할: 셀·누름틀·마스킹·sanitize 를 적용한다.
- 경계: oldText·raw·lookalikes 는 D. find/replace/newText 는 호출자 반향.
- 명령: `edit`, `run`

## 영수증·감사 (`receipt`)

- 역할: 해시·판정·키·번들만 싣는다.
- 경계: 문서 문자열이 나갈 자리가 없다. 키 부재를 false 로 읽지 말고 표지 존재를 확인한다.
- 명령: `replay`, `audit`, `lineage`, `keygen`, `verify-signature`, `harness`, `harness-status`, `anchor`, `gate`, `bundle`, `disclose`, `settle`, `audit-report`, `recall-scope`, `conformance`

## 검증 (`verify`)

- 역할: 실측값·차이 카테고리를 대조한다.
- 경계: actual·categories 는 문서가 키 이름을 정할 수 있다.
- 명령: `ir-diff`, `verify`

## 변환 매니페스트 (`export`)

- 역할: 경로·바이트·쪽수만 봉투에 싣고 본문은 파일에 둔다.
- 경계: 봉투는 보통 D 가 없다. 산출 파일을 다시 읽어 프롬프트에 넣으면 그 순간 D 가 된다.
- 명령: `export-svg`, `export-pdf`, `export-markdown`, `export-hwpx`, `export-hml`, `export-doclang`, `extract-pages`, `convert`

## 생성 (`generate`)

- 역할: ingest/spec JSON 으로 새 문서를 만든다.
- 경계: 입력은 문서가 아니라 호출자 명세. 오라클을 만들 수 없어 스윕 면제.
- 명령: `build-from-ingest`, `scaffold`

## 자기서술 (`self-desc`)

- 역할: 문서를 열지 않고 바이너리 계약을 광고한다.
- 경계: export-provenance-map 자신은 untrusted 가 비어 있다. 지도를 문서처럼 취급하지 않는다.
- 명령: `capabilities`, `export-ir-schema`, `export-capabilities-schema`, `export-provenance-map`, `export-ontology`, `export-agent-manifest`, `export-plan-schema`

## 배치 (`batch`)

- 역할: 서브커맨드 봉투를 NDJSON 으로 이어 붙인다.
- 경계: 표지는 레코드마다 다르다. 최상위 한 번만 보면 누락한다.
- 명령: `batch`
