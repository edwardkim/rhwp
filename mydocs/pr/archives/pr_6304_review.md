---
kind: review
status: accepted_with_maintainer_correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6304 검토 - 한양 글꼴 대체 face 선택

- PR: [#6304](https://github.com/edwardkim/rhwp/pull/6304)
- 작성자: `@planet6897`
- 원 source head: `30ceb2b063dd3fdca0086e5b21664997c118a90a`
- 누적 검토 적용: `ce1f87f88` (`git cherry-pick -x`)
- 메인터너 보정: `8b2fee99facaa05758ff5d8edbf7a68fc7df8ed0`

## 변경 검토

Studio의 한양 계열 글꼴 대체에서 CanvasKit 등록 face와 로컬 글꼴 확인 결과를 연결하고,
관련 font family chain 회귀를 추가한다. 변경 경로는 `rhwp-studio/src/core/font-loader.ts`,
`font-rule-runtime.ts`, `font-substitution.ts`, `view/canvaskit-renderer.ts`와 Studio 테스트다.

## 발견 사항

### P2 - 명시적 확인 목록을 이전 전역 감지 결과가 덮어씀

`FontFamilyChainOptions.confirmedLocalFonts`를 명시적으로 전달했을 때 빈 목록을 포함해 그 목록이
판정의 권위 있는 입력이어야 한다. 원 구현은 목록에서 일치 항목을 찾지 못한 뒤에도
`detectedOSFontIndex()`로 fallback하므로, 이전 문서의 전역 감지 결과가 남아 있으면 현재 호출자가
확인하지 않은 local face가 선택될 수 있었다.

메인터너 보정 `8b2fee99f`는 명시적 목록에서 불일치하면 즉시 `null`을 반환하도록 제한했다. 전역 감지
fallback은 옵션을 전달하지 않은 경우에만 유지한다. 같은 보정은 전역 감지 set에 `HY중고딕`이 남아 있어도
빈 `confirmedLocalFonts`로는 그 face를 선택하지 않는 회귀 테스트로 고정했다.

## 검증 상태

- 원 source head의 GitHub required CI는 성공했다.
- `cd rhwp-studio && node --test tests/font-substitution.test.ts tests/font-rule-runtime.test.ts`는
  16/16 성공했다. 설치된 `HY중고딕` 우선, 설치되지 않은 경우 기존 체인 유지, 명시적 빈 목록의
  전역 감지 fallback 차단을 포함한다.
- 브라우저 실측과 PDF/visual sweep은 이 Studio font-chain 보정의 직접 검증 범위가 아니다.

## 최종 판정 - 메인터너 보정 후 수용

`#6304`는 **메인터너 보정 후 수용**한다. contributor 원 commit은 재작성하지 않았고, 보정 커밋과
검증 책임을 위와 같이 분리해 기록했다. 통합 PR 최신 head의 CI 성공은 merge 전 별도 확인한다.
