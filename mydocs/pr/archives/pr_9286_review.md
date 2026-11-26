# PR #9286 구버전 HWPX의 도형 투명도 재검토

메인테이너 보완 후 v0.9.1의 완전 투명 채움을 명시적으로 복구했고, 원본과 가로·묶음 타원의 첫 파선이 한컴 인쇄와 일치했다. 25쪽의 Native·fresh WASM 최저는 100.00%다. 2026-11-24 작업지시자가 기존 devel과 보완본의 PNG가 같은 WASM 보호 문서 14쪽에 한해 통합을 승인했다. [최종 출력과 저장 검증](../assets/semanticist21-20261124-rework/final/README.md)

## 기여자 재제출 시점의 검토

2026-11-24 메인테이너 판정은 **머지 보류**다. 원본 반투명 타원의 위쪽 위치는 개선됐고, 원본 표지가 들어 있는 기여자 fixture도 투명 채움을 보존한다. 공식 v0.9.1에서 실제로 저장한 표지 없는 HWPX의 투명 채움은 후보에서 사라진다. [보완 요청](https://github.com/edwardkim/rhwp/pull/9286#issuecomment-7089120496) · [기여자 답변](https://github.com/edwardkim/rhwp/pull/9286#issuecomment-7103633468)

- 공식 v0.9.1 Linux 실행 파일의 배포 체크섬을 확인한 후 같은 HWP를 `rhwp export-hwpx`로 다시 저장했다. 산출물은 이전 보류 당시 파일과 ZIP 전체 SHA-256이 같다. `0106516be2183eb65cc2e3afe06812e2abd8a59870a232fc1bcb2219d51722bb`이며 두 원본 표지가 모두 없고 `winBrush alpha="1"`을 담고 있다. [배포물·원본·산출물 해시](../assets/semanticist21-20261124-rework/release-0.9.1-export.json)
- 같은 HWP와 구버전 HWPX의 `fillAlpha`는 최신 `devel`에서 255·255이고 후보에서는 255·1이다. 후보는 HWP와 HWPX 양 형식으로 두 번 다시 저장해도 1을 유지한다. 표지가 있는 기여자 fixture는 255를 유지한다. Windows 한컴 2024는 표지 없는 HWPX의 정수 1을 거의 불투명하게 인쇄했다. 파일을 생성한 구버전의 의도와 표준 정수 해석을 모두 확인하기 위해 원본 HWP와 변환본을 따로 인쇄했다. [기준 읽기값](../assets/semanticist21-20261124-rework/devel-alpha-properties.json) · [후보 저장·재열기](../assets/semanticist21-20261124-rework/wasm-api-and-roundtrips.json)
- 후보에서 두 파일 사이에 Native 23,260픽셀, 브라우저 Canvas 22,379픽셀 차이가 난다. 구버전 HWPX의 채움이 본문을 덮는다. 기준 Native는 두 파일을 모두 투명하게, 기준 Canvas는 두 파일을 모두 채워 그렸다. 각 기준에서 두 파일의 픽셀 차이는 0이다. [같은 파일의 기준·후보 비교](../assets/semanticist21-20261124-rework/alpha-reopen.png) · [픽셀 수와 영역](../assets/semanticist21-20261124-rework/alpha-develop-candidate.json)
- 음수 세로 오프셋 원본 타원의 Windows 한컴 2024 Print 대비 raw는 Native 99.94%, fresh WASM 99.95%다. 별도의 획·채움 대조군에서 투명 채움에 불투명 테두리만 남긴 사각형은 한컴과 WASM에서 보이지만, Native에서는 테두리도 사라진다. 이 대조군의 raw는 Native 97.34%, WASM 100.00%다. [타원 원본](../assets/semanticist21-20261124-rework/ellipse-print-pages.png) · [획·채움 대조군](../assets/semanticist21-20261124-rework/stroke-fill-pages.png)

보류 해제에는 표지 없는 v0.9.1 저장본을 식별할 근거나 사용자가 구버전 해석을 선택할 수 있는 복원 경로가 필요하다. 한컴이 작성한 정상 정수 1과 원본 표지가 있는 구버전 파일도 각각 보호해야 한다. Native의 획과 채움은 독립적으로 확인한다. 앞 라운드에서 독립 적용을 확인한 로컬 메인테이너 보정은 `d7e2b3c945`다.

원 head는 `ee1c0ce527`에서 `65ca6f4128`로 바뀌었다. 리뷰어는 `edwardkim`이며 원 head의 CI와 최신 `devel` 개별 병합 시뮬레이션이 통과했다. 실제 로컬 검증은 세 기여를 함께 적용한 `327be655a1`에서 수행했다. [공통 검증 기록](../assets/semanticist21-20261124-rework/README.md)

## 통합 PR #9376

원 PR의 head `65ca6f4128150a570f06c055f8e1720ee6fb98e2`를 `eb375b02a9`의 부모로 보존했다. 메인테이너 보완 커밋은 cdd07a5908 · f997ce9409 · e4ce75728a이다. 구 HWPX 투명 채움의 명시적 복구와 묶음·가로 타원 파선의 위치을 독립 한컴 Print와 실제 저장·편집 경로에서 검증했다. Native와 fresh WASM의 해당 25쪽은 모두 100.00%였다. [통합 셀프 리뷰](pr_9376_review.md) · [원격 제품 CI](https://github.com/edwardkim/rhwp/actions/runs/45279878825)
