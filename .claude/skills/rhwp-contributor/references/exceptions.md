# 환경 차이와 예외 처리

예외의 권위는 [CONTRIBUTING.md](../../../../CONTRIBUTING.md)와
해당 역할의 [local_validation.md](../../../../mydocs/manual/pr_review/local_validation.md)다.
과거 사례의 임시 조치를 일반 면제 규칙으로 확대하지 않는다.

- sparse checkout이 필요한 workspace member를 누락하면 checkout 환경을 먼저 해결한다.
  검사 대상 누락을 성공이나 fmt 면제로 기록하지 않는다.
- Windows에서는 저장소의 newline 설정과 UTF-8 본문 전달 절차를 따른다.
  사용자 변경을 대량 재포맷하거나 되돌리지 않는다.
- 동일 작업의 열린 PR과 기존 worktree를 확인한다. 사용자 승인 없이 가로채거나 중복 PR을 만들지 않는다.
  특정 과거 이슈 번호나 worktree 이름을 모든 작업에 대한 영구 금지 규칙으로 삼지 않는다.
- CI 미발행, 취소, 실행 실패와 성공을 구분한다. required check가 없다는 이유로 통과를 추정하지 않는다.
  보호 브랜치 예외는 현재 GitHub 운영/메인터너 절차와 승인으로 처리한다.
- 도구, 폰트, private fixture 또는 변환 환경이 없으면 접근 불가와 검증 미완료를 기록한다.
  다른 입력이나 임의 기준으로 바꾼 결과를 원래 사례의 성공으로 보고하지 않는다.
- Docker 대신 host 도구를 쓰는 경우 정본이 허용하는 fallback 조건과 실행 환경을 명시한다.
  host 결과를 Docker/최적화 WASM 검증 통과로 바꾸어 쓰지 않는다.
- 사용자가 추가 검증을 제한했으면 제한을 존중하고 미실행 항목을 남긴다.
  그 제한을 이후 모든 기여의 필수 게이트 면제로 일반화하지 않는다.
