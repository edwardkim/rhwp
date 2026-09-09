# PR #6734 통합 검토 기록

## code candidate 판정: 수용

## 범위

- PR: [#6734](https://github.com/edwardkim/rhwp/pull/6734)
- code candidate head: `a239517aa32519ad8dce938768e63d3e45593ea7`
- 기준 devel: `d1831146587b1ac2346f9ed1216a64c2943a02f9`
- 포함 source PR: #6702, #6732, #6733
- #6702는 공개 canonical fixture를 필수화한 메인터너 보정을 포함한다.

## 실제 PR CI

- [CI](https://github.com/edwardkim/rhwp/actions/runs/33890061932): full lane의 lint, native-skia,
  frontend package, 4 build archive와 4 default-feature shard, aggregate `Build & Test`가 모두 success다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33890062198): JavaScript/TypeScript, Python,
  Rust worker와 preflight가 success다. Rust worker는 12분 29초 실행됐다.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33890061664),
  [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/33890062147),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/33890062253),
  [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/33891510549)는 success다.
- WASM Build, Frontend unit gates, Refresh nextest target duration data는 정책상 expected skip이다.

## 근거와 한계

- focused local regression 5건, formatter, all-target clippy와 native-skia 범위 한정 PNG 증적은
  PR 본문 및 [PR별 검토 기록](pr_6702_review.md)에 연결돼 있다.
- PNG는 target caption, logical page count, page 10 body-overflow와 마지막 페이지 비공백만
  뒷받침한다. font glyph 대체 때문에 Hancom/Studio pixel equivalence를 주장하지 않는다.

## trailing 및 merge 조건

- 이 문서와 실제 CI 기록은 code candidate 성공 뒤의 trailing commit이다.
- trailing head의 required CI가 success 또는 policy expected skip이고 `MERGEABLE`, `CLEAN`일 때만
  일반 merge commit으로 병합한다.
- merge 뒤 실제 `devel` CI가 성공하기 전에는 #6702, #6732, #6733의 comment/close, issue 처리,
  contributor branch 정리를 하지 않는다.
