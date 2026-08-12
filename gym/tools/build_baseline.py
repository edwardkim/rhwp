"""[#4653] 기준 풀이 왕복 — 베이스라인 제출물을 기계적으로 생성한다.

## 왜 필요한가

과제를 손으로 늘리면 "돌아가지 않는 과제" 가 섞인다. pack 이 8개로 늘어나는
순간 그 위험은 8배가 된다. 그래서 pack 마다 `reference/<과제ID>.json` 에
**기준 풀이**를 두고, 이 스크립트가 그것을 실행해 제출물을 만든 뒤 곧바로
채점한다. 신규 과제는 이 왕복을 통과해야만 등재된다 — 즉 **저장소에 들어간
모든 과제는 풀 수 있음이 실측된 과제**다.

기준 풀이는 정답 노출이므로 `reference/` 로 분리해 명시한다(기존
`baselines/*/answer.json` 과 같은 성격이다). 과제를 푸는 에이전트는 이 폴더를
보지 않는 것이 규칙이고, 보더라도 측정되는 것은 "스스로 경로를 찾는 능력"이
아니게 될 뿐 채점은 정직하게 돌아간다.

## 기준 풀이 형식

```json
{
  "id": "TE01",
  "steps": [
    { "run": ["edit", "replace-text", "{input}", "--find", "규제",
              "--replace", "점검", "-o", "{sub:edited.hwp}", "--json"] },
    { "answer": { "remaining": { "cmd": ["search", "{sub:edited.hwp}", "--json", "--", "규제"],
                                 "path": "matchCount" } } }
  ]
}
```

- `run` — rhwp 를 실행한다. `{input}`(과제 입력)·`{sub:이름}`(제출 폴더 안 경로)
  자리표를 쓴다. `allowExits` 로 판정성 종료 코드를 허용한다.
- `answer` — 봉투에서 값을 길어 `answer.json` 에 합친다(라이브 재계산).
- `copy` — 과제 입력이나 자산을 제출 폴더로 복사한다.

사용:
  python gym/tools/build_baseline.py --agent claude-fable-5 [--pack <id>] [--bin <경로>]
"""

import argparse
import io
import json
import os
import shutil
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

for stream in (sys.stdout, sys.stderr):
    try:
        stream.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

from gym.core import runner  # noqa: E402
from gym.core.checks import dig  # noqa: E402

ROOT = runner.ROOT
PACKS_DIR = runner.PACKS_DIR


def resolve(token, task, sub_dir):
    if token == "{input}":
        return task["input"]
    if token.startswith("{sub:") and token.endswith("}"):
        path = os.path.join(sub_dir, token[5:-1])
        # 중첩 제출 경로(capsules/… 등)를 미리 만든다 — 기준 풀이가 폴더 생성까지
        # 신경 쓰게 하면 풀이가 절차 잡음으로 지저분해진다.
        os.makedirs(os.path.dirname(path), exist_ok=True)
        return path
    if "{sub:" in token:
        # 계획서 JSON 처럼 자리표가 문자열 안에 박힌 경우 — 한 문자열에 여러 개가
        # 있을 수 있다(다세대 계획서는 input·output 을 모두 {sub:} 로 가리킨다).
        # 첫 하나만 바꾸면 나머지가 리터럴로 남아 엉뚱한 이름의 파일이 생긴다(#4664).
        out = []
        rest = token
        while "{sub:" in rest:
            head, rest = rest.split("{sub:", 1)
            name, rest = rest.split("}", 1)
            path = os.path.join(sub_dir, name)
            os.makedirs(os.path.dirname(path) or sub_dir, exist_ok=True)
            out.append(head + path.replace("\\", "\\\\"))
        out.append(rest)
        return "".join(out)
    return token


def run_step(bin_path, args, task, sub_dir, allow_exits):
    resolved = [resolve(a, task, sub_dir) for a in args]
    proc = subprocess.run([bin_path] + resolved, cwd=ROOT, capture_output=True)
    out = proc.stdout.decode("utf-8", errors="replace")
    if proc.returncode not in allow_exits:
        raise RuntimeError(
            f"기준 풀이 실패 (exit {proc.returncode}, 허용 {allow_exits}): "
            f"{' '.join(resolved[:4])}\n  {proc.stderr.decode('utf-8', 'replace')[:300]}")
    try:
        return json.loads(out)
    except ValueError:
        return None


