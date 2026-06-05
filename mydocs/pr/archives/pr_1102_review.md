# PR #1102 검토 — 회전 90°/270° 이미지의 bbox 이중회전 정정

- 검토일: 2026-05-26
- PR: https://github.com/edwardkim/rhwp/pull/1102
- 관련 이슈: 없음 (closes 미지정)
- 관련 발견: **#1127** (paint/builder.rs 영역 동일 정정 누락 — 본 PR 외 별도 등록)
- 관련 영역: #1067 (HWPX 도형 IR + 첫 180° 회전 결함, CLOSED)
- 검토자: Claude (rhwp 메인테이너 보조)

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #1102 |
| 제목 | fix(renderer): 회전 90°/270° 이미지의 bbox 이중회전 정정 — 페이지 폭 초과/잘림 해소 |
| 작성자 | HaimLee-4869 (Lee eunjung) — 기존 컨트리뷰터 (9번째 PR) |
| base ← head | `devel` ← `pr/image-rotation-bbox` |
| head SHA | `e8b6ee4ca9d29d1a675bbc22b6719c5c58ee97b8` |
| commits | 1 (단일) |
| 상태 | OPEN / mergeable / mergeStateStatus=BEHIND |
| 변경 | 4 files, +47 / -12 |
| 본질 변경 | `src/renderer/render_tree.rs` (+21, `ShapeTransform::effective_image_bbox` 신규) · `src/renderer/svg.rs` (+3/-1) · `src/renderer/canvas.rs` (+18/-6) · `src/renderer/web_canvas.rs` (+5/-3) |
| GitHub CI | 미실행 (fork 패턴) |
| 외부 정답지 | 한컴2024 PDF + 한컴 실측 (170.6×242.7mm) |

## 2. 컨트리뷰터 누적 사이클

`gh pr list --author HaimLee-4869` — 누적 9개 PR. 직전:
- #1101 (글자겹침 한컴2024 정합) — **MERGED** (2026-05-26)
- #1088 (para-float vertical_offset) — **OPEN, 시각 회귀로 수정 요청** (자동 검증 통과했으나 시각 회귀)

PR #1088 사례 영역으로 **자동 검증 통과 ≠ 시각 무결성** 영역 인지 필요. 본 PR 영역 영역도 시각 검증 게이트 핵심.

## 3. 문제와 원인

### 증상
90°/270° 회전 이미지가 페이지 폭 초과 + 우측 잘림. 컨트리뷰터 자체 문서 "참고2 클라우드 시범사업 망 구성도" 영역 — 가로로 긴 원본 (243×170mm) 을 90° 회전해 세로 (170×243mm) 로 배치하는 영역.

### 원인 — 이중 회전

HWPX 의 `<hp:sz>` 영역 = **회전 후 외접 사각형 치수** (portrait 170×243mm). 이 bbox 위에 추가로 `rotate(90, cx, cy)` transform 영역 적용 → 사실상 두번 회전 → 시각적으로 다시 가로 (landscape) 영역 반전 → 페이지 폭 초과.

`<hp:sz>` ↔ `<hp:curSz>` (회전 전 영역 243×170mm) 영역 = width/height swap 관계. rotation transform 자체가 90° 회전 영역 담당하므로 **bbox 는 회전 전 치수 영역** 입력 필요.

## 4. 변경 내용

### 4.1 ShapeTransform::effective_image_bbox 신규 (render_tree.rs, +21)

```rust
pub fn effective_image_bbox(&self, bbox: &BoundingBox) -> BoundingBox {
    let r = self.rotation.rem_euclid(360.0);
    let is_perpendicular = (r - 90.0).abs() < 1.0 || (r - 270.0).abs() < 1.0;
    if !is_perpendicular {
        return *bbox;
    }
    let cx = bbox.x + bbox.width / 2.0;
    let cy = bbox.y + bbox.height / 2.0;
    let new_w = bbox.height;
    let new_h = bbox.width;
    BoundingBox::new(cx - new_w / 2.0, cy - new_h / 2.0, new_w, new_h)
}
```

**평가**:
- 분기 영역 정확 — 90°/270° (±1° 톨러런스) 만 swap, 0°/180°/45° 영역 영역 입력 bbox 그대로 반환 (회귀 0)
- 중심 (cx, cy) 영역 swap 전후 동일 → rotation transform 식 그대로 유지 가능
- 모듈러 산수 `rem_euclid(360.0)` 영역 영역 음수 회전 영역 (-90° → 270°) 정합

**관찰**:
- unit test 없음 — 본 메서드 단독 영역 영역 회귀 가드 부재
- `is_perpendicular` 영역 영역 0°/180° 외 영역 처리 — 예: 89.5° 영역 (오차 영역) 도 분기 진입 (의도) — 한컴 실제 산출 영역 의 회전 영역 영역 정수 단위 (0/90/180/270) 영역 영역 일반적이므로 합리적

### 4.2 3개 renderer backend 적용

- `svg.rs:410` (RenderNodeType::Image 분기) ✅
- `canvas.rs:150` (일반 path) + `canvas.rs:245` (LayerOp path) ✅
- `web_canvas.rs:529` (RenderNodeType::Image 분기) ✅

**평가**: 본 PR 의 정정 영역 3개 backend 영역 일관 적용. 메모리 룰 `feedback_image_renderer_paths_separate` 일부 정합.

### 4.3 🚨 누락 영역 — paint/builder.rs (별도 이슈 #1127)

