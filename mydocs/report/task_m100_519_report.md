# Task #519: 그림(Picture) flip / rotation 속성 SVG 렌더링 적용 — 최종 결과 보고서

## 요약

HWP 의 Picture 컨트롤에 `horz_flip / vert_flip / rotation_angle` 속성이 설정되어 있어도 rhwp 의 SVG 렌더링이 이를 무시하던 결함을 수정. 6개 `ImageNode` 생성 지점에 `transform: extract_shape_transform(&pic.shape_attr)` 한 줄씩 추가. 신규 코드 0줄, 기존 헬퍼 재사용.

## 본질

`src/renderer/svg.rs:565 open_shape_transform` 은 `RenderNodeType::Image` 의 `transform: ShapeTransform` 필드를 읽어 `<g transform="...">` 래퍼를 생성한다. 그러나 Picture 의 `ImageNode` 6개 생성 지점 모두 `..ImageNode::new(...)` 로 `ShapeTransform::default()` (변환 없음) 가 들어가서 변환이 항상 무시됐다.

다른 도형(Rectangle/Line/Ellipse/Group/Curve 등)은 `shape_layout.rs:530` 에서 `extract_shape_transform` 헬퍼로 정상 처리되고 있었으므로, **Picture 만 누락된 결함**이었다.

회귀 케이스: `samples/exam_eng.hwp` 4페이지 28번 안내문 박스의 종이-말림(curl) 데코레이션. 한컴은 `flip=(h=true, v=true)` (180° 회전 효과) 를 적용하여 curl 을 박스 좌상단에 배치하지만, rhwp 는 이를 무시하여 원본 우하단 위치로 그렸고 표 본문과 겹쳐 사실상 보이지 않았다.

## 수정 위치 (4 파일, 6개 지점)

| 파일 | 라인 | 컨텍스트 |
|------|------|----------|
| `src/renderer/layout/picture_footnote.rs` | 17, 116, 326 | 메인 그림 (앵커/floating) |
| `src/renderer/layout/paragraph_layout.rs` | 16, 1796, 2070, 2179 | 문단 내 인라인 TAC 그림 (3 분기) |
| `src/renderer/layout/table_cell_content.rs` | 16, 644 | 표 셀 내부 그림 |
| `src/renderer/layout.rs` | 2773 | 페이지 컨텐츠 직접 그림 (Task #347 경로) |

각 지점에 `transform: extract_shape_transform(&pic.shape_attr),` 한 줄 추가 + 필요한 import 갱신만 수행.

## 영향 범위 (Stage 1 분포 조사)

`samples/*.hwp` 150 파일 전체 스캔 결과 `horz_flip / vert_flip / rotation_angle != 0` 인 Picture 는 **단 1건** — `samples/exam_eng.hwp [s0 p189 c1] bin_id=2 flip=(h=true,v=true)`. 회귀 위험 최소.

## 검증

### 단위 테스트

```
cargo test --lib              : 1103 passed; 0 failed; 1 ignored
cargo test --test svg_snapshot:    6 passed; 0 failed
```

### 빌드 / Lint

```
cargo build --release         : Finished in 3m 24s (0 errors, 0 warnings)
cargo clippy --lib            : 0 errors, 0 warnings
```

`cargo clippy --all-targets` 의 46+2 errors 는 사전 존재 (test 파일의 `Box::new(_)` 등 stylistic) — `git stash` 로 베이스라인 확인, 본 변경 무관.

### exam_eng.hwp 8 페이지 SVG diff

| 페이지 | diff lines |
|--------|-----------|
| 001~003, 005~008 | 0 (변화 없음) |
| 004 | 4 (`<g transform>` 래퍼 1쌍 추가) |

페이지 4 추가 transform:
```xml
<g transform="translate(1602.43,0) scale(-1,1) translate(0,2203.93) scale(1,-1)">
  <!-- 28번 그림 (BIN0002.jpg, 수평+수직 대칭) -->
</g>
```

### 시각 비교 (output/svg/task519_baseline/)

| 파일 | 결과 |
|------|------|
| `pdf_p28.png` | PDF (정답): curl 좌상단 명확 |
| `before_p28.png` | SVG-before: curl 우하단 (본문과 겹쳐 비가시) |
| `after_p28.png` | SVG-after: curl 좌상단 (PDF 와 일치) ★ |

## 작업 통찰

- **헬퍼 재사용으로 구현 비용 0**: `extract_shape_transform` 가 이미 다른 도형에서 사용되고 있었음. Picture 만 누락된 명백한 결함.
- **회귀 위험 최소**: 150 파일 중 1건만 영향. SVG snapshot 테스트(`svg_snapshot.rs`) 의 6개 케이스 모두 영향 없음.
- **`rotation_angle` 단위**: 코드베이스 관습상 raw `i16` 을 도(degree) 단위로 직접 사용 (`shape_layout.rs:530` 와 동일). HWP 스펙의 1/100° 가정과 다를 수 있으나 일관성 유지가 우선.

## 산출물

- `mydocs/plans/task_m100_519.md` — 수행계획서
- `mydocs/plans/task_m100_519_impl.md` — 구현계획서
- `mydocs/working/task_m100_519_stage{1,2}.md` — 단계별 완료 보고서
- `mydocs/report/task_m100_519_report.md` — 본 보고서
- `output/svg/task519_baseline/{pdf,before,after}_p28.png` — 시각 비교 캡처

## 잔존 사항

없음. 회귀 위험이 최소이고 완전한 수정.
