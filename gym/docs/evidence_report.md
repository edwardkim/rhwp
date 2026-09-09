---
kind: guide
status: active
canonical: gym/docs/evidence_report.md
last_verified: 2026-09-03
---

# Gym 증적 seal·HTML 규약

`gym/tools/evidence_report.py`는 한 번의 Gym 전수 실행에서 나온 JSON·metadata·process sidecar를
검증하고, 입력 집합을 deterministic manifest로 seal한 뒤 사람용 단일 HTML로 변환한다.

이 도구는 벤치마크를 실행하거나 다시 채점하지 않는다. JSON 봉투가 유일한 기계 판독 정본이며,
manifest는 입력 집합의 영수증, HTML은 검증된 seal의 비권위 파생 뷰다.

```bash
python3 gym/tools/evidence_report.py --evidence-dir <evidence-dir> --seal
python3 gym/tools/evidence_report.py --evidence-dir <evidence-dir> --out <report.html>
```

인간 메인테이너가 증적을 만드는 전체 순서는
[`Gym 벤치마크 수동 운영 매뉴얼`](../../mydocs/manual/gym_benchmark_operations.md)을 따른다.

## 1. 두 단계인 이유

`--seal`과 `--out`을 분리한다.

1. `--seal`은 41개 입력을 읽고 구조·집계·종료 코드·실행 신원을 교차검사한다.
2. 검증에 성공하면 각 입력의 SHA-256과 실행 identity를 `evidence-manifest.json`에 원자적으로 쓴다.
3. `--out`은 현재 입력에서 manifest를 전부 재계산한다.
4. seal 이후 한 바이트라도 바뀌었으면 기존 HTML을 덮어쓰지 않고 종료 2다.
5. seal이 맞을 때만 검증된 메모리 모델을 별도 renderer에 넘긴다.

따라서 다른 실행의 JSON을 끼워 넣거나 결과를 고친 뒤 예전 manifest로 HTML을 만드는 경로가 없다.
HTML만 보관하고 원문 JSON과 manifest를 버리면 제3자가 판정을 재검산할 수 없다.

## 2. 필수 입력 41개

### 실행 신원 10개

| 파일 | 의미 |
| --- | --- |
| `run-id.txt` | 영문·숫자·점·밑줄·하이픈으로 된 실행 ID |
| `gym-runner-head.txt` | Gym task/reference/tool을 제공한 commit OID |
| `gym-runner-tree.txt` | runner commit의 tree OID |
| `product-source-head.txt` | 실행 binary를 만든 source commit OID |
| `rhwp-version.txt` | 실행 binary의 `--version` 한 줄 |
| `rhwp-bin.sha256` | SHA-256과 실행 당시 binary 절대경로 |
| `run-started.txt` | timezone이 있는 ISO-8601 시작 시각 |
| `platform.txt` | OS·architecture 원문 |
| `python-version.txt` | Python 버전 한 줄 |
| `rust-version.txt` | Rust 버전 한 줄 |

runner와 product source OID는 서로 달라도 된다. 다르다는 사실을 보존하는 것이 목적이다. 모르는
source를 임의 OID로 채우면 실행 신원 검사가 통과해도 증적 주장은 거짓이다.

### 구조·판정 JSON 7개

| 역할 | 파일 | kind / mode |
| --- | --- | --- |
| 구조 감사 | `audit.json` | `gymAudit` |
| oracle 구조 | `oracle-structural.json` | `gymOracleProbe` / `structural` |
| oracle 자기시험 | `oracle-selftest.json` | `gymOracleProbe` / `selftest` |
| 정답 권위 원장 | `authority-ledger.json` | `gymAuthorityLedger` |
| 양성 기준풀이 | `positive.json` | `gymBaselineVerification` |
| 음성 판별력 | `discrimination.json` | `gymDiscrimination` |
| 경로 필요성 | `trajectory.json` | `gymTrajectoryNecessity` |

모두 `schemaVersion: "1.0"`이어야 한다. positive, discrimination, trajectory에는 동일한 실행
binary의 `binPath`가 있어야 하고, `rhwp-bin.sha256`의 path와도 일치해야 한다. positive의
`agent`는 `maintainer-<run-id>`다.

