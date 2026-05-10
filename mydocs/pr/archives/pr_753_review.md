---
PR: #753
제목: Task #741 — hwp3-sample10.hwp HWP3 외부 file path + ParaShape tab + 사적 char + 제목차례 장식 + 차례 page 번호 정합 (closes #741)
컨트리뷰터: @jangster77 (Taesup Jang) — HWP3 핵심 컨트리뷰터 (5/10 사이클 17번째 PR — PR #732 후속)
base / head: devel / local/task741
mergeStateStatus: BLOCKED (Build & Test 진행 중)
mergeable: MERGEABLE
변경 규모: +2097 / -44, 42 files (대형 PR)
검토일: 2026-05-10
---

# PR #753 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #753 |
| 제목 | Task #741 — hwp3-sample10.hwp HWP3 native 렌더링 8 단계 정정 |
| 컨트리뷰터 | @jangster77 (Taesup Jang) — HWP3 파서 핵심 |
| 사이클 | 16+ 사이클 (PR #451/#486/#487/#499/#506/#553/#556/#589/#605/#609/#673/#675/#678/#723/#732/#753) |
| base / head | devel / local/task741 |
| mergeStateStatus | BLOCKED, mergeable: MERGEABLE |
| 변경 규모 | **+2097 / -44, 42 files** (대형 PR) |
| 커밋 수 | Task #741 본질 commits 8개 + 후속 6개 + Task #722/#724 (PR #732 영역 영역 머지 완료, 자동 제외) |
| closes | #741 |

## 2. 결함 본질 (Issue #741)

`hwp3-sample10.hwp` (Oracle 기술 문서, 763 페이지) HWP3 native 렌더링 결함 8개 영역:
- 페이지 1: image placeholder 미표시
- 페이지 2: 제목차례 자동 장식 부재 + ▷/■/페이지 번호 미표시
- 페이지 28~339: 차례 inline page 번호 (ch=1) 미표시

한컴 viewer + PDF (한글 2022) 영역 정합 도달 목표.

## 3. 단계별 영역

| Stage | 영역 | commit |
|-------|------|--------|
| 1~4 | HWP3 외부 file path 그림 IR (`ImageAttr.external_path`) + image placeholder + TAC 그림 paragraph line_spacing 정합 (ls=600) + HWP5 변환본 paragraph 26 페이지 분할 정합 (vpos-reset 후속 가드) | `a63114e6` |
| 5 | HWP3 사적 graphic char (0x0080~0x7FFF) cross-ref 매핑 — 표준 KSSM (0x8000+) 외 한컴 사적 인코딩. 상위 6값 = 98.5% coverage | `86bf0bdc` |
| 6 | HWP3 ParaShape tabs[40] → Document IR TabDef 변환 + `Hwp3TabDef` 필드 순서 bug 정정 (**30+ 사이클 미발견 본질 결함**) | `ccbb0b6c` |
| 7 | HWP3 leader → HWP5 fill_type 매핑 (1→3 점선) + 제목차례 자동 장식 inject (한컴 사적 로직 cross-ref) + char_shape 위치 정정 | `d03109eb` + `b77d071b` |
| 8 | HWP3 차례 inline page 번호 (ch=1) decode + display | `6f09ece6` |
| 후속 | 외부 file path 그림 영역 native CLI 자동 load + WASM API + Vite middleware + rhwp-studio web canvas + dialog 정정 | `cf2ea841` + `1a3ca416` + `647c508d` |
| CI fix | `tests/issue_516.rs` ImageAttr external_path 누락 정정 | `37776144` |

### 3.1 핵심 본질 발견

**Stage 5 — HWP3 사적 인코딩**
HWP3 hchar 0x0080~0x7FFF 영역 영역 표준 KSSM 조합형 외 한컴 사적 인코딩. johab decoder 가 '?' 반환 → 가로선/▷/■ 누락. 한컴 변환본 cross-ref 영역 매핑 도출 (상위 6값 = 98.5% coverage).

**Stage 6 — Hwp3TabDef 필드 순서 bug**
기존 `(position:u16, type:u8, leader:u8)` — 실제 byte stream 어긋남. `HWP3_DIAG_TABS` 진단으로 default tab pattern 검증 → **`(tab_type:u8, leader:u8, position:u16 LE)`** 로 정정. 30+ 사이클 미발견 본질 결함.

**Stage 7 — 한컴 viewer 자동 장식**
HWP3 paragraph 26 (cc=8 "￼￼ 제목차례 ") → 한컴 viewer 영역 영역 "════════════════════■ 제목차례 ■═════════════════" 자동 장식 inject. **HWP3 spec 외 한컴 사적 로직** (ParaShape `border` / char_shape `attr` 모두 부재 확정). 한컴 변환본 cross-ref 영역 trigger 조건 도출 (새번호 + 쪽번호위치 controls + visible text ≤ 6 chars). 보수적 영역으로 광범위 sweep 회귀 위험 최소화.

**Stage 8 — 차례 inline page 번호 (ch=1)**
HWP3 차례 entries (paragraph 28~339) 가 페이지 번호를 inline `ch=1` control 로 저장. `HWP3_DIAG_CTRL1` 진단으로 byte 패턴 도출 — header_val1 second u16 + ch2 = ASCII digit. 290 ch=1 occurrences 모두 정합.

## 4. 정정 영역 — 42 files

### 4.1 Rust 핵심 (HWP3 파서 + 렌더러)

| 파일 | 변경 |
|------|------|
| `src/parser/hwp3/mod.rs` | +207/-... Stage 5/6/7/8 종합 |
| `src/parser/hwp3/records.rs` | +5/-... Stage 6 (Hwp3TabDef 필드 순서) |
| `src/parser/hwp3/drawing.rs` | +14/-... Stage 6 (signature 확장) |
| `src/parser/hwp3/johab.rs` | +29/-... Stage 5 (사적 인코딩 매핑) |
| `src/model/image.rs` | +7/-... Stage 1~4 (ImageAttr.external_path) |
| `src/renderer/render_tree.rs`, `svg.rs`, `layout/picture_footnote.rs`, `layout/table_cell_content.rs`, `typeset.rs` | Stage 1~4 (image placeholder + vpos-reset 후속 가드) |
| `src/wasm_api.rs` | +131 Stage 후속 (외부 그림 native CLI 자동 load + WASM API) |
| `src/main.rs` | +17 외부 그림 자동 load |
| `src/document_core/commands/document.rs` | +18 |
| `src/model/document.rs` | +89 |
| `src/renderer/web_canvas.rs` | +21 외부 그림 web canvas |

### 4.2 rhwp-studio (외부 그림 영역)

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/core/wasm-bridge.ts` | +40 외부 그림 WASM 바인딩 |
| `rhwp-studio/src/ui/picture-props-dialog.ts` | +39 외부 그림 dialog |
| `rhwp-studio/vite.config.ts` | +38 외부 그림 Vite middleware |
| `rhwp-studio/src/core/types.ts` | +2 |

### 4.3 자료 추가 (대용량)

| 경로 | 크기 | 용도 |
|------|------|------|
| `samples/hwp3-sample10.hwp` | 945 KB | HWP3 native sample |
| `samples/hwp3-sample10-hwp5.hwp` | 1.1 MB | 한컴 HWP5 변환본 (cross-ref 권위) |
| `samples/hwp3-sample10-hwpx.hwpx` | 847 KB | 한컴 HWPX 변환본 |
| `samples/oracle.gif` | 1.8 KB | 외부 file path 그림 fixture |
| `samples/rdb02.gif` | 6 KB | 외부 그림 fixture |
| `samples/s1.jpg` | 17 KB | 외부 그림 fixture |
| `pdf/hwp3-sample10-hwp5-2022.pdf` | **96 MB** | 한글 2022 PDF (1-up 인쇄) |

### 4.4 문서 자료

mydocs 영역 영역 plans (Task #722 / #724 / #741 영역) + working (Stage 1~8) + report (Task #741 final) 추가.

## 5. ⚠️ PDF 96 MB 영역 영역 작업지시자 결정 필요

### 5.1 본질
- `pdf/hwp3-sample10-hwp5-2022.pdf` 96 MB — **GitHub 권장 50 MB 초과**
- PR 본문 영역 영역 90 MB 명시 (Stage 5 머지 시점, 후속 Stage 영역 영역 96 MB 누적)
- Git LFS 미사용 영역 영역 git history 누적

### 5.2 옵션
1. **머지 진행 (현 상태)** — PR 본문 영역 영역 명시된 의도 반영. Git history 영역 영역 96 MB 누적
2. **PDF 분리 후 머지** — 컨트리뷰터에게 PDF 분리 요청 (gist / 외부 호스팅 / Git LFS 영역) — 시간 소요
3. **머지 + 별 후속 PR 영역 PDF 정리** — Git LFS migration 영역 별 작업

> `feedback_pdf_not_authoritative` 정합 — 한글 2022 PDF 영역 영역 정답지 등급 ✅ 정합. 그러나 **저장소 크기 영역 영역 별 본질**.

### 5.3 본 환경 추정 권장
**옵션 1 (머지 진행)** 권장 — PR 본문 명시 + 한글 2022 PDF 정답지 등급 ✅ + 작업지시자 시각 판정 ★ 명시. 향후 Git LFS migration 영역 별 후속.

## 6. 충돌 / mergeable

mergeStateStatus = `BLOCKED` (Build & Test 진행 중), mergeable = `MERGEABLE`.

본 환경 점검:
- PR #732 영역 영역 5/10 머지 완료 영역 영역 Task #722/#724 commits 영역 영역 자동 제외 (PR HEAD ↔ devel 영역 영역 의 `git diff` 영역 영역 Task #741 본질만 남음)
- typeset.rs 영역 영역 PR #732 후속 (`25299a5b`) + 본 PR `ab2fa527` 동일 변경 — 두 commit 영역 영역 동일 본질 영역 영역 본 PR 영역 영역 ab2fa527 영역 영역 cherry-pick 시 empty 또는 자동 정합 (squash 영역 영역 영역 자동 영역 영역 차이 흡수)

## 7. 처리 옵션

본 PR 영역 영역 commits 누적 (Task #722/#724 + Task #741 본질 + 후속 + merge commits) 영역 영역 개별 cherry-pick 어려움.

### 옵션 A — squash cherry-pick (PR HEAD vs devel 영역 영역 차이만 단일 commit)

```bash
git checkout local/devel
git cherry-pick --no-commit ab46d320..pr753-head
git commit -m "Task #741: hwp3-sample10.hwp HWP3 native 렌더링 8 단계 정정 (closes #741)"
```

→ Task #741 본질 commits 의 분리 보존 부재. 컨트리뷰터 author 보존을 위해 옵션 B 권장.

### 옵션 B — Task #741 commits 만 개별 cherry-pick (권장)

```bash
git checkout local/devel
git cherry-pick a63114e6 86bf0bdc ccbb0b6c d03109eb b77d071b 6f09ece6 \
                ab2fa527 cf2ea841 1a3ca416 647c508d 37776144
```

→ Stage 1~4/5/6/7/7후속/8 + 후속 + CI fix 영역 영역 author 보존.
→ `ab2fa527` 영역 영역 typeset.rs PR #732 후속 영역 영역 devel 영역 영역 이미 적용 (`25299a5b`) → empty cherry-pick 가능 → `--allow-empty` skip.

→ **권장**.

## 8. 검증 게이트

### 8.1 자기 검증
- [ ] cherry-pick 충돌 점검 (typeset.rs `ab2fa527` 영역 영역 empty 처리)
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release` ALL GREEN (PR 본문 영역 영역 1166 통과 명시)
- [ ] `cargo clippy --release --lib` 신규 경고 0
- [ ] `cd rhwp-studio && npx tsc --noEmit` 통과
- [ ] **광범위 sweep — 7 fixture / 170 페이지 / 회귀 0** ✅ (PR 본문 영역 영역 8회 단계별 검증 명시)
- [ ] hwp3-sample10 native sweep 추가 (763 페이지) — 결정적 검증

### 8.2 시각 판정 게이트 — **★ 작업지시자 시각 판정 권위**

본 PR 본질은 **한컴 viewer + PDF (한글 2022) 영역 정합** (`feedback_pdf_not_authoritative` 정합 — 한글 2022 PDF 영역 정답지 등급 ✅).

작업지시자 시각 판정 항목:
- 페이지 1: image placeholder 표시 (HWP3 native + HWP5 변환본)
- 페이지 2: "════════════════════■ 제목차례 ■══════════════════════" 자동 장식 + ▷ markers + 점선 leader + 페이지 번호 (1, 4, 5, ..., 134)
- 페이지 28~339: 차례 inline page 번호 정합

> WASM 빌드 + native SVG export 영역 영역 시각 판정 권장.

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 16+ 사이클 (HWP3 핵심) |
| `feedback_image_renderer_paths_separate` | image placeholder 영역 영역 다중 경로 (svg.rs / picture_footnote / table_cell_content / web_canvas) 동기 정정 — **권위 사례** |
| `feedback_process_must_follow` | Task #722/#724 + #741 단계별 분리 (Stage 1~8) — 위험 좁힘 + 단계별 self-review |
| `feedback_hancom_compat_specific_over_general` | Stage 5 사적 char 매핑 (상위 6값) + Stage 7 제목차례 trigger 조건 (visible text ≤ 6) — 일반화 영역 영역 회귀 위험 좁힘 |
| `feedback_pdf_not_authoritative` | 한글 2022 PDF 정답지 등급 ✅ 정합 |
| `reference_authoritative_hancom` | 한컴 viewer + PDF cross-ref 영역 영역 권위 자료 |
| `feedback_visual_judgment_authority` | 작업지시자 한컴 viewer + PDF 정합 시각 판정 ★ 권위 |
| `feedback_visual_regression_grows` | 광범위 sweep 170/170 same 8회 단계별 검증 — 시각 회귀 검출 강화 |
| `feedback_self_verification_not_hancom` | rhwp 자기 라운드트립 통과 + 한컴 변환본 cross-ref 영역 영역 정답지 정합 입증 |

## 10. 처리 순서 (승인 후)

1. PDF 96 MB 영역 영역 작업지시자 결정 확정
2. `local/devel` 영역 영역 옵션 B (Task #741 commits 개별 cherry-pick + author 보존)
3. 자기 검증 (cargo test + clippy + tsc + 광범위 sweep + hwp3-sample10 추가 검증)
4. WASM 빌드 + 작업지시자 시각 판정 (한컴 viewer + PDF 정합 ★)
5. 시각 판정 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
6. PR #753 close (closes #741 자동 정합)

---

작성: 2026-05-10
