---
kind: guide
status: active
canonical: gym/docs/certify_report.md
last_verified: 2026-08-18
---

# gym 능력 리포트·인증서 예외 경로 규약

이 문서는 `gym/report.py` 와 `gym/certify.py` 의 **예외 경로 계약**,
**스코어카드·JSON·미가용 pack 삼원**, **재현 core**, **보고 카드**를
고정한다. 작업 기록은
[`mydocs/working/gym_certify_report.md`](../../mydocs/working/gym_certify_report.md)
를 본다. 시험 계약은 `scripts/tests/test_gym_report.py`(리포트 칸)와
`scripts/tests/test_gym_certify.py`(인증서 칸)가 기계로 고정한다.

채점기(`score.py` / `gym/core/runner.py`)·기준 풀이 조립기
(`build_baseline.py`)·커버리지(`coverage.py`)는 이 문서의 대상이 아니다.
리포트가 그 산출을 소비할 수는 있어도, 이 규약이 그 도구의 판정 삼원을
바꾸지 않는다. expert-challenges pack, tutorial, PARK 도 이 문서가
고치지 않는다.

새 플래그는 없다. 리포트는 `--bin` `--scorecard` `--coverage` `--json`
`--out` 만 쓴다. 인증서는 `--bin` `--verify` `--out` `--at` 만 쓴다.
종료 코드도 예전과 같다. 리포트 0=합성 성공, 2=도구 실패. 인증서
0=발급/재현 통과, 1=재현 불일치, 2=도구 실패.

## 1. 왜 이 기둥이 필요한가

리포트는 흩어진 계기(점수·커버리지·축·runner 신원)를 한 장으로 합친다.
인증서는 그 한 장이 **같은 벤치마크·같은 바이너리**에서 다시 나오는지를
결정론적으로 증명한다. 암호 서명이 아니다. 재현이 증명이다.

그런데 예전이 실패를 데이터로 남기지 못하는 자리가 있었다.

- `--scorecard` 경로가 없으면 `FileNotFoundError` 스택이 올라간다.
  기계가 "없는 스코어카드"를 읽지 못한다.
- `--bin` 모드에서 `score.py` 가 `scorecard.json` 을 안 남기면
  `json.load(open(...))` 이 같은 스택으로 죽는다. 하위 도구 실패와
  산출 부재가 한 줄로 접힌다.
- 스코어카드가 `{` 로 잘리거나 배열이면 `JSONDecodeError` 또는
  `AttributeError`. 깨진 JSON 을 0점으로 합성하면 거짓이다.
- 미가용 pack 은 `packsUnavailable` 에 id 만 남고, 카드 한 줄로
  접힌다. 인증서가 그 집합을 재현 대조하지 않으면 벤치마크를 몰래
  줄여 "명령이 없는 pack 을 지운 척" 할 수 있다.
- 인증서 `--verify` 가 없는 파일·깨진 JSON 에서 스택을 올린다.
  재현 실패(exit 1)와 도구 실패(exit 2)가 구분되지 않는다.
- `verify` 가 `cert.get("report", {})` 로 `report: null` 을 받으면
  `None.get` 으로 죽는다. 위조 탐지 전에 도구가 죽는다.

2026 벤치마크의 다른 위기는 false-pass 이지만, 리포트·인증서 자체의
위기는 **false-silence** 와 **false-crash** 다. 없는 산출을 만점처럼
합성하면 침묵이고, 깨진 입력을 스택으로 올리면 기계가 kind 를 못
읽는다. 둘 다 능력 증명이 아니다.

그래서 예외는 삼키지 않고 kind 로 남긴다. 합성 성공 칸의 숫자
(정확도 정수 나눗셈, 축 라벨, 커버리지 분리)는 그대로다. 미가용
pack 은 0점도 unavailable 위장도 아니다. 축 합산에서 빼고
`packsUnavailable` 과 `unavailable-pack` 예외 칸에 같이 남긴다.

## 2. 사용

```bash
python gym/report.py --bin target/debug/rhwp
python gym/report.py --bin target/debug/rhwp --json
python gym/report.py --scorecard sc.json --coverage cov.json
python gym/report.py --bin target/debug/rhwp --out report.md

python gym/certify.py --bin target/debug/rhwp --out cert.json
python gym/certify.py --verify cert.json --bin target/debug/rhwp
```

