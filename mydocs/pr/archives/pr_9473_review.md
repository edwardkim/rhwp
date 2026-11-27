# PR #9473 검토

[통합 PR](https://github.com/edwardkim/rhwp/pull/9473)에서 9414~9471번의 열린 원 PR 16건을 검토했습니다. 기준 `devel`은 `77e4c3d1369623818e8cf3213e3fdc21d8e6502e`입니다. 원본 고유 커밋 29개를 체리픽했고, 원 head와 적용 SHA는 개별 기록에 남겼습니다.

[#9414](pr_9414_review.md) · [#9441](pr_9441_review.md) · [#9456](pr_9456_review.md) · [#9459](pr_9459_review.md) · [#9460](pr_9460_review.md) · [#9461](pr_9461_review.md) · [#9462](pr_9462_review.md) · [#9463](pr_9463_review.md) · [#9464](pr_9464_review.md) · [#9465](pr_9465_review.md) · [#9466](pr_9466_review.md) · [#9467](pr_9467_review.md) · [#9468](pr_9468_review.md) · [#9469](pr_9469_review.md) · [#9470](pr_9470_review.md) · [#9471](pr_9471_review.md)

## 적용 범위와 로컬 검증

차트 자동 값축, HFT 라틴 글자 폭과 줄 끝 하이픈, 표 분할과 HWPX 행 주소를 검증했습니다. Native 편집에서는 셀 편집·한글 조합, 저장, 중첩 표 문단, 클립보드·Tab 이동, 검색·치환, 접근성, 서식과 미저장 Print PDF를 확인했습니다.

코드 `b25bbad4f477a5f6c9bfc212f58b6f03e2a090dc`에서 기본 nextest 14,261건, Native Skia 4,225건, Rust lint와 워크스페이스 빌드가 통과했습니다. 정본 보호 23개 문서의 Native·fresh WASM 각 2,023쪽을 대조했으며 4,046행 모두 하한을 충족했습니다. 기준 devel과 출력 PNG 해시도 전부 일치했습니다. [로컬 검증](../assets/integration-8678-20261128/local-verification.json) · [정본 보호 쪽별 대조](../assets/integration-8678-20261128/protected-page-comparison.tsv)

같은 조건으로 재현한 #9414 WASM 11쪽은 원본 head와 PNG가 같았습니다. #9441 Native 11쪽 중 10쪽이 같았고 9쪽은 99.47%에서 100.00%로 개선됐습니다. #9467 Native·WASM 22쪽의 PNG는 원본 head와 같았습니다. 실제 참고문헌 256~264쪽을 두 한컴 2020 PDF에 대조한 36행의 최저값은 98.60%였습니다.

새로 생성한 독립 Print PDF 두 파일을 반영했습니다. #9463의 현행 정본은 `HYHWPEQ.TTF` 1.13 공급을 확인했고, 보호 셀의 일반 텍스트 읽기와 쓰기 거부를 각각 검증했습니다. #9451은 공개 대체 표본의 저장·재열기를 확인했으며 제보된 비공개 원본의 검증이 남아 있습니다.

## GitHub Actions

제출 head `39fce9ff85c5a5bca3558337c3ab10521104a099`의 Full CI와 최종 정책 판정이 통과했습니다. [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/46048240002) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/46048239552) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/46048239579) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/46048239635) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/46048239745) · [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/46052132384)

이 기록을 포함한 새 head의 CI와 병합 가능 상태를 확인한 뒤 병합합니다. 이후 원 PR에 검토 결과와 병합 SHA를 남기고 연결 이슈의 종료 여부를 확인합니다.
