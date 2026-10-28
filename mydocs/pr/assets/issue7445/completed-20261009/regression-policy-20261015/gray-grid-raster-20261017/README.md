# 생물독소 회색 표 선의 인쇄 coverage 보정

## 원본·독립 근거

원본 `samples/issue5941/1480000-201900698-native-neartop-reset.hwp`는205쪽이며
SHA-256은 `18765ac22270a2cb4ece6a1e6539c2dea6ca6ccde2573b42fb9acce7c34fa83a`다.
독립 한컴2020 Print `pdf/issue5941-1480000-201900698-2020.pdf`의 SHA-256은
`fe01f1ddffc3153d015adab93178f0fa8a5e9624763ff529362d4410d50955dc`다.
compat2022·print·96DPI, 기존 공통 공급 폴더와 실제 `Library/Fonts`를 사용했다.
글꼴 바이너리·원본·PDF·비교기 허용치를 변경하지 않았다.

143쪽 표42의 선은 원본과 PDF 모두 `#bfbfbf`,0.48CSSpx(0.36pt)다.
기존 AA가 이 폭을 두 장치 픽셀로 나누면서 실제 선이RGB238~243으로 옅어졌다.
독립 PDF의 같은 내부 선은RGB224다. 좌표·색·선폭의 오류와 raster coverage를 구별한다.

## 구현·회귀

`renderer/raster_stroke.rs`는 실제 affine raster 변환을 읽고, 인쇄/고품질의1장치px 미만
직교 단일 실선만 법선 방향으로 최대0.5장치px 이동한다. 폭·색·길이·문서 배치는 그대로다.
Native와 Web Canvas는 같은 계산을 소비하며 CanvasKit adapter도 같은 계약을 구현한다.
화면 프로필·사선·복합선·화살표·그림자·굵은 선·특이 변환은 이 처리 대상이 아니다.

정식 `gray_grid_keeps_its_print_coverage_cell_owners_and_pdf_origin`은 source 표42의
셀 단일 소유·내용 보존·205쪽과 독립 PDF의 좌우 경계(±2CSSpx)를 검사한다.
실제 Native Print PNG의 내부 선 위치(±2px)와AA 색(RGB224±8)도 확인한다.
모델 선 존재만으로 최종 raster의 선 소실을 놓치지 않도록 한다.
before는4PASS/1FAIL(0.807초), after는5PASS(0.799초)다.
CanvasKit adapter 단위4건과 Studio build도 통과했다.

## C80 전쪽 결과·생산 출처

| 범위 | Native raw | fresh WASM raw |
| --- | ---: | ---: |
| 수정 전143쪽 |86.88424%|89.99173%|
| 수정 후143쪽 |99.97632%|99.97501%|
| 전205쪽 최저(C80) |93.26163%|93.26176%|

양쪽205/205쪽·90% 미달0쪽이다. 판정은 색상 경계 보정 전
`silhouette_raw_match_percent`이며 조정 점수로90/98% 자격을 대신하지 않았다.
143쪽 review·overlay에서 흐린 grid 복원과 표 뒤 내용 보존을 직접 확인했다.
이는143쪽의 인쇄 선 보정 증거이며, 최저63쪽의 캡션 겹침은 별도C81 보정 중이다.
점수90% 통과를 전체 시각 완료로 보고하지 않는다.

생산 부모 `58cc38bb92f40fbc796a1b991a49e88aee39ce2d`, 기준 devel
`552514f423ec91f1b7171f6e6b0882a4efe034f2`다. C80 후보 source diff SHA-256은
`2e2b80320b4fe1cb2d0cfb83ba7506935a92fcd3866baec9d5e1b718fa7e7389`다.
Native binary SHA-256은 `4944fa7c2c66f53d8b4d068f63c4c78a859128b680f1c81d3d05278040255a21`,
fresh WASM SHA-256은 `880e72f19a4af5d9ce3402d1be3ad840616871edd1364a074b7146c5f1e1f73f`다.
WASM은 Mac locked wrapper의web/no-opt 로컬 대체 빌드이며 Docker 최적화 통과가 아니다.

## 남은 검증·실제 Studio

- 현재 실제 Studio Canvas2D 캡처의143쪽 인쇄 raw는84.38222%다. 회색 선과 화면 합성의
  차이가 남았으며 Native/fresh WASM export 통과로 이 소비 경로의 완료를 대신하지 않는다.
- 생물독소와rowbreak18의 실제 CanvasKit 요청은 글꼴 요구 목록 준비 실패로Canvas2D로
  전환됐다. 새 adapter의 단위 통과와 실제 CanvasKit 사용 성공은 다른 증거다.
- low-slack6535의 선별 Native는95.72492→95.69965%로0.02527%p 하락했다.
  같은 쪽을 직접 대조했지만 아직 고정 base 전쪽 비교까지 완료한 무회귀 판정은 아니다.
- 다른 원본의raw 미달과 최종 전체 회귀·필수 게이트·실제 소비 검증은 계속한다. PR은 보류다.

TSV·상세 source/font hash·로그는 ignored
`output/pr-review/7445-completed-20261009/regression-policy-audit-20261015/rebase-552514f4-20261016/thin-stroke-c80/`
아래에 보존한다.

| 출력 | review | overlay |
| --- | --- | --- |
| 수정 전 Native |[review](before-native-review.png)|[overlay](before-native-overlay.png)|
| 수정 후 Native |[review](after-native-review.png)|[overlay](after-native-overlay.png)|
| 수정 전 WASM |[review](before-wasm-review.png)|[overlay](before-wasm-overlay.png)|
| 수정 후 WASM |[review](after-wasm-review.png)|[overlay](after-wasm-overlay.png)|