def build_task(bin_path, pack_id, task, reference, sub_root):
    sub_dir = os.path.join(sub_root, pack_id, task["id"])
    shutil.rmtree(sub_dir, ignore_errors=True)
    os.makedirs(sub_dir, exist_ok=True)
    answer = {}
    for step in reference["steps"]:
        if "run" in step:
            run_step(bin_path, step["run"], task, sub_dir, step.get("allowExits", [0]))
        elif "copy" in step:
            src = os.path.join(ROOT, resolve(step["copy"]["from"], task, sub_dir))
            shutil.copyfile(src, resolve(step["copy"]["to"], task, sub_dir))
        elif "write_json" in step:
            # 과제가 요구하는 부속 파일(정책·명세서 등)을 기준 풀이가 직접 쓴다.
            spec = step["write_json"]
            path = resolve(spec["to"], task, sub_dir)
            body = json.loads(json.dumps(spec["body"]).replace(
                "{input}", task["input"].replace("\\", "/")))
            io.open(path, "w", encoding="utf-8", newline="\n").write(
                json.dumps(body, ensure_ascii=False, indent=2) + "\n")
        elif "keyring_from" in step:
            # 발급한 키의 공개키로 키링을 조립한다 — 서명 과제의 채점 재료.
            spec = step["keyring_from"]
            with io.open(resolve(spec["key"], task, sub_dir), encoding="utf-8") as fh:
                key = json.load(fh)
            keyring = {"schemaVersion": "1.0", "kind": "keyring",
                       "keys": [{"keyId": spec["keyId"], "publicKey": key["publicKey"],
                                 "revoked": None}]}
            io.open(resolve(spec["out"], task, sub_dir), "w", encoding="utf-8",
                    newline="\n").write(json.dumps(keyring, ensure_ascii=False, indent=2) + "\n")
        elif "answer" in step:
            for key, spec in step["answer"].items():
                if "const" in spec:
                    answer[key] = spec["const"]
                    continue
                env = run_step(bin_path, spec["cmd"], task, sub_dir,
                               spec.get("allowExits", [0]))
                if env is None:
                    raise RuntimeError(f"{task['id']}: 답안 봉투 파싱 실패")
                value = dig(env, spec.get("path", ""))
                # 개수를 묻는 과제(len_answer_eq)는 배열이 아니라 길이가 답이다.
                answer[key] = len(value) if spec.get("len") else value
        else:
            raise RuntimeError(f"{task['id']}: 알 수 없는 기준 풀이 단계 {list(step)}")
    if answer:
        io.open(os.path.join(sub_dir, "answer.json"), "w", encoding="utf-8",
                newline="\n").write(json.dumps(answer, ensure_ascii=False, indent=2) + "\n")
    return sub_dir


def verify_built_task(bin_path, pack_id, task, sub_root):
    """방금 만든 제출물을 같은 pack 경로에서 실제 채점한다."""
    result = runner.score_task(task, os.path.join(sub_root, pack_id), bin_path)
    if result.get("pass"):
        return None
    if result.get("error"):
        return f"{pack_id}/{task['id']}: {result['error']}"
    failed = []
    for check in result.get("checks", []):
        if not check.get("ok"):
            name = check.get("name", check.get("op", "검사"))
            failed.append(f"{name}: {check.get('error', '판정 불일치')}")
    return f"{pack_id}/{task['id']}: " + "; ".join(failed or ["채점 실패"])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--agent", default="claude-fable-5")
    ap.add_argument("--pack", action="append", default=None)
    ap.add_argument("--bin", default=None)
    a = ap.parse_args()

    bin_path = runner.find_bin(a.bin)
    sub_root = os.path.join(runner.GYM, "submissions", a.agent)
    pack_ids = a.pack or runner.discover_packs()

    built = failed = skipped = 0
    for pack_id in pack_ids:
        ref_dir = os.path.join(PACKS_DIR, pack_id, "reference")
        if not os.path.isdir(ref_dir):
            print(f"[{pack_id}] 기준 풀이 없음 — 건너뜀")
            continue
        _manifest, tasks = runner.load_pack(pack_id)
        for task in tasks:
            ref_path = os.path.join(ref_dir, f"{task['id']}.json")
            if not os.path.exists(ref_path):
                skipped += 1
                continue
            with io.open(ref_path, encoding="utf-8") as fh:
                reference = json.load(fh)
            try:
                build_task(bin_path, pack_id, task, reference, sub_root)
                failure = verify_built_task(bin_path, pack_id, task, sub_root)
                if failure:
                    failed += 1
                    print(f"  X {failure}")
                else:
                    built += 1
            except (RuntimeError, OSError, KeyError, IndexError, TypeError) as e:
                failed += 1
                print(f"  X {pack_id}/{task['id']}: {e}")
    print(f"기준 풀이 왕복: 성공 {built} · 실패 {failed} · 기준 풀이 없음 {skipped}")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
