# 15 봉투 필드 카탈로그

엔진이 읽거나 쓰는 키만 적는다. 없는 키를 요구하는 테스트는 실패해야 한다.

## engagement.json

| 키 | 필수 | 형 | 메모 |
| --- | --- | --- | --- |
| objective | yes | string | 데이터 |
| corpus | yes | string | 상대=engagement 옆 |
| questions | yes | array | 비면 exit 2 |
| questions[] | yes | string\|object | |
| questions[].id | no | string | 기본 Qn |
| questions[].text | no* | string | keywords 없으면 필요 |
| questions[].keywords | no* | string[] | text 없으면 필요 |
| deliverable | no | string | 기본 objective |
| searchLimit | no | int | 절단은 대장에 |

## corpus_map.json

| 키 | 메모 |
| --- | --- |
| schemaVersion | `1` |
| generatedBy | `tools/strategist/engagement.py` |
| corpus | 절대 경로로 고정됨 |
| documentCount | 파일 수 |
| mappedCount | status=ok |
| documents[].file | corpus 상대 posix |
| documents[].sizeBytes | |
| documents[].status | `ok` \| `failed` |
| documents[].info | ok 일 때 info 봉투 |
| documents[].infoExit | failed 일 때 |
| documents[].explain | 광고+성공 시 |
| documents[].explainExit | explain 실패 시 |

## evidence.json

| 키 | 메모 |
| --- | --- |
| entryCount | len(entries) |
| truncatedSearches[] | file, keyword, totalMatchCount, omittedCount |
| failures[] | phase, file, reason |
| entries[].id | EV-n |
| entries[].kind | search \| data |
| entries[].question | search 만 |
| entries[].keyword | search 만 |
| entries[].dataKind | data 만 |
| entries[].file | |
| entries[].quote | text 또는 raw |
| entries[].context | search |
| entries[].normalized | data |
| entries[].currency | 있으면 |
| entries[].unit | 있으면 |
| entries[].command | 재현 명령 |
| entries[].section|paragraph|page|charOffset|length|cell|textbox | 있는 것만 |

## validate 판정

| 키 | 메모 |
| --- | --- |
| mode | `validate` |
| claimCount | |
| ledgerEntryCount | |
| violationCount | |
| violations[].claim | CLAIM-n 또는 null |
| violations[].kind | placeholder \| unknown-evidence \| unlinked |
| violations[].detail | |
| verdict | pass \| fail |
| swsAudit | 선택 |

## search 매치 (입력 봉투)

| 키 | 대장 |
| --- | --- |
| text | quote |
| context | context |
| section, paragraph, page, charOffset, length, cell, textbox | copy_coords |
| truncated, totalMatchCount, omittedCount | 절단 배열 |

## extract-data item (입력 봉투)

| 키 | 대장 |
| --- | --- |
| kind | dataKind |
| raw | quote |
| normalized | normalized |
| currency, unit | 있으면 |
| 좌표 키 | copy_coords |

다음: [16_journeys.md](16_journeys.md).
