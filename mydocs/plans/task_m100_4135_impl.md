# Task M100 #4135 후속 구현 계획서 — 블록 합계 실동작·한글 IME 셀 나누기

> 수행계획서: [`task_m100_4135.md`](task_m100_4135.md)
>
> 수동 피드백: [`task_m100_4135_manual_validation.md`](../feedback/task_m100_4135_manual_validation.md)
>
> 기준: `upstream/devel@94ff48d2b81dee5241110db9d2417dffbfb7f9ec`
>
> 브랜치: `codex/issue-4135-contextual-shortcut`
>
> 상태: **Recovery R2 구현·focused 검증 완료, 결과 승인 대기 — R3 제품 소스 변경 금지**

## 1. 문제를 다시 정의한다

기존 WIP는 F5 셀 블록의 `Ctrl/Cmd+Shift+S`를 `table:block-sum`으로 보내는 데는 성공했다.
그러나 명령 본체가 선택 범위를 무시하고 앵커 셀에 `=SUM(above)`를 실행해 첫 셀에 `0`을
썼다. 수정자 없는 셀 나누기도 영문 `key='s'`만 알아 한글 IME에서는 `ㄴ`을 입력했다.

이번 후속 작업은 다음 두 사용자 결과를 바로잡는다.

1. 선택 블록의 **오른쪽 또는 아래 빈 결과 칸**에 행별·열별 합계를 기록한다.
2. 셀 블록에서 수정자 없는 물리 `S`가 한글/영문 입력 상태 모두 셀 나누기를 연다.

## 2. 승인할 수용 계약

### 2.1 블록 합계

| 선택 상태 | 기대 결과 |
| --- | --- |
| 각 행의 숫자 셀 + 선택 범위 오른쪽 끝의 빈 결과 셀 | 각 행 합계를 해당 오른쪽 빈 셀에 기록 |
| 각 열의 숫자 셀 + 선택 범위 아래 끝의 빈 결과 셀 | 각 열 합계를 해당 아래 빈 셀에 기록 |
| 단일 셀, 불연속 제외 셀, 결과용 빈 가장자리 없음 | 문서 변경 없이 거절 |
| 오른쪽 끝 열과 아래 끝 행이 모두 비어 방향이 모호함 | 문서 변경 없이 거절 |
| 일부 계산 dry-run 실패 | 어느 결과 셀도 쓰지 않음 |
| Undo 1회 | 한 번의 블록 합계 결과 전체를 원복 |

결과 셀은 선택 블록에 포함되어야 한다. 가로형은 선택 영역의 맨 오른쪽 열, 세로형은 맨 아래
행이 모두 빈 셀일 때만 방향을 확정한다. 공식 문서가 양쪽 가장자리가 동시에 빈 모호한 선택의
우선순위를 명시하지 않으므로, 추측해 쓰지 않고 fail-closed한다.

### 2.2 셀 나누기·합치기 물리 키

| 입력 | 기대 결과 |
| --- | --- |
| 영문 `s/S`, `code='KeyS'` | `table:cell-split` |
| 한글 `ㄴ`, `key='Process', code='KeyS'` | `table:cell-split`, `ㄴ` 문자 미입력 |
| 영문 `m/M`, 한글 `ㅡ`, `Process/KeyM` | 대칭 계약으로 `table:cell-merge` |
| Ctrl/Meta/Alt가 있는 `S`/`M` | 수정자 없는 셀 명령으로 가로채지 않음 |

`M`은 이번 수동 피드백에 직접 나오지는 않았지만 같은 셀 블록 분기와 같은 IME 결함 구조를
공유한다. `S`만 특례로 고치면 바로 대칭 회귀가 남으므로 같은 Recovery 단계의 물리 키
계약으로 묶는다.

## 3. 설계 경계

### 포함

