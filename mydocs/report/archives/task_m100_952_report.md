# Task #952 — 최종 보고서

- 이슈: [#952](https://github.com/edwardkim/rhwp/issues/952)
- 마일스톤: M100
- 브랜치: `local/task952`
- 기간: 2026-05-17 (1일)

## 1. 작업 범위 (최종 확정)

원 task #952 의 "외곽선 밖 내용 돌출" 증상을 깊이 분석한 결과 **3개 별개 회귀**로 분리:

| # | 항목 | 본 task 처리 |
|---|------|------------|
| 1 | 페이지 외곽선 paper/body 분류 | ✅ Fix 완료 |
| 2 | sample16 page 18 본문 다음 페이지 밀림 | 별도 task 분리 |
| 3 | 시험지 page 1 문9 vertical 처짐 | 별도 task 분리 |

## 2. Issue 1 — Fix 본질

### 2.1 회귀 source 식별

- `4bb11289 fix: 쪽테두리 종이기준/본문기준 bit 해석 반전 정정 (closes #920)` 의 비트 해석 반전이 회귀 source
- task877 baseline (`(attr & 0x01) != 0`): sample16 정합, 시험지 회귀
- #920 (`(attr & 0x01) == 0`): 시험지 정합, sample16 회귀
- → 두 해석 모두 부분 정답

### 2.2 Spec 정답 발견

`src/parser/hwpx/section.rs:562`:
```rust
if text_border.eq_ignore_ascii_case("PAPER") {
    attr |= 0x0000_0001;
}
```

→ bit 0 = 1 = textBorder=PAPER. 그러나 **한컴 viewer 실측 결과 textBorder/bit 0 무관 모든 sample 이 paper-based outline 렌더**.

→ `paper_based = true` 강제가 정답.

### 2.3 시각 검증 (5+ samples)

| Sample | attr | textBorder | 한컴 시각 | 본 fix 후 |
|--------|------|-----------|----------|----------|
| sample16 HWP3/HWP5/HWPX | 0x01 | PAPER | paper | ✓ |
| 시험지 (3-11월) HWP5/HWPX | 0x00/0x40 | CONTENT | paper | ✓ |
| biz_plan, 국립국어원, text-align-2 등 | 0x01 | - | paper | ✓ |

cargo test: 1286 passed, 0 failed.

### 2.4 Fix 구현

```rust
// src/renderer/layout.rs:770
let paper_based = true;
```

+ `RHWP_DEBUG_PAGE_BORDER` 환경변수 영구화 (attr 비트 값 + paper_based 결정 추적).

## 3. Issue 2 / 3 (별도 task 분리 사유)

### Issue 2 — sample16 page 18 본문 누락

장기 layout 결함 (회귀 commit 없음). typeset.rs 의 multi-TAC-TopAndBottom shape 후 cursor over-advance ~430px. 디버그 결과:
```
pi=394 y_in=255.8 → y_out=767.3 (dy=511.5 ✓ 정상)
pi=395 y_in=1197.9 (← 추가 +430px, 어디서?)
```

typeset.rs (2700+ 줄) 의 multi-state (zone_y_offset, current_height, vpos-reset, wrap-around, multi-column) 복잡도로 본 session 안에서 안전한 fix 불가. **별도 task** 의 깊은 분석 + 다수 sample 회귀 검증 필요.

### Issue 3 — 시험지 문9 vertical 처짐

HWP5 column vertical layout 회귀. 다른 root cause. 별도 task.

## 4. archive/task936 와의 비교

본 task 의 진행 사이클 — Issue 1 의 명확한 fix + Issue 2/3 의 별도 분리 — 는 archive/task936 의 "9회 시도 + 5회 revert" 패턴과 대조적. **부분 해결 + 명확한 분리**가 본질적 typeset 결함의 안전한 fix 보다 효과적이라는 교훈.

## 5. 변경 파일 요약

- `src/renderer/layout.rs` — Issue 1 fix + RHWP_DEBUG_PAGE_BORDER
- `samples/3-09월_교육_통합_2022.{hwp,hwpx}`, `3-09월_교육_통합_2023.{hwp,hwpx}`, `3-10월_교육_통합_2022.{hwp,hwpx}`, `3-11월_실전_통합_2022.{hwp,hwpx}` — 시험지 회귀 fixture (8 files)
- `pdf/3-09월_교육_통합_2022.pdf` 외 3개 — 한컴 2022 권위 PDF (4 files)
- `mydocs/plans/task_m100_952.md`, `mydocs/working/task_m100_952_stage4.md`, `mydocs/report/task_m100_952_report.md`, `mydocs/orders/20260517.md`

## 6. 후속

- Issue 2 별도 task 등록 — sample16 page 18 typeset cursor over-advance 분석 + fix
- Issue 3 별도 task 등록 — HWP5 시험지 column vertical layout 분석 + fix
- 본 task #952 close (Issue 1 만 해결, 별도 task 분리 명시)
