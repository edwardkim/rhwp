# Task M100-3438 Stage 1 — 스타일 저장의 실제 반환 계약 반영

- Issue: #3438
- 브랜치: `local/task_m100_3438`
- 기준: `upstream/devel` (`8e81cbd996b66f873d21b74085a9dcee78ae3901`)
- 완료일: 2026-08-13

## 범위

#3438의 두 항목 중 한컴 2022 실측 없이도 독립적으로 판정 가능한 스타일 저장 반환 계약만
처리했다. 모달이 열린 상태에서의 Ctrl+Z 동작과 `history-jumped` 구독 제거 여부는 변경하지
않았다.

## 변경

- `StyleEditDialog`의 `createStyle()` 음수 ID 실패 가정을 제거했다. Rust 구현은 새 스타일을
  추가한 뒤 그 ID를 반환하며 음수 실패 경로가 없다.
- 기존 스타일 수정에서 `updateStyle()`이 `false`이면 snapshot operation이 `null`을 반환하게
  했다. 결과적으로 undo 기록, 커서 이동, 리프레시가 모두 생략된다.
- `updateStyleShapes()`의 실재하는 `false` 반환값을 검사하고, 생성·수정 뒤 모양 적용 실패는
  예외로 드러낸다.
- Studio 소스 가드는 다이얼로그 배선뿐 아니라 `src/wasm_api.rs`의 세 API 반환 계약도 직접
  대조한다.

## 검증

| 명령 | 결과 |
| --- | --- |
| `node --test tests/style-undo-routing.test.ts tests/undo-noop-skip.test.ts` | 12 passed |
| `npx.cmd tsc --noEmit` | 통과 |
| `npm.cmd test` | 870 passed, 1 skipped, 0 failed |
| `npm.cmd run build` | 통과 |

초기 TypeScript 검사에서 설치본 `node_modules`에 `@noble/hashes`가 빠진 것을 확인해
`npm.cmd ci`로 잠금 파일 기준 의존성만 복구했다. 추적 파일 변경은 없다.

세션의 브라우저 연결을 사용할 수 없어 실제 Studio UI 스모크는 수행하지 못했다. 이를
자동화 테스트 통과로 대체해 기록하지 않는다.

## 한컴 환경 정정

Windows 설치 경로에서 한컴 Office 2018·2022·2024의 `Hwp.exe`를 확인했다. 앞선 레지스트리
조회가 Assistant 항목만 찾아 한컴 Office가 없다고 판단한 것은 잘못이었다. 한컴 2022 실측이
선결인 #3438 모달 판정, #3351, #3416은 후속 Windows 검증 대상으로 유지한다.
