# Task M100 #430 구현 계획서

**이슈**: #430 그림 자동 크롭 미적용 — exam_kor 헤더 "국어 영역(A 형)"
**브랜치**: `local/task430`
**근거**: [Stage 1 완료 보고서](../working/task_m100_430_stage1.md)

## 1. 개요

`ImageNode` 에 원본 크기(HWPUNIT) 필드를 추가하고, `pic.shape_attr.{original_width, original_height}` 를 전달하도록 한다. SVG/Canvas 렌더러의 crop 스케일 계산을 올바른 공식으로 수정한다.

## 2. 변경 사항 상세

### 2.1 `src/renderer/render_tree.rs` — ImageNode 필드 추가

```rust
pub struct ImageNode {
    // 기존 필드 유지
    /// 원본 이미지 크기 (HWPUNIT) — crop 좌표 보정용
    /// `pic.shape_attr.{original_width, original_height}` 그대로
    pub original_size_hu: Option<(i32, i32)>,
    // ...
}
```

`ImageNode::new` 의 default 초기화에 `original_size_hu: None` 추가.

### 2.2 ImageNode 생성 사이트에 original_size_hu 채우기 (핵심 3곳)

| 파일 | 라인 | 변경 |
|------|------|------|
| `src/renderer/layout/picture_footnote.rs` | 98 (`layout_picture`) | `original_size_hu: Some((picture.shape_attr.original_width, picture.shape_attr.original_height))` 추가 |
| `src/renderer/layout/picture_footnote.rs` | 295 (`layout_body_picture`) | 동일 |
| `src/renderer/layout.rs` | 2610 | 동일 (`pic.shape_attr` 사용) |

다른 ImageNode 생성 사이트(`paragraph_layout.rs`, `shape_layout.rs`, `table_layout.rs::308`, `table_cell_content.rs::635`)는 본 이슈 범위 외 — 동일 패턴이 재발할 경우 후속 이슈로 정리.

### 2.3 `src/renderer/svg.rs::render_image_node` — 스케일 공식 수정

`FitToSize` + `crop` 분기(1086-1141)에서:

```rust
if let Some((cl, ct, cr, cb)) = img.crop {
    if let Some((img_w, img_h)) = parse_image_dimensions(&render_data) {
        let img_w = img_w as f64;
        let img_h = img_h as f64;
        // 원본 크기(HU)가 있으면 정확한 스케일 사용, 없으면 기존 폴백 동작
        let (scale_x, scale_y) = if let Some((ow_hu, oh_hu)) = img.original_size_hu {
            (ow_hu as f64 / img_w, oh_hu as f64 / img_h)
        } else {
            // 폴백: 기존(부정확하지만 호환성) 동작
            let s = cr as f64 / img_w;
            (s, s)
        };
        let src_x = cl as f64 / scale_x;
        let src_y = ct as f64 / scale_y;
        let src_w = (cr - cl) as f64 / scale_x;
        let src_h = (cb - ct) as f64 / scale_y;
        // ... (이후 is_cropped 판정 및 SVG 출력은 기존 로직 유지)
    }
}
```

### 2.4 `src/renderer/web_canvas.rs::draw_image_with_fill_mode` — 동일 수정

`draw_image_with_fill_mode` 시그니처에 이미 `original_size: Option<(f64, f64)>` 가 있지만 px 단위라 crop 보정에 부적합. 새로운 인자 `original_size_hu: Option<(i32, i32)>` 를 추가하거나, 호출부에서 ImageNode.original_size_hu 를 직접 참조해 동일한 공식을 적용.

호출부 (line 316):
```rust
self.draw_image_with_fill_mode(
    data, &node.bbox, img.fill_mode, img.original_size, img.crop,
    img.original_size_hu,  // 추가
);
```

함수 내부의 crop 분기(2042-2061)도 svg.rs와 동일한 공식으로 교체.

### 2.5 `src/main.rs` dump 보강 (Stage 1에서 이미 적용)

`src/main.rs:1643` 의 표 셀 내부 그림 dump 형식에 `orig`, `cur`, `crop` 추가. 디버깅 가치가 있으므로 유지.

## 3. 검증 계획

### 3.1 단위 테스트

`src/renderer/svg.rs` (또는 별도 테스트 파일)에 crop 스케일 계산 함수 추출 후 단위 테스트:

```rust
// 기대 입력: original_size_hu=(174000, 26580), crop=(0,0,102366,26580), img_px=(2320,354)
// 기대 출력: src=(0, 0, 1365, 354)
```

### 3.2 통합 검증

1. `cargo build --release` → 빌드 통과
2. `./target/release/rhwp export-svg samples/exam_kor.hwp -p 0 -o /tmp/out/`
   - 페이지 1 헤더 영역 이미지가 "국어 영역" 만 표시되는지 시각 확인 (원본 JPEG 임베디드 + viewBox 크롭)
3. 페이지 14, 20도 동일 확인 (`(화법과 작문)`, `(언어와 매체)` 별도 텍스트는 정상 표시 유지)
4. `cargo test --release` 전체 통과
5. `cargo clippy` 신규 경고 없음
6. 그림 포함 다른 샘플 회귀 점검 — `re_sample_gen` 테스트가 자동 비교

## 4. 단계 분할

본 구현 계획서를 3개 stage로 나눠 진행:

### Stage 2 — 모델 + 코어 렌더러 수정
- `render_tree.rs::ImageNode` 에 `original_size_hu` 필드 추가
- `picture_footnote.rs` 두 사이트(98, 295), `layout.rs:2610` 에서 채우기
- `svg.rs::render_image_node` crop 스케일 공식 수정
- 단위 테스트 추가
- 빌드 통과 확인
- 산출물: `mydocs/working/task_m100_430_stage2.md`

### Stage 3 — Web Canvas 동기화 + 회귀 검증
- `web_canvas.rs::draw_image_with_fill_mode` 시그니처 + 공식 수정
- 호출부 인자 추가
- exam_kor 1/14/20쪽 SVG 출력 시각 확인
- `cargo test --release`, `cargo clippy` 통과
- 산출물: `mydocs/working/task_m100_430_stage3.md`

### Stage 4 — 최종 보고
- 변경 요약, before/after 이미지 비교, 회귀 결과
- 산출물: `mydocs/report/task_m100_430_report.md`
- `mydocs/orders/{오늘}.md` 갱신

## 5. 위험 / 회귀 가능성

- **폴백 경로**: `original_size_hu` 가 없는 ImageNode 생성 사이트(인라인 TAC 등)는 기존 부정확 공식을 폴백으로 유지 → 본 이슈 범위 밖에서는 동작 변화 없음.
- **HWPX 경로**: `src/parser/hwpx/section.rs:1166` 가 crop을 파싱하지만 original_size_hu가 채워지지 않을 수 있음. HWPX 경로 picture 변환부 점검 후 동일 처리.
- **0 처리**: `original_size_hu = (0, 0)` 인 비정상 데이터 시 0 division 가능 → `> 0` 가드 추가.

## 6. 비범위 (Out of Scope)

- HWP "박스 비율 자동 클리핑" (시나리오 B) — 본 케이스는 명시 crop이 있어 해당 없음.
- 인라인 TAC (`paragraph_layout.rs`) 의 crop 누락 — 별도 이슈로 분리.
- 그룹/도형 picture (`shape_layout.rs`) — 별도 이슈.

---

승인 요청: 본 구현 계획대로 Stage 2부터 진행해도 좋은지 확인 부탁드립니다.
