# Task #703 구현 계획서

**Issue**: #703
**브랜치**: `local/task703`
**작성일**: 2026-05-08
**수행계획서**: `mydocs/plans/task_m100_703.md` (승인 완료)

---

## 1. 정정 위치 정밀 추적 결과 (수행계획서 보강)

수행계획서 단계에서 root cause 위치를 `height_measurer.rs` 로 추정했으나, 추가 디버그 트레이싱(`typeset.rs:1107-1112` 의 fit 검사 트리거 + `place_table_with_text` 의 cur_h 누적 추적) 결과 실제 정정 위치는 **`typeset.rs` 의 표 컨트롤 처리 분기**임을 확정.

### 근거

```bash
# height_measurer 와 typeset 모두 BehindText/InFrontOfText 처리 0 건
$ grep -n "BehindText\|InFrontOfText\|글뒤로\|글앞으로" \
    src/renderer/height_measurer.rs src/renderer/typeset.rs
# (결과 없음)

# pagination/engine.rs:976 에는 이미 처리됨 — 본 정정의 reference
$ grep -nA2 "BehindText\|InFrontOfText" src/renderer/pagination/engine.rs | head
src/renderer/pagination/engine.rs:976:if matches!(table.common.text_wrap, ...InFrontOfText | BehindText) {
src/renderer/pagination/engine.rs:977:    st.current_items.push(PageItem::Shape { ... });
src/renderer/pagination/engine.rs:978:    continue;
```

### 결함 메커니즘 정정

calendar_year pi=1 (글뒤로 1×1 wrapper Table 캐리어 빈 paragraph) 처리 흐름:

1. `typeset_section` 메인 루프 → has_table=true → `typeset_table_paragraph` (line 633-639)
2. `typeset_table_paragraph` 의 Control::Table 분기 (line 1369) → `typeset_block_table` (TAC 아님)
3. `typeset_block_table` → fits 검사 통과 → `place_table_with_text` (line 1693)
4. `place_table_with_text` → `st.current_height += pre_height + table_total_height` (line 1594)
   - **여기서 BehindText 표의 height 가 본문 흐름에 잘못 가산**

`pagination/engine.rs:976` 와 동등한 BehindText/InFrontOfText 가드가 typeset.rs 에 누락된 것이 본질.

## 2. 단계 구성 (3 단계)

### Stage 1 — TDD RED

**목표**: 결함 재현 + 검증 기준 확립.

**작업**:
1. 단위 테스트 추가 (`src/renderer/typeset.rs` 의 `#[cfg(test)] mod tests`):
   - `test_behindtext_table_does_not_consume_flow_height`:
     - 빈 paragraph + 1×1 BehindText Table 컨트롤 인풋 fixture
     - paginate 실행 후 cur_h 가 lh+ls 만 (≈21.33 px) 누적되어야 함을 검증
     - **현재**: 표 height (37.1 px) 가 추가로 가산되어 RED
2. 통합 검증 스크립트 작성 (`tests/issue_703_pagination.rs` 또는 기존 통합 위치):
   - `samples/basic/calendar_year.hwp` 페이지 수 == 1 검증
   - `samples/통합재정통계(2010.11월).hwp` 페이지 수 == 1 검증
   - `samples/통합재정통계(2011.10월).hwp` 페이지 수 == 1 검증
   - **현재**: 모두 2 페이지 (RED)

**완료 조건**: 모든 새 테스트가 RED (fail) 상태로 확인.

**커밋**: `Task #703 Stage 1: TDD RED — BehindText/InFrontOfText 표 본문 흐름 누락 검증 단위 테스트`

### Stage 2 — GREEN (정정 구현)

**목표**: BehindText/InFrontOfText 표 본문 흐름 제외 분기 추가.

**작업**:
1. `src/renderer/typeset.rs` 의 `typeset_table_paragraph` (line 1332) Control::Table 분기 (line 1370 부근) 에 reference 와 동일한 가드 추가:

```rust
Control::Table(table) => {
    // 글앞으로 / 글뒤로: Shape처럼 취급 — 공간 차지 없음
    // (pagination/engine.rs:976-981 와 동일 시멘틱)
    use crate::model::shape::TextWrap;
    if matches!(table.common.text_wrap,
        TextWrap::InFrontOfText | TextWrap::BehindText)
    {
        st.current_items.push(PageItem::Shape {
            para_index: para_idx,
            control_index: ctrl_idx,
        });
        continue;
    }
    let is_column_top = st.current_height < 1.0;
    // ... (기존 흐름)
}
```

