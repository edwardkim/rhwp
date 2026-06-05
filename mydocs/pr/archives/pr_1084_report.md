# PR #1084 처리 보고서

- PR: <https://github.com/edwardkim/rhwp/pull/1084>
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/1079>
- 작성일: 2026-05-26
- 처리 방식: `-x` cherry-pick 수용

## 1. 처리 요약

PR #1084의 단일 커밋을 현재 `local/devel`에 체리픽했다.

```text
원본 커밋: b2fca6c6c5b6c23fb2979a8db1ba2dd00f41dba9
반영 커밋: da485767 Task #1079: 그림 pushdown ↔ 파일 vpos 이중 계상 정정 — 1페이지 정합
```

변경 내용:

```text
1. 그림 문단 앞 gap이 그림 높이 이상이면 파일 vpos가 이미 그림 공간을 반영한 것으로 판정
2. 해당 케이스에서 typeset pushdown 높이 추가를 생략
3. renderer에서도 그림을 gap 안에 배치하고 후속 flow를 추가 진행하지 않도록 정합
4. samples/pr-149.hwp 기반 회귀 가드 2개 추가
```

## 2. 검증

자동 검증:

| command | result |
|---|---|
| `cargo fmt --check` | pass |
| `cargo check` | pass |
| `cargo test --test issue_1079_picture_pushdown_vpos` | pass, 2 passed |
| `cargo test --lib` | pass, 1395 passed / 0 failed / 6 ignored |
| `docker compose --env-file .env.docker run --rm wasm` | pass |

메인테이너 시각 판정:

```text
통과
```

## 3. 판단

수용 판단:

```text
PR #1084는 samples/pr-149.hwp에서 발생하던 비-TAC TopAndBottom 그림의 pushdown/vpos
이중 계상 문제를 해결한다.
체리픽 후 현재 local/devel 기준 자동 검증과 메인테이너 시각 판정을 모두 통과했다.
```

따라서 PR #1084는 체리픽 수용으로 처리하는 것이 타당하다.

## 4. 주의 사항

이번 변경은 그림 배치의 공유 경로를 건드린다.

```text
- gap_before >= picture_height - 8px 이면 파일 vpos가 이미 그림 공간을 반영한 것으로 본다.
- #409 계열처럼 gap이 작아 파일 vpos가 그림 공간을 반영하지 않은 케이스는 기존 pushdown을 유지한다.
```

현재 검증에서는 `pr-149.hwp` 회귀 가드와 기존 lib 테스트가 통과했다.
향후 유사 문서에서 tolerance 경계 케이스가 발견되면 별도 정밀화 대상으로 다룬다.

## 5. 다음 절차

승인 후 진행:

```text
1. pr_1084_review.md / pr_1084_report.md 커밋
2. local/devel → devel fast-forward merge
3. devel 기준 검증
4. origin/devel push
5. PR #1084에 체리픽 반영 댓글 작성 후 close
6. 이슈 #1079 close(completed)
```
