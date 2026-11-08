# #8594 연결 슬롯 보존과 외부 그림 글머리표의 출력 경계

## 확인된 회귀와 공통 보정

리베이스로 들어온 #8683은 #8594의 Link 슬롯 보존도 반영했다. 내용 없는 슬롯이 lookup에서 사라지지
않게 된 것은 정상이나, 그림 글머리표는 빈 바이트를 실제 Image로 만들어 placeholder를 출력했다.
현재 devel `08963f6825`/이전 후보 `429eb75b89`의 1쪽 Native raw는99.39107%로 이전 후보99.67721%보다0.28614%p 낮았다.
독립 한컴 Print에는 Golf 문단의 그림·문자 마커가 모두 없다. 문자 ●를 되살리는 초기 진단 기대값도
독립 기준과 달라 기각했으며, 정상 기대값으로 등록하지 않았다.

공통 그림 참조 판별을 공유하고 빈 바이트를 Image로 만들지 않는다. 참조가 있지만 원본 그림이 없으면
문자 마커로 바꾸지 않으며, 본문 들여쓰기·다른 문자 마커·정상 내장 그림의 기존 경로는 유지한다.
BinData 배열이나 Link 슬롯 보존·저장을 되돌리지 않는다. 문서·문단 번호별 분기는 없다.

## 입력과 실제 검증

- 입력: `tests/fixtures/semanticist21_next10_review/paragraph-head-edited.hwp`, SHA-256 `2728d6263b06ffbc0e714d7d1e120cc8b09605e9c7660d45c0b20cc51b59ae0b`.
- 독립 Print: `pdf/semanticist21-20261105-next10/paragraph-head-edited-hwp-2020.pdf`, SHA-256 `73a8218740c074d520475a5430c0b657f16b452b0a14c6fb340971aa519eaede`.
- Print·compat2022·96DPI·PDFium·2px raw, 전1쪽 Native **100.00000%**, fresh WASM **100.00000%**.
- Mac의 기존 최소 글꼴 sandbox를 전체 출력 프로세스에 적용했다. 초기 드라이버의 격리 누락 실행은
  제외했고, Chrome의 자체 확인이 차단한 뒤 같은 빌드로 올바른 격리 실행을 새로 수행했다.
- `issue_8594_link_picture_bullet_fallback`은 관계형이다. d0341bbf88에서 실제1 FAIL → 후보1 PASS이며
  빈 Image·문자 fallback 부재와 원본 Link/그림 참조, 본문 들여쓰기 소유를 확인한다.
- 보정 후 #7728 기존·새 관계/고정형12 PASS. 같은 후보로 N/W 각각 전18쪽을 새로 출력해 최저
  **98.83905% / 98.83565%**, 누락0을 재확인했다. 두 backend 각각의18개 PNG는
  [직전 최종 보정](../../issue7728/phase10-all-pages-98-20261106/README.md)과 hash가 모두 같다.
- Native review와 WASM overlay를 직접 열어 Golf 마커 부재, 정상 앞뒤 마커·본문 줄 소속을 확인했다.
  100%는2px raw 지표이며 독립 PDF와 모든 색상 픽셀이 동일하다는 뜻은 아니다.
- #8594의 정상 그림·HWP/HWPX 저장 검사와 기존 문자 fallback 검사의 최종 suite, 전체 nextest·Skia·lint,
  다른 원본 전쪽·실제 Studio 검증은 다음 최종 게이트에서 계속한다. PR 준비 완료 기록이 아니다.

## 빌드 출처

- source: parent `d0341bbf88` + 두 공통 그림 글머리표 source diff, patch SHA-256 `da3e04e61d7ede6c4ff2acc7a955d492634d94ec76f4d8b6059e6a25e6ec9c7a`.
- Native SHA-256 `2c19b97491eb8cdd052d20b4038efd02346b1ecc3c99e403cf4d1b340d098c3b`.
- WASM SHA-256 `40680cf9cdfd60b13c8e9ad960f213b3d61a3eab8c43e73ca8e1108da415025c`.
- JS SHA-256 `370ddff0fd1a1cf6f889edb1b2a42f420fac62f7f57df4b9cff19829e60cdd5f`.
- Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Root pkg·Studio public·frozen package의
  JS/WASM hash 일치를 확인했다. Docker 최적화 또는 실제 Studio 초기화 응답 검증으로 대신 보고하지 않는다.
- 전체 로그·진단·초기 부적합 실행은 ignored output에만 보존한다.

## 제출 자료 SHA-256

- `native-p001-overlay.png`: `4d1f332141f1046340c54dcbab37b98fa27083091eeedf265e1bcdb84c82acb6`.
- `native-p001-raw.png`: `21c3092871758b8b0f523d5e77db4bc6f2eb7664f2f094a030091aa568a9dc94`.
- `native-p001-review.png`: `9ea42b80c2dc28a4e20dd98f9171d5928034acca19572ddf0831605419755097`.
- `native-silhouette.tsv`: `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`.
- `wasm-p001-overlay.png`: `83c4012144e4146cc7a6c352ccd68844d83a873a95cc5f334f97eb10fccbab53`.
- `wasm-p001-raw.png`: `6be705be76c346f95b4476a8f372457a4b933085d5c60c2b4732cdd7448a8058`.
- `wasm-p001-review.png`: `f928ab8059bde3403d3e9b6f008cb84e8df8974baebf4b616b05838c4d4f4ab8`.
- `wasm-silhouette.tsv`: `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`.
