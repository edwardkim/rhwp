# 17 — 실사용 여정

gym 과제가 아니다. 실 에이전트가 파일을 처음 받을 때다.

| ID | 제목 | 정지 | 종류 |
| --- | --- | --- | --- |
| J01 | 처음 보는 메모 | X05 | no-special |
| J02 | 신청서 채우기 전 라우팅 | X10 | form |
| J03 | 보고서 표 추출 전 | X10 | table |
| J04 | 편람 조문 | X10 | structure |
| J05 | 설명회 차트 | X10 | chart |
| J06 | 외부 메일 문서 | X03 | security |
| J07 | 은닉 의심 | X03 | security |
| J08 | 법령 40쪽 | X10 | long |
| J09 | 논문 각주 | X10 | notes |
| J10 | 암호 문서 첫 시도 | X02 | encrypted |
| J11 | 암호 풀린 뒤 서식 | X04 | encrypted |
| J12 | 빈 경로 | X01 | empty |
| J13 | 0바이트 | X06 | empty |
| J14 | 사람용 메뉴 후 JSON | X10 | no-special |
| J15 | 첫 항목만 실행 | X10 | mixed |
| J16 | 보안 다음 표 | X03 | mixed |
| J17 | 보안 다음 서식 | X03 | mixed |
| J18 | 질문이 메뉴 자체 | X10 | no-special |
| J19 | 폴더에서 파일 하나 | X10 | mixed |
| J20 | 편집 요청이 따라옴 | X10 | form |
| J21 | HWPX 신청서 | X10 | form |
| J22 | HWP3 표본 | X05 | no-special |
| J23 | HML 메모 | X05 | no-special |
| J24 | capabilities 를 먼저 연 실수 | X10 | no-special |
| J25 | export-text 를 먼저 연 실수 | X03 | security |
| J26 | invented --rank | X07 | usage |
| J27 | 파일 두 개 한 줄 | X07 | usage |
| J28 | jq 로 보안만 필터 | X03 | security |
| J29 | why 개수 보고 인계 | X08 | table |
| J30 | 메뉴에 없는 검색 | X09 | no-special |
| J31 | 스튜디오에서 연 파일 | X10 | mixed |
| J32 | MCP hwp_explore 호출 | X10 | mixed |
| J33 | 온보딩 닥터 다음 | X10 | mixed |
| J34 | 배포 전 점검 요청 | X10 | mixed |
| J35 | 수신 후 점검 요청 | X10 | mixed |
| J36 | RAG 청크 전 | X10 | mixed |
| J37 | 요약 전 | X10 | mixed |
| J38 | 메일머지 전 | X10 | mixed |
| J39 | 세션 열기 전 | X10 | mixed |
| J40 | 한글 경로 | X10 | mixed |

나머지 여정은 `fixtures/journeys.json` 에 있다. 모두 `notGym: true`.
각 여정의 첫 살아 있는 동사는 `explore` 이거나, 메뉴가 가리킨 기존 명령이다.