리포트 인자:

| 인자 | 기본 | 의미 |
|---|---|---|
| `--bin` | 없음 | rhwp 바이너리. 전 pack 채점+커버리지를 돌린다. |
| `--scorecard` | 없음 | 이미 있는 `score.py` 스코어카드 JSON. |
| `--coverage` | 없음 | 이미 있는 `coverage.py --json` 산출. |
| `--json` | 꺼짐 | 사람용 카드 대신 JSON 봉투. |
| `--out` | stdout | 출력 파일. UTF-8, BOM 없음, LF. |

인증서 인자:

| 인자 | 기본 | 의미 |
|---|---|---|
| `--bin` | 필수 | rhwp 바이너리. 발급과 재현 모두 필요하다. |
| `--verify` | 없음 | 검증할 인증서 JSON. 있으면 재현 모드. |
| `--out` | stdout | 발급 인증서 출력 파일. |
| `--at` | 없음 | `certifiedAt` 메타. 재현 core 에 들어가지 않는다. |

`--scorecard` 만 주고 `--coverage` 를 빼면 예전처럼 필수 인자 오류다.
커버리지를 선택적으로 만들고 싶어도 새 플래그를 만들지 않는다. 파일
모드의 계약은 `(--scorecard + --coverage)` 쌍이다. `--bin` 모드에서
`coverage.py` 부재·실패는 예전처럼 빈 커버리지다.

새 플래그는 없다. `--strict` `--limit` `--task` `--timeout` `--json`
(인증서) 을 붙이지 않는다. 기계 봉투는 리포트 `--json` 과 인증서
stdout/ `--out` JSON 이다.

종료 코드:

| 도구 | 코드 | 상수 | 의미 |
|---|---|---|---|
| report | 0 | `EXIT_OK` | 합성 성공. 미가용 pack 이 있어도 0. |
| report | 2 | `EXIT_TOOL_FAILED` | 없는 스코어카드, 깨진 JSON, 인자 부족, 하위 도구 실패 |
| certify | 0 | `EXIT_OK` | 발급 성공 또는 재현 통과 |
| certify | 1 | `EXIT_VERIFY_FAIL` | 재현 core 불일치 또는 `wrong-kind` |
| certify | 2 | `EXIT_TOOL_FAILED` | 없는 인증서, 깨진 JSON, 없는 스코어카드, report.py 실패 |
| 둘 다 | 2 | argparse | 필수 인자 없음. 예전 argparse 계약 |

도구 자리 오류를 위한 새 종료 코드를 만들지 않는다. 미가용 pack 은
데이터가 아니라 실패가 아니므로 0 이다. 재현 불일치는 1 이다. 없는
파일·깨진 JSON 은 2 이다. 1 과 2 를 섞으면 CI 가 위조와 하네스
결함을 구분하지 못한다.

## 3. 두 봉투 — 바꾸지 않는 칸

리포트 `kind` = `gymCapabilityReport`, `schemaVersion` = `1.0`.
인증서 `kind` = `gymCapabilityCertificate`, `schemaVersion` = `1.0`.
이 두 칸을 올리면 리더보드·게이트·사람이 읽지 못한다.

리포트 고정 키(`REPORT_KEYS`):

| 키 | 의미 |
|---|---|
| `kind` | `gymCapabilityReport` |
| `schemaVersion` | `1.0` |
| `agent` | 스코어카드 agent. 없으면 null |
| `runner` | 실행 신원. 없으면 null |
| `accuracy` | `{score, max, percent}` — 측정된 것 통과율 |
| `coverage` | `{percent, covered, agentFacingTotal, uncoveredByCategory}` |
| `axisProfile` | 축 라벨별 합산. scored pack 만 |
| `packsScored` | 스코어카드 total.packsScored |
| `packsUnavailable` | status=`unavailable` 인 pack id 목록 |
| `packsErrored` | status=`error` 인 pack id 목록 (부가) |
| `exceptions` | 예외 레코드 목록 |
| `exceptionCount` | `exceptions` 길이 |
| `trusted` | 구조 예외가 없으면 true |

부가 키 `packsErrored` · `exceptions` · `exceptionCount` · `trusted` 는
집계를 뒤집지 않는다. `trusted` 는 정보성 kind
(`unavailable-pack` · `empty-packs` · `empty-total`) 만 있을 때 참이다.
깨진 JSON·비객체 스코어카드가 있으면 거짓이다.

