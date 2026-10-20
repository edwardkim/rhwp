# 정렬 기준 출력 충돌의 독립 진단

Issue: #7642

한컴 Office 2020의 기존 Print와 새 구조 대조군에서 동일 수식의 열 정렬이 달라 원인 후보를 대조했다. 실제 출력은 `11.0.0.9136`, `PrintMethod=0` (1-up)이다. 원본 기준 PDF를 변경하거나 이번 결과로 원본의 시각 통과를 선언하지 않는다. 정식 회귀 fixture가 아니다.

## 관측

- 기존 구조/카탈로그 HWP를 정상 HWP로 다시 저장한 뒤 Print해도 기존 로컬 행 정렬이 재현됐다.
- 구조 입력의 첫 equation ID만 18210000에서 18410000으로 바꾸어도 정렬은 바뀌지 않았다. XML의 다른 부분과 archive의 다른 entry는 보존했다.
- 첫 식·라벨·스타일·ID를 유지하고 두 번째 식을 & 있음/& 없음/빈 EQALIGN/중첩 행렬로 바꾼 네 문서는 첫 식의 공통 열을 모두 같은 위치로 출력했다. 이 문맥 후보로 기존 파일의 로컬 정렬을 설명할 수 없다.
- 기존 단어 식의 Print 등호 x는 90.655/160.685 CSS96, 네 새 문서에서는 160.365/160.365다. 값은 진단 관측이며 구현 조건·회귀 좌표로 사용하지 않는다.
- source/code/문서 간 원인은 아직 미확인이다. 파일 ID·타입·폭 수치로 조건을 추가해 상충하는 출력을 흉내 내지 않는다.

## 실행 job

### resave

- `structural-resaved-hancom-2020.hwp`: `7668a8b4-a084-495e-bdde-af7b596d11f7`, 입력 SHA `9cb3eef2652d2caf9c786c7c4cc69ecdb1b1621eb0d749ca8c89e3fbd93cf3fd`
- `structural-resaved-hancom-2020.pdf`: `cc713186-cdbc-4281-8bb5-513d288ca38b`, 입력 SHA `af0f0c33d6fbb37d8e49d91e8d8a3591688e5f3fcf4beb8a152cd868961c4d7e`
- `catalog-resaved-hancom-2020.hwp`: `4f80d7f8-1935-4a75-b80a-1cc88c56d83c`, 입력 SHA `5821913b3311590a0a724f93a8f0c5d84966c06bbf42cdd3d8e30fe5664ad3b5`
- `catalog-resaved-hancom-2020.pdf`: `e1ab7b48-0e61-4792-a0fe-57e85beed55a`, 입력 SHA `028d5b5f4e316b5f1506900bc35f6394c3d77e72b168ef7ca01a60aa20e8c1de`

### instance-id

- `instance-id-hancom-2020.hwp`: `697bf3f9-2958-423c-8cb5-96171bbca7e0`, 입력 SHA `99d2819305eecbe407bb143676274b9ac73a005f8e9322942452fad5785a1bd9`
- `instance-id-hancom-2020.pdf`: `980c6d67-1f95-4e06-8ac4-d4496027171c`, 입력 SHA `6ad9d80a8d9f03145a611df25db0fa78021442dccc7d78b20b051560d812bdbc`

### marked

- `alignment-context-marked-hancom-2020.hwp`: `17e79a56-6a07-4e97-a8ce-6220b8c69dd7`, 입력 SHA `045f76df6c794c4c627af24866410805aadde5ea2db1f03e2de4ee0db5fd6798`
- `alignment-context-marked-hancom-2020.pdf`: `940417dd-3435-4bef-95ef-c616d6854d75`, 입력 SHA `b5eb99e8c3928a6b69af2fb39edb9a1827a1501f28e6b929227483f7463495b7`

### unmarked

- `alignment-context-unmarked-hancom-2020.hwp`: `75b8137b-96c3-4f79-9d41-b982f24af500`, 입력 SHA `26e79a9a70cb5331d719cb97ba40e8bcc39be7b68f3048c3dfbcd26312515ea8`
- `alignment-context-unmarked-hancom-2020.pdf`: `9c421819-afb4-4ac3-86a8-5f835ca4701f`, 입력 SHA `e058b6dd6c5eef9a871f1b903afa1a1ceb6ec20cb81c67435248ed3a91337804`

### empty

- `alignment-context-empty-hancom-2020.hwp`: `a1491656-c989-478e-a6fb-4a87f789ed96`, 입력 SHA `15d6c1ab7e174e5d30314150ad65d1575f18183c5658a1987235953d1c9b5cee`
- `alignment-context-empty-hancom-2020.pdf`: `f3f4251a-e601-4cc5-95b6-3a3e4d1a0ebf`, 입력 SHA `9f0818e4332d3964e500d31e9486003dc37e1738ee185118ef0938e9855a32b0`

### nested_matrix

