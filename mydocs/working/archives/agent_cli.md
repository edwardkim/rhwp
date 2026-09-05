---
kind: working
status: active
issue: 5316
---

# 에이전트 CLI 분석·디버깅 스킬 고도화 (#5316)

작업 브랜치: `feat/agent-cli`
대상 스킬: `.agents/skills/rhwp-cli/`
이슈: [agent: CLI 분석·디버깅 스킬 고도화](https://github.com/edwardkim/rhwp/issues/5316)

이슈 번호 5315 는 보안 스윕(#5307) 고도화다. 본 파동은 열린 이슈
「agent: CLI 분석·디버깅 스킬 고도화」(#5316) 를 닫는다.

## 1. 한 줄

실사용 에이전트가 기존 `rhwp` CLI 만으로 HWP/HWPX 를 내보내고
레이아웃을 좁히고 저장 계약을 읽도록, 스킬을 요청→명령 매핑·디버그 순서·
예외 봉투·픽스처·계약 시험으로 닫는다. 새 CLI 없음. gym 없음.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- `.agents/skills/rhwp-cli/` SKILL.md + references/ + examples/ + fixtures
- 요청→명령: export-svg/png/pdf/text/markdown, dump-pages, dump, dump-records,
  diag, info, export-render-tree, ir-diff, thumbnail, convert, hwp5-*
- 레이아웃/겹침 순서: export-svg --debug-overlay → dump-pages → dump →
  ir-diff → export-render-tree → hwp5-inventory-diff
- 페이지는 0부터, 단위는 HWPUNIT
- 자기 round-trip ≠ 한컴 호환
- HWPX→HWP 저장 계약 (oracle vs generated)
- 예외 봉투: missing file, bad page index, native-skia missing, load fail
- `mydocs/working/agent_cli.md` (이 파일)
- 형제 에이전트 PR 과 같은 계약 시험
- additions 5000–10000, 최소 5000
- PR 전 `cargo fmt --all -- --check`
- isolation worktree, 브랜치 `feat/agent-cli` from `upstream/devel`
- 한국어 PR, base `devel`, `closes #5316`, `--body-file`
- `git add -A` 금지

금지:

- gym/
- 다른 스킬
- 열린 PR 파일
- DocumentCore 편집 구현
- 새 rhwp CLI 명령

## 3. 왜 스킬만 키우나

K 트랙의 `rhwp-cli` 는 이미 SKILL.md 한 장으로 매핑과 디버그 순서를 적고 있다.
구멍은 명령별 레시피·예외 봉투·페이지/단위 함정·oracle/generated 계약·
픽스처·시험이 없어서 에이전트가 `-p 4` 를 한컴 4쪽으로 넣거나
라운드트립 통과를 한컴 호환으로 읽거나 `export-layout` 같은 이름을 발명하는
경로가 표본으로 막혀 있지 않다는 점이다.

구현 (`src/main.rs` 디스패치) 은 그대로다.
종료 코드는 #2707, ir-diff JSON 은 #3274, PNG feature 스텁은 기존 계약.
이 파동은 그 계약을 스킬이 같은 단어로 인용하는지만 본다.

## 4. 범위

만진 것:

| 경로 | 역할 |
|------|------|
| `.agents/skills/rhwp-cli/SKILL.md` | 라우터. 매핑, 6단, 봉투, 하지 않는 것 |
| `references/00`–`28` | 명령 장·디버그 순서·단위·왕복·저장 계약·예외 |
| `examples/01`–`24` | 워크스루 |
| `fixtures/` | 명령 맵·봉투·시나리오·트레이스 |
| `scripts/tests/test_agent_cli.py` | 파일 계약 (바이너리 없음) |
| `tests/cases/agent_cli_*.rs` | 같은 계약을 Rust 로 |
| `mydocs/working/agent_cli.md` | 이 기록 |

만지지 않은 것:

- `src/` CLI 구현, DocumentCore
- `gym/` 전부
- 다른 `.agents/skills/*` 본문
- 공개 샘플 HWP 바이너리

## 5. 기존 계약의 지도

| 계약 | 출처 |
|------|------|
| 종료 코드 0/1/2/3/4 | #2707, cli_commands.md |
| 페이지 0 기준 | 공통 `-p`, export-text pages[].page |
| 파일 없음 stderr | `오류: 파일을 읽을 수 없습니다` |
| 페이지 범위 stderr | `오류: 페이지 번호가 범위를 벗어났습니다 (0~N)` |
| PNG feature 부재 | native-skia 스텁, exit 2 |
| 로드 실패 | `오류: 문서 파싱 실패` |
| ir-diff --json 차이 | exit 3, identical:false |
| oracle vs generated | cli_commands.md §4 |
| 자기 왕복 ≠ 한컴 | render-diff / hwp5-roundtrip 주의문 |
| HWPUNIT | 1인치=7200, 1px=75, 1mm≈283.46 |

capability 카탈로그의 `rhwp-cli` 행은 `LEGACY-d86c935bc` 로 이미 있다.
이 파동은 같은 책임 단위를 확장한다. 새 capability ID 를 만들지 않는다.
고도화 근거 이슈는 #5316 (CAP-5316 은 같은 행의 주석이지 새 등록이 아니다).

## 6. 디렉터리 규약

```
.agents/skills/rhwp-cli/
  SKILL.md
  references/          00–28 + _gen_pack.py
  examples/            워크스루 24 + README
  fixtures/
    skill_index.json   목록의 단일 출처
    command_map.json
    debug_order.json
    page_units.json
    hwp5_family.json
    envelopes/         예외·성공 표본
    transcripts/       발화→명령
    traces/            짧은 트레이스
    scenario_catalog.json
```

`skill_index.json` 이 목록의 단일 출처다.

## 7. 시험

```
python -m unittest scripts.tests.test_agent_cli
```

확인하는 것:

- 레이아웃, frontmatter, 자식 문서
- 디버그 6단 순서
- 페이지 0 기준 · HWPUNIT
- oracle/generated 순서
- 예외 봉투 4종 stderr/exit
- 새 CLI 금지, gym 비범위
- 형제 스킬 경로가 사라지지 않음
- scenario_catalog 가 기존 명령만 씀

Rust `tests/cases/agent_cli_*.rs` 가 같은 불변식을 픽스처만 읽고 고정한다.
바이너리를 부르지 않는다. CI 의 `rust-test-suite-manifest --prepare` 가
`tests/cases` 원본을 하네스에 배정한다.

## 8. fmt 게이트

```
cargo fmt --all -- --check
```

rustfmt `newline_style=Unix`. 렌더/레이아웃 변경 없음. 시각 검증 해당 없음.
