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
   다른 문서다(결함 아님).
2. **IR 동일성** — `ir-diff` 가 `identical: true` 를 내야 한다. rhwp 자신이
   "두 문서의 구조는 같다" 고 말한 뒤에도 관측이 어긋난다면, 그것은 **내부
   모순**이고 결함 후보다.

이 관문을 세운 실측 근거: 표본 25쌍에서 어긋난 2건 중 1건은 본문 해시부터
달랐다(다른 개정판 — 결함 아님), 나머지 1건은 IR 동일 판정에도 쪽수가 달랐다
(진짜 후보).

사용:
  python gym/tools/differential.py [--limit N] [--bin <경로>] [-o 결과.json]
"""

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


def observe(bin_path, path, args, key):
    code, env = run_cli(bin_path, [a.replace("{f}", path) for a in args])
    if env is None:
        return f"exit{code}"
    return env.get(key)


def body_hash(bin_path, path):
    """공백을 무시한 본문의 해시 — 두 파일이 같은 문서인지 가르는 1차 관문."""
    _code, env = run_cli(bin_path, ["export-text", path, "--json"])
    if env is None:
        return None
    text = "".join(p.get("text", "") for p in env.get("pages", []))
    norm = "".join(text.split())
    return hashlib.sha256(norm.encode("utf-8", "replace")).hexdigest()


def find_twins():
    seen = {}
    for root, _dirs, files in os.walk(os.path.join(ROOT, "samples")):
        for name in files:
            stem, ext = os.path.splitext(name)
            if ext.lower() in (".hwp", ".hwpx"):
                rel = os.path.relpath(os.path.join(root, name), ROOT).replace("\\", "/")
                seen.setdefault(stem, {})[ext.lower()] = rel
    return sorted((k, v[".hwp"], v[".hwpx"]) for k, v in seen.items()
                  if ".hwp" in v and ".hwpx" in v)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0, help="검사할 쌍 수 (0=전부)")
    ap.add_argument("--bin", default=None)
    ap.add_argument("-o", "--out", default=None)
    a = ap.parse_args()

    bin_path = runner.find_bin(a.bin)
    pairs = find_twins()
    if a.limit:
        pairs = pairs[:a.limit]

    findings, differing_but_other_doc, compared = [], 0, 0
    for stem, hwp, hwpx in pairs:
        diverged = []
        for label, args, key in OBSERVATIONS:
            va = observe(bin_path, hwp, args, key)
            vb = observe(bin_path, hwpx, args, key)
            compared += 1
            if va != vb:
                diverged.append({"observation": label, "hwp": va, "hwpx": vb})
        if not diverged:
            continue
        # 관문 1 — 본문이 다르면 애초에 다른 문서다.
        ha, hb = body_hash(bin_path, hwp), body_hash(bin_path, hwpx)
        if ha != hb:
            differing_but_other_doc += 1
            continue
        # 관문 2 — rhwp 자신이 IR 동일이라 말했는가.
        _code, env = run_cli(bin_path, ["ir-diff", hwp, hwpx, "--json"])
        identical = bool(env and env.get("identical"))
        findings.append({
            "stem": stem, "hwp": hwp, "hwpx": hwpx,
            "irIdentical": identical,
            "irDiffCount": (env or {}).get("diffCount"),
            "diverged": diverged,
            # IR 이 같다고 해놓고 관측이 어긋나면 내부 모순 — 우선순위가 높다.
            "severity": "contradiction" if identical else "review",
        })

    report = {
        "kind": "gymDifferentialReport", "schemaVersion": "1.0",
        "runner": {"bin": os.path.basename(bin_path)},
        "pairs": len(pairs), "observationsCompared": compared,
        "sameNameDifferentDocument": differing_but_other_doc,
        "findings": findings,
        "contradictions": sum(1 for f in findings if f["severity"] == "contradiction"),
    }
    out = a.out or os.path.join(runner.GYM, "differential-report.json")
    with io.open(out, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(json.dumps(report, ensure_ascii=False, indent=2))

    print(f"쌍둥이 {report['pairs']}쌍 · 관측 대조 {compared}건")
    print(f"이름만 같은 다른 문서(제외): {differing_but_other_doc}쌍")
    print(f"결함 후보: {len(findings)}건 (그중 IR 동일 모순 {report['contradictions']}건)")
    for f in findings:
        mark = "!!" if f["severity"] == "contradiction" else "  "
        detail = ", ".join(f"{d['observation']} {d['hwp']}≠{d['hwpx']}" for d in f["diverged"])
        print(f" {mark} {f['stem'][:46]:48} irIdentical={f['irIdentical']} | {detail}")
    print(f"→ {out}")
    return 0 if report["contradictions"] == 0 else 3


if __name__ == "__main__":
    sys.exit(main())
