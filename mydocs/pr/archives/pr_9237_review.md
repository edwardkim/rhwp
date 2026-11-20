# PR #9237 검토 — 도형 캡션·분할 표와 Studio 편집 네 건 통합

## 최종 판정

**메인터너 보정 후 수용 가능 — 원 PR 네 건의 기능과 조판에 대한 로컬 검증 완료.**
[코드 HEAD CI](https://github.com/edwardkim/rhwp/actions/runs/44719850146)에서 아래 제품 변경을 포함한
`cad876ecbcb093b714a36a816ffeb66c38d06ce2`의 필수 검사가 성공했다. 병합은 이번 문서 커밋이 포함된
최신 HEAD의 필수 검사가 성공한 뒤 진행한다.

## 접수와 변경 범위

| 항목 | 값 |
| --- | --- |
| 통합 PR·작성자·base | [#9237](https://github.com/edwardkim/rhwp/pull/9237) · `jangster77` · `devel` |
| 고정 비교 base | `0a437a7eefb19bfe81fc12866887e4548ea9554d` |
| 로컬 검증 제품 소스 | `3901aa782d4445efb171ec4ffb026f2a459c2daf` |
| 검토 방식 | collaborator 자가 검토 |
| 원 PR별 근거 | [#9201](pr_9201_review.md) · [#9202](pr_9202_review.md) · [#9206](pr_9206_review.md) · [#9207](pr_9207_review.md) |

메인터너는 도형 캡션의 셀 주소와 너비 호출, 머리말·꼬리말 줄과 문단 끝 빈 줄의 글자 높이,
그리고 시행령에서 검증한 원본 글자 폭과 물리 문단의 마지막 음절 자간 소비를 보정했다.
[통합 보정 원인과 범위](../assets/integration_open_prs_20261120/maintainer-cause.md).

## 검증과 시각 증적

- 최소 글꼴 Nextest 13,839건, Native Skia 라이브러리 4,034건과 그림·직접 PDF 38건,
  Studio·편집기 SDK·공용 PDF 2,822건이 통과했다. 대상 형식의 실제 편집·저장·재열기와 재저장도
  확인했다. [전체 실행 영수증](../assets/integration_open_prs_20261120/final-current-test-receipt.json) ·
  [편집·WASM 응답 결과](../assets/integration_open_prs_20261120/final-product-consumers.json).
- 정본 보호 Native·fresh WASM 전체 46조건·4,046쪽에서 하락·누락은 0쪽이었다. 시행령 19쪽과
  #7905 시행규칙 1쪽에서 상승한 Native·WASM 값 네 건을 새 하한으로 반영했다. [전쪽 TSV와 점수](../assets/integration_open_prs_20261120/final-protected/validation.json) ·
  [상승 쪽 review·overlay](../assets/integration_open_prs_20261120/final-protected/gain-reviews/README.md).
- 새 한컴 2020·2024 `Hancom PDF` Print의 입력·영수증과 Native/fresh WASM의 전체 TSV를 검증했다.
  시행령 7쪽은 두 경로 모두 100.00%, 16쪽은 99.99%다. 새 2024 Print의 4쪽 우측 셀에는 일부 줄 분할
  차이가 남는다. [대조 결과와 대표 review·overlay](../assets/integration_open_prs_20261120/README.md) ·
  [Print 영수증](../assets/integration_open_prs_20261120/official-print-references.json).
- 실제 Studio는 Canvas2D 8건·CanvasKit 소프트웨어 16건에서 요청 모드·프로필이 일치했다. 시행령과
  신청서의 로컬 글꼴 프로그램은 이번 검증 브라우저에만 공급했다.
  [실제 캡처·런타임 증거](../assets/integration_open_prs_20261120/actual-studio-renderers/README.md).

## 원 PR과 해결 이슈 후속 처리

통합 PR의 최종 CI와 병합 SHA를 확인한 뒤 원 PR 네 건에 각 검토 결과와 증적을 남긴다. 원 PR이
해결하는 #9040·#7938·#7940의 상태는 해당 동작과 통합 SHA를 함께 확인한 뒤 정리한다.
