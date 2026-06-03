# 타스크 96 최종 결과 보고서 — 객체 묶음(Container/Group) 렌더링 구현

## 날짜
2026-02-16

## 요약

HWP 문서의 객체 묶음(Container/Group) 내 자식 도형들이 올바른 위치에 렌더링되도록 구현하였다. 아핀 변환 행렬 합성 순서 오류 수정, 글상자 세로 정렬 지원, 웹 Canvas 화살표 렌더링을 포함한다.

## 수행 내역

### 1. 아핀 변환 행렬 합성 순서 수정

**문제**: 그룹 자식 도형의 렌더링 위치가 원본 HWP과 불일치
**원인**: `parse_shape_component_full()`에서 변환 행렬 합성 순서가 `T × S × R`이었으나, 실제 HWP 스펙은 `T × R × S`
**수정**: `src/parser/control.rs` — rotation 먼저 합성 후 scale 합성으로 순서 교정

```rust
// 수정 전: result = compose(&result, &scale); compose(&result, &rotation);
// 수정 후:
result = compose(&result, &rotation);  // 회전 먼저
result = compose(&result, &scale);     // 스케일 후
```

**검증**: `samples/basic/docdo.hwp`(그룹) ↔ `docdo-1.hwp`(비그룹) SVG 비교 — 오차 ~0.1px 이내

### 2. TextBox 세로 정렬 (위/가운데/아래)

**문제**: KTX.hwp 범례 글상자 텍스트가 상단에 몰려 표시
**원인**: `list_attr` bit 5~6(세로 정렬 플래그)를 파싱하지 않음

**수정 파일**:
- `src/model/shape.rs` — `TextBox.vertical_align` 필드 추가
- `src/parser/control.rs` — 독립 도형 및 그룹 자식의 LIST_HEADER에서 세로 정렬 파싱
- `src/renderer/layout.rs` — `layout_textbox_content()`에서 세로 정렬 오프셋 계산

### 3. 그룹 자식 LIST_HEADER 데이터 파싱

**문제**: 독립 도형에서는 세로 정렬이 정상인데, 그룹핑하면 미적용
**원인**: `parse_container_children()`에서 LIST_HEADER 레코드를 `continue`로 건너뛰면서 데이터(list_attr, margins, max_width)를 파싱하지 않음

**수정**: LIST_HEADER 데이터를 캡처하여 TextBox 생성 시 속성 반영. SHAPE_COMPONENT 인라인 텍스트 속성을 fallback으로 추가.

### 4. 웹 Canvas 화살표 렌더링

**문제**: SVG에서는 범례 화살표가 표시되지만 웹 Canvas에서는 미표시
**원인**: SVG는 `<marker>` 요소로 화살표 지원하나, Canvas 2D에는 동등 기능 없음

**수정**: `src/renderer/web_canvas.rs`
- `calc_arrow_dims()` — 화살표 크기 계산 (SVG 렌더러와 동일 로직)
- `draw_arrow_head()` — 9종 ArrowStyle(Arrow, ConcaveArrow, Diamond, Circle, Square 등) Canvas 경로 드로잉
- `draw_line()` — 시작/끝 화살표 지원 및 선 끝점 조정

## 수정 파일 목록

| 파일 | 수정 내용 |
|------|----------|
| `src/model/shape.rs` | `TextBox.vertical_align` 필드, `render_b`/`render_c` 아핀 성분 |
| `src/parser/control.rs` | 행렬 합성 순서 수정, LIST_HEADER 파싱, 4-tuple 반환 |
| `src/renderer/layout.rs` | 아핀 변환 레이아웃, 세로 정렬 오프셋, 그룹 자식 라우팅 |
| `src/renderer/web_canvas.rs` | 화살표 렌더링 (9종, 시작/끝) |
| `src/main.rs` | dump-controls 개선 |
| `mydocs/tech/hwp_spec_5.0.md` | 스펙 오타 수정, 행렬 합성 순서 주석 |

## 커밋 이력

| 해시 | 내용 |
|------|------|
| `b2da7f3` | 그룹 자식 도형 렌더링 좌표 수정 (아핀 변환 + 스펙 문서) |
| `e4cda09` | 글상자 세로 정렬 + 웹 Canvas 화살표 렌더링 |

## 트러블슈팅 문서

- `mydocs/troubleshootings/task_96_group_child_matrix_composition_order.md` — 행렬 합성 순서
- `mydocs/troubleshootings/task_96_group_child_textbox_vertical_align.md` — 그룹 자식 세로 정렬

## 테스트 결과

- 532개 테스트 통과
- WASM 빌드 성공
- Vite 빌드 성공

## 샘플 검증

| 파일 | 검증 항목 | 결과 |
|------|----------|------|
| docdo.hwp / docdo-1.hwp | 그룹 vs 비그룹 위치 일치 | 정상 (~0.1px 이내) |
| tbox-center.hwp | 독립 글상자 세로 중앙 정렬 | 정상 |
| tbox-center-02.hwp | 그룹 자식 글상자 세로 중앙 정렬 | 정상 |
| KTX.hwp | 범례 세로 정렬 + 화살표 | 정상 |

## 알려진 제한사항

- 웹 Canvas의 스페이스/탭 폭이 SVG(네이티브)보다 좁게 표시될 수 있음: 네이티브는 `font_size × 0.5` 히우리스틱, WASM은 브라우저 `measureText()` 실측값 사용으로 차이 발생
