# Task M100 #1392 — 1단계 완료 보고서 (전수 측정 + 경로 확정)

- 브랜치: `local/task1392`
- 작성일: 2026-06-14
- 코드 수정: 없음 (조사 전용)
- 산출물: `output/poc/task1392/shapecomment_dist.tsv`

## 1. 전수 분포 (이슈 범위 대폭 확장)

이슈 본문은 aift만 보고 "도형 설명 15건"이라 했으나, samples/hwpx 전수 정밀 계수
(깊이 추적 파서 — 직접 부모 기준):

| 부모 요소 | 전수 건수 | 현재 RT 보존 | 결손 원인 |
|-----------|----------|------------|-----------|
| **pic** (그림) | 148 | **0** | `write_picture`(picture.rs) shapeComment 미방출 |
| **equation** (수식) | 44 | **0** | `parse_equation`(:4820) shapeComment arm 부재 → **IR 미적재** + `render_equation` 미방출 |
| **rect** (사각형) | 30 | **보존** (온새미로 1/1 실측) | 이미 정상 — `write_rect`→`write_shape_comment`(shape.rs:109) |
| **container** (묶음) | 7 | **0** | `parse_container`(:3703) shapeComment arm 부재(caption arm만 #1403 존재) → IR 미적재 + `write_container_close` 미방출 |
| **합계** | **229** (27파일) | rect 외 전량 소실 | |

→ 단일 경로(pic)가 아니라 **4경로 중 3경로 결손**. aift 15건도 pic 13 + container 2.

## 2. 경로별 결손 축 확정

| 경로 | parser | serializer | 비고 |
|------|--------|-----------|------|
| pic | ✓ 정상 (`common.description`, :2182) | ✗ 미방출 | serializer만 |
| equation | ✗ arm 부재 (`parse_equation`) | ✗ 미방출 (`render_equation` 문자열) | **파서+serializer** |
| container | ✗ arm 부재 (`parse_container`, caption만) | ✗ 미방출 (`write_container_close`) | **파서+serializer** |
| rect | ✓ 정상 (`parse_shape_object`, :3464) | ✓ 정상 | 무수정 (기준선) |

- IR: pic/rect/equation/container 모두 `common: CommonObjAttr` → `description` 필드 보유.
  파서가 적재만 하면 serializer가 동일 값 방출 가능.

## 3. 방출 순서 (실물 확정)

- aift pic 내 caption·shapeComment 공존 9건 전수 **caption → shapeComment** 순서
  (OWPML 자식 순서 일치). → pic/container serializer 방출은 **caption 직후**.
- rect(기존 정상)는 outMargin → caption(#1403) → shapeComment 순서 — 동일 규칙.
- equation은 caption 미보유(IR·실물 모두) → outMargin 직후.

## 4. 게이트 동승 영향 사전 판정

- 현재 게이트는 description 미비교(사각). 2단계 수정 후 4경로 모두 parse→serialize→
  재parse 대칭 → **신규 xfail 0 예상**.
- equation/container는 파서 적재가 신규로 생기므로, 적재 후 재파싱 대칭을 2단계
  실샘플 테스트로 보증.

## 5. 구현계획 보정 (승인 요청)

원안(pic 단일 경로 2단계)을 **4경로 커버**로 확장. 단계 수(3)는 유지하되 2단계를
세분:

- **2a. 파서**: `parse_equation`·`parse_container`에 `b"shapeComment"` arm 추가
  (`read_dutmal_text` — pic/rect와 동일 헬퍼). pic/rect는 무수정.
- **2b. serializer**: `write_shape_comment` `pub(super)` 공유 →
  `write_picture`(caption 직후) / `render_equation`(outMargin 직후) /
  `write_container_close`(caption 직후) 방출. rect 무수정.
- **2c. 게이트**: `ObjectComment` 동승 — Picture arm + `diff_shape_char_shapes`
  (도형 description 접근자) + equation은 top-level 컨트롤 비교에 description 추가.
  Group 자식은 기존 재귀 자동 동승.
- 테스트: 경로별 방출(순서)·미보유 미방출·게이트 검출 + 실샘플 4종(aift/math-001/
  온새미로/k-water-rfp) 왕복.

3단계(전수 검증 + 문서)는 불변.

승인 요청드립니다.
