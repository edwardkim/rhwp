# 16 여정 목록

기계 원본: `fixtures/journeys.json`.

| id | 이름 | 코퍼스 | 예제 | 기대 |
| --- | --- | --- | --- | --- |
| J01 | 수주 근거 풀코스 | gov_rfp | E01 | 엔진→CLAIM→validate pass |
| J02 | 분기 전략 | quarterly | E02 | 금액+인력 EV |
| J03 | 실패 문서 보존 | mixed_failed | E03 | failed 행 유지 |
| J04 | page 없음 | gov_rfp | E04 | 키 생략 |
| J05 | 게이트 통과 | gov_rfp | E05 | EV 동거 |
| J06 | unknown EV | gov_rfp | E06 | exit 3 |
| J07 | placeholder | gov_rfp | E07 | exit 3 |
| J08 | 0건 절 | quarterly | E08 | CLAIM 없음 |
| J09 | 전망 거부 | — | E09 | ST-FORECAST |
| J10 | FDE 오인 | — | E10 | 인계 |
| J11 | Chief 인계 | — | E11 | objective 수신 |
| J12 | searchLimit | gov_rfp | E12 | truncated |
| J13 | extract-data | gov_rfp | E13 | amount EV |
| J14 | unlinked | gov_rfp | E14 | exit 3 |
| J15 | 부분 가독 | mixed_failed | E15 | L2 정직 |
| J16 | SWS 미달 | gov_rfp | E16 | exit 불변 |
| J17 | 깨진 json | — | E17 | exit 2 |
| J18 | 재독 | gov_rfp | E18 | command 재실행 |
| J19 | command | gov_rfp | E19 | 제3자 재현 |
| J20 | scaffold 없음 | gov_rfp | E20 | spec 납품 |
| J21 | 표 칸 | gov_rfp | E21 | cell |
| J22 | 글상자 | gov_rfp | E22 | textbox |
| J23 | omittedCount | gov_rfp | E23 | 절단 수치 |
| J24 | 지도 수치 | mixed_failed | E24 | count 불변 |
| J25 | 빈 questions | — | E17 | exit 2 |
| J26 | 빈 corpus | — | E17 | exit 2 |
| J27 | capabilities 실패 | — | — | exit 1 |
| J28 | search 전패 | — | — | exit 1 |
| J29 | explain 실패 | mixed_failed | E03 | explainExit |
| J30 | 다중 키워드 | gov_rfp | E01 | 질문당 키워드 N |
| J31 | 상대 corpus | gov_rfp | E01 | resolve |
| J32 | 대소문자 확장자 | gov_rfp | — | .HWP |
| J33 | validate evidence 경로 | gov_rfp | E05 | --evidence |
| J34 | no-sws-audit | gov_rfp | E16 | 게이트만 |
| J35 | 연결표 갱신 | gov_rfp | E05 | 표 행 단위 |
| J36 | CLAIM 여러 EV | gov_rfp | E05 | 동거 집합 |
| J37 | 날짜 data EV | quarterly | E13 | kind=date |
| J38 | 통화 복사 | gov_rfp | E13 | currency |
| J39 | 길이 키 | gov_rfp | E04 | length |
| J40 | 회신 3부 | gov_rfp | E01 | 확인/산출/다음 |

다음: [17_stop_rules.md](17_stop_rules.md).
