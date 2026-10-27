# PR #8053 구현·검증 기록 정정

## 현재 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

## 구현과 과거 실제 CI

머리말 내부 용지 기준 shape의 좌표를 종이 원점으로 다시 환원하지 않고 해당 머리말 frame에서 소비한다. 음수 vertical_offset을 signed HWPUNIT로 해석하며 본문과 머리말의 서로 다른 frame을 구분한다. header_paper_shape_frame_origin은 원본 빈 문서의 여백과 입력 offset으로 계산한 결과를 검사한다. 이는 임의 실물 y값의 핀이 아니라 source 단위의 좌표계 계약이다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 엔진·테스트·baseline은 이번 변경에서 수정하지 않았다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8053_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

위 대체 입력에서 전체 페이지 Native/fresh WASM TSV와 같은 페이지 review/overlay를 새로 산출했다. 배경 실물 문서의 검증과 구분한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440919692`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.
