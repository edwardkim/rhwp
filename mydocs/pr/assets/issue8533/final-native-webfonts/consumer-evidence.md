# 실제 출력 소비 경로와 공개 프로그램 출처

제품 소스는 `c031a38a41670f5e4f6ab0268ef46c62ab1445f5`이며 [전쪽 TSV](tsv-index.md)와 [PNG 출처](png-index.md)를 함께 확인한다.

## 일반 Native CLI

Mac 추가 글꼴 경로를 파일 접근 정책으로 차단했다. 시스템 글꼴 이름까지 제거한 깨끗한 VM과 같다고 주장하지 않는다. 공개 공급 프로그램 9개를 준비했고 기존 사용자 글꼴을 교체하지 않았다.

| 실행 | 실제 경과 초 |
| --- | ---: |
| focus-online | 16.202 |
| focus-offline | 12.436 |
| svg-offline | 5.276 |
| pdf-svg | 5.425 |
| pdf-direct | 9.023 |
| whole-offline | 81.853 |

전 383쪽은 일반 CLI의 단일 `export-png` 호출로 생성했다. 온라인·오프라인 집중 4쪽의 PNG 해시는 동일했다. `--handbook-gate` 정본 실행은 719.427초였으며 Cargo 작업과 경합했으므로 단독 성능 수치로 해석하지 않는다.

