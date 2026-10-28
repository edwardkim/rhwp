# PR #8094 리뷰 — fix(layout): 캡션 자동 번호 뒤 캐럿과 클릭 위치를 그린 글자에 맞춘다

## 최종 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8094](https://github.com/edwardkim/rhwp/pull/8094) / semanticist21.
- 원 contributor head: `b9f84e4105b146902297a4e3c46fd82e0f2c6225`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8094_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 공개 재현 계약에 대응하는 입력과 상태만 명시한다. 배경 사례로 언급한 별도 실물 문서의 개선으로 확대하지 않는다.

- 실제 검증 입력: [pr8094-caption-number-sudal.hwpx](../assets/pr8319_evidence_alignment_20261027/inputs/pr8094-caption-number-sudal.hwpx).
- 입력 SHA-256: `dab243d31a31079621231c4f625ea6d9eb99119f3e050aaf7c3411f9e29b5189`.
- 독립 Print PDF: [pr8094-caption-number-sudal-2020.pdf](../../../pdf/pr8319_evidence_alignment_20261027/pr8094-caption-number-sudal-2020.pdf).
- PDF SHA-256: `acf82f44ad2c2e53f4ba6c068ace272038a4f23fa9f5f006ceecd76e155a7a6c`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

원 단위 검사 createEmpty()에는 저장용 스타일 ID가 없어 export가 실패했다. 정상 createBlankDocument() seed를 추가하고 반환된 실제 picture controlIdx로 같은 공개 편집을 수행한 유효 대조군이다. 원 메모리 객체와 바이트가 같다고 주장하지 않는다.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 1 | 98.11257% | 98.11257% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8094/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8094/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8094/native/overlay.png) |
| wasm | 1 | 98.11342% | 98.11342% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8094/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8094/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8094/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

자동 번호의 원본 자리표와 표시 문자열을 projection으로 구분하고 후속 문자의 source offset을 보존한다. caption_auto_number_caret는 표시한 캡션의 실제 글자 끝과 caret/hit 결과를 같은 tree에서 대조한다. parser의 raw 8유닛 슬롯, OLE caption·picture footnote의 owner 주소와 continuation을 함께 확인한다. cap 번호를 본문 문자열에 덧붙여 logical 길이를 바꾸는 방식으로 위치를 맞추지 않는다.


실제 회귀 source:
- [tests/cases/caption_auto_number_caret.rs](../../../tests/cases/caption_auto_number_caret.rs).
- [tests/issue_2069_ole_object_selection.rs](../../../tests/issue_2069_ole_object_selection.rs).

통합 당시의 source/test에 대한 기록이다. 이번 원문 검토의 추가 보정은 별도로 기록한다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8094#issuecomment-6440935724)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `mydocs/pr/assets/pr8319_evidence_alignment_20261027/inputs/pr8094-caption-number-sudal.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 1쪽 최저 raw 98.11257%, 해당 쪽 98.11257% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8094/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8094/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8094/native/silhouette.tsv)
- WASM: 전체 1쪽 최저 raw 98.11342%, 해당 쪽 98.11342% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8094/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8094/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8094/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
