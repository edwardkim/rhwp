# task_m100_5795 처리결과 — 스킬 마크다운 체크아웃 개행 고정

- 이슈: [#5795](https://github.com/edwardkim/rhwp/issues/5795)
- 기준: `fb434269e` (devel)

## 1. 증상 (실측)

Windows 기본 설정(Git for Windows `core.autocrlf=true`)으로 `devel` 을 클린 체크아웃하면
스킬 계약 **14본**이 손대지 않은 상태에서 실패한다.

```
agent_knowledge_map_skill_contract::skill_frontmatter_names_knowledge_map ... FAILED
agent_fde_skill_contract::skill_frontmatter_names_fde                     ... FAILED
agent_fidelity_compare_skill_contract::skill_frontmatter_names_...        ... FAILED
agent_bug_hunter_skill_contract::skill_frontmatter_names_bug_hunter       ... FAILED
    panicked: frontmatter 필요
```

## 2. 원인

계약이 `SKILL.md` 첫 바이트를 개행까지 포함해 본다 — `assert!(text.starts_with("---\n"))`.
인덱스 바이트는 LF 라 Ubuntu CI 는 초록인데, Windows 체크아웃은 작업 트리에 CRLF 로 풀어
`---\r\n` 이 된다.

- `.claude/skills/*/SKILL.md` **27개 전부** 작업 트리 CRLF (인덱스는 전부 LF)
- 같은 판정을 쓰는 계약 14본 — 그중 1본은 `.agents/skills/bug-hunter/SKILL.md` 를 읽는다
- `.gitattributes` 에 스킬 마크다운 개행 지정이 없었다

## 3. 변경

`.gitattributes` 두 줄. 선례는 같은 파일의 `tests/golden_svg/**/*.svg text eol=lf` (#1786).

```
.claude/skills/**/*.md text eol=lf
.agents/skills/**/*.md text eol=lf
```

인덱스 바이트가 이미 LF 이므로 **저장소 내용은 바뀌지 않는다.** 바뀌는 것은 체크아웃뿐이라
Linux/macOS·CI 는 영향이 없다. `*.md` 로 좁혔다 — 같은 트리의 TSV/CSV 픽스처는
"CRLF 입력 자체가 계약"이라 건드리면 안 된다(같은 파일의 기존 주석).

## 4. 검증 (같은 PC, 같은 체크아웃)

작업 트리를 새 속성으로 다시 풀고(`git checkout`) 계약 14본을 전부 돌렸다.

| 스위트 | 계약 | 전 | 후 |
| --- | --- | --- | --- |
| 005·007·015·016·019·021·026·028·029·030·031·032 | 각 1본 | FAILED | ok |
| 020 | 2본 (fde·fidelity_compare) | FAILED | ok |
| **합계** | **14본** | 14 FAILED | **14 ok** |

- `.claude/skills/*/SKILL.md` CRLF **27/27 → 0/27**
- `python tools/skill_router/gate_new_skill.py` — OK (27 skills × 3 scans)
- `cargo test --test regression_suite_011` — 122 passed (skills·cli_json 계약)
- `cargo test --test regression_suite_020` 전체 — 121 passed

## 5. 이미 체크아웃한 사람이 할 일

속성은 다음 체크아웃부터 적용된다. 지금 작업 트리를 맞추려면:

```bash
git ls-files .claude/skills .agents/skills | grep '\.md$' | xargs rm -f
git checkout -- .claude/skills .agents/skills
```
