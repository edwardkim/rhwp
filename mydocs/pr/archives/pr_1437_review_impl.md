# PR #1437 처리 계획

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1437
- 이슈: https://github.com/edwardkim/rhwp/issues/1436
- head: `edwardkim/rhwp:task_m100_1436`
- base: `edwardkim/rhwp:devel`

## 커밋 구성

- `5aa78304` `task 1436: 크기 고정 속성 JSON 노출`
- `84b0034b` `task 1436: 크기 고정 개체 조작 차단`
- `4196dde6` `task 1436: 크기 고정 선택 핸들 잠금 표시`
- `8e1eb878` `task 1436: 최종 보고서 추가`

## 처리 단계

1. PR 생성 및 base/head 확인
2. 로컬 필수 검증 완료 결과 확인
3. 리뷰 문서와 오늘할일을 archive 규칙에 맞춰 같은 PR head에 추가
4. 문서 커밋 push 후 GitHub Actions 재실행 완료 확인
5. CI 통과 시 PR merge
6. #1436 close 여부 확인
7. 필요 시 PR 감사 코멘트와 수동 issue close 처리
8. `local/devel`을 `upstream/devel`에 동기화
9. 임시 원격 브랜치 `task_m100_1436` 삭제

## 검증 기록

- `cargo build --release`: 통과
- `cargo test --release --lib`: 통과
- `cargo test --profile release-test --tests`: 통과
- `cargo fmt --check`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과
- `npm --prefix rhwp-studio run build`: 통과
- `git diff --check`: 통과

## 대기 항목

- GitHub Actions 재실행 결과 확인
- merge 후 #1436 자동 close 확인
