# PR #979 최종 보고 — rhwp-studio 플랫폼별 커서 이동 단축키 정합

## 1. 결정

**merge** — 한컴 공식 근거 명확 + 검증 통과. 순수 프론트엔드 변경.

| 항목 | 값 |
|------|-----|
| 번호 | #979 |
| 제목 | Task #945: rhwp-studio 플랫폼별 커서 이동 단축키 정합 |
| 작성자 | postmelee (Taegyu Lee) — 기존 컨트리뷰터 |
| base ← head | `devel` ← `feature/issue-945-platform-nav-shortcuts` |
| 연결 이슈 | Refs #945 |
| 처리 | cherry-pick (`ed3f7631` → 최신 local/devel, 충돌 없음) |

## 2. 검증 결과

| 항목 | 결과 |
|------|------|
| cherry-pick `7f7bec70` | ✅ 충돌 없음 (orders/20260518 자동 병합) |
| TS 단위 테스트 (navigation-keymap 12 + 전체 19) | ✅ 19 passed, 0 failed |
| `npm run build` (rhwp-studio) | ✅ 성공 (tsc + Vite) |
| CI (Build & Test / Analyze / Canvas diff / CodeQL) | ✅ 전부 pass |

## 3. 평가 요약

### 강점
- 한컴 공식 도움말 단축키 체계 정확 반영 (Mac `⌥/⌘+Arrow`,
  Windows `Ctrl+Arrow`), 공식 URL 근거 명시.
- 신규 `navigation-keymap.ts` 순수 함수: 플랫폼 감지 견고
  (userAgentData→platform→userAgent 폴백), 수식어 배타 검사.
- 일반 keydown ↔ IME pending navigation 동일 keymap 공유
  (이슈 #945 핵심 해소).
- Windows Alt+Arrow 단어 이동 오작동 차단 (기존 버그 수정,
  단위 테스트로 검증).
- 순수 rhwp-studio 변경 — Rust/파서/렌더러/WASM 무관.

### 비고 (PR 범위 밖, 기록)
`package.json` 의 `"test": "node --test tests/*.test.ts"` 는
node 22+ 에서 `--experimental-strip-types` 없이 .ts 직접 실행 불가
(`ERR_UNKNOWN_FILE_EXTENSION`). CI 가 rhwp-studio npm test 를
게이트에 포함하지 않아 그동안 드러나지 않음. PR #979 코드 버그가
아닌 기존 빌드 스크립트 이슈 — 별도 정리 권고.

## 4. 처리

- cherry-pick → 검증 통과 → `local/devel` merge
- PR #979 close (cherry-pick 반영 명시) + 이슈 #945 close
- `pr_979_review.md` / `pr_979_report.md` → `pr/archives/`
