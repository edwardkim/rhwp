# PR #9376 구역·도형·연결선 보완 통합 셀프 리뷰

semanticist21 님의 세 기여 커밋을 보존하고 보완본의 동작을 검토했다. 제품을 빌드한 커밋은 `e4ce75728af706c8fdab93882172ad07843b00b8`, 원격 제품 CI를 통과한 제출 커밋은 `88f03ab788766c71f1e59a0fe0d20de6fb591a70`, 기준 devel은 `28a6afc1ebe3e9873605f2d306ca094b422abf24`이다. [통합 PR #9376](https://github.com/edwardkim/rhwp/pull/9376) · [독립 한컴 Print와 측정 기록](../assets/semanticist21-20261124-rework/final/README.md)

| 원 PR | 기여 SHA | 누적 병합 SHA | 확인한 동작 | Native·WASM 각각 |
| --- | --- | --- | --- | ---: |
| [#9281 검토](pr_9281_review.md) | `f528069f3b` | `980b84b40e` | 첫 구역을 A4·1쪽으로 시작할 조건, 다음 구역의 번호와 바탕쪽 보존 | 32쪽, 100.00% |
| [#9286 검토](pr_9286_review.md) | `65ca6f4128` | `eb375b02a9` | 구 HWPX 투명 채움의 명시적 복구와 묶음·가로 타원 파선의 위치 | 25쪽, 100.00% |
| [#9325 검토](pr_9325_review.md) | `6f4ded68c7` | `327be655a1` | 본문·셀에서 가운데 선분을 끌 때의 이동 축과 끝점, 0개·1개 꺾임의 인쇄 | 12쪽, 100.00% |

## 셀프 리뷰 결과

구역 첫 본문 줄과 하단 쪽 번호, 타원의 오른쪽 파선, 끝점 사이의 연결선을 원본과 인쇄본에서 확대해 확인했다. 8개 최신 devel PR의 대조군 21쪽씩도 두 백엔드 모두 기준과 후보 PNG가 같았다. [구역](../assets/semanticist21-20261124-rework/final/section-print.png) · [파선 윤곽](../assets/semanticist21-20261124-rework/final/ellipse-east.png) · [꺾임](../assets/semanticist21-20261124-rework/final/connector-print.png) · [기준 기능 대조](../assets/semanticist21-20261124-rework/final/rebase-pages.tsv)

보호 문서 4,046행에 기준 devel 대비 하락은 0건이다. 정본 100.00%를 재현하지 못한 WASM 14쪽은 99.87–99.99%이며, 해당 쪽의 기준과 후보 PNG 해시가 모두 같다. 작업지시자는 이번 통합에서 이 14쪽만 수용했다. Native 2,023쪽은 모두 정본 하한 이상이다. 보호 정본과 허용치는 유지했다. [14쪽 실제 값](../assets/semanticist21-20261124-rework/final/protected-host-below.tsv) · [전쪽 원장](../assets/semanticist21-20261124-rework/final/protected-host-pages.tsv)

전체 Rust 회귀 14,066건, Native 공식 검사 4,072건, Studio 2,832건이 통과했다. 실제 브라우저에서 GitHub PR 안의 비교·오버레이 12개를 열었고, 공개된 원본 TSV 180개·해시 목록·PNG 12개를 해당 제출 커밋의 바이트와 대조했다. [로컬 검사](../assets/semanticist21-20261124-rework/final/test-results.json) · [TSV 원본별 해시](../assets/semanticist21-20261124-rework/final/silhouette/files.tsv)

## 제출 커밋의 원격 검사

| 검사 | 실행 결과 |
| --- | --- |
| CI | [성공](https://github.com/edwardkim/rhwp/actions/runs/45279878825) |
| CodeQL | [성공](https://github.com/edwardkim/rhwp/actions/runs/45279878917) |
| Render Diff | [성공](https://github.com/edwardkim/rhwp/actions/runs/45279878807) |
| Adapter inter-diff | [성공](https://github.com/edwardkim/rhwp/actions/runs/45279878919) |
| Proptest roundtrip | [성공](https://github.com/edwardkim/rhwp/actions/runs/45279878851) |
| CI Impact Policy | [성공](https://github.com/edwardkim/rhwp/actions/runs/45285341990) |

Adapter의 첫 실행에서는 체크아웃에 22분 39초가 걸려 Rust 컴파일 중 작업의 25분 상한에 도달했다. 로컬에서는 같은 제출 커밋으로 SVG·Null·Trace·PNG·Skia 경로의 비교 7건과 실행기·하네스 18건이 통과했다. 같은 제출 커밋에서 이 작업 한 건을 재실행해 성공했다. [첫 실행](https://github.com/edwardkim/rhwp/actions/runs/45279878919/job/138061642373) · [재실행](https://github.com/edwardkim/rhwp/actions/runs/45279878919/job/138076824966)

병합 직전에 검토 문서 커밋의 필수 검사와 정책 결과, 최신 devel과 원 PR 세 건의 head를 다시 확인한다.
