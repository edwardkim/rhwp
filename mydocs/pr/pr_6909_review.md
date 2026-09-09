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

- 생성된 PDF, sweep PNG, 비교 이미지, JSON은 검토 중간 산출물이다.
- 이 PR 또는 후속 통합 PR에는 해당 산출물을 stage, commit, 첨부하지 않는다. review 문서에는 재현 조건과 판독 결과만 남긴다.

## 판정

코드 수준의 차단 결함은 발견하지 못했다. 누적 통합 PR 생성은 가능하며, 최종 GitHub 승인과 병합은 통합 head의 CI가 녹색인 상태에서 진행한다.
