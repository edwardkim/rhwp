# Task M100-1067 — HWPX 도형 IR 화 + HWP/HWPX 회전 부호 정정 (최종 보고서)

- 이슈: [#1067](https://github.com/edwardkim/rhwp/issues/1067) (CLOSED)
- 마일스톤: v1.0.0 (M100)
- 브랜치: `local/task1067`
- 일시: 2026-05-22
- 작업지시자 SVG 시각 판정: 통과 ("정상 — 정답지 이미지 정합")

## 1. 작업지시자 보고 증상 + 정답 이미지

- HWP: 첫 도형이 180도 반시계 방향으로 잘못 회전
- HWPX: 첫 도형 출력 안 됨

정답 이미지 (작업지시자 첨부): 두 도형이 거울 대칭으로 배치 (h-flip + rotate 적용)

## 2. 본질 식별 (다각형 dump + SVG 진단 통한 단계적 발견)

### 2.1 HWPX 파서 본질 — `<hc:pt>` 점 파싱 누락

`examples/dump_polygon_transform.rs` 진단 결과:
- HWP IR: `points: 4` ✓
- HWPX IR: `points: 0` ← 빈 Vec

→ `parse_shape_object` 가 `pt0`/`pt1`/`pt2`/`pt3` (rect 용) 만 처리, polygon 의 가변
`<hc:pt>` 무시. polygon path 빈 상태 → 도형 미표시.

### 2.2 SVG/canvas 렌더링 본질 — flip + 회전 동시 적용 시 회전 부호

`output/poc/issue_1067/hwp/shape-001.svg` 분석:
- 첫 도형 transform: `translate(...) scale(-1, 1) rotate(270, ...)`

작업지시자 보고: "왼쪽 첫 번째 다각형이 한컴과 180도 반시계 방향".

→ flip 와 회전 동시 적용 시 SVG 가 `rotate(θ)` 그대로 사용하면 한컴 표준과 180도 차이.
정정: `rotate(-θ)` (부호 반전) — flip + rotate 행렬 동등성으로 한컴 시각 정합.

### 2.3 추가 본질 — U+FFFC OBJECT REPLACEMENT CHARACTER 표시

작업지시자 보고: "OBJ 가 함께 출력되는 이유?"

원인: HWP/HWPX 의 inline 컨트롤 (treat_as_char) 이 paragraph text 에 U+FFFC placeholder 로
표현. SVG/canvas renderer 가 paragraph text 를 그대로 emit → 폰트에 glyph 부재로 tofu 표시.

→ `draw_text` 가 U+FFFC 를 skip 하여 invisible 처리.

## 3. 정정 영역 매트릭스 (Stage 1+2+3)

### 3.1 HWPX 파서 — polygon 점 파싱 (`src/parser/hwpx/section.rs`)

- `parse_shape_object` 의 child element 처리에 `b"pt"` 분기 추가:
  ```rust
  b"pt" => {
      let mut px: i32 = 0;
      let mut py: i32 = 0;
      for attr in ce.attributes().flatten() {
          match attr.key.as_ref() {
              b"x" => px = parse_i32(&attr),
              b"y" => py = parse_i32(&attr),
              _ => {}
          }
      }
      polygon_points.push(crate::model::Point { x: px, y: py });
  }
  ```
- `PolygonShape` + `CurveShape` 생성 시 `points: polygon_points` 전달.

### 3.2 SVG renderer 회전 부호 정정 (`src/renderer/svg.rs::open_shape_transform`)

```rust
let flip_negate_rotation = transform.horz_flip ^ transform.vert_flip;
// ... flip parts ...
if transform.rotation != 0.0 {
    let effective_rotation = if flip_negate_rotation {
        -transform.rotation
    } else {
        transform.rotation
    };
    parts.push(format!("rotate({},{},{})", effective_rotation, cx, cy));
}
```

### 3.3 Web Canvas renderer 동일 정정 (`src/renderer/web_canvas.rs::open_shape_transform`)

SVG 와 동일 패턴 — flip + 회전 동시 적용 시 회전 부호 반전.

### 3.4 U+FFFC OBJECT REPLACEMENT CHARACTER skip (svg + web_canvas)

`draw_text` 진입 시 U+FFFC 제거:
```rust
let text: String = text.chars().filter(|&c| c != '\u{FFFC}').collect();
if text.is_empty() { return; }
```

### 3.5 rhwp-studio canvaskit LayerPathOp.transform (Stage 1 - 부분적)

`rhwp-studio/src/core/types.ts` + `canvaskit-renderer.ts`:
- `LayerPathOp` 인터페이스에 `transform?: LayerPathTransform` 필드 추가
- `renderPath` 가 canvas.save/rotate/scale/restore 적용

(canvas2d backend 가 기본 — Rust web_canvas 가 처리하므로 canvaskit 정정은 보완적)

## 4. 정량 입증

### 4.1 IR 비교

| 항목 | HWP (정답지) | HWPX (Stage 2 후) |
|------|------------|------------------|
| polygon points | 4 ✓ | **0 → 4** (정정) |
| polygon rotation_angle | 270 / 90 | 270 / 90 ✓ |
| polygon horz_flip | true / false | true / false ✓ |

### 4.2 SVG transform 정합

| 도형 | Stage 1 (정정 전) | Stage 2 (정정 후) |
|------|-----------------|------------------|
| 도형 1 (flip + 270) | `scale(-1,1) rotate(270,...)` | `scale(-1,1) rotate(-270,...)` ✓ |
| 도형 2 (rotate 90) | `rotate(90,...)` | `rotate(90,...)` ✓ |

### 4.3 U+FFFC 제거

| Stage | SVG 내 U+FFFC 출현 |
|-------|--------------------|
| Before | 4 |
| After | **0** ✓ |

### 4.4 회귀 가드 (`tests/issue_1067_shape_rotation.rs`)

5 가드 5/5 통과:
- `issue_1067_hwpx_polygon_points_mapped`
- `issue_1067_hwpx_polygon_points_match_oracle`
- `issue_1067_hwpx_polygon_flip_rotation_preserved`
- `issue_1067_svg_rotation_sign_negated_with_flip`
- `issue_1067_svg_no_object_replacement_character`

### 4.5 CI 패턴

| 항목 | 결과 |
|------|------|
| cargo test --release --lib | **1324 passed / 0 failed** |
| cargo clippy --release --lib -D warnings | clean |
| cargo fmt --all --check | clean |
| WASM Docker 빌드 (4.91 MB) | success |
| WASM 동기화 (rhwp-studio/public) | done |

### 4.6 작업지시자 시각 판정

| 단계 | 결과 |
|------|------|
| Stage 1 (rhwp-studio canvaskit) | 변화 없음 (canvas2d backend 사용 중 — 보완적) |
| Stage 2 (SVG 회전 부호 정정) | **"정상 — 정답지 이미지 정합"** ✓ |

## 5. 잔존 이슈 — 별도 task 분리

### (1) HWPX TAC 도형 paragraph 가로 위치 결함
- 작업지시자 보고: "rhwp-studio hwpx 의 경우 두 도형이 붙어 보입니다"
- HWP/HWPX IR 동일이지만 SVG x 좌표 ~6px 차이
- 본질은 paragraph_layout 의 TAC + space character 좌표 계산
- 별도 task — 이슈 #1071 등록

### (2) 도형을 한글자처럼 캐럿 이동
- 작업지시자 명세: "도형 컨트롤은 한글자처럼 캐럿 이동", "줄/문단 끝 도형이면 도형 뒤 깜빡"
- 영역: rhwp-studio cursor + Rust char_offsets 동기화
- 별도 task — 이슈 #1071 통합 등록

## 6. 산출물

| 위치 | 내용 |
|------|------|
| `mydocs/plans/task_m100_1067.md` | 수행 계획서 |
| `mydocs/plans/task_m100_1067_impl.md` | 구현 계획서 |
| `mydocs/report/task_m100_1067_report.md` | 최종 보고서 (본 문서) |
| `tests/issue_1067_shape_rotation.rs` | 회귀 가드 5 |
| `examples/dump_polygon_transform.rs` | polygon IR 진단 도구 |
| `examples/dump_polygon_layer_json.rs` | layer JSON path transform 진단 도구 |
| `examples/dump_shape_para.rs` | paragraph + control 위치 진단 도구 |
| `output/poc/issue_1067/hwp/shape-001.svg` | SVG 산출물 (HWP) |
| `output/poc/issue_1067/hwpx/shape-001.svg` | SVG 산출물 (HWPX) |
| WASM 산출물 | `pkg/rhwp_bg.wasm` 4.91 MB + `rhwp-studio/public/` 동기화 |

## 7. 메모리 룰 정합

- ✅ `feedback_visual_judgment_authority` — 작업지시자 시각 판정 게이트 (Stage 1 미개선 → Stage 2 SVG 부호 정정 후 통과)
- ✅ `feedback_diagnosis_layer_attribution` — IR dump → SVG transform 분석 → 본질 정확 식별 (회전 부호 + U+FFFC + HWPX pt)
- ✅ `feedback_image_renderer_paths_separate` — svg + web_canvas + canvaskit 모두 별도 사본 정정
- ✅ `feedback_self_verification_not_hancom` — Rust SVG 정합 → 한컴 표준 정합 별개 본질 (회전 부호)
- ✅ `feedback_hancom_compat_specific_over_general` — flip+rotate 조합 시 회전 부호 반전 (case-specific)
- ✅ `feedback_process_must_follow` — 수행 → 구현 → Stage 별 시각 판정
- ✅ `feedback_assign_issue_before_work` — 본 task + 잔존 분리 task 모두 assignee 본인
- ✅ `project_hwpx_to_hwp_adapter_limit` 정합 — HWPX 파서 누락 contract 점진 정정

## 8. 후속 (잔존 + 별도 task)

- 이슈 #1071 (신규): HWPX TAC 도형 paragraph layout 가로 위치 + 한글자처럼 캐럿 이동
- 본 task #1067 의 진단 도구는 후속 task 에서 재사용 가능
