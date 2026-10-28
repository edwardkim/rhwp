# PR #8014 리뷰 — fix(render): 선 종류가 없음인 직선을 그리지 않는다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8014](https://github.com/edwardkim/rhwp/pull/8014) / semanticist21.
- 원 contributor head: `4c857d41e760bbb80f88650558d376504bb10874`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8014_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **통계청 인구동향 보도자료 41쪽**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [2026년+7월+인구동향.hwp](../assets/pr8319_evidence_alignment_20261027/official-inputs/2026년+7월+인구동향.hwp), SHA-256 `6f87a7a4de1e89bab89be40cec8523d106300a259ee953481d3d288f91fd99ad`.

출처: [공식 게시물](https://mods.go.kr/board.es?mid=a10301010000&bid=204&act=view&list_no=447111). 2026년 7월 인구동향 41쪽 보도자료의 제목·본문 대응을 확인했다. 사용자 Mac 파일과 바이트 동일성은 확인하지 못했다.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

- 선행 대조 검사 입력: [195.hwpx](../assets/s21_20261027/inputs/195.hwpx).
- 입력 SHA-256: `56fb2054fa2a57d09a4e024edc5bd29f6f67a47424e2b19096a18bbb986d4684`.
- 독립 Print PDF: [195-2020.pdf](../assets/s21_20261027/inputs/195-2020.pdf).
- PDF SHA-256: `081082724f2e50034da71cfb3a91c291820d00641eb5bc2fd3348e9c4791c5c7`.
- 대표 대응: 이 입력의 독립 PDF 1쪽 ↔ RHWP 1쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 2 | 99.98236% | 99.98384% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8014/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8014/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8014/native/overlay.png) |
| wasm | 2 | 99.76268% | 99.76268% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8014/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8014/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8014/wasm/overlay.png) |

위 표의 수치와 PNG는 선행 대조 검사 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

직선의 border_line.attr 하위 6비트가 0이면 실제 Line 노드를 지우지 않고 visible=false로 만든다. 따라서 출력 제외와 선택·개체 소유를 구분하며, 실선 대조군의 선 하나만 줄어드는지 검사한다. 같은 shape_layout의 일반·변환 경로가 line_visible을 소비하는지 함께 확인했다.

tests/cases/line_shape_type_none.rs의 동일 입력 SOLID/NONE 대조. 이 SVG assertion 자체를 독립 Print 일치율로 해석하지 않는다.


실제 회귀 source:
- [tests/cases/line_shape_type_none.rs](../../../tests/cases/line_shape_type_none.rs).

통합 당시의 source/test에 대한 기록이다. 이번 원문 검토의 추가 보정은 별도로 기록한다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8014#issuecomment-6440888736)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.

## 쪽 경계 보정 후 Native 확인

추가 보정 후 Native와 Print는 모두 41쪽이며 Native 전체 쪽의 최저 raw는 95.98915%다. 표 뒤 주석은 14쪽에, 다음 표는 15쪽에 보존된다. 원 PR의 빨간 선은 이 대체 문서에서 통합 전에도 재현되지 않아 그 실패를 증명한 것으로 표시하지 않는다.

[전체 페이지 Native TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/native/silhouette.tsv)

- 물리 14쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/native-pagefix/review_014.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/native-pagefix/overlay_014.png)
- 물리 15쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/native-pagefix/review_015.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/native-pagefix/overlay_015.png)

fresh WASM과 수정 후 영향 문서의 시각 대조 결과는 아래 항목에 기록한다. 전체 Rust 회귀 검증은 별도로 확인한다.

<!-- source-claim-native-start -->
## 추가 확보 문서의 Native 직접 대조

