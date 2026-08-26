"""[#4659] 위조 불가능한 리더보드 — 점수판을 검증 사다리 위에 세운다.

## 착상

AI 벤치마크 리더보드의 병폐는 점수의 신뢰다: 수치는 자기 신고이고, 소급 수정이
가능하고, 같은 결과의 재등재를 막을 방법이 없다. 이 저장소에는 그 문제의 해답
전체가 이미 있다 — 검증 사다리. 그렇다면 **운동장이 자기 사다리 위에서 돌면
된다**: 사다리를 시험하는 운동장의 점수 자체가 사다리로 봉인되는 자기 순환.
새 암호학 0줄 — 전부 기존 rhwp 명령의 조합이다.

## 등재 사슬

```
scorecard.json + admission.json          (채점기가 발급)
  → keygen                               (에이전트 신원 — 공개키만 커밋)
  → settle propose --sign-key            (명세서·스코어카드·입장 봉투 3해시 고정 + Ed25519 서명)
  → settle record --ledger               (append-only 원장 — 같은 스코어카드 재등재 불가, P3)
  → anchor add + anchor checkpoint       (투명성 로그 + 머클 에폭, 5년 축)
```

| 공격 | 막는 축 |
|---|---|
| 점수 위조(스코어카드 수정) | 청구의 capsuleSha256 고정 (P1) |
| 소급 조작(과거 항목 수정) | 원장·앵커의 줄 해시 체인 — 다음 줄이 폭로 |
| 이중 등재(같은 결과 재탕) | 원장 전역 capsuleSha256 유일성 (P3) |
| 대리 제출 | 청구 Ed25519 서명 + keyring 판정 (4년 축) |

## 정직 조항

- 이 사슬이 봉인하는 것은 "이 스코어카드가 이 시점에 이 신원으로 등재되었고
  이후 변조되지 않았다" 까지다. **채점 자체의 재현**은 스코어카드에 박힌 runner
  신원(version·commit·capabilities digest)과 커밋된 제출물로 제3자가 수행한다.
- render 는 검증을 통과한 원장 항목만 순위에 올린다. 검증 불가 항목은 숨기지
  않고 unverified 로 표기한다 — 부재를 실패로 위장하지 않는 결 그대로.

사용:
  python gym/tools/leaderboard.py attest --agent <이름>   # 등재 (채점 후)
  python gym/tools/leaderboard.py verify                  # 전 사슬 재검증
  python gym/tools/leaderboard.py render                  # 검증본에서 순위표 생성
  python gym/tools/leaderboard.py invite --agent <이름>   # 초대장 (판 지문)

I/O·JSON·사슬·CLI 실패는 분류된 LeaderboardError 로 접힌다. 새 암호 원시함수는
없다 — sha256 은 hashlib, 서명은 rhwp settle/keygen 이 한다. 규약 정본은
`gym/docs/leaderboard.md`, 운영 메모는 `gym/tools/README_leaderboard.md`.
"""

import argparse
import hashlib
import io
import json
import os
import subprocess
import sys
from collections.abc import Mapping, Sequence

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

for stream in (sys.stdout, sys.stderr):
    try:
        stream.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

from gym.core import runner  # noqa: E402

ROOT = runner.ROOT
GYM = runner.GYM
BOARD = os.path.join(GYM, "leaderboard")
KEYS = os.path.join(BOARD, "keys")          # 비밀키 — 커밋 금지(.gitignore)
CLAIMS = os.path.join(BOARD, "claims")
LEDGER = os.path.join(BOARD, "ledger.ndjson")
ANCHOR = os.path.join(BOARD, "anchor.ndjson")
CHECKPOINT = os.path.join(BOARD, "checkpoint.json")
KEYRING = os.path.join(BOARD, "keyring.json")
WORKORDER = os.path.join(BOARD, "workorder.json")

SCHEMA_VERSION = "1.0"
INVITE_KIND = "gymLeaderboardInvite"
VERIFICATION_KIND = "gymLeaderboardVerification"
DEFAULT_GUEST = "친구-에이전트"
JOIN_STEP_COUNT = 3
COMMIT_SHORT = 10
MODES = ("attest", "verify", "render", "invite")

# 초대장·지문 키. 시험이 이 집합을 계약으로 고정한다. 새 암호 필드 없음.
FINGERPRINT_KEYS = (
    "members",
    "ledgerEntries",
    "ledgerChain",
    "anchorChain",
    "merkleRoot",
    "workorderSha256",
    "ledgerSnapshotSha256",
)
INVITE_KEYS = (
    "schemaVersion",
    "kind",
    "guest",
    "board",
    "fingerprint",
    "join",
    "promise",
    "note",
)
INVITE_BOARD_KEYS = ("repo", "path")
RANK_OK_KEYS = ("ok", "agent", "score", "max", "seq", "runner")
PACK_SCORE_KEYS = ("score", "max")
ERROR_CODES = ("io", "format", "chain", "cli", "schema", "leaderboard")


class LeaderboardError(Exception):
    """분류된 리더보드 실패. 암호 실패가 아니라 I/O·형식·사슬·CLI 오류다."""

    code = "leaderboard"

    def __init__(self, message, *, path=None, cause=None):
        super().__init__(message)
        self.path = path
        self.cause = cause

    def as_dict(self):
        body = {"ok": False, "code": self.code, "why": str(self)}
        if self.path:
            body["path"] = self.path
        if self.cause is not None:
            body["causeType"] = type(self.cause).__name__
        return body


class LeaderboardIOError(LeaderboardError):
    code = "io"


class LeaderboardFormatError(LeaderboardError):
    code = "format"


class LeaderboardChainError(LeaderboardError):
    code = "chain"


class LeaderboardCliError(LeaderboardError):
    code = "cli"


class LeaderboardSchemaError(LeaderboardError):
    code = "schema"


