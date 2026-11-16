# #8199 그림을 넣은 여권발급신청서 저장본과 한/글 인쇄 기준

## 입력

- 원본: `samples/issue2020/passport_application_lawgo.hwp` (국가법령정보센터 공개 빈 서식, flSeq=164628771 과 바이트 동일, SHA-256 `ec8bb9495d78bce5de338dae7a4b68a35a96e66fa3a7f324c8318ee8626cac8f`)
- 편집: [#8199](https://github.com/edwardkim/rhwp/issues/8199) 제보의 재현 순서를 그대로 실행한 [`generate.mjs`](generate.mjs)
  - 구역 0 문단 0, 글자 위치 0 에 1×1 PNG 를 `insertPicture`(2835×2835 HU) → `setPictureProperties({textWrap:"InFrontOfText"})` → `exportHwp()`
  - 실행: `node generate.mjs <pkg> samples/issue2020/passport_application_lawgo.hwp passport-picture-at-start.hwp`
  - 사용한 fresh WASM: 코드 `e2eb9fdef` 로 `scripts/wasm-pack-locked.sh --target web` 빌드, `rhwp_bg.wasm` SHA-256 `c4e391dc0937238c93340fde08b1a7c6033ee80502b484c7173abd308d022ef0`
  - 실행 결과: `{"insert":{"ok":true,"paraIdx":0,"controlIdx":3,"logicalOffset":1}, "pagesBefore":2, "pagesAfterEdit":2, "pagesReopened":2}`

## 저장본

- 파일: `passport-picture-at-start.hwp` (89,088 바이트, SHA-256 `6be52359642a60194f9f3eda0ae40a4e2d8084bd55979fb12d6d8ece61aa8fb9`)
- 구역 0 문단 0: `char_count` 49, 조절 문자 6개(`secd·cold·사각형·그림·표·표`), 줄 정보 `ls[0] ts=0`, `ls[1] ts=40`(둘째 표의 스트림 위치)
- 수정 전 코드는 같은 편집에서 `ls[1] ts=32` 를 그대로 남긴다(제보 첨부 `F05_pic_start.hwp`).

## 기준 PDF

- 파일: `passport-picture-at-start-hwp2022-print.pdf` (158,700 바이트, SHA-256 `afd03ca1bbdeab436c539994d069aa193741877f5e08f07b2af60adef59d33f7`)
- 위 저장본을 한/글 2022(`12.0.0.535`, Windows)에서 열어 **인쇄 → Microsoft Print to PDF** 로 출력했다(`Producer: Microsoft: Print To PDF`, A4, **2쪽**). 원본(2쪽)과 쪽수가 같다. 빌드 번호는 출력한 PC 의 한/글 정보에서 확인했다(PDF 메타데이터에는 없음).
- 수정 전 저장본(제보 첨부)은 한/글 2022 에서 3쪽이다(제보 첨부 `hancom_F05_pic_start.pdf`, `Creator: Hwp 2022 12.0.0.893`, 「PDF로 저장」 경로).

## 대표 이미지 (1쪽, 위 기준 PDF 대비)

- `passport-picture-at-start-p001-native-review.png`, `…-native-overlay.png` — Native(native-skia) `export-png --profile print`, 실루엣 일치율 99.99%
- `passport-picture-at-start-p001-wasm-review.png`, `…-wasm-overlay.png` — fresh WASM canvas2d `print`, 99.91%
