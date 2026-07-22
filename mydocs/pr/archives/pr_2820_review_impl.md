# PR #2820 메인테이너 보정 기록

## 검토에서 발견한 잔여 결함

원 PR 적용 뒤 후속 문단의 한 줄 위치는 정상화됐지만 visual sweep의 rhwp 쪽에서 좌하단 봉투
그림이 사라졌다. PDF와 원본 HWPX를 대조한 결과 폰트나 clipping 문제가 아니라 crop 좌표계
해석 문제였다.

## 원인과 보정

- 내장 PNG: 192×108px
- HWPX `imgDim`: 144000×81000 HU
- crop: 전체 `0,0,144000,81000`
- 종전 고정 75 HU/px 환산: 1920×1080 가상 원본으로 해석해 실제 이미지의 1/10만 표시
- 보정: 유효한 `imgDim`이 있으면 이를 crop reference size로 우선 사용하고, 없을 때만 75 HU/px
  fallback을 유지

`Picture::crop_reference_size()`를 추가하고 layout ImageNode에서 SVG·WebCanvas·Skia·CanvasKit과
PageLayerTree JSON까지 같은 기준을 전달했다. `issue2817` 단위 테스트, paint JSON 및 CanvasKit
계약 테스트를 추가했다. 보정 커밋은 `90ef435c9`, 직접 회귀 가드 보강은 `98a2f3113`이다.

통합 PR #2990의 첫 CI에서 Native Skia crop 테스트 fixture가 새 `imgDim` 기준을 지정하지 않아
실패했다. 실제 ImageNode 계약과 같이 `original_size_hu`를 지정하도록 fixture를 보정했고,
CI와 같은 공식 Native Skia 3종(필터된 lib 56건, placeholder integration 2건, PDF integration
4건)을 로컬에서 모두 통과했다. 같은 누락을 PR 준비 단계에서 잡도록
`mydocs/manual/pr_review_workflow.md`의 renderer 전체 게이트에도 이 3종을 명시했다.

## 기존 snapshot 영향

`exam_kor`의 354.4px 환산 오차가 실제 354px로 정정됐고, `복학원서`의 878×1001 전체 이미지에는
불필요한 중첩 crop SVG가 제거됐다. 두 변화의 source geometry를 확인한 뒤 golden을 갱신했으며
snapshot 8/8과 전체 integration 회귀를 다시 통과했다. golden 커밋은 `808d98162`이다.

## 판정

최종 review PNG에서 봉투 그림이 PDF와 같은 위치·크기로 표시된다. crop 기준을 문서의 명시 좌표에
연결하고 기존 문서의 fallback을 보존한 구조적 보정이므로 통합한다.
