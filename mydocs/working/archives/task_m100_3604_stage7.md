---
kind: working
status: completed
issue: 3604
stage: 7
last_verified: 2026-08-01
---

# #3604 Stage 7: 저장 대화상자 암호 설정 통합

## 목표

- 별도 `암호 설정하여 저장...` 파일 메뉴를 제거한다.
- `다른 이름으로 저장`, `HWP 형식으로 저장`, `HWPX 형식으로 저장`이 공통 저장 대화상자를
  먼저 열고, 사용자가 그 안의 `암호 설정...`을 선택하면 새 암호·확인을 입력하게 한다.
- HML에는 암호 설정 control을 제공하지 않는다.

## 구현 계획

1. `SaveAsDialog`의 반환값에 파일명과 암호 설정 요청 여부를 함께 담는다.
2. HWP/HWPX dialog에만 `암호 설정...` command를 두고, 선택하면 저장 흐름이 기존 password
   confirmation dialog를 이어서 연다.
3. native File System Access picker와 download fallback 모두 동일한 파일명·password serializer를
   사용하게 저장 흐름을 통합한다.
4. 별도 menu command와 중복 저장 함수를 제거하고 Studio source contract·production build로 검증한다.

## 보안 경계

- Save As dialog가 암호 자체를 받거나 보관하지 않는다. 암호 설정을 누른 뒤 별도 password dialog가
  한 번만 받고, serializer 호출 뒤 지역 참조를 비운다.
- filename, public result, browser storage, log에는 암호를 넣지 않는다.

## 테스트 결과

| 검증 | 결과 | 근거 |
| --- | --- | --- |
| Studio password/save-format source contract 11건 | 성공 | 다른 이름, HWP, HWPX 공통 저장 대화상자와 별도 menu 제거를 확인 |
| `npx tsc --noEmit` | 성공 | `SaveAsDialogResult`와 공통 저장 흐름 type 검사 통과 |
| `npm run build` | 성공 | Vite production bundle 생성 |
| `git diff --check` | 성공 | whitespace 오류 없음 |

## 구현 결과

- `SaveAsDialog`는 파일명과 `configurePassword` 선택을 반환한다. HWP/HWPX 호출에서만
  `암호 설정...` button을 표시한다.
- `다른 이름으로 저장`, `HWP 형식으로 저장`, `HWPX 형식으로 저장`은 모두 `saveAsFormat()`과
  `promptSaveAsOptions()`를 공유한다. 세 경로 모두 같은 파일명·암호 선택을 native picker와
  download fallback에 전달한다.
- 별도 `file:save-as-password` command와 파일 메뉴 항목을 제거했다.
- 실제 암호는 Save As dialog가 아닌 기존 password confirmation dialog에서만 받고, 저장 호출
  뒤 지역 참조를 비운다.
