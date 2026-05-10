# Task #768 Stage 2 (분석) 보고서 — 한계 도달

**Issue**: [#768](https://github.com/edwardkim/rhwp/issues/768)
**Stage**: 2 — 분석 + 정정 시도 + 한계 도달
**작성일**: 2026-05-10

---

## 산출물

`RHWP_TASK768_DEBUG=1` instrument 4 위치 + 정정 시도 3차례. 본 결함의 본질이 단순 가드 로직으로 해소되지 않음을 확인.

## Stage 2 분석 결과

### TASK768_DEBUG trace (페이지 0..3)

```
TASK768_CB: pi=94 column_type=Column has_diff_col_def=false col_count=2 current_column=1 current_height=80.00
TASK768_ADV: action=push_new_page (last col) before=(col=1, h=80.00, col_count=2) → page+1
```

→ pi=94 의 `column-break + has_diff_col_def=false + col_count=2 + current_column=col_count-1` 분기에서 push_new_page. 페이지 잔여 561 px 무시.

다른 column-break 들 (pi=37, 47, 57): `has_diff_col_def=true` 라 process_multicolumn_break 진입 (정상).

## 정정 시도 흐름

### 시도 1: 무가드 wrap-around

```rust
} else if !st.current_items.is_empty() {
    let is_last_col_in_multi = st.col_count > 1
        && st.current_column + 1 >= st.col_count;
    if is_last_col_in_multi {
        self.advance_column_break_wrap_in_multi(&mut st);
    } else {
        st.advance_column_or_new_page();
    }
}
```

- 결과: RED PASS (pi=94 페이지 인덱스 2)
- 부작용: shortcut.hwp 페이지 8 → 7 (PDF 7 정합 ✓), 그러나 페이지 3 에 PDF 페이지 4 까지 누적 (다단 zone 다수)
- 회귀: 3건 (test_539, test_548, test_exam_math_page_count)
- 원인: column-break + last col 케이스가 광범위. exam_math.hwp 등 다른 샘플의 column-break 들도 wrap 으로 합쳐져 페이지 수 절반 수준 감소

### 시도 2: vpos=0 가드 추가

```rust
let curr_first_vpos = para.line_segs.first().map(|s| s.vertical_pos);
let is_column_start = matches!(curr_first_vpos, Some(0));
if is_last_col_in_multi && is_column_start {
    self.advance_column_break_wrap_in_multi(&mut st);
}
```

- shortcut pi=94: vpos=0 → wrap ✓
- exam_math pi=20: vpos=7001 → push_new_page (정상) ✓
- 그러나 exam_math pi=64/90/...: vpos=0 → wrap (회귀)
- 회귀: 3건 → 3건 (test_exam_math_page_count: 11→11 잔존)

### 시도 3: vpos=0 + controls=0 가드 추가

```rust
let no_controls = para.controls.is_empty();
if is_last_col_in_multi && is_column_start && no_controls {
    ...
}
```

- shortcut pi=94: controls=0 → wrap ✓
- exam_math pi=64: controls=2 (수식) → push_new_page (정상) ✓
- exam_math 페이지 수: 11 → 20 (PDF 정합) ✓
- 그러나 21_언어_기출_편집가능본.hwp pi=44/71/103/110/135: controls=0, vpos=0 → wrap (회귀)
- 회귀: 3건 → 2건 (test_539, test_548)

### 추가 가드 후보 검토

| 후보 | shortcut pi=94 | 21_언어_기출 pi=44 | 결정 |
|------|---------------|------------------|------|
| ps_id 차이 | ps_id=12 | ps_id=? (확인 필요) | 휴리스틱 |
| paragraph text 패턴 (각괄호 시작 등) | "<편집..." | "다음..." | 너무 휴리스틱 |
| line_segs 개수 | 1 | 1 (텍스트만 짧으면 동일) | 구분 안 됨 |
| 다음 paragraph 의 ColumnBreakType lookahead | pi=95 None, pi=96 MultiColumn | 모두 None? | lookahead 복잡 |

→ **단순 가드만으로 shortcut pi=94 와 다른 column-break 들을 안정적으로 구분할 수 없음**.

## 본질 진단 (정정 한계)

### 한컴 다단 분배 algorithm 의 미지

PDF 권위 자료를 보면 한컴 (한글 2022) 의 column-break 처리는 paragraph 단위 미세 조건에 따라 다르게 동작:

- shortcut.hwp: column-break wrap (같은 다단 영역 내 row 추가)
- exam_math.hwp: column-break push_new_page (페이지 break)
- 21_언어_기출: column-break (test_539/548 의 시각 정합 검증) — wrap 또는 다른 동작?

이 차이는 IR 의 단순 필드 (column_type, vpos, controls) 만으로 식별되지 않음. 한컴 내부의 다른 신호 (예: 페이지 잔여 공간 검사, ParaShape 의 특정 flag, 또는 column-break paragraph 의 위치 의미론) 가 작용.

### 영향 범위 광범위

본 정정 적용 시 수정 영향이 다단 영역 전체에 미치고, 회귀가 시각 정합 검증 (test_539, test_548) 까지 전파. 단순 가드 추가만으로는 회귀 0 달성 어려움.

근본적 해결을 위해서는:
1. 한컴 다단 분배 algorithm 의 정확한 디코딩 (HWPSPEC + 한글 동작 역공학)
2. 다단 분배 algorithm 자체 재설계 (vpos-reset trigger / column-break / [다단나누기] 통합 처리)
3. 페이지 잔여 공간 자동 계산 + lookahead

이는 본 task 의 영역을 크게 초과.

## 권고 다음 단계

### 옵션 A: 본 task 보류 (revert + 이슈 상태 변경)

- 현재 정정 revert (이미 완료)
- 이슈 #768 상태: open 유지, 라벨 추가 (depth=high, 다단 분배 algorithm 영역)
- 별도 RFC / 깊은 분석 task 등록 필요

### 옵션 B: 부분 정정 + 회귀 인지 PR

- 시도 3 정정 (vpos=0 + controls=0 가드) 적용
- 회귀 2건 (test_539, test_548) 의 baseline 갱신 (test 자체를 본 정정 후 동작으로 갱신)
- 위험: PDF 정합 미달 가능, 실제 시각 결함을 마스킹할 수 있음

### 옵션 C: revert + RED test 보존

- 정정 revert (현재 상태)
- RED test (`tests/issue_768.rs`) 는 보존 (`#[ignore]` 또는 결함 추적용)
- 별도 task 로 본격 분석/구현

## 작업지시자 결정 요청

본 task 의 root cause 가 식별되었으나 안정적 정정은 다단 분배 algorithm 의 깊은 재설계가 필요함을 확인. 옵션 A/B/C 중 선택 요청.

현재 상태:
- src/renderer/typeset.rs: 변경 없음 (revert 완료)
- tests/issue_768.rs: 신규 작성 + FAIL 상태 (RED)
- mydocs/plans/task_m100_768.md, task_m100_768_impl.md: 작성 완료
- mydocs/working/task_m100_768_stage1.md, _stage2.md: 작성 완료
- 회귀 0 (전체 테스트 통과 — issue_768 만 FAIL)

## 부록: revert 후 검증

```
$ cargo test --release
test result: ok. 1217 passed; 0 failed; 2 ignored;
... (issue_768 만 FAIL — 결함 재현 유지)

$ cargo test --test issue_768
[issue_768] pi=94 등장 페이지 인덱스 = 3 (page_count=8), PDF 권위 = 2
test issue_768_pi94_appears_on_page3_not_page4 ... FAILED
```
