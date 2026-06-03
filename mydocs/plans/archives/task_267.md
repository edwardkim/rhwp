# Task 267 수행 계획서: 테이블 셀 높이 버그 수정

## 현상

- samples/table-bug-1.hwp: 병합된 행의 하단 셀 높이가 과도하게 렌더링
- 원인: 셀 패딩 필드에 비정상적으로 큰 값(1700)이 저장되어 있어 행 높이/텍스트 위치가 잘못 계산

## 근본 원인

HWP LIST_HEADER의 `list_attr bit 16` ("안 여백 지정", hwplib: `isApplyInnerMargin`)이
0인 셀에서도 패딩 필드에 값이 저장되어 있으나, 렌더링에서 무시해야 함.

## 구현

### 파서 수정 (control.rs)
- `list_attr bit 16` 체크: 0이면 셀 패딩을 `{0,0,0,0}`으로 클리어
- 렌더러는 `padding == 0`이면 자동으로 테이블 기본 패딩 사용 (기존 로직)

## 교차 검증

hwplib `ListHeaderPropertyForCell.java`:
- bit 16: `isApplyInnerMagin()` — 안 여백 지정 여부
- true이면 셀 개별 패딩, false이면 테이블 기본 패딩

## 참조 파일

| 파일 | 변경 |
|------|------|
| src/parser/control.rs | list_attr bit 16 체크 + 패딩 클리어 |
| src/parser/tags.rs | CTRL_PAGE_HIDE: pghi → pghd (이전 타스크에서 발견) |
