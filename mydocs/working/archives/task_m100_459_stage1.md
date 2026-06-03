# Task #459 Stage 1 보고서: `on_first_multicolumn_page` 가드 제거

## 변경 내용

### 1. `src/renderer/typeset.rs:856` (기본 경로, TypesetEngine)

```rust
// 변경 전
let col_breaks = if st.col_count > 1 && st.current_column == 0 && st.on_first_multicolumn_page {
    Self::detect_column_breaks_in_paragraph(para)
} else {
    vec![0]
};

// 변경 후
// [Task #459] on_first_multicolumn_page 가드 제거: 다단 구역이 여러 페이지에 걸칠 때
// 후속 페이지에서도 LINE_SEG vpos-reset 으로 인코딩된 단 경계를 인식해야 함.
let col_breaks = if st.col_count > 1 && st.current_column == 0 {
    Self::detect_column_breaks_in_paragraph(para)
} else {
    vec![0]
};
```

### 2. `src/renderer/pagination/engine.rs:607` (RHWP_USE_PAGINATOR=1 fallback)

동일한 패턴으로 가드 제거. 두 엔진 모두 동일 시멘틱 유지.

## 디버깅 노트

- 처음에 `pagination/engine.rs`만 수정 → 효과 없음. 디폴트 경로가 `TypesetEngine`(`typeset.rs`)임을 `rendering.rs:883` 의 `RHWP_USE_PAGINATOR` 환경변수 확인 후 발견.
- `typeset.rs`의 동일 가드를 함께 수정하여 양 엔진 일관성 유지.

## 검증 결과 (회귀 베이스라인 vs 수정 후)

| 샘플 | 페이지 수 (전→후) | LAYOUT_OVERFLOW (전→후) |
|------|-------------------|------------------------|
| exam_kor.hwp | 20 → 20 | 36 → 16 |
| exam_eng.hwp | 8 → 8 | 9 → 9 |
| exam_math.hwp | 20 → 20 | 0 → 0 |
| exam_science.hwp | 4 → 4 | 5 → 5 |
| exam_social.hwp | 4 → 4 | 4 → 4 |

- exam_kor에서 20건 overflow 해소 (페이지 2/5/8의 다단 vpos-reset 케이스).
- 다른 샘플들 SVG 동일 (회귀 0).
- 페이지 수 모든 샘플에서 동일 유지.

## exam_kor 페이지 2 (목표 케이스) 상세

```
변경 전: 단 0 pi=39 lines=0..4 / 단 1 pi=39 lines=4..7 → 좌측 39px overflow
변경 후: 단 0 pi=39 lines=0..2 / 단 1 pi=39 lines=2..7 → overflow 없음
```

HWP 원본 LINE_SEG의 vpos 리셋 위치(ls[2])와 정확히 일치.

## 추가로 자동 해소된 케이스

| 페이지 | 문단 | 원래 분할 | 수정 후 분할 |
|--------|------|-----------|--------------|
| 2 | pi=39 | 0..4 / 4..7 | 0..2 / 2..7 |
| 5 | pi=157 | (height-based) | 0..4 / 4..15 (vpos-reset@line4) |
| 8 | pi=297 | (height-based) | 0..5 / 5..11 (vpos-reset@line5) |

모두 HWP가 인코딩한 단 경계를 정확히 따름.

## 다음 단계

- `cargo test --release` 결과 확인 후 통과 시 최종 보고서 작성.
