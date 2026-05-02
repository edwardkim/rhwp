---
타스크: #460 HWP3 파서 렌더러 중립 재구현
문서: 구현계획서
브랜치: local/task460
작성일: 2026-04-29
상태: 승인 대기
---

# 구현 계획서 — Task #460

## 브랜치 기준

`local/task460` = `origin/devel` 기준.
이 브랜치에는 Task #417 렌더러 변경(U+FFFC)만 존재. Task #425 Stage 1/2/3 없음(미merge).

## Stage 1: AutoNumber U+FFFC → 공백 변환 (파서 처리)

### 대상
`src/parser/hwp3/mod.rs` 줄 201-270 (`18..=21` match arm)

### 현황
chars 18-21 모두 `text_string.push('\u{FFFC}')` 후 control 생성.
ch=18(AutoNumber)만 renderer에서 FFFC를 탐색해 번호로 치환 (paragraph_layout.rs 2792-2794).

### 변경
```rust
// 수정 전 (줄 206-207):
char_offsets.push(utf16_len);
utf16_len += 1;
text_string.push('\u{FFFC}');

// 수정 후:
char_offsets.push(utf16_len);
utf16_len += 1;
// AutoNumber(ch=18)은 HWP5 패턴(" ")과 일치하도록 공백으로 저장
if ch == 18 {
    text_string.push(' ');
} else {
    text_string.push('\u{FFFC}');
}
```

`ch == 18`일 때 `' '`(공백)을 push → 캡션 텍스트 "그림 " + `' '` = `"그림  "` → `"  "` 패턴 매칭 ✓

### 렌더러 정리
`src/renderer/layout/paragraph_layout.rs` 줄 2785-2795:
- HWP3 주석(줄 2785) 제거
- `'\u{fffc}'` 분기(줄 2792-2795) 제거

```rust
// 수정 후 (줄 2783-2797):
// 각 줄의 텍스트에서 AutoNumber 위치를 찾아 번호로 대체
// HWP5/HWPX/HWP3 공통: 공백 두 개("  ") 패턴 탐색
for line in &mut composed.lines {
    for run in &mut line.runs {
        if let Some(pos) = run.text.find("  ") {
            run.text = format!("{}{}{}", &run.text[..pos+1], num_str, &run.text[pos+1..]);
            return;
        }
    }
}
```

### 검증
- `cargo test --lib` 통과
- `export-svg hwp3-sample.hwp` → 캡션 AutoNumber 번호 정상 출력 확인

### 커밋
`Task #460 Stage 1: HWP3 AutoNumber U+FFFC → 공백 (파서 처리, 렌더러 분기 제거)`

---

## Stage 2: 혼합 단락 LINE_SEG 높이 보정 (파서 처리)

### 대상
`src/parser/hwp3/mod.rs` 줄 1253 이후 (line_segs 확정 직후)

### 현황
Para-relative TopAndBottom 그림이 있는 단락에서:
- LINE_SEGs vpos=0으로 초기화됨
- `document.rs`의 `need_vpos_recalc`가 순차 누적으로 덮어씀
- 줄 6-12의 para-relative 위치가 그림 구역[8400, 44700] HU 내 → 그림과 텍스트 겹침

### 해결 메커니즘

마지막 "그림 위쪽" LINE_SEG의 `line_height`를 그림 하단까지의 거리로 확장:
- `compose_paragraph`가 `ComposedLine.line_height = line_seg.line_height`로 복사
- 렌더러 순차 `y += line_height`가 자동으로 그림 하단으로 점프
- `document.rs` advance = `lh + ls` (th=0이므로 조건 FALSE) → 올바른 vpos 누적

### 수치 검증 (pi=76)
```
LINE_SEG advance = lh+ls when th==lh (=1600 HU)
pos[0]=0, pos[1]=1600, ..., pos[5]=8000 HU  ← fig_top=8400 미만 → 마지막 위쪽 seg
pos[6]=9600 HU  ← fig_top=8400 이상 → 그림 구역 진입

수정: seg[5].line_height = fig_bottom(44700) - pos[5](8000) = 36700 HU
      seg[5].text_height = 0      (advance = lh+ls = 36700 확보)
      seg[5].line_spacing = 0

렌더러 y += line_height:
  줄 5: y=y_start+106.7px, line_height=489.3px → 줄 6: y_start+596px ✓

document.rs 재계산:
  seg[5] advance: th=0 → lh+ls = 36700 HU ✓
  총 advance: 8000+36700+11200 = 55900 HU → 올바른 단락 높이 ✓
  seg_lh_total=55900 > obj_total=44700 → 추가 보정 없음 ✓
```

