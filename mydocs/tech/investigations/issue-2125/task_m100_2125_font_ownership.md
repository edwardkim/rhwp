---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-2125/README.md
last_verified: 2026-07-16
---

# Task M100 #2125 — frontend font canonical ownership

- 이슈: #2125
- 단계: Stage 1 — current inventory와 dependency graph
- 기준 source commit: `upstream/devel` `e750e02f0c020cd3e5e7a94bef07586a2ec14820`
- 계획 commit: `ecb3f70b9ae2419ff2994d2370d5e9c0dbd26207`
- 작성일: 2026-07-13
- 선행 snapshot: `mydocs/metrics/frontend/2026-07-11/metrics.json`
- 선행 font 변경: #2190 / PR #2196

## 1. 결론

현재 font source of truth는 `web/fonts`이며 36개 WOFF2, 22,651,296 bytes다. #2125 구현 후 source
of truth는 공통 `assets/fonts`가 된다. Studio, extension, VS Code, npm/editor와 legacy `/web`는 모두
canonical source의 소비자다.

확정 ownership은 다음과 같다.

| 표면 | owner | 계약 |
|------|-------|------|
| source binary/license | `assets/fonts` | 36개 WOFF2와 `FONTS.md`, Source Han Serif Korean OFL |
| Studio public asset | `rhwp-studio/public/fonts` | `../../assets/fonts` link, runtime `fonts/...` 유지 |
| legacy `/web` | `web/fonts` | `../assets/fonts` compatibility link, canonical 아님 |
| Chrome/Firefox | 각 build script | canonical 36개를 `dist/fonts`로 exact copy |
| Safari | `rhwp-safari/build.sh` | Chrome dist의 font artifact 상속 |
| VS Code | webpack CopyPlugin | canonical에서 승인된 11개만 `dist/media/fonts`로 copy |
| npm/editor | package docs | 직접 bundle하지 않고 Studio/self-hosting 계약 설명 |

이번 이동은 ownership/source path 변경이다. runtime URL, fallback, 파일 구성과 binary 내용은 변경하지 않는다.

## 2. #2124 이후 provenance

#2124 snapshot은 당시 36개, 22,630,940 bytes를 기록했다. 현재와 비교한 결과
`NotoSansKR-Regular.woff2`만 다르며 #2190 / PR #2196의 KS 기호 범위 보강으로 설명된다.

| 항목 | #2124 snapshot | 현재 migration 기준 |
|------|----------------|-----------------------|
| file count | 36 | 36 |
| total bytes | 22,630,940 | 22,651,296 |
| Noto Sans KR bytes | 541,864 | 562,220 |
| Noto Sans KR SHA-256 | `3aa0d1d63f3b5b2a33053a41a34512eac63abfb1dc2112aeda04733656a1a6d9` | `d1bf8649914a4fe9477a8735bf056383e44e466141fb3d61897252e06d900c1a` |
| delta | Phase 0 당시 subset | #2190의 `U+2500-257F`, `U+25A0-25FF` 보강 |

나머지 35개 WOFF2는 #2124 snapshot과 bytes/hash가 동일하다. #2124 snapshot은 역사 evidence로
보존하고 현재 표를 #2125 이동 전 manifest로 사용한다.

## 3. 이동 전 WOFF2 manifest

