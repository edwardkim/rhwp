# PR #9314 재검토

메인테이너 보완 소스 `10f5bca362`로 기울인 도형을 가로로 늘리고 복원했다. Native와 fresh WASM에서 각 13쪽의 최저 일치율은 99.97%와 99.77%다. 한컴 2024로 HWP·HWPX를 재인쇄한 두 상태에서 한컴 직접 편집본과 픽셀 차이가 0개였다. 지원하지 않는 기울기 편집을 거절해도 기존 너비와 저장 상태를 보존한다. [보완 검증과 전쪽 증거](../assets/semanticist21-9288-9314-20261124/corrected/README.md)

검토일은 2026-11-24 KST이다. 기여자 semanticist21의 head는 `0cb5fcd1c941ec24e83d8420597dbeb5586e4b05`이고 리뷰어는 edwardkim이다. 기여자는 #9286과 함께 적용한 Mac 한컴 결과로 이전 보류에 답했다. 이 답변 이후 #9314의 제품 커밋은 그대로다. [원 PR](https://github.com/edwardkim/rhwp/pull/9314) · [이전 보류](https://github.com/edwardkim/rhwp/pull/9314#issuecomment-7089122424) · [기여자 답변](https://github.com/edwardkim/rhwp/pull/9314#issuecomment-7105310342)

#9286을 포함한 #9376이 반영된 `devel` `9ee9a5b0b1`에서 #9288·#9314의 원 기여 커밋을 병합한 로컬 후보는 `38b38f2924dfa6a863e6dcef3bab27f14e479248`이다. #9288은 누름틀 조회·선택을 변경한다. #9314만 같은 base에 병합한 Git tree는 `f7d871ed18bfbc109624a707c5becf7f9ca4bbad`이다.

## 원본과 인쇄 근거

[한컴 HWPX 원본](../../../tests/fixtures/shape_skew/hancom_skew.hwpx)의 SHA-256은 `2ccdf6296314f520c574bf1f65eb8340d3d907a1149e60d587338e432aae7c69`이다. Windows 한컴 2024 13.0.0.3901의 인쇄 드라이버로 만든 [독립 PDF](../../../pdf/pr9314-review/hancom-skew-original-2024.pdf)는 1쪽이며 SHA-256은 `2ef220f5e9e717a7f869585e1c6b491d82275a07a5d40ad38b66c63b521714fc`이다. 원본의 왼쪽 위 반투명 사각형을 유지했다. 후보 SVG의 채움 불투명도는 `0.698`, 획 불투명도는 `0.498`이다.

## 너비 편집에서 기울기 손실

독립 Print PDF와 원본 1쪽의 실루엣을 비교했다. 기여자가 답변한 반투명 도형 표시 문제는 현재 후보에서 해소됐다.

| 독립 한컴 원본 인쇄와 비교한 문서 | Native | fresh WASM Canvas |
| --- | ---: | ---: |
| 기존 `devel`에서 원본 열기 | 50.30% | 50.56% |
| 후보에서 원본 열기 | 99.97% | 99.77% |
| 후보에서 기울인 도형의 너비를 되돌려 저장 후 열기 | 77.99% | 78.03% |
| 후보에서 일반 사각형의 너비를 되돌려 저장 후 열기 | 99.97% | 99.77% |

원래 너비를 같은 값으로 재적용했을 때도 양쪽 렌더러의 출력이 편집 전과 같았다. [쪽별 TSV와 출력 해시](../assets/semanticist21-9288-9314-20261124/skew-pages.tsv) · [원본 전쪽 비교](../assets/semanticist21-9288-9314-20261124/original-page.png) · [너비 복원 후 전쪽 비교](../assets/semanticist21-9288-9314-20261124/width-restored-page.png)

