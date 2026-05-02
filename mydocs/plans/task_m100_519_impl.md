# Task #519: 그림(Picture) flip / rotation 속성 SVG 렌더링 적용 — 구현계획서

## 변경 대상 파일

- `src/renderer/layout/picture_footnote.rs` (line 107-117, 316-326): `ImageNode.transform` 채우기
- (필요 시) `src/renderer/render_tree.rs`: `ShapeTransform` 변환 헬퍼 추가
- (조사용) `examples/scan_picture_transforms.rs`: 임시 스캔 도구 (Stage 1 종료 시 삭제)

## 비변경 (검증 필요)

- `src/renderer/svg.rs:565 open_shape_transform`: 이미 horz_flip/vert_flip/rotation 모두 처리 중 (수정 불필요)
- `src/parser/control/shape.rs`: `shape_attr.horz_flip / vert_flip / rotation_angle` 정상 파싱 확인됨 (`examples/check_pic28_transform.rs` 로 검증 완료, Stage 1 에서 정식 도구로 재확인)

## 단계별 구현

### Stage 1: 베이스라인 + flip/rotation 분포 조사

**목적**: 회귀 케이스 식별 + `rotation_angle` 단위 확정.

**작업**:

1. `examples/scan_picture_transforms.rs` 작성 — `samples/*.hwp` 와 `samples/hwpx/*.hwpx` 전체 순회, `Picture` 컨트롤 중 `horz_flip || vert_flip || rotation_angle != 0` 인 것 모두 출력 (파일·sec·para·ctrl 인덱스 + bin_id + 변환값 + render_sx/render_b 행렬)
2. `rotation_angle` 단위 확정 — HWP 스펙 (`HWP_5.0_Spec.pdf` 또는 `mydocs/tech/`) 확인. 일반적으로 1/100° 단위. `src/model/shape.rs` 의 `HwpUnit16` 정의도 함께 확인
3. exam_eng p4 28번 회귀 베이스라인 캡처:
   - SVG: `target/release/rhwp export-svg samples/exam_eng.hwp -p 3 -o output/svg/task519_baseline/`
   - PNG: Chrome headless 렌더 (이전과 동일 방식) → `output/svg/task519_baseline/before.png`
   - PDF: Quartz Swift 렌더 (이전과 동일 방식) → `output/svg/task519_baseline/pdf_p4.png`

**산출물**:
- `mydocs/working/task_m100_519_stage1.md` — flip/rotation 분포표 + `rotation_angle` 단위 확정 + 베이스라인 캡처 위치

**완료 조건**:
- 영향받는 Picture 컨트롤 목록 확보
- `rotation_angle` 단위 확정 (1/100° 가정 검증)
- 회귀 비교용 PNG 3종 (PDF / SVG-before / 다음 단계의 SVG-after 자리)

---

### Stage 2: ImageNode.transform 채우기

**목적**: `picture.shape_attr` 의 변환 속성을 `ImageNode.transform` 에 전달.

**작업**:

1. `src/renderer/render_tree.rs` 의 `ShapeTransform` 에 `From<&ShapeComponentAttr>` 또는 헬퍼 메서드 추가 (가독성):
   ```rust
   impl ShapeTransform {
       pub fn from_shape_attr(sa: &ShapeComponentAttr) -> Self {
           Self {
               rotation: sa.rotation_angle as f64 / 100.0, // Stage 1 에서 단위 확정
               horz_flip: sa.horz_flip,
               vert_flip: sa.vert_flip,
           }
       }
   }
   ```
2. `src/renderer/layout/picture_footnote.rs:107-117` `ImageNode` 생성 시:
   ```rust
   RenderNodeType::Image(ImageNode {
       section_index,
       para_index,
       control_index,
       crop,
       original_size_hu,
       effect: picture.image_attr.effect,
       brightness: picture.image_attr.brightness,
       contrast: picture.image_attr.contrast,
       transform: ShapeTransform::from_shape_attr(&picture.shape_attr),  // ★ 추가
       ..ImageNode::new(bin_data_id, image_data)
   }),
   ```
3. `:316-326` 동일 수정
4. exam_eng p4 28번 SVG 재생성 → Chrome 렌더 → `output/svg/task519_baseline/after.png` 로 저장

**산출물**:
- `mydocs/working/task_m100_519_stage2.md` — 코드 diff 요약 + before/after PNG 비교 + PDF 와 시각 일치 확인

**완료 조건**:
- exam_eng p4 28번 curl 데코레이션이 박스 좌상단에 표시 (PDF 와 일치)
- 빌드/clippy 무경고

---

### Stage 3: 회귀 검증 + 최종 보고서

**목적**: 다른 샘플 회귀 0 확인 + 임시 도구 정리 + 최종 보고서.

**작업**:

1. `cargo test --lib` 통과 확인
2. `cargo clippy --all-targets -- -D warnings` 무경고 확인
3. SVG snapshot 회귀 확인 (`tests/svg_snapshot/` 또는 동등):
   - Stage 1 에서 식별한 다른 flip/rotation 케이스 SVG 비교
   - flip 이 새로 적용되어 변경된 경우 `cargo insta review` 후 의도된 변경으로 승인 (PDF 와 시각 일치 확인)
   - 변경 없는 SVG 는 변경 없어야 함 (회귀 없음)
4. `examples/scan_picture_transforms.rs` 삭제
5. 최종 보고서 작성 (`mydocs/report/task_m100_519_report.md`):
   - 본질 + 수정 위치 + 변경 줄 수
   - cargo test / clippy / SVG 회귀 결과
   - exam_eng p4 28번 PDF vs SVG 일치 시각 캡처
   - Stage 1 분포 결과 요약
6. `mydocs/orders/20260502.md` 갱신 (`완료` 상태)

**산출물**:
- `mydocs/working/task_m100_519_stage3.md`
- `mydocs/report/task_m100_519_report.md`
- 정리된 source 트리 (임시 도구 제거)

**완료 조건**:
- `cargo test --lib` + `cargo clippy` 무결
- 다른 SVG 회귀 의도되지 않은 변경 0건
- 최종 보고서 작성 + orders 갱신
