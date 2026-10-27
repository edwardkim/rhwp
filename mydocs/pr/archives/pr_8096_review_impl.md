# PR #8096 구현·검증 기록 정정

## 현재 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

## 구현과 과거 실제 CI

equation_line_metrics::refresh_single_tac_host_row는 하나의 TAC Form/Equation만 있는 빈 host의 실제 개체 높이·baseline을 소비한다. 정상 저장 줄을 무조건 합성으로 재분류하지 않는다. form_only_paragraph_without_lineseg는 HWPX 재열기에서 LineSeg가 여전히 없는 것을 먼저 확인한 뒤 다섯 Form 종류가 순서대로 출력되고 각 원본 높이+600HU 문단 간격만큼 전진하는지 검사한다. Native034의 개선 기록과 fresh WASM의 최종 증거는 구분하며 배경/BorderType 기본값 보정은 공통 FormAppearance 소비로 연결한다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 엔진·테스트·baseline은 이번 변경에서 수정하지 않았다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8096_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

공개 재현 계약의 검증 범위만 인정한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440936868`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.
