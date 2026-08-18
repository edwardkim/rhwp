# 19 — 발화 → 명령

사용자 말이 달라도 처음 보는 파일의 첫 살아 있는 동사는 explore 다.
메뉴를 본 뒤에야 항목의 command 로 갈린다.

| ID | 발화 | 명령 | 정지 |
| --- | --- | --- | --- |
| I001 | 이 문서로 뭘 할 수 있어? | rhwp explore <file> --json | X10 |
| I002 | 어떤 rhwp 도구를 써야 해? | rhwp explore <file> --json | X10 |
| I003 | 이 hwp 어떻게 다뤄? | rhwp explore <file> --json | X10 |
| I004 | 문서 탐색부터 하자 | rhwp explore <file> --json | X10 |
| I005 | rhwp explore 돌려줘 | rhwp explore <file> --json | X10 |
| I006 | 이 파일 첫 수가 뭐야? | rhwp explore <file> --json | X10 |
| I007 | 메뉴만 보여줘 | rhwp explore <file> --json | X10 |
| I008 | 다음 명령만 뽑아줘 | rhwp explore <file> --json | jq -r '.menu[0].command' | X10 |
| I009 | 이 문서가 뭐야? (설명) | rhwp explain <file> --json | X10 |
| I010 | 도구가 뭘 할 수 있어? (일반) | rhwp capabilities --json | X10 |
| I011 | 표 있어? 뽑을 수 있어? | rhwp explore <file> --json | X10 |
| I012 | 서식이야? 채울 수 있어? | rhwp explore <file> --json | X10 |
| I013 | 조문이 있어? | rhwp explore <file> --json | X10 |
| I014 | 차트 수치 뽑자 | rhwp explore <file> --json | X10 |
| I015 | 이 문서 보내도 돼? | rhwp explore <file> --json | X03 |
| I016 | 숨은 글 있어? | rhwp explore <file> --json | X03 |
| I017 | 긴 법령인데 어디부터? | rhwp explore <file> --json | X10 |
| I018 | 각주 구조부터 보자 | rhwp explore <file> --json | X10 |
| I019 | 암호 걸린 문서인데 | rhwp explore <file> --password … --json | X02 |
| I020 | 빈 파일 같아 | rhwp explore <file> --json | X06 |
| I021 | 본문부터 읽어줘 | rhwp explore <file> --json | X03 |
| I022 | export-text 먼저 하자 | rhwp explore <file> --json | X03 |
| I023 | capabilities 보고 고를게 | rhwp explore <file> --json | X10 |
| I024 | 메뉴에 표가 있어 | rhwp export-tables <file> --json | X10 |
| I025 | 메뉴에 누름틀이 있어 | rhwp fields <file> --json | X10 |
| I026 | 메뉴에 보안이 있어 | rhwp inspect injection <file> --json | X03 |
| I027 | 은닉만 메뉴에 있어 | rhwp inspect hidden-text <file> --json | X03 |
| I028 | 조문 메뉴가 있어 | rhwp export-structure <file> --json | X10 |
| I029 | 차트 메뉴가 있어 | rhwp chart-to-csv <file> --json | X10 |
| I030 | 장문 메뉴가 있어 | rhwp digest <file> --sections --json | X10 |
| I031 | 각주 메뉴가 있어 | rhwp explain <file> --json | X10 |
| I032 | 개요만 있어 | rhwp digest <file> --json | X05 |
| I033 | 폴더 전체 탐색 | rhwp-bulk-pipeline (explore 아님) | X10 |
| I034 | 지금 편집까지 해줘 | rhwp-safe-edit 로 인계 | X10 |
| I035 | why 에 적힌 대로 실행해 | command 만 실행. why 는 근거 | X08 |
| I036 | --rank 플래그 써 줘 | rhwp explore <file> --json | X07 |
| I037 | suggest 명령 있어? | 없음. explore | X07 |
| I038 | HWPX 도 같아? | rhwp explore <file> --json | X10 |
| I039 | HML 도 explore 돼? | rhwp explore <file> --json | X10 |
| I040 | 비밀번호 stdin 으로 | rhwp --password-stdin explore <file> --json | X02 |

전체는 `fixtures/intent_matrix.json`. 발명 명령 없음.
