# PR #9380 누름틀 연속 입력과 도형 기울기 셀프 리뷰

semanticist21 님의 두 원본 커밋을 보존하고 보완한 결과를 수용한다. 검증 기준 `devel`은 `9ee9a5b0b12115f06b31e07ec89b94e454c1a744`, 제품 보완 소스는 `10f5bca362a34f3c3f27ac15e5d921d17d1d8097`, GitHub 제품 CI를 통과한 제출 커밋은 `834abd7ac515412ec0ac06066274f3492c46d9be`이다. [통합 PR #9380](https://github.com/edwardkim/rhwp/pull/9380) · [로컬 검증 원장](../assets/semanticist21-9288-9314-20261124/corrected/README.md)

| 원 PR | 원 기여 커밋 | 보완 후 확인한 동작 |
| --- | --- | --- |
| [#9288 개별 리뷰](pr_9288_review.md) | `3fc45e7057300b21c49ced254cb476fdab69ac39` | F11로 같은 위치의 두 누름틀 중 하나를 고르면 본문과 표 셀 모두 두 번째 글자와 한글 조합까지 선택한 필드에 저장된다. |
| [#9314 개별 리뷰](pr_9314_review.md) | `0cb5fcd1c941ec24e83d8420597dbeb5586e4b05` | 기울인 도형의 너비를 바꿨다가 복원하면 원래 20° 기울기와 위치를 보존한다. 지원하지 않는 요청을 거절한 뒤에도 기존 너비가 유지된다. |

## 동작과 저장 결과

#9288은 Studio와 코어가 문단·셀 주소 및 컨트롤 번호로 선택한 필드의 현재 범위를 조회하도록 보완했다. 화면 갱신 전 두 키가 들어오는 경우를 포함해 브라우저의 F11 확인 46개가 통과했다. 입력 12조합에서 편집 직후와 HWP·HWPX 재열기의 필드별 값이 일치했다. HWPX XML도 별도로 판독했다. 실행 취소·다시 실행과 배포용 WASM의 공개 API 세 개를 확인했다. [필드별 저장값](../assets/semanticist21-9288-9314-20261124/corrected/f11-roundtrips.tsv) · [XML 판독](../assets/semanticist21-9288-9314-20261124/corrected/field-xml.tsv) · [브라우저 결과](../assets/semanticist21-9288-9314-20261124/corrected/browser-results.json)

#9314는 너비를 바꿀 때 저장된 변환 행렬에 배율을 합성하고, 기울기와 회전의 조합은 속성을 쓰기 전에 검사한다. Windows 한컴에서 직접 너비를 14,912 → 16,000 → 14,912 HWPUNIT로 편집하고 저장한 HWP·HWPX 각각을 대조했다. 복원 뒤 처음 열었을 때와 다른 픽셀은 두 렌더러 모두 0개였다. 한컴 직접 편집본과 rhwp 저장본을 한컴에서 다시 인쇄한 결과도 각 형식에서 일치했다. 각 13쪽의 독립 인쇄 대비 2px 실루엣 일치율 최저는 Native 99.97%, fresh WASM 99.77%였다. [편집별 전쪽 TSV](../assets/semanticist21-9288-9314-20261124/corrected/shape-pages.tsv) · [한컴 재인쇄](../assets/semanticist21-9288-9314-20261124/corrected/windows-and-backend-pixels.json) · [±89°와 ±90° 경계](../assets/semanticist21-9288-9314-20261124/corrected/cli-skew-limits.json)

공개 원본 1쪽과 너비 복원본은 각 렌더러에서 같은 PNG다. 한컴과 비교한 전체 픽셀 일치율은 Native 99.77539%, WASM 99.76742%이고 잉크 영역 색상 일치율은 각각 41.23348%, 39.81992%다. 두 값은 RGB 채널 차이 32를 적용한 보조 측정이다. 반투명 사각형과 기울어진 양 변을 오버레이에서 확대 판독했다. GitHub의 PR 본문에서도 두 비교 그림이 실제로 표시되는 것을 확인했다. [원본 전쪽](../assets/semanticist21-9288-9314-20261124/original-page.png) · [두 렌더러의 오버레이](../assets/semanticist21-9288-9314-20261124/original-overlay.png) · [복원된 도형](../assets/semanticist21-9288-9314-20261124/corrected/native-width-restored.png)

## 회귀와 수용 범위

- 공개 글꼴 5개 환경에서 전체 Rust 회귀 14,074개와 집중 36개가 통과했다. Native 공식 검사 4,072개, Studio 2,835개, 브라우저 71개, 배포 라이브러리 3,976개, 문서 예제 8개가 통과했다. 일반 CLI와 Skia CLI가 같은 경로를 사용해 발생했던 실패 2개는 기본 기능 CLI의 해시를 고정한 뒤 전체 회귀로 재검증했다. [검사별 결과 및 건너뜀](../assets/semanticist21-9288-9314-20261124/corrected/local-gates.json)
- 직전 #9376의 구역·타원·연결선 6쪽씩은 Native와 fresh WASM 모두 독립 인쇄 대비 100.00%였다. 보호 문서 23종의 4,046행은 기준 devel과 후보 PNG가 전부 같았다. Native 2,023쪽은 모두 공식 하한을 충족했다. 작업지시자는 양쪽 모두 99.87~99.99%로 공식 100.00%를 재현하지 못한 기존 WASM 14쪽에 한해 이번 병합을 승인했다. [4,046행](../assets/semanticist21-9288-9314-20261124/corrected/protected-pages.tsv) · [예외 14쪽](../assets/semanticist21-9288-9314-20261124/corrected/protected-below.tsv)
- 두 원 PR의 자동 종료 대상 이슈는 0개다. 각주·머리말 누름틀 입력은 별도 기여에서 다룬다. [PR #9354](https://github.com/edwardkim/rhwp/pull/9354)

## 제출 커밋의 GitHub 검사

제출 커밋에서 34개 검사가 성공했고 5개가 실행 조건에 따라 건너뛰어졌다. 실패와 대기는 0개였다.

| 검사 | 결과 |
| --- | --- |
| Build & Test, Rust 회귀 A–E, Lint, Native Skia, Frontend | [성공](https://github.com/edwardkim/rhwp/actions/runs/45314827856) |
| CodeQL | [성공](https://github.com/edwardkim/rhwp/actions/runs/45314827868) |
| Canvas 시각 비교 | [성공](https://github.com/edwardkim/rhwp/actions/runs/45314827906) |
| Adapter inter-diff | [성공](https://github.com/edwardkim/rhwp/actions/runs/45314827830) |
| Proptest roundtrip | [성공](https://github.com/edwardkim/rhwp/actions/runs/45314828008) |
| 최종 CI Impact Policy | [성공](https://github.com/edwardkim/rhwp/actions/runs/45318688505) |

이 기록을 담은 최종 커밋의 필수 검사와 정책 판정, 최신 devel과의 병합 시뮬레이션 및 두 원 PR의 SHA를 확인하면 일반 merge commit으로 통합한다.
