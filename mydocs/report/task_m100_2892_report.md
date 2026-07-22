# task m100-2892: 개체 설명문/수식 스크립트 입력 길이 가드

## 이슈

- edwardkim/rhwp#2892 — 개체 설명문/수식 스크립트 입력 길이 미검증으로 인한 CTRL_DATA 손상 방지
  (#2851/#2862/#2866/#2878 형제 필드 재발)

## 근거

`src/serializer/byte_writer.rs:70-76`의 `write_hwp_string()`은 UTF-16 코드 유닛 수를
`as u16`으로 캐스팅해 길이 프리픽스를 만든다. 검증 없이 65536자 이상 문자열이 들어오면
캐스팅이 랩어라운드되어 손상된 레코드가 만들어진다. 이 sink에 도달하는 두 입력 경로를
확인했다.

- `src/serializer/control.rs:1905` — `w.write_hwp_string(&common.description)` ←
  `picture-props-dialog.ts`의 개체 설명문 서브 대화상자(`showDescriptionPrompt()`)
- `src/serializer/control.rs:2386` — `w.write_hwp_string(&eq.script)` ←
  `equation-editor-dialog.ts`의 수식 스크립트 입력(`scriptArea`)

참고: `equation-props-dialog.ts`의 스크립트 필드는 `readOnly = true`인 조회 전용이라
편집 경로가 아니므로 스코프에서 제외했다.

## 변경 사항

- `rhwp-studio/src/ui/picture-props-dialog.ts`
  - `MAX_OBJECT_DESCRIPTION_LEN = 4000` 상수 추가
  - `showDescriptionPrompt()`의 textarea에 `maxLength` 지정, 상한 초과 시 에러 라벨 표시 후
    저장 거부
  - `handleOk()` 진입 시 방어적으로 한 번 더 상한 검증(초과 시 서브 대화상자 재표시)
- `rhwp-studio/src/ui/equation-editor-dialog.ts`
  - `MAX_EQUATION_SCRIPT_LEN = 8000` 상수 추가
  - `scriptArea`에 `maxLength` 지정, 에러 라벨 요소 추가
  - `handleOk()`에서 상한 초과 시 에러 라벨 표시 후 저장 거부; 대화상자 `open()` 시
    에러 라벨 초기화
- `rhwp-studio/tests/object-description-equation-script-length-guard.test.ts` (신규)
  - 두 상수가 65536보다 충분히 작은지 소스 가드로 검증
  - `handleOk()`가 상한 초과 시 저장을 거부하는 가드 분기를 포함하는지 소스 가드로 검증

`.rs` 파일은 수정하지 않았다.

## 작업 중 발견한 사항 — worktree 브랜치 노후화

작업 시작 시 이 worktree(`rhwp-wt-u`)의 현재 브랜치가 `push-2878`였고, 이는
`origin/devel`(최신 tip: `95509062` PR #2834 merge)에서 상당히 갈라진 상태였다. 특히
`picture-props-dialog.ts`는 `origin/devel`에서 `handleOk()`가 `buildPicturePropsPatch()` /
`captureApplyForm()` / `applyPropertyPatch()`를 사용하는 구조로 리팩터링되어 있었던 반면,
`push-2878` 브랜치의 파일은 그 이전의 인라인 `updated: Record<string, unknown>` 구조였다.

`git fetch origin devel` 후 `origin/devel`을 베이스로 새 브랜치
(`task/m100-2883-object-desc-eq-script-len-guard`)를 만들고, 두 파일 중
`equation-editor-dialog.ts`는 깨끗이 cherry-pick되었지만 `picture-props-dialog.ts`는
충돌이 발생해 실제 `origin/devel`의 최신 `handleOk()` 구조에 맞춰 다시 작성했다.
(`git stash`는 사용하지 않았다 — 임시 커밋 + cherry-pick으로 처리했다.)

## 검증

- `npm test` — 501개 중 500 통과, 1 실패(`cell-flow-boundary.test.ts`, 이번 변경과 무관한
  기존 known-failure). 신규 테스트 2건(`개체 설명문/수식 스크립트 상한은 ...`,
  `handleOk()가 상한 초과 시 저장을 거부한다`) 모두 통과.
- `npx tsc --noEmit` — 기존 베이스라인과 동일하게 `@wasm/rhwp.js` 모듈 미발견 TS2307 2건만
  존재(`wasm-bridge.ts`, `hwpctl/index.ts`), 신규 에러 없음.
