# M100 #2513 최종 보고서 - embed loadFile 기본 대화상자 교착

## 1. 결과

`@rhwp/editor`의 `loadFile(data, fileName)` 기본 호출이 iframe 안내창 응답을 기다리다 timeout될
수 있는 원인을 public SDK 경계에서 해결했다.

- 옵션 생략: `suppressDialogs: true`
- 명시 `false`: 기존 대화형 안내창 유지
- 명시 `true`: 비대화형 로드 유지
- raw protocol과 top-level Studio 기본값: 변경 없음

## 2. 실제 사용자 흐름

embed 사용자는 iframe 내부 버튼을 찾아 누르는 대신 `loadFile()` Promise 완료를 문서 준비 신호로
사용한다. 변경 후 zero-option 호출은 HWPX 검증 경고를 그대로 열기로 처리하고 로컬 글꼴은 웹 대체
글꼴로 표시해, 숨겨진 iframe UI 응답 없이 완료된다. 대화형 선택이 필요한 caller만 explicit false를
사용한다.

## 3. 구현

- `npm/editor/index.js`: 옵션 생략과 explicit 값을 구분하는 기본값 계산
- `npm/editor/tests/load-file-options.contract.test.mjs`: 생략/false/true 전송 계약
- `rhwp-studio/e2e/embed-transport.test.mjs`: 수동 안내창 click 우회 제거, zero-option 실제 로드
- `npm/editor/README.md`, `npm/editor/index.d.ts`: 공개 기본값과 opt-out 문서화

커밋:

- `061d429b` Task #2513: embed loadFile 기본 대화상자 교착 방지
- `3c8b4fb8` Task #2513: embed loadFile 회귀 검증과 문서 동기화

## 4. 대안과 결정

- 선택: public SDK의 생략 기본값만 true로 변경. 최소 surface이며 explicit false로 호환 가능하다.
- 기각: `createEditor()` 전역 옵션 추가. 인스턴스 상태와 API surface가 불필요하게 늘어난다.
- 기각: opt-in 유지와 문서 강조만 수행. 실제 consumer timeout의 원인을 제거하지 못한다.
- 기각: raw protocol/Studio 전체 기본값 변경. SDK 외 consumer의 기존 계약까지 바꾼다.

## 5. 검증

- 계약 RED: 생략 경로가 기대 true 대신 false를 전송해 실패
- 계약 GREEN: 1/1 통과
- embed E2E: 12개 단언 통과, 7페이지 load, HWP/HWPX export 성공
- npm editor: 19/19
- Studio: 457/457, production build 성공
- Cargo release-test: 3,364 passed, 23 ignored
- Cargo fmt, Clippy `-D warnings`, diff check: 통과

초기 E2E의 `_waitReady()` hang은 worktree의 gitignored `pkg/` WASM 산출물 부재로 분리했다.
WASM을 생성하고 Vite를 재시작한 뒤 동일 시나리오가 통과했으므로 제품 회귀로 오인하지 않았다.

## 6. 호환성과 운영

- MessageChannel v1, exact origin/session, transferable buffer 계약 유지
- caller ArrayBuffer 비변형 유지
- legacy request/response 경로 유지
- migration, data, permission, secret 변경 없음
- 렌더 출력 코드 변경 없음

## 7. 외부 작업 상태

- upstream issue: [#2513](https://github.com/edwardkim/rhwp/issues/2513)
- 작업 branch: `fix/issue-2513`
- PR base: `edwardkim/rhwp:devel`
- fork push: `cskwork/rhwp:fix/issue-2513` 완료
- upstream PR: [#2518](https://github.com/edwardkim/rhwp/pull/2518), base `devel`, OPEN
- reviewer 요청: 공식 `RequestReviews`는 contributor 권한 부족으로 거부됨
- 대체 요청: [PR 댓글](https://github.com/edwardkim/rhwp/pull/2518#issuecomment-5018473005)에서 `@edwardkim` 태그 완료
- GitHub Actions: preflight 성공, 나머지 check 진행 중
- issue close/merge: 수행하지 않음
