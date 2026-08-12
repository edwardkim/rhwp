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
GYM = runner.GYM
BOARD = os.path.join(GYM, "leaderboard")
KEYS = os.path.join(BOARD, "keys")          # 비밀키 — 커밋 금지(.gitignore)
CLAIMS = os.path.join(BOARD, "claims")
LEDGER = os.path.join(BOARD, "ledger.ndjson")
ANCHOR = os.path.join(BOARD, "anchor.ndjson")
CHECKPOINT = os.path.join(BOARD, "checkpoint.json")
KEYRING = os.path.join(BOARD, "keyring.json")
WORKORDER = os.path.join(BOARD, "workorder.json")


def sha256_of(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def run_cli(bin_path, args, ok=(0,)):
    proc = subprocess.run([bin_path] + args, cwd=ROOT, capture_output=True)
    out = proc.stdout.decode("utf-8", errors="replace")
    if proc.returncode not in ok:
        raise SystemExit(f"rhwp {' '.join(args[:3])} exit {proc.returncode}: "
                         f"{proc.stderr.decode('utf-8', 'replace')[:300]}")
    try:
        return proc.returncode, json.loads(out)
    except ValueError:
        return proc.returncode, None


def read_json(path):
    with io.open(path, encoding="utf-8") as fh:
        return json.load(fh)


def write_json(path, body):
    with io.open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(json.dumps(body, ensure_ascii=False, indent=2) + "\n")


def chain_walk(path, kind):
    """ndjson 줄 해시 체인 검증 — anchor_log::load_kind 의 파이썬 거울.

    (rhwp 에는 settlementLedger 를 단독 검증하는 CLI 가 없어 여기서 같은
    알고리즘을 거울로 둔다. 규약: prevEntryHash = 직전 줄 원문 바이트의 sha256,
    seq 연번, kind 고정.)
    """
    if not os.path.exists(path):
        return [], None
    entries, prev = [], None
    with io.open(path, encoding="utf-8") as fh:
        for i, line in enumerate(fh.read().splitlines()):
            if not line.strip():
                continue
            entry = json.loads(line)
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
    """anchor 의 머클 규약 거울 — 잎은 줄 바이트 해시, 홀수 층은 마지막 복제."""
    if not leaf_hashes:
        return None
    level = list(leaf_hashes)
    while len(level) > 1:
        nxt = []
        for i in range(0, len(level), 2):
            a = level[i]
            b = level[i + 1] if i + 1 < len(level) else a
            nxt.append(hashlib.sha256((a + b).encode("ascii")).hexdigest())
        level = nxt
    return level[0]


def line_hashes(path):
    if not os.path.exists(path):
        return []
    with io.open(path, encoding="utf-8") as fh:
        return [hashlib.sha256(line.encode("utf-8")).hexdigest()
                for line in fh.read().splitlines() if line.strip()]


def ensure_board():
    os.makedirs(KEYS, exist_ok=True)
    os.makedirs(CLAIMS, exist_ok=True)
    if not os.path.exists(WORKORDER):
        write_json(WORKORDER, {
            "schemaVersion": "1.0", "kind": "workorder",
            "workorderId": "gym-leaderboard-standing",
            "title": "운동장 상설 발주 — 전 pack 채점 결과의 등재",
            "acceptancePolicy": {
                "schemaVersion": "1.0", "kind": "admissionPolicy", "default": "deny",
                "rules": [],
                "note": "입장 판정은 채점기의 gymAdmission 봉투가 담당한다",
            },
            "unitPrice": {"amount": "0", "currency": "KRW", "per": "scorecard"},
        })
    if not os.path.exists(KEYRING):
        write_json(KEYRING, {"schemaVersion": "1.0", "kind": "keyring", "keys": []})


def cmd_attest(a, bin_path):
    ensure_board()
    sub = os.path.join(GYM, "submissions", a.agent)
    scorecard = os.path.join(sub, "scorecard.json")
    admission = os.path.join(sub, "admission.json")
    for f in (scorecard, admission):
        if not os.path.exists(f):
            raise SystemExit(f"없음: {f} — 먼저 `python gym/score.py --agent {a.agent}` 를 돌려라")
    adm = read_json(admission)
    if adm.get("verdict") != "allow":
        raise SystemExit(f"입장 봉투 verdict={adm.get('verdict')} — 등재 불가")

    # 1) 신원 키 (한 번만 발급, 공개키를 keyring 에 등재)
    key = os.path.join(KEYS, f"{a.agent}.key.json")
    key_id = f"gym/{a.agent}"
    if not os.path.exists(key):
        run_cli(bin_path, ["keygen", "--key-id", key_id, "--out", key, "--json"])
        pub = read_json(key)["publicKey"]
        ring = read_json(KEYRING)
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
    """원장 항목 하나의 전 사슬 재검증 — 판정은 전부 데이터."""
    result = {"seq": entry["seq"], "claimSha256": entry.get("claimSha256", "")[:16]}
    claim_path = None
    for name in sorted(os.listdir(CLAIMS)):
        p = os.path.join(CLAIMS, name)
        if sha256_of(p) == entry.get("claimSha256"):
            claim_path = p
            break
    if claim_path is None:
        result.update({"ok": False, "why": "claim 파일 없음(원장 해시와 일치하는 파일 부재)"})
        return result
    claim = read_json(claim_path)
    # 원장 항목과 claim 의 교차 대조 — 원장의 capsuleSha256 을 바꿔치기해도
    # claim 파일이 남아 있으면 여기서 폭로된다(첫 공격 실증이 잡은 구멍).
    cross = claim.get("capsuleSha256") == entry.get("capsuleSha256")
    agent_epoch = os.path.basename(claim_path).replace(".claim.json", "")
    keep = os.path.join(BOARD, "scorecards", agent_epoch)
    scorecard = os.path.join(keep, "scorecard.json")
    admission = os.path.join(keep, "admission.json")
    result["agent"] = read_json(admission).get("agent") if os.path.exists(admission) else "?"

    # ① 3해시 고정 + 서명 (rhwp 가 판정)
    code, env = run_cli(bin_path, ["settle", "verify", claim_path,
                                   "--workorder", WORKORDER, "--capsule", scorecard,
                                   "--gate-envelope", admission,
                                   "--keyring", KEYRING, "--json"], ok=(0, 3))
    checks = {
        "pin.workorder": bool(env and env.get("workorderOk")),
        "pin.scorecard": bool(env and env.get("capsuleOk")),
        "pin.admission": bool(env and env.get("gateOk")),
        "admission.allow": bool(env and env.get("gateVerdict") == "allow"),
        "signature": bool(env and env.get("signerOk")),
        "ledger.crossPin": cross,
    }
    # ② 투명성 로그 등재 + 로그 체인 (rhwp 가 판정)
    code, aenv = run_cli(bin_path, ["anchor", "verify", claim_path,
                                    "--log", ANCHOR, "--json"], ok=(0, 3))
    checks["anchor.logged"] = bool(aenv and aenv.get("logged"))
    checks["anchor.chain"] = bool(aenv and aenv.get("logChainOk"))
    result["checks"] = checks
    result["ok"] = all(checks.values())
    if result["ok"]:
        card = read_json(scorecard)
        result["score"] = card["total"]["score"]
        result["max"] = card["total"]["max"]
        result["runner"] = card["runner"]
        # pack 별 점수 — 총점만으로는 어느 능력이 강한지 사라진다(리더보드의 결).
        result["packs"] = {p["id"]: {"score": p.get("score"), "max": p["max"]}
                           for p in card["packs"] if p.get("status") == "scored"}
    return result


def cmd_verify(a, bin_path):
    ensure_board()
    entries, err = chain_walk(LEDGER, "settlementLedger")
    print(f"원장 체인: {len(entries)}항목 · {'무결' if err is None else '파손: ' + err}")
    aentries, aerr = chain_walk(ANCHOR, "anchorLog")
    print(f"앵커 체인: {len(aentries)}항목 · {'무결' if aerr is None else '파손: ' + aerr}")
    # 원장 꼬리 봉인 — 앵커의 마지막 항목은 attest 규약상 원장 스냅샷이다.
    snapshot_ok = bool(aentries) and os.path.exists(LEDGER) and         aentries[-1].get("capsuleSha256") == sha256_of(LEDGER)
    print(f"원장 스냅샷 봉인: {'일치' if snapshot_ok else '불일치!! (원장이 앵커 이후 변경됨)'}")
    if os.path.exists(CHECKPOINT):
        cp = read_json(CHECKPOINT)
        root = merkle_root(line_hashes(ANCHOR)[:cp.get("upToSeq", 0) + 1])
        match = root == cp.get("merkleRoot")
        print(f"체크포인트: upToSeq {cp.get('upToSeq')} · 머클 루트 "
              f"{'재계산 일치' if match else '불일치!!'}")
    results = [verify_entry(bin_path, e, entries) for e in entries]
    ok = sum(1 for r in results if r["ok"])
    print(f"항목 검증: {ok}/{len(results)} 통과")
    for r in results:
        mark = "O" if r["ok"] else "X"
        bad = [k for k, v in r.get("checks", {}).items() if not v]
        print(f"  {mark} seq{r['seq']} {r.get('agent', '?'):20} "
              + (f"{r.get('score')}/{r.get('max')}" if r["ok"] else f"실패: {bad or r.get('why')}"))
    write_json(os.path.join(BOARD, "verification.json"),
               {"kind": "gymLeaderboardVerification", "schemaVersion": "1.0",
                "ledgerEntries": len(entries), "ledgerChain": err or "ok",
                "anchorChain": aerr or "ok", "ledgerSnapshotSealed": snapshot_ok,
                "verified": ok, "results": results})
    return 0 if (err is None and aerr is None and snapshot_ok and ok == len(results)) else 3


def cmd_render(a, bin_path):
    entries, err = chain_walk(LEDGER, "settlementLedger")
    results = [verify_entry(bin_path, e, entries) for e in entries]
    lines = ["# 운동장 리더보드 — 위조 불가능한 점수판", "",
             "모든 순위는 검증 사슬(3해시 고정·Ed25519 서명·append-only 원장·머클 앵커)을",
             "**렌더 시점에 재검증**한 항목만 오른다. 재현 방법:",
             "`python gym/tools/leaderboard.py verify`", "",
             "| 순위 | 에이전트 | 총점 | 최강 능력 | commit | seq | 사슬 |",
             "|---|---|---|---|---|---|---|"]
    ranked = sorted((r for r in results if r["ok"]),
                    key=lambda r: (-r["score"], r["seq"]))
    for i, r in enumerate(ranked, 1):
        run = r["runner"]
        # 각 선수의 최강 능력 — 만점 비율이 가장 높은 pack.
        best = max(r.get("packs", {}).items(),
                   key=lambda kv: (kv[1]["score"] / kv[1]["max"] if kv[1]["max"] else 0, kv[1]["max"]),
                   default=(None, None))
        best_txt = f"{best[0]} {best[1]['score']}/{best[1]['max']}" if best[0] else "—"
        lines.append(f"| {i} | {r['agent']} | **{r['score']} / {r['max']}** "
                     f"| {best_txt} | `{run['rhwpCommit'][:10]}` "
                     f"| {r['seq']} | 검증됨 |")
    unverified = [r for r in results if not r["ok"]]
    for r in unverified:
        lines.append(f"| — | seq {r['seq']} | — | — | — | {r['seq']} | **unverified** |")

    # pack 별 능력 격자 — 총점이 숨기는 강약을 드러낸다.
    pack_ids = sorted({pid for r in ranked for pid in r.get("packs", {})})
    if pack_ids and len(ranked) > 1:
        lines += ["", "## 능력 격자 (pack 별 점수)", "",
                  "| 에이전트 | " + " | ".join(pack_ids) + " |",
                  "|---|" + "---|" * len(pack_ids)]
        for r in ranked:
            cells = []
            for pid in pack_ids:
                pk = r.get("packs", {}).get(pid)
                if pk is None:
                    cells.append("—")
                elif pk["score"] == pk["max"]:
                    cells.append(f"**{pk['score']}**")   # 만점 강조
                else:
                    cells.append(f"{pk['score']}/{pk['max']}")
            lines.append(f"| {r['agent']} | " + " | ".join(cells) + " |")
        lines.append("")
        lines.append("`—` = 미제출(그 pack 을 아예 풀지 않음) · **굵게** = 만점")
    lines += ["",
              f"원장 체인: {'무결' if err is None else '파손'} · 항목 {len(results)} · "
              f"검증 {len(ranked)} · unverified {len(unverified)}", "",
              "정직 조항: 이 사슬이 봉인하는 것은 \"이 스코어카드가 이 시점에 이 신원으로",
              "등재되었고 이후 변조되지 않았다\" 까지다. 채점 자체의 재현은 스코어카드의",
              "runner 신원(version·commit·capabilities digest)과 커밋된 제출물로 제3자가 수행한다."]
    out = os.path.join(BOARD, "leaderboard.md")
    with io.open(out, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(lines) + "\n")
    print(f"리더보드 렌더 → {os.path.relpath(out, ROOT)} (검증 {len(ranked)}·unverified {len(unverified)})")
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("mode", choices=["attest", "verify", "render"])
    ap.add_argument("--agent", default=None)
    ap.add_argument("--bin", default=None)
    a = ap.parse_args()
    bin_path = runner.find_bin(a.bin)
    if a.mode == "attest":
        if not a.agent:
            raise SystemExit("attest 는 --agent 가 필요하다")
        return cmd_attest(a, bin_path)
    if a.mode == "verify":
        return cmd_verify(a, bin_path)
    return cmd_render(a, bin_path)


if __name__ == "__main__":
    sys.exit(main())
