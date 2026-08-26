---
kind: working
status: active
canonical: mydocs/working/gym_leaderboard.md
last_verified: 2026-08-18
---

# gym 리더보드 — 순위·초대 예외 경로 보강 작업 기록

Issue: #5235
PR: https://github.com/edwardkim/rhwp/pull/5244
Branch: `feat/gym-leaderboard-rank-invite`
Date: 2026-08-18

## 1. 결론

`gym/tools/leaderboard.py` 의 순위 정렬·최강 pack·마크다운 렌더·초대 봉투를
순수 함수로 분리한 원 PR 위에, I/O·JSON·사슬·CLI 예외를 분류해 도구가 죽지
않게 닫았다. 새 암호 원시함수는 넣지 않았다. 서명은 계속 rhwp `settle` /
`keygen` 이고, 파일 해시는 `hashlib.sha256` 뿐이다.

이 가지는 새 PR 을 열지 않는다. 같은 브랜치 `feat/gym-leaderboard-rank-invite`
에 이어서 밀어 #5244 를 키운다.

검증:

- `python -m unittest scripts.tests.test_gym_leaderboard scripts.tests.test_gym_audit -q`
- `python gym/tools/audit.py`
- `cargo fmt --all` 은 실행하지 않음 (Python/문서만, 사용자 지시)

## 2. 배경

