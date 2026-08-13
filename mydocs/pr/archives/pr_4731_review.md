---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4731 검토 - HWPX container 재귀 깊이 한계

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4731](https://github.com/edwardkim/rhwp/pull/4731) |
| 작성자 / 원 head | @kevin9327 / `f22568721e` |
| 검토 적용 commit | `3c033c537` |
| 메인터너 보정 | `3de1aff81` `fix(security): HWPX container 재귀 한계 보강` |
| 통합 PR | [#4738](https://github.com/edwardkim/rhwp/pull/4738) |
| base / code candidate | `devel` `b5c14346d0` / `3de1aff81` |
| 최신 기준선 merge / Full CI head | `77a61f68f` / `77a61f68f` |

## 범위와 보정

원 PR은 HWPX `<hp:container>`의 입력 제어 재귀를 깊이 제한 없이 호출하던 경로에
가드를 추가하고, 상한 초과 거부와 정상 중첩 파싱을 시험한다. 이는
[이슈 #4730](https://github.com/edwardkim/rhwp/issues/4730)의 네 재귀 경로 중 HWPX
container 하나만 다룬다.

원안의 256 상한과 32MiB 전용 스레드 테스트는 `parse_container`의 큰 재귀 프레임에서
기본 스레드 스택을 대표하지 못한다. 통합 보정은 최대 64개 그룹(깊이 0~63)을 허용하고
65번째를 거부하도록 경계를 명확히 했다. 회귀 테스트도 기본 스레드에서 상한 초과가
스택 고갈보다 먼저 오류가 되는지 검증한다. 원 PR 본문의 `closes #4730`은 남은 세 경로와
모순되므로 관련 이슈 표현으로 고쳤다.

## 완료한 검증

- 최신 `upstream/devel` `b5c14346d0` 위에 원 기능 commit을 `3c033c537`으로 cherry-pick하고,
  메인터너 보정 `3de1aff81`을 추가했다. `git diff --check upstream/devel...HEAD`를 통과했다.
- Windows PowerShell에서 `cargo test --target-dir target/pr-review --lib container_nesting` —
  기본 debug 스레드의 경계 회귀 2/2 통과.
- Windows PowerShell에서
  `cargo test --profile release-test --target-dir target/pr-review --lib container_nesting` —
  release-test 경계 회귀 2/2 통과.
- `rustfmt --edition 2021 --check src\\parser\\hwpx\\section.rs`를 통과했다.
  `cargo fmt --check`는 이 호스트의 긴 경로 OS error 206으로 실행 자체가 실패해 대상 파일
  Rustfmt 검사로 대체했다.
- code candidate `3de1aff81`의 GitHub Full CI, CodeQL, Build & Test aggregate가 모두 성공했다.
- 검토 중 `devel`이 `cdeb1ba540`으로 전진해 오늘할일 충돌을 해소하는 최신 기준선 merge
  `77a61f68f`를 만들었다. 이 head에서 기본 debug focused 회귀를 다시 통과했고, GitHub
  Full CI, CodeQL, Build & Test aggregate도 모두 성공했다.

Rust parser와 unit test만 바뀌며 fixture·renderer·WASM 출력은 변경하지 않는다. 따라서
시각·Hancom 검증은 적용 대상이 아니다. Cargo incremental 환경 변수는 지정하지 않았다.

## 판정

**self-review 수용.** 원 contributor commit과 fork branch는 보존하고, 메인터너 보정과
검토 기록을 최신 `devel` 기반 통합 PR에 분리한다. 이 최종 기록 commit을 올린 뒤 최신
trailing head의 required check·mergeability를 다시 확인하고, 작업지시자 승인 뒤에만 merge한다.
