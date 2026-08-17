"""[운동장 차등 오라클] 같은 문서의 두 형식이 같은 답을 내는가.

## 착상

운동장 채점기는 정답을 골든 파일로 박제하지 않는다 — 기대값을 **채점 시점에
rhwp 로 재계산**한다. 이 성질에는 아직 쓰지 않은 쓸모가 하나 있다:

> 같은 문서의 HWP 판과 HWPX 판에 **같은 관측**을 물리면 답이 같아야 한다.
> 다르면, 둘 중 하나의 읽기 경로가 틀린 것이다.

즉 골든 파일 없이도 **차등(differential) 테스트**가 성립한다. 사람이 기대값을
적어둔 자리에서만 회귀를 잡는 보통의 테스트와 달리, 이 방식은 **아무도 기대값을
적어두지 않은 자리**까지 훑는다. 저장소에 쌍둥이 픽스처가 139쌍 있으므로
관측을 N개 얹으면 즉시 139×N 개의 판정이 생긴다.

## 오검출을 막는 관문 (이것이 없으면 도구가 거짓말을 한다)

같은 이름의 두 파일이 **실제로 같은 문서라는 보장이 없다**(개정판을 각각 저장한
경우가 있다). 그래서 관측이 어긋난 쌍은 곧바로 결함으로 부르지 않고 두 관문을
통과시킨다.

1. **본문 동일성** — 공백을 무시한 본문이 바이트로 같아야 한다. 다르면 그냥
   다른 문서다(결함 아님). 본문 해시를 못 구하면 동일 문서로 치지 않는다.
2. **IR 동일성** — `ir-diff` 가 `identical: true` 를 내야 한다. rhwp 자신이
   "두 문서의 구조는 같다" 고 말한 뒤에도 관측이 어긋난다면, 그것은 **내부
   모순**이고 결함 후보다.

이 관문을 세운 실측 근거: 표본 25쌍에서 어긋난 2건 중 1건은 본문 해시부터
달랐다(다른 개정판 — 결함 아님), 나머지 1건은 IR 동일 판정에도 쪽수가 달랐다
(진짜 후보).

보고 봉투: `kind=gymDifferential`, `schemaVersion=1.0`. 판정·집계·보고는 순수
함수라 `scripts/tests/test_gym_differential.py` 가 바이너리 없이 고정한다.

사용:
  python gym/tools/differential.py [--limit N] [--bin <경로>] [-o 결과.json]
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

for stream in (sys.stdout, sys.stderr):
    try:
        stream.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

from gym.core import runner  # noqa: E402

ROOT = runner.ROOT

REPORT_KIND = "gymDifferential"
SCHEMA_VERSION = "1.0"
TWIN_EXTS = (".hwp", ".hwpx")

#: 두 형식에서 같아야 하는 관측. (이름, 인자 템플릿, 봉투 경로)
OBSERVATIONS = [
    ("pageCount", ["info", "{f}", "--json"], "pageCount"),
    ("tableCount", ["export-tables", "{f}", "--json"], "tableCount"),
    ("paragraphCount", ["explain", "{f}", "--json"], "paragraphCount"),
    ("fieldCount", ["fields", "{f}", "--json"], "fieldCount"),
    ("footnoteCount", ["explain", "{f}", "--json"], "footnoteCount"),
    ("endnoteCount", ["explain", "{f}", "--json"], "endnoteCount"),
]


def run_cli(bin_path, args):
    proc = subprocess.run([bin_path] + args, cwd=ROOT, capture_output=True)
    try:
        return proc.returncode, json.loads(proc.stdout.decode("utf-8"))
    except ValueError:
        return proc.returncode, None


def observation_from_result(code, env, key):
    """CLI 결과에서 대조 가능한 관측을 뽑는다. 순수.

    종료 코드를 문자열로 붙이면 실제 값과 충돌하므로 kind 로 가른다.
    """
    if env is None:
        return {"kind": "nojson", "code": code}
    if not isinstance(env, dict):
        return {"kind": "badenv", "code": code}
    if key not in env:
        return {"kind": "missing", "key": key}
    return {"kind": "value", "value": env[key]}


def observations_equal(left, right):
    """관측 동일성. 숫자 6 과 6.0 은 같고, bool 은 int 로 접히지 않는다."""
    return _values_equal(left, right)


def _values_equal(left, right):
    if left is right:
        return True
    if isinstance(left, bool) or isinstance(right, bool):
        return isinstance(left, bool) and isinstance(right, bool) and left is right
    if isinstance(left, (int, float)) and isinstance(right, (int, float)):
        return float(left) == float(right)
    if isinstance(left, str) and isinstance(right, str):
        return left == right
    if isinstance(left, list) and isinstance(right, list):
        return len(left) == len(right) and all(
            _values_equal(a, b) for a, b in zip(left, right)
        )
    if isinstance(left, dict) and isinstance(right, dict):
        if set(left) != set(right):
            return False
        return all(_values_equal(left[k], right[k]) for k in left)
    return left == right


def observation_display(obs):
    """사람이 읽는 한 칸. 값 관측은 raw, 실패는 exitN."""
    if isinstance(obs, dict):
        kind = obs.get("kind")
        if kind == "value":
            return obs.get("value")
        if kind == "nojson":
            return f"exit{obs.get('code')}"
        if kind == "missing":
            return None
        if kind:
            return kind
    return obs


def pages_text(env):
    """export-text 봉투에서 쪽 본문을 이어 붙인다. 봉투가 아니면 None."""
    if not isinstance(env, dict):
        return None
    pages = env.get("pages")
    if not isinstance(pages, list):
        return None
    parts = []
    for page in pages:
        if isinstance(page, dict):
            parts.append(page.get("text") or "")
    return "".join(parts)


def normalize_body(text):
    """공백(개행·탭 포함)을 무시하는 본문."""
    return "".join(text.split())


def hash_text(text):
    return hashlib.sha256(text.encode("utf-8", "replace")).hexdigest()


def body_hash_from_env(env):
    """공백 무시 본문 해시. 봉투가 아니면 None — 없음은 동일이 아니다."""
    text = pages_text(env)
    if text is None:
        return None
    return hash_text(normalize_body(text))


def same_body_hash(left, right):
    return left is not None and right is not None and left == right


def ir_identity(env):
    """(identical, diffCount). 봉투가 없으면 identical=False."""
    if not isinstance(env, dict):
        return False, None
    return bool(env.get("identical")), env.get("diffCount")


def classify_pair(body_same, ir_identical, diverged):
    """관측이 갈린 쌍의 심각도. 갈림 없으면 None, 다른 문서면 other-doc."""
    if not diverged:
        return None
    if not body_same:
        return "other-doc"
    return "contradiction" if ir_identical else "review"


def diverged_rows(observed):
    """(label, hwp, hwpx) 목록에서 어긋난 행만, 관측 이름 순."""
    rows = [
        {"observation": label, "hwp": left, "hwpx": right}
        for label, left, right in observed
        if not observations_equal(left, right)
    ]
    rows.sort(key=lambda row: row["observation"])
    return rows


def make_finding(stem, hwp, hwpx, ir_identical, ir_diff_count, diverged):
    return {
        "stem": stem,
        "hwp": hwp,
        "hwpx": hwpx,
        "irIdentical": ir_identical,
        "irDiffCount": ir_diff_count,
        "diverged": list(diverged),
        "severity": "contradiction" if ir_identical else "review",
    }


def path_rank(rel):
    """같은 줄기에서 대표 경로를 고르는 키 — 얕은 경로, 그다음 사전순."""
    norm = rel.replace("\\", "/")
    return (norm.count("/"), norm)


def pick_twin_paths(hwps, hwpxs):
    """한 줄기에 파일이 여러 개일 때 대표 HWP/HWPX 를 결정적으로 고른다.

    같은 디렉터리에 양쪽이 있으면 그 짝(디렉터리 경로가 앞선 것)을 쓰고,
    없으면 얕고 사전순인 경로를 고른다. walk 순서에 의존하지 않는다.
    """
    hwps = sorted(hwps, key=path_rank)
    hwpxs = sorted(hwpxs, key=path_rank)
    if not hwps or not hwpxs:
        return None
    hwp_by_dir = {}
    for path in hwps:
        hwp_by_dir.setdefault(os.path.dirname(path.replace("\\", "/")), path)
    hwpx_by_dir = {}
    for path in hwpxs:
        hwpx_by_dir.setdefault(os.path.dirname(path.replace("\\", "/")), path)
    local = sorted(set(hwp_by_dir) & set(hwpx_by_dir))
    if local:
        directory = local[0]
        return hwp_by_dir[directory], hwpx_by_dir[directory]
    return hwps[0], hwpxs[0]


def find_twins_in(samples_dir, root=None):
    """(stem, hwp, hwpx) 목록. 줄기 사전순. 상대경로는 `/` 로 정규화."""
    if not os.path.isdir(samples_dir):
        return []
    base = samples_dir if root is None else root
    seen = {}
    for dirpath, dirnames, files in os.walk(samples_dir):
        dirnames.sort()
        for name in sorted(files):
            stem, ext = os.path.splitext(name)
            ext_l = ext.lower()
            if ext_l not in TWIN_EXTS:
                continue
            rel = os.path.relpath(os.path.join(dirpath, name), base).replace("\\", "/")
            seen.setdefault(stem, {}).setdefault(ext_l, []).append(rel)
    pairs = []
    for stem, by_ext in seen.items():
        picked = pick_twin_paths(by_ext.get(".hwp", []), by_ext.get(".hwpx", []))
        if picked:
            pairs.append((stem, picked[0], picked[1]))
    pairs.sort(key=lambda item: (item[0], item[1], item[2]))
    return pairs


def find_twins():
    return find_twins_in(os.path.join(ROOT, "samples"), root=ROOT)


def select_pairs(pairs, limit):
    """limit<=0 이면 전부. 앞부분은 정렬된 입력의 접두라 결정적이다."""
    items = list(pairs)
    if not limit:
        return items
    return items[:limit]


def observe(bin_path, path, args, key):
    code, env = run_cli(bin_path, [a.replace("{f}", path) for a in args])
    return observation_from_result(code, env, key)


def body_hash(bin_path, path):
    """공백을 무시한 본문의 해시 — 두 파일이 같은 문서인지 가르는 1차 관문."""
    _code, env = run_cli(bin_path, ["export-text", path, "--json"])
    return body_hash_from_env(env)


def compare_twins(pairs, run, observations=None):
    """쌍 목록을 대조한다. 순수 — run(args)->(code, env) 만 주입하면 된다.

    반환: (관측대조건수, 이름만같은다른문서, findings)
    """
    observations = OBSERVATIONS if observations is None else observations
    findings = []
    other_doc = 0
    compared = 0
    for stem, hwp, hwpx in pairs:
        observed = []
        for label, args, key in observations:
            left = observation_from_result(*run([a.replace("{f}", hwp) for a in args]), key)
            right = observation_from_result(*run([a.replace("{f}", hwpx) for a in args]), key)
            compared += 1
            observed.append((label, left, right))
        diverged = diverged_rows(observed)
        if not diverged:
            continue
        ha = body_hash_from_env(run(["export-text", hwp, "--json"])[1])
        hb = body_hash_from_env(run(["export-text", hwpx, "--json"])[1])
        if not same_body_hash(ha, hb):
            other_doc += 1
            continue
        _code, env = run(["ir-diff", hwp, hwpx, "--json"])
        identical, diff_count = ir_identity(env)
        findings.append(make_finding(stem, hwp, hwpx, identical, diff_count, diverged))
    findings.sort(key=lambda row: (row["stem"], row["hwp"], row["hwpx"]))
    return compared, other_doc, findings


def build_report(*, bin_name, pairs_count, compared, other_doc, findings):
    ordered = sorted(findings, key=lambda row: (row.get("stem") or "", row.get("hwp") or ""))
    contradictions = sum(1 for row in ordered if row.get("severity") == "contradiction")
    reviews = sum(1 for row in ordered if row.get("severity") == "review")
    return {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "ok": contradictions == 0,
        "runner": {"bin": bin_name},
        "pairs": pairs_count,
        "observationsCompared": compared,
        "sameNameDifferentDocument": other_doc,
        "findings": ordered,
        "contradictions": contradictions,
        "reviews": reviews,
    }


def write_report(report, path):
    with io.open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(json.dumps(report, ensure_ascii=False, indent=2))
        fh.write("\n")


def format_finding_detail(finding):
    return ", ".join(
        f"{row['observation']} {observation_display(row['hwp'])}≠{observation_display(row['hwpx'])}"
        for row in finding.get("diverged", [])
    )


def render_summary(report, out_path=None):
    lines = [
        f"쌍둥이 {report['pairs']}쌍 · 관측 대조 {report['observationsCompared']}건",
        f"이름만 같은 다른 문서(제외): {report['sameNameDifferentDocument']}쌍",
        f"결함 후보: {len(report['findings'])}건 (그중 IR 동일 모순 {report['contradictions']}건)",
    ]
    for finding in report["findings"]:
        mark = "!!" if finding["severity"] == "contradiction" else "  "
        lines.append(
            f" {mark} {finding['stem'][:46]:48} irIdentical={finding['irIdentical']} | "
            f"{format_finding_detail(finding)}"
        )
    if out_path:
        lines.append(f"→ {out_path}")
    return lines


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0, help="검사할 쌍 수 (0=전부)")
    ap.add_argument("--bin", default=None)
    ap.add_argument("-o", "--out", default=None)
    a = ap.parse_args()

    bin_path = runner.find_bin(a.bin)
    pairs = select_pairs(find_twins(), a.limit)
    compared, other_doc, findings = compare_twins(
        pairs, lambda args: run_cli(bin_path, args)
    )
    report = build_report(
        bin_name=os.path.basename(bin_path),
        pairs_count=len(pairs),
        compared=compared,
        other_doc=other_doc,
        findings=findings,
    )
    out = a.out or os.path.join(runner.GYM, "differential-report.json")
    write_report(report, out)
    for line in render_summary(report, out):
        print(line)
    return 0 if report["ok"] else 3


if __name__ == "__main__":
    sys.exit(main())
