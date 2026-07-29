# Stage 5 완료보고서 — 문서화 + 최종 회귀 (M100 #2660)

- **이슈**: [edwardkim/rhwp#2660](https://github.com/edwardkim/rhwp/issues/2660)
- **브랜치**: `local/task2660`
- **구현계획서**: [`../plans/task_m100_2660_impl.md`](../plans/task_m100_2660_impl.md)
- **작성일**: 2026-07-21

## 1. 문서화 — `npm/editor/README.md` (호스트 정본 문서)

- `### editor.notifySaved(fileName?)` API 절 추가 — 반환값, fileName 의미,
  capability 게이팅(미광고 시 요청 없이 예외) 명기.
- `## 저장 계약 — 내보내기와 저장 완료 통지` 절 추가:
  - `exportHwp/Hwpx/Hml`은 문서를 "저장됨"으로 표시하지 않음 — 통지 전까지
    자동복구 draft 보존, 통지 없이 종료 시 "문서 복구" 안내가 뜨는 동작 설명.
  - **iframe 계약**: 업로드 **성공 시에만** `notifySaved()` (실패 시 미호출 →
    draft 보존으로 재시도/복구 가능).
  - **팝업 계약**: 핸드오프 즉시 `await editor.notifySaved()` 후 `window.close()`
    — resolve가 draft 삭제 완료를 보장. `postMessage` targetOrigin은 `'*'` 대신
    명시적 오리진 사용 권고(문서 유출 방지).
  - SDK 없는 포크/주입 통합용 `window.rhwpStudio.notifySaved(fileName?)` 안내.

## 2. 최종 회귀 결과

로컬 WASM `pkg/` 구본을 Docker WASM 빌드(`docker compose run --rm wasm`,
7m 56s)로 재생성한 뒤 수행했다.

| 검증 | 결과 | 로그 |
|---|---|---|
| `npm test` (단위 459 + SDK 21 포함) | **458/459 통과** — 잔여 1건은 `cell-flow-boundary`(Windows에서 `spawnSync('node_modules/.bin/tsc')` 실행 불가, HEAD 동일 재현 확인된 기존 환경 이슈; CI Linux 통과) | `$TMPDIR/task2660_full_test.log` |
| `npx tsc --noEmit` | **오류 0건** (WASM pkg 재생성으로 기존 3건도 해소) | `$TMPDIR/task2660_s5_tsc.log` |
| `npm run build` (tsc && vite build) | **성공** (PWA precache 포함) | `$TMPDIR/task2660_s5_build.log` |
| `e2e/embed-save-ack.test.mjs` | **17/17 PASS** (새 WASM 반영 재실행) | `$TMPDIR/task2660_s5_e2e1.log` |
| `npm run e2e:embed` | 11/12 — 1 FAIL은 Stage 4에서 베이스 src 치환으로 기존 환경 이슈 실증 | `$TMPDIR/task2660_s5_e2e2.log` |
| `autosave-recovery.test.mjs` | 18 PASS / 4 FAIL — 동일하게 베이스와 실패 집합 일치 실증 (새 WASM에서도 동일 → 환경 요인 재확인) | `$TMPDIR/task2660_s5_e2e3.log` |

## 3. 비고

- 로컬 검증 환경: Windows 11 + Node v24.16.0 + headless Chrome. e2e 인프라의
  Windows 휴대성 2건은 Stage 4에서 수정 완료.
- 기존 환경 실패(단위 1건, e2e 5건)는 모두 베이스 커밋에서 동일 재현되어 본
  타스크와 무관 — merge 전 CI(Linux)에서 최종 확인 권고.

## 4. 다음 단계

최종 결과보고서(`report/task_m100_2660_report.md`) 승인 → 이후 절차(local/devel
merge, 이슈 클로즈)는 작업지시자 승인에 따름.
