# #7642 복합 도형의 실제 채움 경계와 본문 clip

## 독립 근거와 원인

같은 source/PDF/Cairo/96DPI로 다시 출력한 stage2170의 9월 자료 7쪽은
고정 devel fe5725dc38보다 Native raw가 약 0.98%p 낮다. stage2188의 새 review와
overlay, stage2191의 원래 raw 마스크를 직접 대조했다. 증가한 불일치의 큰 부분은
수식이 아니라 `기하` 글상자의 분홍색 이중선 위쪽 두 행이다. 독립 Print에는
바깥 띠가 있지만 현재 clip 위로 나간 띠가 출력에서 잘린다.

복합선의 실제 PathCommand는 부모 도형의 배치 bbox 밖에 있다. 이 bbox는
회전·대칭의 원점과 편집 소유를 위해 그대로 유지해야 한다. 기존 본문 clip 확장
의도는 가시 도형의 범위를 포함하는 것이지만 `layout.rs::expand_clip`이 실제
채움 경로 대신 배치 bbox만 소비한다. 단일 굵은 띠였던 base는 이 오류를 가렸다.

## 생산·소비 경로와 수정 범위

`shape_layout/compound_rectangle::append_border`의 실제 채움 경로 → 변환 후
채움 경계 → 기존 flow/float clip 합집합 → Body.clip_rect → 공통 LayerBuilder →
Native/Canvas/SVG/CanvasKit의 같은 clip 순서를 확인한다. 실제 경로의 경계를
배치 bbox와 합쳐 기존 clip 계산에만 연결한다. 도형의 bbox·좌표·페이지 배치,
복합선 비율과 장치 격자는 바꾸지 않는다. 특정 문서·색상·px 여백 분기를 만들지 않는다.

직선·3차 곡선·호를 실제 채움 경로에서 계산하고, 회전·대칭은 painter와 같은
중심·순서로 적용한다. control point의 바깥 위치를 실제 곡선 극값으로 오인하지
않는다. 기존 셀 clip과 글상자 clip 밖의 자손은 상위 본문 clip을 넓히지 않는다.
기존 flow/float 분류와 용지 하단 상한은 유지한다.

## 검증과 반례

- 원본 9월·10월 7쪽과 해당 전체 문서의 Native/fresh WASM, 최신 base 대비 하락을 확인한다.
- 회전·대칭·곡선의 경계, 셀/글상자에 잘리는 자손, 정상 본문과 앞뒤 내용의 보존을 확인한다.
- 실제 바깥 띠와 내부 공백이 독립 Print처럼 남는지 확대 판독한다. 단일선으로 되돌리지 않는다.
- 관련 전쪽 90%와 직접 판독 뒤에 새 이름의 정식 관계 검사를 추가한다. 기존 핀을 먼저 바꾸지 않는다.

진행 중인 stage2170은 보정 전 source 6b363c39ed의 결과로 보존한다. 새 source는
공유 target에서 빌드 잡 8개로 순차 검증하고 생산 SHA를 구분한다. WMF의 남은
1픽셀 표본 차이는 별도 관측이며 이 본문 clip의 원인으로 섞지 않는다.
