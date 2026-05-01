# PR #371 구현계획서 — 정상 머지 (rebase 후 충돌 해결)

## 처리 방향 (옵션 A)

PR #371 의 결함 정정은 단독이며 메인테이너 동시 정정 없음. CONFLICTING 상태이지만 변경 범위가 좁고 명확하므로 정상 머지.

작성자 (@planet6897) attribution 보존 + 머지 후 PR close.

## 충돌 분석 (사전)

PR #371 의 변경 파일 11개:
- `src/renderer/mod.rs` — 핵심 변경
- `tests/golden_svg/{form-002,issue-157,issue-267}/*.svg` — 골든 SVG 갱신
- `mydocs/{plans,working,report}/task_m100_370*.md` — 작성자 문서 6개
- `mydocs/orders/20260427.md` — orders 갱신

**충돌 가능 위치**:
1. **`src/renderer/mod.rs`** — devel 에 PR #366 흡수 commit 으로 `pub mod page_number;` 추가됨. PR #371 은 다른 줄 (`generic_fallback`) 변경 → **자동 머지 가능성 높음**
2. **`mydocs/orders/20260427.md`** — 메인테이너가 v0.7.7 릴리즈 시 갱신 + Task #361, #362 항목 추가. PR #371 은 다른 항목 추가 → **수동 해결 필요 가능성**
3. **골든 SVG** — devel 에 동일 파일 변경 없음 → 충돌 없음 (예상)
4. **PR 의 task_m100_370* 문서** — devel 에 없음 → 충돌 없음

## 단계

### Stage 1: 브랜치 + rebase

```bash
git checkout local/devel
git checkout -b local/pr371
git fetch origin pull/371/head:pr-371-head
# rebase 수행
git rebase local/devel pr-371-head
# 또는 cherry-pick 으로 안전하게
```

선택지:
- **Option 1 (cherry-pick)**: PR 의 commit 들을 새로 만든 브랜치에 cherry-pick (작성자 attribution 보존)
- **Option 2 (rebase + force)**: pr-371-head 를 local/devel 위로 rebase 후 PR head 갱신

본 PR 은 작성자가 외부 fork 가 아닌 같은 저장소 브랜치 (`local/task370`) 사용. 그러나 메인테이너가 PR 브랜치를 push 할 수 없을 수 있으므로 **Option 1 (cherry-pick) 권장**.

### Stage 2: 충돌 해결

자동 충돌 해결 시도 후 실패 시:
- `mod.rs`: PR #366 의 `pub mod page_number;` 와 PR #371 의 `generic_fallback` 변경 통합 (다른 줄이라 충돌 없을 가능성 높음)
- `orders/20260427.md`: 두 변경 통합 (Task #361, #362 + 작성자의 #370 항목)

### Stage 3: 자동 회귀 + 시각 검증

- `cargo build --release`
- `cargo test --lib` (1014+ passed 확인 — Task #361, #362, PR #366 효과 유지)
- `cargo test --test svg_snapshot` (6/6 — 골든 SVG 갱신 영향 확인)
- `cargo test --test page_number_propagation` (2/2)
- `cargo clippy --lib -- -D warnings`
- `cargo check --target wasm32-unknown-unknown --lib`
- 7 핵심 샘플 + form-002 + k-water-rfp + kps-ai 페이지 수 + LAYOUT_OVERFLOW 회귀 0
- form-002.hwpx 10쪽 SVG 출력 — `<text>` 의 font-family 에 `Nanum Myeongjo`, `Noto Serif CJK KR` 포함 확인

### Stage 4: 작업지시자 시각 판정 (선택사항)

WASM 빌드 후 form-002 10쪽 시각으로 볼드 한글 표시 확인. macOS/Linux 환경 의존이므로 작업지시자 환경에 따라 판정 가능.

### Stage 5: merge + close

- `local/pr371` → `local/devel` merge (작업지시자 승인 후)
- `local/devel` → `devel` (FF) push
- PR #371 댓글 + close (정중한 톤)
- 이슈 #370 close

## 작성자 attribution 보존

- cherry-pick 으로 author = planet6897 보존
- 골든 SVG 갱신 + 메인테이너 충돌 해결만 별도 commit

## 주의 사항

작업지시자 강조: **"머지시 주의해야 합니다"**

신중 점검:
- [ ] PR 의 mod.rs 변경이 정확히 두 분기 (한글 / 영문 세리프) 만 변경하는지 확인
- [ ] devel 의 mod.rs 와 자동 머지 시 다른 변경 손실 없음
- [ ] 골든 SVG 갱신이 폰트 체인 문자열만 변경하는지 (다른 시각 변경 없음)
- [ ] cargo test --lib 1014 passed 유지 (Task #361, #362, PR #366 효과)
- [ ] Linux 환경에서 한글 세리프 폴백 정상 (`Noto Serif CJK KR` 추가 후)

## 리스크

| 리스크 | 대응 |
|------|------|
| `mod.rs` 충돌 (PR #366 흡수와 동일 파일) | 다른 줄 변경 — 자동 머지. 충돌 시 수동 통합 |
| `orders/20260427.md` 충돌 | 두 변경 통합 |
| 골든 SVG 변경이 다른 영향 | 골든 갱신 후 svg_snapshot 통과 확인 |
| Linux/macOS 환경별 시각 차이 | 작업지시자 판정 |

## 다음 단계

본 구현계획서 승인 후 Stage 1~5 진행.