- 셀 블록 범위·빈 결과 가장자리로 가로/세로 계산 작업 목록을 만드는 순수 planner
- 결과 셀을 제외한 선택 범위만 참조하는 명시적 `SUM(start:end)` 계산식
- 27열 이상도 표현하는 A..Z, AA.. 다중 문자 열 참조의 계산식 파서·평가기 확장
- 모든 결과의 dry-run 성공 뒤 하나의 snapshot operation에서 일괄 기록
- IME 조기 반환보다 앞에서 동작하는 수정자 없는 `KeyS`/`KeyM` 문맥 resolver
- 기존 full/embed `Ctrl/Cmd+Shift+S` 라우팅 WIP의 회귀 검증

### 이번 단계에서 제외

- 계산식 메타데이터 저장과 원본 셀 변경 시 자동 재계산. 현 코어의
  `evaluateTableFormula(writeResult=true)` 자체가 결과 문자열만 저장하므로 별도 코어 기능 설계가
  필요하다. 이번 수용 계약은 **실행 시점의 올바른 결과 배치와 값**까지다.
- 중첩 표·병합 셀이 포함된 블록 계산. 현 flat 계산 API의 좌표 계약으로 잘못된 표나 병합 anchor를
  쓰지 않도록 문서 변경 없이 거절하고 후속 범위로 남긴다.
- 블록 평균·곱의 공식 정합 확장. planner는 재사용 가능하게 만들되 이번 브라우저 수용 판정은
  이슈 단축키가 가리키는 블록 합계에 한정한다.
- 전역 `matchShortcut` 후보 목록 계약의 전면 개편, embed 파일 수명주기 정책 변경.

## 4. Recovery 단계별 구현과 승인 게이트

기존 수행계획의 Stage 1~4는 단축키 라우팅 WIP의 역사적 단계명으로 그대로 보존한다.
아래 후속 작업은 그 번호를 재사용하지 않고 `Recovery R1~R4`로 구분한다.

### Recovery R1 — 누락된 RED 계약과 공식 기준선 고정

- Studio 순수 테스트에 가로/세로 결과 배치, 빈 결과 가장자리, 모호/불연속/병합/중첩 거절,
  all-or-nothing 작업 목록 계약을 추가한다.
- 키보드 테스트에 영문/한글/Process `KeyS`와 대칭 `KeyM`, 수정자 가드를 추가한다.
- Rust 계산식 테스트에 `Z`, `AA`, 다중 문자 열 범위를 추가해 현 제한을 RED로 고정한다.
- 제품 코드는 바꾸지 않는다.
- focused RED 결과와 실패 이유를 `mydocs/working/task_m100_4135_recovery_r1.md`에 기록하고
  커밋한다.
- **중단점**: Recovery R1 결과를 보고한 뒤 작업지시자의 별도 승인을 기다린다.

### Recovery R2 — 선택 범위 블록 합계 구현

- 계산식 셀 참조를 단일 `char`에서 다중 문자 열 식별자로 확장하고 base-26 열 좌표를 검증한다.
- 순수 planner가 선택 범위와 셀 공백/병합 정보를 받아 가로 또는 세로 계산 job을 만든다.
- `blockCalcCommand()`의 `canExecute`와 실행 경로를 실제 다중 셀 블록에 맞춘다.
- 모든 job을 먼저 dry-run하고, 전부 성공할 때 한 snapshot에서 결과 셀들을 기록한다.
- Recovery R1 RED와 계산식/undo focused 테스트를 GREEN으로 만든다.
- 결과를 `mydocs/working/task_m100_4135_recovery_r2.md`에 기록하고 커밋한다.
- **중단점**: Recovery R2 결과를 보고한 뒤 작업지시자의 별도 승인을 기다린다.

### Recovery R3 — 한글 IME 셀 나누기·합치기 구현

- 수정자 없는 물리 `KeyS`/`KeyM` resolver를 IME 조기 반환보다 앞에 배치한다.
- 한글 자모와 `Process` 이벤트를 소비해 `ㄴ`/`ㅡ`가 셀에 들어가지 않게 한다.
- `Ctrl/Cmd+Shift+S` 문맥 resolver, 일반 Save As, embed consume-only 계약을 보존한다.
- 키보드 focused 테스트와 기존 contextual shortcut 테스트를 GREEN으로 만든다.
- 결과를 `mydocs/working/task_m100_4135_recovery_r3.md`에 기록하고 커밋한다.
- **중단점**: Recovery R3 결과를 보고한 뒤 작업지시자의 별도 승인을 기다린다.

