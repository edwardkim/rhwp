# 대표 페이지와 직접 판독

점수와 직접 판독은 별도다. 아래는 실제 원 주장·완료된 고정 페이지에 대응하는 최종 패널이다.
일부 Native 판독은 현재 RHWP/PDF PNG 쌍의 SHA가 이전 판독 쌍과 각각 같을 때만 재사용했다.
새 product의 PNG·TSV를 과거 수치로 재표기하지 않았다.
PR #8564의 동적 파일명 재평가 차이는 runtime-checks.md에 명시한다.
원본·PDF SHA는 measurements.md의 같은 key를 따른다.

### PR #8526 · native · api-8526-inserted-style-hwpx · p1

스타일 삽입 후 동일 본문 보존. 삽입 ID와 참조 보존의 직접 증거는 실제 API 기록이다.

같은 본문의 글자와 시작 위치를 대조했다. 스타일 삽입 ID·참조 무변경은 그림만으로 입증하지 않으며 별도 실제 API 검사를 연결한다. raw 100.0%.

- [review](png/pr8526/native/api-8526-inserted-style-hwpx/review_001.png), SHA-256 `f15002a84c7e2637595cf5d8da9ee37962a6334cf79b9918451112c56071a79d`.
- [overlay](png/pr8526/native/api-8526-inserted-style-hwpx/overlay_001.png), SHA-256 `cd758a0187520fc49a2451844a663835c3e94245e874e7222592cb5d435bee07`.

### PR #8526 · wasm · api-8526-inserted-style-hwpx · p1

스타일 삽입 후 동일 본문 보존. 삽입 ID와 참조 보존의 직접 증거는 실제 API 기록이다.

기존 스타일 본문의 글자와 원점을 대조했다. 스타일 참조 보존은 별도의 실제 API 결과로 검증하며 빈 여백을 기능 증거로 대신하지 않는다. raw 100.0%.

- [review](png/pr8526/wasm/api-8526-inserted-style-hwpx/review_001.png), SHA-256 `a01e608a691cf4b656f631657e50d414bf72c140511d10d86624a599bea627c8`.
- [overlay](png/pr8526/wasm/api-8526-inserted-style-hwpx/overlay_001.png), SHA-256 `4cebe3032391d49645f94899a81361a86b50dc95a2c261f5646cda49aa6f2330`.

### PR #8527 · native · 8527-levels-hwp-2022 · p1

원 PR의 8~10수준 문단 번호 표시

빈 번호 머리를 가진 개요9·10 본문 시작이 독립 Print와 대응한다. 상위 가시 마커7개와 아래 번호1/8/9/10 문단의 순서·간격이 보존된다. raw 100.0%.

- [review](png/pr8527/native/8527-levels-hwp-2022/review_001.png), SHA-256 `96ed7723791263f6c7024fc6c7728362ad28d2ddaa0af73b5045896e13ff2a75`.
- [overlay](png/pr8527/native/8527-levels-hwp-2022/overlay_001.png), SHA-256 `70658c662212e8eba77da5244f0a62770caa4400918753b4d78f6b0c1621884e`.

### PR #8527 · wasm · 8527-levels-hwp-2022 · p1

원 PR의 8~10수준 문단 번호 표시

빈 번호 머리를 가진 개요9·10 본문 시작이 독립 Print와 대응한다. 상위 가시 마커7개와 아래 번호1/8/9/10 문단의 순서·간격이 보존된다. raw 100.0%.

- [review](png/pr8527/wasm/8527-levels-hwp-2022/review_001.png), SHA-256 `7276f69f6ad5414634ee06bc178fa949c5eb70e6bda400ebc5c2a43735d1bb4e`.
- [overlay](png/pr8527/wasm/8527-levels-hwp-2022/overlay_001.png), SHA-256 `90f8eab20f8a3e05d958efb0bea30063e6e1bf5ec16c3e78be0d596ccf5384ba`.

### PR #8528 · native · 8528-follow-two-sizes-hwp-2022 · p1

같은 번호 모양에서 본문 끝 글자 크기와 번호 크기의 추종 관계

20pt·10pt 본문 두 행에서 이탤릭 번호의 크기가 끝 글자 크기를 따르며 본문과의 간격이 독립 Print와 맞는다. raw 100.0%.

- [review](png/pr8528/native/8528-follow-two-sizes-hwp-2022/review_001.png), SHA-256 `a1d7f795a08f059ba86930efe70e4e10800eb330867040ee05cb7c17e8bc912e`.
- [overlay](png/pr8528/native/8528-follow-two-sizes-hwp-2022/overlay_001.png), SHA-256 `8ae7d153f2109f856b8216cc278bb41f1f859c8f065630c1b55fec24aee21663`.

### PR #8528 · wasm · 8528-follow-two-sizes-hwp-2022 · p1

같은 번호 모양에서 본문 끝 글자 크기와 번호 크기의 추종 관계

큰 본문과 작은 본문의 기울임 번호 크기 추종·번호 뒤 간격과 본문 기준선을 대조했다. raw 100.0%.

- [review](png/pr8528/wasm/8528-follow-two-sizes-hwp-2022/review_001.png), SHA-256 `ee9683aa17f4a564977a0ae65f99ba6040fd2e5987bc166df29386139a83c557`.
- [overlay](png/pr8528/wasm/8528-follow-two-sizes-hwp-2022/overlay_001.png), SHA-256 `fbddd2a177cfc71e81bdfea69ff2e6ce0458371c81cdb3c7ae7c17e9211a7110`.

### PR #8536 · native · 8536-note-formats-hwp-2022 · p7

원문자 각주·쪽 번호와 기호 번호의 실제 표시

7쪽 원문자 알파벳 각주 A/B/C와 하단 G쪽번호, 18쪽 기호 */†/‡와 하단 기호번호를 대조했다. 위첨자·각주 순서·본문 내용·하단 쪽번호 소속이 보존된다. 18쪽 구분선의 약1px 세로 획차이를 raw100의 완전픽셀동일로 해석하지 않는다. raw 100.0%.

- [review](png/pr8536/native/8536-note-formats-hwp-2022/review_007.png), SHA-256 `26a420afdbbaf86c284b59054b708eeda9cd1ccca702090fe2fc272937e1fc5c`.
- [overlay](png/pr8536/native/8536-note-formats-hwp-2022/overlay_007.png), SHA-256 `c69cf0685b120bc93b4bd9a73fcd51cc50627969e454e87e7c34f9638aa8e32f`.

### PR #8536 · native · 8536-note-formats-hwp-2022 · p18

원문자 각주·쪽 번호와 기호 번호의 실제 표시

7쪽 원문자 알파벳 각주 A/B/C와 하단 G쪽번호, 18쪽 기호 */†/‡와 하단 기호번호를 대조했다. 위첨자·각주 순서·본문 내용·하단 쪽번호 소속이 보존된다. 18쪽 구분선의 약1px 세로 획차이를 raw100의 완전픽셀동일로 해석하지 않는다. raw 100.0%.

- [review](png/pr8536/native/8536-note-formats-hwp-2022/review_018.png), SHA-256 `ba6926682d791c8b6c7ee24b0dec1a21c330c7ec84f333ed3e77638e8f0ad351`.
- [overlay](png/pr8536/native/8536-note-formats-hwp-2022/overlay_018.png), SHA-256 `310740b54ebf7f52e9a3dc759ac4b0f6cb638b29011dd18455b421c113598c39`.

### PR #8536 · wasm · 8536-note-formats-hwp-2022 · p7

원문자 각주·쪽 번호와 기호 번호의 실제 표시

원문자·로마자·기호 번호와 하단 쪽 번호가 보존된다. 파선·점선·원형·물결·입체·실선의 구분을 양쪽에서 판독했다. 일부 가는 선에 약 1픽셀 래스터 경계 차이가 보이므로 raw100을 엄격한 픽셀 동일성으로 해석하지 않는다. raw 100.0%.

- [review](png/pr8536/wasm/8536-note-formats-hwp-2022/review_007.png), SHA-256 `488370c9e7499dd7f3f0d381d2f74f32895b8d7fa6161adafb98792fa47bbf3e`.
- [overlay](png/pr8536/wasm/8536-note-formats-hwp-2022/overlay_007.png), SHA-256 `d71d711041a620c76c8c90a924f0869937669098b2b9803c386cb6f439916573`.

### PR #8536 · wasm · 8536-note-formats-hwp-2022 · p18

원문자 각주·쪽 번호와 기호 번호의 실제 표시

원문자·로마자·기호 번호와 하단 쪽 번호가 보존된다. 파선·점선·원형·물결·입체·실선의 구분을 양쪽에서 판독했다. 일부 가는 선에 약 1픽셀 래스터 경계 차이가 보이므로 raw100을 엄격한 픽셀 동일성으로 해석하지 않는다. raw 100.0%.

- [review](png/pr8536/wasm/8536-note-formats-hwp-2022/review_018.png), SHA-256 `d72a5921debdbda87f0ee7f27ed4ee782cb130c5f44cef992aa96856f64fb3d9`.
- [overlay](png/pr8536/wasm/8536-note-formats-hwp-2022/overlay_018.png), SHA-256 `d36ea10d0b93164b61a411ac7e237cd39469030a9989090abfe62a226dc4c4bb`.

### PR #8537 · native · 8536-note-formats-hwp-2022 · p2

원 PR과 동일 SHA인 문서의 파선·점선·물결선·입체선 구분선

