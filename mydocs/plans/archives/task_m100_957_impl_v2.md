# 구현 계획서 V2 — Task #957 Stage 2 — Fix A 적용 계획

- 이슈: [#957](https://github.com/edwardkim/rhwp/issues/957)
- Stage 1 결과: 빈 caption + 잘못된 pic_y 의 복합 결함으로 +430.6px advance
- 선택: **Fix A** — 빈 caption 시 result_y advance skip

## 1. 변경 위치

`src/renderer/layout.rs:3470-3475` (Bottom caption 의 result_y advance 분기)

## 2. 변경 내용

### Before
```rust
if matches!(caption.direction, CaptionDirection::Bottom) {
    let cap_bottom = cap_y + caption_h;
    if cap_bottom > result_y {
        result_y = cap_bottom;
    }
}
```

### After
```rust
// [Task #957] 빈 caption (text 없음 + controls 없음) 은 SVG 에 invisible 하므로
// layout 공간 차지 안 함. 비-empty caption 만 result_y advance.
// 빈 caption (paragraphs.len()=N + text="") 시 pic_y + pic_h + caption_h 가
// phantom 위치로 누적되어 후속 paragraph 가 다음 페이지로 밀리는 결함
// (sample16 page 18 "나. 주요 과업내용" 후 본문 +430.6px advance).
let caption_is_empty = caption.paragraphs.iter().all(|p|
    p.text.chars().all(|c| c <= '\u{001F}' || c == '\u{FFFC}')
    && p.controls.is_empty()
);
if !caption_is_empty && matches!(caption.direction, CaptionDirection::Bottom) {
    let cap_bottom = cap_y + caption_h;
    if cap_bottom > result_y {
        result_y = cap_bottom;
    }
}
```

## 3. 영향 분석

### 3.1 변경 직접 영향
- pi=394 ci=1 picture: empty caption → result_y advance skip → +430.6px 제거
- caption text 보유한 pictures: 기존 동작 유지 ✓
- caption=None pictures: 본 분기 자체 미진입 (기존)

### 3.2 다른 sample 영향 (예상)
- `hwp3-sample14.hwp` page 4 "Visual Block을 이용한 대소문자 변경" (Task #864 영역) — caption text 보유, 영향 없음
- `exam_kor` p.21/37/60 의 Square wrap picture — caption 없거나 text 보유, 영향 없음
- 다른 TAC TopAndBottom picture w/o caption text — 동일 fix 적용 (의도)

## 4. 위험 평가

| 위험 | 평가 | 완화 |
|------|------|------|
| 빈 caption 이 의도적으로 frame 만 보존하는 case | **낮음** (HWP3 sample 다수에서 빈 caption 은 phantom) | Stage 4 다중 sample 검증 |
| caption_is_empty 판정의 false negative (다른 controls 포함 시) | **낮음** | `paragraphs.controls.is_empty()` 가드 |
| 회귀 가능성 | **낮음** — 정상 caption 영향 없음 | cargo test 1288 + 다중 sample |
| Caption Top direction 동작 | **없음** — Top 분기는 별도 (offset_inline_image_y), 본 fix 미영향 | - |

## 5. 검증 계획 (Stage 3-4)

### Stage 3 단위 검증
1. cargo build --release
2. sample16 page 18 SVG render:
   - pi=395 "○ 통합모델..." y 위치 확인 (목표 ~y 770~800)
   - 본문 paragraph 같은 페이지 표시 확인
3. PNG render → 한컴 viewer page 16 정합 확인

### Stage 4 회귀 검증
1. `cargo test --release --lib` 전체 (1288 tests)
2. 추가 sample SVG render + 시각 확인:
   - hwp3-sample14 (caption 보유 사례)
   - hwp3-sample10/11/13 (다이어그램)
   - exam_kor/exam_math/exam_eng
   - 시험지 (3-09월/3-10월/3-11월)
   - shortcut.hwp
3. golden SVG diff 회귀 0

## 6. Stage 5 (시각 검증 + 최종 보고서)

- 한컴 PDF (pdf/hwp3-sample16-hwp5-2022.pdf) 와 정합 비교
- rhwp-studio UI 시각 확인
- 최종 보고서 작성
- commit + PR (작업지시자 승인 후)

## 7. 진행 규칙

- 자동진행 안함
- Stage 3 종료 시 단위 검증 보고서 + 승인
- Stage 4 종료 시 회귀 검증 보고서 + 승인
- Stage 5 종료 시 최종 보고서 + PR 승인
