# PR #9434 검토 — 저장 조판과 홀짝 쪽 번호 개선 네 건 통합

`upstream/devel` `48a678399072d0df11380e1ad6a3b6aa77dd677a` 위에서 planet6897 님의 원 PR 네 건을 검토했다. 원 커밋 16개를 적용했고, 각 원 head 및 적용 SHA는 개별 검토에 기록했다. 리베이스 전후에 대응하는 기존 23개 커밋의 패치가 일치했다.

| 원 PR | 확인한 동작 | 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | --- | ---: | ---: | ---: |
| [#9409](pr_9409_review.md) | 표 이어받기 원점과 저장 줄 사이 그림의 배치 | 138 | 95.77% | 95.65% |
| [#9413](pr_9413_review.md) | 마지막 줄 앵커 그림의 쪽 소속과 세로 위치 | 7 | 99.93% | 99.94% |
| [#9425](pr_9425_review.md) | 홀짝 시작 쪽 번호와 쪽별 머리말 | 11 | 95.53% | 95.54% |
| [#9427](pr_9427_review.md) | HWPX 단 첫 표 위의 저장 간격 | 54 | 95.37% | 95.37% |

한컴 2020 Print PDF와 Native·fresh WASM 각 210쪽을 대조했다. 전쪽이 90% 이상이었으며 같은 인쇄본과 글꼴로 비교한 최신 devel·원 PR 대비 하락은 각 0쪽이었다. 대표 10쪽에서 변경 영역의 배치를 직접 판독했다. [전쪽 비교와 대표 이미지](../assets/planet6897_20261125/integration-review/rebase-validation.json)

리베이스 전후 후보의 PNG 420쪽 중 407쪽의 해시가 같았다. KAF WASM 42–54쪽의 작은 영역에서 픽셀이 달랐다. 이 13쪽의 최신 devel과 후보 PNG 해시는 모두 같았고, 420쪽의 raw 실루엣 표시값도 리베이스 전과 일치했다. KAF 36쪽은 독립 인쇄본 교체 영향을 확인했다. HWPX 글상자 간격은 [#9422](https://github.com/edwardkim/rhwp/issues/9422)에서 추적한다.

#9425의 별도 264쪽 대조군은 원 PR과 통합 후보를 같은 입력으로 재측정했을 때 Native와 WASM 각각의 TSV가 같고, 각 경로에서 90% 미만인 쪽이 14개였다. 작성자가 제출 시 사용한 정확한 264쪽 재저장 입력은 확보하지 못했다. 정식 11쪽 인쇄 대조에서 쪽 번호와 머리말을 직접 확인했다. [#9425 개별 검토](pr_9425_review.md)

## 정본 보호와 로컬 검사

정본 보호 46조건·4,046행의 하락과 누락은 0행이다. 새 한컴 인쇄에 따른 하한 조정과 화학물질 표시 기준 WASM 26쪽의 하한 조정은 영수증에 기록했다. 26쪽은 최신 devel과 후보에서 99.99%였고, 해당 WASM 103쪽의 PNG 해시는 모두 같았다. [보호 쪽별 TSV](../assets/planet6897_20261125/integration-review/protected-page-comparison.tsv) · [인쇄본 변경 영수증](../assets/visual-protected-baseline/updates/issue6782-mcp-print-20261126.json) · [26쪽 조정 영수증](../assets/visual-protected-baseline/updates/issue6782-wasm-p26-20261126.json)

`HBATANG.TTF`는 한컴오피스 2020의 한컴바탕 1.21을 공급했다. 개요 탐색 정본에는 공한 OTF 6종을 공급했다. 새 Mac 허용 해시를 공급한 14조건·1,441쪽의 PNG 해시는 최신 devel과 후보에서 모두 일치했다. [글꼴 공급 매니페스트](../../verification/protected-font-supply/manifest.json)

집중 nextest 19건, 전체 nextest 14,196건, Native Skia 4,225건, Rustfmt, Native·WASM·workspace Clippy와 workspace 빌드가 통과했다. 전체 nextest는 45건을 건너뛰었다. 행정업무운영 편람 HWPX Native 383쪽과 기여자 공통 보호 8행이 통과했다. 브라우저에서 [#9424](pr_9424_review.md)의 자동 줄바꿈 본문·표 셀과 선두 수식의 선택 영역을 최신 devel과 대조했다. [검증 기록](../assets/planet6897_20261125/integration-review/rebase-validation.json)

## GitHub Actions

코드 head `d67e841907284f2415b720771139e6ce942c3bab`의 CI와 연계 검사 및 최종 정책 판정이 성공했다. [CI](https://github.com/edwardkim/rhwp/actions/runs/45833230527) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/45833230469) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/45833230413) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/45833230549) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/45833230472) · [CI Impact Policy 최종 판정](https://github.com/edwardkim/rhwp/actions/runs/45838937815)

최종 기록 커밋의 CI와 병합 가능 상태를 확인한 뒤 병합한다. 원 PR 네 건에는 병합 SHA와 대표 쪽 판독 결과를 남기고, 연결 이슈 #9396·#8482·#9419의 상태를 확인한다.
