---
kind: snapshot
status: active
canonical: mydocs/plans/task_m100_6731.md
issue: 6731
last_verified: 2026-09-05
---

# #6731 Stage 2 — RED 보호 계약 결과

## 추가한 계약

제품 구현을 바꾸기 전에 다음 경계를 테스트로 고정했다.

1. password open helper와 `WasmBridge.loadDocumentWithPassword()`는 metadata를 반환하지 않는
   `void` command다.
2. open 성공 뒤 `getDocumentInfo()` query가 metadata를 별도로 읽는다.
3. `DocumentInfo`는 승인된 8개 필드만 선언하고 password·credential·secret 필드를 갖지 않는다.
4. 폰트 해소 cache key는 `langId`, `fontName`, `altType`으로만 구성하며 credential을 받지 않는다.
5. Node runtime snapshot은 Studio 암호 open 모듈을 직접 import하지 않고 canonical font rows만
   SHA-256 입력으로 사용한다.
6. HWP3·HWP5·HWPX를 실제로 연 뒤 `DocumentInfo`, localStorage와 sessionStorage 어디에도 입력 암호가
   나타나지 않는다.

## RED 결과

```text
node --test tests/hwp-password-open.test.ts
tests 7 / pass 6 / fail 1
```

실패는 `#6731 암호 open command와 DocumentInfo query는 반환값 경계로 분리된다` 한 건뿐이다.
현재 구현이 `loadPasswordProtectedDocument(): Promise<DocumentInfo>`에서
`return wasm.loadDocumentWithPassword(...)`를 사용하기 때문에 예상대로 실패했다.

나머지 기존 보안 계약과 새 `DocumentInfo`·font cache·snapshot digest 계약은 모두 통과했다. 따라서
이번 변경의 목표는 이미 안전한 필드·저장 경로를 다시 만드는 일이 아니라 CodeQL이 password source로
삼은 반환값 연결을 끊는 일로 좁혀졌다.

## 브라우저 기준선

7700 포트의 현재 Studio와 headless Chrome으로 다음 명령을 실행했다.

```text
npm run e2e:hwp-password-open
```

HWP5 EncryptVersion 4, HWP3 압축 암호 문서, HWPX ODF AES-256-CBC 세 형식이 모두 통과했다. 각 형식에서
취소·오입력·정상 열기·저장 보호 lifecycle, storage 무암호와 `DocumentInfo` 무암호·허용 필드 계약을
확인했다. HWP3 144dpi Canvas 경계도 유지됐다. 생성된 HTML 보고서는 `output/`의 로컬 증적이며 source
commit에는 포함하지 않는다.

## Stage 3 진입 조건

- `openPasswordProtectedDocument(): Promise<void>`로 command 의미를 드러낸다.
- `WasmBridge.loadDocumentWithPassword(): void`로 반환 경계를 닫는다.
- `loadDocumentForOpen()`만 open 성공 뒤 `wasm.getDocumentInfo()`를 호출한다.
- E2E 직접 호출부도 command 실행 뒤 query하는 순서로 갱신한다.
- CodeQL workflow, query, path filter, SHA-256 구현과 폰트 cache는 바꾸지 않는다.

Stage 3 구현 뒤 단위 계약 7건이 모두 통과해야 하며, 실패를 피하기 위한 별도 sanitizer는 추가하지
않는다.
