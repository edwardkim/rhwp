# Stage 4 완료보고서 — Task #1171

- **이슈**: [#1171](https://github.com/edwardkim/rhwp/issues/1171)
- **브랜치**: `local/task1171`
- **단계 목표**: 그림 속성 대화상자가 글상자 picture 의 cellPath 로 by_path API 를 호출하도록 정합.
- **작성일**: 2026-06-02

## 결론 — 코드 변경 불필요 (검증으로 확정)

`rhwp-studio/src/command/commands/insert.ts:272-287` 의 기존 cellPath 재구성이 글상자 picture
(depth-1)에 그대로 올바르게 동작함을 확인했다. 따라서 **insert.ts / picture-props-dialog.ts /
cursor.ts 는 무변경**.

### 근거
- 재구성 조건: `ref.cellIdx !== undefined && ref.cellParaIdx !== undefined &&
  ref.outerTableControlIdx !== undefined && (type image/shape/line)`. JS 에서 `0 !== undefined`
  는 참이므로 sentinel 값 0 에도 조건 성립.
- Stage 1 의 `last_image_indices()` 가 글상자 sentinel `{control_index:0, cell_index:0,
  cell_para_index:0}` 에서 (cellIdx=0, cellParaIdx=0, outerTableControlIdx=0) 을 채우므로,
  재구성 결과 = `[{controlIdx:0, cellIdx:0, cellParaIdx:0}]`, innerControlIdx=`ref.ci`(inner picture).
- 키 이름(`controlIdx`/`cellIdx`/`cellParaIdx`)이 백엔드 `parse_cell_path_json` 규약과 일치.
- E2E(Stage 3 보고서)에서 동일 cellPath 형식의 `getCellPicturePropertiesByPath`/
  `setCellPicturePropertiesByPath` round-trip(width 15040→20040) 성공으로 확정.

### 후속 (범위 밖)
다단계 중첩(표→글상자→표→picture) 은 스칼라 재구성이 depth-1 만 만들어 부족하다. 그 경우
`ref.cellPath`(findPictureAtClick 이 주는 full array) 직접 사용 + 키 변환이 필요하나, 본 이슈
(depth-1 글상자)에서는 불요. 수행계획서 §8 후속으로 남긴다.

## 검증
- 행위 검증은 Stage 3 보고서의 E2E(`textbox-picture-1171.test.mjs`) round-trip 으로 통합 수행.
- `npx tsc --noEmit`: 프런트 변경 파일(input-handler-mouse.ts, input-handler-picture.ts) 에러 0.
