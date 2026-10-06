---
kind: guide
status: active
canonical: gym/docs/leaderboard.md
last_verified: 2026-09-02
---

# gym 리더보드 규약 — invite · attest · verify · render

이 문서는 `gym/tools/leaderboard.py` 의 **초대·등재·검증·렌더** 계약과
**예외 경로**를 고정한다. 운영 한 줄 요약은
[`gym/tools/README_leaderboard.md`](../tools/README_leaderboard.md) 를, 작업
기록은
[`mydocs/working/gym_leaderboard.md`](../../mydocs/working/archives/gym_leaderboard.md)
를 본다. 시험 계약은 `scripts/tests/test_gym_leaderboard.py` 가 기계로 고정한다.

새 암호 원시함수는 없다. 파일 해시는 `hashlib.sha256`, 서명·원장·앵커는 기존
rhwp 명령(`keygen` · `settle propose/record/verify` · `anchor add/checkpoint/verify`)
이다. 이 도구는 그 사다리를 **운동장 점수판 위에 얹는 조합**일 뿐이다.

## 1. 왜 이 기둥이 필요한가

AI 벤치마크 리더보드의 병폐는 점수의 신뢰다.

| 병폐 | 흔한 리더보드 | 이 점수판 |
|---|---|---|
| 점수 위조 | 자기 신고 JSON | 청구 `capsuleSha256` 고정 (P1) |
| 소급 수정 | 표를 고치면 끝 | 원장·앵커 줄 해시 체인 + 원장 스냅샷 봉인 |
| 이중 등재 | 같은 결과 재탕 | 원장 전역 `capsuleSha256` 유일성 (P3) |
| 대리 제출 | 이름만 바꾸면 됨 | Ed25519 서명 + keyring (`signerOk`) |

운동장은 이미 검증 사다리를 가지고 있다. 그래서 **운동장이 자기 사다리 위에서
돈다.** 채점 결과(스코어카드 + 입장 봉투)를 사다리로 봉인하고, 순위표는 렌더
시점에 그 사슬을 다시 밟은 항목만 올린다.

봉인 범위(정직 조항)는 여기까지다: "이 스코어카드가 이 시점에 이 신원으로
등재되었고 이후 변조되지 않았다." 채점 자체의 재현은 스코어카드에 박힌 runner
신원(`version` · `commit` · `capabilities digest`)과 커밋된 제출물로 제3자가
수행한다. 이 도구는 채점기를 대신하지 않는다.

## 2. 사용 — 모드 네 개. 새 CLI 없음

```bash
python gym/score.py --agent <이름>                      # 채점 (이 도구 밖)
python gym/tools/leaderboard.py attest --agent <이름>    # 등재
python gym/tools/leaderboard.py verify                   # 전 사슬 재검증
python gym/tools/leaderboard.py render                   # 검증본에서 순위표
python gym/tools/leaderboard.py invite --agent <이름>    # 초대장 (판 지문)
```

`argparse` 의 `choices` 는 `attest` · `verify` · `render` · `invite` 뿐이다.
`sign` · `keygen` · `audit` 같은 새 모드를 넣지 않는다. 서명이 필요하면 rhwp
`settle` 을 부른다.

| 인자 | 기본 | 의미 |
|---|---|---|
| `mode` | (필수) | 위 네 값 |
| `--agent` | None | `attest` 는 필수. `invite` 는 없으면 `친구-에이전트` |
| `--bin` | None | `runner.find_bin` 이 해석. `invite` 는 바이너리를 쓰지 않는다 |

종료 코드:

| exit | 누가 | 의미 |
|---|---|---|
| 0 | attest / invite / render 성공, verify 전항 통과 | 정상 |
| 2 | 분류된 I/O·형식·CLI 실패, 바이너리 탐색 실패 | 하네스/환경 |
| 3 | verify 사슬 파손·스냅샷 불일치·항목 실패, attest 의 원장 거부(SystemExit) | 판정 데이터 |
| SystemExit | attest 의 입장 거부·파일 없음·`--agent` 없음 | 사용자 오류 |

