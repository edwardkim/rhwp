# PR #632 1차 검토 보고서

## 1. PR 메타 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | #632 |
| 제목 | Task #631: HWP 권위값 더블체크로 vpos-reset 인접 line 보존 |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu / Jaeook Ryu) — 8번째 사이클 PR |
| base / head | `devel` ← `planet6897:pr-task631` |
| state / mergeable | OPEN / **CONFLICTING** / **DIRTY** (PR base 89 commits 뒤) |
| 변경 | 9 files, +742 / -1 (src 1 +18/-1 + 거버넌스 7 + samples/aift.pdf) |
| commits | 6 (Stage 1~4 + 본질 + PDF 추가) |
| labels / milestone | 없음 / 미지정 |
| 연결 이슈 | **closes #631** (PR 본문 명시) |
| 작성일 / 갱신 | 2026-05-06 03:36 |

### CI 상태 (모두 통과)
- Build & Test ✅
- Analyze (rust / python / javascript-typescript) ✅
- Canvas visual diff ✅
- CodeQL ✅
- WASM Build SKIPPED

### 댓글 영역
- 댓글 없음

---

## 2. Issue #631 권위 영역

### 결함
`aift.hwp` 의 page 18 (=PDF 인쇄 page 12) 하단의 `pi=222` 단락 (`- 기능별 에이전트 구현 : 두아즈가...`) 에서 PDF 영역은 첫 2줄이 page 18 영역에 들어가지만 본 환경 렌더러 영역은 1줄만 들어가고 line 1~3 영역이 다음 페이지 영역으로 밀림.

### 진단 (Issue 본문)
페이지 18 영역에서 본 환경 렌더러 영역이 HWP 대비 +15.4 px 더 누적 영역 → pi=222 의 두 번째 줄 (line slot ≈25.6 px) 영역이 body 하단 (971.4 px) 영역을 넘어가 다음 페이지 영역으로 밀림.

### Issue #631 assignee 영역
- assignee 미지정 — 컨트리뷰터 자기 등록 영역.

---

## 3. 본 환경 정합 상태 점검

### 본 환경 typeset.rs 영역 (직접 확인)
- `LAYOUT_DRIFT_SAFETY_PX = 10.0` 영역 line 908 영역 잔존 — PR 의 정정 영역의 본질 영역
- 두 곳 (typeset.rs:1046 + 1074) 영역에서 차감 → 합계 20 px 보수 마진 영역

### Task #332 stage4b 영역 (PR 본문 명시)
PR 본문: "Task #332 stage4b 커밋 (`0211e574`) 영역에서 명시적으로 알려진 회귀 — `aift pi=222`, `21_언어 pi=10 line 1`, `hwp-multi-002 pi=68` 영역 가운데 마지막 미해결분."

→ 본 PR 영역의 본질 영역 — 알려진 회귀 영역의 종결 영역 정합.