def classify_exception(exc):
    """예외를 보고 코드로 접는다. 모르는 타입은 leaderboard."""
    if isinstance(exc, LeaderboardError):
        return exc.code
    if isinstance(exc, (FileNotFoundError, NotADirectoryError, PermissionError,
                        IsADirectoryError)):
        return "io"
    if isinstance(exc, OSError):
        return "io"
    if isinstance(exc, json.JSONDecodeError):
        return "format"
    if isinstance(exc, UnicodeError):
        return "format"
    if isinstance(exc, (TypeError, ValueError, KeyError, AttributeError, IndexError)):
        return "schema"
    if isinstance(exc, (subprocess.SubprocessError, FileNotFoundError)):
        return "cli"
    return "leaderboard"


def format_classified(exc, prefix="실패"):
    code = classify_exception(exc)
    path = getattr(exc, "path", None)
    loc = f" ({path})" if path else ""
    return f"{prefix} ({code}){loc}: {exc}"


def empty_fingerprint():
    """빈 판의 지문. 파일이 없어도 invite 가 죽지 않게 한다."""
    return {
        "members": 0,
        "ledgerEntries": 0,
        "ledgerChain": "ok",
        "anchorChain": "ok",
        "merkleRoot": None,
        "workorderSha256": None,
        "ledgerSnapshotSha256": None,
    }


def sha256_of(path):
    """파일 바이트의 SHA-256 hex. hashlib 만 쓴다 — 새 암호 원시함수 없음."""
    h = hashlib.sha256()
    try:
        with open(path, "rb") as f:
            for chunk in iter(lambda: f.read(1 << 20), b""):
                h.update(chunk)
    except OSError as e:
        raise LeaderboardIOError(f"sha256 실패: {e}", path=path, cause=e) from e
    return h.hexdigest()


def try_sha256(path):
    """sha256_of 의 비상승 거울. (digest|None, err|None)."""
    try:
        return sha256_of(path), None
    except LeaderboardError as e:
        return None, str(e)


def run_cli(bin_path, args, ok=(0,)):
    if not bin_path:
        raise LeaderboardCliError("rhwp 바이너리 경로가 비어 있다")
    try:
        proc = subprocess.run([bin_path] + list(args), cwd=ROOT, capture_output=True)
    except OSError as e:
        raise LeaderboardCliError(f"rhwp 실행 실패: {e}", cause=e) from e
    out = proc.stdout.decode("utf-8", errors="replace")
    if proc.returncode not in ok:
        raise SystemExit(f"rhwp {' '.join(list(args)[:3])} exit {proc.returncode}: "
                         f"{proc.stderr.decode('utf-8', 'replace')[:300]}")
    try:
        return proc.returncode, json.loads(out)
    except ValueError:
        return proc.returncode, None


def read_json(path):
    try:
        with io.open(path, encoding="utf-8") as fh:
            return json.load(fh)
    except OSError as e:
        raise LeaderboardIOError(f"JSON 읽기 실패: {e}", path=path, cause=e) from e
    except json.JSONDecodeError as e:
        raise LeaderboardFormatError(f"JSON 파싱 실패: {e}", path=path, cause=e) from e
    except UnicodeError as e:
        raise LeaderboardFormatError(f"JSON 인코딩 실패: {e}", path=path, cause=e) from e


