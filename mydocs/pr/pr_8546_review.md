# PR #8546 리뷰 — TAC 도형이 소유한 선행 줄

## 최종 판정

**머지 보류 — 한컴 2024 Print 대조 2쪽이 53.45%로 미달합니다.** 원 PR의
2020 경로 개선은 확인했으며, 통합 head에서 2024 조판 경로를 메인터너 보정한 뒤
Native/fresh WASM 전쪽과 기존 #5524 대조군을 다시 검사합니다.

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

## 현재 통합 코드의 전쪽 검증

`e8bc925845`의 Native Skia 실행 파일 SHA-256
`be6d76635717b47b109fec1ccfe6c66ab7a261752334cf4c1ea7dd741f34088d`에서
96 DPI, 2px raw 실루엣으로 측정했습니다. fresh WASM과 기준 devel 대조는 보정 뒤
새 head에서 확인합니다.

| 출력 | 1쪽 | 2쪽 | 주장 쪽의 직접 판독 |
| --- | ---: | ---: | --- |
| 한컴 2020 Print, `--compat 2022` | 100.00% | 93.14% | 표 A·B와 뒤 글줄 순서가 보존되고 겹치지 않습니다. 표 B가 약 2px 위인 차이는 #8544와 연결됩니다. |
| 한컴 2024 Print, `--compat 2024` | 100.00% | **53.45%** | 표 A 아래쪽과 표 B 위쪽이 겹치며 Print와 다릅니다. |

[2020 전쪽 TSV](../../output/pr-review/planet6897-20261103/visual/e8bc925845/native/pr8546-leading_shape_line-2020-ae054dc3/silhouette.tsv)와
[2024 전쪽 TSV](../../output/pr-review/planet6897-20261103/visual/e8bc925845/native/pr8546-2024-diagnostic/silhouette.tsv)는
현재 로컬 진단 경로입니다. PR 제출 전에 정식 자산 경로로 옮기고 최종 head의 TSV로 교체합니다.
[2020 2쪽 review](../../output/pr-review/planet6897-20261103/claim-pages/native/pr8546-final-2020/review/review_002.png)·
[overlay](../../output/pr-review/planet6897-20261103/claim-pages/native/pr8546-final-2020/overlay/overlay_002.png),
[2024 2쪽 review](../../output/pr-review/planet6897-20261103/claim-pages/native/pr8546-final-2024/review/review_002.png)·
[overlay](../../output/pr-review/planet6897-20261103/claim-pages/native/pr8546-final-2024/overlay/overlay_002.png)을
같은 배율로 판독했습니다.

## 원인과 메인터너 보정 범위

2024 모드의 `typeset_tac_table`은 표 앞 저장 줄을 빈 앵커로 보고 흐름에서
회수합니다. 이 입력의 선행 줄은 빈 앵커가 아니라 TAC 도형이 소유한 줄입니다.
진단 실행에서 문단 1의 표 앞줄 `anchor_reclaim=49.4px`, `tac_seg_idx=1`이
관측됐고, 이는 겹침 높이와 대응합니다. 흐름의 해당 분기는
`src/renderer/typeset.rs`의 `hangul2024_layout()` 단일 TAC 표 처리입니다.
도형·그림이 실제로 소유한 저장 줄은 회수하지 않도록 공통 조건을 바로잡고,
빈 앵커를 회수하는 기존 #5524 경로는 대조군으로 유지합니다.

새 head에서 2020·2024 Print 전쪽 Native/fresh WASM, 관계형 표 순서 회귀,
최소 글꼴 집중·전체 Rust와 기준 devel 대비 페이지별 raw를 재검증한 뒤
판정 문구를 갱신합니다.