### 본 환경 PDF 영역 정합 영역
- 본 환경 `pdf/aift-2022.pdf` (PR #670 영구 보존 영역, 1,767,558 bytes)
- PR 의 `samples/aift.pdf` (1,723,119 bytes, 다른 SHA) — **본 환경 명명 규약 영역과 충돌 영역**

| 영역 | 본 환경 영역 | PR 영역 |
|------|-------------|---------|
| 위치 | `pdf/aift-2022.pdf` (PR #670 패턴) | `samples/aift.pdf` (PR 본문 영역) |
| 크기 | 1,767,558 bytes | 1,723,119 bytes |
| SHA | `8cbfb6c2...` | `b42c25d8...` |

→ PR 의 PDF 영역은 **다른 PDF** (한컴 환경 / 시점 차이 영역). 본 환경 명명 규약 (`pdf/{stem}-{버전}.pdf`) 와 충돌 영역.

---

## 4. PR 의 본질 정정 영역

### 4.1 본질 정정 (단 1곳, +18/-1)

`src/renderer/typeset.rs:1078~1098` — partial-split 루프 영역에 HWP 권위값 더블체크 추가:

```rust
for li in cursor_line..line_count {
    let content_h = fmt.line_heights[li];
    if cumulative + content_h > avail_for_lines && li > cursor_line {
        // [Task #631] HWP 권위값 더블체크
        let hwp_authoritative = para.line_segs.get(li + 1)
            .map(|next| next.vertical_pos == 0)
            .unwrap_or(false)
            && para.line_segs.get(li).map(|cur| {
                let bottom_px = crate::renderer::hwpunit_to_px(
                    cur.vertical_pos + cur.line_height, self.dpi);
                bottom_px <= st.base_available_height()
            }).unwrap_or(false);
        if !hwp_authoritative {
            break;
        }
    }
    cumulative += fmt.line_advance(li);
    end_line = li + 1;
}
```

### 4.2 조건 영역 (모두 AND)
1. typeset 누적 추정 영역으로 fit 실패 (기존 break 조건)
2. **다음 줄 vpos == 0** — HWP 가 명시적으로 페이지 경계 신호 인코딩 영역
3. **현재 줄 vpos+lh ≤ body_available_height** — HWP 좌표상 본문 안 영역

→ 조건 2~3 모두 참일 때만 break 우회. **조건 2 영역이 매우 좁아 회귀 위험 영역 최소** (구조적 가드 영역, 측정 의존 부재 영역).

### 4.3 PR #622 의 본 사이클 패턴 정합 영역
- PR #622 (Task #619): 다단 paragraph 영역의 vpos-reset 미처리 영역 — `col_count > 1 && li > cursor_line && vertical_pos == 0` 가드
- 본 PR (Task #631): partial-split 영역의 vpos-reset 인접 line 보존 영역 — 다음 줄 `vertical_pos == 0` + 현재 줄 `vpos+lh ≤ body_h` 더블체크

→ **본 사이클의 vpos-reset 영역의 권위 사례 누적 영역**. 둘 다 구조적 가드 영역 + HWP 인코딩 신호 영역 존중 영역.

### 4.4 광범위 회귀 검증 (PR 본문)
- 155 샘플 / 1,255 페이지
- 부정적 회귀 0건
- **1,300+ 누락 콘텐츠 복구**:
  - hwpctl_API_v2.4: +2 페이지 / +460 text elements
  - hwpspec: +1 페이지 / +855 text elements
  - aift, hongbo, hwp3-sample4/5, loading-fail-01: 페이지 끝 1줄 복원

---

## 5. 본 환경 cherry-pick simulation 결과

본 환경 임시 clone (`/tmp/pr632_test`) 에서 진행:

### cherry-pick
- 6 commits 영역 — 5 commits 충돌 0 + 1 commit (`86fba08` Stage 4) 영역에서 `mydocs/orders/20260506.md` add/add 충돌 영역
- 충돌 처리: `git checkout --ours` 영역 본 환경 영역 보존 정합 (PR #622/#627 영역과 동일 패턴)

### 결정적 검증 결과 (모두 통과)

| 항목 | 결과 |
|------|------|
| `cargo test --lib --release` | ✅ **1156 passed** (회귀 0) |
| `cargo test --test svg_snapshot --release` | ✅ 7/7 |
| `cargo test --test issue_546 --release` | ✅ 1/1 |
| `cargo test --test issue_554 --release` | ✅ 12/12 (`task554_no_regression_aift` / `_exam_kor` 등 통과) |
| `cargo test --test issue_418 --release` | ✅ 1/1 |
| `cargo test --test issue_501 --release` | ✅ 1/1 |
| `cargo clippy --lib --release -- -D warnings` | ✅ 0 |

### 권위 영역 직접 측정 (aift.hwp page 18/19)

| 항목 | PR 본문 | 본 환경 측정 |
|------|---------|--------------|
| 페이지 18 pi=222 | `lines=0..2` | **`lines=0..2 vpos=67980..0 [vpos-reset@line2]`** ✓ |
| 페이지 19 pi=222 | `lines=2..4` | **`lines=2..4 vpos=0..1920 [vpos-reset@line2]`** ✓ |

PR 본문 명세와 100% 일치.

---

## 6. 옵션 분류

본 환경 cherry-pick simulation 결과 + samples/aift.pdf 영역의 본 환경 명명 규약 충돌 영역 + 거버넌스 영역 본 환경 명명 규약 (m100) 정합 영역 기반:

### 옵션 A — 전체 cherry-pick (6 commits 단계별 보존)
**진행 영역**:
```bash
git checkout local/devel
git cherry-pick b6efbf7 898527d b7cc2d4 8f60ae6 86fba08 2ad3bfa
# Stage 4 commit 영역의 mydocs/orders/20260506.md add/add 충돌 → ours 보존
```

**장점**:
- TDD 흐름 영역 (Stage 1 진단 → Stage 2 구현계획 → Stage 2 정정 → Stage 3 광범위 검증 → Stage 4 보고서) 모두 보존
- 6 commits 단계별 보존 → bisect 영역 정합
- author Jaeook Ryu 6 commits 모두 보존
- 거버넌스 산출물 영역 (`task_m100_631*`) 본 환경 명명 규약 정합 영역

**잠재 위험**:
- **`samples/aift.pdf` 영역 (1.7 MB) 추가** — 본 환경 명명 규약 (`pdf/{stem}-2022.pdf` PR #670 패턴) 영역과 충돌 영역
- 본 환경 영구 보존 영역의 중복 영역 (본 환경 `pdf/aift-2022.pdf` 영역 + PR 의 `samples/aift.pdf` 영역 = 다른 PDF 영역, 동시 존재 영역의 모호성 영역)

### 옵션 B — src + 거버넌스 영역만 cherry-pick (`samples/aift.pdf` 제외, PR #621/#642/#668 패턴 정합)
**진행 영역**:
```bash
git checkout local/devel
git cherry-pick b6efbf7 898527d b7cc2d4 8f60ae6 86fba08
# 마지막 commit (2ad3bfa, samples/aift.pdf 추가) 영역 제외
# Stage 4 commit 영역의 orders 충돌 → ours 보존
```

**장점**:
- src 영역 + 거버넌스 산출물 영역 + Stage 1~4 모두 보존
- **`samples/aift.pdf` 영역 제외** — 본 환경 명명 규약 영역 (PR #670 의 `pdf/aift-2022.pdf`) 영역과 충돌 영역 회피
- 본 환경 영구 보존 영역의 일관성 영역 정합

**잠재 위험**:
- 컨트리뷰터의 PDF 영역 보존 의지 영역 손실 — 단 본 환경의 `pdf/aift-2022.pdf` 영역 (PR #670) 영역으로 충분 영역

### 권장 영역 — 옵션 B (samples/aift.pdf 영역 제외)

**사유**:
1. **본 환경 PDF 명명 규약 영역 정합** — `pdf/{stem}-2022.pdf` 영역 (PR #670 패턴) 정합. PR 의 `samples/aift.pdf` 영역은 다른 SHA 영역 (다른 PDF) → 영구 영역 충돌 회피
2. **본 환경 결정적 검증 모두 통과** — cargo test 1156 / svg_snapshot 7/7 / issue_546/554/418/501 통과 / clippy 0
3. **권위 영역 100% 일치** — PR 본문 측정과 본 환경 측정 정확 일치 (lines=0..2 / lines=2..4 / [vpos-reset@line2] 마커)
4. **PR #622 영역의 vpos-reset 가드 영역 패턴 정합** — 본 사이클의 vpos-reset 영역 권위 사례 누적 영역
5. **광범위 회귀 검증 영역 정합** — 컨트리뷰터의 155 샘플 / 1,255 페이지 + 본 환경 결정적 검증 일치 영역

### 옵션 영역 요약 표

| 옵션 | 진행 가능 | Issue #631 정정 | 결정적 검증 | samples/aift.pdf 영역 | 권장 |
|------|----------|----------------|------------|-------------------|------|
| **A** (전체 6 commits) | ✅ 충돌 1 (orders ours) | ✅ | ✅ 1156/7/12 | ⚠️ 명명 규약 충돌 | ❌ |
| **B** (src + 거버넌스 5 commits) | ✅ | ✅ | ✅ 동일 | ✅ 제외 | ⭐ |

---

## 7. 잠정 결정

### 권장 결정
- **옵션 B 진행** — 5 commits cherry-pick (PDF commit 제외)
- 본 환경 결정적 검증 + WASM 빌드 + rhwp-studio public 갱신 + 시각 판정 ★

### 검증 영역 (옵션 B 진행 시 본 환경 직접 점검)
1. cherry-pick (5 commits) — orders 충돌 ours 보존
2. `cargo test --lib --release` 1156 passed
3. `cargo test --test svg_snapshot / issue_546 / issue_554 / issue_418 / issue_501` 통과
4. `cargo clippy --lib -- -D warnings` 0
5. Docker WASM 빌드 + byte 측정
6. `rhwp-studio/public/{rhwp_bg.wasm, rhwp.js}` 영역 갱신
7. **시각 판정 ★** — `samples/aift.hwp` page 18/19 영역의 pi=222 영역 작업지시자 시각 판정 (본 환경 `pdf/aift-2022.pdf` PR #670 영구 보존 영역 비교)

---

## 8. 메모리 룰 관점

- **`feedback_hancom_compat_specific_over_general` 권위 사례 강화 누적** — HWP 권위값 더블체크 영역은 구조적 판단 (`vertical_pos == 0` + `vpos+lh ≤ body_h` 영역, 측정 의존 없음). PR #621 (다중 줄 가드) + PR #622 (다단 vpos-reset) + 본 PR (vpos-reset 인접 line 보존) = 본 사이클의 권위 사례 누적
- `reference_authoritative_hancom` — `pdf/aift-2022.pdf` (PR #670 영구 보존 영역) 권위 정답지 영역 비교 영역
- `feedback_pdf_not_authoritative` (5/7 갱신) — 한글 2020/2022 PDF 정답지 영역 정합
- `feedback_close_issue_verify_merged` — Issue #631 close 시 본 PR 머지 검증 + 수동 close
- `feedback_assign_issue_before_work` — Issue #631 assignee 미지정 영역
- PR #670 패턴 정합 — `pdf/aift-2022.pdf` 영역 (한글 2022) vs `samples/aift.pdf` 영역 (PR 추가 영역, 다른 SHA) 의 영역 충돌 영역

---

## 9. 다음 단계 (CLAUDE.md PR 처리 4단계)

1. (완료) PR 정보 + Issue 연결 + base/head + mergeable + CI 상태 확인
2. (현재) `pr_632_review.md` 작성 → 승인 요청
3. (필요 시) `pr_632_review_impl.md` 작성 → 승인 요청
4. 검증 (빌드/테스트/clippy + 시각 판정 ★) + 판단 → `pr_632_report.md` 작성

### 작업지시자 결정 요청
1. **옵션 결정** — 옵션 A (전체 6 commits, samples/aift.pdf 포함) / 옵션 B (5 commits, samples/aift.pdf 제외, **권장**)
2. **시각 판정 권위 영역** — `samples/aift.hwp` page 18 / 19 영역 작업지시자 직접 시각 판정 진행 가/부
3. **WASM 빌드 + rhwp-studio public 갱신** 가/부

결정 후 본 환경 cherry-pick + 결정적 검증 + WASM 빌드 + 시각 판정 ★ + `pr_632_report.md` 작성.
