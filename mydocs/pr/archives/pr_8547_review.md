# PR #8547 리뷰 — 저장 쪽 경계에서 걸침 묶음 이어받기

## 최종 판정

**메인터너 보정 후 수용 가능**

완료한 실측과 남은 통합 공통 검증을 구분합니다. 통합 PR #8622 생성과 code head CI는 완료했습니다. 원 PR 직접 승인·닫기와 통합 병합은 아직 수행하지 않았습니다.
당시 base 대비 전쪽 대조는 완료했습니다. 새 보정의 필수 검증도 완료했고 아래에 실제 증거를 연결합니다. 준비 완료 후 PR 생성·CI 모니터링은 작업지시자가 승인했습니다.


최신 통합 code head `acbc0bfeb5660133b0732ce77eae16bd50b29f9f`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41965861415)와 CodeQL·Render Diff·Adapter·Proptest가 모두 성공했습니다.
작업지시자가 CI 완료 뒤 문서 기록·병합·후속 처리를 승인했습니다. 원 head는 그대로 보존하고 보정된 [통합 PR #8622](https://github.com/edwardkim/rhwp/pull/8622)로 처리합니다.
이 문서는 code head CI 성공 뒤의 문서 전용 trailing 기록입니다. push 뒤 trailing head의 실제 fast-pass·필수 검사·최신 mergeability 확인을 병합 조건으로 둡니다.

## 추가 보정의 Native·fresh WASM 결과 — 필수 회귀 검증 완료

공통 물리 프레임 전달 보정 `e61ed1ca7329d3dacffc9c7fc76e602f30f43f66`의
Native·fresh WASM을 96 DPI·같은 원본·같은 독립 Print PDF로 전 6쪽 재측정했습니다.
3쪽 raw는 Native 97.95% → 100.00%, WASM 97.98% → 100.00%로 개선됐고 나머지
쪽의 표시값은 유지됐습니다. 두 경로 모두 Print와 6쪽이 일치합니다.

| 새 보정 근거 | Native | fresh WASM |
| --- | --- | --- |
| 전 6쪽 TSV | [TSV](../assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/native.tsv) · 최저 99.85% | [TSV](../assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/wasm.tsv) · 최저 99.87% |
| 3쪽 · 위생 앞 조각의 아래 외곽 | [review](../assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/native-p003-review.png) / [overlay](../assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/native-p003-overlay.png) | [review](../assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/wasm-p003-review.png) / [overlay](../assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/wasm-p003-overlay.png) |
| 4쪽 · 영양사 이어받기와 뒤 행 | [review](../assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/native-p004-review.png) / [overlay](../assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/native-p004-overlay.png) | [review](../assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/wasm-p004-review.png) / [overlay](../assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/wasm-p004-overlay.png) |
| 전쪽 무하락 대조 | [Native 비교](../assets/planet6897_20261103/intermediate-evidence-summary.md) | [WASM 비교](../assets/planet6897_20261103/intermediate-evidence-summary.md) |
| 실제 제품과 소스 연결 | [제품·소스](../assets/planet6897_20261103/intermediate-evidence-summary.md) | [빌드·패키지·증적 해시](../assets/planet6897_20261103/intermediate-evidence-summary.md) |

[앞·뒤 쪽수·내용 컷 대조](../assets/planet6897_20261103/intermediate-evidence-summary.md)에서
6쪽과 내용 컷은 그대로이며 실제 물리 점유만 보정됐습니다. Native·WASM의 3·4쪽 review와
별도 overlay를 직접 판독해 3쪽 표 하단, 4쪽 영양사 이어받기와 뒤 행의 소유·순서를
확인했습니다. 100.00%는 2px 관용 실루엣 표시값이며 엄격 RGB 완전 일치를 뜻하지 않습니다.

새 보정의 필수 회귀·정상 대조군 검증은 남아 있습니다. 아래 보정 전 제품의 전체 검증
통과를 새 보정의 통과로 표시하지 않으며, 이를 완료한 뒤 최종 판정을 갱신합니다.

## 번호 예약 보정을 포함한 최신 제품의 재확인

제품 `c183a1ac61c2aa4188989fd93032cae68d3538c1`에서도 같은 원본과 독립 Print를
전 6쪽 대조했습니다. [Native TSV](../assets/planet6897_20261103/pr_8547/closing-frame-c183a1ac61/native.tsv)는
최저 99.85%, [fresh WASM TSV](../assets/planet6897_20261103/pr_8547/closing-frame-c183a1ac61/wasm.tsv)는
최저 99.87%이며 3·4쪽은 양쪽 모두 100.00%입니다. 6쪽 쪽수도 일치합니다.

| 쪽 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| 3 · 위생 앞 조각의 아래 외곽 | [review](../assets/planet6897_20261103/pr_8547/closing-frame-c183a1ac61/native-p003-review.png) / [overlay](../assets/planet6897_20261103/pr_8547/closing-frame-c183a1ac61/native-p003-overlay.png) | [review](../assets/planet6897_20261103/pr_8547/closing-frame-c183a1ac61/wasm-p003-review.png) / [overlay](../assets/planet6897_20261103/pr_8547/closing-frame-c183a1ac61/wasm-p003-overlay.png) |
| 4 · 영양사 이어받기와 뒤 행 | [review](../assets/planet6897_20261103/pr_8547/closing-frame-c183a1ac61/native-p004-review.png) / [overlay](../assets/planet6897_20261103/pr_8547/closing-frame-c183a1ac61/native-p004-overlay.png) | [review](../assets/planet6897_20261103/pr_8547/closing-frame-c183a1ac61/wasm-p004-review.png) / [overlay](../assets/planet6897_20261103/pr_8547/closing-frame-c183a1ac61/wasm-p004-overlay.png) |

양쪽 두 쪽을 직접 읽어 위생사·영양사의 쪽 소유와 표 하단·후속 행을 확인했습니다.
[실제 제품과 증적 해시](../assets/planet6897_20261103/intermediate-evidence-summary.md)를
연결했습니다. 원본의 호스트 위치·개체 오프셋·선언 높이와 마지막 셀 종료의 관계를
검사하는 `stored_restart_block_owns_source_host_frame_bottom`을 추가했습니다. 글꼴 설치나
절대 px에 의존하지 않으며 기존 6쪽 내용·소속 검사도 유지합니다. 새 검사의 최소 글꼴
실행은 통과했고 최신 통합 source의 필수 실행 41개도 모두 완료했습니다.

## 새 보정의 최소 글꼴 집중 회귀

원본·독립 Print의 양쪽 전쪽 측정 뒤 실제 최소 글꼴 환경에서 정식 검사를 실행했습니다.
기본 개요 4개와 저장 줄 재시작 표 2개, 총 6개가 통과했습니다.
[실행 명령·종료 코드·검사 파일 해시](../assets/planet6897_20261103/rust-validation-c183a1ac61.md)를
연결합니다. 원본 장평을 누락한 첫 검사 실패와 수정 근거도 같은 기록에 구분했습니다.
전체 Rust·Skia·lint·doctest의 필수 실행 41개는 모두 통과했습니다. 새 base Native/fresh WASM 대조도 아래 공통 기록에서 완료로 확인했습니다.

## 보정 전 검토 코드와 증거 출처

| 항목 | 값 |
| --- | --- |
| 원 PR | [PR #8547](https://github.com/edwardkim/rhwp/pull/8547) |
| 원 head | `802078a02454a66ef3dc917d6ba2c6461709dcd5` |
| 현재 통합 적용 커밋 | `3af1303ef138b10668d9ae9c066029a9b121c3c2, 2c508b4ac0e759f8bf81a4001641b28bbf6900bc` |
| 보정 전 필수 검증 제품 / 새 base | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` / `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 전쪽 Native 측정 producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| fresh WASM 실제 빌드 | `7bb9e66fd006109ca9f7ee4c4d3f752c82e60fae` (위 제품 코드와 동등한 구현) |
| reviewer | `jangster77` 요청. `2026-11-05 03:16:02 UTC` 원격 재조회에서 원 head 불변·Open·non-Draft 확인 |

기존 전쪽 TSV/PNG는 실제 측정 producer의 결과로 재사용합니다. 최신 소스의 재출력으로 표시하지 않습니다.
보정 전 제품의 최소 글꼴 전체 Rust 12,561개 통과·Native Skia·Rust lint 및 fresh WASM 실제 실행과
새 base 기능 보존 집중 대조 14쪽을 완료했습니다. [제품 검증](../assets/planet6897_20261103/intermediate-evidence-summary.md)과
[최신 리베이스 대조](../assets/planet6897_20261103/rebase_scope_22947f6a90/README.md)를 연결합니다.

## 보정 전 전쪽 Native/fresh WASM 증적

| 원본 · 독립 Print | Native TSV · 전쪽 최저 | fresh WASM TSV · 전쪽 최저 |
| --- | --- | --- |
| [gangseo_exempt_certificates.hwp](../../../samples/issue8532/gangseo_exempt_certificates.hwp) · [issue8532_gangseo_exempt_certificates-2020.pdf](../../../pdf/issue8532_gangseo_exempt_certificates-2020.pdf) | [TSV](../assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo_exempt_certificates-e5b4dd3b-native.tsv) · 97.95000% (6쪽) | [TSV](../assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo_exempt_certificates-e5b4dd3b-wasm.tsv) · 97.98000% (6쪽) |

96 DPI, 색상 경계 보정 전 2px 내용 실루엣입니다. 입력·Print PDF는 기존 동일 파일을 재사용하고 전체 쪽수와 누락을 확인했습니다.

| 주장 쪽·범위 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| gangseo_exempt_certificates.hwp 3쪽 · 위생 걸침 묶음의 앞 조각과 저장 쪽 경계 | [review](../assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p003-p003-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p003-p003-native-overlay.png) | [review](../assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p003-p003-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p003-p003-wasm-overlay.png) |
| gangseo_exempt_certificates.hwp 4쪽 · 위생 걸침 묶음 이어받기와 뒤 행 | [review](../assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p004-p004-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p004-p004-native-overlay.png) | [review](../assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p004-p004-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p004-p004-wasm-overlay.png) |

## 보정 전 경계의 직접 판독

Native/fresh WASM의 3·4쪽을 각각 직접 대조했습니다. 위생 묶음의 앞 조각은 3쪽, 이어받는 내용은 4쪽에 보존됩니다. 다만 3쪽 마지막 행 아래 외곽 높이는 독립 Print와 다릅니다. 원본 저장값·원 해결 PR과 현재 조각의 요구/예약/실제 높이를 대조하기 전에는 이 차이를 정상이나 해결 완료로 판정하지 않습니다. 4쪽의 행·내용 순서와 외곽은 대응합니다.

[직접 판독 기록과 PNG 해시](../assets/planet6897_20261103/direct-reading-current.md)에 실제 관측을 보존했습니다.

## 통합 검증 완료와 병합 전 조건

정확한 code head의 모든 필수 로컬 실행과 GitHub CI가 성공했습니다. 아래 최신 통합 기록과 최종 code CI 증거를 확인했습니다. 이번 후속 문서 head의 실제 fast-pass·required check·mergeability를 별도로 확인한 뒤 승인된 병합을 진행합니다.

아래 선행 분석은 당시 코드·증적의 기록입니다. 당시 남은 검증 문장은 현재 상태가 아니며 위 현재 기록과 구분합니다.

<details>
<summary>선행 구현 분석·독립 근거·당시 실행 기록</summary>

## 접수 및 적용

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8547](https://github.com/edwardkim/rhwp/pull/8547), `planet6897` → `devel` |
| 원 head | `802078a02454a66ef3dc917d6ba2c6461709dcd5` |
| 통합 적용 | 코드 `8767690869`, 증적 `dc87f4fa2e`; 충돌 없이 순서대로 체리픽 |
| 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 관련 이슈 | [#8532](https://github.com/edwardkim/rhwp/issues/8532) |
| reviewer | `jangster77` 요청. `2026-11-05 03:16:02 UTC` 원격 재조회에서 원 head 불변·Open·non-Draft 확인 |

## 주장과 독립 기준

저장된 셀 줄의 쪽 되감김에서 한 줄 높이의 걸침 묶음 앞 조각을 수용하고,
이어받는 블록의 예약 높이를 실제 배치와 맞춥니다. 원 PR이 제공한
[HWP](../../../samples/issue8532/gangseo_exempt_certificates.hwp)와
[한컴 Print PDF](../../../pdf/issue8532_gangseo_exempt_certificates-2020.pdf)를 대조합니다.
PDF 파일명에는 `2020`이 있지만 메타데이터의 Creator는 `Hwp 2022 12.0.0.4547`,
Producer는 `Hancom PDF 1.3.0.547`이며 6쪽입니다. 입력 SHA-256은
`797268d10fc7605ae5e0e299e9fb386ea6c204f35ef4c4879bb611e66ef4545a`,
PDF SHA-256은 `30429a79cd935c983ca5ad66642e3b189c91d0c5485c31ab2d153026f17e88b7`입니다.

원 PR head에 첨부된 [Native 전쪽 TSV](../assets/issue_8532/native/fixture.tsv)는 최저
97.92%(3쪽), [fresh WASM 전쪽 TSV](../assets/issue_8532/wasm/fixture.tsv)는 최저
98.03%(3쪽)이고 두 출력 모두 Print와 6쪽입니다. 이는 **작성자 head의 결과**입니다.
3쪽의 걸침 묶음 앞 조각과 4쪽의 이어받는 조각을 각각
[3쪽 Native review](../assets/issue_8532/review/fx_native_review_p3.png)·
[overlay](../assets/issue_8532/review/fx_native_overlay_p3.png),
[4쪽 Native review](../assets/issue_8532/review/fx_native_review_p4.png)·
[overlay](../assets/issue_8532/review/fx_native_overlay_p4.png)로 직접 확인했습니다.
3쪽 끝 행과 4쪽 첫 행의 소속, 표 선과 글 내용이 Print와 대응합니다.

`dc87f4fa2e`의 Native Skia 실행 파일 SHA-256
`24c7e929b04c14adc46fbcc80b384e74fffd5524bdd2484068ebe6e974952337`에서
같은 입력·Print와 96 DPI로 다시 측정했습니다.
[통합 head 전쪽 TSV](../assets/planet6897_20261103/pr_8547/gangseo-native.tsv)의
raw 값은 1~6쪽 순서로 99.85%, 99.99%, **97.95%**, 100.00%, 100.00%, 100.00%입니다.
쪽수 6/6, 90% 미만 0쪽입니다. 새 head의 주장 쪽도
[3쪽 review](../assets/planet6897_20261103/pr_8547/gangseo-native-review-p003.png)·
[overlay](../assets/planet6897_20261103/pr_8547/gangseo-native-overlay-p003.png),
[4쪽 review](../assets/planet6897_20261103/pr_8547/gangseo-native-review-p004.png)·
[overlay](../assets/planet6897_20261103/pr_8547/gangseo-native-overlay-p004.png)에서
앞 조각·이어받는 조각과 뒤 행의 소속을 직접 대조했습니다.

## 통합 코드의 fresh WASM 전쪽 측정

같은 제품 코드 `dc87f4fa2e`의 fresh WASM
(`c7e166e1b087641e727d326dd5ff504653f96134808242f9d1fd83228bbae903`)을
2026-11-04에 같은 입력·독립 Print PDF, 96 DPI로 측정했습니다.
[전쪽 TSV](../assets/planet6897_20261103/pr_8547/gangseo-wasm.tsv)는 6/6쪽,
90% 미만 0쪽이며 1~6쪽 raw는 99.87%, 99.99%, **97.98%**, 100.00%, 100.00%,
100.00%입니다.
[3쪽 review](../assets/planet6897_20261103/pr_8547/gangseo-wasm-review-p003.png)·
[overlay](../assets/planet6897_20261103/pr_8547/gangseo-wasm-overlay-p003.png)와
[4쪽 review](../assets/planet6897_20261103/pr_8547/gangseo-wasm-review-p004.png)·
[overlay](../assets/planet6897_20261103/pr_8547/gangseo-wasm-overlay-p004.png)를 같은 입력의
같은 쪽에 연결했습니다.

## 남은 검증

- 최소 글꼴 환경에서 `issue_8532_rowspan_hard_break`와 관련 대조군, 전체 회귀를 실행합니다.
- 기준 devel과 같은 입력·Print·글꼴 조건에서 페이지별 하락을 대조하고,
  다른 문서에 대한 #8547의 영향도 분리해 판정합니다.
- 통합 PR이 병합되면 실제 merge SHA와 검증한 증거를 원 PR에 연결하고 후속 처리합니다.

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

[후속 댓글의 실제 원본·쪽·Native/WASM PNG 대응표](../assets/planet6897_20261103/source-comment-evidence.md#pr-8547)를 고정했습니다. 이미지 수·출력 경로·해시를 게시할 댓글과 대조합니다.
