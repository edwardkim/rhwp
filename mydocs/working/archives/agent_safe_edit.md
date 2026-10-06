---
kind: working
status: active
issue: 5294
---

# 에이전트 안전 편집 스킬 고도화 (#5294)

작업 브랜치: `feat/agent-safe-edit`
대상 스킬: `.claude/skills/rhwp-safe-edit/`
이슈: [agent: 안전 편집(run 계획·dry-run·verify) 스킬 고도화](https://github.com/edwardkim/rhwp/issues/5294)

## 1. 한 줄

실사용 에이전트가 HWP/HWPX 를 고칠 때 원본을 깨지 않도록, 이미 devel 에 있는
`edit` 과 `run` 을 문서·픽스처·시험으로 배선한다. 새 편집 로직 발명 금지. gym 금지.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- 원본 훼손 없이 편집하는 실사용 경로를 키운다.
- 기존 edit + run 계획 (dry-run, `-o`, `--verify`, exit 3/4 를 데이터로)을 문서화·배선.
- `.claude/skills/rhwp-safe-edit/` 확장 — SKILL.md + `references/`
  (single edit, run plans, verify loops, failure envelopes).
- 워크스루를 마크다운/JSON 픽스처 + 테스트로 남긴다.
- `mydocs/working/agent_safe_edit.md` (이 파일).
- additions 5000–10000, 최소 5000.
- PR 전 `cargo fmt --all -- --check`.
- isolation worktree, 브랜치 `feat/agent-safe-edit` from `upstream/devel`.
- 한국어 PR, base `devel`, `closes #5294`, `--body-file`.
- `git add -A` 금지.

금지:

- 새 edit 로직 발명.
- gym/ 를 열지 않는다. gym 을 만지지 않는다 (gym 금지).
- 다른 네 에이전트 스킬을 이 파동에서 고치지 않는다
  (`rhwp-onboarding`, `rhwp-mcp-session`, `rhwp-provenance`, `rhwp-doc-triage`).

## 3. 왜 스킬만 키우나

K8 안전 편집 스킬은 이미 있다 (원 PR #4215, #4253 통합, 커밋 1ac20c359).
SKILL.md 한 장이 판단 트리와 함정 10개조를 담고, exit 3 을 판정 데이터로 읽으라고 적는다.

트랙 K 기록 (`mydocs/tech/agent_roadmap/track_k_skills.md` K8 DoD)이 남긴 구멍은
"임의 편집 요청 표본에서 무계획 실행 0"을 **실행 로그·표본 테스트로** 확인하지
않았다는 점이다. 판단 트리가 구조적으로 그 경로를 배제한다는 설계상 주장만 있었다.

이 파동은 그 구멍을 메운다.

- 1층/3층/루프/봉투를 자식 문서로 쪼개 에이전트가 한 장을 다 읽지 않고도
  해당 갈래를 끝까지 따라가게 한다.
- 16편 워크스루가 "발견 없이 `--in-place`" 경로를 표본에서 빼 둔다.
- JSON 픽스처가 유효/무효 계획과 실패 봉투의 키를 기계가 읽게 한다.
- Python·Rust 시험이 문서와 픽스처가 같은 단어를 쓰는지 고정한다.

구현(`run_plan_engine`, `edit fill-fields` 등)은 그대로다. 테스트가 기존 계약
테스트(`edit_*_contract.rs`, `plan_schema_contract.rs`, `skills_contract.rs`)를
대체하지 않는다. 스킬 팩이 그 계약을 인용하는지만 본다.

## 4. 범위

만진 것:

| 경로 | 역할 |
|------|------|
| `.claude/skills/rhwp-safe-edit/SKILL.md` | 라우터. 자식 문서 표, 하지 않는 것, CAS·조건절 함정 추가 |
| `references/single_edit.md` | 1층 6종 + csv-to-table + batch fill |
| `references/run_plans.md` | 계획서 스키마 1.1, action 4, if, assertions, CAS |
| `references/verify_loops.md` | 발견→dry-run→-o/--verify→재독→눈검증→ir-diff |
| `references/failure_envelopes.md` | exit 0/1/2/3/4, 성공 안 미완료, 분기 의사코드 |
| `examples/01`–`16` + README | 워크스루 |
| `fixtures/plans/*.json` | 유효 12 · 무효 9 |
| `fixtures/envelopes/*.json` | 분기 표본 19 |
| `fixtures/loops/*.json` | 루프 정의 6 |
| `fixtures/catalog.json` | 목록의 단일 출처 |
| `scripts/tests/test_agent_safe_edit.py` | 파일 계약 (바이너리 없음). `tests/` 신규 integration 은 suite-policy 상 `tests/cases/` 만 허용이라 이 파동은 Python 시험을 정본으로 둔다 |
| `mydocs/working/agent_safe_edit.md` | 이 기록 |

만지지 않은 것:

- `src/` 편집 엔진, `run_plan_engine`, 새 CLI 플래그
- `gym/` 전부
- `.claude/skills/rhwp-onboarding/`
- `.claude/skills/rhwp-mcp-session/`
- `.claude/skills/rhwp-provenance/`
- `.claude/skills/rhwp-doc-triage/`
- 다른 스킬 SKILL.md
- 공개 샘플 HWP 바이너리 (새 픽스처를 스킬 폴더에 넣지 않음)

## 5. 기존 계약의 지도 (발명하지 않은 것)

문서가 가리키는 구현·매뉴얼의 위치다. 스킬이 이 표를 복제하지 않고 인용한다.

| 계약 | 출처 |
|------|------|
| 종료 코드 0/1/2/3/4 | cli_commands.md §종료 코드, #2707 |
| fill-fields 봉투 · 반복 필드 | cli_commands.md, #3329, #3476 |
| replace-text 0건 무산출 | cli_commands.md, #3373 |
| set-cell overflow · keep-style | #3381, #3391, #3480 |
| insert-image HWPUNIT · overflow | #3719 §6-5 |
| redact `-o` 필수 · `--no-raw` | #3719 §6-11 |
| sanitize 재실행 0 | #3719 §6-11 |
| 입력 형식 보존 | #3383 |
| run 선검증→원자→저널 | #3703, main.rs `run_plan_engine` |
| 조건절 if 1회 입력 기준 | #3719 §6-8 |
| 계획 스키마 1.1 · CAS | #4378, plan_schema.rs, PLAN_SCHEMA_VERSION |
| `--verify` 봉투-exit | #3702, edit_verify_contract.rs |
| 스킬 실재 명령 가드 | #4508, skills_contract.rs |
| dry-run / changedPages 실측 | agent_surface_playbook.md §9 |
| run 실측 저널 | agent_task_playbook.md §12 |

action 4종은 스키마 `$defs` 와 실행기 `match action` 이 같다.
다섯 번째를 스킬이 만들어 넣지 않는다. `insert_image` 를 유효 계획에 넣은
픽스처는 `invalid_unknown_action.json` 뿐이다.

## 6. 1층과 3층의 엄격함 차이 (문서가 특히 붙든 곳)

같은 찾기라도 층이 다르면 판정이 다르다. 에이전트가 여기를 섞으면
"1층에서 성공한 계획을 3층에 넣었더니 exit 2" 가 된다.

| 상황 | 1층 `edit` | 3층 `run` |
|------|------------|-----------|
| 없는 필드 | `notFound` + exit 0 | 선검증 `invalid` + exit 2 |
| 치환 0건 | `replacedCount: 0`, 산출 없음, exit 0 | `일치 0건` invalid + exit 2 |
| `--verify` 차이 | 산출 **남김**, exit 3 | `assertions.verify` 면 산출 **안 남김**, exit 3 |
| 동명 순번 생략 | `ambiguous` + 첫 칸, exit 0 | 동일 (칸은 존재하므로 선검증 통과) |
| 그림/마스킹/메타 | CLI 있음 | action 없음 |

스킬은 이 표를 세 문서(single_edit, run_plans, failure_envelopes)에 반복한다.
시험이 "1층 verify 는 outputKept, 3층은 아님" 픽스처를 고정한다.

## 7. exit 3 세 갈래

이슈가 "exit 3/4 를 데이터로" 라고 한 이유다. 같은 숫자가 세 가지 다음 행동을 뜻한다.

1. 1층 `--verify` — 파일 있음. `InspectKeptOutput`.
2. 3층 `assertions.verify` — 파일 없음. `ReplanNoOutput`.
3. 3층 CAS `preconditionFailed` — 파일 없음, `invalid[]` 비어 있음, `nextCall`.
   사용법(2)이 아니다.

exit 4 는 `convert`/`export-hwpx --verify-pages` 전용. `edit`/`run` 기본 경로는 내지 않는다.

성공 코드(0) 안의 미완료 (`notFound`, `ambiguous`, `overflow`, `batch` 행 `notFound`,
`verify: null`)가 더 위험하다. 예외가 안 나므로 에이전트가 완성본으로 넘긴다.
failure_envelopes.md §3 과 12·11·14 편이 이 갈래의 표본이다.

## 8. 디렉터리 규약

```
.claude/skills/rhwp-safe-edit/
  SKILL.md                 라우터 (frontmatter name == 폴더명)
  references/
    single_edit.md
    run_plans.md
    verify_loops.md
    failure_envelopes.md
  examples/
    README.md
    01_… 16_….md
  fixtures/
    catalog.json
    plans/{valid_*,invalid_*}.json
    envelopes/*.json
    loops/*.json
```

`catalog.json` 이 목록의 단일 출처다.  stray JSON 이 생기면 시험이 실패한다.
워크스루가 가리키는 픽스처 파일명도 catalog 와 같아야 한다.

상대 링크는 스킬 폴더 안에서 해석되고, `mydocs/manual/…` 로 나가는 링크는
저장소 루트 기준으로 존재해야 한다. Python 시험이 깨진 링크를 모은다.

## 9. 시험

### 9.1 파일 계약 (바이너리 없음)

```
python -m unittest scripts.tests.test_agent_safe_edit
```

확인하는 것:

- 레이아웃, frontmatter, 자식 문서 네 장
- 레퍼런스별 계약 토큰
- catalog ↔ 디스크 파일 1:1
- 유효 계획은 `planVersion`/`input`/`output`/`steps` + action ∈ 4
- 무효 계획은 알려진 규칙 하나를 깬다
- 봉투마다 `_skillMeta.exit` ∈ {0,1,2,3,4} 와 `branch`
- CAS 봉투는 exit 3 + 빈 `invalid` + `nextCall`
- 루프 단계가 `rhwp` 로 시작하고 `--dry-run` 을 포함한다
- `rhwp <머리>` 가 알려진 집합에 속한다
- 상대 링크가 존재한다
- gym/packs 등을 편집하라고 하지 않는다
- 형제 스킬 경로가 사라지지 않았다 (본문은 요구하지 않음)

### 9.2 기존 구현 시험 (이 파동이 돌리지 않아도 되는 이유)

`edit_fill_fields_contract.rs`, `edit_replace_text_contract.rs`,
`edit_set_cell_contract.rs`, `edit_verify_contract.rs`,
`plan_schema_contract.rs` 가 엔진을 이미 지킨다. 이 파동은 엔진을 바꾸지
않았으므로 그 시험들을 재실행하는 것이 게이트는 아니다.
로컬에서 의심되면 그 테스트 이름만 골라 돌린다.

### 9.4 fmt 게이트

```
cargo fmt --all -- --check
```

이슈 DoD 의 하드 게이트. Rust 파일을 넣었으므로 PR 생성 전에 통과해야 한다.
`newline_style = Unix`. Python·Markdown 은 rustfmt 대상이 아니다.

## 10. 에이전트가 이 스킬을 소비하는 순서

1. SKILL.md 판단 트리 — 1건이면 1층, 여러 건이면 3층.
2. 해당 레퍼런스 한 장.
3. 같은 번호의 examples/ 편.
4. 명령을 조립할 때 fixtures/plans 또는 envelopes 의 키를 베낀다.
5. 종료 코드 → failure_envelopes 의사코드.
6. 완료 식을 만족할 때만 산출물을 사용자에게 넘긴다.

무계획 실행(발견 없이 `--in-place`, dry-run 없이 `-o`, `filledCount` 만 보고 완료)은
위 순서에 칸이 없다. 워크스루의 "하지 않는 것" 절이 그 칸을 비워 둔다.

## 11. 의도적으로 복제하지 않은 것

- MCP 도구 스키마. `capabilities --mcp` 가 단일 출처. 세션 루프는 verify_loops 에
  참고 한 절만 있고, rhwp-mcp-session 스킬 본문은 그대로다.
- 출처 표지 소비. `untrustedContent` 키 부재 ≠ false 는 한 줄로만 가리키고
  rhwp-provenance 에 맡긴다.
- 문서 트리아지 순서 (`info`→`explain`→…). 발견은 fields/export-tables/search 만.
- 온보딩 닥터. 바이너리 위치는 SKILL 의 `cargo build --release` 한 줄.
- gym pack · 채점 · admission. 문장에 gym 이 나오면 "만지지 않는다"는 표지다.

## 12. 검증 실측 (이 작업)

로컬에서 수행한 것:

```
python -m unittest scripts.tests.test_agent_safe_edit
cargo fmt --all -- --check
```

(실행 시각·통과 수는 PR 본문의 테스트 체크리스트에 적는다.)

수행하지 않은 것:

- 전체 `cargo test` (엔진 변경 없음)
- clippy (Rust 는 테스트만, 경고 0을 이 파동의 엔진 변경으로 요구하지 않음)
- 시각 SVG 전후 (레이아웃 변경 없음)
- WASM
- gym audit / certify

## 13. PR 메모

- base: `devel`
- head: `kevin9327:feat/agent-safe-edit` (origin 이 fork)
- 제목 한국어, 본문 `--body-file` UTF-8 without BOM
- `closes #5294`
- `git add -A` 를 쓰지 않고 경로를 지정해 add
- 생성 후 `gh pr view --json body` 로 한글·BOM·`??` 확인

## 14. 남은 일 (이 PR 밖)

- K8 원 DoD 의 "임의 편집 요청 표본을 실제 에이전트 세션으로 돌려 무계획 실행 0"은
  이 문서·픽스처가 표본을 제공할 뿐, 라이브 에이전트 세션 로그는 여기 없다.
- 바인딩 `Plan.check/run` README 는 이 devel 스냅샷에 파일이 없을 수 있다.
  CLI 두 호출이 정본이라고 적었다.
- capability 카탈로그에 `rhwp-safe-edit` 행이 원래 없었다. 이 파동은 스킬을
  새로 만들지 않고 확장만 하므로 카탈로그 행을 추가하지 않았다.
  등록이 필요하면 별도 이슈에서 `CAP-5294` 를 논의한다.
- skills_contract.rs 를 레퍼런스 md 까지 재귀 스캔하도록 넓히는 것은
  다른 스킬에 영향을 줄 수 있어 이 파동에서 하지 않았다.

## 15. 파일 점검표 (작성 시점)

레퍼런스 4, 예제 16+README, 계획 21, 봉투 19, 루프 6, catalog 1,
Python 시험 1, 작업 기록 1, SKILL 1.

합계 줄 수는 `git diff --shortstat upstream/devel` 이 정본이다.
이슈 창 5000–10000 을 벗어나면 예제·픽스처를 더하거나 줄인다.
패딩용 난문은 넣지 않는다 — 각 줄은 기존 계약의 키·분기·함정이다.
