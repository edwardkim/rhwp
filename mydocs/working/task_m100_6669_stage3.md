---
kind: working
status: completed
canonical: mydocs/working/task_m100_6669_stage3.md
issue: 6669
last_verified: 2026-09-03
---

# Task M100 #6669 Stage 3 — 공개 fixture와 재현 가능한 샘플

## 1. 결과

실제 전수 실행이나 사설 코퍼스에 의존하지 않는 공개 합성 fixture를 고정했다. 41개 필수 입력과
deterministic `evidence-manifest.json`을
`scripts/tests/fixtures/gym-evidence-report/complete/`에 두고, 그 seal에서 생성된 단일 HTML을
`gym/examples/evidence-report.html`에 저장했다.

fixture는 실제 Git OID, 실제 제품 binary 또는 과거 #6628 증적의 복사본이 아니다. 두 개의 합성
task만 사용한다.

| 축 | 합성 내용 | 판정 |
| --- | --- | --- |
| positive | 2/2 성공 | PASS |
| discrimination | 2/2 음성 대조 거부 | PASS |
| score error | scorer 예외와 result 행이 exact 대응하는 의도된 거부 1건 | PASS 유지 |
| trajectory | 다단계 load-bearing 1건 | PASS |
| trajectory N/A | 단일-step 1건 | 성공 수치와 분리 |
| authority | self-live 1건, contract-constant 1건 | 경계 표시 |

FAIL, 미설명 score error, seal 변조, `trajectory.ok=true/trusted=false`는 고정 fixture를 임시
디렉터리에 복사한 뒤 시험별로 변형한다. 고정된 PASS 원본과 샘플 HTML은 오염시키지 않는다.

## 2. 재현 계약

`scripts/tests/test_gym_evidence_report.py`는 더 이상 자체 메모리 객체만으로 정상 입력을 만들지 않고
공개 fixture를 복사해 사용한다. 다음을 byte-for-byte 검사한다.

1. fixture의 파일 집합이 41개 필수 입력과 manifest 하나로 정확히 구성되는지
2. `--seal` 재실행 결과가 추적된 `evidence-manifest.json`과 같은지
3. `--out` 결과가 추적된 `gym/examples/evidence-report.html`과 같은지
4. 공개 fixture의 의도된 score error가 reported 1, intended 1, unexplained 0인지

직접 재현 명령은 fixture 상위 `README.md`에 기록했다. manifest와 샘플은 사람이 직접 수정하는
source가 아니라 회귀 대조용 생성 산출물이다.

## 3. 고정 hash

| 산출물 | bytes | SHA-256 |
| --- | ---: | --- |
| `evidence-manifest.json` | 8,051 | `b6171710dfdff64b5a2db45a3ddceb6d96a9f2edfadfd994a257b836a1d1075a` |
| `gym/examples/evidence-report.html` | 19,058 | `e438908b9661fdbe8eb9ef694205896b5efa347f1b859c6b2169510a14c1f7f9` |

실행 identity fingerprint는
`b6818ef1147689a04869d315468a1874582176e3f4f6011a3c3d15e49c1339cb`다.

## 4. 검증

| 검증 | 결과 |
| --- | --- |
| fixture `--seal` | PASS, 종료 0 |
| fixture `--out` | PASS HTML, 종료 0 |
| `python3 -m unittest scripts.tests.test_gym_evidence_report` | 20/20 통과 |
| evidence report + 인접 Gym focused 묶음 | 576/576 통과 |
| 샘플 내 외부 URL·script·raw binary path·hostname 검색 | 0건 |
| `git diff --check` | 통과 |

기존 build-baseline 시험의 `ResourceWarning`과 의도된 argparse 음성 시험 출력은 기존 계약이며
실패는 아니다.

## 5. 다음 단계

- Stage 4에서 `gym/docs/evidence_report.md`, 인간 메인테이너용 수동 운영 절차,
  `gym/AGENTS.md`의 AI 에이전트 지침과 관련 진입점 링크를 같은 계약으로 현행화한다.
- Stage 5에서 저장된 샘플을 브라우저로 열어 데스크톱·모바일 폭, 표 overflow, 상태색·상태문자,
  `<details>`와 offline request 0을 시각 판정한다.
- #6628 전수 positive/discrimination/trajectory는 이번 단계에서도 재실행하지 않았다.

## 6. Stage 4 진입 조건

1. 공개 fixture의 합성 범위와 실제 전수 증적 비복제 원칙을 승인한다.
2. manifest와 HTML을 byte-for-byte 회귀 대조용 생성 산출물로 추적하는 방식을 승인한다.
3. 승인 뒤 Stage 3 변경을 커밋하고 인간·AI 문서 계약 현행화로 진행한다.

## 7. 승인 기록

- 2026-09-03: 메인테이너가 공개 합성 fixture, manifest·HTML byte-for-byte 회귀 대조와
  Stage 4 문서·에이전트 지침 현행화 진입을 승인했다.