p2 파선·p3 점선·p12 물결선은 본문·각주 소속과 함께 대조했다. p14는 THICK3D라는 저장 속성을 가지지만 이 독립 Print와 RHWP 모두 구분선이 보이지 않는다. 이를 입체선의 눈에 보이는 렌더링 구현 증거로 승격하지 않는다. 코드·이름 보존은 별도 HWP/HWPX 교차 저장 assertion으로 검증한다. 선의 1px 위상 차이가 raw100에서 남을 수 있음을 구분한다. raw 100.0%.

- [review](png/pr8537/native/8536-note-formats-hwp-2022/review_002.png), SHA-256 `ad0f28f9c4f559c5fbf299e6b653b7859dc3221b84fa9ce53e4eadd82161cebe`.
- [overlay](png/pr8537/native/8536-note-formats-hwp-2022/overlay_002.png), SHA-256 `e11754c22f15c8095d419df7d1bb59a49b050c7b17ccdd5268cf3e95e7ef7cc2`.

### PR #8537 · native · 8536-note-formats-hwp-2022 · p3

원 PR과 동일 SHA인 문서의 파선·점선·물결선·입체선 구분선

p2 파선·p3 점선·p12 물결선은 본문·각주 소속과 함께 대조했다. p14는 THICK3D라는 저장 속성을 가지지만 이 독립 Print와 RHWP 모두 구분선이 보이지 않는다. 이를 입체선의 눈에 보이는 렌더링 구현 증거로 승격하지 않는다. 코드·이름 보존은 별도 HWP/HWPX 교차 저장 assertion으로 검증한다. 선의 1px 위상 차이가 raw100에서 남을 수 있음을 구분한다. raw 100.0%.

- [review](png/pr8537/native/8536-note-formats-hwp-2022/review_003.png), SHA-256 `5b690a89e7cda7feb638eb059fc94680579a9a8a462a0b3a4ab5c299975ee9f5`.
- [overlay](png/pr8537/native/8536-note-formats-hwp-2022/overlay_003.png), SHA-256 `44fb855b3270e8f778f0699d96c5dc39d04dd8adbfc614715541d02e5783c020`.

### PR #8537 · native · 8536-note-formats-hwp-2022 · p12

원 PR과 동일 SHA인 문서의 파선·점선·물결선·입체선 구분선

p2 파선·p3 점선·p12 물결선은 본문·각주 소속과 함께 대조했다. p14는 THICK3D라는 저장 속성을 가지지만 이 독립 Print와 RHWP 모두 구분선이 보이지 않는다. 이를 입체선의 눈에 보이는 렌더링 구현 증거로 승격하지 않는다. 코드·이름 보존은 별도 HWP/HWPX 교차 저장 assertion으로 검증한다. 선의 1px 위상 차이가 raw100에서 남을 수 있음을 구분한다. raw 99.98307%.

- [review](png/pr8537/native/8536-note-formats-hwp-2022/review_012.png), SHA-256 `8efe420f0af214653aa665140e0808529e294ff3f0b45bf4d8a51570e099c51a`.
- [overlay](png/pr8537/native/8536-note-formats-hwp-2022/overlay_012.png), SHA-256 `34d95c02c085711d6e87f44b44a7bf2d57e7eaf80bfc7a92daae14883fd76e9e`.

### PR #8537 · native · 8536-note-formats-hwp-2022 · p14

원 PR과 동일 SHA인 문서의 파선·점선·물결선·입체선 구분선

p2 파선·p3 점선·p12 물결선은 본문·각주 소속과 함께 대조했다. p14는 THICK3D라는 저장 속성을 가지지만 이 독립 Print와 RHWP 모두 구분선이 보이지 않는다. 이를 입체선의 눈에 보이는 렌더링 구현 증거로 승격하지 않는다. 코드·이름 보존은 별도 HWP/HWPX 교차 저장 assertion으로 검증한다. 선의 1px 위상 차이가 raw100에서 남을 수 있음을 구분한다. raw 100.0%.

- [review](png/pr8537/native/8536-note-formats-hwp-2022/review_014.png), SHA-256 `abd297da9155901c09805e15cd09c2186f3337081734a7d3443e31c7eda178a6`.
- [overlay](png/pr8537/native/8536-note-formats-hwp-2022/overlay_014.png), SHA-256 `64d2873f449f4b61aee53f3a8135198ba44e1b2de4682515d5b1a60d1c783146`.

### PR #8537 · wasm · 8536-note-formats-hwp-2022 · p2

원 PR과 동일 SHA인 문서의 파선·점선·물결선·입체선 구분선

원문자·로마자·기호 번호와 하단 쪽 번호가 보존된다. 파선·점선·원형·물결·입체·실선의 구분을 양쪽에서 판독했다. 일부 가는 선에 약 1픽셀 래스터 경계 차이가 보이므로 raw100을 엄격한 픽셀 동일성으로 해석하지 않는다. raw 100.0%.

- [review](png/pr8537/wasm/8536-note-formats-hwp-2022/review_002.png), SHA-256 `53e6b6d2ab47b0a3ae34c6ea379b5da455c67103cdfe608ce1978e237d98f49b`.
- [overlay](png/pr8537/wasm/8536-note-formats-hwp-2022/overlay_002.png), SHA-256 `44fb7b22cce648a8141ff086e2cd8916b931d021d788da06f4c93a295decd7d0`.

### PR #8537 · wasm · 8536-note-formats-hwp-2022 · p3

원 PR과 동일 SHA인 문서의 파선·점선·물결선·입체선 구분선

원문자·로마자·기호 번호와 하단 쪽 번호가 보존된다. 파선·점선·원형·물결·입체·실선의 구분을 양쪽에서 판독했다. 일부 가는 선에 약 1픽셀 래스터 경계 차이가 보이므로 raw100을 엄격한 픽셀 동일성으로 해석하지 않는다. raw 100.0%.

- [review](png/pr8537/wasm/8536-note-formats-hwp-2022/review_003.png), SHA-256 `59dc626868679aa32c3b8912c24f286706c739dd9aff6933a291d4f722b8aca6`.
- [overlay](png/pr8537/wasm/8536-note-formats-hwp-2022/overlay_003.png), SHA-256 `79e0530beae59460372da6c29ed1bfe7f9afd37299c735ee1b89b2361a254c38`.

### PR #8537 · wasm · 8536-note-formats-hwp-2022 · p12

원 PR과 동일 SHA인 문서의 파선·점선·물결선·입체선 구분선

원문자·로마자·기호 번호와 하단 쪽 번호가 보존된다. 파선·점선·원형·물결·입체·실선의 구분을 양쪽에서 판독했다. 일부 가는 선에 약 1픽셀 래스터 경계 차이가 보이므로 raw100을 엄격한 픽셀 동일성으로 해석하지 않는다. raw 99.98279%.

- [review](png/pr8537/wasm/8536-note-formats-hwp-2022/review_012.png), SHA-256 `5f3247bce9a8ec81203e24d7f06db1122cdef910ec96b47f616ca8215b94a08f`.
- [overlay](png/pr8537/wasm/8536-note-formats-hwp-2022/overlay_012.png), SHA-256 `7c26475fc118a967253362f8ddc04707efabbb3b02ede11884ed2951cb6afde8`.

### PR #8537 · wasm · 8536-note-formats-hwp-2022 · p14

원 PR과 동일 SHA인 문서의 파선·점선·물결선·입체선 구분선

원문자·로마자·기호 번호와 하단 쪽 번호가 보존된다. 파선·점선·원형·물결·입체·실선의 구분을 양쪽에서 판독했다. 일부 가는 선에 약 1픽셀 래스터 경계 차이가 보이므로 raw100을 엄격한 픽셀 동일성으로 해석하지 않는다. raw 100.0%.

- [review](png/pr8537/wasm/8536-note-formats-hwp-2022/review_014.png), SHA-256 `1084ba76b2d2519e4dce81b613981f6bb976f714e0ae02d94a287df113b22cbf`.
- [overlay](png/pr8537/wasm/8536-note-formats-hwp-2022/overlay_014.png), SHA-256 `616bf297e98fab5d53138adea599b6ad1d35ee83e8cccd57438c651bb4ae7084`.

### PR #8538 · native · 8538-shape-hwp-2022 · p4

본문 아래 각주 구분선과 작게 표시한 내용 번호

4쪽의 본문 바로 아래 구분선과 각주 두 줄, 5쪽 하단 작은 번호·각주 두 줄을 대조했다. 본문 개행·1) 2) 소속·선 시작·종료·각주 위치를 보존한다. raw 100%를 픽셀 바이트의 동일성으로 표현하지 않는다. 6쪽 review·overlay도 실제 열어 보통 크기 번호 1)·2), 각주 구분선·쪽 하단의 두 각주 행과 본문 네 행의 순서·소속을 Print와 대조했다. 이 기록은 Native 판독이며 fresh WASM 판독을 대신하지 않는다. raw 100.0%.

- [review](png/pr8538/native/8538-shape-hwp-2022/review_004.png), SHA-256 `156d611fd25a7c341d3671d9733b848d49706ca3b496e1a8ecb8f89703bb39d7`.
- [overlay](png/pr8538/native/8538-shape-hwp-2022/overlay_004.png), SHA-256 `026fc9813e56fa146c31ec67efb17d6bd52eec4cb46a0de09d2d8961ec139eb6`.

### PR #8538 · native · 8538-shape-hwp-2022 · p5

본문 아래 각주 구분선과 작게 표시한 내용 번호

