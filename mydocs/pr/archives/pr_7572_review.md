# PR #7572 검토 — 본문 조합 중 명시적 배치의 쪽 나눔 경계

## 최종 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#7572](https://github.com/edwardkim/rhwp/pull/7572) / semanticist21.
- 원 contributor head: `59276bb29419c3a439866d132f5be69a6c759c7c`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_7572_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 공개 재현 계약에 대응하는 입력과 상태만 명시한다. 배경 사례로 언급한 별도 실물 문서의 개선으로 확대하지 않는다.

- 실제 검증 입력: [178.hwpx](../assets/s21_20261027/inputs/178.hwpx).
- 입력 SHA-256: `9ba1c6e882408656bccb85688b8ba4588626a91e61b068b106c8cf5fcf06ac23`.
- 독립 Print PDF: [178-2020.pdf](../assets/s21_20261027/inputs/178-2020.pdf).
- PDF SHA-256: `1bf0df15cbdacdc8246f3087cd0a74584951ee786c02fd11949fda9c67849ecd`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 3 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_7572/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_7572/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_7572/native/overlay.png) |
| wasm | 3 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_7572/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_7572/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_7572/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

원 PR은 편집 서식의 상속 규칙 변경을 요구하지 않는다. `beginBatch`와 `endBatch` 사이의
본문 `applyCharFormat`·`replaceBodyTextLocal`이 중간 쪽 나눔을 확정하지 않아야 한다는
계약을 실패 검사 두 개로 제시한다. 원 PR의 측정 시간은 성능 보장으로 사용하지 않는다.

보정한 `commands/formatting.rs`의 본문 서식 변경은
`rebuild_paragraph_deferred_in_batch`를 통해 스타일과 해당 문단 조합을 즉시 갱신하고,
배치 중에는 구역을 dirty로 남긴다. `commands/text_editing.rs`의 local replace도 문단
조합 결과를 갱신한 뒤 배치 여부를 먼저 확인한다. `flowChanged`가 true여도 배치 중에는
직접 `paginate()`하지 않으며 `documentPaginationPending`을 true로 반환한다.
마지막 쪽 소속은 공통 `endBatch`가 확정한다. 특정 문서·음절·좌표로 분기하지 않는다.


실제 회귀 source:
- [tests/cases/body_ime_format_batch.rs](../../../tests/cases/body_ime_format_batch.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/7572#issuecomment-6440874055)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
