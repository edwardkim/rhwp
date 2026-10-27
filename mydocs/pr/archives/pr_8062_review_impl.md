# PR #8062 구현·검증 기록 정정

## 현재 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

## 구현과 과거 실제 CI

저장된 개체 줄의 잉크 높이와 다음 줄 원점까지의 흐름 걸음은 다르다. overlapping_object_lines_advance_by_the_stored_vpos_step은 조판 부호 줄과 큰 TAC shape를 둔 뒤 lineHeightSum+lineSpacingSum이 원본 마지막 vpos+height의 범위를 이중 합산하지 않는지 검사한다. 원본 HWPUNIT에서 환산한 점유 계약을 절대 화면 좌표 핀으로 오인해 제거하지 않는다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 엔진·테스트·baseline은 이번 변경에서 수정하지 않았다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8062_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

위 대체 입력에서 전체 페이지 Native/fresh WASM TSV와 같은 페이지 review/overlay를 새로 산출했다. 배경 실물 문서의 검증과 구분한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440929829`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.
