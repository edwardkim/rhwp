# 묶기·풀기 원 PR의 현재 API 저장본과 한컴 Print 증거

검증 제품은 `6fc2b3490d`의 Native079와 최적화 fresh WASM031이다. 원 #7588·#8041·#8042의
정식 테스트 helper를 현재 제품에 연결해 HWP/HWPX 14쌍을 다시 생성했다. helper 소스 SHA와
입력·PDF·TSV SHA, 실제 실행 명령·한컴 빌드·Print 경로는 [provenance.json](provenance.json)에 있다.

HWPX 14개는 기존 한컴 Print 입력과 바이트까지 같아 입력/PDF SHA가 함께 맞는 영수증으로
기준을 재사용했다. HWP 14개는 저장 제품을 확인한 뒤 실제 한컴2020 MCP로 새로 Print했다.
모두 `11.0.0.9136`, PrintMethod0, `hancom2020_pdf_driver_one_up`, 전체 1쪽이다. 입력 전처리는
각 영수증에 구분했다. 다른 형식의 PDF를 같은 입력의 기준으로 추정하지 않았다.

## 전쪽 결과

28개 조건 모두 Native/fresh WASM raw100%, 전체 쪽수1=1이다. TSV는 원문 바이트를 보존했다.
100%는 2px 이웃 실루엣 값이며 RGB 완전 일치를 뜻하지 않는다. 아래 표는 HWP와 HWPX의
독립 Print·전쪽 TSV를 각각 연결한다.

