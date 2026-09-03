---
kind: working
status: completed
canonical: mydocs/working/task_m100_6669_stage0.md
issue: 6669
last_verified: 2026-09-03
---

# Task M100 #6669 Stage 0 — 증적 입력·상태·위협 계약

## 1. 결론

#6669의 생성기는 **검증 가능한 증적만 seal하고, seal된 같은 바이트만 HTML로 파생**한다.
유효한 실패 결과는 붉은/미완료 상태의 HTML로 남기되, 파일 부재·깨진 JSON·schema/집계 위조·
실행 신원 혼합처럼 입력 자체를 믿을 수 없는 경우에는 HTML을 남기지 않는다.

출력 형식은 승인된 단일 self-contained HTML이다. JSON 봉투와 로그는 계속 정본이며 HTML은
새 판정을 만들지 않는다. #6628의 전수 positive/discrimination/trajectory는 재실행하지 않는다.

Stage 0 대사에서 #6628이 추가한 정답 권위 원장이 기존 수동 매뉴얼의 실행 목록과 #6669 계획의
입력 목록에서 빠진 것을 확인했다. `authority-ledger.json`은 1,035개 task의 `self-live`와 외부
정답을 구분하는 부모 이슈의 핵심 증적이므로 일곱 번째 필수 JSON으로 포함한다. 이 도구는 순수
파일 검사이며 현재 tree에서 task/reference/entry 1,035/1,035/1,035, issue 0을 확인했다.

## 2. 필수 입력 집합

### 2.1 실행 신원 metadata

| 파일 | 계약 | HTML 공개 |
| --- | --- | --- |
| `run-id.txt` | `[A-Za-z0-9][A-Za-z0-9._-]{0,127}` 한 줄 | 예 |
| `gym-runner-head.txt` | runner commit의 40~64자리 hex OID | 예 |
| `gym-runner-tree.txt` | runner tree의 40~64자리 hex OID | 예 |
| `product-source-head.txt` | binary를 만든 source commit OID; runner와 달라도 됨 | 예 |
| `rhwp-version.txt` | `rhwp --version`의 한 줄 결과 | 예, 제어문자 제거·길이 제한 |
| `rhwp-bin.sha256` | 64자리 SHA-256과 실행 당시 binary path | hash와 basename만 공개 |
| `run-started.txt` | timezone을 포함한 ISO-8601 시각 | 예 |
| `platform.txt` | OS·architecture 정보 | hostname·사용자 식별자 마스킹 후 공개 |
| `python-version.txt` | Python 버전 | 예, 길이 제한 |
| `rust-version.txt` | Rust 버전 | 예, 길이 제한 |

현행 매뉴얼은 `GYM_RUN_ID`를 증적 디렉터리 이름과 positive agent에만 사용하고,
`product source SHA`는 runner와 다를 때 별도로 기록하라고만 한다. 이동 가능한 archive와 혼합 실행
거부를 위해 `run-id.txt`와 `product-source-head.txt`를 항상 만드는 것으로 Stage 4에서 보완한다.

### 2.2 실행 역할과 JSON 봉투

| 역할 | JSON·텍스트 | kind / mode / schema | 동반 파일 |
| --- | --- | --- | --- |
| Python Gym 계약 | `unit.txt` | 비구조화 로그 | `unit.exit`, `unit.seconds` |
| 전 pack 정합 | `audit.json` | `gymAudit` / — / `1.0` | `.stderr`, `.exit`, `.seconds` |
| oracle 구조 | `oracle-structural.json` | `gymOracleProbe` / `structural` / `1.0` | `.stderr`, `.exit`, `.seconds` |
| oracle 자기시험 | `oracle-selftest.json` | `gymOracleProbe` / `selftest` / `1.0` | `.stderr`, `.exit`, `.seconds` |
| 정답 권위 원장 | `authority-ledger.json` | `gymAuthorityLedger` / — / `1.0` | `.stderr`, `.exit`, `.seconds` |
| 양성 기준풀이 | `positive.json` | `gymBaselineVerification` / — / `1.0` | `.stderr`, `.exit`, `.seconds` |
| 음성 판별력 | `discrimination.json` | `gymDiscrimination` / — / `1.0` | `.stderr`, `.exit`, `.seconds` |
| 경로 필요성 | `trajectory.json` | `gymTrajectoryNecessity` / — / `1.0` | `.stderr`, `.exit`, `.seconds` |

