# Stage 10 - #5177 파생 suite 작성자 안내

## 발견

`CONTRIBUTING.md`의 일반 PR 체크리스트는 `rust-test-suite-manifest --check`를 포함했고,
같은 문서의 별도 문장은 이전 `--generate` 절차를 안내했다. 새 `tests/cases` 원본만 넣는
기여자는 파생 manifest가 없는 정상 상태에서 이 명령을 실행해 실패하거나, 생성 결과를 PR에
커밋해야 한다고 오해할 수 있었다.

## 정리

기여자, PR reviewer, CI의 책임을 다음과 같이 분리했다.

- 기여자: `tests/cases/**` 원본과 계약 단위 테스트·변경 범위 Rust test만 실행하고, 파생 파일을 만들거나 stage하지 않는다.
- reviewer: 분리된 review worktree에서 한 번의 `--prepare` 뒤 `--check`와 integration 검증을 실행하고 결과를 복원한다.
- CI: review와 같은 준비·엄격 검사로 generated harness와 Cargo target block을 확인한다.

## 반영

`CONTRIBUTING.md`, 개발 환경 안내, 로컬 사전 검증 안내에서 이전 `--generate` 지시를 제거하고,
`--check`가 준비된 파생 상태를 전제로 한다는 점과 review worktree 한정 실행 규칙을 명시했다.
