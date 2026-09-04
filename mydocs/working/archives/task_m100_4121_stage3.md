# Task M100-4121 Stage 3 완료보고서 — HF 선택 소비자와 history 계약

## 결과

머리말/꼬리말 선택을 삭제·입력·IME·평문 붙여넣기·복사·잘라내기·부분 글자 서식에
연결했다. 범위 mutation은 Stage 1 코어 API를 한 번 호출하는 HF 전용 snapshot command로
원자화했으며, Undo/Redo는 target·caret·`preferredPage`와 연산별 선택 정책을 함께
복원한다.

이번 Stage까지 선택 생성·표시와 주요 소비자가 연결됐지만, 전체 Rust/Studio/WASM 게이트와
Both/Odd/Even Header/Footer의 최종 통합 사용자 여정은 Stage 4에 남아 있다. 따라서
#4121 전체가 해결됐다고 판정하거나 이슈를 닫지 않는다.

## 선택을 소비하는 편집 경로

- `Backspace`, `Delete`, cut은 선택 범위를 `replaceRangeInHeaderFooter(..., '')` 한 번으로
  삭제한다.
- typing과 실제 브라우저 CompositionEvent 기반 IME 입력은 기존 선택을 한 번 치환한다.
  composition 중간 입력은 별도 history entry로 나누지 않고 최종 조합 결과까지 같은
  snapshot에 담는다.
- paste는 HF 모드에서 `text/plain`을 사용한다. 줄바꿈은 Stage 1 범위 치환 API를 통해
  HF 문단 경계로 보존한다.
- copy는 Stage 1 `copySelectionInHeaderFooter`의 평문과 기존 rhwp fallback HTML marker를
  clipboard event에 제공한다. cut은 copy 성공 뒤에만 삭제한다.
- toolbar/shortcut 글자 서식은 HF 선택에만 Stage 1 부분 서식 API를 적용한다. 선택이 없는
  HF 캐럿의 다음 입력 서식 예약은 계획대로 비범위다.

## history와 선택 복원

`SubmodeSelectionSnapshotCommand`를 추가해 실행 전후 HF 편집 문맥과 선택을 별도로
기록한다. 실행 뒤 문맥은 실제 코어 결과의 새 문단·offset으로 갱신하므로 다문단 붙여넣기와
IME 최종 캐럿도 정확히 복원한다.

| 연산 | 실행 직후 | Undo | Redo |
| --- | --- | --- | --- |
| Delete/Backspace/cut | 시작점의 접힌 캐럿 | 삭제 전 HF 선택 복원 | 다시 삭제, 접힌 캐럿 |
| typing/IME/paste | 삽입 끝의 접힌 캐럿 | 내용·캐럿만 복원, 선택 없음 | 다시 치환, 접힌 캐럿 |
| 부분 글자 서식 | 같은 HF 선택 유지 | 이전 서식과 같은 선택 | 변경 서식과 같은 선택 |

HF 선택 복원은 `Cursor.selectHeaderFooterRange()`가 현재 target과 문단 수·문자 offset을
모두 검증한 뒤에만 수행한다. stale/invalid 범위는 기존 선택 상태를 오염시키지 않고
fail-closed한다. 복원되는 편집 문맥에는 원래 `preferredPage`도 포함된다.

## RED/GREEN 회귀

초기 focused test 7건은 HF 선택 snapshot, 범위 치환 라우팅, clipboard, 부분 서식과
Undo/Redo 선택 정책이 없어 모두 실패했다. 구현 뒤 다음 계약으로 고정했다.

1. HF target/range를 검증한 뒤에만 history 선택 복원
2. 실행 전후 문맥과 선택을 분리한 `SubmodeSelectionSnapshotCommand`
3. 선택 delete/replace의 단일 코어 범위 API 및 selection history
4. typing/IME의 원자적 선택 치환
5. copy/cut/paste의 HF 우선 라우팅
6. 선택 범위 부분 글자 서식과 선택 유지
7. Undo의 `selectionBefore`, Redo의 `selectionAfter` 계약

## 실제 Chrome E2E

Stage 2의 실제 마우스·Shift 선택과 반복 페이지 scroll-in 여정 위에 다음 동작을 추가했다.

1. 선택 copy의 `text/plain`과 fallback HTML marker 확인
2. cut 삭제와 Undo 선택 복원·Redo 선택 해제 확인
3. 선택 위 typing 치환과 Undo 시 내용만 복원되는 계약 확인
4. `AA\nBB` 평문 paste가 두 HF 문단이 되는지 확인
5. 실제 Bold toolbar 버튼으로 부분 서식 적용 및 Undo/Redo 뒤 선택 유지 확인
6. 실제 CompositionEvent/InputEvent로 선택 위 한글 IME 치환과 Undo/Redo 확인

Stage 2~3을 합친 26개 브라우저 판정이 모두 통과했다. HTML 보고서는 로컬
`output/e2e` 산출물이며 source commit에는 포함하지 않는다.

## 검증 결과

| 검증 | 결과 |
| --- | --- |
| Stage 3 focused Studio test | 7개 통과, 실패 0 |
| 관련 selection/history focused 묶음 | 47개 통과, 실패 0 |
| mutation routing 포함 targeted test | 15개 통과, 실패 0 |
| Stage 1 focused Rust integration 재검증 | 6개 통과, 실패 0 |
| Studio 전체 `npm test` | 1,254개 통과, 실패 0, 기존 skip 1 |
| Studio `npm run build` | TypeScript·Vite build 통과, 239 modules |
| 실제 Chrome `npm run e2e:issue-4121` | 26개 판정 통과 |
| `npm run e2e:manifest-check` | tracked 121 / manifest 121, 통과 |

## Stage 4 경계

Stage 4에서는 Rust/Studio/WASM 전체 필수 게이트를 실행하고 4페이지 Both 및 홀짝 분리
Header/Footer 문서의 mouse/Shift·다문단·scroll·delete/replace/format/copy/cut·Undo/Redo
통합 여정을 실제 브라우저에서 검증한다. 사용자 VDI에서 확인한 한컴 동작과 rhwp의 의도적
차이인 즉시 반복 페이지 repaint 및 본문 클릭 비차단도 최종 보고서에 명시한다.

따라서 Stage 3 체크포인트만으로 원격 push, PR 생성 또는 #4121 close를 진행하지 않는다.