`invite` 는 rhwp 를 부르지 않는다. 지문은 커밋된 원장·앵커·발주서·키링에서만
나온다. 그래서 `main(["invite"])` 는 바이너리 없이 0 을 돌려야 한다.

## 3. 등재 사슬 (attest)

```
scorecard.json + admission.json     채점기가 발급
  → keygen                          에이전트 신원. 공개키만 keyring 에
  → settle propose --sign-key       명세서·스코어카드·입장 봉투 3해시 + 서명
  → settle record --ledger          append-only 원장. 같은 스코어카드 재등재 거부
  → 스코어카드·입장 보존            gym/leaderboard/scorecards/<agent>-<epoch>/
  → anchor add (claim)              청구를 투명성 로그에
  → anchor add (ledger)             원장 바이트 스냅샷
  → anchor checkpoint               머클 에폭
```

규약의 함정과 막은 자리:

1. **기입을 먼저 시도한다.** `settle record` 가 exit 3 (이중 등재) 이면 이번
   attest 의 claim·sidecar 를 지운다. 원장이 유일한 진실인데 미등재 파일이
   굴러다니면 사람이 헷갈린다.
2. **보존은 등재 확정 뒤에만.** 스코어카드·입장 봉투 복사는 record 가 0 을 준
   뒤에만 한다. 검증이 커밋본을 참조하므로, 거부된 제출을 보존하면 거짓
   증거가 된다.
3. **원장 꼬리 봉인.** 항목이 하나일 때 원장 한 줄을 고치면 다음 줄이 없어
   체인만으로는 폭로되지 않는다. 그래서 attest 마다 원장 파일 자체의
   SHA-256 을 앵커에 올린다. verify 는 "앵커의 마지막 항목 == 지금 원장
   바이트 해시" 를 요구한다.

`attest` 가 거부하는 사용자 오류 (SystemExit, 암호 실패가 아님):

| 조건 | 메시지 표식 |
|---|---|
| `--agent` 없음 | `attest 는 --agent 가 필요하다` |
| scorecard/admission 없음 | `없음: … 먼저 gym/score.py` |
| 입장 봉투가 객체 아님 | `입장 봉투가 객체가 아니다` |
| `verdict != allow` | `입장 봉투 verdict=…` |
| 원장 거부 | `원장 거부(이중 등재)` |
| keygen 산출에 publicKey 없음 | `LeaderboardSchemaError` → `attest 실패 (schema)` |
| keyring.keys 가 목록 아님 | 위와 같음 |
| 바이너리 없음·실행 실패 | `attest 실패 (cli)` |
| JSON 읽기/쓰기 실패 | `attest 실패 (io|format)` |

비밀키는 `gym/leaderboard/keys/<agent>.key.json` 에만 남는다. 이 경로는
`.gitignore` 다. 커밋되는 것은 공개키·서명된 청구·스코어카드뿐이다.

## 4. 초대 (invite)

초대는 **권한이 아니라 안내**다. `attest` 는 아무 이름이든 받는다. 초대장은
"어디로 오면 되는지"와 "네가 합류하는 판이 위조본이 아님을 어떻게 확인하는지"
를 한 봉투로 묶을 뿐이다.

`construct_invite(guest)` 는 쓰기를 하지 않는다. `cmd_invite` 가
`gym/leaderboard/invite.json` 에 쓴다. 시험은 임시 판으로 경로를 돌려, 커밋된
`gym/leaderboard/` 를 읽기만 한다.

### 4.1 초대장 봉투

`kind=gymLeaderboardInvite`, `schemaVersion=1.0`. 키 집합은 `INVITE_KEYS`.