`rhwp edit set-shape`로 원본의 첫 문단, 컨트롤 5(가로로 20° 기울인 사각형)의 `width`만 14912 → 16000 → 14912 HWPUNIT로 변경하고 HWPX로 저장했다. 별도 대조군은 같은 원본의 기울기 없는 사각형(컨트롤 2)만 12000 → 13000 → 12000으로 편집했다. 원래 너비를 그대로 쓰는 대조군도 확인했다. [편집 후 HWPX](../assets/semanticist21-9288-9314-20261124/skewed-width-restored.hwpx) · [일반 사각형 대조군](../assets/semanticist21-9288-9314-20261124/plain-width-restored-control.hwpx)

| 96 DPI로 다시 인쇄한 Windows 한컴 2024 결과 | 원본 인쇄와 다른 픽셀 | 도형 |
| --- | ---: | --- |
| 기울인 사각형의 너비 복원 | 1,163 | 평행사변형이 직사각형으로 변함 |
| 일반 사각형의 너비 복원 | 0 | 원본과 동일 |

원본의 `scaMatrix.e2`는 `-0.36397`이다. 기울인 사각형의 너비를 되돌린 뒤 이 값은 `0`이며, 너비만 원래 값으로 돌아왔다. 같은 두 편집을 기준 `devel` 바이너리로 실행해도 저장 행렬이 지워진다. 기여자의 원본 표시 개선은 확인했고, 기존 크기 편집 경로에는 기울기 보존이 더 필요하다. [기울인 도형의 저장본을 한컴에서 인쇄한 PDF](../../../pdf/pr9314-review/skewed-width-restored-2024.pdf) · [일반 사각형 대조군의 인쇄 PDF](../../../pdf/pr9314-review/plain-width-restored-control-2024.pdf) · [확대 비교](../assets/semanticist21-9288-9314-20261124/skew-resize-comparison.png)

