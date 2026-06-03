# Task #703 Stage 2 — GREEN 완료 보고서

**Issue**: #703
**브랜치**: `local/task703`
**작성일**: 2026-05-08
**구현계획서**: `mydocs/plans/task_m100_703_impl.md`
**선행 단계**: Stage 1 — TDD RED (커밋 `e0b8fd0e`)

---

## 1. 작업 내용

`typeset.rs:1370` 의 Control::Table 분기에 BehindText/InFrontOfText 가드 추가 (`pagination/engine.rs:976-981` 와 동일 시멘틱).

### 정정 diff (`src/renderer/typeset.rs` line 1370 부근, 13 줄 추가)

```rust
Control::Table(table) => {
+    // [Issue #703] 글앞으로 / 글뒤로 표는 Shape처럼 취급 — 본문 흐름 공간 차지 없음.
+    // pagination/engine.rs:976-981 와 동일 시멘틱: 데코레이션 표는 절대 좌표로 배치되며
+    // current_height 누적에 영향을 주지 않는다.
+    if matches!(
+        table.common.text_wrap,
+        crate::model::shape::TextWrap::InFrontOfText
+            | crate::model::shape::TextWrap::BehindText
+    ) {
+        st.current_items.push(PageItem::Shape {
+            para_index: para_idx,
+            control_index: ctrl_idx,
+        });
+        continue;
+    }
    let is_column_top = st.current_height < 1.0;
    ...
}
```

## 2. 테스트 결과

### 단위 테스트

```
$ cargo test --lib --release -- test_typeset_703
test renderer::typeset::tests::test_typeset_703_behind_text_table_no_flow_advance ... ok
test result: ok. 1 passed; 0 failed
```

Stage 1 의 RED → **GREEN 전환** 확인.

### 통합 테스트

```
$ cargo test --release --test issue_703
test issue_703_calendar_year_single_page ... ok
test issue_703_tonghap_2010_11_single_page ... ignored
test issue_703_tonghap_2011_10_single_page ... ignored
test result: ok. 1 passed; 0 failed; 2 ignored
```

| 테스트 | Stage 1 | Stage 2 | 비고 |
|--------|---------|---------|------|
| `issue_703_calendar_year_single_page` | RED (left=2) | **GREEN** | 정상 |
| `issue_703_tonghap_2010_11_single_page` | RED (left=2) | ignored | Issue #704 로 분리 |
| `issue_703_tonghap_2011_10_single_page` | RED (left=2) | ignored | Issue #704 로 분리 |

### 직접 페이지 수 검증

```
$ target/release/rhwp dump-pages samples/basic/calendar_year.hwp | grep -c "^=== 페이지"
1   # ← 한글2022 PDF (1) 와 정합
```

Stage 1 시점 `samples/basic/calendar_year.hwp` 는 2 페이지였음. **결함 정정 확정.**

### 라이브러리 회귀 (lib unittest 1158 건)

```
$ cargo test --lib --release
test result: ok. 1158 passed; 0 failed; 2 ignored
```

회귀 0. 광범위 회귀 검증 (196 샘플 SVG/PDF 비교 + svg_snapshot) 은 Stage 3 에서 수행.

## 3. 통합재정통계 케이스 분리 — Issue #704

수행계획서의 후속 분리 시나리오 ("옵션 A 적용 후 잔존시 별도 task") 가 발생.

본 결함과 다른 본질로 확인:
- 통합재정통계 pi=0 의 표는 `wrap=TopAndBottom tac=true` 1×1 wrapper (BehindText 가 아님)
- 본 task 의 BehindText 가드와 무관
- 0.84 px borderline (5.9 px drift + 각주 117 px + safety 50 px 의 합산 결과)

[**Issue #704**](https://github.com/edwardkim/rhwp/issues/704) 등록. 본 task 의 통합 테스트는 `#[ignore]` 처리 (테스트 코드 보존, 향후 #704 정정 시 ignore 해제).

## 4. 변경 파일

| 파일 | 변동 |
|------|------|
| `src/renderer/typeset.rs` | +13 줄 (BehindText/InFrontOfText 가드) |
| `tests/issue_703.rs` | 통합재정통계 2 테스트 `#[ignore]` 처리 + 주석 갱신 |
| `mydocs/working/task_m100_703_stage2.md` | 신규 (본 보고서) |

## 5. 다음 단계

Stage 3 — 광범위 회귀 검증:
- 196 샘플 SVG/PDF 페이지 수 비교 재실행 (예상 변동: calendar_year 2→1, 그 외 0)
- `cargo clippy --release -- -D warnings`
- `cargo build --release`
- svg_snapshot 회귀 검증
- 최종 보고서 작성

## 6. 작업지시자 승인 요청

Stage 2 GREEN + #704 분리 + Stage 3 진행 승인 부탁드립니다.
