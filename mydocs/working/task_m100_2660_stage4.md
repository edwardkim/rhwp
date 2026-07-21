# Stage 4 완료보고서 — E2E 검증 (M100 #2660)

- **이슈**: [edwardkim/rhwp#2660](https://github.com/edwardkim/rhwp/issues/2660)
- **브랜치**: `local/task2660`
- **구현계획서**: [`../plans/task_m100_2660_impl.md`](../plans/task_m100_2660_impl.md)
- **작성일**: 2026-07-21

## 1. 신규 E2E — `e2e/embed-save-ack.test.mjs` (17 assertion 전건 PASS)

실행: Vite dev(7700) + headless Chrome(Windows 로컬), Node v24.16.0.
로그 `$TMPDIR/task2660_s4_e2e.log`, 보고서 `output/e2e/embed-save-ack-report.html`.

| TC | 내용 | 결과 |
|---|---|---|
| TC-3 | `rhwp-connected` capabilities에 `notify-saved-v1` 광고 | PASS |
| TC-1 | SDK loadFile→dirty+flush→draft 존재→`exportHwp`(15,360B)→**export만으로 dirty·draft 유지**→`notifySaved('footnote-01-저장본.hwp')`→`{ok, wasDirty:true}`+dirty 해제+fileName 반영+draft 0건→스튜디오 재생성 시 복구 다이얼로그 **없음** | PASS (10 assertion) |
| TC-2 | 동일 절차에서 notifySaved 생략(팝업 강제 종료 동일 조건)→재생성 시 "문서 복구" 다이얼로그 **표시** | PASS (2 assertion) |
| TC-4 | 직접 페이지에서 `window.rhwpStudio.notifySaved()`→`{ok, wasDirty:true}`+dirty 해제+draft 0건→리로드 시 다이얼로그 없음 | PASS (5 assertion) |

핵심 검증: **positive(통지→다이얼로그 없음)와 negative(미통지→다이얼로그 표시)가
쌍으로 성립** — #2660이 보고한 증상과 수정 효과를 모두 재현·확인.

## 2. 기존 E2E 회귀

| 테스트 | 태스크 브랜치 | 베이스(a803f079) src 치환 재실행 | 판정 |
|---|---|---|---|
| `npm run e2e:embed` (embed-transport) | 11 PASS / 1 FAIL | **동일 1 FAIL** ("additive selection diagnostics가 실제 auto 요청을 보존한다") | 기존 로컬 환경 이슈 — 렌더러 선택 영역, 본 diff 미접촉 |
| `autosave-recovery.test.mjs` | 18 PASS / 4 FAIL | **동일 4 FAIL** (복구본 dirty 유지 2건, draft 삭제 2건) | 기존 로컬 환경 이슈 — 베이스와 결과 완전 동일 |

두 회귀 모두 작업트리 src를 베이스 커밋으로 치환해 재실행하는 방법으로
**본 타스크 커밋과의 인과를 분리**했다 (실패 집합이 베이스와 완전히 일치).
CI(Linux)에서는 통과하는 테스트들이며, 최종 판정은 Stage 5 이후 CI에서 재확인한다.

## 3. 환경 정비 (부수 수정·기록)

- **`e2e/helpers.mjs` 1줄 수정**: `getReportFilename()`이 '/'로만 split하여 Windows
  `process.argv[1]`(역슬래시 경로)에서 보고서 생성이 ENOENT로 죽는 버그 →
  `split(/[\\/]/)`로 휴대성 수정 (Linux 동작 불변).
- **`e2e/embed-transport.test.mjs` `/@fs` URL 정규화**: Windows 절대 경로(`D:\...`)가
  `/@fsD:/...`로 붙어 모듈 로드가 실패 → 슬래시 정규화 (신규 테스트에도 동일 적용).
  이 수정이 없으면 Windows 네이티브에서 embed E2E 자체가 실행 불가.
- 로컬 `node_modules` 불완전(`pixelmatch`, `@noble/hashes` 등 누락)으로
  `npm install --no-audit` 수행 — **package-lock.json 무변경**(추가 설치만).
  실행 중 Vite가 네이티브 바이너리를 잠가 `npm ci`는 EPERM으로 불가했음을 기록.
- 첫 E2E 실행 시 Vite가 신규 의존성 최적화로 full-reload를 보내 1회 실패
  ("Execution context was destroyed") — 재실행으로 해소 (1회성).

## 4. 다음 단계

Stage 5 — `npm/editor/README.md` 저장 계약 문서화 + 최종 회귀 + 최종 결과보고서.
승인 후 진행.