- `alignment-context-nested_matrix-hancom-2020.hwp`: `535dada7-8b29-4faa-adde-9aef3f890716`, 입력 SHA `7b9186d5bbda97286e5076cf22f8297860938be40a20994cdc2649be1d75474e`
- `alignment-context-nested_matrix-hancom-2020.pdf`: `b58fa2ca-8b80-42c3-84fb-17d6215ef55d`, 입력 SHA `cc03b149781843e518da2bb1a69e301d8ef68a02fa4d09349c131d5e1f231b56`

## 파일 SHA-256

- `alignment-context-empty-hancom-2020.hwp`: `9f0818e4332d3964e500d31e9486003dc37e1738ee185118ef0938e9855a32b0`
- `alignment-context-empty-hancom-2020.pdf`: `7277cb7f30b39846a80f978ca4f87eea69eb320ee47f2f4eb73ea1a29ef9ae07`
- `alignment-context-empty-input.hwpx`: `15d6c1ab7e174e5d30314150ad65d1575f18183c5658a1987235953d1c9b5cee`
- `alignment-context-marked-hancom-2020.hwp`: `b5eb99e8c3928a6b69af2fb39edb9a1827a1501f28e6b929227483f7463495b7`
- `alignment-context-marked-hancom-2020.pdf`: `a55a1c4fcb023ac9a37201da277291dd44d5eac9dfa4f759c7b5fcbcb967bd6f`
- `alignment-context-marked-input.hwpx`: `045f76df6c794c4c627af24866410805aadde5ea2db1f03e2de4ee0db5fd6798`
- `alignment-context-nested_matrix-hancom-2020.hwp`: `cc03b149781843e518da2bb1a69e301d8ef68a02fa4d09349c131d5e1f231b56`
- `alignment-context-nested_matrix-hancom-2020.pdf`: `44a9467e8ce22924487681cdfc5057d6d8cb2cd731e6abcab71d90a51837e1a6`
- `alignment-context-nested_matrix-input.hwpx`: `7b9186d5bbda97286e5076cf22f8297860938be40a20994cdc2649be1d75474e`
- `alignment-context-unmarked-hancom-2020.hwp`: `e058b6dd6c5eef9a871f1b903afa1a1ceb6ec20cb81c67435248ed3a91337804`
- `alignment-context-unmarked-hancom-2020.pdf`: `a06d03702636b005369b92f733c5adbc76488dab262bb5bac0327593b5e90a98`
- `alignment-context-unmarked-input.hwpx`: `26e79a9a70cb5331d719cb97ba40e8bcc39be7b68f3048c3dfbcd26312515ea8`
- `catalog-resaved-hancom-2020.hwp`: `028d5b5f4e316b5f1506900bc35f6394c3d77e72b168ef7ca01a60aa20e8c1de`
- `catalog-resaved-hancom-2020.pdf`: `cc71f764abe130b9fbf50d043edc5652b41d990f2f24f174c9a6992776af47bf`
- `instance-id-hancom-2020.hwp`: `6ad9d80a8d9f03145a611df25db0fa78021442dccc7d78b20b051560d812bdbc`
- `instance-id-hancom-2020.pdf`: `c3dbba198ddbda9c6819e71a03659b1d63d43f683815700d55e989ca6f4f3776`
- `instance-id-input.hwpx`: `99d2819305eecbe407bb143676274b9ac73a005f8e9322942452fad5785a1bd9`
- `structural-resaved-hancom-2020.hwp`: `af0f0c33d6fbb37d8e49d91e8d8a3591688e5f3fcf4beb8a152cd868961c4d7e`
- `structural-resaved-hancom-2020.pdf`: `e0abc2b656cfa6e11603a39f26ca6f277595ca4f272751d223d4da19fac3cc9e`

## Stage 186 — 인자 없는 MATRIX의 후속 문맥 영향

첫 EQALIGN과 namespace·문단·라벨·ID·스타일·구역 및 다른 ZIP entry 바이트를 유지하고 다른 수식의 script만 `x`로 바꿔 정상 저장·Print했다. XML을 재직렬화하지 않은 대조군에서도 후속 bare `MATRIX` 하나를 남기면 첫 수식의 열 정렬이 바뀐다. 이 관측을 RHWP의 문서별 우회 조건이나 회귀 기대값으로 사용하지 않는다. 원본 30쪽과 18쪽의 실패 증거는 유지한다.

| 대조군 | 첫 행/둘째 행 = 원점(CSS, 관측) | HWP job | Print job |
|---|---|---|---|
| all_x | [160.365, 160.365] | `22f67c30-98e4-41f6-829e-9867620ecbfc` | `bd2764e5-202d-4efc-928d-55a2ffb722d3` |
| first_half | [90.655, 160.685] | `1838c66b-5297-4e1e-98e5-ed4cada9d29e` | `11cb3a5a-b641-4dbf-b9c0-6d593851caa2` |
| second_half | [160.365, 160.365] | `bcb39110-c184-4062-b3e7-aabc799d59f7` | `0761d33c-3b41-4d0e-83e3-0e35ba790ffb` |
| empty_commands | [90.655, 160.685] | `56dc6d6a-9070-464d-a015-fc60909ac097` | `8f678ada-c635-4d3d-bfda-f8d3fe5a5ae5` |
| bare_matrix | [90.655, 160.685] | `297b5b3d-ba54-4383-8ccd-432421ba1a02` | `12d26675-da59-44f8-ab0f-9d544d247089` |
| bare_pile | [160.365, 160.365] | `bed10196-fa8d-4841-a528-01bd8ec4bedd` | `98d4dd4a-5c95-4e06-a12a-2b0f94436837` |
| two_bare | [90.655, 160.685] | `256c3bac-ff05-4a4b-922d-220c286e2c10` | `355451b9-7f56-41d2-a5cb-9972d2cd7cae` |

