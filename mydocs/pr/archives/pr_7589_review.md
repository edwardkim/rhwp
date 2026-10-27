# PR #7589 검토 — 입력 상자의 저장 개행과 Print 표시

## 최종 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#7589](https://github.com/edwardkim/rhwp/pull/7589) / semanticist21.
- 원 contributor head: `300d1af385064eb32bb85c94f3a40ab7213d2d2a`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_7589_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 공개 재현 계약에 대응하는 입력과 상태만 명시한다. 배경 사례로 언급한 별도 실물 문서의 개선으로 확대하지 않는다.

- 실제 검증 입력: [pr7589-multiline-true.hwpx](../assets/pr_7589_form_multiline/pr7589-multiline-true.hwpx).
- 입력 SHA-256: `8b3d341410df07fc32f0909b5b84f274ec48e61bb70bf1bcc6050c2e62599457`.
- 독립 Print PDF: [pr7589-multiline-true-2020.pdf](../assets/pr_7589_form_multiline/pr7589-multiline-true-2020.pdf).
- PDF SHA-256: `21482ec033b610614f675e32bb983cb3d75f9acbbf7ca7bbe00702f61f8f7868`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_7589/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_7589/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_7589/native/overlay.png) |
| wasm | 1 | 92.37734% | 92.37734% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_7589/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_7589/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_7589/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

원 PR은 같은 `FIRST\nSECOND`에서 MultiLine=1/0의 SVG가 달라야 한다고 주장한다.
파서·직렬화가 개행과 속성을 보존하는지 검사한 점은 유지한다. 그러나 그 뒤의
`assert_ne!`는 이 입력의 한컴 인쇄 동작을 입증하지 않는다.

사용자가 두 HWPX를 수정 없이 한컴의 파일 → 인쇄 → PDF로 출력했고,
win10-ted의 `C:\temp`에서 원본 바이트를 가져왔다. 두 PDF는 각각 A4 한 쪽이며
실제 Creator는 `Hwp 2022 12.0.0.4605`다. 파일명의 2020과 실제 제품을 구분한다.
두 Print의 래스터는 동일하고 입력 상자에는 `FIRST`만 표시된다.

공통 `FormObjectNode`의 표시 문자열에서 첫 저장 줄을 선택하되 모델 원문과
MultiLine 속성은 보존하도록 보정했다. SVG·Canvas·Native가 같은 표시 결과를
소비하며 특정 문서나 좌표에 따른 예외는 없다. 모든 입력 상자의 편집 중 화면까지
MultiLine이 무의미하다고 일반화하지 않는다.


실제 회귀 source:
- [tests/cases/form_multiline_rendering.rs](../../../tests/cases/form_multiline_rendering.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/7589#issuecomment-6440877968)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
