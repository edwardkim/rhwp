# PR #622 처리 보고서

## 1. 처리 결과

| 항목 | 값 |
|------|-----|
| PR | #622 — 다단 paragraph 내 vpos-reset 미처리 정정 (closes #619) |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu / Jaeook Ryu) — 6번째 사이클 PR (PR #629/#620/#578/#670/#621 직후) |
| 연결 이슈 | #619 (closed) |
| 처리 옵션 | 옵션 A — 3 commits 단계별 cherry-pick |
| devel commits | `20e660c` (Stage 1) + `a7eb171` (Stage 2/3) + `2d20cc0` (Stage 4 보고서) |
| 처리 일자 | 2026-05-07 |

## 2. cherry-pick 결과

3 commits 단계별 보존 (author Jaeook Ryu, committer edward):

| Stage | hash | 변경 |
|-------|------|------|
| Stage 1 | `20e660c` | TypesetEngine partial-split 다단 vpos-reset forced break (src/renderer/typeset.rs +10 LOC) + 수행/구현 계획서 + Stage 1 보고서 |
| Stage 2/3 | `a7eb171` | 한컴 2010/2020 PDF 비교 + 회귀 가드 샘플 10개 byte-equivalent 검증 (Stage 2/3 보고서) |
| Stage 4 | `2d20cc0` | 최종 보고서 (orders add/add 충돌 → 본 환경 영역 보존 정합) |

### 충돌 처리
- `mydocs/orders/20260506.md` add/add 충돌 영역 발생 — 본 환경의 5/6 orders 영역과 PR 의 5/6 orders 영역 동시 영역
- 본 환경 영역 보존 (`git checkout --ours`) → 본 환경의 PR #578/#629/#611/#620/#642 처리 영역 누적 영역 정합
- src/renderer/typeset.rs 영역은 충돌 부재 (auto-merge 통과)

