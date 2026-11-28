# PR #9495 리뷰 — 셀 편집 뒤 연구노트 표 순서와 비겹침 보존

## 최종 판정

**승인 — #7065의 표 겹침과 후행 표 배치 순서 보정을 확인했다.** 소스 `854872d9411da27d6bda43047a20799f3c1ff741`을 검증했고, 이 소스를 담은 제출 head `129dc6865c899d6d44b20ef307500b36bc92f807`의 GitHub Actions도 통과했다. 메인테이너가 이번 제출에서 수용한 기존 정본 하한 미달의 범위는 아래에 기록한다. 병합 전에는 이 기록을 포함한 최신 head의 필수 검사와 병합 가능 상태, 병합 승인을 확인한다.

## 검토 대상

| 항목 | 확인값 |
| --- | --- |
| PR·작성자·대상 | [#9495](https://github.com/edwardkim/rhwp/pull/9495) · `edwardkim` · `devel` |
| 검토 기준 | `upstream/devel` `ca96e9f342a541da8aee40f96495180b6b2ee863` |
| 검토 방식 | 메인테이너 셀프 리뷰 |
| 연결 이슈 | [#7065](https://github.com/edwardkim/rhwp/issues/7065) · `Refs` |

## 조판과 실제 재검증

- 같은 앵커를 사용하는 자리차지 표에서 셀 줄이 선언 프레임을 넘고 실제로 남은 공간에도 들어가지 않으면 저장 스택 허용을 적용하지 않는다. 후행 표는 배치 시점의 실제 점유를 기준으로 다음 단이나 쪽에 놓인다. 후행 표만 새 단을 열 때 앞 단에서 소비한 기존 앵커 오프셋을 다시 더하지 않는다. [셀 줄 프레임 확인](../../../src/renderer/float_placement.rs) · [표 수용과 실제 배치](../../../src/renderer/typeset/table/block/entry.rs)
- 셀에 Enter 7회·9회를 입력한 같은 HWP 저장본을 `layout-anomaly --types Table --overlap-tolerance 1 --json`으로 다시 열었다. `devel`은 두 경우 각각 3쪽과 겹침 1건이었다. 후보는 각각 4쪽이었고 겹침과 쪽 밖 표가 없었다. Enter 11회·15회 대조군은 두 소스에서 모두 4쪽과 겹침 0건이었다. Enter 7회의 제목·본문·서명은 각각 2·3·4쪽이다. [3쪽 본문](../assets/issue_7065/wasm/labnote-001-enter7-p3.png) · [4쪽 서명](../assets/issue_7065/wasm/labnote-001-enter7-p4.png)
- Native 321개와 실제 Chrome의 새 WASM 316개 편집 상태에서 표의 순서·겹침·저장 재열기 동작을 확인했다. 본문과 떨어져 제목·서명이 2쪽에 몰리는 한컴 편집 저장본은 현상 재현 근거로 남겼다. 목표 배치의 판정은 본문 행의 소속과 보존, 제목→본문 조각→서명 순서 및 비겹침을 기준으로 했다. 미편집 공개 연구노트는 Native와 새 WASM에서 각각 전 9쪽이 독립 한컴 2020 Print 대비 100.00%였다. [비교 기준과 전쪽 TSV](../../report/task_m100_7065_report.md#제출-시각-증적) · [편집 저장본 판정 근거](../../report/task_m100_7065_report.md#검증-기준-정정)
- 정본 보호 4,046행 중 Native 180행과 WASM 23행이 공식 하한에 미달했다. 같은 입력과 공급 조건으로 출력한 `devel`과 후보에서 미달 203행의 공식 표시값과 PNG SHA-256이 모두 일치했다. 지정한 굴림 TTC 5.04를 공급한 중첩 표 WASM 8쪽도 두 소스 모두 99.97%였다. 이 기존 203행만 이번 PR의 수용 범위로 기록한다. [203행의 전쪽 대조표](../assets/issue_7065/protected-below.tsv)
- #9481을 포함한 기준에서 집중 회귀 25건, 최소 글꼴 환경의 전체 Rust 회귀 14,291건과 Native Skia 4,227건, 필수 린트와 워크스페이스 빌드가 통과했다. 기존 표 순서·이어받기와 SVG snapshot 검사도 통과했다. [로컬 검증과 실행 조건](../../report/task_m100_7065_report.md#9481-반영-후-재검증)

## 제출 head의 GitHub Actions

2026년 11월 28일에 확인한 제출 head `129dc6865c899d6d44b20ef307500b36bc92f807`의 Full CI에서 빌드·회귀 A–E·Native Skia·린트가 통과했다. CodeQL 분석, Canvas 시각 비교, Adapter 비교, Proptest 및 최종 CI 영향 정책도 성공했다.

| 검사 | 결과 |
| --- | --- |
| Build & Test | [성공](https://github.com/edwardkim/rhwp/actions/runs/46115714042/job/140895934758) |
| CodeQL | [성공](https://github.com/edwardkim/rhwp/actions/runs/46115714109) |
| Render Diff | [성공](https://github.com/edwardkim/rhwp/actions/runs/46115714064) |
| Adapter inter-diff | [성공](https://github.com/edwardkim/rhwp/actions/runs/46115714035) |
| Proptest roundtrip | [성공](https://github.com/edwardkim/rhwp/actions/runs/46115714032) |
| CI Impact Policy | [최종 성공](https://github.com/edwardkim/rhwp/actions/runs/46118274631) |
