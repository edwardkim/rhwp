---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-26
name: logic-tests-miss-build-artifacts
description: "로직 테스트 통과는 빌드 산출물 정합을 보장하지 않는다 — print.html 누락이 테스트 15개를 통과하며 두 릴리즈 배포된 사례 (#3433)"
metadata: 
  node_type: memory
  type: feedback
---

**로직 테스트가 전부 통과해도 빌드 산출물 누락은 잡히지 않는다.** 두 검증은 층위가 다르다.

2026-07-27 #3433 사례. 브라우저 확장에서 Ctrl+P 인쇄가 "파일을 찾을 수 없음" 으로 실패했고,
원인은 `rhwp-studio/public/print.html` 이 확장 dist 에 복사되지 않은 것이었다. 그동안:

- `print-surface.test.ts` 등 **print 테스트 15개가 전부 통과**했다. URL 계산(`new URL(path,
  baseUrl)`)과 명령 계약을 검증할 뿐 그 URL 이 가리키는 파일의 실재는 보지 않는다.
- 웹앱(GitHub Pages)은 정상이었다. vite 가 `public/` 을 서빙하므로 같은 코드가 웹에서만 동작.
- 그 결과 **v0.8.0·v0.8.1 두 릴리즈가 결함을 안고 스토어까지 나갔다.**

**Why:** 근본 원인은 복사 누락 자체가 아니라 그것이 드러나지 않은 구조다. `build.mjs` 의
`copy()` 가 원본 부재 시 `SKIP (not found)` 경고만 내고 넘어가 **누락이 빌드 성공으로
위장**됐다. 경고는 로그에 묻히고 exit code 는 0이다.

**How to apply:**

- 자산을 참조하는 코드를 다룰 때, 그 자산이 **각 배포 형태(웹/확장/패키지)의 산출물에 실제로
  들어가는지** 별도로 확인한다. 로직 테스트 통과를 근거로 삼지 않는다.
- 빌드 스크립트에서 "실패해도 계속 진행" 하는 지점(`|| true`, 경고 후 continue, optional
  copy)은 누락을 은폐하는 후보다. 필수 자산에는 존재 검증 게이트를 두고 없으면 실패시킨다.
- **게이트를 추가하면 실제로 잡히는지 실증한다.** #3433 에서는 원본을 임시 제거해
  `MISSING: print.html` → exit 1 을 확인했다. 추가만 하고 동작을 확인하지 않으면 그 게이트도
  같은 부류의 위장이 된다.
- "웹에서는 되는데 확장/패키지에서만 깨진다" 는 신고를 받으면 이 계열을 먼저 의심한다.

같은 계열: [[project_extension_publicdir_false]](확장 `publicDir:false` 함정과 필수 산출물
게이트), [[feedback_hallucination_locked_by_tests]](과거 믿음이 테스트로 제도화되는 사슬 —
이쪽은 테스트가 틀린 것을 고정한 경우, 이 항목은 테스트가 옳지만 층위가 달라 못 잡은 경우).
