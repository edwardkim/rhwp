# PR #9063 검증 영수증과 재현 조건

원시 빌드 로그·MCP 응답·분석 JSON은 임시 산출물이다. 최종 TSV·입력·기준 PDF·대표 비교 PNG를 보존하고 실행 사실과 신원을 이 문서에 요약한다.

## 보정 전 접수 검증

- 원 head: `b43050954c10512705d714114f5af23f1ee98ac6`.
- Native 검증 source: `bc8e562aebc9e4f2f8c170728d84bb468a869268`, tree `2d5f2f6a4b5625cd033f7a78b63eaf73c5834512`, CLI SHA-256 `fb7b83b48a0aa94645199fad8dbf0ddcc6dc72d28f7deb01aea2ae570484e28d`.
- Native build: `cargo build --locked --profile release-test --features native-skia --bin rhwp`, exit 0.
- 최적화 WASM source: `9a1f85f3b79fcbc777f86b70000c08369f29c01b`. Rust `1.93.1 (01f6ddf75 2026-02-11)`, wasm-pack `0.15.0`, jobs 8, release codegen units 16.
- Docker locked wrapper·최적화 build exit `0`. 공용 target은 `/home/edward/mygithub/rhwp/target/pr-review`다.
- 원 공개 입력 17문서·22쪽: Native/fresh 최적화 WASM 각각 전쪽 raw 100.00%. `native/`·`wasm/` TSV를 따른다.
- 최소 글꼴 집중 회귀 12개 PASS. 공식 자동 수집 suite 2개를 사용했다.
- 브라우저: Chromium `152.0.7946.0`, 고정 build `1660786`.
- 원 head의 CI·CodeQL·Render Diff·Adapter·Proptest·정책 감사 성공은 보정 후 제품 검증과 구분한다. 원격 실행 ID는 검토 기록을 따른다.

## 같은 글꼴 공급

- 명시 환경 ID: `native-webfont-0e5b38844ae4b38d55cb70d7`. `HCR Dotum` → `HCRDotum`, `함초롬바탕` → `HCRBatang`, 원본 메트릭 보존.
- TrueType face: `HCR Batang`, `HCR Dotum`. 글꼴 프로그램은 로컬 검증에만 사용하며 커밋하지 않는다.

| 공개 source | source SHA-256 | 공급 Native 프로그램 SHA-256 | bytes |
| --- | --- | --- | --- |
| https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_four@1.0/HCRDotum.woff | `49bf312aec309ba88c3eb612c591c2d8e28b765c156c808691850137431bb1bf` | `9d0ccb452d2d1b3f585b0179a52be072db95b2a05cd9e47f9be76e594a15fe57` | 22244940 |
| https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff | `b3192018e3bd4da76b29b0530f71c79fb6f465a2b7cd41308bc38f9a4ae09a6c` | `2d9a6fc7545efc7eef039c1134ebead1c981bec47b5caafe3239e28b50de09ee` | 28413328 |

- 첫 WASM 진단의 Noto Serif KR 선택은 최종 증거에서 제외했다. 최종 22쪽은 위 HCR 프로그램을 명시하고 실제 loaded SHA 일치를 확인했다.

## 독립 Desktop Print

- 서비스 engine 2020, 실제 한컴 `11.0.0.9136`, Print method `0`, mode `hancom2020_pdf_driver_one_up`.
- 각 요청은 start → status의 succeeded → download 순서로 실행했다. 내려받은 크기·SHA는 서버 반환값과 일치한다.
- 입력 저장 메타데이터: 공개 원본 17개는 HWP5, 한컴오피스 2010 `8.0.0.466`. 합성 입력의 실제 작성 프로그램을 이 메타데이터로 추정하지 않았다.