2. (조사 단계) Shape/Picture 컨트롤도 동일 가드 누락 여부 확인. 누락이면 같이 추가:
   - `Control::Shape(shape_obj) => { ... }` (line 1409 부근) 의 BehindText/InFrontOfText 분기 점검
   - `Control::Picture(_)` 의 BehindText/InFrontOfText 분기 점검
   - **본 task 는 Table 만 정정** — Shape/Picture 는 별도 task 가 필요하면 후속 등록

3. Stage 1 의 단위 테스트 + 통합 테스트 GREEN 확인.

4. 직접 정합 검증:
```bash
target/release/rhwp dump-pages samples/basic/calendar_year.hwp 2>&1 | grep -c "^=== 페이지"
# 기대: 1
target/release/rhwp dump-pages "samples/통합재정통계(2010.11월).hwp" 2>&1 | grep -c "^=== 페이지"
# 기대: 1
```

**완료 조건**: 3 파일 모두 1 페이지 출력 + Stage 1 테스트 GREEN.

**커밋**: `Task #703 Stage 2: GREEN — typeset_block_table 분기에 BehindText/InFrontOfText 가드 추가`

### Stage 3 — 광범위 회귀 검증 + 최종 보고서

**목표**: 196 샘플 회귀 0 확정 + 산출물 정리.

**작업**:
1. 196 샘플 SVG 페이지 수 비교 재실행:
```bash
python3 /tmp/compare2.py > /tmp/svg_vs_pdf_after.tsv
diff /tmp/svg_vs_pdf.tsv /tmp/svg_vs_pdf_after.tsv
```
- **기대 변동**: calendar_year 2→1, 통합재정통계 2건 2→1. 그 외 0.
- 회귀 (정정과 무관한 페이지 수 변동) 발견 시 stop + 분석.

2. cargo 검증:
```bash
cargo test --lib --release 2>&1 | tail -5     # 1140+ tests, 회귀 0
cargo clippy --release -- -D warnings 2>&1 | tail -5
cargo build --release 2>&1 | tail -3
```

3. svg_snapshot 회귀 0 확인 (기존 골든 SVG 비교 통과).

4. (선택) WASM 빌드 크기 측정 (참고용).

5. 산출물:
   - `mydocs/working/task_m100_703_stage1.md` (TDD RED 단계 보고)
   - `mydocs/working/task_m100_703_stage2.md` (GREEN 단계 보고)
   - `mydocs/working/task_m100_703_stage3.md` (회귀 검증 단계 보고)
   - `mydocs/report/task_m100_703_report.md` (최종 보고서)
   - 각 단계 보고서는 해당 단계 커밋과 같이 push.

6. 최종 보고서 작성 후 작업지시자 승인 → `gh issue close 703` (또는 commit 메시지 `closes #703`).

**완료 조건**: 회귀 0 + 모든 검증 통과 + 최종 보고서 작성 + 승인.

**커밋**: `Task #703 Stage 3: 회귀 검증 0 + 최종 보고서 (closes #703)`

## 3. 회귀 위험 시나리오 및 대응

| 시나리오 | 위험 | 대응 |
|---------|------|------|
| 글뒤로 표를 본문 흐름 차지로 의도한 문서 | 페이지 수 감소 (정합 방향) | 196 샘플 비교 결과 확인. 의외의 변동 발생 시 분석 |
| paginate engine vs typeset 두 경로 불일치 잠재 | typeset 만 수정하면 engine 미반영 | 본 task 범위 외. 필요시 별도 task 등록 |
| Shape/Picture BehindText 도 동일 결함 잠재 | 미처리 케이스 | Stage 2 작업 2 에서 Table 만 정정. Shape/Picture 는 발견 즉시 별도 task 등록 |

## 4. 타이밍 및 리소스

- 단계당 빌드 + 테스트: 평균 2~3분 cargo (release)
- Stage 1: 30분 ~ 1시간 (테스트 fixture 작성)
- Stage 2: 30분 ~ 1시간 (소스 정정 + 검증)
- Stage 3: 30분 ~ 1시간 (광범위 회귀 + 보고서)

## 5. 작업지시자 승인 요청

다음 항목 승인 부탁드립니다:

1. **3 단계 구성** (RED → GREEN → 회귀 검증) 채택
2. **정정 위치 변경**: 수행계획서의 "height_measurer" → 본 구현계획서의 "typeset_block_table / place_table_with_text" (디버그 트레이싱 결과)
3. **Shape/Picture 동일 결함 처리**: Stage 2 작업 2 에서 조사 후 누락 시 본 task 에 흡수 vs 별도 task 등록 (현재안: 별도 task)
4. **단위 테스트 위치**: `src/renderer/typeset.rs#mod tests` 인-파일 vs `tests/issue_703_*.rs` 별도 파일 (현재안: 양쪽 — 인-파일은 단위, 별도 파일은 통합)

승인 후 Stage 1 시작합니다.
