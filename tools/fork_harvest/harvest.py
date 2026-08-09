#!/usr/bin/env python3
"""fork harvest — rhwp 포크 생태계 읽기 전용 수확 하니스.

포크 위의 개선은 upstream 으로 돌아오지 않으면 사라진다. 이 도구는 공개
GitHub 데이터를 **읽기만** 해서 "어느 포크가 upstream 보다 앞서 있고,
무엇을 바꿨으며, 거둘 만한가" 를 보고서로 만든다.

동작 단계:
  1. 발견   — `gh api repos/{repo}/forks` 페이지네이션 전수 열거
  2. 선별   — 활동 필터(생성 이후 push 존재 + 최근 --days 일 이내)
  3. 대조   — 각 후보의 기본 브랜치를 upstream 브랜치와 compare API 로 대조
  4. 분류   — ahead>0 포크의 변경 파일 확장자 기반 분류(code/tests/docs/…)
  5. 보고   — TSV + 마크다운 수확 보고서(우선순위 휴리스틱 포함)

읽기 전용 경계 (절대 불변):
  - GitHub 에 대한 모든 호출은 GET 뿐이다. 포크에 push·이슈·PR·코멘트 등
    어떤 쓰기 작업도 하지 않으며, 그런 코드 경로 자체가 없다.
  - 포크 소유자에 대해서는 로그인명(login) 외 어떤 정보도 수집하지 않는다.
  - 수확 후보의 upstream 반영은 사람이 통상 PR 절차로 진행한다.

기준 브랜치 선택 (--base auto 의 근거):
  upstream(edwardkim/rhwp) 의 기본 브랜치는 main 이고 기여 기준은 devel 인데,
  실측(2026-08-08)상 main 은 devel 대비 ahead 23 / behind 1232 로 발산해 있다.
  따라서 모든 포크를 무조건 devel 과 대조하면 main 을 그대로 포크만 한
  저장소가 전부 ahead=23 으로 오탐된다. `auto` 는 포크 브랜치와 같은 이름의
  브랜치가 upstream 에 있으면 그것을(main↔main, devel↔devel), 없으면
  upstream 기본 브랜치를 기준으로 삼아 이 오탐을 제거한다.

옵트인 비콘 (--beacon):
  포크 루트에 AGENT_WORK.json (스키마: what/why/files/gates/wantsUpstream —
  mydocs/manual/fork_harvest_convention.md 참조) 이 있으면 그 선언을 보고서에
  우선 반영하고 우선순위를 가산한다.

exit 규약:
  0 — 완주(모든 후보 대조 성공)
  1 — 부분 실패 있음(개별 포크 오류 행 존재, 또는 쿼터 보호로 조기 중단;
      보고서에 부분 결과임을 정직하게 표기)
  2 — 구성 오류(gh 없음/미인증, 잘못된 인자, upstream 접근 불가,
      시작 시점 쿼터 부족)

사용 예:
  python tools/fork_harvest/harvest.py --days 180 --limit 120 --beacon
  python tools/fork_harvest/harvest.py --limit 5 --out-dir output/fork_harvest

필요 도구: python 표준 라이브러리 + gh CLI(인증된 상태) 뿐.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import subprocess
import sys
from dataclasses import dataclass, field

# Windows 콘솔(cp949)에서도 유니코드 요약 출력이 죽지 않게 한다.
for _stream in (sys.stdout, sys.stderr):
    if hasattr(_stream, "reconfigure"):
        _stream.reconfigure(encoding="utf-8", errors="replace")

# ---------------------------------------------------------------------------
# gh 호출 (GET 전용)
# ---------------------------------------------------------------------------

GH_TIMEOUT_SEC = 60


class GhError(RuntimeError):
    def __init__(self, path: str, returncode: int, stderr: str):
        super().__init__(f"gh api GET {path} 실패 (exit {returncode}): {stderr.strip()[:300]}")
        self.path = path
        self.returncode = returncode
        self.stderr = stderr


def gh_get(path: str) -> object:
    """`gh api <path>` 를 GET 으로만 호출해 JSON 을 돌려준다.

    읽기 전용 보증: -X/--method 를 절대 지정하지 않으며(gh 기본 = GET),
    -f/-F 필드 입력도 쓰지 않는다. 이 함수가 이 파일의 유일한 GitHub 통로다.
    """
    proc = subprocess.run(
        ["gh", "api", path],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=GH_TIMEOUT_SEC,
    )
    if proc.returncode != 0:
        raise GhError(path, proc.returncode, proc.stderr or "")
    return json.loads(proc.stdout)


def rate_remaining() -> tuple[int, int]:
    """core 쿼터 (잔량, 창 리셋 epoch). rate_limit 엔드포인트는 쿼터를 소모하지 않는다.

    리셋 epoch 를 함께 돌려주는 이유: 실행 중 쿼터 창이 리셋되면 시작-종료 잔량
    델타가 실제 사용량을 과소 보고한다(2026-08-08 첫 회전에서 실측 — 약 190 호출이
    델타 7 로 보였다). 창이 바뀌었으면 보고서에 델타 신뢰 불가를 표기한다.
    """
    data = gh_get("rate_limit")
    core = data["resources"]["core"]
    return int(core["remaining"]), int(core["reset"])


# ---------------------------------------------------------------------------
# 분류·우선순위 휴리스틱
# ---------------------------------------------------------------------------

CODE_EXT = {
    ".rs", ".ts", ".tsx", ".js", ".mjs", ".cjs", ".py", ".swift", ".kt",
    ".java", ".c", ".h", ".cc", ".cpp", ".hpp", ".go", ".sh", ".ps1",
}
DOC_EXT = {".md", ".txt", ".adoc", ".rst"}
CONFIG_EXT = {".toml", ".yml", ".yaml", ".json", ".lock", ".xml", ".cfg", ".ini"}


def classify_files(filenames: list[str]) -> dict[str, int]:
    """변경 파일 목록을 code/tests/docs/config/other 로 집계한다."""
    counts = {"code": 0, "tests": 0, "docs": 0, "config": 0, "other": 0}
    for name in filenames:
        lower = name.lower().replace("\\", "/")
        ext = os.path.splitext(lower)[1]
        is_testpath = (
            "/tests/" in f"/{lower}" or lower.startswith("tests/")
            or "_test." in lower or ".test." in lower or "/fuzz/" in f"/{lower}"
        )
        if is_testpath and (ext in CODE_EXT or ext in CONFIG_EXT):
            counts["tests"] += 1
        elif ext in CODE_EXT:
            counts["code"] += 1
        elif ext in DOC_EXT:
            counts["docs"] += 1
        elif ext in CONFIG_EXT:
            counts["config"] += 1
        else:
            counts["other"] += 1
    return counts


def primary_category(counts: dict[str, int]) -> str:
    if counts["code"] > 0:
        return "code"
    if counts["tests"] > 0:
        return "tests"
    if counts["docs"] > 0:
        return "docs"
    if counts["config"] > 0:
        return "config"
    return "other"


def priority_score(row: "ForkRow", now: dt.datetime) -> float:
    """수확 우선순위 점수. 근거는 fork_harvest_convention.md 에 문서화.

    beacon(wantsUpstream) +3 / 분류 code +2 · tests +1.5 · docs +1 · 기타 +0.5
    ahead 규모 0~2 (20커밋에서 포화) / 최근 push 30일 +1 · 90일 +0.5
    """
    score = 0.0
    if row.beacon and row.beacon.get("wantsUpstream"):
        score += 3.0
    score += {"code": 2.0, "tests": 1.5, "docs": 1.0}.get(row.category, 0.5)
    score += min(row.ahead_by, 20) / 10.0
    pushed = parse_iso(row.pushed_at)
    if pushed is not None:
        age_days = (now - pushed).days
        if age_days <= 30:
            score += 1.0
        elif age_days <= 90:
            score += 0.5
    return round(score, 2)


def priority_label(score: float) -> str:
    if score >= 4.0:
        return "high"
    if score >= 2.5:
        return "mid"
    return "low"


# ---------------------------------------------------------------------------
# 자료 구조
# ---------------------------------------------------------------------------


@dataclass
class ForkRow:
    login: str
    full_name: str
    branch: str
    created_at: str
    pushed_at: str
    base: str = ""
    ahead_by: int = 0
    behind_by: int = 0
    file_counts: dict = field(default_factory=dict)
    total_files: int = 0
    category: str = ""
    recent_commits: list = field(default_factory=list)
    beacon: dict | None = None
    score: float = 0.0
    label: str = ""
    error: str = ""


def parse_iso(value: str) -> dt.datetime | None:
    if not value:
        return None
    try:
        return dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


# ---------------------------------------------------------------------------
# 수집 단계
# ---------------------------------------------------------------------------


def list_forks(repo: str) -> list[dict]:
    """포크 전수 열거(페이지네이션). 소유자 정보는 login 만 유지한다."""
    forks: list[dict] = []
    page = 1
    while True:
        batch = gh_get(f"repos/{repo}/forks?per_page=100&sort=oldest&page={page}")
        if not isinstance(batch, list) or not batch:
            break
        for item in batch:
            forks.append(
                {
                    "login": item.get("owner", {}).get("login", ""),
                    "full_name": item.get("full_name", ""),
                    "default_branch": item.get("default_branch", ""),
                    "created_at": item.get("created_at", ""),
                    "pushed_at": item.get("pushed_at", ""),
                    "archived": bool(item.get("archived", False)),
                }
            )
        if len(batch) < 100:
            break
        page += 1
    return forks


def upstream_branch_names(repo: str) -> set[str]:
    names: set[str] = set()
    page = 1
    while True:
        batch = gh_get(f"repos/{repo}/branches?per_page=100&page={page}")
        if not isinstance(batch, list) or not batch:
            break
        names.update(b["name"] for b in batch if "name" in b)
        if len(batch) < 100:
            break
        page += 1
    return names


def fetch_beacon(full_name: str, branch: str) -> dict | None:
    """포크 루트의 AGENT_WORK.json 을 읽는다(없으면 None — 정상)."""
    import base64

    try:
        data = gh_get(f"repos/{full_name}/contents/AGENT_WORK.json?ref={branch}")
    except GhError:
        return None
    if not isinstance(data, dict) or data.get("encoding") != "base64":
        return None
    try:
        raw = base64.b64decode(data.get("content", "")).decode("utf-8", errors="replace")
        manifest = json.loads(raw)
    except (ValueError, TypeError):
        return {"_malformed": True}
    if not isinstance(manifest, dict):
        return {"_malformed": True}
    keep = {k: manifest.get(k) for k in ("what", "why", "files", "gates", "wantsUpstream") if k in manifest}
    return keep or {"_malformed": True}


# ---------------------------------------------------------------------------
# 출력
# ---------------------------------------------------------------------------

TSV_COLUMNS = [
    "fork", "branch", "base", "ahead", "behind", "category",
    "files_code", "files_tests", "files_docs", "files_config", "files_other",
    "total_files", "recent_commits", "pushed_at", "beacon", "score", "priority",
    "error",
]


def sanitize_cell(value: object) -> str:
    return str(value).replace("\t", " ").replace("\r", " ").replace("\n", " ¶ ")


def write_tsv(path: str, rows: list[ForkRow]) -> None:
    lines = ["\t".join(TSV_COLUMNS)]
    for r in rows:
        fc = r.file_counts or {}
        lines.append(
            "\t".join(
                sanitize_cell(v)
                for v in [
                    r.full_name, r.branch, r.base, r.ahead_by, r.behind_by, r.category,
                    fc.get("code", 0), fc.get("tests", 0), fc.get("docs", 0),
                    fc.get("config", 0), fc.get("other", 0), r.total_files,
                    " | ".join(r.recent_commits),
                    r.pushed_at,
                    json.dumps(r.beacon, ensure_ascii=False) if r.beacon else "",
                    r.score, r.label, r.error,
                ]
            )
        )
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lines) + "\n")


def write_markdown(path: str, args, stats: dict, rows: list[ForkRow], partial_reason: str) -> None:
    now = stats["now"].strftime("%Y-%m-%d %H:%M UTC")
    ahead_rows = [r for r in rows if not r.error and r.ahead_by > 0]
    error_rows = [r for r in rows if r.error]
    cat_dist: dict[str, int] = {}
    for r in ahead_rows:
        cat_dist[r.category] = cat_dist.get(r.category, 0) + 1

    md: list[str] = []
    md.append(f"# 포크 수확 보고서 — {args.repo}")
    md.append("")
    md.append(f"- 실행: {now} / 기준 브랜치: `{args.base}` / 활동 창: 최근 {args.days}일 / limit: {args.limit or '없음'} / beacon: {'on' if args.beacon else 'off'}")
    md.append(f"- 포크 총수 {stats['total_forks']} → 활동 필터 통과 {stats['active_forks']} → 대조 시도 {stats['compared']} → **ahead>0 {len(ahead_rows)}** / 오류 {len(error_rows)}")
    quota_line = f"- API 쿼터: 시작 잔량 {stats['rate_start']} → 종료 잔량 {stats['rate_end']} (델타 {stats['rate_start'] - stats['rate_end']})"
    if stats.get("rate_window_reset"):
        quota_line += " — **실행 중 쿼터 창 리셋 경과: 델타는 실사용량을 과소 보고함**"
    md.append(quota_line)
    if partial_reason:
        md.append(f"- **부분 결과**: {partial_reason}")
    md.append("- 경계: 이 보고서는 공개 데이터의 **읽기 전용** 수집 결과다. 포크에 어떤 쓰기 작업도 하지 않았다.")
    md.append("")
    if cat_dist:
        md.append("## 분류 분포 (ahead>0)")
        md.append("")
        for cat in sorted(cat_dist, key=cat_dist.get, reverse=True):
            md.append(f"- {cat}: {cat_dist[cat]}")
        md.append("")
    md.append("## 수확 후보 (우선순위순)")
    md.append("")
    if ahead_rows:
        md.append("| 우선순위 | 포크 | 브랜치 | 기준 | ahead/behind | 분류 | 파일 | 최근 커밋 | beacon |")
        md.append("|---|---|---|---|---|---|---|---|---|")
        for r in ahead_rows:
            beacon_cell = "—"
            if r.beacon:
                beacon_cell = "말 안 됨" if r.beacon.get("_malformed") else sanitize_cell(r.beacon.get("what", "(선언)"))[:60]
            commits = sanitize_cell("; ".join(r.recent_commits))[:120] or "—"
            md.append(
                f"| {r.label} ({r.score}) | {r.full_name} | {r.branch} | {r.base} "
                f"| +{r.ahead_by}/-{r.behind_by} | {r.category} | {r.total_files} | {commits} | {beacon_cell} |"
            )
    else:
        md.append("(ahead>0 포크 없음)")
    md.append("")
    if error_rows:
        md.append("## 오류 행")
        md.append("")
        for r in error_rows:
            md.append(f"- {r.full_name} ({r.branch}): {sanitize_cell(r.error)[:200]}")
        md.append("")
    md.append("## 한계")
    md.append("")
    md.append("- 각 포크의 **기본 브랜치만** 대조한다. `harvest/*` 등 토픽 브랜치 스캔은 후속.")
    md.append("- compare API 는 파일 300개·커밋 250개에서 절단되므로 대형 발산 포크의 파일 분류는 하한값이다.")
    md.append("- 우선순위는 휴리스틱이며 최종 판단은 사람이 한다.")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(md) + "\n")


# ---------------------------------------------------------------------------
# 메인
# ---------------------------------------------------------------------------


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="harvest.py",
        description="rhwp 포크 생태계 읽기 전용 수확 하니스 (발견→선별→대조→보고)",
        epilog=(
            "읽기 전용 보증: GitHub 호출은 전부 GET 이며 포크에 push·이슈·PR 등 "
            "쓰기 작업을 절대 하지 않는다. 소유자 정보는 로그인명만 수집한다. "
            "exit: 0 완주 / 1 부분 실패 / 2 구성 오류."
        ),
    )
    p.add_argument("--repo", default="edwardkim/rhwp", help="upstream 저장소 (기본: edwardkim/rhwp)")
    p.add_argument("--base", default="auto",
                   help="대조 기준 브랜치. auto=포크 브랜치와 동명 upstream 브랜치 우선, 없으면 upstream 기본 브랜치 (기본: auto)")
    p.add_argument("--days", type=int, default=180, help="활동 창(일). 이 기간 내 push 가 있어야 후보 (기본: 180)")
    p.add_argument("--limit", type=int, default=0, help="대조할 후보 포크 상한 (0=무제한; 후보는 pushed_at 내림차순 정렬 후 절단)")
    p.add_argument("--beacon", action="store_true", help="ahead>0 포크에서 AGENT_WORK.json 옵트인 매니페스트를 조회해 우선 반영")
    p.add_argument("--out-dir", default="output/fork_harvest", help="TSV·마크다운 출력 디렉터리 (기본: output/fork_harvest — gitignore 대상)")
    p.add_argument("--min-remaining", type=int, default=100, help="core 쿼터 잔량이 이 값 밑으로 내려가면 중단하고 부분 결과 표기 (기본: 100)")
    return p


def main(argv: list[str]) -> int:
    args = build_parser().parse_args(argv)
    if args.days <= 0 or args.limit < 0 or args.min_remaining < 0 or "/" not in args.repo:
        print("구성 오류: --days>0, --limit>=0, --min-remaining>=0, --repo 는 owner/name 형식이어야 한다", file=sys.stderr)
        return 2

    now = dt.datetime.now(dt.timezone.utc)

    # -- 시작 전 구성 검증 (실패는 전부 exit 2) ---------------------------
    try:
        rate_start, reset_start = rate_remaining()
        if rate_start < args.min_remaining:
            print(f"구성 오류: 시작 시점 쿼터 잔량 {rate_start} < --min-remaining {args.min_remaining}", file=sys.stderr)
            return 2
        upstream = gh_get(f"repos/{args.repo}")
        upstream_default = upstream.get("default_branch", "main")
        branch_names = upstream_branch_names(args.repo)
        if args.base != "auto" and args.base not in branch_names:
            print(f"구성 오류: upstream 에 브랜치 '{args.base}' 가 없다", file=sys.stderr)
            return 2
    except (GhError, FileNotFoundError, subprocess.TimeoutExpired, json.JSONDecodeError, KeyError) as exc:
        print(f"구성 오류: gh 접근 실패 — {exc}", file=sys.stderr)
        return 2

    # -- 1. 발견 -----------------------------------------------------------
    try:
        forks = list_forks(args.repo)
    except GhError as exc:
        print(f"구성 오류: 포크 열거 실패 — {exc}", file=sys.stderr)
        return 2
    total_forks = len(forks)

    # -- 2. 선별 -----------------------------------------------------------
    cutoff = now - dt.timedelta(days=args.days)
    candidates = []
    for fk in forks:
        if fk["archived"] or not fk["login"] or not fk["default_branch"]:
            continue
        created, pushed = parse_iso(fk["created_at"]), parse_iso(fk["pushed_at"])
        if created is None or pushed is None:
            continue
        if pushed <= created or pushed < cutoff:
            continue
        candidates.append(fk)
    candidates.sort(key=lambda fk: (fk["pushed_at"], fk["login"]), reverse=True)
    active_forks = len(candidates)
    if args.limit:
        candidates = candidates[: args.limit]

    # -- 3~4. 대조·분류 ----------------------------------------------------
    rows: list[ForkRow] = []
    partial_reason = ""
    calls_since_check = 0
    for fk in candidates:
        if calls_since_check >= 20:
            calls_since_check = 0
            try:
                if rate_remaining()[0] < args.min_remaining:
                    partial_reason = (
                        f"쿼터 보호 중단 — 잔량이 --min-remaining({args.min_remaining}) 아래로 접근, "
                        f"{len(rows)}/{len(candidates)} 후보만 대조함"
                    )
                    break
            except GhError:
                pass  # 잔량 조회 실패는 치명적이지 않다
        row = ForkRow(
            login=fk["login"], full_name=fk["full_name"], branch=fk["default_branch"],
            created_at=fk["created_at"], pushed_at=fk["pushed_at"],
        )
        if args.base == "auto":
            row.base = fk["default_branch"] if fk["default_branch"] in branch_names else upstream_default
        else:
            row.base = args.base
        try:
            cmp = gh_get(
                f"repos/{args.repo}/compare/{row.base}...{row.login}:{row.branch}"
            )
            calls_since_check += 1
            row.ahead_by = int(cmp.get("ahead_by", 0))
            row.behind_by = int(cmp.get("behind_by", 0))
            if row.ahead_by > 0:
                filenames = [f.get("filename", "") for f in cmp.get("files", [])]
                row.file_counts = classify_files(filenames)
                row.total_files = len(filenames)
                row.category = primary_category(row.file_counts)
                subjects = [
                    c.get("commit", {}).get("message", "").splitlines()[0]
                    for c in cmp.get("commits", [])
                ]
                row.recent_commits = [s for s in subjects if s][-3:]
                if args.beacon:
                    row.beacon = fetch_beacon(row.full_name, row.branch)
                    calls_since_check += 1
                row.score = priority_score(row, now)
                row.label = priority_label(row.score)
        except GhError as exc:
            calls_since_check += 1
            row.error = str(exc)
        except (subprocess.TimeoutExpired, json.JSONDecodeError, ValueError) as exc:
            row.error = f"{type(exc).__name__}: {exc}"
        rows.append(row)

    rows.sort(key=lambda r: (-r.score, -r.ahead_by, r.full_name))

    # -- 5. 보고 -----------------------------------------------------------
    try:
        rate_end, reset_end = rate_remaining()
    except GhError:
        rate_end, reset_end = -1, reset_start
    stats = {
        "now": now, "total_forks": total_forks, "active_forks": active_forks,
        "compared": len(rows), "rate_start": rate_start, "rate_end": rate_end,
        "rate_window_reset": reset_end != reset_start,
    }
    os.makedirs(args.out_dir, exist_ok=True)
    tsv_path = os.path.join(args.out_dir, "harvest.tsv")
    md_path = os.path.join(args.out_dir, "harvest.md")
    write_tsv(tsv_path, rows)
    write_markdown(md_path, args, stats, rows, partial_reason)

    ahead = sum(1 for r in rows if not r.error and r.ahead_by > 0)
    errors = sum(1 for r in rows if r.error)
    quota_note = str(rate_start - rate_end) if rate_end >= 0 else "?"
    if stats["rate_window_reset"]:
        quota_note += " (창 리셋 경과 — 과소 보고)"
    print(
        f"포크 {total_forks} / 활동 {active_forks} / 대조 {len(rows)} / ahead>0 {ahead} / 오류 {errors}"
        f" — 쿼터 델타 {quota_note}"
    )
    print(f"출력: {tsv_path} , {md_path}")
    if partial_reason:
        print(f"부분 결과: {partial_reason}", file=sys.stderr)
    return 1 if (errors or partial_reason) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
