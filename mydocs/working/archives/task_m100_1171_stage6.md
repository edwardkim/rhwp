# Stage 6 완료보고서 — Task #1171 (후속)

- **이슈**: [#1171](https://github.com/edwardkim/rhwp/issues/1171)
- **브랜치**: `local/task1171`
- **단계 목표**: 글상자(Shape text_box) **위에 이미지 드롭** 시 삽입 실패 결함 수정 —
  한컴처럼 본문(body) 레벨 떠있는 개체(글상자 sibling)로 삽입.
- **작성일**: 2026-06-03
- **발견 경위**: 작업지시자 실환경 수동 테스트 중 발견. 본 이슈의 hit-test/속성 범위와는
  다른 **이미지 삽입** 경로의 별개 결함이나, 같은 "글상자 경로 → 표 전용 resolver" 계열이라
  후속으로 흡수(작업지시자 승인).

## 현상 / 근본 원인 (pre-existing, #1171 변경과 무관)

- 현상: 글상자 위에 이미지를 넣으면 삽입 실패. 콘솔
  `그림 삽입 실패: 렌더링 오류: 경로[0]: controls[N]가 표가 아닙니다`.
- 한컴: 글상자 위 이미지는 본문 레벨 독립 떠있는 개체(글상자 sibling). 글상자 이동과 무관.
- 원인 사슬:
  1. `finishImagePlacement`(input-handler-table.ts)가 드롭 지점 `hitTestFromEvent` 조회.
  2. 글상자 내부 hit 은 기존(Task #919/#1151)대로 `cellPath`(글상자 sentinel)를 반환 →
     `inCell=true` 로 판정되어 `cellPathJson` 에 글상자 경로가 들어감.
  3. `insertPicture` 네이티브(`insert_picture_native`, object_ops.rs:1855)가
     `resolve_cell_by_path`(**표 전용**)로 경로 검증 → 글상자라서 거부 → 삽입 실패.
- **회귀 아님 확인**: 본 브랜치 diff 에 `cursor_rect.rs`(글상자 cellPath hit)·
  `insert_picture_native`(라인 1813~) 모두 없음. 둘 다 local/devel 선재 코드.

## 수정 (`rhwp-studio/src/engine/input-handler-table.ts` `finishImagePlacement`)

드롭 hit 이 글상자(`hit.isTextBox`)면 표 셀과 달리 cellPath 를 쓰지 않고 본문
para(`parentParaIndex` = 글상자를 소유한 본문 문단)에 floating 으로 삽입:
- `isTextBoxHit = hit.isTextBox === true`
- `inCell = cellPath.length>0 && parentParaIndex!=undefined && !isTextBoxHit`
- `paraIdx = (inCell || isTextBoxHit) ? parentParaIndex : paragraphIndex`
- `cellPathJson = inCell ? JSON.stringify(cellPath) : ''`

→ 글상자: 본문 분기(`insert_picture_native` 본문 floating, paper-offset 위치)로 삽입되어
한컴처럼 본문 sibling. 실제 표 셀(#1151)·본문 클릭은 기존 동작 유지.

## 검증 (TDD)

신규 E2E `rhwp-studio/e2e/textbox-picture-insert-1171.test.mjs`:
- 가짜 글상자 hit(parentParaIdx=25, isTextBox, cellPath sentinel) 주입 후
  `finishImagePlacement` 호출 → 본문 para25 의 cellPath 없는 image 증가 검증.
- **수정 전(red)**: before=0 → after=0 (삽입 실패, 콘솔 warn). FAIL.
- **수정 후(green)**: before=0 → after=1 (본문 sibling 삽입). PASS.
- 회귀: `textbox-picture-1171.test.mjs`(Stage1-4 select/속성) 통과 유지.
- `npx tsc --noEmit`: input-handler-table.ts 에러 0.

## 범위 메모

- 본 수정은 **드롭 제스처**에 한정. 표 셀 안 이미지 삽입(#1151)은 불변.
- 글상자 **내부 콘텐츠로서** 이미지를 편집 중 삽입하는 경로(글상자 텍스트 편집 상태)는
  별도이며 본 이슈 범위 밖.
