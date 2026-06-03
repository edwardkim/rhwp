# Task #391 구현 계획서 — 다단 섹션 #359 vpos-reset 선제 가드 오발동 수정

- **이슈**: [#391](https://github.com/edwardkim/rhwp/issues/391)
- **브랜치**: `local/task391`
- **수행계획서**: [`task_m100_391.md`](task_m100_391.md)

## 단계 구성 (4 단계)

TDD 흐름 — Red → Green → 회귀 검증 → 통합/보고서.

---

## 단계 1 — 재현 정량 진단 + Red 테스트 (Diagnosis)

### 목표

- 현재(devel) `exam_eng.hwp` 의 페이지 수, 단별 used height, 단독 1-item 단 위치를 baseline 으로 기록.
- 단계 2 수정 효과를 정량 비교할 수 있도록 단위 테스트 케이스 정의.

### 작업

1. **baseline 측정**
   ```bash
   ./target/release/rhwp dump-pages samples/exam_eng.hwp > mydocs/working/task_m100_391_baseline.txt
   ```
   출력에서 페이지 수, 단별 items/used/hwp_used, 단독 1-item 단의 page/col/pi 인덱스 추출.

2. **단위 테스트 추가** — `tests/exam_eng_multicolumn.rs` (신규, 통합 테스트 디렉터리)
   ```rust
   //! Task #391: 다단 섹션 단 전환 정상화 회귀 테스트
   #[test]
   fn exam_eng_page_count() {
       // exam_eng.hwp 는 8 페이지여야 함 (#359 이전 동작 복원)
       let result = paginate_sample("samples/exam_eng.hwp");
       assert_eq!(result.pages.len(), 8, "exam_eng 8 페이지 기대 (현재 11p 회귀)");
   }

   #[test]
   fn exam_eng_no_single_item_column() {
       // 단당 최소 5 items 이상 (단독 1-item 단 금지)
       // 단, 마지막 페이지의 마지막 단은 자연 미달 허용
       let result = paginate_sample("samples/exam_eng.hwp");
       for (pi, page) in result.pages.iter().enumerate() {
           let is_last = pi + 1 == result.pages.len();
           for (ci, col) in page.column_contents.iter().enumerate() {
               let is_last_col_of_last_page = is_last && ci + 1 == page.column_contents.len();
               if !is_last_col_of_last_page {
                   assert!(col.items.len() >= 5,
                       "p{} col{} items={} (단독 1-item 단 발생)", pi, ci, col.items.len());
               }
           }
       }
   }
   ```

3. **테스트 실행 → Red 확인**
   ```bash
   cargo test --test exam_eng_multicolumn 2>&1 | tail -20
   ```
   - `exam_eng_page_count`: assertion failed (11 != 8)
   - `exam_eng_no_single_item_column`: 단독 단 발견 시 panic

### 산출물

- `tests/exam_eng_multicolumn.rs` (신규)
- `mydocs/working/task_m100_391_baseline.txt` (baseline 측정값)
- `mydocs/working/task_m100_391_stage1.md` (단계 보고서)

### 단계 1 완료 조건

- baseline 정량 데이터 수집 완료
- 단위 테스트 추가 + Red 확인
- 단계 1 보고서 작성 + 커밋 (`Task #391 단계 1: 재현 정량 진단 + Red 테스트`)

---

## 단계 2 — 가드 비활성화 조건 추가 (Green)

### 목표

`next_will_vpos_reset` 계산에 `is_last_column` 조건 추가 → 다단 비-마지막 단에서 가드 발동 안 함.

### 작업

`src/renderer/typeset.rs:431-444` 수정:

**Before:**
```rust
let next_will_vpos_reset = if !st.current_items.is_empty() && para_idx + 1 < paragraphs.len() {
    let next_para = &paragraphs[para_idx + 1];
    let next_force_break = next_para.column_type == ColumnBreakType::Page
        || next_para.column_type == ColumnBreakType::Section;
    if next_force_break {
        false
    } else {
        let next_first_vpos = next_para.line_segs.first().map(|s| s.vertical_pos);
        let curr_last_vpos = para.line_segs.last().map(|s| s.vertical_pos);
        matches!((next_first_vpos, curr_last_vpos), (Some(nv), Some(cl)) if nv == 0 && cl > 5000)
    }
} else { false };
```

**After:**
```rust
// [Task #391] 다단 비-마지막 단에서는 vpos-reset 이 "다음 단으로의 정상 전환" 시그널이지
// 단독 항목 페이지 위험 신호가 아니다. 마지막 단 (current_column + 1 >= col_count)
// 에서만 가드 발동. 단단 (col_count == 1) 에서는 항상 마지막 단 → 단단 거동 무변화.
let is_last_column = st.current_column + 1 >= st.col_count;
let next_will_vpos_reset = if !st.current_items.is_empty()
    && para_idx + 1 < paragraphs.len()
    && is_last_column
{
    let next_para = &paragraphs[para_idx + 1];
    let next_force_break = next_para.column_type == ColumnBreakType::Page
        || next_para.column_type == ColumnBreakType::Section;
    if next_force_break {
        false
    } else {
        let next_first_vpos = next_para.line_segs.first().map(|s| s.vertical_pos);
        let curr_last_vpos = para.line_segs.last().map(|s| s.vertical_pos);
        matches!((next_first_vpos, curr_last_vpos), (Some(nv), Some(cl)) if nv == 0 && cl > 5000)
    }
} else { false };
```

### 검증

```bash
cargo build --release 2>&1 | tail -3
cargo test --test exam_eng_multicolumn 2>&1 | tail -10  # Green 기대
```

### 단계 2 완료 조건

- 코드 변경 (~5 줄)
- 단위 테스트 Green 확인
- 단계 2 보고서 + 커밋 (`Task #391 단계 2: 가드 is_last_column 조건 추가 (Green)`)

### 만약 Green 실패 시

baseline 비교로 잔존 단 채움 부족 (~수십 px) 확인. 누적 공식 다단 분기 (`is_last_column ? total_height : height_for_fit`) 는 단계 4 에서 추가 검토.

---

## 단계 3 — 회귀 검증 (Regression Check)

### 목표

7 핵심 샘플 + 추가 다단/단단 샘플의 페이지 수 + LAYOUT_OVERFLOW 무회귀 확인.

### 작업

1. **회귀 비교 스크립트 실행**
   ```bash
   for s in form-01 aift KTX k-water-rfp exam_eng kps-ai hwp-multi-001 \
            21_언어 exam_math exam_kor biz_plan; do
     pages=$(./target/release/rhwp dump-pages samples/${s}.hwp 2>/dev/null \
             | grep -c "^=== 페이지")
     overflow=$(RHWP_LAYOUT_OVERFLOW_LOG=1 ./target/release/rhwp export-svg \
                samples/${s}.hwp -o /tmp/regression/${s} 2>&1 \
                | grep -c "LAYOUT_OVERFLOW")
     echo "${s}: ${pages}p, overflow=${overflow}"
   done > mydocs/working/task_m100_391_regression.txt
   ```

2. **단위 테스트 전체 실행**
   ```bash
   cargo test --lib 2>&1 | tail -5             # 1008+ passed 기대
   cargo test --test svg_snapshot 2>&1 | tail -5
   cargo test --test issue_301 2>&1 | tail -5
   cargo clippy --lib -- -D warnings
   ```

3. **다단 단 채움 시각 검증**
   ```bash
   ./target/release/rhwp export-svg samples/exam_eng.hwp -o output/svg/exam_eng/
   ./target/release/rhwp export-svg samples/k-water-rfp.hwp -o output/svg/k-water-rfp/
   ```
   작업지시자 시각 판정 요청 (단계 3 보고서에 명기).

### 합격 기준

| 샘플 | 페이지 | LAYOUT_OVERFLOW |
|---|---|---|
| form-01 | 1 → 1 | 0 → 0 |
| aift | 77 → 77 | 3 → 3 |
| KTX | 27 → 27 | 1 → 1 |
| **k-water-rfp** | 27 → 27 | **0 → 0** |
| **exam_eng** | 11 → **8** | 0 → 0 |
| kps-ai | 81 → 81 | 4 → 4 |
| hwp-multi-001 | 10 → 10 | 0 → 0 |

핵심 가드: **k-water-rfp 0 → 0** 유지 (#359 회귀 회피).

### 단계 3 완료 조건

- 회귀 비교 표 + cargo test 결과 첨부한 단계 3 보고서
- 작업지시자 시각 판정 승인
- 커밋 (`Task #391 단계 3: 회귀 검증`)

### 만약 회귀 발생 시

- k-water-rfp overflow 재발: 분석 후 추가 수정. 누적 공식 분기 또는 가드 조건 재정의.
- 다른 샘플 페이지 수 변동: 변동 원인 분석 후 단계 3 보고서에 명기. 무해 (시각 판정 통과) 면 진행, 아니면 수정.

---

## 단계 4 — 통합 검증 + 최종 보고서 + WASM

### 작업

1. **WASM 빌드 확인**
   ```bash
   docker compose --env-file .env.docker run --rm wasm 2>&1 | tail -10
   ls -lh pkg/rhwp_bg.wasm pkg/rhwp.js
   ```

2. **최종 보고서 작성** — `mydocs/report/task_m100_391_report.md`
   - 증상 → 원인 → 수정 → 회귀 검증 표 → 시각 판정 결과
   - WASM 빌드 사이즈 비교

3. **CHANGELOG 갱신** — `CHANGELOG.md`
   ```markdown
   ## [Unreleased]
   ### Fixed
   - **#391** 다단 섹션 vpos-reset 가드 오발동 수정. exam_eng 11 → 8 페이지 복원.
     k-water-rfp 등 단단 케이스 무회귀.
   ```

4. **오늘할일 갱신** — `mydocs/orders/{오늘날짜}.md` 에 #391 완료 항목 추가.

5. **타스크 브랜치 커밋** — 단계 4 산출물 커밋.

6. **Issue close 승인 요청** — 작업지시자 승인 후 `gh issue close 391`.

### 단계 4 완료 조건

- 최종 보고서 작성
- CHANGELOG / orders 갱신 + 커밋
- WASM 빌드 통과
- 작업지시자 승인 후 이슈 close
- 작업지시자 승인 후 `local/task391` → `devel` merge

---

## 승인 요청

본 구현 계획서 승인 후 단계 1 진행.