def write_json(path, body):
    try:
        text = json.dumps(body, ensure_ascii=False, indent=2) + "\n"
    except (TypeError, ValueError) as e:
        raise LeaderboardFormatError(f"JSON 직렬화 실패: {e}", path=path, cause=e) from e
    try:
        parent = os.path.dirname(path)
        if parent:
            os.makedirs(parent, exist_ok=True)
        with io.open(path, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(text)
    except OSError as e:
        raise LeaderboardIOError(f"JSON 쓰기 실패: {e}", path=path, cause=e) from e


def try_read_json(path):
    """read_json 의 비상승 거울. (obj|None, err|None)."""
    try:
        return read_json(path), None
    except LeaderboardError as e:
        return None, str(e)


def try_write_json(path, body):
    """write_json 의 비상승 거울. err|None."""
    try:
        write_json(path, body)
        return None
    except LeaderboardError as e:
        return str(e)


def safe_listdir(path):
    """os.listdir 의 비상승 거울. (names, err|None). 없으면 빈 목록."""
    try:
        if not os.path.isdir(path):
            return [], None if not os.path.exists(path) else f"디렉터리가 아님: {path}"
        return sorted(os.listdir(path)), None
    except OSError as e:
        return [], f"목록 실패: {e}"


def chain_walk(path, kind):
    """ndjson 줄 해시 체인 검증 — anchor_log::load_kind 의 파이썬 거울.

    (rhwp 에는 settlementLedger 를 단독 검증하는 CLI 가 없어 여기서 같은
    알고리즘을 거울로 둔다. 규약: prevEntryHash = 직전 줄 원문 바이트의 sha256,
    seq 연번, kind 고정.)

    읽기·JSON 실패는 예외를 올리지 않고 (지금까지의 항목, 이유 문자열) 을
    돌려준다. 사슬 검증이 죽으면 폭로가 아니라 침묵이 된다.
    """
    if not path:
        return [], "경로가 비어 있다"
    try:
        if not os.path.exists(path):
            return [], None
    except OSError as e:
        return [], f"존재 확인 실패: {e}"
    entries, prev = [], None
    try:
        with io.open(path, encoding="utf-8") as fh:
            raw = fh.read()
    except OSError as e:
        return [], f"읽기 실패: {e}"
    except UnicodeError as e:
        return [], f"인코딩 실패: {e}"
    for i, line in enumerate(raw.splitlines()):
        if not line.strip():
            continue
        try:
            entry = json.loads(line)
        except json.JSONDecodeError as e:
            return entries, f"{i+1}행: JSON 파싱 실패 ({e.msg})"
        if not isinstance(entry, dict):
            return entries, f"{i+1}행: 객체가 아님"
        if entry.get("kind") != kind:
            return entries, f"{i+1}행: kind {entry.get('kind')} != {kind}"
        if entry.get("seq") != len(entries):
            return entries, f"{i+1}행: seq 연번 위반"
        if entry.get("prevEntryHash") != prev:
            return entries, f"{i+1}행: prevEntryHash 불일치 (append-only 위반)"
        prev = hashlib.sha256(line.encode("utf-8")).hexdigest()
        entries.append(entry)
    return entries, None


def merkle_root(leaf_hashes):
    """anchor 의 머클 규약 거울 — 잎은 줄 바이트 해시, 홀수 층은 마지막 복제.

    잎은 hex 문자열이어야 한다. 다른 타입은 SchemaError — 조용히 str() 하면
    다른 머클이 된다.
    """
    if not leaf_hashes:
        return None
    if isinstance(leaf_hashes, (str, bytes)):
        raise LeaderboardSchemaError("merkle 잎은 문자열 목록이어야 한다")
    try:
        level = list(leaf_hashes)
    except TypeError as e:
        raise LeaderboardSchemaError(f"merkle 잎이 순회 가능하지 않다: {e}") from e
    if any(not isinstance(x, str) for x in level):
        raise LeaderboardSchemaError("merkle 잎은 hex 문자열이어야 한다")
    while len(level) > 1:
        nxt = []
        for i in range(0, len(level), 2):
            a = level[i]
            b = level[i + 1] if i + 1 < len(level) else a
            nxt.append(hashlib.sha256((a + b).encode("ascii")).hexdigest())
        level = nxt
    return level[0]


def line_hashes(path):
    if not path:
        return []
    try:
        if not os.path.exists(path):
            return []
        with io.open(path, encoding="utf-8") as fh:
            return [hashlib.sha256(line.encode("utf-8")).hexdigest()
                    for line in fh.read().splitlines() if line.strip()]
    except (OSError, UnicodeError) as e:
        raise LeaderboardIOError(f"줄 해시 실패: {e}", path=path, cause=e) from e


def try_line_hashes(path):
    try:
        return line_hashes(path), None
    except LeaderboardError as e:
        return [], str(e)


def default_workorder():
    return {
        "schemaVersion": SCHEMA_VERSION, "kind": "workorder",
        "workorderId": "gym-leaderboard-standing",
        "title": "운동장 상설 발주 — 전 pack 채점 결과의 등재",
        "acceptancePolicy": {
            "schemaVersion": SCHEMA_VERSION, "kind": "admissionPolicy", "default": "deny",
            "rules": [],
            "note": "입장 판정은 채점기의 gymAdmission 봉투가 담당한다",
        },
        "unitPrice": {"amount": "0", "currency": "KRW", "per": "scorecard"},
    }


def default_keyring():
    return {"schemaVersion": SCHEMA_VERSION, "kind": "keyring", "keys": []}


def ensure_board():
    try:
        os.makedirs(KEYS, exist_ok=True)
        os.makedirs(CLAIMS, exist_ok=True)
    except OSError as e:
        raise LeaderboardIOError(f"점수판 디렉터리 생성 실패: {e}", path=BOARD, cause=e) from e
    if not os.path.exists(WORKORDER):
        write_json(WORKORDER, default_workorder())
    if not os.path.exists(KEYRING):
        write_json(KEYRING, default_keyring())


def cmd_attest(a, bin_path):
    try:
        return _cmd_attest_body(a, bin_path)
    except SystemExit:
        raise
    except LeaderboardError as e:
        raise SystemExit(format_classified(e, "attest 실패")) from e
    except (OSError, ValueError, TypeError) as e:
        raise SystemExit(format_classified(e, "attest 실패")) from e


def _cmd_attest_body(a, bin_path):
    ensure_board()
    if not getattr(a, "agent", None):
        raise SystemExit("attest 는 --agent 가 필요하다")
    sub = os.path.join(GYM, "submissions", a.agent)
    scorecard = os.path.join(sub, "scorecard.json")
    admission = os.path.join(sub, "admission.json")
    for f in (scorecard, admission):
        if not os.path.exists(f):
            raise SystemExit(f"없음: {f} — 먼저 `python gym/score.py --agent {a.agent}` 를 돌려라")
    adm = read_json(admission)
    if not isinstance(adm, dict):
        raise SystemExit("입장 봉투가 객체가 아니다 — 등재 불가")
    if adm.get("verdict") != "allow":
        raise SystemExit(f"입장 봉투 verdict={adm.get('verdict')} — 등재 불가")

    # 1) 신원 키 (한 번만 발급, 공개키를 keyring 에 등재)
    key = os.path.join(KEYS, f"{a.agent}.key.json")
    key_id = f"gym/{a.agent}"
    if not os.path.exists(key):
        run_cli(bin_path, ["keygen", "--key-id", key_id, "--out", key, "--json"])
        key_body = read_json(key)
        if not isinstance(key_body, dict) or "publicKey" not in key_body:
            raise LeaderboardSchemaError("keygen 산출에 publicKey 가 없다", path=key)
        pub = key_body["publicKey"]
        ring = read_json(KEYRING)
        if not isinstance(ring, dict):
            raise LeaderboardSchemaError("keyring 이 객체가 아니다", path=KEYRING)
        ring.setdefault("keys", [])
        if not isinstance(ring["keys"], list):
            raise LeaderboardSchemaError("keyring.keys 가 목록이 아니다", path=KEYRING)
        ring["keys"].append({"keyId": key_id, "publicKey": pub, "revoked": None})
        write_json(KEYRING, ring)

    # 2) 서명 청구 — 명세서·스코어카드·입장 봉투 3해시 고정
    epoch = len(chain_walk(LEDGER, "settlementLedger")[0])
    claim = os.path.join(CLAIMS, f"{a.agent}-{epoch:04d}.claim.json")
    run_cli(bin_path, ["settle", "propose",
                       "--workorder", WORKORDER, "--capsule", scorecard,
                       "--gate-envelope", admission,
                       "--sign-key", key, "-o", claim, "--json"])

    # 3) 원장 기입 — 같은 스코어카드는 두 번 못 들어온다 (P3).
    #
    # 기입을 **먼저** 시도한다. 거부되면 이번 attest 의 부산물(claim·보존본)을
    # 남기지 않는다 — 원장이 유일한 진실이지만, 미등재 파일이 굴러다니면 사람이
    # 헷갈린다(첫 실증에서 실제로 0001 잔여물이 생겼다).
    code, env = run_cli(bin_path, ["settle", "record", claim, "--ledger", LEDGER, "--json"],
                        ok=(0, 3))
    if code == 3:
        os.remove(claim)
        sidecar = claim + ".sig.json"
        if os.path.exists(sidecar):
            os.remove(sidecar)
        raise SystemExit(f"원장 거부(이중 등재): {json.dumps(env, ensure_ascii=False)[:200]}")

    # 등재 확정 후에만 스코어카드·입장 봉투를 보존한다(검증이 커밋본을 참조).
    keep_dir = os.path.join(BOARD, "scorecards", f"{a.agent}-{epoch:04d}")
    os.makedirs(keep_dir, exist_ok=True)
    for src in (scorecard, admission):
        dst = os.path.join(keep_dir, os.path.basename(src))
        with open(src, "rb") as fi, open(dst, "wb") as fo:
            fo.write(fi.read())

    # 4) 투명성 로그 + 머클 에폭.
    #
    # claim 에 이어 **원장 파일 자체의 스냅샷 해시**를 등재한다 — 원장의 꼬리
    # 줄은 다음 줄이 생기기 전까지 체인이 봉인하지 못한다(5년 축의 알려진 한계).
    # 첫 공격 실증에서 이 한계가 실제로 뚫렸다: 항목이 1개일 때 원장 소급 수정이
    # verify 를 통과했다. 스냅샷 등재 규약: attest 마다 (claim, 원장 스냅샷) 이
    # 짝으로 앵커에 오르고, 검증은 "앵커의 마지막 항목 == 현재 원장 바이트 해시"
    # 를 요구한다.
    run_cli(bin_path, ["anchor", "add", claim, "--log", ANCHOR, "--json"])
    run_cli(bin_path, ["anchor", "add", LEDGER, "--log", ANCHOR, "--json"])
    run_cli(bin_path, ["anchor", "checkpoint", "--log", ANCHOR, "-o", CHECKPOINT, "--json"])

    print(f"등재 완료 — {a.agent} (epoch {epoch})")
    print(f"  claim   {os.path.relpath(claim, ROOT)}")
    print(f"  ledger  seq {epoch} · anchor checkpoint 갱신")
    return 0


def verify_entry(bin_path, entry, ledger_entries):
    """원장 항목 하나의 전 사슬 재검증 — 판정은 전부 데이터.

    I/O·형식 실패는 예외를 올리지 않고 ok=False + why 로 접는다. 한 항목의
    읽기 실패가 전체 verify 를 죽이면 폭로가 아니라 침묵이다.
    """
    if not isinstance(entry, dict):
        return {"ok": False, "seq": None, "why": "원장 항목이 객체가 아님"}
    result = {"seq": entry.get("seq"), "claimSha256": str(entry.get("claimSha256", ""))[:16]}
    names, list_err = safe_listdir(CLAIMS)
    if list_err:
        result.update({"ok": False, "why": f"claims 목록 실패: {list_err}"})
        return result
    claim_path = None
    for name in names:
        p = os.path.join(CLAIMS, name)
        digest, digest_err = try_sha256(p)
        if digest_err or digest is None:
            continue
        if digest == entry.get("claimSha256"):
            claim_path = p
            break
    if claim_path is None:
        result.update({"ok": False, "why": "claim 파일 없음(원장 해시와 일치하는 파일 부재)"})
        return result
    claim, claim_err = try_read_json(claim_path)
    if claim_err or not isinstance(claim, dict):
        result.update({"ok": False, "why": f"claim 읽기 실패: {claim_err or '객체가 아님'}"})
        return result
    # 원장 항목과 claim 의 교차 대조 — 원장의 capsuleSha256 을 바꿔치기해도
    # claim 파일이 남아 있으면 여기서 폭로된다(첫 공격 실증이 잡은 구멍).
    cross = claim.get("capsuleSha256") == entry.get("capsuleSha256")
    agent_epoch = os.path.basename(claim_path).replace(".claim.json", "")
    keep = os.path.join(BOARD, "scorecards", agent_epoch)
    scorecard = os.path.join(keep, "scorecard.json")
    admission = os.path.join(keep, "admission.json")
    adm_body, _adm_err = try_read_json(admission) if os.path.exists(admission) else (None, None)
    if isinstance(adm_body, dict):
        result["agent"] = adm_body.get("agent", "?")
    else:
        result["agent"] = "?"

    # ① 3해시 고정 + 서명 (rhwp 가 판정)
    try:
        _code, env = run_cli(bin_path, ["settle", "verify", claim_path,
                                        "--workorder", WORKORDER, "--capsule", scorecard,
                                        "--gate-envelope", admission,
                                        "--keyring", KEYRING, "--json"], ok=(0, 3))
    except (LeaderboardError, SystemExit) as e:
        result.update({"ok": False, "why": f"settle verify 실패: {e}"})
        return result
    checks = {
        "pin.workorder": bool(env and env.get("workorderOk")),
        "pin.scorecard": bool(env and env.get("capsuleOk")),
        "pin.admission": bool(env and env.get("gateOk")),
        "admission.allow": bool(env and env.get("gateVerdict") == "allow"),
        "signature": bool(env and env.get("signerOk")),
        "ledger.crossPin": cross,
    }
    # ② 투명성 로그 등재 + 로그 체인 (rhwp 가 판정)
    try:
        _code, aenv = run_cli(bin_path, ["anchor", "verify", claim_path,
                                         "--log", ANCHOR, "--json"], ok=(0, 3))
    except (LeaderboardError, SystemExit) as e:
        result.update({"ok": False, "why": f"anchor verify 실패: {e}", "checks": checks})
        return result
    checks["anchor.logged"] = bool(aenv and aenv.get("logged"))
    checks["anchor.chain"] = bool(aenv and aenv.get("logChainOk"))
    result["checks"] = checks
    result["ok"] = all(checks.values())
    if result["ok"]:
        card, card_err = try_read_json(scorecard)
        if card_err or not isinstance(card, dict):
            result.update({"ok": False, "why": f"scorecard 읽기 실패: {card_err or '객체가 아님'}"})
            return result
        total = card.get("total") if isinstance(card.get("total"), dict) else {}
        result["score"] = total.get("score")
        result["max"] = total.get("max")
        result["runner"] = card.get("runner") if isinstance(card.get("runner"), dict) else {}
        packs = card.get("packs") if isinstance(card.get("packs"), list) else []
        result["packs"] = {p["id"]: {"score": p.get("score"), "max": p["max"]}
                           for p in packs
                           if isinstance(p, dict) and p.get("status") == "scored" and "id" in p and "max" in p}
    return result


def cmd_verify(a, bin_path):
    try:
        return _cmd_verify_body(a, bin_path)
    except SystemExit:
        raise
    except LeaderboardError as e:
        print(format_classified(e, "verify 실패"), file=sys.stderr)
        return 2
    except (OSError, ValueError, TypeError) as e:
        print(format_classified(e, "verify 실패"), file=sys.stderr)
        return 2


def _cmd_verify_body(a, bin_path):
    ensure_board()
    entries, err = chain_walk(LEDGER, "settlementLedger")
    print(f"원장 체인: {len(entries)}항목 · {'무결' if err is None else '파손: ' + err}")
    aentries, aerr = chain_walk(ANCHOR, "anchorLog")
    print(f"앵커 체인: {len(aentries)}항목 · {'무결' if aerr is None else '파손: ' + aerr}")
    # 원장 꼬리 봉인 — 앵커의 마지막 항목은 attest 규약상 원장 스냅샷이다.
    ledger_digest, ledger_digest_err = try_sha256(LEDGER) if os.path.exists(LEDGER) else (None, None)
    snapshot_ok = bool(aentries) and ledger_digest is not None and (
        aentries[-1].get("capsuleSha256") == ledger_digest)
    if ledger_digest_err:
        print(f"원장 스냅샷 해시 실패: {ledger_digest_err}")
    print(f"원장 스냅샷 봉인: {'일치' if snapshot_ok else '불일치!! (원장이 앵커 이후 변경됨)'}")
    if os.path.exists(CHECKPOINT):
        cp, cp_err = try_read_json(CHECKPOINT)
        if cp_err or not isinstance(cp, dict):
            print(f"체크포인트: 읽기 실패 — {cp_err or '객체가 아님'}")
        else:
            leaves, leaves_err = try_line_hashes(ANCHOR)
            if leaves_err:
                print(f"체크포인트: 앵커 줄 해시 실패 — {leaves_err}")
            else:
                try:
                    up_to = int(cp.get("upToSeq", 0))
                except (TypeError, ValueError):
                    up_to = 0
                root = merkle_root(leaves[:up_to + 1])
                match = root == cp.get("merkleRoot")
                print(f"체크포인트: upToSeq {cp.get('upToSeq')} · 머클 루트 "
                      f"{'재계산 일치' if match else '불일치!!'}")
    results = [verify_entry(bin_path, e, entries) for e in entries]
    ok = sum(1 for r in results if r.get("ok"))
    print(f"항목 검증: {ok}/{len(results)} 통과")
    for r in results:
        mark = "O" if r.get("ok") else "X"
        bad = [k for k, v in r.get("checks", {}).items() if not v]
        print(f"  {mark} seq{r.get('seq')} {str(r.get('agent', '?')):20} "
              + (f"{r.get('score')}/{r.get('max')}" if r.get("ok") else f"실패: {bad or r.get('why')}"))
    write_err = try_write_json(os.path.join(BOARD, "verification.json"),
                               {"kind": VERIFICATION_KIND, "schemaVersion": SCHEMA_VERSION,
                                "ledgerEntries": len(entries), "ledgerChain": err or "ok",
                                "anchorChain": aerr or "ok", "ledgerSnapshotSealed": snapshot_ok,
                                "verified": ok, "results": results})
    if write_err:
        print(f"verification.json 쓰기 실패: {write_err}", file=sys.stderr)
        return 2
    return 0 if (err is None and aerr is None and snapshot_ok and ok == len(results)) else 3


def board_fingerprint():
    """지금 이 점수판의 지문 — 신참이 '진짜 판'에 합류하는지 확인할 값.

    초대장을 받은 친구는 이 지문을 커밋된 원장·앵커에서 스스로 재계산해,
    자기 키를 걸기 전에 판이 위조본이 아님을 확인한다. 새 비밀 0 — 전부
    커밋된 파일에서 나온다. 읽기 실패는 예외를 올리지 않고 지문 필드에 접는다.
    """
    fp = empty_fingerprint()
    ledger_entries, lerr = chain_walk(LEDGER, "settlementLedger")
    _anchor_entries, aerr = chain_walk(ANCHOR, "anchorLog")
    fp["ledgerEntries"] = len(ledger_entries)
    fp["ledgerChain"] = lerr or "ok"
    fp["anchorChain"] = aerr or "ok"
    ring, ring_err = (try_read_json(KEYRING) if os.path.exists(KEYRING)
                      else ({"keys": []}, None))
    if ring_err or not isinstance(ring, dict):
        fp["members"] = 0
    else:
        keys = ring.get("keys")
        fp["members"] = len(keys) if isinstance(keys, list) else 0
    checkpoint, _cp_err = (try_read_json(CHECKPOINT) if os.path.exists(CHECKPOINT)
                           else ({}, None))
    if isinstance(checkpoint, dict):
        fp["merkleRoot"] = checkpoint.get("merkleRoot")
    wo, wo_err = try_sha256(WORKORDER) if os.path.exists(WORKORDER) else (None, None)
    fp["workorderSha256"] = wo if not wo_err else None
    snap, snap_err = try_sha256(LEDGER) if os.path.exists(LEDGER) else (None, None)
    fp["ledgerSnapshotSha256"] = snap if not snap_err else None
    return fp


def validate_fingerprint(fp):
    """지문 봉투 위반 목록. 비어 있으면 스키마 통과."""
    issues = []
    if not isinstance(fp, dict):
        return ["지문이 객체가 아니다"]
    for key in FINGERPRINT_KEYS:
        if key not in fp:
            issues.append(f"지문에 {key} 가 없다")
    if "members" in fp and not isinstance(fp["members"], int):
        issues.append("members 가 int 가 아니다")
    if "ledgerEntries" in fp and not isinstance(fp["ledgerEntries"], int):
        issues.append("ledgerEntries 가 int 가 아니다")
    for chain_key in ("ledgerChain", "anchorChain"):
        if chain_key in fp and not isinstance(fp[chain_key], str):
            issues.append(f"{chain_key} 가 문자열이 아니다")
    for hash_key in ("workorderSha256", "ledgerSnapshotSha256", "merkleRoot"):
        val = fp.get(hash_key)
        if val is not None and not isinstance(val, str):
            issues.append(f"{hash_key} 가 문자열도 None 도 아니다")
        if isinstance(val, str) and hash_key != "merkleRoot" and len(val) != 64:
            issues.append(f"{hash_key} 길이 {len(val)} != 64")
    return issues


def construct_invite(guest):
    """초대장 봉투 — 판 지문과 합류 3줄. 쓰기는 호출자가 한다."""
    if guest is None or (isinstance(guest, str) and not guest.strip()):
        guest = DEFAULT_GUEST
    if not isinstance(guest, str):
        raise LeaderboardSchemaError(f"손님 이름이 문자열이 아니다: {type(guest).__name__}")
    fp = board_fingerprint()
    return {
        "schemaVersion": SCHEMA_VERSION, "kind": INVITE_KIND,
        "guest": guest,
        "board": {"repo": "edwardkim/rhwp", "path": "gym/leaderboard"},
        "fingerprint": fp,
        "join": [
            f"python gym/score.py --agent {guest}",
            f"python gym/tools/leaderboard.py attest --agent {guest}",
            "python gym/tools/leaderboard.py verify",
        ],
        "promise": (
            "너의 비밀키는 gym/leaderboard/keys/ 에만 남고 커밋되지 않는다"
            "(.gitignore). 점수판에 오르는 것은 공개키·서명·스코어카드뿐이다."),
        "note": (
            "이 초대는 권한이 아니라 안내다. 문은 이미 열려 있다 — attest 는 "
            "누구의 이름이든 받는다. 초대장은 네가 합류하는 판이 위조본이 "
            "아님을 fingerprint 로 확인하라는 뜻이다."),
    }


def validate_invite(invite):
    """초대장 봉투 위반 목록. 암호 검증이 아니라 스키마·합류 3줄 계약이다."""
    issues = []
    if not isinstance(invite, dict):
        return ["초대장이 객체가 아니다"]
    for key in INVITE_KEYS:
        if key not in invite:
            issues.append(f"초대장에 {key} 가 없다")
    if invite.get("schemaVersion") != SCHEMA_VERSION:
        issues.append(f"schemaVersion {invite.get('schemaVersion')!r} != {SCHEMA_VERSION!r}")
    if invite.get("kind") != INVITE_KIND:
        issues.append(f"kind {invite.get('kind')!r} != {INVITE_KIND!r}")
    if "guest" in invite and not isinstance(invite["guest"], str):
        issues.append("guest 가 문자열이 아니다")
    board = invite.get("board")
    if board is not None:
        if not isinstance(board, dict):
            issues.append("board 가 객체가 아니다")
        else:
            for key in INVITE_BOARD_KEYS:
                if key not in board:
                    issues.append(f"board.{key} 가 없다")
    join = invite.get("join")
    if join is not None:
        if not isinstance(join, list):
            issues.append("join 이 목록이 아니다")
        elif len(join) != JOIN_STEP_COUNT:
            issues.append(f"join 길이 {len(join)} != {JOIN_STEP_COUNT}")
        else:
            if not any("score.py" in str(s) for s in join):
                issues.append("join 에 score.py 가 없다")
            if not any("attest" in str(s) for s in join):
                issues.append("join 에 attest 가 없다")
            if not any("verify" in str(s) for s in join):
                issues.append("join 에 verify 가 없다")
    if "fingerprint" in invite:
        issues.extend(f"fingerprint.{i}" for i in validate_fingerprint(invite["fingerprint"]))
    return issues


def cmd_invite(a, bin_path):
    """친구 초대장 발급 — 외부 에이전트를 이름으로 점수판에 부른다.

    리더보드는 처음부터 문이 열려 있다(attest 는 --agent 하나만 받는다). 이
    명령은 그 열린 문에 **초대장**을 붙일 뿐이다: 지금 판의 지문과, 신참이
    자기 신원으로 합류하는 3줄 절차를 한 봉투로 묶는다. 초대는 권한이 아니라
    안내다 — 아무나 스스로 등재할 수 있고, 초대장은 '어디로 오면 되는지'를
    가리킨다.
    """
    try:
        ensure_board()
        guest = getattr(a, "agent", None) or DEFAULT_GUEST
        invite = construct_invite(guest)
        issues = validate_invite(invite)
        if issues:
            raise LeaderboardSchemaError("초대장 스키마 위반: " + "; ".join(issues))
        fp = invite["fingerprint"]
        out = os.path.join(BOARD, "invite.json")
        write_err = try_write_json(out, invite)
        if write_err:
            raise LeaderboardIOError(write_err, path=out)
        print(f"초대장 발급 → {os.path.relpath(out, ROOT)}")
        print(f"  손님: {guest}")
        print(f"  판 지문: 멤버 {fp['members']} · 원장 {fp['ledgerEntries']}항목 "
              f"· 사슬 {fp['ledgerChain']}/{fp['anchorChain']}")
        print("  합류 3줄:")
        for step in invite["join"]:
            print(f"    $ {step}")
        return 0
    except LeaderboardError as e:
        print(format_classified(e, "invite 실패"), file=sys.stderr)
        return 2
    except (OSError, ValueError, TypeError) as e:
        print(format_classified(e, "invite 실패"), file=sys.stderr)
        return 2


def validate_rank_row(row, require_ok=True):
    """순위 행 위반 목록. 렌더가 죽지 않게 하기 위한 스키마이지 암호 검증이 아니다."""
    issues = []
    if not isinstance(row, Mapping):
        return ["행이 객체가 아니다"]
    if require_ok and not row.get("ok"):
        issues.append("ok 가 참이 아니다")
    for key in ("score", "seq"):
        if key not in row:
            issues.append(f"{key} 가 없다")
        elif rank_key(row) is None and key in row:
            issues.append(f"{key} 를 숫자로 읽지 못한다")
    if require_ok:
        for key in RANK_OK_KEYS:
            if key not in row:
                issues.append(f"{key} 가 없다")
    packs = row.get("packs")
    if packs is not None and not isinstance(packs, Mapping):
        issues.append("packs 가 객체가 아니다")
    return issues


def pack_ratio(pk):
    """pack 항목의 (score/max, max). 결손·비숫자·NaN 은 None — ZeroDivision 없음."""
    if not isinstance(pk, Mapping):
        return None
    if "score" not in pk or "max" not in pk:
        return None
    score = _finite_float(pk["score"])
    mx = _finite_float(pk["max"])
    if score is None or mx is None:
        return None
    if mx == 0:
        return (0.0, 0.0)
    return (score / mx, mx)


def _finite_float(value):
    try:
        number = float(value)
    except (TypeError, ValueError):
        return None
    if number != number or number in (float("inf"), float("-inf")):
        return None
    return number


def rank_key(row):
    """검증 행의 정렬 키 (-score, seq). 결손·NaN 이면 None → unverified 로 접는다."""
    if not isinstance(row, Mapping):
        return None
    score = _finite_float(row.get("score"))
    if score is None or "seq" not in row:
        return None
    try:
        seq = int(row["seq"])
    except (TypeError, ValueError):
        return None
    return (-score, seq)


def rank_results(results):
    """검증된 항목만 (-score, seq) 로 순위. ok=False 는 unverified 로 분리.

    ok=True 인데 score/seq 가 없으면 순위에 올리지 않고 unverified 로 접는다.
    목록이 아니면 둘 다 빈 목록. 행이 객체가 아니면 가짜 unverified 한 줄을 남긴다.
    """
    if not isinstance(results, Sequence) or isinstance(results, (str, bytes)):
        return [], []
    ranked_src = []
    unverified = []
    for r in results:
        if not isinstance(r, Mapping):
            unverified.append({"ok": False, "seq": None, "why": "행이 객체가 아님"})
            continue
        if not r.get("ok"):
            unverified.append(r)
            continue
        key = rank_key(r)
        if key is None:
            bad = dict(r)
            bad["ok"] = False
            bad.setdefault("why", "score/seq 결손")
            unverified.append(bad)
            continue
        ranked_src.append((key, r))
    ranked_src.sort(key=lambda t: t[0])
    return [row for _key, row in ranked_src], unverified


def best_pack(packs):
    """만점 비율이 가장 높은 pack. 동률이면 max 가 큰 쪽. 없으면 None.

    cmd_render 가 쓰던 키와 같다: (score/max 또는 max=0 이면 0, max).
    반환은 (pack_id, score, max). 결손·비객체 pack 은 건너뛴다.
    """
    if not packs or not isinstance(packs, Mapping):
        return None
    scored = []
    for pid, pk in packs.items():
        ratio = pack_ratio(pk)
        if ratio is None:
            continue
        scored.append((ratio[0], ratio[1], pid, pk.get("score"), pk.get("max")))
    if not scored:
        return None
    _ratio, _mx, pid, score, mx = max(scored, key=lambda t: (t[0], t[1]))
    return (pid, score, mx)


def short_commit(run, n=COMMIT_SHORT):
    """runner.rhwpCommit 의 짧은 표기. 결손이면 이모지 대시가 아니라 — ."""
    if not isinstance(run, Mapping):
        return "—"
    commit = run.get("rhwpCommit")
    if not isinstance(commit, str) or not commit:
        return "—"
    return f"`{commit[:n]}`"


def render_markdown(results, err, ranked=None):
    """순위표 마크다운 문자열. 쓰기는 호출자가 한다.

    헤더·검증 행·**unverified** 행·정직 조항을 항상 포함한다. ranked 를
    넘기지 않으면 rank_results 로 계산한다. runner/score 결손은 칸을 — 로 남긴다.
    """
    if not isinstance(results, Sequence) or isinstance(results, (str, bytes)):
        results = []
    if ranked is None:
        ranked, unverified = rank_results(results)
    else:
        unverified = [r for r in results if isinstance(r, Mapping) and not r.get("ok")]
        if not isinstance(ranked, Sequence) or isinstance(ranked, (str, bytes)):
            ranked = []
    lines = ["# 운동장 리더보드 — 위조 불가능한 점수판", "",
             "모든 순위는 검증 사슬(3해시 고정·Ed25519 서명·append-only 원장·머클 앵커)을",
             "**렌더 시점에 재검증**한 항목만 오른다. 재현 방법:",
             "`python gym/tools/leaderboard.py verify`", "",
             "| 순위 | 에이전트 | 총점 | 최강 능력 | commit | seq | 사슬 |",
             "|---|---|---|---|---|---|---|"]
    for i, r in enumerate(ranked, 1):
        if not isinstance(r, Mapping):
            continue
        run = r.get("runner") if isinstance(r.get("runner"), Mapping) else {}
        best = best_pack(r.get("packs") or {})
        best_txt = f"{best[0]} {best[1]}/{best[2]}" if best else "—"
        agent = r.get("agent", "?")
        score = r.get("score", "—")
        mx = r.get("max", "—")
        seq = r.get("seq", "—")
        lines.append(f"| {i} | {agent} | **{score} / {mx}** "
                     f"| {best_txt} | {short_commit(run)} "
                     f"| {seq} | 검증됨 |")
    for r in unverified:
        if not isinstance(r, Mapping):
            continue
        seq = r.get("seq", "—")
        lines.append(f"| — | seq {seq} | — | — | — | {seq} | **unverified** |")

    pack_ids = sorted({
        pid for r in ranked
        if isinstance(r, Mapping) and isinstance(r.get("packs"), Mapping)
        for pid in r.get("packs", {})
    })
    if pack_ids and len(ranked) > 1:
        lines += ["", "## 능력 격자 (pack 별 점수)", "",
                  "| 에이전트 | " + " | ".join(pack_ids) + " |",
                  "|---|" + "---|" * len(pack_ids)]
        for r in ranked:
            if not isinstance(r, Mapping):
                continue
            packs = r.get("packs") if isinstance(r.get("packs"), Mapping) else {}
            cells = []
            for pid in pack_ids:
                pk = packs.get(pid)
                if not isinstance(pk, Mapping):
                    cells.append("—")
                elif pack_ratio(pk) is None:
                    cells.append("—")
                elif pk.get("score") == pk.get("max"):
                    cells.append(f"**{pk.get('score')}**")
                else:
                    cells.append(f"{pk.get('score')}/{pk.get('max')}")
            lines.append(f"| {r.get('agent', '?')} | " + " | ".join(cells) + " |")
        lines.append("")
        lines.append("`—` = 미제출(그 pack 을 아예 풀지 않음) · **굵게** = 만점")
    lines += ["",
              f"원장 체인: {'무결' if err is None else '파손'} · 항목 {len(results)} · "
              f"검증 {len(ranked)} · unverified {len(unverified)}", "",
              "정직 조항: 이 사슬이 봉인하는 것은 \"이 스코어카드가 이 시점에 이 신원으로",
              "등재되었고 이후 변조되지 않았다\" 까지다. 채점 자체의 재현은 스코어카드의",
              "runner 신원(version·commit·capabilities digest)과 커밋된 제출물로 제3자가 수행한다."]
    return "\n".join(lines) + "\n"


def cmd_render(a, bin_path):
    try:
        entries, err = chain_walk(LEDGER, "settlementLedger")
        results = [verify_entry(bin_path, e, entries) for e in entries]
        ranked, unverified = rank_results(results)
        text = render_markdown(results, err, ranked=ranked)
        out = os.path.join(BOARD, "leaderboard.md")
        parent = os.path.dirname(out)
        if parent:
            os.makedirs(parent, exist_ok=True)
        with io.open(out, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(text)
        print(f"리더보드 렌더 → {os.path.relpath(out, ROOT)} "
              f"(검증 {len(ranked)}·unverified {len(unverified)})")
        return 0
    except LeaderboardError as e:
        print(format_classified(e, "render 실패"), file=sys.stderr)
        return 2
    except (OSError, ValueError, TypeError) as e:
        print(format_classified(e, "render 실패"), file=sys.stderr)
        return 2


def resolve_bin(explicit):
    """runner.find_bin 의 분류 거울. 없으면 CliError."""
    try:
        path = runner.find_bin(explicit)
    except SystemExit as e:
        raise LeaderboardCliError(str(e) or "rhwp 바이너리를 찾지 못했다") from e
    except Exception as e:  # noqa: BLE001 — CLI 입구에서만 분류
        raise LeaderboardCliError(f"바이너리 탐색 실패: {e}", cause=e) from e
    if not path:
        raise LeaderboardCliError("rhwp 바이너리 경로가 비어 있다")
    return path


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("mode", choices=list(MODES))
    ap.add_argument("--agent", default=None)
    ap.add_argument("--bin", default=None)
    a = ap.parse_args(argv)
    try:
        # invite 는 바이너리가 필요 없다. 지문은 커밋된 파일에서만 나온다.
        if a.mode == "invite":
            bin_path = None
        else:
            bin_path = resolve_bin(a.bin)
    except LeaderboardError as e:
        print(format_classified(e, "바이너리"), file=sys.stderr)
        return 2
    if a.mode == "attest":
        if not a.agent:
            raise SystemExit("attest 는 --agent 가 필요하다")
        return cmd_attest(a, bin_path)
    if a.mode == "verify":
        return cmd_verify(a, bin_path)
    if a.mode == "invite":
        return cmd_invite(a, bin_path)
    return cmd_render(a, bin_path)


if __name__ == "__main__":
    sys.exit(main())
