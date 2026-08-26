---
kind: guide
status: active
canonical: gym/tools/README_leaderboard.md
last_verified: 2026-08-18
---

# leaderboard.py — 운영 메모

위조 불가능한 점수판의 **한 페이지 운영 메모**다. 초대·등재·검증·렌더와
예외 경로의 정본은 [`gym/docs/leaderboard.md`](../docs/leaderboard.md) 다.
작업 기록은
[`mydocs/working/gym_leaderboard.md`](../../mydocs/working/gym_leaderboard.md).

## 30초

```bash
python gym/score.py --agent <이름>
python gym/tools/leaderboard.py attest --agent <이름>
python gym/tools/leaderboard.py verify
python gym/tools/leaderboard.py render
python gym/tools/leaderboard.py invite --agent <이름>
python -m unittest scripts.tests.test_gym_leaderboard scripts.tests.test_gym_audit -q
python gym/tools/audit.py
```

채점 → 등재 → 재검증 → 순위표. 초대는 판 지문을 붙이는 안내일 뿐 권한이
아니다. 새 모드는 없다.

## 언제 돌리나

- 채점을 끝낸 에이전트를 명예의 전당에 올릴 때 (`attest`).
- 커밋된 원장·앵커가 아직 무결한지 CI/릴리스 전에 볼 때 (`verify`).
- 순위표 마크다운을 다시 그릴 때 (`render`).
- 친구·다른 에이전트를 부를 때 (`invite`).

돌리지 않는 때:

- pack 채점. `score.py` 다.
- 손상 입력 게이트. `robustness.py` 다.
- 릴리스 차등. `release_diff.py` 다.
- 포맷 변경. 이번 도구는 Python 만 고친다.

## 모드

| 모드 | 바이너리 | 쓰기 | 성공 exit |
|---|---|---|---|
| `attest --agent` | 필요 | claims, ledger, anchor, checkpoint, scorecards, keyring | 0 |
| `verify` | 필요 | verification.json | 0 (전항 통과) |
| `render` | 필요 | leaderboard.md | 0 |
| `invite [--agent]` | **불필요** | invite.json | 0 |

`invite` 는 커밋된 파일을 읽어 지문만 계산한다. `main(["invite"])` 가
`find_bin` 을 부르지 않는 이유다.

## 종료 코드

| 코드 | 언제 |
|---|---|
| 0 | attest/invite/render 성공, verify 전항 통과 |
| 2 | I/O·형식·CLI·바이너리 탐색. 하네스/환경 |
| 3 | verify 사슬 파손·스냅샷 불일치·항목 실패 |
| SystemExit | attest 의 `--agent` 없음, 입장 거부, 파일 없음, 이중 등재 |

원장 거부는 "환경이 깨졌다"가 아니라 "같은 스코어카드가 이미 있다"는
판정이다. 그래서 SystemExit 메시지를 유지한다.

## 초대장에서 볼 키

성공 판정: `kind == gymLeaderboardInvite`, `validate_invite` 빈 목록.

지문:

- `members` — keyring 신원 수. 깨진 keyring 은 0.
- `ledgerEntries` — 원장 체인 길이. 자기 신고가 아니다.
- `ledgerChain` / `anchorChain` — `"ok"` 또는 파손 이유.
- `merkleRoot` — 체크포인트. 없으면 null.
- `workorderSha256` / `ledgerSnapshotSha256` — 파일 바이트 SHA-256.

합류 3줄이 `score.py` · `attest` · `verify` 를 포함하는지 `validate_invite` 가
본다. 길이가 3이 아니면 위반.

## 검증 봉투에서 볼 키

- `ok` 가 아니라 `verified == ledgerEntries` 그리고 두 체인이 `"ok"` 그리고
  `ledgerSnapshotSealed`.
- `results[].checks` 의 거짓 키가 어느 축이 깨졌는지 말한다.
- `why` 는 claim 없음·JSON 깨짐·CLI 실패.

`schemaVersion` 은 `1.0`. 키를 빼면 `validate_invite` /
`validate_fingerprint` 가 위반을 낸다.

## 순위표에서 볼 것

- 검증된 행만 숫자 순위. unverified 는 `—` 순위 + `**unverified**`.
- 동점은 낮은 seq (먼저 등재).
- 최강 능력은 만점 비율, 동률은 더 큰 max.
- 능력 격자는 선수 2명 이상일 때만.
- 정직 조항이 빠지면 렌더 시험이 붉어진다.

## 예외 경로 — 운영자가 헷갈리는 것

