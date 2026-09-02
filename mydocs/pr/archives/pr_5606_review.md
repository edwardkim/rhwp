---
kind: pr-review
status: approved
pr: 5606
issue: 5605
---

# PR #5606 검토 기록 - 환경 인식형 Rust test 동시성 지침

- PR: [#5606](https://github.com/edwardkim/rhwp/pull/5606) `docs: 환경별 Rust test 동시성 지침을 정정한다`
- 관련 이슈: [#5605](https://github.com/edwardkim/rhwp/issues/5605)
- 작성자·self-review: `jangster77` collaborator self PR
- base: `devel`
- 검토한 code candidate: `d0c7dae7ae04f729aac86b878c7af0904223ee94`
- 작성 시점 참고값: Open, non-draft, `MERGEABLE`, `CLEAN`
- 라우팅: `collaborator_self_merge` + `intake_and_review` + `review_only_fast_pass`
- 로드 문서: `pr_review_workflow.md`, `pr_review/README.md`, `collaborator_self_merge.md`, `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`

## 검토 범위

- 개발·검토·배포 안내에서 고정 `--test-threads` 값을 제거하고, 모든 Nextest 예시에 `--test-threads <현재_환경에_맞는_값>`을 명시한다.
- 사용자가 현재 호스트의 논리 CPU, 메모리, 동시 작업을 기준으로 실제 정수를 선택해야 하며, macOS 측정값을 범용 권고값으로 복사하지 않음을 명확히 한다.
- Native Skia 전체 lib 검증을 `--features native-skia --lib`로 정정하고, 뒤의 `skia`는 Cargo feature가 아니라 테스트 필터라서 전체 회귀가 아니라는 점을 문서화한다.
- 구현·테스트·workflow·의존성 잠금 파일을 변경하지 않는 문서 전용 PR이다. renderer 또는 fixture 변경이 없으므로 시각 검증 경로는 적용하지 않는다.

## 검증 근거

- 로컬: `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check` 통과.
- 측정 근거: 이 macOS 호스트에서 target을 매 회 제거한 Native Skia lib 실행은 thread 1/4/8에서 각각 190.64초/159.81초/144.37초였다. 이는 환경 고정 권고가 아닌 문서 정정 근거다.
- 명령 의미 확인: 기존 `--features native-skia skia --lib` 실행은 236.49초에 58개 통과·4,087개 필터링으로 전체 회귀가 아니었다. 캐시·작업량이 달라 성능 비교에는 쓰지 않는다.
- GitHub Actions: [CI](https://github.com/edwardkim/rhwp/actions/runs/32240118446), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/32240118281), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/32240118324), [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/32240118285)가 candidate `d0c7dae7a`에서 성공했다.
- CI의 Lint, Native Skia, build archive, Rust shard, frontend gate는 PR 전체가 허용된 `mydocs/` 변경인 B 경로 fast-pass에 따라 `SKIPPED`로 집계됐고, Build & Test aggregate는 성공했다.

## 결론

차단 결함은 발견하지 못했다. 이 review·오늘할일 trailing commit은 허용된 `mydocs/` 범위만 추가한다. 최종 merge 조건은 이 새 최신 head의 CI aggregate 성공과 작업지시자 승인이다.
