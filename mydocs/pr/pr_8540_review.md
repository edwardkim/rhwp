# PR #8540 리뷰 — scaffold 개요 번호 형식

## 최종 판정

**메인터너 보정 후 수용 가능**

완료한 실측과 남은 통합 공통 검증을 구분합니다. 원 PR의 직접 승인·닫기와 통합 PR 생성은 수행하지 않았습니다.
최종 수용은 WASM의 당시 base 대비 전쪽 대조와 전체 증적 감사를 완료한 뒤 판정합니다. PR 생성은 작업지시자 승인 대기입니다.

## 현재 검토 코드와 증거 출처

| 항목 | 값 |
| --- | --- |
| 원 PR | [PR #8540](https://github.com/edwardkim/rhwp/pull/8540) |
| 원 head | `3de1e510d9039490485505bfb7250182d59e2512` |
| 현재 통합 적용 커밋 | `34701d18a88053c27999ba67b482fc7efcec25c3` |
| 최신 검증 제품 / 새 base | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` / `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 전쪽 Native 측정 producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| fresh WASM 실제 빌드 | `7bb9e66fd006109ca9f7ee4c4d3f752c82e60fae` (위 제품 코드와 동등한 구현) |
| reviewer | `jangster77` 요청; 2026-11-04T17:44:08.556Z 고정한 원 head·Open·non-Draft 확인 |

기존 전쪽 TSV/PNG는 실제 측정 producer의 결과로 재사용합니다. 최신 소스의 재출력으로 표시하지 않습니다.
최신 제품의 최소 글꼴 전체 Rust 12,561개 통과·Native Skia·Rust lint 및 fresh WASM 실제 실행과
새 base 기능 보존 집중 대조 14쪽을 완료했습니다. [제품 검증](assets/planet6897_20261103/intermediate-evidence-summary.md)과
[최신 리베이스 대조](assets/planet6897_20261103/rebase_scope_22947f6a90/README.md)를 연결합니다.

## 현재 전쪽 Native/fresh WASM 증적

| 원본 · 독립 Print | Native TSV · 전쪽 최저 | fresh WASM TSV · 전쪽 최저 |
| --- | --- | --- |
| [scaffold_outline_a.hwpx](assets/scaffold_outline_numbering/scaffold_outline_a.hwpx) · [scaffold_outline_a-2024.pdf](../../pdf/issue8540/scaffold_outline_a-2024.pdf) | [TSV](assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_a-649191af-native.tsv) · 100.00000% (1쪽) | [TSV](assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_a-649191af-wasm.tsv) · 100.00000% (1쪽) |
| [scaffold_outline_b.hwpx](assets/scaffold_outline_numbering/scaffold_outline_b.hwpx) · [scaffold_outline_b-2024.pdf](../../pdf/issue8540/scaffold_outline_b-2024.pdf) | [TSV](assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_b-82b08f33-native.tsv) · 100.00000% (1쪽) | [TSV](assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_b-82b08f33-wasm.tsv) · 100.00000% (1쪽) |

96 DPI, 색상 경계 보정 전 2px 내용 실루엣입니다. 입력·Print PDF는 기존 동일 파일을 재사용하고 전체 쪽수와 누락을 확인했습니다.

| 주장 쪽·범위 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| scaffold_outline_a.hwpx 1쪽 · A/B 수준별 개요 번호·본문의 순서와 간격 | [review](assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_a-649191af-p001-native-review.png) / [overlay](assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_a-649191af-p001-native-overlay.png) | [review](assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_a-649191af-p001-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_a-649191af-p001-wasm-overlay.png) |
| scaffold_outline_b.hwpx 1쪽 · A/B 수준별 개요 번호·본문의 순서와 간격 | [review](assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_b-82b08f33-p001-native-review.png) / [overlay](assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_b-82b08f33-p001-native-overlay.png) | [review](assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_b-82b08f33-p001-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8540/current-4b149b6cdf/pr8540-scaffold_outline_b-82b08f33-p001-wasm-overlay.png) |

A/B 각각의 Native·WASM review와 overlay를 함께 직접 판독했습니다. 수준별 번호의 문자·본문 순서와 줄바꿈·간격을 확인했습니다. 2px raw 100%를 모든 획의 완전한 픽셀 일치로 확대하지 않습니다.

## 남은 통합 검증과 원격 조건

동일 환경의 당시 base 대비 WASM 80개 입력·406쪽 0.1%p 하락 게이트와 주장별 증적 최종 감사가 남았습니다.
[완료·대기 검증 기록](assets/planet6897_20261103/intermediate-evidence-summary.md)을 따릅니다.

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
| reviewer | `jangster77` 요청 완료. 최신 원 head·CI·mergeability는 원격 조치 직전 재확인합니다. |

## 변경과 검토 범위

`outline_numbering` 명세를 scaffold 모델에 추가하고, 수준별 번호 정의와 구역 참조를
생성합니다. 필드 생략 시 기존 출력이 유지되는지, 형식 코드와 HWPX 왕복이 맞는지,
실제 개요 번호가 한컴 Print와 일치하는지 확인합니다. 이 변경은 **새 scaffold 문서**에
영향을 주며 이미 저장된 다른 문서의 조판 경로는 바꾸지 않습니다.

## 검증 입력과 결과

원 PR이 커밋한 [A/B HWPX와 JSON](assets/scaffold_outline_numbering/)을 그대로 사용했습니다.
원 PR PDF는 `FileSaveAsPdf`로 만든 자료이므로 기준으로 쓰지 않았습니다. 한컴 2024 MCP의
`PrintToPDFEx`, `pdf_print_method=0`, `frame_print_to_pdf_ex_one_up`에서 각 1쪽을 새로
출력했습니다. 작업 ID·입력/PDF SHA-256·제품 버전은
[Print 근거](assets/issue_8540/print-provenance.md)에 기록했습니다.

| 원본 | 기준 Print PDF | Native 전쪽 TSV | fresh WASM 전쪽 TSV | 페이지별 raw 최저값 |
| --- | --- | --- | --- | --- |
| [A HWPX](assets/scaffold_outline_numbering/scaffold_outline_a.hwpx) | [A PDF](../../pdf/issue8540/scaffold_outline_a-2024.pdf) | [A Native](assets/issue_8540/a-native.tsv) | [A WASM](assets/issue_8540/a-wasm.tsv) | 1쪽 100.00% / 100.00% |
| [B HWPX](assets/scaffold_outline_numbering/scaffold_outline_b.hwpx) | [B PDF](../../pdf/issue8540/scaffold_outline_b-2024.pdf) | [B Native](assets/issue_8540/b-native.tsv) | [B WASM](assets/issue_8540/b-wasm.tsv) | 1쪽 100.00% / 100.00% |

96 DPI, 색상 경계 보정 전 2px 실루엣 기준입니다. 90% 미만·누락 쪽은 없고 PDF 쪽수도
각 1쪽으로 일치합니다. 이 측정의 renderer 바이너리와 WASM package는 통합 코드
`0884eafc7b2c34fae20a765109ff314514175e4b`에서 빌드했습니다. #8540 체리픽 뒤
최종 source 전체 재검증 결과를 별도로 연결한 뒤 판정을 갱신합니다.

2026-11-04에 최종 제품 코드 `dc87f4fa2e`의 Native Skia 실행 파일
(`24c7e929b04c14adc46fbcc80b384e74fffd5524bdd2484068ebe6e974952337`)로
A/B를 같은 Print PDF·96 DPI에서 다시 출력했습니다. 두 문서 모두 1/1쪽, raw 100.00%입니다.
기존 `0884eafc7b2c` 실행의 Native PNG와 PDF PNG가 각각 바이트 단위로 일치해
위 Native review/overlay를 재사용합니다.
[출력별 SHA 대조](assets/issue_8540/current-native-output-sha.tsv)에 재출력과 재사용 근거를 남겼습니다.
최종 제품의 fresh WASM 전쪽 대조와 전체 회귀 결과는 완료 후 추가합니다.

## 시각 증적과 직접 판독

| 문서 | Native review / overlay | fresh WASM review / overlay | 직접 판독 |
| --- | --- | --- | --- |
| A 1쪽 | [review](assets/issue_8540/a-native-review.png) · [overlay](assets/issue_8540/a-native-overlay.png) | [review](assets/issue_8540/a-wasm-review.png) · [overlay](assets/issue_8540/a-wasm-overlay.png) | `I.`·`1.`·`가.`·`가)`·`(1)`·`(가)`·`①`·`II.` 순서와 간격이 Print와 일치합니다. |
| B 1쪽 | [review](assets/issue_8540/b-native-review.png) · [overlay](assets/issue_8540/b-native-overlay.png) | [review](assets/issue_8540/b-wasm-review.png) · [overlay](assets/issue_8540/b-wasm-overlay.png) | `i)`·`A.`·`(a)`·`①`·`ii)`가 Print와 일치합니다. |

개요 번호·본문 순서와 누락을 확대 대조했습니다. 새로운 절대 px 핀은 추가하지 않았습니다.
`tests/cases/scaffold_outline_numbering.rs`는 구조·문자열 계약을 확인합니다.
최종 head에서 focused/전체 Rust, lint, Native/fresh WASM, devel 대비 0.1%p 대조가 남았습니다.

## Merge 후 contributor PR comment 계획

통합 PR이 merge되면 최종 merge SHA와 CI run, [Visual Sweep 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment),
위 A/B 1쪽의 실제 Native/fresh WASM raw 100.00% 및 각 review·overlay의 merge SHA 고정
raw URL을 원 PR 댓글에 담겠습니다. 원 PR에서 개요 번호 생성 기능을 제공했고,
메인터너는 별도의 Print 기준과 시각 증거를 보강했음을 구분하겠습니다.
댓글은 UTF-8 Markdown `--body-file`로 게시한 뒤 API로 다시 확인합니다.

</details>
