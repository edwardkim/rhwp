# 최종 결과 보고서 — Task #577

**제목**: exam_science.hwp 2번 문제 보기 이미지 하단 클리핑 (TopAndBottom 셀 내부 이미지 1라인 오프셋)
**브랜치**: `local/task577`
**마일스톤**: M100 (v1.0.0)
**이슈**: [#577](https://github.com/edwardkim/rhwp/issues/577)
**작업 기간**: 2026-05-04

---

## 1. 결론

`samples/exam_science.hwp` 페이지 1, 2번 문제(보기 ①~⑤)에서 이미지 하단이 cell-clip 영역을 약 10.81 px 초과하여 잘려 보이던 결함을 정정했다. 동일 산식 결함으로 잠재 오버플로(②: 11.55, ④: 12.30 px) 가 있던 우측 보기들도 함께 정정.

## 2. 원인 요약

`src/renderer/layout/table_layout.rs:1547..` 의 비-TAC Picture 분기에서 `compute_object_position` 호출 시, `layout_composed_paragraph` 가 advance 시킨 `para_y` (= cell_y + pad_top + line_height) 를 그대로 anchor 로 사용. HWP 의도는 TopAndBottom 이미지가 anchor 라인을 displace 하므로, 라인 높이를 소비하지 않은 `para_y_before_compose` (= cell_y + pad_top) 가 anchor 가 되어야 함.

```
관측 image_y - cell_y = 19.10 px
  = pad_top(3.78) + line_height(15.32, lh=1150 HU)
정정 후 image_y - cell_y = 3.78 px = pad_top
```

## 3. 변경 사항

### 코드

`src/renderer/layout/table_layout.rs` (1 hunk, +14/-3 line)

비-TAC Picture 분기에 `anchor_y` 도입:

```rust
let anchor_y = if matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
              && matches!(pic.common.vert_rel_to, VertRelTo::Para)
{ para_y_before_compose } else { para_y };
let cell_area = LayoutRect { y: anchor_y, height: ..., ..inner_area };
let (pic_x, pic_y) = self.compute_object_position(
    ..., anchor_y, para_alignment,
);
```

`para_y += pic_h;` (다음 단락 시작점)는 무변경.

### 산출물

| 경로 | 종류 |
|------|------|
| `src/renderer/layout/table_layout.rs` | 코드 수정 |
| `mydocs/plans/task_m100_577.md` | 수행 계획서 |
| `mydocs/plans/task_m100_577_impl.md` | 구현 계획서 |
| `mydocs/working/task_m100_577_stage1.md` | 분석·재현·기준선 캡처 |
| `mydocs/working/task_m100_577_stage2.md` | 코드 수정 |
| `mydocs/working/task_m100_577_stage3.md` | 시각·자동 검증 |
| `mydocs/report/task_m100_577_report.md` | 본 최종 보고서 |
| `output/svg/task577_baseline*/` | baseline SVG |
| `output/svg/task577_after*/` | after-fix SVG |

## 4. 검증 결과

### exam_science.hwp 1페이지 보기 ①~⑤

5개 이미지 모두 `image_y - cell_y = 3.78 px`(= pad_top, HWP IR 정합) 로 정정. cell-clip 내부에 정상 배치 확인.

| 보기 | image bottom | cell bottom | 결과 |
|------|--------------|-------------|------|
| ① | 832.09 | 843.87 | ✅ |
| ② | 840.09 | 843.87 | ✅ |
| ③ | 888.60 | 903.47 | ✅ |
| ④ | 900.44 | 903.47 | ✅ |
| ⑤ | 953.00 | 957.52 | ✅ |

### LAYOUT_OVERFLOW

| 샘플 | baseline | after-fix |
|------|----------|-----------|
| exam_science.hwp | 1건 (9.5 px) | 1건 (3.4 px) — 6.1 px 감소 |
| mel-001.hwp | 8건 (3.5~18.8 px) | **0건** |

### 빌드 / 테스트

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ |
| `cargo test --release --lib` | ✅ 1125 passed |
| `cargo clippy --release -- -D warnings` | ⚠ 사전 존재 에러 2건 (`src/renderer/layout.rs:313-314`) — 본 변경 무관 |

## 5. 부수 효과

비-TAC TopAndBottom Picture (`vert_rel_to=Para`) 가 들어 있는 다른 셀들도 동일 산식으로 정정되어, 일부 페이지의 콘텐츠 좌표가 IR vpos 정합 방향(LAYOUT_OVERFLOW 가 줄어드는 방향)으로 이동. 모든 이동된 이미지는 여전히 cell-clip 내부에 정상 포함됨.

상세는 Stage 3 보고서 참조.

## 6. 후속 가능 항목

- 본 변경으로 baseline 좌표가 이동한 페이지들의 시각 회귀 검증을 후속 PR/타스크에서 baseline 갱신 후 진행 권고.
- `cargo clippy` 의 사전 존재 doc_lazy_continuation 에러 정리는 별도 타스크로 분리.

## 7. 종결

승인 후 `local/task577` → `local/devel` merge 권고. 본 계획서·구현 계획서를 `mydocs/plans/archives/` 로 이동.
