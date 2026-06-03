# Task #1253 구현 계획

## Stage0: 분석과 기준 고정

- GitHub 이슈 #1253 등록
- `local/task_m100_1253` 브랜치 생성
- 기존 HWP5/HWPX/Studio JSON 매핑 확인
- 한컴 UI 기준으로 RHWP-studio UI 차이를 정리

## Stage1: 미주 모양 UI 정합

- `rhwp-studio/src/ui/endnote-shape-dialog.ts`를 한컴 기준에 맞게 보강한다.
- 구분선 종류, 굵기, 색 선택 UI가 한컴처럼 실제 선 모양을 보여주도록 한다.
- `미주 사이`, `구분선 아래` 입력값이 내부 필드명 때문에 혼동되지 않도록 표시/적용 흐름을 명확히 한다.
- 필요하면 `EndnoteShapeSettings`에 사용자 의미가 드러나는 별칭 또는 주석을 추가한다.

## Stage2: 간격 매핑과 렌더 공통 로직 검토

- `get_endnote_shape_native`/`apply_endnote_shape_native`에서 JSON 이름과 내부 저장 필드의 의미를 재검토한다.
- HWP5 파서와 HWPX 파서의 `betweenNotes`, `belowLine`, `aboveLine` 매핑을 현재 스펙 보완 주석과 대조한다.
- `src/renderer/typeset.rs`, `src/renderer/height_cursor.rs`의 공통 미주 간격 정책이 `미주 사이`와 `구분선 아래`를 독립적으로 소비하는지 확인한다.
- PR #1232에서 도입한 공통 로직이 한컴 기준 UI 값과 일치하도록 필요한 부분만 보정한다.

## Stage3: 검증과 보고

- 대상 회귀 테스트를 실행한다.
- 한컴오피스 2024 스크린샷 기준으로 RHWP-studio UI와 렌더링을 작업지시자가 시각 확인한다.
- 단계 보고서와 최종 보고서를 작성한다.
- 커밋 후 PR을 준비한다.
