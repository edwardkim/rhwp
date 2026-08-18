#!/usr/bin/env python3
"""Generate rhwp-contributor examples, checklists, envelopes, transcripts.

Deterministic. No rhwp binary. No gym. Existing commands only:
git / gh / cargo / python / node / rhwp replay|audit|lineage (pointers).

This generator is the single source for examples/ and fixtures/ listings.
Tests lock catalog ↔ disk and the hard-gate command spelling.
"""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIX = ROOT / "fixtures"
EXAMPLES = ROOT / "examples"
ISSUE = 5322
SKILL = "rhwp-contributor"
BRANCH = "feat/agent-contributor"
HARD_GATE = "cargo fmt --all -- --check"
STALE_FMT = "cargo fmt --check"
CLIPPY = "cargo clippy -- -D warnings"

FORBIDDEN_WORKTREES = [
    r"C:\Users\swsz9\rhwp",
    r"C:\Users\swsz9\rhwp-desk",
    r"C:\Users\swsz9\rhwp-desk-design",
    r"C:\Users\swsz9\rhwp-handoff",
    r"C:\Users\swsz9\rhwp-scaffold-final",
    r"C:\Users\swsz9\rhwp-doc-repro",
]

STEPS = [
    ("issue", "이슈 선등록", "gh issue view / gh pr list --state open"),
    ("analyze", "정본·계약 분석", "mydocs/manual + 기존 계약 시험"),
    ("branch", "upstream/devel 브랜치", "git fetch upstream devel"),
    ("implement", "범위 안 구현", "named git add, no DocumentCore"),
    ("gate", "로컬 게이트", HARD_GATE),
    ("receipt", "작업 영수증 포인터", "rhwp replay --capsule"),
    ("working", "처리 결과 문서", "mydocs/working/agent_contributor.md"),
    ("pr", "한국어 PR", "gh pr create --base devel --body-file"),
]


