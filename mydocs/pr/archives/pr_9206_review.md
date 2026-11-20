# PR #9206 검토 — Studio 문단 간격 단위 보존

## 최종 판정

**보호 미달 두 쪽 보정 완료 — 대상 문서 Native·fresh WASM 160쪽이 페이지별 보호값을 충족했다.**

[메인터너 보정 원인](../assets/integration_open_prs_20261120/maintainer-cause.md) · [전쪽 결과](../assets/integration_open_prs_20261120/maintainer-repair/verification.json). 최종 통합 승인에는 새 코드의 전체 검증이 필요하다.
이 PR의 문단 간격 저장과 서식 복사 동작은 검증했다. [통합 검증](../assets/integration_open_prs_20261120/README.md).

## 접수와 적용

| 항목 | 확인 |
| --- | --- |
| 원 PR·작성자·리뷰어 | [#9206](https://github.com/edwardkim/rhwp/pull/9206) · `JinHo-von-Choi` · `jangster77` |
| 원 head·관련 이슈 | `f12f4e7d847acc37c1b8f9257b709b4f110cd3cf`, 열린 PR, MERGEABLE, 11파일 · [#7938](https://github.com/edwardkim/rhwp/issues/7938) |
| 적용 순서 | 세 번째. `467fe9b9db9e9faa8e07dc0e0cc6ddb5ad7ea8c9`부터 `576d6f1b3da02966fe107be513520885d2eaa716`까지 3개 |
| 고정 기준·제품 후보 | `0a437a7eefb19bfe81fc12866887e4548ea9554d` · `25f9587840d09b08af2b37be178ab93379a7b9dc` |

## 검증한 편집 계약

Studio의 입력·표시 pt와 WASM 저장 원시값을 구분한다. 실제 사용자가 간격을 바꿀 때
저장 단위로 한 번 변환하고, 대화상자를 닫거나 서식을 복사할 때 원시 간격을 보존한다.
구버전 WASM의 전체 원문 탐색은 상세 간격과 모양 복사에 필요한 시점에 한정한다.
문단 저장 단위 교정은 조판 결과에 영향을 줄 수 있어 통합 시각 검증에 포함했다.

- 실제 브라우저에서 본문 간격을 셀로 복사한 뒤 이웃 셀, 실행 취소와 다시 실행을 확인했다.
  최신 최적화 WASM과 공개 v0.9.0 WASM 모두 HWP·HWPX 저장·재열기를 통과했다.
- 직접 입력의 위·아래·왼쪽 원시값은 후보 `1500/1000/2000`이다. 같은 흐름의 기준에서
  관측한 값은 `750/500/2000`이다. 별도 한컴 저장본의 후보 UI는 위 5.0pt, 아래 0.0pt,
  왼쪽 20.0pt를 표시했다. 일반 커서 조회는 추가 원문 탐색 0회였다.
  [저장 원시값과 생산자](../assets/pr-9206/evidence.json) ·
  [실제 Studio 화면](../assets/pr-9206/studio-production-spacing.png).
- 집중 기능 검사와 원래 2,822개 Studio 테스트가 통과했다. 원시값은 HWPX XML과 브라우저
  표시값을 함께 확인했다. 이 동작에는 한컴 PDF 그림을 대조 기준으로 사용하지 않았다.

최종 Studio의 실제 브라우저에서 본문에서 표 셀로 문단 모양을 복사했다. 대상 셀의 원시 간격은
3000·2000 HWPUNIT, 이웃 셀은 800·400 HWPUNIT를 유지했다. 실행 취소·재실행과 HWP·HWPX
저장 결과도 확인했다.
[최종 제품 SHA와 실행 결과](../assets/integration_open_prs_20261120/final-product-consumers.json).

## Merge 후 contributor PR comment 계획

문단 간격 저장 단위를 보존한 기여에 감사한다. 두 형식의 재열기, 셀 서식 복사, 구 WASM의
조회 범위 검증 결과를 설명한다. 통합 CI·병합 SHA·devel 반영을 확인한 다음 실제 Studio
대화상자 이미지를 해당 SHA로 고정하고 게시 본문·이미지 응답과 #7938 상태를 확인한다.
