# 공급망·Action 소비 계약

기준 `024d610633ede166ebcbb915f9c90e7a9d0649d8` → 검증 source `2320351d2325a41d900e91f4e07f559d4a924a7d`. 2026-11-09에 실제 lockfile 전후 graph와 공식 archive/tag를 확인했다. 전체 변경 의존성의 위험 신호와 호환 계약 확인이며 전체 코드 보안 감사는 아니다. [소비 범위와 판정](dependency-review.md)을 함께 따른다.

## Cargo archive·MSRV·라이선스

직접 10종과 전이 3종의 공식 archive SHA-256이 Cargo.lock checksum과 모두 일치했다. 정확 버전 13종의 OSV 응답 advisory ID는 0개다. 프로젝트 Rust는 1.93.1이며 선언된 MSRV 최대 1.88이다. 선언이 없는 crate도 실제 build/lint를 통과했다.

| crate | version | MSRV | license | 공식 archive SHA-256 |
| --- | --- | --- | --- | --- |
| cfb | 0.16.0 | 1.74 | MIT | [fef29b910ef2b9dbecc60168d1568d643c633b0518af970c17e83bd5f6e2c94f](https://static.crates.io/crates/cfb/cfb-0.16.0.crate) |
| crc32fast | 1.5.4 | 1.63 | MIT OR Apache-2.0 | [5f97d3416d5337d15c4b0d78c8c91df922ca143f57ec20e2d1c188333047ef04](https://static.crates.io/crates/crc32fast/crc32fast-1.5.4.crate) |
| ctutils | 0.4.8 | 1.87 | Apache-2.0 OR MIT | [261af4dc7a06ea8a527a8717d8cbba3417530d6ea14e1edb3de6b67806be474f](https://static.crates.io/crates/ctutils/ctutils-0.4.8.crate) |
| encoding_rs | 0.8.44 | 1.88 | (Apache-2.0 OR MIT) AND BSD-3-Clause | [89253175e704ab869b04f9db224d38630e0211a53841a79181deae0889e169f5](https://static.crates.io/crates/encoding_rs/encoding_rs-0.8.44.crate) |
| log | 0.4.35 | 1.71.0 | MIT OR Apache-2.0 | [5d15f86d12da44cd8d4ec3c73984537eb7f4cadcb9776a575a0a8a05ca11005c](https://static.crates.io/crates/log/log-0.4.35.crate) |
| ml-dsa | 0.1.2 | 1.87 | Apache-2.0 OR MIT | [6f6d49c3f3f0e10e18ca975ffda66f8da0ec36521c1d911adcbc521bf0743f5c](https://static.crates.io/crates/ml-dsa/ml-dsa-0.1.2.crate) |
| ml-kem | 0.3.4 | 1.87 | Apache-2.0 OR MIT | [84ca78b66f46050d49fa21693ae8adab2081d8101b65d8067316b6b70b64a0d1](https://static.crates.io/crates/ml-kem/ml-kem-0.3.4.crate) |
| module-lattice | 0.2.4 | 1.87 | Apache-2.0 OR MIT | [c4a1e412c8ee83a1c678f427806219c49c96b2511b9f07497514f0e43fb26155](https://static.crates.io/crates/module-lattice/module-lattice-0.2.4.crate) |
| svgtypes | 0.16.2 | 1.85 | Apache-2.0 OR MIT | [9f12b12995e3638c70618c1facb1ce65302f6a6f706b675f07c58a5d7391120e](https://static.crates.io/crates/svgtypes/svgtypes-0.16.2.crate) |
| unicode-properties | 0.1.5 | 선언 없음 | (MIT OR Apache-2.0) AND Unicode-3.0 | [993ad287eb20e8ba755312801cfe1413e0f25d604075863acaddca10b04c1833](https://static.crates.io/crates/unicode-properties/unicode-properties-0.1.5.crate) |
| ureq | 3.5.1 | 1.85 | MIT OR Apache-2.0 | [e7be76686bdf7198002062423ca0d8cf9b75a27c8e1de804513b97e97b555d9d](https://static.crates.io/crates/ureq/ureq-3.5.1.crate) |
| ureq-proto | 0.7.1 | 1.85 | MIT OR Apache-2.0 | [622dce33c252115466d83f63e9744a18f12b4406aae51a01c15ad8fcf1185eb3](https://static.crates.io/crates/ureq-proto/ureq-proto-0.7.1.crate) |
| zip | 9.4.0 | 1.88 | MIT | [3cec733c9820347a592f6375c3c0089b479ba0a0b1cbf3dd1958b02959b7c033](https://static.crates.io/crates/zip/zip-9.4.0.crate) |

## npm lockfile 전후 비교

NPMScan dependency-audit의 dev 포함 변경 graph를 검사했다. Studio는 changed 20 / added 0 / removed 0 / flagged 0, VS Code는 changed 2 / added 0 / removed 22 / flagged 0이다. Chrome·Firefox 각 19개 변경 node는 전후 전체 metadata가 Studio의 감사 node와 동일함을 대조한 뒤 재사용했다. 새 lifecycle script·출처 교체·identity mismatch·보고된 취약점은 0개다. 변경 없는 전체 graph나 계정 권한을 감사한 결과로 확대하지 않는다.

| graph | node | before | after | 설치 script | 새 script | 보고된 취약점 |
| --- | --- | --- | --- | --- | --- | --- |
| Studio | @anthropic-ai/sdk | 1.3.0 | 1.6.0 | False | False | False |
| Studio | @oxc-project/types | 0.155.0 | 0.156.0 | False | False | False |
| Studio | @rolldown/binding-android-arm-eabi | 1.2.15 | 1.2.16 | False | False | False |
| Studio | @rolldown/binding-android-arm64 | 1.2.15 | 1.2.16 | False | False | False |
| Studio | @rolldown/binding-darwin-arm64 | 1.2.15 | 1.2.16 | False | False | False |
| Studio | @rolldown/binding-darwin-x64 | 1.2.15 | 1.2.16 | False | False | False |
| Studio | @rolldown/binding-freebsd-x64 | 1.2.15 | 1.2.16 | False | False | False |
| Studio | @rolldown/binding-linux-arm-gnueabihf | 1.2.15 | 1.2.16 | False | False | False |
| Studio | @rolldown/binding-linux-arm64-gnu | 1.2.15 | 1.2.16 | False | False | False |
| Studio | @rolldown/binding-linux-arm64-musl | 1.2.15 | 1.2.16 | False | False | False |
| Studio | @rolldown/binding-linux-ppc64-gnu | 1.2.15 | 1.2.16 | False | False | False |
| Studio | @rolldown/binding-linux-s390x-gnu | 1.2.15 | 1.2.16 | False | False | False |
| Studio | @rolldown/binding-linux-x64-gnu | 1.2.15 | 1.2.16 | False | False | False |
| Studio | @rolldown/binding-linux-x64-musl | 1.2.15 | 1.2.16 | False | False | False |
| Studio | @rolldown/binding-openharmony-arm64 | 1.2.15 | 1.2.16 | False | False | False |
| Studio | @rolldown/binding-win32-arm64-msvc | 1.2.15 | 1.2.16 | False | False | False |
| Studio | @rolldown/binding-win32-x64-msvc | 1.2.15 | 1.2.16 | False | False | False |
| Studio | postcss | 8.5.31 | 8.5.32 | False | False | False |
| Studio | rolldown | 1.2.15 | 1.2.16 | False | False | False |
| Studio | vite | 8.3.5 | 8.3.6 | False | False | False |
| VS Code | rechoir | 0.8.0 | 1.0.0 | False | False | False |
| VS Code | webpack-cli | 7.2.3 | 7.3.1 | False | False | False |

SDK 1.6.0의 라이선스는 Apache-2.0이다. unicode-properties의 Unicode-3.0 고지는 기존 `assets/licenses/Unicode-3.0.txt`와 SHA-256 `e7a93b009565cfce55919a381437ac4db883e9da2126fa28b91d12732bc53d96`로 동일했다. 이번 대상의 고지 표만 현행화했다.

## 공식 Action과 호출 계약

| Action | before | after | runtime 유지 | 제거 input | 새 input | 기본값 변경 |
| --- | --- | --- | --- | --- | --- | --- |
| github/codeql-action/analyze | 9b3254085494b55a0f7c4c2b3d2d52cc712ce370 | 33d7b85da1a04171e211befe2c739303b9a4c066 | True | [] | [] | {} |
| github/codeql-action/init | 9b3254085494b55a0f7c4c2b3d2d52cc712ce370 | 33d7b85da1a04171e211befe2c739303b9a4c066 | True | [] | [] | {} |
| softprops/action-gh-release | efb35369e0ad2afab669f228072c1b0d510eae64 | 8a0cd651e5b2365bd640854a3fa5dfc70e4875f3 | True | [] | [] | {} |
| taiki-e/install-action | 68a389c788b2cd04016ce8c80ee23b80d2ab3d68 | 3a3daafd3b7f23b8f65d9f5ba2db203fb59b79e2 | True | [] | [] | {} |

softprops v3.0.4·CodeQL v4.39.3·taiki-e v2.90.2의 공식 tag와 full SHA를 대조했다. Node24/composite runtime 유지, 제거된 input·기본값 변경 없음. workflow의 실제 변경 16줄은 uses 또는 버전 주석이며 job permissions·조건·명령·검사 범위를 바꾸지 않는다. release 게시 동작 자체는 실행하지 않았다.

## Native HTTPS 실제 소비

빈 cache에서 기준/후보 CLI를 각각 실행했다. 두 exit 0, 13개 다운로드 영수증과 프로그램 해시가 모두 같았다. 요청의 15초 timeout·redirect 0·응답 크기 한도는 유지됐다. 별도 cold cache의 명령은 다음과 같고 렌더링은 page 0만 수행했다. 편람 383쪽 gate와 구분한다.

```sh
/home/edward/mygithub/rhwp/output/pr-review/dependabot-20261109/build/base-rhwp export-png '/home/edward/mygithub/rhwp/samples/2025 행정업무운영 편람(최종).hwpx' --compat 2024 --profile print --page 0 --dpi 96 --json --output /home/edward/mygithub/rhwp/output/pr-review/dependabot-20261109/transport/base
```

```sh
/home/edward/mygithub/rhwp/output/pr-review/dependabot-20261109/build/candidate-rhwp export-png '/home/edward/mygithub/rhwp/samples/2025 행정업무운영 편람(최종).hwpx' --compat 2024 --profile print --page 0 --dpi 96 --json --output /home/edward/mygithub/rhwp/output/pr-review/dependabot-20261109/transport/candidate
```

| 공식 다운로드 URL | 프로그램 SHA-256 | bytes |
| --- | --- | --- |
| https://cdn.jsdelivr.net/gh/fonts-archive/KoPubDotum@295e0fc83931fda749f129b6690c0fa93f798007/KoPubDotum-Light.ttf | 018a0f18613a98d446696ea3d36194e288d5d8004aa58327789c830f6179e23d | 2994860 |
| https://cdn.jsdelivr.net/gh/edwardkim/rhwp@7d06d5c706f1d89f1662e739abf2b1b25d2a98e4/assets/fonts/NotoSansKR-Regular.woff2 | 39c5925cf712ba90568994b008266fec755f41fd14ba6112b8c9b6d1a08566b1 | 2538340 |
| https://cdn.jsdelivr.net/gh/edwardkim/rhwp@7d06d5c706f1d89f1662e739abf2b1b25d2a98e4/assets/fonts/NotoSerifKR-Regular.woff2 | 35f7479023ae8fa60fde58eb702070d03fbdb43de9228c4a7f8b1c27ffe80518 | 8434876 |
| https://cdn.jsdelivr.net/gh/fonts-archive/KoPubDotum@295e0fc83931fda749f129b6690c0fa93f798007/KoPubDotum-Medium.ttf | d0adfdcb40b7e73389128cc0222b53a91e5aa0e653f89967485a1f8b46124eb7 | 2975032 |
| https://cdn.jsdelivr.net/gh/fonts-archive/KoPubBatang@7ef37ed4015cf0f638952e3d9937e4192f7cea97/KoPubBatang-Bold.ttf | 94de26a11aa25619526ac8183fde4f46e74e22c527acb42aea2e93f5313bbc5b | 6536212 |
| https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff | 24b49246526c86ddd7bf44d3ee88a3cc044f51f51af4e01c29a1e699059981e1 | 28413328 |
| https://cdn.jsdelivr.net/gh/fonts-archive/KoPubDotum@295e0fc83931fda749f129b6690c0fa93f798007/KoPubDotum-Bold.ttf | 967efe18063dd971c63dde3e8fcb247e0fcf16486a24ba2e284a716e44b90275 | 2968028 |
| https://cdn.jsdelivr.net/gh/fonts-archive/KoPubBatang@7ef37ed4015cf0f638952e3d9937e4192f7cea97/KoPubBatang-Medium.ttf | 7a798592beb0d35bb557b010c2794cbcc988e89fcbcee0eb5edbb35161141ab4 | 6414208 |
| https://cdn.jsdelivr.net/gh/fonts-archive/KoPubBatang@7ef37ed4015cf0f638952e3d9937e4192f7cea97/KoPubBatang-Light.ttf | b7438d61595fe9584706ab2cb3c9aa2acdc486d61c8299374a30b3e77bc2045f | 6481172 |
| https://cdn.jsdelivr.net/gh/edwardkim/rhwp@7d06d5c706f1d89f1662e739abf2b1b25d2a98e4/assets/fonts/D2Coding-Regular.woff2 | b72cc97f865e6399e78afda96b2d6fff92886f3ebca294d3104d7cb2ffbe4fc7 | 4185576 |
| https://cdn.jsdelivr.net/gh/edwardkim/rhwp@7d06d5c706f1d89f1662e739abf2b1b25d2a98e4/assets/fonts/Pretendard-Regular.woff2 | 75f36828366a8771fdb41000e56c1d0824ac3e1d2bf1ed320fbb135016876d6e | 2745556 |
| https://cdn.jsdelivr.net/gh/google/fonts@2fe1e8c6b137fb13041a7cfca6fdcd8a910ea4dc/ofl/notoserif/NotoSerif%5Bwdth,wght%5D.ttf | 4d8e6761424656867019081a1a01336f3cb086982682698714054fc33f782713 | 1887192 |
| https://cdn.jsdelivr.net/gh/edwardkim/rhwp@7d06d5c706f1d89f1662e739abf2b1b25d2a98e4/assets/fonts/NotoSerifKR-Bold.woff2 | 6bdf40409ec4c749fb2feb0d032d5252bde526ede5c898e3c50a20ade2041ae6 | 8439348 |

원 문서의 모든 글꼴을 공급한 것으로 주장하지 않는다. 기존의 bounded font-supply 경고는 base와 후보에 동일하게 남았다. 글꼴 파일은 검증용 로컬 자료이며 Git에 넣지 않았다.
