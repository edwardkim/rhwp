# PR #9277 검토 — 규제영향분석서·ParameterSet·정책연구 문서

`jangster77`의 collaborator 자가 검토 PR이다. 고정 devel 기준은
`2480af27df441ffd199cc24602afddbd2d13dcfe`이며 GitHub에서 코드를 검증한 head는
`4ee9126f2c8e21318b9e23ad29cfefd885104156`이다. [PR #9277](https://github.com/edwardkim/rhwp/pull/9277)

## 조판과 회귀 검사

- #7749 규제영향분석서는 저장한 표 이어받기 컷과 부모의 위 바깥여백으로 측정과 배치의
  원점을 정한다. 15쪽의 마지막 빈 줄을 점유 높이에 포함했다. 30쪽에서 저장 유닛 56의
  제목을 한 번 출력하며 유닛 57부터의 표는 31쪽에 배치한다. 30쪽의 용지 밖 셀 글줄이
  73개에서 0개로 줄었다. [원본 유닛과 출력 트리](../assets/issues7749_7753_20261121/carried-child-ownership.json)
- #7750 ParameterSet은 세로 용지의 바탕쪽 그림을 원래 원점에 배치하고, 저장 표 조각의
  행 소속과 물리 높이를 유지한다. 첫 장의 그림과 43–45쪽 코드 표를 확대 대조했다.
  [2022·2024 한컴 인쇄본과 전쪽 TSV](../assets/issues7749_7753_20261121/measurements.json)
- #7753 정책연구 HWP/HWPX는 저장 그림 흐름, 캡션과 후속 본문 순서, 이어받은 표 행과
  각주 소속을 보존한다. 잘린 불투명 비트맵의 재표본화 결과에는 공통 RGB555 표면을
  사용한다. HWPX 11쪽의 예전 캡션 좌표 검사는 이미지·캡션의 셀 소속과 순서 검사로
  교체했다. 다섯 원본의 쪽수와 조판 회귀를 복원했다.
  [기존 검사와 복원 범위](../assets/issues7749_7753_20261121/restored-axes.json)

## 시각 검증

| 독립 한컴 Print | 쪽수 | Native Skia 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| 규제영향분석서 2020 | 31 | 98.70816% | 98.69136% |
| 규제영향분석서 2024 | 31 | 99.24550% | 99.24007% |
| ParameterSet 2022 | 74 | 100.00000% | 100.00000% |
| ParameterSet 2024 | 74 | 100.00000% | 100.00000% |
| 정책연구 HWP 2024 | 215 | 98.96557% | 98.86514% |
| 정책연구 HWPX 2024 | 215 | 99.00639% | 99.01378% |

두 렌더 경로에서 각각 640쪽을 96 DPI의 색 경계 보정 전 2px 실루엣으로 측정했다. Native 502쪽과
WASM 493쪽이 100.00000%였다. 정책연구 HWP의 13쪽은 보관 Print와 새 한컴 2024 Print 사이의
실루엣이 95.42735%다. 같은 한컴 2024 빌드의 HWP·HWPX 인쇄에서 잘린 그림과 마스크의 처리를
확인해 HWP 전쪽 비교에는 새 Print를 사용했다. HWP 175쪽 소제목은 Print보다 약 26.6px 위에 있으며
동일 Ubuntu devel과 후보의 해당 Native PNG 해시는 같다.
[직접 판독과 기준 선택](../assets/issues7749_7753_20261121/analysis.md) ·
[원본·기준 PDF SHA-256](../assets/issues7749_7753_20261121/print-references.json)

보호 정본 46조건과 4,046쪽은 같은 Ubuntu의 devel과 후보에서 raw와 PNG SHA-256이 모두 같다.
화학물질 고시 91쪽과 HWP 사양서 8쪽도 일치한다. 보관 정본보다 낮게 측정된 52쪽은 같은
Ubuntu에서 devel이 기록한 값과 같다. 편람 HWPX Native 383쪽은 최저 99.96504%로 통과했고,
공통 보호 8조건도 통과했다. [보호 목록 전쪽 비교](../assets/issues7749_7753_20261121/protected-comparison.tsv) ·
[편람과 공통 보호 검사](../assets/issues7749_7753_20261121/public-gates/public-gates.json)

## 실행 검증

최소 글꼴 전체 Nextest 13,854건과 새 집중 검사 5건이 통과했다. Native Skia 라이브러리 4,035건,
그림·직접 PDF 38건, Studio 2,822건과 workspace 빌드·lint도 통과했다.
[로컬 실행 영수증](../assets/issues7749_7753_20261121/local-validation.json)

고정 코드 head의 GitHub 워크플로 6개가 통과했다.
[CI](https://github.com/edwardkim/rhwp/actions/runs/44844490592) ·
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44844490644) ·
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44844490685) ·
[Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44844490652) ·
[Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44844490638) ·
[CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44844490503)

검토 기록을 포함한 head의 필수 검사 확인 후 `devel`에 병합한다.