| 입력 | input SHA-256 | 비동기 job | PDF | PDF SHA-256 | bytes |
| --- | --- | --- | --- | --- | --- |
| after_1fit_n1.hwp | `d769fb93a9d691ba3896b48ee80da62f9d2bf57033358d518ce0efeb9c2fa603` | `4c5aa585-c67c-42c8-b212-9c9ee8db90e0` | `pdf/tac_space_tail/after_1fit_n1-hwp-2020.pdf` | `1ada5ebc2041236e6740eee39408d0ecac57c045c35b4c5dd42ee5049b021d46` | 13264 |
| after_1fit_n2.hwp | `f499ca0b9b8e7154e2532d0e4d1b55bf928fdc955198b927935d2506fe36e42e` | `ed5eb219-e5f2-4fe1-91fd-2a7d31080b01` | `pdf/tac_space_tail/after_1fit_n2-hwp-2020.pdf` | `3b64d5ce7f7ab78644c4b0d8fd8c8cb1ce2c1022da5857b2fc1bd583afd74dc1` | 13262 |
| after_1fit_n3.hwp | `ab59b9f6b6a7e7d4c242048884782fdc7f097dd26cc8e78b4c43621ff1bba610` | `01b7f6c3-6c85-4a73-af2c-93546c14c39a` | `pdf/tac_space_tail/after_1fit_n3-hwp-2020.pdf` | `732eb19afed69dc25b6c83f55ebd3abb752b9dd5204ffa00766bd48784dac3ca` | 13273 |
| after_3fit_n3.hwp | `b6785c41acb59338f5d8264fc32cc1b74f94d7e6a73a5594d07e2cb026f88348` | `9da2bb48-32f6-497b-807e-ffa536d823f3` | `pdf/tac_space_tail/after_3fit_n3-hwp-2020.pdf` | `8fa0552bc5eb924d5d88c4b7b7f1b0eec5119afd4166587d1b0bd2bb3beda322` | 13273 |
| after_3fit_n4.hwp | `310d4bee61ccbf44a54f1b79801d2268df34b8a82779df3a11ffd64e82e83f49` | `2ee18f55-bc6c-4d8d-b759-8e2b4de70d79` | `pdf/tac_space_tail/after_3fit_n4-hwp-2020.pdf` | `6addea861db6d42047477d89994b4ead7c2e423e37bb99d39efe0c2d809481dc` | 13269 |
| after_3fit_n5.hwp | `225861c384227f3bde4edcecefc1fb63ba0f987040cba12a652af3771bfd33a1` | `54e28f1f-e91e-4dbb-ad72-4d867dc17fa8` | `pdf/tac_space_tail/after_3fit_n5-hwp-2020.pdf` | `f912856f0d29a1971fce30ed7af43028b0bf31a0b93c2975def589f0dc7054f9` | 13278 |
| at_table_n0.hwp | `7172135f74a6786a2fc507e7cf7d6354eb8398ff7cf957b55efad3a91ddb36d4` | `e8f50015-ce1c-4301-9c65-5554cfcd851b` | `pdf/tac_space_tail/at_table_n0-hwp-2020.pdf` | `20c288ab035e4bc787030a9423fbd4948e70a4bb36a62430db8308aaf9839623` | 10388 |
| at_table_n2.hwp | `7674482ec23bd2ec5988979cd5cd3eb910414ab4da3399430572623873f957eb` | `8f320f60-108d-415e-ab78-2707b143d367` | `pdf/tac_space_tail/at_table_n2-hwp-2020.pdf` | `d641d7a24b3cc0511d85b0d229bcc69b3d9f217d17dbe9fdbd604181ef91797e` | 13265 |
| at_table_n3.hwp | `1077b4edf44a37b417c1f8f806ed57944bd2faa5100dc4c795653aac0c874585` | `439ee1ab-2eb3-4958-b7a7-c79d2a05abbd` | `pdf/tac_space_tail/at_table_n3-hwp-2020.pdf` | `c23eae0027eb489dbaa4d2b576cd68321bd5b2ef81573baa4c101930b5697a7f` | 13275 |
| edited-picture_in_front_over_1px.hwp | `57cffa887aaa2840b376dd174724c6e7782b92da0f755ed70f25011b21445fd3` | `27d9839a-f48b-4e02-a8b3-90309ff9a248` | `pdf/pr9063/edited-picture_in_front_over_1px-2020.pdf` | `5613022dc29a59a5211db4ef32b7e54b9676f88301cfb81cc554adcf1a762be8` | 14965 |
| edited-rect_top_and_bottom_over_12px.hwp | `3bb79f3655633b7b93e4542582cc050d1a0c2d7ae5eb6a6c9933b67a59b1bf26` | `7af7a72e-89d2-401e-b486-cf1ba81bd099` | `pdf/pr9063/edited-rect_top_and_bottom_over_12px-2020.pdf` | `fa4ba3974f951d788abfb91ed9c34e5103544c72eca30e2bc13fa0b96fc25ce9` | 12502 |
| picture_in_front_after_table_fits_6px.hwp | `46ef0370ba614d9155b25e56bfabbc5c463cd1a373e7c3aca9ad241e050212e8` | `243c4703-8bb2-4a84-b502-9dd49fa69bf2` | `pdf/issue8660/picture_in_front_after_table_fits_6px-hwp-2020.pdf` | `92d1ce3948e59128908e8b1e00627c87a59cabf3a08647e275dadb3fe8068eb2` | 14850 |
| picture_in_front_after_table_over_6px.hwp | `f0981a22db4b36f8211a819b452569f7d7763464f054f06912438a6443455d90` | `61519b25-6632-4d0a-8b04-f005799883e7` | `pdf/issue8660/picture_in_front_after_table_over_6px-hwp-2020.pdf` | `3654ca51740a4804d95c896d4983ef9ef8053afc6b3d115e218b976c337d18ab` | 15285 |
| picture_in_front_fits.hwp | `265cad4671b3b88fc50fe5733dec624bb8b9a479033bf56bc5b759351b8f9861` | `53ef524a-a861-4176-9751-fb7de3c3793a` | `pdf/issue8660/picture_in_front_fits-hwp-2020.pdf` | `751ce76bad21d278af42d0ac288145b4c899e69fa7d21d5cb4959ff51b04829f` | 14299 |
| picture_in_front_over_1px.hwp | `db3478155966d97b7d2fc72d5cc17e5a889dafbf7e79789c66ab5e8601485403` | `9dfc69a3-949a-407c-a909-84bfb8f2694f` | `pdf/issue8660/picture_in_front_over_1px-hwp-2020.pdf` | `0613e0214b96afe6aab0acf7e71f21d8ea9ab4dd0e6851e4623a97ac58a3022f` | 14732 |
| picture_in_front_with_table_host_over_12px.hwp | `5c7ae93c910705f9987cb1bced9b757add800cc6d6fffbb8e994bf8799c5959e` | `34ce0cab-4612-4ff9-8b62-62673510b691` | `pdf/issue8660/picture_in_front_with_table_host_over_12px-hwp-2020.pdf` | `48b978576c5354f7759fb09d60b2d42732e3353f02df75af88f30a545a3a10da` | 14595 |
| picture_top_and_bottom_over_12px_text_follows.hwp | `7d74aa34f7b86ffa5565fd0472bd5cc24e25a23fa17dc5650bf4aa83b61725ca` | `52fac800-f6ff-447d-b701-ba4d63815ba7` | `pdf/issue8660/picture_top_and_bottom_over_12px_text_follows-hwp-2020.pdf` | `276dfe90575d7ea201952423a105d197ac417c58feae26cec5e863daceebb77f` | 15620 |
| picture_unrestricted_over_12px.hwp | `1b93ae668a83bbb7e772325628c3f2fcb1463d0071ac00f906ebc68d5bd8d4a5` | `f6a70db5-3814-4cc1-b33b-e2e4e2d32999` | `pdf/issue8660/picture_unrestricted_over_12px-hwp-2020.pdf` | `9fa09352faad09f0f65a327dda08df00ea1c75aa5dbe776d55fd8140a8530bb9` | 14298 |
| rect_top_and_bottom_over_12px.hwp | `1126e123ad9fb8bc616d5955703c4697f81681862ea74e5667fe1ad9105b9658` | `dc18f046-4e68-4ff9-b003-54c788f1d856` | `pdf/issue8660/rect_top_and_bottom_over_12px-hwp-2020.pdf` | `bf5faa4890a442f963c18e013555b9013f2c2acf15f82b73f32f36726446180d` | 12279 |

편집 파생본의 입력은 `edit-boundary/<문서>/edited.hwp`에 있다. 두 파생본의 독립 Print 2쪽은 원본의 2쪽과 96 DPI RGB가 같으며 `ANCHOR` 문단의 bbox도 같다.

## 보정 전 편집 경계

| 경로 | 그림 1px 초과·사각형 12px 초과 | fit |
| --- | --- | --- |
| Native 실제 API | 열기 2쪽 → 첫 문단 X 삽입 1쪽 → HWP 재열기 2쪽 | 1→1→1쪽 |
| 최적화 WASM 실제 브라우저 API | 2→1→2쪽 | 1→1→1쪽 |

