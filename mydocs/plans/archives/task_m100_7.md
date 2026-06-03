# Task #7: HWPX switch/case 네임스페이스 분기 처리 — 수행계획서

## 목표

HWPX 파서에서 `<switch>/<case>/<default>` 네임스페이스 분기를 처리하여, `HwpUnitChar` 케이스의 문단 간격/줄간격 값을 우선 적용한다.

## 현상

- `paraPr`의 `margin`(prev/next), `lineSpacing`이 `<switch>` 안에 있을 때 파싱되지 않음
- default 케이스의 큰 값이 적용되어 간격이 과도하게 벌어짐
- 고정값 줄간격에서 TAC 표와 문단의 병행 배치가 안 됨

## 구현 단계

### 1단계: paraPr 파싱에서 switch/case 처리

- `parse_para_shape()`에서 `<switch>` 시작 시 내부 진입
- `<case required-namespace="...HwpUnitChar">` 태그를 만나면 내부의 `margin`, `lineSpacing` 파싱
- case 값이 있으면 default 값을 덮어씀 (HwpUnitChar 우선)
- `<default>` 내의 값은 case가 없을 때만 사용

### 2단계: 검증

- `tac-img-02.hwpx` 19페이지 레이아웃 확인
- paraPr id=215: prev=400, next=400, line=1800/Fixed 확인
- `cargo test` 전체 통과
- 67페이지 전체 내보내기 회귀 없음

## 영향 범위

- `src/parser/hwpx/header.rs` — paraPr 파싱

## 검증 기준

- paraPr id=215의 spacing이 HwpUnitChar 케이스 값(400/400/1800)으로 적용
- 19페이지 LAYOUT_OVERFLOW 감소