4쪽의 본문 바로 아래 구분선과 각주 두 줄, 5쪽 하단 작은 번호·각주 두 줄을 대조했다. 본문 개행·1) 2) 소속·선 시작·종료·각주 위치를 보존한다. raw 100%를 픽셀 바이트의 동일성으로 표현하지 않는다. 6쪽 review·overlay도 실제 열어 보통 크기 번호 1)·2), 각주 구분선·쪽 하단의 두 각주 행과 본문 네 행의 순서·소속을 Print와 대조했다. 이 기록은 Native 판독이며 fresh WASM 판독을 대신하지 않는다. raw 100.0%.

- [review](png/pr8538/native/8538-shape-hwp-2022/review_005.png), SHA-256 `7bf16f39e76933967226770e1d3d9534b6bf452ec260b247d72dc4ff81edfdbe`.
- [overlay](png/pr8538/native/8538-shape-hwp-2022/overlay_005.png), SHA-256 `a0d0cbafca519b188f58def213efaee36342a397fc42bb5b9bfb088538626261`.

### PR #8538 · wasm · 8538-shape-hwp-2022 · p4

본문 아래 각주 구분선과 작게 표시한 내용 번호

4쪽은 각주가 본문 마지막 줄 바로 아래에 있고, 5·6쪽은 작은 번호·보통 번호의 크기와 구분선 끝·본문 뒤 각주 내용 두 줄이 독립 Print와 대응한다. raw 100.0%.

- [review](png/pr8538/wasm/8538-shape-hwp-2022/review_004.png), SHA-256 `7e22eaf36b3f01ac364f30de44ca2539fce24239a0460125cd9f80d91d8beba5`.
- [overlay](png/pr8538/wasm/8538-shape-hwp-2022/overlay_004.png), SHA-256 `14b2b36af42923adf8862a2e5558c61b45466e2db0e1e091a11fca4929bc1024`.

### PR #8538 · wasm · 8538-shape-hwp-2022 · p5

본문 아래 각주 구분선과 작게 표시한 내용 번호

4쪽은 각주가 본문 마지막 줄 바로 아래에 있고, 5·6쪽은 작은 번호·보통 번호의 크기와 구분선 끝·본문 뒤 각주 내용 두 줄이 독립 Print와 대응한다. raw 100.0%.

- [review](png/pr8538/wasm/8538-shape-hwp-2022/review_005.png), SHA-256 `1d9dac01b18411d952ef04790bf1155dc3f4dc144e6e649149ac3b48f27a63fe`.
- [overlay](png/pr8538/wasm/8538-shape-hwp-2022/overlay_005.png), SHA-256 `4737c8e5dd93765b36b116773cc577408b581477a6edc68635de7ed56b401820`.

### PR #8541 · native · api-8541-split-hwpx · p1

실제 공개 API 구역 분할 뒤 앞·뒤 내용의 쪽 소속

분할본은 앞·뒤가 각 1·2쪽에 있고 합침본은 한 쪽에서 앞·뒤 순서를 보존한다. 원본 내용의 쪽 소속과 줄 순서를 Print와 대조했다. raw 100.0%.

- [review](png/pr8541/native/api-8541-split-hwpx/review_001.png), SHA-256 `b83a1a3d9994aa8659a94b410db4e98562ce32d4f911845b041700c749354cdd`.
- [overlay](png/pr8541/native/api-8541-split-hwpx/overlay_001.png), SHA-256 `0f6df898f0a0764d0ad796969ea0fafbcda83d6586aeba760481ee3010600c86`.

### PR #8541 · native · api-8541-split-hwpx · p2

실제 공개 API 구역 분할 뒤 앞·뒤 내용의 쪽 소속

분할본은 앞·뒤가 각 1·2쪽에 있고 합침본은 한 쪽에서 앞·뒤 순서를 보존한다. 원본 내용의 쪽 소속과 줄 순서를 Print와 대조했다. raw 100.0%.

- [review](png/pr8541/native/api-8541-split-hwpx/review_002.png), SHA-256 `cb3f06727b449546ab24ef33d9796a5f298fb3d6a49f93e1838470ea84b849ff`.
- [overlay](png/pr8541/native/api-8541-split-hwpx/overlay_002.png), SHA-256 `0f1b74b71147fb7e98a9f47c2c91b503e0b7cda77085f44b5b53a8fcda47a2ac`.

### PR #8541 · wasm · api-8541-split-hwpx · p1

실제 공개 API 구역 분할 뒤 앞·뒤 내용의 쪽 소속

분할 뒤 앞·뒤 글자가 각각 첫째·둘째 쪽의 본문 시작에 보존된다. raw 100.0%.

- [review](png/pr8541/wasm/api-8541-split-hwpx/review_001.png), SHA-256 `480a5071d6ab35e5390852b701557e02187c812504b4f46f1de76f35098315d2`.
- [overlay](png/pr8541/wasm/api-8541-split-hwpx/overlay_001.png), SHA-256 `16d0713c842ca7911546826e542f315a697ba3727fc4cf7e88ee88dbe9ad4119`.

### PR #8541 · wasm · api-8541-split-hwpx · p2

실제 공개 API 구역 분할 뒤 앞·뒤 내용의 쪽 소속

분할 뒤 앞·뒤 글자가 각각 첫째·둘째 쪽의 본문 시작에 보존된다. raw 100.0%.

- [review](png/pr8541/wasm/api-8541-split-hwpx/review_002.png), SHA-256 `958b320ea95cb3311d144b588680aece87e907ecc1dbcf6a3b33e5c9ee6b9250`.
- [overlay](png/pr8541/wasm/api-8541-split-hwpx/overlay_002.png), SHA-256 `551bcae5bf334c5a4795b0e8bf45a505181b346ab2f03a7286f182cf72def477`.

### PR #8541 · native · api-8541-merged-hwpx · p1

구역 합침 뒤 앞·뒤 문단 순서 보존

분할본은 앞·뒤가 각 1·2쪽에 있고 합침본은 한 쪽에서 앞·뒤 순서를 보존한다. 원본 내용의 쪽 소속과 줄 순서를 Print와 대조했다. raw 100.0%.

- [review](png/pr8541/native/api-8541-merged-hwpx/review_001.png), SHA-256 `f04f213c489a7e71039143a10e5685c713034f9074c47f1048c0c68a3261bbe4`.
- [overlay](png/pr8541/native/api-8541-merged-hwpx/overlay_001.png), SHA-256 `4c0c6b15637004e086a1e6a2994ea74ead8f04dbf2749398496044088858c399`.

### PR #8541 · wasm · api-8541-merged-hwpx · p1

구역 합침 뒤 앞·뒤 문단 순서 보존

합친 첫 쪽에서 앞·뒤 문단의 순서와 줄 간격이 보존된다. raw 100.0%.

- [review](png/pr8541/wasm/api-8541-merged-hwpx/review_001.png), SHA-256 `6575a38422c7b0d643a0c229f2775015dada0c71eb1c3d5cd736577106b8268e`.
- [overlay](png/pr8541/wasm/api-8541-merged-hwpx/overlay_001.png), SHA-256 `0694ef3ba20f6508a3f50b34e5eab238cad6a5a688ce6f643e28b863808b2aea`.

### PR #8549 · native · api-8549-moved-terminal-tab-hwp · p1

그림 이동 뒤 탭·글자처럼 취급 개체가 있는 문단의 두 물리 행과 저장 줄 시작

옮긴 그림 전후 탭·글자처럼 취급 개체의 두 물리 행과 순서가 Print와 맞는다. 그림 가장자리의 색상·안티앨리어싱 차이는 픽셀 완전 일치로 표현하지 않는다. raw 99.92003%.

- [review](png/pr8549/native/api-8549-moved-terminal-tab-hwp/review_001.png), SHA-256 `d5375326734560caf652cc5b38cc3ee30b7a68717c5d72447e776033971ac4b2`.
- [overlay](png/pr8549/native/api-8549-moved-terminal-tab-hwp/overlay_001.png), SHA-256 `545fb8bdd8276d19d3448d90fa8415528fb8def968b569555dd44af5ae725fbb`.

### PR #8549 · wasm · api-8549-moved-terminal-tab-hwp · p1

그림 이동 뒤 탭·글자처럼 취급 개체가 있는 문단의 두 물리 행과 저장 줄 시작

이동된 그림과 좌우 본문 두 물리 행의 배치·탭 뒤 내용 보존을 대조했다. 그림 자체에는 래스터 색상 경계 차이가 있으나 행 소속과 본문 원점은 대응한다. raw 99.92149%.

- [review](png/pr8549/wasm/api-8549-moved-terminal-tab-hwp/review_001.png), SHA-256 `f82e6c98bbab36f9edbc5f0640d711d43b817304f750dbb7beab812a26d1a358`.
- [overlay](png/pr8549/wasm/api-8549-moved-terminal-tab-hwp/overlay_001.png), SHA-256 `12024b1408be551dc3e1f1365987efa707529b5b99b16691e2b457139fd61468`.

### PR #8558 · native · api-8558-actual-r3-character-bullet-hwp · p1

실제 API 문자 글머리표의 모양·간격

실제 API 저장본의 문자 글머리표 모양·간격과 HWPX 빨간 사각 그림 글머리표·가 문단 간격을 Print와 대조했다. API 구조 검사는 별도로 연결한다. raw 100.0%.

