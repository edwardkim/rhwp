---
kind: canonical
status: active
canonical: mydocs/manual/gym_benchmark_operations.md
last_verified: 2026-09-03
---

# Gym 벤치마크 수동 운영 매뉴얼

이 문서는 인간 개발자·메인테이너가 rhwp Gym의 구조 계약과 전수 벤치마크를 로컬에서
재현 가능하게 실행하고 판정하는 정본이다. 참가자가 개별 과제를 푸는 방법은
[`gym/README.md`](../../gym/README.md), 각 도구의 세부 계약은 [`gym/docs/`](../../gym/docs/)를
따른다. AI 에이전트가 Gym을 다룰 때는 이 문서와 함께
[`gym/AGENTS.md`](../../gym/AGENTS.md)를 적용한다.

## 1. 운영 경계

Gym은 AI 에이전트가 rhwp CLI/API를 조합해 과제를 수행하는 능력을 학습·평가하는
벤치마크다. 다음과 같은 용도로는 사용하지 않는 것을 권고한다.

- Gym 결과를 한컴 조판 동등성이나 제품 정확성의 독립 정답으로 승격하지 않는다.
- 일반 PR, `devel`/`main` push, 태그, Release Binary, npm·extension 게시를 Gym 전수
  결과로 자동 차단하지 않는다.
- `release_gate.py`의 역사적 `gate`·`block` 명칭을 제품 릴리스 승인으로 해석하지 않는다.
- 사설 코퍼스나 비공개 자산을 공개 Gym의 재현 조건으로 만들지 않는다.

전수 실행의 목적은 세 축을 함께 확인하는 것이다.

| 축 | 질문 | 도구 |
| --- | --- | --- |
| 양성 기준풀이 | 저장소의 모든 공개 과제를 기준풀이로 실제 완주할 수 있는가 | `build_baseline.py` |
| 음성 판별력 | 일하지 않은 오답·입력 복사·garbage를 채점기가 거부하는가 | `discriminate.py` |
| 경로 필요성 | 다단계 풀이의 마지막 의미 스텝이 실제 결과에 필요한가 | `trajectory.py` |

세 축 중 하나라도 미충족이면 **Gym 벤치마크 완주**를 선언하지 않는다. 다른 축의
성공으로 실패를 상쇄하지 않는다.

## 2. 언제 무엇을 실행하는가

### 빠른 구조 계약

Gym 문서·task·reference·채점기·도구를 수정한 PR의 기본 로컬 확인이다. 전수 기준풀이를
실행하지 않으므로 제품 릴리스 검증으로 오해하지 않는다.

```bash
python3 -m unittest discover -s scripts/tests -p 'test_gym_*.py'
python3 gym/tools/audit.py --json
python3 gym/tools/oracle_probe.py --json
python3 gym/tools/oracle_probe.py --selftest --json
```

### 선택적 pack canary

알려진 실패가 특정 pack에 모여 있거나 전수 실행 전에 바이너리·환경을 확인할 때만
사용한다. `build_baseline.py`에는 task 단위 CLI 필터가 없으므로 존재하지 않는 `--task`나
`--limit`를 만들지 않는다.

```bash
python3 gym/tools/build_baseline.py \
  --agent "maintainer-canary" \
  --pack <pack-id> \
  --bin "<rhwp-bin>" \
  --json
```

canary 통과는 전수 통과가 아니다. 최종 벤치마크 증적에는 아래 세 축을 전건 실행한다.

### 전수 벤치마크

다음 경우에 사람이 명시적으로 실행한다.

- task/reference/check/runner 또는 세 감사 도구의 의미가 바뀐 경우
- Gym 내부의 정답 권위·판별력·경로 필요성을 정산하는 경우
- 제품 변경이 Gym 과제 수행에 미치는 영향을 별도 조사하는 경우
- 이슈 수행계획이나 메인테이너가 전수 증적을 요구한 경우

일반 제품 PR과 릴리스에는 자동으로 추가하지 않는다.

## 3. 실행 전 신원과 환경 고정

필수 도구는 Git, Python 3, Rust toolchain과 Cargo다. WSL/Linux 셸을 기준으로 한다.
실행 전 다음 신원을 기록한다.