각 `.exit`는 음이 아닌 정수 한 줄, `.seconds`는 음이 아닌 정수 한 줄이어야 한다. stderr와
`unit.txt`는 내용이 비어 있어도 파일 자체가 있어야 하며 manifest의 hash에 포함한다. HTML에는
원문 로그를 복제하지 않는다.

JSON 한 파일의 기본 상한은 64 MiB, metadata와 로그 한 파일의 seal 상한은 16 MiB로 둔다.
현재 1,035-task 봉투보다 충분히 크지만 무제한 메모리 사용은 허용하지 않는다. 상한 초과는
`oversized-input` 구조 오류로 seal을 거부한다.

## 3. 두 단계 계보

### 3.1 `--seal`

1. 필수 입력 전체를 읽고 크기·UTF-8·JSON 객체·kind/schema/mode·역할별 집계를 검사한다.
2. 실행 신원과 역할 간 대사를 수행한다.
3. 파일 이름 사전순으로 SHA-256을 계산한다.
4. 생성기 `kind`, schema, version, 실행 신원 fingerprint와 입력 hash 목록만 가진
   `evidence-manifest.json`을 임시 파일에 쓴 뒤 원자적으로 교체한다.

manifest에는 seal 시각이나 절대경로를 넣지 않는다. 같은 입력과 생성기 버전이면 manifest도
byte-identical하다. 이 파일은 Gym 결과 정본이 아니라 입력 집합 영수증이다.

### 3.2 `--out`

1. manifest의 생성기 kind/schema/version과 입력 파일 목록을 검사한다.
2. 현재 파일의 SHA-256을 다시 계산해 seal 값과 전부 대조한다.
3. 역할별 의미 계약을 다시 계산한다.
4. 모두 일치할 때만 HTML을 임시 파일에서 원자적으로 교체한다.

seal 이후 입력 하나라도 바뀌었거나 추가 필수 파일이 누락되면 기존 `--out` 대상도 덮어쓰지 않고
종료 2다.

## 4. 실행 신원 혼합 거부 규칙

다음 조건을 모두 만족해야 한다.

1. `run-id.txt`가 유효하고 `positive.agent == "maintainer-<run-id>"`다.
2. positive, discrimination, trajectory의 `binPath`가 byte-for-byte 같다.
3. `rhwp-bin.sha256`에 기록된 실행 당시 path가 세 `binPath`와 같다.
4. runner head/tree와 product source head는 각각 유효한 OID다. product와 runner는 합법적으로
   다를 수 있으므로 서로 같다고 강제하지 않는다.
5. audit, authority ledger, positive, discrimination이 모두 PASS를 주장할 때 `taskCount`가 같다.
6. 같은 PASS 조건에서 audit의 `referenceCount`, authority ledger의 `referenceCount`·`entryCount`,
   positive와 discrimination의 task 수가 같은 전수 집합을 가리킨다.
7. 같은 PASS 조건에서 positive의 pack 집합과 audit pack id 집합이 같다. 배열 순서는 의미가 없고
   중복은 거부한다.
8. manifest에 기록된 identity fingerprint와 재계산 값이 같다.

trajectory는 단일-step을 제외한 다단계 과제만 세므로 전수 task 수와 같다고 강제하지 않는다.
서로 다른 source와 runner 조합 자체도 지원 대상이므로 각각의 SHA를 표시하되 하나로 합치지 않는다.
audit나 positive가 이미 정직한 FAIL/INCOMPLETE를 보고한 실행은 실제 누락 때문에 수가 다를 수 있다.
이때 cardinality 차이만으로 입력을 폐기하지 않고 비녹색 보고서에 그대로 보인다. PASS 봉투들끼리의
차이만 성공 위장 또는 실행 혼합으로 보아 seal을 거부한다.

