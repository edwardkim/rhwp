# task_m100_5791 stage1 — 명령별 `--help` 부재 분석

- 이슈: [#5791](https://github.com/edwardkim/rhwp/issues/5791)
- 기준 커밋: `fb434269e` (devel), 바이너리 `rhwp v0.8.4`
- 실행 환경: Windows 11 / `cargo build --bin rhwp` (debug)

## 1. 실측 — `capabilities` 98종 전수

`rhwp <명령> --help` 를 98종에 그대로 돌렸다.

| 거동 | 개수 | 예 |
| --- | --- | --- |
| `오류: 알 수 없는 옵션: --help` → exit 2 | 79 | `fields` `info` `search` `export-text` `digest` `explain` |
| `--help` 를 **파일 경로로 읽어** 실패 → exit 1 | 5 | `dump-pages` `dump-extents` `dump-note-shape` `dump-records` `core-pages` |
| 사용법 한 줄 → exit 0 | 14 | `export-pdf` `threat-scan` `hwp5-*` 12종 |

하위 명령은 전멸이었다 — `edit` 88종 · `inspect` 4종 · 그룹 자신 `edit`/`inspect`/`batch`
전부 exit 2.

같은 저장소의 뒤에 나온 에이전트 바이너리(`rhwp-agent`, `rhwp-q-*`)는 하위 명령마다
`--help|-h` 를 받는다(`src/bin/rhwp-q-kit/main.rs:146`). 본 CLI 만 규약 밖이었다.

## 2. 비용

명령별 진입점이 없으니 대안은 `rhwp --help` 통짜뿐이다 — **71,978 B / 1,163 줄**.
필요한 절은 `fields` 213 B(5줄) · `info` 168 B · `inspect hidden-text` 631 B 다.
한 명령을 알아내려고 **338배**를 읽는다. 저장소 내부 분석
(`mydocs/tech/agent_architecture/layer_model.md` §4.6 B5)이 같은 자리를 이미 지목했고,
그때 기록된 29,590 B 는 지금 71,978 B 로 늘었다.

`capabilities` 로 우회할 수도 없다. `edit` 항목의 `flags` 는 하위 88종의 **합집합 68개**라
`edit fill-fields` 에 `--bold`·`--rows` 가 있는 것처럼 보인다. 하위별 옵션을 물어볼 창구가
자기서술에도 없다.

## 3. 원인

`src/main.rs` 의 최상위 `match` 는 `--help` 를 **명령 이름 자리에서만** 안다
(`Some("--help") | Some("-h") => print_help()`). 명령 뒤에 붙은 `--help` 는 각 명령의
옵션 파서로 흘러가고, 파서는 모르는 토큰이므로 사용법 오류(2)를 낸다. `dump-*` 계열은
옵션 파서가 아니라 **첫 위치 인자를 파일 경로로 받으므로** `--help` 를 파일 이름으로 읽는다.

## 4. 설계 판단

**텍스트를 새로 쓰지 않는다.** `rhwp --help` 안에는 이미 명령별 절이 있다
(`  <명령> <인자> [옵션]` 머리줄 + 6칸 들여쓴 본문). 명령별 도움말은 새 문서를 만드는
일이 아니라 **있는 절을 고르는 일**이다.

고르는 자리를 어디에 둘 것인가가 유일한 선택지였다.

| 안 | 내용 | 판단 |
| --- | --- | --- |
| A | 도움말 텍스트를 `.txt` 로 빼고 `include_str!` | 1,100줄 이동 — 명령 추가 PR 과 충돌면이 커진다 |
| B | 출력 함수를 `&mut Vec<String>` 싱크로 바꾼다 | `println!` 호출 지점 1,163개를 전부 고쳐야 한다 |
| **C** | **출력 경계 한 곳에서 거른다** | 호출 지점 0줄 변경. 통짜 출력은 바이트 그대로 |

C 를 택했다. `help/sink.rs` 가 형제 모듈이 쓰는 `println!` 을 같은 이름의 매크로로 가려
`emit` 한 곳으로 모은다. 스코프가 없으면 그대로 stdout 으로 찍으므로 통짜 출력은 변하지 않고
(**71,978 B / 1,163 줄 동일 확인**), 스코프가 걸리면 머리줄 기준으로 절만 모은다.

절이 없는 명령(top-level 3종 `dump-extents`·`measure-width`·`core-pages`, `edit` 하위 19종)은
`capabilities` 의 요약·선언 플래그로 최소 답을 만든다. **없는 사용법을 지어내지 않는다.**

그룹(`edit`·`inspect`)을 물으면 절 전체(예: `edit` 603줄)가 아니라 **목차**(이름 + 한 줄 요약,
88줄)를 낸다. 알고 싶은 것은 "하위가 무엇이고 어느 것을 더 볼까"이고 다음 한 수가
`rhwp edit <하위> --help` 이기 때문이다.

## 5. 안전 경계

- `--help` 가 **값 자리**면 가로채지 않는다 — `edit replace-text <파일> --find --help` 는
  "--help" 를 찾는 치환이다. 바로 앞 토큰이 플래그면 값으로 본다.
- 모르는 명령(`rhwp bogus --help`)은 기존 오류 경로(exit 2) 그대로.
- 디스패치보다 **앞**에서 답한다 — 옵션 파서나 파일 열기에 닿기 전이라 부작용이 없다.