- 그림의 원래 offset 15770HU·높이 2067HU와 사각형 offset 16595HU·높이 2067HU는 편집 전후 같다. 두 호스트의 저장 줄은 없었다.
- `ANCHOR`의 원점은 Native `(113.38667, 772.26667)`px로 같지만, 그림은 2쪽 y132.26667px에서 1쪽 y981.53333px로 돌아갔다.
- 이 진단은 저장 줄 유효성과 개체의 쪽 영역 제한을 문서 전체 edited 플래그로 함께 해제하던 공통 분기를 지목한다.
- 실제 PNG 직접 비교는 `edit-boundary/native-edit-review.png`, 진단 source는 해당 `.rs.txt`·`.mjs.txt`를 따른다.

## 현행 보완 검증

- 보정 source·최종 local gate·추가 Print 영수증은 아래 최종 검증 기록에 갱신한다. 보정 전 결과를 새 source의 통과 결과로 재사용하지 않는다.

## 보정 전 산출물 신원

| 파일 | SHA-256 | bytes | Studio public 일치 |
| --- | --- | --- | --- |
| `rhwp.js` | `2d48f8a6c3ff7f848f08725bb54d04094b296af1a40ae61203081a7cf8061a1a` | 586144 | True |
| `rhwp_bg.wasm` | `e6e08941e256e8b5e1e01d4525d2c458b90eec2b8545de328945f2c980a226fa` | 29377488 | True |

초기 검증 source와 뒤 문서 기록 사이의 `src`·`tests`·Cargo 입력·locked WASM wrapper의 Git object는 모두 같았다. 이 동일성은 보정 전 기록에만 적용한다.

| 경로 | 동일 Git object |
| --- | --- |
| `src` | `7cf8fb94e62c1d0d1bab8a2f609fa1d951f87b38` |
| `tests` | `2d0f9c49e1d05bf5518a7f2f44c10e3e607194a8` |
| `Cargo.toml` | `67ffbf80f174ba3465e4454d140f758395833b2e` |
| `Cargo.lock` | `34d08aa8d5150817968568e1ab6f68af228eeeac` |
| `scripts/wasm-pack-locked.sh` | `855ca67dcd4b175287c297d4d015851aab3cf8af` |

## 보정 source와 실제 출력

- 공통 보완 commit: `1720c96477`, 합성 줄 원점 보완: `e2d7fb08b5`.
- 정식 회귀 commit: `5b028b3f03`. 실제 WASM build source는 `e2d7fb08b5`이며 회귀·문서 변경과 제품 source의 동일성을 별도 확인한다.
- Native CLI 명령: `cargo build --locked --profile release-test --features native-skia --bin rhwp`, exit 0.
- fresh WASM: Docker locked wrapper, release/codegen-units 16/jobs 8, 최적화 포함 12m 16s, exit 0.
- 실제 JS SHA-256: `2d48f8a6c3ff7f848f08725bb54d04094b296af1a40ae61203081a7cf8061a1a`.
- 실제 WASM SHA-256: `51eb72d147f04dda8cb46a23cd8d1be8d2986a6c1a74cb2b8a92ea51c7e794ef`.
- pkg와 worktree Studio public의 두 파일 해시는 각각 같다. 실제 Studio 초기 로드도 canonical `observeWasmArtifact`로 같은 WASM을 확인했다.

| 검사 | 실제 전체 범위 | 결과 | 증거 |
| --- | --- | --- | --- |
| 원본·실제 저장 파생본 | Native/fresh WASM 각각 39개·58쪽 | 전쪽 raw 100.00%, 쪽수 일치 | `correction/native/`, `correction/wasm/` |
| Native 편집·반복 저장 | 12개 입력 × live·두 형식의 첫째/둘째 재열기 = 60상태·100쪽 | 전쪽 raw 100.00% | `correction/live-native/` |
| fresh WASM 실제 live Canvas | 12개·20쪽 | 전쪽 raw 100.00% | `correction/live-wasm/` |
| WASM HWP/HWPX 반복 저장 | 12개 × 2형식 × 2회 = 48상태 | 쪽 소유·본문 순서·내용 보존 | 아래 모델 검사 설명 |
| 실제 SVG 소비 | 6개·10쪽 | 전쪽 raw 100.00% | `correction/svg/` |
| 새 base #9042·#9066 보존 | 4개·Native/fresh WASM 각 8쪽 | 전쪽 raw 100.00% | `correction/base-preservation/` |
| 실제 Studio 키보드 | 그림·자리차지 뒤 본문·사각형·fit 4개 | `X` 실키 후 2→2 / fit 1→1쪽, 실제 Canvas2D | `correction/studio-keyboard.png` |

### 저장 모델과 참조의 일치

첫 문단 X·호스트 문단 X·실제 Enter를 구분했다. Native는 실제 편집 직후 PNG와 HWP/HWPX 첫째·둘째 저장의 재열기 PNG를 각 저장 형식의 독립 Print와 대조했다. 첫 형식 PDF를 다른 형식에 적용한 초기 계산은 제외했다. 독립 PDF의 TEXT 잉크 bbox 이동은 TAC 테스트 주석에 기록했다.

WASM은 실제 브라우저의 portable 조판·Print Canvas를 실행하고 같은 문서에서 HWP/HWPX를 각각 두 번 저장·다시 열었다. 전체 렌더 트리의 본문 문자열을 순서대로 결합해 순서·누락·중복을 대조하고 각 개체의 단일 쪽 소유를 확인했다. 12개·48상태에서 모두 보존했다. 첫 1회 입력이 아니라 두 번째 직렬화 결과를 다시 연 상태까지 검사했다.

WASM의 HWPX 변환 경로에서 생성한 저장 바이트가 Native와 다른 경우는 실제 WASM 저장 파일도 별도 Print했다(`correction-inputs/wasm-hwpx-edit.*`). 서로 다른 바이트를 동일 파일이라고 기록하지 않았다. 두 입력 모두 Native/fresh WASM 전쪽 raw 100.00%다.

Studio 검사는 격리한 후보 Vite와 headless Chromium의 실제 앱 열기 이벤트·숨은 편집 textarea·실제 키보드 입력·Canvas2D를 사용했다. 진단 API 호출만의 브라우저 검사와 구분한다. 첫 리소스 관측의 Chrome inspector cache 부족은 canonical 관측 helper로 보정했고 실제 로드한 WASM SHA를 확인했다. 보안·제품 설정을 바꾸지 않았다.

### 직접 판독

`correction/native-detail.png`와 `correction/wasm-detail.png`는 같은 96 DPI 영역을 같은 2배 최근접 배율로 확대한 비교다. 바깥 라벨 외에 원문 픽셀을 이동·왜곡·지우지 않았다. 그림 윤곽·사각형 선·그림 아래 AFTER10/AFTER11의 기준선·간격을 확인했다. `correction/native-page1.png`에서 ANCHOR와 AFTER00~09가 첫 쪽에 남고 누락·겹침 없이 이어짐을 확인했다. 이 샘플에는 수식이 없어 수식 검증을 주장하지 않는다.

