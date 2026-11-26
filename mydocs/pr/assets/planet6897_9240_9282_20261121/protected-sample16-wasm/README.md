# sample16 세 저장본의 WASM 보호값 재현

제품 코드 `19646e69fc9`의 fresh WASM으로 2022 HWP, 2024 HWP, HWPX 각 64쪽을 비교했다. 기준 입력, 데스크톱 한컴 2020·2024 Print PDF, 조판 세대, 96 DPI와 2px raw 비교는 정본과 같다. 이전 공급에서 51·61·62쪽이 낮았다. 이번 공급에서는 세 저장본의 9쪽 모두 100.00%이며, 전 192쪽 중 기존 표시값보다 낮은 쪽은 없다. 최저값은 2022 HWP 100.00%, 2024 HWP와 HWPX 99.99%다. [검증 기록](validation.json)

기존 Print Type3의 `○`(U+25CB)는 `SPMJ.otf`의 윤곽과 같다. 현재 사용자 글꼴의 `SPSMJ.otf`도 같은 Unicode를 다른 큰 원형에 매핑한다. 먼저 Print와 일치하는 `SPMJ.otf`를 공급했다. 원래 보존한 `SPSMJ.otf`를 함께 공급하자 51쪽의 큰 원형 차이가 해소됐다. 위 두 프로그램만 검증에 선택했으며 개인 글꼴 설치는 그대로다. [독립 윤곽 대조](../../pr8896/font-supply/circle-contour-equivalence.json) · [HWPX 51쪽 review](sample16-hwpx-review-051.png) · [overlay](sample16-hwpx-overlay-051.png)

원 수용 환경은 과거 Linux의 글꼴 목록 70개를 기록한다. 이 Mac에서는 55개를 해시 단위로 확인했다. 따라서 전쪽 표시값 재현 기록과 과거 모든 공급 파일의 동일성은 구분한다. 정본 TSV는 그대로 보존했다. [2022 HWP](sample16-hwp2022.tsv) · [2024 HWP](sample16-hwp2024.tsv) · [HWPX](sample16-hwpx.tsv)

최종 제품 소스 `65b6fe383c3bd8cbcf83de7909c7a6c24eb32c35`와 고정 `devel`을 같은 공급 조건으로 실행했다. 2022 HWP·2024 HWP·HWPX의 Native와 fresh WASM **6조건·384쪽**에서 고정 기준과 후보의 TSV가 각각 바이트 단위로 일치한다. 2022 HWP는 두 경로의 전쪽이 100.00%이며 나머지 두 저장본의 최저는 99.99%다. [최종 전쪽 TSV 및 생산물 조건](../protected-candidate-v5/sample16-paired.json)
