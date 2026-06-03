# 최종 결과 보고서 — Task M100 #418

## 이슈

[#418](https://github.com/edwardkim/rhwp/issues/418) — `samples/hwpspec.hwp` 20 페이지 이미지 이중 출력 (PR 처리 후 회귀)

## 결과 요약

`samples/hwpspec.hwp` 20 페이지의 빈 문단 + TAC Picture (s2:pi=83/86/89) 가 `paragraph_layout` 와 `layout.rs::layout_shape_item` 양쪽에서 emit 되어 `<image>` 6 개 (3 쌍의 이중 출력) 로 그려지던 결함을 정정 — **6 → 3 으로 정상화**.

본질적으로 **Task #376 의 정정 commit (`45419a2`) 이 devel 에 머지되지 않은 누락 정황** — 이슈 #376 은 close 되었으나 정정 코드가 임시 브랜치 (`pr-360-head`) 에만 존재. 본 task 에서 동일한 정정을 정확히 재적용 + 회귀 테스트로 재발 방지.

## 결함 origin

| 항목 | 값 |
|------|-----|
| 회귀 정정 commit (미머지) | [`45419a2`](https://github.com/edwardkim/rhwp/commit/45419a2) — Task #376 (2026-04-27, @planet6897) |
| 머지 누락 정황 | commit 이 `pr-360-head` 임시 브랜치에만 존재. devel / main 미머지 |
| 이슈 #376 상태 | CLOSED — 정정 commit 머지 검증 누락 |
| 메인테이너 책임 | 이슈 close 시 정정 commit 의 devel 머지 검증 절차 누락 |

## 변경 파일

### 변경 1 — `src/renderer/layout/paragraph_layout.rs:2042` 다음

빈 문단 + TAC Picture 분기에서 `set_inline_shape_position` 호출 추가:

```rust
line_node.children.push(img_node);
// [Task #418/#376] layout_shape_item 의 Task #347 분기 (빈 문단 + TAC Picture
// 직접 emit) 와 이중 렌더링되지 않도록 인라인 위치를 등록한다.
tree.set_inline_shape_position(
    section_index, para_index, tac_ci, img_x, img_y,
);
img_x += tac_w;
```

### 변경 2 — `src/renderer/layout.rs:2554` 분기

`get_inline_shape_position` 가드 추가:

```rust
let has_real_text = para.text.chars()
    .any(|c| c > '\u{001F}' && c != '\u{FFFC}');
// [Task #418/#376] paragraph_layout 의 빈 문단 + TAC Picture 분기에서 이미
// emit 된 경우, 여기서 또 push 하면 이중 emit 이 된다. 등록된 경우 push 를
// 스킵하고 result_y 만 갱신한다.
let already_registered = tree.get_inline_shape_position(
    page_content.section_index, para_index, control_index,
).is_some();
if !has_real_text && !already_registered {
    // ImageNode 생성 + push + set_inline_shape_position
    // ... result_y = pic_y + pic_h;
} else if !has_real_text && already_registered {
    // paragraph_layout 가 이미 emit — push 스킵, result_y 만 갱신
    result_y = pic_y + pic_h;
}
```

### 변경 3 — `tests/issue_418.rs` (신규 회귀 테스트)

```rust
#[test]
fn hwpspec_page20_no_duplicate_image_emit() {
    let svg = doc.render_page_svg_native(19).expect("...");
    let image_count = svg.matches("<image").count();
    assert_eq!(image_count, 3,
        "회귀: 빈 문단 + TAC Picture 이중 emit (Task #376 정정 누락 회귀)");
}
```

### 변경 4 — 트러블슈팅 신설 `mydocs/troubleshootings/tac_picture_double_emit.md`

향후 같은 영역 수정 시 점검 절차 / 회귀 방지 가이드.

## 단계별 진행

| 단계 | 작업 | 커밋 | 보고서 |
|------|------|------|--------|
| Stage 1 | `paragraph_layout.rs` 의 Picture 분기에 `set_inline_shape_position` 추가 | `b7c6775` | `mydocs/working/task_m100_418_stage1.md` |
| Stage 2 | `layout.rs` 의 layout_shape_item 분기에 `get_inline_shape_position` 가드 | `295b58d` | `mydocs/working/task_m100_418_stage2.md` |
| Stage 3 | 회귀 테스트 + 자동 검증 + WASM 빌드 | `f19304b` | `mydocs/working/task_m100_418_stage3.md` |

## 검증 결과

| 항목 | 결과 |
|------|------|
| 신규 회귀 테스트 (`cargo test --test issue_418`) | ✅ **1/1 passed** |
| 전체 lib test (`cargo test --lib`) | ✅ **1023 passed**, 0 failed |
| svg_snapshot (`cargo test --test svg_snapshot`) | ✅ 6/6 passed (다른 샘플 무회귀) |
| clippy (`cargo clippy --lib -- -D warnings`) | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 22s, 4,101,019 bytes (Task #416 → +769 bytes, 가드 코드 추가) |
| `samples/hwpspec.hwp` 20 페이지 SVG | ✅ **6 → 3** (이중 출력 해소) |
| `samples/hwpspec.hwp` 1 페이지 (Task #416 효과) | ✅ 보존 (`<image width="793.72"`) |
| 작업지시자 시각 판정 | ✅ 통과 |

### hwpspec p20 image y 좌표 변화

| 정정 전 (6 개) | 정정 후 (3 개) |
|--------------|--------------|
| 445.43 (paragraph_layout) | **445.43** ✅ |
| 442.76 (layout.rs 중복) | (제거) |
| 604.09 (paragraph_layout) | **604.09** ✅ |
| 601.43 (layout.rs 중복) | (제거) |
| 741.43 (paragraph_layout) | **741.43** ✅ |
| 738.76 (layout.rs 중복) | (제거) |

## 메모리 / 트러블슈팅 갱신

### 신규 메모리

`feedback_close_issue_verify_merged.md`:

> 이슈 close 시 정정 commit 이 devel 에 머지됐는지 확인 필수
>
> Why: Task #376 정정 commit 이 임시 브랜치에만 있고 devel 미머지 → 동일 결함 재발 (Task #418)
>
> How to apply: `git branch --contains <commit>` 또는 `git merge-base --is-ancestor <commit> devel` 로 검증. MISSING 결과면 close 보류 + 머지 + push 후 close

### 신규 트러블슈팅

`mydocs/troubleshootings/tac_picture_double_emit.md`:

- 결함 패턴 / 회귀 이력 (Task #376 → 머지 누락 → Task #418 재정정)
- 정정 코드 / 회귀 방지 단위 테스트
- 영향 받는 / 받지 않는 케이스 매트릭스
- 영역 수정 시 점검 절차

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 트러블슈팅 폴더 사전 검색 의무 | ✅ Stage 0 에서 git log 로 origin 추적 → Task #376 의 정정 commit 식별 |
| 작업지시자 시각 판정 게이트 | ✅ Stage 2 후 hwpspec p20 시각 판정 통과 |
| 이슈 close 시 정정 commit devel 머지 검증 | ✅ 본 task 에서 신규 메모리로 등록 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/task418` 에서 커밋 |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 + 신규 트러블슈팅 커밋
2. `local/task418` → `local/devel` → `devel` 머지 + push
3. 이슈 #418 close

## 산출물

- 변경 파일:
  - `src/renderer/layout/paragraph_layout.rs`
  - `src/renderer/layout.rs`
  - `tests/issue_418.rs` (신규)
- 트러블슈팅: `mydocs/troubleshootings/tac_picture_double_emit.md` (신규)
- 메모리: `feedback_close_issue_verify_merged.md` (신규)
- WASM 빌드: `pkg/rhwp.js`, `pkg/rhwp_bg.wasm`
- 시각 산출물: `output/svg/issue-418-fixed/hwpspec_020.svg`
- 수행 계획서: `mydocs/plans/task_m100_418.md`
- 구현 계획서: `mydocs/plans/task_m100_418_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_418_stage{1,2,3}.md`
- 최종 보고서: `mydocs/report/task_m100_418_report.md` (본 문서)

## 참고

- 이슈: [#418](https://github.com/edwardkim/rhwp/issues/418)
- 브랜치: `local/task418`
- 회귀 origin: [`45419a2`](https://github.com/edwardkim/rhwp/commit/45419a2) (Task #376, devel 미머지)
- 관련 이슈: [#376](https://github.com/edwardkim/rhwp/issues/376) (CLOSED, 머지 누락 정황), #347 (TAC 그림 z-order), #287 (빈 runs TAC 수식 인라인)
- 작업 일자: 2026-04-28