def dump(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(obj, ensure_ascii=False, indent=2) + "\n"
    path.write_text(text, encoding="utf-8", newline="\n")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not text.endswith("\n"):
        text += "\n"
    path.write_text(text, encoding="utf-8", newline="\n")


def meta(command: str, exit_code: int, branch: str = "ok") -> dict:
    return {
        "_skillMeta": {
            "skill": SKILL,
            "issue": ISSUE,
            "command": command,
            "exit": exit_code,
            "branch": branch,
            "hardGate": HARD_GATE,
            "staleFmtRejected": True,
        }
    }


EXAMPLES_SPEC = [
    (
        "01_issue_first.md",
        "01 — 이슈 선등록",
        "단 1. 코드를 쓰기 전에 이슈 번호와 DoD 를 확보한다.",
        "issue-first.md",
        "fixtures/checklists/step_01_issue.json",
        [
            "같은 제목의 열린 이슈·PR 을 검색한다.",
            "없으면 이슈를 만들고, 있으면 그 번호를 쓴다. #5322 는 이미 있다.",
            "DoD 에 `cargo fmt --all -- --check` 와 base=devel 을 적는다.",
        ],
        [
            'gh issue list --repo edwardkim/rhwp --search "contributor" --state open',
            'gh pr list --repo edwardkim/rhwp --search "contributor" --state open',
            "gh issue view 5322 --repo edwardkim/rhwp",
        ],
        [
            "이슈 번호가 있다",
            "열린 중복 PR 이 없다 (또는 예외 경로)",
            "DoD 에 HARD GATE 명령이 있다",
        ],
    ),
    (
        "02_duplicate_open_pr.md",
        "02 — 열린 PR 중복",
        "단 1 예외. 같은 주제의 열린 PR 이 있으면 새 PR 을 만들지 않는다.",
        "exceptions.md",
        "fixtures/envelopes/duplicate_open_pr.json",
        [
            "`gh pr list --state open` 에 같은 head 또는 같은 이슈가 있다.",
            "새 `gh pr create` 는 거절한다.",
            "이슈에 중복이라고 적거나 기존 PR 에 합류한다.",
        ],
        [
            'gh pr list --repo edwardkim/rhwp --search "feat/agent-contributor" --state open',
        ],
        [
            "새 PR 을 만들지 않았다",
            "이슈 번호는 그대로 인용했다",
        ],
    ),
    (
        "03_analyze_canonical.md",
        "03 — 정본을 읽고 이슈에 기록",
        "단 2. AGENTS.md 로딩 순서와 local_validation.md §4.3 을 인용한다.",
        "analyze-canonical.md",
        "fixtures/checklists/step_02_analyze.json",
        [
            "CONTRIBUTING.md 의 `cargo fmt --all -- --check` 를 확인한다.",
            "형제 스킬 계약 시험 패턴을 읽는다.",
            "DocumentCore 편집 로직을 설계하지 않는다고 이슈에 적는다.",
        ],
        [
            "gh issue view 5322 --repo edwardkim/rhwp",
        ],
        [
            "정본 경로가 이슈에 있다",
            "검증 계획에 HARD GATE 가 있다",
            "비범위에 gym · 새 CLI · DocumentCore 가 있다",
        ],
    ),
    (
        "04_branch_from_devel.md",
        "04 — upstream/devel 에서 분기",
        "단 3. fetch 없이 로컬 devel 에서 나누지 않는다.",
        "branch-isolation.md",
        "fixtures/transcripts/fetch_devel.json",
        [
            "`git fetch upstream devel` 이 첫 명령이다.",
            "브랜치 이름은 이슈가 정한 `feat/agent-contributor`.",
            "base 는 항상 devel 이다.",
        ],
        [
            "git fetch upstream devel",
            "git rev-parse upstream/devel",
            "git merge-base --is-ancestor upstream/devel HEAD",
        ],
        [
            "upstream/devel 이 조상이다",
            "브랜치가 feat/agent-contributor 다",
        ],
    ),
    (
        "05_isolation_worktree.md",
        "05 — 격리 워크트리",
        "단 3. 본진을 더럽히지 않는다. 빈 경로에 worktree add.",
        "isolation-worktree.md",
        "fixtures/envelopes/worktree_created.json",
        [
            "`git worktree list` 로 빈 이름을 확인한다.",
            "새 경로 `rhwp-agent-contributor` 는 목록에 없어야 한다.",
            "금지 경로(본진, desk*, handoff, scaffold-final, doc-repro)를 쓰지 않는다.",
        ],
        [
            "git worktree list",
            "git worktree add -b feat/agent-contributor C:\\Users\\swsz9\\rhwp-agent-contributor upstream/devel",
        ],
        [
            "새 경로가 list 에 있다",
            "금지 경로를 checkout 하지 않았다",
        ],
    ),
    (
        "06_never_steal_named_worktree.md",
        "06 — named worktree 를 훔치지 않는다",
        "단 3 예외. 이미 있는 이름을 강제 checkout 하지 않는다.",
        "isolation-worktree.md",
        "fixtures/layouts/forbidden-worktrees/registry.json",
        [
            "대상 경로가 `git worktree list` 에 있으면 그 자리는 끝난 것이다.",
            "접미를 바꿔 새 경로를 만든다. `rhwp-desk*` 는 계열 전체가 금지.",
            "다른 작업의 브랜치를 reset 하지 않는다.",
        ],
        [
            "git worktree list",
        ],
        [
            "기존 named 를 checkout 하지 않았다",
            "새 빈 경로를 썼다",
        ],
    ),
    (
        "07_implement_without_documentcore.md",
        "07 — DocumentCore 편집 로직을 발명하지 않는다",
        "단 4. 이 스킬은 절차다. src/document_core 에 새 연산을 짜지 않는다.",
        "implement-scope.md",
        "fixtures/checklists/step_04_implement.json",
        [
            "만질 경로: `.claude/skills/rhwp-contributor/`, working 문서, 계약 시험.",
            "`src/document_core/` 는 읽기만. 쓰기 경로를 추가하지 않는다.",
            "새 rhwp 하위명령을 만들지 않는다.",
        ],
        [
            "git diff --name-only upstream/devel",
        ],
        [
            "diff 에 document_core 가 없다",
            "diff 에 gym/ 가 없다",
            "새 [[bin]] 이 없다",
        ],
    ),
    (
        "08_never_git_add_all.md",
        "08 — git add -A 거부",
        "단 4. 이름을 댄 파일만 올린다.",
        "staging-named-files.md",
        "fixtures/envelopes/git_add_a_rejected.json",
        [
            "`git add -A` / `git add .` 를 쓰지 않는다.",
            "스킬·working·계약 시험 경로를 하나씩 add 한다.",
            "`git diff --cached --name-only` 로 범위를 확인한다.",
        ],
        [
            "git add -- .claude/skills/rhwp-contributor/SKILL.md",
            "git add -- mydocs/working/agent_contributor.md",
            "git diff --cached --name-only",
        ],
        [
            "인덱스에 범위 밖 파일이 없다",
            "셸에 git add -A 가 없다",
        ],
    ),
    (
        "09_fmt_all_check.md",
        "09 — HARD GATE cargo fmt --all -- --check",
        "단 5. 이 명령이 0 이 아니면 gh pr create 금지.",
        "fmt-hard-gate.md",
        "fixtures/envelopes/fmt_pass.json",
        [
            "정본 명령은 `cargo fmt --all -- --check` 다.",
            "`crates/` 가 있으면 반드시 통과해야 한다.",
            "실패하면 `cargo fmt --all` 후 다시 --check.",
        ],
        [
            HARD_GATE,
        ],
        [
            "종료 코드 0",
            "PR 본문 첫 칸을 [x] 로 표시할 수 있다",
        ],
    ),
    (
        "10_fmt_stale_check_rejected.md",
        "10 — 낡은 cargo fmt --check 는 게이트가 아니다",
        "단 5 함정. 그 명령만 돌리고 통과했다고 쓰지 마라.",
        "fmt-hard-gate.md",
        "fixtures/envelopes/fmt_stale_check_only.json",
        [
            f"`{STALE_FMT}` 는 낡은 표기다.",
            "CI Lint Format check 는 `--all -- --check` 다.",
            "에이전트가 낡은 명령을 게이트로 보고하면 거절한다.",
        ],
        [
            HARD_GATE,
        ],
        [
            "본문에 낡은 명령만 있지 않다",
            "HARD GATE 종료 코드 0",
        ],
    ),
    (
        "11_clippy_deny_warnings.md",
        "11 — clippy -D warnings",
        "단 5. 경고 한 건도 실패다.",
        "clippy-and-tests.md",
        "fixtures/envelopes/clippy_pass.json",
        [
            "`cargo clippy -- -D warnings` 를 fmt 다음에 실행한다.",
            "같은 checkout 에서 cargo 를 동시에 돌리지 않는다.",
            "경고를 allow 로 숨기지 않는다.",
        ],
        [
            CLIPPY,
        ],
        [
            "종료 코드 0",
            "PR 본문에 명령을 적었다",
        ],
    ),
    (
        "12_related_cargo_test.md",
        "12 — 관련 cargo test",
        "단 5. 이 스킬 계약을 이름으로 실행한다.",
        "clippy-and-tests.md",
        "fixtures/envelopes/test_related_pass.json",
        [
            "Rust: `cargo test --test agent_contributor_skill_contract`.",
            "Python: `python -m unittest scripts.tests.test_agent_contributor`.",
            "본문에 실행한 시험 이름을 적는다.",
        ],
        [
            "cargo test --test agent_contributor_skill_contract -- --nocapture",
            "python -m unittest scripts.tests.test_agent_contributor",
        ],
        [
            "관련 시험이 통과했다",
            "본문에 명령이 있다",
        ],
    ),
    (
        "13_visual_evidence_render.md",
        "13 — 렌더/레이아웃 시각 근거",
        "단 5. 레이아웃을 바꾸면 SVG/PDF 전후를 남긴다. 이 파동은 N/A.",
        "visual-evidence.md",
        "fixtures/envelopes/visual_required.json",
        [
            "렌더 변경이면 공개 샘플로 export-svg 전후를 남긴다.",
            "한컴 PDF 는 정답지가 아니다. 환경을 같이 적는다.",
            "스킬 문서만 만지면 시각 근거는 N/A 라고 본문에 적는다.",
        ],
        [
            "rhwp export-svg samples/basic/blank.hwp -o /tmp/after.svg",
        ],
        [
            "필요하면 전후 파일이 있다",
            "불필요하면 N/A 사유가 있다",
        ],
    ),
    (
        "14_work_receipt_capsule.md",
        "14 — 영수증은 포인터만",
        "단 6. rhwp-work-receipt 를 다시 쓰지 않는다.",
        "work-receipt-pointers.md",
        "fixtures/envelopes/receipt_pointer.json",
        [
            "`rhwp replay --capsule` / `audit` / `lineage` 를 가리킨다.",
            "`.claude/skills/rhwp-work-receipt/` 본문을 수정하지 않는다.",
            "새 `receipt` 명령을 발명하지 않는다.",
        ],
        [
            "rhwp replay --plan-json <계획> --capsule work.capsule.json --json",
            "rhwp audit <폴더> --json",
            "rhwp lineage <머리캡슐> --json",
        ],
        [
            "포인터 명령을 안내했다",
            "영수증 스킬 파일을 고치지 않았다",
        ],
    ),
    (
        "15_working_doc.md",
        "15 — 처리 결과 문서",
        "단 7. mydocs/working/agent_contributor.md 에 실측을 남긴다.",
        "working-doc.md",
        "fixtures/checklists/step_07_working.json",
        [
            "이슈 번호, 브랜치, 만진 경로, 시험, fmt 게이트를 적는다.",
            "mydocs/pr/ 메인터너 기록은 만들지 않는다.",
        ],
        [
            "git add -- mydocs/working/agent_contributor.md",
        ],
        [
            "working 문서가 이슈 번호를 인용한다",
            "HARD GATE 명령이 본문에 있다",
        ],
    ),
    (
        "16_korean_pr_body_file.md",
        "16 — 한국어 PR, --body-file",
        "단 8. PowerShell 에서 한글을 파이프하지 않는다. UTF-8 without BOM.",
        "korean-pr.md",
        "fixtures/pr-bodies/closes_5322.md",
        [
            "제목·본문은 한국어.",
            "`--base devel` 과 `--body-file`.",
            "본문에 `closes #5322`.",
        ],
        [
            "gh pr create --repo edwardkim/rhwp --base devel --head kevin9327:feat/agent-contributor --title \"agent: 기여 절차(contributor) 스킬 고도화\" --body-file pr_body.md",
        ],
        [
            "body-file 을 썼다",
            "closes #5322 가 있다",
            "첫 칸이 fmt 게이트다",
        ],
    ),
    (
        "17_pr_first_checkbox_fmt.md",
        "17 — PR 템플릿 첫 체크박스 = fmt 게이트",
        "단 8. 첫 칸이 cargo fmt --all -- --check 다.",
        "pr-template-checkboxes.md",
        "fixtures/checklists/step_08_pr.json",
        [
            "본문 테스트 절의 첫 `- [x]` 가 HARD GATE 다.",
            "`cargo fmt --check` 칸을 복사하지 않는다.",
            "열린 PR 이 템플릿 파일을 고치고 있으면 그 파일을 가로채지 않는다.",
        ],
        [
            HARD_GATE,
        ],
        [
            "첫 칸 텍스트에 --all -- --check 가 있다",
            "낡은 --check 만 있는 칸이 없다",
        ],
    ),
    (
        "18_sparse_missing_crates.md",
        "18 — 스파스 체크아웃에 crates/ 없음",
        "예외. crates/ 를 꺼낸 뒤 HARD GATE 를 다시 돌린다.",
        "exceptions.md",
        "fixtures/layouts/sparse-missing-crates/README.md",
        [
            "`Test-Path crates` 가 false 면 fmt --all 이 부분 검사일 수 있다.",
            "`git sparse-checkout add crates` 로 꺼낸다.",
            "crates/ 가 있으면 HARD GATE 는 반드시 통과해야 한다.",
        ],
        [
            "git sparse-checkout add crates",
            HARD_GATE,
        ],
        [
            "crates/ 가 있다",
            "HARD GATE 종료 코드 0",
        ],
    ),
    (
        "19_windows_autocrlf_unix.md",
        "19 — Windows autocrlf vs rustfmt Unix",
        "예외. newline_style=Unix. CRLF 가 들어가면 게이트 실패.",
        "rustfmt-unix.md",
        "fixtures/envelopes/fmt_fail_crlf.json",
        [
            "`rustfmt.toml` 의 `newline_style = Unix`.",
            "`core.autocrlf=true` 면 로컬에서 false 로 끈다.",
            "생성기는 `newline=\"\\n\"` 으로 저장한다.",
        ],
        [
            "git config --local core.autocrlf false",
            "git config --local core.eol lf",
            HARD_GATE,
        ],
        [
            "autocrlf 가 false 다",
            "HARD GATE 가 줄끝 때문에 실패하지 않는다",
        ],
    ),
    (
        "20_ci_noci_vs_failure.md",
        "20 — CI noci 와 FAILURE 를 섞지 않는다",
        "예외. 검사가 안 뜬 것과 빨간 검사는 다르다.",
        "exceptions.md",
        "fixtures/envelopes/ci_noci.json",
        [
            "noci: 문서 전용 paths-ignore, required check 미발행.",
            "FAILURE: Lint/Build 가 실제로 빨갛다. 고친다.",
            "이 파동은 스킬+시험이 있어 noci 대상이 아니다.",
        ],
        [
            "gh pr checks <번호>",
        ],
        [
            "없는 검사를 실패로 보고하지 않았다",
            "빨간 검사를 noci 로 무시하지 않았다",
        ],
    ),
    (
        "21_closes_issue.md",
        "21 — closes #이슈",
        "단 8. PR 본문이 이슈를 닫는다.",
        "korean-pr.md",
        "fixtures/pr-bodies/closes_5322.md",
        [
            "본문에 `closes #5322` 한 줄이 있다.",
            "다른 이슈 번호를 꾸며내지 않는다.",
        ],
        [
            "gh pr view --json body",
        ],
        [
            "body 에 closes #5322 가 있다",
        ],
    ),
    (
        "22_no_new_cli.md",
        "22 — 새 rhwp CLI 명령을 만들지 않는다",
        "단 4. contribute/pr-gate 같은 명령을 추가하지 않는다.",
        "implement-scope.md",
        "fixtures/envelopes/new_cli_rejected.json",
        [
            "Cargo.toml `[[bin]]` 개수는 그대로다.",
            "안내하는 명령은 git/gh/cargo/기존 rhwp 뿐이다.",
        ],
        [
            "git diff upstream/devel -- Cargo.toml",
        ],
        [
            "새 [[bin]] 이 없다",
            "SKILL.md 가 새 CLI 를 만들지 않는다고 밝힌다",
        ],
    ),
    (
        "23_named_file_stage.md",
        "23 — 이름을 댄 파일만 stage",
        "단 4. cached diff 가 이슈 범위다.",
        "staging-named-files.md",
        "fixtures/transcripts/stage_named.json",
        [
            "경로를 나열해 `git add -- <path>` 한다.",
            "cached 목록에 gym/ · document_core · 다른 스킬이 없어야 한다.",
        ],
        [
            "git add -- .claude/skills/rhwp-contributor/",
            "git add -- mydocs/working/agent_contributor.md",
            "git add -- tests/cases/agent_contributor_skill_contract.rs",
            "git add -- scripts/tests/test_agent_contributor.py",
            "git diff --cached --name-only",
        ],
        [
            "cached 가 이슈 범위다",
        ],
    ),
    (
        "24_full_procedure_walkthrough.md",
        "24 — 8단 전 구간 워크스루",
        "단 1–8 을 한 번에 닫는 실기여 레시피. gym 아님.",
        "procedure-order.md",
        "fixtures/checklists/step_01_issue.json",
        [
            "이슈 #5322 확인 → 분석 → fetch → isolation worktree.",
            "구현 → named add → HARD GATE → clippy → 관련 test.",
            "영수증 포인터 → working 문서 → 한국어 PR closes #5322.",
        ],
        [
            "git fetch upstream devel",
            HARD_GATE,
            CLIPPY,
            "cargo test --test agent_contributor_skill_contract -- --nocapture",
            "python -m unittest scripts.tests.test_agent_contributor",
            "gh pr create --base devel --body-file pr_body.md",
        ],
        [
            "8단이 모두 닫혔다",
            "HARD GATE 가 0 이다",
            "PR URL 이 있다",
        ],
    ),
]


def example_markdown(spec) -> str:
    name, title, lead, ref, fixture, steps, commands, checks = spec
    cmd_block = "\n".join(commands)
    step_lines = "\n".join(f"{i}. {s}" for i, s in enumerate(steps, 1))
    check_lines = "\n".join(f"- [ ] {c}" for c in checks)
    return f"""# {title}

{lead}

권위: [references/{ref}](../references/{ref}).
픽스처: [{fixture}](../{fixture}).

## 0. 하지 않는 것

- `{STALE_FMT}` 를 게이트로 쓰지 않는다. 정본은 `{HARD_GATE}`.
- `git add -A` 를 쓰지 않는다.
- named worktree 를 훔치지 않는다.
- DocumentCore 편집 로직을 발명하지 않는다.
- 새 rhwp CLI 를 만들지 않는다.
- `gym/` 과 `rhwp-work-receipt` 본문을 고치지 않는다.

## 1. 절차

{step_lines}

## 2. 명령

```bash
{cmd_block}
```

## 3. 체크리스트

{check_lines}

## 4. 실측 메모

- 이슈는 #{ISSUE} (`agent: 기여 절차(contributor) 스킬 고도화`).
- 브랜치는 `{BRANCH}`, remotes 는 origin=kevin9327 fork, upstream=edwardkim/rhwp.
- HARD GATE 실패 시 `gh pr create` 를 호출하지 않는다.
- `crates/` 가 이 워크트리에 있으면 `{HARD_GATE}` 는 반드시 0 이어야 한다.
- rustfmt `newline_style=Unix`. Windows 에서 CRLF 가 섞이면 게이트가 실패한다.
- 작업 영수증이 필요하면 `.claude/skills/rhwp-work-receipt/` 를 읽고
  `rhwp replay --capsule` / `rhwp audit` / `rhwp lineage` 만 호출한다.
- 이 레시피는 gym 과제가 아니다. Maker seat 로 실 PR 을 닫는다.

## 5. 다음

레시피 색인 [references/recipe-index.md](../references/recipe-index.md).
전체 순서는 [24_full_procedure_walkthrough.md](24_full_procedure_walkthrough.md).
이슈 #{ISSUE}. 브랜치 `{BRANCH}`.
"""


def write_examples() -> list[str]:
    names = []
    rows = []
    for spec in EXAMPLES_SPEC:
        name = spec[0]
        write_text(EXAMPLES / name, example_markdown(spec))
        names.append(name)
        rows.append(f"| [{name}]({name}) | {spec[1].split('—', 1)[-1].strip()} |")
    readme = f"""# rhwp-contributor 워크스루

실기여 에이전트가 8단을 닫는 레시피. gym 아님. 새 CLI 없음.

정본 게이트: `{HARD_GATE}`.
낡은 표기 `{STALE_FMT}` 는 거부.

| 파일 | 한 줄 |
|------|------|
{chr(10).join(rows)}

생성기: [`../references/_gen_pack.py`](../references/_gen_pack.py).
색인: [`../references/recipe-index.md`](../references/recipe-index.md).
"""
    write_text(EXAMPLES / "README.md", readme)
    names.append("README.md")
    return names


def write_checklists() -> list[str]:
    names = []
    details = {
        "issue": [
            "gh issue list / gh pr list --state open",
            "이슈 번호 확보 (있으면 재사용)",
            "DoD 에 HARD GATE 기록",
        ],
        "analyze": [
            "AGENTS.md 로딩 순서",
            "CONTRIBUTING.md fmt 절",
            "local_validation.md §4.3",
            "DocumentCore 비범위 명시",
        ],
        "branch": [
            "git fetch upstream devel",
            "isolation worktree",
            "금지 경로 회피",
            "named 미훔침",
        ],
        "implement": [
            "이슈 범위 파일만",
            "git add -- <path>",
            "새 CLI 없음",
            "DocumentCore 미발명",
        ],
        "gate": [
            HARD_GATE,
            CLIPPY,
            "관련 cargo test",
            "시각 근거 또는 N/A",
            "crates/ 있으면 fmt 필수 통과",
        ],
        "receipt": [
            "rhwp replay --capsule 포인터",
            "audit / lineage 포인터",
            "work-receipt 스킬 미수정",
        ],
        "working": [
            "mydocs/working/agent_contributor.md",
            "이슈·브랜치·시험·게이트 기록",
        ],
        "pr": [
            "--base devel",
            "--body-file UTF-8 without BOM",
            "closes #5322",
            "첫 체크박스 = fmt 게이트",
            "한국어 제목·본문",
        ],
    }
    for idx, (key, title, evidence) in enumerate(STEPS, 1):
        name = f"step_{idx:02d}_{key}.json"
        dump(
            FIX / "checklists" / name,
            {
                "id": f"step-{idx:02d}-{key}",
                "step": idx,
                "title": title,
                "evidence": evidence,
                "items": details[key],
                "hardGate": HARD_GATE,
                "forbids": [
                    STALE_FMT,
                    "git add -A",
                    "steal named worktree",
                    "invent DocumentCore edit",
                    "new rhwp CLI",
                    "edit gym/",
                    "rewrite rhwp-work-receipt",
                ],
                "issue": ISSUE,
            },
        )
        names.append(name)
    return names


def write_envelopes() -> list[str]:
    specs = [
        (
            "issue_created.json",
            "gh",
            0,
            "ok",
            {"issue": ISSUE, "created": False, "reused": True, "title": "agent: 기여 절차(contributor) 스킬 고도화"},
        ),
        (
            "duplicate_open_pr.json",
            "gh",
            3,
            "duplicate",
            {"openPrs": 1, "action": "do-not-create", "search": "feat/agent-contributor"},
        ),
        (
            "worktree_created.json",
            "git",
            0,
            "ok",
            {"path": r"C:\\Users\\swsz9\\rhwp-agent-contributor", "stolen": False, "branch": BRANCH},
        ),
        (
            "git_add_a_rejected.json",
            "git",
            2,
            "usage",
            {"command": "git add -A", "rejected": True, "reason": "named files only"},
        ),
        (
            "fmt_pass.json",
            "cargo",
            0,
            "ok",
            {"command": HARD_GATE, "cratesPresent": True, "newlineStyle": "Unix"},
        ),
        (
            "fmt_stale_check_only.json",
            "cargo",
            2,
            "stale",
            {"command": STALE_FMT, "acceptedAsGate": False, "mustUse": HARD_GATE},
        ),
        (
            "fmt_fail_crlf.json",
            "cargo",
            1,
            "crlf",
            {"command": HARD_GATE, "reason": "CRLF vs newline_style=Unix", "autocrlf": True},
        ),
        (
            "clippy_pass.json",
            "cargo",
            0,
            "ok",
            {"command": CLIPPY, "denyWarnings": True},
        ),
        (
            "test_related_pass.json",
            "cargo",
            0,
            "ok",
            {
                "commands": [
                    "cargo test --test agent_contributor_skill_contract",
                    "python -m unittest scripts.tests.test_agent_contributor",
                ]
            },
        ),
        (
            "visual_required.json",
            "rhwp",
            0,
            "layout",
            {"needed": True, "kind": "svg-before-after", "hancomIsOracle": False},
        ),
        (
            "visual_na.json",
            "rhwp",
            0,
            "ok",
            {"needed": False, "reason": "skill docs and contract tests only"},
        ),
        (
            "receipt_pointer.json",
            "replay",
            0,
            "ok",
            {
                "pointers": ["replay --capsule", "audit", "lineage"],
                "rewritesWorkReceiptSkill": False,
            },
        ),
        (
            "ci_noci.json",
            "gh",
            0,
            "noci",
            {"classification": "noci", "requiredChecksMissing": True, "isFailure": False},
        ),
        (
            "ci_failure.json",
            "gh",
            3,
            "failure",
            {"classification": "FAILURE", "check": "Lint", "isNoci": False},
        ),
        (
            "new_cli_rejected.json",
            "git",
            2,
            "usage",
            {"invented": ["rhwp contribute", "rhwp pr-gate"], "rejected": True},
        ),
        (
            "sparse_crates_missing.json",
            "git",
            1,
            "sparse",
            {"cratesPresent": False, "next": "git sparse-checkout add crates"},
        ),
        (
            "pr_created.json",
            "gh",
            0,
            "ok",
            {
                "base": "devel",
                "bodyFile": True,
                "closes": ISSUE,
                "firstCheckbox": HARD_GATE,
                "titleKo": True,
            },
        ),
        (
            "named_worktree_stolen.json",
            "git",
            2,
            "stolen",
            {"path": r"C:\\Users\\swsz9\\rhwp-desk", "rejected": True},
        ),
    ]
    names = []
    for name, command, code, branch, body in specs:
        env = {**body, **meta(command, code, branch)}
        dump(FIX / "envelopes" / name, env)
        names.append(name)
    return names


def write_transcripts() -> list[str]:
    items = [
        (
            "fetch_devel.json",
            [
                "git fetch upstream devel",
                "git rev-parse upstream/devel",
            ],
        ),
        (
            "worktree_add.json",
            [
                "git worktree list",
                "git worktree add -b feat/agent-contributor C:\\Users\\swsz9\\rhwp-agent-contributor upstream/devel",
            ],
        ),
        (
            "stage_named.json",
            [
                "git add -- .claude/skills/rhwp-contributor/",
                "git add -- mydocs/working/agent_contributor.md",
                "git add -- tests/cases/agent_contributor_skill_contract.rs",
                "git add -- scripts/tests/test_agent_contributor.py",
                "git diff --cached --name-only",
            ],
        ),
        (
            "fmt_gate.json",
            [HARD_GATE],
        ),
        (
            "clippy.json",
            [CLIPPY],
        ),
        (
            "related_tests.json",
            [
                "cargo test --test agent_contributor_skill_contract -- --nocapture",
                "python -m unittest scripts.tests.test_agent_contributor",
            ],
        ),
        (
            "receipt_pointer.json",
            [
                "rhwp replay --plan-json <계획> --capsule work.capsule.json --json",
                "rhwp audit <폴더> --json",
                "rhwp lineage <머리캡슐> --json",
            ],
        ),
        (
            "pr_create.json",
            [
                "git push -u origin HEAD",
                "gh pr create --repo edwardkim/rhwp --base devel --head kevin9327:feat/agent-contributor --title \"agent: 기여 절차(contributor) 스킬 고도화\" --body-file pr_body.md",
            ],
        ),
        (
            "sparse_add_crates.json",
            [
                "git sparse-checkout add crates",
                HARD_GATE,
            ],
        ),
        (
            "autocrlf_fix.json",
            [
                "git config --local core.autocrlf false",
                "git config --local core.eol lf",
                HARD_GATE,
            ],
        ),
        (
            "duplicate_pr_search.json",
            [
                'gh pr list --repo edwardkim/rhwp --search "feat/agent-contributor" --state open',
            ],
        ),
        (
            "ci_checks.json",
            [
                "gh pr checks",
            ],
        ),
    ]
    names = []
    for name, argv in items:
        dump(
            FIX / "transcripts" / name,
            {
                "id": name.replace(".json", ""),
                "issue": ISSUE,
                "argv": argv,
                "forbids": ["git add -A", STALE_FMT, "gh pr create --base main"],
                "hardGate": HARD_GATE,
            },
        )
        names.append(name)
    return names


SCENARIO_SEEDS = [
    ("issue-reuse", "issue", "gh", "이미 #5322 가 있으면 새 이슈를 만들지 않는다"),
    ("issue-search-dup-pr", "issue", "gh", "열린 PR 검색 없이 구현을 시작하지 않는다"),
    ("analyze-agents", "analyze", "read", "AGENTS.md 로딩 순서를 건너뛰지 않는다"),
    ("analyze-contributing-fmt", "analyze", "read", "CONTRIBUTING 의 fmt --all -- --check 를 인용한다"),
    ("analyze-local-validation", "analyze", "read", "§4.3 범위별 검증표를 읽는다"),
    ("analyze-no-doccore-design", "analyze", "read", "DocumentCore 새 연산을 설계하지 않는다"),
    ("fetch-first", "branch", "git", "fetch 없이 로컬 devel 에서 나누지 않는다"),
    ("base-devel", "branch", "git", "base 는 devel 이지 main 이 아니다"),
    ("isolation-new-path", "branch", "git", "빈 경로에 worktree add 한다"),
    ("no-steal-rhwp", "branch", "git", r"C:\Users\swsz9\rhwp 본진에서 구현하지 않는다"),
    ("no-steal-desk", "branch", "git", "rhwp-desk* 를 checkout 하지 않는다"),
    ("no-steal-handoff", "branch", "git", "rhwp-handoff 를 쓰지 않는다"),
    ("no-steal-scaffold", "branch", "git", "rhwp-scaffold-final 을 쓰지 않는다"),
    ("no-steal-doc-repro", "branch", "git", "rhwp-doc-repro 를 쓰지 않는다"),
    ("no-steal-listed", "branch", "git", "worktree list 의 기존 이름을 훔치지 않는다"),
    ("scope-skill-only", "implement", "git", "이 파동은 contributor 스킬과 시험만"),
    ("no-gym", "implement", "git", "gym/ 을 편집하지 않는다"),
    ("no-other-skills", "implement", "git", "다른 스킬 SKILL.md 를 고치지 않는다"),
    ("no-open-pr-files", "implement", "git", "열린 PR 전용 파일을 가로채지 않는다"),
    ("no-new-cli", "implement", "git", "새 rhwp 하위명령을 추가하지 않는다"),
    ("no-doccore-write", "implement", "git", "document_core 편집 로직을 발명하지 않는다"),
    ("named-add", "implement", "git", "git add -- <path> 만"),
    ("reject-add-a", "implement", "git", "git add -A 거부"),
    ("reject-add-dot", "implement", "git", "git add . 거부"),
    ("fmt-hard-gate", "gate", "cargo", HARD_GATE),
    ("fmt-stale-reject", "gate", "cargo", f"{STALE_FMT} 는 게이트가 아니다"),
    ("fmt-then-apply", "gate", "cargo", "실패 시 cargo fmt --all 후 다시 check"),
    ("fmt-crates-present", "gate", "cargo", "crates/ 가 있으면 반드시 통과"),
    ("fmt-unix-nl", "gate", "cargo", "newline_style=Unix"),
    ("clippy-deny", "gate", "cargo", CLIPPY),
    ("related-rust-test", "gate", "cargo", "agent_contributor_skill_contract"),
    ("related-python-test", "gate", "python", "scripts.tests.test_agent_contributor"),
    ("manifest-generate", "gate", "node", "cases 추가 시 rust-test-suite-manifest --generate"),
    ("visual-when-layout", "gate", "rhwp", "레이아웃 변경은 SVG 전후"),
    ("visual-na-docs", "gate", "read", "스킬만 바꾸면 시각 N/A"),
    ("receipt-pointer-replay", "receipt", "replay", "rhwp replay --capsule"),
    ("receipt-pointer-audit", "receipt", "audit", "rhwp audit --json"),
    ("receipt-pointer-lineage", "receipt", "lineage", "rhwp lineage --json"),
    ("receipt-no-rewrite", "receipt", "read", "work-receipt 스킬을 재작성하지 않는다"),
    ("working-doc-issue", "working", "read", "working 문서에 #5322"),
    ("working-doc-gate", "working", "read", "working 문서에 HARD GATE"),
    ("pr-body-file", "pr", "gh", "--body-file UTF-8 without BOM"),
    ("pr-korean", "pr", "gh", "제목·본문 한국어"),
    ("pr-closes", "pr", "gh", "closes #5322"),
    ("pr-first-checkbox", "pr", "gh", "첫 칸 = fmt 게이트"),
    ("pr-after-fmt", "pr", "gh", "fmt 실패면 pr create 금지"),
    ("sparse-add-crates", "exception", "git", "sparse-checkout add crates"),
    ("autocrlf-false", "exception", "git", "core.autocrlf=false"),
    ("noci-not-failure", "exception", "gh", "noci ≠ FAILURE"),
    ("failure-not-noci", "exception", "gh", "빨간 Lint 를 문서로 무시하지 않는다"),
    ("dup-pr-stop", "exception", "gh", "열린 중복 PR 이면 중지"),
]


def expand_scenarios() -> list[dict]:
    out = []
    for i, (sid, step, command, summary) in enumerate(SCENARIO_SEEDS, 1):
        out.append(
            {
                "id": f"SC{i:03d}-{sid}",
                "step": step,
                "command": command,
                "summary": summary,
                "hardGate": HARD_GATE,
                "staleFmtRejected": True,
                "issue": ISSUE,
            }
        )
    extras = [
        "이슈 본문에 금지 목록을 적지 않고 구현을 시작한다",
        "분석 없이 형제 PR 파일을 복사한다",
        "origin/devel 을 upstream/devel 로 착각한다",
        "worktree add 대상이 이미 존재하는데 덮어쓴다",
        "sparse 본진 설정을 새 워크트리에 남긴 채 crates 없이 fmt 했다고 쓴다",
        "PowerShell 에서 gh --body-file - 로 한글을 파이프한다",
        "PR 템플릿의 cargo test 칸을 첫 칸으로 두고 fmt 를 빼먹는다",
        "시각 근거 없이 페이지 수 핀을 바꾼다",
        "관련 시험 이름을 본문에 적지 않는다",
        "clippy 경고를 allow 로 숨긴다",
        "fmt 와 clippy 를 같은 checkout 에서 동시에 돌린다",
        "영수증을 남긴다고 새 prove 명령을 스케치한다",
        "lineage 계약을 이 스킬 레퍼런스에 복제한다",
        "gym pack 으로 기여 절차를 채점한다",
        "메인터너 mydocs/pr 기록을 기여자가 작성한다",
        "closes 없이 PR 을 연다",
        "base=main 으로 연다",
        "fork head 없이 --head 를 생략한다",
        "BOM 이 있는 body-file 을 올린다",
        "CI noci 안내를 소스 변경 PR 에 적용한다",
        "FAILURE 로그를 읽지 않고 재실행만 반복한다",
        "다른 기여자 커밋을 rebase 로 지운다",
        "리뷰 판단을 에이전트가 대신 승인한다",
        "미병합 브랜치 명령을 정본처럼 인용한다",
        "rustfmt.toml 의 newline_style 을 이 파동에서 바꾼다",
        "toolchain 을 이 파동에서 바꾼다",
        "Cargo.toml [[bin]] 을 추가한다",
        "tests/*.rs 최상위에 원본을 새로 둔다",
        "generated suite 를 수기 수정한다",
        "문서만 바꿨는데 crates 없는 상태를 숨긴다",
    ]
    start = len(out) + 1
    for j, summary in enumerate(extras):
        out.append(
            {
                "id": f"SC{start + j:03d}-pitfall-{j + 1:02d}",
                "step": "pitfall",
                "command": "read",
                "summary": summary,
                "rejected": True,
                "hardGate": HARD_GATE,
                "issue": ISSUE,
            }
        )
    assert len(out) >= 80, len(out)
    return out


def write_layouts() -> None:
    dump(
        FIX / "layouts" / "forbidden-worktrees" / "registry.json",
        {
            "skill": SKILL,
            "issue": ISSUE,
            "forbidden": FORBIDDEN_WORKTREES,
            "forbiddenGlobs": [r"C:\\Users\\swsz9\\rhwp-desk*"],
            "rule": "never steal named worktrees",
            "allowedExample": r"C:\\Users\\swsz9\\rhwp-agent-contributor",
        },
    )
    write_text(
        FIX / "layouts" / "forbidden-worktrees" / "README.md",
        """# 금지 워크트리

본진과 이미 이름이 있는 워크트리를 작업 디렉터리로 쓰지 않는다.

- C:\\Users\\swsz9\\rhwp
- C:\\Users\\swsz9\\rhwp-desk*
- C:\\Users\\swsz9\\rhwp-handoff
- C:\\Users\\swsz9\\rhwp-scaffold-final
- C:\\Users\\swsz9\\rhwp-doc-repro
- git worktree list 의 모든 기존 경로

허용 예: C:\\Users\\swsz9\\rhwp-agent-contributor (비어 있을 때만).
""",
    )
    write_text(
        FIX / "layouts" / "sparse-missing-crates" / "README.md",
        """# 스파스 체크아웃에 crates/ 가 없는 레이아웃

본진 sparse 규칙이 새 워크트리에 상속되면 `crates/` 가 빠질 수 있다.

닫는 명령:

```
git sparse-checkout add crates
cargo fmt --all -- --check
```

`crates/` 가 생긴 뒤에는 HARD GATE 가 반드시 통과해야 한다.
이 폴더는 그 상태를 설명하는 픽스처이며 실제 crate 소스를 복제하지 않는다.
""",
    )
    write_text(
        FIX / "layouts" / "sparse-missing-crates" / "marker.txt",
        "crates_present=false\nnext=git sparse-checkout add crates\nhard_gate=cargo fmt --all -- --check\n",
    )


def write_pr_bodies() -> list[str]:
    body = f"""## 변경 요약

실사용 에이전트가 rhwp 기여를 공식 절차대로 완주하도록
`.claude/skills/rhwp-contributor/` 를 레시피·예외·픽스처·계약 시험으로 고도화한다.
HARD GATE 는 `{HARD_GATE}` 다. `{STALE_FMT}` 는 낡은 표기다.

## 관련 이슈

closes #{ISSUE}

## 테스트

- [x] `{HARD_GATE}` 통과 (PR 생성·push 직전 필수. `{STALE_FMT}` 만으로는 부족)
- [x] `{CLIPPY}` (관련 범위)
- [x] 관련 `cargo test --test agent_contributor_skill_contract`
- [x] `python -m unittest scripts.tests.test_agent_contributor`
- [x] 시각 근거: N/A (렌더/레이아웃 변경 없음)
- [ ] 작업 증빙 `rhwp replay --capsule` (문서 편집이 있으면 권장)

## 성능 영향 및 측정 결과

- 예상 영향: 영향 없음
- 재현·측정: 미측정 (스킬·문서·계약 시험)
"""
    write_text(FIX / "pr-bodies" / "closes_5322.md", body)
    write_text(
        FIX / "pr-bodies" / "first_checkbox_fmt.md",
        f"""# 첫 체크박스 표본

- [x] `{HARD_GATE}` 통과
- [ ] `{STALE_FMT}`  ← 이 칸을 게이트로 두지 않는다

첫 칸만 인정한다. 두 번째 낡은 칸은 거절 표본이다.
""",
    )
    return ["closes_5322.md", "first_checkbox_fmt.md"]


def write_commits() -> None:
    write_text(
        FIX / "commits" / "korean_message.txt",
        "feat(agent): 기여 절차(contributor) 스킬을 공식 8단과 fmt 관문으로 고도화한다\n",
    )


def write_gate_matrix() -> None:
    dump(
        FIX / "gate-matrix.json",
        {
            "hardGate": HARD_GATE,
            "staleRejected": [STALE_FMT, "cargo fmt -- --check"],
            "clippy": CLIPPY,
            "tests": [
                "cargo test --test agent_contributor_skill_contract",
                "python -m unittest scripts.tests.test_agent_contributor",
            ],
            "rustfmt": {"newline_style": "Unix"},
            "cratesPresentImpliesFmtMustPass": True,
            "firstPrCheckbox": HARD_GATE,
        },
    )


def write_scenario_cards(scenarios: list[dict]) -> None:
    root = FIX / "scenario-cards"
    for sc in scenarios:
        sid = sc["id"]
        body = {
            **sc,
            "hardGate": HARD_GATE,
            "staleFmt": STALE_FMT,
            "never": [
                "git add -A",
                "steal named worktree",
                "invent DocumentCore edit logic",
                "new rhwp CLI",
                "edit gym/",
                "rewrite rhwp-work-receipt",
                "base main",
            ],
            "closeWith": {
                "issue": "gh issue view / do not invent a second issue",
                "branch": "git fetch upstream devel + isolation worktree",
                "gate": HARD_GATE,
                "pr": f"gh pr create --base devel --body-file (closes #{ISSUE})",
            },
            "notes": [
                f"시나리오 {sid} 는 기여 절차 계약의 한 칸이다.",
                "에이전트는 이 카드를 읽고 해당 단을 닫는 명령을 실행한다.",
                "명령이 helperCommands(replay/audit/lineage) 이면 포인터만 따른다.",
                "noci 와 FAILURE 를 같은 상태로 기록하지 않는다.",
                "crates/ 가 보이면 fmt 게이트를 생략하지 않는다.",
            ],
        }
        dump(root / f"{sid}.json", body)
    dump(
        root / "index.json",
        {
            "skill": SKILL,
            "issue": ISSUE,
            "count": len(scenarios),
            "hardGate": HARD_GATE,
            "ids": [sc["id"] for sc in scenarios],
        },
    )


def write_catalog(example_names: list[str], envelopes: list[str], transcripts: list[str]) -> None:
    refs = [
        "README.md",
        "procedure-order.md",
        "issue-first.md",
        "analyze-canonical.md",
        "branch-isolation.md",
        "isolation-worktree.md",
        "implement-scope.md",
        "staging-named-files.md",
        "fmt-hard-gate.md",
        "rustfmt-unix.md",
        "clippy-and-tests.md",
        "visual-evidence.md",
        "work-receipt-pointers.md",
        "working-doc.md",
        "korean-pr.md",
        "pr-template-checkboxes.md",
        "exceptions.md",
        "pitfalls.md",
        "decision-tree.md",
        "recipe-index.md",
        "command-field-catalog.md",
    ]
    dump(
        FIX / "catalog.json",
        {
            "catalogVersion": "1.0",
            "skill": SKILL,
            "issue": ISSUE,
            "branch": BRANCH,
            "note": "공식 기여 8단. 새 CLI 없음. gym 없음. DocumentCore 발명 없음.",
            "hardGate": HARD_GATE,
            "staleFmt": STALE_FMT,
            "clippy": CLIPPY,
            "newlineStyle": "Unix",
            "firstPrCheckbox": HARD_GATE,
            "base": "devel",
            "bodyFile": True,
            "closes": ISSUE,
            "neverGitAddA": True,
            "neverStealNamedWorktrees": True,
            "neverInventDocumentCore": True,
            "noNewCli": True,
            "commands": ["git", "gh", "cargo", "python", "node"],
            "helperCommands": ["replay", "audit", "lineage"],
            "forbiddenWorktrees": FORBIDDEN_WORKTREES,
            "references": refs,
            "examples": [n for n in example_names if n != "README.md"],
            "envelopes": envelopes,
            "transcripts": transcripts,
            "checklists": [f"step_{i:02d}_{k}.json" for i, (k, _, _) in enumerate(STEPS, 1)],
        },
    )


def main() -> None:
    example_names = write_examples()
    write_checklists()
    envelopes = write_envelopes()
    transcripts = write_transcripts()
    write_layouts()
    write_pr_bodies()
    write_commits()
    write_gate_matrix()
    scenarios = expand_scenarios()
    dump(
        FIX / "scenario_catalog.json",
        {
            "catalogVersion": "1.0",
            "skill": SKILL,
            "issue": ISSUE,
            "count": len(scenarios),
            "hardGate": HARD_GATE,
            "scenarios": scenarios,
        },
    )
    write_scenario_cards(scenarios)
    write_catalog(example_names, envelopes, transcripts)
    print(
        f"generated examples={len(example_names)} envelopes={len(envelopes)} "
        f"transcripts={len(transcripts)} scenarios={len(scenarios)}"
    )


if __name__ == "__main__":
    main()