- [review](png/pr8558/native/api-8558-actual-r3-character-bullet-hwp/review_001.png), SHA-256 `11c3f8d3e39b5eda832a043d3a0c2bde62c077e0d4eaa1fee8e141eeeaac0005`.
- [overlay](png/pr8558/native/api-8558-actual-r3-character-bullet-hwp/overlay_001.png), SHA-256 `091866572cfd119b85564a9335e92c29ec9bab20672040bdc0e74166e220b490`.

### PR #8558 · wasm · api-8558-actual-r3-character-bullet-hwp · p1

실제 API 문자 글머리표의 모양·간격

문자 글머리표의 마름모 윤곽·기울임과 본문 간격이 대응한다. raw 100.0%.

- [review](png/pr8558/wasm/api-8558-actual-r3-character-bullet-hwp/review_001.png), SHA-256 `027ec7dc59d616c6d3a637cffbdcd1500739edf74cbd9e4d30fa6fa2856dcf52`.
- [overlay](png/pr8558/wasm/api-8558-actual-r3-character-bullet-hwp/overlay_001.png), SHA-256 `32811347cca877cfc72d0718085b4a1af2023f94680b7f83e417ef70723de940`.

### PR #8558 · native · api-8558-actual-r3-picture-bullet-hwpx · p1

실제 API 그림 글머리표를 hc:img로 저장한 같은 HWPX의 한컴 Print 대조

실제 API 저장본의 문자 글머리표 모양·간격과 HWPX 빨간 사각 그림 글머리표·가 문단 간격을 Print와 대조했다. API 구조 검사는 별도로 연결한다. raw 100.0%.

- [review](png/pr8558/native/api-8558-actual-r3-picture-bullet-hwpx/review_001.png), SHA-256 `4a1eba678c0ea83c6fc9e89df69c61a3fbc0d771c0317170fb31eca78bd1ef79`.
- [overlay](png/pr8558/native/api-8558-actual-r3-picture-bullet-hwpx/overlay_001.png), SHA-256 `070e7f26b7b275ab71918245beb35d80b0416ab3e00c1bb713f155f2c8680493`.

### PR #8558 · wasm · api-8558-actual-r3-picture-bullet-hwpx · p1

실제 API 그림 글머리표를 hc:img로 저장한 같은 HWPX의 한컴 Print 대조

빨간 그림 글머리표의 색상·사각형 크기·본문 사이 간격과 기준선을 대조했다. raw 100.0%.

- [review](png/pr8558/wasm/api-8558-actual-r3-picture-bullet-hwpx/review_001.png), SHA-256 `63cc469205352d3ff5a23d540f947d4936fcfcb62e1bd3516a2549ff2e25174d`.
- [overlay](png/pr8558/wasm/api-8558-actual-r3-picture-bullet-hwpx/overlay_001.png), SHA-256 `83f7a8a483b6be07b294be35206efb41564308fe54031a5482db6ee6af452a82`.

### PR #8564 · native · api-8564-context-after · p1

실제 날짜·파일 경로 코드 앞뒤 입력이 필드 밖에 남는 저장 결과

날짜 앞뒤 A앞·뒤B와 파일명 앞의 앞·뒤C를 직접 대조했다. 인쇄 때 파일명 코드가 임시 이름으로 재평가되어 문자열 길이가 다르다. api/pr8564-print-context.json에 독립 영수증·빈 기본 Scripts·실제 PDF 문자열을 기록했고 위치 오차나 동일 문자열의 일치로 주장하지 않는다. raw 92.01038%.

- [review](png/pr8564/native/api-8564-context-after/review_001.png), SHA-256 `dc6950f9575863c4acd419b6a900db3b14ef840af5d6f378f64f5911d2cabfe1`.
- [overlay](png/pr8564/native/api-8564-context-after/overlay_001.png), SHA-256 `12581830679eda740fbff35a56c97fc2bc6b807a2a8ee7c6f59dbe24da75f616`.

### PR #8564 · wasm · api-8564-context-after · p1

실제 날짜·파일 경로 코드 앞뒤 입력이 필드 밖에 남는 저장 결과

날짜 필드 전후 A앞·뒤B, 경로 필드 전후 앞·뒤C가 보존된다. 파일명 필드는 MCP scripts_disabled 인쇄의 임시 서버 파일명과 RHWP 저장 파일명이 달라 내용이 중첩되지 않는다. 기존 독립 변환 영수증으로 확인한 동적 필드 재평가 차이이며 동일 문자열 일치로 주장하지 않는다. raw 92.28612%.

- [review](png/pr8564/wasm/api-8564-context-after/review_001.png), SHA-256 `5a8c8b7f69736b227bdf8d0382ba7fcb983b6566a16a439be11183b1efa385a4`.
- [overlay](png/pr8564/wasm/api-8564-context-after/overlay_001.png), SHA-256 `9783b2df224eedfc4d29b0704c8ade2196f41acd617ca892e5041ed99e19aea7`.

### PR #8570 · native · api-8570-non-inline-cell-table-hwpx · p1

실제 API 비인라인 중첩 표의 셀 내부 포함과 후속 문단

실제 공개 API 생성 HWPX와 독립 Print의 같은 1쪽이다. 안쪽 표는 두 번째 바깥 칸 안에 있고 위·아래 여백, 2×2 격자와 안쪽 표 문자열, 바깥 다음 행이 보존된다. 선의 안티앨리어싱·농도 차이를 raw 100%의 픽셀 완전 일치로 부르지 않는다. raw 100.0%.

- [review](png/pr8570/native/api-8570-non-inline-cell-table-hwpx/review_001.png), SHA-256 `8d5f526fef5c3ec07aa4b69055bdb7e3c73f9c901026c17420bd04acf46094cf`.
- [overlay](png/pr8570/native/api-8570-non-inline-cell-table-hwpx/overlay_001.png), SHA-256 `d0bd4570ba75959c1b94157e4d946d140d9e1cef3daa58d274eaf36acd3a4dd9`.

### PR #8570 · wasm · api-8570-non-inline-cell-table-hwpx · p1

실제 API 비인라인 중첩 표의 셀 내부 포함과 후속 문단

비인라인 안쪽 표가 중앙 셀 내부에 포함되고, 외곽 표의 위·아래 행과 안쪽 표 두 행 및 안쪽 표 문자열이 독립 Print와 대응한다. raw 100.0%.

- [review](png/pr8570/wasm/api-8570-non-inline-cell-table-hwpx/review_001.png), SHA-256 `5c51de8c944e7adf0dcd964b9d0b82eeafe632bad012ea9e71a1a1c66da8fbcb`.
- [overlay](png/pr8570/wasm/api-8570-non-inline-cell-table-hwpx/overlay_001.png), SHA-256 `bfae0658d51eded697e2d09899f4a82b084d51e4e93f73ae9e91922acf0f9aa0`.

### PR #8571 · native · api-8571-header-table-hwpx · p1

실제 머리말 셀 경로 API가 생성한 표의 배치와 본문 보존

실제 API 저장본의 hp·hq 셀과 머리말 표 외곽·중앙 경계를 독립 Print와 대조했다. raw 100.0%.

- [review](png/pr8571/native/api-8571-header-table-hwpx/review_001.png), SHA-256 `9822933ac5ceaa66b75376b53ec200de83f27bef992b75fcaf3b94469c6e895c`.
- [overlay](png/pr8571/native/api-8571-header-table-hwpx/overlay_001.png), SHA-256 `dcb3464a8a9c5e7f0dd3b518c1cf12177bc5b4e15c70edf80a1d913af9960378`.

### PR #8571 · wasm · api-8571-header-table-hwpx · p1

실제 머리말 셀 경로 API가 생성한 표의 배치와 본문 보존

실제 API 저장본 머리말의 hp·hq 셀과 표 외곽·중앙 경계 원점을 대조했다. raw 100.0%.

- [review](png/pr8571/wasm/api-8571-header-table-hwpx/review_001.png), SHA-256 `a38ece90d61048a4117d8e788280ac57a3333e4b308fc2b3c492891ad89fb0ec`.
- [overlay](png/pr8571/wasm/api-8571-header-table-hwpx/overlay_001.png), SHA-256 `ff1051f2e2509b7718bcb4dcef4244910eef03ba96649b7486a74a32f87884bb`.

### PR #8571 · native · header-tac-pair-wrap · p1

같은 머리말 문단에서 가용 너비를 넘은 표 두 개의 줄바꿈

머리말 가용 폭을 넘긴 두 표가 각각 다른 줄을 차지하며 격자와 본문 시작이 Print와 맞는다. raw 100.0%.

- [review](png/pr8571/native/header-tac-pair-wrap/review_001.png), SHA-256 `4fb7151411c4392bf7a61fa4b1b850e1bdcb3b342d88b8f47c2687025468504a`.
- [overlay](png/pr8571/native/header-tac-pair-wrap/overlay_001.png), SHA-256 `e63fa0593142a5370b89a97e92f9f20ea227c47b9666a60b9b9241601230ffc5`.

### PR #8571 · wasm · header-tac-pair-wrap · p1

같은 머리말 문단에서 가용 너비를 넘은 표 두 개의 줄바꿈

머리말 첫 표·둘째 표가 가용 너비에 따라 서로 다른 줄에 놓이고 겹치지 않으며 독립 Print와 대응한다. raw 100.0%.

