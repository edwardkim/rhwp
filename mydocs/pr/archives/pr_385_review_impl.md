# PR #385 구현계획서 — 정상 머지 (옵션 A)

## 처리 방향 (옵션 A)

작업지시자 결정 — 정상 머지. 정책 검토 (PageRenderTree 안정성, semver, native bridge 방향, serde 의존성) 항목은 작업지시자가 진행 가능 판단.

작성자 (@postmelee) attribution 보존 + cherry-pick 머지.

## 처리 단계

### Stage 0: PR 댓글 정정 + Ready 전환 안내

이전 댓글에서 정책 검토 항목 4개 제기했으나 작업지시자가 정상 머지 결정. 정정 댓글:
- 정책 검토 결과 진행 가능
- DRAFT → Ready 전환 부탁 (또는 메인테이너가 직접 머지)

### Stage 1: cherry-pick

```bash
git checkout local/devel
git checkout -b local/pr385
git fetch origin pull/385/head:pr-385-head
git log pr-385-head --oneline | head -5  # commit 수 확인
git cherry-pick <commits>
```

### Stage 2: 충돌 해결 (있으면)

PR diff 의 22 파일:
- `Cargo.toml` — serde 의존성 추가 (충돌 가능성 낮음)
- `src/document_core/queries/rendering.rs` — devel 의 PR #366 흡수와 다른 위치
- `src/renderer/*`, `src/model/*` — Serialize derive 추가 (devel 변경 적음)
- `mydocs/orders/20260427.md` — 기존 항목과 통합 (수동 가능성)
- `mydocs/plans/task_m100_363*.md`, `mydocs/working/...`, `mydocs/report/...` — 신규
- `mydocs/manual/native_render_tree_bridge_api.md` — 신규
- `mydocs/tech/hwp_spec_errata.md` — 보강

자동 머지 가능성 높음.

### Stage 3: 자동 회귀 검증

- `cargo build --release`
- `cargo test --lib` (1014 → 1016+ 예상)
- `cargo test --test svg_snapshot` (6/6, 회귀 0)
- `cargo test --test issue_301` (1/1)
- `cargo test --test page_number_propagation` (2/2)
- `cargo clippy --lib -- -D warnings`
- `cargo check --target wasm32-unknown-unknown --lib`
- 7 핵심 샘플 회귀 0

### Stage 4: 머지 + close + push

- `local/pr385` → `local/devel` (no-ff)
- `local/devel` → `devel` (FF) push
- PR #385 댓글 + close (정중하고 차분하게, Firefox 기여 인사 포함)
- 이슈 #363 close
- 최종 보고서 (`pr_385_report.md`)

## 작성자 attribution

cherry-pick 으로 author = postmelee (Taegyu Lee) 보존. 추가 메인테이너 변경 (orders 통합 등) 은 별도 commit.

## 리스크

| 리스크 | 대응 |
|------|------|
| `Cargo.toml` 충돌 (serde 추가) | devel 에 serde 직접 의존성 없음 → 자동 머지 |
| Serialize derive 가 다른 변경과 충돌 | devel 의 model/renderer 변경 적음 → 자동 머지 가능성 높음 |
| `orders/20260427.md` 통합 | 수동 통합 (기존 항목 + #363 항목) |
| public API 노출의 향후 호환성 부담 | 작업지시자 결정 진행 — 본 task 범위 외 |

## 다음 단계

본 구현계획서 승인 후 Stage 0~4 진행.