- Gym runner source commit과 tree SHA
- 제품 후보를 만든 source commit(동일 checkout이면 runner SHA와 같음)
- `rhwp --version`과 바이너리 SHA-256
- OS·architecture·Python·Rust 버전
- 시작 시각과 각 축의 종료 코드·경과 시간

먼저 저장소와 작업 트리를 확인한다.

```bash
git status --short --branch
git rev-parse HEAD
git rev-parse HEAD^{tree}
python3 --version
rustc --version
cargo --version
```

전수 도구는 `gym/submissions/` 아래에 많은 파일을 쓴다. 주 작업 checkout에서 직접
실행하지 않고 exact SHA의 disposable worktree를 사용한다. 아래 변수는 이 실행을 위해
새로 만든 경로만 가리켜야 한다.

```bash
RHWP_REPO=$(git rev-parse --show-toplevel)
GYM_RUN_ID=$(date +%Y%m%d-%H%M%S)
GYM_TMP=$(mktemp -d "/tmp/rhwp-gym-${GYM_RUN_ID}.XXXXXX")
GYM_WORKTREE="$GYM_TMP/worktree"
GYM_TARGET="$GYM_TMP/target"
GYM_EVIDENCE="$RHWP_REPO/output/gym/$GYM_RUN_ID"
GYM_RUNNER_SHA=$(git rev-parse HEAD)
GYM_PRODUCT_SHA="$GYM_RUNNER_SHA"

mkdir -p "$GYM_EVIDENCE"
git worktree add --detach "$GYM_WORKTREE" "$GYM_RUNNER_SHA"
cd "$GYM_WORKTREE"
```

현재 source와 Gym runner가 같은 commit이면 disposable worktree에서 바이너리를 만든다.

```bash
CARGO_TARGET_DIR="$GYM_TARGET" cargo build --locked --bin rhwp
RHWP_BIN="$GYM_TARGET/debug/rhwp"
```

제품 후보 commit과 Gym runner commit이 다르면 후보 checkout에서 바이너리를 먼저 만들고,
그 **절대경로**를 `RHWP_BIN`에 넣는다. `GYM_PRODUCT_SHA`도 후보 checkout의 commit으로 바꾼다.

```bash
RHWP_BIN="<candidate-checkout의 binary 절대경로>"
GYM_PRODUCT_SHA=$(git -C "<candidate-checkout>" rev-parse HEAD)
```

두 경우 모두 Gym runner worktree에서 다음 공통 metadata를 기록한다.

```bash
"$RHWP_BIN" --version | tee "$GYM_EVIDENCE/rhwp-version.txt"
sha256sum "$RHWP_BIN" | tee "$GYM_EVIDENCE/rhwp-bin.sha256"
printf '%s\n' "$GYM_RUN_ID" >"$GYM_EVIDENCE/run-id.txt"
git rev-parse HEAD | tee "$GYM_EVIDENCE/gym-runner-head.txt"
git rev-parse HEAD^{tree} | tee "$GYM_EVIDENCE/gym-runner-tree.txt"
printf '%s\n' "$GYM_PRODUCT_SHA" >"$GYM_EVIDENCE/product-source-head.txt"
date -Is | tee "$GYM_EVIDENCE/run-started.txt"
uname -a >"$GYM_EVIDENCE/platform.txt"
python3 --version >"$GYM_EVIDENCE/python-version.txt" 2>&1
rustc --version >"$GYM_EVIDENCE/rust-version.txt" 2>&1
```

서로 다른 task/reference tree를 섞어 복사하지 않는다. commit이 아닌 제품 binary는
`product-source-head.txt`를 위조해 채우지 말고 실행 신원을 확보한 뒤 시작한다.

## 4. 실행 순서

각 명령의 종료 코드는 명령 직후 기록한다. JSON stdout과 진단 stderr를 분리하면 자동
재검산과 사람 진단을 모두 보존할 수 있다.

### 4.1 구조·도구 사전 점검

