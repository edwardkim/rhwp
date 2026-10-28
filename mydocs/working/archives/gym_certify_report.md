---
kind: working
status: active
canonical: mydocs/working/gym_certify_report.md
last_verified: 2026-08-18
---

# gym certify/report 채점 산출 예외 경로·문서·시험 보강

Issue: #5275
Branch: `feat/gym-certify-report-hardening`
Date: 2026-08-18
Worktree: `C:\Users\swsz9\rhwp-gym-certify-report` (isolation, not rhwp-desk*)

## 1. 결론

`gym/report.py` 와 `gym/certify.py` 의 합성·재현 성공 칸은 그대로 두고,
예전이 스택을 올리거나 미가용 pack 을 한 줄로만 접던 자리를 kind 로
남겼다. 없는 스코어카드는 `missing-scorecard` 이지 0점이 아니다.
깨진 JSON 은 `malformed-json` 이지 빈 카드가 아니다. 미가용 pack 은
`unavailable-pack` 이지 축 합산의 0점이 아니다.

검증:

- `python -m unittest scripts.tests.test_gym_report scripts.tests.test_gym_certify`
- `python gym/tools/audit.py`
- `cargo fmt --all -- --check` (HARD GATE, PR 전)

건드리지 않은 것:

- `gym/score.py` · `gym/core/runner.py` · `gym/tools/build_baseline.py`
- `gym/packs/expert-challenges` · `gym/tutorial` · `gym/PARK.md`
- 열린 PR 5210–5274 의 파일
- 새 CLI 플래그, 새 pack, 새 과제
- 정확도 정수 나눗셈, 축 라벨 규칙, 지문 알고리즘, verify 시그니처

규범: `gym/docs/certify_report.md`. 이 파일은 작업 기록이다.

## 2. 배경

원 도입은 리포트를 표준 계기로, 인증서를 재현 증명으로 두었다.
리포트는 scorecard + coverage 를 한 장으로 합친다. 인증서는 그 장의
재현 core(지문·바이너리 신원·정확도·커버리지·축)를 다시 돌려 대조한다.

대비 `upstream/devel` 의 구현은 리포트 약 190줄, 인증서 약 180줄,
시험은 리포트 4건·인증서 6건이었다. 성공 칸은 단단했다. 예외 칸은
열려 있었다.

그 상태의 빈틈:

1. `report.main` 이 `json.load(open(scorecard))` 를 그대로 부른다.
   파일이 없으면 FileNotFoundError. kind 가 없다. 종료 코드도 없다.
2. `--bin` 모드가 `submissions/_report/scorecard.json` 을 같은 방식으로
   연다. score.py 가 산출을 안 남기면 스택. 하위 도구 실패와 산출
   부재가 구분되지 않는다.
3. 잘린 JSON·빈 파일·배열 스코어카드가 JSONDecodeError 또는
   AttributeError. 깨진 입력을 0점으로 합성할 위험과, 도구가 죽을
   위험이 같이 있다.
4. 미가용 pack 은 `p["id"]` 로만 모은다. id 가 없으면 KeyError.
   인증서는 그 집합을 재현 대조하지 않는다. 벤치마크에서 명령 없는
   pack 을 지우면 점수가 같아 보여도 측정 폭이 줄어든다.
5. `certify.main --verify` 가 없는 파일·깨진 JSON 에서 스택.
   재현 실패(1)와 도구 실패가 한 예외로 접힌다.
6. `verify` 가 `cert.get("report", {})` 로 `report: null` 을 받으면
   기본값이 쓰이지 않아 `None.get` 으로 죽는다.
7. `_run_report` 가 비-0 을 RuntimeError 로 올린다. 스코어카드 부재를
   일반 실패로 뭉갠다.
8. 카탈로그가 코드에 없어 문서·시험이 같은 표를 공유하지 않는다.

이슈 #5275 의 DoD 는 이 빈틈을 닫는 것이다. additions >= 3000.
unittest + audit.py. PR 전 `cargo fmt --all -- --check`. 새 CLI/pack
없음. 열린 PR 파일 미수정.

