# PR #6931 검토 기록: WMF 기본 텍스트 정렬 TA_TOP

## 최신 시각 증적 갱신 (2026-09-09)

Visual Sweep 보정 `4fe5530d6`을 사용한 [공통 재산출 기록](pr_6886_6932_planet6897_visual_sweep.md)을 적용했다. 제품 소스·기준 PDF와 기존 제품 수용 범위는 변경하지 않았다. 아래 이전 PNG 유지 또는 글꼴 깨짐 서술과 이전 래스터 측정값은 이 재산출 기록으로 대체한다.

| 대상 | 쪽 | 픽셀 일치율 | 잉크 일치율 | 최종 증적 |
| --- | --- | --- | --- | --- |
| #6931 | 6 | 비교 기준 PDF 없음 | 산출하지 않음 | [PNG](../assets/pr_6886_6932_planet6897_20260909/pr6931-p006.png) |

후속 PR·이슈 코멘트에는 위 최종 PNG를 직접 표시하고, 기존 문단 배치 차이까지 해결했다고 기록하지 않는다.


## 판정: 승인

SetTextAlign이 없는 WMF의 기본 상태를 TA_TOP으로 두는 수정은 공식 형식 계약 및 양성·음성 회귀에 부합하므로 수용한다.

**원 PR 변경 범위의 로컬 수용 판정이다.** 후속 재검증으로 #6910·#6926의 보류 사유를 해소했고 전체 회귀 9,327개가 통과했다. 원격 통합 PR 생성·머지 승인과 최종 원격 CI gate는 별도다.

## 검토 기준

- 검토일: 2026-09-09, macOS 로컬.
- 원 PR: [#6931](https://github.com/edwardkim/rhwp/pull/6931), 기여자 `planet6897`.
- 연결 이슈: [#6919](https://github.com/edwardkim/rhwp/issues/6919).
- 확인한 최신 원격 head: `116a1e743ed77402d467c15bab3ac4e19676f7af`.
- 로컬 적용 커밋: `5d2e91408`.
- 통합 브랜치: `review/planet6897-batch-20260909`.
- 테스트·재산출 기준 소스: `976ecf3f48095ccab2d01fc679dab0b91899a69f`.
- 원 PR 최신 head의 CI 그린을 확인했다. 원 PR의 CI와 아직 생성하지 않은 통합 PR의 CI는 다르다.
- [공통 구현·검증 기록](pr_6886_6932_planet6897_impl.md) / [시각 증적 기록](pr_6886_6932_planet6897_visual_sweep.md).

## 변경 범위와 코드 검토

명시 TA_BASELINE 레코드는 그대로 유지한다. 기본 장치 컨텍스트의 수직 정렬만 수정한다.

## 실행한 검증

`issue_6919_wmf_default_text_align`의 기본값=명시 TA_TOP, 명시 TA_BASELINE 불변, 실제 WMF 계약 검사가 통과했다. 원본 HWP를 찾아 물리 6쪽 SVG/PNG를 현재 통합본으로 산출하고 차트 제목이 상단 테두리 아래에 있음을 직접 확인했다.

최종 메인터너 보정 후 공통 전체 회귀는 9,327개 통과·실패 0·별도 건너뜀 46개다. focused 5개, fmt, native/WASM/workspace all-targets Clippy, workspace 빌드와 manifest 검사도 통과했다. 제품 소스가 같아 기존 Native Skia·WASM package·Studio·TypeScript·Chrome·시각 결과를 재사용했으며, 이전 실패와 재실행 범위는 공통 기록에 구분했다.

## 입력 문서와 시각 증적

- 입력: `samples/issue6919/148726703-chart-preview.wmf; 원본 /Users/tsjang/Downloads/korea_downloads/korea_policy_downloads/148726703_sjcu1112.hwp`.
- 실제 검토 범위: 물리 6쪽 / 문서 내부 표시 -2- / 전체 87쪽.
- 한컴 PDF 전체 일치 판정은 해당 검증의 근거로 사용하지 않았다.

![현재 통합본 #6931 증적](../assets/pr_6886_6932_planet6897_20260909/pr6931-p006.png)

공식 근거: [MS-WMF TextAlignmentMode Flags 2.1.2.3](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wmf/2cf0d802-5db7-42f6-bb75-50ff195a6c7c). 수직 정렬 비트가 없으면 TA_TOP이다. 소스 주석의 절 번호 2.1.2.18 대신 이 문서에서는 확인한 2.1.2.3을 인용한다.

## 잔여 사항과 수용 범위

독립 Windows GDI+ 재실행은 하지 않았다. 기여자의 GDI+ 비교와 공식 규약은 구분한다. 본문의 대체 글리프는 남지만 WMF 차트 제목의 기본 정렬 계약과 섞어 승인 범위를 넓히지 않는다.

## 원 PR·이슈 코멘트 계획

TA_TOP 기본값 및 명시 BASELINE 불변을 기록하고 현재 6쪽 차트 이미지를 직접 표시한다. 한컴 PDF 또는 Windows GDI+를 이번에 새 실행했다고 쓰지 않는다.

- 지금은 원격 댓글·review·close를 수행하지 않았다.
- 모든 통합 gate와 merge 후 devel CI 완료 뒤 실제 merge SHA·PR/devel CI URL을 기록한다.
- 증적은 공통 기록의 댓글 템플릿처럼 commit 고정 raw 이미지로 본문에 직접 표시한다. `/tmp` 경로·로그 링크를 공개 증적으로 사용하지 않는다.
- 기존 수용 댓글이 있으면 본문을 수정하고 중복 등록하지 않는다. UTF-8 body-file 방식으로 게시한 뒤 API로 실제 본문을 재조회한다.
- 연결 이슈의 실제 범위가 남으면 전체 해결로 닫지 않는다. 기여자 fork 브랜치는 보존한다.