`set_shape_properties_native`는 크기 변경 뒤 `finish_top_level_edit`를 호출한다. 이 과정에서 사각형의 저장 행렬과 전단 계수가 지워진다. 이어서 실행되는 `apply_skew`는 저장 행렬에서 종전 각도를 구하므로, 너비만 입력하면 0°로 처리한다. 이전 각도 보존, 크기 변경 뒤 외접 상자·이동 행렬 재계산 및 양 형식 저장·재열기가 보완 범위다. [편집 순서](../../../src/document_core/commands/object_ops/shape.rs#L640) · [기존 행렬 초기화](../../../src/document_core/commands/object_ops/shape/group_geometry.rs#L59)

원 기여만 적용한 시점에는 **머지 보류**로 판정했다. 너비를 원상 복원하면 기울기와 인쇄 결과가 보존되어야 한다. 오류로 거절된 복합 편집은 이전 너비까지 보존해야 한다. 아래에 두 번째 경계의 실측 결과를 기록했다.

## 재현 명령

저장소 루트에서 다음 두 번의 편집을 실행한다. 두 번째로 생성된 HWPX를 한컴 2024에서 열고 인쇄하면 오른쪽 위 사각형의 기울기가 사라진 것을 확인할 수 있다.

```sh
mkdir -p output/pr9314-resize
rhwp edit set-shape tests/fixtures/shape_skew/hancom_skew.hwpx \
  --section 0 --para 0 --ctrl 5 --props '{"width":16000}' \
  -o output/pr9314-resize/wider.hwpx
rhwp edit set-shape output/pr9314-resize/wider.hwpx \
  --section 0 --para 0 --ctrl 5 --props '{"width":14912}' \
  -o output/pr9314-resize/restored.hwpx
```

## 각도 설정과 실패한 편집의 상태

fresh WASM의 공개 `setShapeProperties`로 새 사각형에 가로·세로 각도를 (20°, 10°), (−20°, −10°), (20°, 0°), (0°, −20°)로 지정했다. 네 경우 모두 HWP·HWPX를 다시 열었을 때 각도와 외접 상자 크기를 보존했고, 0°로 되돌리자 원래 크기로 돌아왔다.

30° 회전한 사각형에 `{"width":14600,"skewX":20}`을 한 번에 전달하면 API가 오류를 반환한 뒤에도 너비가 14392에서 14600으로 바뀐다. 직후의 문서와 HWP·HWPX 재열기에서 모두 14600이었다. 같은 도형에 `{"skewX":20}`만 전달하면 오류 뒤의 너비는 세 경우 모두 14392였다. 크기 등 다른 속성을 바꾸기 전에 도형 종류·각도·회전·대칭 조건을 검증하거나 실패한 요청 전체를 되돌려야 한다. [입력값과 형식별 편집 결과](../assets/semanticist21-9288-9314-20261124/wasm-shape-operations.json)

Native에서 행정업무운영 편람 HWPX 383쪽을 후보 바이너리로 다시 렌더링했다. 후보 PNG와 Print PNG가 앞선 기준 측정의 해당 입력 해시와 각각 같음을 확인해 원래 실루엣 값을 대조했다. 최저 99.97%, 쪽별 하락 0건이다. 기여자의 도형 기울이기 Rust 회귀 2개도 모두 통과했다. 편집본 1쪽이 90% 하한에 미달해 전체 보호 문서 검증은 보완 후의 단계로 남겼다. 원 head의 GitHub 검사 상태는 성공 35개, 건너뜀 6개이며 최신 devel과 충돌이 없다. [383쪽 TSV](../assets/semanticist21-9288-9314-20261124/handbook-native.tsv) · [PR 검사](https://github.com/edwardkim/rhwp/pull/9314/checks)

## 메인테이너 보완 범위

작업지시자가 2026-11-24에 로컬 보완과 재검증을 승인했다. 크기 편집이 원래 좌표와 저장 행렬에 미치는 영향을 한컴 표본과 대조하고, 지원하지 않는 복합 기울기 편집을 문서 변경 전에 판정한다. 일반 사각형·원래 기울인 도형·회전 도형과 각도 변경을 HWP·HWPX 저장, Native·fresh WASM 렌더링, Windows 한컴 인쇄로 검증한다.

## 최종 로컬 검증과 통합 판정

공개 글꼴 5개 환경에서 최종 Rust 회귀 14,074개, 집중 36개가 통과했다. Native 공식 검사 4,072개, Studio 2,835개가 통과했다. Studio는 2개를 건너뛰었다. Native·WASM·워크스페이스 Clippy와 배포 빌드도 통과했다. [검사별 명령과 결과](../assets/semanticist21-9288-9314-20261124/corrected/local-gates.json)

**메인테이너 보완 후 수용 가능.** 최신 devel과 후보에서 보호 문서 4,046행의 PNG가 모두 같았고, Native 하한 미달은 0쪽이다. WASM은 양쪽에서 같은 14쪽이 공식 100.00%에 대해 99.87~99.99%였다. 2026년 11월 24일 작업지시자는 같은 픽셀인 기존 14쪽에 한정한 예외와 통합 PR의 CI 성공 후 셀프 리뷰·머지를 승인했다. 두 원 기여 SHA와 공식 보호값을 보존했다. [전쪽 비교](../assets/semanticist21-9288-9314-20261124/corrected/protected-pages.tsv) · [기존 차이 14쪽](../assets/semanticist21-9288-9314-20261124/corrected/protected-below.tsv)

## Merge 후 contributor PR comment 계획

기여자가 한/글 12.30에서 확인한 기울기 행렬과 반투명 도형 근거에 감사드린다. 한컴 원본 1쪽과 너비 확대·복원을 포함한 양쪽 각 13쪽, Native 최저 99.97%·WASM 최저 99.77%, 처음과 복원 뒤 픽셀 차이 0개 및 Windows 한컴 재인쇄를 설명한다. 기울기 입력의 CLI 경계 8개와 실패한 편집의 상태 보존도 안내한다. 첫 댓글에 병합 SHA에서 내려받아 해시를 대조한 원본 전쪽 PNG와 Native·WASM 너비 복원 비교 PNG를 넣고 Visual Sweep 가이드에 연결한다.