채점기 보강(#5260)과 fuzz_corpus 보강(#5256)의 예외 접기를 참고하되,
리포트·인증서의 정체성은 유지했다.

- 리포트는 채점하지 않는다. 스코어카드를 읽는다.
- 인증서는 서명하지 않는다. 재현한다.
- 커버리지와 정확도를 한 숫자로 합치지 않는다.
- `--scorecard` 단독 허용 같은 계약 변경을 하지 않는다.

## 3. 한 일

### 3.1 리포트 `gym/report.py`

- `EXCEPTION_KINDS` / `EXCEPTION_KIND_HELP` — 문서·시험이 보는 표.
- `FATAL_EXCEPTIONS` / `CATCHABLE_EXCEPTIONS` — 삼키는 경계.
- `ReportError` — kind 를 가진 운영 예외.
- `classify_os_error(exc, role)` — FileNotFound 는 역할로
  `missing-scorecard` / `missing-coverage` / `missing-bin` 으로 갈린다.
- `load_text` / `parse_json_text` / `load_json_object` — 빈 경로,
  디렉터리, 부재, 권한, 디코드, 파싱, 비객체를 순서대로 접는다.
- `validate_scorecard` / `validate_coverage` — compile 이 죽지 않게
  예외 목록만 낸다.
- `compile_report` — 예전 숫자 공식 유지. `exceptions` ·
  `packsErrored` · `trusted` 를 뒤에 붙인다. unavailable pack 마다
  `unavailable-pack` 한 줄.
- `render_card` — 예전 줄 유지. 예외 절은 `## 예외 경로`. 커버리지
  없는 카드에 `커버리지` 단어를 쓰지 않는다.
- `_from_bin` — scorecard.json 부재는 `missing-scorecard`. 깨진
  JSON 은 `malformed-json`. 커버리지 실패는 예전처럼 빈 객체.
- `_run` — SystemExit 대신 `report-tool-failed`.
- `main(argv=None)` — ReportError 를 stderr `kind: message`, exit 2.
- CLI 플래그 불변. `REPORT_CLI_FLAGS` 를 시험이 대조한다.

### 3.2 인증서 `gym/certify.py`

- 같은 카탈로그 패턴. 인증서만의 kind: `missing-cert` `missing-report`
  `malformed-cert` `malformed-report` `wrong-kind` `fingerprint-empty`
  `verify-mismatch`.
- 지문 알고리즘을 `collect_fingerprint_entries` +
  `hash_fingerprint_entries` 로 쪼갰다. 해시 식은 그대로다. `.pyc` 와
  `__pycache__` 제외도 그대로다.
- `reproducible_core` 가 비객체 report/runner/coverage 를 빈 객체로
  본다. `None.get` 이 없다.
- `_run_report` 가 CertifyError. `classify_report_failure` 가 stderr
  표식으로 `missing-scorecard` 를 일반 실패에서 가른다.
- `certify` 가 `unavailablePacks` 와 `unavailable-pack` 예외를 붙인다.
  `--at` 은 예전처럼 `certifiedAt` 만. core 밖.
- `verify` 시그니처 `(bool, list[str])` 유지. 깨진 인증서는 False.
  미가용 집합 불일치를 diffs 에 더한다. kind 불일치 문구는 예전
  그대로.
- `main(argv=None)` — 없는 인증서·깨진 JSON 은 exit 2. 재현 실패는
  exit 1. 발급 성공은 0.

### 3.3 시험

- `scripts/tests/test_gym_report.py` — 예전 4건을 유지하고 카탈로그·
  적재·합성·카드·CLI·`--bin` 산출 부재를 더했다.
- `scripts/tests/test_gym_certify.py` — 예전 6건을 유지하고 카탈로그·
  지문 도우미·적재·분류·미가용·verify 방어·CLI 를 더했다.
- 문서 백틱 대조: 각 `EXCEPTION_KINDS` 항목이
  `gym/docs/certify_report.md` 에 `` `kind` `` 로 적혀 있어야 한다.

### 3.4 문서

- `gym/docs/certify_report.md` — 규약 정본. 왜, 사용, 봉투, pack
  삼원, kind 표 두 장, JSON 적재, 합성, 지문, 재현, 경계, 시험
  행렬, 비목표.
- `mydocs/working/gym_certify_report.md` — 이 기록.

## 4. 예외 세 자리 (이슈 DoD)

### 4.1 없는 스코어카드

자리:

- `python gym/report.py --scorecard missing.json --coverage cov.json`
- `python gym/report.py --bin <bin>` 이후 `submissions/_report/scorecard.json` 없음
- `python gym/certify.py --bin <bin>` 가 그 리포트를 부를 때

동작:

- 리포트 CLI: stderr `missing-scorecard: …`, exit 2. 카드를 짓지 않음.
- 리포트 `_from_bin`: ReportError `missing-scorecard`.
- 인증서: report.py 비-0 + 표식 → CertifyError `missing-scorecard`,
  exit 2. 빈 인증서 없음.

하지 않는 것: 빈 스코어카드를 0/0 으로 합성해 "만점 아님" 한 줄로
접기. 그것은 침묵이다.

### 4.2 깨진 JSON

자리:

- 스코어카드 `{` / `[]` / `""` / 숫자 스칼라
- 커버리지 같은 형태 (`--coverage` 를 준 경우)
- 인증서 `--verify` 같은 형태
- report.py stdout 이 카드 텍스트이거나 잘린 JSON

동작:

- 파싱 실패: `malformed-json`, 파일 모드 exit 2.
- 파싱은 되나 객체 아님: `malformed-scorecard` /
  `malformed-coverage` / `malformed-cert` / `malformed-report`.
- `compile_report` 에 비객체 가 직접 들어오면 던지지 않고 카드 +
  예외. CLI 적재 단계는 그 전에 끊는다.

하지 않는 것: `except Exception: scorecard = {}` 로 삼켜 0점 카드
발급. 깨진 입력을 측정으로 위장한다.

### 4.3 미가용 pack

자리:

- 스코어카드 pack `status=unavailable`
- 리포트 `packsUnavailable`
- 인증서 `unavailablePacks` 와 재현 집합 대조

동작:

- 축 프로파일에서 제외 (예전).
- `packsUnavailable` 에 id (예전, id 없으면 `?`).
- `exceptions` 에 `unavailable-pack` 한 줄 (추가).
- `trusted` 는 그대로 true. 부재는 구조 오류가 아니다.
- 카드에 미가용 줄 + 예외 절.
- 인증서 발급 시 같은 목록을 복사.
- verify 때 집합이 달라지면 "미가용 pack 불일치", exit 1.

하지 않는 것: 미가용 pack 을 0점으로 축에 넣기. error pack 을
unavailable 로 세기. 미가용 때문에 리포트 exit 를 2 로 올리기.

## 5. 성공 칸 회귀 점검

리포트 픽스처 SCORECARD 는 편집 2 + 조사 1 + 보안 unavailable 1.
합성 결과는 예전과 같아야 한다.

| 칸 | 값 |
|---|---|
| 편집 score/max/percent | 4 / 6 / 66 |
| 조사 score | 1 |
| 정확도 percent | 62 |
| 커버리지 percent | 82 |
| 축에 보안 | 없음 |
| packsUnavailable | `["d"]` |

추가로 `exceptions` 에 `unavailable-pack` / pack `d` 가 있다. 예전
시험은 extra 키를 보지 않으므로 통과한다.

인증서 픽스처 FIXED_REPORT 는 정확도 100, 지문 목킹 `FP`.
verify True, 정확도 위조 False, 지문 위조 False. 그대로다.

지문 민감도: packs/tasks JSON 변경, assets CSV 변경,
tools/build_baseline.py 변경. 알고리즘을 쪼개도 해시 식은 같다.

## 6. 경계 — 일부러 안 만진 파일

이슈가 명시한 금지:

- `score.py` `runner.py` `build_baseline.py`
- expert-challenges, tutorial, PARK
- PR 5210–5274 의 파일

그래서 넣지 않은 것:

- 채점기 `EXCEPTION_KINDS` 재사용 (runner 를 import 하면 그 PR 과
  겹칠 수 있다). 리포트·인증서는 자기 카탈로그를 가진다.
- gym/README.md · PARK.md 링크 추가. 입문 문서는 다른 이슈(#5263).
- coverage.py 선택 실패를 `missing-coverage` 로 승격. 예전 계약은
  선택적이다.
- `--scorecard` 단독 허용. 새 계약이다. 이슈는 새 CLI 금지.

열린 PR 파일 대조: certify.py / report.py / test_gym_certify.py /
test_gym_report.py / gym/docs/certify_report.md /
mydocs/working/gym_certify_report.md 는 5210–5274 에 없었다.

## 7. 검증 로그

작업 트리 `C:\Users\swsz9\rhwp-gym-certify-report` 에서 실행했다.

```text
python -m unittest scripts.tests.test_gym_report scripts.tests.test_gym_certify
# Ran 151 tests in 0.280s  OK

python gym/tools/audit.py
# gym 정합 감사: 18 pack 전부 통과 — 위반 0

cargo fmt --all -- --check
# exit 0 (변경 .rs 없음. sparse 트리에 crates/ 를 보탠 뒤 통과)

git diff --shortstat upstream/devel
# 추적 4파일 + 신규 문서 2파일. insertions >= 3000
```

- unittest 151건 OK. 예전 성공 칸 10건 + 예외 칸.
- audit: pack 정합 위반 0. pack JSON 을 안 만졌으므로.
- rustfmt HARD GATE: `cargo fmt --all -- --check` 통과. 변경된 `.rs`
  는 없어 별도 rustfmt 대상이 없었다.
- size gate: 문서 포함 insertions >= 3000.

## 8. PR

- base: `devel`
- title: `feat(gym): certify/report 채점 산출 예외·문서·시험 강화 (#5275)`
- body: 한글, `--body-file`, `closes #5275`
- 템플릿 칸: 변경 요약, 관련 이슈, 테스트, 성능.
- `cargo fmt --all -- --check` 를 PR 전에 실행했다고 적는다.

## 9. 남긴 위험 · 후속

1. `--bin` 실주행은 이 작업의 unittest 가 아니다. score.py /
   build_baseline 의 라이브 오라클은 #5260 · #5273 자리. 여기서는
   산출 부재·깨진 JSON 을 목킹으로만 고정한다.
2. 이미 발급된 인증서 JSON 에는 `exceptions` 칸이 없다. verify 는
   그 칸을 core 에 넣지 않으므로 예전 인증서도 재현된다. 미가용
   집합 대조는 report.packsUnavailable 만 본다. 예전 리포트에 그
   칸이 있으면 그대로 비교된다.
3. 리포트 `trusted=false` 를 리더보드가 아직 읽지 않는다. 이 작업이
   리더보드를 건드리지 않는다. 후속이 읽으면 구조 예외 있는 카드를
   입장에서 걸 수 있다.
4. 지문에 `gym/docs/certify_report.md` 를 넣지 않았다. 문서를 고칠
   때마다 모든 인증서가 무효가 되는 것을 피한다.
5. Windows 에서 디렉터리 `json.load` 는 PermissionError 일 수 있다.
   `load_text` 가 isdir 을 먼저 봐 malformed-* 로 접는다.

## 10. 파일 목록

| 경로 | 역할 |
|---|---|
| `gym/report.py` | 리포트 합성 + 예외 적재 |
| `gym/certify.py` | 인증서 발급/재현 + 예외 적재 |
| `scripts/tests/test_gym_report.py` | 리포트 계약 |
| `scripts/tests/test_gym_certify.py` | 인증서 계약 |
| `gym/docs/certify_report.md` | 규약 정본 |
| `mydocs/working/gym_certify_report.md` | 이 기록 |

`git add -A` 를 쓰지 않는다. 위 여섯 경로만 스테이징한다.
