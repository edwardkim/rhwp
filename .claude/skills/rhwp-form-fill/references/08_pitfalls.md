# 08 — 함정 (레시피 01·05 · form_filling_guide 실측)

| ID | 함정 | 증상 | 처방 |
| --- | --- | --- | --- |
| P01 | `--data` 를 CP949 로 저장 | `stream did not contain valid UTF-8` exit 1 | UTF-8 (`encoding='utf-8'`) |
| P02 | `--name-field` 컬럼이 매 행 notFound | 파일명 용도 컬럼 | 게이트에서 제외. 실패 아님 |
| P03 | `batch fill` 에 stdin 파일 목록 | 아무 일도 안 일어남 | `--form` + `--data` |
| P04 | `ambiguous` 무시 | 14개 중 1개만 채워 제출 | `이름[N]` 루프 |
| P05 | 헤더만 있는 CSV | exit 2, 데이터 행 없음 | 상류 명단 0건부터 |
| P06 | name-field 값 중복 | 나중 행이 덮어씀 (`_2` 접미) | 명령이 중복을 오류로 안 봄. 유일키는 호출자 |
| P07 | 머리말/각주 필드 부재 | 사람 눈에 칸이 있는데 목록 없음 | 사각지대. 재귀 확장 금지 |
| P08 | 페이지를 1부터 | `--page 1` = 둘째 쪽 | 0 기준 |
| P09 | insert-image 를 mm/px 로 | 도장이 점 또는 overflow | HWPUNIT, 1mm ≈ 283.46 |
| P10 | 로고 셀 기관명 채움 | 로고와 텍스트 겹침 | nested + export-tables 후 건너뜀 |
| P11 | 보고만 믿음 | filledCount 맞는데 값 안 보임 | 재독 / `--verify` |
| P12 | 새 명령·플래그 발명 | 알 수 없는 하위명령 exit 2 | cli_commands 표면만 |
| P13 | 성명과 이름을 동의어로 | notFound | fields 의 name 그대로 |
| P14 | 순번 1 기준 | 첫 칸이 안 바뀜 | `[0]` 이 첫 칸 |
| P15 | dry-run 없이 원본 옆 기본명 | `_filled` 가 입력 옆에 생김 | `-o output/` 습관 |
| P16 | batch 요약 줄만 읽음 | 어느 행이 실패인지 모름 | NDJSON 행별 게이트 |
| P17 | HWPX 에 `-o .hwp` | 형식 변환 + 경고 | 입력 확장자 유지 |
| P18 | sanitize 를 채움보다 먼저 | 채움 저장이 메타를 다시 남김 | 제출 직전 |
| P19 | removedCount 0 을 실패 | 멱등 | 첫 실행 여부만 확인 |
| P20 | overflow 를 실패로 중단 | 여러 줄이 정상인 칸 | 보고일 뿐. 판단은 호출자 |
| P21 | 한글 Windows 콘솔 CP949 | JSON 키가 깨짐 | 파일로 `--data @` + UTF-8 |
| P22 | 빈 문자열 `--data '{}'` | filledCount 0, 통과처럼 보임 | 의도한 키 개수와 대조 |
| P23 | 선택 표본 없음인데 규제서 가정 | 테스트/에이전트가 파일을 만듦 | 없으면 목차1×5 로 연습 |
| P24 | gym pack 으로 여정을 재작성 | 범위 밖 | 이 스킬은 실사용만 |

## 1순위: ambiguous

실무에서 가장 비싼 실수다. `filledCount: 1` 과 exit 0 만 보면 완료처럼
보인다. 반드시 `ambiguous` 배열을 본다.

## 2순위: name-field notFound

자동화 게이트를 처음 짜면 전원 실패로 오탐한다. 레시피 05 가 실측한
함정이다.

## 3순위: UTF-8

한국어 Windows 기본 저장이 CP949 인 편집기가 많다. `@파일` 은 항상
UTF-8 로 다시 쓴다.

## 4순위: stdin 축 혼동

`batch fields` 와 `batch fill` 을 같은 파이프에 넣지 않는다.
