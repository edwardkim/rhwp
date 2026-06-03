# Task #775 구현 계획서

수행 계획서: [`task_m100_775.md`](./task_m100_775.md)

## 옵션 비교

### 옵션 A — 가드 조건 정밀화 (단일 컬럼 한정)

```rust
// src/renderer/typeset.rs:1403
if matches!(table.common.text_wrap,
    TextWrap::InFrontOfText | TextWrap::BehindText)
    && self.paper_layout.column_count == 1
{
    st.current_items.push(PageItem::Shape { ... });
    continue;
}
```

- **장점**: 다단 영역 종전 동작 유지 → 본 회귀 즉시 해소
- **단점**: 다단 + InFrontOfText 케이스에서 Task #703 의 본질 (데코레이션 표는 흐름 영향 0) 적용 불가. calendar_year.hwp 가 단일 컬럼인지 확인 필요

### 옵션 B — vert_rel_to 분기

```rust
// vert_rel_to == Page 일 때만 push-only
if matches!(table.common.text_wrap,
    TextWrap::InFrontOfText | TextWrap::BehindText)
    && table.common.vert_rel_to == VertRelTo::Page
{
    st.current_items.push(PageItem::Shape { ... });
    continue;
}
```

- **장점**: vert=Page (절대 좌표 데코레이션) 만 push-only, vert=Para (paragraph 흐름 의존) 는 종전 동작
- **단점**: calendar_year.hwp 의 vert_rel_to 가 Page 이어야 본 케이스 보존 가능

### 옵션 C — typeset fix 철회 + layout 단계 정정

- typeset.rs:1403-1416 revert
- layout.rs 측에서 InFrontOfText 표가 다음 paragraph y_in 누적에 영향 안 주도록 정정

- **장점**: 가장 정밀한 본질 정정 가능
- **단점**: layout 측 변경 범위 큼, calendar_year.hwp / exam_eng.hwp 양쪽 정합 별도 fix 필요

## 권장: **옵션 B** (1차) → 실패 시 옵션 A → 옵션 C

옵션 B 가 의미적으로 가장 정확. vert=Page 만 절대 좌표 데코레이션이고 vert=Para 는 paragraph 흐름 일부. exam_eng.hwp pi=181 ci=1 의 vert=Para(672=2.4mm) 이므로 옵션 B 적용 시 본 케이스에서 종전 동작 (cur_h 누적) 으로 복귀 → 회귀 해소.

calendar_year.hwp 의 IR 검사가 옵션 B 실현가능성 결정.

## 단계별 구현

### Stage 1 — RED 테스트 + IR 사전 검사

1. `tests/issue_775_exam_eng_p4.rs` 신규: cell-clip y ≈ 277 ± 5 px 가드
2. `./target/release/rhwp dump samples/calendar_year.hwp` 으로 표 IR 의 vert_rel_to / column_count 확인
3. RED 확인 (cargo test --release issue_775 → FAIL)

### Stage 2 — GREEN 본질 정정 (옵션 B 우선)

1. `src/renderer/typeset.rs:1403-1416` 의 가드 조건을 옵션 B 로 정밀화
2. 본 테스트 통과 + `tests/issue_703.rs::issue_703_calendar_year_single_page` 통과 확인
3. 옵션 B 가 calendar_year.hwp 회귀 시 옵션 A 로 fallback

### Stage 3 — 라이브러리 회귀 + 단위 테스트

```bash
cargo test --release 2>&1 | grep -E "test result|FAILED"
```
- 1250+ 테스트 통과 (회귀 0)

### Stage 4 — 광범위 sweep

1. golden_svg/ 골든 SVG 회귀 검사 (issue-147, form-002 등)
2. exam_eng.hwp 전 페이지 + samples/ 다단 + InFrontOfText/BehindText 케이스 sweep
3. PDF 권위 자료 양쪽 정합 (calendar_year + exam_eng)

### Stage 5 — 최종 보고서

`mydocs/report/task_m100_775_report.md` 작성:
- 회귀 진원지 + bisect 결과
- 옵션 B 채택 사유
- 양쪽 케이스 정합 입증
- 회귀 가드 추가 + golden_svg 갱신 여부

## 파일 변경 예상

| 파일 | 변경 종류 | 라인 |
|------|----------|------|
| `src/renderer/typeset.rs` | 가드 조건 정밀화 | ~5 |
| `tests/issue_775_exam_eng_p4.rs` | 신규 통합 테스트 | ~40 |
| `mydocs/plans/task_m100_775.md` | 수행 계획서 | (이미 작성) |
| `mydocs/plans/task_m100_775_impl.md` | 구현 계획서 (본 문서) | (이미 작성) |
| `mydocs/working/task_m100_775_stage{1..4}.md` | 단계별 보고서 | ~80 each |
| `mydocs/report/task_m100_775_report.md` | 최종 보고서 | ~150 |

## 위험성

- **회귀 위험**: Task #703 의 calendar_year.hwp 본 케이스가 vert=Para 라면 옵션 B 가 그대로 회귀 → 옵션 A fallback 필요
- **광범위 영향**: typeset.rs 의 InFrontOfText 표 처리 변경은 다단 + 본문 인접 표 모두 영향. golden_svg 갱신 가능성

## 진행 조건

본 구현 계획서 승인 후 Stage 1 시작.