- [review](png/pr8571/wasm/header-tac-pair-wrap/review_001.png), SHA-256 `99c155b0ed9c1e05cca571fcf08304366d3fe84047a71df7005ba3a61084aaa0`.
- [overlay](png/pr8571/wasm/header-tac-pair-wrap/overlay_001.png), SHA-256 `7c4551c06dcc567c7fa6e4f275e0f7e41be97ca6f001b37662da6a0aa655636d`.

### PR #8572 · native · 8572-fix-typed-hwp-2022 · p1

원 PR의 표안한글 편집 저장본: 제목 표 확장·다음 표 간격 및 다음 쪽 첫 문단

수정 저장본 p1의 표안한글 제목 표 확장·후속 Ⅰ 표 간격과 p2 첫 3D 프린터 문단을 같은 Print와 대조했다. 표안한글은 PDF 텍스트에서도 확인된 원문이다. raw 100.0%.

- [review](png/pr8572/native/8572-fix-typed-hwp-2022/review_001.png), SHA-256 `5539f5fe5ea371b8cfab5ab753092afee6ce4410ad4102da9e5c7b779aa34f06`.
- [overlay](png/pr8572/native/8572-fix-typed-hwp-2022/overlay_001.png), SHA-256 `2ea015f464f8e6612af16eadbece245a081681b9bf38c87b91c1b07395e8f58e`.

### PR #8572 · native · 8572-fix-typed-hwp-2022 · p2

원 PR의 표안한글 편집 저장본: 제목 표 확장·다음 표 간격 및 다음 쪽 첫 문단

수정 저장본 p1의 표안한글 제목 표 확장·후속 Ⅰ 표 간격과 p2 첫 3D 프린터 문단을 같은 Print와 대조했다. 표안한글은 PDF 텍스트에서도 확인된 원문이다. raw 100.0%.

- [review](png/pr8572/native/8572-fix-typed-hwp-2022/review_002.png), SHA-256 `bd2c54129f5a2b754cc933a119599356962607215b703c42d54e61cb6acee4a4`.
- [overlay](png/pr8572/native/8572-fix-typed-hwp-2022/overlay_002.png), SHA-256 `648a9b060584b2b89b3db47e8611ee14be53860d656429b4d887b01074119a1c`.

### PR #8572 · wasm · 8572-fix-typed-hwp-2022 · p1

원 PR의 표안한글 편집 저장본: 제목 표 확장·다음 표 간격 및 다음 쪽 첫 문단

편집으로 커진 제목 표 아래에 두 번째 제목·본문이 순서대로 놓인다. 첫 쪽 하단 각주와 둘째 쪽 첫 문단·쪽 번호가 독립 Print와 대응한다. raw 99.99918%.

- [review](png/pr8572/wasm/8572-fix-typed-hwp-2022/review_001.png), SHA-256 `581659eab29ba44b35d2e6fe9d34475ea6ca55916dee8a6cb4335019989b63cb`.
- [overlay](png/pr8572/wasm/8572-fix-typed-hwp-2022/overlay_001.png), SHA-256 `2f9051d8cc68d53340ee3c46ad873664f6008de632e833b54848d81f885d8a20`.

### PR #8572 · wasm · 8572-fix-typed-hwp-2022 · p2

원 PR의 표안한글 편집 저장본: 제목 표 확장·다음 표 간격 및 다음 쪽 첫 문단

편집으로 커진 제목 표 아래에 두 번째 제목·본문이 순서대로 놓인다. 첫 쪽 하단 각주와 둘째 쪽 첫 문단·쪽 번호가 독립 Print와 대응한다. raw 100.0%.

- [review](png/pr8572/wasm/8572-fix-typed-hwp-2022/review_002.png), SHA-256 `ec216b7dc39a42bdb3a4b3d8f5aac993f65ee9b9f01b9594caee442130cab2a7`.
- [overlay](png/pr8572/wasm/8572-fix-typed-hwp-2022/overlay_002.png), SHA-256 `0041184c71c7c53e00f700a56c9d3088f09434c2d5c948e4f63ec5100034d8fd`.

### PR #8577 · native · 8577-hancom-page-number-top-hwpx-2022 · p1

머리말 높이가 다른 두 구역의 위쪽 쪽 번호 기준선

머리말 높이가 다른 두 쪽의 상단 쪽번호 -1-·-2-와 본문 1쪽·2쪽을 독립 Print와 대조했다. 상단 번호 기준선·장식·본문과의 선후 관계가 모두 일치한다. raw 100.0%.

- [review](png/pr8577/native/8577-hancom-page-number-top-hwpx-2022/review_001.png), SHA-256 `fed71742bf25df29d5eda63d2783b860afbd0cf3d29669f16dc44580950544dc`.
- [overlay](png/pr8577/native/8577-hancom-page-number-top-hwpx-2022/overlay_001.png), SHA-256 `78d0f26375d061e29242db4e7bcffef05de3d02480c741a52d26555333e376db`.

### PR #8577 · native · 8577-hancom-page-number-top-hwpx-2022 · p2

머리말 높이가 다른 두 구역의 위쪽 쪽 번호 기준선

머리말 높이가 다른 두 쪽의 상단 쪽번호 -1-·-2-와 본문 1쪽·2쪽을 독립 Print와 대조했다. 상단 번호 기준선·장식·본문과의 선후 관계가 모두 일치한다. raw 100.0%.

- [review](png/pr8577/native/8577-hancom-page-number-top-hwpx-2022/review_002.png), SHA-256 `594ab0a3e52cdd523ed508cce7ffb19b72b9062c6a143247aec8594b76d8ac86`.
- [overlay](png/pr8577/native/8577-hancom-page-number-top-hwpx-2022/overlay_002.png), SHA-256 `bab679d85a058f66b2c0cfda48b57e6bb6ab7f6b54d50b70af9e61fabf2eb0f0`.

### PR #8577 · wasm · 8577-hancom-page-number-top-hwpx-2022 · p1

머리말 높이가 다른 두 구역의 위쪽 쪽 번호 기준선

서로 다른 머리말 높이의 두 구역에서 위쪽 쪽 번호·장식 하이픈의 기준선과 본문 시작을 대조했다. raw 100.0%.

- [review](png/pr8577/wasm/8577-hancom-page-number-top-hwpx-2022/review_001.png), SHA-256 `796361eff50ccbbd5b9ffb84639e256db74073732bdbb84a8648edfc162e1c22`.
- [overlay](png/pr8577/wasm/8577-hancom-page-number-top-hwpx-2022/overlay_001.png), SHA-256 `6596f128bb1cc255b8a8611d2b41196d7ec66695cf3378223a18107143c8e398`.

### PR #8577 · wasm · 8577-hancom-page-number-top-hwpx-2022 · p2

머리말 높이가 다른 두 구역의 위쪽 쪽 번호 기준선

서로 다른 머리말 높이의 두 구역에서 위쪽 쪽 번호·장식 하이픈의 기준선과 본문 시작을 대조했다. raw 100.0%.

- [review](png/pr8577/wasm/8577-hancom-page-number-top-hwpx-2022/review_002.png), SHA-256 `d4989b334edc10123a1763e9ab3dcf157e0fc27fa4a5cd335dc14d9952d71e9c`.
- [overlay](png/pr8577/wasm/8577-hancom-page-number-top-hwpx-2022/overlay_002.png), SHA-256 `ad5e2409b6aa1ce2eb1fa0de1cc2d656910437e94037795c46d1b9526706bc5a`.

### PR #8586 · native · 8586-hancom-page-number-sections-hwpx-2022 · p1

원 PR의 구역별 쪽 번호 소속과 표시 위치

두 구역의 우상단 -1-·-2- 쪽 번호와 본문 쪽 소속·상단 간격을 Print와 대조했다. raw 100.0%.

- [review](png/pr8586/native/8586-hancom-page-number-sections-hwpx-2022/review_001.png), SHA-256 `11f1d11e4a10f5a959969ad219cd5012b5cde4b9a1329b752673ce2be236b161`.
- [overlay](png/pr8586/native/8586-hancom-page-number-sections-hwpx-2022/overlay_001.png), SHA-256 `93d682761860a516ea068f49d7c18f69a6fcaf38524466fc617d45967e4d1546`.

### PR #8586 · native · 8586-hancom-page-number-sections-hwpx-2022 · p2

원 PR의 구역별 쪽 번호 소속과 표시 위치

두 구역의 우상단 -1-·-2- 쪽 번호와 본문 쪽 소속·상단 간격을 Print와 대조했다. raw 100.0%.

- [review](png/pr8586/native/8586-hancom-page-number-sections-hwpx-2022/review_002.png), SHA-256 `ae3f936ef7ef44cd4b08c87e95bfab1f54d0abeb1a5d0e4362c47e1da212d4be`.
- [overlay](png/pr8586/native/8586-hancom-page-number-sections-hwpx-2022/overlay_002.png), SHA-256 `5760dcdec1f0cb64905b49bd9dbb4c0cdd3d68c8d27433354f3446484a85c6a1`.

### PR #8586 · wasm · 8586-hancom-page-number-sections-hwpx-2022 · p1

원 PR의 구역별 쪽 번호 소속과 표시 위치

두 구역의 오른쪽 위 쪽 번호1·2와 해당 본문 쪽 소속이 독립 Print와 대응한다. raw 100.0%.

- [review](png/pr8586/wasm/8586-hancom-page-number-sections-hwpx-2022/review_001.png), SHA-256 `ae6cd983117917ea41a2f75d988fcd2c393461047850082c9442b82c06dcf683`.
- [overlay](png/pr8586/wasm/8586-hancom-page-number-sections-hwpx-2022/overlay_001.png), SHA-256 `aa2dba7d003b7c9fe6348ab2223426796155addeed2d54e154dfb57da86eb640`.

