# 한글 3.0 sample10 전쪽 검증

원본 `samples/hwp3-sample10.hwp`의 SHA-256은 `d9ceb35d8abfb73e9afbe349bccb2a986cf552d66ae7f3f34dcfe4d9d385cb48`이다. 독립 한컴 2020 Print 기준은 `pdf/issue8371/hwp3-sample10-2020.pdf`, SHA-256 `310e3251e910ac8c7c6f34d0e50d85c4a0ecbbadd65e69fe4f5071370931a918`이다. 기존 수동 Print PDF를 재출력하거나 입력을 HWPX로 변환하지 않았다.

코드 `8f3d099de5b8569ae44405858d8210588ed3898e`의 Native 실행 파일 SHA-256은 `08f1728c75d3eabeebe1518ca6dda25e9fb39c832654817c9c6fab9705dd3686`, fresh WASM SHA-256은 `c00f4eb8881fb217338ebe2fe370e67fc4dbb3add8f008e3bd036a2e9a52a87e`다. 기존 연결 그림 `samples/oracle.gif`, `samples/rdb02.gif`, `samples/s1.jpg`를 공급했다. 각각 SHA-256은 `464e863dd2c1650fc6997b03a5d96c9413e61bbabaf7337c783db27203cc2761`, `bfadf4cdbbeeb5f3d8632cb54c8c3696977f405204b651f99a7f3f34c8f39532cf`, `77dea18ce7f8f93b0931e133dec222aec642eef0ed87b8f2031b94dcbea5c514`이다.

96 DPI PDFium 및 색상 경계 보정 전 2px 내용 실루엣을 사용했으며 독립 PDF·Native·fresh WASM은 모두 763쪽이다. Native 최저는 757쪽 99.41135%, fresh WASM 최저도 같은 쪽 99.39720%이고 90% 미만은 없다. 전쪽 원본 TSV는 [Native](sample10-native.tsv)와 [WASM](sample10-wasm.tsv)이다.

최저 쪽의 [Native review](sample10-native-p757-review.png)·[overlay](sample10-native-p757-overlay.png), [WASM review](sample10-wasm-p757-review.png)·[overlay](sample10-wasm-p757-overlay.png)를 함께 대조했다. 가운데 흐름도의 선과 글자 농도에는 기준 PDF와 차이가 남는다. 별도 기재한 실물 차이를 높은 점수로 면제하지 않는다.
