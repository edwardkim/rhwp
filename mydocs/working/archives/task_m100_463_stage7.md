# Task #463 Stage 7 완료보고서 (15페이지 헤더 그림 자르기)

## 발견 (Stage 6 머지 후 사용자 검토)

작업지시자가 15페이지 출력 검토 결과:
> "국어 영역(A형)(화법과 작문) -> 국어 영역(화법과 작문) : 이렇게 보여야 함"

PDF 와 비교: PDF 헤더는 "국어 영역(화법과 작문)" 인데 SVG 는 "국어 영역**(A형)**(화법과 작문)" 로 출력 — 잘못된 비트맵 우측이 노출됨.

## 원인

samples/exam_kor.hwp 의 sec1 master[1] (Odd) 헤더 1×3 표 가운데 셀에 `treat_as_char=true` 인라인 그림 (bin_id=27) 이 들어있다. 그림 속성:

| 항목 | 값 |
|------|-----|
| common_w × common_h | 11272 × 2924 HU (표시 크기) |
| original w × h | **174000 × 26580 HU** (원본 비트맵) |
| crop | l=0, t=0, **r=102473**, b=26580 |

→ 좌측 102473/174000 ≈ **58.9%** 만 표시해야 "국어 영역" 만 보임. 우측 41% 는 "(A형)" 영역으로 잘려야 함.

## 버그 위치

`src/renderer/layout/picture_footnote.rs::layout_picture` 는 crop 정상 처리 (84-101 라인). 그러나 인라인 TAC 그림은 이 함수를 거치지 않고 `paragraph_layout.rs` 의 텍스트 흐름 안에서 직접 `RenderNodeType::Image(ImageNode { … })` 를 emit — 세 곳 모두 `crop` / `original_size_hu` 필드 미설정.

| emit 위치 | 라인 (수정 전) | 시나리오 |
|-----------|---------------|----------|
| `paragraph_layout.rs` | 1741 | run 내부 인라인 TAC |
| `paragraph_layout.rs` | 2001 | run 범위 밖 미매칭 TAC |
| `paragraph_layout.rs` | 2086 | 빈 문단 + TAC 만 |

세 곳 모두 `effect, brightness, contrast, ..ImageNode::new(…)` 만 채움 → SVG 렌더에서 `crop=None` → 비트맵 전체 표시.

## 수정

세 emit 사이트에 동일한 crop / original_size_hu 추출 로직 추가 (picture_footnote.rs:84-101 와 동일):

```rust
let crop = {
    let c = &pic.crop;
    if c.right > c.left && c.bottom > c.top
        && (c.left != 0 || c.top != 0 || c.right != 0 || c.bottom != 0) {
        Some((c.left, c.top, c.right, c.bottom))
    } else { None }
};
let original_size_hu = if pic.shape_attr.original_width > 0
    && pic.shape_attr.original_height > 0 {
    Some((pic.shape_attr.original_width, pic.shape_attr.original_height))
} else { None };
```

`ImageNode { …, crop, original_size_hu, … }` 으로 설정.

→ `svg.rs::render_image_node` (1100-1124 라인) 의 crop 분기가 활성화되어 `<svg viewBox="0 0 1366.31 354" …>` 으로 비트맵 좌측 58.9% 만 표시.

## 검증

### 시각 비교 (15p 헤더)

| 단계 | 좌측 | 중앙 | 우측 |
|------|------|------|------|
| Stage 6 (이전) | 홀수형 | 국어 영역**(A형)**(화법과 작문) | 3 |
| Stage 7 (이후) | 홀수형 | 국어 영역(화법과 작문) ✓ | 3 |
| PDF (참조) | 홀수형 | 국어 영역(화법과 작문) | 3 |

### 14p / 16p 헤더 회귀 (Both/extension master)

| 페이지 | 좌측 | 중앙 | 비고 |
|--------|------|------|------|
| 14p (even, master[0]) | 2 | 국어 영역(화법과 작문) | (A형) 사라짐 |
| 16p (last, master[0]+master[2] overlap) | 2/4 겹침 | 국어 영역(화법과 작문) | crop fix 적용됨, 그러나 별도 이슈 (아래) |

### 단위 테스트

```
cargo test --release --lib
test result: ok. 1069 passed; 0 failed
```

### 회귀 (다른 샘플 4종)

- `2010-01-06.hwp` (6p) ✓
- `biz_plan.hwp` (6p) ✓
- `21_언어_기출_편집가능본.hwp` (15p) ✓
- `exam_eng.hwp` (8p) ✓

모두 정상 SVG 내보내기.

## 잔존 이슈 (별개 추적 필요)

### 16p 좌측 "2와 4 겹쳐 보임"

**현상**: 16페이지 (sec1 마지막 짝수 페이지) 좌측 헤더 셀에 "**2**" (master[0] Both) 와 "**4**" (master[2] Both is_ext=true overlap=true) 가 같은 위치에 그려져 시각적 겹침.

**PDF 참조**: 좌측에 "**4**" 만 표시. (master[2] 이 master[0] 을 대체해야 함)

**원인**: `src/document_core/queries/rendering.rs:996-1019` 에서 `overlap=true` 확장 master 는 `extra_master_pages` 에 추가만 하고 `active_master_page` (master[0]) 는 그대로 둠 → 둘 다 렌더.

**HWP 스펙 해석**: `overlap` 의 의미가 모호함. PDF 동작은 "확장 master 가 같은 apply_to 일 때 대체" 에 가까움. 추가 조사 필요.

**다음 작업**: 별도 이슈로 추적 권장. 본 Stage 7 의 crop fix 는 이 이슈와 독립.

> 사용자 보고에 "15페이지에서 2/4 겹침" 이라 명시되었으나 15p 출력에는 해당 현상 없음 — 16p (다음 짝수, sec1 마지막) 에서만 발생. 페이지 번호 혼동 가능성.

## 변경 파일

| 파일 | 변경 라인 |
|------|----------|
| `src/renderer/layout/paragraph_layout.rs` | +30 (3 emit 사이트 × 10라인 crop/orig 추출) |
