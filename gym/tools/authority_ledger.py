"""[#6628] Gym 정답 권위와 기준풀이 출처 전수 원장.

이 원장은 "기준풀이가 통과한다"를 제품 정답으로 승격하지 않는다. task의 실제
check가 현재 rhwp를 호출하는지, 공개 입력 fixture와 직접 비교하는지, 작성자가 고정한
내부 계약만 검사하는지를 보수적으로 분류한다. reference가 기준 제출물을 어떻게
만드는지도 별도 축으로 기록한다.

사용:
    python3 gym/tools/authority_ledger.py
    python3 gym/tools/authority_ledger.py --json
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


HERE = Path(__file__).resolve().parent
GYM_ROOT = HERE.parent
REPO_ROOT = GYM_ROOT.parent
sys.path.insert(0, str(GYM_ROOT))

from core import checks as check_registry  # noqa: E402


REPORT_KIND = "gymAuthorityLedger"
SCHEMA_VERSION = "1.0"
AUTHORITY_CLASSES = (
    "self-live",
    "contract-constant",
    "independent-fixture",
    "external-oracle",
)
BASELINE_SOURCES = ("self-live", "contract-constant")

CLASS_DEFINITIONS = {
    "self-live": (
        "채점 check가 제출물 또는 입력을 현재 rhwp로 다시 읽어 판정한다. "
        "벤치마크 내부 일관성 근거이며 독립 제품 정답이 아니다."
    ),
    "contract-constant": (
        "task 작성자가 고정한 값, 형식 또는 제출물 사이 관계를 검사한다. "
        "외부 구현의 정답 근거가 아니다."
    ),
    "independent-fixture": (
        "현재 rhwp를 호출하지 않고 저장소의 공개 입력 fixture와 직접 관계를 검사한다. "
        "그 관계의 범위만 독립적이며 전체 의미 동등성을 증명하지 않는다."
    ),
    "external-oracle": (
        "한컴 또는 독립 구현에서 얻은 공개 증적을 명시적으로 인용한다. "
        "인용된 증적 범위를 넘어 제품 정확성을 주장하지 않는다."
    ),
}

ISSUE_CODES = (
    "missing-packs-root",
    "task-read",
    "task-not-object",
    "task-empty-id",
    "reference-missing",
    "reference-without-task",
    "reference-read",
    "reference-not-object",
    "reference-id-mismatch",
    "checks-not-list",
    "checks-empty",
    "check-not-object",
    "unknown-check-op",
    "missing-live-command",
    "authority-metadata-not-object",
    "multiple-authority",
    "unknown-authority",
    "authority-conflict",
    "authority-evidence-not-list",
    "authority-evidence-required",
    "authority-evidence-invalid",
    "evidence-outside-repo",
    "evidence-path-missing",
    "input-evidence-invalid",
    "reference-steps-not-list",
    "reference-steps-empty",
    "reference-step-not-object",
    "baseline-command-invalid",
    "baseline-answer-not-object",
    "baseline-answer-spec-invalid",
    "multiple-baseline-source",
    "baseline-source-unclassified",
    "authority-unclassified",
    "duplicate-entry",
    "entry-count-mismatch",
    "summary-mismatch",
)

CAVEATS = {
    "self-live": "current-rhwp-dependent; not an independent product oracle",
    "contract-constant": "task-authored contract; not external correctness evidence",
    "independent-fixture": "independent only for the cited public fixture relation",
    "external-oracle": "external only for the scope of the cited evidence",
}


def json_pointer_part(value: str) -> str:
    """RFC 6901 한 칸 이스케이프."""
    return str(value).replace("~", "~0").replace("/", "~1")


def issue(code, *, pack="", task="", path="", pointer="", message=""):
    if code not in ISSUE_CODES:
        code = "authority-unclassified"
    return {
        "code": code,
        "pack": pack,
        "task": task,
        "path": path,
        "pointer": pointer,
        "message": message,
    }


def read_json(path: Path):
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def repo_relative(path: Path, repo_root: Path):
    """repo 내부 경로면 POSIX 상대 경로, 밖이면 None."""
    try:
        return path.resolve().relative_to(repo_root.resolve()).as_posix()
    except (OSError, ValueError):
        return None


def evidence_path(raw, repo_root: Path, *, pack, task, pointer, issues):
    """명시 증적 경로를 repo 안의 실재 파일로 제한한다."""
    if not isinstance(raw, str) or not raw.strip():
        issues.append(issue(
            "authority-evidence-invalid",
            pack=pack,
            task=task,
            pointer=pointer,
            message="증적 경로가 비어 있거나 문자열이 아니다",
        ))
        return None
    candidate = Path(raw)
    if not candidate.is_absolute():
        candidate = repo_root / candidate
    relative = repo_relative(candidate, repo_root)
    if relative is None:
        issues.append(issue(
            "evidence-outside-repo",
            pack=pack,
            task=task,
            path=str(raw),
            pointer=pointer,
            message="증적이 공개 저장소 루트 밖을 가리킨다",
        ))
        return None
    if not candidate.is_file():
        issues.append(issue(
            "evidence-path-missing",
            pack=pack,
            task=task,
            path=relative,
            pointer=pointer,
            message="증적 파일이 저장소에 없다",
        ))
        return None
    return relative


def explicit_authority(task_doc, repo_root: Path, *, pack, task, issues):
    """선택적 authority 메타데이터. 외부/독립 승격은 실재 증적을 요구한다."""
    raw = task_doc.get("authority")
    if raw is None:
        return None, []
    if not isinstance(raw, dict):
        code = "multiple-authority" if isinstance(raw, list) else "authority-metadata-not-object"
        issues.append(issue(
            code,
            pack=pack,
            task=task,
            pointer="/authority",
            message="authority는 class 하나와 evidence 목록을 가진 객체여야 한다",
        ))
        return None, []

    raw_class = raw.get("class")
    if isinstance(raw_class, list):
        issues.append(issue(
            "multiple-authority",
            pack=pack,
            task=task,
            pointer="/authority/class",
            message="task의 primary authority는 정확히 하나여야 한다",
        ))
        return None, []
    if raw_class not in AUTHORITY_CLASSES:
        issues.append(issue(
            "unknown-authority",
            pack=pack,
            task=task,
            pointer="/authority/class",
            message=f"알 수 없는 authority class: {raw_class!r}",
        ))
        return None, []

    raw_evidence = raw.get("evidence", [])
    if not isinstance(raw_evidence, list):
        issues.append(issue(
            "authority-evidence-not-list",
            pack=pack,
            task=task,
            pointer="/authority/evidence",
            message="authority.evidence는 저장소 상대 경로 목록이어야 한다",
        ))
        return raw_class, []
    if raw_class in ("independent-fixture", "external-oracle") and not raw_evidence:
        issues.append(issue(
            "authority-evidence-required",
            pack=pack,
            task=task,
            pointer="/authority/evidence",
            message=f"{raw_class}는 공개 증적 경로가 하나 이상 필요하다",
        ))

    evidence = []
    for index, value in enumerate(raw_evidence):
        relative = evidence_path(
            value,
            repo_root,
            pack=pack,
            task=task,
            pointer=f"/authority/evidence/{index}",
            issues=issues,
        )
        if relative is not None:
            evidence.append(relative)
    return raw_class, evidence


def classify_authority(task_doc, task_path: Path, repo_root: Path, *, pack, task, issues):
    checks = task_doc.get("checks")
    task_relative = repo_relative(task_path, repo_root) or task_path.as_posix()
    if not isinstance(checks, list):
        issues.append(issue(
            "checks-not-list", pack=pack, task=task, path=task_relative,
            pointer="/checks", message="checks가 목록이 아니다",
        ))
        checks = []
    if not checks:
        issues.append(issue(
            "checks-empty", pack=pack, task=task, path=task_relative,
            pointer="/checks", message="채점 authority를 판정할 check가 없다",
        ))

    live_pointers = []
    fixture_pointers = []
    contract_pointers = []
    for index, check in enumerate(checks):
        pointer = f"/checks/{index}"
        if not isinstance(check, dict):
            issues.append(issue(
                "check-not-object", pack=pack, task=task, path=task_relative,
                pointer=pointer, message="check가 객체가 아니다",
            ))
            continue
        op = check.get("op")
        if op not in check_registry.REGISTRY:
            issues.append(issue(
                "unknown-check-op", pack=pack, task=task, path=task_relative,
                pointer=f"{pointer}/op", message=f"등록되지 않은 check op: {op!r}",
            ))
            continue
        if check_registry.needs_cli(op):
            command = check.get("cmd")
            if (
                not isinstance(command, list)
                or not command
                or any(not isinstance(item, str) or not item for item in command)
            ):
                issues.append(issue(
                    "missing-live-command", pack=pack, task=task, path=task_relative,
                    pointer=f"{pointer}/cmd", message=f"{op}의 현재 rhwp 명령이 유효하지 않다",
                ))
            live_pointers.append(f"{pointer}/cmd")
        elif op == "differs_from_input":
            fixture_pointers.append(pointer)
        else:
            contract_pointers.append(pointer)

    explicit, declared_evidence = explicit_authority(
        task_doc, repo_root, pack=pack, task=task, issues=issues,
    )

    if live_pointers:
        inferred = "self-live"
    elif fixture_pointers:
        inferred = "independent-fixture"
    elif contract_pointers:
        inferred = "contract-constant"
    else:
        inferred = None

    authority = inferred
    if explicit is not None:
        if live_pointers and explicit != "self-live":
            issues.append(issue(
                "authority-conflict", pack=pack, task=task, path=task_relative,
                pointer="/authority/class",
                message="현재 rhwp check가 있으므로 더 독립적인 authority로 승격할 수 없다",
            ))
        elif explicit == "self-live" and not live_pointers:
            issues.append(issue(
                "authority-conflict", pack=pack, task=task, path=task_relative,
                pointer="/authority/class",
                message="self-live 선언을 뒷받침할 현재 rhwp check가 없다",
            ))
        elif explicit == "contract-constant" and fixture_pointers:
            issues.append(issue(
                "authority-conflict", pack=pack, task=task, path=task_relative,
                pointer="/authority/class",
                message="공개 입력 fixture 관계를 contract-constant로 낮춰 적었다",
            ))
        else:
            authority = explicit

    if authority is None:
        issues.append(issue(
            "authority-unclassified", pack=pack, task=task, path=task_relative,
            message="task check에서 primary authority를 분류할 수 없다",
        ))

    signals = []
    for name, pointers in (
        ("self-live", live_pointers),
        ("independent-fixture", fixture_pointers),
        ("contract-constant", contract_pointers),
    ):
        if pointers:
            signals.append(name)

    locators = []
    primary_pointers = {
        "self-live": live_pointers,
        "independent-fixture": fixture_pointers,
        "contract-constant": contract_pointers,
        "external-oracle": ["/authority"],
    }.get(authority, [])
    if primary_pointers:
        locators.append({
            "role": "task-authority",
            "path": task_relative,
            "pointers": sorted(primary_pointers),
        })

    if authority == "independent-fixture" and explicit is None:
        raw_input = task_doc.get("input")
        relative = evidence_path(
            raw_input,
            repo_root,
            pack=pack,
            task=task,
            pointer="/input",
            issues=issues,
        ) if isinstance(raw_input, str) else None
        if relative is None and not isinstance(raw_input, str):
            issues.append(issue(
                "input-evidence-invalid", pack=pack, task=task,
                path=task_relative, pointer="/input",
                message="independent-fixture의 input 경로가 문자열이 아니다",
            ))
        if relative is not None:
            locators.append({
                "role": "public-input-fixture",
                "path": relative,
                "pointers": [],
            })

    for path in declared_evidence:
        locators.append({
            "role": "declared-authority-evidence",
            "path": path,
            "pointers": [],
        })

    if authority in AUTHORITY_CLASSES and not locators:
        issues.append(issue(
            "authority-evidence-required", pack=pack, task=task, path=task_relative,
            message="분류를 재계산할 source/evidence locator가 없다",
        ))

    return authority, signals, locators, explicit is not None


def classify_baseline(reference_doc, reference_path: Path, repo_root: Path, *, pack, task, issues):
    relative = repo_relative(reference_path, repo_root) or reference_path.as_posix()
    steps = reference_doc.get("steps")
    if not isinstance(steps, list):
        issues.append(issue(
            "reference-steps-not-list", pack=pack, task=task, path=relative,
            pointer="/steps", message="reference.steps가 목록이 아니다",
        ))
        steps = []
    if not steps:
        issues.append(issue(
            "reference-steps-empty", pack=pack, task=task, path=relative,
            pointer="/steps", message="기준풀이 출처를 판정할 step이 없다",
        ))

    live_pointers = []
    constant_pointers = []
    for step_index, step in enumerate(steps):
        step_pointer = f"/steps/{step_index}"
        if not isinstance(step, dict):
            issues.append(issue(
                "reference-step-not-object", pack=pack, task=task, path=relative,
                pointer=step_pointer, message="reference step이 객체가 아니다",
            ))
            continue
        if "run" in step:
            command = step.get("run")
            if (
                not isinstance(command, list)
                or not command
                or any(not isinstance(item, str) or not item for item in command)
            ):
                issues.append(issue(
                    "baseline-command-invalid", pack=pack, task=task, path=relative,
                    pointer=f"{step_pointer}/run", message="reference run 명령이 유효하지 않다",
                ))
            live_pointers.append(f"{step_pointer}/run")

        if "answer" in step:
            answer = step.get("answer")
            if not isinstance(answer, dict) or not answer:
                issues.append(issue(
                    "baseline-answer-not-object", pack=pack, task=task, path=relative,
                    pointer=f"{step_pointer}/answer", message="reference answer가 비어 있거나 객체가 아니다",
                ))
                continue
            for key in sorted(answer):
                spec = answer[key]
                pointer = f"{step_pointer}/answer/{json_pointer_part(key)}"
                if isinstance(spec, dict) and "cmd" in spec:
                    command = spec.get("cmd")
                    if (
                        not isinstance(command, list)
                        or not command
                        or any(not isinstance(item, str) or not item for item in command)
                    ):
                        issues.append(issue(
                            "baseline-command-invalid", pack=pack, task=task, path=relative,
                            pointer=f"{pointer}/cmd", message="reference answer cmd가 유효하지 않다",
                        ))
                    live_pointers.append(f"{pointer}/cmd")
                elif isinstance(spec, dict) and "const" in spec:
                    constant_pointers.append(f"{pointer}/const")
                else:
                    issues.append(issue(
                        "baseline-answer-spec-invalid", pack=pack, task=task, path=relative,
                        pointer=pointer, message="answer spec에는 cmd 또는 const 하나가 필요하다",
                    ))

    if live_pointers and constant_pointers:
        issues.append(issue(
            "multiple-baseline-source", pack=pack, task=task, path=relative,
            message="한 task의 기준풀이에 self-live와 contract-constant 출처가 함께 있다",
        ))
        source = None
        pointers = live_pointers + constant_pointers
    elif live_pointers:
        source = "self-live"
        pointers = live_pointers
    elif constant_pointers:
        source = "contract-constant"
        pointers = constant_pointers
    else:
        source = None
        pointers = []
        issues.append(issue(
            "baseline-source-unclassified", pack=pack, task=task, path=relative,
            message="reference에서 기준 제출물의 출처를 분류할 수 없다",
        ))

    locators = []
    if pointers:
        locators.append({
            "role": "baseline-source",
            "path": relative,
            "pointers": sorted(pointers),
        })
    return source, locators


def empty_counts(names):
    return {name: 0 for name in names}


def recompute_summary(entries):
    authority = empty_counts(AUTHORITY_CLASSES)
    baseline = empty_counts(BASELINE_SOURCES)
    explicit = 0
    for entry in entries:
        if not isinstance(entry, dict):
            continue
        primary = entry.get("authority")
        if primary in authority:
            authority[primary] += 1
        source = entry.get("baselineSource")
        if source in baseline:
            baseline[source] += 1
        if entry.get("explicitAuthority") is True:
            explicit += 1
    return {
        "byAuthority": authority,
        "byBaselineSource": baseline,
        "explicitAuthorityCount": explicit,
    }


def build_ledger(gym_root=GYM_ROOT, repo_root=None):
    gym_root = Path(gym_root).resolve()
    repo_root = Path(repo_root).resolve() if repo_root is not None else gym_root.parent.resolve()
    packs_root = gym_root / "packs"
    issues = []
    entries = []
    task_file_count = 0
    reference_file_count = 0
    seen_keys = set()

    if not packs_root.is_dir():
        issues.append(issue(
            "missing-packs-root",
            path=repo_relative(packs_root, repo_root) or packs_root.as_posix(),
            message="gym/packs 루트가 없다",
        ))
        pack_dirs = []
    else:
        pack_dirs = sorted(path for path in packs_root.iterdir() if path.is_dir())

    for pack_dir in pack_dirs:
        pack = pack_dir.name
        tasks_dir = pack_dir / "tasks"
        reference_dir = pack_dir / "reference"
        task_paths = sorted(tasks_dir.glob("*.json")) if tasks_dir.is_dir() else []
        reference_paths = sorted(reference_dir.glob("*.json")) if reference_dir.is_dir() else []
        reference_file_count += len(reference_paths)
        task_names = {path.name for path in task_paths}
        for reference_path in reference_paths:
            if reference_path.name not in task_names:
                issues.append(issue(
                    "reference-without-task",
                    pack=pack,
                    path=repo_relative(reference_path, repo_root) or reference_path.as_posix(),
                    message="짝 task가 없는 reference다",
                ))
        for task_path in task_paths:
            task_file_count += 1
            task_relative = repo_relative(task_path, repo_root) or task_path.as_posix()
            try:
                task_doc = read_json(task_path)
            except (OSError, UnicodeError, ValueError) as exc:
                issues.append(issue(
                    "task-read", pack=pack, path=task_relative,
                    message=f"task JSON을 읽을 수 없다: {type(exc).__name__}: {exc}",
                ))
                continue
            if not isinstance(task_doc, dict):
                issues.append(issue(
                    "task-not-object", pack=pack, path=task_relative,
                    message="task JSON이 객체가 아니다",
                ))
                continue
            task = task_doc.get("id")
            if not isinstance(task, str) or not task:
                issues.append(issue(
                    "task-empty-id", pack=pack, path=task_relative,
                    pointer="/id", message="task id가 비었다",
                ))
                continue

            key = f"{pack}/{task}"
            if key in seen_keys:
                issues.append(issue(
                    "duplicate-entry", pack=pack, task=task, path=task_relative,
                    message="같은 pack/task가 원장에 두 번 나타난다",
                ))
                continue
            seen_keys.add(key)

            reference_path = reference_dir / task_path.name
            reference_relative = repo_relative(reference_path, repo_root) or reference_path.as_posix()
            if not reference_path.is_file():
                issues.append(issue(
                    "reference-missing", pack=pack, task=task, path=reference_relative,
                    message="짝 reference가 없다",
                ))
                continue
            try:
                reference_doc = read_json(reference_path)
            except (OSError, UnicodeError, ValueError) as exc:
                issues.append(issue(
                    "reference-read", pack=pack, task=task, path=reference_relative,
                    message=f"reference JSON을 읽을 수 없다: {type(exc).__name__}: {exc}",
                ))
                continue
            if not isinstance(reference_doc, dict):
                issues.append(issue(
                    "reference-not-object", pack=pack, task=task, path=reference_relative,
                    message="reference JSON이 객체가 아니다",
                ))
                continue
            if reference_doc.get("id") != task:
                issues.append(issue(
                    "reference-id-mismatch", pack=pack, task=task, path=reference_relative,
                    pointer="/id", message=f"reference id가 task id와 다르다: {reference_doc.get('id')!r}",
                ))

            authority, signals, authority_evidence, explicit = classify_authority(
                task_doc,
                task_path,
                repo_root,
                pack=pack,
                task=task,
                issues=issues,
            )
            baseline_source, baseline_evidence = classify_baseline(
                reference_doc,
                reference_path,
                repo_root,
                pack=pack,
                task=task,
                issues=issues,
            )

            entries.append({
                "key": key,
                "pack": pack,
                "task": task,
                "taskPath": task_relative,
                "referencePath": reference_relative,
                "authority": authority,
                "authoritySignals": signals,
                "authorityEvidence": authority_evidence,
                "explicitAuthority": explicit,
                "baselineSource": baseline_source,
                "baselineEvidence": baseline_evidence,
                "caveat": CAVEATS.get(authority, "unclassified authority"),
            })

    if len(entries) != task_file_count:
        issues.append(issue(
            "entry-count-mismatch",
            message=f"task 파일 {task_file_count}개와 원장 entry {len(entries)}개가 다르다",
        ))

    summary = recompute_summary(entries)
    report = {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "ok": False,
        "root": repo_relative(packs_root, repo_root) or packs_root.as_posix(),
        "authorityClasses": {
            name: CLASS_DEFINITIONS[name] for name in AUTHORITY_CLASSES
        },
        "classificationRule": (
            "self-live if any scoring check invokes current rhwp; otherwise explicit external or "
            "independent evidence; otherwise public input fixture relation; otherwise task-authored contract"
        ),
        "taskCount": task_file_count,
        "referenceCount": reference_file_count,
        "entryCount": len(entries),
        "summary": summary,
        "entries": entries,
        "issueCount": 0,
        "issues": issues,
        "exit": 1,
    }

    recalculated = recompute_summary(report["entries"])
    if recalculated != report["summary"]:
        issues.append(issue(
            "summary-mismatch",
            message="summary가 entries에서 재계산한 값과 다르다",
        ))
    report["issueCount"] = len(issues)
    report["ok"] = not issues and report["taskCount"] == report["entryCount"]
    report["exit"] = 0 if report["ok"] else 1
    return report


def render_human(report):
    summary = report["summary"]
    lines = [
        f"Gym 정답 권위 원장: {report['entryCount']}/{report['taskCount']} task 분류",
        "  authority: " + " · ".join(
            f"{name} {summary['byAuthority'][name]}" for name in AUTHORITY_CLASSES
        ),
        "  baseline: " + " · ".join(
            f"{name} {summary['byBaselineSource'][name]}" for name in BASELINE_SOURCES
        ),
    ]
    if report["ok"]:
        lines.append("  판정: 위반 0 — self-live는 독립 제품 정답이 아님")
    else:
        lines.append(f"  판정: 위반 {report['issueCount']}건")
        for row in report["issues"][:20]:
            where = "/".join(value for value in (row["pack"], row["task"]) if value)
            lines.append(f"  [{row['code']}] {where or row['path']}: {row['message']}")
    return "\n".join(lines)


def parse_args(argv=None):
    parser = argparse.ArgumentParser(description="Gym 정답 권위와 기준풀이 출처 전수 원장")
    parser.add_argument("--json", action="store_true", help="기계 판독 JSON 봉투")
    return parser.parse_args(argv)


def main(argv=None):
    args = parse_args(argv)
    report = build_ledger()
    if args.json:
        print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
    else:
        print(render_human(report))
    return report["exit"]


if __name__ == "__main__":
    raise SystemExit(main())
