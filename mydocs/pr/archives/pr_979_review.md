# PR #979 검토 — rhwp-studio 플랫폼별 커서 이동 단축키 정합

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #979 |
| 제목 | Task #945: rhwp-studio 플랫폼별 커서 이동 단축키 정합 — Mac/Windows 한컴 공식 단축키 반영 |
| 작성자 | postmelee (Taegyu Lee) — 기존 컨트리뷰터 |
| base ← head | `devel` ← `feature/issue-945-platform-nav-shortcuts` |
| 연결 이슈 | Refs #945 (관련 #223), assignee 본인 지정 완료 |
| mergeable | MERGEABLE / BEHIND (충돌 아님) |
| CI | Build & Test ✅ / Analyze(rust/js/py) ✅ / Canvas visual diff ✅ / CodeQL ✅ |
| 커밋 | 2 (ed3f7631 fix + 3634ea2d devel merge) |

## 2. 배경 (이슈 #945)

rhwp-studio 입력 핸들러가 `ctrlKey || metaKey` 를 한 계열로 묶고,
Alt/Option+Arrow 를 플랫폼 구분 없이 단어 이동 처리. 결과:

- macOS Option+Arrow 단어 이동은 맞지만 Command+Arrow 줄 이동 미정리
- Windows/Linux 에서 한컴 공식 Ctrl+Arrow 대신 Alt+Arrow 도 단어 이동
- IME 조합 중 pending navigation 과 일반 keydown 이 다른 플랫폼 판단

한컴 공식 도움말 근거 (Mac/Windows 단축키 별도 열):
Mac `⌥+←/→`=단어, `⌘+←/→`/`Home/End`=줄, `⌘+↑/↓`=문단 /
Windows `Ctrl+←/→`=단어, `Home/End`=줄, `Ctrl+↑/↓`=문단.

## 3. 변경 내용

순수 **rhwp-studio (프론트엔드 TS)** 변경. Rust/파서/렌더러/WASM 무관.

| 파일 | 변경 |
|------|------|
| `navigation-keymap.ts` (신규) | `detectPlatformKind()`, `getNavigationAction()`, `shouldSuppressUnmappedNavigation()` — 플랫폼별 keymap 순수 함수 |
| `input-handler-keyboard.ts` | 플랫폼별 nav shortcut 을 Ctrl/Meta 공통 처리보다 우선 판별, 전역 Alt/Option+Arrow 분기 제거 |
| `input-handler-text.ts` | IME pending navigation 도 동일 keymap 사용, Win/Linux Alt+Arrow fallback suppress |
| `selection-renderer.ts` | E2E 검증용 `selection-highlight` class 명시 |
| `tests/navigation-keymap.test.ts` (신규), `e2e/navigation-shortcuts.test.mjs` (신규) | 플랫폼별 단위 + 브라우저 E2E |
| 문서 7 (plans/working/report/orders) | Task #945 산출물 |

## 4. 검토 의견

### 4.1 강점

- 한컴 공식 도움말 단축키 체계를 정확 반영 (Mac/Windows 분리).
  공식 근거 URL 명시 — 임의 결정 아님.
- 플랫폼 감지 견고: `userAgentData.platform` → `platform` →
  `userAgent` 폴백 + 테스트 오버라이드(`__rhwpTestPlatformKind`).
- 수식어 조합 배타 검사 (`!ctrlKey && !metaKey && !altKey` 등)
  — 의도치 않은 매칭 방지.
- `shouldSuppressUnmappedNavigation()`: Windows Alt+Arrow 오작동
  차단 (기존 버그 수정).
- 일반 keydown 과 IME pending navigation 이 동일 keymap 공유
  — 경로 간 판단 불일치 해소 (이슈 핵심).
- 순수 함수 + 명시 타입 → 단위 테스트 용이, 신규 단위/E2E 동반.

### 4.2 검토 포인트

- **본질이 시각 판정이 아닌 기능(키 입력) 동작**. 시각 회귀가
  아니므로 E2E `navigation-shortcuts.test.mjs` + 단위 테스트가
  핵심 검증 게이트. (작업지시자 시각 판정 부담 낮음)
- 순수 프론트엔드 — Rust 빌드/WASM 영향 없음. 검증은
  `npm test` + `npm run build` + E2E 중심.
- `selection-renderer.ts` class 추가는 동작 변경 아닌 E2E 셀렉터
  보조 — 기존 렌더 영향 점검 필요(저위험).

## 5. 검증 계획

- [ ] cherry-pick (최신 devel 위, #971/#976/#983-985 동일 패턴)
- [ ] `npm test` (navigation-keymap 단위 포함)
- [ ] `npm run build` (rhwp-studio)
- [ ] E2E `navigation-shortcuts.test.mjs` (headless) — 핵심 게이트
- [ ] Rust 영향 없음 확인 (변경 파일 rhwp-studio 한정)

## 6. 판단 (잠정)

한컴 공식 근거 명확 + 설계 견고(순수 함수 keymap, 경로 통합) +
기능 회귀 위험 낮음(프론트엔드 한정, E2E 동반). 검증 통과 시
수용 권고. 시각 본질이 아니라 E2E/단위 테스트가 판정 핵심.

검증 결과에 따라 `pr_979_report.md` 작성.
