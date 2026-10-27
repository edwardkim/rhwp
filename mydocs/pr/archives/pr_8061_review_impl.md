# PR #8061 구현·검증 기록 정정

## 현재 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

## 구현과 과거 실제 CI

- [anchored_cell_paint.rs](../../../src/renderer/layout/anchored_cell_paint.rs)의 그림 생성 후
  layer 부여 → 셀 순서 조립 → 같은 쪽 도형과 묶음 정렬 → 기존 paint replay가 소비한다.
- [render_tree.rs](../../../src/renderer/render_tree.rs)의 내부 묶음은 `serde(skip)`이며
  임시 NodeId로 z-order를 정하지 않는다. 편집 주소와 절대 좌표는 유지한다.
- [새 회귀](../../../tests/cases/verified_cell_picture_layer_scope.rs)의 8개 조건은 셀 주소,
  하나의 bitmap·PHOTO 보존, 원본 단위의 포함, 독립 Print 좌표 ±2px, 실제 그리기 순서를
  검사한다. Native 전용 1개는 같은 8조건의 실제 PNG 가림 색상도 검사한다.
- 잘못된 이전 `cell_behind_text_picture_above_paper_shapes.rs` 검사는 삭제했다.
  과거 후보 `61528013511c87c732524f91c6f224286a055337`에서 이 검사가 PASS했던 것은
  잘못된 가정을 검출하지 못한 결과다. 해당 PASS를 현재 결함의 검증 근거로 쓰지 않는다.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 엔진·테스트·baseline은 이번 변경에서 수정하지 않았다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8061_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

위 대체 입력에서 전체 페이지 Native/fresh WASM TSV와 같은 페이지 review/overlay를 새로 산출했다. 배경 실물 문서의 검증과 구분한다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440928574`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.
