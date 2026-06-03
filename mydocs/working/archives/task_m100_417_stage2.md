---
타스크: #417 HWP 3.0 정식 파서 구현
단계: Stage 2 — 그림 겹침 수정 + 캡션 렌더링 복원 + AutoNumber 번호 표시 수정
브랜치: local/task417
작성일: 2026-04-28
상태: 완료
---

# Stage 2 완료 보고서

## 1. 이슈

작업지시자가 `samples/hwp3-sample.hwp` WASM 빌드 후 rhwp-studio 확인 결과에서 두 가지 렌더링 오류 보고:

1. **후속 단락 겹침**: pi=76 그림 이후 텍스트(pi=77)가 그림 위로 겹쳐 표시됨
2. **그림 번호 누락**: 캡션에 그림 번호가 표시되지 않음 ("그림 . NAT에 의한..." → "그림 2. NAT에 의한..." 이어야 함)

## 2. 원인 분석

### 2.1 후속 단락 겹침 — `layout_body_picture` 반환 값 오류

`src/renderer/layout/picture_footnote.rs` `layout_body_picture()`:

```rust
// 수정 전 (잘못된 반환)
(VertRelTo::Para, _) => y_offset + total_height,
// y_offset = 앵커 단락 para_start_y (vert_offset 미포함)
// → base_y(= para_start + vert_offset)보다 vert_offset만큼 작은 위치 반환
// → 후속 단락이 그림 안으로 겹침
```

Para-relative 그림은 실제로 `base_y = para_start + vert_offset` 위치에 렌더링되지만,
반환 값이 `para_start + total_height`(= `base_y - vert_offset + total_height`)이었음.
후속 단락의 시작 y가 그림 하단보다 vert_offset(112px)만큼 낮아 텍스트가 그림 위로 겹쳤음.

### 2.2 페이지네이터 (`pagination/engine.rs`)

페이지네이터는 FullParagraph(text) + pic_h를 합산하여 단락 높이를 계산한다.
HWP3 혼합 단락(텍스트 + Para-relative 그림)에서:
- FullParagraph 높이: LINE_SEG 기반 텍스트 높이 (277px, 그림 공간 미포함)
- Shape 높이: pic_h (484px)
- 페이지네이터 합계: 761px

렌더러의 실제 결과는 `base_y + total_height = (para_start + 112) + 484 + cap_h ≈ 621px`.
페이지네이터가 761px로 과대 추정하여 보수적으로 페이지 분할 → 결과적으로 overflow 없음.

### 2.3 캡션 너비 0 (`parse_paragraph_list`)

HWP3 파서에서 모든 그림의 `caption.width = 0` (데이터 없음).
`layout_body_picture`에서 `cap_w = hwpunit_to_px(caption.width, ...)` = 0px → 캡션 박스 크기 0 → 캡션 렌더링 불가.

### 2.4 그림 번호 누락 — AutoNumber U+FFFC 미처리

- HWP5/HWPX: AutoNumber 문자(0x0012)를 공백(' ')으로 저장 → 렌더러가 `"  "` (연속 두 공백) 패턴으로 탐색
- HWP3: AutoNumber 위치를 U+FFFC(OBJECT REPLACEMENT CHARACTER, ￼)로 저장

`apply_auto_numbers_to_composed`에서 `"  "` 패턴만 처리하고 U+FFFC는 미처리 → 캡션에 번호 미표시.

### 2.5 그림 번호 불일치 — HWP3 그림 카운터 방식 차이 (추가 수정)

1차 수정 후 rhwp-studio에서 확인 결과, 그림 번호가 한컴과 1씩 차이 발생:

- rhwp: 그림 2. (pi=76) vs 한컴: **그림 3.** (pi=76)

**원인**: HWP5/HWPX에서는 `AutoNumber(type=Picture)` 컨트롤을 만날 때만 그림 카운터를 올린다.
HWP3에서는 `Control::Picture` 개체 자체(캡션 유무 무관)가 카운터를 올린다.
이 문서에는 꼬리말에 tac=true 그림(로고, bin_id=1)이 있어 그림 카운터 선점:

```
꼬리말 로고(tac=true, 캡션 없음): counters[3] = 1 (번호 미표시)
pi=41 그림: counters[3] = 2 → 캡션 "그림 2." ✓
pi=76 그림: counters[3] = 3 → 캡션 "그림 3." ✓ (한컴 일치)
pi=78 그림: counters[3] = 4 → 캡션 "그림 4."
pi=97 그림: counters[3] = 5 → 캡션 "그림 5."
```

작업지시자 확인 사항: "표번호 카운트와 그림번호 카운트의 자동번호 채번은 독립적일수 있음."
→ HWP3에서 표 카운터와 그림 카운터는 서로 독립(이 문서에는 표 AutoNumber 없음).

### 2.6 알려진 미해결 한계: 혼합 단락 내부 겹침 (HWP3 vpos↔reflow 불일치)

pi=76은 텍스트 277px + Para-relative 그림(vert_offset=112px, h=484px)이 동일 단락에 존재하는 혼합 단락.
HWP3에서 각 텍스트 줄의 vpos(절대 위치)가 그림 위/아래로 분리 배치되도록 사전 계산되어 있지만,
우리 렌더러는 텍스트를 재플로우(reflow)하여 LINE_SEG 없이 연속 배치한다.

그림을 텍스트 끝(y_offset)으로 이동하면 pi=76 영역의 렌더러 높이가 paginator 예측치를 초과하여
pi=77~ 연쇄 overflow 발생. 이 문제는 vpos 기반 텍스트 위치 계산 구현 없이는 해결 불가.
**별도 GitHub 이슈 등록 필요.**

