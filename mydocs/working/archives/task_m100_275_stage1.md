# Task #275 단계 1 완료보고서 — Placeholder arm

**이슈**: [#275](https://github.com/edwardkim/rhwp/issues/275)
**브랜치**: `local/task275`
**계획서**: [`task_m100_275_impl.md`](../plans/task_m100_275_impl.md) §3 단계 1

## 1. 변경 내용

### 1.1 `src/renderer/web_canvas.rs`

`render_node` match 에 `RenderNodeType::Placeholder` arm 추가 (FootnoteMarker arm 과 catch-all `_ =>` 사이).

렌더 동작 (svg.rs:351-365 와 동등):
1. `fill_color` 로 배경 rect
2. `stroke_color` + `StrokeDash::Dash` (6 3 패턴) 로 점선 테두리 (1px)
3. 중앙 라벨
   - 폰트: `sans-serif`
   - 크기: `(min(w, h) * 0.06).clamp(12.0, 28.0)` (svg.rs 와 동일 공식)
   - 색상: `stroke_color`
   - 정렬: text-align=center, baseline=middle
4. text-align / baseline 기본값 복원 (다른 노드 영향 차단)

추가 라인: **+26 줄**

### 1.2 기존 인프라 재사용

- `set_line_dash(&StrokeDash::Dash)` — 기존 헬퍼 (web_canvas.rs:551). 패턴 [6, 3] 은 svg.rs `stroke-dasharray="6 3"` 와 일치
- `color_to_css` — 기존 COLORREF(BGR) 변환기. svg.rs 의 `color_to_svg` 와 동일한 변환식이므로 출력 색상 일치 보장

## 2. 검증

### 2.1 컴파일

- `cargo check --lib --target wasm32-unknown-unknown`: **clean**
- `cargo check --lib` (native): **clean**

### 2.2 테스트

- `cargo test --lib`: **949 passed / 14 failed / 1 ignored**
  - **14 failures 는 baseline 기존 실패** (stash 후 devel 비교로 확인). 모두 `cfb_writer::tests` / `wasm_api::tests` 의 직렬화·roundtrip 테스트. 본 타스크 범위 밖.
  - 추가/회귀 테스트 **없음**

### 2.3 Clippy

- `cargo clippy --lib --target wasm32-unknown-unknown -- -D warnings`: baseline 16 error 존재 (본 변경 외 파일·라인). **본 변경 라인 (402-438) 은 clippy-clean**.

### 2.4 시각 검증 — 연기

현재 `samples/` 에 Placeholder 로 떨어지는 파일 없음 (bitmap/한셀OLE 은 단계 2의 `<image>` 경로로 해결됨). 단계 3 에서 shape_layout.rs 임시 수정으로 강제 Placeholder 재현 및 시각 확인 예정.

## 3. 미커밋 상태

세션 중 생성된 임시 파일이 누적되어 있어 단계 1 커밋을 잠시 보류. 승인 후:

```
variable 커밋 (Placeholder arm 단독):
  src/renderer/web_canvas.rs                  (수정)
  mydocs/plans/task_m100_275.md               (신규, 수행계획서)
  mydocs/plans/task_m100_275_impl.md          (신규, 구현계획서)
  mydocs/working/task_m100_275_stage1.md      (신규, 본 보고서)
```

임시 파일 (`first-readme.txt`, `preview.log`, `public/samples/_*.hwp`, `e2e/debug-load-bug.test.mjs`) 은 단계 4 에서 정리.

## 4. 다음 단계

**단계 2 — RawSvg `<image>` 단일 경로 (A)**
- `try_parse_single_image_svg` 유틸 + 단위 테스트
- `RawSvg` match arm 의 A 경로 (동기, 기존 draw_image 재사용)
- bitmap.hwp / 한셀OLE.hwp 재현 샘플 해결 예상

## 5. 승인 요청

단계 1 승인 후 바로 커밋 + 단계 2 착수할 수 있도록 허가 부탁드립니다.
