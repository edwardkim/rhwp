# 작업지시자 피드백 — PR #323 회귀: 21_언어 1페이지 col 1 시작 위치

**일자**: 2026-04-25
**대상**: PR #323 (Task #321)
**관련 이슈**: #326

## 피드백 원문 요지

PR #323 의 v3 커밋 (3932b83) `Task #321 v3: Paper(용지) 기준 도형은 col 1+ reserve에서 제외` 가 21_언어 page 1 col 1 시작 위치에서 회귀 유발.

| 브랜치 | 단 1 used | hwp_used | diff | 단 1 시작 |
|--------|-----------|----------|------|-----------|
| devel | 1223.1 | 38.9 | +1184.3 | body 하단 시작 (≈1184px reserve) |
| PR #323 | 1174.7 | 1213.1 | -38.4 | body 상단 시작 (reserve=0) |

## 작성자 판단

**Option A 권장** — v3 의 Paper 제외 가드를 **\"본문과 겹치지 않을 때만\"** 으로 정밀화.

```rust
if matches!(common.vert_rel_to, VertRelTo::Paper) {
    let shape_top = hwpunit_to_px(common.vertical_offset as i32, dpi);
    let shape_bottom = shape_top + hwpunit_to_px(common.height as i32, dpi);
    let body_top = layout.body_area.y;
    if shape_bottom <= body_top {
        continue;  // 본문과 겹치지 않으면 제외
    }
}
```

## 후속 진행

이슈 #326 등록, task321 브랜치에서 v3 재도입 + 정밀화 가드 적용 예정.
