---
kind: pr-review
pr: 5455
issue: 5454
status: merged
merged_at: 2026-08-18T10:27:44Z
merge_commit: 30fa47bcf5ab272c915d1852f69386d8d5b5eca1
---

# PR #5455 검증 기록 - PR Rust CodeQL 분석 경로 분리

## 범위

- PR Rust CodeQL 분석만 제품 Rust 경로로 한정했다: `src/**`, `crates/**`, `rhwp-desk/src/**`, `build.rs`.
- `devel` push와 schedule의 Rust CodeQL은 별도 설정 파일을 사용하지 않아 전체 경로 분석을 유지한다.
- 분석 job 이름과 required check는 변경하지 않았다.

## 검증 근거

- 로컬: `scripts.tests.test_codeql_workflow` 15건, CodeQL YAML 파싱, `git diff --check`를 통과했다.
- 원격 CI: 모든 실행 check가 성공 또는 영향 범위에 따른 skip으로 완료됐다.
- Rust CodeQL job: 기준 15분 13초에서 14분 19초로 54초(5.9%) 단축됐다. 오류 없이 추출한 Rust 파일은 1,376개에서 624개로 줄었고, 추출은 19초, query는 37초 감소했다.

## 결론 및 후속 처리

PR은 `30fa47bcf5ab272c915d1852f69386d8d5b5eca1`로 병합됐다. `Closes #5454`에 따라 이슈가 자동 종료됐으며, 기능 head 브랜치의 로컬·원격 참조를 삭제하고 `devel`을 병합 commit까지 동기화했다.
