# PR #7988 적용 기록 — Unicode 원문 검색 범위

## 적용 신원

- 원 PR https://github.com/edwardkim/rhwp/pull/7988 · author donggyun112 <ssddgg99@daum.net>.
- 원 head `a7c884a79efed47ad59d0fbf78227cc5dbe46624` · 최신 base `7d01ef610f378157b79d36ef23e8ed4a3c8ab051` · 검증 head `2dee7730306dcc7f57615c87c25cb09af3c90f63`.
- #7978 → #7985와 누름틀 보정 → #7988의 아래 6개 commit → 실제 UI 회귀 보강 순서다.
- `cherry-pick -x`의 원 SHA와 author를 보존했고 #7988 적용 충돌은 없었다.

| 원 commit | 리베이스 후 적용 commit |
| --- | --- |
| `e30de90ec8f33ff5647466bcb2db1ab0b2a46dfa` | `b76a4f38fc8b0d8e8e31e36e6e4ed743d41e875a` |
| `66f9e83e00dcc5c64851eea498cd6884c6dbe0b6` | `f81c7672878146a5aca12b41f247ad57652ef4fe` |
| `b0ed20bd9b535a3b7be74ba75d1ba9981f880870` | `5e782e5d3c5437fbefb0ecd913d144f7e7270e6a` |
| `5dc80dc0dc292cacc0b5aa5c9e986fe0568ea7cd` | `8a43eb8a874aa9440f205b2647182c1c43d4ef00` |
| `d9295283fcaa24e040ad8a9595ff6d32cc32cec2` | `8cd540c2e0ece92ca11b9ace7d4695cdb1656b8a` |
| `a7c884a79efed47ad59d0fbf78227cc5dbe46624` | `d3a8bddda06344b64dffe4fb130fbdec5a761bb3` |

## 메인터너 범위와 증거

검색 제품 수정은 원 기여 그대로다. `2dee7730306dcc7f57615c87c25cb09af3c90f63`에서 실제 UI의 모두 바꾸기·undo/redo 3사례와
manifest 설명을 보강했다. [최종 검토](pr_7988_review.md)에 구현 소비 경로·독립 기대값·Native 4개
수정 전 FAIL/후 PASS·수정 전 실제 UI 4실패·최종 fresh WASM 검사·전체 회귀 결과를 연결했다.

초기 producer 이후 UI 파일만 추가했고, #7995를 반영한 rebase에서도 Rust source/test/build 입력은 같다.
중단한 전체 회귀는 성공으로 계산하지 않았다. 리베이스 후 최신 base 정책·전체 회귀·fresh WASM·frontend·
실제 Chrome을 다시 수행했으며 정확한 SHA와 WASM 해시는 검토 문서에 있다.

## 통합 PR code CI와 남은 후속 처리

[통합 PR #7997](https://github.com/edwardkim/rhwp/pull/7997)의 정확한 code head
`fd3d97b0b9effa2d0d4a1f65f47e3475ba4ec5c8`에서 [CI](https://github.com/edwardkim/rhwp/actions/runs/39041517840) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39041517836) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39041517829) · [Adapter](https://github.com/edwardkim/rhwp/actions/runs/39041517834) · [Proptest](https://github.com/edwardkim/rhwp/actions/runs/39041517854) · [Controller](https://github.com/edwardkim/rhwp/actions/runs/39042954892)를 확인했다.
집계 Build & Test·언어별 CodeQL·Canvas·Adapter·Proptest와 정책 status가 완료됐고
실패·진행 중 check는 없다. GHAS CodeQL의 neutral과 영향 정책상 skip을 실제 성공 worker와 구분한다.
PR 생성 전 source PR CI를 재사용한 결과가 아닌 이 통합 PR의 실제 실행이다.

사용자는 CI 모니터링 이후 병합·후속 처리를 승인했다. 이 기록은 동일 PR의 문서-only trailing이며
제품·테스트·workflow·fixture·baseline은 변경하지 않는다. 최신 base와 merge tree의 문서 링크·
오늘할일 보존을 확인하고 push한 뒤, trailing head의 실제 fast-pass·required aggregate·mergeability를
다시 확인한다. 아직 실행 전인 trailing CI·merge·이슈 종료·정리는 완료로 표시하지 않는다.

원본 로컬 영수증·로그의 종료 후 보존 경로는 `/home/tsjang/rhwp-pr-evidence/pr-7997/`다.
저장소의 ignored `output/pr-review/donggyun112-20261017/`는 실행 당시 경로이며,
최종 보존 파일을 해시 확인한 뒤 `post_merge.md`에 따라 root `output/` 전체를 정리한다.
공용 `target/pr-review`와 contributor fork는 보존한다.
