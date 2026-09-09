# PR #6886 검토 기록: 회전 그림의 선언 프레임 복원

## 판정: 승인

회전 270도 그림의 너비·높이를 중복 전치하지 않고 선언 프레임 중심에 배치하는 변경을 수용한다. 문서 전체의 글꼴 일치까지 승인한 것은 아니다.

**원 PR 변경 범위의 로컬 수용 판정이다.** 후속 재검증으로 #6910·#6926의 보류 사유를 해소했고 전체 회귀 9,327개가 통과했다. 원격 통합 PR 생성·머지 승인과 최종 원격 CI gate는 별도다.

## 검토 기준

- 검토일: 2026-09-09, macOS 로컬.
- 원 PR: [#6886](https://github.com/edwardkim/rhwp/pull/6886), 기여자 `planet6897`.
- 연결 이슈: [#6866](https://github.com/edwardkim/rhwp/issues/6866).
- 확인한 최신 원격 head: `438c932a3d75c2fb6b962d2e106f21dd6c81058d`.
- 로컬 적용 커밋: `28c54bcd6, aca8718dd`.
- 통합 브랜치: `review/planet6897-batch-20260909`.
- 테스트·재산출 기준 소스: `976ecf3f48095ccab2d01fc679dab0b91899a69f`.
- 원 PR 최신 head의 CI 그린을 확인했다. 원 PR의 CI와 아직 생성하지 않은 통합 PR의 CI는 다르다.
- [공통 구현·검증 기록](pr_6886_6932_planet6897_impl.md) / [시각 증적 기록](pr_6886_6932_planet6897_visual_sweep.md).

## 변경 범위와 코드 검토

12쪽 화살표의 세로형 프레임을 복원한다. 각도 0도 그림과 선언 크기의 기존 계약은 유지한다.

## 실행한 검증

`issue_6866_rotated_picture_box` 관련 회귀 통과. 12쪽을 현재 통합 CLI로 재산출하여 한컴 PDF의 화살표 방향·크기와 대조했다.

최종 메인터너 보정 후 공통 전체 회귀는 9,327개 통과·실패 0·별도 건너뜀 46개다. focused 5개, fmt, native/WASM/workspace all-targets Clippy, workspace 빌드와 manifest 검사도 통과했다. 제품 소스가 같아 기존 Native Skia·WASM package·Studio·TypeScript·Chrome·시각 결과를 재사용했으며, 이전 실패와 재실행 범위는 공통 기록에 구분했다.

## 입력 문서와 시각 증적

- 입력: `samples/issue6866/156627451-quantum-science-press-note.hwpx`.
- 실제 검토 범위: 12쪽.
- 기준 PDF: [pdf/156627451-quantum-science-press-note-2020.pdf](../../../pdf/156627451-quantum-science-press-note-2020.pdf).

![현재 통합본 #6886 증적](../assets/pr_6886_6932_planet6897_20260909/pr6886-p012.png)

## 잔여 사항과 수용 범위

일부 굵은 본문에 사각 물음표 모양의 대체 글리프가 보인다. 원인과 변경 귀속은 미분리이며 화살표 수정으로 텍스트 표시까지 해결됐다고 주장하지 않는다.

## 원 PR·이슈 코멘트 계획

회전 프레임 복원과 12쪽 화살표만 검증 범위로 기재한다. 글꼴 표시 잔여를 숨기지 않는다.

- 지금은 원격 댓글·review·close를 수행하지 않았다.
- 모든 통합 gate와 merge 후 devel CI 완료 뒤 실제 merge SHA·PR/devel CI URL을 기록한다.
- 증적은 공통 기록의 댓글 템플릿처럼 commit 고정 raw 이미지로 본문에 직접 표시한다. `/tmp` 경로·로그 링크를 공개 증적으로 사용하지 않는다.
- 기존 수용 댓글이 있으면 본문을 수정하고 중복 등록하지 않는다. UTF-8 body-file 방식으로 게시한 뒤 API로 실제 본문을 재조회한다.
- 연결 이슈의 실제 범위가 남으면 전체 해결로 닫지 않는다. 기여자 fork 브랜치는 보존한다.
