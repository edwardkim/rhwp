# Task #462 최종 결과보고서: TAC Picture 인라인 문단 line advance 누락

## 요약

빈 문단 + TAC `Control::Picture` 만 있는 인라인 그림 문단의 line advance 가 이미지 박스 높이만 사용하고 LINE_SEG 의 `line_height + line_spacing` 을 무시하던 버그를 수정. exam_kor 페이지 1 우측 단의 pi=25 (그림 박스) ↔ pi=26 ("①-독서 목적을 고려하면…") 사이 여백이 ~0px → **20.51px** 로 복원되어 PDF 와 정합.

## 변경 파일

- `src/renderer/layout.rs:2724` 및 동일 로직의 `2727` (already_registered 분기)

## 수정 내용

```rust
// 변경 전
result_y = pic_y + pic_h;

// 변경 후
// [Task #462] LINE_SEG 의 lh+ls 를 advance 로 사용 — 이미지 박스
// 높이만 사용하면 leading + line_spacing 이 누락되어 다음 문단이
// 그림 바로 아래에 붙음. max(pic_h) 는 LINE_SEG 가 비정상적으로
// 작은 경우의 안전장치.
let line_advance = para.line_segs.first()
    .map(|ls| hwpunit_to_px(ls.line_height + ls.line_spacing, self.dpi))
    .unwrap_or(pic_h);
result_y = pic_y + line_advance.max(pic_h);
```

## 원인

- `Control::Picture` (그림) 의 TAC 인라인 경로 (`layout.rs:2628~2728`) 가 `result_y = pic_y + pic_h` 로 다음 문단 진행을 결정.
- 이미지 박스 높이 (`pic_h = 23675 HU = 315.67 px`) 와 HWP 의 LINE_SEG 가 정한 줄 높이 (`lh = 24525 HU = 327 px`) 사이에 leading (~11.33 px) 이 존재하고, 추가로 `line_spacing` (`ls = 688 HU = 9.17 px`) 이 다음 문단까지의 간격을 결정함.
- 두 값을 누락하여 합 **20.51 px 의 화이트스페이스가 이미지 아래에서 사라지고**, 후속 문단이 이미지 바로 밑에 붙음.

## 안전성

- TAC Picture 가 없는 문단은 이 분기에 진입하지 않으므로 영향 없음 (`if let Control::Picture(pic) = ctrl` + `pic.common.treat_as_char` 가드).
- `max(pic_h)` 는 LINE_SEG 가 비정상적으로 작은 값을 가질 때 이미지가 잘리지 않도록 하한선 보장.
- `Control::Shape` 의 TAC 보정 (`layout.rs:1929-1958`) 은 기존대로 유지 — 두 경로는 서로 다른 컨트롤 타입을 처리.

## 검증 결과

### exam_kor 페이지 1 (목표 케이스)

| 항목 | 변경 전 | 변경 후 |
|------|---------|---------|
| pi=25 이미지 top | 797.91 | 797.91 |
| pi=25 이미지 bottom | 1113.58 | 1113.58 |
| pi=26 baseline | 1129.36 | **1149.87** |
| 그림과 본문 갭 | ~0 px | **20.51 px** |

HWP 소스 LINE_SEG `lh+ls = 25213 HU = 336.17 px` advance 정합.

### 회귀 검증

| 샘플 | 페이지 수 (전→후) | LAYOUT_OVERFLOW (전→후) |
|------|-------------------|------------------------|
| exam_kor.hwp | 20 → 20 | 16 → 19 (+3) |
| exam_eng.hwp | 8 → 8 | 9 → 11 (+2) |
| exam_math.hwp | 20 → 20 | 0 → 0 |
| exam_science.hwp | 4 → 4 | 5 → 5 |
| exam_social.hwp | 4 → 4 | 4 → 4 |

- **페이지 수 모든 샘플 동일** (회귀 0).
- 신규 overflow 5건은 본 수정의 정당한 부작용:
  - exam_kor pi=77 page 2 col 0: +7.3px
  - exam_kor pi=103/104/105 page 3 col 1: 기존 overflow 가 ~+20px 증가 (이미 큰 overflow 였음)
  - exam_eng pi=194 page 3 col 1: +11.5px
- HWP 의 정확한 advance 를 재현하기 시작하면서, 변경 전 잘못 작은 advance 로 col_bottom 안에 들어갔던 케이스가 정직하게 살짝 넘게 됨. 시각적으로는 PDF 와 더 가까워짐.

### `cargo test --release`

- **1120 passed, 0 failed**

## 결론

- 목표 버그(exam_kor 페이지 1 우측 단 그림 ↔ 본문 여백) 해결.
- 같은 메커니즘의 모든 TAC Picture 인라인 케이스에서 PDF 정합성 향상.
- 페이지 수 변동 0, 단위/통합 테스트 1120건 모두 통과.

## 후속 과제

- exam_kor pi=103/104/105 (페이지 3 우측 단) 의 큰 overflow 는 본 이슈와 별개로 다른 원인 — 별도 조사 필요.
- TAC `Control::Shape` 의 보정 (`layout.rs:1929-1958`) 에도 `line_spacing` 이 누락되어 있을 수 있음 — 별도 이슈로 추적 검토.
