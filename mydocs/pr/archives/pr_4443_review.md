---
kind: pr-review
status: completed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4443 검토 기록

## 결론

- 수용 가능하다. Noto Sans KR의 ASCII advance 표를 실제 번들 폰트 값으로 재생성하고, 표의
  일관성을 테스트로 고정한다.
- 최신 contributor head `1137d93e7`는 `devel` 대상이며 non-draft, `MERGEABLE`, required check
  성공 상태였다.

## 누적 검토와 검증

- `0e5b2e71c`을 누적 branch에 `-x` 체리픽했다.
- 폰트 메트릭 변경은 renderer 출력에 영향을 줄 수 있으나, contributor의 Canvas visual diff 성공과
  누적 `release-test` 전체 5,645건 통과를 확인했다. 다른 renderer 변경과 함께 컴파일·회귀했다.
- 누적 branch의 Rust formatter, Studio production build, 최신 WASM build도 통과했다.

## 범위

- ASCII 폭 데이터와 그 검증만 변경하며, 한국어 glyph·폰트 fallback·문서 레이아웃 규칙은 바꾸지 않는다.
