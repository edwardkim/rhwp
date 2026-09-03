---
kind: working
status: completed
canonical: mydocs/working/task_m100_6669_stage4.md
issue: 6669
last_verified: 2026-09-03
---

# Task M100 #6669 Stage 4 — 인간 운영 매뉴얼과 AI 지침 현행화

## 1. 결과

Gym 증적 시각화를 사람이 반복 실행하는 절차와 AI 에이전트가 지켜야 할 경계에 연결했다. 세 문서의
책임을 분리했다.

| 문서 | 책임 |
| --- | --- |
| `mydocs/manual/gym_benchmark_operations.md` | 인간 메인테이너의 격리 실행·입력 수집·seal·HTML·정리 순서 |
| `gym/docs/evidence_report.md` | 41개 입력, 검증·상태·종료 코드·manifest·redaction의 도구 규약 |
| `gym/AGENTS.md` | AI 참가자/감사자 역할과 JSON 권위·공개 금지·fail-closed 행동 지침 |

`gym/README.md`, `mydocs/manual/README.md`, `llms.txt`에는 상세 내용을 복제하지 않고 위 정본으로
가는 최소 링크만 추가했다.

## 2. 운영 절차 교차검사에서 발견한 누락

기존 수동 매뉴얼은 #6669 구현 전의 절차라 새 loader가 요구하는 다음 입력을 생성하지 않았다.

- `run-id.txt`
- `product-source-head.txt`
- `authority-ledger.json`과 exit/seconds/stderr sidecar

링크만 추가했다면 문서를 그대로 실행해도 `--seal`이 항상 missing-input으로 종료 2가 됐을 것이다.
따라서 다음과 같이 보완했다.

1. runner SHA와 product source SHA를 별도 변수로 둔다.
2. 동일 source는 같은 OID를 기록하고, 다른 후보 checkout은 그 checkout의 실제 HEAD를 기록한다.
3. commit 신원을 모르는 binary에는 임의 OID를 채우지 않는다.
4. authority ledger를 구조·oracle 사전 점검과 함께 JSON+sidecar로 수집한다.
5. 41개 입력 완성 뒤 `--seal`, 성공 뒤 `--out` 순서를 명시한다.
6. seal/report stdout·stderr·exit도 별도 보존하되 seal 입력 41개에는 포함하지 않는다.

## 3. 문서에 고정한 판정 경계

- `--seal`은 FAIL/INCOMPLETE를 정직하게 고정해도 성공이므로 종료 0이다.
- `--out` 종료 0은 PASS HTML, 종료 1은 유효한 FAIL/INCOMPLETE HTML, 종료 2는 새 출력 없음이다.
- 역할·전체 판정은 PASS/FAIL/INCOMPLETE 세 상태이고, trajectory 단일-step N/A는 별도 수치다.
- JSON이 기계 판독 정본이며 manifest와 HTML은 각각 입력 영수증과 비권위 파생 뷰다.
- HTML이 redaction됐다는 사실은 사설 원문 JSON·stderr의 공개 허가가 아니다.
- 공개 fixture·샘플은 합성 회귀 자산이며 실제 전수 결과로 인용하지 않는다.
- 일반 PR, branch, tag, 제품 release 또는 게시 게이트에 보고서를 연결하지 않는다.

## 4. 검증

| 검증 | 결과 |
| --- | --- |
| 변경한 5개 Markdown의 내부 상대 링크 | 이상 없음 |
| `python3 -m unittest scripts.tests.test_gym_evidence_report` | 20/20 통과 |
| `git diff --check` | 통과 |
| 전체 `check_document_metadata.py` | 기존 4개 파일의 16건만 보고 |

metadata 검사에서 보고된 네 파일은 이번 브랜치에서 변경하지 않았고 `upstream/devel`과 동일하다.
이번에 수정한 `mydocs` 장기 문서는 모두 기존 유효 metadata를 유지하며, 신규
`gym/docs/evidence_report.md`도 같은 front matter 규약으로 작성했다.

## 5. Stage 5 진입 조건

1. 인간 절차·도구 규약·AI 행동 지침의 책임 분리를 승인한다.
2. 운영 교차검사에서 누락된 세 입력 생성 명령의 추가를 승인한다.
3. 승인 뒤 Stage 4 문서를 커밋하고 전체 Gym 빠른 계약·구조 도구와 브라우저 시각 검증을 수행한다.

## 6. 승인 기록

- 2026-09-03: 메인테이너가 인간 운영 절차·도구 규약·AI 지침의 책임 분리와 누락된 증적 입력
  생성 명령 보완을 승인하고 Stage 5 진입을 승인했다.