### 검사 환경의 실행 오류와 복구

- 첫 워크트리 브라우저 실행은 `puppeteer-core` 연결이 없어 실제 capture 전에 실패했다. 기본 작업공간의 기존 node_modules를 ignored 링크로 연결하고 실패한 대상만 다시 실행했다. 새 설치·제품 보정·점수 완화는 없다.
- 최소 글꼴 wrapper의 첫 Node 호출은 숨겨진 NVM 경로로 인해 실행되지 않았다. 해당 Node 24.15.0 runtime만 read-only로 연결했고 같은 wrapper의 실제 공개 글꼴 5종을 다시 확인했다.
- Native 편람의 실제 자식 명령은 exit 0과 passed 결과를 만들었으나 긴 감독 셸은 exit 143을 반환했다. 이를 exit 0으로 바꾸지 않았다. canonical `visual_sweep_handbook.compare`로 실제 383쪽 manifest·해시·세대·Print·raw 보호값을 독립 재감사해 exit 0, passed, 하락 0을 확인했다.
- 반복 저장 probe의 일시적 ANCHOR 문자열 가정은 표 호스트의 실제 다른 라벨을 놓쳤다. 전체 본문이 비어 있지 않은지 확인하고 실제 본문 문자열·소유를 대조하도록 고쳐 해당 한 사례를 다시 실행했다. 제품 결함이나 PASS로 숨기지 않았다.
- Skia Clippy의 첫 호출은 exit 101이었다. bwrap 안의 `/etc/resolv.conf`가 가리키는 WSL `/mnt/wsl/resolv.conf`가 없어 의존 도구의 DNS 조회가 실패했다. host와 wrapper의 `getent hosts github.com`을 대조하고 DNS 대상 파일만 read-only로 연결했다. 같은 명령의 실제 재실행 결과는 최종 Rust 표에 구분한다. 이미 통과한 검사를 반복하거나 제품 코드·검사 기준을 바꾸지 않았다.

## 보정 검증의 독립 Print 원장

아래 입력은 공개 합성 원본의 실제 편집·저장 파생본이다. 각 비동기 작업은 succeeded/result success, engine 2020, 한컴 11.0.0.9136, Print method 0 및 one-up을 확인했다. 내려받은 파일의 크기·SHA는 서버 반환과 같다.

