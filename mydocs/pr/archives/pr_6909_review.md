# PR #6909 검토 기록

## 대상

- 원 PR: #6909 `Task #6806: 도형 리사이즈가 생성 시 크기(original_*)를 덮지 않게 한다`
- 기여자 head: `b0ab0ea8ddbe7b8226ecb908df45cbae36a81e20`
- 검토 기준: `upstream/devel` `ad84192839eb7b8534715ab085dd91d69a5c4a38`
- 누적 검토 head: `a6d4bdcf10a834cacfed0ba59e52714c168a4dd4`
- 누적 적용 commit: `a6d4bdcf1` (원 PR #6909), 충돌 없음
- GitHub 상태 재확인: head 유지, `CLEAN`, 원 PR CI 전체 성공

## 원 PR 변경 링크

- [원 PR의 고정 변경 commit](https://github.com/edwardkim/rhwp/pull/6909/changes/b0ab0ea8ddbe7b8226ecb908df45cbae36a81e20)
- [body/cell 도형 setter의 original 크기 보존](https://github.com/edwardkim/rhwp/pull/6909/changes/b0ab0ea8ddbe7b8226ecb908df45cbae36a81e20#diff-cf0909897bbf3e0166c2e32822d5aa3e25ed0753919e993a52c3c67f94b9035f)
- [도형 resize, undo, 저장 형상 회귀 시험](https://github.com/edwardkim/rhwp/pull/6909/changes/b0ab0ea8ddbe7b8226ecb908df45cbae36a81e20#diff-b49b7572bb6d24edfc48bcbe58d867f72a5656ab8151ee57f8be8af24a15394a)

## 변경 검토

- body와 cell 도형 속성 setter가 Line, Arc, Rectangle의 `current_width`와 `current_height`만 바꾸고 `original_width`와 `original_height`는 보존한다.
- Polygon과 Curve에만 있던 원본 크기 보존 우회 코드를 제거해 같은 규칙을 모든 도형 경로에 적용한다.
- Rectangle 좌표 계산은 새 current 크기를 기준으로 계속 갱신한다.
- 회귀는 실제 HWP의 선 도형을 절반 폭으로 조정하고, SVG 길이, undo 렌더링, 저장 전후 original 크기 보존을 확인한다.

## 검증

- 공통 lint, 전체 release-test, Native Skia 결과는 `pr_6883_review.md`의 같은 누적 head 검증을 따른다.
- focused: `issue_6806_shape_resize_original_scale` 3/3 성공.
- Docker compose wrapper는 이 호스트에 Docker 실행 파일이 없어 미실행이다. 다만 lint 묶음의 `wasm32-unknown-unknown` Clippy는 통과했으며, Docker 부재는 로컬 수용 판단의 차단 사유가 아니다. 통합 PR CI가 동일 범위를 다시 확인한다.
- object visual regression은 기준 브랜치 빌드 중 사용자 지시에 따라 중단됐다(exit 130). 결과를 성공으로 사용하지 않으며, 중간 산출물은 포함하지 않는다.

## 자산 정책

- #6909는 최종 PDF/PNG 증적을 새로 만들지 않는다. #6883 fixture의 한컴 PDF와 review PNG는 누적 renderer 검토의 기준 증적으로만 포함한다.
- 중단된 object visual regression의 원시 raster, compare·overlay 이미지, export SVG, render-tree·metric JSON은 중간 산출물로 제외한다.

## 판정

코드 수준의 차단 결함은 발견하지 못했다. 누적 통합 PR 생성은 가능하며, 최종 GitHub 승인과 병합은 통합 head의 CI가 녹색인 상태에서 진행한다.

## Merge 후 확정 기록

- 원 PR #6909의 변경은 통합 PR [#6944](https://github.com/edwardkim/rhwp/pull/6944)로 수용되었고, merge commit은 [`74d0a68b74919761cc30343f7511dbc5d0fe32d3`](https://github.com/edwardkim/rhwp/commit/74d0a68b74919761cc30343f7511dbc5d0fe32d3)이다.
- 통합 head `44b2c1def8dc43ff4bd4dfcce7264dfa725eb8a5`의 필수 CI는 모두 성공했다. 로컬 검토에서 PR #6909 관련 focused test 3/3도 통과했다.
- 이슈 #6806은 `original_width`/`original_height` 보존이라는 이번 하위 범위는 해결됐지만, 기존 메인터너 기록의 잔여 resize 경로 때문에 계속 OPEN으로 유지한다.
- 원 contributor PR #6909은 수용 경로와 이슈 범위 제한을 설명한 뒤 superseded로 close한다.

## Merge 후 contributor PR comment 계획

- 게시 대상: PR #6909과 이슈 #6806.
- 게시 순서: 이 아카이브 기록을 담은 문서 전용 후속 PR이 `devel`에 merge된 뒤, #6909에 통합 merge SHA와 CI/focused-test 결론을 남긴다. 이어서 #6806에 이번 수용 범위와 OPEN 유지 사유를 기록한다.
- 코멘트에는 #6909 원 head가 직접 merge된 것이 아니라 #6944에 cherry-pick 수용되었다는 점과, 이슈 전체 해결을 주장하지 않는다는 점을 명시한다.
