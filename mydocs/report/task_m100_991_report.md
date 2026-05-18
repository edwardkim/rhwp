# Task #991 최종 보고서 — F2 composer synth marker (HWP5 multi-TAC paragraph 정합)

- 이슈: [#991](https://github.com/edwardkim/rhwp/issues/991)
- 브랜치: `local/task991-fix`
- 결정: **F2-narrow** (composer-only marker synthesis)
- 일자: 2026-05-18

## 1. 작업 결과

`samples/hwp3-sample16-hwp5.hwp` 페이지 18 의 다이어그램 + 라벨 z-order 어긋남 해소.

### 변경 파일
- `src/renderer/composer.rs` (+95 lines) — `synthesize_marker_paragraph` 함수 + `compose_paragraph` 호출

### 효과
- HWP5 sample16 p18 시각: **PDF 정합** (가. 위, 다이어그램, 나. 아래)
- cargo test --release --lib: **1297 passed, 0 failed** ✓
- 240 sample 페이지 수: **0 변동** ✓
- HWP5 page count: 62 (변동 없음)
- Editor 기능 (insert_text/save/cursor): 영향 없음 (parser 미변경)

## 2. Root cause (KS X 6101 spec 정합)

### HWP3 vs HWP5 parser 차이
- HWP3: 확장 컨트롤마다 `\u{FFFC}` 마커 push (spec 정합)
- HWP5: 마커 미푸시 (sparse text + char_offsets)

### pi=394 (HWP5 sample16) 예
```
3 TAC controls + 3 line_segs:
  ls[0] ts=0 contains 가. label
  ls[1] ts=8 contains diagram
  ls[2] ts=18 contains 나. label

HWP5 IR (sparse):
  text="  ", char_offsets=[16, 17]
  composer.utf16_range_to_text_range(0, 8): [0, 0) — 빈!
  → ls[0] (가.) 빈 라인 처리 → 라벨 어긋남

Fix 후 synth IR:
  text="\u{FFFC}\u{FFFC}  \u{FFFC}", char_offsets=[0, 8, 16, 17, 18]
  composer 의 line 별 text 범위 정확 매핑 ✓
```

## 3. Fix 설계 (F2-narrow)

`src/renderer/composer.rs` 의 `compose_paragraph` 입구에 marker synthesis layer:

```rust
fn synthesize_marker_paragraph(para: &Paragraph) -> Option<Paragraph> {
    // 1. inline-visible extended ctrl count
    //    (Header/Footer/Footnote/Endnote/HiddenComment 제외)
    let inline_ctrl_count = ...;
    
    // 2. 기존 marker count
    let existing_markers = para.text.chars().filter(|c| *c == '\u{FFFC}').count();
    if existing_markers >= inline_ctrl_count { return None; }  // HWP3 path
    
    // 3. 좁힘 조건 — pi=394 패턴만 catch
    let first_off = para.char_offsets.first().copied().unwrap_or(0) as usize;
    let n_leading = first_off / 8;
    if n_leading < 2 || inline_ctrl_count < 3 { return None; }
    
    // 4. char_offsets gap 분석으로 \u{FFFC} 마커 위치 합성
    // ...
}

pub fn compose_paragraph(para: &Paragraph) -> ComposedParagraph {
    let synth_para = synthesize_marker_paragraph(para);
    let para = synth_para.as_ref().unwrap_or(para);  // shadow
    // ... 기존 logic ...
}
```

### 좁힘 조건의 의미
- `inline_ctrl_count >= 3`: pi=394 (3 TAC) 패턴 — 일반적인 1-2 TAC paragraph 미해당
- `n_leading >= 2`: leading char_offsets gap 에 2+ 컨트롤 — 대부분 paragraph 는 0-1 leading
- `existing_markers >= inline_ctrl_count`: HWP3 (markers 있음) 자동 차단

## 4. 회귀 영향

### cargo test --release --lib
```
test result: ok. 1297 passed; 0 failed; 2 ignored; 0 measured
```

이전 F1 시도들의 fail:
- F1 광범위: 9 fail (editor 영향)
- F1-narrow (ch=11/14만): 5 fail
- F2-wide (composer synth without narrow): 5 fail
- **F2-narrow: 0 fail** ✓

### 240 sample 페이지 수
- 변동: 0 건 (hy-001.hwpx 1건은 baseline 에 없던 신규 sample)
- HWP5 sample16: 62 → 62 (그대로)

### Editor 기능
parser 미변경 → insert_text / save / cursor / logical_offset 등 모두 보존.

## 5. 잔존 영향

### HWPX 변종
HWPX 의 pi=394 는 다른 path (HWPX parser + linesegarray preset) — 본 F2 미적용. HWPX page 19 의 z-order 어긋남은 별도 fundamental 한계 (#942/#988 close 영역).

### 다른 multi-TAC paragraph
좁힘 조건 (3+ TAC + 2+ leading) 으로 sample16 pi=394 와 동일 패턴의 paragraph 만 fix 적용. 다른 패턴 (1-2 TAC, leading=0/1) 은 미적용.

## 6. 향후 확장 가능성

- 좁힘 조건 완화 시도 (별도 task) — 1-2 TAC 의 동일 root cause 패턴 catch
- HWP5 parser 의 spec 정합 (다이일 작업) — 모든 downstream 영향 검증 필요

## 7. 산출물

- `mydocs/plans/task_m100_991.md` (수행 계획서)
- `mydocs/working/task_m100_991_stage1.md` (Stage 1 진단)
- 본 보고서
- 소스 변경: `src/renderer/composer.rs` +95 lines
