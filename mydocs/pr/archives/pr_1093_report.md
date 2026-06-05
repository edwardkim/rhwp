# PR #1093 처리 보고서

- PR: <https://github.com/edwardkim/rhwp/pull/1093>
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/1046>
- 참고 이슈: <https://github.com/edwardkim/rhwp/issues/1065>
- 작성일: 2026-05-26
- 처리 방식: `-x` cherry-pick 수용 후보

## 1. 처리 요약

PR #1093의 단일 커밋을 현재 `local/devel`에 체리픽했다.

```text
원본 커밋: f269935f6b1f341d2ea0da3f91d8a481361a18cd
반영 커밋: 5c3f15be Task #1046: 본문 하단 overflow 정합 — 측정 통일(B) (rebase onto stream/devel)
```

이 PR은 닫힌 PR #1048의 rebase 재제출이다.

변경 내용:

```text
1. 표 첫 fragment 배치 가능성 판단 시 host_spacing.before와 positive vertical_offset overhead를 반영
2. 다행 표의 첫 비분할 블록이 fresh page에는 들어가면 다음 페이지로 이월
3. LayoutEngine.last_item_content_bottom을 추가해 trailing spacing 기반 overflow 오검출을 줄임
4. LayoutOverflow에 section_index / is_first_in_column 정보를 추가
5. paginate_pass(force_breaks) 경로를 추가하고 typeset 호출부와 정합
6. #1046 계획/작업/보고 문서 반영
```

## 2. 검증

자동 검증:

| command | result |
|---|---|
| `cargo fmt --check` | pass |
| `cargo check` | pass |
| `cargo test --lib` | pass, 1396 passed / 0 failed / 6 ignored |
| `docker compose --env-file .env.docker run --rm wasm` | pass |

메인테이너 시각 판정:

```text
통과
```

## 3. 판단

수용 후보 판단:

```text
PR #1093은 #1046의 본문 하단 overflow 잔여 문제를 cut/추정 높이와 렌더러 실제 height의
측정 통일 관점에서 줄이는 변경이다.
현재 local/devel에는 #1084/#1091 변경이 이미 반영되어 있었으나, 체리픽은 충돌 없이 적용되었다.
자동 검증과 wasm 빌드는 통과했다.
```

따라서 PR #1093은 체리픽 수용으로 처리하는 것이 타당하다.

## 4. 주의 사항

이번 변경은 `src/renderer/typeset.rs`, `src/renderer/layout.rs`,
`src/renderer/layout/paragraph_layout.rs`, `src/document_core/queries/rendering.rs`의
페이지네이션/overflow 측정 경로를 건드린다.

주의 지점:

```text
1. trailing spacing으로 인한 false-positive overflow를 줄이지만, 실제 콘텐츠 overflow를 놓치지 않아야 한다.
2. 표 첫 fragment 이월 정책은 페이지 분할 위치를 바꿀 수 있다.
3. #1084의 그림 pushdown/vpos 정정과 #1091의 다단 미주 vpos 정합 변경은 체리픽 후 유지된다.
4. PR 본문에서 #1065 개선 효과를 언급하지만 #1065는 별도 이슈로 유지한다.
```

## 5. 다음 절차

승인 후 진행:

```text
1. pr_1093_review.md / pr_1093_report.md 커밋
2. local/devel → devel fast-forward merge
3. devel 기준 검증
4. origin/devel push
5. PR #1093에 체리픽 반영 댓글 작성 후 close
6. 이슈 #1046 close(completed)
```

이슈 #1065는 이번 PR 수용으로 자동 close하지 않는다.