### 삽입 위치

줄 1253 (`para.line_segs = ...` 완료) 직후, 줄 1255 (기존 후처리 주석) 이전:

```rust
// HWP3 혼합 단락: Para-relative TopAndBottom 그림 구역 내 줄을 그림 하단 아래로 재배치.
// 마지막 "그림 위쪽" LINE_SEG의 line_height를 그림 하단까지 확장 →
// compose_paragraph/렌더러의 순차 y+=line_height가 그림 구역을 자동 점프.
fixup_hwp3_mixed_para_line_segs(&mut para);
```

### 헬퍼 함수 (mod.rs 하단 또는 별도 블록)

```rust
fn fixup_hwp3_mixed_para_line_segs(para: &mut crate::model::paragraph::Paragraph) {
    use crate::model::control::Control;
    use crate::model::shape::{TextWrap, VertRelTo};

    // Para-relative TopAndBottom 비-TAC 그림 구역 탐색
    let Some((fig_top_hu, fig_bottom_hu)) = para.controls.iter().find_map(|c| {
        if let Control::Picture(p) = c {
            if !p.common.treat_as_char
                && p.common.text_wrap == TextWrap::TopAndBottom
                && p.common.vert_rel_to == VertRelTo::Para
                && p.common.height > 0
            {
                Some((p.common.vertical_offset as i32,
                      p.common.vertical_offset as i32 + p.common.height as i32))
            } else { None }
        } else { None }
    }) else { return };

    if para.line_segs.len() <= 1 { return }

    // LINE_SEG 누적 위치 계산 (document.rs advance 공식과 동일)
    let mut pos: i32 = 0;
    let mut split_idx: Option<usize> = None;
    for (i, seg) in para.line_segs.iter().enumerate() {
        let advance = if seg.text_height > 0 && (seg.text_height as i32) < (seg.line_height as i32) {
            seg.text_height as i32 + seg.line_spacing as i32
        } else {
            seg.line_height as i32 + seg.line_spacing as i32
        };
        if pos < fig_top_hu && pos + advance > fig_top_hu {
            split_idx = Some(i);
            break;
        }
        pos += advance;
    }

    let Some(idx) = split_idx else { return };

    // 마지막 그림-위쪽 LINE_SEG: line_height를 그림 하단까지 확장
    let gap = fig_bottom_hu - pos;
    if gap <= 0 { return }

    let seg = &mut para.line_segs[idx];
    seg.line_height = gap as u16;
    seg.text_height = 0;   // advance = lh+ls 보장
    seg.line_spacing = 0;
}
```

### 검증
- `cargo test --lib` 통과 (HWP5/HWPX 회귀 없음)
- `export-svg hwp3-sample.hwp` → 그림-텍스트 겹침 없음
- `dump-pages hwp3-sample.hwp -p N` → 혼합 단락 높이 55900 HU ✓

### 커밋
`Task #460 Stage 2: HWP3 혼합 단락 LINE_SEG 높이 보정 (파서 처리, 그림 겹침 해소)`

---

## Stage 3: 최종 검증 + 보고서

### 검증 항목

| 항목 | 명령 | 기준 |
|------|------|------|
| 단위 테스트 | `cargo test --lib` | 기존 통과 수 유지, 0 failed |
| Clippy | `cargo clippy -- -D warnings` | 경고 없음 |
| HWP3 SVG | `rhwp export-svg samples/hwp3-sample.hwp` | 그림 겹침 없음, AutoNumber 정상 |
| HWP5/HWPX 회귀 | 샘플 10종 byte 비교 | 변화 없음 |

### 산출물
- `mydocs/working/task_m100_460_stage1.md`
- `mydocs/working/task_m100_460_stage2.md`
- `mydocs/report/task_m100_460_report.md`
- `mydocs/orders/20260429.md` #460 상태 갱신

### 커밋
`Task #460 Stage 3: 최종 검증 + 결과 보고서`

---

## 수정 파일 요약

| 파일 | 변경 종류 |
|------|---------|
| `src/parser/hwp3/mod.rs` | AutoNumber ch=18 → `' '` 치환 + fixup 함수 추가 |
| `src/renderer/layout/paragraph_layout.rs` | U+FFFC 분기 제거 (4줄) |
