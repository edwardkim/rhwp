# Task 1436 최종 보고서

## 이슈

- GitHub Issue: https://github.com/edwardkim/rhwp/issues/1436
- 제목: 개체 속성: 크기 고정 시 크기/회전/기울기 조작 비활성화 필요
- 브랜치: `local/task_m100_1436`
- 작업 모드: 기여자 모드. 오늘할일 문서는 생성하지 않았다.

## 구현 요약

`CommonObjAttr.size_protect`를 Studio 개체 속성 JSON의 `sizeProtect`로 노출하고, 그림 속성 UI와 직접 조작 경로에 연결했다.

- HWP5/HWPX에 이미 저장되는 크기 보호 값을 `sizeProtect`로 조회/저장한다.
- 개체 속성 대화상자의 `크기 고정` 체크를 실제 속성과 연동한다.
- 크기 고정 상태에서는 너비, 높이, 비율 유지, 회전, 대칭, 기울기 입력을 비활성화한다.
- 키보드 리사이즈, 마우스 리사이즈, 회전 핸들, 회전/대칭 명령을 크기 고정 상태에서 차단한다.
- 선택 렌더링에서 크기 고정 개체는 한컴처럼 조작 가능 핸들 대신 금지 표시 핸들로 표시한다.

## 테스트와 검증

- `cargo test --test issue_1436_size_protect_properties -- --nocapture`: 통과
- `npm --prefix rhwp-studio run build`: 통과
- `cargo build --release`: 통과
- `cargo test --release --lib`: 통과
- `cargo test --profile release-test --tests`: 통과
- `cargo fmt --check`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과
- `git diff --check`: 통과

## 남은 범위

- `비율 유지`는 파일 포맷 속성이 아니라 #1428의 사용자 설정으로 분리했다.
- 드롭다운/메뉴처럼 일시 UI로 바깥 클릭 닫힘이 자연스러운 컴포넌트는 모달 정책 범위에서 제외했다.