원 PR(#5244)은 다음을 순수 함수로 뽑았다.

- `rank_results` — 검증된 항목만 `(-score, seq)`
- `best_pack` — 만점 비율, 동률은 큰 max
- `render_markdown` — 헤더·unverified·정직 조항
- `construct_invite` — 지문 + 합류 3줄. 쓰기는 호출자

대비 `upstream/devel` 삽입은 약 384줄, 파일 2개
(`leaderboard.py`, `test_gym_leaderboard.py`)였다.

그 상태의 빈틈:

1. `read_json` / `write_json` / `sha256_of` 가 없는 파일·깨진 JSON 에서 원시
   예외를 올린다. verify 한 항목의 읽기 실패가 전체 명령을 죽인다.
2. `chain_walk` 가 `json.loads` 예외를 그대로 올린다. 한 줄이 깨지면 사슬
   폭로가 아니라 크래시다.
3. `run_cli` 가 빈 경로·없는 바이너리에서 `FileNotFoundError` 로 죽는다.
4. `rank_results` 가 `ok=True` 인데 `score` 없는 행에서 `KeyError`.
5. `best_pack` / 렌더가 결손 pack·runner 에서 `KeyError`.
6. `invite` 도 `find_bin` 을 타서, 바이너리 없는 환경에서 초대장이 안 나온다.
7. 초대장·지문 스키마가 코드에만 있어 문서·시험이 같은 표를 공유하지 않는다.

## 3. 한 일

### 3.1 도구

`gym/tools/leaderboard.py`

- 예외 계층 `LeaderboardError` 와 code `io`/`format`/`chain`/`cli`/`schema`.
- `classify_exception` / `format_classified` / `as_dict`.
- `try_sha256` / `try_read_json` / `try_write_json` / `try_line_hashes` /
  `safe_listdir` — 비상승 거울.
- `chain_walk` 가 JSON·타입·읽기 실패를 `(항목, 이유)` 로 접음.
- `merkle_root` 가 비문자열 잎을 `SchemaError` 로 거절. 조용한 `str()` 없음.
- `board_fingerprint` 가 깨진 keyring/없는 파일을 죽지 않고 접음.
- `validate_invite` / `validate_fingerprint` / `validate_rank_row`.
- `rank_results` / `best_pack` / `pack_ratio` / `_finite_float` — NaN/Inf/
  결손을 unverified 또는 건너뜀.
- `render_markdown` / `short_commit` — runner 없으면 `—`.
- `verify_entry` 가 claims 목록·claim 읽기·settle/anchor CLI 실패를
  `ok=False` 로.
- `cmd_*` 바깥에서 분류된 실패를 exit 2 로.
- `invite` 는 바이너리를 찾지 않음. `main(argv=None)` 시험 입구.
- 상수 `INVITE_KEYS` · `FINGERPRINT_KEYS` · `MODES` (네 값, 새 CLI 없음).

### 3.2 시험

`scripts/tests/test_gym_leaderboard.py`

- 기존 33건 유지 (체인·머클·커밋된 판·순위·pack·렌더·임시 초대).
- 예외 분류, JSON I/O, 사슬 깨짐, 머클 스키마, 지문/초대 validate,
  순위/pack 결손, 렌더 방어, 명령 래퍼, verify_entry, 입장 deny,
  resolve_bin, 상수 계약, hashlib 만 쓰는지.

임시 판으로 경로를 돌려 커밋된 `gym/leaderboard/` 를 쓰지 않는다.

### 3.3 문서

- `gym/docs/leaderboard.md` — 규약. invite/attest/verify/render·예외 표.
- `gym/tools/README_leaderboard.md` — 운영 한 페이지.
- 이 파일 — 작업 기록.

packs·checks·coverage·다른 도구는 손대지 않았다. 새 CLI 모드 없음.
`cargo fmt --all` 없음. `git add -A` 없음.

## 4. 예외 경로 표 (시험과 같은 칸)

| 입력 | 기대 | 시험 |
|---|---|---|
| `LeaderboardIOError` | code `io` | `test_hierarchy_codes` |
| `FileNotFoundError` | `io` | `test_stdlib_exceptions_fold` |
| `JSONDecodeError` | `format` | 위와 같음 |
| `TypeError` | `schema` | 위와 같음 |
| `sha256_of(없는 파일)` | `LeaderboardIOError` | `test_sha256_of_missing_raises_io` |
| `try_sha256(없는 파일)` | `(None, err)` | `test_try_sha256_missing_is_none_and_err` |
| `read_json("{")` | `FormatError` | `test_read_json_malformed_raises_format` |
| `write_json(lambda)` | `FormatError` | `test_write_json_rejects_unserializable` |
| `try_write_json(디렉터리)` | err | `test_try_write_json_to_directory_is_error` |
| `safe_listdir(파일)` | `([], err)` | `test_safe_listdir_file_is_error` |
| `chain_walk("", …)` | 이유 문자열 | `test_empty_path_is_error` |
| 깨진 JSON 줄 | `(이전 항목, JSON …)` | `test_malformed_json_is_row_error_not_raise` |
| 배열 줄 | `객체가 아님` | `test_non_object_row_is_rejected` |
| `merkle_root("ab")` | `SchemaError` | `test_string_leaf_is_schema_error` |
| `merkle_root([1])` | `SchemaError` | `test_non_string_item_is_schema_error` |
| 지문 키 결손 | 위반 목록 | `test_validate_fingerprint_missing_key` |
| `construct_invite(123)` | `SchemaError` | `test_construct_invite_rejects_non_string_guest` |
| `construct_invite("  ")` | `친구-에이전트` | `test_construct_invite_blank_guest_uses_default` |
| 깨진 keyring | `members=0`, 비상승 | `test_fingerprint_bad_keyring_does_not_raise` |
| `rank_results("ab")` | `([], [])` | `test_rank_results_non_list_is_empty` |
| `ok=True` 에 score 없음 | unverified | `test_ok_row_without_score_is_unverified` |
| `score="NaN"` | unverified | `test_ok_row_with_non_numeric_score_is_unverified` |
| 깨진 pack 만 | `best_pack` None | `test_best_pack_all_malformed_is_none` |
| runner 없음 렌더 | 칸 `—`, 비상승 | `test_missing_runner_does_not_raise` |
| `run_cli("")` | `CliError` | `test_run_cli_empty_bin_is_cli_error` |
| `cmd_attest(agent=None)` | SystemExit | `test_cmd_attest_requires_agent` |
| 입장 deny | SystemExit | `test_admission_deny` |
| `main(["invite", …])` | 0, 바이너리 불필요 | `test_main_invite_does_not_need_bin` |
| `verify_entry("nope")` | `ok=False` | `test_non_dict_entry` |
| find_bin SystemExit | `CliError` | `test_systemexit_from_find_bin` |

## 5. 호환

기존 시험이 기대한 것:

- 유효 체인 2줄이 오류 없이 걷는다.
- 과거 줄 변조는 다음 줄 `prevEntryHash` 로 폭로.
- seq 공백 거부, 없는/빈/공백 파일은 빈 체인.
- 머클 결정성·홀수 층 복제·두 잎 이어 붙임.
- 커밋된 원장·앵커 무결, claim 파일 해시 일치.
- 지문에 7키, `ledgerEntries` 가 실제 체인 길이.
- 순위는 점수 내림·seq 오름, unverified 분리.
- `best_pack` 비율·동률 max·max=0.
- 렌더 헤더·정직 조항·unverified 행·능력 격자.
- 임시 판 초대가 커밋된 판을 쓰지 않음.

바뀐 것:

- `sha256_of` / `read_json` / `write_json` 이 원시 예외 대신 분류 예외.
  기존 시험은 존재하는 파일만 부르므로 통과.
- `chain_walk` 가 깨진 JSON 에서 예외 대신 이유 문자열. 기존 시험은 유효/
  변조된 객체 줄만 쓴다.
- `rank_results` 가 결손 score 를 unverified 로. 기존 시험 행은 score 가 있다.
- `best_pack` 이 결손 항목을 건너뜀. 기존 시험 pack 은 완전하다.
- `render_markdown` 이 runner 없으면 `—`. 기존 시험은 runner 가 있다.
- `main` 의 `invite` 가 `find_bin` 을 안 탄다. 새 시험이 고정.
- `cmd_invite` / `cmd_verify` / `cmd_render` 가 분류 실패 때 2 를 준다.
  기존 시험은 성공 경로만 본다.

`ok` 의미는 그대로다. 검증된 항목만 순위에 오른다.

## 6. 의도적으로 안 한 일

- 새 CLI 모드. 사용자 지시. `MODES` 는 네 값.
- 새 암호 (자체 Ed25519, HMAC, blake2). 사용자 지시. 서명은 rhwp.
- pack/과제/checks/coverage 변경. 원 PR 범위와 같다.
- 커밋된 `gym/leaderboard/` 재발급. 읽기만.
- 실제 rhwp 바이너리로 attest/verify 주행. 이 작업의 게이트는 unittest.
- `cargo fmt --all`. 사용자 지시. Rust 를 건드리지 않았다.
- 새 PR. 같은 가지에 커밋·푸시만.
- `git add -A`. 경로를 명시한다. 추적되지 않은 `gym/packs/work-receipt/` 는
  넣지 않는다.

## 7. 결정 기록

### 7.1 invite 가 바이너리를 안 찾는 이유

초대장은 커밋된 원장·앵커·발주서의 해시와 키링 길이만 묶는다. rhwp 를 부르지
않는다. `find_bin` 이 실패하면 신참을 부르는 안내 자체가 환경에 묶인다.
`main` 에서 `invite` 만 바이너리 탐색을 건너뛴다. attest/verify/render 는
그대로 찾는다.

### 7.2 항목 실패가 verify 를 안 죽이는 이유

한 claim 파일이 깨졌다고 전체 검증이 크래시하면, 나머지 항목의 폭로가
사라진다. `verify_entry` 는 `ok=False` + `why` 로 접고 다음 항목으로 간다.
바닥글의 `verified/N` 이 그 숫자를 말한다.

### 7.3 NaN 을 순위에 안 올리는 이유

`float("NaN")` 은 예외가 아니다. 그대로 두면 `ok=True` 행이 정렬 키 NaN 으로
순위에 오른다. `_finite_float` 가 NaN/Inf 를 거절하고 `rank_results` 가
unverified 로 접는다. 점수를 0 으로 바꾸지 않는다 — 그건 지어낸 숫자다.

### 7.4 머클 잎을 str() 하지 않는 이유

정수 잎을 조용히 문자열로 바꾸면 체크포인트와 다른 루트가 나온다. 재계산
일치가 거짓 음성이 된다. `SchemaError` 로 거절한다.

### 7.5 새 암호를 안 넣는 이유

모듈 문자열의 첫 문장이 "새 암호학 0줄" 이다. 여기서 Ed25519 를 다시 구현하면
rhwp `settle verify` 와 어긋날 수 있고, 검증 사다리의 단일 권위가 둘로 갈린다.
해시는 hashlib, 서명은 rhwp. 시험 `test_hashlib_sha256_is_the_only_digest` 가
`nacl`/`cryptography`/blake2 import 를 막는다.

## 8. 재현

```text
# 이 작업나무에서
python -m unittest scripts.tests.test_gym_leaderboard scripts.tests.test_gym_audit -q
python gym/tools/audit.py
```

바이너리가 있으면:

```text
python gym/tools/leaderboard.py verify
python gym/tools/leaderboard.py render
python gym/tools/leaderboard.py invite --agent 손님
```

이 작업은 바이너리 주행을 게이트에 넣지 않았다.

## 9. 파일 목록

| 경로 | 역할 |
|---|---|
| `gym/tools/leaderboard.py` | 도구. 예외 접기·순수 함수·기존 CLI 4모드 |
| `scripts/tests/test_gym_leaderboard.py` | 계약 시험 |
| `gym/docs/leaderboard.md` | 규약 정본 |
| `gym/tools/README_leaderboard.md` | 운영 메모 |
| `mydocs/working/gym_leaderboard.md` | 이 기록 |

## 10. 후속

- 릴리스 게이트(`release_gate.py`)가 verify 의 exit 3 을 이미 본다. 이번
  변경의 exit 2 (환경) 와 3 (사슬) 분리가 게이트 메시지에 드러나는지 한 번
  보면 좋다. 게이트 코드는 이 가지에서 안 고친다.
- 커밋된 판의 `invite.json` 은 만들지 않았다. 초대는 호출자가 임시/로컬로
  발급한다.
- 실제 attest 는 여전히 바이너리가 필요하다. 순수 시험이 대체하지 않는다.

## 11. 크기와 범위

원 PR 대비 `upstream/devel` 는 파일 2개 · 삽입 384 였다. 이번 커밋은 같은
가지에 예외 경로·시험·규약 문서를 얹어 삽입 3000 을 넘긴다. 늘어난 줄의
대부분은 (1) 분류된 예외와 비상승 거울, (2) 그 칸을 고정하는 unittest,
(3) invite/attest/verify/render 를 같은 표로 적는 한국어 규약이다.

넣지 않은 줄:

- pack JSON, 과제, 기준 풀이
- Rust 소스, `cargo fmt`
- 새 바이너리 명령
- 커밋된 원장·앵커 재발급
- 추적되지 않은 `gym/packs/work-receipt/` (다른 작업의 잔여. 이 커밋에 넣지 않음)

로컬에서 `audit.py` 가 `work-receipt` 의 `pack.json 이 없다` 를 내는 것은
그 잔여 디렉터리 때문이다. CI 클론에는 그 경로가 없다. 이 커밋의 게이트는
추적 파일만 본다.

## 12. 커밋 메시지 초안

```text
feat(gym): leaderboard 예외 경로와 초대·등재·검증·렌더 규약을 보강한다

I/O·JSON·사슬·CLI 실패를 분류해 한 항목의 읽기 실패가 전체 검증을
죽이지 않게 한다. 순위·최강 pack·렌더는 결손·NaN 을 unverified 로
접고, invite 는 바이너리 없이 판 지문만 묶는다. 새 암호와 새 CLI 는
없다. 규약·운영 메모·작업 기록을 같은 표로 고정한다.
```

이 초안을 실제 커밋에 쓴다. 새 PR 없음. `git add -A` 없음.