### PR #8586 · wasm · 8586-hancom-page-number-sections-hwpx-2022 · p2

원 PR의 구역별 쪽 번호 소속과 표시 위치

두 구역의 오른쪽 위 쪽 번호1·2와 해당 본문 쪽 소속이 독립 Print와 대응한다. raw 100.0%.

- [review](png/pr8586/wasm/8586-hancom-page-number-sections-hwpx-2022/review_002.png), SHA-256 `28d6ae9beb8824a33ec9baf60f9d39603dcaee6fd1292788d75d28b722177f45`.
- [overlay](png/pr8586/wasm/8586-hancom-page-number-sections-hwpx-2022/overlay_002.png), SHA-256 `c5d6cb0ebeecad3966fd3486708eecacb01c84c6b8441c8a3123229590e314dd`.

### PR #8602 · native · api-8602-empty-deleted-hwp · p1

실제 삭제 후 빈 문단의 한컴 열기·Print 결과. PARA_TEXT 유무는 원문 레코드 검사로 입증한다.

삭제 후 빈 문단은 인쇄에서 별도 글자를 만들지 않고 앞 문단을 보존한다. PARA_TEXT 삭제 자체는 그림의 차이가 아니라 독립 CFB 레코드 검사로 입증한다. raw 100.0%.

- [review](png/pr8602/native/api-8602-empty-deleted-hwp/review_001.png), SHA-256 `6529b4a165b673e4d25fb258534a17217ea31d3e9e64a0d2134ce04a198a3e8d`.
- [overlay](png/pr8602/native/api-8602-empty-deleted-hwp/overlay_001.png), SHA-256 `e9bc34c6117d817bd29269651cc7c66079827c82b04eb18241b68c6263a47ec6`.

### PR #8602 · wasm · api-8602-empty-deleted-hwp · p1

실제 삭제 후 빈 문단의 한컴 열기·Print 결과. PARA_TEXT 유무는 원문 레코드 검사로 입증한다.

삭제 뒤 앞 문단이 보존되고 불필요한 글자·페이지가 생기지 않는다. 빈 PARA_TEXT 생략은 그림이 아닌 별도의 독립 CFB 원시 레코드 검사로 입증한다. raw 100.0%.

- [review](png/pr8602/wasm/api-8602-empty-deleted-hwp/review_001.png), SHA-256 `84456791ff6510d77f19828fe87c7dec7f2d2e9eb2fcd3d32090060a3b17d913`.
- [overlay](png/pr8602/wasm/api-8602-empty-deleted-hwp/overlay_001.png), SHA-256 `120175558b059966d99c9ca6470f02fe04768c164dd27c61ce9501d8c22c6f3e`.

### PR #8616 · native · 8616-hancom-mac-shapes-hwp-2022 · p1

원 PR 한컴 저장본의 14개 겹침 모양·PUA 숫자와 전각 폭

14개 모양의 순서·두 자리 12·반전 모양·뒤 한글·끝 문자열이 같은 첫 쪽에 남는다. PUA 자체가 본문으로 노출되지 않고 원·사각·삼각·순환 모양과 숫자가 보인다. 획 농도·경계의 차이는 raw와 구분한다. raw 99.2128%.

- [review](png/pr8616/native/8616-hancom-mac-shapes-hwp-2022/review_001.png), SHA-256 `6c05051d7303ccd6f805896eca715ee55b55a71d9bb1becc59b4602abd58cc99`.
- [overlay](png/pr8616/native/8616-hancom-mac-shapes-hwp-2022/overlay_001.png), SHA-256 `ac1768517e758a10d51a7902c071f4f2413e22ec0bd67f69967204c7cf088512`.

### PR #8616 · wasm · 8616-hancom-mac-shapes-hwp-2022 · p1

원 PR 한컴 저장본의 14개 겹침 모양·PUA 숫자와 전각 폭

14개 모양의 원·반전원·사각형·삼각형·순환 기호와 PUA 숫자12, 아래 한글 겹침 글자·후속 끝 글자가 보존된다. 자원 글리프의 미세한 윤곽 차이는 있으나 전각 칸·문자 순서는 대응한다. raw 99.22558%.

- [review](png/pr8616/wasm/8616-hancom-mac-shapes-hwp-2022/review_001.png), SHA-256 `240fa3772f15738811f045896c3a5227afb8c789e25961bf7bbaba3dda69058a`.
- [overlay](png/pr8616/wasm/8616-hancom-mac-shapes-hwp-2022/overlay_001.png), SHA-256 `6d075f06986388f5942857e111b6c12b34463dbd3cc6550fc3fa05a6ad5b5448`.

### PR #8616 · native · 8616-hancom-mac-dutmal-hwpx-2022 · p1

원 PR의 본말·덧말 실제 표시

漢字·한·대한민국만세·글자·스타일의 본말과 위·아래 덧말의 소속을 대조했다. 본문 앞 가·뒤 나·후속 문자열과 덧말의 위아래 위치, 차례제목의 색상·크기를 보존한다. raw 100.0%.

- [review](png/pr8616/native/8616-hancom-mac-dutmal-hwpx-2022/review_001.png), SHA-256 `89935d0d0bdf54e469f50ea4da0b3d56ff006b721b64de048c7776ac50fe90d7`.
- [overlay](png/pr8616/native/8616-hancom-mac-dutmal-hwpx-2022/overlay_001.png), SHA-256 `941a0ec793e14f1cb6a26d4fa01f7710e75c1e1cc93dbfe99a9cda7273247690`.

### PR #8616 · wasm · 8616-hancom-mac-dutmal-hwpx-2022 · p1

원 PR의 본말·덧말 실제 표시

본말 漢字·한·대한민국만세·글자·스타일 및 위·아래 덧말의 소속과 순서, 후속 본문 원점이 독립 Print와 대응한다. raw 100.0%.

- [review](png/pr8616/wasm/8616-hancom-mac-dutmal-hwpx-2022/review_001.png), SHA-256 `edbcf4e78ba397d7e9f5097b741c8326215cbd8519a67e4b99b122b2ae4ddfa0`.
- [overlay](png/pr8616/wasm/8616-hancom-mac-dutmal-hwpx-2022/overlay_001.png), SHA-256 `be331932cce0c8e7b1f9394004e610f664bd295a641d156edb26732735cc0ea8`.

### PR #8629 · native · 8629-hancom-mac-200-b5-hwp-2022 · p1

원 PR의 원고지 B5 200자 격자·묶음 숫자·금칙 매달기

B5 200자 원고지의 한글 한 칸·영문 숫자 묶음·쉼표와 마침표·줄끝 영문 매달기 E FG·다음 줄 끝 다.의 소속을 대조했다. 글자가 누락되지 않고 격자 내 원점·행 구성이 유지된다. 하단 격자 획의 서브픽셀 차이를 숨기지 않는다. raw 100.0%.

- [review](png/pr8629/native/8629-hancom-mac-200-b5-hwp-2022/review_001.png), SHA-256 `d0606c75c34350b6b0099e5076a9662c3e2493f118fc6c247276eb80bca725b0`.
- [overlay](png/pr8629/native/8629-hancom-mac-200-b5-hwp-2022/overlay_001.png), SHA-256 `338ce6cd106d0523419033d6c6a6fc22376ad68188c2c732c95c058470faec68`.

### PR #8629 · wasm · 8629-hancom-mac-200-b5-hwp-2022 · p1

원 PR의 원고지 B5 200자 격자·묶음 숫자·금칙 매달기

B5 원고지 격자에서 한글 한 칸·영문 묶음·두 자리 숫자·문장부호 매달기와 다음 줄 마지막 다.가 Print와 대응한다. 격자 선의 1픽셀 양자화 차이를 엄격 픽셀 동일성으로 주장하지 않는다. raw 100.0%.

- [review](png/pr8629/wasm/8629-hancom-mac-200-b5-hwp-2022/review_001.png), SHA-256 `6251e570ef09d4f912a55d96c5bcb78d1f2dd145b5f2ace8c7d1c9c1d69c2831`.
- [overlay](png/pr8629/wasm/8629-hancom-mac-200-b5-hwp-2022/overlay_001.png), SHA-256 `662a7966824227e7647bc332bafdfbf2ebd1dbc3e08dd2b2c63e68a65000d268`.

### PR #8616 · native · 8616-existing-caret-false-hwp · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

머리·앞·겹침 슬롯·각주 번호·뒤끝의 순서와 같은 줄 소속을 대조했다. 본문 아래 각주와 하단 각주의 두 모드에서 구분선·번호 위치가 독립 Print와 대응한다. raw 100.0%.

- [review](png/pr8616/native/8616-existing-caret-false-hwp/review_001.png), SHA-256 `96e5ce30a80fbcd1b0fa58f4249303d90332eab8f21dcb2662a1affc206d51d2`.
- [overlay](png/pr8616/native/8616-existing-caret-false-hwp/overlay_001.png), SHA-256 `21e22b3f1112c7d2d0685a5a0ae49b405ffca0823742c801de12e461256b7c42`.

### PR #8616 · wasm · 8616-existing-caret-false-hwp · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

머리·앞·겹침 슬롯·각주 번호·뒤끝의 순서와 같은 줄 소속을 대조했다. 본문 아래 각주와 하단 각주의 두 모드에서 구분선·번호 위치가 독립 Print와 대응한다. raw 100.0%.

