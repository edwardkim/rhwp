# M100 #2513 Stage 2 완료 보고서 - iframe 회귀와 공개 문서

## 1. 결론

fresh iframe의 zero-option `loadFile()`이 안내창 버튼 polling/click 없이 완료됨을 실제 E2E로
증명했다. 7페이지 HWP 로드 뒤 HWP/HWPX export와 MessageChannel v1·legacy 호환 단언까지
모두 통과했다.

공개 README와 타입 선언도 SDK의 생략 기본값 `suppressDialogs: true`와 대화형 opt-out
`suppressDialogs: false`를 정확히 설명하도록 동기화했다.

## 2. 변경 파일

- `rhwp-studio/e2e/embed-transport.test.mjs`
  - `대체 글꼴로 보기` 버튼을 최대 60초 polling/click하던 11줄 우회 제거
  - zero-option `await editor.loadFile(...)` 직접 완료 검증
  - 기존 auto renderer 진단 단언에 맞게 `renderer: 'auto'` 요청 명시
- `npm/editor/README.md`
  - `suppressDialogs` 기본값을 `true`로 수정
  - 대화형 흐름은 `{ suppressDialogs: false }`로 명시하도록 예제와 설명 수정
- `npm/editor/index.d.ts`
  - 기본값 `true`와 explicit `false` 계약 문서화

## 3. 기준 재현과 환경 분리

초기 기준 E2E는 60초 이상 멈췄다. 브라우저와 HTTP 상태를 분리한 결과, 당시 교착 위치는
`loadFile`이 아니라 `createEditor()`의 `_waitReady()`였다.

- `/`와 `/@vite/client`: HTTP 200
- `/src/core/wasm-bridge.ts`: HTTP 500
- 원인: worktree에 추적되지 않는 `pkg/` WASM 산출물이 없어 `@wasm/rhwp.js` import 실패
- 결과: Studio runtime이 설치되지 않아 `ready`와 `loadFile` 요청 단계에 도달하지 못함

이는 제품 결함과 별개인 E2E 사전조건 실패이므로 소스 우회를 추가하지 않았다. 로컬
`wasm-pack 0.15.0`과 설치된 `wasm32-unknown-unknown` target으로 `pkg/`를 생성하고 Vite를
재시작한 뒤 동일 E2E를 재실행했다.

## 4. GREEN 증거

```bash
rtk env CARGO_TARGET_DIR=/private/tmp/rhwp-issue-2513-target \
  wasm-pack build --target web --out-dir pkg
```

- exit 0, release WASM `pkg/` 생성

```bash
rtk env VITE_URL=http://127.0.0.1:7700 npm run e2e:embed
```

- exit 0, 12개 단언 전체 통과
- `pageCount=7`
- HWP export 15,360 bytes
- HWPX export 13,059 bytes, ZIP magic 유지
- caller ArrayBuffer 보존
- MessageChannel v1, forged peer 차단, destroy, legacy 경로 보존

```bash
rtk npm --prefix npm/editor test
```

- exit 0, 19/19 통과

```bash
rtk git diff --check
```

- exit 0, 출력 없음

## 5. 범위 보호

- `rhwp-studio/src/embed/rpc-router.ts`와 `rhwp-studio/src/main.ts` 변경 없음
- top-level Studio와 raw protocol의 기본 `suppressDialogs: false` 유지
- MessageChannel version, origin/session 검증, transport 변경 없음
- `pkg/`, `.env.docker`, Cargo target은 git 추적 대상이 아님

## 6. 독립 검토

- 판정: PASS
- 변경 범위 3파일 확인
- 수동 안내창 우회 완전 제거 확인
- zero-option load와 explicit false 문서 계약 확인
- E2E 12개 단언, npm editor 19개 테스트, diff check 재통과

초기 sandbox Chromium Mach port 권한 오류는 승인된 실행 환경에서 같은 명령을 재실행해
통과했으므로 코드 실패로 분류하지 않았다.

## 7. 다음 단계와 승인

- Stage 2 상태: 완료, 독립 검토 PASS
- Stage 3 상태: 2026-07-20 작업지시자 사전 승인 완료
- 다음 작업: 필수 Cargo gate, 최종 보고서, 커밋·fork push·upstream PR·리뷰 요청
- issue close: 승인받지 않았으므로 수행하지 않음
