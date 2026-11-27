# PR #9467 검토

[planet6897 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9467)

## 검토 버전

- 원 PR head `e1b1b7d51259524c9879f41cd1bbfd0c033665f2`
- 기준 devel `77e4c3d1369623818e8cf3213e3fdc21d8e6502e`
- 통합 코드 후보 `b25bbad4f477a5f6c9bfc212f58b6f03e2a090dc`
- 검토 브랜치 `review/integration-8678-chart-layout-native-editing`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `4277bd4ed85e80157cc87c388d19773eebcaf0dd` | `9277ba302694969674536ca090dec2051f441631` |
| `88150007fc52c0c8bf938c5319fba0206a39ad4d` | `112dafbd7e117d608f6e9fbdaa4c5965fcd5b786` |
| `ad5db4fb45c17ebc8fbdd5e851369c304779e016` | `63560baac482ab1c8c111d10d0cf35e1381e9be6` |
| `196bea2b12fb8c4bcf8d1982f43117553559a598` | `ae21ef70370b2c3af189a9636b4a8b70cd3ae5e8` |
| `9a81140ebfdcbb5e2e46008c85b14d7ab4b2fdaf` | `e3339e1c2b4df2bf1d2bb7e0239dbfc869af81db` |
| `e1b1b7d51259524c9879f41cd1bbfd0c033665f2` | `5c31ba36ba40f8282474d63ba50803d99df921be` |

## 검토 근거

#9441의 라틴 글리프 폭 변경을 선행 반영했습니다. 저장 HWP5 줄의 소프트 하이픈은 LINE_SEG bit 19가 설정된 경우에 표시합니다. HWP3 가져오기와 재조판에서는 실제로 하이픈 위치에서 줄이 끊긴 경우에 bit 19를 설정합니다.

264쪽 HWP3 입력을 HWP5로 저장해 8,189개 줄 세그먼트를 대조했습니다. bit 19가 설정된 줄은 0개에서 111개로 바뀌었고 그 외 세그먼트 값은 같았습니다. 11쪽 공개 fixture의 9쪽은 한컴 2020 Print에 대해 Native와 fresh WASM 모두 100.00%, 두 경로의 전쪽 최저값은 99.80%였습니다.

[원 PR의 비교 TSV](../assets/issue_9426/hyphen/) · [줄 끝 표지 회귀](../../../tests/cases/issue_9426_soft_hyphen_line_end_mark.rs)

## 적용 전후 출력 대조

11쪽 공개 HWP5 표본을 같은 한컴 2020 Print와 같은 글꼴로 대조했습니다. #9467 적용 전후 PNG가 바뀐 쪽은 9쪽 한 곳이며, Native 99.47%에서 100.00%로 개선됐습니다. 3·8쪽을 포함한 나머지 10쪽의 PNG 해시와 값은 같습니다.

264쪽 보조 자료는 저장 경로를 확인했습니다. RHWP로 먼저 변환한 HWP5의 인쇄본에서 낮게 나온 180·260·261쪽을 원본 HWP3의 한컴 2020 직접 인쇄본과 다시 비교했습니다. 세 쪽의 적용 전후 Native PNG 해시는 같았으며 각 값은 99.92%·99.59%·98.75%였습니다. 직접 인쇄 PDF SHA-256은 `c72fd9d2df95657496662cde20b757d0f536f28b398048d2eb25ab8355125ecc`입니다.

264쪽 원본의 참고문헌 260쪽을 확대 대조했습니다. 원 HWP3 직접 Print와 한컴 2020이 직접 재저장한 HWP의 Print에서는 `Delegationbe-`와 `Bundesver-`의 하이픈이 보였습니다. 두 기준에서 해당 하이픈의 유무를 PDF 텍스트로도 확인했습니다. 과거 RHWP 변환 HWP5를 한컴에서 인쇄한 결과에는 두 하이픈이 빠져 있었습니다. #9467을 적용한 원 HWP3 출력의 `Delegationbe-`와 이어지는 줄은 독립 HWP3 Print와 같은 구간에 배치됐습니다.

원본 PR head를 별도로 빌드해 최신 통합 코드 후보와 같은 Mac, HWP, 한컴 2020 Print, 글꼴 해시 및 비교 조건에서 공개 11쪽을 다시 출력했습니다. Native 11쪽과 WASM 11쪽의 출력 PNG와 기준 PNG 해시가 각각 전부 일치했습니다. 이 조건에서 통합으로 인한 출력 하락은 0쪽입니다. [Native·WASM 쪽별 PNG 대조](../assets/integration-8678-20261128/source-9467-native-wasm-raster-comparison.tsv)

작성자가 사용한 한컴 2024 기준 PDF 원본은 저장소와 이 검토 환경에서 확보하지 못했습니다. 원 PR의 2024 TSV는 보존했습니다. 동일 입력·글꼴로 얻은 두 head의 PNG 일치는 참조 PDF를 변경해도 유지되는 출력 동일성입니다.

## 실제 문서 참고문헌 256~264쪽

최신 후보를 한컴 2020 11.0.0.9172의 두 독립 Print와 대조했습니다. 원 HWP3 직접 인쇄본 기준 최저 일치율은 Native 98.60%, fresh WASM 98.62%였습니다. 같은 원본을 한컴이 HWP로 재저장한 뒤 인쇄한 기준에서는 Native 98.73%, fresh WASM 98.74%였습니다. 두 입력의 9쪽을 두 경로에서 측정한 36행이 모두 90%를 넘었습니다. [참고문헌 쪽별 결과](../assets/integration-8678-20261128/issue-9426-bibliography-page-comparison.tsv)

## Merge 후 contributor PR comment 계획

저장 LINE_SEG bit 19의 실제 경계와 9쪽의 하이픈을 확인한 결과를 안내합니다. 공개 11쪽의 9쪽 Native·fresh WASM review·overlay를 [하이픈 증적](../assets/issue_9426/hyphen/)과 함께 연결합니다. HWP3 원본 한컴 직접 인쇄 대조도 출처와 구간을 적습니다.

통합 PR의 병합 SHA와 최신 head의 실제 CI를 고정합니다. 연결할 증적은 병합 SHA의 raw PNG와 대조하고, 한국어 존댓말로 원 PR에 검토와 반영 결과를 남깁니다.