## 5. 역할별 정직 판정

생성기는 역할별 `PASS`, `FAIL`, `INCOMPLETE`, `NOT_APPLICABLE`을 사용한다.

- `PASS`: 실행과 의미 계약이 모두 충족됨.
- `FAIL`: 신뢰할 수 있는 완료 봉투가 실제 benchmark/계약 실패를 보고함.
- `INCOMPLETE`: 도구·환경·예외·미설명 오류 때문에 성공/실패를 완전히 판정할 수 없음.
- `NOT_APPLICABLE`: trajectory의 단일-step처럼 그 감사 축의 대상이 아님.

전체 `PASS`는 모든 필수 역할이 PASS이고 허용된 `NOT_APPLICABLE`만 있을 때다. HTML 생성기의
종료 코드는 다음과 같이 분리한다.

| 종료 | 의미 | HTML |
| --- | --- | --- |
| 0 | 신뢰 가능한 전체 PASS 보고서 생성 | 생성 |
| 1 | 신뢰 가능한 FAIL 또는 INCOMPLETE 보고서 생성 | 비녹색으로 생성 |
| 2 | 입력 구조·identity·manifest를 신뢰할 수 없음 | 새 출력 없음 |

### 5.1 구조·oracle·권위

- unit process exit가 0이 아니면 `INCOMPLETE`다. 비구조화 로그를 추측해 FAIL로 승격하지 않는다.
- audit는 process/envelope exit 0, `ok=true`, issue 0, tool/missing-root false이고 자체
  `validate_report`가 깨끗해야 PASS다. 정합 issue는 FAIL, tool 실패는 INCOMPLETE다.
- 두 oracle은 process exit 0, `ok=true`, issue 0, 정확한 mode여야 PASS다.
- authority ledger는 process/envelope exit 0, `ok=true`, issue 0, task/reference/entry가 같은
  양수이고 summary가 entries에서 재계산돼야 PASS다.

### 5.2 positive

- process/envelope exit 0, `ok=true`, `taskCount>0`, `built==taskCount`.
- `failed`, `skipped`, `missingArtifact`, `failedScore`, `buildError`는 모두 0.
- `len(results)==taskCount`이고 모든 행이 `ok=true`, `kind=ok`이며 pack/task key가 중복되지 않음.
- 의미 실패 행이 있으면 FAIL이다. JSON/집계 모순은 seal 거부다.

### 5.3 discrimination과 `scoreErrors`

- 기존 `validate_report`를 먼저 통과해야 한다.
- false-pass가 하나라도 있으면 FAIL이다.
- load/build/tool error, tool failure 또는 알 수 없는 `skipped`가 있으면 INCOMPLETE다.
- 각 `results[]`의 `error`가 만든 정확한 문자열
  `"<pack>/<task> (<control>): <error>"`의 multiset과 `scoreErrors`를 대사한다.
- `discriminates=true`인 행과 정확히 대응하는 scorer error는 **의도된 음성 거부**로 표시하며
  그 자체로 전체 PASS를 뒤집지 않는다.
- 대응되지 않는 `scoreErrors`, 오류가 있는데 `discriminates=false`인 행, 중복 수 불일치는
  **미설명 오류**이므로 INCOMPLETE다.

비교는 redaction 전 원문으로 하고, HTML에는 마스킹·길이 제한된 진단만 표시한다.

### 5.4 trajectory

- `ok`와 `trusted`를 별도 필드로 보인다.
- process/envelope exit 0, `ok=true`, `trusted=true`, theater/exception/tool error가 비고,
  missing-bin/tool-failed false, `loadBearing==taskCount`일 때 PASS다.
- theater는 FAIL이다.
- exception, missing binary, tool failure 또는 `ok=true/trusted=false`는 INCOMPLETE다.
- `skipped[].reason=single-step`은 `NOT_APPLICABLE` 분포이며 PASS를 막지 않는다. 그 밖의 skip
  reason은 INCOMPLETE다.

