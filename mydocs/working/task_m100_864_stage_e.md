# Task #864 Stage E HWP3 그림 caption 위치 정정

**작성일**: 2026-05-13
**브랜치**: `local/task864`

## E.1 정정 구현

### 변경 파일

**`src/renderer/layout.rs`** (treat_as_char inline picture caption y 계산):

```diff
                         if let Some(ref caption) = pic.caption {
                             use crate::model::shape::CaptionDirection;
                             let caption_spacing = hwpunit_to_px(caption.spacing as i32, self.dpi);
                             let caption_h = self.calculate_caption_height(&pic.caption, styles);
+                            // [Task #864 Stage E] inline (TAC) image 의 실제 layout 진행 높이는
+                            // LINE_SEG 의 line_height (이미지 박스 + leading). pic_y + pic_h
+                            // 사용 시 image render 영역 안에 caption 이 겹침 (HWP3 sample14
+                            // page 3 "Cut&Paste 할 영역" 결함). image_advance 사용.
+                            let image_advance = para.line_segs.first()
+                                .map(|ls| hwpunit_to_px(ls.line_height, self.dpi))
+                                .unwrap_or(pic_h)
+                                .max(pic_h);
                             let cap_y = match caption.direction {
-                                CaptionDirection::Bottom => pic_y + pic_h + caption_spacing,
+                                CaptionDirection::Bottom => pic_y + image_advance + caption_spacing,
                                 CaptionDirection::Top => pic_y,
-                                _ => pic_y + pic_h + caption_spacing,
+                                _ => pic_y + image_advance + caption_spacing,
                             };
```

**dump 출력 보강** (`src/main.rs`): picture caption 정보 표시 추가 (대상 image 의 caption 진단/검증 편의).

```rust
if let Some(ref cap) = pic.caption {
    let cap_text: String = cap.paragraphs.iter().map(|p| p.text.clone()).collect::<Vec<_>>().join("|");
    println!("{}  caption: dir={:?} width={} paras={} text={:?}",
        prefix, cap.direction, cap.width, cap.paragraphs.len(), cap_text);
}
```

## E.2 검증

### 빌드

```bash
$ cargo build --release
    Finished `release` profile [optimized] target(s) in 20.63s
```

### 테스트

```bash
$ cargo test --release --lib
test result: ok. 1230 passed; 0 failed; 2 ignored; 0 measured
```

회귀 0.

### Clippy

경고 0.

### hwp3-sample14 페이지 3 시각 정합

**변경 전**: "Cut&Paste 할 영역" 텍스트가 BMP image 영역 (132.3 ~ 288.5 px) 안 y=241 에 그려져 BMP 안에 겹침 → 시각적으로 보이지 않음.

**변경 후**: caption y = 132.3 + 156.2 + 0 = 288.5 px → BMP 아래에 정상 표시.

→ **한컴 PDF page 3 정합** ✓

### 다른 페이지/sample 회귀

- hwp3-sample14 page 2: 기존 (Stage A-C) 정정 정합 유지 ✓
- hwp3-sample14 전체 11 페이지: 시각 회귀 0
- hwp3-sample4: caption text 가 모두 빈 상태 → 변경 무시각효과 (line_advance 와 pic_h 차이는 회귀 없음)

## E.3 영향 범위

`pic.caption` 가 존재하는 모든 inline (treat_as_char) HWP3 picture 에 영향.

| 조건 | 변경 전 | 변경 후 |
|---|---|---|
| line_height ≤ pic.common.height | image_advance = pic_h → 동일 | 동일 |
| line_height > pic.common.height (HWP3 일반) | caption 이 image 안 겹침 | caption 이 image 아래 정상 |

대부분의 HWP3 inline image 가 `line_height > image_height` 이므로 본 정정은 대부분 케이스에서 시각 개선.

## Stage E 결론

inline picture caption 의 y 계산을 `pic.common.height` 대신 LINE_SEG.line_height (실제 layout 진행 높이) 사용으로 정정. HWP3 sample14 page 3 의 "Cut&Paste 할 영역" caption 정상 표시.

📋 **Stage E 완료. 종합 보고 + 커밋 승인 요청 진행합니다.**
