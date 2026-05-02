# Task #464 단계 3 보고서 — typeset.rs + engine.rs 수정 + 결함 검증

**이슈**: #464
**브랜치**: `local/task464_v2`
**전제**: 단계 1, 2 완료

---

## 1. 코드 수정

### 1.1 `src/renderer/typeset.rs:904` — col_break 감지 가드 완화

```rust
// [Task #464] current_column == 0 가드 제거: col 1 (마지막 단) 에서도
// vpos-reset 인코딩된 col_break 를 감지해 페이지 break 를 트리거.
let col_breaks = if st.col_count > 1 {
    Self::detect_column_breaks_in_paragraph(para)
} else {
    vec![0]
};
```

### 1.2 `src/renderer/typeset.rs:1975~1981` — 페이지 break 트리거 통합

```rust
// 다음 col_break 가 있으면 다음 단 또는 새 페이지로 이동.
// [Task #464] 마지막 단(col 1)에서 col_break 발생 시에도 페이지 break 를
// 트리거하도록 advance_column_or_new_page 통합 호출 사용
// (이전 코드는 단 변경만 처리하고 페이지 break 미처리 → col 1 의 후속 lines 가
//  같은 단에 누적되어 overflow 발생).
if bi + 1 < col_breaks.len() {
    st.advance_column_or_new_page();
}
```

기존 코드 (flush_column + 직접 column 변경 + height reset) 를 통합 메서드 호출로 대체.

### 1.3 `src/renderer/pagination/engine.rs:609` — 동기 가드 완화

```rust
// [Task #464] current_column == 0 가드 제거: col 1 (마지막 단) 에서도
// vpos-reset 인코딩된 col_break 를 감지해 페이지 break 를 트리거.
// paginate_multicolumn_paragraph 는 이미 advance_column_or_new_page 사용.
let col_breaks = if st.col_count > 1 {
    Self::detect_column_breaks_in_paragraph(para)
} else {
    vec![0]
};
```

`paginate_multicolumn_paragraph:929` 는 이미 `st.advance_column_or_new_page()` 를 사용하므로 추가 수정 불필요.

## 2. 결함 검증

### 2.1 LAYOUT_OVERFLOW 사라짐

```bash
./target/release/rhwp export-svg samples/exam_kor.hwp 2>&1 | grep "OVERFLOW" | wc -l
→ 0
```

수정 전 16 건 → 수정 후 **0 건**.

### 2.2 dump-pages 페이지 분리 정상

수정 전:
```
페이지 15: pi=101, pi=102, pi=103 [전체], pi=104, pi=105 [모두 col 1 누적, overflow]
```

수정 후:
```
페이지 15: PartialParagraph pi=103 lines=0..3
페이지 16: PartialParagraph pi=103 lines=3..6
            FullParagraph    pi=104
            FullParagraph    pi=105
```

PDF 정답과 일치.

### 2.3 시각 검증

페이지 16 col 0 시작에 pi=103 lines 3.. + pi=104 + pi=105 정상 layout (Chrome 헤드리스 캡처 확인).

## 3. 회귀 검증

### 3.1 5종 다단 샘플 56 SVG 비교

```
변경 파일:
  exam_kor/exam_kor_015.svg  (의도된 변경 — pi=103 lines 0..3 만)
  exam_kor/exam_kor_016.svg  (의도된 변경 — pi=103 lines 3..6 + pi=104, 105)

다른 54 SVG 동일 — 회귀 0건.
```

| 샘플 | 페이지 | 변경 | 비고 |
|---|---|---|---|
| exam_kor | 20 | 2 | 의도된 (15p, 16p) |
| exam_eng | 8 | 0 | |
| exam_math | 20 | 0 | |
| exam_science | 4 | 0 | |
| exam_social | 4 | 0 | |

### 3.2 페이지 수 변동 0

exam_kor.hwp 페이지 수: **20 페이지 유지** (수정 전후 동일).

### 3.3 cargo test 통과

```
test result: ok. 1094 passed; 0 failed; 1 ignored; 0 measured
```

기타 통합 테스트 모두 통과.

## 4. 단계 4 진입

자동승인. 단계 4 (최종 정리: clippy + 보고서 + orders + 커밋) 즉시 진입.