| 키 | 형 | 의미 |
|---|---|---|
| `schemaVersion` | str | 항상 `1.0` |
| `kind` | str | 항상 `gymLeaderboardInvite` |
| `guest` | str | 손님 이름. 빈 문자열·None 은 `친구-에이전트` |
| `board.repo` | str | `edwardkim/rhwp` |
| `board.path` | str | `gym/leaderboard` |
| `fingerprint` | object | 아래 4.2 |
| `join` | list[str] | 합류 3줄. 길이 3, score.py / attest / verify |
| `promise` | str | 비밀키는 keys/ 에만, 커밋되지 않는다 |
| `note` | str | 초대는 권한이 아니라 안내 |

`validate_invite()` 가 이 계약을 다시 검사한다. 손님 이름이 문자열이 아니면
`LeaderboardSchemaError`. `join` 이 3줄이 아니거나 동사가 빠지면 위반 목록.

합류 3줄 (손님이 `이름` 일 때):

```text
python gym/score.py --agent 이름
python gym/tools/leaderboard.py attest --agent 이름
python gym/tools/leaderboard.py verify
```

### 4.2 판 지문

`board_fingerprint()` 가 지금 판의 값을 **커밋된 파일에서 재계산**한다. 새
비밀은 없다.

| 키 | 출처 |
|---|---|
| `members` | keyring.json 의 `keys` 길이. 목록이 아니면 0 |
| `ledgerEntries` | 원장 체인 길이 |
| `ledgerChain` | `chain_walk` 의 오류 또는 `"ok"` |
| `anchorChain` | 앵커 `chain_walk` 의 오류 또는 `"ok"` |
| `merkleRoot` | checkpoint.json 의 값. 없으면 None |
| `workorderSha256` | 발주서 파일 바이트 SHA-256. 없으면 None |
| `ledgerSnapshotSha256` | 원장 파일 바이트 SHA-256. 없으면 None |

읽기 실패는 예외를 올리지 않는다. keyring 이 깨진 JSON 이면 `members=0` 으로
접고 나머지 필드는 채운다. 초대장이 죽으면 신참이 판을 확인하는 길 자체가
막힌다.

`validate_fingerprint()` 는 키 집합·타입·해시 길이(64 hex, merkleRoot 제외)를
본다. 해시 값을 재계산하지는 않는다 — 그건 신참이 `verify` 로 한다.

빈 판의 지문은 `empty_fingerprint()` 와 같다. 시험이 그 키 집합을 고정한다.

## 5. 검증 (verify)

`cmd_verify` 가 하는 일:

1. 원장 체인 (`settlementLedger`) 과 앵커 체인 (`anchorLog`) 을 `chain_walk`.
2. 원장 스냅샷 봉인: 앵커 마지막 항목의 `capsuleSha256` == 지금 원장 바이트
   SHA-256.
3. 체크포인트가 있으면 앵커 줄 해시를 다시 머클하고 `merkleRoot` 와 비교.
4. 각 원장 항목을 `verify_entry` 로 재검증.
5. `gym/leaderboard/verification.json` 에 봉투를 쓴다.

`kind=gymLeaderboardVerification`, `schemaVersion=1.0`.

| 키 | 의미 |
|---|---|
| `ledgerEntries` | 원장 항목 수 |
| `ledgerChain` | `"ok"` 또는 파손 이유 |
| `anchorChain` | `"ok"` 또는 파손 이유 |
| `ledgerSnapshotSealed` | bool |
| `verified` | `ok=True` 항목 수 |
| `results` | 항목별 판정 |

exit 0 은 `ledgerChain ok` · `anchorChain ok` · 스냅샷 봉인 · 전항 통과 일 때만.
그 외 판정 실패는 3. verification.json 쓰기 실패는 2.

### 5.1 항목 검증 (`verify_entry`)

한 항목의 실패가 전체 verify 를 죽이면 폭로가 아니라 침묵이다. 그래서
I/O·형식·CLI 실패는 `ok=False` + `why` 로 접는다.

