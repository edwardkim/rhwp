# exit 코드

세 명령이 같은 가족을 쓴다 (#2707).

| 코드 | 이름 | 언제 | stdout |
| ---: | --- | --- | --- |
| 0 | OK | attest 성공, verify 일치, audit 전건, lineage valid | 봉투 |
| 1 | IO | 파일/폴더를 읽을 수 없음. 머리 캡슐 없음 | 0바이트 (replay --json 엔진 오류만 봉투) |
| 2 | 사용법 | 인자·플래그·형식·빈 감사 폴더 | 0바이트 |
| 3 | 판정 | verify 불일치, audit failed[], lineage invalid | 봉투 |

exit 3 을 재시도하거나 도구 버그로 올리지 않는다. 봉투의
`reproduced` / `failed` / `brokenAt` 을 읽는다.

`fixtures/exceptions/` 가 명령×코드 행렬을 닫는다.
