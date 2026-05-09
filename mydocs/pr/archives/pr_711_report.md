---
PR: #711
제목: Task #705 — 한컴 호환 셀 안 PageHide 컨트롤 본질 정정
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터
처리: 옵션 A — 7 commits 단계별 보존 cherry-pick + 메인테이너 가드 갱신 + no-ff merge
처리일: 2026-05-09
머지 commit: 2bc982cb
---

# PR #711 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (7 commits cherry-pick + 메인테이너 가드 갱신 + no-ff merge `2bc982cb`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `2bc982cb` (--no-ff merge) |
| 메인테이너 가드 갱신 commit | `3253f7d0` (test_634_gukrip_page3 count=3→0) |
| Issue #705 | close 자동 정합 (closes #705) |
| 시각 판정 | ★ **통과 (작업지시자 직접, 정답지 정합 확정)** |
| 자기 검증 | lib **1173** (+6 test_705) + 통합 ALL GREEN + clippy clean |

## 2. 정정 본질 — 두 페이지네이션 경로 + 3 결함

### 2.1 결함 #1 — 셀 안 PageHide 수집 누락 (engine.rs + typeset.rs)

```rust
// 두 경로 동기 정정
Control::Table(table) => {
    Self::collect_pagehide_in_table(table, pi, &mut page_hides);
}

fn collect_pagehide_in_table(table, pi, page_hides) {
    for cell in &table.cells {
        for cp in &cell.paragraphs {
            for ctrl in &cp.controls {
                match ctrl {
                    Control::PageHide(ph) => page_hides.push((pi, ph.clone())),
                    Control::Table(inner) => Self::collect_pagehide_in_table(inner, pi, page_hides),
                    _ => {}
                }
            }
        }
    }
}
```

→ `feedback_image_renderer_paths_separate` 권위 룰 정합 (engine.rs + typeset.rs 두 경로 동기).

### 2.2 결함 #2 — layout.rs `hide_fill`/`hide_border` 가드 부재

기존: `hide_master`, `hide_header`, `hide_footer`, `hide_page_num` 만 가드. PR #711 가 6 필드 모두 가드 정합화.

### 2.3 결함 #3 — main.rs dump 셀 안 PageHide 분기 부재

dump 도구 영역의 셀 안 controls 매칭에 PageHide 분기 추가.

## 3. PR supersede 영역

| PR | 결과 |
|----|------|
| PR #638 (Task #634) | close — src 무변경 회귀 가드만 머지 |
| PR #640 (Task #637) | close — 분석 docs (H2 측정 누락 영역) |
| PR #641 (Task #639) | close — cover-style 휴리스틱 폐기 (작업지시자 권고) |
| **PR #711 (Task #705)** | **머지** — 본질 정정 supersede |

## 4. 본 환경 cherry-pick + 검증

### 4.1 cherry-pick (7 commits)
```
f3021404 Task #705 Stage 0: 사전 측정 + 198 샘플 재조사
9588efe5 Task #705 Stage 1 (RED): 셀 안 PageHide 검증 통합 테스트 4건
94ab495c Task #705 Stage 2 (GREEN #1): 셀 안 PageHide 페이지네이션 수집 정정
e6141605 Task #705 Stage 3 (GREEN #2): layout.rs hide_fill/hide_border 가드 추가
c47cd5ee Task #705 Stage 4 (#3): main.rs dump 셀 안 PageHide 분기 추가
2dfe4052 Task #705 Stage 5: 회귀 검증 완료 + 최종 보고서
77deb239 Task #705 후속: issue-267 골든 SVG 갱신 (page number footer 제거)
```
충돌 0건 (auto-merging main.rs + typeset.rs).

### 4.2 메인테이너 가드 갱신 commit (`3253f7d0`)

`test_634_gukrip_page3_shows_page_number` 가드 갱신:
- BEFORE: `count == 3` (rhwp 의 한컴 부정합 행위 보존, PR #634 시점)
- AFTER: `count == 0` (한컴 권위 정합, PR #711 시각 판정 통과)

### 4.3 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 (27.98s) |
| `cargo test --release --lib test_705` | ✅ **6/6 PASS** (신규 회귀 가드) |
| `cargo test --release --lib test_634_gukrip` | ✅ **2/2 PASS** (가드 갱신 후) |
| `cargo test --release` | ✅ lib **1173** (1167 → 1173, +6 test_705) + 통합 ALL GREEN, failed 0 |
| `cargo clippy --release --all-targets` | ✅ 신규 경고 0 |
| 광범위 sweep | 7 fixture / **170 페이지 / diff 2** (aift p2/p3 의도된 변경) |
| WASM 빌드 (Docker) | ✅ 4,606,179 bytes |

### 4.4 광범위 sweep diff 분석

```
aift: total=77 same=75 diff=2  diff_pages=[aift_002.svg aift_003.svg]
다른 6 fixture: total=93 same=93 diff=0
```

→ 의도된 변경:
- `aift_002.svg` — 셀[167] 6 필드 모두 true 적용 영역 (background 제거 + page_num 미표시)
- `aift_003.svg` — 셀[31] hide_page_num 적용 영역 (page_num 미표시)

다른 fixture 회귀 0.

### 4.5 시각 판정 ★ 통과

작업지시자 시각 판정 — 한컴 편집기 [감추기] 다이얼로그 영역 + page 3 쪽번호 영역 직접 확인:
- 한컴 PDF 권위본 (`pdf/2022년 국립국어원 업무계획-2022.pdf` page 3 마지막 줄 `- 1 -`) 정합
- aift.hwp page 2 + 국립국어원 page 3 등 셀 안 PageHide 영역 모두 정합

## 5. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @planet6897 30+ 사이클 핵심 컨트리뷰터 |
| `feedback_pr_supersede_chain` | PR #638/#640/#641 close → PR #711 본질 정정 supersede (작업지시자 권고 영역) |
| `feedback_visual_judgment_authority` | CI FAILURE + test_634 가드 영역 vs 작업지시자 시각 판정 충돌 → **시각 판정 권위로 가드 갱신** (PR #706 form-002 패턴 정합) |
| `feedback_visual_regression_grows` | test_634 가드 영역 의 본질 부정확 영역 — 가드 의도 변경 신규 사례 (count=3 → count=0) |
| `feedback_image_renderer_paths_separate` | 두 페이지네이션 경로 (engine.rs + typeset.rs) 동기 정정 — collect_pagehide_in_table 재귀 함수 |
| `feedback_rule_not_heuristic` | PR #641 cover-style 휴리스틱 폐기 영역 의 본질 정정 |
| `feedback_close_issue_verify_merged` | Issue #705 close 영역 의 PR 머지 정합 |

## 6. 잔존 후속

- 본 PR 본질 정정 영역 의 잔존 결함 부재
- aift.hwp / 2022 국립국어원 / KTX / kps-ai / tac-img-02 모든 영향 샘플 영역 의 셀 안 PageHide 영역 한컴 권위 정합 영역

---

작성: 2026-05-09