순서:

1. 항목이 객체가 아니면 즉시 실패.
2. `claims/` 목록. 디렉터리가 아니면 `why=claims 목록 실패`.
3. 각 파일의 SHA-256 이 원장의 `claimSha256` 과 같은 것을 찾는다. 해시 실패
   파일은 건너뛴다.
4. claim JSON 을 읽는다. 파싱 실패면 `why=claim 읽기 실패`.
5. `claim.capsuleSha256 == entry.capsuleSha256` (교차 대조). 원장만 고치면
   여기서 폭로된다.
6. `rhwp settle verify` — 3해시 고정 + 서명 + 입장 allow.
7. `rhwp anchor verify` — 로그 등재 + 로그 체인.
8. 통과하면 스코어카드에서 `score` / `max` / `runner` / pack 점수를 읽는다.

checks 키:

| 키 | 출처 |
|---|---|
| `pin.workorder` | `workorderOk` |
| `pin.scorecard` | `capsuleOk` |
| `pin.admission` | `gateOk` |
| `admission.allow` | `gateVerdict == allow` |
| `signature` | `signerOk` |
| `ledger.crossPin` | 교차 대조 |
| `anchor.logged` | `logged` |
| `anchor.chain` | `logChainOk` |

`ok` 는 모든 checks 가 참일 때만. 스코어카드 읽기 실패면 이미 통과한 checks 를
뒤집고 `why` 를 남긴다 — 점수를 지어내지 않는다.

## 6. 렌더 (render)

`cmd_render` 는 원장을 다시 `verify_entry` 한 뒤 `render_markdown` 으로
`gym/leaderboard/leaderboard.md` 를 쓴다. 쓰기는 호출자가 한다.
`render_markdown` 자체는 순수 함수다.

### 6.1 순위 (`rank_results`)

- `ok` 가 참이고 `score`/`seq` 가 유한 숫자인 행만 순위에 오른다.
- 정렬 키는 `(-score, seq)`. 동점이면 먼저 등재된 쪽이 위.
- `ok` 가 없거나 거짓이면 unverified.
- `ok=True` 인데 score/seq 가 없거나 NaN/Inf 이면 unverified 로 접고
  `why=score/seq 결손`.
- 행이 객체가 아니면 가짜 unverified 한 줄을 남긴다.
- 입력이 목록이 아니면 `([], [])`.

순위에 오르지 못한 행은 **숨기지 않는다.** 표에 `**unverified**` 로 남긴다.
부재를 실패로 위장하지 않는 저장소의 결 그대로다.

### 6.2 최강 pack (`best_pack`)

만점 비율(`score/max`)이 가장 높은 pack. 동률이면 `max` 가 큰 쪽. 없으면
None.

- `packs` 가 객체가 아니거나 비면 None.
- 항목에 `score`/`max` 가 없거나 비숫자·NaN 이면 건너뛴다.
- `max==0` 이면 비율 0. ZeroDivision 없음.

반환은 `(pack_id, score, max)`. 렌더는 `이름 점수/만점` 으로 쓴다.

### 6.3 마크다운

항상 포함한다:

- 헤더 `# 운동장 리더보드 — 위조 불가능한 점수판`
- 재검증 안내와 `python gym/tools/leaderboard.py verify`
- 표 헤더 `| 순위 | 에이전트 | 총점 | 최강 능력 | commit | seq | 사슬 |`
- 검증 행: 순위, 총점 `**score / max**`, 최강 pack, commit 10자, `검증됨`
- unverified 행: `| — | seq N | — | — | — | N | **unverified** |`
- 바닥글: 원장 체인 무결/파손, 항목 수, 검증 수, unverified 수
- 정직 조항 문단

검증된 선수가 2명 이상이고 pack 점수가 있으면 **능력 격자**를 붙인다.
만점 칸은 `**N**`, 미제출은 `—`, 부분 점수는 `s/m`.