- [review](png/pr8616/wasm/8616-existing-caret-false-hwp/review_001.png), SHA-256 `af7a19c7a183ba54ec28ec08bf5c3e85c4e015ced24556b3e6e9e283b7eeb939`.
- [overlay](png/pr8616/wasm/8616-existing-caret-false-hwp/overlay_001.png), SHA-256 `c10b3f281e93c3f2a8db8b40fadec48eb3e9d157d1e840946d6cd6c52a341595`.

### PR #8616 · native · 8616-existing-caret-false-hwpx · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

HWPX 원본의 큰 겹침 글리프가 독립 Print에서도 같은 크기와 위치에 있다. 머리·앞·겹침·번호·뒤끝의 소속, 본문 아래/하단 각주 두 모드의 구분선과 번호를 대조했다. 크기 차이를 HWP 원본과 혼동하지 않는다. raw 100.0%.

- [review](png/pr8616/native/8616-existing-caret-false-hwpx/review_001.png), SHA-256 `f4187a3690d243b6bb625b6e51dcb3a9dc4b524664553ec582040da3f18ac378`.
- [overlay](png/pr8616/native/8616-existing-caret-false-hwpx/overlay_001.png), SHA-256 `6ba1846d51341ce67503338111158a1421e048a7c6f15142f66333d002f594b8`.

### PR #8616 · wasm · 8616-existing-caret-false-hwpx · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

HWPX 원본의 큰 겹침 글리프가 독립 Print에서도 같은 크기와 위치에 있다. 머리·앞·겹침·번호·뒤끝의 소속, 본문 아래/하단 각주 두 모드의 구분선과 번호를 대조했다. 크기 차이를 HWP 원본과 혼동하지 않는다. raw 100.0%.

- [review](png/pr8616/wasm/8616-existing-caret-false-hwpx/review_001.png), SHA-256 `514546fc76ab47467d5a58658d6c89bd0c25ca787a8fcd44a6c339ada9d6ae65`.
- [overlay](png/pr8616/wasm/8616-existing-caret-false-hwpx/overlay_001.png), SHA-256 `74661ce6a285c5db84fd2a500b5c1e0e3ac587bd93c9dad5c43dc9f88100deec`.

### PR #8616 · native · 8616-existing-caret-true-hwp · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

머리·앞·겹침 슬롯·각주 번호·뒤끝의 순서와 같은 줄 소속을 대조했다. 본문 아래 각주와 하단 각주의 두 모드에서 구분선·번호 위치가 독립 Print와 대응한다. raw 100.0%.

- [review](png/pr8616/native/8616-existing-caret-true-hwp/review_001.png), SHA-256 `c194a1ac615bde14cad7ed1508c7ac6a8ae31f411a878c75ad62f4a8649a7a55`.
- [overlay](png/pr8616/native/8616-existing-caret-true-hwp/overlay_001.png), SHA-256 `972aa4fed9daafdeb132bc34f2615ac125d9e49d46ed98ec7661807b37a9bbe8`.

### PR #8616 · wasm · 8616-existing-caret-true-hwp · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

머리·앞·겹침 슬롯·각주 번호·뒤끝의 순서와 같은 줄 소속을 대조했다. 본문 아래 각주와 하단 각주의 두 모드에서 구분선·번호 위치가 독립 Print와 대응한다. raw 100.0%.

- [review](png/pr8616/wasm/8616-existing-caret-true-hwp/review_001.png), SHA-256 `50f28e5c5f9dcbf1baea79ebb890f56d10df8728f47dc5eee056d14f9f3bbec5`.
- [overlay](png/pr8616/wasm/8616-existing-caret-true-hwp/overlay_001.png), SHA-256 `ba7c6ae4e9a950e8015b8127f072d207d8266bd82ecbe21a95fb4f1eb9840267`.

### PR #8616 · native · 8616-existing-caret-true-hwpx · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

HWPX 원본의 큰 겹침 글리프가 독립 Print에서도 같은 크기와 위치에 있다. 머리·앞·겹침·번호·뒤끝의 소속, 본문 아래/하단 각주 두 모드의 구분선과 번호를 대조했다. 크기 차이를 HWP 원본과 혼동하지 않는다. raw 100.0%.

- [review](png/pr8616/native/8616-existing-caret-true-hwpx/review_001.png), SHA-256 `4f5b7f6356d0fd57b20e929846a7029ce73accd38947bc643a8171bb0f49ab54`.
- [overlay](png/pr8616/native/8616-existing-caret-true-hwpx/overlay_001.png), SHA-256 `14aaa49f12ce0599bed982f3c93147797c3c869eff2ecd780554b579a0ed957d`.

### PR #8616 · wasm · 8616-existing-caret-true-hwpx · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

HWPX 원본의 큰 겹침 글리프가 독립 Print에서도 같은 크기와 위치에 있다. 머리·앞·겹침·번호·뒤끝의 소속, 본문 아래/하단 각주 두 모드의 구분선과 번호를 대조했다. 크기 차이를 HWP 원본과 혼동하지 않는다. raw 100.0%.

- [review](png/pr8616/wasm/8616-existing-caret-true-hwpx/review_001.png), SHA-256 `3f70d2d5c6eb868450cd649d2a87086f8606ddc8dea1f42e684e03c9c855b76e`.
- [overlay](png/pr8616/wasm/8616-existing-caret-true-hwpx/overlay_001.png), SHA-256 `bde35d4abae11ffeac9c546636285637740448d4e7e294847efcdc1836179df1`.

### PR #8538 · native · 8538-shape-hwp-2022 · p6

기존 고정 좌표 검사에 대응하는 완료 페이지

4쪽의 본문 바로 아래 구분선과 각주 두 줄, 5쪽 하단 작은 번호·각주 두 줄을 대조했다. 본문 개행·1) 2) 소속·선 시작·종료·각주 위치를 보존한다. raw 100%를 픽셀 바이트의 동일성으로 표현하지 않는다. 6쪽 review·overlay도 실제 열어 보통 크기 번호 1)·2), 각주 구분선·쪽 하단의 두 각주 행과 본문 네 행의 순서·소속을 Print와 대조했다. 이 기록은 Native 판독이며 fresh WASM 판독을 대신하지 않는다. raw 100.0%.

- [review](png/pr8538/native/8538-shape-hwp-2022/review_006.png), SHA-256 `4615240256b7ddaa2a4fd54d4be522cc387f39ee28ecd4a2700d2ebd2342c78c`.
- [overlay](png/pr8538/native/8538-shape-hwp-2022/overlay_006.png), SHA-256 `389d0130abc5a4b1e27e0aa967af1a4aaf6ddf78db40d583eae84b99b76a33f3`.

### PR #8538 · wasm · 8538-shape-hwp-2022 · p6

기존 고정 좌표 검사에 대응하는 완료 페이지

4쪽은 각주가 본문 마지막 줄 바로 아래에 있고, 5·6쪽은 작은 번호·보통 번호의 크기와 구분선 끝·본문 뒤 각주 내용 두 줄이 독립 Print와 대응한다. raw 100.0%.

- [review](png/pr8538/wasm/8538-shape-hwp-2022/review_006.png), SHA-256 `6c5d12c28ef7cfbd7e127b41fbbcb5c6f37548c3a9c0423915de7cfd9c8f3f13`.
- [overlay](png/pr8538/wasm/8538-shape-hwp-2022/overlay_006.png), SHA-256 `57f7e2b78a33dfbd18a6f563ae348a412514887aa8107fe7c12374eddd79930f`.

### PR #8538 · native · 8538-shape-hwpx-2022 · p4

기존 고정 좌표 검사에 대응하는 완료 페이지

HWPX 저장본의 본문 아래 각주와 작은 번호·보통 번호 두 모드를 각각 독립 Print와 대조했다. 본문 마지막 줄·구분선·각주 번호 끝·각주 두 줄의 순서와 기준선이 대응한다. raw 100.0%.

- [review](png/pr8538/native/8538-shape-hwpx-2022/review_004.png), SHA-256 `ec599e374fdb549de78cdd28f1671a833266f74c230b2758b0797fe8d499a760`.
- [overlay](png/pr8538/native/8538-shape-hwpx-2022/overlay_004.png), SHA-256 `b9161559b05f46b65ae5bbb37d13b16fc5f249ad69ce0c43f87b205398a10c46`.

### PR #8538 · wasm · 8538-shape-hwpx-2022 · p4

기존 고정 좌표 검사에 대응하는 완료 페이지

HWPX 저장본의 본문 아래 각주와 작은 번호·보통 번호 두 모드를 각각 독립 Print와 대조했다. 본문 마지막 줄·구분선·각주 번호 끝·각주 두 줄의 순서와 기준선이 대응한다. raw 100.0%.

- [review](png/pr8538/wasm/8538-shape-hwpx-2022/review_004.png), SHA-256 `e269e579f0a7c974ee4229e5637447816b580793bdf7bb9a7b713119118f9ff3`.
- [overlay](png/pr8538/wasm/8538-shape-hwpx-2022/overlay_004.png), SHA-256 `273f332fcc2f6675095836134d1cce130c339d13a923f223f194a8c9a91842bb`.

### PR #8538 · native · 8538-shape-hwpx-2022 · p5

기존 고정 좌표 검사에 대응하는 완료 페이지

