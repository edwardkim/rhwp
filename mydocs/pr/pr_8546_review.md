# PR #8546 리뷰 — TAC 도형이 소유한 선행 줄

## 최종 판정

**머지 보류 — 메인터너 보정으로 2024 Print 대조 2쪽이 53.45%에서 93.44%로
개선됐으나, 최종 head의 fresh WASM·회귀·기준 devel 대조가 진행 중입니다.**
2020 경로의 93.14%는 유지됐습니다. 완료 판정은 남은 검증을 마친 뒤 갱신합니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8546](https://github.com/edwardkim/rhwp/pull/8546), `planet6897`, `devel` 대상 |
| 원 head | `c08cb8030eff594f6f987832cc35d2bb8ae68943` |
| 통합 체리픽 | `e8bc925845` — #8531·#8522·#8473의 선행 커밋은 기존 통합 브랜치에 이미 포함 |
| 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 관련 이슈 | [#8543](https://github.com/edwardkim/rhwp/issues/8543) 해결 대상. [#8544](https://github.com/edwardkim/rhwp/issues/8544)의 표 뒤 간격 문제는 별도 범위. |
| reviewer | `jangster77` 요청 및 GitHub API 재확인 완료 |

## 입력과 독립 기준

원 PR이 제공한 [한컴 2020 저장 HWP](../../samples/issue8543/leading_shape_line-2020.hwp)를
[2020 Print PDF](../../pdf/issue8543/leading_shape_line-2020.pdf) 및
[2024 Print PDF](../../pdf/issue8543/leading_shape_line-2024.pdf)와 각각 대조합니다.
두 PDF 모두 2쪽입니다. HWP SHA-256은
`6ffbd93970a8e1b4d8f18fa304c8dc1256e291bf6c708cb82b27634197d63e86`,
2020 PDF는 `44c7430ef2acc229fc287d23473f41a64b0a6f1db86c490c3a10ce415d326671`,
2024 PDF는 `c18f02ec19ff26b9c30e6e13180ebc0e7c96eb88596ffc40ddbabe8e3254183c`입니다.

## 통합 코드의 전쪽 Native 검증

보정 전 `e8bc925845`의 Native Skia 실행 파일 SHA-256은
`be6d76635717b47b109fec1ccfe6c66ab7a261752334cf4c1ea7dd741f34088d`입니다.
보정과 #8547 체리픽을 포함한 `dc87f4fa2e`의 실행 파일 SHA-256은
`24c7e929b04c14adc46fbcc80b384e74fffd5524bdd2484068ebe6e974952337`입니다.
두 출력을 같은 입력·Print PDF, 96 DPI, 색상 경계 보정 전 2px raw 실루엣으로 측정했습니다.

| 출력 | 1쪽 | 2쪽 | 주장 쪽의 직접 판독 |
| --- | ---: | ---: | --- |
| 보정 전 한컴 2020 Print, `--compat 2022` | 100.00% | 93.14% | 표 A·B와 뒤 글줄 순서가 보존됩니다. |
| 보정 전 한컴 2024 Print, `--compat 2024` | 100.00% | **53.45%** | 표 A 아래쪽과 표 B 위쪽이 겹쳤습니다. |
| 현재 한컴 2020 Print, `--compat 2022` | 100.00% | **93.14%** | 표 A·B와 뒤 글줄 순서가 보존되고 겹치지 않습니다. |
| 현재 한컴 2024 Print, `--compat 2024` | 100.00% | **93.44%** | 표 A·B가 겹치지 않고 Print의 내용·순서와 대응합니다. 표 선과 글자에 작은 위치 차이가 남습니다. |

[2020 전쪽 TSV](assets/planet6897_20261103/pr_8546/leading-shape-2020-native.tsv)와
[2024 전쪽 TSV](assets/planet6897_20261103/pr_8546/leading-shape-2024-native.tsv),
[2020 2쪽 review](assets/planet6897_20261103/pr_8546/leading-shape-2020-native-review-p002.png)·
[overlay](assets/planet6897_20261103/pr_8546/leading-shape-2020-native-overlay-p002.png),
[2024 2쪽 review](assets/planet6897_20261103/pr_8546/leading-shape-2024-native-review-p002.png)·
[overlay](assets/planet6897_20261103/pr_8546/leading-shape-2024-native-overlay-p002.png)을
같은 배율로 판독했습니다.

## 원인과 메인터너 보정 범위

2024 모드의 `typeset_tac_table`은 표 앞 저장 줄을 빈 앵커로 보고 흐름에서
회수합니다. 이 입력의 선행 줄은 빈 앵커가 아니라 TAC 도형이 소유한 줄입니다.
진단 실행에서 문단 1의 표 앞줄 `anchor_reclaim=49.4px`, `tac_seg_idx=1`이
관측됐고, 이는 겹침 높이와 대응합니다. 흐름의 해당 분기는
`src/renderer/typeset.rs`의 `hangul2024_layout()` 단일 TAC 표 처리입니다.
`ad0d4b1277`에서 도형·그림이 실제로 소유한 저장 줄은 회수하지 않도록
공통 조건을 바로잡았습니다. 빈 앵커를 회수하는 기존 #5524 경로는 대조군으로 유지합니다.

새 head에서 2020·2024 Print 전쪽 fresh WASM, 관계형 표 순서 회귀,
최소 글꼴 집중·전체 Rust와 기준 devel 대비 페이지별 raw를 재검증한 뒤 판정을 갱신합니다.