`runner` 나 `rhwpCommit` 이 없으면 칸을 `—` 로 남긴다. 렌더가 죽으면 순위표가
안 나오고, 그건 검증 실패를 숨기는 것과 같다.

## 7. 줄 해시 체인과 머클

### 7.1 `chain_walk(path, kind)`

rhwp 의 `anchor_log::load_kind` 파이썬 거울. settlementLedger 를 단독 검증하는
CLI 가 없어 같은 알고리즘을 여기에 둔다.

규약:

- 빈 줄은 건너뛴다.
- 각 줄은 JSON 객체.
- `kind` 가 인자와 같아야 한다.
- `seq` 는 0부터 빈틈없이.
- `prevEntryHash` 는 직전 **줄 원문 바이트**의 SHA-256. 첫 줄은 `null`.
- 현재 줄의 해시는 `hashlib.sha256(line.encode("utf-8"))`.

파손이 있으면 `(지금까지의 항목, "N행: 이유")` 를 돌려준다. 예외를 올리지
않는다.

| 실패 | 이유 문자열 |
|---|---|
| 경로 빈 문자열 | `경로가 비어 있다` |
| 파일 없음 | `([], None)` — 빈 판은 파손이 아니다 |
| 읽기/인코딩 실패 | `읽기 실패` / `인코딩 실패` |
| JSON 파싱 실패 | `N행: JSON 파싱 실패` |
| 객체가 아님 | `N행: 객체가 아님` |
| kind 불일치 | `N행: kind X != Y` |
| seq 연번 위반 | `N행: seq 연번 위반` |
| prev 불일치 | `N행: prevEntryHash 불일치 (append-only 위반)` |

과거 줄을 고치면 다음 줄의 `prevEntryHash` 가 어긋나 폭로된다. 이것이 소급
수정의 1차 방어다. 항목이 하나일 때는 5. 의 스냅샷 봉인이 2차 방어다.

### 7.2 `merkle_root(leaf_hashes)`

앵커 체크포인트의 머클 규약 거울.

- 빈 목록·None → None.
- 잎은 **hex 문자열 목록**. `str`/`bytes` 통째로 넘기거나 잎이 문자열이 아니면
  `LeaderboardSchemaError`. 조용히 `str()` 하면 다른 머클이 된다.
- 홀수 층은 마지막 잎을 복제해 짝을 맞춘다.
- 부모는 `sha256((a + b).encode("ascii"))`. 잎을 이어 붙인 뒤 ASCII 로 해시.

새 머클 변형을 만들지 않는다. rhwp `anchor checkpoint` 가 쓰는 규약과 같아야
재계산이 의미가 있다.

### 7.3 `line_hashes` / `sha256_of`

둘 다 `hashlib.sha256` 뿐이다. 파일이 없으면 `line_hashes` 는 `[]`,
`sha256_of` 는 `LeaderboardIOError`. `try_sha256` / `try_line_hashes` 는
예외를 올리지 않는 거울이다.

## 8. 예외 경로 — 도구가 죽지 않는 계약

도구 자신이 예외로 죽으면 게이트는 "사슬이 무결하다"는 거짓 음성을 못 내고,
CI 가 붉어져 사슬 결함과 하네스 결함을 구분할 수 없다. 그래서 I/O·형식·CLI
경로는 분류 가능한 코드로 접힌다.

예외 계층:

| 클래스 | code | 언제 |
|---|---|---|
| `LeaderboardError` | `leaderboard` | 기본 |
| `LeaderboardIOError` | `io` | 파일 없음·권한·디렉터리 |
| `LeaderboardFormatError` | `format` | JSON 파싱/직렬화·인코딩 |
| `LeaderboardChainError` | `chain` | (예약. chain_walk 는 문자열로 접음) |
| `LeaderboardCliError` | `cli` | 빈 바이너리·실행 실패·find_bin |
| `LeaderboardSchemaError` | `schema` | 타입·키 결손·머클 잎 |

