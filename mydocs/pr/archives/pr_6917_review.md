# PR #6917 검토 기록: 비첫 문단의 셀 내부 Square 그림 앵커

## 판정: 승인

표 셀의 첫 문단이 아닌 Square 그림을 호스트 문단 위치에 두는 변경을 수용한다.

**원 PR 변경 범위의 로컬 수용 판정이다.** 후속 재검증으로 #6910·#6926의 보류 사유를 해소했고 전체 회귀 9,327개가 통과했다. 원격 통합 PR 생성·머지 승인과 최종 원격 CI gate는 별도다.

## 검토 기준

- 검토일: 2026-09-09, macOS 로컬.
- 원 PR: [#6917](https://github.com/edwardkim/rhwp/pull/6917), 기여자 `planet6897`.
- 연결 이슈: [#6892](https://github.com/edwardkim/rhwp/issues/6892).
- 확인한 최신 원격 head: `1af8d3eb961cd9d27af62fa1e335a490da439f9e`.
- 로컬 적용 커밋: `7721bee92`.
- 통합 브랜치: `review/planet6897-batch-20260909`.
- 테스트·재산출 기준 소스: `976ecf3f48095ccab2d01fc679dab0b91899a69f`.
- 원 PR 최신 head의 CI 그린을 확인했다. 원 PR의 CI와 아직 생성하지 않은 통합 PR의 CI는 다르다.
- [공통 구현·검증 기록](pr_6886_6932_planet6897_impl.md) / [시각 증적 기록](pr_6886_6932_planet6897_visual_sweep.md).

## 변경 범위와 코드 검토

첫 문단 또는 TopAndBottom 문단의 기존 처리는 유지하고 비첫 문단 Square 객체를 잘못된 빈 문단 특례에서 제외한다.

## 실행한 검증

`issue_6892_cell_float_host_para_anchor` 관련 회귀 통과. 8쪽의 공정도가 설명문 아래에 놓이는 것을 현재 통합본과 한컴 PDF로 대조했다. 첫 문단 그림 대조군도 관련 회귀 범위에 포함된다.

최종 메인터너 보정 후 공통 전체 회귀는 9,327개 통과·실패 0·별도 건너뜀 46개다. focused 5개, fmt, native/WASM/workspace all-targets Clippy, workspace 빌드와 manifest 검사도 통과했다. 제품 소스가 같아 기존 Native Skia·WASM package·Studio·TypeScript·Chrome·시각 결과를 재사용했으며, 이전 실패와 재실행 범위는 공통 기록에 구분했다.

## 입력 문서와 시각 증적

- 입력: `samples/issue6892/156726122-recycling-press-release.hwpx`.
- 실제 검토 범위: 8쪽 / 전체 8쪽.
- 기준 PDF: [samples/issue6892/pdf/156726122-recycling-press-release-2020.pdf](../../../samples/issue6892/pdf/156726122-recycling-press-release-2020.pdf).

![현재 통합본 #6917 증적](../assets/pr_6886_6932_planet6897_20260909/pr6917-p008.png)

## 잔여 사항과 수용 범위

기여자가 포함한 기존 PDF를 재사용했다. 이 문서는 화면의 미세 글꼴·선 두께까지 완전 일치한다는 판정이 아니다.

## 원 PR·이슈 코멘트 계획

8쪽 공정도의 문단 아래 배치와 첫 문단 대조 계약을 기록한다. 현재 비교 PNG와 기존 한컴 PDF를 직접 연결한다.

- 지금은 원격 댓글·review·close를 수행하지 않았다.
- 모든 통합 gate와 merge 후 devel CI 완료 뒤 실제 merge SHA·PR/devel CI URL을 기록한다.
- 증적은 공통 기록의 댓글 템플릿처럼 commit 고정 raw 이미지로 본문에 직접 표시한다. `/tmp` 경로·로그 링크를 공개 증적으로 사용하지 않는다.
- 기존 수용 댓글이 있으면 본문을 수정하고 중복 등록하지 않는다. UTF-8 body-file 방식으로 게시한 뒤 API로 실제 본문을 재조회한다.
- 연결 이슈의 실제 범위가 남으면 전체 해결로 닫지 않는다. 기여자 fork 브랜치는 보존한다.
