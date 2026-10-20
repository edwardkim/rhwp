# 저장 줄이 없는 그림의 관계 회귀 입력

원본 `samples/p122.hwp`를 보존한 별도 대조군이다. 그림 크기·순서·문자·정렬·여백을
설정한 진단 모델을 한컴 2020 MCP로 정상 저장한 뒤 LineSeg만 제거했다. 상대 크기 두 입력은
기존 정상 저장본을 사용하고 후속 문단을 같은 쪽에 놓았다. 같은 NO_LS 입력을 다시 MCP의
한컴 PrintMethod 0 / one-up으로 출력했다. 재저장 대조군의 통과를 원본 일치로 보고하지 않는다.

동일 최종 source의 Native/fresh WASM에서 아래 13개 입력 26쪽은 쪽수가 같고 최저
99.99937%다. 저장 대조군·영어·국어 57쪽도 두 경로에서 90% 이상이다. CENTER의 사이 글자와
106% 표시 크기의 줄 상자는 확대 판독으로 보정했다. 점수만으로 미세 정렬 차이를 면제하지 않았다.

관계 검사는 소유 글줄, 그림 순서·겹침, 후속 본문 보존, CENTER의 공통 정렬점, 줄 간격 속성,
선언 글자 크기와 그림 너비의 비율을 검사한다. 절대 px 위치·영역·SVG 해시는 고정하지 않는다.
간격의 독립 기대값은 상위 디렉터리의 한컴 정상 저장본 LineSeg에서, 선언 크기는 입력 속성에서
읽는다. 출력 JSON의 반올림은 무차원 비율로만 허용한다.

실행 로그·전쪽 TSV·source/binary/WASM 해시·생성 진단은 ignored
`output/pr-review/issue7642/failed-inputs-20261008/`의 `picture-logical-font-*`에 보존한다.
[단계 기록](../../../../mydocs/working/task_m100_7642_stage4.md)에서 실제 소비 경로와 before/after를 확인한다.

| 파일 stem (HWP와 -print-2020.pdf) | Save job | Print job | HWP SHA-256 | PDF SHA-256 |
| --- | --- | --- | --- | --- |
| normalized-quarter-reflow-false | 기존 정상 저장 | 24e59242-d2c0-4ec3-bc7d-21d118d8793e | `8647f8ca68776fd1a2e640e69a5a8e428c6cb287ba17b2cb883fcce5efae20a8` | `0ea824cccc05c0dbab4edbddbf5df407481cb554e85d6e909d21d59702b5d7c9` |
| normalized-quarter-reflow-true | 기존 정상 저장 | 55405e59-d0d3-4c21-9157-0afa7b0929cf | `a7188cf5dd388df0c930224178a3cba44715cd07ccdccde5c8ebcde7cbfa0c3f` | `ee84d81a4b3889da9658b5ee0936c8f726f30897ea9a63c9564408a2c427e6b4` |
| picture-flow-same-row-no-ls | b5d3bb59-f497-49ac-b446-6914bbf8ef22 | 360bdd73-11ad-4535-8e64-75e38a4dba0f | `aeee538f8d361f81bb7b356ab662c453f13f67a4e2be4c2af911eabd16ea1d12` | `1ad87bdcb0265492849d495f7a539c37aec00b42e61940023ade8b88c164dad9` |
| picture-flow-wrapped-row-no-ls | b3738630-817f-434e-8810-a4264302fe07 | e11013aa-14cf-4e60-90e7-991ab00eed56 | `52af34ce881875d302b604010f4a67ca51b2f85386a40e4f68e4714836064f1c` | `880e9aef5665b4a46c0418009a246f8a095bed2151ceaaeddfce1eb960ac39cb` |
| picture-flow-center-row-no-ls | ac6e519a-f1a3-4e52-b54e-9d7f17567255 | 1f88b210-ad78-4163-862a-db618c351146 | `48f55c0d01011d3580f2f1fdef2d565077ffa1e66c7aaddfca10761dd43dd297` | `0e8536e76112d9439cb9c50418ad5918ba88310ab0630c2745b64a677dfd159f` |
| picture-flow-authored-break-no-ls | 65dcb9a0-c27a-42e2-b30c-de3f700629e3 | 330f0b10-05f8-4e56-b2f2-8803668b54fc | `c7d856b48d19cabb51e50b0befdf2fbc67a903a633038b7084dc6c2113177b93` | `9b8037d88922205acf7459e6e4a44a8535894923c3e75aed06b672bb0bb6ed1c` |
| picture-flow-column-transition-no-ls | d7e2ef9f-9160-4acf-b658-aa2cc2174265 | 0501b7ed-17da-4460-864f-668232fde4ac | `29745472fdb513c8d4d1a7a6f1a22a56580acd9b06b6620ed7bed7d0b3000c32` | `d9dcf28fe3b5ee63ff05880f44e2d5eda3200c40f008981aa2d702acd1059e23` |
| picture-flow-mixed-center-row-no-ls | e88c34de-732a-45ed-bef1-fa5ac3344f98 | 91cf57b5-33df-436c-a70c-c0afeb8da31c | `8db7c5325b0e1676c29190d55110465453203be310de435492a29d6522835ad6` | `a2a1178aea825b6e2e734a43b18b35cd83f53d650ef28491a9ba03ddf146ea4f` |
| picture-flow-mixed-end-picture-no-ls | 2110abd2-55ee-43dd-b805-bbeeca03675d | 97d0379a-2739-46e9-babf-292f63ca996e | `8478f28ec6bfbeb85f4d5ed3947b3063380549ec956a5fbe7fc5559aabd0dd7e` | `1a37e74b889946e8ceeff9fd3cf7200d356fe2af6401c93634879a9208f3bb3e` |
| picture-flow-outer-margin-row-no-ls | 4f7b8d1c-99a4-4287-96c5-2f6a081879db | 5db140d9-bf1c-4739-96d4-474472504802 | `16a916b8c79e055f26d2033f7104c2cb39487bf21993356b805ad8616ac7cebb` | `6fab8b80a40cf81355adc11a923b777f7515977a25a7963b3296a00ba6118f50` |
| picture-flow-tail-gap-boundary-no-ls | 7a497475-10fc-4a61-956b-22920a04b049 | b2b68362-3ead-4f6d-8ccc-905517729a25 | `07ebb414f77d99c4b1a609e31f79904e95cedf4fe21f315cf6760007a8aa3c38` | `10eab539ec29e8efd87d993e820fcc5335974aad5075042709fe733f2b922b96` |
| picture-flow-tail-gap-only-fit-no-ls | dbb07112-2611-4977-9843-43bba68f1330 | 4735ae4e-dbc2-43cb-9ef4-7130463a38fd | `4faca24711f2dcd84dc22b3047dfb56911d70c952789978071a8b898facc2747` | `71728ac7e179fd1f850a348d951b5fd1c227ee4a8c74219a26646b5bdfa1d155` |
| picture-flow-relative-font-row-no-ls | 382a2d57-2211-41be-8a80-da4278d8e9ca | 1e865bf3-388b-4b51-b834-b0910c972b98 | `6e0073478832e25398466ad001c1566b626ca0c49634184135857d88ef0f8870` | `109ed14cbc38ee7e9604e64066d84261bd34806077d9f5ebee1510c1936834be` |
