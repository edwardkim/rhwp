# -*- coding: utf-8 -*-
"""[#4535] 에이전트 축 주도 실측 하네스.

"주도한다"는 형용사를 git 과 바이너리에서 산출되는 숫자로 바꾼다.

원칙:
1) 수치의 모집단은 **origin/devel 병합 이력**이다 — 열린 PR·자기 브랜치로
   부풀리지 않는다(인정된 기여만). 귀속은 git author 다(이 저장소의 통합
   리뷰 브랜치는 cherry-pick 시 원저자를 보존함을 실측 확인).
2) 스냅샷은 일자 보고서다 — 숫자는 매일 움직이는 것이 정상이므로 멱등
   가드의 대상이 아니다. 같은 커밋에서 재실행하면 같은 숫자가 나온다(재현).
3) 경로군은 아래 AXIS 표가 전부다 — 恣意 편집을 막기 위해 보고서에 경로군
   원문을 그대로 싣는다.

사용:
  python tools/agent_dominance_metrics.py [--ref origin/devel] [--out <md>]
"""

import argparse
import datetime
import glob
import io
import json
import os
import subprocess
import sys
from collections import defaultdict

for stream in (sys.stdout, sys.stderr):
    try:
        stream.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# 에이전트 축 경로군 — 보고서에 원문 그대로 실린다.
AXIS = [
    ("로드맵·조망", ["mydocs/tech/agent_roadmap"]),
    ("지식지도·통합 문서", ["mydocs/manual/agent_knowledge_map.md",
                        "mydocs/manual/mcp_integration_guide.md",
                        "mydocs/manual/agent_codex"]),
    ("스킬", [".claude/skills"]),
    ("에이전트 코어(src)", ["src/mcp_serve.rs", "src/agent_profiles.rs",
                         "src/provenance.rs", "src/schema_registry.rs",
                         "src/capsule_sign.rs"]),
    ("계약 가드(tests)", ["tests/*contract*.rs"]),
    ("하네스 도구", ["tools/roadmap_progress.py", "tools/agent_preflight.py",
                  "tools/gen_agent_codex.py", "tools/agent_dominance_metrics.py"]),
]


def sh(args):
    p = subprocess.run(args, cwd=ROOT, capture_output=True)
    return p.stdout.decode("utf-8", errors="replace")


def git_stats(ref, paths, since=None):
    """경로군의 author별 (커밋 수, 추가 줄 수)."""
    cmd = ["git", "log", ref, "--pretty=AUTHOR:%an", "--numstat"]
    if since:
        cmd.insert(2, f"--since={since}")
    cmd += ["--"] + paths
    commits = defaultdict(int)
    added = defaultdict(int)
    author = None
    for line in sh(cmd).splitlines():
        if line.startswith("AUTHOR:"):
            author = line[7:]
            commits[author] += 1
        elif line and line[0].isdigit() and author:
            parts = line.split("\t")
            if parts[0].isdigit():
                added[author] += int(parts[0])
    return commits, added


def share(counter, who):
    total = sum(counter.values())
    mine = counter.get(who, 0)
    return mine, total, (100.0 * mine / total if total else 0.0)


def surface_metrics():
    """바이너리·저장소에서 표면 절대 수치."""
    out = {}
    rhwp = None
    for rel in ("target/debug/rhwp.exe", "target/debug/rhwp"):
        p = os.path.join(ROOT, rel.replace("/", os.sep))
        if os.path.exists(p):
            rhwp = p
            break
    if rhwp:
        try:
            caps = json.loads(sh([rhwp, "capabilities"]))
            out["자기서술 명령 수"] = len(caps["commands"])
            fields = set()
            for c in caps["commands"]:
                fields.update(c.get("recordFields") or [])
            out["봉투 recordFields 유니크"] = len(fields)
        except ValueError:
            pass
    tests = glob.glob(os.path.join(ROOT, "tests", "*contract*.rs"))
    out["계약 가드 파일"] = len(tests)
    fn = 0
    for t in tests:
        fn += io.open(t, encoding="utf-8", errors="replace").read().count("#[test]")
    out["계약 가드 테스트 함수"] = fn
    skills = os.path.join(ROOT, ".claude", "skills")
    if os.path.isdir(skills):
        out["스킬 수"] = len([d for d in os.listdir(skills)
                           if os.path.isdir(os.path.join(skills, d))])
    km = os.path.join(ROOT, "mydocs", "manual", "agent_knowledge_map.md")
    if os.path.exists(km):
        for line in io.open(km, encoding="utf-8"):
            if "전수 사전 — " in line:
                out["지식지도 사전 필드"] = line.split("— ")[1].split("개")[0].strip()
                break
    codex = os.path.join(ROOT, "mydocs", "manual", "agent_codex")
    if os.path.isdir(codex):
        lines = 0
        for f in glob.glob(os.path.join(codex, "*.md")):
            lines += sum(1 for _ in io.open(f, encoding="utf-8", errors="replace"))
        out["대전(Codex) 줄 수"] = lines
    return out


