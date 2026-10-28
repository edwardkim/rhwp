# PR #6898 검토 기록: TAC 뒤 그림의 문자 슬롯 오프셋

## 최신 시각 증적 갱신 (2026-09-09)

Visual Sweep 보정 `4fe5530d6`을 사용한 [공통 재산출 기록](pr_6886_6932_planet6897_visual_sweep.md)을 적용했다. 제품 소스·기준 PDF와 기존 제품 수용 범위는 변경하지 않았다. 아래 이전 PNG 유지 또는 글꼴 깨짐 서술과 이전 래스터 측정값은 이 재산출 기록으로 대체한다.

| 대상 | 쪽 | 픽셀 일치율 | 잉크 일치율 | 최종 증적 |
| --- | --- | --- | --- | --- |
| #6898 | 7 | 94.617% | 49.774% | [PNG](../assets/pr_6886_6932_planet6897_20260909/pr6898-p007.png) |

후속 PR·이슈 코멘트에는 위 최종 PNG를 직접 표시하고, 기존 문단 배치 차이까지 해결했다고 기록하지 않는다.


## 판정: 승인

빈 char_offsets에서 제어문자 슬롯을 복원하되 앞선 TAC 형제가 있는 경우만 보정하는 변경을 수용한다.

**원 PR 변경 범위의 로컬 수용 판정이다.** 후속 재검증으로 #6910·#6926의 보류 사유를 해소했고 전체 회귀 9,327개가 통과했다. 원격 통합 PR 생성·머지 승인과 최종 원격 CI gate는 별도다.

## 검토 기준

- 검토일: 2026-09-09, macOS 로컬.
- 원 PR: [#6898](https://github.com/edwardkim/rhwp/pull/6898), 기여자 `planet6897`.
- 연결 이슈: [#6879](https://github.com/edwardkim/rhwp/issues/6879).
- 확인한 최신 원격 head: `95bf64cb747db7681b723cbb98e3ce057997625e`.
- 로컬 적용 커밋: `9d91bd1c9, bd9ab659b, 672eb9f64, 980c80203`.
- 통합 브랜치: `review/planet6897-batch-20260909`.
- 테스트·재산출 기준 소스: `976ecf3f48095ccab2d01fc679dab0b91899a69f`.
- 원 PR 최신 head의 CI 그린을 확인했다. 원 PR의 CI와 아직 생성하지 않은 통합 PR의 CI는 다르다.
- [공통 구현·검증 기록](pr_6886_6932_planet6897_impl.md) / [시각 증적 기록](pr_6886_6932_planet6897_visual_sweep.md).

## 변경 범위와 코드 검토

공유된 앵커 오프셋을 흐름과 그리기에 일관되게 사용하며 TAC가 없는 기존 문서의 위치는 유지한다.

## 실행한 검증

관련 회귀 4개가 전체 회귀에서 통과했다. 8쪽 문서의 7쪽 그림 두 개를 재산출해 PDF와 대조했다.

최종 메인터너 보정 후 공통 전체 회귀는 9,327개 통과·실패 0·별도 건너뜀 46개다. focused 5개, fmt, native/WASM/workspace all-targets Clippy, workspace 빌드와 manifest 검사도 통과했다. 제품 소스가 같아 기존 Native Skia·WASM package·Studio·TypeScript·Chrome·시각 결과를 재사용했으며, 이전 실패와 재실행 범위는 공통 기록에 구분했다.

## 입력 문서와 시각 증적

- 입력: `samples/issue6879/156767332-broadcast-revenue-attachment.hwp`.
- 실제 검토 범위: 7쪽.
- 기준 PDF: [pdf/156767332-broadcast-revenue-attachment-2020.pdf](../../../pdf/156767332-broadcast-revenue-attachment-2020.pdf).

![현재 통합본 #6898 증적](../assets/pr_6886_6932_planet6897_20260909/pr6898-p007.png)

## 잔여 사항과 수용 범위

최신 원격 head는 이력이 재작성됐다. 기존 적용 커밋과 patch-id 및 관련 코드·테스트·샘플·PDF·증적의 동등성을 확인했으므로 동일 패치를 중복 적용하지 않았다. 제목 글꼴·외곽선 차이는 전체 시각 일치 주장에 포함하지 않는다.

## 원 PR·이슈 코멘트 계획

최신 원격 SHA와 로컬 적용 이력의 관계를 설명하고 7쪽 두 그림의 위치 검증만 명시한다.

- 지금은 원격 댓글·review·close를 수행하지 않았다.
- 모든 통합 gate와 merge 후 devel CI 완료 뒤 실제 merge SHA·PR/devel CI URL을 기록한다.
- 증적은 공통 기록의 댓글 템플릿처럼 commit 고정 raw 이미지로 본문에 직접 표시한다. `/tmp` 경로·로그 링크를 공개 증적으로 사용하지 않는다.
- 기존 수용 댓글이 있으면 본문을 수정하고 중복 등록하지 않는다. UTF-8 body-file 방식으로 게시한 뒤 API로 실제 본문을 재조회한다.
- 연결 이슈의 실제 범위가 남으면 전체 해결로 닫지 않는다. 기여자 fork 브랜치는 보존한다.
