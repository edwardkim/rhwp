# PR #6932 검토 기록: 큰 셀 내부 객체의 저장 높이 신뢰 조건

## 판정: 승인

비흐름 객체의 실제 흐름 높이를 고려하여 잘못 저장된 줄 높이를 신뢰하지 않는 변경을 수용한다.

**원 PR 변경 범위의 로컬 수용 판정이다.** 후속 재검증으로 #6910·#6926의 보류 사유를 해소했고 전체 회귀 9,327개가 통과했다. 원격 통합 PR 생성·머지 승인과 최종 원격 CI gate는 별도다.

## 검토 기준

- 검토일: 2026-09-09, macOS 로컬.
- 원 PR: [#6932](https://github.com/edwardkim/rhwp/pull/6932), 기여자 `planet6897`.
- 연결 이슈: [#6912](https://github.com/edwardkim/rhwp/issues/6912).
- 확인한 최신 원격 head: `82632a01c2e009e74c50d0a8692e1e1b2997e08c`.
- 로컬 적용 커밋: `e263d6a48, 976ecf3f4`.
- 통합 브랜치: `review/planet6897-batch-20260909`.
- 테스트·재산출 기준 소스: `976ecf3f48095ccab2d01fc679dab0b91899a69f`.
- 원 PR 최신 head의 CI 그린을 확인했다. 원 PR의 CI와 아직 생성하지 않은 통합 PR의 CI는 다르다.
- [공통 구현·검증 기록](pr_6886_6932_planet6897_impl.md) / [시각 증적 기록](pr_6886_6932_planet6897_visual_sweep.md).

## 변경 범위와 코드 검토

거대 객체의 흐름 높이가 저장된 line height와 맞지 않으면 저장 위치 특례를 사용하지 않는다. OLE 없는 최소 사각형 문서와 실제 OLE 문서를 구분하여 확인했다.

## 실행한 검증

최소 문서의 셀 상대 위치·셀 하단 경계·앵커 줄 위치 회귀 2개가 통과했다. 현재 2쪽 사각형이 셀 위쪽부터 그려지는 것을 정본과 직접 대조했다. 실제 OLE 원본은 #6910의 4쪽 증적을 함께 참조한다.

최종 메인터너 보정 후 공통 전체 회귀는 9,327개 통과·실패 0·별도 건너뜀 46개다. focused 5개, fmt, native/WASM/workspace all-targets Clippy, workspace 빌드와 manifest 검사도 통과했다. 제품 소스가 같아 기존 Native Skia·WASM package·Studio·TypeScript·Chrome·시각 결과를 재사용했으며, 이전 실패와 재실행 범위는 공통 기록에 구분했다.

## 입력 문서와 시각 증적

- 입력: `samples/issue6912/156564340-ip-dispute-mediation-poster-slice.hwpx`.
- 실제 검토 범위: 최소 문서 2쪽 / 전체 2쪽.
- 기준 PDF: [pdf/156564340-ip-dispute-mediation-poster-slice-2020.pdf](../../../pdf/156564340-ip-dispute-mediation-poster-slice-2020.pdf).

![현재 통합본 #6932 증적](../assets/pr_6886_6932_planet6897_20260909/pr6932-p002.png)

## 잔여 사항과 수용 범위

내부 패딩 약 0.93px 차이는 이 회귀의 ±2px 근거에 포함된다. 전체 프레임 절대 y까지 맞춘 것은 아니다. 기존 메인터너 8b78b954f와 같은 취지의 흐름 높이 guard가 중복되어 있으나 이번 검토에서 소스를 다시 변경하지 않았다.

## 원 PR·이슈 코멘트 계획

최소 문서의 사각형과 실제 #6910 OLE 포스터를 구별하고, 저장 높이 신뢰 조건의 복원 및 작은 패딩 잔여를 함께 기록한다.

- 지금은 원격 댓글·review·close를 수행하지 않았다.
- 모든 통합 gate와 merge 후 devel CI 완료 뒤 실제 merge SHA·PR/devel CI URL을 기록한다.
- 증적은 공통 기록의 댓글 템플릿처럼 commit 고정 raw 이미지로 본문에 직접 표시한다. `/tmp` 경로·로그 링크를 공개 증적으로 사용하지 않는다.
- 기존 수용 댓글이 있으면 본문을 수정하고 중복 등록하지 않는다. UTF-8 body-file 방식으로 게시한 뒤 API로 실제 본문을 재조회한다.
- 연결 이슈의 실제 범위가 남으면 전체 해결로 닫지 않는다. 기여자 fork 브랜치는 보존한다.