인증서 고정 키(`CERT_KEYS`):

| 키 | 의미 |
|---|---|
| `kind` | `gymCapabilityCertificate` |
| `schemaVersion` | `1.0` |
| `benchmarkFingerprint` | 측정 입력 sha256 hex 64자 |
| `report` | 리포트 봉투 전체 |
| `proof` | 재현 방법 한 줄. 고정 문구 |
| `certifiedAt` | `--at` 이 있을 때만. 재현 core 밖 |
| `unavailablePacks` | 리포트 `packsUnavailable` 복사 |
| `exceptions` | 예외 레코드 |
| `exceptionCount` | 예외 개수 |
| `trusted` | 구조 예외 0 이면 true |

`proof` 문구는
`reproduce: 같은 bin + 같은 pack 정의로 --verify 하면 core 가 일치한다`.
이 문자열을 바꾸면 이미 발급된 인증서와 사람이 읽는 안내가 갈린다.

정확도와 커버리지는 **다른 것**이다. 정확도는 측정된 pack 의 통과율이다.
커버리지는 에이전트-대면 능력 중 gym 이 재는 비율이다. 한 숫자로
뭉치면 "적게 재고 만점"과 "많이 재고 중간"이 같아진다. 카드는 두 줄을
따로 쓴다. 커버리지가 없으면 그 줄을 뺀다.

## 4. pack 상태 삼원 — 리포트가 소비하는 칸

채점기가 남기는 pack `status` 는 `scored` · `unavailable` · `error`
다. 리포트는 이 삼원을 다시 채점하지 않는다. 읽기만 한다.

| status | 축 합산 | 총점 | `packsUnavailable` | `packsErrored` | 예외 kind |
|---|---|---|---|---|---|
| `scored` | 예 | 스코어카드 total 을 신뢰 | 아니오 | 아니오 | 없음 |
| `unavailable` | 아니오 | 아니오 | 예 | 아니오 | `unavailable-pack` |
| `error` | 아니오 | 아니오 | 아니오 | 예 | 없음(채점기 자리) |

규칙:

1. **부재는 실패가 아니다.** 오래된 바이너리에게 0점은 거짓말이다.
   그 자리는 `unavailable` 이다. 리포트는 축에서 빼고 id 를 남긴다.
2. **도구 실패는 부재가 아니다.** pack.json 이 깨진 것을 "명령이 없다"고
   부르면 다음 사람이 바이너리를 탓한다. 그 자리는 `error` 이다.
   리포트는 `packsErrored` 에만 넣고 `unavailable-pack` 을 붙이지 않는다.
3. **error 를 unavailable 로 세지 않는다.** `packsUnavailable` 은
   `status==unavailable` 인 id 다. `len(packs) - packsScored` 로
   되돌리면 error 가 명령 부재로 위장된다.
4. **없는 id 는 `?`.** pack 행에 id 가 없거나 공백이면 자리를 비우지
   않고 `?` 로 남긴다. 침묵이 더 위험하다.

`total` 의 `score` / `max` / `packsScored` 는 스코어카드가 계산한
값이다. 리포트는 다시 합산하지 않는다. 축 프로파일만 scored pack 을
다시 더한다. 두 숫자가 어긋나면 스코어카드 버그이지 리포트가 고치는
칸이 아니다.

## 5. 예외 kind 카탈로그 — 리포트

`gym/report.py` 의 `EXCEPTION_KINDS` 와 `EXCEPTION_KIND_HELP` 가 정본이다.
시험이 문서에 각 kind 가 백틱으로 적혔는지 대조한다.