`bare_matrix`와 `two_bare`는 원본의 개별 행 원점을 재현했고, `all_x`·`second_half`·`bare_pile`는 공유 열 원점을 출력했다. 실제 원인은 서비스/한컴의 파서·출력 상태까지 추가 확인이 필요하다. 자료가 충돌하는 범위를 정상 출력 일치로 보고하지 않는다.

### 파일 해시

- `stage186-all_x/all_x-hancom-2020.hwp` SHA-256 `8f250362f57dbdd777ff3888ca57994f9525069f631394bc6fa045a5b461cbf5`.
- `stage186-all_x/all_x-hancom-2020.pdf` SHA-256 `9e6856488810459c2f38f1cc9497e82d123dc70b70d4096c23ab88a7c5045f47`.
- `stage186-all_x/all_x-input.hwpx` SHA-256 `25851690a3bbced1214fee2578785b4fc0ce6046ee643b5ce55f1a4f0714f9e3`.
- `stage186-first_half/first_half-hancom-2020.hwp` SHA-256 `f5b6a52d0838b3fabfee7790a58f603e647946f9818f6b908992df6e5de74a61`.
- `stage186-first_half/first_half-hancom-2020.pdf` SHA-256 `fb138da0ee3928fcb8ec539fe371af731db28c6f94891a8c22d531e454a1dcba`.
- `stage186-first_half/first_half-input.hwpx` SHA-256 `c73b197907c46bf0b537975948f01174a9722e40e9e4a052c29a59c8e14591db`.
- `stage186-second_half/second_half-hancom-2020.hwp` SHA-256 `51263d83ddbddb8d2c73082fad61b7b52bb98fcf80178fe605e45ad7a7fe394d`.
- `stage186-second_half/second_half-hancom-2020.pdf` SHA-256 `a361755549ac0bb1c02fb8dc71cbfb4bc5bccfab2c2bdfe2e61a7971f92538f8`.
- `stage186-second_half/second_half-input.hwpx` SHA-256 `209d3738985149bff726cef8f72e594c589f3a6172d1310adf5b2ec2d6942720`.
- `stage186-empty_commands/empty_commands-hancom-2020.hwp` SHA-256 `9218c2d9c4bbbcc0e4cd845d15057eccc9d133afe2b687e333378e5b515e7931`.
- `stage186-empty_commands/empty_commands-hancom-2020.pdf` SHA-256 `4d4e99dc4418a489dd9abf6a621239231d0d968513785196e21141774bed4ce3`.
- `stage186-empty_commands/empty_commands-input.hwpx` SHA-256 `f2bd34b5adabd094b85fa1245674cddf20ea6e79799d588e6b836a0a85715390`.
- `stage186-bare_matrix/bare_matrix-hancom-2020.hwp` SHA-256 `b077b062c89c59a3e854341a0a43ae53ce2cff61adf8fa6ef331519460c8f02d`.
- `stage186-bare_matrix/bare_matrix-hancom-2020.pdf` SHA-256 `5253f6c36058f566da70dee76ace47962a09699d40dd74f506ae37f16b33d188`.
- `stage186-bare_matrix/bare_matrix-input.hwpx` SHA-256 `552a3e712f4f7d81d55c76b3c49a61484eb0c6ebd90539560ecd1b577b990f4c`.
- `stage186-bare_pile/bare_pile-hancom-2020.hwp` SHA-256 `a83a1dd765bf672af879b1231fa3b06c8e019675dffa267f57c513a3a91a2d6c`.
- `stage186-bare_pile/bare_pile-hancom-2020.pdf` SHA-256 `e15f719fac5fc78c39768b3faf5da3f018248267b00f87a8ee47ac81b2580be4`.
- `stage186-bare_pile/bare_pile-input.hwpx` SHA-256 `5a1ad5b3339a7055c458ab4a6fa6fe424e539424d08d14bd25c274833847f9ab`.
- `stage186-two_bare/two_bare-hancom-2020.hwp` SHA-256 `b2c868def6a195d1260e7f350b8de82539962d7a4993b3d0957b34d8ad7e38f3`.
- `stage186-two_bare/two_bare-hancom-2020.pdf` SHA-256 `1de7feff26a2d93a6a0d4b606ffb1d4ce7917ecee250d54484668ff96b6f7cdd`.
- `stage186-two_bare/two_bare-input.hwpx` SHA-256 `4a854e49286ed6dd5a99ead1f4e0d78457bd1b1240df23273b173d9a0ad3a87e`.
