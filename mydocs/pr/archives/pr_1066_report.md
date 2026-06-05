# PR #1066 처리 보고서 — 시험지 미주(Endnote) 본문 하단 overflow 정합

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#1066](https://github.com/edwardkim/rhwp/pull/1066) |
| 작성자 | @planet6897 |
| 연결 이슈 | [#1062](https://github.com/edwardkim/rhwp/issues/1062) |
| 후속 이슈 | [#1065](https://github.com/edwardkim/rhwp/issues/1065) |
| 처리 결정 | cherry-pick 통합 |
| PR head | `d80af1704864e5475f371995cd472511d117c0f6` |
| cherry-pick commit | `8a1f9fd2` |
| 처리 일자 | 2026-05-22 |

## 처리 절차

### Stage 0: 검토

`mydocs/pr/pr_1066_review.md`에서 코드 검토를 수행했다.

- 본질 변경은 `src/renderer/typeset.rs` 미주 루프 1곳.
- 다단 미주의 fit/누적 기준을 렌더러 vpos 전진
  (`last.vpos + line_height + line_spacing - first.vpos`)으로 정합.
- 단단 경로는 종전 정책 유지.
- 최신 `devel`의 `#1067` 변경과 본질 충돌 없음.
- 차단 이슈 없음.

검토 중 PR 본문과 보고서의 `cargo test --release` passed 수가 서로 다른 점을
확인했다. 이는 코드 차단 이슈는 아니며, 본 처리에서는 현 `devel` 기준 로컬
검증 결과를 별도 기록한다.

### Stage 1: cherry-pick

리뷰 시작 시 로컬 `devel`이 원격보다 4커밋 뒤였으므로 먼저 동기화했다.

```text
git pull --ff-only
5e910168..1cd948d3
```

이후 PR head를 가져와 현 `devel` 위에 cherry-pick했다.

```text
git fetch origin pull/1066/head
git cherry-pick FETCH_HEAD
```

결과:

```text
8a1f9fd2 Task #1062: 시험지 미주(Endnote) 본문 하단 overflow 정합 (#1062, #1049 후속)
12 files changed, 722 insertions(+), 6 deletions(-)
```

`src/renderer/typeset.rs`는 자동 병합되었고 충돌은 없었다.

### Stage 2: 검증

| 항목 | 결과 |
|------|------|
| `docker compose --env-file .env.docker run --rm wasm` | 통과 — Done in 2m 01s |
| `cargo fmt --check` | 통과 |
| `cargo test --release --lib` | 통과 — 1335 passed / 0 failed / 6 ignored |
| 작업지시자 시각 판정 | 통과 |

`cargo test --release --lib` 실행 중 기존 경고 6건이 출력되었으나 테스트 실패는
없었다.

### Stage 3: push

cherry-pick commit `8a1f9fd2`를 `origin/devel`에 push했다.

```text
1cd948d3..8a1f9fd2  devel -> devel
```

GitHub branch rule의 required status check expected 경고는 maintainer 권한으로
bypass되어 push가 완료되었다.

### Stage 4: GitHub 정리

PR #1066은 cherry-pick 통합 완료 댓글을 남기고 close했다.

- PR: https://github.com/edwardkim/rhwp/pull/1066
- 상태: CLOSED
- closedAt: `2026-05-22T07:30:23Z`

Issue #1062는 completed로 close했다.

- Issue: https://github.com/edwardkim/rhwp/issues/1062
- 상태: CLOSED
- closedAt: `2026-05-22T07:30:43Z`

close 댓글에는 commit `8a1f9fd2`, WASM 빌드, `cargo fmt --check`,
`cargo test --release --lib`, 작업지시자 시각 판정 통과, 잔여 #1065 분리를
기록했다.

## 변경 요약

다단 시험지 문서에서 overflow 항목이 본문이 아니라 가상 미주 문단 범위임을
확인하고, 미주 누적 높이를 렌더러 vpos 전진과 맞췄다.

종전:

- 다단 미주 누적이 `height_for_fit` 기준.
- 마지막 `line_spacing`이 미주마다 빠져 약 6px씩 과소 누적.
- 페이지당 미주가 과밀 배치되어 본문 하단 overflow 발생.

정정:

- 다단 미주는 `line_segs`의 vpos 전진을 기준으로 fit/누적 계산.
- fit에는 trailing line spacing 제외 의미를 보존.
- 누적에는 vpos 전진을 반영.
- `fmt.height_for_fit` floor로 기존보다 더 조밀하게 배치되는 회귀 차단.

## 잔여와 후속

PR 자체가 잔여를 #1065로 분리했다.

- 3-09 2022 잔여 20건.
- 소폭 악화 4파일(+8, 3~23px).
- 원인은 typeset 미주 분할점과 렌더러 vpos base reset 지점 미정렬로 분석됨.

본 PR의 trailing line spacing 과소 누적 정정과는 별도 축이므로 #1065에서
추적한다.

## 최종 판단

PR #1066은 cherry-pick 방식으로 `devel`에 통합했고, WASM 빌드와 핵심 로컬
검증 및 작업지시자 시각 판정을 통과했다. PR과 연결 이슈 #1062 모두 처리
완료 상태다.