## 3. 본 환경 결정적 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1155 passed** (회귀 0) |
| `cargo test --test svg_snapshot --release` | ✅ 7/7 |
| `cargo test --test issue_546 --release` | ✅ 1/1 |
| `cargo test --test issue_554 --release` | ✅ 12/12 |
| `cargo test --test issue_418 --release` | ✅ **1/1** (단일 단 partial-table 회귀 차단 영역 정합) |
| `cargo test --test issue_501 --release` | ✅ 1/1 |
| `cargo clippy --lib -- -D warnings` | ✅ 0 |
| Docker WASM 빌드 | ✅ **4,578,641 bytes** (PR #621 baseline 4,606,564 **-27,923 bytes**) |
| `rhwp-studio/public/{rhwp_bg.wasm, rhwp.js}` | ✅ 갱신 (vite dev server web 영역) |

## 4. 권위 영역 직접 측정 (PR 본문 표 100% 일치)

| 항목 | PR 본문 | 본 환경 측정 |
|------|---------|--------------|
| 페이지 8 단 1 pi=181 | `lines=0..8` | **`lines=0..8 vpos=77316..0 [vpos-reset@line8]`** ✓ |
| 페이지 9 단 0 pi=181 | `lines=8..13` | **`lines=8..13 vpos=0..7264 [vpos-reset@line8]`** ✓ |
| LAYOUT_OVERFLOW_DRAW 17.1px | 사라짐 | **부재** ✓ |

dump-pages 출력의 `[vpos-reset@line8]` 마커 영역 본 환경 직접 재현 영역.

## 5. 본질 정정 영역

### 다단 한정 구조적 가드 (단 1곳, +10 LOC)
```rust
// src/renderer/typeset.rs::typeset_paragraph partial-split 루프 (line 1077-1093)
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

### 본질
- `pi=181 line_segs[8].vertical_pos = 0` — HWP 가 line 8 을 다음 단 / 페이지 최상단에서 시작하도록 인코딩한 vpos-reset 신호
- 활성 `TypesetEngine` partial-split 루프가 문단 *내부* line.vertical_pos=0 신호를 인식하지 않음 → 17.1 px overflow 발생
- 다단 한정 가드 추가 → HWP 의 인코딩 의도 정합 영역 도달

### `feedback_hancom_compat_specific_over_general` 권위 사례 강화
- 다단 한정 가드는 **구조적 판단** (`col_count > 1` + `li > cursor_line` + `vertical_pos == 0`) — 측정값 의존 없음
- 본 사이클의 권위 사례 누적: PR #621 (다중 줄 가드, line_segs.len() >= 2) + 본 PR (다단 vpos-reset, col_count > 1) — 모두 구조적 판단 영역의 패턴 정합

## 6. 메인테이너 시각 판정 ★ 통과 (web editor 영역)

권위 영역 — `samples/21_언어_기출_편집가능본.hwp` (한컴 2020 PDF 권위 정답지 정합):
- **페이지 8 우측 단 마지막 줄** — line 8 정상 분할 (col_bottom 너머 부재)
- **페이지 9 좌측 단 첫 줄** — line 8 정상 배치 (vpos-reset 영역 정합)

작업지시자 평가: "웹 에디터에서 시각 판정 통과되었습니다."

## 7. 회귀 가드 영역

### 컨트리뷰터의 회귀 가드 sweep (PR 본문 §Verification)
10개 회귀 가드 샘플 LAYOUT_OVERFLOW + 페이지 분포 변경 전후 완전 동일:
- exam_eng / exam_kor / exam_science / exam_math / exam_social
- hwp-multi-001 / hwp-multi-002
- k-water-rfp / kps-ai / aift

### 본 환경 결정적 검증 (회귀 차단 영역 정합)
- `issue_418` (단일 단 partial-table split 영역) 통과 → 다단 한정 가드 (`col_count > 1`) 영역 정합
- `issue_554` 12/12 (`task554_no_regression_exam_kor` / `_aift` 등 통과)

## 8. devel 머지 + push

### 진행
1. `git cherry-pick 9c5a187 b5d0abf 6f4a5b2` (3 commits)
2. 3번째 commit 영역에서 `mydocs/orders/20260506.md` add/add 충돌 → `git checkout --ours` 영역 + `git cherry-pick --continue` 정합
3. devel ← local/devel ff merge
4. push: `d58217b..2d20cc0`

### 분기 처리
- 본 cherry-pick 시점 origin/devel 분기 0 — `feedback_release_sync_check` 정합

## 9. PR / Issue close

- PR #622: 한글 댓글 등록 + close (`gh pr close 622`)
- Issue #619: 한글 댓글 등록 + close (`gh issue close 619`)

> `closes #619` 키워드 영역은 cherry-pick merge 영역으로 자동 처리 안 됨 — 수동 close 진행. `feedback_close_issue_verify_merged` 정합.

## 10. 잔여 영역 (PR 본문 §Remaining)

- **PartialParagraph + Shape bbox 2.4 px overflow** (page=7 col=1 pi=181) — 텍스트 자체는 단 안, bbox 영역만 잔여
- **본 PR 영역 외**, 별도 이슈 분리 후보 영역 (PR 본문 명시)

## 11. 메모리 룰 적용

- `feedback_hancom_compat_specific_over_general` — **권위 사례 강화 누적**. 다단 한정 가드 (`col_count > 1` + `li > cursor_line` + `vertical_pos == 0`) 는 구조적 판단 (측정 의존 없음). PR #621 의 다중 줄 가드 + 본 PR 의 다단 vpos-reset 가드 = 본 사이클의 권위 사례 누적 영역
- `reference_authoritative_hancom` — 한컴 2020 PDF (8줄) = 정답 / 한컴 2010 PDF (5줄) = 부정확 → 본 PR 의 한컴 2020 정합 정합. `pdf/21_언어_기출_편집가능본-2022.pdf` (PR #670 영구 보존 영역) 영역 정합
- `feedback_pdf_not_authoritative` (5/7 갱신) — 한글 2020/2022 PDF 정답지 영역 정합. 한컴 2010 PDF 영역의 등급 미달 영역 (이슈 #345 정합) 의 권위 사례 강화
- `feedback_close_issue_verify_merged` — Issue #619 close 시 본 PR 머지 검증 + 수동 close
- `feedback_assign_issue_before_work` — Issue #619 assignee 미지정 영역 (외부 컨트리뷰터 자기 등록 사례)

## 12. 본 사이클 (5/7) PR 처리 누적 — **10건**

| # | PR | Task / Issue | 결과 |
|---|-----|--------------|------|
| 1 | PR #620 | Task #618 (Picture flip/rotation) | 시각 판정 ★ + close |
| 2 | PR #642 | Task #598 (각주 마커) | 시각 판정 ★ + close |
| 3 | PR #601 | Task #594 (복수 제목행) | 옵션 A-2 + close + Issue #652 신규 |
| 4 | PR #659 | Task #653 (ir-diff 표 속성) | 시각 판정 ★ + close |
| 5 | PR #602 | Issue #449 (rhwpDev) | close + Issue #449 reopen |
| 6 | PR #668 | Task #660 (Neumann ingest) | 첫 PR + 시각 판정 ★ + close + 후속 #665/#666/#667 |
| 7 | PR #609 | Task #604 (Document IR) | 11 commits 단계별 + 시각 판정 ★ + close |
| 8 | PR #670 | (이슈 미연결) 한글 2022 PDF 199 | 옵션 D 변형 + 메모리 룰 갱신 + close |
| 9 | PR #621 | Task #617 (표 셀 padding) | 옵션 B + 시각 판정 ★ + close |
| 10 | **PR #622** | **Task #619 (다단 vpos-reset)** | **옵션 A + web editor 시각 판정 ★ + close** |

### 본 사이클의 메모리 룰 권위 사례 강화
- `feedback_hancom_compat_specific_over_general` — PR #621 (다중 줄 가드) + PR #622 (다단 vpos-reset 가드) = 구조적 판단 영역의 누적 정합

본 PR 의 **다단 한정 구조적 가드 + 회귀 가드 영구 보존 + 본 환경 명명 규약 (m100) 정합 + 권위 영역 100% 일치 + 메인테이너 web editor 시각 판정 ★ 통과 + `feedback_hancom_compat_specific_over_general` 권위 사례 강화 패턴 모두 정합**.
