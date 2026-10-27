# PR #8029 구현·검증 기록 정정

## 현재 판정

**증적 대응 정정 — 원문 미첨부·단위 계약의 검증 범위로 한정.**

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 현재 실행 근거는 파일을 소비하지 않는 단위 계약에 한정한다. 유사 문서로 대체한 시각 증거가 있는 것으로 표시하지 않으며 배경 실물 문서의 직접 검증을 주장하지 않는다.**

## 구현과 과거 실제 CI

정확한 이름/별칭으로 metric_index를 먼저 조회하고, 실패한 경우에만 마지막 하이픈의 Regular·Medium·Bold를 분리한다. Bold 접미사는 기존 bold 요청과 OR로 합성하며 family에도 같은 별칭 규칙을 적용한다. 등록된 완전한 face를 우선하므로 정상 exact lookup을 덮지 않고, 알 수 없는 family나 Light 같은 미지원 접미사를 임의로 추정하지 않는다.

tests/cases/metric_postscript_weight_suffix.rs: Regular/Medium 동일 metric, Bold의 실제 bold metric, SourceHanSansK 별칭, 미등록 family 및 Light 거절.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 엔진·테스트·baseline은 이번 변경에서 수정하지 않았다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8029_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

파일을 소비하지 않는 기존 단위 계약만 검증했다. 실물 문서의 시각 완료 또는 대체 문서의 Visual Sweep 완료를 주장하지 않는다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440892871`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.
