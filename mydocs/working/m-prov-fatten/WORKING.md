# M-prov: 출처 표지 지도·주입 경계 고도화

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5480
브랜치: `feat/m-prov-fatten` (`upstream/devel` 기준 격리 worktree)
범위: `tools/provenance_map/` · `mydocs/working/m-prov-fatten/`
비범위: inspect/replay/hwp5-inventory/proptest/page-count/fidelity 구현 · gym · 새 CLI

## 무엇을

`rhwp export-provenance-map --json` 의 단일 출처는
`crates/rhwp-contracts/src/provenance.rs` 의 `MAP` 이다.
이 작업은 그 표를 복제하지 않고, 표가 말하지 않는 소비자 경계를 고정한다.

- 명령별 `untrustedFields` 카탈로그 (기원·모드 존재·금지 자리)
- 금지 자리 목록 (시스템 프롬프트·경로·URL·run 계획·권한 판단 등)
- 모드별 봉투 표본 (`untrustedContent`/`untrustedFields` 부분집합)
- 작업 문서 (가족별 경계, 모드 존재표, 소비자 점검표)

## 왜

표지는 판정이지 방어가 아니다. 지도에 경로가 있어도 소비자가
그 값을 시스템 프롬프트나 `-o` 이름에 넣으면 문서가 에이전트를 조종한다.
금지 자리와 모드별 표본이 없으면 6개월 뒤 표지만 남은 알리바이가 된다.

## 실측 규모

- MAP 항목(중복 포함): 66
- 고유 명령: 65
- 문서 파생 경로: 81
- 필드 카탈로그: 65
- 봉투 표본: 96
- 금지 자리: 25
- 필드×자리 금지 쌍: 923

## 하지 않은 것

- 새 CLI / 새 표지 키 발명 없음
- `tests/provenance_contract.rs` 미수정 (기존 드리프트 가드 유지)
- inspect·replay·hwp5-inventory·proptest·page-count·fidelity 구현 미수정
- gym 없음

## 검증

```bash
python tools/provenance_map/fatten_provenance_map.py
python tools/provenance_map/test_fatten_provenance_map.py
cargo fmt --all -- --check
```

## 명령 가족

### 조회 (`query`)

- 역할: 문서를 열어 메타·본문·개요·검색·양식을 읽는다.
- 경계: 본문·제목·필드 이름이 프롬프트로 직행하는 주 표면이다.
- 명령 (15): `info`, `word-count`, `bookmarks`, `form-value`, `charts`, `headers-footers`, `header-footer`, `export-text`, `export-structure`, `digest`, `search`, `extract-data`, `fields`, `explain`, `explore`

### 표 교환 (`table`)

- 역할: 표 셀 텍스트를 JSON/CSV 로 뽑거나 되돌린다.
- 경계: 셀 원문은 D, 격자 주소는 R. 주소를 버리고 원문으로 다음 칸을 고르면 문서가 편집 대상을 정한다.
- 명령 (3): `export-tables`, `table-to-csv`, `csv-to-table`

### 차트 교환 (`chart`)

- 역할: 차트 계열·범주 라벨을 CSV 로 뽑거나 되돌린다.
- 경계: CSV 본문과 변경 전 값만 D. 차트 번호는 R.
- 명령 (2): `chart-to-csv`, `csv-to-chart`

### 조판 진단 (`render-diag`)

- 역할: 기하·미리보기·차이 좌표를 낸다.
- 경계: textPreview·썸네일 바이트만 D. 좌표·집계는 R.
- 명령 (4): `dump-pages`, `render-diff`, `layout-anomaly`, `thumbnail`

### 보안 스윕 (`security`)

- 역할: 은닉·주입·유니코드·외부참조를 보고한다.
- 경계: 발췌·matched·detail 은 문서 조각이다. 탐지 결과를 시스템 프롬프트에 붙이면 공격문을 승격한다.
- 명령 (4): `inspect`, `armor`, `scan`, `threat-scan`

### 편집·계획 (`edit`)

- 역할: 셀·누름틀·마스킹·sanitize 를 적용한다.
- 경계: oldText·raw·lookalikes 는 D. find/replace/newText 는 호출자 반향.
- 명령 (2): `edit`, `run`

### 영수증·감사 (`receipt`)

- 역할: 해시·판정·키·번들만 싣는다.
- 경계: 문서 문자열이 나갈 자리가 없다. 키 부재를 false 로 읽지 말고 표지 존재를 확인한다.
- 명령 (15): `replay`, `audit`, `lineage`, `keygen`, `verify-signature`, `harness`, `harness-status`, `anchor`, `gate`, `bundle`, `disclose`, `settle`, `audit-report`, `recall-scope`, `conformance`

### 검증 (`verify`)

- 역할: 실측값·차이 카테고리를 대조한다.
- 경계: actual·categories 는 문서가 키 이름을 정할 수 있다.
- 명령 (2): `ir-diff`, `verify`

### 변환 매니페스트 (`export`)

- 역할: 경로·바이트·쪽수만 봉투에 싣고 본문은 파일에 둔다.
- 경계: 봉투는 보통 D 가 없다. 산출 파일을 다시 읽어 프롬프트에 넣으면 그 순간 D 가 된다.
- 명령 (8): `export-svg`, `export-pdf`, `export-markdown`, `export-hwpx`, `export-hml`, `export-doclang`, `extract-pages`, `convert`

### 생성 (`generate`)

- 역할: ingest/spec JSON 으로 새 문서를 만든다.
- 경계: 입력은 문서가 아니라 호출자 명세. 오라클을 만들 수 없어 스윕 면제.
- 명령 (2): `build-from-ingest`, `scaffold`

### 자기서술 (`self-desc`)

- 역할: 문서를 열지 않고 바이너리 계약을 광고한다.
- 경계: export-provenance-map 자신은 untrusted 가 비어 있다. 지도를 문서처럼 취급하지 않는다.
- 명령 (7): `capabilities`, `export-ir-schema`, `export-capabilities-schema`, `export-provenance-map`, `export-ontology`, `export-agent-manifest`, `export-plan-schema`

### 배치 (`batch`)

- 역할: 서브커맨드 봉투를 NDJSON 으로 이어 붙인다.
- 경계: 표지는 레코드마다 다르다. 최상위 한 번만 보면 누락한다.
- 명령 (1): `batch`

## 표지 읽는 법

1. 키 부재는 미표기다. false 로 승격하지 않는다.
2. `untrustedContent` 와 `untrustedFields` 가 서로 다른 말을 하면 계약 위반.
3. `untrustedFields` 는 지도 목록의 부분집합 — 모드마다 실제로 실린 경로만.
4. D 는 화면 또는 nonce 격벽만. 그 외 자리는 `fixtures/forbidden_slots/`.
5. 탐지 신호는 흐름을 바꿔야 신호다 (정지, 재시도 아님).
