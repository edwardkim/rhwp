# PR #9325 연결선 가운데 점 재검토

메인테이너 보완 후 본문·셀 안의 가운데 선분을 실제 Studio 마우스로 옮기고 양 형식으로 저장·재열기했다. 꺾임 0개·1개 인쇄 대조군을 포함한 12쪽의 Native·fresh WASM 최저는 100.00%다. 2026-11-24 작업지시자가 기존 devel과 보완본의 PNG가 같은 WASM 보호 문서 14쪽에 한해 통합을 승인했다. [최종 조작과 출력 검증](../assets/semanticist21-20261124-rework/final/README.md)

## 기여자 재제출 시점의 검토

2026-11-24 메인테이너 판정은 **Studio 편집 완료 기준으로 머지 보류**다. 기여자가 답변한 두 인쇄 결함은 해소됐다. 남은 동작은 사용자가 화면의 가운데 손잡이를 드래그할 때 새 API를 호출하는 것이다. [보완 요청](https://github.com/edwardkim/rhwp/pull/9325#issuecomment-7089123105) · [기여자 답변](https://github.com/edwardkim/rhwp/pull/9325#issuecomment-7103632904)

| 검증 | 후보에서 확인한 결과 |
| --- | --- |
| 본문·셀 × 직각·곡선 × HWP·HWPX | 기존 8개 모두 Windows 한컴 2020 Print 대비 Native와 fresh WASM raw 100.00%. 곡선은 가운데를 지나고, 셀의 선·표 높이·뒤 본문 위치가 맞는다. |
| 엔진 API로 가운데 점 이동 | 본문·셀의 두 종류 모두 해당 축의 좌표만 변경됐다. 크기·다른 축은 유지되며 HWP·HWPX 각각 연속 두 번 저장·재열기 후에도 같다. |
| 실제 Studio 가운데 손잡이 드래그 | Chromium에서 선택한 곡선의 손잡이를 오른쪽 40px·아래 26px로 끌었다. 가운데 좌표는 (10100, 8500) 그대로이고, 폭은 19000→18998, 높이는 17000→18953 HWPUNIT로 바뀌었다. |

화면에서는 가운데 손잡이를 아래쪽 크기 조절 방향 `s`로 등록한다. 마우스 처리기는 양 끝점만 별도로 처리하고 `s`는 일반 크기 조절로 전달한다. 후보의 `connectorMidAxis`는 `x`였다. 같은 실제 마우스 동작은 최신 `devel`에 해당하는 기준 빌드에서도 크기 조절로 연결됐다. [손잡이 등록](../../../rhwp-studio/src/engine/table-object-renderer.ts) · [마우스 처리](../../../rhwp-studio/src/engine/input-handler-mouse.ts) · [브라우저 전후 좌표](../assets/semanticist21-20261124-rework/candidate-drag.json) · [드래그 전후 화면](../assets/semanticist21-20261124-rework/studio-midpoint.png)

가운데 점을 가진 연결선의 실제 손잡이가 축별 API로 전달되고 크기·끝점이 보존되는지 확인하면 편집 기능으로 수용할 수 있다. 엔진·인쇄 개선을 먼저 제출하려는 경우에는 PR의 완료 범위를 명시하고 남은 Studio 경로를 후속 작업에 연결할 수 있다. 꺾임이 0개·1개인 비편집 대조군의 기존 인쇄 차이는 각각 79.15%·68.37%이며, 최신 기준·후보 PNG는 두 backend에서 같다. 이 수치는 앞선 8개 인쇄 보완의 재발로 집계하지 않았다. [인쇄 비교](../assets/semanticist21-20261124-rework/connector-print-pages.png) · [API 저장 결과](../assets/semanticist21-20261124-rework/wasm-api-and-roundtrips.json) · [쪽별 결과](../assets/semanticist21-20261124-rework/pages.tsv)

원 head는 `aee210f9f2`에서 `6f4ded68c7`로 바뀌었다. 리뷰어는 `edwardkim`이며 원 head의 CI와 최신 `devel` 개별 병합 시뮬레이션이 통과했다. 실제 로컬 검증은 세 기여를 함께 적용한 `327be655a1`에서 수행했다. [공통 검증 기록](../assets/semanticist21-20261124-rework/README.md)

## 통합 PR #9376

원 PR의 head `6f4ded68c7391bf0497930230ef9aeba1c8ed29e`를 `327be655a1`의 부모로 보존했다. 메인테이너 보완 커밋은 f997ce9409 · af7f716c37 · 53058f69c4이다. 본문·셀에서 가운데 선분을 끌 때의 이동 축과 끝점, 0개·1개 꺾임의 인쇄을 독립 한컴 Print와 실제 저장·편집 경로에서 검증했다. Native와 fresh WASM의 해당 12쪽은 모두 100.00%였다. [통합 셀프 리뷰](pr_9376_review.md) · [원격 제품 CI](https://github.com/edwardkim/rhwp/actions/runs/45279878825)