def table(rows, headers):
    md = ["| " + " | ".join(headers) + " |",
          "|" + "|".join("---" for _ in headers) + "|"]
    for r in rows:
        md.append("| " + " | ".join(str(x) for x in r) + " |")
    return "\n".join(md)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ref", default="origin/devel")
    ap.add_argument("--who", default="kevin9327")
    ap.add_argument("--out", default=None)
    a = ap.parse_args()
    today = datetime.date.today().isoformat()
    head = sh(["git", "rev-parse", "--short", a.ref]).strip()
    since30 = (datetime.date.today() - datetime.timedelta(days=30)).isoformat()

    all_paths = [p for _, ps in AXIS for p in ps]
    md = []
    md.append(f"# 에이전트 축 주도 실측 — {today} (기준 {a.ref} @ {head})")
    md.append("")
    md.append("> `tools/agent_dominance_metrics.py` 가 git 병합 이력과 바이너리")
    md.append("> 자기서술에서 산출했다. 모집단은 devel 병합분(열린 PR 불포함),")
    md.append("> 귀속은 git author. 같은 커밋에서 재실행하면 같은 숫자가 나온다.")
    md.append("")

    md.append("## 1. 총괄 — 에이전트 축 전체 (경로군 합집합)")
    md.append("")
    commits, added = git_stats(a.ref, all_paths)
    mc, tc, pc = share(commits, a.who)
    ma, ta, pa = share(added, a.who)
    md.append(table([
        ("커밋", f"{mc:,}", f"{tc:,}", f"**{pc:.0f}%**"),
        ("추가 줄", f"{ma:,}", f"{ta:,}", f"**{pa:.0f}%**"),
    ], ["지표", a.who, "전체", "점유"]))
    md.append("")
    top = sorted(added.items(), key=lambda kv: -kv[1])[:5]
    md.append("상위 기여자 (추가 줄): " +
              " · ".join(f"{k} {v:,}" for k, v in top))
    md.append("")

    md.append("## 2. 경로군별 점유")
    md.append("")
    rows = []
    for name, paths in AXIS:
        c, ad = git_stats(a.ref, paths)
        mc, tc, pc = share(c, a.who)
        ma, ta, pa = share(ad, a.who)
        rows.append((name, f"{mc}/{tc} ({pc:.0f}%)", f"{ma:,}/{ta:,} ({pa:.0f}%)"))
    md.append(table(rows, ["경로군", "커밋 (점유)", "추가 줄 (점유)"]))
    md.append("")
    md.append("경로군 정의(원문): " + json.dumps(dict(AXIS), ensure_ascii=False))
    md.append("")

    md.append(f"## 3. 최근 30일 동적 (--since {since30})")
    md.append("")
    c30, a30 = git_stats(a.ref, all_paths, since=since30)
    mc, tc, pc = share(c30, a.who)
    ma, ta, pa = share(a30, a.who)
    md.append(table([
        ("커밋", f"{mc:,}", f"{tc:,}", f"**{pc:.0f}%**"),
        ("추가 줄", f"{ma:,}", f"{ta:,}", f"**{pa:.0f}%**"),
    ], ["지표(30일)", a.who, "전체", "점유"]))
    md.append("")

    md.append("## 4. 표면 절대 수치 (바이너리·저장소 실측)")
    md.append("")
    sm = surface_metrics()
    md.append(table(sorted(sm.items()), ["표면", "수치"]))
    md.append("")
    md.append("## 5. 읽는 법 (정직 조항)")
    md.append("")
    md.append("- 점유율은 **작업량 귀속**이지 가치 서열이 아니다 — 리뷰·머지 판단은")
    md.append("  메인테이너의 몫이며 이 표에 잡히지 않는다.")
    md.append("- 경로군 밖 기여(렌더러·파서 본체)는 이 축의 모집단이 아니다.")
    md.append("- 외부 도구와의 비교는 원리로만 말한다: 자기서술(capabilities)·")
    md.append("  봉투 계약·출처 표지·검증 사다리·표류 가드·살아있는 교본을 함께")
    md.append("  갖춘 문서 CLI 관행은 표준이 아니다 — 이 표면 자체가 차별점이다.")
    text = "\n".join(md) + "\n"

    out = a.out or os.path.join(ROOT, "mydocs", "report", f"agent_metrics_{today}.md")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    fm = ("---\nkind: report\nstatus: active\n"
          f"canonical: mydocs/report/{os.path.basename(out)}\n"
          f"last_verified: {today}\n---\n\n")
    io.open(out, "w", encoding="utf-8", newline="\n").write(fm + text)
    print(f"보고서: {out}")
    print(f"헤드라인(30일): 커밋 {mc}/{tc} ({pc:.0f}%) · 추가 줄 {ma:,}/{ta:,} ({pa:.0f}%)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
