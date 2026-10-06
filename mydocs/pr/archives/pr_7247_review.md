# PR #7247 최종 검토 — 셀 내부 평문 붙여넣기의 문단 경계

## Metadata와 처리 경로

2026-09-20 작성 시점 참고값이다. merge 전에 최신 head·CI·mergeability를 재조회한다.

| 항목 | 값 |
| --- | --- |
| PR | [#7247](https://github.com/edwardkim/rhwp/pull/7247), Task #6638: 셀 안 평문 붙여넣기가 개행을 문단으로 나누지 않는 문제 수정 |
| 작성자 | lpaiu-cs |
| Issue | [#6638](https://github.com/edwardkim/rhwp/issues/6638), OPEN |
| Base | devel; 최신 통합 확인 기준 `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db` |
| 검증한 contributor head | `48357de0426fa8afff417ec26eb2e213dccaa6ab` |
| Head 저장소·branch | `lpaiu-cs/rhwp`, `fix/6638-cell-paste-paragraphs` |
| 원 변경 규모 | 1 commit, 4 files, +63 / -2 |
| 상태 | Ready, MERGEABLE / CLEAN, maintainer_can_modify=true |
| 기존 원격 리뷰 | [postmelee COMMENTED](https://github.com/edwardkim/rhwp/pull/7247#pullrequestreview-5256638975), 동일 source SHA |

- base route: collaborator_external_pr, 원 contributor PR에 문서만 추가하는 경로.
- modifiers: intake_and_review, local_validation, review_only_fast_pass.
- loaded documents: pr_review_workflow.md, pr_review/README.md와 위 경로 문서.
- source commit은 원 작성자 lpaiu-cs의 위 commit 하나다. collaborator는 이 리뷰 Markdown만
  single-parent 후속 commit으로 추가하며 원 commit의 author·내용·history를 변경하지 않는다.

## 변경과 검토 결과

`pastePlainText`가 개행을 만나면 셀 안에서는 기존 `SplitParagraphInCellCommand`를,
본문에서는 기존 `SplitParagraphCommand`를 사용한다. 앞 조건에서 셀을 제외하던 누락을 제거하고
이미 있는 셀 커서 반환과 history 경로를 재사용한다. 빈 줄·마지막 개행도 분할 명령을 실행한다.

실제 ClipboardEvent 진입점부터 문단별 텍스트, 마지막 커서, Undo/Redo까지 검토했다.
독립 기대값은 plain text의 LF/CRLF 문단 경계를 보존하는 이슈 계약과 기존 셀 Enter 분할 동작이다.
수정이 필요한 코드 결함은 발견하지 못했다. OS 클립보드와 중첩 셀까지 실행 검증했다고 확대하지 않는다.

## 완료한 검증

Ready 전환 이후 source SHA가 바뀌지 않았다. 아래 직접 검증과 동일 head의 CI를 재사용한다.

| 검증 | 실행·결과 |
| --- | --- |
| 타입·단위 | `rhwp-studio`에서 `npx tsc --noEmit` 통과. `npm test`: 1,758 passed / 2 skipped / 0 failed |
| 실제 브라우저 | 해당 head에서 새로 빌드한 WASM으로 `npm run e2e:issue-6638` 실행. headless Chrome에서 `첫째\n\n셋째\n`가 `['첫째', '', '셋째', '']`로 유지되고 cursor `cellParaIndex=3, charOffset=0`, Undo/Redo 통과 |
| 추가 경계 | 일반 셀의 LF, CRLF, 중간 빈 줄, 마지막 개행, 개행만 있는 입력 각각 텍스트·Undo/Redo assertion 통과 |
| reviewer 음성 대조 | 수정 전 `input-handler-keyboard.ts`로 같은 E2E 실행 시 4문단 대신 `첫째셋째` 1문단으로 실패. 이후 원 head 파일 복구 확인 |
| CI | [CI 35270489840](https://github.com/edwardkim/rhwp/actions/runs/35270489840), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35270489910), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35270489568) 성공 |
| 현재 base 통합 | `git merge-tree --write-tree 722fb38af 48357de04` 통과. tree `8d0dc4dfc96d5542eea7996a0251939a1fb58a5d`, 공백 검사 통과 |

완료 로그는 로컬 `pr7247-npm-test.log`, `pr7247-e2e.log`, `pr7247-negative.log`와 기존 원격 리뷰에
기록했다. 추가 경계 로그의 일반 셀 다섯 사례는 통과했으나, 뒤따른 중첩 셀 진단은 테스트 설정의
노드 접근·셀 경로 오류로 중단됐다. 추가 진단 스크립트 전체가 성공했다고 표시하지 않으며,
중첩 셀은 **미검증**으로 남긴다. 이 오류를 이번 제품 변경으로 재현된 회귀라고 판정할 근거도 없다.

로컬 WASM은 Docker 경로의 환경 문제로 native wasm-pack `--no-opt` 빌드를 사용했다.
제품 Rust source/helper 변경이 없어 Rust lint·전체 회귀는 비해당이다. 제품·테스트 보정 없이
같은 source SHA의 완료 검증을 재사용하므로 이번 문서 추가를 위해 브라우저·단위 전체를 반복하지 않았다.

## 입력 커밋·조판 원칙

검증 입력 커밋 확인: **비해당**. E2E는 저장소의 테스트 코드로 새 문서를 생성하고 `createTable`로
일반 셀을 구성한다. 별도 HWP/HWPX/PDF 파일을 로드하지 않으며, 이슈 첨부 HWP 파일 자체를
검증했다고 주장하지 않는다. 기대값과 입력 문자열은 원 PR의 커밋된 E2E에 포함돼 있다.

조판 원칙·직접 Visual Sweep: **비해당**. 입력 이벤트에서 기존 분할 명령을 선택하는 변경으로,
측정·레이아웃·pagination·paint·baseline 코드를 바꾸지 않는다. 실제 브라우저의 문단 구조와 커서
검증을 수행했으며 한컴 기준 PDF와의 시각 일치 증거로 승격하지 않는다.

## 최종 판정과 남은 단계

- 최종 판정: **승인**. 위 일반 셀 붙여넣기 변경의 코드 검토와 적용 검증을 충족한다.
- 문서 추가 후 최신 head의 required CI 통과와 mergeability 재확인이 병합 전 조건이다.
- 사용자는 이번 단계에서 문서 commit·원 PR push만 승인했다. CI 통과를 알려준 뒤 최신 head를
  재조회하여 기존 COMMENTED와 별도로 최종 APPROVE를 제출하고 병합한다. 이번 단계에서는 CI를
  계속 감시하거나 APPROVE·merge·issue close를 실행하지 않는다.
- 단일 PR이며 추가 코드 보정·통합·복수 선택 단계가 없어 별도 review_impl/report는 생략한다.
  오늘할일을 새로 만들지 않고 검토·검증·후속 조건을 이 리뷰에 기록한다.
- merge 후 실제 merge SHA·devel 포함과 이슈 완료 기준 충족을 확인하고 필요한 종료 코멘트를 남긴다.
  기여자 fork branch는 정리 대상으로 삼지 않는다.
