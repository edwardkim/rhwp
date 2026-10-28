# Task M100 #1402 구현계획서 — 열거 토큰 화이트리스트 정합 검사

- 수행계획서: `mydocs/plans/task_m100_1402.md` (승인 완료)
- 브랜치: `local/task1402`
- 작성일: 2026-06-14
- 단계: 2단계

## 0. 사전 조사 확정

- serializer 방출 함수는 priv지만 각 파일에 테스트 모듈 존재(section/table/field/
  picture/shape) → **같은 모듈 내 단위 테스트에서 호출 가능**. field.rs는 이미
  `field_type_str_covers_main_variants` 선례 존재.
- enum은 전 variant 명시 → 테스트에서 각 variant를 함수에 넣어 토큰 수집 가능.
- 검사 형태: **모듈별 "방출 토큰 ∈ 화이트리스트" 단위 테스트**.

## 1단계 — 검사 인프라 + 화이트리스트 + 정정

### 1.1 화이트리스트 구성

- 속성별 허용 집합 = (실물 관측 토큰, `output/poc/task1402/observed_tokens.txt`)
  ∪ (owpml/한컴 스펙 표준값 — corpus에 없을 수 있는 IR 변형 토큰을 명시 허용).
- 실물 검증 가능 + 영향 큰 속성 우선: numType, numberingType, pageBreak, textWrap,
  textFlow, vertAlign/horzAlign, vertRelTo/horzRelTo, applyPageType, lineSpacing,
  align, fieldType 등.

### 1.2 정합 테스트

- 각 방출 함수에 대해 전 enum variant를 넣어 토큰을 얻고, 화이트리스트에 속하는지
  단언. 화이트리스트 밖 토큰 방출 시 실패.
- **numType 회귀 가드** (강조): AutoNumberType::Picture → "PICTURE" 보장 +
  "FIGURE" 절대 방출 금지 (#1387 재발 봉인).

### 1.3 circleType TIRANGLE 판정

- owpml(hancom-io/hwpx-owpml-model) 또는 한컴 스펙에서 `SHAPE_REVERSAL_TRIANGLE`
  정식 철자 확인:
  - 정식이 TRIANGLE이면 → 파서·serializer 동시 정정(양쪽 동일 철자라 한쪽만 고치면
    roundtrip 깨짐), 화이트리스트는 TRIANGLE.
  - 정식이 TIRANGLE(한컴 오타 그대로)이면 → 현행 유지, 화이트리스트에 TIRANGLE 명시
    + 사유 주석.
- 확인 불가 시 현행 유지(roundtrip 정합) + 화이트리스트 명시 + 미확인 기록.

### 1.4 보고: `_stage1.md` (발견·정정·화이트리스트 근거)

## 2단계 — 전수 검증 + 문서

1. `cargo test` 전체 + `cargo test --test hwpx_roundtrip_baseline` (B=0 유지)
2. CI급 (release-test + fmt + clippy)
3. 매뉴얼: 검사 인프라 문서화 (신규 열거 방출 추가 시 화이트리스트 갱신 절차)
4. 최종 보고서 (검사 커버리지 + 발견 토큰 + numType 가드)

## 위험 관리

| 위험 | 단계 | 대응 |
|------|------|------|
| 화이트리스트가 owpml 미반영 토큰 오탐 | 1 | 실물 ∪ 스펙 — 미확인은 명시 허용 + 기록 |
| TIRANGLE 정정이 한쪽만 → roundtrip 회귀 | 1 | 파서·serializer 동시 정정 (동일 철자 보장) |
| 46함수 전수는 과대 | 1 | 핵심 속성 우선, 확장 가능 헬퍼 구조 |
