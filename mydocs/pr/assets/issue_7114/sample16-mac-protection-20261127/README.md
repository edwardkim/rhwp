# sample16 수식 6쪽 보호 검증

sample16 HWP 2022·HWP 2024·HWPX 각 64쪽을 기존 독립 한컴 Print PDF와 96 DPI의 Native Skia·fresh WASM Canvas로 비교했다. 최신 devel과 #9306의 병합 후보는 같은 Mac 공급에서 384쪽의 PNG 해시와 두 자리 표시값이 모두 일치했다.

한컴 설치본 HYHWPEQ.TTF 1.13을 사용한 실측값은 HWP 2022 Native 6쪽이 99.96%, 나머지 다섯 조건의 6쪽이 각각 99.97%다. 같은 영역을 확대해 등호·더하기·괄호·분수선의 윤곽 차이를 확인했다. 허용된 1.10의 Native 6쪽은 세 형식에서 각각 99.95%였다. 사용자는 글꼴 차이로 확인된 하락을 실측값으로 조정하도록 승인했다.

[전쪽 비교](paired-pages.tsv) · [글꼴 파일·순서와 생산 SHA](conditions.json) · [6쪽 확대 비교](sample16-hwpx-p6-equation-detail.png)