`classify_exception()` 이 stdlib 예외도 같은 코드로 접는다.
`FileNotFoundError`/`PermissionError`/`OSError` → io,
`JSONDecodeError`/`UnicodeError` → format,
`TypeError`/`ValueError`/`KeyError` → schema,
그 외 → leaderboard.

| 경로 | 함수 | 접는 곳 | 보고 |
|---|---|---|---|
| 없는 파일 sha256 | `sha256_of` | `LeaderboardIOError` | `try_sha256` 은 (None, err) |
| JSON 없음 | `read_json` | IOError | `try_read_json` |
| JSON 깨짐 | `read_json` | FormatError | `try_read_json` |
| 직렬화 불가 | `write_json` | FormatError | `try_write_json` |
| 디렉터리에 쓰기 | `write_json` | IOError | `try_write_json` |
| 목록 대상이 파일 | `safe_listdir` | `([], 이유)` | claims 목록 실패 |
| 사슬 JSON 깨짐 | `chain_walk` | `(항목, 이유)` | verify 바닥글 |
| 머클 잎 타입 | `merkle_root` | SchemaError | 호출 측 |
| 빈 바이너리 | `run_cli` | CliError | attest/verify why |
| 없는 바이너리 | `run_cli` | CliError | 위와 같음 |
| find_bin 실패 | `resolve_bin` | CliError | main exit 2 |
| 손님 이름 타입 | `construct_invite` | SchemaError | invite exit 2 |
| 초대장 스키마 | `cmd_invite` | SchemaError | invite exit 2 |
| 입장 거부 | `cmd_attest` | SystemExit | 사용자 오류 |
| 항목 claim 없음 | `verify_entry` | `ok=False` | results |
| 항목 JSON 깨짐 | `verify_entry` | `ok=False` | results |
| settle/anchor 실패 | `verify_entry` | `ok=False` | results |
| 순위 score NaN | `rank_results` | unverified | 표 |
| pack 결손 | `best_pack` | 건너뜀 | `—` |
| runner 없음 | `render_markdown` | 칸 `—` | 표 |
| verification 쓰기 실패 | `cmd_verify` | exit 2 | stderr |
| render 쓰기 실패 | `cmd_render` | exit 2 | stderr |

`cmd_attest` / `cmd_verify` / `cmd_invite` / `cmd_render` 의 바깥은
`LeaderboardError` 와 `OSError`/`ValueError`/`TypeError` 를 잡아 분류 메시지로
바꾼다. attest 는 SystemExit 을 다시 올려 사용자 오류를 유지한다.

시험은 이 표의 각 칸을 임시 디렉터리와 목킹으로 고정한다. 바이너리 없이
돌아간다. 서명 검증은 rhwp 게이트의 몫이다.

## 9. 정직 조항 — 다시

이 사슬이 봉인하는 것:

- 스코어카드 바이트가 등재 시점의 청구에 고정됐다.
- 그 청구가 이 신원의 키로 서명됐다.
- 원장에 한 번만 들어갔다.
- 그 이후 원장·앵커 바이트가 줄 체인과 스냅샷으로 봉인됐다.

이 사슬이 **봉인하지 않는** 것:

- 채점이 옳았는가. (runner 신원 + 제출물로 제3자가 재현)
- 과제가 판별력이 있는가. (`discriminate.py`)
- 도구가 손상 입력에 죽지 않는가. (`robustness.py`)
- 초대받은 사람이 믿을 만한가. (초대는 안내)

숫자를 지어내지 않는다. 스코어카드를 못 읽으면 점수를 비우고 `ok=False`.
keyring 을 못 읽으면 `members=0`. 바이너리가 없으면 invite 만 돌고, 나머지
모드는 exit 2.

## 10. 시험 지도

```bash
python -m unittest scripts.tests.test_gym_leaderboard scripts.tests.test_gym_audit -q
python gym/tools/audit.py
```

