---
kind: review
status: accepted-local
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-04
---

# PR #3897 검토 - 진단 명령 미지 플래그와 stdout 계약

- 원 head: `0c057605f1ed14e0c68fb10c09ebb22566c6c4fa`
- 범위: `bench`·`dump`·`diag`의 미지 옵션 거부 및 bench 전건 실패 stdout 처리.
- 시각 검증: 불필요. CLI 진단 표면만 변경한다.

## 결과

`diagnostics_flag_contract` 11건이 통과했다. 새 분기는 미지 플래그에 exit 2와 빈
stdout을 보장하고, bench가 성공 행이 없을 때 표와 TSV를 쓰지 않도록 한다. 전체
release-test도 통과했다. 별도 결함은 찾지 못했다.

## 후속 기록

수용 판단은 완료된 로컬 검증을 기준으로 한다. #3887의 확대 preflight가 아직 다른
미선언 명령 여섯 개를 잡으므로 그 PR만 실제 결함 보정이 필요하다.