### Recovery R4 — 통합·실브라우저 검증과 최종 판정

- focused Node/Rust, source-side test tier policy, Studio 전체 테스트·production build, embed E2E를
  실행한다.
- 로컬 서버에서 작업지시자와 동일한 여정으로 다음을 실측한다.
  1. 숫자 3열 + 빈 결과 1열을 포함해 여러 행 선택 → `Cmd+Shift+S` → 행별 합계
  2. 숫자 여러 행 + 빈 결과 1행을 포함해 여러 열 선택 → `Cmd+Shift+S` → 열별 합계
  3. 한글/영문 각각 수정자 없는 `S` → 셀 나누기 대화상자, 문자 미입력
  4. 셀 블록 밖 `Cmd+Shift+S` → Save As, embed에서는 기존 차단 계약
- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`를 통과시킨다.
- 최종 보고서는 Recovery R4 결과 승인 뒤에만 작성·확정한다.
- **중단점**: 통합 결과를 보고하고 최종 결과 승인을 기다린다. push·PR은 여전히 별도 승인이다.

## 5. 예상 변경 파일

| Recovery | 파일 | 변경 |
| --- | --- | --- |
| R1 | `rhwp-studio/tests/**`, `src/document_core/table_calc/**`의 test module | RED 계약 |
| R2 | `src/document_core/table_calc/{tokenizer,parser,evaluator}.rs` | 다중 문자 열 참조 |
| R2 | `rhwp-studio/src/command/block-calculation-plan.ts` | 선택 범위→계산 job 순수 planner |
| R2 | `rhwp-studio/src/command/commands/table.ts` | multi-cell·dry-run·snapshot 실행 |
| R3 | `rhwp-studio/src/command/contextual-shortcut.ts`, `rhwp-studio/src/engine/input-handler-keyboard.ts` | IME 물리 `S`/`M` 라우팅 |
| R4 | `mydocs/working/**`, `mydocs/report/**`, `mydocs/orders/20260828.md` | 검증·최종 판정 기록 |

## 6. 리스크와 중단 조건

| 리스크 | 대응/중단 조건 |
| --- | --- |
| 선택 방향을 잘못 추론해 원본 셀 덮어쓰기 | 결과 가장자리 전체가 비어 한 축만 확정될 때만 실행 |
| 여러 결과 중 일부만 기록 | 전체 dry-run 선행, 한 snapshot; preflight와 write 결과 불일치 시 Recovery R2 중단 |
| 병합/중첩 좌표가 flat API와 불일치 | planner 입력에서 감지해 no-op; 지원 확대는 별도 승인 |
| 계산식 parser 변경이 기존 A1/와일드카드 계약 회귀 | 기존 전체 table_calc 테스트 + Z/AA 경계 테스트 |
| IME 조합 문자가 명령 뒤 입력됨 | `preventDefault`와 IME 전 조기 resolver를 focused·브라우저 양쪽에서 확인 |
| 공식 자동 재계산과 현 plain-result 모델 차이 | 이번 완료 범위를 즉시 결과 정합으로 명시하고 후속 코어 설계 없이 확장하지 않음 |

## 7. 기존 WIP 처리

| commit | 보존 판정 |
| --- | --- |
| `a191509f9` | 계획 승인 전 만들어진 단축키 RED WIP |
| `2141666e0` | 단축키 도달성 구현 WIP; Recovery 단계에서 회귀 후보로 재검증 |
| `1d3c78c1d` | 과한 완료 판정을 담은 기록 WIP; 정정 공지를 추가하고 이력 보존 |

세 커밋은 삭제·squash·amend하지 않는다. 이 계획 승인 뒤에도 각 Recovery 단계는 구현·focused
검증·보고·커밋을 한 묶음으로 끝내고, 작업지시자가 결과를 승인하기 전에는 다음 Recovery 단계를
시작하지 않는다.
