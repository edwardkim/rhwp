# Task M100 #1904 2단계 완료보고서 — 준비운동: object_ops 도메인별 분할

- 이슈: #1904 / 브랜치: `local/task1904` / 작성일: 2026-07-04 / 단계: 2/4

## 수행 내용

`commands/object_ops.rs`(9,845줄, 7개 도메인 응집) → `commands/object_ops/` 8파일 분할.
**함수 이동만** — impl DocumentCore 분산으로 메서드 경로·외부 인터페이스 무변경.

| 파일 | 줄수 | 내용 |
|---|---|---|
| picture.rs | 3,737 | 그림 23fn + 인라인 테스트 2블록(issue_1151 계열) |
| shape.rs | 2,582 | 도형 22fn + resize_clamp 테스트 |
| table.rs | 1,493 | 표/셀 17fn |
| note.rs | 780 | 각주/미주 13fn |
| equation.rs | 473 | 수식 9fn |
| common.rs | 411 | 공통 속성 9항목 + insert_new_number |
| connector.rs | 347 | 커넥터 3fn |
| mod.rs | 18 | 모듈 선언 + `MIN_SHAPE_SIZE`(pub(crate) — 다도메인 사용) |

수반된 기계적 조정(로직 무변경): ① 함수 내부 `super::super::helpers` 상대경로 →
`crate::document_core::helpers` 절대경로 정규화 ② 도메인 간 교차 호출되는 private 연관
함수 23개 `pub(crate)` 승격(crate 외부 표면 불변) ③ cargo fix 로 미사용 import 제거.

## 게이트 (manifest 기준 전수)

| 게이트 | 결과 |
|---|---|
| fmt / clippy / 워닝 | clean / 무경고 / 0 |
| 전체 테스트 | **FAILED 0** (2,858 passed — 이동한 인라인 테스트 39개 포함 무손실) |
| **OVR 5샘플** | baseline(`00014ecf`) 대비 **회귀 0건** 전수 |
| golden SVG / roundtrip 3종 | 8 / 4·3·3 전부 ok |

## 효과 (거버넌스 관점)

- SOLID **S**: 변경 이유 7가지가 한 파일 → 도메인별 1가지로 분리. 컨트리뷰터 PR 충돌면 축소.
- 복잡도: 최대 파일 9,845 → 3,737 (−62%). 단 1,200줄 초과 파일 수는 1→3(picture/shape/table)
  — **총량 분해는 후속**(picture 의 테스트 2블록 분리 등)으로 개선 여지, 정직 기록.

## 다음 단계

3단계: `typeset_section_with_variant`(7,059줄·CC 282) 해체 라운드 1 — 분기 비접촉 추출 2~3 PR.
