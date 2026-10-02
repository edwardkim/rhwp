# #7333 Stage 9 — 100% 두 쪽 보기의 다음 쪽 객체 선택 표시

## 분석

사용자가 100% 두 쪽 보기에서 7쪽 도형을 클릭했을 때 선택 핸들이 8쪽에 나타나는 것을
확인했다. 클릭 경로는 `findPictureAtClick()`이 실제 hit 쪽을 `PictureSelectionRef.pageIndex`에
저장한다. 그러나 `renderPictureObjectSelection()`과 `findPictureBbox()`는 그 ref를 다시 해석할 때
`orderedControlLayoutPages()`로 대상 쪽 뒤의 모든 쪽까지 검색한다. 클릭으로 확정된 쪽에서 control을
찾지 못한 경우 동일 주소의 다른 layout 항목을 선택 표시로 사용할 수 있으므로, pointer selection의
페이지 소유 계약이 rendering fallback에서 깨진다.

이 회차는 pointer selection에 기록된 유효 `pageIndex`가 있으면 해당 쪽만 control layout을
검색하도록 한다. pageIndex가 없는 키보드·명령 selection은 기존 전체 검색을 유지한다. 실제 클릭
쪽에 control이 없으면 다음 쪽으로 그리지 않고 선택 표시를 지운다. 이 규칙은 저장 address가 여러
쪽 layout에 나타나는 경우에도 클릭 위치와 선택 표시가 다른 쪽으로 이동하지 않게 한다.

회귀 검사는 100% 두 쪽 보기에서 7쪽 ref가 7쪽만 조회하고, page hint가 없는 기존 selection은
전체 검색 순서를 유지함을 확인한다. Studio TypeScript 검사와 해당 Node 회귀 검사를 실행한다.

## 코드 수정

`rhwp-studio/src/engine/picture-hit-policy.ts`에
`exactSelectedControlLayoutPages()`를 추가했다. 유효한 pointer `pageIndex`는 단 하나의
layout page 배열로 변환하고, page hint가 없거나 범위를 벗어난 선택만 기존처럼 전 페이지를
검색한다.

`findPictureBbox()`와 `renderPictureObjectSelection()`이 이 계약을 사용하게 바꿨다. 따라서
7쪽에서 고른 도형 ref는 7쪽 control layout에서만 bbox를 찾고, 없으면 선택 renderer가 clear된다.
8쪽의 같은 주소/후보를 fallback으로 찾아 핸들을 표시하지 않는다. 기존
`orderedControlLayoutPages()`는 page hint가 없는 다른 소비처의 전체 fallback 계약을 유지한다.

## 검증 결과

- `git diff --check` 통과
- `cd rhwp-studio && node --test tests/picture-hit-policy.test.ts`
  - 4 passed, 0 failed
  - pointer page hint가 있으면 `[selectedPage]`만, hint가 없거나 범위를 벗어나면 전체 page
    순서를 쓰는 회귀 검사를 추가했다.
- `cd rhwp-studio && npx tsc --noEmit` 통과
- `npx vite --host 0.0.0.0 --port 7700`가 `/Users/tsjang/rhwp/rhwp-studio`에서 실행 중임을
  확인했다. 이 회차는 Studio selection policy의 TypeScript 변경으로, Rust/WASM 산출물을
  바꾸지 않는다. 포트 7700의 Vite module은 변경된 source를 HMR/새로고침에서 사용한다.

Native/WASM 문서 조판 출력은 변경하지 않았으므로 Visual Sweep 대상이 아니다. 클릭한 실제
쪽에서만 selection overlay를 조회하는 Studio 상호작용 계약을 수정했다.