| kind | 자리 | 점수에 미치는 영향 | 종료 |
|---|---|---|---|
| `missing-scorecard` | `--scorecard` 파일 없음, 또는 `--bin` 후 scorecard.json 없음 | 합성하지 않음 | 2 |
| `missing-coverage` | `--coverage` 파일 없음 | 합성하지 않음 | 2 |
| `missing-bin` | `--bin` 값이 공백 | 하위 도구를 시작하지 않음 | 2 |
| `missing-report-arg` | `--bin` 도 쌍도 없음 | 예전 필수 인자 문구 | 2 |
| `malformed-json` | UTF-8 JSON 파싱 실패 | 합성하지 않음 | 2 |
| `malformed-scorecard` | 스코어카드가 객체 아님(배열·스칼라·디렉터리) | 파일 모드 2. compile 은 빈 카드+예외 | 2 또는 0 |
| `malformed-coverage` | 커버리지가 객체 아님 | 파일 모드 2. compile 은 빈 커버리지 | 2 또는 0 |
| `malformed-pack-row` | packs[i] 가 객체 아님 | 그 칸만 건너뜀. trusted=false | 0 |
| `unavailable-pack` | pack status=`unavailable` | 축·총점에서 제외. 목록+예외 | 0 |
| `empty-packs` | packs 없음/빈 목록 | 축 빈 목록. trusted 유지 | 0 |
| `empty-total` | total 없음 또는 max=0 | 정확도 0/0. 만점이 아님 | 0 |
| `permission` | 읽기·쓰기 권한 없음 | 합성·기록 중단 | 2 |
| `os-error` | 그 밖 OSError | 합성·기록 중단 | 2 |
| `decode-error` | UTF-8 디코드 실패 | 합성하지 않음 | 2 |
| `type-error` | 값 타입 불일치 | 해당 자리 | 2 |
| `value-error` | 값 형태 불일치 | 해당 자리 | 2 |
| `write-error` | `--out` 기록 실패 | 카드가 디스크에 없음 | 2 |
| `report-tool-failed` | build_baseline/score 비-0 | 산출을 지어내지 않음 | 2 |
| `unexpected` | 미분류 운영 예외 | 해당 자리 | 2 |

`FileNotFoundError` 의 kind 는 역할에 따른다. 스코어카드 자리면
`missing-scorecard`, 커버리지 자리면 `missing-coverage`, 바이너리
자리면 `missing-bin`. 한 예외 타입을 한 kind 로 고정하면 없는
스코어카드를 없는 커버리지로 부른다.

`--bin` 모드에서 `coverage.py` 가 없거나 stdout 이 깨져도
`missing-coverage` 가 아니다. 커버리지 측정기는 선택적이다. 빈
객체를 넣고 정확도·축만 낸다. `--coverage` 를 **준** 뒤에 파일이
없을 때만 `missing-coverage` 다.

## 6. 예외 kind 카탈로그 — 인증서

`gym/certify.py` 의 `EXCEPTION_KINDS` 와 `EXCEPTION_KIND_HELP` 가 정본이다.
리포트와 이름이 같은 kind 는 같은 뜻이다. 인증서만의 자리는 아래에
적는다.

| kind | 자리 | 재현에 미치는 영향 | 종료 |
|---|---|---|---|
| `missing-cert` | `--verify` 파일 없음·빈 경로 | 재현을 시작하지 않음 | 2 |
| `missing-bin` | `--bin` 공백 | report.py 를 부르지 않음 | 2 |
| `missing-scorecard` | report.py stderr 에 스코어카드 부재 | 빈 인증서를 발급하지 않음 | 2 |
| `missing-report` | 인증서에 report 칸이 없음 | verify 가 False | 1 |
| `malformed-json` | 인증서 파일 또는 report stdout 파싱 실패 | 발급/검증 중단 | 2 |
| `malformed-cert` | 인증서가 객체 아님 | 검증 중단 | 2 |
| `malformed-report` | report stdout 이 객체 아님·빈 본문 | 발급 중단 | 2 |
| `wrong-kind` | kind 가 `gymCapabilityCertificate` 가 아님 | 재현 실패. 예전 문구 | 1 |
| `unavailable-pack` | 리포트 미가용 pack 을 인증서에 복사 | 발급은 0. 집합 불일치는 1 | 0 또는 1 |
| `fingerprint-empty` | 지문 입력이 0개 | 발급은 하되 예외 칸 | 0 |
| `report-tool-failed` | report.py 비-0, 더 구체적 분류 없음 | 발급 중단 | 2 |
| `verify-mismatch` | 재현 core 불일치(내부 분류) | 사람 문구는 예전 접두 | 1 |
| `permission` | 인증서 읽기·쓰기 권한 | 중단 | 2 |
| `os-error` | 그 밖 OSError | 중단 | 2 |
| `decode-error` | UTF-8 디코드 실패 | 중단 | 2 |
| `write-error` | `--out` 기록 실패 | 인증서가 디스크에 없음 | 2 |
| `type-error` | 값 타입 불일치 | 해당 자리 | 2 |
| `value-error` | 값 형태 불일치 | 해당 자리 | 2 |
| `unexpected` | 미분류 | 해당 자리 | 2 |