```bash
SECONDS=0
python3 -m unittest discover -s scripts/tests -p 'test_gym_*.py' \
  >"$GYM_EVIDENCE/unit.txt" 2>&1
GYM_EXIT=$?
printf '%s\n' "$GYM_EXIT" >"$GYM_EVIDENCE/unit.exit"
printf '%s\n' "$SECONDS" >"$GYM_EVIDENCE/unit.seconds"

SECONDS=0
python3 gym/tools/audit.py --json \
  >"$GYM_EVIDENCE/audit.json" 2>"$GYM_EVIDENCE/audit.stderr"
GYM_EXIT=$?
printf '%s\n' "$GYM_EXIT" >"$GYM_EVIDENCE/audit.exit"
printf '%s\n' "$SECONDS" >"$GYM_EVIDENCE/audit.seconds"

SECONDS=0
python3 gym/tools/oracle_probe.py --json \
  >"$GYM_EVIDENCE/oracle-structural.json" 2>"$GYM_EVIDENCE/oracle-structural.stderr"
GYM_EXIT=$?
printf '%s\n' "$GYM_EXIT" >"$GYM_EVIDENCE/oracle-structural.exit"
printf '%s\n' "$SECONDS" >"$GYM_EVIDENCE/oracle-structural.seconds"

SECONDS=0
python3 gym/tools/oracle_probe.py --selftest --json \
  >"$GYM_EVIDENCE/oracle-selftest.json" 2>"$GYM_EVIDENCE/oracle-selftest.stderr"
GYM_EXIT=$?
printf '%s\n' "$GYM_EXIT" >"$GYM_EVIDENCE/oracle-selftest.exit"
printf '%s\n' "$SECONDS" >"$GYM_EVIDENCE/oracle-selftest.seconds"

SECONDS=0
python3 gym/tools/authority_ledger.py --json \
  >"$GYM_EVIDENCE/authority-ledger.json" \
  2>"$GYM_EVIDENCE/authority-ledger.stderr"
GYM_EXIT=$?
printf '%s\n' "$GYM_EXIT" >"$GYM_EVIDENCE/authority-ledger.exit"
printf '%s\n' "$SECONDS" >"$GYM_EVIDENCE/authority-ledger.seconds"
```

단위 계약 실패, `audit.ok=false`, `audit.issueCount>0`, `audit.toolFailed=true`, 또는 oracle
probe·authority ledger의 `ok=false`가 있으면 전수 **수용 판정**으로 넘어가지 않는다. 원인
진단을 위해 나머지를 실행할 수는 있지만 그 실행은 진단이라고 명시한다.

### 4.2 양성 기준풀이 전수

`--agent`는 다른 실행과 충돌하지 않는 이름을 사용한다. JSON 진행 메시지는 stderr,
최종 봉투는 stdout으로 나온다.

```bash
SECONDS=0
python3 gym/tools/build_baseline.py \
  --agent "maintainer-$GYM_RUN_ID" \
  --bin "$RHWP_BIN" \
  --json \
  >"$GYM_EVIDENCE/positive.json" \
  2>"$GYM_EVIDENCE/positive.stderr"
GYM_EXIT=$?
printf '%s\n' "$GYM_EXIT" >"$GYM_EVIDENCE/positive.exit"
printf '%s\n' "$SECONDS" >"$GYM_EVIDENCE/positive.seconds"
```

### 4.3 음성 판별력 전수

```bash
SECONDS=0
python3 gym/tools/discriminate.py \
  --bin "$RHWP_BIN" \
  --json \
  >"$GYM_EVIDENCE/discrimination.json" \
  2>"$GYM_EVIDENCE/discrimination.stderr"
GYM_EXIT=$?
printf '%s\n' "$GYM_EXIT" >"$GYM_EVIDENCE/discrimination.exit"
printf '%s\n' "$SECONDS" >"$GYM_EVIDENCE/discrimination.seconds"
```

### 4.4 경로 필요성 전수

```bash
SECONDS=0
python3 gym/tools/trajectory.py \
  --bin "$RHWP_BIN" \
  --json \
  >"$GYM_EVIDENCE/trajectory.json" \
  2>"$GYM_EVIDENCE/trajectory.stderr"
GYM_EXIT=$?
printf '%s\n' "$GYM_EXIT" >"$GYM_EVIDENCE/trajectory.exit"
printf '%s\n' "$SECONDS" >"$GYM_EVIDENCE/trajectory.seconds"
```

