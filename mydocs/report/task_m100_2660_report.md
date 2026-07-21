# 최종 결과보고서 — 호스트 저장 완료 통지 API notifySaved (M100 #2660)

- **이슈**: [edwardkim/rhwp#2660](https://github.com/edwardkim/rhwp/issues/2660)
- **브랜치**: `local/task2660` (`upstream/devel` a803f079 기준)
- **수행계획서**: [`../plans/task_m100_2660.md`](../plans/task_m100_2660.md)
- **구현계획서**: [`../plans/task_m100_2660_impl.md`](../plans/task_m100_2660_impl.md)
- **작성일**: 2026-07-21
- **관련**: #1448 (PR #1450, 자동 백업/복구), 후속 이슈 #2661 (hwpctl SaveAs)

## 1. 문제와 해법 요약

외부 호스트가 rhwp-studio에서 내보내기(직렬화 바이트 취득)로 문서를 저장하는
통합(팝업 opener postMessage / iframe embed RPC)에서는 **저장 완료를 스튜디오에
알릴 수단이 없어**, 사용자가 저장했다고 느꼈음에도 dirty 상태와 자동복구
draft(IndexedDB, #1448)가 남아 다음 실행 시 "문서 복구" 다이얼로그가 떴다.

**해법**: 코어 함수 하나(`completeHostSave`)를 두 표면으로 노출하는 저장 완료
통지 API를 추가했다.

- 코어: `markClean('host-save')` → 기존 정리 사슬 재사용 + **draft 삭제 완료를
  await** (팝업 `window.close()`에 IndexedDB 삭제가 잘리지 않는 계약).
- 표면 A: `window.rhwpStudio.notifySaved(fileName?)` — 팝업/포크 통합용, 프로덕션
  상시 노출.
- 표면 B: embed RPC `notifySaved` + `'notify-saved-v1'` capability +
  `@rhwp/editor` SDK `editor.notifySaved(fileName?)` (capability 게이팅).
- `exportHwp` 등 내보내기 자체는 무수정 — 업로드 실패 시 백업 보존을 위해
  자동 markClean을 하지 않는 설계를 유지.
- 부수 강화: autosave flush 진행 중 discard 시 draft가 부활하는 경합을
  제너레이션 카운터로 방어.

## 2. 변경 파일

| 영역 | 파일 | 내용 |
|---|---|---|
| 코어 | `rhwp-studio/src/main.ts` | `completeHostSave` + `window.rhwpStudio` + embed 핸들러 |
| 코어 | `rhwp-studio/src/recovery/autosave-manager.ts` | `discardGeneration` 경합 방어 |
| RPC | `rhwp-studio/src/embed/protocol.ts` | `'notify-saved-v1'` capability (버전 1 유지) |
| RPC | `rhwp-studio/src/embed/rpc-router.ts` | `notifySaved` 인터페이스·라우팅 (fileName 정규화) |
| SDK | `npm/editor/transport.js`, `index.js`, `index.d.ts` | 클라이언트 capability + `notifySaved` + 선언 |
| 문서 | `npm/editor/README.md` | API 절 + **저장 계약** 절 (iframe/팝업/targetOrigin) |
| 테스트 | `rhwp-studio/tests/{autosave-manager,embed-protocol}.test.ts` | 경합·계약·라우팅 (TDD red→green) |
| 테스트 | `npm/editor/tests/{notify-saved-v1.contract,transport}.test.mjs` | SDK 계약 |
| E2E | `rhwp-studio/e2e/embed-save-ack.test.mjs` (신규) | TC-1~4, 17 assertion |
| E2E 인프라 | `rhwp-studio/e2e/{helpers,embed-transport.test}.mjs` | Windows 휴대성 (보고서 경로, /@fs URL) |

Rust/WASM, `document-dirty-state.ts`, `autosave-store.ts`, `embed/runtime.ts`,
`hwpctl/index.ts` 무수정.

## 3. 커밋 이력 (local/task2660)

| 커밋 | 내용 |
|---|---|
| 339d88a3 | 수행계획서 + 오늘할일 |
| 29c61139 | 구현계획서 (5단계) |
| 06212e7c | Stage 1 — 코어 + window API + 경합 방어 |
| ef57d635 | Stage 2 — embed RPC + capability |
| e91a7c7a | Stage 3 — SDK notifySaved |
| c058305e | Stage 4 — E2E 4케이스 + e2e Windows 휴대성 |
| (본 커밋) | Stage 5 — README 저장 계약 + 최종 회귀 + 최종보고서 |

## 4. 검증 총괄

전 단계 TDD(red→green): 신규 단위·계약 테스트는 모두 실패를 먼저 확인한 뒤
구현했다.

| 검증 | 결과 |
|---|---|
| 단위+SDK `npm test` | **458/459** (잔여 1건 `cell-flow-boundary`는 Windows spawnSync 기존 환경 이슈 — HEAD 동일 재현) |
| `npx tsc --noEmit` | **0 오류** (Docker WASM pkg 재생성 후) |
| `npm run build` | **성공** |
| 신규 E2E (embed-save-ack) | **17/17 PASS** — 통지→다이얼로그 없음 / 미통지→표시(원 증상 재현) / capability / window API |
| 회귀 `e2e:embed` | 11/12 — 1 FAIL은 **베이스 src 치환 재실행으로 기존 환경 이슈 실증** |
| 회귀 `autosave-recovery` | 18 PASS / 4 FAIL — 동일 방법으로 베이스와 실패 집합 완전 일치 실증 |

기존 환경 실패(단위 1, e2e 5)는 모두 베이스 커밋에서 동일 재현 — 본 타스크와
무관하며, merge 전 CI(Linux)에서 최종 확인을 권고한다.

## 5. 호스트 적용 안내 (통합 앱 후속 1줄)

팝업 통합 호스트의 내보내기 핸들러에 통지 한 줄을 추가하면 된다:

```js
const bytes = await editor.exportHwp();
const base64 = uint8ArrayToBase64(bytes);
window.opener?.postMessage(
  { action: 'documentExported', content: base64, format: 'hwp' },
  HOST_ORIGIN, // 보안 권고: '*' 대신 명시적 오리진
);
await editor.notifySaved(); // ← 추가: dirty 해제 + 복구 draft 삭제 완료 대기
window.close();
```

## 6. 범위 외 기록 (후속 후보)

- #2661: hwpctl `SaveAs()` dirty 미정리 (등록 완료, 우선순위 낮음).
- `beginDocument({discardPreviousDraft})` 경로의 draft 부활 경합 —
  `discardCurrentDraft` 세대 카운터의 보호 범위 밖 (발생 조건 좁음, Stage 1
  보고서에 기록).
- e2e `cell-flow-boundary` 단위 테스트와 일부 e2e의 Windows 네이티브 실행 한계
  (spawnSync `.bin/tsc`, 렌더러 선택 진단, 복구 다이얼로그 타이밍) — CI Linux
  기준으로는 영향 없음.

## 7. 완료 조건 대비 (#2660 수용 기준)

- [x] 팝업 흐름: 통지 후 재실행 시 복구 다이얼로그 없음 (E2E TC-1/TC-4)
- [x] iframe 흐름: 통지 시 dirty 해제+draft 삭제, 미호출 시 draft 보존 (E2E TC-1/TC-2)
- [x] `'notify-saved-v1'` capability 광고 (E2E TC-3, 단위)
- [x] flush↔discard 경합 시 draft 미부활 (단위, red→green)
- [x] 기존 내부 저장/복구 회귀 없음 (단위 458/459 + E2E 회귀, 잔여 실패는 베이스 동일)
- [x] README 저장 계약 문서화

## 8. 통합 절차 (정정: 컨트리뷰터 PR 워크플로우)

당초 메인테이너용 local/devel merge로 기재했으나, 본 저장소에서 우리는
fork(johndoekim) 기반 컨트리뷰터이므로 CONTRIBUTING.md의 Fork & PR 워크플로우를
따른다 (작업지시자 지적 반영).

- PR 전 체크리스트 수행: `cargo clippy -- -D warnings` 통과,
  `cargo test --profile release-test --tests --no-fail-fast` **3,447 passed / 0 failed**
  (`$TMPDIR/task2660_cargo_full_test.log`). `cargo fmt --check`는 로컬
  `core.autocrlf=true`(CRLF) 체크아웃으로 전 파일 판정 불가 — 본 PR은 .rs 0건
  변경이므로 CI 판정에 위임.
- fork 브랜치 `feature/2660-notify-saved` push 후 PR 생성:
  **[edwardkim/rhwp#2667](https://github.com/edwardkim/rhwp/pull/2667)** (base: devel,
  `Closes #2660` 포함 — 메인테이너 merge 시 이슈 자동 클로즈).
- 이후 절차: CI(빌드+테스트+Clippy) 통과 확인 → 메인테이너 리뷰 대응.
