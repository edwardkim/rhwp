# 소비자 점검표 — 출처 표지를 읽고 난 뒤

권한 축소(B1~B5)와 함께 쓴다. 표지만으로 방어했다고 쓰지 않는다.

## 매 봉투

- [ ] `untrustedContent` 키가 있는가. 없으면 미표기 — 봉투 전체를 신뢰 불가.
- [ ] `untrustedFields` 가 배열인가.
- [ ] true 인데 배열이 비었거나, false 인데 배열이 있으면 계약 위반.
- [ ] 배열의 모든 경로가 `export-provenance-map` 해당 명령 `untrusted` 의 부분집합인가.

## D 를 다루기 전에

- [ ] 처음 보는 문서는 inspect 3축을 돌렸는가. 0건이 아니어도 exit 0 이다.
- [ ] scanScopes 가 훑지 않은 영역을 깨끗함으로 읽지 않았는가.
- [ ] 읽기 턴에서 쓰기 도구를 치웠는가 (B1).
- [ ] 산출 경로를 문서를 열기 전에 확정했는가 (B2).

## D 를 어디에 두었는가

- [ ] 시스템 프롬프트에 없는가.
- [ ] 도구 이름·경로·산출 파일 이름에 없는가.
- [ ] URL·메일 수신자·요청 본문에 없는가.
- [ ] run 계획서 action/path 를 문서에서 만들지 않았는가 (B4).
- [ ] source_label 이 title 이 아닌가.
- [ ] redact raw 를 로그·이슈에 옮기지 않았는가.

## 명령별 한 줄