## 6. HTML 정보 구조

1. **권위 경고**: JSON이 정본이며 Gym이 제품·한컴 동등성이나 릴리스 적합성을 증명하지 않음.
2. **전체 상태**: PASS/FAIL/INCOMPLETE와 가장 먼저 확인할 원인.
3. **실행 신원**: run id, runner head/tree, product source, binary version/hash, 환경, 시작 시각.
4. **역할 카드**: unit, audit, 두 oracle, authority, positive, discrimination, trajectory.
5. **pack 분포**: positive 성공/실패/skip, discrimination control/false-pass/error,
   trajectory load-bearing/theater/N/A를 같은 pack 행에서 비교.
6. **상세**: 실패·미완료만 `<details>`로 펼치며 의도된 score error와 미설명 오류를 분리.
7. **원문 계보**: 입력 파일, kind, schema, size, SHA-256, 생성기 버전.

색만으로 상태를 전달하지 않는다. 모든 배지에 텍스트와 기호를 함께 쓰고 표·요약 문장을 제공한다.

## 7. 보안·개인정보 위협 모델

| 위협 | 기본 대응 |
| --- | --- |
| JSON/HTML injection | 모든 자유 문자열 HTML escape, raw JSON/script 미삽입 |
| 절대경로·사용자 홈 노출 | binary path는 대사에만 쓰고 basename만 표시; POSIX/Windows 절대경로 마스킹 |
| 사설 코퍼스 파일명·본문 노출 | `.hwp/.hwpx/.pdf` 등 파일 토큰 마스킹; 원문 payload 미출력 |
| 과대 JSON·로그 | 역할별 byte 상한, 문자열별 400자 상한과 원문 digest/길이 표시 |
| 제어문자·터미널 escape | 허용 whitespace 외 C0/C1 제거 |
| seal 뒤 입력 교체 | manifest SHA-256 재검산 실패 시 종료 2·출력 미변경 |
| 서로 다른 실행 혼합 | run id/agent, binPath/hash, task/pack cardinality, identity fingerprint 교차검사 |
| 외부 자원 추적 | URL, script, 외부 CSS/font/image를 생성하지 않음 |

오류 문자열의 상세가 마스킹돼도 pack/task/control, 오류 분류, 원문 길이와 SHA-256은 남겨 JSON에서
정확한 원인을 다시 찾을 수 있다.

## 8. Stage 0 검증 증적

현재 `devel@d770ef80e`의 빠른 구조 결과:

| 검사 | 결과 |
| --- | --- |
| `audit.py --json` | 21 pack, task/reference 1,035/1,035, issue 0, exit 0 |
| oracle structural | `gymOracleProbe/1.0`, issue 0 |
| oracle selftest | 14/14, issue 0 |
| authority ledger | task/reference/entry 1,035/1,035/1,035, issue 0, exit 0 |

이 검사는 입력 형식 대사를 위한 빠른 구조 점검이다. #6628에서 이미 확정한 positive,
discrimination, trajectory 전수 결과를 다시 실행하지 않았다.

## 9. Stage 1 진입 조건

1. 일곱 JSON 입력과 authority ledger 추가 보정을 메인테이너가 승인한다.
2. 유효한 FAIL/INCOMPLETE는 HTML로 남기고, 신뢰 불가능한 입력은 종료 2로 출력하지 않는 경계를
   승인한다.
3. `run-id.txt`, `product-source-head.txt`, authority ledger의 수동 매뉴얼 보완을 승인한다.
4. score error의 exact row 대응 규칙과 기본 개인정보 마스킹을 승인한다.

승인 뒤 이 문서를 완료 상태로 바꾸고 계획·Stage 0 절편을 커밋한 다음 Stage 1 로더 구현으로
전환한다.

## 10. 승인 기록

- 2026-09-03: 메인테이너가 일곱 JSON 입력, authority ledger 보정, 세 단계 상태·종료 코드,
  실행 신원 교차검사, score error 대응 및 기본 개인정보 마스킹 계약을 승인했다.
