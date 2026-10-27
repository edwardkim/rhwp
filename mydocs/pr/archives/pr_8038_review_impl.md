# PR #8038 구현·검증 기록 정정

## 현재 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

## 구현과 과거 실제 CI

clip 확대는 글줄 bbox나 caret를 키우는 방식과 구분한다. 실제 SVG top underline 획 반 두께와 결합 강조점 U+0307의 변환 후 em 상자를 본문 clip이 포함하는지 확인하고, 본문 하단·caret가 보존되는지 검사한다. 원 PR의 점 글리프 가정을 그대로 사용하지 않고 raised_dot_clip 및 emphasis_source_style_boundary의 독립 Print에서 확인한 글꼴/잉크 범위를 따른다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 엔진·테스트·baseline은 이번 변경에서 수정하지 않았다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8038_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

공개 재현 계약의 검증 범위만 인정한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440905167`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.
