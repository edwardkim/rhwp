# Task M100 #5177 Stage 2 - 기여자·검토자 생성 명령 범위 분리

## 배경

Stage 1의 계약 테스트가 모든 개발자 문서에 `--prepare` 명령을 요구했다. 그러나 기여자용
`CONTRIBUTING.md`의 정책은 원본-only 제출이며, 파생 suite 생성 명령은 PR review와 CI의 책임이다.

## 수정

- 기여자 가이드 검사는 `tests/cases/` 원본-only 제출과 CI 검증 안내만 확인한다.
- `--prepare` 명령 요구는 PR review·개발 환경 가이드에만 둔다.

## 기대 결과

기여자는 generated 산출물을 수동 생성·커밋하지 않고, 검토자와 CI만 결정론적 준비 단계를 실행한다.
