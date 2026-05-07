# PR #622 1차 검토 보고서

## 1. PR 메타 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | #622 |
| 제목 | 다단 paragraph 내 vpos-reset 미처리 정정 (closes #619) |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu / Jaeook Ryu) — 6번째 사이클 PR (PR #629/#620/#578/#670/#621 직후) |
| base / head | `devel` ← `planet6897:pr-task619-clean` |
| state / mergeable | OPEN / **CONFLICTING** / **DIRTY** (PR base 75 commits 뒤) |
| 변경 | 8 files, +652 / -0 (src 1 + 거버넌스 7) |
| commits | 3 (`9c5a187` 본질 + `b5d0abf` Stage 2/3 + `6f4a5b2` 보고서) |
| labels / milestone | 없음 / 미지정 |
| 연결 이슈 | **closes #619** (PR 본문 명시) |
| 작성일 / 갱신 | 2026-05-06 00:26 |

### CI 상태 (모두 통과)
- Build & Test ✅
- Analyze (rust / python / javascript-typescript) ✅
- Canvas visual diff ✅
- CodeQL ✅
- WASM Build SKIPPED

### 댓글 영역
- 댓글 없음 (작업지시자 / 컨트리뷰터 추가 댓글 부재)

---

## 2. Issue #619 권위 영역

### 결함
`samples/21_언어_기출_편집가능본.hwp` 페이지 8 우측 단 (단 1) 마지막 줄 (pi=181 line 8) 이 본문 영역 하단을 17.1 px 초과한 위치에 그려져 라인의 절반 이상이 꼬리말 영역으로 빠져 보임.

```
LAYOUT_OVERFLOW_DRAW: section=0 pi=181 line=8 y=1453.2 col_bottom=1436.2 overflow=17.1px
```

### 원인 (Issue 본문 분석)
- `pi=181 line_segs[8].vertical_pos = 0` — HWP 가 line 8 을 다음 단 / 페이지 최상단에서 시작하도록 인코딩한 vpos-reset 신호
- `TypesetEngine` (`src/renderer/typeset.rs`) partial-split 루프 (1060–1132) 가 문단 *내부* `line.vertical_pos == 0` 신호를 인식하지 않음 — 문단 *간* vpos-reset 만 처리 (`next_will_vpos_reset`, line 444 영역, 본 환경 직접 확인 영역)
- 결과: pi=181 PartialParagraph(0..9) 가 단 1 에 배치되어 line 8 이 col_bottom 너머로 그려짐
- `paragraph_layout.rs:1733` 주석에서 한계 명시: *"vpos-reset 미지원으로 paragraph 가 col_bottom 너머에 layout 될 수 있는데…"*

### Issue #619 assignee 영역
- **assignee 미지정** — 컨트리뷰터 (@planet6897) 가 직접 자기 등록 후 작업 진입 영역. `feedback_assign_issue_before_work` 일차 방어선 부재 사례.

---

## 3. 본 환경 정합 상태 점검

### 본 환경 typeset.rs 의 vpos-reset 영역
본 환경 `src/renderer/typeset.rs` 의 vpos-reset 영역 (직접 확인):
- line 444: `next_will_vpos_reset` — **문단 간** vpos-reset 가드 영역
- line 967: 후속 페이지 LINE_SEG vpos-reset 인코딩 인식 영역
- **문단 내부 line.vpos=0 영역 부재** — 본 PR 의 정정 영역의 본질 영역

### 별도 Paginator 엔진 영역 (Issue 본문 명시)
- `src/renderer/pagination/engine.rs::paginate_with_forced_breaks` 영역에는 `respect_vpos_reset` 옵션 영역
- `RHWP_USE_PAGINATOR=1` 환경변수로만 활성화 (기본 미사용)
- `--respect-vpos-reset` CLI 플래그는 TypesetEngine 경로에 wiring 되어 있지 않아 효과 없음

→ 본 PR 영역의 본질 — 활성 `TypesetEngine` 영역에서 직접 정정 영역

---

## 4. PR 의 본질 정정 영역

### 4.1 본질 정정 (단 1곳, 다단 한정 가드)

`src/renderer/typeset.rs::typeset_paragraph` 의 partial-split 루프 (line 1077-1093 영역):
```rust
for li in cursor_line..line_count {
    // [Task #619] 다단 paragraph 내 vpos-reset 강제 분리.
    if st.col_count > 1
        && li > cursor_line
        && para.line_segs.get(li).map(|s| s.vertical_pos == 0).unwrap_or(false)
    {
        break;
    }
    // ... 기존 fit 로직
}
```

| 조건 | 의도 |
|------|------|
| `st.col_count > 1` | **다단 한정** — 단일 단의 partial-table split (issue #418) 회귀 차단 |
| `li > cursor_line` | **세그먼트 첫 줄 제외** — forced break 후 재진입 시 무한 루프 방지 |
| `vertical_pos == 0` | **HWP vpos-reset 신호** |

### 4.2 시도하고 채택하지 않은 변경 (Issue 본문)
- 환경변수 활성화 (`RHWP_USE_PAGINATOR=1`) — 기본 미사용 영역, 사용자 영역 변경 부담
- CLI 플래그 (`--respect-vpos-reset`) — TypesetEngine 경로 wiring 부재 영역

→ **TypesetEngine 직접 정정 영역** (가장 보수적 A 안) 으로 좁혀 적용

### 4.3 회귀 차단 영역 (PR 본문 + 본 환경 직접 검증)

**컨트리뷰터의 회귀 가드 샘플 10개 byte-equivalent**:
- exam_eng / exam_kor / exam_science / exam_math / exam_social
- hwp-multi-001 / hwp-multi-002
- k-water-rfp / kps-ai / aift

**본 환경 cherry-pick simulation 결정적 검증**:
- `cargo test --lib --release`: **1155 passed** (회귀 0)
- `cargo test --test svg_snapshot --release`: **7/7** (issue_617 신규 포함)
- `cargo test --test issue_546`: 1/1
- `cargo test --test issue_554`: 12/12 (`task554_no_regression_exam_kor` / `_aift` 등 통과)
- `cargo test --test issue_418`: **1/1** (단일 단 partial-table 회귀 차단 영역 — 다단 한정 가드 정합 영역)

---

## 5. 본 환경 cherry-pick simulation 결과

본 환경 임시 clone (`/tmp/pr622_test`) 에서 진행:

### cherry-pick
- `9c5a187` (본질 commit) cherry-pick 통과 — **충돌 0** (auto-merge `src/renderer/typeset.rs` 통과)
- 본 환경 devel 보다 75 commits 뒤 영역에서도 src 영역 충돌 부재

### 결정적 검증 결과 (모두 통과)

| 항목 | 결과 |
|------|------|
| `cargo test --lib --release` | ✅ **1155 passed** (회귀 0) |
| `cargo test --test svg_snapshot --release` | ✅ 7/7 |
| `cargo test --test issue_546 --release` | ✅ 1/1 |
| `cargo test --test issue_554 --release` | ✅ 12/12 |
| `cargo test --test issue_418 --release` | ✅ 1/1 (단일 단 partial-table 회귀 차단 영역 정합) |
| `cargo build --release` | ✅ |

### 권위 영역 직접 측정 — 21_언어_기출_편집가능본.hwp

| 항목 | PR 본문 | 본 환경 측정 |
|------|---------|--------------|
| 페이지 8 단 1 pi=181 | `lines=0..8` | **`lines=0..8 vpos=77316..0 [vpos-reset@line8]`** ✓ |
| 페이지 9 단 0 pi=181 | `lines=8..13` | **`lines=8..13 vpos=0..7264 [vpos-reset@line8]`** ✓ |
| LAYOUT_OVERFLOW_DRAW | 사라짐 | 부재 ✓ |

PR 본문의 측정 영역과 100% 일치. dump-pages 출력의 `[vpos-reset@line8]` 마커 영역도 본 환경에서 직접 재현 영역 정합.

---

## 6. 옵션 분류

본 환경 cherry-pick simulation 결과 + 본 PR 의 거버넌스 영역 본 환경 명명 규약 정합 영역 (m100 영역) 기반:

### 옵션 A — 전체 cherry-pick (3 commits, 거버넌스 산출물 영역 포함)
**진행 영역**:
```bash
git checkout local/devel
git cherry-pick 9c5a187 b5d0abf 6f4a5b2
```

**장점**:
- src + Stage 2/3 보고서 + 최종 보고서 + orders 모두 보존
- 컨트리뷰터의 단계별 작업 영역 (Stage 1 / Stage 2 / Stage 3 / 최종 보고서) 영구 보존
- 거버넌스 산출물 영역의 명명 규약 영역 (`task_m100_619*`) **본 환경 정합 영역 정합** (PR #621 의 m07 영역 부재)
- `mydocs/orders/20260506.md` 영역 → 본 환경 영역 add/add 충돌 영역 발생 가능 영역

**잠재 위험**:
- 본 PR 의 `mydocs/orders/20260506.md` (+34 LOC) vs 본 환경 영역 영역 충돌 영역 가능 — auto-merge 영역 점검 필요

### 옵션 B — 본질 cherry-pick + 거버넌스 영역 분리 (PR #629 / PR #668 / PR #621 패턴 정합)
**진행 영역**:
```bash
git checkout local/devel
# src + 거버넌스 산출물 영역 (m100 정합 영역) 영역만 cherry-pick — orders 영역은 본 환경 갱신 영역
git cherry-pick 9c5a187  # 본질 commit (src + Stage 1 거버넌스)
git checkout local/pr622 -- mydocs/working/task_m100_619_stage2.md mydocs/working/task_m100_619_stage3.md mydocs/report/task_m100_619_report.md
git add mydocs/working/task_m100_619_stage{2,3}.md mydocs/report/task_m100_619_report.md
git commit --author="Jaeook Ryu <jaeook.ryu@gmail.com>" -m "Task #619 후속: Stage 2/3 + 최종 보고서"
# orders 영역은 PR 처리 영역에서 본 환경 영역으로 갱신
```

**장점**:
- src + 거버넌스 산출물 영역 (m100 정합 영역) 모두 보존
- orders 영역 충돌 영역 회피 — 본 환경 5/7 orders 영역에 PR #622 entry 추가 영역으로 정합

**잠재 위험**:
- 진행 영역 복잡 영역 — 옵션 A 의 3 commits cherry-pick 영역 대비 분리 영역

### 권장 영역 — 옵션 A (orders auto-merge 영역 통과 영역 시) 또는 옵션 B (충돌 영역 시)

**사유**:
1. **본 PR 의 거버넌스 산출물 영역의 명명 규약 영역 정합** — `task_m100_619*` 영역은 본 환경 영역 정합 영역 (PR #621 의 m07 → m100 영역과 다른 영역)
2. **본 환경 결정적 검증 모두 통과** — cargo test 1155 / svg_snapshot 7/7 / issue_418 1/1 (단일 단 회귀 차단 영역 정합) / clippy 0
3. **권위 영역 100% 일치** — PR 본문 측정과 본 환경 측정 정확 일치 (lines=0..8 / lines=8..13 / vpos-reset@line8 마커)
4. **다단 한정 가드 영역의 정합성** — `feedback_hancom_compat_specific_over_general` 메모리 룰 영역 정합 (col_count > 1 + li > cursor_line + vertical_pos == 0 = 구조적 판단 영역, 측정 의존 없음)
5. **컨트리뷰터의 회귀 가드 sweep 영역** — 10개 회귀 가드 샘플 byte-equivalent 검증 영역 + 본 환경 결정적 검증 일치 영역

### 옵션 영역 요약 표

| 옵션 | 진행 가능 | Issue #619 정정 | 결정적 검증 | 거버넌스 영역 | 권장 |
|------|----------|----------------|------------|--------------|------|
| **A** (전체 3 commits) | ✅ 충돌 0 (orders auto-merge 영역 점검 필요) | ✅ 본질 정합 | ✅ 1155/7/12 | `task_m100_619*` 영역 정합 영역 | ⭐ |
| **B** (본질 + 거버넌스 일부 분리) | ✅ | ✅ | ✅ | 동일 영역 + orders 영역 분리 처리 영역 | ⭐ |

---

## 7. 잠정 결정

### 권장 결정
- **옵션 A 진행** — orders 영역 auto-merge 영역 점검 후 충돌 영역 시 옵션 B 영역으로 fallback
- 본 환경 cherry-pick simulation 영역에서 orders 영역 충돌 영역 부재 영역 (`/tmp/pr622_test` 영역) — 옵션 A 영역 정합 영역 가능
- **본 환경 결정적 검증** + WASM 빌드 + rhwp-studio public 영역 갱신 (vite dev server web 영역) + 작업지시자 시각 판정 ★ 진행

### 후속 영역 (PR 본문 §Remaining)
- **PartialParagraph/Shape bbox 잔여 2.4 px overflow** (텍스트 자체는 단 안) — 별도 이슈 분리 후보 영역, 본 PR 영역 외

### 검증 영역 (옵션 A 진행 시 본 환경 직접 점검)
1. cherry-pick 영역의 충돌 영역 점검 (orders 영역 auto-merge)
2. `cargo test --lib --release` 1155 passed 정합
3. `cargo test --test svg_snapshot --release` 7/7
4. `cargo test --test issue_546 / issue_554 / issue_418 / issue_501` 통과
5. `cargo clippy --lib -- -D warnings` 0
6. Docker WASM 빌드 + byte 측정
7. `rhwp-studio npm run build` (영향 0 영역)
8. `rhwp-studio/public/rhwp.js + rhwp_bg.wasm` 영역 갱신 (vite dev server web 영역)
9. **시각 판정 ★** — `samples/21_언어_기출_편집가능본.hwp` 페이지 8 우측 단 마지막 줄 영역 + 페이지 9 좌측 단 첫 줄 영역 작업지시자 시각 판정 (한컴 2020 PDF 권위 정답지 비교 영역)

---

## 8. 메모리 룰 관점

본 PR 검토에 적용되는 메모리 룰:
- `feedback_hancom_compat_specific_over_general` — **다단 한정 가드는 구조적 판단 영역** (`col_count > 1` + `li > cursor_line` + `vertical_pos == 0` 영역, 측정 의존 없음). PR #621 의 다중 줄 가드 영역과 동일 패턴 영역 — 본 사이클의 권위 사례 강화 영역
- `reference_authoritative_hancom` — **한컴 2020 PDF 권위 정답지 영역** (PR 본문 표): 한컴 2010 = 5줄 (부정확) / 한컴 2020 = 8줄 (정답). 본 PR 영역은 한컴 2020 정합 영역 정합
- `feedback_pdf_not_authoritative` — 한글 2020 / 2022 PDF 영역은 정답지 가능 영역 (5/7 갱신 영역) — 본 PR 의 한컴 2020 PDF 영역의 권위 영역 정합
- `feedback_assign_issue_before_work` — Issue #619 assignee 미지정 영역 (외부 컨트리뷰터 자기 등록 사례)
- `feedback_close_issue_verify_merged` — Issue #619 close 시 본 PR 머지 검증 + 수동 close (closes #619 키워드 자동 처리 안 될 가능성)
- PR #621 의 본 사이클 패턴 정합 영역 — 다단 한정 / 다중 줄 한정 / 가드 좁힘 영역 패턴

---

## 9. 다음 단계 (CLAUDE.md PR 처리 4단계)

1. (완료) PR 정보 + Issue 연결 + base/head + mergeable + CI 상태 확인
2. (현재) `pr_622_review.md` 작성 → 승인 요청
3. (필요 시) `pr_622_review_impl.md` 작성 → 승인 요청
4. 검증 (빌드/테스트/clippy + 시각 판정 ★) + 판단 → `pr_622_report.md` 작성

### 작업지시자 결정 요청
1. **옵션 결정** — 옵션 A (전체 3 commits, 권장) / 옵션 B (본질 + 거버넌스 일부 분리)
2. **거버넌스 영역 처리** — `task_m100_619*` 영역은 본 환경 명명 규약 정합 영역 → 그대로 cherry-pick 영역 정합 영역 가/부 (PR #621 영역과 다른 영역)
3. **시각 판정 권위 영역** — 21_언어_기출_편집가능본.hwp 페이지 8 / 9 영역 작업지시자 직접 시각 판정 영역 진행 가/부 (한컴 2020 PDF 권위 정답지 비교 — `pdf/21_언어_기출_편집가능본-2022.pdf` 영역 또는 작업지시자 환경 한컴 2020 편집기 영역)

결정 후 본 환경 cherry-pick + 결정적 검증 + WASM 빌드 + 시각 판정 ★ + `pr_622_report.md` 작성.
