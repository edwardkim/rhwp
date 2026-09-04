# PR #2390 검토 — embed loadFile suppressDialogs (sxngt 첫 PR)

- PR: https://github.com/edwardkim/rhwp/pull/2390 — closes #2389 (자기 등록,
  재현 로그·원인 분석 동봉)
- 첫 기여자. TS/JS 전용(6파일 +94/−12), Rust 무접촉

## 변경 본질

embed `loadFile` 응답이 로드 후 안내창(HWPX lineseg 검증 모달 #177, 로컬 글꼴
감지)의 사용자 선택 await 에 묶여 임베더가 교착 — iframe 오버레이 패턴에선
안내창이 보이지도 않아 완전 교착. **opt-in `suppressDialogs`(기본 false)**:

- rpc-router → main.ts(loadBytes/initializeDocument) 관통, true 면 검증은
  '그대로 열기(as-is)'·글꼴 안내 생략 → 로드 즉시 응답
- `@rhwp/editor` `loadFile(data, fileName, options?)` 노출 + README 안내
- 기본 false 로 대화형 동작 불변. 기존 `skipUnsavedGuard` 와 같은 결.
- 세부 정확성: `loadBytes(..., undefined, {suppressDialogs})` 로 startTime
  기본값 보존, 라우터는 `=== true` 엄격 비교.

## 로컬 재실증 (devel merge 충돌 0)

| 게이트 | 결과 |
|--------|------|
| rhwp-studio `npm ci` → `npx tsc --noEmit` | 통과 |
| rhwp-studio `npm test` (라우터 param 전달 신규 테스트 포함) | 356 pass / 0 fail |
| npm/editor `node --test` | 0 fail |
| CI (첫 기여자 — 워크플로 승인 후) | 대기 → 확인 |

## 판단

**merge 권고.** 이슈의 원인 분석(응답 보류 지점 특정, pageCount 정상 응답
대조)이 정확하고, opt-in 설계로 기존 동작을 보존한다. 첫 PR 로서 재현
로그·테스트·문서(README/d.ts) 동봉이 충실.

## 처리 결과 (2026-07-18)

CI 전 항목 green 확인 → merge(admin) + 첫 기여 환영 코멘트(+fork devel 동기화
안내). #2389 는 close-issues 워크플로 자동 close. sxngt 첫 PR 완결.
