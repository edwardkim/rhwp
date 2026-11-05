# PR #8540 리뷰 — scaffold 개요 번호 형식

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
| 원 PR | [PR #8540](https://github.com/edwardkim/rhwp/pull/8540) |
| 원 head | `3de1e510d9039490485505bfb7250182d59e2512` |
| 현재 통합 적용 커밋 | `6d003e0f2047222b4bb79c354359543ceb45e850` |
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
| [scaffold_outline_a.hwpx](../assets/scaffold_outline_numbering/scaffold_outline_a.hwpx) · [scaffold_outline_a-2024.pdf](../../../pdf/issue8540/scaffold_outline_a-2024.pdf) | [TSV](../assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_a-649191af-native.tsv) · 100.00000% (1쪽) | [TSV](../assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_a-649191af-wasm.tsv) · 100.00000% (1쪽) |
| [scaffold_outline_b.hwpx](../assets/scaffold_outline_numbering/scaffold_outline_b.hwpx) · [scaffold_outline_b-2024.pdf](../../../pdf/issue8540/scaffold_outline_b-2024.pdf) | [TSV](../assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_b-82b08f33-native.tsv) · 100.00000% (1쪽) | [TSV](../assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_b-82b08f33-wasm.tsv) · 100.00000% (1쪽) |

96 DPI, 색상 경계 보정 전 2px 내용 실루엣입니다. 입력·Print PDF는 기존 동일 파일을 재사용하고 전체 쪽수와 누락을 확인했습니다.

| 주장 쪽·범위 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| scaffold_outline_a.hwpx 1쪽 · A/B 수준별 개요 번호·본문의 순서와 간격 | [review](../assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_a-649191af-p001-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_a-649191af-p001-native-overlay.png) | [review](../assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_a-649191af-p001-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_a-649191af-p001-wasm-overlay.png) |
| scaffold_outline_b.hwpx 1쪽 · A/B 수준별 개요 번호·본문의 순서와 간격 | [review](../assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_b-82b08f33-p001-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_b-82b08f33-p001-native-overlay.png) | [review](../assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_b-82b08f33-p001-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_b-82b08f33-p001-wasm-overlay.png) |

A/B 각각의 Native·WASM review와 overlay를 함께 직접 판독했습니다. 수준별 번호의 문자·본문 순서와 줄바꿈·간격을 확인했습니다. 2px raw 100%를 모든 획의 완전한 픽셀 일치로 확대하지 않습니다.

## 통합 검증 완료와 병합 전 조건

정확한 code head의 모든 필수 로컬 실행과 GitHub CI가 성공했습니다. 아래 최신 통합 기록과 최종 code CI 증거를 확인했습니다. 이번 후속 문서 head의 실제 fast-pass·required check·mergeability를 별도로 확인한 뒤 승인된 병합을 진행합니다.

아래 선행 분석은 당시 코드·증적의 기록입니다. 당시 남은 검증 문장은 현재 상태가 아니며 위 현재 기록과 구분합니다.

<details>
<summary>선행 구현 분석·독립 근거·당시 실행 기록</summary>

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8540](https://github.com/edwardkim/rhwp/pull/8540), `planet6897`, `devel` 대상 |
| 원 head | `3de1e510d9039490485505bfb7250182d59e2512` |
| 통합 적용 | `dfac7066081763b5a7a9a15660aa019786d13a48`; 메인터너 Print 증적 `64034040c7` |
| 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 관련 이슈 | [#8534](https://github.com/edwardkim/rhwp/issues/8534) 참조. 한글 숫자·한자 숫자 직렬화는 이 PR에서 열지 않습니다. |
| reviewer | `jangster77` 요청. `2026-11-05 03:16:02 UTC` 원격 재조회에서 원 head 불변·Open·non-Draft 확인 |

## 변경과 검토 범위

`outline_numbering` 명세를 scaffold 모델에 추가하고, 수준별 번호 정의와 구역 참조를
생성합니다. 필드 생략 시 기존 출력이 유지되는지, 형식 코드와 HWPX 왕복이 맞는지,
실제 개요 번호가 한컴 Print와 일치하는지 확인합니다. 이 변경은 **새 scaffold 문서**에
영향을 주며 이미 저장된 다른 문서의 조판 경로는 바꾸지 않습니다.

## 검증 입력과 결과

원 PR이 커밋한 [A/B HWPX와 JSON](../assets/scaffold_outline_numbering)을 그대로 사용했습니다.
원 PR PDF는 `FileSaveAsPdf`로 만든 자료이므로 기준으로 쓰지 않았습니다. 한컴 2024 MCP의
`PrintToPDFEx`, `pdf_print_method=0`, `frame_print_to_pdf_ex_one_up`에서 각 1쪽을 새로
출력했습니다. 작업 ID·입력/PDF SHA-256·제품 버전은
[Print 근거](../assets/issue_8540/print-provenance.md)에 기록했습니다.

| 원본 | 기준 Print PDF | Native 전쪽 TSV | fresh WASM 전쪽 TSV | 페이지별 raw 최저값 |
| --- | --- | --- | --- | --- |
| [A HWPX](../assets/scaffold_outline_numbering/scaffold_outline_a.hwpx) | [A PDF](../../../pdf/issue8540/scaffold_outline_a-2024.pdf) | [A Native](../assets/issue_8540/a-native.tsv) | [A WASM](../assets/issue_8540/a-wasm.tsv) | 1쪽 100.00% / 100.00% |
| [B HWPX](../assets/scaffold_outline_numbering/scaffold_outline_b.hwpx) | [B PDF](../../../pdf/issue8540/scaffold_outline_b-2024.pdf) | [B Native](../assets/issue_8540/b-native.tsv) | [B WASM](../assets/issue_8540/b-wasm.tsv) | 1쪽 100.00% / 100.00% |

96 DPI, 색상 경계 보정 전 2px 실루엣 기준입니다. 90% 미만·누락 쪽은 없고 PDF 쪽수도
각 1쪽으로 일치합니다. 이 측정의 renderer 바이너리와 WASM package는 통합 코드
`0884eafc7b2c34fae20a765109ff314514175e4b`에서 빌드했습니다. #8540 체리픽 뒤
최종 source 전체 재검증 결과를 별도로 연결한 뒤 판정을 갱신합니다.

2026-11-04에 최종 제품 코드 `dc87f4fa2e`의 Native Skia 실행 파일
(`24c7e929b04c14adc46fbcc80b384e74fffd5524bdd2484068ebe6e974952337`)로
A/B를 같은 Print PDF·96 DPI에서 다시 출력했습니다. 두 문서 모두 1/1쪽, raw 100.00%입니다.
기존 `0884eafc7b2c` 실행의 Native PNG와 PDF PNG가 각각 바이트 단위로 일치해
위 Native review/overlay를 재사용합니다.
[출력별 SHA 대조](../assets/issue_8540/current-native-output-sha.tsv)에 재출력과 재사용 근거를 남겼습니다.
최종 제품의 fresh WASM 전쪽 대조와 전체 회귀 결과는 완료 후 추가합니다.

## 시각 증적과 직접 판독

| 문서 | Native review / overlay | fresh WASM review / overlay | 직접 판독 |
| --- | --- | --- | --- |
| A 1쪽 | [review](../assets/issue_8540/a-native-review.png) · [overlay](../assets/issue_8540/a-native-overlay.png) | [review](../assets/issue_8540/a-wasm-review.png) · [overlay](../assets/issue_8540/a-wasm-overlay.png) | `I.`·`1.`·`가.`·`가)`·`(1)`·`(가)`·`①`·`II.` 순서와 간격이 Print와 일치합니다. |
| B 1쪽 | [review](../assets/issue_8540/b-native-review.png) · [overlay](../assets/issue_8540/b-native-overlay.png) | [review](../assets/issue_8540/b-wasm-review.png) · [overlay](../assets/issue_8540/b-wasm-overlay.png) | `i)`·`A.`·`(a)`·`①`·`ii)`가 Print와 일치합니다. |

개요 번호·본문 순서와 누락을 확대 대조했습니다. 새로운 절대 px 핀은 추가하지 않았습니다.
`tests/cases/scaffold_outline_numbering.rs`는 구조·문자열 계약을 확인합니다.
최종 head에서 focused/전체 Rust, lint, Native/fresh WASM, devel 대비 0.1%p 대조가 남았습니다.

## Merge 후 contributor PR comment 계획

통합 PR이 merge되면 최종 merge SHA와 CI run, [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment),
위 A/B 1쪽의 실제 Native/fresh WASM raw 100.00% 및 각 review·overlay의 merge SHA 고정
raw URL을 원 PR 댓글에 담겠습니다. 원 PR에서 개요 번호 생성 기능을 제공했고,
메인터너는 별도의 Print 기준과 시각 증거를 보강했음을 구분하겠습니다.
댓글은 UTF-8 Markdown `--body-file`로 게시한 뒤 API로 다시 확인합니다.

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

[후속 댓글의 실제 원본·쪽·Native/WASM PNG 대응표](../assets/planet6897_20261103/source-comment-evidence.md#pr-8540)를 고정했습니다. 이미지 수·출력 경로·해시를 게시할 댓글과 대조합니다.
