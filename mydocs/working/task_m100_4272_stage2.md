# Task M100 #4272 Stage 2 — 중첩 셀 선택 텍스트 복사·붙여넣기

- 이슈: [#4272](https://github.com/edwardkim/rhwp/issues/4272)
- 선행 단계: [Stage 1 — 선택 하이라이트](task_m100_4272_stage1.md)
- 기준 commit: `615b104a8` (Stage 1 구현)
- 작업 브랜치: `fix/issue-4272-nested-cell-text-selection`
- 작성일: 2026-08-09 KST
- 상태: 구현 및 focused 로컬 검증 완료

## 목표

중첩 표 안쪽 셀에서 선택 표시가 보이는 데서 끝나지 않고, 사용자가 선택한 텍스트를 Ctrl+C로
복사해 Ctrl+V로 붙여넣는 기본 사용자 여정을 전체 `cellPath` 기준으로 완결한다.

## RED 재현

- 샘플: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 위치: 물리 5쪽 깊이 3 중첩 셀의 `23,504` (offset `0 -> 6`)
- 논리 selection과 하이라이트 1개가 존재하는 상태에서 실제 Ctrl+C를 실행했다.
- 기존 Studio는 평면 API를 다음 인자로 호출했다.
  `copySelectionInCell(0, 7, 1, 0, 0, 0, 0, 6)`
- 이 좌표는 안쪽 셀이 아니라 바깥 셀을 가리켜 `{"ok":true,"text":""}`를 반환했다. 내부
  클립보드는 존재하지만 텍스트가 비어 있어 이어지는 Ctrl+V도 사용자 입력을 붙여넣지 못했다.

물리 5쪽 수정 뒤 작업지시자가 요청한 물리 11쪽 자식 표 셀도 검증했다. 이 선택의 전체 path 마지막
문단은 22지만 호환용 평면 필드 `cellParaIndex`는 0이다. 최초 Stage 2 라우팅은 path API에도 평면
필드 0을 넘겨 다시 빈 문자열을 만들었다. 즉 물리 5쪽은 최내곽 문단도 0이라 우연히 통과한 것이며,
11쪽 검증이 문단 축 퇴행을 드러냈다.

최초 CDP 진단 시 Puppeteer에 지원되지 않는 `keyboard.press('Control+C')` 표기를 사용해 테스트가
제품 코드 진입 전에 실패했다. 이를 제품 실패로 계산하지 않고 `Control` keydown → `c` → keyup으로
교정한 재현만 RED 근거로 사용했다.

## 반영 내용

- Rust native/WASM
  - `copySelectionInCellByPath`: 선택 문단마다 전체 경로의 마지막 `cellParaIndex`만 바꿔 실제
    최내곽 셀 문단을 내부 클립보드에 복사한다.
  - `exportSelectionInCellHtmlByPath`: 같은 경로의 선택 범위를 시스템 클립보드용 HTML fragment로
    변환한다.
- Studio
  - `onCopy`는 깊이 2 이상 셀에서 plain text와 HTML 모두 path API를 사용한다.
  - path API의 시작·끝 문단 축은 `cellParaIndexOf()`로 전체 path의 마지막 엔트리에서 읽는다.
  - 깊이 1 셀과 본문은 기존 API를 유지한다.
  - Ctrl+V는 기존 `DeleteSelectionCommand`의 path 삭제와
    `pasteInternalInCellByPath`를 재사용한다.
- 래칫
  - 실제 샘플 Rust 테스트가 선택 rect뿐 아니라 plain text `23,504`, 내부 클립보드와 HTML
    fragment를 함께 검증한다.
  - Studio source guard가 중첩 셀 plain text·HTML 경로 라우팅과 깊이 1 fallback을 고정한다.
  - CDP E2E가 실제 mouse drag → Ctrl+C → Ctrl+V를 수행한다.
  - 물리 11쪽 문단 22 선택의 path API 호출 인자를 별도 CDP E2E로 고정한다.

## 성능 경계

복사는 사용자 Ctrl+C 이벤트에서만 실행되며 드래그 hot path를 변경하지 않는다. 선택 문단 범위만
순회하고, 문서 전체 순회·페이지 재조판·새 렌더 트리 생성을 추가하지 않는다. Stage 1의 선택 rect
성능 관측도 16-step drag에서 path API 17회, 합계 약 3.6ms로 유지됐다. 이 값은 해당 실행의 진단
수치이며 장비 독립 merge threshold가 아니다.

## 검증

| 검증 | 결과 |
|---|---|
| 실제 샘플 Rust #4272 rect·plain text·HTML 통합 래칫 | 1/1 통과 |
| Studio #4272 copy path 라우팅 + selection page hint focused 테스트 | 2/2 통과 |
| Studio 전체 `npm test` (샌드박스 밖) | 817/817 통과 |
| TypeScript `tsc --noEmit` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |
| E2E 매니페스트 양방향 검사 | 92/92 통과 |
| Docker WASM 표준 빌드 | 통과 |
| 호스트 Chrome CDP #4272 선택 → Ctrl+C → Ctrl+V | 통과 |
| 호스트 Chrome CDP #4272 물리 11쪽 문단 22 선택 → Ctrl+C | 통과 |
| 호스트 Chrome CDP #4252 인접 중첩 표 객체 선택 | 통과 |
| 호스트 Chrome CDP 기존 일반 copy-paste E2E | 통과 |

Studio 전체 테스트는 알려진 `spawnSync()` 샌드박스 `EPERM` 오탐을 피하도록 처음부터 샌드박스 밖에서
실행했다.

## WASM 산출물

- 위치: `pkg/rhwp_bg.wasm` (gitignore 대상, 실행 중인 dev 서버가 참조)
- 크기: 7,712,704 bytes
- SHA-256: `aa08e4cdda05f21388a429db0f620ff2b4e563a3597f06d73b8b1c25b6e3c587`
- `pkg/rhwp.js`, `pkg/rhwp.d.ts`, `pkg/rhwp_bg.wasm.d.ts`에서 두 새 API export를 확인했다.

## CDP 시각·상태 증적

- [선택·복사·붙여넣기 상태 JSON](../../output/4272/nested-cell-text-selection.json)
- [선택 하이라이트 PNG](../../output/4272/nested-cell-text-selection.png)
- [붙여넣기 완료 PNG](../../output/4272/nested-cell-copy-paste.png)
- [HTML 보고서](../../output/e2e/issue-4272-nested-cell-text-selection-report.html)
- [물리 11쪽 복사 상태 JSON](../../output/4272/page11-child-cell-copy.json)
- [물리 11쪽 선택 PNG](../../output/4272/page11-child-cell-copy.png)
- [물리 11쪽 HTML 보고서](../../output/e2e/issue-4272-page11-nested-cell-copy-report.html)

관측 결과:

- 선택 깊이 3, offset `0 -> 6`, 하이라이트 1개
- Ctrl+C 내부 클립보드 `23,504`
- Ctrl+V 뒤 대상 셀 텍스트 `23,504`, 캐럿 offset 6, 선택 해제
- 브라우저 warning/error 0건
- 물리 11쪽 path 마지막 문단 22, offset `66 -> 89`, 복사 텍스트
  ` 다른 목적 등을 위하여 조사권을 남용하여`, path API 시작·끝 문단 인자 22

## 다음 승인 게이트

Stage 2 후보는 준비됐다. 로컬 커밋 뒤 전체 PR 검증 게이트, 원격 push, PR 생성과 이슈
comment·close는 각각 프로젝트 절차의 승인 경계를 따른다.