| 파일 | Bytes | SHA-256 |
|------|------:|---------|
| `Cafe24Ssurround-v2.0.woff2` | 391868 | `2d726dbf8863d66e6d5cf56e2532c5d14d8a4b514461b565c513c1da6ae2f41a` |
| `Cafe24Supermagic-Regular-v1.0.woff2` | 765684 | `65434a217b9532867a1fe41417d0ad51210a60027a51dca40748aafed48ef0ba` |
| `D2Coding-Bold.woff2` | 1579172 | `41273810f9aed2775c6f1b9eb7e4f12997da644002872b20a31d3d3144e0cc0a` |
| `D2Coding-Regular.woff2` | 1485508 | `358df12fdf9b65d0fd31554a826f97af4eeefd96a047297a28a856315fe872c1` |
| `GowunBatang-Bold.woff2` | 447856 | `aef1b91ea9ea4688fb296044959b38dfeeaa3e1650af5d543a2b5c160fa30b36` |
| `GowunBatang-Regular.woff2` | 495092 | `0c6a288aa0c12081dccde769c312f71f290b353243747dd24fb8c57f94f01d72` |
| `GowunDodum-Regular.woff2` | 406476 | `349c098a6181952dab564bf891b8efcacfbf855adc28d40b4aa2e5937c101c56` |
| `Happiness-Sans-Bold.woff2` | 336604 | `54becdfd4f7f58e872eba8fbc67778e29cff94c1cd03f9a0ebadfbe482b4aba0` |
| `Happiness-Sans-Regular.woff2` | 291556 | `4fc37e7663c3d440b5a6af6938bde0c6618b666553ed521745f3362050bd4988` |
| `Happiness-Sans-Title.woff2` | 348700 | `057e6821a062d9192964d99cf058fe6608230e9e4a464c9652b3756e02460e22` |
| `HappinessSansVF.woff2` | 613412 | `cd29de60acf8c466938400561fcbfb14812dd5981f4f78b0c8da74609682423c` |
| `LatinModernMath-Regular.woff2` | 391760 | `1177d9025d1e797c54b3c7676976365abd90f903423091de7f0f4e7a4b372162` |
| `NanumGothic-Bold.woff2` | 417252 | `0bd36842cc218a7b0e1c0f5ec796d90152a02e4cd487b2d868f1130708fe551a` |
| `NanumGothic-ExtraBold.woff2` | 376972 | `cffea22fa5791731b3de9f9c48b58f4c9eff52581f29377d364a751e63b81ca3` |
| `NanumGothic-Regular.woff2` | 340688 | `4a000e3bca1298c43c70dc97447e537a27e0a8c1585f410be57dfeff06d3f713` |
| `NanumGothicCoding-Bold.woff2` | 534060 | `2fe66da1c9c3f8beca2c618a139e7edabd36a26dd810a324ced06f8b214b2ccd` |
| `NanumGothicCoding-Regular.woff2` | 521332 | `87ab32ebaf4e1f803099ad1d096744092c8f985b2e08a24e2304f5ac05d81d6a` |
| `NanumMyeongjo-Bold.woff2` | 575580 | `f2c95c35d72782f71cb60e3cc537c461ae3f5ee7e7504db9653ba7a7024dd548` |
| `NanumMyeongjo-ExtraBold.woff2` | 694516 | `b3d23b3f93f4bdad6c9b7f69b11156f90f1576b8383f6117448ee79e27609e56` |
| `NanumMyeongjo-Regular.woff2` | 502068 | `561e9193a6e848b3f7ce9f8f5907310a13e175bc9d69e77da2842a7517a97c16` |
| `NotoSansKR-Bold.woff2` | 559192 | `dab3d492daa687386292cff1499225db580f931f800ee0d124eb0636378c6020` |
| `NotoSansKR-ExtraLight.woff2` | 592496 | `3b5cac6af73fd94fa91ded10f6cfc54c7ce52189986fe7ca6fd457208f1d3673` |
| `NotoSansKR-Regular.woff2` | 562220 | `d1bf8649914a4fe9477a8735bf056383e44e466141fb3d61897252e06d900c1a` |
| `NotoSerifKR-Bold.woff2` | 1033556 | `504d6af6abb882acd912d47bafa383c4512a6c8c33635e4e8fb37111d9310941` |
| `NotoSerifKR-Regular.woff2` | 971428 | `c9a1e3ac69994d680883b53b1fd83d520733b704792522a932bfaa276117de45` |
| `Pretendard-Black.woff2` | 800404 | `c5fd0c3568fc1368a3edc0d0fbb36df029935954276e3573451b3bae09e27296` |
| `Pretendard-Bold.woff2` | 791156 | `4609c3356e536fafe38f4add0daeceb3d8595d3057bce13c428c33ddbd43d362` |
| `Pretendard-ExtraBold.woff2` | 793540 | `dd7c1e156f508eb962acc7a33a7a1896d1e0b71e11156fad96e731689ceb6dc3` |
| `Pretendard-ExtraLight.woff2` | 734392 | `df43dc9165dff4542114674bcd8b79b7daae6dec004004586d5d076fec6fe2aa` |
| `Pretendard-Light.woff2` | 757000 | `b7426635cce2ea2b95c9c802e43fba1c620e0dafaf25f737c069b8b4e09fa841` |
| `Pretendard-Medium.woff2` | 778432 | `d03481330eeba0659ab5b87f25ceb504a35de377dd90a0d0aba2982eb2d05e2c` |
| `Pretendard-Regular.woff2` | 765892 | `fad853f7f47c6c8b103171e7193fa095708cdcd70850a71d93aa5379e8a61d63` |
| `Pretendard-SemiBold.woff2` | 785856 | `c863f76a7de5c1ddc1ed8b2fa794964530774592c4f31407a84e2a2ae93f17f0` |
| `Pretendard-Thin.woff2` | 694804 | `1539755224a64719d5b18406762c476db74fcc299b9e4641ca1e9812fbc7a09b` |
| `SourceHanSerifK-OldHangul-subset.woff2` | 239628 | `9e419cd16df2ea3b220aa7751320d956ac2493440ba412484d98325078f09d43` |
| `SpoqaHanSans-Regular.woff2` | 275144 | `6568bf80b50c14f2e54f56f2c07e668416f39f880baf5ea2825d315767f98f69` |

