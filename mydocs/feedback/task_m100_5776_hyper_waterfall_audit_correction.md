---
kind: feedback
status: completed
canonical: mydocs/manual/github_operations.md
last_verified: 2026-08-24
---

# Task M100 #5776 Hyper-Waterfall 감사 판단 정정

## 정정 대상

#5776 로컬 candidate가 Hyper-Waterfall 문서 규칙을 준수했는지 검토하는 과정에서, 처음에는 일반
Issue Workflow와 #4080 승인 게이트 사고를 우선 적용해 수행·구현·Stage·최종 문서가 모두 필요하다고
판단했다. 그 판단에 따라 7종 문서를 소급 작성하는 복구안을 제안했다.

## 놓친 더 구체적인 규칙

루트 `AGENTS.md`는 GitHub Actions·저장소 설정·path filter 운영에
[`github_operations.md`](../manual/github_operations.md)를 우선 적용하도록 지정한다. 이 문서 §3은
trigger·path filter·비용 라우팅을 O2로 분류하고, §3.1의 조건을 만족하는 작은 O1/O2 변경은 단일 운영
변경 기록으로 처리하며 별도 수행·구현·Stage·최종 문서 묶음을 형식적으로 만들지 말라고 명시한다.

#5776 candidate는 다음 조건을 모두 만족한다.

- 제품 소스·제품 테스트·package lock·공개 API를 변경하지 않는다.
- secret, branch protection, release·publish, runner를 변경하지 않는다.
- workflow path 2개와 classifier·policy 매핑을 고치는 작고 결정적인 diff다.
- 기존 #5776과 작업지시자의 명시적 로컬 진행 지시가 근거다.
- baseline, 검증, 기대 run/no-run, rollback을 한 기록에 담을 수 있다.

따라서 일반 절차를 적용해 “비준수”라고 단정한 최초 감사 판단과 7종 소급 문서 제안은 과잉이었다.
원인은 변경 대상을 먼저 등급화하지 않고 일반 지침과 과거 사고 기록을 더 구체적인 운영 정본보다 먼저
적용한 데 있다.

## 정정된 판정과 실제 보완점

- 로컬 구현 착수는 작업지시자가 승인했고, O2 단축 경로에는 별도 Stage 승인 묶음이 필요하지 않다.
- remote push·PR 생성·merge는 실행하지 않아 별도 원격 승인 게이트를 침범하지 않았다.
- 기존 #5776 문서는 원인·consumer 매핑·검증 결과를 담았지만, O2 등급, live baseline, expected
  run/no-run, 적용 승인·상태, 관찰 완료 조건, rollback이 부족했다.
- 일일 작업 문서의 #5776 상태를 Actions 관찰 전인데도 `완료`로 적은 것은 잘못이었다.

## 적용한 교정

1. 기존 #5776 문서를 단일 O2 운영 변경 기록으로 보강했다.
2. 수행·구현·Stage·최종 문서를 추가 생성하지 않았다.
3. 일일 작업 상태를 `진행중`으로 되돌리고 remote 적용·관찰을 완료 조건으로 명시했다.
4. remote push·PR 생성·merge 승인은 계속 별도 게이트로 남겼다.

앞으로 GitHub 운영 작업은 먼저 `AGENTS.md`의 라우팅에 따라 O0~O4 등급을 확정하고, 해당 정본의
단축·확장 경로를 선택한 뒤 일반 Hyper-Waterfall 지침을 보조 규칙으로 적용한다.
