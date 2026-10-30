# 한글 3.0 sample10 전쪽 검증

원본 `samples/hwp3-sample10.hwp`의 SHA-256은 `d9ceb35d8abfb73e9afbe349bccb2a986cf552d66ae7f3f34dcfe4d9d385cb48`이다. 독립 한컴 2020 Print 기준은 `pdf/issue8371/hwp3-sample10-2020.pdf`, SHA-256 `310e3251e910ac8c7c6f34d0e50d85c4a0ecbbadd65e69fe4f5071370931a918`이다. 기존 수동 Print PDF를 재출력하거나 입력을 HWPX로 변환하지 않았다.

코드 `8f3d099de5b8569ae44405858d8210588ed3898e`의 Native 실행 파일 SHA-256은 `08f1728c75d3eabeebe1518ca6dda25e9fb39c832654817c9c6fab9705dd3686`, fresh WASM SHA-256은 `c00f4eb8881fb217338ebe2fe370e67fc4dbb3add8f008e3bd036a2e9a52a87e`다. 기존 연결 그림 `samples/oracle.gif`, `samples/rdb02.gif`, `samples/s1.jpg`를 공급했다. 각각 SHA-256은 `464e863dd2c1650fc6997b03a5d96c9413e61bbabaf7337c783db27203cc2761`, `bfadf4cdbbeeb5f3d8632cb54c8c3696977f405204b651f99a7f3f34c8f39532cf`, `77dea18ce7f8f93b0931e133dec222aec642eef0ed87b8f2031b94dcbea5c514`이다.

96 DPI PDFium 및 색상 경계 보정 전 2px 내용 실루엣을 사용했으며 독립 PDF·Native·fresh WASM은 모두 763쪽이다. Native 최저는 757쪽 99.41135%, fresh WASM 최저도 같은 쪽 99.39720%이고 90% 미만은 없다. 전쪽 원본 TSV는 [Native](sample10-native.tsv)와 [WASM](sample10-wasm.tsv)이다.

최저 쪽의 [Native review](sample10-native-p757-review.png)·[overlay](sample10-native-p757-overlay.png), [WASM review](sample10-wasm-p757-review.png)·[overlay](sample10-wasm-p757-overlay.png)를 함께 대조했다. 757쪽 흐름도의 기준 PDF에서는 일부 글자가 가려지지만 RHWP는 원본 글자를 온전히 표시한다. 288 DPI [확대 비교](rebase-55f40/sample10-p757-detail-review.png)·[overlay](rebase-55f40/sample10-p757-detail-overlay.png)와 [PDFium·Poppler 비교](rebase-55f40/sample10-p757-pdf-rasterizers-review.png)·[overlay](rebase-55f40/sample10-p757-pdf-rasterizers-overlay.png)에서 두 PDF 렌더러가 같은 가림을 보였다.

PDF 757쪽의 그리기 순서를 독립적으로 읽으면 `fail` 텍스트 뒤에 불투명 RGB 이미지가 해당 문자 영역과 겹쳐 그려진다. 예를 들어 텍스트의 PDF 쪽 좌표는 `(251.14, 311.00)–(269.19, 319.99)`, 바로 뒤 이미지의 영역은 `(251.98, 309.50)–(275.85, 321.01)`이다. 이 이미지에는 별도 alpha mask가 없다. 이는 기준 PDF의 가림을 설명하는 관측이며, 한컴이 해당 이미지를 생성한 원인이나 편집 화면의 표시까지 단정하지 않는다. 원본의 문자를 의도적으로 숨기는 보정은 추가하지 않았다.
