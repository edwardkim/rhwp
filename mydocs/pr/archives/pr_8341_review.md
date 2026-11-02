# [PR #8341](https://github.com/edwardkim/rhwp/pull/8341) 리뷰 — fix(serializer/hwpx): 제목 차례 표시를 자기 글자 모양 run에 두고 경계를 밀지 않는다 ([이슈 #8305](https://github.com/edwardkim/rhwp/issues/8305))

## 최종 판정

**통합 PR에 반영됨 — 원 PR별 최종 검토 진행 중.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 머지됐다. 이 원 PR의 주장·변경 범위와 독립 증거를 대조한 뒤 원 PR 종료 여부를 판정한다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `74685790f5` → `bc4fa7f74a`, `2aecaab467` → `3fb51885a3`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_8305_titlemark_boundary_runs` 5건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

2026-11-03 00시경 KST에도 원 PR head와 열린 상태를 재확인했다. [일괄 조회 결과](../assets/review_8438_followup/original-pr-heads-20261103.md)는 기능·시각 최종 판정을 대신하지 않는다.

## 접수와 적용

- 원 PR: [PR #8341](https://github.com/edwardkim/rhwp/pull/8341), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `2aecaab46753b16ce51b426895b9b97dfeb5d5c7`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `2aecaab46753b16ce51b426895b9b97dfeb5d5c7`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [이슈 #8305](https://github.com/edwardkim/rhwp/issues/8305).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

제목 차례 표시를 원래 글자 모양 run에 두어 후속 경계가 밀리지 않도록 한다.
UTF-16 축의 표시 소유와 뒤 글자 모양을 저장 왕복에서 확인한다.

## 체리픽과 보정

- 원본 `74685790f53ab58f17a1a0cc0b00fe55cb1fc4a6` → 통합 `f331c82fe514b9c38c98882adedd4004c595c2e5`.
- 원본 `2aecaab46753b16ce51b426895b9b97dfeb5d5c7` → 통합 `c21bc6f69ffb8903b2fb9c2c50f1dab347d419e9`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_8305_titlemark_boundary_runs.rs`](../../../tests/cases/issue_8305_titlemark_boundary_runs.rs): 이전 실행 5건 통과, 0건 실패, 0건 건너뜀.

검토 후보에 포함된 입력 바이트를 Git blob과 대조했다. 아래 자료는 원 PR의 제출 증거이며 새 후보의 실행 결과를 뜻하지 않는다.
- [`1790387_policy_progress-hwpx-2020.pdf`](../../../pdf/issue4599/1790387_policy_progress-hwpx-2020.pdf) — SHA-256 `4c0d9d0876410766d99f4589916971f97b751a3d71901068637c7e2cc1061f95`.

기여자가 제출한 기준 자료: [`pp_hwpx_base_silhouette.tsv`](../../../mydocs/pr/assets/issue_8305/native/pp_hwpx_base_silhouette.tsv). 해당 자료만으로 새 통합 후보의 시각 검증을 통과 처리하지 않는다.

