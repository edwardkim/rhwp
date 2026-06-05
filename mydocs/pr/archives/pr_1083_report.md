# PR #1083 처리 보고서

- PR: <https://github.com/edwardkim/rhwp/pull/1083>
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/1073>
- 작성일: 2026-05-26
- 처리 방식: `-x` cherry-pick 수용

## 1. 처리 요약

PR #1083의 단일 커밋을 현재 `local/devel`에 체리픽했다.

```text
원본 커밋: f2a85f86f0234f42f420c40c0b5a2c24ec7914f5
반영 커밋: 8530a722 Task #1073: 중첩 표(셀 내부 표) 페이지 분할 — 본문 하단 overflow 해소
```

변경 내용:

```text
1. 텍스트 없는 단일 중첩 표 문단을 중첩 표 행 단위 CellUnit 으로 분해
2. MeasuredCell.nested_split_row_count 로 중첩 표 행 분할 가능성 판정
3. start_cut/end_cut 을 NestedTableSplit(start_row, end_row) 으로 연결
4. 연속 페이지에서 이전 행부터 시작한 rowspan 라벨 셀을 공란화
5. samples/kps-ai.hwp 기반 회귀 가드 3개 추가
```

## 2. 검증

자동 검증:

| command | result |
|---|---|
| `cargo fmt --check` | pass |
| `cargo check` | pass |
| `cargo test --test issue_1073_nested_table_split` | pass, 3 passed |
| `cargo test --lib` | pass, 1395 passed / 0 failed / 6 ignored |
| `docker compose --env-file .env.docker run --rm wasm` | pass |

메인테이너 시각 판정:

```text
통과
```

## 3. 판단

수용 판단:

```text
PR #1083은 현재 local/devel에 없던 중첩 표 페이지 분할 기능을 구현한다.
samples/kps-ai.hwp / pdf/kps-ai-2022.pdf 기준의 문제 증상과 PR 구현 범위가 일치한다.
체리픽 후 자동 검증과 메인테이너 시각 판정을 모두 통과했다.
```

따라서 PR #1083은 체리픽 수용으로 처리하는 것이 타당하다.

## 4. 잔여

PR 본문과 작업 보고서에 명시된 known limitation:

```text
break row 가 한컴 대비 약 2 중첩행 늦을 수 있다.
구조/콘텐츠 정합과 overflow 0은 달성했지만, break row 정확 일치는 별도 정밀화 대상이다.
2단계 이상 중첩, 텍스트 동거 문단 중첩 표는 atom 폴백을 유지한다.
```

## 5. 다음 절차

승인 후 진행:

```text
1. pr_1083_review.md / pr_1083_report.md 를 커밋에 포함
2. local/devel → devel fast-forward merge
3. devel 기준 검증
4. origin/devel push
5. PR #1083에 체리픽 반영 댓글 작성 후 close
6. 이슈 #1073 close(completed)
```