### process sidecar 24개

- `unit.txt`, `unit.exit`, `unit.seconds`
- 일곱 JSON 각각의 `<base>.stderr`, `<base>.exit`, `<base>.seconds`

JSON stdout과 진단 stderr를 섞지 않는다. `.exit`와 `.seconds`는 음이 아닌 정수 한 줄이다.
`evidence-manifest.json`, seal/report summary와 HTML은 41개 입력을 모두 만든 뒤 생기는 산출물이라
필수 입력 수에 포함하지 않는다.

## 3. 입력 거부 계약

다음은 HTML의 FAIL이 아니라 **입력을 신뢰할 수 없는 상태**라서 seal 또는 출력을 거부한다.

- 필수 파일 누락, symlink, 크기 상한 초과, UTF-8 BOM·decode 오류
- JSON 문법 오류, duplicate key, NaN·Infinity, 잘못된 kind/schema/mode
- 원문 행에서 재계산한 count·summary·`ok`·exit와 봉투 주장의 불일치
- process `.exit`와 JSON exit/판정 불일치
- run ID와 positive agent 불일치
- 세 실행 축의 `binPath` 또는 binary hash 기록 path 불일치
- PASS를 주장하는 audit/authority/positive/discrimination의 task·reference·pack 집합 불일치
- manifest input SHA-256, identity fingerprint 또는 generator version 불일치
- 검증과 원자적 교체 사이 입력·manifest 변경

FAIL/INCOMPLETE 봉투는 실제 실패 때문에 task 수가 다를 수 있다. 정직하게 비녹색 상태를 보고한
실행은 cardinality 차이만으로 폐기하지 않는다. 네 축이 모두 PASS를 주장하면서 집합이 다를 때만
성공 위장이나 실행 혼합으로 보아 거부한다.

입력 오류는 stderr의 `gymEvidenceReportError` JSON에 정렬된 code·file·message로 나온다. 새
manifest/HTML은 남기지 않으며 기존 파일도 덮어쓰지 않는다.

## 4. 상태 모델

HTML의 전체 상태는 unit과 일곱 JSON 역할의 판정을 접은 값이다. 역할과 전체 판정은 세 상태만
사용하고, N/A는 trajectory 개별 과제의 별도 수치다.

| 상태 | 의미 |
| --- | --- |
| `PASS` | 모든 적용 역할이 신뢰 가능하고 판정 조건을 만족 |
| `FAIL` | 입력 구조와 신원은 신뢰 가능하지만 실제 false-pass, positive 실패, theater 등이 있음 |
| `INCOMPLETE` | tool 실패·예외·알 수 없는 skip·미설명 score error·`trusted=false` 등으로 판정 불완전 |

trajectory의 단일-step처럼 그 축의 검사 대상이 아닌 항목은 `N/A`로 세며 PASS 분자나 FAIL에
넣지 않는다.

우선순위는 `INCOMPLETE` > `FAIL` > `PASS`다. 미완료가 실패보다 좋다는 뜻이 아니라, 실패 여부를
온전히 판정하지 못했다는 뜻이다.

### discrimination score error

`scoreErrors` 문자열을 같은 result 행의 `pack/task (control): error`와 exact multiset으로
대사한다.

- 오류가 있는 행도 `discriminates=true`이고 `scoreErrors`에 정확히 대응하면 의도된 음성 거부다.
- 대응하지 않는 `scoreErrors`, 기록이 빠진 result 오류, 오류가 있으면서 false-pass인 행은
  `unexplained`이고 discrimination은 INCOMPLETE다.

### trajectory

`trajectory.ok`와 `trajectory.trusted`를 합치지 않는다. `ok=true`여도 예외·tool 오류·binary
부재 또는 `trusted=false`면 INCOMPLETE다. `reason: single-step`인 skip만 N/A로 표시한다.

## 5. CLI 출력과 종료 코드

### `--seal`

