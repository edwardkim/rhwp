# PR #1691 review 처리 계획서

- PR: https://github.com/edwardkim/rhwp/pull/1691
- 제목: `task 1672: 행정업무 편람 페이지 수를 PDF 기준과 맞춤`
- 작성일: 2026-06-30
- 처리 경로: collaborator self-merge 후보 예외 경로
- 관련 문서: `mydocs/pr/archives/pr_1691_review.md`
- 관련 이슈: #1672

## 1. 커밋 구성

| 커밋 | 제목 | 내용 |
|---|---|---|
| `4b67aefec` | `task 1672: 행정업무 편람 페이지 과대 분할 완화` | RowBreak 표 조기 분할 완화, overlay guide/빈 컬럼 브레이크 흐름 높이 보정, 1차 페이지 수 축소 |
| `07555d200` | `task 1672: 행정업무 편람 페이지 수 정합` | PDF 기준 383쪽에 맞춘 RowBreak 분할 허용치 보정, 샘플 HWP/HWPX/PDF oracle 추가 |

review 문서와 오늘할일 문서는 PR head 에 추가 커밋으로 포함한다.

## 2. Stage 구성

### Stage A — PR head 문서 보강

- `mydocs/pr/archives/pr_1691_review.md` 작성
- `mydocs/pr/archives/pr_1691_review_impl.md` 작성
- `mydocs/orders/20260630.md` 생성·갱신
- 문서 커밋 후 `upstream`의 `task_m100_1672` 브랜치에 push

### Stage B — merge 전 확인

- PR head 최신 커밋 기준 GitHub Actions 결과 확인
- `mergeable` 최신값 확인
- 열린 PR 겹침 상태 재확인
- 작업지시자에게 merge 승인 요청

### Stage C — merge 후 후속 처리

- #1672 자동 close 여부 확인
- 자동 close 실패 시 수동 close/comment
- PR 브랜치 삭제와 로컬 작업 브랜치 정리
- 필요 시 오늘할일과 report 문서 후속 갱신

## 3. 검증 기준

이미 수행한 로컬 검증:

```text
cargo fmt --check
cargo test --release --lib
cargo test --profile release-test --tests
cargo test --doc
cargo clippy --all-targets -- -D warnings
cargo build --release
git diff --check upstream/devel..HEAD
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
wasm-pack build --target web --out-dir pkg
```

페이지 수 기준:

- PDF oracle: 383
- HWP: 383
- HWPX: 383
- `k-water-rfp.hwp`: 27
- `hwpspec.hwp`: 178

문서 추가 커밋 후에는 GitHub Actions 최신 head 결과를 최종 판단 기준으로 삼는다.

## 4. 작업지시자 확인 필요 사항

- merge 전 최종 승인 여부
- 겹침 PR(#1690/#1688/#1683/#1170)과의 처리 순서
- #1672 close 코멘트가 필요할 경우 코멘트 톤과 요약 범위

## 5. 현재 결론

PR #1691은 로컬 검증과 페이지 수 기준을 만족했다. review 문서와 오늘할일을 PR head 에 포함한 뒤,
최신 GitHub Actions green 및 작업지시자 승인 조건으로 merge 후보에 올린다.
