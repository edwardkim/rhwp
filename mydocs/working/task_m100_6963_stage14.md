# #6963 단계 14 — 한컴 실측 기반 링크 삭제 확인

> 후속 사용자 정정 반영: 아래 최초 검증에서 제외했던 Backspace도 이제 같은 전체 삭제
> 확인 경로를 사용한다. 최신 조건·검증은 문서 끝의 'Backspace 추가 보정'을 따른다.

- 기준: 로컬 `2a4ddcbcc`.
- 사용자 제공 한컴 화면: 제목 '지우기', '[하이퍼링크]를 지울까요?', '지움'/'취소'.
- 사용자 요청: Delete로 하이퍼링크를 지울 때 확인 후 표시 문자열 전체와 필드를 함께 삭제.
- 구현: 기존 Studio 모달과 snapshot operation을 사용해 필드 제거와 텍스트 삭제를
  하나의 이력으로 묶는다. 취소는 mutation·history·dirty를 만들지 않는다.
- 확인창을 연 뒤 문서 세대/대상 내용/편집 가능 상태가 바뀌면 오래된 대상을 삭제하지 않는다.
- Delete 대상 위치의 세부 한컴 조건은 사용자에게 확인 중이다. Backspace·선택 범위 삭제의
  한컴 동작은 아직 확인되지 않았으므로 임의로 같은 동작이라고 보고하지 않는다.
- 검증: 실제 WASM/Studio confirm callback, 취소·삭제·undo/redo·저장 왕복·외부 변경 차단,
  본문·중첩 셀·글상자. TypeScript/build·Node 회귀와 실제 브라우저 확인.
- 기존 앞뒤 경계 정책과 undo 보정은 유지한다. 원격 push·comment는 승인 대기다.

## 구현·검증 결과 (2026-09-13)

- 현재 적용 조건: 선택 영역 없이 Delete가 지울 글자가 링크 범위 `[start, end)`에 있는
  경우 확인창을 연다. 링크 맨 앞과 내부에 적용하며, 맨 뒤의 일반 텍스트 Delete,
  Backspace, 선택 범위 삭제는 기존 경로다. 이는 사용자 요청을 바탕으로 정한 현재 계약이며
  한컴의 모든 커서·선택 조건을 직접 검증했다는 의미는 아니다.
- `input-handler-hyperlink-delete.ts`가 대상 문단·링크와 문서 세대를 보관하고, 확인 시
  다시 검증한다. `deleteHyperlink` snapshot 안에서 필드와 표시 문자열을 순서대로
  제거한다. 둘째 단계 실패 시 전체를 복구하고, 성공 시 Undo 한 번으로 복구한다.
- mutation-routing 목록의 새 3개 호출은 위 snapshot 내부의 필드 제거·본문/셀 삭제다.
  호출을 우회 허용한 것이 아니라 실제 WASM으로 원자 복구·이력을 검증한 경로를 기록했다.
- 실제 WASM runner: 18개 검증 묶음 통과. 취소·중복 확인창·전체 삭제·3회 undo/redo,
  HWP/HWPX 저장 왕복, 중첩 셀·글상자, 삭제 중 실패 복구, 모달 중 외부 편집·문서 교체·
  읽기 전용 전환 차단을 포함한다. 로그: `/private/tmp/pr6984-delete-wasm.log`.
- `npm test`: 1,502 pass / 2 skip / 0 fail.
  로그: `/private/tmp/pr6984-delete-node.log`.
- `npm run build`: TypeScript·Vite·PWA 빌드 통과.
  로그: `/private/tmp/pr6984-delete-build.log`.
- 실제 브라우저 `http://127.0.0.1:7794/`의 새 문서에서 `링크` / `https://example.com`
  삽입 → Home → Delete → 확인창 → 취소 시 보존 → 다시 Delete·지움 시 전체 제거 →
  ⌘Z로 파란색·밑줄 복구를 직접 확인했다. 하이퍼링크 고치기를 열어 표시 문자열과
  주소도 원본과 같음을 확인했다.
- UI 증적: `/private/tmp/pr6984-delete-confirm.jpg`,
  `/private/tmp/pr6984-delete-removed.jpg`, `/private/tmp/pr6984-delete-undo.jpg`.
- 이 단계는 Studio TypeScript 변경이며 단계 13에서 빌드한 WASM을 재사용했다.
  앞 단계 Rust 변경에 대한 push 전 전체 lint/release 게이트는 별도 승인 후 진행 단계에
  남아 있다. 원격 PR 반영·코멘트 게시·병합은 수행하지 않았다.

## Backspace 추가 보정 (2026-09-13)

- 기준 커밋 `ea012ec25` 이후 사용자 요청으로 Backspace를 같은 확인 경로에 연결했다.
  Delete는 커서 위치, Backspace는 커서 바로 앞 글자가 링크인지 판정한다.
  따라서 Backspace는 링크 끝·내부에서 확인창을 열고, 문단 시작이나 삭제 대상 글자가
  링크 밖이면 기존 삭제 경로를 따른다. 선택 범위 삭제는 이번 변경에 포함하지 않는다.
- 필드 제거·전체 문자열 삭제는 기존 snapshot을 공유하므로 취소는 무변경이며,
  확인 후 Undo 한 번으로 주소·문자열·서식을 복구한다.
- 실제 WASM runner 19개 묶음 통과: Backspace 끝·내부, 바깥 경계 제외, 취소,
  undo/redo·HWP/HWPX 왕복 및 중첩 셀·글상자 양방향 삭제를 추가 검증했다.
  `npm test` 1,502 pass / 2 skip / 0 fail, `npm run build` 통과.
  로그는 `/private/tmp/pr6984-backspace-{wasm,node,build}.log`.
- 최신 코드를 새로 불러온 실제 브라우저에서 링크 끝 Backspace → 취소 시 보존 →
  다시 Backspace·지움 시 전체 삭제 → ⌘Z 복구를 확인했다. 링크 내부 Backspace에서도
  같은 확인창을 확인했다. 증적은 `/private/tmp/pr6984-backspace-confirm.jpg`,
  `/private/tmp/pr6984-backspace-removed.jpg`, `/private/tmp/pr6984-backspace-undo.jpg`.
- 사용자 확인 서버는 `http://127.0.0.1:7794/`이며 새로고침 후 검증한다.
  원격 push·코멘트 게시 승인 대기는 유지한다.