`wrong-kind` 는 exit 1 이다. 다른 봉투를 인증서로 들이미는 것은
도구 고장이 아니라 재현 실패다. 예전 `verify` 가 False 를 주던
칸을 2 로 올리면 이미 그 exit 를 읽는 스크립트가 깨진다.

`verify-mismatch` 는 카탈로그 이름이다. 사람이 보는 문구는 예전
그대로다.

- 지문: `벤치마크 지문 불일치 — pack 정의가 인증 시점과 다르다(축소·변조 가능)`
- 바이너리: `바이너리 신원(capabilitiesSha256) 불일치 — 다른 바이너리다`
- 정확도: `정확도 불일치: 인증 … vs 재현 …`
- 커버리지: `커버리지 불일치: 인증 … vs 재현 …`
- 축: `축별 프로파일 불일치`
- 미가용: `미가용 pack 불일치: 인증 … vs 재현 …`

앞 다섯 문구를 바꾸면 `test_gym_certify.py` 의 위조 탐지 칸이 깨진다.
여섯 번째는 #5275 가 더한 칸이다. 미가용 집합이 달라지면 벤치마크가
줄어든 것과 같은 급이다.

## 7. 치명 예외 — 삼키지 않는 자리

양쪽 모두 `FATAL_EXCEPTIONS = (KeyboardInterrupt, SystemExit, MemoryError, GeneratorExit)`.

운영 예외(`CATCHABLE_EXCEPTIONS`)만 접는다. `except Exception` 으로
`BaseException` 을 삼키지 않는다. 시험이 `is_fatal_exception` 과
`is_catchable_exception` 경계를 고정한다.

`wrap_exception` 이 치명 예외를 받으면 다시 던진다. ReportError /
CertifyError 는 그대로 통과한다. FileNotFound 는 역할로 kind 가
갈린다. JSONDecodeError 는 항상 `malformed-json` 이다. UnicodeError
는 `decode-error` 다. PermissionError 는 `permission` 이다.

bool 은 int 의 하위 타입이다. pack `score: true` 를 1점으로 세지
않는다. `as_int` 는 `type(v) is int` 이거나 정수 float 만 받는다.
`"3"` 과 `True` 는 기본값 0 이다.

## 8. JSON 적재 계약

파일 적재는 한 함수로 모은다. 리포트는 `load_json_object(path, role=)`.
인증서는 `load_cert` / `load_json_object(path, role="cert")`.

순서:

1. 경로가 비었거나 공백이면 역할별 missing-*.
2. 경로가 디렉터리면 역할별 malformed-*.
3. 파일이 없으면 역할별 missing-*.
4. 권한이 없으면 `permission`.
5. UTF-8 이 아니면 `decode-error`.
6. 그 밖 읽기 실패는 `os-error`.
7. `json.loads` 실패는 `malformed-json`.
8. 결과가 dict 가 아니면 역할별 malformed-*.

역할 표:

| role | 없음 | 비객체 |
|---|---|---|
| scorecard | `missing-scorecard` | `malformed-scorecard` |
| coverage | `missing-coverage` | `malformed-coverage` |
| bin | `missing-bin` | (해당 없음, 실행 자리) |
| cert | `missing-cert` | `malformed-cert` |
| report | (stdout 자리) | `malformed-report` |

빈 파일은 JSON 이 아니므로 `malformed-json` 이다. `[]` 는 파싱은
되지만 객체가 아니므로 malformed-scorecard / malformed-coverage /
malformed-cert 다. 이 둘을 섞으면 "파일이 깨졌다"와 "배열을 카드로
줬다"를 구분하지 못한다.

기록은 UTF-8, BOM 없음, `newline="\n"`, indent=2, 끝 개행 하나.
`write-error` 는 부모 폴더를 못 만들거나 권한이 없을 때다.

## 9. 스코어카드 합성 — 죽지 않는 칸

`compile_report(scorecard, coverage)` 는 순수 함수다. 파일·바이너리에
가지 않는다. 깨진 입력을 던져도 예외를 올리지 않고 카드 + `exceptions`
를 낸다. CLI 적재 단계는 그 앞에서 ReportError 로 끊을 수 있다.

