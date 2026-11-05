# PR #8546 리뷰 — TAC 도형이 소유한 선행 줄

## 최종 판정

**메인터너 보정 후 수용 가능**

완료한 실측과 남은 통합 공통 검증을 구분합니다. 원 PR의 직접 승인·닫기와 통합 PR 생성은 수행하지 않았습니다.
최종 수용은 WASM의 당시 base 대비 전쪽 대조와 전체 증적 감사를 완료한 뒤 판정합니다. PR 생성은 작업지시자 승인 대기입니다.

## 이전 완료 제품과 재사용 증거의 출처

| 항목 | 값 |
| --- | --- |
| 원 PR | [PR #8546](https://github.com/edwardkim/rhwp/pull/8546) |
| 원 head | `c08cb8030eff594f6f987832cc35d2bb8ae68943` |
| 현재 통합 적용 커밋 | `437a3f1507ad8c585ea3d59993531bc54f3fe433` |
| 이전 완료 제품 / 당시 base | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` / `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 전쪽 Native 측정 producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| fresh WASM 실제 빌드 | `7bb9e66fd006109ca9f7ee4c4d3f752c82e60fae` (위 제품 코드와 동등한 구현) |
| reviewer | `jangster77` 요청; 2026-11-04T17:44:08.556Z 고정한 원 head·Open·non-Draft 확인 |

기존 전쪽 TSV/PNG는 실제 측정 producer의 결과로 재사용합니다. 최신 소스의 재출력으로 표시하지 않습니다.
이전 a7 제품의 최소 글꼴 전체 Rust 12,561개 통과·Native Skia·Rust lint 및 fresh WASM 실제 실행과
새 base 기능 보존 집중 대조 14쪽을 완료했습니다. [제품 검증](assets/planet6897_20261103/intermediate-evidence-summary.md)과
[최신 리베이스 대조](assets/planet6897_20261103/rebase_scope_22947f6a90/README.md)를 연결합니다.

## 현재 전쪽 Native/fresh WASM 증적

| 원본 · 독립 Print | Native TSV · 전쪽 최저 | fresh WASM TSV · 전쪽 최저 |
| --- | --- | --- |
| [leading_shape_line-2020.hwp](../../samples/issue8543/leading_shape_line-2020.hwp) · [leading_shape_line-2020.pdf](../../pdf/issue8543/leading_shape_line-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading_shape_line-2020-ae054dc3-native.tsv) · 93.14000% (2쪽) | [TSV](assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading_shape_line-2020-ae054dc3-wasm.tsv) · 92.99000% (2쪽) |
| [leading_shape_line-2020.hwp](../../samples/issue8543/leading_shape_line-2020.hwp) · [leading_shape_line-2024.pdf](../../pdf/issue8543/leading_shape_line-2024.pdf) | [TSV](assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading_shape_line-2020-e4eb27fc-native.tsv) · 93.44000% (2쪽) | [TSV](assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading_shape_line-2020-e4eb27fc-wasm.tsv) · 93.09000% (2쪽) |

96 DPI, 색상 경계 보정 전 2px 내용 실루엣입니다. 입력·Print PDF는 기존 동일 파일을 재사용하고 전체 쪽수와 누락을 확인했습니다.

| 주장 쪽·범위 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| leading_shape_line-2020.hwp 2쪽 · 맨 앞 TAC 도형 줄 뒤 표 A·B 순서와 간격 | [review](assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-claim-p002-p002-native-review.png) / [overlay](assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-claim-p002-p002-native-overlay.png) | [review](assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-claim-p002-p002-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-claim-p002-p002-wasm-overlay.png) |
| leading_shape_line-2020.hwp 2쪽 · 2024 Print의 TAC 도형 줄 뒤 표 A·B 순서와 간격 | [review](assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-2024-p002-p002-native-review.png) / [overlay](assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-2024-p002-p002-native-overlay.png) | [review](assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-2024-p002-p002-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-2024-p002-p002-wasm-overlay.png) |

## 현재 경계의 직접 판독

2024 기준 2쪽의 선행 TAC 도형 → 표 A → 표 B → 뒤 본문 순서를 Native/WASM에서 다시 직접 대조했습니다. 표끼리 겹치지 않지만 표 B와 뒤 본문의 세로 위치·표 선에는 작은 차이가 남습니다. 선행 작성자가 별도 이슈 #8544로 구분한 범위와 현재 원 PR의 줄 소유 계약을 구분해 판단합니다.

[직접 판독 기록과 PNG 해시](assets/planet6897_20261103/direct-reading-current.md)에 실제 관측을 보존했습니다.

## 남은 통합 검증과 원격 조건

동일 환경의 당시 base 대비 WASM 80개 입력·406쪽 0.1%p 하락 게이트와 주장별 증적 최종 감사가 남았습니다.
[완료·대기 검증 기록](assets/planet6897_20261103/intermediate-evidence-summary.md)을 따릅니다.

아래 선행 분석은 당시 코드·증적의 기록입니다. 당시 남은 검증 문장은 현재 상태가 아니며 위 현재 기록과 구분합니다.

<details>
<summary>선행 구현 분석·독립 근거·당시 실행 기록</summary>

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

## 통합 코드의 전쪽 fresh WASM 검증

동일 제품 코드 `dc87f4fa2e`의 fresh WASM SHA-256은
`c7e166e1b087641e727d326dd5ff504653f96134808242f9d1fd83228bbae903`입니다.
2026-11-04, 같은 입력·PDF·96 DPI에서 두 Print 기준 모두 전체 2쪽이 일치하고
90% 미만은 0쪽입니다.

| 독립 Print 기준 | 1쪽 raw | 2쪽 raw | 전쪽 TSV와 주장 쪽 PNG |
| --- | ---: | ---: | --- |
| 2020 | 100.00% | 92.99% | [TSV](assets/planet6897_20261103/pr_8546/leading-shape-2020-wasm.tsv) · [review](assets/planet6897_20261103/pr_8546/leading-shape-2020-wasm-review-p002.png) · [overlay](assets/planet6897_20261103/pr_8546/leading-shape-2020-wasm-overlay-p002.png) |
| 2024 | 100.00% | 93.09% | [TSV](assets/planet6897_20261103/pr_8546/leading-shape-2024-wasm.tsv) · [review](assets/planet6897_20261103/pr_8546/leading-shape-2024-wasm-review-p002.png) · [overlay](assets/planet6897_20261103/pr_8546/leading-shape-2024-wasm-overlay-p002.png) |

2024의 선행 도형 줄, 표 A·B의 순서와 겹침 해소를 같은 쪽에서 직접 대조했습니다.
표 B와 뒤 글줄에는 원 PR이 별도 이슈로 설명한 작은 세로 간격 차이가 남아 있으며,
완전한 픽셀 일치로 판정하지 않습니다. 회귀는 실제 절대 좌표 대신 두 호환 모드의
표 소속·앞뒤 순서·겹침과 원본 바깥 여백 관계를 검사합니다.

## 원인과 메인터너 보정 범위

2024 모드의 `typeset_tac_table`은 표 앞 저장 줄을 빈 앵커로 보고 흐름에서
회수합니다. 이 입력의 선행 줄은 빈 앵커가 아니라 TAC 도형이 소유한 줄입니다.
진단 실행에서 문단 1의 표 앞줄 `anchor_reclaim=49.4px`, `tac_seg_idx=1`이
관측됐고, 이는 겹침 높이와 대응합니다. 흐름의 해당 분기는
`src/renderer/typeset.rs`의 `hangul2024_layout()` 단일 TAC 표 처리입니다.
`ad0d4b1277`에서 도형·그림이 실제로 소유한 저장 줄은 회수하지 않도록
공통 조건을 바로잡았습니다. 빈 앵커를 회수하는 기존 #5524 경로는 대조군으로 유지합니다.

두 호환 모드를 검사하도록 보완한 관계형 표 순서 회귀, 최소 글꼴 전체 Rust와
기준 devel 대비 페이지별 raw 대조를 완료한 뒤 최종 판정을 갱신합니다.

</details>

## 최신 리베이스 이후 통합 검증 상태

최신 base는 `f529dc92d4d172bf32390d7749743a45d9e41bfc`, 검증 시작 source는
`ba2ebfd34a4ebf402bce8e002254b672a09ef764`다. 원 PR 39개와 메인터너 보정을 담은
168개 커밋을 충돌 0건으로 리베이스했고 range-diff 전 행이 `=`로 대응했다.
[원 head와 현재 적용 SHA](assets/planet6897_20261103/integration-commits.md)를 구분한다.

최신 source의 최소 글꼴 Rust·Native Skia·lint·최적화 WASM 및 영향 경계 검증은 진행 중이다.
완료 판정이나 최종 승인으로 표시하지 않는다. 검증 중 소스 해시를 고정하고 문서 갱신과 구분한다.
충돌 없는 리베이스 뒤 기존 전쪽 비교를 반복하지 않는 작업지시자의 범위를 따른다.
새 base 기능은 같은 시스템 글꼴 환경의 실제 base/candidate 출력과 영향 경계로 대조한다.
새 base 대비 허용 하락은 0.1%p이며, 보호 PR #8389·#8397은 원 TSV의 소수 둘째 자리
표시값 하락을 허용하지 않는다. 기존 TSV·PNG의 producer는 원래 출처대로 유지한다.

직전 c183 제품에서는 [최소 글꼴 Rust 12,563 PASS와 필수 40개 실행](assets/planet6897_20261103/rust-validation-c183a1ac61.md),
[Native 80입력·406쪽](assets/planet6897_20261103/current-c183a1ac61/native-validation.md),
[fresh WASM 80입력·406쪽](assets/planet6897_20261103/current-c183a1ac61/wasm-validation.md),
[실제 브라우저/API 실행](assets/planet6897_20261103/runtime-validation-c183a1ac61.md)을 완료했다.
양쪽 전쪽 최저는 각각 92.45232%·92.87288%다. 이 결과를 최신 리베이스 제품의 재출력으로
취급하지 않는다. 보호 범위 Native는 17문서·1,770쪽·원 TSV 2,153행의 하락·누락 0행이며,
같은 c183 WASM 보호 범위의 남은 집계를 진행 중이다. [공식 증거 목록](assets/planet6897_20261103/README.md)과
[재실행 명령·입력·Print PDF·조판 세대·글꼴 계약](assets/planet6897_20261103/visual-reproduction.md)을 연결한다.

원 PR별 주장 쪽의 직접 판독·회귀 주석·최종 본문 감사를 마친 뒤 수용 판정을 확정한다.
통합 PR 생성·push·원격 review·comment·close는 아직 수행하지 않았다. PR 생성 전 승인을 기다린다.