| 입력 | input SHA-256 | job | PDF | PDF SHA-256 | bytes |
| --- | --- | --- | --- | --- | --- |
| `mydocs/pr/assets/pr9063/correction-inputs/host-edit.hwpx` | `e8d5df9a4654d03339fcdc7af372a2cbe630b8962f3407f6317d6b03f536ccd2` | `4ac64079-018f-4ab4-9638-9f3dbbd322e2` | `pdf/pr9063/correction-host-edit-hwpx-2020.pdf` | `acb4c42f81c9eb577ec74d53f8a3a212369ec6a97169ac0f8195fbbdb51efe8b` | 14954 |
| `mydocs/pr/assets/pr9063/correction-inputs/host-edit.hwp` | `48157dbbd03d7af410810d8e8f6cf9b4fbd3e7dcf4756cb056d023687f514eb9` | `49f09e7d-b18a-4ebd-8b89-f931e6b84fc5` | `pdf/pr9063/correction-host-edit-2020.pdf` | `31e66ecaa1252aee0ba73237c1c73bc03bbf02d2b342ef57bd75e3cae442f10a` | 14954 |
| `mydocs/pr/assets/pr9063/correction-inputs/host-spacing.hwpx` | `3b89f492ea528ca7bab5120371ff8f624bef78bd345d106cbe46c766668c9828` | `438cef78-87fe-411b-bcb1-d9f9e69319df` | `pdf/pr9063/correction-host-spacing-hwpx-2020.pdf` | `9211b70ae91dfb1f0b88b25856c4d5277e79ae2886a76cb9ef0ec0084d4eacda` | 14532 |
| `mydocs/pr/assets/pr9063/correction-inputs/host-spacing.hwp` | `c7da4aa30466b2d39fda127a767231488abc2a7bbf695bbb1f7004942a9f75ed` | `d316b59b-7337-44dd-b16c-ddd8c9bf56ca` | `pdf/pr9063/correction-host-spacing-2020.pdf` | `262d3940fb0807a89ed1d8dba89a5fda30a76f1ddd246223f003a9c063dac632` | 14532 |
| `mydocs/pr/assets/pr9063/correction-inputs/hwpx-edit.hwpx` | `f952fe45712f70881dc91e1c74cb4daae5f321ca0a4207555932c4439b02d90a` | `03e18feb-91df-416e-bc26-177f5cf04338` | `pdf/pr9063/correction-hwpx-edit-hwpx-2020.pdf` | `56e0edacb6d908738535eca53bb054832aeec01c810bac26ab3e687af7fdb601` | 15850 |
| `mydocs/pr/assets/pr9063/correction-inputs/hwpx-edit.hwp` | `72435f223369d805f84f628cb74e031eab0be6d9471a756e61556883b15b28c2` | `7eecb8ad-b456-4cb0-b99f-baa70535e554` | `pdf/pr9063/correction-hwpx-edit-2020.pdf` | `80ef42b6d3d5070501b6cb7f5718b56476ce4d9f6640e1e5cfd9ae82ce0c65b4` | 13309 |
| `mydocs/pr/assets/pr9063/correction-inputs/picture_in_front_after_table_fits_6px.hwpx` | `7970bf751a1db02435a1b0b8ea6ec80efeb6d5aec8cb063d52aad136366cc01a` | `05871eea-6722-4402-a4d8-44dcebec515d` | `pdf/pr9063/correction-picture_in_front_after_table_fits_6px-hwpx-2020.pdf` | `6bf9d9643d5366d7031482f23a55d1fe3d4aaa2a92aca0a21c69fa0de6f67194` | 15058 |
| `mydocs/pr/assets/pr9063/correction-inputs/picture_in_front_after_table_fits_6px.hwp` | `41e6207ff370046e2eab34794ebe93567b23ef8914764d12b384c594cb2aa87f` | `f0b6f58e-b947-4e43-8158-440c63ef894b` | `pdf/pr9063/correction-picture_in_front_after_table_fits_6px-2020.pdf` | `c176e94966d1bf704ee1ad6bb95871c611f6215ba6ada8d41a316ccbc04c825c` | 15058 |
| `mydocs/pr/assets/pr9063/correction-inputs/picture_in_front_after_table_over_6px.hwpx` | `a1ef3c859a46d1aad64ab7202ff1a2c9f3f747afba500f9d61379d6e2e3a849b` | `fff6d741-aa50-4d4e-b1c2-a3f1ba69d216` | `pdf/pr9063/correction-picture_in_front_after_table_over_6px-hwpx-2020.pdf` | `68a5e614c3af410f97800f3970513be4ba88433d765900159325ff97cd7b42af` | 15510 |
| `mydocs/pr/assets/pr9063/correction-inputs/picture_in_front_after_table_over_6px.hwp` | `3b2e0069685c76cf3c1d3d502a5ba1a1735439b631ddc53f6ab467775c63f93e` | `519d46b9-1c8d-4a01-b273-e4d6c6075ec9` | `pdf/pr9063/correction-picture_in_front_after_table_over_6px-2020.pdf` | `d6a8d931cb640f1986df1101e3bb39f94d19958fb2fc8f1e285c70c830e4fbc4` | 15510 |
| `mydocs/pr/assets/pr9063/correction-inputs/picture_in_front_fits.hwpx` | `c16297227c27a8284818124fa5f55edfd2ec76e8553b87ca8679fd1c1f540df3` | `52f9b98f-cc55-42d6-9a2c-0b384bbfe77b` | `pdf/pr9063/correction-picture_in_front_fits-hwpx-2020.pdf` | `c8e9c6a35f0afbef9f53a84adf8048b9997f66a2446002b65afbf9e1d8d5f5ff` | 14530 |
| `mydocs/pr/assets/pr9063/correction-inputs/picture_in_front_fits.hwp` | `aca89f7ce1d60a7e8b6510840dfa14f42d8fa1a0bdb7c1553a02b909b92f7677` | `446310d0-ce7e-4535-b15c-bd3a25b9402b` | `pdf/pr9063/correction-picture_in_front_fits-2020.pdf` | `faa62193a1b0bcd9943bd6db689194bfa01822f4ab6951a18f6e3bb78ffdd935` | 14530 |
| `mydocs/pr/assets/pr9063/correction-inputs/picture_in_front_with_table_host_over_12px.hwpx` | `d95e15383be2ea12e7c3e237b94985a1b4efc940f7336be304696a2705f6d5db` | `8b821df2-cfe3-4108-8a02-c15d3d480d4a` | `pdf/pr9063/correction-picture_in_front_with_table_host_over_12px-hwpx-2020.pdf` | `02a5536852d012213ccdefdced47af7635e01b3744c623a424debdf301d43c17` | 14834 |
| `mydocs/pr/assets/pr9063/correction-inputs/picture_in_front_with_table_host_over_12px.hwp` | `6a53127e5e513b58e05d665406f6ca9aca1e0b7889ff5d239f44c40c6a84fe0b` | `cc8a85aa-a9ce-4843-8e86-de6c6c0ca38c` | `pdf/pr9063/correction-picture_in_front_with_table_host_over_12px-2020.pdf` | `2a015031812a6c31230d992b1710b4c56de5c790723321a52e09f64010b67752` | 14834 |
| `mydocs/pr/assets/pr9063/correction-inputs/picture_top_and_bottom_over_12px_text_follows.hwpx` | `f952fe45712f70881dc91e1c74cb4daae5f321ca0a4207555932c4439b02d90a` | `6ba90b52-27de-473c-b0b8-2cb884d479dc` | `pdf/pr9063/correction-picture_top_and_bottom_over_12px_text_follows-hwpx-2020.pdf` | `94d229e00c0cdf38c036c76e0bd35d0e8aa021e110e188394ce6d3d04a2460a1` | 15850 |
| `mydocs/pr/assets/pr9063/correction-inputs/picture_top_and_bottom_over_12px_text_follows.hwp` | `a5a9c827207fec54a25874036ac49e12385e50fa5c74704791943f470f82fd71` | `e3962b6a-84bf-4ff5-ad03-18697feed486` | `pdf/pr9063/correction-picture_top_and_bottom_over_12px_text_follows-2020.pdf` | `861d263d22dc61c2513e7368e51e4d3b2e31827e372f5bb95da0caf175cbcf41` | 15850 |
| `mydocs/pr/assets/pr9063/correction-inputs/picture_unrestricted_over_12px.hwpx` | `215a03516d5ef57d646c4a05712113999796a926b04fcb58af15307d2ea0f2ce` | `ecdc0791-4314-4ddc-8962-cefd9a722a23` | `pdf/pr9063/correction-picture_unrestricted_over_12px-hwpx-2020.pdf` | `23684390d982fce0d1dba4f67b088a4df298ef441f36aa3c14e3502f5d0ac198` | 14532 |
| `mydocs/pr/assets/pr9063/correction-inputs/picture_unrestricted_over_12px.hwp` | `ab11e02127a20b0fef119dc9eaf101460959bab8837c82bfd52115fa3d3a5bb2` | `51b99448-f103-4de7-965d-a135f0a8eea9` | `pdf/pr9063/correction-picture_unrestricted_over_12px-2020.pdf` | `4df413f90cd6da869807cba48c2ad4058729e746c182666b6f62b87ba1215170` | 14532 |
| `mydocs/pr/assets/pr9063/correction-inputs/split-edit.hwpx` | `3ae033487bce06803f4605ed60ed6c95993076ad47bc43909a686468d938dff9` | `d9b78f24-7cc2-4f4a-8803-0e6885b801d6` | `pdf/pr9063/correction-split-edit-hwpx-2020.pdf` | `b23b3dc3bf981a31ee9751079a09db6882ef4be52cdec61f70ff752ac5c20268` | 14733 |
| `mydocs/pr/assets/pr9063/correction-inputs/split-edit.hwp` | `98f309e7a26aae7985ba45cf6f75d54e11532296e79a36d2e44e2f136786ea7d` | `9dc93f2b-f7bd-4473-9ce6-cb14296c5b0a` | `pdf/pr9063/correction-split-edit-2020.pdf` | `f805872c5f184e247395131e78b0b61e9014fc30d0d630573b2354e44fcb0504` | 14733 |
| `mydocs/pr/assets/pr9063/correction-inputs/wasm-hwpx-edit.hwpx` | `f952fe45712f70881dc91e1c74cb4daae5f321ca0a4207555932c4439b02d90a` | `977907fd-53bc-480d-ad19-264baed2570e` | `pdf/pr9063/correction-wasm-hwpx-edit-hwpx-2020.pdf` | `09934e6a8363888c5e7a7bbd340b49b8104f1cade483d781e76f347998fb48cb` | 15850 |
| `mydocs/pr/assets/pr9063/correction-inputs/wasm-hwpx-edit.hwp` | `e5d7f550c35c6285ec8328dc6665865a7502d7cf8d65dc244d4c5e44e38c8432` | `c24e357f-ea30-4184-b43f-8341dd3a0d24` | `pdf/pr9063/correction-wasm-hwpx-edit-2020.pdf` | `d58e8ff0fe1bb26c137a37643ee20ea867c9925d7100a36f0efb60d881eb8406` | 15864 |