합성 규칙(예전 칸, 바꾸지 않음):

1. `axis_label` 은 첫 ` (` 앞. 빈 문자열은 `미분류`.
2. scored pack 만 축에 더한다. score/max 는 `as_int`.
3. 축 percent 는 `100 * score // max`. max 0 이면 0.
4. 축 정렬은 `(-percent, axis)`.
5. 정확도는 `total.score` / `total.max` 의 같은 공식.
6. 커버리지 `percent` 는 입력의 `coveragePercent` 를 옮긴다.
7. `packsUnavailable` 은 unavailable pack 의 id.

추가 칸:

8. 각 unavailable pack 마다 `unavailable-pack` 예외 한 줄.
9. packs 가 없거나 `[]` 이면 `empty-packs`.
10. total 이 없거나 max 0 이면 `empty-total`.
11. packs[i] 가 객체가 아니면 `malformed-pack-row`, 그 칸 skip.
12. coverage 가 객체가 아니면 `malformed-coverage`, 빈 커버리지.
13. scorecard 가 객체가 아니면 `malformed-scorecard`, 빈 카드.

카드 렌더는 예전 줄을 유지한다. 정확도 줄, 선택적 커버리지 줄,
채점 pack, 미가용 pack, runner. 뒤에 오류 pack 줄과 신뢰 줄과
`## 예외 경로` 를 붙일 수 있다. 커버리지가 없으면 그 단어를 카드에
쓰지 않는다. 이 불변식은 `test_coverage_is_optional` 이 지킨다.

예외 레코드 키: `kind` `message` 필수. `where` `path` `pack` `role`
은 있을 때만. 빈 문자열은 생략한다.

## 10. 벤치마크 지문

지문은 측정 입력의 결정론적 sha256 이다. 문서와 mydocs 는 점수를
바꾸지 않아 넣지 않는다.

넣는 것:

- 트리 `packs` `core` `profiles` `tools` ( `__pycache__` 제외,
  `.pyc` 제외)
- 파일 `score.py` `report.py` `certify.py`

알고리즘(바꾸지 않음):

1. 각 파일을 `(posix 상대경로, 바이트)` 로 모은다.
2. 상대경로로 정렬한다.
3. 각 칸마다 `rel utf-8` + `NUL` + `sha256(bytes)` 를 누적한다.
4. 바깥 sha256 hex 64자를 낸다.

같은 바이트라도 상대경로가 다르면 지문이 다르다. pack 하나를
다른 폴더로 옮기는 것은 다른 벤치마크다. 빈 디렉터리는 입력이
0개다. 지문 값은 여전히 64자지만 `fingerprint-empty` 를 남긴다.
그 인증서는 "아무것도 안 재고 봉인했다"는 뜻이다.

`--at` 과 git commit 과 agent 이름은 지문에 없다. 같은 바이너리·
같은 pack 이면 다른 날 발급해도 core 가 같다.

## 11. 재현 core

`reproducible_core(report, fingerprint)` 가 대조하는 다섯 칸:

| 칸 | 출처 | 빠지는 것 |
|---|---|---|
| `benchmarkFingerprint` | 인자 | — |
| `capabilitiesSha256` | `report.runner` | `rhwpCommit` `rhwpVersion` |
| `accuracy` | `report.accuracy` 전체 | — |
| `coverage` | percent / covered / agentFacingTotal | uncoveredByCategory |
| `axisProfile` | `report.axisProfile` 전체 | — |

`report` 가 객체가 아니면 빈 객체로 본다. `runner` / `coverage` 가
객체가 아니면 빈 객체다. `None.get` 으로 죽지 않는다.

`verify` 순서:

1. 인증서가 객체가 아니면 False.
2. kind 가 `gymCapabilityCertificate` 가 아니면 False. 문구는
   `kind 가 gymCapabilityCertificate 가 아니다: …`.
3. report 가 객체가 아니면 False.
4. `_run_report` 가 CertifyError 면 False, 이유 `kind: message`.
5. `compare_core` 다섯 칸.
6. `compare_unavailable` 집합 비교. 순서는 보지 않는다.

성공은 `(True, [])`. 실패는 `(False, 이유 목록)`. 예외를 올리지
않는다. 이 시그니처를 바꾸면 기존 시험과 호출자가 깨진다.

## 12. report.py 실패를 인증서가 읽는 법

