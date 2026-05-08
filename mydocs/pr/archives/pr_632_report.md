# PR #632 처리 보고서

## 1. 처리 결과

| 항목 | 값 |
|------|-----|
| PR | #632 — Task #631 HWP 권위값 더블체크로 vpos-reset 인접 line 보존 |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu / Jaeook Ryu) — 8번째 사이클 PR |
| 연결 이슈 | #631 (closed) |
| 처리 옵션 | 옵션 B — 5 commits cherry-pick (samples/aift.pdf 제외) |
| devel commits | `51c22a6` Stage 1 + `7127ded` Stage 2 구현계획 + `5fdc096` Stage 2 정정 + `e415f62` Stage 3 + `e098562` Stage 4 |
| 처리 일자 | 2026-05-07 |

## 2. cherry-pick 결과

5 commits 단계별 보존 (author Jaeook Ryu, committer edward):

| Stage | hash | 변경 |
|-------|------|------|
| Stage 1 | `51c22a6` | 정밀 진단 (pi=222 LINE_SEG vpos-reset 미활용 + 누적 drift 의심) |
| Stage 2 구현계획 | `7127ded` | LAYOUT_DRIFT_SAFETY_PX 이중 차감 + HWP 권위값 더블체크 영역 |
| Stage 2 정정 | `5fdc096` | typeset.rs partial-split 루프에 HWP 권위값 더블체크 추가 (+18/-1) |
| Stage 3 | `e415f62` | 광범위 회귀 검증 (155 샘플 / 1,255 페이지 / 회귀 0) |
| Stage 4 | `e098562` | 최종 보고서 |