원 [PR #8341](https://github.com/edwardkim/rhwp/pull/8341)의 기준 PDF와
수정 후 전체 TSV는 통합 [PR #8438](https://github.com/edwardkim/rhwp/pull/8438)에
이미 포함됐다. 기존 [Native TSV](../assets/issue_8305/native/pp_hwpx_silhouette.tsv)와
[WASM TSV](../assets/issue_8305/wasm/pp_hwpx_silhouette.tsv)는 작성자가 제출한
당시 입력·head의 결과로 사용한다. 별도 재출력 결과를 이 기록과 혼동하지 않는다.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)·[PR #8397](https://github.com/edwardkim/rhwp/pull/8397) 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. [PR #7905](https://github.com/edwardkim/rhwp/pull/7905) 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. [PR #8040](https://github.com/edwardkim/rhwp/pull/8040) 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. [PR #8410](https://github.com/edwardkim/rhwp/pull/8410)·[PR #8412](https://github.com/edwardkim/rhwp/pull/8412)의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

### 새 base 재저장 동일성

검증 기준 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`의 Native 실행 파일로 다시 저장한
사양 대상 재저장 HWPX의 SHA-256은 `14ef498440ef15c3fe0a667a072c2141b1761eb5f0e0e970f6da44495be466c9`로 기존 비교 입력과 바이트 단위로 같다.
같은 입력의 이전 Print와 시각 결과를 재사용할 수 있다는 뜻이며, 별도 미달이나
기능 판정을 면제하지 않는다.

2026-11-03 현재 제품 코드 `dd284b5cda`에서도 같은 원본을 HWPX에 다시 저장했다.
`--verify --verify-pages`는 IR 차이 0건과 전후 64쪽을 보고했으며, 저장본의
SHA-256은 위 기존 비교 입력과 동일하다. 원 PR의 기존 2020 Print에 대한 현재
Native·fresh WASM 전쪽 대조와 제목 차례 영역의 직접 판독을 별도로 진행한다.

현재 코드의 Native는 같은 2020 Print와 전 64쪽을 대조해 최저 92.61%,
90% 미만 0쪽이다. 새 TSV는 기존 [Native TSV](../assets/issue_8305/followup8438/native-saved-original2020-all.tsv)와
SHA-256 `12052dcb751417f113f0f7c4a7d91d06d63517aa5ceca1cefe52881c7e6a0dc2fdd`로
바이트가 같아 기존 파일을 재사용한다. 현재 실행에서 제목 차례를 포함하는
[20쪽 비교](../assets/issue_8305/followup8438/native-current-review-020.png)·
[overlay](../assets/issue_8305/followup8438/native-current-overlay-020.png)와
[59쪽 비교](../assets/issue_8305/followup8438/native-current-review-059.png)·
[overlay](../assets/issue_8305/followup8438/native-current-overlay-059.png)를
확인했다. 2024 Print 35쪽 미달과 현재 fresh WASM은 별도 확인 대상이다.

### 한컴 버전별 차트 출력과 전쪽 검증

동일 원본과 재저장본의 한컴 2020 Print는 둘 다 29쪽 WMF 차트를 확대·잘라
출력한다. 같은 재저장본의 한컴 2024 Print는 차트를 전체 표시한다. 두 기준의
출력 동작이 다르므로 한 버전을 다른 버전의 인쇄 결과로 취급하지 않는다.
[출력 근거와 파일 해시](../assets/issue_8305/followup8438/print-source.md).

2024 Print와 64쪽을 비교했을 때 35쪽 raw는 Native 89.11%, WASM 89.18%로
양쪽 모두 기준 미달이다. 35쪽 차트는 WMF이며 그 숫자의 원본 LOGFONT는
`Calibri`, 굵기 700이다. 검토 시스템에는 Calibri 일반체만 있어 Windows의
Calibri Bold를 검증용 `output/`에 공급했다. 다만 재측정은 Native 89.12%,
WASM 89.19%로 90%를 넘지 못했다. 글꼴 공급만으로 해결됐다고 판정하지 않고
실제 WMF 공급·그리기와 나머지 차이를 확인한 뒤 재검증한다. 글꼴 바이트는
저장소·배포물에 넣지 않는다.

현재 코드 `dd284b5cda`의 같은 2024 Print 35쪽 Native raw도 89.11%다.
2px 이웃을 적용한 실루엣 불일치 5,549픽셀 중 차트 영역은 5,432픽셀이며,
상단 문단과 아래 표 영역은 각각 97픽셀과 20픽셀이다. 차트의 여러 가로
격자선은 같은 위치에서 RHWP가 RGB 235, PDF가 RGB 225로 출력하는 예가 있어
내용 마스크 경계에서 다르게 분류된다. 원본 `BinData/image7.wmf`에는 RGB
217·논리 너비 1인 펜이 있으며, 독립 2024 PDF는 이 차트를 3,729×2,049픽셀,
약 600 DPI 이미지로 포함한다. 이는 표본화 경로를 조사할 근거이며, 임의 색상
조정이나 측정 기준 변경의 근거는 아니다. 2020 기준 결과와 별도로 보정한다.

별도 Node 진단에서는 현재 fresh WASM을 한컴 2024 Print 프로필로 열고 35쪽 layer의 이미지 연산을 확인했다. 해당 차트는 `image/svg+xml`, 32,313바이트로 노출됐다. Calibri·Arial·맑은 고딕 등 SVG에 열거된 이름에 시스템에서 선택한 face를 각각 등록해도 이 진단의 페이로드는 SVG로 유지됐다. 이는 제한된 글꼴 등록을 사용한 layer 진단이며 Visual Sweep의 실제 글꼴 공급과 동일하다고 가정하지 않는다. WMF 장치 표면 생성이 어떤 조건에서 기존 SVG 경로로 돌아가는지는 아직 확인해야 한다.

원본 `image7.wmf`의 표준 WMFC 레코드 9개에서 EMF를 재조립하면 선언 크기와 실제 크기가 모두 70,668바이트이고 `EMR_HEADER` 및 ` EMF` 서명도 일치한다. 현재 `metafile_surface::payload`는 이처럼 유효한 EMF를 포함한 WMF를 명시적으로 반환 제외한다. 그 뒤 WMF 이미지 해소 경로는 SVG를 반환하므로 위 관측과 일치한다. 따라서 단순 글꼴 추가로 장치 표면이 생성되지 않는 이유를 코드와 원본 바이트에서 확인했다. 기존 EMF·ROP의 표현을 보존하면서 2024 장치 표본화에 전달할 수 있는지는 별도 검증이 필요하다.

공통 변환기가 이미 선택한 EMF+/WMF 내용을 2024 장치 표면으로 합성하는 후보를 Native에서 실행했다. 기존 한컴 2024 Print 35쪽 raw는 89.11%에서 89.50%로 상승했지만 90% 미만이다. 차이의 픽셀 영역은 차트 사각형 안에 한정됐다. 한 격자선 픽셀은 이전 RGB 235, 후보 229, PDF 225였다. 따라서 기존 SVG 경로를 보존하면서 장치 표면을 적용하는 것만으로는 완료되지 않는다. 현재 메타파일 표면 PNG에는 최종 합성 표식이 붙고 `bitmap_raster::prepare_bitmap`은 이를 저역 통과 대상에서 제외하므로, 고해상도 인쇄 표면을 최종 raster에 축소하는 단계의 실제 표본 처리와 다른 WMF·EMF 대조군을 추가 확인한다. 후보는 아직 채택·완료하지 않았다.

후속 후보에서 메타파일 장치 표면도 최종 raster로 축소할 때 공통 면적 표본기를 사용하도록 시험했다. 같은 2024 Print의 35쪽 Native raw는 92.57%, fresh WASM은 92.60%로 상승했다. [35쪽 비교](../assets/issue_8305/followup8438/native-resample-review-035.png)·[overlay](../assets/issue_8305/followup8438/native-resample-overlay-035.png)를 보존한다. 이 수치는 해당 후보의 35쪽만의 결과이고 전쪽, 기존 메타파일의 기능 보존과 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)·[PR #8397](https://github.com/edwardkim/rhwp/pull/8397) 기준 하락 여부를 확인하기 전이므로 최종 판정이 아니다.

확대 판독에서 기본 시스템 글꼴로 그린 차트의 값 글자가 한컴 PDF보다 가늘었다. 검증용으로 확보한 Calibri Bold를 `output/`에서 한 실행에만 공급하자 같은 후보의 Native 출력 PNG가 바뀌고 값 글자 굵기가 한컴과 가까워졌다. 전체 raw는 92.57351%에서 92.57265%로 거의 같아 페이지 점수만으로 이 차이를 판단할 수 없다. 동일 실행에서 `--font-path`만 전달한 경우 출력 PNG는 기본 실행과 같았고, 메타파일 글꼴 DB가 읽는 환경 경로로 공급한 경우에만 바뀌었다. 이는 추가 글꼴을 갖춘 진단이며 기본 시스템 환경의 TSV를 대체하지 않는다. 글꼴 파일은 저장소와 배포물에 포함하지 않는다.

전체 메타파일 표면에 축소를 적용한 위 후보는 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)의 sample16 2024 HWP와 HWPX에서 각각 18쪽 99.99→99.95%, 35쪽 99.99→99.98%로 하락했다. 같은 글꼴 환경에서 WMFC 합성만 추가하고 기존 표면 축소 계약을 유지한 후보의 HWP 18·35쪽은 각각 99.99273·99.99458%로 기준 표시값 99.99%를 유지했다. 따라서 관찰한 하락은 공통 축소 확대 후보에서 발생하며 허용하지 않는다. 기존 WMF의 검증된 장치 계약과 WMFC 내장 EMF에서 선택한 중간 표면을 구분할 근거를 검토한다. sample16은 선행 PR의 기능 보존 대조군이며 이 PR의 별도 개선 대상이 아니다.

2026-11-03 조판 보정 전 기준 후보 `dd284b5cda`의 Native 대조는 선행 PR 두 건의
기준 2,153행을 모두 산출했고, 각 문서의 17개 실행 manifest도 완료 상태다. Visual Sweep
표시값 기준 하락은 0행이다. 이 결과는 위 차트 축소 실험과 현재 수정 중인 코드의 결과가
아니며, 그 코드의 Native·fresh WASM 전체 비교와 종료 결과를 대신하지 않는다. 동일한
기준 후보의 fresh WASM 비교와 현재 수정본의 집중 검사는 실행 중이다.

2026-11-03 내장 EMF+가 실제로 채택된 경우에만 중간 표면을 만드는 후보는
최소 글꼴 집중 검사 7건이 통과했지만, 2024 Print 35쪽 Native raw가 89.11%여서
시각 검증에 실패했고 35쪽 review와 overlay를 직접 확인했다. 변환기가 내장 EMF+를
지원하지 않아 WMF 폴백을 선택하는 경우에도 중간
장치 축소가 필요한지 보정해 재검증한다. 이전 후보의 92.57%를 이 결과로 재사용하지 않는다.

이어 WMFC의 내장 EMF+를 지원하지 않아 공통 변환기가 WMF 폴백을 선택하는
경우에도 선택된 결과를 중간 장치 표면에서 축소하도록 보정했다. 해당 후보의
2024 Print 35쪽 Native raw는 92.57351%다. 현재 출력의 review와 overlay는 위의
92.57% 실험 이미지와 각각 SHA-256까지 같아 기존 이미지를 재사용한다. 같은
후보의 sample16 HWP와 HWPX 18·35쪽은 모두 표시값 99.99%로 선행 PR 기준을
유지했다. fresh WASM과 전쪽 대조는 계속 진행 중이다. 차트 결과는 35쪽 Native
선택 검증이다.

같은 후보에서 기존 한컴 2020 Print 기준 전 64쪽의 [Native TSV](../assets/issue_8305/followup8438/native-wmfc-2020-all.tsv)를
다시 산출했다. 최저 92.61%, 90% 미만과 누락은 0쪽이다. 이전 TSV와 표시값을
대조하면 55쪽이 95.89%에서 95.94%로 올랐고 나머지 쪽은 같다. 한컴 2024
기준 전쪽과 fresh WASM은 계속 확인 중이다.

같은 후보의 한컴 2024 Print 기준 [Native 전쪽 TSV](../assets/issue_8305/followup8438/native-wmfc-2024-all.tsv)는
64쪽 중 최저 42쪽 90.69%이며 90% 미만과 누락은 없다. 최저 쪽의 직접 판독과
fresh WASM 전쪽 대조는 별도로 진행한다.

회귀 대조군 sample16 HWPX의 메타파일 세 개(`image3.WMF`, `image5.WMF`, `image7.wmf`)에서는 WMFC 서명이 발견되지 않았다. 반면 이 차트의 `image7.wmf`는 앞서 확인한 유효한 WMFC 내장 EMF를 가진다. 이 바이트 차이는 일반 WMF의 기존 장치 계약과 내장 EMF를 선택한 새 경로를 구분할 수 있는 근거이며, 파일명이나 문서 번호로 동작을 갈라야 한다는 뜻은 아니다.

출력 경로 구분을 위해 HWPX 복사본의 `image7.wmf`에서 WMFC 레코드 9개만 제거하고 WMF 길이를 갱신한 임시 진단본을 만들었다. 기존 제품 코드로 같은 35쪽을 출력하면 원본 출력과 다른 33,178픽셀이 모두 차트 사각형 안에 나타났다. 내장 EMF 내용과 일반 WMF 폴백이 이 입력에서 같은 출력을 만든다고 가정할 수 없으며, 이 조작본은 한컴 기준이나 정식 fixture가 아니다.

표본화 진단으로 같은 현재 Native 엔진의 전체 35쪽을 600 DPI로 출력한 뒤
Lanczos로 96 DPI 크기에 축소해 동일 PDF와 비교했다. 전쪽 raw는 89.11%에서
93.17%, 차트 영역 raw는 71.45%에서 83.13%로 달라졌다. 이 변환 이미지는
원래 96 DPI 출력이 아니므로 통과 TSV나 변경 후 결과로 사용하지 않는다.
한컴 PDF의 고해상도 차트와 장치 표본화가 차이에 관여한다는 진단이며,
기존 WMF·ROP 및 다른 해상도에서의 독립 대조를 거쳐 공통 경로를 정해야 한다.

### 2026-11-02 새 base 최소 글꼴 집중 회귀

검증 기준 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`에서 `issue_8305_titlemark_boundary_runs` 5건을 실행해 모두 통과했다.
전체 선택 범위는 28건 통과이며 전체 회귀 실행 결과가 아니다. 시스템 글꼴을 격리하고
저장소의 재배포 가능한 최소 글꼴만 노출했다. 기존 Print 시각 증거와는 별도로 판정한다.

### 현재 후보의 한컴 2020 Print 재대조

제품 코드 `dd284b5cda`의 fresh WASM(SHA-256 `cfe1e98fc6c4d7b2a4ac2fa23c6565fc3b32afe2d4ecf2ce71880d22318e86d8`)으로 같은 저장 HWPX와 기존 한컴 2020 Print를 96 DPI에서 비교했다. 전 64쪽 최저는 92.41%, 90% 미만은 0쪽이다. 산출 TSV의 SHA-256 `1d60128ddabaf52e276f010954cc8419da833ad8b000bbe8fd15096c791a55a0`은 [기존 TSV](../assets/issue_8305/followup8438/wasm-saved-original2020-all.tsv)와 같아 재사용한다. 현재 출력의 [20쪽 비교](../assets/issue_8305/followup8438/wasm-current-review-020.png)·[overlay](../assets/issue_8305/followup8438/wasm-current-overlay-020.png), [59쪽 비교](../assets/issue_8305/followup8438/wasm-current-review-059.png)·[overlay](../assets/issue_8305/followup8438/wasm-current-overlay-059.png)를 보존한다. 이 결과는 별도 한컴 2024 Print 35쪽의 미달을 해결한 근거가 아니다.

### 기존 한컴 2020 기준의 저장본 WASM 대조

원 PR이 사용한 [한컴 2020 Print](../../../pdf/issue4599/1790387_policy_progress-hwpx-2020.pdf)와
재저장 HWPX를 96 DPI로 비교한 [64쪽 WASM TSV](../assets/issue_8305/followup8438/wasm-saved-original2020-all.tsv)는
최저 92.41%(42쪽), 90% 미만 0쪽이다. 중단됐던 계산은 기존 PNG와 입력 해시를 확인해
쪽별 체크포인트부터 이어서 완료했다. 이 결과는 이전 통합 후보의 WASM 패키지로 산출한 값이며
새 base에서 빌드한 WASM의 실행 결과가 아니다. 새 base는 이후 HWP 저장 경로가 바뀌었고,
그 외 렌더러는 이전 후보와 같음을 별도로 확인했다. 2024 Print 비교 결과와 혼동하지 않는다.

같은 저장본과 기준 PDF의 [Native 전체 TSV](../assets/issue_8305/followup8438/native-saved-original2020-all.tsv)는
64쪽 최저 92.61%(30쪽), 90% 미만 0쪽이다. 기존 Native 실행 파일 SHA-256은
`34dfddd062947e91c33cb3c2b2c16a10f9903d2d1184fcdb646683e6a0dc2f1f`이다.
중단 전에 완성된 PNG와 같은 실행 파일로 나머지 쪽을 다시 내보낸 PNG를 원본
해시와 함께 대조했다. 이 결과는 표 원점 후보 수정 전의 측정이다.

원 PR이 지목한 제목 차례 영역을 같은 입력의 기존 PNG로 다시 구성했다.
[20쪽 Native review](../assets/issue_8305/followup8438/native-original2020-review-020.png) ·
[20쪽 Native overlay](../assets/issue_8305/followup8438/native-original2020-overlay-020.png) ·
[59쪽 Native review](../assets/issue_8305/followup8438/native-original2020-review-059.png) ·
[59쪽 Native overlay](../assets/issue_8305/followup8438/native-original2020-overlay-059.png) ·
[20쪽 WASM review](../assets/issue_8305/followup8438/wasm-original2020-review-020.png) ·
[20쪽 WASM overlay](../assets/issue_8305/followup8438/wasm-original2020-overlay-020.png) ·
[59쪽 WASM review](../assets/issue_8305/followup8438/wasm-original2020-review-059.png) ·
[59쪽 WASM overlay](../assets/issue_8305/followup8438/wasm-original2020-overlay-059.png).
제목과 주변 본문의 순서·표시는 유지되며, 세부 글자 모양과 점선 차이도 함께 보인다.

### 2026-11-02 의존성 리베이스 후 전체 Rust 회귀

최소 글꼴 환경에서 전체 선택 12,369건을 두 실행으로 완료해 모두 통과했다.
첫 실행의 중단과 나머지 307건의 재실행, 정확한 목록 대조는
[공통 실행 근거](../assets/review_8438_followup/rebase-de2295-rust.md)에 기록했다.
이는 이 PR의 시각 검증이나 최종 판정을 대신하지 않는다.