| 보이는 것 | 의미 | 다음 |
|---|---|---|
| `attest 실패 (cli)` | 바이너리 없음/실행 실패 | `--bin` 또는 `cargo build --bin rhwp` |
| `없음: …/scorecard.json` | 채점을 안 함 | `gym/score.py --agent` |
| `입장 봉투 verdict=deny` | 게이트가 거절 | 입장 정책·제출을 고친다 |
| `원장 거부(이중 등재)` | 같은 스코어카드 | 재채점 후 새 스코어카드 |
| `verify 실패 (io)` | 파일 권한·경로 | 판 디렉터리 |
| `원장 체인: 파손` | 줄 해시/seq/kind | 원장을 손대지 말 것 |
| `원장 스냅샷 봉인: 불일치` | 원장이 앵커 이후 변경 | 꼬리 변조. 복구는 커밋 이력 |
| `invite 실패 (schema)` | 초대장 키 결손 | 도구 버그. 시험이 막아야 함 |
| exit 2 (main 바이너리) | find_bin 실패 | invite 가 아니면 `--bin` |

시그널 종료·없는 바이너리는 **cli** 다. 사슬 파손이 아니다.

## 로컬에서 순수 함수만 보고 싶을 때

모듈로 불러 순위·초대만 본다. 바이너리 불필요.

```python
import importlib.util
from pathlib import Path
p = Path("gym/tools/leaderboard.py")
spec = importlib.util.spec_from_file_location("lb", p)
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
print(m.rank_results([
    {"ok": True, "agent": "a", "score": 3, "max": 5, "seq": 1,
     "runner": {"rhwpCommit": "abc1234567890"}, "packs": {}},
    {"ok": False, "seq": 0},
]))
print(m.validate_invite(m.construct_invite("손님")))
```

커밋된 판을 건드리지 않으려면 `m.BOARD` 와 하위 경로를 임시 디렉터리로 돌린다.
시험의 `_redirect` 가 그 패턴이다.

## 시험

```bash
python -m unittest scripts.tests.test_gym_leaderboard
python -m unittest scripts.tests.test_gym_audit
python gym/tools/audit.py
```

기존 클래스(`ChainWalkTests` … `InviteEnvelopeTests`)는 원 PR 게이트.
이번 가지의 `ExceptionClassifyTests` · `JsonIoExceptionTests` ·
`CommandWrapperTests` · `VerifyEntryExceptionTests` 가 예외 접기를 고정한다.

`cargo fmt --all` 은 이 변경에 실행하지 않는다. Python·문서만 고친다.

## 새 키를 넣을 때 최소 체크

1. 초대장/지문/검증 봉투의 키가 `INVITE_KEYS` / `FINGERPRINT_KEYS` 와 같은가.
2. `validate_invite` / `validate_fingerprint` 가 그 키를 보는가.
3. 임시 판 시험에서 커밋된 `gym/leaderboard/` 를 쓰지 않는가.
4. 새 암호 필드가 아닌가. 해시는 sha256 hex 64자.
5. 모드를 늘리지 않았는가. `MODES` 는 네 값.
6. `test_gym_leaderboard` 가 그 키를 한 줄이라도 고정하는가.

자세한 표는 규약 문서 4~8절이다.

## 파일 배치 (한 줄)

커밋: `workorder.json` · `keyring.json` · `ledger.ndjson` · `anchor.ndjson` ·
`checkpoint.json` · `claims/` · `scorecards/` · `leaderboard.md` ·
`verification.json`.

커밋 금지: `keys/*.key.json`. 초대 `invite.json` 은 로컬 안내.

원장 N줄이면 앵커는 대략 2N (claim + 원장 스냅샷). 마지막 앵커 줄이 원장
바이트 해시여야 스냅샷 봉인이 참이다.

## 자주 하는 실수

1. `attest` 전에 `score.py` 를 안 돌린다 → `없음: scorecard.json`.
2. 같은 제출을 두 번 `attest` → 원장 거부. 재채점해야 한다.
3. `keys/` 를 커밋하려고 한다 → `.gitignore`. 공개키만 keyring.
4. unverified 행을 표에서 지운다 → 숨김은 위조. 렌더가 남긴다.
5. 초대장이 있어야 `attest` 된다고 생각한다 → 문은 이미 열려 있다.
6. `invite` 에 바이너리가 필요하다고 생각한다 → 지문은 파일 해시뿐.
7. 깨진 claim 하나 때문에 verify 가 크래시해야 한다고 생각한다 → 항목은
   `ok=False` 로 접고 나머지를 계속 본다.

## 관련

- 이슈 #4659, #4664, #5235
- PR https://github.com/edwardkim/rhwp/pull/5244
- 친구 안내 `gym/INVITE.md`
- gym 개요 `gym/README.md` 「위조 불가능한 리더보드」
