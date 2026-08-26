# 21 — 명령 상자 (발명 금지)

살아 있는 동사는 이것이다.

| 명령 |
| --- |
| rhwp explore <file> --json |
| rhwp explain <file> --json |
| rhwp capabilities --json |
| rhwp inspect injection <file> --json |
| rhwp inspect hidden-text <file> --json |
| rhwp fields <file> --json |
| rhwp export-tables <file> --json |
| rhwp export-structure <file> --json |
| rhwp chart-to-csv <file> --json |
| rhwp digest <file> --sections --json |
| rhwp digest <file> --json |

없는 것: `suggest`, `affordances`, `next`, `recommend`, `--rank` 플래그,
세션 제안 도구, 편집 하위명령으로의 탐색. 오타 난 하위명령은 exit 2.

## 경로 자리

메뉴의 `command` 는 `<file>` 자리표시자를 쓴다. 소비자가 자기 경로로
치환한다. 원본 경로에 공백이 있으면 따옴표를 붙인다.

## 전역 비밀번호

`--password` / `--password-stdin` 은 하위명령 앞이나 어디에나 올 수
있다. pre-scan 이 집어 간다. `explore` 가 모르는 옵션으로 거절하지 않게
전역으로 빼 둔 것이다.
