# PR #8546 리뷰 — TAC 도형이 소유한 선행 줄

## 최종 판정

**메인터너 보정 후 수용 가능**

완료한 실측과 남은 통합 공통 검증을 구분합니다. 통합 PR #8622 생성과 code head CI는 완료했습니다. 원 PR 직접 승인·닫기와 통합 병합은 아직 수행하지 않았습니다.
WASM의 당시 base 대비 전쪽 대조와 최신 필수 검증은 완료했으며 아래에 실제 증거를 연결합니다. 준비 완료 후 PR 생성·CI 모니터링은 작업지시자가 승인했습니다.


최신 통합 code head `acbc0bfeb5660133b0732ce77eae16bd50b29f9f`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41965861415)와 CodeQL·Render Diff·Adapter·Proptest가 모두 성공했습니다.
작업지시자가 CI 완료 뒤 문서 기록·병합·후속 처리를 승인했습니다. 원 head는 그대로 보존하고 보정된 [통합 PR #8622](https://github.com/edwardkim/rhwp/pull/8622)로 처리합니다.
이 문서는 code head CI 성공 뒤의 문서 전용 trailing 기록입니다. push 뒤 trailing head의 실제 fast-pass·필수 검사·최신 mergeability 확인을 병합 조건으로 둡니다.

## 이전 완료 제품과 재사용 증거의 출처

| 항목 | 값 |
| --- | --- |
| 원 PR | [PR #8546](https://github.com/edwardkim/rhwp/pull/8546) |
| 원 head | `c08cb8030eff594f6f987832cc35d2bb8ae68943` |
| 현재 통합 적용 커밋 | `012e2a08cfc0a9458e310538c61990a4e464a07b` |
| 이전 완료 제품 / 당시 base | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` / `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 전쪽 Native 측정 producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| fresh WASM 실제 빌드 | `7bb9e66fd006109ca9f7ee4c4d3f752c82e60fae` (위 제품 코드와 동등한 구현) |
| reviewer | `jangster77` 요청. `2026-11-05 03:16:02 UTC` 원격 재조회에서 원 head 불변·Open·non-Draft 확인 |

기존 전쪽 TSV/PNG는 실제 측정 producer의 결과로 재사용합니다. 최신 소스의 재출력으로 표시하지 않습니다.
이전 a7 제품의 최소 글꼴 전체 Rust 12,561개 통과·Native Skia·Rust lint 및 fresh WASM 실제 실행과
새 base 기능 보존 집중 대조 14쪽을 완료했습니다. [제품 검증](../assets/planet6897_20261103/intermediate-evidence-summary.md)과
[최신 리베이스 대조](../assets/planet6897_20261103/rebase_scope_22947f6a90/README.md)를 연결합니다.

## 현재 전쪽 Native/fresh WASM 증적

| 원본 · 독립 Print | Native TSV · 전쪽 최저 | fresh WASM TSV · 전쪽 최저 |
| --- | --- | --- |
| [leading_shape_line-2020.hwp](../../../samples/issue8543/leading_shape_line-2020.hwp) · [leading_shape_line-2020.pdf](../../../pdf/issue8543/leading_shape_line-2020.pdf) | [TSV](../assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading_shape_line-2020-ae054dc3-native.tsv) · 93.14000% (2쪽) | [TSV](../assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading_shape_line-2020-ae054dc3-wasm.tsv) · 92.99000% (2쪽) |
| [leading_shape_line-2020.hwp](../../../samples/issue8543/leading_shape_line-2020.hwp) · [leading_shape_line-2024.pdf](../../../pdf/issue8543/leading_shape_line-2024.pdf) | [TSV](../assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading_shape_line-2020-e4eb27fc-native.tsv) · 93.44000% (2쪽) | [TSV](../assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading_shape_line-2020-e4eb27fc-wasm.tsv) · 93.09000% (2쪽) |

96 DPI, 색상 경계 보정 전 2px 내용 실루엣입니다. 입력·Print PDF는 기존 동일 파일을 재사용하고 전체 쪽수와 누락을 확인했습니다.

| 주장 쪽·범위 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| leading_shape_line-2020.hwp 2쪽 · 맨 앞 TAC 도형 줄 뒤 표 A·B 순서와 간격 | [review](../assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-claim-p002-p002-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-claim-p002-p002-native-overlay.png) | [review](../assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-claim-p002-p002-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-claim-p002-p002-wasm-overlay.png) |
| leading_shape_line-2020.hwp 2쪽 · 2024 Print의 TAC 도형 줄 뒤 표 A·B 순서와 간격 | [review](../assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-2024-p002-p002-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-2024-p002-p002-native-overlay.png) | [review](../assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-2024-p002-p002-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8546/current-4b149b6cdf/pr8546-leading-shape-2024-p002-p002-wasm-overlay.png) |

## 현재 경계의 직접 판독

2024 기준 2쪽의 선행 TAC 도형 → 표 A → 표 B → 뒤 본문 순서를 Native/WASM에서 다시 직접 대조했습니다. 표끼리 겹치지 않지만 표 B와 뒤 본문의 세로 위치·표 선에는 작은 차이가 남습니다. 선행 작성자가 별도 이슈 #8544로 구분한 범위와 현재 원 PR의 줄 소유 계약을 구분해 판단합니다.

[직접 판독 기록과 PNG 해시](../assets/planet6897_20261103/direct-reading-current.md)에 실제 관측을 보존했습니다.

## 통합 검증 완료와 병합 전 조건

정확한 code head의 모든 필수 로컬 실행과 GitHub CI가 성공했습니다. 아래 최신 통합 기록과 최종 code CI 증거를 확인했습니다. 이번 후속 문서 head의 실제 fast-pass·required check·mergeability를 별도로 확인한 뒤 승인된 병합을 진행합니다.

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
| reviewer | `jangster77` 요청. `2026-11-05 03:16:02 UTC` 원격 재조회에서 원 head 불변·Open·non-Draft 확인 |

## 입력과 독립 기준

원 PR이 제공한 [한컴 2020 저장 HWP](../../../samples/issue8543/leading_shape_line-2020.hwp)를
[2020 Print PDF](../../../pdf/issue8543/leading_shape_line-2020.pdf) 및
[2024 Print PDF](../../../pdf/issue8543/leading_shape_line-2024.pdf)와 각각 대조합니다.
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

[2020 전쪽 TSV](../assets/planet6897_20261103/pr_8546/leading-shape-2020-native.tsv)와
[2024 전쪽 TSV](../assets/planet6897_20261103/pr_8546/leading-shape-2024-native.tsv),
[2020 2쪽 review](../assets/planet6897_20261103/pr_8546/leading-shape-2020-native-review-p002.png)·
[overlay](../assets/planet6897_20261103/pr_8546/leading-shape-2020-native-overlay-p002.png),
[2024 2쪽 review](../assets/planet6897_20261103/pr_8546/leading-shape-2024-native-review-p002.png)·
[overlay](../assets/planet6897_20261103/pr_8546/leading-shape-2024-native-overlay-p002.png)을
같은 배율로 판독했습니다.

## 통합 코드의 전쪽 fresh WASM 검증

동일 제품 코드 `dc87f4fa2e`의 fresh WASM SHA-256은
`c7e166e1b087641e727d326dd5ff504653f96134808242f9d1fd83228bbae903`입니다.
2026-11-04, 같은 입력·PDF·96 DPI에서 두 Print 기준 모두 전체 2쪽이 일치하고
90% 미만은 0쪽입니다.

| 독립 Print 기준 | 1쪽 raw | 2쪽 raw | 전쪽 TSV와 주장 쪽 PNG |
| --- | ---: | ---: | --- |
| 2020 | 100.00% | 92.99% | [TSV](../assets/planet6897_20261103/pr_8546/leading-shape-2020-wasm.tsv) · [review](../assets/planet6897_20261103/pr_8546/leading-shape-2020-wasm-review-p002.png) · [overlay](../assets/planet6897_20261103/pr_8546/leading-shape-2020-wasm-overlay-p002.png) |
| 2024 | 100.00% | 93.09% | [TSV](../assets/planet6897_20261103/pr_8546/leading-shape-2024-wasm.tsv) · [review](../assets/planet6897_20261103/pr_8546/leading-shape-2024-wasm-review-p002.png) · [overlay](../assets/planet6897_20261103/pr_8546/leading-shape-2024-wasm-overlay-p002.png) |

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

## 최신 통합 검증 상태

현재 base는 `d4a72efd7c91d1e25092ed30b9d44e43cbebb9f0`입니다. 직전 문서 갱신 뒤 PR #8618의 이슈 라벨링 workflow·계약 검사·운영 문서 3개가 추가됐습니다.
184개 패치가 충돌 0건·range-diff 모두 `=`로 리베이스됐습니다. 제품·테스트·fixture·검증 도구의 내용은
리베이스 전과 같으므로 완료 증거를 재사용합니다. 실제 측정 source는
`ba2ebfd34a4ebf402bce8e002254b672a09ef764`이며 문서 commit을 새 렌더 제품으로 표시하지 않습니다.
[현재 적용 SHA 39개](../assets/planet6897_20261103/integration-commits.md)를 확인합니다.

- [최소 글꼴 환경의 필수 실행 41개](../assets/planet6897_20261103/rebase8609/rust-regression-validation.md)는 모두 exit 0입니다.
  전체 nextest 12,583 PASS·45 SKIP, Native Skia lib 4,040 PASS·12 IGNORE,
  CI Native 모듈 21개와 doctest 8 PASS·3 IGNORE를 포함합니다.
- [최신 Native 영향 10입력·43쪽](../assets/planet6897_20261103/rebase8609/native-own-validation.md)과
  [fresh WASM 같은 43쪽](../assets/planet6897_20261103/rebase8609/wasm-own-validation.md)은 전쪽 90% 이상,
  직전 후보 대비 정밀 raw 0.1%p 초과 하락·누락·쪽수 불일치 0건입니다.
- [최신 필수 편람 Native 383쪽](../assets/planet6897_20261103/rebase8609/handbook-native-validation.md)은
  최저 표시 raw 99.96%, 기준 표시값 하락·fallback 0건·exit 0입니다.
- [최신 실제 API·브라우저](../assets/planet6897_20261103/rebase8609/runtime-validation.md)는 실제
  271 assertion·17 Worker·12쪽 렌더/저장과 가져오기·편집 산출물 대조를 완료했습니다.
- 기존 c183의 [Native 80입력·406쪽](../assets/planet6897_20261103/current-c183a1ac61/native-validation.md)과
  [fresh WASM 전406쪽](../assets/planet6897_20261103/current-c183a1ac61/wasm-validation.md)은 양쪽 90% 이상입니다.
  충돌 없는 리베이스 뒤 전쪽 측정을 반복하지 않는 지시에 따라 원래 producer를 명시해 재사용합니다.
- 보호 PR #8389·#8397의 [c183 Native](../assets/planet6897_20261103/current-c183a1ac61/pr8389_8397/native-validation.md)와
  [c183 fresh WASM](../assets/planet6897_20261103/current-c183a1ac61/pr8389_8397/wasm-validation.md)은
  각 17문서·1,770쪽·원 TSV 2,153행에서 표시값 하락·누락 0행입니다.

원 PR별 주장에 대응하는 입력·쪽의 대표 review/overlay는 아래 원래 출처를 유지합니다.
최신 영향 입력의 그림은 최신 source로 별도 표시하며, 다른 문서의 대표 그림으로 대체하지 않습니다.
[직접 판독 원장](../assets/planet6897_20261103/direct-reading-current.md)과
[입력/PDF·compat·글꼴·명령](../assets/planet6897_20261103/visual-reproduction.md)을 연결합니다.
회귀 측정 주석 갱신은 실제 assertion·helper 본문과 구분했고 주석만 바뀐 전체 검사는 반복하지 않았습니다.

[새 base Native 대조](../assets/planet6897_20261103/rebase8609/native-base-preservation.md)와
[fresh WASM 대조](../assets/planet6897_20261103/rebase8609/wasm-base-preservation.md)는 각각 14입력·41쪽을 완료했습니다.
양쪽 모두 raw 하락 자체가 0쪽이며 90% 미만·누락·쪽수 차이·제품 해시 오류 0건입니다.
문서-only 리베이스의 [내용 동일성](../assets/planet6897_20261103/latest-rebase.md)을 확인해 완료 결과를 재사용했습니다.
추가 전체 회귀는 필요하지 않습니다. 제출 전 필수 검증은 완료했고 통합 PR #8622의 정확한 code head CI가 모두 성공했으며 이번 trailing head는 별도로 확인합니다.
원 PR 39개는 원격 재조회에서 head 불변·Open·non-Draft·reviewer 요청을 확인했습니다.
통합 PR 생성 전 본인 계정의 우선 처리할 열린 PR은 0건이었습니다. 원격 push·통합 PR 생성과 code head CI는 완료했습니다. 이 후속 문서 작성 시점에는 병합·원 PR 댓글·닫기를 아직 수행하지 않았습니다.
작업지시자는 준비 완료 후 PR 생성·CI 모니터링을 승인했습니다. 문서-only trailing commit의 병합·링크·기존 오늘할일 보존을 확인한 뒤 push하고 새 head CI를 확인합니다.

## Merge 후 contributor PR comment 계획

[통합 PR #8622](https://github.com/edwardkim/rhwp/pull/8622)의 실제 merge SHA와 devel 반영을 확인한 뒤 원 PR을 통합 완료로 정리합니다.
[이슈별 처리 범위](../assets/planet6897_20261103/source-followup-scope.md)에 따라 해결 이슈와 남은 별도 범위를 구분합니다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고, 위 전쪽 TSV와 같은 원본·PDF·주장 쪽의 review 및 별도 overlay를 사용합니다.
실제 그림은 직접 판독 원장의 producer·페이지·해시를 유지하며 다른 문서의 대표 그림으로 대신하지 않습니다.
원 PR의 기여와 메인터너 보정 이유·검증 결과·남은 차이를 한국어 존댓말로 설명합니다.
이미지 URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/...` 형식으로 고정하고 게시 전 응답의 이미지 형식·SHA-256을 원본과 대조합니다.
기존 같은 검토 댓글이 있으면 수정하고 없을 때만 처음 게시합니다. 게시 뒤 API 본문과 이미지 표시를 재확인하며 contributor fork branch는 삭제하지 않습니다.

[최종 code CI와 오탐 처리](../assets/planet6897_20261103/code-head-ci-completion.md)를 함께 기록했습니다. 현재 기록 시점에는 merge·원 PR close/comment를 아직 실행하지 않았으며, 실제 원격 완료 사실은 원 PR 타임라인과 통합 PR에 남깁니다.

[후속 댓글의 실제 원본·쪽·Native/WASM PNG 대응표](../assets/planet6897_20261103/source-comment-evidence.md#pr-8546)를 고정했습니다. 이미지 수·출력 경로·해시를 게시할 댓글과 대조합니다.
