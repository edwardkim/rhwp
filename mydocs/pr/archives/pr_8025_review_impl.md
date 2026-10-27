# PR #8025 구현·검증 기록 정정

## 현재 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

## 구현과 과거 실제 CI

CanvasTextFont::for_positioned_text는 style.bold만 보지 않고 is_visually_bold를 사용한다. 실제 faux-bold 획을 이미 적용하는 face는 굵기 descriptor를 중복 적용하지 않는 조건도 유지한다. HY헤드라인M·HY견고딕·한양견고딕·HYGothic-Extra와 일반 함초롬바탕의 적용·비적용을 같은 descriptor 경로에서 검사한다.

tests/cases/canvas_heavy_display_face_bold.rs. Canvas 실제 출력 증거는 해당 글꼴·producer의 Print 대조와 별도로 연결해야 하며 descriptor 검사만으로 수용하지 않는다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 엔진·테스트·baseline은 이번 변경에서 수정하지 않았다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8025_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

공개 재현 계약의 검증 범위만 인정한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440890024`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.