## 3. 수정 내용

### 3.1 `src/renderer/layout/picture_footnote.rs` — base_y 기반 반환

```rust
// 수정 후: vert_offset이 적용된 실제 그림 상단 y(base_y)를 기준으로 반환
(VertRelTo::Para, _) => base_y + total_height,
```

### 3.2 `src/renderer/pagination/engine.rs` — 주석 추가 (코드 변경 없음)

```rust
// Para-relative TopAndBottom 그림은 렌더러에서 텍스트 렌더링 후(y_offset)에
// 배치되므로 vert_offset은 이미 FullParagraph 높이에 포함된다.
// 따라서 pic_h만 추가한다.
st.current_height += pic_h + margin_top + margin_bottom;
```

### 3.3 `src/parser/hwp3/mod.rs` — caption.width 보정 (단독 그림/혼합 단락 모두)

```rust
// tac=false + TopAndBottom 그림의 caption.width = 0 → pic.common.width로 보정
if caption.width == 0 {
    caption.width = pic.common.width;
}
```

### 3.4 `src/renderer/layout/paragraph_layout.rs` — HWP3 AutoNumber U+FFFC 처리

```rust
// HWP5/HWPX: AutoNumber 문자(0x0012)를 공백(' ')으로 저장 → "  " 패턴 탐색
// HWP3: AutoNumber 위치를 U+FFFC(OBJECT REPLACEMENT CHARACTER)로 저장 → '\u{fffc}' 탐색
for line in &mut composed.lines {
    for run in &mut line.runs {
        if let Some(pos) = run.text.find("  ") {
            run.text = format!("{}{}{}", &run.text[..pos+1], num_str, &run.text[pos+1..]);
            return;
        }
        if run.text.contains('\u{fffc}') {
            run.text = run.text.replacen('\u{fffc}', &num_str, 1);
            return;
        }
    }
}
```

### 3.5 `src/parser/hwp3/mod.rs` + `src/parser/mod.rs` — HWP3 그림 카운터 방식 수정

HWP3 문서 판별을 위해 `doc.header.version.major = 3` 설정:

```rust
// src/parser/hwp3/mod.rs
let mut doc = Document::default();
doc.header.version.major = 3;  // HWP3 자동번호 카운팅 방식 표시
```

`assign_auto_numbers_in_controls`에 `is_hwp3: bool` 파라미터 추가.
HWP3 모드에서 `Control::Picture` 자체가 그림 카운터를 올리고,
캡션의 `AutoNumber(type=Picture)`는 재증가 없이 현재 카운터 값을 사용:

```rust
Control::Picture(pic) if is_hwp3 => {
    counters[3] += 1;   // 그림 개체마다 카운터 증가 (캡션 유무 무관)
    let pic_num = counters[3];
    // 캡션 AutoNumber(Picture)에 현재 카운터 값 직접 할당 (재증가 없음)
    ...
}
```

HWP5/HWPX는 기존 동작 유지 (캡션 AutoNumber를 만날 때만 카운터 증가).

## 4. 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1016 passed / 0 failed / 1 ignored |
| `cargo clippy -- -D warnings` | clean |
| `rhwp export-svg samples/hwp3-sample.hwp` | 20 SVG 생성 성공 |

### 4.1 후속 단락 겹침 수정 확인 (hwp3-sample_008.svg, page 8)

- pi=76 그림: y=316.3 ~ y=800.3 (h=484px)
- 캡션: y=810 ✓ (그림 하단+9.7px)
- pi=77 텍스트: y=835 ✓ (그림 하단 이후, 겹침 없음)

### 4.2 캡션 렌더링 확인

- page 3: 캡션 렌더링 ✓ (이전 0px 박스 문제 해결)
- page 8: 캡션 2건 렌더링 ✓

### 4.3 AutoNumber 수정 확인

- HWP3 캡션 텍스트의 U+FFFC가 번호 문자열로 치환되어 표시됨
- 그림 번호: page 3=그림2., page 8=그림3./그림4., page 10=그림5. ✓ (한컴과 일치)
- 기존 HWP5/HWPX의 `"  "` 패턴 처리는 유지됨

### 4.4 기존 알려진 LAYOUT_OVERFLOW (Stage 1 이래 동일, 미해결)

| 위치 | overflow | 원인 |
|------|----------|------|
| 꼬리말 (전 페이지) | 36px | footer height(15mm) < fixed_line_spacing(24.5mm) |
| page 3 pi=41 Shape | 136px | HWP3 vpos↔reflow 불일치 |
| page 8 pi=78 Shape | 124px | HWP3 vpos↔reflow 불일치 |
| page 10 pi=97 Shape | 16px | HWP3 vpos↔reflow 불일치 |
| pi=76 혼합 단락 내부 | — | 텍스트와 그림 겹침 (vpos 미사용), 별도 이슈 필요 |

## 5. 커밋 대상 파일

- `src/parser/hwp3/mod.rs` — caption.width 보정 + `version.major=3` 표시
- `src/parser/mod.rs` — HWP3 그림 카운터 방식 분기 (`is_hwp3`)
- `src/renderer/layout/picture_footnote.rs` — base_y 기반 y_offset 반환
- `src/renderer/pagination/engine.rs` — 주석 추가 (코드 변경 없음)
- `src/renderer/layout/paragraph_layout.rs` — HWP3 AutoNumber U+FFFC 처리
- `mydocs/working/task_m100_417_stage2.md` — 이 보고서

## 6. 다음 단계

Stage 3: 하이퍼링크 URL 추출 구현 (추가정보블록 #1 TagID 3, `src/parser/hwp3/mod.rs` ~938번)
