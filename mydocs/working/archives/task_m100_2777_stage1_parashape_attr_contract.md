---
kind: working
status: active
issue: 2777
stage: 1
last_verified: 2026-08-14
---

# #2777 Stage 1: ParaShape 자동 간격과 쪽나눔 보호 비트 정규화

## 배경

기존 HWPX 경로는 `widowOrphan`을 `attr2 bit 5`에 저장했다. 이 자리는 편집 계층의
`autoSpaceKrNum` 정본 자리와 같아, 자동 간격 편집이 외톨이줄 보호를 켜고 HWPX 저장 시
`widowOrphan="1"`을 방출하는 충돌이 발생했다. 같은 경로에서 `autoSpacing`은 파싱되지 않고
직렬화도 `0/0`으로 고정되어 값이 왕복마다 유실됐다.

## 계약

- `breakSetting`: `attr1 bits 16-19` (`widowOrphan`, `keepWithNext`, `keepLines`, `pageBreakBefore`)
- `autoSpacing`: `attr2 bits 4-5` (`eAsianEng`, `eAsianNum`)
- `verticalAlign`: `attr1 bits 20-21`

## 변경

- HWPX 파서와 직렬화기를 위 계약으로 대칭 배선한다.
- 렌더러와 서식 조회에서 구 `attr2 5-8` breakSetting 폴백을 제거한다.
- HWP5 입력에서 한컴이 사용하지 않는 구 `attr2 6-8`만 `attr1 17-19`로 이관한다.
- 의미가 모호한 `attr2 bit 5`는 자동 간격 정본으로 유지해 충돌을 다시 만들지 않는다.

## 회귀 단정

- HWPX `breakSetting`과 `autoSpacing`이 서로 다른 비트에 파싱되는지 확인한다.
- `autoSpaceKrNum`만 켠 문단이 `widowOrphan="0"`으로 저장되는지 확인한다.
- 식별 가능한 구 HWP5 `attr2 6-8` 이관과 모호한 bit 5 보존을 확인한다.

## 검증 상태

회귀 단정을 코드에 추가했다. 실행 검증은 이 stage에서 수행하지 않았다.
