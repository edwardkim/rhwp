"""[#4653] pack·task 스키마와 재현성 선언.

## pack manifest (`packs/<id>/pack.json`)

```json
{
  "schemaVersion": "1.0",
  "kind": "gymPack",
  "id": "table-editing",
  "title": "표 편집",
  "axis": "편집 (좌표 지정)",
  "requires": { "commands": ["export-tables", "edit", "table-to-csv"] },
  "runner": { "rhwpVersion": "0.8.3", "rhwpCommit": "…", "capabilitiesSha256": "…" }
}
```

`requires.commands` 는 이 pack 을 채점하려면 바이너리에 있어야 하는 명령이다.
없으면 **0점이 아니라 `unavailable`** 로 보고한다 — 부재를 실패로 위장하지
않는 것이 이 저장소의 결이고, pack 이 늘어날수록 이 구분이 중요해진다
(오래된 바이너리로 신규 pack 을 돌린 사람에게 "너는 0점"이라고 말하면 거짓말이다).

`runner` 는 **기준 실행의 신원**이다. 점수는 바이너리마다 달라질 수 있으므로
"이 점수가 어느 바이너리에서 났는가"를 pack 과 스코어카드 양쪽에 남긴다.
"""

import hashlib
import json
import os
import subprocess

PACK_KIND = "gymPack"
PROFILE_KIND = "gymProfile"
SCHEMA_VERSION = "1.0"

#: 과제 파일의 필수 키.
TASK_REQUIRED = ("id", "tier", "title", "input", "instructions", "submit", "checks")

#: 편집 과제 — 전역 훑기 연산자를 금지한다(#4600 재발 방지).
EDITING_AXES = ("편집", "보안")


def _fail(errors, where, message):
    errors.append(f"{where}: {message}")


def validate_pack(manifest, pack_dir, errors):
    where = os.path.basename(pack_dir)
    if manifest.get("kind") != PACK_KIND:
        _fail(errors, where, f"kind 가 {PACK_KIND} 가 아니다")
    if manifest.get("schemaVersion") != SCHEMA_VERSION:
        _fail(errors, where, f"schemaVersion 이 {SCHEMA_VERSION} 이 아니다")
    if manifest.get("id") != where:
        _fail(errors, where, f"pack id({manifest.get('id')}) 가 폴더 이름과 다르다")
    for key in ("title", "axis"):
        if not manifest.get(key):
            _fail(errors, where, f"{key} 가 비었다")
    requires = manifest.get("requires", {})
    if not isinstance(requires.get("commands"), list) or not requires["commands"]:
        _fail(errors, where, "requires.commands 가 비었다 — 요구 capability 선언은 필수")
    runner = manifest.get("runner", {})
    for key in ("rhwpVersion", "rhwpCommit", "capabilitiesSha256"):
        if not runner.get(key):
            _fail(errors, where, f"runner.{key} 가 비었다 — 기준 실행 신원 선언은 필수")


def validate_task(task, pack, known_commands, errors):
    where = f"{pack.get('id')}/{task.get('id')}"
    for key in TASK_REQUIRED:
        if key not in task:
            _fail(errors, where, f"필수 키 없음: {key}")
    if not isinstance(task.get("tier"), int) or not 1 <= task.get("tier", 0) <= 3:
        _fail(errors, where, "tier 는 1~3 정수")
    if not task.get("checks"):
        _fail(errors, where, "checks 가 비었다")

    from . import checks as check_registry

    editing = any(task.get("axis", pack.get("axis", "")).startswith(a) for a in EDITING_AXES)
    for check in task.get("checks", []):
        op = check.get("op")
        if op not in check_registry.REGISTRY:
            _fail(errors, where, f"미등록 연산자: {op}")
            continue
        if editing and op in check_registry.GLOBAL_SCAN_OPS and not check.get("allowGlobalScan"):
            _fail(errors, where,
                  f"편집 과제에 전역 훑기 연산자({op}) — 좌표를 지목하는 연산자를 쓰거나 "
                  "allowGlobalScan 으로 사유를 명시하라(#4600)")
        cmd = check.get("cmd")
        if check_registry.needs_cli(op):
            if not cmd:
                _fail(errors, where, f"{op} 는 cmd 가 필요하다")
            elif known_commands is not None and cmd[0] not in known_commands:
                _fail(errors, where, f"CLI 에 없는 명령: {cmd[0]}")
        elif cmd:
            _fail(errors, where, f"{op} 는 CLI 를 부르지 않는데 cmd 가 있다")


def validate_profile(profile, pack_ids, errors):
    where = f"profiles/{profile.get('id')}"
    if profile.get("kind") != PROFILE_KIND:
        _fail(errors, where, f"kind 가 {PROFILE_KIND} 가 아니다")
    if not profile.get("packs"):
        _fail(errors, where, "packs 가 비었다")
    for pid in profile.get("packs", []):
        if pid not in pack_ids:
            _fail(errors, where, f"없는 pack 참조: {pid}")


def capabilities_digest(bin_path):
    """`rhwp capabilities` 원문의 sha256 — 명령 표면의 지문."""
    proc = subprocess.run([bin_path, "capabilities"], capture_output=True)
    raw = proc.stdout
    return hashlib.sha256(raw).hexdigest(), raw


def known_commands(bin_path):
    _, raw = capabilities_digest(bin_path)
    try:
        return {c["name"] for c in json.loads(raw.decode("utf-8"))["commands"]}
    except (ValueError, KeyError):
        return None


def runner_identity(bin_path, repo_root):
    """실행 시점 신원 — pack 의 `runner` 선언과 대조할 값."""
    digest, raw = capabilities_digest(bin_path)
    version = ""
    try:
        version = json.loads(raw.decode("utf-8")).get("version", "")
    except ValueError:
        pass
    commit = ""
    try:
        commit = subprocess.run(["git", "rev-parse", "HEAD"], cwd=repo_root,
                                capture_output=True).stdout.decode("utf-8").strip()
    except OSError:
        pass
    return {"rhwpVersion": version, "rhwpCommit": commit, "capabilitiesSha256": digest}
