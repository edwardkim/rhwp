# 저장 표 통합 검토의 출력 증거

초기 `upstream/devel` 기준은 `32014de6e0ade1a3baea8852c02270e40b27b345`다.
Native와 fresh WASM은 제품 소스 `e766629fc7519e23b8aa8dcb861564eb9a63349d`에서 만들었다.
초기 제출 때 회귀 커밋 `d4df7bf7a6f9584f0bc1796c240a01af9b5048ae`과 제품 소스는 같았다.

한컴 Print PDF를 96 DPI에서 인쇄 배치로 대조했다. TSV는 색상 경계 보정 전 2px 실루엣의
`silhouette_raw_match_percent` 표시값을 담고 있다. 각 디렉터리의 `review_*.png`는 RHWP·한컴 PDF·
overlay를, `overlay_*.png`는 해당 쪽의 차이를 담는다.

| 원 PR과 원본 | 한컴·후보 Native·후보 fresh WASM | Native | fresh WASM |
| --- | ---: | --- | --- |
| [#9362](../../archives/pr_9362_review.md) 제목 표·본문 표와 꼬리 글줄 | 2쪽 | [TSV](negative-spacing/native/silhouette.tsv) · [1쪽 비교](negative-spacing/native/review_001.png) | [TSV](negative-spacing/wasm/silhouette.tsv) · [1쪽 비교](negative-spacing/wasm/review_001.png) |
| [#9378](../../archives/pr_9378_review.md) 앞 글줄과 표의 이어받기 | 2쪽 | [TSV](visible-host/native/silhouette.tsv) · [2쪽 비교](visible-host/native/review_002.png) | [TSV](visible-host/wasm/silhouette.tsv) · [2쪽 비교](visible-host/wasm/review_002.png) |
| [#9381](../../archives/pr_9381_review.md) 13pt 제목 | 3쪽 | [TSV](negative-anchor/native/silhouette.tsv) · [2쪽 비교](negative-anchor/native/review_002.png) | [TSV](negative-anchor/wasm/silhouette.tsv) · [2쪽 비교](negative-anchor/wasm/review_002.png) |
| [#9381](../../archives/pr_9381_review.md) 10pt 제목 | 2쪽 | [TSV](negative-anchor-fits/native/silhouette.tsv) · [1쪽 비교](negative-anchor-fits/native/review_001.png) | [TSV](negative-anchor-fits/wasm/silhouette.tsv) · [1쪽 비교](negative-anchor-fits/wasm/review_001.png) |

후보는 각 원본에 대해 두 경로 모두 전쪽 100.00%였다. 원 PR 제출값과 페이지별로 대조해 하락이
없음을 확인했다. #9362 1쪽은 Native 98.98%, fresh WASM 98.97%에서 각각 100.00%로 개선됐다.
#9066·#9411·#9415의 영향 경계 19쪽에 대해 기준과 후보의 각 TSV와 PNG 해시도 일치했다.
[페이지별 비교, 바이너리 및 PNG 해시](fixture-audit.json)

## 한컴 재저장 대조군

제공된 #9362 합성 입력을 한컴 2020 11.0.0.9172로 재저장해 160%의 정상 저장 간격 388 HWPUNIT와
다음 줄 원점 2036 HWPUNIT를 관찰했다. 같은 공개 입력의 문단 모양을 90%로 바꾸고 한컴에서 재저장해
간격 −64 HWPUNIT와 원점 1584 HWPUNIT를 관찰했다. 두 HWP를 동일 엔진의 1-up PDF 드라이버로
인쇄했고, 각 2쪽을 Native와 fresh WASM으로 비교했다. 두 대조군의 양쪽 TSV는 전쪽 100.00%다.
[재저장 및 인쇄 영수증](9362-hancom-controls/receipts.json)

- 160% 재저장본 [Native TSV](9362-hancom-controls/resaved-native.tsv) · [fresh WASM TSV](9362-hancom-controls/resaved-wasm.tsv)
- 90% 재저장본 [Native TSV](9362-hancom-controls/compact-native.tsv) · [fresh WASM TSV](9362-hancom-controls/compact-wasm.tsv)

후보의 Native Skia 바이너리와 fresh WASM JS·바이너리를 별도 경로에 고정했다. macOS 호스트에서
`scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt`를 실행한 뒤 별도의 호스트
`wasm-opt`(Binaryen 130) 최적화를 거쳤다. Chrome 158.0.8127.88에서 이 WASM의 Canvas를
출력했고, Studio가 제공한 JS·WASM 해시, 쪽수 2·2·3·2, 해당 쪽의 표·본문 및 중복 여부를 확인했다.

정본 보호 23종·4,046개 출력 쪽의 표시값은 전부 정본과 같았다. 원 PR 세 건이 제출한 공통 검사와 편람 TSV 27개도 각각 전쪽 대조했으며 하락이 없었다. [정본 보호 검사](protected-audit.json) · [원 PR 전쪽 비교](fixture-audit.json)

편람 HWPX 383쪽은 Native 최저 99.97%, fresh WASM 최저 99.96%로 각각 기준 표시값을 유지했다. [편람 Native 전쪽 TSV](handbook-hwpx-native.tsv) · [편람 fresh WASM 전쪽 TSV](handbook-hwpx-wasm.tsv)

## 코드 검사

macOS에서 fmt, Native·WASM32 Clippy, workspace 빌드·all-targets Clippy와
기준 devel 대비 테스트 manifest 검사를 통과했다. Native Skia는 최소 글꼴 환경에서
라이브러리와 CI의 21개 통합 모듈을 실행해 4,225건 통과, 실패 0건, ignored 12건을 기록했다.
Ubuntu 최소 글꼴 환경의 집중 회귀는 12건을 통과했다. macOS 최소 글꼴 환경의 전체 nextest는 14,127건 통과, 실패 0건, skip 45건이었으며 SVG golden 검사도 통과했다.
[실행 건수와 글꼴 격리 근거](regression-audit.json)

## #9412 병합 후 재검증

제품 기준 `5fa8cede504eca188d2e003931a50e9a8434d6da`와 후보 `c8c869349182a68067b8da69fbabcbd164b0cc73`의 Native 및 최적화 WASM을 각각 빌드했다.
#9412가 검증한 71개 입력·인쇄 조합에서 각 출력 경로의 PNG 110쪽이 기준과 바이트 단위로
같았고, 52,694개 칸이 있는 대형 표의 161쪽 배치 JSON도 같았다.
[#9412 쪽별 대조 기록](9412-preservation.json)

정본 보호 23종의 두 출력 경로 4,046개 쪽은 기준 표시값과 모두 일치했다. 화학물질 표시 기준
WASM 26쪽은 macOS 같은 조건에서 기준과 후보가 99.99%였으며 둘의 PNG가 같았다. 동일한
WASM과 검증된 글꼴 52개를 Ubuntu Chromium 158.0.8127.36에 공급해 문서 전체를
다시 검사했다. 기준과 후보의 103쪽 PNG는 전부 같았고 26쪽은 #9412의 PNG 해시와
100.00%를 재현했다. [호스트 대조 기록](chemical-wasm-cross-host.json) ·
[Ubuntu 전쪽 TSV](chemical-wasm-ubuntu.tsv)

원 PR 네 문서 9쪽은 Native와 WASM에서 모두 100.00%였다. #9362의 8.9px 간격과 90%
압축 대조본을 확인했고, 브라우저와 Studio의 쪽·표·본문 검사도 통과했다.
최소 글꼴 환경의 Ubuntu 집중 회귀 45건과 전체 nextest 14,169건,
macOS Native Skia 4,225건이 통과했다. [실행 건수](regression-audit.json)
