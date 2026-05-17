# Task #952 Stage 4 — 시각 검증 + 회귀 분리 + 종합 보고

## 1. Issue 1 fix 적용 + 검증

### 1.1 Fix 본질

**Root cause**: `4bb11289 fix: 쪽테두리 종이기준/본문기준 bit 해석 반전 정정 (closes #920)` 의 `paper_based = (attr & 0x01) == 0` 비트 반전이 회귀 source.

**Spec 분석 결과** — bit 0 은 outline 위치를 결정하지 않음:
- HWPX schema (`section.rs:562`): textBorder="PAPER" → bit 0 = 1
- 한컴 viewer 실측: textBorder/attr 무관 모든 sample 이 paper-based outline

| Sample | attr (HWP5) | textBorder (HWPX) | fillArea (HWPX) | 한컴 시각 |
|--------|-------------|---------------------|-----------------|-----------|
| sample16 (HWP3→HWP5 변환) | 0x01 | PAPER | PAPER | paper |
| 3-09월/3-11월 시험지 | 0x00 | CONTENT | PAPER | **paper** |
| biz_plan, 국립국어원, text-align-2, pua-test | 0x01 | - | - | paper |

**Fix**: `paper_based = true` 강제. (회귀 history: task877 의 `!= 0` → #920 의 `== 0` 모두 부분 정답이었음.)

### 1.2 시각 검증 결과 (3개 fmt × 2 sample)

| Sample | 포맷 | 외곽선 좌표 | 검증 |
|--------|------|------------|------|
| sample16 page 17 | HWP3 | x=18.93~774.77 (paper) | ✓ task877 baseline 정합 |
| sample16 page 17 | HWP5 | x=18.93~774.77 | ✓ |
| sample16 page 17 | HWPX | x=18.93~774.77 | ✓ |
| 시험지 (3-11월) page 1 | HWP5 | x=26.45~767.25 | ✓ 한컴 viewer 정합 |
| 시험지 (3-11월) page 1 | HWPX | x=26.45~767.25 | ✓ |

### 1.3 cargo test 검증

- 1286 passed, 0 failed (rebase 전)
- rebase 후 빌드 통과 (cargo build --release)

## 2. Issue 2 root cause (별도 task 분리)

**증상**: sample16 page 18 "나. 주요 과업내용" 후 본문 (pi=395~401 "○ 통합모델...") 이 다음 페이지로 밀림.

**진단**:
- dump-pages: pi=395~401 가 page 18 layout 에 배치된 것으로 표시 (단 0 items=12, used=941.4px)
- SVG render 실태: pi=395 가 y=1220px 에 emit (body 외 영역)
- 회귀 commit 없음 — 9e038d27 (Stage 3 v2) 에서도 동일 증상 → **장기 layout 결함**

**Cursor over-advance**: pi=394 (TAC TopAndBottom: caption + diagram + caption) 후 y_offset 이 +430px 초과 누적:
- pi=394 y_in=255.8 → y_out=767.3 (dy=511.5 ✓ 정상)
- pi=395 y_in=**1197.9** (← +430px 추가 누적, 어디서 발생하는지 미확정)

**위치 후보**: typeset.rs 의 multi-TAC-TopAndBottom shape 다음 paragraph cursor 갱신 영역.

→ **별도 task 분리**. 본 task #952 범위 외.

## 3. Issue 3 root cause (별도 task 분리)

**증상**: 시험지 (3-11월) page 1 우측 단 문9 가 한컴 대비 ~250px 아래.

| 문제 | rhwp y | 한컴 (PDF 참조) |
|------|--------|----------------|
| 문6 | 103 | ~ |
| 문7 | 319 | ~ |
| 문8 | 562 | ~ |
| 문9 | **1061** | ~810 |

**진단**: HWP5 format. Issue 2 (HWP3 parser) 와 다른 root cause. **HWP5 column vertical layout 또는 endnote/footnote/picture 영향**.

→ **별도 task 분리**.

## 4. 결과

### 4.1 commit 대상

- `src/renderer/layout.rs` — paper_based = true + RHWP_DEBUG_PAGE_BORDER 영구화
- `samples/3-09월_교육_통합_2022.{hwp,hwpx}` — 시험지 sample (Issue 1 attr=0 검증)
- `samples/3-09월_교육_통합_2023.{hwp,hwpx}`
- `samples/3-10월_교육_통합_2022.{hwp,hwpx}`
- `samples/3-11월_실전_통합_2022.{hwp,hwpx}` — 본 task 의 핵심 fixture
- `pdf/3-09월_교육_통합_2022.pdf` 외 3개 — 한컴 2022 권위 PDF
- `mydocs/plans/task_m100_952.md`
- `mydocs/working/task_m100_952_stage4.md`
- `mydocs/report/task_m100_952_report.md`
- `mydocs/orders/20260517.md`

### 4.2 후속 task 등록 필요 (작업지시자 처리)

- Issue 2 별도 task — sample16 page 18 typeset cursor over-advance 분석 + fix
- Issue 3 별도 task — HWP5 시험지 column vertical layout 분석 + fix
