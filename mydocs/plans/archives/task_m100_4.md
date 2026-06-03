# Task #4: 비-TAC 그림(어울림 배치) 높이 미반영 — 수행계획서

## 목표

비-TAC 그림(treat_as_char=false)이 본문에 배치될 때, 그림 높이가 후속 요소의 y 좌표에 반영되도록 수정한다.

## 현상

- `samples/tac-img-02.hwpx` 21페이지, `s0:pi=330`
- 비-TAC 그림(172.1×88.2mm)과 후속 표(pi=334)가 겹쳐 렌더링

## 원인

- `layout_shape_item()`이 void 함수로 y_offset을 반환하지 않음
- `layout_body_picture()`는 `VertRelTo::Para`일 때 갱신된 y_offset을 반환하지만 호출부에서 무시

## 구현

### 1단계: layout_shape_item 반환 타입 변경 및 y_offset 반영

- `layout_shape_item()` 반환 타입을 `f64`로 변경
- `layout_body_picture()`의 반환값을 캡처하여 반환
- 호출부에서 `y_offset`에 반영

## 검증 기준

- 21페이지에서 그림과 표가 겹치지 않음
- `cargo test` 전체 통과
- 67페이지 전체 내보내기 정상