| 클래스 | 고정하는 것 |
|---|---|
| `ChainWalkTests` | 유효 체인, 소급 수정 폭로, seq 공백, 빈/공백 파일 |
| `MerkleTests` | 결정성, 홀수 층 복제, 두 잎 이어 붙임 |
| `CommittedBoardTests` | 커밋된 원장·앵커 무결, claim 파일 해시 |
| `InviteTests` | 커밋된 판 지문 키 |
| `RankResultsTests` | 점수·seq 동점, unverified 분리 |
| `BestPackTests` | 비율·동률 max·max=0 |
| `RenderMarkdownTests` | 헤더·정직 조항·격자·파손 바닥글 |
| `InviteEnvelopeTests` | 임시 판, 커밋된 판 무터치 |
| `ExceptionClassifyTests` | 예외 계층·stdlib 접기 |
| `JsonIoExceptionTests` | sha256/read/write/listdir |
| `ChainWalkMalformedTests` | JSON 깨짐, 비객체 행 |
| `MerkleSchemaTests` | 잎 타입 거절 |
| `FingerprintAndInviteSchemaTests` | 지문·초대장 validate, cmd_invite |
| `RankAndPackExceptionTests` | NaN, 비목록, 결손 pack |
| `RenderDefensiveTests` | runner 없음, 깨진 packs |
| `CommandWrapperTests` | attest/verify/render/main |
| `VerifyEntryExceptionTests` | 비객체, claim 없음, 깨진 claim |
| `ConstantContractTests` | 키 집합, 모드 4개, hashlib 만 |
| `AttestAdmissionExceptionTests` | 입장 객체/deny |
| `ResolveBinTests` | find_bin 실패 분류 |

`test_gym_audit` 는 전 pack 정합을 지킨다. 이번 변경은 pack 을 건드리지
않으므로 그대로 초록이어야 한다.

바이너리 없이 돈다. `run_cli` 실호출 시험은 없는 바이너리의 `OSError` 뿐이다.

## 11. 하지 않는 것

- 새 암호 원시함수. Ed25519 구현을 여기 두지 않는다.
- 새 CLI 모드. `choices` 는 네 값.
- pack/과제/checks 변경. 채점 기둥과 섞지 않는다.
- 커밋된 `gym/leaderboard/` 를 시험이 쓰기. 경로는 임시 판으로 돌린다.
- unverified 행을 표에서 빼기. 숨김은 위조다.
- 시그널 종료를 성공으로 접기. CLI 실패는 cli 코드다.
- `cargo fmt --all`. Python·문서만 고친다.

이 문서가 코드와 다르면 코드와 시험을 이긴다. 문서만 고치고 시험을 안 고치면
계약이 아니다.

## 12. 점수판 파일 배치

`gym/leaderboard/` 는 커밋된 진실이다. 시험은 이 트리를 쓰지 않고 경로 상수를
임시 디렉터리로 돌린다.

```text
gym/leaderboard/
├── workorder.json          상설 발주서. attest 가 없으면 기본본을 씀
├── keyring.json            공개키만. keys[].revoked
├── ledger.ndjson           settlementLedger 줄 체인
├── anchor.ndjson           anchorLog 줄 체인
├── checkpoint.json         머클 에폭
├── verification.json       마지막 verify 봉투
├── leaderboard.md          마지막 render 산출
├── invite.json             마지막 invite 산출 (로컬, 필수는 아님)
├── claims/
│   ├── <agent>-<epoch>.claim.json
│   └── <agent>-<epoch>.claim.json.sig.json
├── scorecards/
│   └── <agent>-<epoch>/{scorecard.json,admission.json}
└── keys/                   비밀키. .gitignore
    └── <agent>.key.json
```

커밋되는 것: 발주서, 키링(공개키), 원장, 앵커, 체크포인트, 청구, 서명 sidecar,
보존 스코어카드·입장, 렌더된 마크다운, 검증 봉투.

