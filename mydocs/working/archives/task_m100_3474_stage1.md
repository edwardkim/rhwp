---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-27
---

# Task #3474 Stage 1 — HWP5 암호 문서 UI

Issue: [#3474](https://github.com/edwardkim/rhwp/issues/3474)

## 확인한 현재 경로

- Studio의 파일 선택·드롭·최근 문서·PWA·embed 열기는 결국 `loadBytes()` →
  `WasmBridge.loadDocument()`로 모인다.
- WASM에는 `HwpDocument.openWithPassword(data, password)`가 이미 있다.
- 현재 bridge는 새 `HwpDocument` 생성 전에 기존 `doc`를 해제한다. 따라서 암호 문서 감지·오답에서
  기존 문서가 사라져 #3474의 원자성 요구에 맞지 않는다.

## 구현 범위

1. bridge가 일반·비밀번호 열기 모두 임시 `HwpDocument`를 완성한 뒤에만 기존 문서를 교체하도록 바꾼다.
2. `비밀번호가 필요한 암호 문서` 오류일 때만 Studio 비밀번호 대화상자를 열고, 확인 시
   `openWithPassword` 경로로 재시도한다.
3. 대화상자는 password input·label·Enter·취소·오답 안내를 제공하며, 브라우저 암호 자동완성을 요청하지 않고
   입력값을 로그·최근 문서·저장소·URL·메타데이터에 기록하지 않는다. 취소·오답은 기존 문서를 보존한다.
4. non-password HWP/HWPX/HML의 기존 열기 경로와 미지원 암호화·DRM의 기존 오류는 유지한다.

## HWP3 암호화 보류 기록

- HWP3 `DocInfo` 오프셋 96은 비영 값으로 암호 문서를 식별한다
  (`mydocs/tech/한글문서파일구조3.0.md`). 현 HWP3 파서는 이 값을 메타데이터에는 배선하지만,
  본문 복호화 알고리즘은 구현하지 않았다.
- HWP3 복호화 명세는 후속으로 확보한 뒤 `src/parser/hwp3/`에만 구현하고 `openWithPassword` 공통
  API로 연결한다. 이번 #3474 변경은 HWP5 EncryptVersion 4 UI에 한정하며 HWP3 암호 문서 지원을
  주장하지 않는다.
- `samples/HWP3-password-123456.hwp`는 그 후속 구현의 회귀 fixture로만 보관한다. 현재 단계는 이 파일을
  열거나 비밀번호를 코드·문서·검증 로그에 기록하지 않는다.

## 검증 계획

- bridge 원자성·HWP5 비밀번호 오류 분류 unit test
- Studio password dialog 정상·오답·취소·입력값 비보존 test
- TypeScript·Studio test·HWP5 실제 fixture의 브라우저 확인

## 구현 및 검증 결과

- `WasmBridge`는 일반 열기와 `openWithPassword` 열기 모두 다음 `HwpDocument`의 파싱·편집 준비·파일명
  설정이 성공한 뒤에만 기존 문서를 교체한다. 따라서 암호 필요·오입력·손상·취소에서는 기존 문서,
  최근 문서 및 자동저장 초기화 경로에 도달하지 않는다.
- Studio는 HWP5 `EncryptVersion 4`의 명시적 암호 필요 오류에서만 대화상자를 연다. 지원하지 않는 방식과
  DRM은 기존 거부 오류를 그대로 유지한다.
- 입력은 `type=password`, label·modal ARIA·Enter·확인·취소를 제공하며 닫을 때 DOM 값을 지운다. 암호는
  열기 호출의 지역 변수 외에는 URL·최근 문서·자동저장·문서 메타데이터·local/session storage에 전달하지
  않는다.
- `node --test tests/hwp-password-open.test.ts` — 4 passed
- `npm test` — 674 passed, 0 failed
- `npm run build` — passed
- headless Chrome에서 실제 HWP5 암호 fixture를 파일 선택으로 열어 취소·오입력·Enter 성공·저장소
  비보존을 확인했다. E2E 보고서는 로컬 `output/` 생성물이며 저장소에 포함하지 않는다.
