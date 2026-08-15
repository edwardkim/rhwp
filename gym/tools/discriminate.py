"""gym 판별력 감사 — 각 과제가 "일 안 한 제출"을 실제로 거부하는가(약한 오라클 색출).

## 왜 이 도구인가 (프론티어를 선제 차단)

2026 벤치마크의 최대 위기는 **false-pass**다: OpenAI 감사에서 SWE-Bench Verified
최난도 과제의 59.4%가 버그를 안 고쳐도 테스트가 통과했다(약한 오라클). 채점이
'일을 했나'가 아니라 '파일이 있나' 만 보면, 아무것도 안 한 제출도 만점을 받는다.

이 감사기는 그 결함을 gym 에 **못 들어오게** 막는다 — 사후에 손으로 발견하는 대신,
각 과제에 **음성 대조**(일 안 한 제출)를 넣어 채점해 **반드시 실패**하는지 본다.

음성 대조 구성:
- **answer 과제** — 모든 답 키에 명백한 오답(sentinel). answer_eq 가 진값과 대조하니
  거부해야 한다.
- **artifact 과제** — 입력을 산출물로 그대로 복사하는 대조와 1KiB synthetic garbage
  대조를 모두 실행한다. `differs_from_input`만으로는 garbage가 통과할 수 있으므로
  형식·핵심값 검사도 함께 요구한다.

음성 대조에 **통과하는** 과제 = 판별력 없는 약한 오라클(false-pass). 이걸 리포트한다.
통과 못 하면(=거부) 그 과제는 진짜 일을 요구하는 것이다.

## 사용

    python gym/tools/discriminate.py --bin target/debug/rhwp        # 전 과제 판별 감사
    python gym/tools/discriminate.py --bin target/debug/rhwp --json
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = os.path.dirname(HERE)
REPO_ROOT = os.path.dirname(GYM_ROOT)
sys.path.insert(0, GYM_ROOT)

from core import runner  # noqa: E402

# 진값과 절대 같을 리 없는 오답 — 숫자 진값엔 문자열이라 타입부터 다르고, 문자열
# 진값엔 이 특이 문자열이라 값이 다르다. answer_eq 는 어느 쪽이든 거부한다.
WRONG_SENTINEL = "__NEGATIVE_CONTROL_definitely_wrong__"
GARBAGE_BYTES = (b"RHWP_GYM_GARBAGE_NEGATIVE_CONTROL\x00" * 64)


def answer_keys(task: dict) -> set[str]:
    keys = set()
    for check in task.get("checks", []):
        if check.get("answer"):
            keys.add(check["answer"])
    return keys


def build_negative(task: dict, neg_pack_dir: str, artifact_mode: str = "input-copy") -> None:
    """음성 대조 제출물 — 오답 answer.json + artifact별 무편집/garbage 대조."""
    sub_dir = os.path.join(neg_pack_dir, task["id"])
    shutil.rmtree(sub_dir, ignore_errors=True)
    os.makedirs(sub_dir, exist_ok=True)

    keys = answer_keys(task)
    if keys:
        with open(os.path.join(sub_dir, "answer.json"), "w", encoding="utf-8") as fh:
            json.dump({k: WRONG_SENTINEL for k in keys}, fh, ensure_ascii=False)

    submit = task.get("submit", {})
    if submit.get("kind") == "artifact":
        src = os.path.join(REPO_ROOT, task["input"])
        for rel in submit.get("files", []):
            dst = os.path.join(sub_dir, rel)
            os.makedirs(os.path.dirname(dst) or sub_dir, exist_ok=True)
            if artifact_mode == "input-copy" and os.path.isfile(src):
                shutil.copyfile(src, dst)   # 무편집 복사 = 일 안 함
            elif artifact_mode == "garbage":
                with open(dst, "wb") as fh:
                    fh.write(GARBAGE_BYTES)
            else:
                raise ValueError(f"지원하지 않는 artifact 음성 대조: {artifact_mode}")


def discriminate(bin_path: str, gym_root: str, neg_root: str) -> dict:
    packs_dir = os.path.join(gym_root, "packs")
    results = []
    false_pass = []
    false_pass_controls = []
    task_count = 0
    for pack_id in sorted(os.listdir(packs_dir)):
        pack_dir = os.path.join(packs_dir, pack_id)
        tasks_dir = os.path.join(pack_dir, "tasks")
        if not os.path.isdir(tasks_dir):
            continue
        neg_pack_dir = os.path.join(neg_root, pack_id)
        for name in sorted(os.listdir(tasks_dir)):
            if not name.endswith(".json"):
                continue
            with open(os.path.join(tasks_dir, name), encoding="utf-8") as fh:
                task = json.load(fh)
            task_count += 1
            artifact = task.get("submit", {}).get("kind") == "artifact"
            controls = ("input-copy", "garbage") if artifact else ("wrong-answer",)
            for control in controls:
                control_pack_dir = os.path.join(neg_root, control, pack_id)
                build_negative(task, control_pack_dir, artifact_mode=control)
                result = runner.score_task(task, control_pack_dir, bin_path)
                # 음성 대조는 '실패'(=거부) 해야 판별력이 있다. pass=True 면 약한 오라클.
                discriminates = not result.get("pass")
                results.append({"pack": pack_id, "task": task["id"], "control": control,
                                "discriminates": discriminates})
                if not discriminates:
                    label = f"{pack_id}/{task['id']}"
                    if label not in false_pass:
                        false_pass.append(label)
                    false_pass_controls.append(f"{label} ({control})")
    return {
        "kind": "gymDiscrimination",
        "schemaVersion": "1.0",
        "ok": len(false_pass) == 0,
        "taskCount": task_count,
        "controlCount": len(results),
        "discriminating": task_count - len(false_pass),
        "falsePass": false_pass,
        "falsePassControls": false_pass_controls,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="gym 판별력 감사 — 약한 오라클(false-pass) 색출")
    ap.add_argument("--bin", required=True)
    ap.add_argument("--json", action="store_true")
    a = ap.parse_args()
    bin_path = runner.find_bin(a.bin)
    neg_root = os.path.join(GYM_ROOT, "submissions", "_negative_control")
    shutil.rmtree(neg_root, ignore_errors=True)
    report = discriminate(bin_path, GYM_ROOT, neg_root)
    if a.json:
        sys.stdout.write(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    elif report["ok"]:
        print(f"gym 판별력 감사: {report['taskCount']} 과제 전부 음성 대조를 거부 — 약한 오라클 0")
    else:
        print(f"gym 판별력 감사: 약한 오라클(false-pass) {len(report['falsePass'])}건 — "
              "일 안 한 제출이 통과한다:")
        for t in report["falsePass"]:
            print(f"  - {t}")
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