task·pack 수는 계속 늘 수 있다. 과거 보고서의 숫자를 기대값으로 하드코딩하지 말고 같은
실행에서 나온 봉투의 `taskCount`, `packs`, `results`로 집계를 재계산한다.

### 4.5 증적 seal과 사람용 HTML

일곱 JSON과 metadata·exit·seconds·stderr를 모두 기록한 직후 입력 집합을 seal한다. 먼저
`--seal`이 41개 필수 입력의 구조·집계·실행 신원과 SHA-256을 검증해
`evidence-manifest.json`을 쓴다. 성공한 seal 없이 `--out`부터 실행하지 않는다.

```bash
python3 gym/tools/evidence_report.py \
  --evidence-dir "$GYM_EVIDENCE" \
  --seal \
  >"$GYM_EVIDENCE/evidence-seal-summary.json" \
  2>"$GYM_EVIDENCE/evidence-seal.stderr"
GYM_SEAL_EXIT=$?
printf '%s\n' "$GYM_SEAL_EXIT" >"$GYM_EVIDENCE/evidence-seal.exit"
```

`GYM_SEAL_EXIT`가 0일 때만 HTML을 만든다.

```bash
python3 gym/tools/evidence_report.py \
  --evidence-dir "$GYM_EVIDENCE" \
  --out "$GYM_EVIDENCE/evidence-report.html" \
  >"$GYM_EVIDENCE/evidence-report-summary.json" \
  2>"$GYM_EVIDENCE/evidence-report.stderr"
GYM_REPORT_EXIT=$?
printf '%s\n' "$GYM_REPORT_EXIT" >"$GYM_EVIDENCE/evidence-report.exit"
```

종료 0은 유효한 seal의 전체 PASS, 종료 1은 유효한 seal의 FAIL 또는 INCOMPLETE다. 둘 다 HTML이
생성된다. 종료 2는 입력·신원·manifest가 무효하거나 혼합됐다는 뜻이며 새 HTML을 남기지 않는다.
종료 1을 생성 실패로 오해해 보고서를 버리거나, 종료 2를 성공 상태로 접지 않는다. HTML은 사람이
읽는 비권위 파생 뷰이며 JSON 봉투가 계속 기계 판독 정본이다. 전체 입력·상태·redaction 계약은
[`Gym 증적 seal·HTML 규약`](../../gym/docs/evidence_report.md)을 따른다.

## 5. 통과 판정

### 양성 기준풀이

다음 조건을 모두 만족해야 한다.

- 프로세스 exit 0, 봉투 `ok=true`, `exit=0`
- `taskCount>0`이고 `built==taskCount`
- `failed`, `skipped`, `missingArtifact`, `failedScore`, `buildError`가 모두 0
- 모든 `results[].ok`가 참

`skipped`는 기준풀이가 없다는 뜻이므로 전수 수용에서는 성공이 아니다.

### 음성 판별력

다음 조건을 모두 확인한다.

- 프로세스 exit 0, `ok=true`
- `falsePass`와 `falsePassControls`가 비어 있음
- `discriminating==taskCount`
- `loadErrors`, `buildErrors`, `toolErrors`가 비어 있고 `toolFailed=false`
- `controlCount`가 실제 `results` 행 수와 같음

`scoreErrors`는 일부러 잘못 만든 artifact가 채점 초기에 거부된 기록일 수 있으므로
비어 있지 않다는 이유만으로 실패시키지 않는다. 각 행이 해당 음성 대조의 **의도된 거부**와
대응하는지 정산한다. 원인을 설명할 수 없는 `scoreErrors`는 통과가 아니라 미판정이다.

### 경로 필요성

다음 조건을 모두 만족해야 한다.

- 프로세스 exit 0, `ok=true`, `exit=0`, `trusted=true`
- `theater`, `exceptions`, `toolErrors`가 비어 있음
- `missingBin=false`, `toolFailed=false`
- `loadBearing==taskCount`

`skipCount`의 단일-step 과제는 경로 필요성의 적용 대상이 아니므로 그 자체로 예외가 아니다.
대신 `exceptions`에 들어간 기준풀이 부재·깨진 JSON·실행 실패는 모두 미완료다.

## 6. 실패를 분류하는 법