## 4. license/inventory manifest

| 파일 | Bytes | SHA-256 | 이동 후 owner |
|------|------:|---------|----------------|
| `web/fonts/FONTS.md` | 5831 | `129f12ff89f53061a256a59d7e2f7737de32fdf461e3566414ee3f7bd5549734` | `assets/fonts/FONTS.md` |
| `web/fonts/SourceHanSerifK-OFL.txt` | 4463 | `9ff5bb567e1b92c801fc1069e5fbf992ff8efccacb9db94e5959a5b3ba9bb903` | `assets/fonts/SourceHanSerifK-OFL.txt` |
| `THIRD_PARTY_LICENSES.md` | 9511 | `fef407c3519a1eaafe6943ec4651aa97cf2868254c878eff55e9610e902764ec` | repository license index |

`FONTS.md`와 root license index는 Stage 3에서 current canonical path를 설명하도록 문구를 갱신하므로
문서 hash는 이동 후 불변 조건이 아니다. WOFF2와 OFL 원문은 byte 불변을 요구한다.

## 5. source path dependency graph

다음은 canonical source 경로를 직접 소유하거나 참조하므로 Stage 2/3에서 갱신해야 한다.

| 영역 | 현재 참조 | 이동 후 계약 |
|------|-----------|--------------|
| protected font ignore | `.gitignore`의 `/web/fonts/*.woff2` | `/assets/fonts/*.woff2` |
| Render Diff trigger | `.github/workflows/render-diff.yml` `web/fonts/**` | `assets/fonts/**` |
| CI trigger | `.github/workflows/ci.yml` `assets/**` ignore | 비런타임 하위 경로만 ignore |
| CI frontend detector | `web/` prefix만 존재 | `assets/fonts/` 추가 |
| Chrome build | `ROOT/web/fonts` 전체 | `ROOT/assets/fonts` 전체 |
| Firefox build | `ROOT/web/fonts` 전체 | `ROOT/assets/fonts` 전체 |
| VS Code build | `../web/fonts` 11개 | `../assets/fonts` 11개 |
| Studio direct test | CanvasKit coverage가 `../web/fonts` 읽음 | `../assets/fonts` 읽음 |
| extension dist test | source list를 `web/fonts`에서 읽음 | `assets/fonts`에서 읽음 |
| metrics | inventory/license/canonical이 `web/fonts` | `assets/fonts`, 두 compatibility link metadata |
| subset tool | WOFF2 기본 출력 `web/fonts` | `assets/fonts` |
| license/package docs | root license와 npm/editor가 `web/fonts` 안내 | source와 runtime path 구분 |