| 입력 | PDF | Native TSV | fresh WASM TSV |
| --- | --- | --- | --- |
| [pr7588-pr8042-group-0-hwp](inputs/hwp/pr7588-pr8042-group-0.hwp) | [Print](pdf/hwp/pr7588-pr8042-group-0-2020.pdf) | [100%](native/pr7588-pr8042-group-0-hwp_silhouette.tsv) | [100%](wasm/pr7588-pr8042-group-0-hwp_silhouette.tsv) |
| [pr7588-pr8042-group-0-hwpx](inputs/hwpx/pr7588-pr8042-group-0.hwpx) | [Print](pdf/hwpx/pr7588-pr8042-group-0-2020.pdf) | [100%](native/pr7588-pr8042-group-0-hwpx_silhouette.tsv) | [100%](wasm/pr7588-pr8042-group-0-hwpx_silhouette.tsv) |
| [pr7588-pr8042-group-30-hwp](inputs/hwp/pr7588-pr8042-group-30.hwp) | [Print](pdf/hwp/pr7588-pr8042-group-30-2020.pdf) | [100%](native/pr7588-pr8042-group-30-hwp_silhouette.tsv) | [100%](wasm/pr7588-pr8042-group-30-hwp_silhouette.tsv) |
| [pr7588-pr8042-group-30-hwpx](inputs/hwpx/pr7588-pr8042-group-30.hwpx) | [Print](pdf/hwpx/pr7588-pr8042-group-30-2020.pdf) | [100%](native/pr7588-pr8042-group-30-hwpx_silhouette.tsv) | [100%](wasm/pr7588-pr8042-group-30-hwpx_silhouette.tsv) |
| [pr7588-pr8042-ungroup-0-hwp](inputs/hwp/pr7588-pr8042-ungroup-0.hwp) | [Print](pdf/hwp/pr7588-pr8042-ungroup-0-2020.pdf) | [100%](native/pr7588-pr8042-ungroup-0-hwp_silhouette.tsv) | [100%](wasm/pr7588-pr8042-ungroup-0-hwp_silhouette.tsv) |
| [pr7588-pr8042-ungroup-0-hwpx](inputs/hwpx/pr7588-pr8042-ungroup-0.hwpx) | [Print](pdf/hwpx/pr7588-pr8042-ungroup-0-2020.pdf) | [100%](native/pr7588-pr8042-ungroup-0-hwpx_silhouette.tsv) | [100%](wasm/pr7588-pr8042-ungroup-0-hwpx_silhouette.tsv) |
| [pr7588-pr8042-ungroup-30-hwp](inputs/hwp/pr7588-pr8042-ungroup-30.hwp) | [Print](pdf/hwp/pr7588-pr8042-ungroup-30-2020.pdf) | [100%](native/pr7588-pr8042-ungroup-30-hwp_silhouette.tsv) | [100%](wasm/pr7588-pr8042-ungroup-30-hwp_silhouette.tsv) |
| [pr7588-pr8042-ungroup-30-hwpx](inputs/hwpx/pr7588-pr8042-ungroup-30.hwpx) | [Print](pdf/hwpx/pr7588-pr8042-ungroup-30-2020.pdf) | [100%](native/pr7588-pr8042-ungroup-30-hwpx_silhouette.tsv) | [100%](wasm/pr7588-pr8042-ungroup-30-hwpx_silhouette.tsv) |
| [pr8041-case-1-after-hwp](inputs/hwp/pr8041-case-1-after.hwp) | [Print](pdf/hwp/pr8041-case-1-after-2020.pdf) | [100%](native/pr8041-case-1-after-hwp_silhouette.tsv) | [100%](wasm/pr8041-case-1-after-hwp_silhouette.tsv) |
| [pr8041-case-1-after-hwpx](inputs/hwpx/pr8041-case-1-after.hwpx) | [Print](pdf/hwpx/pr8041-case-1-after-2020.pdf) | [100%](native/pr8041-case-1-after-hwpx_silhouette.tsv) | [100%](wasm/pr8041-case-1-after-hwpx_silhouette.tsv) |
| [pr8041-case-1-before-hwp](inputs/hwp/pr8041-case-1-before.hwp) | [Print](pdf/hwp/pr8041-case-1-before-2020.pdf) | [100%](native/pr8041-case-1-before-hwp_silhouette.tsv) | [100%](wasm/pr8041-case-1-before-hwp_silhouette.tsv) |
| [pr8041-case-1-before-hwpx](inputs/hwpx/pr8041-case-1-before.hwpx) | [Print](pdf/hwpx/pr8041-case-1-before-2020.pdf) | [100%](native/pr8041-case-1-before-hwpx_silhouette.tsv) | [100%](wasm/pr8041-case-1-before-hwpx_silhouette.tsv) |
| [pr8041-case-2-after-hwp](inputs/hwp/pr8041-case-2-after.hwp) | [Print](pdf/hwp/pr8041-case-2-after-2020.pdf) | [100%](native/pr8041-case-2-after-hwp_silhouette.tsv) | [100%](wasm/pr8041-case-2-after-hwp_silhouette.tsv) |
| [pr8041-case-2-after-hwpx](inputs/hwpx/pr8041-case-2-after.hwpx) | [Print](pdf/hwpx/pr8041-case-2-after-2020.pdf) | [100%](native/pr8041-case-2-after-hwpx_silhouette.tsv) | [100%](wasm/pr8041-case-2-after-hwpx_silhouette.tsv) |
| [pr8041-case-2-before-hwp](inputs/hwp/pr8041-case-2-before.hwp) | [Print](pdf/hwp/pr8041-case-2-before-2020.pdf) | [100%](native/pr8041-case-2-before-hwp_silhouette.tsv) | [100%](wasm/pr8041-case-2-before-hwp_silhouette.tsv) |
| [pr8041-case-2-before-hwpx](inputs/hwpx/pr8041-case-2-before.hwpx) | [Print](pdf/hwpx/pr8041-case-2-before-2020.pdf) | [100%](native/pr8041-case-2-before-hwpx_silhouette.tsv) | [100%](wasm/pr8041-case-2-before-hwpx_silhouette.tsv) |
| [pr8041-case-3-base-0-after-hwp](inputs/hwp/pr8041-case-3-base-0-after.hwp) | [Print](pdf/hwp/pr8041-case-3-base-0-after-2020.pdf) | [100%](native/pr8041-case-3-base-0-after-hwp_silhouette.tsv) | [100%](wasm/pr8041-case-3-base-0-after-hwp_silhouette.tsv) |
| [pr8041-case-3-base-0-after-hwpx](inputs/hwpx/pr8041-case-3-base-0-after.hwpx) | [Print](pdf/hwpx/pr8041-case-3-base-0-after-2020.pdf) | [100%](native/pr8041-case-3-base-0-after-hwpx_silhouette.tsv) | [100%](wasm/pr8041-case-3-base-0-after-hwpx_silhouette.tsv) |
| [pr8041-case-3-base-0-before-hwp](inputs/hwp/pr8041-case-3-base-0-before.hwp) | [Print](pdf/hwp/pr8041-case-3-base-0-before-2020.pdf) | [100%](native/pr8041-case-3-base-0-before-hwp_silhouette.tsv) | [100%](wasm/pr8041-case-3-base-0-before-hwp_silhouette.tsv) |
| [pr8041-case-3-base-0-before-hwpx](inputs/hwpx/pr8041-case-3-base-0-before.hwpx) | [Print](pdf/hwpx/pr8041-case-3-base-0-before-2020.pdf) | [100%](native/pr8041-case-3-base-0-before-hwpx_silhouette.tsv) | [100%](wasm/pr8041-case-3-base-0-before-hwpx_silhouette.tsv) |
| [pr8041-case-3-base-1-after-hwp](inputs/hwp/pr8041-case-3-base-1-after.hwp) | [Print](pdf/hwp/pr8041-case-3-base-1-after-2020.pdf) | [100%](native/pr8041-case-3-base-1-after-hwp_silhouette.tsv) | [100%](wasm/pr8041-case-3-base-1-after-hwp_silhouette.tsv) |
| [pr8041-case-3-base-1-after-hwpx](inputs/hwpx/pr8041-case-3-base-1-after.hwpx) | [Print](pdf/hwpx/pr8041-case-3-base-1-after-2020.pdf) | [100%](native/pr8041-case-3-base-1-after-hwpx_silhouette.tsv) | [100%](wasm/pr8041-case-3-base-1-after-hwpx_silhouette.tsv) |
| [pr8041-case-3-base-1-before-hwp](inputs/hwp/pr8041-case-3-base-1-before.hwp) | [Print](pdf/hwp/pr8041-case-3-base-1-before-2020.pdf) | [100%](native/pr8041-case-3-base-1-before-hwp_silhouette.tsv) | [100%](wasm/pr8041-case-3-base-1-before-hwp_silhouette.tsv) |
| [pr8041-case-3-base-1-before-hwpx](inputs/hwpx/pr8041-case-3-base-1-before.hwpx) | [Print](pdf/hwpx/pr8041-case-3-base-1-before-2020.pdf) | [100%](native/pr8041-case-3-base-1-before-hwpx_silhouette.tsv) | [100%](wasm/pr8041-case-3-base-1-before-hwpx_silhouette.tsv) |
| [pr8041-case-3-base-2-after-hwp](inputs/hwp/pr8041-case-3-base-2-after.hwp) | [Print](pdf/hwp/pr8041-case-3-base-2-after-2020.pdf) | [100%](native/pr8041-case-3-base-2-after-hwp_silhouette.tsv) | [100%](wasm/pr8041-case-3-base-2-after-hwp_silhouette.tsv) |
| [pr8041-case-3-base-2-after-hwpx](inputs/hwpx/pr8041-case-3-base-2-after.hwpx) | [Print](pdf/hwpx/pr8041-case-3-base-2-after-2020.pdf) | [100%](native/pr8041-case-3-base-2-after-hwpx_silhouette.tsv) | [100%](wasm/pr8041-case-3-base-2-after-hwpx_silhouette.tsv) |
| [pr8041-case-3-base-2-before-hwp](inputs/hwp/pr8041-case-3-base-2-before.hwp) | [Print](pdf/hwp/pr8041-case-3-base-2-before-2020.pdf) | [100%](native/pr8041-case-3-base-2-before-hwp_silhouette.tsv) | [100%](wasm/pr8041-case-3-base-2-before-hwp_silhouette.tsv) |
| [pr8041-case-3-base-2-before-hwpx](inputs/hwpx/pr8041-case-3-base-2-before.hwpx) | [Print](pdf/hwpx/pr8041-case-3-base-2-before-2020.pdf) | [100%](native/pr8041-case-3-base-2-before-hwpx_silhouette.tsv) | [100%](wasm/pr8041-case-3-base-2-before-hwpx_silhouette.tsv) |

