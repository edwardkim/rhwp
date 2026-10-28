---
kind: working
status: completed
canonical: mydocs/working/task_m100_6669_stage1.md
issue: 6669
last_verified: 2026-09-03
---

# Task M100 #6669 Stage 1 — fail-closed 로더와 증적 seal

## 1. 결과

Stage 0의 41개 필수 입력을 검증하고 `evidence-manifest.json`으로 고정하는 Stage 1 구현을
완료했다. HTML renderer는 아직 구현하지 않았다. 현재 도구는 `--seal`만 공개하고, 다음 Stage에서
검증된 manifest를 소비하는 `--out`을 연결한다.

주요 결과:

- 필수 파일 누락·symlink·크기 상한·UTF-8/BOM·중복 JSON key·비유한 숫자 거부
- 일곱 JSON의 kind/schema/mode와 역할별 집계를 원문 행에서 재계산
- 기존 audit/discrimination/trajectory validator 재사용
- run id, runner/product OID, binary SHA/path, positive agent와 세 실행 축 신원 교차검사
- 의도된 scorer 예외와 미설명 `scoreErrors`의 exact multiset 대사
- PASS/FAIL/INCOMPLETE와 trajectory 단일-step N/A 분리
- 정렬된 입력 SHA-256, 생성기 버전, 신원 fingerprint의 deterministic manifest
- 검증과 manifest 교체 사이 입력 변경 감지와 원자적 쓰기
- seal 뒤 입력 변경 시 `verify_seal` 실패

## 2. 구현

### `gym/tools/evidence_report.py`

새 도구는 다음 계층을 제공한다.

1. 고정 파일명 41개의 byte snapshot 수집
2. metadata와 JSON envelope 구조 검증
3. 역할별 의미 집계와 process exit 대사
4. 실행 신원·전수 집합 교차검사
5. 상태 모델과 score error 정산
6. manifest 생성·원자적 저장·재검산

`evidence-manifest.json`에는 절대 binary path를 넣지 않는다. basename과 path SHA-256만 남겨
동일성은 검사하되 로컬 경로는 노출하지 않는다. manifest에 현재 시각을 넣지 않으므로 같은 입력과
생성기 버전에서 byte-identical하다.

CLI 성공 stdout은 `gymEvidenceSeal`, 입력 오류 stderr는 `gymEvidenceReportError` JSON 봉투다.
입력을 신뢰할 수 없으면 종료 2이며 기존 manifest를 덮어쓰지 않는다.

### `gym/tools/discriminate.py`

positive와 trajectory에는 이미 `binPath`가 있지만 discrimination 봉투에는 없었다. 세 실행 축이
같은 binary를 사용했는지 대사할 수 있도록 선택적 provenance 필드 `binPath`를 추가했다.
false-pass 판정, control 수, 종료 코드와 schemaVersion은 바꾸지 않았다.

## 3. 구현 중 계약 정정

Stage 0의 cardinality 규칙을 **PASS 주장 봉투**에만 강제하도록 명확히 했다.

- audit·authority·positive·discrimination이 모두 PASS인데 task/reference/pack 집합이 다름:
  성공 위장 또는 실행 혼합을 배제할 수 없으므로 seal 거부.
- 한 역할이 이미 정직한 FAIL/INCOMPLETE이고 그 실패로 수가 달라짐:
  실패 원인을 없애지 않고 다음 HTML에 비녹색으로 표시.

cardinality를 모든 실패 봉투에 무조건 적용하면 `missing-reference`처럼 실제로 보고해야 할 실패
증적 자체가 폐기된다. 이 정정은 valid failure는 보이고, 거짓 PASS는 거부한다는 Stage 0 상위
불변식을 보존한다. run-id/agent와 세 binary path 불일치는 상태와 관계없이 항상 거부한다.

## 4. 계약 시험

신규 `scripts/tests/test_gym_evidence_report.py` 13건이 다음을 고정한다.

- 정상 bundle의 manifest 재생성 byte 동일성
- manifest에 raw `/tmp/...` binary path 미노출
- 필수 JSON 부재 시 기존 manifest 보존
- 깨진 JSON과 duplicate key 거부
- schema, PASS cardinality, run agent, binary path 불일치 거부
- positive 집계 위조 거부
- seal 이후 입력 변경 검출
- 설명된 score error는 의도된 음성 거부로 정산
- 미설명 score error는 구조 폐기가 아니라 INCOMPLETE로 seal
- valid false-pass는 FAIL로 seal
- `ok=true/trusted=false` trajectory는 INCOMPLETE
- discrimination producer가 실제 `binPath`를 기록

검증 결과:

| 명령 | 결과 |
| --- | --- |
| `python3 -m unittest scripts.tests.test_gym_evidence_report` | 13/13 통과 |
| 위 시험 + `scripts.tests.test_gym_discriminate` | 163/163 통과 |
| audit/build-baseline/trajectory/authority focused 묶음 | 406/406 통과 |
| `python3 -m py_compile ...` | 통과 |
| `git diff --check` | 통과 |

기존 build-baseline 시험의 `ResourceWarning`은 기존 test helper가 닫지 않은 임시 파일 경고이며
실패는 아니다. 이번 변경에서 새 경고나 실패를 추가하지 않았다.

## 5. 비범위와 남은 일

- HTML/CSS/SVG renderer와 `--out`은 Stage 2 범위다.
- path·파일명·본문 마스킹은 Stage 1 모델이 raw path를 manifest에서 제외한 상태이며, 자유 진단
  문자열의 실제 redaction/rendering은 Stage 2에서 구현한다.
- 공개 고정 fixture와 샘플 HTML은 Stage 3에서 추가한다.
- 수동 매뉴얼, discrimination 규약, `gym/AGENTS.md` 현행화는 Stage 4에서 한 번에 처리한다.
- #6628 전수 positive/discrimination/trajectory는 반복하지 않았다.

## 6. Stage 2 진입 조건

1. Stage 1 로더·seal 결과와 discrimination `binPath` provenance 추가를 승인한다.
2. PASS cardinality에만 실행 혼합 거부를 적용하는 계약 정정을 승인한다.
3. 승인 뒤 Stage 1 변경을 커밋하고, 같은 검증 모델만 소비하는 self-contained HTML renderer로
   전환한다.

## 7. 승인 기록

- 2026-09-03: 메인테이너가 Stage 1 결과, discrimination `binPath` provenance 보완과
  PASS cardinality 조건부 실행 혼합 거부 계약을 승인했다.
