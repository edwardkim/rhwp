# PR #8093 구현·검증 기록 정정

## 현재 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

## 구현과 과거 실제 CI

Form의 treatAsChar를 논리 한 칸으로 투영하고, inline_control_projection·equation_line_metrics·inline_box_rows·nested_table_frames가 Form 메트릭을 소비한다. 같은 줄 본문과 개체의 높이·기준선 합집합을 사용하며 정상 저장 LineSeg를 임의로 덮지 않는다. 기존 화면 폭 90px 하한은 원본 Form 폭과 실제 caret 구간의 관계로 바꿨다. 입력·Enter 뒤 문단 소유도 유지하고 양쪽 100% 페이지의 독립 가/나 원점만 ±2px로 보호한다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 엔진·테스트·baseline은 이번 변경에서 수정하지 않았다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8093_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

공개 재현 계약의 검증 범위만 인정한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440934691`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.
