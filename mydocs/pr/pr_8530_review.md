# PR #8530 리뷰 — 저장 첫 조각의 정확한 컷과 원본 프레임 보존

## 최종 판정

**메인터너 보정 후 수용 가능 — 최종 공통 검증 대기**

원 PR의 정확한 컷 수용을 유지하면서 메인터너가 첫 프레임의 높이·중첩 셀 경계를
보정했습니다. 새 base 후보의 Native/fresh WASM 전3쪽과 주장 쪽의 직접 판독을
완료했고 새 회귀도 최소 글꼴 환경에서 통과했습니다. 최종 통합 검증을 마친 뒤 판정을 확정합니다.
PR 생성은 작업지시자 승인 대기이며 원 PR에 원격 승인·닫기를 실행하지 않았습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [PR #8530](https://github.com/edwardkim/rhwp/pull/8530), `planet6897` → `devel` |
| 원 head | `7cde7a2d44142e28817db46b12fedff9b7360a15` |
| 통합 적용 | `1466017424`, `2de9f20e7b` |
| 메인터너 구현 보정 | `13c20a5d79`, `ffcbdebd96`, `b52a15a4f5` |
| 고정 devel | `782ab3e2d06380d7d36660f07d76ef4c8c7c0c0a` |
| 실측 제품 코드 | `dd0ac546e3389f1703e67cb86f1c5bdf615ff0c9` |
| reviewer | `jangster77` 요청. 2026-11-04 원 head·Open·non-Draft 확인 |
| 관련 이슈 | [이슈 #8529](https://github.com/edwardkim/rhwp/issues/8529) |
| 기본·보조 경로 | `maintainer_general`; `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |

## 원 주장과 메인터너 보정

원 PR은 첫 조각을 정확히 채우는 `[2, 3]` 컷이 HU→px 부동소수 잡음 때문에 거부되어
문단이 다음 쪽으로 밀리는 문제를 고칩니다. 실제 초과를 수용하도록 예산을 넓히지 않고,
같은 컷의 동등 경계를 판단하는 계약을 유지합니다.

메인터너는 같은 입력의 마지막 행 과대 높이와 중첩 표 우측 넘침도 확인했습니다.
최종 수용 행 계획을 저장 원점에 게시하고, 부분 표의 온전한 셀에도 기존 부모 clip을
적용했습니다. 첫 프레임의 잔여 높이는 스캔에서 올바르게 확정됐으나, 이후 일반 쪽 하단의
빈 띠를 이미 닫힌 첫 프레임에 다시 배정해 높이를 늘리는 실제 소비 경로가 남아 있었습니다.

독립적으로 인증한 원본 프레임과 실제 수용·확정 높이가 모두 닫힌 경우, 빈 띠 계산도
그 프레임을 소비하도록 보정했습니다. 문서 ID나 관측 오차를 상수로 넣지 않습니다.
인증되지 않은 프레임·다른 컷·이어받기 조각에는 이 제한을 적용하지 않습니다.
생산·측정·배치와 수정 전후 진단은 [분석 기록](../working/task_m100_8529_source_frame_row_origin.md)에 있습니다.

## 검증 입력과 전쪽 결과

원본은 [rowbreak_exact_fit.hwpx](../../samples/issue8529/rowbreak_exact_fit.hwpx),
독립 기준은 [한컴 2022 Print PDF](../../pdf/issue8529_rowbreak_exact_fit-2020.pdf)입니다.
PDF 파일명의 `2020`과 실제 생성 제품을 구분하며 기존 유효한 Print를 재사용했습니다.

| 자료 | SHA-256 |
| --- | --- |
| HWPX | `6f29966a3e061fb1763b60d8b4bc8c4b0898a53b66989ab13449ecc147d05bf7` |
| Print PDF | `c04a7cc89855a61a83c978d59686026adc68c672000e38deb9687397ec78a81f` |
| Native Skia 실행 파일 | `1c429f56010fd3450b0b3e815dec5092000cb3309156b497532140199dd1ac6c` |
| 최적화 fresh WASM | `ee4043521f602cf7df7815851d4067fc7825e6db82a0c8c6a6b5cfc8363b0e7d` |

96 DPI·Print 2022·전체 시스템 글꼴 공급, 색상 경계 보정 전 2px 실루엣입니다.
원본·PDF의 독립 전체 쪽수는 3쪽이고 양쪽 모두 같은 3쪽을 출력했습니다.
원본·PDF는 현재 Git 내용과도 바이트 단위로 일치합니다.

| 쪽 | Native raw | fresh WASM raw | 확인할 내용 |
| --- | --- | --- | --- |
| 1 | 99.94% | 99.96% | 첫 프레임·끝 행·중첩 셀 경계와 둘째 문단 |
| 2 | 100.00% | 100.00% | 셋째 문단의 이어받기와 일몰 머리 행 |
| 3 | 100.00% | 100.00% | 미설정 행부터 마지막 계획까지의 내용 보존 |

전쪽 원본 TSV: [Native](assets/planet6897_20261103/pr_8530/pr8530-rowbreak_exact_fit-b431a6d3-native.tsv) ·
[fresh WASM](assets/planet6897_20261103/pr_8530/pr8530-rowbreak_exact_fit-b431a6d3-wasm.tsv).
수치는 TSV의 표시값이며 1쪽 원시 raw는 Native 99.93715%, fresh WASM 99.95896%입니다.
이전 제품의 1쪽 Native 90.68%·WASM 90.59%를 새 제품의 결과로 재사용하지 않습니다.

## 원 주장 쪽의 시각 증적과 직접 판독

| 같은 입력의 쪽 | Native review · overlay | fresh WASM review · overlay |
| --- | --- | --- |
| 1쪽: 수용 컷·첫 물리 프레임 | [review](assets/planet6897_20261103/pr_8530/pr8530-claim-p001-rowbreak_exact_fit-native-review.png) · [overlay](assets/planet6897_20261103/pr_8530/pr8530-claim-p001-rowbreak_exact_fit-native-overlay.png) | [review](assets/planet6897_20261103/pr_8530/pr8530-claim-p001-rowbreak_exact_fit-wasm-review.png) · [overlay](assets/planet6897_20261103/pr_8530/pr8530-claim-p001-rowbreak_exact_fit-wasm-overlay.png) |
| 2쪽: 다음 조각의 시작 | [review](assets/planet6897_20261103/pr_8530/pr8530-claim-p002-rowbreak_exact_fit-native-review.png) · [overlay](assets/planet6897_20261103/pr_8530/pr8530-claim-p002-rowbreak_exact_fit-native-overlay.png) | [review](assets/planet6897_20261103/pr_8530/pr8530-claim-p002-rowbreak_exact_fit-wasm-review.png) · [overlay](assets/planet6897_20261103/pr_8530/pr8530-claim-p002-rowbreak_exact_fit-wasm-overlay.png) |

독립 PDF와 같은 배율에서 마지막 행의 높이·가운데 정렬, 중첩 표의 우측 경계,
줄 시작과 이어받기를 직접 확인했습니다. 원본의 첫 물리 프레임 68,454 HU를 닫은 뒤에
용지 하단의 여유를 다시 더하지 않는 것을 확인했습니다.

## 회귀와 최종 공통 검증

[기존 컷·내용 소유 회귀](../../tests/cases/issue_8529_rowbreak_exact_fit.rs)를 유지합니다.
[새 프레임 회귀](../../tests/cases/issue_8529_accepted_opening_frame.rs)는 원본 선언 높이와
실제 표·셀 포함, 다음 쪽의 내용 소유를 검사합니다. 전3쪽의 양쪽 90% 이상을 먼저 확인했고,
양쪽 98% 이상인 1쪽에서만 ±2 CSSpx를 사용합니다. 설치된 글꼴에 의존하지 않습니다.
새 회귀는 최소 글꼴 5개·Native Skia 기능 환경에서 1건 통과했습니다(exit 0).
수정 전·후의 실제 렌더 트리도 원본 XML의 프레임 높이와 대조해 각각 조건 위반·통과를 확인했습니다.
새 검사는 기존 컷·내용 검사를 바꾸지 않고 별도 이름으로 추가했습니다.

최종 통합 후보의 전체 Rust·Skia·lint, 고정 devel 대비 0.1%p 하락 대조,
PR #8389·#8397의 원 TSV 표시값 무하락 검증을 마친 뒤 최종 판정을 갱신합니다.

## Merge 후 원 PR 후속 처리

통합 PR의 merge SHA·CI와 같은 입력·주장 쪽의 TSV·review/overlay를 원 PR에 연결합니다.
댓글 게시와 원 PR 닫기는 최종 통합 검증과 merge 이후 승인된 후속 절차를 따릅니다.

<!-- planet6897-20261103-generated-review -->
