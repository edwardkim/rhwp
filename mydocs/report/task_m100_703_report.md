# Task #703 최종 결과 보고서

**Issue**: [#703](https://github.com/edwardkim/rhwp/issues/703) — BehindText/InFrontOfText 표가 paragraph 높이에 포함되어 trailing 항목이 다음 페이지로 밀림
**브랜치**: `local/task703`
**기간**: 2026-05-08 (1 일, 3 단계)
**마일스톤**: M100 (v1.0.0)
**상태**: 완료, 작업지시자 승인 대기

---

## 1. 결함 요약

**증상**: `samples/basic/calendar_year.hwp` 가 한글2022 PDF 1 페이지인데 SVG 2 페이지로 분할. 마지막 PushButton (pi=12) 이 page 2 로 밀려 거의 빈 페이지 발생.

**Root cause**: `typeset.rs` 의 표 컨트롤 처리 분기에 BehindText / InFrontOfText (글뒤로 / 글앞으로) 가드 누락. 데코레이션 표가 본문 흐름 cur_h 에 잘못 가산.

`pagination/engine.rs:976-981` 에는 동일 가드가 있으나 main pagination 인 typeset.rs 경로에 미반영됐던 결함.

## 2. 분석 출발

196 매칭 (HWP/HWPX, 한글2022 PDF) 쌍 비교 → SVG/PDF 비율 ≥1.5× 5 건 식별:
1. `basic/shortcut` (10×) — 별도 후속
2. `basic/calendar_year` (2×) — **본 task**
3. `통합재정통계(2010.11월)` (2×) — Issue #704 분리
4. `통합재정통계(2011.10월)` (2×) — Issue #704 분리
5. `p122` (0.33×, 역방향) — 별도

`calendar_year.hwp` 페이지 분할 위치를 디버그 인스트루먼트로 추적:
- `typeset.rs:1107-1112` 의 fit 검사가 트리거점
- 누적 cur_h drift +31.8 px 중 23.77 px 가 pi=1 (글뒤로 1×1 wrapper Table 캐리어 빈 paragraph) 단일 문단에서 발생
- `place_table_with_text` (line 1594) 의 `cur_h += pre_height + table_total_height` 가 BehindText 표에도 적용

## 3. 정정 내용 (Stage 2)

`src/renderer/typeset.rs` line 1370 부근 Control::Table 분기에 13 줄 추가:

```rust
Control::Table(table) => {
    // [Issue #703] 글앞으로 / 글뒤로 표는 Shape처럼 취급 — 본문 흐름 공간 차지 없음.
    // pagination/engine.rs:976-981 와 동일 시멘틱: 데코레이션 표는 절대 좌표로 배치되며
    // current_height 누적에 영향을 주지 않는다.
    if matches!(
        table.common.text_wrap,
        crate::model::shape::TextWrap::InFrontOfText
            | crate::model::shape::TextWrap::BehindText
    ) {
        st.current_items.push(PageItem::Shape {
            para_index: para_idx,
            control_index: ctrl_idx,
        });
        continue;
    }
    // ... 기존 흐름
}
```

## 4. 단계별 진행

| 단계 | 커밋 | 작업 | 결과 |
|------|------|------|------|
| Stage 1 (RED) | `e0b8fd0e` | TDD RED 테스트 작성 (단위 1 + 통합 3) | 4 tests RED |
| Stage 2 (GREEN) | `5c4cc190` | typeset.rs +13줄 가드 추가 | 단위 + calendar_year 통합 GREEN. 통합재정통계 2 → #704 분리 |
| Stage 3 (회귀) | (본 보고서 커밋) | 196 샘플 + 1158 lib + svg_snapshot + clippy | 회귀 0, 정합 +2 |

## 5. 검증 결과

### 196 샘플 SVG/PDF 페이지 수 비교 (회귀 검증)

| 변동 | 파일 | baseline | after |
|------|------|----------|-------|
| 정합 (의도) | `basic/calendar_year` | 2 페이지 | **1 페이지** (PDF=1) |
| 정합 (추가) | `table-ipc` | 13 페이지 | **10 페이지** (PDF=10) |

`table-ipc.hwp` 는 11개 표가 `wrap=글앞으로` (InFrontOfText) 로 본 정정의 광범위 효과 자동 적용. **의도하지 않은 추가 정합** 으로 본질 정정의 타당성 강화.

기타 194 파일 SVG 변동 0 (**회귀 0**).

### 라이브러리 / 통합 / svg_snapshot

```
cargo test --lib --release        : 1158 passed; 0 failed; 2 ignored
cargo test --release --test issue_703  : 1 passed; 0 failed; 2 ignored
cargo test --release --test svg_snapshot: 7 passed; 0 failed
cargo test --release --tests      : 전체 통합 0 fail
```

### clippy / build

```
cargo clippy --release -- -D warnings : 0 신규 경고
cargo build --release                  : 정상
```

## 6. 분리된 후속 이슈

**Issue [#704](https://github.com/edwardkim/rhwp/issues/704)** — 통합재정통계 페이지 분할:
- 본 task 분석 단계에서 함께 식별됐으나 다른 본질
- TopAndBottom TAC 1×1 wrapper + 각주 117 px + safety 50 px + drift 5.9 px = 0.84 px borderline
- 본 task 정정으로 미해결 (BehindText 가 아니므로 가드 적용 안 됨)
- `tests/issue_703.rs` 의 통합재정통계 2 케이스는 `#[ignore]` 처리 (#704 정정 시 ignore 해제)

## 7. 변경 파일 합계

| 파일 | 변동 |
|------|------|
| `src/renderer/typeset.rs` | +13 핵심 정정 + 66 단위 테스트 = +79 |
| `tests/issue_703.rs` | 신규 +51 (3 통합 테스트) |
| `mydocs/plans/task_m100_703.md` | 신규 (수행계획서) |
| `mydocs/plans/task_m100_703_impl.md` | 신규 (구현계획서) |
| `mydocs/working/task_m100_703_stage1.md` | 신규 (Stage 1 보고서) |
| `mydocs/working/task_m100_703_stage2.md` | 신규 (Stage 2 보고서) |
| `mydocs/working/task_m100_703_stage3.md` | 신규 (Stage 3 보고서) |
| `mydocs/report/task_m100_703_report.md` | 신규 (본 보고서) |
| `mydocs/report/svg_vs_pdf_diff_20260508.tsv` | 신규 (baseline 데이터) |
| `mydocs/report/svg_vs_pdf_diff_20260508_after.tsv` | 신규 (after 데이터) |

## 8. 작업지시자 승인 요청

다음 항목 승인 부탁드립니다:

1. **Issue #703 close**: 본 보고서 승인 후 commit 메시지 `closes #703`
2. **`local/task703` merge 방향**:
   - `local/devel` 머지 (작업지시자 표준 절차)
   - 또는 `devel` 머지 (본 컨버세이션 시작 시 `devel` 브랜치였음)
3. **미커밋 PDF 변경 (작업지시자 영역)** 처리는 본 task 와 별개

## 9. 메트릭

- 단계 수: 3 (계획 그대로)
- 신규 정정 코드: +13 줄
- 신규 테스트 코드: +117 줄 (단위 66 + 통합 51)
- 회귀 0
- 정합 +2 (calendar_year + table-ipc)
- 후속 이슈 등록: 1 건 (#704)
