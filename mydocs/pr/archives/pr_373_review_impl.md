# PR #373 구현계획서 — cherry-pick 방식 머지

## 처리 방향 (옵션 A)

PR #373 의 결함 정정은 단독이며 메인테이너 동시 정정 없음. BEHIND 상태이지만 변경 범위가 작고 명확하므로 cherry-pick 방식 머지.

작성자 (@planet6897) attribution 보존 + 머지 후 PR close.

## 단계

### Stage 1: cherry-pick

```bash
git checkout local/devel
git checkout -b local/pr373
git fetch origin pull/373/head:pr-373-head
git log pr-373-head --oneline | head -5  # commit 수 확인
git cherry-pick <commits>
```

### Stage 2: 충돌 해결 (있으면)

PR diff 의 3 파일 모두 devel 의 최근 변경과 무관 → 자동 머지 예상. 충돌 시 수동 통합.

### Stage 3: 자동 회귀 + 시각 검증

- `cargo build --release`
- `cargo test --lib` (1014+ passed 유지)
- `cargo test --test svg_snapshot` (6/6)
- `cargo test --test issue_301` (1/1)
- `cargo test --test page_number_propagation` (2/2)
- `cargo clippy --lib -- -D warnings`
- `cargo check --target wasm32-unknown-unknown --lib`
- 7 핵심 샘플 회귀 0
- **시각 검증**: `samples/hwpx/hwpx-h-02.hwpx` 9쪽 SVG 출력 — `fill="#cdf2e4"` `<rect>` 등장 확인

### Stage 4: orders 갱신 (메인테이너)

`mydocs/orders/20260427.md` 에 #372 항목 추가 (PR 에 작성자가 안 넣었으므로 메인테이너가 추가).

### Stage 5: merge + close

- `local/pr373` → `local/devel` merge
- `local/devel` → `devel` (FF) push
- PR #373 댓글 + close (간결, 사실 중심)
- 이슈 #372 close
- 최종 보고서 (`pr_373_report.md`)

## 작성자 attribution

cherry-pick 으로 author 보존 (Jaeook Ryu). 추가 메인테이너 변경 (orders 갱신, 검토 문서) 은 별도 commit.

## 리스크

| 리스크 | 대응 |
|------|------|
| svg.rs 의 `<rect>` 가 다른 케이스에서 영향 | 가드 조건 (`!= 0xFFFFFF && != 0`) 정확. 7 핵심 샘플 svg_snapshot 으로 검증 |
| cherry-pick 시 충돌 | PR diff 와 devel 변경 비교 — svg.rs/plans/pdf 모두 devel 에 무변경 → 충돌 가능성 낮음 |
| BEHIND 라 base 가 오래됨 | cherry-pick 으로 자동 해결 |

## 다음 단계

본 구현계획서 승인 후 Stage 1~5 진행.
