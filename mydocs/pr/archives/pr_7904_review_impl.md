# PR #7904 구현·검증 기록 정정

## 현재 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

## 구현과 과거 실제 CI

본문 끝 인라인 개체 뒤의 IME 경로에서 삽입은 logical, 삭제와 예약 서식 범위는 scalar로 소비한다. replaceTextAtRaw는 권한·범위를 확인한 뒤 원문 fragment를 캡처하고 batch 변경 실패 시 복원한다. composition 완료에서 실제 원문과 조합 길이를 확인한 경우에만 이력을 기록하며 Undo·취소·Redo의 fragment 소유권을 분리한다. 일반 본문 중간·셀·머리말·각주의 fast path까지 새 동작이 해결했다고 주장하지 않는다. 실제 Studio behavior runner·routing guard와 브라우저 검증을 원 작성자의 과거 CI와 구분한다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 엔진·테스트·baseline은 이번 변경에서 수정하지 않았다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_7904_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

공개 재현 계약의 검증 범위만 인정한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440887549`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.
