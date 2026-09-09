# PR #6906 검토 기록: 짝이 없는 UTF-16 서로게이트 보존

## 최신 시각 증적 갱신 (2026-09-09)

Visual Sweep 보정 `4fe5530d6`을 사용한 [공통 재산출 기록](pr_6886_6932_planet6897_visual_sweep.md)을 적용했다. 제품 소스·기준 PDF와 기존 제품 수용 범위는 변경하지 않았다. 아래 이전 PNG 유지 또는 글꼴 깨짐 서술과 이전 래스터 측정값은 이 재산출 기록으로 대체한다.

이 PR에 독립적인 새 시각 증적을 생성한 것으로 기록하지 않는다. 관련 통합 증적 범위는 공통 기록을 따른다.


## 판정: 승인

짝이 없는 UTF-16 서로게이트를 U+25A1로 보존하고 정상 쌍은 유지하는 파서 계약을 수용한다.

**원 PR 변경 범위의 로컬 수용 판정이다.** 후속 재검증으로 #6910·#6926의 보류 사유를 해소했고 전체 회귀 9,327개가 통과했다. 원격 통합 PR 생성·머지 승인과 최종 원격 CI gate는 별도다.

## 검토 기준

- 검토일: 2026-09-09, macOS 로컬.
- 원 PR: [#6906](https://github.com/edwardkim/rhwp/pull/6906), 기여자 `planet6897`.
- 연결 이슈: [#6873](https://github.com/edwardkim/rhwp/issues/6873).
- 확인한 최신 원격 head: `891eb812d73bda8762e2ff87a4313ca376b45f4b`.
- 로컬 적용 커밋: `cff474c8d, 7942312e1, eaea84110; PDF 경로 정리 8b78b954f`.
- 통합 브랜치: `review/planet6897-batch-20260909`.
- 테스트·재산출 기준 소스: `976ecf3f48095ccab2d01fc679dab0b91899a69f`.
- 원 PR 최신 head의 CI 그린을 확인했다. 원 PR의 CI와 아직 생성하지 않은 통합 PR의 CI는 다르다.
- [공통 구현·검증 기록](pr_6886_6932_planet6897_impl.md) / [시각 증적 기록](pr_6886_6932_planet6897_visual_sweep.md).

## 변경 범위와 코드 검토

한컴 HWP→HWPX 결과의 문자 내용을 기준으로 한다. PDF의 대체 글리프 모양을 문자열 의미의 정답으로 사용하지 않는다.

## 실행한 검증

`issue_6873` 관련 회귀가 전체 실행에서 통과했다. 충주·경주 원본과 제공된 한컴 HWPX, 정상 서로게이트 쌍 대조군을 사용하는 계약을 확인했다.

최종 메인터너 보정 후 공통 전체 회귀는 9,327개 통과·실패 0·별도 건너뜀 46개다. focused 5개, fmt, native/WASM/workspace all-targets Clippy, workspace 빌드와 manifest 검사도 통과했다. 제품 소스가 같아 기존 Native Skia·WASM package·Studio·TypeScript·Chrome·시각 결과를 재사용했으며, 이전 실패와 재실행 범위는 공통 기록에 구분했다.

## 입력 문서와 시각 증적

- 입력: `samples/issue6873/19211507-chungju-paid-restroom-certificate.hwp 및 18096141-gyeongju-gas-subsidy-plan.hwp`.
- 실제 검토 범위: 각 1쪽, 문자열 정본은 samples/issue6873/hwpx의 두 파일.
- 기준 PDF: [pdf/19211507-chungju-paid-restroom-certificate-2020.pdf](../../../pdf/19211507-chungju-paid-restroom-certificate-2020.pdf).

문자 정본은 `samples/issue6873/hwpx/`의 두 한컴 HWPX다. 두 번째 PDF는 [경주 기준 PDF](../../../pdf/18096141-gyeongju-gas-subsidy-plan-2020.pdf)다.

## 잔여 사항과 수용 범위

이슈에 언급된 세 번째 3690000-202400022 문서는 이번 실물 재검증 범위가 아니다. PDF 두 개는 root pdf/로 이동한 기존 보정을 유지했으며 새 변환본으로 바꾸지 않았다.

## 원 PR·이슈 코멘트 계획

U+25A1 문자열 계약, 검증한 두 문서와 정상 쌍 대조군을 적는다. 기여자 제공 이미지를 사용할 경우 새 통합본 캡처가 아님을 표시하고 HWPX 정본 링크를 함께 제공한다.

- 지금은 원격 댓글·review·close를 수행하지 않았다.
- 모든 통합 gate와 merge 후 devel CI 완료 뒤 실제 merge SHA·PR/devel CI URL을 기록한다.
- 증적은 공통 기록의 댓글 템플릿처럼 commit 고정 raw 이미지로 본문에 직접 표시한다. `/tmp` 경로·로그 링크를 공개 증적으로 사용하지 않는다.
- 기존 수용 댓글이 있으면 본문을 수정하고 중복 등록하지 않는다. UTF-8 body-file 방식으로 게시한 뒤 API로 실제 본문을 재조회한다.
- 연결 이슈의 실제 범위가 남으면 전체 해결로 닫지 않는다. 기여자 fork 브랜치는 보존한다.