커밋되지 않는 것: `keys/*.key.json`. 초대 JSON 은 로컬 안내용이라 이 가지는
커밋하지 않는다.

원장 한 줄과 앵커 한 쌍(claim + 원장 스냅샷)이 attest 1회의 최소 단위다.
항목 N개면 앵커는 대략 2N 줄이다. 마지막 앵커 줄이 원장 스냅샷이어야
`ledgerSnapshotSealed` 가 참이다.

## 13. 공격 시나리오와 폭로 지점

아래는 새 암호가 아니라 기존 사다리가 이미 막는 자리와, 이 도구가 그 판정을
어디에 드러내는가다.

### 13.1 스코어카드 부풀림

등재 후 `scorecards/<id>/scorecard.json` 의 총점을 고친다. `settle verify` 의
`capsuleOk` 가 거짓이 된다. `verify_entry.checks["pin.scorecard"]` 가 거짓,
항목은 unverified, 렌더는 순위에서 빼고 표에 남긴다.

### 13.2 원장 첫 줄 소급

`ledger.ndjson` 의 첫 줄 `capsuleSha256` 을 바꾼다. 항목이 2개 이상이면 둘째
줄의 `prevEntryHash` 가 어긋나 `chain_walk` 가 `N행: prevEntryHash 불일치` 를
준다. 항목이 1개면 체인만으로는 부족하고, 앵커 마지막 항목의 원장 스냅샷
해시가 달라 `ledgerSnapshotSealed=false`, exit 3.

교차 대조 `ledger.crossPin` 은 원장만 고치고 claim 파일을 그대로 두면 추가로
폭로한다.

### 13.3 같은 스코어카드 재등재

`attest` 를 같은 제출로 한 번 더 부른다. `settle record` 가 exit 3,
`duplicate: true`. 이번 claim 파일을 지우고 SystemExit. 원장 길이는 그대로.

### 13.4 대리 이름

남의 이름으로 등재하려면 그 이름의 비밀키가 필요하다. 키가 없으면 keygen 이
새 신원을 만들고 keyring 에 다른 공개키가 오른다. 기존 항목의 `signerOk` 는
그 키로 다시 검증된다. 남의 비밀키 없이 기존 항목을 가로채지 못한다.

### 13.5 초대장 위조

초대 JSON 의 지문을 고친다. 신참이 커밋된 원장에서 `board_fingerprint` 를
다시 계산하면 값이 다르다. 초대장은 권한이 아니므로, 위조된 초대장을 들고
`attest` 해도 원장 규칙은 그대로다. 지문은 "이 판이 그 판인가"만 말한다.

## 14. 모드별 부작용과 재실행

| 모드 | 멱등 | 재실행 |
|---|---|---|
| attest | 같은 스코어카드는 거부 | 재채점 후 새 스코어카드 |
| verify | 쓰기 대상은 verification.json 뿐 | 언제든 |
| render | 쓰기 대상은 leaderboard.md 뿐 | 언제든 |
| invite | invite.json 을 덮어씀 | 언제든. 권한 변화 없음 |

verify/render 는 원장을 바꾸지 않는다. 깨진 사슬을 verify 가 고치지 않는다.
고치는 쪽은 커밋 이력이거나 새 attest 다.

## 15. 관련

- 이슈 #4659 (위조 불가능한 리더보드), #4664 (친구 초대), #5235 (순위·초대
  순수 시험)
- PR: https://github.com/edwardkim/rhwp/pull/5244
- 가지: `feat/gym-leaderboard-rank-invite`
- 친구 안내: `gym/INVITE.md`
- gym 개요의 해당 절: `gym/README.md` 「위조 불가능한 리더보드」
- 분업: 채점은 `gym/score.py`, 수동 구·신 차등 판정은 `gym/tools/release_gate.py`
