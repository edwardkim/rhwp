# PR #7406 원본 물리 1쪽 글꼴 차이 증거

- 입력: `samples/issue2006/1790387_prep_final_report.hwpx`.
- 기준: `pdf/issue2006/1790387_prep_final_report-2024.pdf`, SHA-256 `04b95a6e41420fb45934ce2ee5abd8cf6dac4ce12fd47977dacbe7fca28018a8`.
- 범위: **물리 1쪽만**. 2px 관용 내용 실루엣 88.10703%. 다른 쪽의 배치 차이나 페이지 수 차이에 적용하지 않는다.

## 양쪽에 실제 적용된 글꼴

`pdffonts -f 1 -l 1`에서 기준 PDF의 `INPILL+H2hdrM`과 `INPILL+Dotum`은 모두 내장 subset이다. `mutool draw -F stext.json ... 1`의 본문 제목 `HIV 노출 전 예방요법(PrEP) 수요자 타당도`와 영문 제목·기관명은 `INPILL+H2hdrM`, 작은 상단 상자는 `INPILL+Dotum`으로 확인된다.

rhwp의 1쪽 SVG는 제목에 `HY헤드라인M`을 요청하지만, 현재 Mac의 글꼴 목록에는 이 face와 `돋움`이 없다. 이 SVG를 Visual Sweep과 동일한 `rasterize-svg-webfonts.mjs` 방식으로 Chrome에서 연 뒤 DevTools `CSS.getPlatformFontsForNode`로 제목의 첫 35글자를 확인했다. 각 글자의 **실제 사용 글꼴은 `HCR Dotum`** (`isCustomFont=true`)이었다. 원본 PDF의 `H2hdrM`과 다른 글꼴이다. 확인 출력은 Git 제외 `output/pr-review/planet6897-7406-20260925/logs/p1-chrome-fonts.json`에 보존한다.

## 배치 대조

동일 물리 1쪽의 [review](native_review_001.png)와 [overlay](native_overlay_001.png)를 직접 판독했다. 상단 상자 외곽, 두 줄의 국문 제목, 두 줄의 영문 제목, 기관명과 하단 기관명은 같은 페이지 영역에 있고 글줄 수·순서·내용 누락이 같다. 큰 잔여 차이는 제목과 본문의 글자 형태·굵기다. 표·그림 경계가 달라지는 사례는 이 쪽에 없다. 따라서 이 증거는 1쪽의 글꼴 차이 예외에만 사용하며 다른 페이지의 시각 통과를 대신하지 않는다.

- review PNG SHA-256: `44e19eb9350186c75590d24aaeef84074689c6be1f0aab66b738a9a91381ab57`
- overlay PNG SHA-256: `e05d36a975f82e32c867a7a39f9243a9b513ac3fee5afc99f785c518bfe0f572`
