# PR #1091 처리 보고서

- PR: <https://github.com/edwardkim/rhwp/pull/1091>
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/1082>
- 작성일: 2026-05-26
- 처리 방식: `-x` cherry-pick 수용

## 1. 처리 요약

PR #1091의 단일 커밋을 현재 `local/devel`에 체리픽했다.

```text
원본 커밋: e8a3a9fc60ffc27e7db0b0c971fbbc213688d5cc
반영 커밋: 23b202d5 Task #1082: 다단 미주 vpos-absolute 정합 — 본문 하단 overflow 해소
```

변경 내용:

```text
1. TypesetState.prev_body_bottom_vpos 추가
2. 다단 미주 누적을 미주 문단 내부 span이 아니라 직전 배치 아이템 bottom 기준 vpos delta로 계산
3. body → endnote 전환 시 본문 마지막 FullParagraph bottom vpos를 첫 미주의 base로 사용
4. 단 advance 시 미주 base를 None으로 리셋해 새 단 첫 미주는 자체 높이 기준으로 처리
5. dump_page_items에서 미주 paragraphs 인덱싱을 본문 뒤에 합쳐 FullParagraph[미주]로 표시
6. samples/3-09월, 3-11월 교육·실전 통합 샘플 기반 회귀 가드 4개 추가
```

## 2. 검증

자동 검증:

| command | result |
|---|---|
| `cargo fmt --check` | pass |
| `cargo check` | pass |
| `cargo test --test issue_1082_endnote_multicolumn_drift` | pass, 4 passed |
| `cargo test --lib` | pass, 1395 passed / 0 failed / 6 ignored |
| `docker compose --env-file .env.docker run --rm wasm` | pass |

메인테이너 시각 판정:

```text
통과
```

## 3. 판단

수용 판단:

```text
PR #1091은 다단 미주 영역에서 미주 간 vpos 간격을 typeset 누적에 반영하지 않아 발생하던
본문 하단 overflow를 줄인다.
체리픽 후 현재 local/devel 기준 자동 검증과 메인테이너 시각 판정을 모두 통과했다.
```

따라서 PR #1091은 체리픽 수용으로 처리하는 것이 타당하다.

## 4. 주의 사항

이번 변경은 다단 미주 누적의 core typeset 경로를 건드린다.

```text
- col_count > 1 미주에서는 직전 bottom vpos 기준 delta를 사용한다.
- col_count == 1 경로는 기존 formatter 기반 누적을 유지한다.
- #1062 안전 floor(fmt.height_for_fit)는 유지한다.
```

PR 본문에 명시된 잔여 한계:

```text
약 25px 잔여 overflow는 본문 formatter 누적의 trailing line spacing overcount가 미주로 전파되는
작은 base drift로 본다.
단단 본문 formatter 누적의 vpos 정합은 별도 과제다.
```

## 5. 다음 절차

승인 후 진행:

```text
1. pr_1091_review.md / pr_1091_report.md 커밋
2. local/devel → devel fast-forward merge
3. devel 기준 검증
4. origin/devel push
5. PR #1091에 체리픽 반영 댓글 작성 후 close
6. 이슈 #1082 close(completed)
```
