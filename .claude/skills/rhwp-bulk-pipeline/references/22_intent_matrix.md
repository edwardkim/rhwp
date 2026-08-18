# 22 — 발화 → 명령

발화 140개. 에이전트가 사용자 문장을 축으로 옮길 때 본다.

| ID | 발화 | 명령 | 정지 |
| --- | --- | --- | --- |
| I001 | 폴더 문서들 몇 쪽이야 | `batch info --json` | B04 |
| I002 | 형식부터 훑어줘 | `batch info --json` | B04 |
| I003 | 본문 전부 뽑아 | `batch export-text --json` | B05 |
| I004 | 텍스트로 한꺼번에 | `batch export-text --json` | B05 |
| I005 | 개요만 일괄 | `batch export-structure --json --mode outline` | B17 |
| I006 | 조문 구조 뽑아 | `batch export-structure --json --mode clause` | B17 |
| I007 | 표 전부 CSV 말고 JSON 으로 | `batch export-tables --json` | B07 |
| I008 | 서식에 누름틀 있는 파일만 | `batch fields --json` | B08 |
| I009 | 위임전결 어디 있어 | `batch search --query 위임전결 --json` | B17 |
| I010 | 아카이브 전역 검색 | `batch search --query <q> --json` | B09 |
| I011 | 날짜 금액 수확 | `batch extract-data --json` | B10 |
| I012 | 금액만 | `batch extract-data --json --kind amount` | B17 |
| I013 | 너무 많으니 문서당 3개만 | `batch extract-data --json --limit 3` | B10 |
| I014 | HWPX 를 HWP5 로 일괄 | `batch convert --out-dir <dir> --json` | B11 |
| I015 | 변환하고 검증까지 | `batch convert --out-dir <dir> --verify --verify-pages --json` | B16 |
| I016 | 신청서에 명단 채워 | `batch fill --form --data --out-dir --json` | B12 |
| I017 | 메일머지 | `batch fill --form --data --out-dir --json` | B12 |
| I018 | 미리 채움만 확인 | `batch fill --dry-run --form --data --out-dir --json` | B17 |
| I019 | 실패만 다시 | `jq select(.error) \| batch <same axis>` | B05 |
| I020 | 숫자가 맞아? | `gate input N = success + failure` | B13 |
| I021 | 비밀번호 넣어서 배치 | `거부. 단건 --password` | B03 |
| I022 | batch --password | `exit 2` | B03 |
| I023 | 검색어 없이 검색 | `exit 2` | B09 |
| I024 | 같은 이름 두 파일 변환 | `이름 예약 충돌 exit 2` | B11 |
| I025 | 폴더 전체를 텍스트로 | `batch export-text --json` | B05 |
| I026 | 한꺼번에 변환 | `batch convert --out-dir --json` | B11 |
| I027 | 여러 hwp 대량 처리 | `batch info 후 축 선택` | B04 |
| I028 | 코퍼스 추출 | `batch export-text --json` | B05 |
| I029 | 서식 하나에 여러 명 | `batch fill` | B12 |
| I030 | rhwp batch | `축을 물어보고 진행` | B17 |
| I031 | threads 높여 | `batch <axis> --threads N --json` | B17 |
| I032 | 순서가 뒤섞이면 안 돼 | `--threads 해도 입력 순서 보존` | B17 |
| I033 | stderr 요약 어디 | `사람용. 파이프에 태우지 말 것` | B14 |
| I034 | NDJSON 이 아니라 JSON 배열로 | `jq -s 는 게이트에서만` | B14 |
| I035 | 없는 파일 섞여 있어 | `실패 봉투. exit 1 정상` | B05 |
| I036 | 표 병합 유지해 | `batch export-tables (markdown 금지)` | B07 |
| I037 | 누름틀 없는 문서 채워 | `이 스킬 아님. form-fill/table-exchange` | B08 |
| I038 | 한 문서만 파악 | `rhwp-doc-triage` | B17 |
| I039 | 배포 전 점검 | `rhwp-security-sweep` | B17 |
| I040 | MCP 로 배치 변환 | `convert 는 CLI 전용` | B11 |
| I041 | Windows 에서 목록 | `Get-ChildItem ... \| rhwp batch` | B17 |
| I042 | CP949 명단 | `UTF-8 로 재저장` | B12 |
| I043 | -결과 폴더에 변환 | `./-결과` | B18 |
| I044 | verify 실패면 파일 없나 | `산출은 남음. exit 3` | B15 |
| I045 | 페이지 수 불일치 | `exit 4` | B16 |
| I046 | 성공만 세면 되지 | `실패를 지우면 N 게이트가 깨짐` | B13 |
| I047 | head 로 미리보기 | `게이트 전에 쓰지 말 것` | B13 |
| I048 | grep error 한 줄 | `요약을 지울 수 있음. jq 사용` | B14 |
| I049 | 같은 문서 두 번 extract | `limit 은 문서마다 독립` | B10 |
| I050 | 필드 조사 후 채움 | `fields 배치 → 단건/fill 스킬` | B08 |
| I051 | 조문 모드 기본? | `기본 auto` | B06 |
| I052 | search limit | `파일당 1000. 단건 --limit 과 같은 취지` | B17 |
| I053 | 대소문자 무시 검색 | `구분한다. 다른 쿼리를 두 번` | B17 |
| I054 | info 스키마가 단건과 같나 | `같다. 같은 소비 코드` | B04 |
| I055 | fill 레코드에 row | `0 기준 행 번호` | B17 |
| I056 | name-field 생략 | `1 기준 순번 최소 4자리` | B17 |
| I057 | 이름 겹치면 | `_2 접미` | B17 |
| I058 | 파일명 금지 문자 | `_ 치환` | B17 |
| I059 | dry-run 인데 out-dir 왜 | `실행 줄에서 --dry-run 만 빼면 되도록` | B17 |
| I060 | 빈 헤더 CSV | `exit 2` | B12 |
| I061 | 데이터 파일 stdin | `안 됨. --data 파일` | B12 |
| I062 | 목록을 인자로 | `하지 말 것. stdin` | B01 |
| I063 | 암호화 산출 | `batch 미지원` | B03 |
| I064 | output-password | `exit 2` | B03 |
| I065 | password-stdin 배치 | `exit 2` | B03 |
| I066 | 성공 4 실패 1 코드는 | `1` | B14 |
| I067 | 전부 성공 코드는 | `0` | B17 |
| I068 | 사용법 오류 코드는 | `2` | B03 |
| I069 | 페이지 검증 코드는 | `4` | B16 |
| I070 | IR 검증 코드는 | `3` | B15 |
| I071 | 파이프 중간에 jq select | `행 수 변함. 게이트는 원본 목록 기준` | B13 |
| I072 | 재시도 결과를 원본에 덮어쓰기 | `실패 행만 치환. 성공 유지` | B05 |
| I073 | 원본 HWP 를 convert 가 덮나 | `out-dir 로 분리. 원본 불변` | B11 |
| I074 | fill 이 서식을 덮나 | `out-dir 산출. 서식 불변` | B12 |
| I075 | 271건 스윕 시간 | `info 3.0s (가이드 실측)` | B04 |
| I076 | 271건 본문 시간 | `export-text 67.4s (가이드 실측)` | B05 |
| I077 | 10쪽 이상만 | `info 후 jq pageCount>=10` | B04 |
| I078 | RAG 청킹 | `배치 text 후 필요 문서만 단건 pages[]` | B05 |
| I079 | CI 무인 | `exit 1 + jq 실패 행 보고` | B14 |
| I080 | 손상 파일 | `실패 봉투. 스트림 계속` | B05 |
| I081 | panic 격리 | `exitClass runtime 레코드` | B05 |
| I082 | 스키마 필드 추가 | `허용. 삭제·변경은 cli_json_contract` | B17 |
| I083 | 단건 실패 stdout | `0바이트. 배치와 다름` | B14 |
| I084 | 배치 실패 stdout | `실패 레코드 1줄` | B14 |
| I085 | capabilities batch | `단일 출처` | B17 |
| I086 | extract-data 가 §batch 목록에 없다 | `capabilities + 레시피 9가 근거` | B10 |
| I087 | hwp_batch 도구 | `읽기 축. convert 쓰기 제외` | B11 |
| I088 | gym 과제 만들까 | `금지` | B17 |
| I089 | 새 batch merge 명령 | `발명 금지. fill 사용` | B12 |
| I090 | batch export-markdown | `없음. 단건 export-markdown` | B17 |
| I091 | batch thumbnail | `없음` | B17 |
| I092 | batch redact | `없음. security-sweep` | B17 |
| I093 | 폴더를 인자로 | `stdin 목록으로 바꿔` | B01 |
| I094 | *.hwp 글롭을 batch 에 | `쉘이 펼치면 인자 한계. 목록 파일` | B01 |
| I095 | 병렬이라 순서가 달라도 | `아님. 입력 순서 보존` | B17 |
| I096 | CPU 기본 스레드 | `코어 수` | B17 |
| I097 | threads 0 | `사용법 확인. 추측 금지` | B06 |
| I098 | limit 을 배치 전체 상한으로 | `구현 오류. 문서마다` | B10 |
| I099 | counts 가 limit 과 같다 | `아님. counts 는 절단 전` | B10 |
| I100 | truncated 없이 자름 | `계약 위반. truncated:true 필수` | B10 |
| I101 | 금액 0 키 생략? | `kind=all 실측은 키를 넣기도. 단건 명문은 미요청 키 생략` | B10 |
| I102 | 정규화 실패 | `normalized null. raw 만 신뢰` | B10 |
| I103 | 표 자동번호 | `export-tables 한계. 빈 자리` | B07 |
| I104 | 1x1 래퍼 표 | `표로 잡힘. 소비자가 필터` | B07 |
| I105 | 머리말 안 표 | `export-tables 재귀 수집. info 는 놓칠 수 있음` | B07 |
| I106 | 검색 매치 1000 초과 | `잘림. 단건 search --limit 과 같은 취지` | B17 |
| I107 | convert 출력 이름 규칙 | `<out-dir>/<입력이름>.hwp` | B11 |
| I108 | HWP5 입력을 convert | `다시 HWP5 로 씀. 이름은 .hwp` | B11 |
| I109 | 한 건도 안 써진 이유 | `이름 예약 실패. 로그 stderr` | B11 |
| I110 | 절반만 변환됨 | `예약 규약이면 일어나면 안 됨` | B11 |
| I111 | Linux 와 Windows 이름 | `대소문자 충돌을 동일하게 거부` | B11 |
| I112 | fill 행 실패도 남김 | `다른 축과 같은 실패 스키마 + row` | B12 |
| I113 | 서식 못 열면 | `시작 전 한 번만 판정` | B12 |
| I114 | 명단 80행 | `산출 80. 레코드 80` | B12 |
| I115 | 성명으로 파일명 | `--name-field 성명` | B17 |
| I116 | 제출 정리 | `이 스킬 아님. form-fill sanitize` | B17 |
| I117 | 원본 in-place | `하지 말 것` | B17 |
| I118 | 목록 인코딩 | `UTF-8. PowerShell Out-File -Encoding utf8` | B01 |
| I119 | BOM 목록 | `첫 경로가 깨질 수 있음` | B01 |
| I120 | CRLF 목록 | `허용. 한 줄 = 한 경로` | B01 |
| I121 | 공백 있는 경로 | `따옴표 없이 한 줄 전체` | B01 |
| I122 | 네트워크 경로 | `os error 가능. 실패 봉투` | B05 |
| I123 | 잠긴 파일 | `런타임 실패 봉투` | B05 |
| I124 | 권한 거부 | `런타임 실패 봉투` | B05 |
| I125 | 디스크 가득 | `쓰기 축 런타임. 읽기 축은 성공 가능` | B11 |
| I126 | 같은 목록으로 여러 축 | `info → 본작업. 목록 재사용` | B04 |
| I127 | 축을 한 프로세스에 섞기 | `안 됨. 호출을 나눔` | B17 |
| I128 | NDJSON 이어 붙이기 | `재시도 결과를 실패 자리에 치환` | B05 |
| I129 | 성공 파일을 다시 돌리기 | `낭비. 실패만` | B05 |
| I130 | 게이트를 wc -l 결과만 | `NDJSON 줄 수와 목록 줄 수를 같이` | B13 |
| I131 | jq -s 메모리 | `대량이면 스트리밍 카운트` | B13 |
| I132 | Python 으로 게이트 | `error 키 유무로 분기` | B13 |
| I133 | PowerShell ConvertFrom-Json | `한 줄씩` | B14 |
| I134 | Select-String error | `본문에 error 단어가 있는 성공 행을 오탐` | B14 |
| I135 | 실패 메시지 한글 | `os error 2 실측 문구 유지` | B05 |
| I136 | untrustedContent | `실패 봉투에도 출처 표지 필드가 실림` | B14 |
| I137 | schemaVersion 1.0 | `계약` | B17 |
| I138 | 필드 추가만 허용 | `삭제 금지` | B17 |
| I139 | 이슈 번호 | `5311` | B17 |
| I140 | gym 에서 돌려 | `금지. 실 에이전트 경로` | B17 |

기계 원본: `fixtures/intent_matrix.json`.
## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `22_intent_matrix.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