| 공개 URL | 다운로드 원본 SHA-256 | Native 프로그램 SHA-256 | Native 바이트 |
| --- | --- | --- | ---: |
| [공급원](https://cdn.jsdelivr.net/gh/google/fonts@2fe1e8c6b137fb13041a7cfca6fdcd8a910ea4dc/ofl/notoserif/NotoSerif%5Bwdth,wght%5D.ttf) | `4d8e6761424656867019081a1a01336f3cb086982682698714054fc33f782713` | `4d8e6761424656867019081a1a01336f3cb086982682698714054fc33f782713` | 1887192 |
| [공급원](https://cdn.jsdelivr.net/gh/fonts-archive/KoPubDotum@295e0fc83931fda749f129b6690c0fa93f798007/KoPubDotum-Bold.ttf) | `967efe18063dd971c63dde3e8fcb247e0fcf16486a24ba2e284a716e44b90275` | `967efe18063dd971c63dde3e8fcb247e0fcf16486a24ba2e284a716e44b90275` | 2968028 |
| [공급원](https://cdn.jsdelivr.net/gh/fonts-archive/KoPubDotum@295e0fc83931fda749f129b6690c0fa93f798007/KoPubDotum-Light.ttf) | `018a0f18613a98d446696ea3d36194e288d5d8004aa58327789c830f6179e23d` | `018a0f18613a98d446696ea3d36194e288d5d8004aa58327789c830f6179e23d` | 2994860 |
| [공급원](https://cdn.jsdelivr.net/gh/fonts-archive/KoPubDotum@295e0fc83931fda749f129b6690c0fa93f798007/KoPubDotum-Medium.ttf) | `d0adfdcb40b7e73389128cc0222b53a91e5aa0e653f89967485a1f8b46124eb7` | `d0adfdcb40b7e73389128cc0222b53a91e5aa0e653f89967485a1f8b46124eb7` | 2975032 |
| [공급원](https://cdn.jsdelivr.net/gh/fonts-archive/KoPubBatang@7ef37ed4015cf0f638952e3d9937e4192f7cea97/KoPubBatang-Bold.ttf) | `94de26a11aa25619526ac8183fde4f46e74e22c527acb42aea2e93f5313bbc5b` | `94de26a11aa25619526ac8183fde4f46e74e22c527acb42aea2e93f5313bbc5b` | 6536212 |
| [공급원](https://cdn.jsdelivr.net/gh/fonts-archive/KoPubBatang@7ef37ed4015cf0f638952e3d9937e4192f7cea97/KoPubBatang-Light.ttf) | `b7438d61595fe9584706ab2cb3c9aa2acdc486d61c8299374a30b3e77bc2045f` | `b7438d61595fe9584706ab2cb3c9aa2acdc486d61c8299374a30b3e77bc2045f` | 6481172 |
| [공급원](https://cdn.jsdelivr.net/gh/fonts-archive/KoPubBatang@7ef37ed4015cf0f638952e3d9937e4192f7cea97/KoPubBatang-Medium.ttf) | `7a798592beb0d35bb557b010c2794cbcc988e89fcbcee0eb5edbb35161141ab4` | `7a798592beb0d35bb557b010c2794cbcc988e89fcbcee0eb5edbb35161141ab4` | 6414208 |
| [공급원](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff) | `b3192018e3bd4da76b29b0530f71c79fb6f465a2b7cd41308bc38f9a4ae09a6c` | `24b49246526c86ddd7bf44d3ee88a3cc044f51f51af4e01c29a1e699059981e1` | 28413328 |
| [공급원](https://cdn.jsdelivr.net/gh/edwardkim/rhwp@7d06d5c706f1d89f1662e739abf2b1b25d2a98e4/assets/fonts/NotoSerifKR-Regular.woff2) | `c9a1e3ac69994d680883b53b1fd83d520733b704792522a932bfaa276117de45` | `35f7479023ae8fa60fde58eb702070d03fbdb43de9228c4a7f8b1c27ffe80518` | 8434876 |

문서 선언 중 미공급으로 보고된 이름: `#태고딕`, `-윤고딕120`, `-윤명조120`, `-윤명조140`, `Dinbol`, `Yoon가변 윤명조 120_TT`, `정부상징 부처명_16040911`, `한양견고딕`, `한양신명조`, `휴먼명조`.
별도 후보 4개는 공급 한도 때문에 거부했다. 미사용 선언·generic 표시 대체와 원 프로그램 공급 성공을 구분한다. 이 목록이 모두 실제 페이지에 사용됐다고 주장하지 않는다.

## Native SVG·두 PDF backend

| 실제 경로 | 기준 physical page | raw % | 실제 PNG | SHA-256 |
| --- | ---: | ---: | --- | --- |
| svg-browser | 272 | 99.77168 | [raw](png/consumers/svg-browser-handbook-p272.png) | `5dea10be703579c61fe3aaadb3ba3ac23f72a966e08d4e55c577da554fe2868f` |
| pdf-direct | 272 | 100.00000 | [raw](png/consumers/pdf-direct-handbook-p272.png) | `68bd9d8ff4751acb2de8a8e166c7f86f0f54c24ea2924b4fff23369d65246a85` |
| pdf-svg | 272 | 100.00000 | [raw](png/consumers/pdf-svg-handbook-p272.png) | `e8e8adbc00d17c88687a542a45224bc1b56c42e0dfc648dd63d3074e93f35f65` |

PDF의 선택 출력은 파일 내부에서는 1쪽이지만 원본 내용은 편람 physical 272쪽이다. Native SVG의 Chrome 래스터를 Native Skia PNG라고 부르지 않는다.

## 실제 Studio

설치 글꼴은 보존하고 Local Font Access 권한을 명시적으로 거부한 Chrome의 실제 화면이다. page 사각형을 캡처하여 Canvas·DOM·겹친 레이어를 포함했다. Canvas 데이터만 추출한 이미지가 아니다.

| 요청 | 입력 | physical page | 실제 backend | 요청 수용 | 오류 |
| --- | --- | ---: | --- | --- | ---: |
| canvas2d-handbook | `samples/2025 행정업무운영 편람(최종).hwpx` | 272 | canvas2d | 예 | 0 |
| canvas2d-evaluation | `samples/issue6551/113424_evaluation_guideline.hwpx` | 5 | canvas2d | 예 | 0 |
| canvas2d-source-hft | `samples/issue5701/1530000-200800002_slice_p139_tac_reset_tail.hwp` | 4 | canvas2d | 예 | 0 |
| canvas2d-language-control | `samples/issue8136/lang-slot-ranges.hwpx` | 1 | canvas2d | 예 | 0 |
| canvaskit-handbook | `samples/2025 행정업무운영 편람(최종).hwpx` | 272 | canvas2d | 아니오 | 0 |
| canvaskit-evaluation | `samples/issue6551/113424_evaluation_guideline.hwpx` | 5 | canvas2d | 아니오 | 0 |
| canvaskit-source-hft | `samples/issue5701/1530000-200800002_slice_p139_tac_reset_tail.hwp` | 4 | canvas2d | 아니오 | 0 |
| canvaskit-language-control | `samples/issue8136/lang-slot-ranges.hwpx` | 1 | canvas2d | 아니오 | 0 |

위 네 CanvasKit 요청은 모두 준비 제약으로 Canvas2D에 fallback했다. CanvasKit 통과로 세지 않으며 guard를 우회하지 않았다. 별도의 지원 대조군 `group-drawing-02.hwp`는 실제 CanvasKit software로 완료했고 readiness=true, 미지원 연산·미등록 글꼴 대체·숨은 Canvas2D overlay가 없었다.

[실제 CanvasKit 대조군](png/studio/canvaskit-group-drawing.png) · [편람 Canvas2D](png/studio/canvas2d-handbook.png)

글꼴 파일 접근을 차단한 별도 Studio 실행은 CoreText 이름 캐시와 실제 바이트의 불일치로 글리프 누락이 있어 통과 증거로 사용하지 않았다. 최소 글꼴 WASM Sweep은 이 Studio 실행과 별도 판정한다.

## 초기화에 실제 사용한 응답

Vite가 WASM 상대 URL 한 곳을 /@fs 절대 URL로 치환한다. 실제 받은 JS의 sourcemap 원문을 복원하면 root pkg JS와 바이트 해시가 같다. 실제 WASM 응답은 root pkg·Studio/public과 그대로 같다. 재요청 응답으로 대신하지 않았다.

| 실제 case | 수신 JS SHA-256 | URL 정규화 후 JS SHA-256 | 실제 초기 WASM SHA-256 |
| --- | --- | --- | --- |
| canvas2d-handbook | `58f80d95f069efd487914e0a33ba6af3dfc61b04ac821bd86f7e98728b8f13e8` | `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393` | `41f27298cf2b025bf5cebba6ea4cd96f539ceb3a97846368ca37df47d3ab9424` |
| canvas2d-evaluation | `58f80d95f069efd487914e0a33ba6af3dfc61b04ac821bd86f7e98728b8f13e8` | `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393` | `41f27298cf2b025bf5cebba6ea4cd96f539ceb3a97846368ca37df47d3ab9424` |
| canvas2d-source-hft | `58f80d95f069efd487914e0a33ba6af3dfc61b04ac821bd86f7e98728b8f13e8` | `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393` | `41f27298cf2b025bf5cebba6ea4cd96f539ceb3a97846368ca37df47d3ab9424` |
| canvas2d-language-control | `58f80d95f069efd487914e0a33ba6af3dfc61b04ac821bd86f7e98728b8f13e8` | `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393` | `41f27298cf2b025bf5cebba6ea4cd96f539ceb3a97846368ca37df47d3ab9424` |
| canvaskit-handbook | `58f80d95f069efd487914e0a33ba6af3dfc61b04ac821bd86f7e98728b8f13e8` | `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393` | `41f27298cf2b025bf5cebba6ea4cd96f539ceb3a97846368ca37df47d3ab9424` |
| canvaskit-evaluation | `58f80d95f069efd487914e0a33ba6af3dfc61b04ac821bd86f7e98728b8f13e8` | `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393` | `41f27298cf2b025bf5cebba6ea4cd96f539ceb3a97846368ca37df47d3ab9424` |
| canvaskit-source-hft | `58f80d95f069efd487914e0a33ba6af3dfc61b04ac821bd86f7e98728b8f13e8` | `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393` | `41f27298cf2b025bf5cebba6ea4cd96f539ceb3a97846368ca37df47d3ab9424` |
| canvaskit-language-control | `58f80d95f069efd487914e0a33ba6af3dfc61b04ac821bd86f7e98728b8f13e8` | `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393` | `41f27298cf2b025bf5cebba6ea4cd96f539ceb3a97846368ca37df47d3ab9424` |

Vite 수신 JS와 root JS가 그대로 같다고 주장하지 않는다. 실제 수신 응답의 sourcemap 원문에서 WASM 상대 URL 한 곳의 변환을 되돌린 결과를 비교했다. WASM은 root pkg·Studio public·실제 초기화 응답이 같은 해시다. 재요청 응답으로 대신하지 않았다.
