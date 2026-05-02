# Task #519 Stage 1: 베이스라인 + flip/rotation 분포 조사 — 완료 보고서

## 작업 내용

1. `examples/scan_picture_transforms.rs` 임시 도구 작성 → `samples/*.hwp` 150 파일 전체 순회
2. `rotation_angle` 단위 확정 (소스 비교)
3. `samples/exam_eng.hwp` 4페이지 28번 베이스라인 캡처 (PDF / SVG-before)

## flip/rotation 분포

| 파일 | 위치 | bin_id | flip | rotation | 행렬 |
|------|------|--------|------|----------|------|
| `samples/exam_eng.hwp` | s0 p189 c1 | 2 | h=true, v=true | 0 | M=[-0.976,0,30614; 0,-0.881,30190] |

**150 파일 중 단 1건.** 영향 범위가 매우 좁아 회귀 위험 최소.

## `rotation_angle` 단위 확정

기존 헬퍼 `src/renderer/layout/utils.rs:108-115 extract_shape_transform` 가 다른 도형(Rectangle 등 `shape_layout.rs:530`)에서 이미 사용 중이며 `rotation: sa.rotation_angle as f64` 로 변환 없이 직접 도(degree) 단위로 사용한다. 이 코드베이스 관습에 맞춰 Picture 도 동일하게 처리한다 (별도 단위 변환 불필요, 기존 헬퍼 재사용 가능).

## 베이스라인 캡처

| 파일 | 내용 |
|------|------|
| `output/svg/task519_baseline/exam_eng_004.svg` | 현재 SVG 출력 (수정 전) |
| `output/svg/task519_baseline/before.png` | Chrome 헤드리스 렌더 (1200×1700) |
| `output/svg/task519_baseline/before_p28.png` | 28번 영역 크롭 |
| `output/svg/task519_baseline/pdf_p4.png` | PDF 페이지 4 (Quartz 렌더) |
| `output/svg/task519_baseline/pdf_p28.png` | 28번 영역 크롭 |

## 시각 비교 결론

- **PDF**: 박스 좌상단에 curl 데코 + 박스 전체 테두리 명확
- **SVG-before**: 박스 좌상단 깨끗 (원본 BIN0002.jpg 의 curl 이 우하단에 있는 상태로 그려져 본문 내용과 겹쳐 사실상 보이지 않음)
- 한컴은 `flip=(h=true, v=true)` 를 적용 (180° 회전 효과) 하여 curl 이 좌상단으로 이동

## 완료 조건 충족

- [x] 영향받는 Picture 컨트롤 목록 확보 (1건)
- [x] `rotation_angle` 단위 확정 (raw degree, 기존 헬퍼 재사용)
- [x] 회귀 비교용 PNG 3종 (PDF / SVG-before / SVG-after는 Stage 2 산출)

## 다음 단계

Stage 2 — `picture_footnote.rs` 두 지점 수정 + after.png 캡처.
