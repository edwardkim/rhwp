# Task #1771 재현 샘플

## nested_group_vectors.hwpx
- 출처: 국가법령정보센터 `[붙임 12] 도로안전시설 설치 및 관리 지침- 노면요철포장(...)`
  (공개 지침, 328KB, 15쪽) — **중첩 그룹 복합 벡터** 다수(재귀 도형 710개).
- 결함(수정 전): 중첩 그룹이 CONTAINER(0x56) 레코드만으로 직렬화되어 파서의 자식 경계
  (SHAPE_COMPONENT @ child_level) 미인식 → 재파스 children 710→12 대량 소실
  (render-diff Path 335→0, Group 215→3, STRUCT_MISMATCH).
- 기대(수정 후): 중첩 그룹도 SHAPE_COMPONENT('$con') 경계 방출 → 710 전량 보존,
  `render-diff --via hwp` **PASS** (페이지 15=15, 변위 0.00px).
- 검증: `cargo test --test issue_1771_nested_group_roundtrip`