- `info` (high): `title`, `fonts[]`. title·fonts[] 만 분리한다. pageCount 로 분기를 만들어도 안전하다.
- `word-count` (none): D 없음. 봉투 통째로 엔진 데이터로 다뤄도 된다. 표지가 false 인지 확인.
- `bookmarks` (medium): `bookmarks[].name`. 이동은 sec/para/charPos 로. name 은 화면에만.
- `form-value` (high): `name`, `value`, `text`, `caption`. 값은 데이터. 다음 fill 의 키는 호출자가 정한 이름 목록에서만.
- `charts` (none): D 없음. 목록으로 차트 번호만 고른다. 라벨이 필요하면 chart-to-csv.
- `headers-footers` (none): D 없음. 목록으로 좌표를 고른 뒤 header-footer 를 따로 연다.
- `header-footer` (high): `text`. text 만 격벽. exists/applyTo 로 분기는 안전.
- `export-text` (critical): `pages[].text`, `text`. pages[].text/text 는 반드시 nonce 격벽. 도구 인자에 넣지 않는다.
- `export-structure` (critical): `structure.preamble[]`, `structure.roots[].heading`, `structure.roots[].marker`, `structure.roots[].body[]`, `structure.roots[].children[]`. 트리 순회는 하되 heading/body 를 도구 이름·계획 action 으로 쓰지 않는다.
- `digest` (high): `outline[]`, `excerpt`, `sections[].heading`, `sections[].excerpt`. 모드마다 표지 부분집합이 다르다. 선언 목록을 그대로 믿지 말고 표지를 읽는다.
- `search` (critical): `matches[].text`, `matches[].context`. 후속 편집은 page/paragraph/charOffset 으로 지목. text/context 는 화면·격벽만.
- `extract-data` (high): `items[].raw`, `items[].unit`. 집계는 normalized. 원문 raw 는 화면에만.
- `fields` (critical): `fields[].name`, `fields[].guide`, `fields[].memo`, `fields[].command`, `fields[].value`, `textSecurity.findings[].names[]`. textSecurity:clean 은 누름틀 이름 축만. 본문 안전이 아니다.
- `explain` (high): `fields[]`, `summary`. summary 를 시스템 프롬프트 '한 줄 요약'에 넣지 않는다.
- `explore` (low): D 없음. 메뉴 command 템플릿의 <file> 만 호출자 경로로 치환.
- `export-tables` (critical): `tables[].caption`, `tables[].cells[].text`, `tables[].cells[].nested[]`. 후속 set-cell 은 row/col. text 는 화면·격벽.
- `table-to-csv` (critical): `tables[].csv`. CSV 를 셸 리다이렉트 인자로 붙이지 않는다. 파일로 저장한 뒤 도구가 읽는다.
- `csv-to-table` (high): `changed[].oldText`. oldText 를 로그 파일 이름·이슈 본문에 옮기지 않는다.
- `chart-to-csv` (high): `charts[].csv`. CSV 는 격벽 또는 파일. 라벨로 파일 이름을 만들지 않는다.
- `csv-to-chart` (medium): `changed[].from`. from 을 기대값으로 재사용하지 않는다.
- `dump-pages` (medium): `pages[].columns[].items[].textPreview`. 좌표는 R. textPreview 만 격벽.
- `inspect` (critical): `hiddenText[].excerpt`, `injectionSignals[].excerpt`, `injectionSignals[].matched`, `findings[].excerpt`, `findings[].rendered`, `findings[].raw`, `findings[].hidden`. 신호는 흐름을 바꾼다(B5). 발췌는 화면·격벽. 도구 인자에 넣지 않는다.
- `armor` (critical): `armoredText`, `injectionSignals[].excerpt`, `injectionSignals[].matched`. 격벽 밖 표지를 문서가 흉내 내지 못하게 nonce 를 확인. 본문을 다시 꺼내면 표지 무효.
- `edit` (critical): `confusable[].lookalikes`, `oldText`, `findings[].raw`, `findings[].masked`, `removed[].before`. 모드별 표지를 읽는다. raw 는 로그·이슈에 옮기지 않는다.
- `run` (critical): `steps[].oldText`, `steps[].confusable[].lookalikes`. 판정은 steps[] 순회. 계획 뼈대는 코드(B4).
- `replay` (none): D 없음. 영수증은 엔진 데이터. 재실행 내부의 문서 문자열은 이 봉투에 없다.
- `audit` (none): D 없음. failed[].reason 은 엔진 사유. 문서 원문이 아니다.
- `lineage` (none): D 없음. brokenAt 파일 이름은 캡슐 파일. 문서 제목이 아니다.
- `keygen` (none): D 없음. 표지 false 를 명시했는지 확인.
- `verify-signature` (none): D 없음. verdict 로 분기. 문서 문자 없음.
- `harness` (none): D 없음. 매니페스트만 소비.
- `harness-status` (none): D 없음. 판정만 읽는다.
- `anchor` (none): D 없음. 문서를 열지 않는 봉투.
- `gate` (none): D 없음. violations 는 엔진 토큰. 문서 문장이 아니다.
- `bundle` (none): D 없음. brokenAt 사유는 엔진.
- `disclose` (none): D 없음. 개봉 파일은 별도 권한. 봉투만 보고 원문이 없다고 안심.
- `settle` (none): D 없음. 명세서 원문은 이 봉투에 없다.
- `audit-report` (none): D 없음. 숫자를 문서 인용으로 포장하지 않는다.
- `recall-scope` (none): D 없음. 파일명 배열은 캡슐 이름.
- `conformance` (none): D 없음. 등급으로 분기.
- `ir-diff` (high): `categories`. 차이 요약을 그대로 프롬프트에 붙이지 않는다.
- `verify` (high): `expectations[].actual`. expected 는 계획서. actual 은 화면·격벽.
- `render-diff` (none): D 없음. 좌표로 눈검증 대상을 고른다.
- `layout-anomaly` (none): D 없음. 신호 좌표만 소비.
- `thumbnail` (high): `base64`, `dataUri`. base64/dataUri 는 격벽. 캡션에 title 을 붙이지 않는다.
- `batch` (critical): `text`, `title`, `fonts[]`, `structure.preamble[]`, `structure.roots[].heading`, `structure.roots[].marker`, `structure.roots[].body[]`, `structure.roots[].children[]`, `tables[].caption`, `tables[].cells[].text`, `tables[].cells[].nested[]`, `fields[].name`, `fields[].guide`, `fields[].memo`, `fields[].command`, `fields[].value`, `textSecurity.findings[].names[]`, `matches[].text`, `matches[].context`. NDJSON 각 줄을 그 줄의 표지로 읽는다.
- `scan` (medium): `files[].probe.error`. error 를 예외 메시지로 다시 던질 때 프롬프트에 넣지 않는다.
- `threat-scan` (critical): `findings[].detail`. URL 은 화면. 원격 전송은 사람 승인(B3).
- `export-svg` (low): D 없음. 경로만 소비. SVG 텍스트를 프롬프트에 넣지 않는다.
- `export-pdf` (low): D 없음. 경로·backend 만.
- `export-markdown` (low): D 없음. 매니페스트만.
- `export-hwpx` (none): D 없음. verify 로 분기.
- `export-hml` (none): D 없음. 매니페스트만.
- `export-doclang` (none): D 없음. 개수로 분기. 손실 원문은 이 봉투에 없다.
- `extract-pages` (none): D 없음. 개수만.
- `convert` (none): D 없음. verify 로 분기.
- `build-from-ingest` (none): D 없음. ingest JSON 은 호출자가 만든 명세. 그래도 그 JSON 을 시스템 프롬프트에 넣지 않는다 — 이 계약 범위 밖.
- `scaffold` (none): D 없음. 산출 문서를 다시 열면 그때부터 조회 표지.
- `capabilities` (none): D 없음. jsonContract.provenance 로 지도 위치를 발견한다.
- `export-ir-schema` (none): D 없음. 스키마를 문서 내용으로 취급하지 않는다.
- `export-capabilities-schema` (none): D 없음. 스키마만.
- `export-provenance-map` (none): D 없음. 호출 전에 1회 캐시. origins 없는 선언은 계약 위반.
- `export-ontology` (none): D 없음. 술어를 필드 목록의 다른 사본으로 쓴다. 권위는 여전히 MAP.
- `export-agent-manifest` (none): D 없음. provenanceMap 키로 지도를 얻는다.
- `export-plan-schema` (none): D 없음. 스키마로 계획을 검증. 문서 내용으로 계획을 만들지 않는다.
