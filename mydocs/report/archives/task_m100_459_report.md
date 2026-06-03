# Task #459 최종 결과보고서: 다단 후속 페이지에서 LINE_SEG vpos-reset 단 경계 미인식

## 요약

다단 구역이 여러 페이지에 걸칠 때, **첫 페이지가 아닌 후속 페이지**의 좌측 단 → 우측 단 단 경계 인코딩(LINE_SEG의 `vertical_pos` 리셋)이 무시되어 좌측 단 하단이 col_bottom을 초과하여 그려지던 버그를 수정.

`samples/exam_kor.hwp` 페이지 2 좌측 단의 `pi=39` 문단 4줄 중 마지막 2줄이 col_bottom을 39px 초과해 그려지던 문제를 해결. HWP 원본의 LINE_SEG vpos 리셋 위치(`ls[2]`)대로 정확히 분할되어 좌측 단 2줄, 우측 단 5줄로 배치됨.

## 변경 파일

- `src/renderer/typeset.rs:856` (기본 페이지네이션 경로)
- `src/renderer/pagination/engine.rs:607` (RHWP_USE_PAGINATOR=1 fallback)

두 엔진 모두 동일한 가드를 가지고 있어 양쪽 일관성 유지.

## 수정 내용

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

## 원인 분석

- `on_first_multicolumn_page`은 새 페이지 시작 시 `state.rs:218`에서 false로 리셋됨.
- 다단 구역이 페이지를 넘어가면 후속 페이지에서 flag = false → `detect_column_breaks_in_paragraph` 호출이 차단됨.
- 결과: HWP가 LINE_SEG vpos 리셋으로 인코딩한 단 경계가 무시되고, 높이 기반 폴백이 사용됨.
- 높이 기반 폴백은 `available_now` 안에 들어가는 줄을 모두 좌측 단에 배치 → HWP 원본과 다른 위치에서 분할 → col_bottom 초과.

`git log -G "on_first_multicolumn_page"` 결과 최초 커밋 한 번만 나와 가드 도입 이유는 코드 주석/문서에 기록 없음. 초기 보수적 설계의 흔적으로 판단.

## 안전성

- `detect_column_breaks_in_paragraph`는 vpos가 감소하는 경우만 단 경계로 감지하므로, 단일 컬럼 문단이나 vpos-reset이 없는 다단 문단에는 영향 없음 (`col_breaks=[0]`만 반환 → 분기 진입하지 않음).
- `current_column == 0` 조건은 유지 — 이미 우측 단에 있는 paragraph는 단 경계 의미 없음.
- `paginate_multicolumn_paragraph` / `typeset_multicolumn_paragraph`는 `advance_column_or_new_page()`를 사용하여 후속 페이지에서도 단 진행 가능 (특별한 first-page 가정 없음).

## 회귀 검증

### 다단 샘플 5종

| 샘플 | 페이지 수 (전→후) | LAYOUT_OVERFLOW (전→후) |
|------|-------------------|------------------------|
| exam_kor.hwp | 20 → 20 | 36 → 16 |
| exam_eng.hwp | 8 → 8 | 9 → 9 |
| exam_math.hwp | 20 → 20 | 0 → 0 |
| exam_science.hwp | 4 → 4 | 5 → 5 |
| exam_social.hwp | 4 → 4 | 4 → 4 |

- exam_kor에서 20건 overflow 해소 (페이지 2/5/8의 다단 vpos-reset 케이스 자동 해소).
- 다른 샘플들 SVG 동일 — 회귀 0.

### exam_kor 페이지 2 (목표 케이스)

| | 단 0 (좌측) | 단 1 (우측) |
|---|---|---|
| 변경 전 | pi=39 lines=0..4 (4줄, 39px overflow) | pi=39 lines=4..7 (3줄) |
| 변경 후 | pi=39 lines=0..2 (2줄) | pi=39 lines=2..7 (5줄) |
| HWP 원본 | ls[0..2] vpos=86417,88255 (2줄) | ls[2..7] vpos=0,1838,3676,5514,7352 (5줄) |

수정 후 HWP 원본의 LINE_SEG 인코딩과 정확히 일치.

### 자동 해소된 추가 케이스

| 페이지 | 문단 | vpos-reset 위치 | 결과 |
|--------|------|----------------|------|
| 5 | pi=157 | ls[4] | 0..4 / 4..15 |
| 8 | pi=297 | ls[5] | 0..5 / 5..11 |

모두 HWP 인코딩 그대로 분할.

### `cargo test --release`

- **총 1120 tests passed, 0 failed**
- 회귀 테스트 모두 통과 (svg_snapshot, exam_eng_multicolumn 포함).

## 결론

- 목표 버그(exam_kor 페이지 2 좌측 단 overflow) 해결.
- 같은 원인의 다른 케이스(페이지 5/8) 자동 해소.
- 다단 회귀 0건, 페이지 수 변동 0건, 단위/통합 테스트 1120건 모두 통과.

## 후속 과제

- exam_kor 잔여 16건 overflow는 본 이슈와 별개 원인으로 추정 (다른 paragraph indices). 별도 이슈로 분류 필요.
