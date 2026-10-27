# PR #8027 구현·검증 기록 정정

## 현재 판정

**증적 대응 정정 — 공식 대체 입력과 단위 계약의 검증 범위로 한정.**

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 공식 배포 문서로 대조했다. 작성자 원본과의 바이트 동일성은 확인하지 않았다.**

## 구현과 과거 실제 CI

EMF parser의 0x55/0x56/0x57를 각각 POLYBEZIER16/POLYGON16/POLYLINE16로 분리하고, dispatch에서 열린 선과 닫힌 다각형의 record 종류를 서로 바꾸지 않는다. 범주·값을 다시 구성하는 OOXML 차트 로직과 저장된 OLE 미리보기의 EMF 소비 경로를 구분한다. legacy 차트의 물리 배치·축 라벨 보정은 원 record 종류 수정과 별도의 공통 paint 문제로 검토한다.

src/emf/parser/mod.rs의 상수와 record dispatch, src/emf/tests.rs의 16bit record 검사 및 mydocs/pr/assets/legacy_chart_contracts_20261020/README.md와 legacy_chart_physical_layout_20261020/README.md의 독립 Print 기록.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 이 CI는 통합 당시의 실행 이력이며 후속 원문 대조에서 추가한 엔진·회귀 변경의 검증을 대신하지 않는다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8027_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

앞선 통합 당시에는 파일을 소비하지 않는 단위 계약을 검증했다. 추가 확보 문서의 Visual Sweep 결과는 아래에서 별도로 구분한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440891320`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.

<!-- source-claim-native-start -->
## 추가 확보 문서의 Native 직접 대조

원 PR에 입력이 첨부되지 않아 내용이 대응하는 공식 배포 문서를 확보했다. 원 작성자 파일과의 바이트 동일성은 주장하지 않는다. [pr8027-official.hwpx](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8027-official.hwpx)와 같은 입력의 [한컴 Print PDF](../assets/pr8319_evidence_alignment_20261027/official-pdf/hancom-print/pr8027-official-2020.pdf)를 사용했다.
후보 Native 전체 쪽수는 6쪽, PDF는 6쪽이다. 선택 쪽 검토 상태는 `passed`다. 새로 확보한 문서의 전쪽 90% 보정은 작업지시에서 면제했으며 점수 미달·쪽수 차이를 통과로 바꾸지 않는다.

3쪽 차트의 복합 다각형 막대·DIB 패턴과 채움 경로를 보정했다. 가는 선은 PDF 래스터화에 따른 차이도 남는다.

- 물리 3쪽: raw 96.38065% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8027/native/review_003.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8027/native/overlay_003.png)

[전체 페이지 Native TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8027/native/silhouette.tsv)

Native의 고정 `upstream/devel` 대비 가능한 쪽별 raw 0.1%p 초과 하락은 없었다. 문서 쪽수가 다르면 공통 쪽 비교는 누락 쪽을 검증하지 못한다.
<!-- source-claim-native-end -->

## #8319 통합 전 동일 입력 대조

통합 merge의 첫 부모 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`에서 같은 입력과 Print PDF를 Native로 출력했다. 전체 쪽수는 RHWP 6쪽, PDF 6쪽이며 아래 선택 쪽 최저 raw는 93.01369%다. 선택 영역의 일치율은 실제 주장 동작의 개선이나 전체 문서 통과를 단독으로 입증하지 않는다.

- 물리 3쪽: [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8027/native-before-integration/review_003.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8027/native-before-integration/overlay_003.png)

<!-- source-claim-wasm-start -->
## 추가 확보 문서의 fresh WASM 대조

동일 원문과 한컴 Print를 96dpi에서 비교했다. 전체 6쪽의 raw 최저는 94.25969%이며, 90% 미만 쪽은 없음이다. 선정한 주장 영역의 결과는 아래와 같다.

- 물리 3쪽: raw 96.65106% · [review](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8027/wasm/review_003.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8027/wasm/overlay_003.png)

[전체 페이지 WASM TSV](../assets/pr8319_evidence_alignment_20261027/source-claims/pr8027/wasm/silhouette.tsv)

고정 base와 비교 가능한 같은 번호의 쪽에서 raw 0.1%p 초과 하락은 없었다. 원문 전체 페이지의 기존 90% 미만은 작업지시에 따라 명시하며 통과로 표현하지 않는다.
<!-- source-claim-wasm-end -->
