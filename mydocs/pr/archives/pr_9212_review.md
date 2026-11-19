# PR #9212 검토 — #7739–#7743 원본 조판과 회귀 검사 복원

**승인.** 네 원본 581쪽에서 Native·fresh WASM이 모두 98% 이상이고, 정확한 코드 head의 GitHub CI도 통과했다.

| 항목 | 결과 |
| --- | --- |
| PR·작성자 | [#9212](https://github.com/edwardkim/rhwp/pull/9212) · `jangster77` |
| 고정 비교 base | `9f605a0d0aa2410754ec1470078d831a0c09688f` |
| 제품 생산 소스 | `d77d3e0faa0f75d0bfcd49453aa8f144f6b56b68` |
| 원격 검증 code head | `67254f93bc3c0fca3e3bb5f5845b3ad8c515424e` |
| 검토 시점 upstream/devel | `9f605a0d0aa2410754ec1470078d831a0c09688f` |
| 검토 방식 | Collaborator self-review · `MERGEABLE / CLEAN` |

## 원본별 검토

| 이슈 | 쪽수 | Native 최저 | fresh WASM 최저 | 확인한 계약 |
| --- | ---: | ---: | ---: | --- |
| [#7739](https://github.com/edwardkim/rhwp/issues/7739) 화학제품 표시도안 연구 | 103 | 98.79596% (65쪽) | 98.79775% (65쪽) | 32비트 DIB의 인쇄와 이어받기 셀 안 사진의 시작선 |
| [#7741](https://github.com/edwardkim/rhwp/issues/7741) 분할 표 HWPX | 18 | 98.32256% (16쪽) | 98.36100% (16쪽) | 마지막 행과 후속 문단의 쪽 소속·간격 |
| [#7742](https://github.com/edwardkim/rhwp/issues/7742) 사이버대학 신청서 | 45 | 98.31892% (29쪽) | 98.54594% (29쪽) | 원본의 관인, 28쪽 중첩 표의 첫 일곱 행, 29쪽의 여덟째 행 |
| [#7743](https://github.com/edwardkim/rhwp/issues/7743) 교육과정 연결맵 | 415 | 98.21994% (134쪽) | 98.18712% (134쪽) | 목록 말미 줄 소속, 검정 취소선, 분할 셀과 재저장 소속 |

[#7740](https://github.com/edwardkim/rhwp/issues/7740)의 과거 110개 검사에서는 같은 이름을 가진
49개와 이동·통합·삭제 이력 61개를 확인했다. 과거 실패 중 현재 이름이 남은 검사 다섯 건은
집중 재실행에서 통과했다. 독립 Print와 두 경로 모두 98% 이상이 확인된 대상에 회귀를
추가·복원했다. [검사 경로 감사](../assets/issues7739_7743_20261119/historical-test-audit.json) ·
[입력·출력·확대 판독](../assets/issues7739_7743_20261119/analysis.md) · [전체 TSV와 해시](../assets/issues7739_7743_20261119/visual-evidence.json).

기준은 원본에 대응하는 데스크톱 한컴의 Hancom PDF Print다. 사이버대학 원본의 관인과
1-up을 확인할 수 있는 한컴 2020 PDF를 새로 확보해 base·후보에 같은 PDF를 적용했다.
29쪽의 고정 base 대비 최대 하락은 0.35005%p이고, 해당 결과와 확대 판독을 증적에 남겼다.

## 로컬·원격 검증

- 최소 글꼴 macOS 전체 Nextest 13,831건 통과, 45건 제외. Native Skia 3종·직접 PDF 검사,
  fmt·Clippy·workspace 구성 검증도 통과했다. HWP/HWPX 반복 저장과 재열기에서 415쪽·편집
  내용·취소선 속성을 확인했다. [로컬 실행 기록](../assets/issues7739_7743_20261119/local-validation.json).
- 필수 편람 HWPX Native 383쪽·기여자 공통 8조건 통과. 정본 46조건·4,046쪽에서 하락과
  누락 0건을 확인했다. 법령 표 WASM 19쪽 기준을 99.73%에서 99.74%로 갱신했다.
  [보호 검증](../assets/issues7739_7743_20261119/protected-validation.json).
- 실제 Native Skia·Studio Canvas2D 각 12쪽에서 대표 경계를 비교했다. 분할 표의 실제
  CanvasKit 3쪽은 자원 준비를 확인했다. 다른 세 문서의 CanvasKit 요청 9쪽은 Canvas2D
  전환과 이유를 기록했다. [브라우저 검증](../assets/issues7739_7743_20261119/actual-consumer-validation.json).
- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/44593113430/job/135822822858) · [CI](https://github.com/edwardkim/rhwp/actions/runs/44593113430) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44593113243) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44593113289) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44593113336) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44593113052) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44593112903)

## 대표 시각 증적

| 변경 지점 | Native | fresh WASM |
| --- | --- | --- |
| 화학제품 64쪽, 이어받기 셀의 사진 | [review](../assets/issues7739_7743_20261119/chemical-6782/native/review_064.png) · [overlay](../assets/issues7739_7743_20261119/chemical-6782/native/overlay_064.png) | [review](../assets/issues7739_7743_20261119/chemical-6782/wasm/review_064.png) · [overlay](../assets/issues7739_7743_20261119/chemical-6782/wasm/overlay_064.png) |
| 분할 표 11쪽, 행 종료 | [review](../assets/issues7739_7743_20261119/rowbreak-hwpx/native/review_011.png) · [overlay](../assets/issues7739_7743_20261119/rowbreak-hwpx/native/overlay_011.png) | [review](../assets/issues7739_7743_20261119/rowbreak-hwpx/wasm/review_011.png) · [overlay](../assets/issues7739_7743_20261119/rowbreak-hwpx/wasm/overlay_011.png) |
| 사이버대학 28쪽, 중첩 표 조각 | [review](../assets/issues7739_7743_20261119/cyber-6795/native/review_028.png) · [overlay](../assets/issues7739_7743_20261119/cyber-6795/native/overlay_028.png) | [review](../assets/issues7739_7743_20261119/cyber-6795/wasm/review_028.png) · [overlay](../assets/issues7739_7743_20261119/cyber-6795/wasm/overlay_028.png) |
| 교육과정 8쪽, 검정 취소선 | [review](../assets/issues7739_7743_20261119/curriculum-2287/native/review_008.png) · [overlay](../assets/issues7739_7743_20261119/curriculum-2287/native/overlay_008.png) | [review](../assets/issues7739_7743_20261119/curriculum-2287/wasm/review_008.png) · [overlay](../assets/issues7739_7743_20261119/curriculum-2287/wasm/overlay_008.png) |

## 병합 후 PR 코멘트 계획

[CI 정책 판정](https://github.com/edwardkim/rhwp/actions/runs/44598439324)의 성공을 확인했다.
병합 후 댓글에는 코드 CI, 최종 문서 head의 재사용 판정·Build & Test 결과, 실제 병합
SHA와 이슈 종료 상태를 남긴다. 위 표의 네 입력과 같은 쪽 Native review·overlay를
병합 SHA의 원본 이미지로 연결한다. 각 서브이슈에는 그 이슈의 판독 영역과 전쪽 수치,
#7740에는 경로 감사와 복원한 검사 범위를 남긴다.

## 후속 처리

이 기록을 담은 head의 CI 확인 뒤 병합한다. 병합을 확인한 뒤 #7739–#7743의 종료 근거와
#7445의 해당 다섯 행을 갱신한다. 원본 전체가 98%를 충족하며 개별 글리프와 옅은 선의
미세한 차이는 [출력 근거](../assets/issues7739_7743_20261119/analysis.md)에 기록했다.