인증서는 리포트를 자식 프로세스로 부른다.
`python gym/report.py --bin <bin> --json`. stdout 만 파싱한다.
stderr 는 분류에 쓴다.

비-0 이면 `classify_report_failure`:

1. stderr+stdout 에 `missing-scorecard` `missing-bin` `malformed-json`
   `malformed-scorecard` `malformed-report` `report-tool-failed`
   `permission` `decode-error` 표식이 있으면 그 kind.
2. `scorecard.json` 과 (`없다` 또는 `남기지`) 가 같이 있으면
   `missing-scorecard`.
3. `JSON` 과 파싱/decode 가 있으면 `malformed-json`.
4. 그 밖은 `report-tool-failed`.

0 인데 stdout 이 공백이면 `malformed-report`. 0 인데 JSON 객체가
아니면 `malformed-json` 또는 `malformed-report`.

예전 `_run_report` 는 `RuntimeError("report.py 실패: …")` 와
`json.loads` 를 그대로 올렸다. 이제 CertifyError 다. `certify()` 와
`main()` 이 잡아 exit 2 와 kind 한 줄을 stderr 에 쓴다. 빈
인증서를 지어 만점인 척하지 않는다.

## 13. 성공 칸 — 이 작업이 바꾸지 않는 것

아래는 기존 시험이 지키는 칸이다. 예외 보강이 이 칸을 옮기면 회귀다.

리포트:

1. 편집 두 pack `3+1 / 3+3` 은 축 점수 4/6, percent 66.
2. 정확도 5/8 = 62, 커버리지 82. 두 숫자는 다르다.
3. 보안 unavailable pack `d` 는 축에 없고 `packsUnavailable` 에 있다.
4. 커버리지 빈 객체면 카드에 `커버리지` 라는 단어가 없다.
5. CLI 는 `--bin` 또는 `(--scorecard + --coverage)`.
6. 필수 인자 없으면 문구 `필수: --bin <경로> 또는 (--scorecard + --coverage)`,
   exit 2.

인증서:

1. 같은 트리의 지문은 같고 64자 hex 다.
2. pack 과제 JSON 한 칸을 바꾸면 지문이 바뀐다.
3. pack asset 과 `tools/build_baseline.py` 도 지문에 들어간다.
4. 재현 core 에 `rhwpCommit` 과 `agent` 가 없다.
5. 진짜 인증서는 verify True.
6. 정확도 percent 위조는 "정확도" 가 들어간 이유로 False.
7. 축소된 지문은 "벤치마크 지문" 이 들어간 이유로 False.

라이브 오라클 원칙도 그대로다. 리포트는 스코어카드를 다시 채점하지
않는다. 인증서는 리포트를 다시 합성하지 않는다. 각각 한 층을
봉인한다.

## 14. 다른 기둥과의 경계

| 도구 | 축 | 이 문서와의 관계 |
|---|---|---|
| `score.py` / `runner.py` | 종점 채점 | 스코어카드를 생산. 이 작업이 수정하지 않음 |
| `build_baseline.py` | 기준 풀이 조립 | `--bin` 모드가 부를 수 있음. 수정하지 않음 |
| `coverage.py` | 측정 폭 | 선택적 입력. 수정하지 않음 |
| `report.py` | 한 장 계기 | 이 문서의 대상 |
| `certify.py` | 재현 증명 | 이 문서의 대상 |
| `discriminate.py` | 약한 오라클 | 리포트·인증서를 부르지 않음 |
| `trajectory.py` | 경로 | 리포트·인증서를 부르지 않음 |
| `fuzz_corpus.py` | 손상 발견 | 무관 |
| `release_gate.py` | 릴리스 차단 | 이 작업이 게이트 YAML 을 건드리지 않음 |
| expert-challenges | 보스 pack | 수정하지 않음 |
| tutorial / PARK | 입문 문서 | 수정하지 않음 |

리포트가 `--bin` 으로 score/build_baseline 을 부르는 것은 예전
계약이다. 그 호출에 새 플래그를 붙이지 않는다. 하위 도구가 비-0
이면 `report-tool-failed` 다. 그 도구의 내부 kind 를 여기서 재정의
하지 않는다.

## 15. 시험 행렬

바이너리 없이 돈다. 목킹은 `_run` / `_run_report` / `subprocess.run`
만. 실제 rhwp 를 켜지 않는다.

