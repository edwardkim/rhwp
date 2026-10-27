# PR #8031 구현·검증 기록 정정

## 현재 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

## 구현과 과거 실제 CI

collect_page_number_pos는 문단·control·cell을 역순으로 찾아 기존 마지막 선언 우선순위를 중첩 표 셀까지 확장한다. 머리말·꼬리말의 내용 자체를 구역 선언으로 승격하지 않는다. 새 typeset finalize와 legacy pagination engine이 같은 collector를 소비하며, 공개 render tree의 실제 -1- 표시와 legacy 결과의 page_number_pos를 각각 검사한다.

tests/cases/page_number_pos_in_table_cell.rs의 공개 표시 검사와 legacy paginator 검사. 함수 존재만으로 두 소비 경로가 동일하다고 판정하지 않고 finalize 호출을 대조했다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 엔진·테스트·baseline은 이번 변경에서 수정하지 않았다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8031_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

위 대체 입력에서 전체 페이지 Native/fresh WASM TSV와 같은 페이지 review/overlay를 새로 산출했다. 배경 실물 문서의 검증과 구분한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440897417`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.