| 분류 | 대표 증거 | 처리 |
| --- | --- | --- |
| 제품 결함 | 구조 감사가 깨끗한데 특정 positive reference가 실제 rhwp 명령에서 실패 | task/check를 약화하지 않고 별도 제품 이슈·브랜치로 분리 |
| 벤치마크 결함 | audit 위반, false-pass, theater, 기준풀이 부재·부정확한 권위 | Gym 이슈에서 task/reference/check/tool을 수정하고 세 축 재실행 |
| 환경 결함 | 바이너리 부재, 권한, 디스크, OOM, 사용자 중단 | 같은 SHA·명령으로 환경을 복구한 뒤 재실행; 통과로 기록하지 않음 |
| 의도된 제품 변화 | 구·신 동작 차이가 있으나 어느 쪽이 옳은지 Gym만으로 판정 불가 | 제품 정답지·명세·별도 회귀 근거로 판정 |

양성 실패 뒤 discrimination·trajectory를 진단 목적으로 계속 돌릴 수 있다. 이 경우 보고서에
“다른 두 축은 진단 완료, 전체 Gym 수용은 양성 실패로 미완료”라고 쓴다. 실패한 task를
통과시키려고 `allowExits`, check, reference 또는 권위 분류를 느슨하게 하지 않는다.

## 7. 증적과 보고

최소 보고 항목은 다음과 같다.

1. 실행 목적과 이슈 번호
2. Gym runner head/tree SHA와 제품 source SHA
3. 바이너리 버전·SHA-256, OS/architecture, Python·Rust 버전
4. 실행한 정확한 명령, 시작·종료 시각, 각 종료 코드와 경과 시간
5. 구조 감사와 세 축의 봉투 집계
6. 실패 task/control/step, 오류 종류, 제품·벤치마크·환경 분류
7. 예외를 승인했다면 범위·근거·재검토 조건
8. 생성물 위치와 정리 여부

원문 JSON을 보존하되 제출물 전체를 장기 보존할 필요는 없다. 식별에 필요한 요약·해시와
실패 재현 자산만 남긴다. 사설 코퍼스의 경로·파일명·본문·식별 가능한 증거를 공개 보고서나
PR에 넣지 않는다.

전수 실행 직후에는 4.5의 seal을 남기고 HTML을 생성한다. manifest는 입력 집합의 영수증이고 HTML은
그 seal을 사람이 읽기 쉽게 펼친 비권위 파생 뷰다. 둘 중 어느 것도 JSON 봉투의 판정 권위를
대체하지 않는다. HTML을 공개할 때도 사설 코퍼스의 원문 JSON이나 stderr를 함께 게시하지 않는다.

## 8. 정리

먼저 원래 저장소로 돌아와 disposable worktree의 상태를 확인한다.

```bash
cd "$RHWP_REPO"
git -C "$GYM_WORKTREE" status --short
git status --short
```

`GYM_WORKTREE`가 이번 실행에서 만든 정확한 경로인지 확인한 뒤 제거한다. 생성 제출물은
disposable worktree와 함께 사라지고 `GYM_EVIDENCE`는 남는다.

```bash
git worktree remove --force "$GYM_WORKTREE"
rm -rf -- "$GYM_TMP"
git worktree prune
git status --short --branch
```

`$GYM_TMP`가 비어 있거나 `/`, `$HOME`, 저장소 루트 같은 넓은 경로를 가리키면 제거하지
않는다. 주 작업 checkout의 `gym/submissions/`에 파일이 생겼다면 stage하지 말고, 생성
주체와 정확한 경로를 확인한 뒤 해당 실행의 산출물만 정리한다.

## 9. 관련 규약

- [Gym 참가자 안내](../../gym/README.md)
- [기준풀이 조립기](../../gym/docs/build_baseline.md)
- [전 pack 정합 감사](../../gym/docs/audit.md)
- [판별력 감사](../../gym/docs/discriminate.md)
- [트라젝토리 필요성 감사](../../gym/docs/trajectory.md)
- [정답 권위 원장](../../gym/docs/authority_ledger.md)
- [Gym 증적 seal·HTML 규약](../../gym/docs/evidence_report.md)
- [Gym 범위 AI 에이전트 지침](../../gym/AGENTS.md)