리포트 시험 (`test_gym_report.py`):

| 반 | 고정하는 것 |
|---|---|
| 성공 칸 | 축 합산, 정확도/커버리지 분리, 미가용 제외, 선택 커버리지 |
| 카탈로그 | kind 고유, HELP 비지 않음, 문서 백틱, CLI 플래그 불변 |
| JSON 형태 | dict 만 객체, bool 거부, percent 공식, 축 라벨 모서리 |
| 적재 | 없는 스코어카드/커버리지, 깨진 JSON, 배열, 빈 파일, 디렉터리 |
| 합성 | 미가용 예외, 비객체 입력, 빈 packs, 오류 pack, bool 점수 |
| 카드 | 예외 절, 신뢰 줄, 비객체 카드 |
| CLI | exit 2 경로, `--json` 성공, `--out` 성공, 빈 `--bin` |
| --bin 자리 | 산출 부재, 깨진 scorecard.json, 하위 도구 비-0 |

인증서 시험 (`test_gym_certify.py`):

| 반 | 고정하는 것 |
|---|---|
| 성공 칸 | 지문 결정성, 민감도, core 변동 메타 제외, 위조 탐지 |
| 카탈로그 | kind 고유, 문서 백틱, CLI 플래그 불변 |
| 지문 도우미 | 빈 루트, pyc 제외, score/report/certify 포함 |
| 적재 | 없는 인증서, 깨진 JSON, 배열, 디렉터리 |
| 분류 | report 실패 → missing-scorecard / malformed-json / tool-failed |
| 미가용 | 발급 시 예외 칸, 재현 집합 대조 |
| verify 방어 | 비객체, 없는 report, 운영 예외, 신원/커버리지/축 |
| CLI | exit 2/1/0, 발급 stdout/--out, missing-scorecard |

audit.py 는 pack 정합이다. 이 작업이 pack JSON 을 건드리지 않으므로
감사는 통과해야 한다. 감사 실패는 범위 밖 파일을 만진 신호다.

## 16. 비목표

이 문서가 하지 않는 것:

- 새 CLI 플래그, 새 pack, 새 과제, 새 프로파일.
- `score.py` `runner.py` `build_baseline.py` 수정.
- expert-challenges / tutorial / PARK 수정.
- 열린 PR 5210–5274 가 만지는 파일 수정.
- 인증서 암호 서명, 키링, 리더보드 입장 연동.
- 커버리지를 `--scorecard` 단독으로 선택 가능하게 만드는 계약 변경.
- 지문에 `gym/docs` 나 `mydocs` 를 넣는 일. 문서는 점수를 바꾸지 않는다.
- 종료 코드 4·5 같은 새 숫자.
- 정확도 공식의 부동소수화. 정수 나눗셈이 카드 계약이다.

하지 않는 이유가 있다. 능력 증명의 값은 "어제와 같은 계기"에 있다.
계기 자체를 바꾸면 모든 예전 인증서가 한꺼번에 거짓이 된다. 예외
경로는 그 계기가 **죽지 않게** 만드는 일이다. 계기의 눈금을 바꾸는
일이 아니다.

## 17. 구현 좌표

| 심볼 | 파일 | 역할 |
|---|---|---|
| `EXCEPTION_KINDS` | 양쪽 | 카탈로그 |
| `EXCEPTION_KIND_HELP` | 양쪽 | 문서와 같은 표 |
| `ReportError` / `CertifyError` | 각 파일 | kind 를 가진 운영 예외 |
| `load_json_object` | 양쪽 | 파일 → 객체 |
| `compile_report` | report.py | 순수 합성 |
| `render_card` | report.py | 사람용 카드 |
| `benchmark_fingerprint` | certify.py | 측정 입력 해시 |
| `reproducible_core` | certify.py | 재현 다섯 칸 |
| `verify` | certify.py | (ok, diffs) |
| `classify_report_failure` | certify.py | 자식 stderr → kind |
| `main(argv=None)` | 양쪽 | CLI. 새 플래그 없음 |

시험이 문서에 백틱으로 적힌 kind 를 코드 카탈로그와 대조한다.
카탈로그에 kind 를 더하면 이 절과 5·6절 표와 HELP 와 시험을 같이
고친다. 코드만 고치면 문서 시험이 실패한다. 그것이 이 규약의
강제 장치다.