성공하면 stdout에 `gymEvidenceSeal` JSON을 쓰고 종료 0이다. `resultStatus`가 FAIL 또는
INCOMPLETE여도 입력을 정직하게 seal했다면 seal 자체는 성공이다. 입력이 무효하면 종료 2다.

### `--out`

성공적으로 생성한 stdout은 `gymEvidenceReport` JSON이며 report basename, report SHA-256,
identity fingerprint와 result status를 포함한다.

| 종료 | HTML | 의미 |
| ---: | --- | --- |
| 0 | 생성 | 유효한 seal, 전체 PASS |
| 1 | 생성 | 유효한 seal, 전체 FAIL 또는 INCOMPLETE |
| 2 | 새 출력 없음 | 무효·혼합·변조된 입력 또는 출력 오류 |

종료 1의 HTML은 실패 분석에 필요한 정상 산출물이다. 종료 2에서 기존 HTML이 남아 있다면 그것은
이번 실행에서 새로 검증된 결과가 아니므로 report summary와 hash를 함께 대사한다.

`--seal`과 `--out`은 동시에 쓸 수 없다. HTML 출력은 `.html` 확장자여야 하며 필수 입력,
`evidence-manifest.json`, symlink 또는 디렉터리를 출력 대상으로 사용할 수 없다.

## 6. manifest와 HTML

manifest에는 다음만 둔다.

- generator name/version과 manifest schema
- 실행 identity와 그 fingerprint
- 전체·역할별 상태
- reported/intended/unexplained score error 수
- 41개 입력의 상대 이름·bytes·SHA-256 및 JSON kind/schema/mode

binary 절대경로는 manifest에 넣지 않고 basename과 path SHA-256만 둔다.

HTML은 외부 URL·CDN·웹폰트·JavaScript 없이 한 파일로 생성한다. inline CSS 막대의 모든 수치는
표와 `성공/전체` 텍스트로도 표시하며, 상태는 색뿐 아니라 기호와 상태 문자열로 구분한다. CSP는
기본 resource를 모두 막고 inline CSS만 허용한다.

표시 범위:

- 전체 상태와 8개 역할 카드
- pack별 positive·discrimination·trajectory 분포
- 실패·미완료·N/A 상세
- authority class와 비독립 정답 경고
- runner/product/binary 신원과 안전화한 환경
- 전 입력 hash 계보

## 7. 개인정보·민감정보 경계

HTML에 raw JSON, stderr, binary 절대경로를 넣지 않는다. 자유 진단 문자열은 다음 순서로 처리한다.

1. POSIX·Windows 절대경로 마스킹
2. HWP/HWPX/PDF/Office 문서명 token 마스킹
3. 400자 상한
4. HTML escape
5. 마스킹·절단 시 원문 문자 수와 SHA-256만 표시

`uname -a`의 hostname도 `[host]`로 바꾼다. hash는 같은 비공개 진단인지 대사하기 위한 값일 뿐
원문 공개 허가가 아니다. 사설 코퍼스의 JSON·stderr·경로·파일명·본문을 PR이나 공개 artifact에
추가하지 않는다.

## 8. 공개 회귀 fixture

공개 합성 fixture와 생성 샘플은 다음에 있다.

- `scripts/tests/fixtures/gym-evidence-report/complete/`
- `gym/examples/evidence-report.html`

fixture는 실제 Gym 결과가 아니며 두 합성 task만 포함한다. 저장된 manifest와 HTML은 다음 시험이
byte-for-byte 재생성한다.

```bash
python3 -m unittest scripts.tests.test_gym_evidence_report
```

fixture 구성과 수동 재생성 명령은
[`fixture README`](../../scripts/tests/fixtures/gym-evidence-report/README.md)를 따른다. 생성 산출물을
직접 고치지 않는다.

## 9. 운영 경계

이 보고서는 사람이 요청한 수동 Gym 감사의 증적 표현이다. 일반 PR, `devel`/`main`, tag,
Release Binary, npm·extension 게시 게이트에서 실행하거나 소비하지 않는다. 제품 결함 판정과 한컴
조판 동등성에는 독립 제품 정답지가 필요하다.
