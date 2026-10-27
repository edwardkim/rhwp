# PR #8035 구현·검증 기록 정정

## 현재 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

## 구현과 과거 실제 CI

개체 삭제 뒤 텍스트, char_offsets 및 char_shapes 경계를 개체 없는 동일 문단과 비교한다. 맨 앞 그림·중간 그림·선두 수식과 서식 시작/끝 경계가 다른 사례를 구분한다. HWP/HWPX 저장 정규화 때문에 바이트 전체를 맞추지 않고, 각각의 형식으로 왕복한 독립 대조 문단과 글자모양 경계를 비교한다.

tests/cases/delete_inline_control_char_shapes.rs의 3함수와 두 저장 형식. 공통 paragraph 슬롯 제거를 picture/equation/shape/table 삭제 소비자가 호출하는 변경 범위를 별도로 대조한다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 엔진·테스트·baseline은 이번 변경에서 수정하지 않았다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8035_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

공개 재현 계약의 검증 범위만 인정한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440901569`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.
