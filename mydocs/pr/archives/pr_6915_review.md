# PR #6915 검토 기록: 테두리 두께의 600dpi 양자화

## 최신 시각 증적 갱신 (2026-09-09)

Visual Sweep 보정 `4fe5530d6`을 사용한 [공통 재산출 기록](pr_6886_6932_planet6897_visual_sweep.md)을 적용했다. 제품 소스·기준 PDF와 기존 제품 수용 범위는 변경하지 않았다. 아래 이전 PNG 유지 또는 글꼴 깨짐 서술과 이전 래스터 측정값은 이 재산출 기록으로 대체한다.

| 대상 | 쪽 | 픽셀 일치율 | 잉크 일치율 | 최종 증적 |
| --- | --- | --- | --- | --- |
| #6915 | 1 | 85.933% | 12.409% | [PNG](../assets/pr_6886_6932_planet6897_20260909/pr6915-p001.png) |

후속 PR·이슈 코멘트에는 위 최종 PNG를 직접 표시하고, 기존 문단 배치 차이까지 해결했다고 기록하지 않는다.


## 판정: 승인

테두리 폭 16단계를 한컴의 600dpi 정수 격자에 맞추는 변경을 수용한다. 기존 golden 6개 변경은 전체 회귀 통과 항목과 함께 평가했다.

**원 PR 변경 범위의 로컬 수용 판정이다.** 후속 재검증으로 #6910·#6926의 보류 사유를 해소했고 전체 회귀 9,327개가 통과했다. 원격 통합 PR 생성·머지 승인과 최종 원격 CI gate는 별도다.

## 검토 기준

- 검토일: 2026-09-09, macOS 로컬.
- 원 PR: [#6915](https://github.com/edwardkim/rhwp/pull/6915), 기여자 `planet6897`.
- 연결 이슈: [#6913](https://github.com/edwardkim/rhwp/issues/6913).
- 확인한 최신 원격 head: `4cc02ac252b379d6dafb31d44b65105f18d83b56`.
- 로컬 적용 커밋: `3696f4e80`.
- 통합 브랜치: `review/planet6897-batch-20260909`.
- 테스트·재산출 기준 소스: `976ecf3f48095ccab2d01fc679dab0b91899a69f`.
- 원 PR 최신 head의 CI 그린을 확인했다. 원 PR의 CI와 아직 생성하지 않은 통합 PR의 CI는 다르다.
- [공통 구현·검증 기록](pr_6886_6932_planet6897_impl.md) / [시각 증적 기록](pr_6886_6932_planet6897_visual_sweep.md).

## 변경 범위와 코드 검토

mm→HU 반올림 후 600dpi 격자 양자화를 적용한다. 0.2mm는 0.80px, 0.12mm는 0.48px, 0.1mm는 0.32px 계약이다.

## 실행한 검증

`issue_6913_border_width_mm_axis` 관련 회귀 및 변경된 golden 비교 항목이 전체 회귀에서 통과했다. 1쪽 머리 표를 재산출했다. 기여자의 border-width-grid.png도 직접 열어 수정 전·후·정본 비교 범위를 확인했다.

최종 메인터너 보정 후 공통 전체 회귀는 9,327개 통과·실패 0·별도 건너뜀 46개다. focused 5개, fmt, native/WASM/workspace all-targets Clippy, workspace 빌드와 manifest 검사도 통과했다. 제품 소스가 같아 기존 Native Skia·WASM package·Studio·TypeScript·Chrome·시각 결과를 재사용했으며, 이전 실패와 재실행 범위는 공통 기록에 구분했다.

## 입력 문서와 시각 증적

- 입력: `samples/issue6913/156591199-veterans-joint-burial-press-release.hwpx`.
- 실제 검토 범위: 1쪽 / 전체 3쪽.
- 기준 PDF: [pdf/156591199-veterans-joint-burial-press-release-2020.pdf](../../../pdf/156591199-veterans-joint-burial-press-release-2020.pdf).

![현재 통합본 #6915 증적](../assets/pr_6886_6932_planet6897_20260909/pr6915-p001.png)

기여자 제공 확대 대조: [border-width-grid.png](../assets/issue6913/border-width-grid.png). 현재 통합본의 새 산출물과 구분한다.

## 잔여 사항과 수용 범위

16종의 단일 변수 문서를 MCP로 다시 변환한 것은 아니다. 96dpi 전체 페이지의 잉크 일치율만으로 미세 선 두께를 평가하지 않는다. 본문 글꼴·행 위치 잔차는 남아 있다.

## 원 PR·이슈 코멘트 계획

수치 계약과 golden 회귀 통과, 현재 머리 표 증적을 함께 남긴다. 기여자 600dpi 확대 대조는 제공 증적이라고 명시한다.

- 지금은 원격 댓글·review·close를 수행하지 않았다.
- 모든 통합 gate와 merge 후 devel CI 완료 뒤 실제 merge SHA·PR/devel CI URL을 기록한다.
- 증적은 공통 기록의 댓글 템플릿처럼 commit 고정 raw 이미지로 본문에 직접 표시한다. `/tmp` 경로·로그 링크를 공개 증적으로 사용하지 않는다.
- 기존 수용 댓글이 있으면 본문을 수정하고 중복 등록하지 않는다. UTF-8 body-file 방식으로 게시한 뒤 API로 실제 본문을 재조회한다.
- 연결 이슈의 실제 범위가 남으면 전체 해결로 닫지 않는다. 기여자 fork 브랜치는 보존한다.
