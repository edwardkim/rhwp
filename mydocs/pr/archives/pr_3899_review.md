---
kind: review
status: accepted-local
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-04
---

# PR #3899 검토 - stdin BrokenPipe 테스트 헬퍼

- 원 head: `2162212987bc8b34f3c660c9a37705724d19b369`
- 범위: CLI 계약 테스트 세 곳의 조기 종료 BrokenPipe 처리.
- 시각 검증: 불필요. 테스트 전용 변경이다.

## 결과

세 헬퍼는 `BrokenPipe`만 정상적인 조기 종료로 허용하고 다른 stdin 오류는 계속
실패시킨다. 누적 focused contract와 전체 release-test가 통과했으며, 실패 조건을
완화하는 범위가 필요한 오류 종류로 한정돼 있다. 별도 결함은 찾지 못했다.

## 후속 기록

수용 판단은 완료된 로컬 검증을 기준으로 한다. `BEHIND` 표시는 원격 병합 조정
정보로만 기록한다.
