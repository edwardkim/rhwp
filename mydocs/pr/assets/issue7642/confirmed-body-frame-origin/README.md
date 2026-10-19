# 임시 저장 원점과 본문 꼬리의 독립 Print 대조

Issue: #7642

runtime source: `c8d5dccb20f3dbd205824a42ae43a9d436e081d2`. 측정기는 `78f2f6bb` 계열의 raw 2px 실루엣이며 색상 경계 보정값은 판정에 사용하지 않았다. PDFium 153.0.7999.0, pypdfium2 5.13.0, 96dpi, Print profile.

HWP는 compat 2022, HWPX는 compat 2024. 두 원본과 독립 한컴 Print PDF는 각각 46쪽이며, 같은 입력의 46쪽 전체 TSV를 보존한다. 41쪽 본문 꼬리의 쪽 소속·누락·중복·줄 순서·본문 포함과 마지막 기준선을 검사한다. 시험지 본문 보정의 공통 원점 변경에 대한 대조군이다.

| 입력 | Native 전쪽 최저 | WASM 전쪽 최저 | Native 41쪽 | WASM 41쪽 |
|---|---:|---:|---:|---:|
| HWP |90.17559|90.10782|99.51599|99.53458|
| HWPX |90.16447|90.11795|99.51207|99.54438|

대표 review와 standalone overlay를 같은 하위 폴더에 보존한다. 직접 판독에서 네 줄의 순서와 꼬리 구절의 41쪽 소유를 확인했다. 글리프와 머리말의 작은 잔차는 남으며 엄격 RGB 일치를 주장하지 않는다.

- native 입력 `/home/tsjang/rhwp/samples/[2027] 온새미로 1 본교재.hwp` SHA-256 `e8592e74c9a8425c4ee2c5824d012ebe45e9f6dd36880b784ba594b4fd0a31ce`; 기준 `/home/tsjang/rhwp/pdf/[2027] 온새미로 1 본교재-hwp-2020.pdf` SHA-256 `450c793f33a134198936504b38d3b21c01a343a11c09c13e5a7a00e39e2d58c7`.
- native 입력 `/home/tsjang/rhwp/samples/[2027] 온새미로 1 본교재.hwpx` SHA-256 `85071b419b3467466ec391ecde187ad25723ce564178c2ed37c53b09e731138c`; 기준 `/home/tsjang/rhwp/pdf/[2027] 온새미로 1 본교재-hwpx-2024.pdf` SHA-256 `051aa8c4b94ca7cddc21e18d7fdc45b370e492b65c711c801d2c95479117cbab`.
- wasm 입력 `/home/tsjang/rhwp/samples/[2027] 온새미로 1 본교재.hwp` SHA-256 `e8592e74c9a8425c4ee2c5824d012ebe45e9f6dd36880b784ba594b4fd0a31ce`; 기준 `/home/tsjang/rhwp/pdf/[2027] 온새미로 1 본교재-hwp-2020.pdf` SHA-256 `450c793f33a134198936504b38d3b21c01a343a11c09c13e5a7a00e39e2d58c7`.
- wasm 입력 `/home/tsjang/rhwp/samples/[2027] 온새미로 1 본교재.hwpx` SHA-256 `85071b419b3467466ec391ecde187ad25723ce564178c2ed37c53b09e731138c`; 기준 `/home/tsjang/rhwp/pdf/[2027] 온새미로 1 본교재-hwpx-2024.pdf` SHA-256 `051aa8c4b94ca7cddc21e18d7fdc45b370e492b65c711c801d2c95479117cbab`.

독립 PDF의 유일한 전체 구절 `듯하니 불쌍코 가련하다`의 글자 행렬을 읽었다. PDF 좌표 e=89.4000015258789, f=92.9200439453125, 용지 높이841pt에서 96dpi 왼쪽 위 좌표로 환산한 x=119.2000020345052, baseline=997.43994140625를 ±2px로 보호한다. 구현 좌표를 기대값으로 복사하지 않았다.

실행·해시·전쪽 범위의 상세 영수증은 ignored `output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage3489-targeted-pdfium-head-{native,wasm}/receipt.json`, 직접 판독은 `stage3507-onsaemiro-qualified-pairs/receipt.json`에 있다. 코드나 글꼴이 바뀌면 새 source에서 재검증한다.