[src/paint/builder.rs:86-87](src/paint/builder.rs#L86) 영역:

```rust
RenderNodeType::Image(image) => Some(vec![PaintOp::Image {
    bbox: node.bbox,          // ← effective_image_bbox 미적용
    image: image.clone(),
    ...
}]),
```

본 영역은 paint pipeline (CanvasKit 영역의 source) 영역. PaintOp::Image 의 bbox 영역 영역 `node.bbox` 그대로 전달 → CanvasKit JS 영역 (`renderImage`, `rhwp-studio/src/view/canvaskit-renderer.ts:404`) 영역 영역 회전 transform 이중 적용 → rhwp-studio 영역 영역 결함 잔존 가능성.

**메모리 룰 `feedback_image_renderer_paths_separate` 위반** — 4개 renderer path sweep 영역 영역 1개 누락.

→ **별도 이슈 #1127 등록** 완료. PR #1101 의 #1126 등록과 동일 패턴.

## 5. 자동 검증 결과

cherry-pick → `local/pr1102-review` 영역 영역 적용:

| 항목 | 결과 |
|------|------|
| `git cherry-pick e8b6ee4c` | ✅ 충돌 없음 (Auto-merging web_canvas.rs) |
| `cargo build --release` | ✅ 통과 |
| `cargo fmt --all -- --check` | ✅ 위반 0 |
| `cargo clippy --lib --release -- -D warnings` | ✅ warnings 0 (25.88s) |
| `cargo test --release --tests` | ✅ svg_snapshot 8 passed / tab_cross_run 1 passed |
| GitHub CI | ⚠️ 미실행 (fork 패턴) |

## 6. 시각 검증

### 6.1 fixture 영역 부재

현 fixture 영역에 hp:pic + rotation 90°/270° 영역 영역 직접 검증 가능 fixture 없음 (`samples/hwpx/shape-001.hwpx` 영역 영역 도형 회전 — hp:pic 영역 아님).

### 6.2 컨트리뷰터 자체 검증 보고

PR 본문 영역 영역 bbox 좌표 보고:
- 정정 전: `(75.59, 124.40, 645×918 portrait)` + rotation transform → 시각 landscape (918×645) → 우측 끝 좌표 초과
- 정정 후: `(-60.95, 260.93, 918×645 landscape swap)` + rotation transform → 시각 portrait (645×918) @ (75.59, 124.40)
- 우측 끝 720 < 페이지 폭 793 → 잘림 해소
- 한컴2024 PDF 실측 170.6×242.7mm 정합

**평가**: 보고 영역 영역 수학적 정합 (swap 영역 → 동일 transform → 의도 결과). 정량 영역 영역 합리적.

### 6.3 한컴2024 정답지 등급

본 PR 영역 영역 정답지 = 한컴2024 PDF. 본 프로젝트 정답지 등급 = 한컴 2020/2022 (메모리 룰 `feedback_pdf_not_authoritative`). 한컴 2020/2022 영역 영역 동일 시각 재검증 권장 (후속).

## 7. 위험·관찰

| 항목 | 등급 | 영역 |
|------|------|------|
| paint/builder.rs 영역 영역 동일 정정 누락 (#1127) | **고** | rhwp-studio 영역 영역 결함 잔존. 본 PR 머지 + 후속 이슈 처리 영역 권장 |
| 시각 검증 fixture 부재 | 중 | 한컴 회전 이미지 fixture 영역 후속 영역 추가 권장. 회귀 가드 svg_snapshot 후속 영역 |
| 한컴2024 PDF 영역 영역 정답지 등급 외 | 중 | 한컴 2020/2022 영역 영역 재검증 권장 |
| PR #1088 시각 회귀 사례 (동일 컨트리뷰터) | 중 | 본 PR 영역 영역 시각 검증 fixture 부재 → 잔존 위험. 컨트리뷰터 자체 검증 보고 의존 |
| `effective_image_bbox` unit test 부재 | 저 | 90°/270°/180° 영역 영역 회귀 가드 후속 권장 |
| `is_perpendicular` 영역 영역 ±1° 톨러런스 | 저 | 한컴 실제 영역 영역 정수 단위 회전 영역 영역 일반적 — 합리적 |

## 8. 최종 평가 (잠정)

| 항목 | 결과 |
|------|------|
| 본질 해결 | ✅ (SVG/canvas/web_canvas) + ⚠️ (paint/builder 누락) |
| 자동 검증 | ✅ 모두 통과 (build / fmt / clippy --lib / test) |
| 시각 검증 | ⚠️ fixture 부재. 컨트리뷰터 자체 보고 영역 정합 (수학적 정량 영역) |
| 코드 품질 | ✅ 메서드 분리 + 주석 명확, 분기 영역 정확 |
| 메모리 룰 정합 | ⚠️ `image_renderer_paths_separate` 부분 정합 (3/4, #1127 영역 영역 누락) |
| 회귀 가드 | ⚠️ 없음 (회전 이미지 fixture + svg_snapshot 부재) |
| **결정 권장** | **MERGE** — SVG/canvas 영역 영역 해소는 우선 확보. #1127 후속 처리 |

## 9. 작업지시자 결정 요청

1. **MERGE** 진행 — SVG/canvas 영역 해소 우선 확보 (작업지시자 결정: MERGE + 후속 이슈)
2. 후속 이슈 #1127 (paint/builder.rs 정정) — 본 영역 영역 별도 처리
3. 한컴 2020/2022 영역 영역 시각 재검증 영역 후속 권장
4. 회전 이미지 fixture + svg_snapshot 회귀 가드 영역 후속 권장