현재 `assets/**` workflow ignore는 detector보다 먼저 적용된다. 따라서 detector prefix만 추가하면
font-only PR에서 CI가 시작되지 않는다. trigger ignore 축소와 detector 추가를 같은 Stage 2 변경으로 묶는다.

## 6. runtime/distribution path

다음 `fonts/...` 문자열은 source directory가 아니라 runtime URL 또는 배포 artifact 계약이므로 바꾸지 않는다.

| 소비자 | 파일 | 유지 계약 |
|--------|------|-----------|
| Studio loader | `rhwp-studio/src/core/font-loader.ts` | `fonts/<name>.woff2` |
| CanvasKit runtime | `rhwp-studio/src/view/canvaskit-renderer.ts` | `fonts/NotoSansKR-Regular.woff2` |
| legacy web | `web/editor.html` | `fonts/<name>.woff2` |
| npm guide | `npm/README.md` | CSS `/fonts/...` 예제 |
| Chrome manifest | `rhwp-chrome/manifest.json` | WAR `fonts/*` |
| Firefox manifest | `rhwp-firefox/manifest.json` | WAR `fonts/*` |
| Safari manifest | `rhwp-safari/src/manifest.json` | WAR `fonts/*` |

Stage 2 diff에서 이 표의 runtime 문자열이나 font fallback source가 바뀌면 범위 이탈로 중단한다.

## 7. 역사 reference 분류

다음 문서는 당시 canonical path와 결과를 기록하므로 `web/fonts`를 현재 경로로 일괄 치환하지 않는다.

- `mydocs/metrics/frontend/2026-07-11/**`
- #2023/#2124 plan, tech, report, feedback
- #2190 stage/report와 과거 orders
- `mydocs/plans/archives/**`, `mydocs/report/archives/**`, `mydocs/working/archives/**`
- version별 `mydocs/feedback/*amo_notes.md`

현재 운영 authority인 root license, npm/editor README, extension build/deploy manual, publish guide,
font fallback/equation 문서만 Stage 3에서 갱신한다.

## 8. target별 검증 기준

| target | source set | artifact | 검증 |
|--------|------------|----------|------|
| Studio | canonical 36개 | `rhwp-studio/dist/fonts` | 이름·bytes·hash exact |
| Chrome | canonical 36개 | `rhwp-chrome/dist/fonts` | exact parity와 기존 CSP/WAR |
| Firefox | canonical 36개 | `rhwp-firefox/dist/fonts` | Chrome/source exact parity |
| Safari | Chrome dist 36개 | `rhwp-safari/dist/fonts` | full build 후 source exact parity |
| VS Code | 승인 11개 | `rhwp-vscode/dist/media/fonts` | exact subset·hash |
| npm/editor | 직접 bundle 없음 | Studio self-hosting contract | package test와 docs |
| legacy `/web` | compatibility link | `/web/fonts/...` | relative URL 접근 가능 |

자동 검증은 `scripts/frontend-font-assets.test.mjs`를 Stage 2에서 추가하고 CI에서는 Studio,
Chrome/Firefox build와 VS Code compile 뒤 실행한다. Safari는 로컬 full build 결과를 별도로 검증한다.

## 9. Stage 2 진입 조건

- [x] current 36개 manifest와 총 bytes 고정
- [x] #2124 대비 #2190 외 미설명 delta 0
- [x] source/runtime/history reference 분류
- [x] target ownership과 copy contract 확정
- [x] CI trigger/detector 이중 누락 확인
- [x] legacy `/web` compatibility symlink 방향 확정
- [ ] 작업지시자의 Stage 2 승인

Stage 2 승인 전에는 WOFF2, symlink, build/test/tool/workflow source를 수정하지 않는다.