HWPX 저장본의 본문 아래 각주와 작은 번호·보통 번호 두 모드를 각각 독립 Print와 대조했다. 본문 마지막 줄·구분선·각주 번호 끝·각주 두 줄의 순서와 기준선이 대응한다. raw 100.0%.

- [review](png/pr8538/native/8538-shape-hwpx-2022/review_005.png), SHA-256 `343224fce1b7e7d70ae9bd792620f91e110d404b63548bd3023341a3fb32f7ff`.
- [overlay](png/pr8538/native/8538-shape-hwpx-2022/overlay_005.png), SHA-256 `92eb167aedfc167ad4a1f32b7687152b074240276ac844788eab58cdf0eec081`.

### PR #8538 · wasm · 8538-shape-hwpx-2022 · p5

기존 고정 좌표 검사에 대응하는 완료 페이지

HWPX 저장본의 본문 아래 각주와 작은 번호·보통 번호 두 모드를 각각 독립 Print와 대조했다. 본문 마지막 줄·구분선·각주 번호 끝·각주 두 줄의 순서와 기준선이 대응한다. raw 100.0%.

- [review](png/pr8538/wasm/8538-shape-hwpx-2022/review_005.png), SHA-256 `f64e8995fd9783f9034af34a3d05381bcc604fd8741a41d1deb2903767d82da9`.
- [overlay](png/pr8538/wasm/8538-shape-hwpx-2022/overlay_005.png), SHA-256 `5ac5be9c032f7de9bd9fa1f661dc1050af5fb29848a652498d5fc1cf346e1983`.

### PR #8538 · native · 8538-shape-hwpx-2022 · p6

기존 고정 좌표 검사에 대응하는 완료 페이지

HWPX 저장본의 본문 아래 각주와 작은 번호·보통 번호 두 모드를 각각 독립 Print와 대조했다. 본문 마지막 줄·구분선·각주 번호 끝·각주 두 줄의 순서와 기준선이 대응한다. raw 100.0%.

- [review](png/pr8538/native/8538-shape-hwpx-2022/review_006.png), SHA-256 `051eaed8bf185f73df035870cd0ce99e797f8ec1a0a4834381d33f764af948df`.
- [overlay](png/pr8538/native/8538-shape-hwpx-2022/overlay_006.png), SHA-256 `53830a834dcc9f4c42741bd4443bb5e4372d82e01782aaeb6c458d595e7225db`.

### PR #8538 · wasm · 8538-shape-hwpx-2022 · p6

기존 고정 좌표 검사에 대응하는 완료 페이지

HWPX 저장본의 본문 아래 각주와 작은 번호·보통 번호 두 모드를 각각 독립 Print와 대조했다. 본문 마지막 줄·구분선·각주 번호 끝·각주 두 줄의 순서와 기준선이 대응한다. raw 100.0%.

- [review](png/pr8538/wasm/8538-shape-hwpx-2022/review_006.png), SHA-256 `637cd346dafa151245939aef18f958685c3187cf506c10d43992b4bc23b981b7`.
- [overlay](png/pr8538/wasm/8538-shape-hwpx-2022/overlay_006.png), SHA-256 `1ffd28a01369c55861760ca2de8e510569fcf0070dc8415304849d7f78a10ca8`.

### PR #8527 · native · 8527-levels-hwpx-2022 · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

빈 번호 머리를 가진 개요9·10 본문 시작이 독립 Print와 대응한다. 상위 가시 마커7개와 아래 번호1/8/9/10 문단의 순서·간격이 보존된다. raw 100.0%.

- [review](png/pr8527/native/8527-levels-hwpx-2022/review_001.png), SHA-256 `1878dcdcb8014b8c593be42677bf2b1aee2030f3c3a5af72d9ff1c9f68da0416`.
- [overlay](png/pr8527/native/8527-levels-hwpx-2022/overlay_001.png), SHA-256 `11daad41c7329f77bba7c0a31a8e131ac44b659e69e39ad5ff7a1f966ab156b8`.

### PR #8527 · wasm · 8527-levels-hwpx-2022 · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

빈 번호 머리를 가진 개요9·10 본문 시작이 독립 Print와 대응한다. 상위 가시 마커7개와 아래 번호1/8/9/10 문단의 순서·간격이 보존된다. raw 100.0%.

- [review](png/pr8527/wasm/8527-levels-hwpx-2022/review_001.png), SHA-256 `b441d4641fb2f5ada843634b41c3bcaf199b31d99cb1fceba7baa883e39d797d`.
- [overlay](png/pr8527/wasm/8527-levels-hwpx-2022/overlay_001.png), SHA-256 `e4d1998f4ce35f25fd81741cea7d0b9579ecf8a6e9d30a804b108b4d51571486`.

### PR #8570 · native · issue7344-first-cell-no-ls-before · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

상단 결재 표·뒤 본문·중첩 보고 표·하단 붙임 내용의 순서·셀 포함·저장 줄과 탭의 원점을 독립 Print와 대조했다. 셀 앞 간격 보정 뒤 문서 전쪽 배치가 보존된다. raw 100.0%.

- [review](png/pr8570/native/issue7344-first-cell-no-ls-before/review_001.png), SHA-256 `bbd0cc17b6eb752fe1f3d2461adb2c5584245acc89a24bb7bee2f36e57c2545e`.
- [overlay](png/pr8570/native/issue7344-first-cell-no-ls-before/overlay_001.png), SHA-256 `05b90712afd652232461e7f05739f15a02096ee287e768073c49978167537241`.

### PR #8570 · wasm · issue7344-first-cell-no-ls-before · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

상단 결재 표·뒤 본문·중첩 보고 표·하단 붙임 내용의 순서·셀 포함·저장 줄과 탭의 원점을 독립 Print와 대조했다. 셀 앞 간격 보정 뒤 문서 전쪽 배치가 보존된다. raw 99.99788%.

- [review](png/pr8570/wasm/issue7344-first-cell-no-ls-before/review_001.png), SHA-256 `6e5d7ab2515093552ab99ab8ed35ee7ab1fbd37038a0e8549cd7b14f92f078cd`.
- [overlay](png/pr8570/wasm/issue7344-first-cell-no-ls-before/overlay_001.png), SHA-256 `cc6ed2f4b0ba882b26c853f80584bbfe17b900ce6eda499e7e101edd469eff7a`.

### PR #8570 · native · issue7344-stored-fields-tabs · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

상단 결재 표·뒤 본문·중첩 보고 표·하단 붙임 내용의 순서·셀 포함·저장 줄과 탭의 원점을 독립 Print와 대조했다. 셀 앞 간격 보정 뒤 문서 전쪽 배치가 보존된다. raw 100.0%.

- [review](png/pr8570/native/issue7344-stored-fields-tabs/review_001.png), SHA-256 `a7c00a50c220037c5f28f1dd1546b2aeaf028eb8c6928969c663717a1b903081`.
- [overlay](png/pr8570/native/issue7344-stored-fields-tabs/overlay_001.png), SHA-256 `626d1a6a03347e975bc7bef046e347c3774ce47b1063e2857c5dd5e9d9799412`.

### PR #8570 · wasm · issue7344-stored-fields-tabs · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

상단 결재 표·뒤 본문·중첩 보고 표·하단 붙임 내용의 순서·셀 포함·저장 줄과 탭의 원점을 독립 Print와 대조했다. 셀 앞 간격 보정 뒤 문서 전쪽 배치가 보존된다. raw 100.0%.

- [review](png/pr8570/wasm/issue7344-stored-fields-tabs/review_001.png), SHA-256 `91b9eec50700d0ed6c9771b8a3ef8f325586fa69c234ca10462cbe3e097a13f5`.
- [overlay](png/pr8570/wasm/issue7344-stored-fields-tabs/overlay_001.png), SHA-256 `eaa4d8ffc74adf1fefbe96937ceb71cc51b27eac25274a2460b6bf77e48ac1d6`.

### PR #8570 · native · issue7344-broken-cell-ladder · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

상단 결재 표·뒤 본문·중첩 보고 표·하단 붙임 내용의 순서·셀 포함·저장 줄과 탭의 원점을 독립 Print와 대조했다. 셀 앞 간격 보정 뒤 문서 전쪽 배치가 보존된다. raw 99.86918%.

- [review](png/pr8570/native/issue7344-broken-cell-ladder/review_001.png), SHA-256 `fdb54d6d782f8b537c6fb45aef576e127b415785dc4e5103810331773a88db7b`.
- [overlay](png/pr8570/native/issue7344-broken-cell-ladder/overlay_001.png), SHA-256 `2e516baa77c9d10c8bddaa9e29f0a32eaf2ae0f685c51f5b792cef4591a22288`.

### PR #8570 · wasm · issue7344-broken-cell-ladder · p1

기존 고정 좌표 검사에 대응하는 완료 페이지

상단 결재 표·뒤 본문·중첩 보고 표·하단 붙임 내용의 순서·셀 포함·저장 줄과 탭의 원점을 독립 Print와 대조했다. 셀 앞 간격 보정 뒤 문서 전쪽 배치가 보존된다. raw 100.0%.

- [review](png/pr8570/wasm/issue7344-broken-cell-ladder/review_001.png), SHA-256 `daab5f48a99faef3951679ec0a829e1c9c06de9fc9b398b6215eb402d4b99fda`.
- [overlay](png/pr8570/wasm/issue7344-broken-cell-ladder/overlay_001.png), SHA-256 `b6a5180e09e27a1563ae81433fe4df9d6dc4acc204a74f07b4bcc7249943e271`.
