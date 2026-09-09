# PR #6905 검토 기록: 문서 열기 직후 리사이즈의 스크롤 앵커

## 판정: 승인

최초 문서 로드의 상단 정렬을 보존하면서 사용자 스크롤 뒤에는 기존 앵커 보정을 수행하는 변경을 수용한다.

**원 PR 변경 범위의 로컬 수용 판정이다.** 후속 재검증으로 #6910·#6926의 보류 사유를 해소했고 전체 회귀 9,327개가 통과했다. 원격 통합 PR 생성·머지 승인과 최종 원격 CI gate는 별도다.

## 검토 기준

- 검토일: 2026-09-09, macOS 로컬.
- 원 PR: [#6905](https://github.com/edwardkim/rhwp/pull/6905), 기여자 `planet6897`.
- 연결 이슈: [#6902](https://github.com/edwardkim/rhwp/issues/6902).
- 확인한 최신 원격 head: `e3802579ace396ad3a1553e4033c876c3a811c40`.
- 로컬 적용 커밋: `64782bb0e (원 커밋 91cf1982c0cd1b91e01371eb217616c5ae1cd048과 최신 head의 patch-id 동일)`.
- 통합 브랜치: `review/planet6897-batch-20260909`.
- 테스트·재산출 기준 소스: `976ecf3f48095ccab2d01fc679dab0b91899a69f`.
- 원 PR 최신 head의 CI 그린을 확인했다. 원 PR의 CI와 아직 생성하지 않은 통합 PR의 CI는 다르다.
- [공통 구현·검증 기록](pr_6886_6932_planet6897_impl.md) / [시각 증적 기록](pr_6886_6932_planet6897_visual_sweep.md).

## 변경 범위와 코드 검토

CanvasView의 로드·빈 문서·준비 경로에 최초 리사이즈 억제 상태를 두고 사용자 스크롤 시 해제한다. CI 워크플로 추가 변경은 이 PR 범위가 아니다.

## 실행한 검증

Studio 관련 계약 검사 통과. 새 WASM으로 Chrome 임시 프로브를 실행했다. 1400×1000에서 문서 열기 후 1385×940 리사이즈: scrollTop 0→0. 사용자 스크롤 600 후 같은 리사이즈: 630, 기대값 630. 배율은 모두 0.7287305122494432. 기존 맞춤 배율 E2E TC1~TC6도 통과했다.

최종 메인터너 보정 후 공통 전체 회귀는 9,327개 통과·실패 0·별도 건너뜀 46개다. focused 5개, fmt, native/WASM/workspace all-targets Clippy, workspace 빌드와 manifest 검사도 통과했다. 제품 소스가 같아 기존 Native Skia·WASM package·Studio·TypeScript·Chrome·시각 결과를 재사용했으며, 이전 실패와 재실행 범위는 공통 기록에 구분했다.

## 입력 문서와 시각 증적

- 입력: `biz_plan.hwp (Studio /samples/biz_plan.hwp 경로)`.
- 실제 검토 범위: 최초 페이지와 스크롤 상태.
- 한컴 PDF 전체 일치 판정은 해당 검증의 근거로 사용하지 않았다.

## 잔여 사항과 수용 범위

닫힌 #6908의 E2E·CI 변경은 이미 제외됐다. 이번 임시 프로브는 /tmp에만 두었고 package.json 또는 CI에 추가하지 않았다. 정지 PNG만으로 스크롤 전이 성공을 주장하지 않는다.

## 원 PR·이슈 코멘트 계획

입력 문서, 브라우저 크기, 배율, scrollTop 0→0 및 600→630을 본문에 직접 적는다. 통합 PR의 CI 변경으로 잘못 소개하지 않는다.

- 지금은 원격 댓글·review·close를 수행하지 않았다.
- 모든 통합 gate와 merge 후 devel CI 완료 뒤 실제 merge SHA·PR/devel CI URL을 기록한다.
- 증적은 공통 기록의 댓글 템플릿처럼 commit 고정 raw 이미지로 본문에 직접 표시한다. `/tmp` 경로·로그 링크를 공개 증적으로 사용하지 않는다.
- 기존 수용 댓글이 있으면 본문을 수정하고 중복 등록하지 않는다. UTF-8 body-file 방식으로 게시한 뒤 API로 실제 본문을 재조회한다.
- 연결 이슈의 실제 범위가 남으면 전체 해결로 닫지 않는다. 기여자 fork 브랜치는 보존한다.