## 직접 판독과 의미 검사

30도 자식 묶음(HWPX)과 크기·회전 자식 묶음(HWP)의 같은 쪽을 Native/fresh의
review·standalone overlay로 직접 판독했다. 위치·회전·외곽·겹침 순서를 확인했다.

| 대조 | Native | fresh WASM |
| --- | --- | --- |
| 30도 자식 묶음 | [review](native_rotated-group_review.png) · [overlay](native_rotated-group_overlay.png) | [review](wasm_rotated-group_review.png) · [overlay](wasm_rotated-group_overlay.png) |
| 크기·회전 자식 | [review](native_scaled-ellipse_review.png) · [overlay](native_scaled-ellipse_overlay.png) | [review](wasm_scaled-ellipse_review.png) · [overlay](wasm_scaled-ellipse_overlay.png) |

테두리가 약해 보이는 차이도 별도로 조사했다. 원본 선 속성 33HU에서 한컴 Print pen 단위로
환산한 실제 stroke는 0.48 CSS px이며 PDF vector도 0.36pt로 같은 굵기다. 실제 출력에 검은
stroke가 있음을 SVG와 Native/Canvas 픽셀에서 확인했다. 저해상도 경계 농도 차이는 남는다.
600DPI 추가 진단은 농도 차이 분석에만 사용했고 정식 96DPI TSV·PNG·허용치는 바꾸지 않았다.

누름틀의 필드 이름·안내문·범위와 저장 전후 보존은
`ungroup_content_geometry`의 4개 검사로 별도 확인했다. 빈 누름틀의 Print가 보이지 않거나
그림이 같다는 사실만으로 필드 보존을 입증하지 않는다. 원 #8041의 자식 위치·크기·회전·
z-order 및 두 형식 저장은 `group_shapes_child_geometry`의 3개 검사로 확인했다. 두 파일 모두
현재 Native079에서 PASS했고, #8041/#8042 원 head와 현재 정식 소스 바이트가 같다.

본문 활성 누름틀을 실제로 설정한 별도 공개 API 진단은 묶기·풀기×HWP/HWPX 4조건에서
같은 활성 주소를 유지했다. 반면 같은 문단의 도형 뒤 표 셀에 있는 활성 누름틀은 표 번호가
바뀌어도 옛 소유 번호에 남는 반례를 확인했다. 위 100% 이미지는 이 상태 보존을 입증하지
않는다. [공통 주소 분석과 보정 계획](../../../working/task_semanticist21_20261021_group_active_field_owner.md)에
따라 보정·재검증을 계속하며 활성 셀 주소 검증 완료로 판정하지 않는다.

전체 통합 후보의 Rust·lint·다른 문서·최신 devel 기능 보존·원격 CI 판정은 별도다.