원 PR에 입력이 첨부되지 않아 내용이 대응하는 공식 배포 문서를 확보했다. 원 작성자 파일과의 바이트 동일성은 주장하지 않는다. [2026년+7월+인구동향.hwp](../assets/pr8319_evidence_alignment_20261027/official-inputs/2026년+7월+인구동향.hwp)와 같은 입력의 [한컴 Print PDF](../assets/pr8319_evidence_alignment_20261027/official-pdf/hancom-print/2026년+7월+인구동향-2020.pdf)를 사용했다.
후보 Native 전체 쪽수는 41쪽, PDF는 41쪽이다. 선택 쪽 검토 상태는 `passed`다. 새로 확보한 문서의 전쪽 90% 보정은 작업지시에서 면제했으며 점수 미달·쪽수 차이를 통과로 바꾸지 않는다.

원 PR이 지적한 위치는 2쪽이지만, 대체 파일은 통합 전 Native에서도 빨간 선이 재현되지 않아 작성자 원본의 수정 전 실패를 입증하지 않는다. 쪽 경계 보정 후 전체 쪽수는 Print와 같은 41쪽이다.

- 물리 2쪽: raw 99.20078% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/native/review_002.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/native/overlay_002.png)

[전체 페이지 Native TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/native/silhouette.tsv)

Native의 고정 `upstream/devel` 대비 가능한 쪽별 raw 0.1%p 초과 하락은 없었다. 문서 쪽수가 다르면 공통 쪽 비교는 누락 쪽을 검증하지 못한다.
<!-- source-claim-native-end -->

## #8319 통합 전 동일 입력 대조

통합 merge의 첫 부모 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`에서 같은 입력과 Print PDF를 Native로 출력했다. 전체 쪽수는 RHWP 42쪽, PDF 41쪽이며 아래 선택 쪽 최저 raw는 99.01623%다. 선택 영역의 일치율은 실제 주장 동작의 개선이나 전체 문서 통과를 단독으로 입증하지 않는다.

이 대체 입력의 Native에서는 원 PR이 보여 준 수정 전 결함이 같은 위치에서 재현되지 않았다. 현재 출력을 원 작성자 파일의 수정 전후 증거로 확대하지 않는다.

- 물리 2쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/native-before-integration/review_002.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/native-before-integration/overlay_002.png)

<!-- source-claim-wasm-start -->
## 추가 확보 문서의 fresh WASM 대조

동일 원문과 한컴 Print를 96dpi에서 비교했다. 전체 41쪽의 raw 최저는 96.32312%이며, 90% 미만 쪽은 없음이다. 선정한 주장 영역의 결과는 아래와 같다.

- 물리 2쪽: raw 99.42917% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/wasm/review_002.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/wasm/overlay_002.png)

[전체 페이지 WASM TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/wasm/silhouette.tsv)

고정 base와 비교 가능한 같은 번호의 쪽에서 raw 0.1%p 초과 하락은 없었다. 원문 전체 페이지의 기존 90% 미만은 작업지시에 따라 명시하며 통과로 표현하지 않는다.
<!-- source-claim-wasm-end -->

## 쪽 경계 fresh WASM 직접 대조

- 물리 14쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/wasm-pagefix/review_014.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/wasm-pagefix/overlay_014.png)
- 물리 15쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/wasm-pagefix/review_015.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8014/wasm-pagefix/overlay_015.png)

<!-- current-control-start -->
## 현재 후보의 기존 대조군 확인

기존 대조군 원본 `mydocs/pr/assets/s21_20261027/inputs/195.hwpx`과 독립 Print PDF를 같은 물리 1쪽에서 다시 비교했다. 이는 새로 확보한 원문과 다를 수 있다.

- NATIVE: 전체 2쪽 최저 raw 99.98236%, 해당 쪽 99.98384% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8014/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8014/native/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8014/native/silhouette.tsv)
- WASM: 전체 2쪽 최저 raw 99.76268%, 해당 쪽 99.76268% · [review](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8014/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8014/wasm/overlay.png) / [전체 TSV](../assets/pr8319_evidence_alignment_20261027/current-evidence/pr8014/wasm/silhouette.tsv)

이전 증적과 비교한 전체 쪽에서 0.1%p 초과 raw 하락과 쪽 누락이 없다.
<!-- current-control-end -->