`edit-boundary/host-spacing-12pt.hwp`의 원래 간격 대조군은 job `554eba33-f86b-4bf8-bf87-6a160340c31d`, PDF `pdf/pr9063/host-spacing-12pt-2020.pdf`이며 SHA-256 `566e27a55e7adf8655f99345f40efba95a8d557b9398ac41f655dc741cb4ceca`다.

## 제품 입력과 회귀 commit의 동일성

| 제품 입력 | e2d7fb08b5 | 5b028b3f03 |
| --- | --- | --- |
| `src` | `c554f10aaa1114b1406e0fd6645b7186fde1367d` | `c554f10aaa1114b1406e0fd6645b7186fde1367d` |
| `Cargo.toml` | `67ffbf80f174ba3465e4454d140f758395833b2e` | `67ffbf80f174ba3465e4454d140f758395833b2e` |
| `Cargo.lock` | `34d08aa8d5150817968568e1ab6f68af228eeeac` | `34d08aa8d5150817968568e1ab6f68af228eeeac` |
| `build.rs` | `a3103a33faf38a899edb68f64fd0739dd4950787` | `a3103a33faf38a899edb68f64fd0739dd4950787` |
| `rust-toolchain.toml` | `262bc86af74777a65ee87ac929fc13b319bd390d` | `262bc86af74777a65ee87ac929fc13b319bd390d` |
| `scripts/wasm-pack-locked.sh` | `855ca67dcd4b175287c297d4d015851aab3cf8af` | `855ca67dcd4b175287c297d4d015851aab3cf8af` |

두 번째 commit은 정식 회귀·측정 주석·로컬 계획 기록만 보강한다. Native 시각 producer는 `1720c96477` 위의 합성 줄 보완 작업 트리에서 빌드했으며 그 제품 source는 `e2d7fb08b5`와 같다. 실제 CLI SHA-256은 `95aee58d1af191b4a8b798ce2a912a3c11eb0b14c5cd34748cf3eafaf3b724eb`다. WASM 실제 build source와 바이트 해시는 위와 같이 구분했다.

## 보호 검증의 글꼴 준비

#8896에서 검증한 원 프로그램 목록·순서·환경을 재구성했다. 로컬 원본 294개를 기존 기록의 SHA-256과 각각 대조하고 문서별 선택 목록을 공급했다. 이전 작업의 삭제된 디렉터리나 설치 목록을 공급 성공으로 간주하지 않았다. 구형 수식은 검증 대상 PDF 대신 canonical 독립 86문자 표본을 실제 한컴 2020 Print(job `26b067d9-46b9-4266-9b0a-76c4a3d47bb6`)하여 준비했다. 77개 Type3 원 프로그램 hash가 #8896의 독립 표본과 모두 같고, 이번 로컬 생성 SFNT SHA-256은 `bf319dcada899038f84e8b55ed6553c4a58fc3b15d19d5ff67f2821b3cc4e076`다. 글꼴·윤곽 프로그램·중간 문자 표본 PDF는 로컬에만 두었다.

Noto Serif KR은 canonical `_resource`로 저장소 소유 WOFF2를 같은 Native 프로그램으로 준비했다. source SHA `c9a1e3ac69994d680883b53b1fd83d520733b704792522a932bfaa276117de45`, 공급 TTF SHA `0ea7b146035288e22ca008e890bd16dbfb1a1a1ba370af0f15ed2e299ac86d18`를 확인했다. `sample10`의 연결 그림 세 개도 이전에 삭제된 worktree 경로 대신 현재 같은 공개 파일을 명시 공급했다.

## 정본 보호와 편람 gate

정본 TSV와 manifest의 입력·기준 PDF SHA-256·compat·DPI를 확인하고 현재 후보를 실제 다시 출력했다. 기준값·허용치·대상 페이지는 바꾸지 않았다.

| 범위 | 문서/쪽 | raw 최저 | 공식 두 자리 하락 |
| --- | --- | --- | --- |
| Native Skia Print | 18개/1,771쪽 | 97.99% | 0 |
| fresh 최적화 WASM Print Canvas | 18개/1,771쪽 | 94.65% | 0 |
| 합계 | 36경로/3,542쪽 | 각 경로 전쪽 90% 이상 | 0 |

- 원본 candidate TSV: `correction/protection/` 36개, 비교: `correction/protected-comparison.tsv` 3,542행.
- 기준: `mydocs/pr/assets/visual-protected-baseline/final.tsv`, SHA-256 `ee41455cf7f05585487d92961ccb9fd8160d5d1e15c3567019ea89e2b7bf2aeb`.
- 실제 같은 입력·PDF·Print/96 DPI/2px raw 조건을 유지하고, 공식 소수 둘째 자리 표시값의 하락을 0.00%p 조건으로 비교했다.
- 별도 `--handbook-gate` Native HWPX 383쪽도 passed, 하락 0이다. 이 문서는 보호 목록과 중복되며 별도 필수 호출의 실행 결과로 구분한다. 원 감독 셸의 143과 자식의 exit 0·별도 canonical 감사의 exit 0은 위 실행 오류 절에 구분했다.
- WASM sample10은 공개 연결 그림 `samples/oracle.gif`, `samples/rdb02.gif`, `samples/s1.jpg`를 명시 공급해 763쪽 전부 실행했다. 삭제된 이전 worktree 절대 경로 때문에 실행 전에 실패한 시도는 통과로 세지 않았고 현재 동일 파일 경로로 다시 실행했다.
- 각 독립 명령의 성공 결과와 원본 36개 전쪽 TSV를 확인했다. 성공한 결과를 실패한 초기 호출의 성공 코드로 바꿔 기록하지 않았다.

raw 100.00%는 RGB의 완전 동일을 뜻하지 않는다. 원문 픽셀의 안티앨리어싱 색상 차이와 개체 소유·본문 포함/순서의 실제 판독을 구분했다. 보호 대상에 새 예외를 적용하거나 기준값을 낮추지 않았다.

## 정본 PDF 경로와 쪽수 보호 등록

