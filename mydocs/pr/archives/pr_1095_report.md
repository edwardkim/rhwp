# PR #1095 최종 보고 — Task #977 v3: WASM 미등록 폰트 폴백 native EmbeddedTextMeasurer 동기화

## 1. 결정

**merge 수용** — 검증 7/7 통과, CI 전부 pass, 본질 정확, 회귀 없음.

| 항목 | 결과 |
|------|------|
| PR | [#1095](https://github.com/edwardkim/rhwp/pull/1095) |
| 작성자 | planet6897 (Jaeuk Ryu) — 16+ 사이클 |
| 이슈 | closes #977 |
| 변경 | +394 / -9, 6 files (코드 1 + 문서 5) |
| merge 방식 | merge commit (작업지시자 결정) |

## 2. v3 사이클의 본질 분리

| 차수 | PR | 처리 사유 |
|------|----|----------|
| v1 | #980 | 자진 close |
| v2 | #1045 | 메인테이너 close — "PR #1026 본질 흡수" |
| **v3** | **#1095** | PR #1026 정정 후 잔존 — **`measure_hangul_width_hwp`** 도 동일 패턴 통일 필요 |

v3 본질이 v2 close 사유와 명확히 분리됨 (다른 함수 영역).

## 3. 검증 결과 (메인테이너 재검증)

PR head (`pr-1095-head`) 직접 체크아웃 후 로컬 검증:

| 항목 | 명령 | 결과 |
|------|------|------|
| 회귀 가드 #874 | `cargo test --release --test issue_874_ktx_toc_page_number_right_align` | ✅ **1/1 PASS** |
| svg_snapshot (8 golden) | `cargo test --release --test svg_snapshot` | ✅ **8/8 PASS** |
| lib 전체 | `cargo test --release --lib` | ✅ **1382 passed / 0 failed** |
| WASM target check | `cargo check --target wasm32-unknown-unknown --lib` | ✅ OK |
| clippy | `cargo clippy --release --lib -- -D warnings` | ✅ clean |
| fmt | `cargo fmt --all --check` | ✅ clean |
| CI (GitHub Actions) | Build & Test / CodeQL / Canvas visual diff / Analyze | ✅ 전부 pass |

작성자 검증 영역 + 메인테이너 재검증 영역 모두 일치 — **회귀 없음 확인**.

## 4. 코드 변경 평가

`src/renderer/layout/text_measurement.rs` (+21 / -9, 매우 작은 영역):

### 4.1 `measure_char_width_hwp` JS 폴백 분기

```rust
// JS Canvas measureText → native heuristic
if super::is_cjk_char(c) || super::is_fullwidth_symbol(c) {
    return font_size;  // CJK/fullwidth: 1.0 em
}
font_size * 0.5  // 일반 문자: 0.5 em
```

PR #1026 의 narrow_punct (0.3 em) 분기는 위에서 이미 처리 → 보존 확인.

### 4.2 `measure_hangul_width_hwp` JS 폴백 분기

```rust
// JS Canvas '가' 측정 → native CJK 휴리스틱
(font_size * 75.0).round() as i32  // 1.0 em
```

`measure_font` 파라미터 unused 처리 (`_measure_font`) — 시그니처 보존.

### 4.3 영향 영역 평가

| 케이스 | 영향 |
|--------|------|
| 등록 폰트 (맑은 고딕, HCR Batang 등) | `_embedded` Some 반환 → 본 분기 미진입, **무회귀** |
| 미등록 폰트 (나눔바른고딕 등) | native heuristic 동일 폭 → SVG/WASM 일관 정렬 |
| CJK / fullwidth | 1.0 em (PR #1026 정합) |
| narrow_punct | 0.3 em (PR #1026 정합, 보존) |

## 5. 처리

- PR head 직접 검증 → 모두 통과 → CI 통과 → merge 결정
- GitHub PR merge (merge commit, 본 PR 자체 commit 보존)
- review/report → `mydocs/pr/archives/` 이동
- 이슈 #977 close (PR 의 closes #977 로 자동)

## 6. 메모리 룰 정합

- ✅ `feedback_contributor_cycle_check` — planet6897 16+ 사이클, v1/v2 이력 확인
- ✅ `feedback_pr_supersede_chain` — v1/v2 close 후 v3 통합 패턴 (close+통합)
- ✅ `feedback_pr_comment_tone` — 반복 컨트리뷰터, 차분한 사실 중심 merge 메시지
- ✅ `feedback_release_sync_check` — devel merge 전 origin/devel 동기화
- ✅ `feedback_push_full_test_required` — lib + tests + clippy + fmt + WASM 모두 통과

## 7. 후속

- 본 PR merge 후 `pr/pr_1095_review.md` + `pr_1095_report.md` → `pr/archives/` 이동 commit
- 이슈 #977 자동 close 확인
