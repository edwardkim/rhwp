---
kind: snapshot
status: active
canonical: mydocs/plans/task_m100_6731.md
issue: 6731
last_verified: 2026-09-05
---

# #6731 Stage 3 — password open command/query 분리 결과

## 구현 결과

CodeQL이 password source로 해석한 반환값 연결만 최소 범위로 끊었다.

1. `openPasswordProtectedDocument()`를 `Promise<void>` command로 만들고 암호 입력·재시도·취소의
   기존 lifecycle을 유지했다.
2. `WasmBridge.loadDocumentWithPassword()`를 `void` command로 바꿨다. 내부 원자적 문서 교체와
   보호 저장 의도 commit은 그대로 유지한다.
3. `loadDocumentForOpen()`은 command 성공 뒤 `wasm.getDocumentInfo()` query를 별도로 호출한다.
4. #4430 E2E의 직접 facade 호출도 command 실행 뒤 query하는 순서로 갱신했다.
5. CodeQL workflow·query·path, font cache와 snapshot SHA-256 구현은 바꾸지 않았다.

## GREEN 검증

```text
node --test tests/hwp-password-open.test.ts
tests 7 / pass 7 / fail 0

npm test
tests 1376 / pass 1375 / fail 0 / skipped 1

npm run build
TypeScript compile + Vite production build 성공

npm run e2e:hwp-password-open
HWP5 EncryptVersion 4 / HWP3 압축 암호 / HWPX ODF AES-256-CBC 통과
```

브라우저 E2E는 세 형식의 취소·오입력·정상 열기·저장 보호 lifecycle을 유지했고, 입력 암호가
`DocumentInfo`, localStorage와 sessionStorage에 나타나지 않으며 metadata가 승인된 필드만 갖는다는
계약도 통과했다.

## #4430 E2E 기준선 결함 분리

`npm run e2e:issue-4430-content-loss`는 이번에 바꾼 protected reopen 직접 호출과 이후 다수 저장
시나리오를 통과했지만, 후반의 `failed unprotected Save As preserves prior password-protected state`
시나리오에서 30초 timeout으로 종료됐다.

이는 이번 구현의 반환형 변경이 아니라 기준선 test와 제품 계약의 불일치다.

- 통합 커밋 `2ca9aa90f7`(2026-08-30)은 보호 문서에서 Save As 기본 `확인`을 누르면 암호 재입력
  대화상자로 이어지도록 `inheritPassword` 계약을 추가했다.
- 같은 커밋은 `content-loss-save-issue4430.test.mjs`를 갱신하지 않았다.
- 해당 시나리오는 이름 그대로 평문 사본을 요구하면서 현재도 `확인`을 누른 뒤 곧바로
  `picker:error`를 기다린다. 현재 UI 계약에서는 `암호 없이 저장`을 눌러야 한다.
- 실패 스크린샷은 파일 선택기가 아니라 정상적인 `문서 암호 설정` 대화상자에서 대기 중임을
  확인한다.

따라서 #6731 제품 변경에는 이 독립 기준선 결함의 동작 수정까지 섞지 않았다. Stage 4에서 최소 test
정정을 현재 PR에 포함할지 별도 후속으로 분리할지 판정한 뒤, 영향받는 #4430 E2E 전체 성공을
확정해야 한다.

## Stage 4 진입 조건

- 최신 `upstream/devel`과 열린 PR의 동일 파일 변경을 다시 확인한다.
- #4430 E2E 기준선 정정의 범위를 확정하고 전체 브라우저 회귀를 통과시킨다.
- `git diff --check`와 변경 범위 로컬 검증을 다시 수행한다.
- remote push 승인 뒤 PR exact-head CodeQL에서 alert #186 flow 소멸 여부를 확인한다.
- flow가 남으면 sanitizer나 workflow 제외를 임의 추가하지 않고 새 SARIF delta와 수정 계획을
  메인테이너에게 보고한다.