독립 Print 원본 17개는 정본 도구의 원본 상대 경로·형식·엔진 규칙에 따라 `pdf/issue8660/*-hwp-2020.pdf` 및 `pdf/tac_space_tail/*-hwp-2020.pdf`로 옮겼다. 이름 변경 전후 PDF 바이트·SHA는 같다. 위 영수증 표에는 최종 경로를 반영했다. 웹 Print는 정본으로 선택하지 않았다.

`tools/oracle_page_count/regenerate.py`의 canonical 선택·저장 제품/엔진·N-up·본문 범위 검사를 같은 원본 17개에 적용해 17/17쪽수 일치를 확인했다. 전체 다른 문서를 재생성하지 않고 신규 17행만 원장에 추가했으며 기존 행을 제거·변경하지 않았음을 대조했다. 새 경로는 아직 커밋 전이므로 도구의 PDF 바이트 읽기는 기존에 커밋한 동일 blob 경로로 연결했고 새 파일 SHA와 동일함을 먼저 확인했다. 기준값 허용 집합이나 차이 허용치를 넓히지 않았다.

## 최종 Rust 실행

제품 source는 `e2d7fb08b5`, 정식 assertion source는 `5b028b3f03`이다. 이후 Rust 변경은 두 회귀 파일의 `//!` 측정 주석·PDF 경로뿐이며 이를 제외한 코드 바이트는 같다. 쪽수 TSV는 신규 17행만 추가했다. 따라서 전체 회귀 13,524건은 같은 제품/검사 코드의 실제 전체 실행이며, 최종 주석 이후의 집중 회귀·새 데이터 보안/쪽수 실행과 구분한다.

모든 Cargo 명령은 최소 글꼴 bwrap과 같은 절대 `target/pr-review`, `CARGO_BUILD_JOBS=4`를 사용했다. 빌드는 직렬 실행했다. 아래 명령은 wrapper 안에서 실제 수행한 인수다.

