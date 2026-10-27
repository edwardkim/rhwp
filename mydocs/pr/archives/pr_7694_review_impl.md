# PR #7694 구현·검증 기록 정정

## 현재 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

## 구현과 과거 실제 CI

move_inline_control_native는 배포 문서·원본/대상 주소·잠금·logical caret를 먼저 검증하고 영향 구역 복제본에서 remove/insert를 실행한다. 실제 문서 commit 이전에 실패할 수 있는 검사를 끝내는 순서를 확인한다. 같은 위치 no-op은 이벤트를 발행하지 않으며, source 제거 후 sibling 경로와 목적지 offset을 다시 계산한다. inline_control_move의 18개 검사와 분리한 유효 저장본 검사 1개는 Unicode·필드·형광펜·범위 태그·BinData·실제 OOXML 차트 payload 및 공개 HWP/HWPX 저장을 포함한다. 이동 전후 실제 차트의 과거 시각 미달은 뒤의 OLE/차트 공통 보정으로 해결했으며, 원 이동 API와 차트 paint 보정의 범위를 구분한다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 엔진·테스트·baseline은 이번 변경에서 수정하지 않았다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_7694_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

공개 재현 계약의 검증 범위만 인정한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440883725`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.