### 제외 commit
- `2ad3bfa` (samples/aift.pdf 추가) 영역 제외 — 본 환경 `pdf/aift-2022.pdf` (PR #670 영구 보존 영역) 영역과 충돌 영역 (다른 SHA, 다른 PDF)

### 충돌 처리
- `mydocs/orders/20260506.md` add/add 충돌 (Stage 4) → `git checkout --ours` 본 환경 영역 보존 정합 (PR #622/#627 패턴)

## 3. 본 환경 결정적 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1156 passed** (회귀 0) |
| `cargo test --test svg_snapshot --release` | ✅ 7/7 |
| `cargo test --test issue_546 --release` | ✅ 1/1 |
| `cargo test --test issue_554 --release` | ✅ 12/12 |
| `cargo test --test issue_418 --release` | ✅ 1/1 |
| `cargo test --test issue_501 --release` | ✅ 1/1 |
| `cargo clippy --lib --release -- -D warnings` | ✅ 0 |
| Docker WASM 빌드 | ✅ **4,577,370 bytes** (PR #627 baseline 4,578,751 -1,381 bytes) |
| `rhwp-studio/public/{rhwp_bg.wasm, rhwp.js}` | ✅ 갱신 (vite dev server web 영역) |

## 4. 권위 영역 직접 측정 (PR 본문 100% 일치)

| 항목 | PR 본문 | 본 환경 측정 |
|------|---------|--------------|
| 페이지 18 pi=222 | `lines=0..2` | **`lines=0..2 vpos=67980..0 [vpos-reset@line2]`** ✓ |
| 페이지 19 pi=222 | `lines=2..4` | **`lines=2..4 vpos=0..1920 [vpos-reset@line2]`** ✓ |

dump-pages 출력의 `[vpos-reset@line2]` 마커 본 환경 직접 재현.

## 5. 본질 정정 영역

### 5.1 본질 정정 (단 1곳, +18/-1)

`src/renderer/typeset.rs::typeset_paragraph` 줄 단위 분할 루프 (line 1078-1098):

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

### 5.2 조건 영역 (모두 AND)
1. typeset 누적 추정 영역으로 fit 실패 (기존 break 조건)
2. **다음 줄 vpos == 0** — HWP 가 명시적으로 페이지 경계 신호 인코딩
3. **현재 줄 vpos+lh ≤ body_available_height** — HWP 좌표상 본문 안

→ 조건 2 영역이 매우 좁아 회귀 위험 영역 최소.

### 5.3 회귀 발생 원인 (PR 본문)
- `LAYOUT_DRIFT_SAFETY_PX = 10.0` 영역이 두 곳에서 차감 → 합계 20 px 보수 마진
- `pi=222` 진입 시점: `cumulative + content_h = 41.6 > avail_for_lines = 39.2` → break (line 1 탈락)
- 20 px 보수 마진 영역이 본문 잔여 59.2px → 39.2px 영역 축소시켜 2줄 합 (41.6) 영역 수용 안 됨

### 5.4 Task #332 stage4b 영역 종결
PR 본문: "Task #332 stage4b 커밋 (`0211e574`) 영역에서 명시적으로 알려진 회귀 — `aift pi=222`, `21_언어 pi=10 line 1`, `hwp-multi-002 pi=68` 가운데 마지막 미해결분."

→ 본 PR 영역으로 **알려진 회귀 영역의 종결 영역** 정합.

## 6. 컨트리뷰터의 광범위 회귀 검증 (PR 본문)

155 샘플 / 1,255 페이지:

| 카테고리 | 샘플 | 효과 |
|----------|------|------|
| 의도된 수정 | aift, 20250130-hongbo(-no), hwp3-sample4/5 | page 끝 1~수 줄 페이지 내 복원 |
| 회복 — 페이지 통합 | loading-fail-01 | 17→16 페이지 |
| 회복 — 누락 콘텐츠 복구 | hwpctl_API_v2.4 | +2 페이지, **+460 text elements** |
| 회복 — 누락 콘텐츠 복구 | hwpspec | +1 페이지, **+855 text elements** |
| 변화 없음 (147 샘플) | exam_kor/eng/math/science 등 | byte 동일 |

**부정적 회귀 0건. 1,300+ 누락 콘텐츠 복구.**

## 7. devel 머지 + push

### 진행
1. `git cherry-pick b6efbf7 898527d b7cc2d4 8f60ae6 86fba08` (5 commits)
2. Stage 4 commit 영역에서 `mydocs/orders/20260506.md` add/add 충돌 → `git checkout --ours` 본 환경 보존
3. devel ← local/devel ff merge
4. push: `ff13a08..e098562`

### 분기 처리
- 본 cherry-pick 시점 origin/devel 분기 0 — `feedback_release_sync_check` 정합

## 8. PR / Issue close

- PR #632: 한글 댓글 등록 + close (`gh pr close 632`)
- Issue #631: 한글 댓글 등록 + close (`gh issue close 631`)

## 9. 시각 판정 영역 스킵 (작업지시자 결정)

작업지시자 결정 — 시각 판정 영역 스킵 (결정적 검증 + 권위 영역 100% 일치 영역 통과로):
- 권위 영역 100% 일치 (PR 본문 명세 정확 재현)
- 컨트리뷰터의 155 샘플 / 1,255 페이지 광범위 sweep / 회귀 0건 검증
- HWP 권위값 더블체크 영역의 구조적 판단 (측정 의존 없음)

## 10. 메모리 룰 적용

- **`feedback_hancom_compat_specific_over_general` 권위 사례 강화 누적** — 본 사이클의 vpos-reset 영역 권위 사례 누적 영역:
  - PR #621 (다중 줄 가드) — `paragraphs.iter().any(|p| p.line_segs.len() >= 2)`
  - PR #622 (다단 vpos-reset) — `col_count > 1 && li > cursor_line && vertical_pos == 0`
  - **본 PR (vpos-reset 인접 line 보존)** — 다음 줄 `vertical_pos == 0` + 현재 줄 `vpos+lh ≤ body_h` 더블체크
- `reference_authoritative_hancom` — `pdf/aift-2022.pdf` (PR #670 영구 보존 영역) 영역 정합
- `feedback_pdf_not_authoritative` (5/7 갱신) — 한글 2020/2022 PDF 정답지 영역 정합
- `feedback_close_issue_verify_merged` — Issue #631 close 시 본 PR 머지 검증 + 수동 close
- `feedback_assign_issue_before_work` — Issue #631 assignee 미지정 영역
- PR #670 패턴 정합 — `pdf/aift-2022.pdf` 영역의 본 환경 명명 규약 영역 정합 (PR 의 `samples/aift.pdf` 영역 제외 영역)

## 11. 본 사이클 (5/7) PR 처리 누적 — **13건**

| # | PR | Task / Issue | 결과 |
|---|-----|--------------|------|
| 1 | PR #620 | Task #618 (Picture flip/rotation, Task #519 누락) | 시각 판정 ★ + close |
| 2 | PR #642 | Task #598 (각주 마커) | 시각 판정 ★ + close |
| 3 | PR #601 | Task #594 (복수 제목행) | 옵션 A-2 + close + Issue #652 신규 |
| 4 | PR #659 | Task #653 (ir-diff 표 속성) | 시각 판정 ★ + close |
| 5 | PR #602 | Issue #449 (rhwpDev) | close + Issue #449 reopen |
| 6 | PR #668 | Task #660 (Neumann ingest) | 첫 PR + 시각 판정 ★ + close |
| 7 | PR #609 | Task #604 (Document IR) | 11 commits 단계별 + 시각 판정 ★ + close |
| 8 | PR #670 | (이슈 미연결) 한글 2022 PDF 199 | 메모리 룰 갱신 + close |
| 9 | PR #621 | Task #617 (표 셀 padding) | 옵션 B + 시각 판정 ★ + close |
| 10 | PR #622 | Task #619 (다단 vpos-reset) | 옵션 A + web editor 시각 판정 ★ + close |
| 11 | PR #626 | (Follow-up to #599) 수식 replay | 옵션 A + PNG 시각 판정 ★ + close |
| 12 | PR #627 | Task #624 (Task #520 누락 회귀) | 옵션 A + TDD RED→GREEN + web editor 시각 판정 ★ + close |
| 13 | **PR #632** | **Task #631 (vpos-reset 인접 line 보존)** | **옵션 B + 결정적 검증 통과 + 시각 판정 스킵 + close** |

### 본 사이클의 vpos-reset 영역 권위 사례 누적 영역
- PR #621 (다중 줄 가드)
- PR #622 (다단 vpos-reset)
- PR #632 (vpos-reset 인접 line 보존)

→ 모두 구조적 판단 영역 (측정 의존 없음) + HWP 인코딩 신호 영역 존중 영역.

### 본 사이클의 권위 사례 누적 영역
- **`feedback_hancom_compat_specific_over_general`**: PR #621 + PR #622 + **PR #632** (vpos-reset 영역 누적)
- **`feedback_close_issue_verify_merged`**: PR #620 + PR #627 (PR cherry-pick base diff 점검 누락 패턴)

본 PR 의 **HWP 권위값 더블체크 (구조적 가드) + 광범위 회귀 검증 (1,300+ 누락 콘텐츠 복구) + 본 환경 명명 규약 (m100 + samples/aift.pdf 제외) 정합 + 권위 영역 100% 일치 + Task #332 stage4b 알려진 회귀 영역 종결 + `feedback_hancom_compat_specific_over_general` 권위 사례 강화 누적 패턴 모두 정합**.