| 검사 | 명령 | exit | 시간(초) |
| --- | --- | --- | --- |
| focus-issue_8660_restricted_para_object_next_page | `node scripts/run-rust-test.mjs issue_8660_restricted_para_object_next_page -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --test-threads 8` | 0 | 439.83 |
| focus-tac_space_tail_hang_before_float_anchor | `node scripts/run-rust-test.mjs tac_space_tail_hang_before_float_anchor -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --test-threads 8` | 0 | 1.41 |
| focus-issue_2164_cell_enter_overlap | `node scripts/run-rust-test.mjs issue_2164_cell_enter_overlap -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --test-threads 8` | 0 | 20.49 |
| focus-issue_7490_edited_paragraph_indent | `node scripts/run-rust-test.mjs issue_7490_edited_paragraph_indent -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --test-threads 8` | 0 | 1.37 |
| full-nextest | `cargo nextest run --locked --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --tests --test-threads 8 --no-fail-fast` | 0 | 1368.45 |
| native-lib | `cargo test --locked --profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --lib -- --test-threads 8` | 0 | 384.39 |
| native-placeholder | `node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --test-threads 8` | 0 | 343.41 |
| native-direct-pdf | `node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --test-threads 8` | 0 | 19.65 |
| fmt | `cargo fmt --all -- --check` | 0 | 25.55 |
| clippy-native | `cargo clippy --locked --all-targets --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings` | 0 | 180.37 |
| clippy-wasm32 | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings` | 0 | 121.27 |
| workspace-build | `cargo build --locked --workspace --target-dir /home/edward/mygithub/rhwp/target/pr-review` | 0 | 195.23 |
| workspace-clippy | `cargo clippy --locked --workspace --all-targets --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings` | 0 | 147.22 |
| skia-clippy | `cargo clippy --locked --all-targets --features native-skia --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings` | 0 | 140.0 |
| suite-check | `node scripts/rust-test-suite-manifest.mjs --check --base-ref dc3d0ff15e6e7cdb39138c72a5466994dea0e0d4` | 0 | 0.9 |
| suite-check | `node scripts/rust-test-suite-manifest.mjs --check --base-ref dc3d0ff15e6e7cdb39138c72a5466994dea0e0d4` | 0 | 0.9 |
| fmt-final | `cargo fmt --all -- --check` | 0 | 25.39 |
| issue_8660_restricted_para_object_next_page | `node scripts/run-rust-test.mjs issue_8660_restricted_para_object_next_page -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --test-threads 8` | 0 | 332.27 |
| tac_space_tail_hang_before_float_anchor | `node scripts/run-rust-test.mjs tac_space_tail_hang_before_float_anchor -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --test-threads 8` | 0 | 15.72 |
| security_corpus_regression | `node scripts/run-rust-test.mjs security_corpus_regression -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --test-threads 8` | 0 | 18.89 |
| oracle_page_count_baseline | `node scripts/run-rust-test.mjs oracle_page_count_baseline -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --test-threads 8` | 0 | 20.54 |

- 전체 nextest: 13,524 passed / 0 failed / 45 config-skipped. SVG snapshot 결정성 검사도 포함해 통과했고 golden을 재생성하거나 낮추지 않았다.
- Native 공식 lib: 3,853 + 15 + 165 + 2 = 4,035 passed / 12 ignored. placeholder 2와 direct PDF 36을 더해 4,073 passed다.
- 기존 성장/Enter 5건과 편집 들여쓰기 20건도 최소 글꼴에서 통과했다.
- Native·WASM32·workspace all-target·Native Skia Clippy, workspace build, fmt, 고정 base의 manifest 검사가 모두 실제 exit 0이다.

### 생성 harness와 최종 집중 실행

마지막 manifest 검사의 첫 exit는 1이었다. 두 테스트의 측정 주석과 canonical PDF 경로 길이가 바뀌어 자동 배정 weight가 달라졌지만, 생성된 테스트를 갱신하지 않은 것이 원인이다. 이전 물리 manifest와 현재 canonical derive의 source 합집합 2,343개가 같고, 두 파일의 module 문서 주석을 제외한 코드/검사 바이트도 동일함을 확인했다. `--prepare`로만 28개 harness를 다시 생성해 `--check --base-ref dc3d0ff15e6e7cdb39138c72a5466994dea0e0d4`를 통과했다. 정책·source 등록·test filter를 완화하거나 생성 파일을 커밋하지 않았다.

최종 집중 실행의 실제 nextest 요약:

- `issue_8660_restricted_para_object_next_page`: Summary [   0.206s] 12 tests run: 12 passed, 330 skipped.
- `tac_space_tail_hang_before_float_anchor`: Summary [   0.014s] 4 tests run: 4 passed, 326 skipped.
- `security_corpus_regression`: Summary [   2.531s] 6 tests run: 6 passed, 329 skipped.
- `oracle_page_count_baseline`: Summary [   1.974s] 16 tests run: 16 passed, 336 skipped.

### 새 입력 보안과 쪽수 보호

신규 samples 17개를 JSON 배열로 `RHWP_SECURITY_SWEEP_SAMPLES_JSON`에 명시하고 공식 `run-rust-test.mjs security_corpus_regression` 경로로 실행했다. 빈 환경변수로 성공한 이전 전체 검사와 구분한다. 소스 자동 배정 target과 실제 module filter는 launcher가 계산했다.

추가로 원본·실제 저장 파생본 43개를 실제 Native CLI의 `inspect hidden-text`, `inspect injection --include-fields`, `inspect unicode`에 전달했다. 129회 모두 exit 0 및 `clean:true`다. [입력 43개 SHA와 세 축 결과](correction/security-inputs.tsv)를 보존했고, 파일 바이트를 다시 대조했다. 탐지기 비활성화·allowlist·경고 재분류는 하지 않았다.

Oracle 원장은 기존 모든 행이 그대로이며 신규 17행은 모두 Print와 쪽수가 같다. 최종 원장에 대해 공식 `oracle_page_count_baseline` module을 실제 실행했다. 이 데이터 추가를 전체 nextest 실행 전에 이미 포함했던 것으로 기록하지 않는다.

최종 증적 정리에서 보호 비교 TSV의 CRLF가 `git diff --check`에 걸렸다. 커밋 명령 앞의 실패 중단 처리가 없어 로컬 기록 commit이 한 번 생성됐으나, LF로 정규화하고 CSV 파싱 전후의 모든 행·필드 동일성을 확인한 뒤 같은 로컬 기록 commit을 정정했다. raw·공식값·판정·대상은 바꾸지 않았다. 제출 전 전체 base diff와 staged diff의 공백 검사를 다시 수행했다.

## 원 기여의 본문 넘침 래칫 검토

원 기여 `351674bfa4`는 `body_overflow_baseline.tsv`에 제한 off 대조군 `issue8660/picture_unrestricted_over_12px.hwp`의 의도된 넘침 1건을 등록했다. 쪽 영역 제한을 끄면 개체는 같은 쪽 본문 아래 끝을 넘어 그려지며, 새 데스크톱 한컴 2020 Print에서도 1쪽의 같은 배치를 확인했다. Native/fresh WASM 전쪽 raw 100.00%와 전체 회귀의 실제 counter 검사를 근거로 값 1을 유지한다. 원 주석의 웹 Print 참조만 canonical Desktop Print로 교체했고 TSV의 비주석 행은 전후 모두 같다. 이는 새 이월 결함을 래칫에 숨긴 것이 아니다. 실제 쪽 제한을 켠 반례와 fit 대조군은 별도의 관계 회귀로 보호한다.

## GitHub 수용과 검증 결과

- 제출 후보 `fafc84f71187aa1edec73d5204d501abaa0aa50a`, 수용 PR [#9069](https://github.com/edwardkim/rhwp/pull/9069). 원 PR [#9063](https://github.com/edwardkim/rhwp/pull/9063)의 여섯 commit을 보존했다.
- 제품 CI [43558842756](https://github.com/edwardkim/rhwp/actions/runs/43558842756): Archive A 3,796 / B 2,046 / C 2,467 / D 2,515 / E 2,506, 합계 13,330 passed. Native CI는 선택 검사 243 passed다. 로컬 전체 13,524건·Native 공식 4,073건과 범위를 구분한다.
- CodeQL [43558842804](https://github.com/edwardkim/rhwp/actions/runs/43558842804), Render Diff [43558842738](https://github.com/edwardkim/rhwp/actions/runs/43558842738), Adapter [43558842599](https://github.com/edwardkim/rhwp/actions/runs/43558842599), Proptest [43558842615](https://github.com/edwardkim/rhwp/actions/runs/43558842615), 최종 정책 [43561181677](https://github.com/edwardkim/rhwp/actions/runs/43561181677) 모두 success. 최종 check 33 success / 5 정상 skip / 0 fail. CI 22개 job의 실제 step 종료도 확인했다.
- [Self-review](https://github.com/edwardkim/rhwp/pull/9069#issuecomment-6872355751) 뒤 실제 merge `5423ce12b1553af7352c38c24fad663d3b04fcde`를 확인했다. 최신 base/실제 PR ref/실제 merge tree는 `18c2f8053ec711a649c646585cdcecb4babb2f01`로 같았다.
- 원 PR #9063은 간접 MERGED이며 연결 이슈 #8660·#9059는 CLOSED다. 자동 완료를 API로 확인하고, 승인된 완료·감사 댓글은 devel의 archive·이미지 확인 뒤 게시한다.
- 로컬 PR #9063 검증 종료 때 root output 전체·target/pr-review/debug·worktree의 임시 output을 정리해 16.85 GiB를 확보했다. 이후 승인된 원격 단계용 임시 output을 새로 만들었으며 후속 처리 종료 때 다시 전체 삭제한다. 원시 로그·API JSON을 Git에 추가하지 않는다.

## Merge 후 확인

[Duration 갱신 43561454031](https://github.com/edwardkim/rhwp/actions/runs/43561454031)은 success이며 실제 결과 `ready:true`, `successful-pr-worker-measurements`와 `published:true`를 확인했다. 게시한 duration data commit은 `15d0b35a7113ba1312110ad1545668f95575ce98`, source merge는 `5423ce12b1553af7352c38c24fad663d3b04fcde`다. 검사 재실행이나 별도 CI 시작은 하지 않았다.

기여자 댓글에 쓸 네 파일의 merge commit raw HTTP 응답은 PNG이며, 실제 Git blob과 바이트·SHA-256이 같다.

| 이미지 | SHA-256 | bytes |
| --- | --- | --- |
| `mydocs/pr/assets/pr9063/correction/native-detail.png` | `16bab6b291a4887fc61730b397adb7eaf6ecd9bf6cf538030d5baed3ecddf52d` | 22379 |
| `mydocs/pr/assets/pr9063/correction/wasm-detail.png` | `297d99f696a6515ec7f0527b5be54a6f9a62e6e1ba4f58539c548b5d9b5bc074` | 25213 |
| `mydocs/pr/assets/pr9063/native-detail-comparison.png` | `9feb77a8dc2af38135ca49d8afa586ce2e40c5b5f4d55e713836efd76e634159` | 42602 |
| `mydocs/pr/assets/pr9063/wasm-detail-comparison.png` | `19edbe930fdffc6b670dd3daabfab1965a116339edb824c1a6ec9bcf35c6ed30` | 49792 |
