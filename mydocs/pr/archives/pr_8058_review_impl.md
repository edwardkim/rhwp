# PR #8058 구현·검증 기록 정정

## 현재 판정

**메인터너 보정 후 수용 가능 — 공개 재현 계약의 검증 범위로 한정.**

## 구현과 과거 실제 CI

canvas_cluster_fit은 양수 배분 간격을 실제 glyph fit 폭과 분리한다. 좁은 ASCII를 늘리지 않고 가운데 놓으며, box가 넓어도 잉크가 advance 안에 들면 기호를 누르지 않는다. 한글/한자와 fixed_glyph_advances의 원본 점유 계약은 별도 분기다. halfwidth punctuation과 Canvas/Skia의 동일 helper 소비를 확인하며 임의의 허용 배율로 시각 미달을 감추지 않는다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 엔진·테스트·baseline은 이번 변경에서 수정하지 않았다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8058_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

공개 재현 계약의 검증 범위만 인정한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440927420`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.
