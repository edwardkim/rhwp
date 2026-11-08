# 초기 PR 편집 사례24쌍의 새 소스 전쪽 검증

생산 소스 `ddf17f86476293b114b6236bfa6cf1ebc9d9d474`, base `552f5228d2dbfd6046c838f14241704f1bcb8a05`.
Native/fresh 각24쌍27쪽 전체에서 최저92.07644%, 미달/누락0, 원본 Print 쪽수 일치, exit0/source guard=true.
전체27쪽의 Native/fresh RGBA는 같고 이전 b4의 해당 전체 페이지와도 같다.
현재16개 review와 같은 좌표 확대4개를 직접 열어 표/문단/끝내용/각주·구분선/편집색/빈누름틀 소속을 확인했다.
글자 획·장평·미세 자간 및 각주 번호 크기 차이는 남는다. 통합 전체 회귀/다른 문서 전체 시각 통과와 구분한다.

정확한 입력/독립 Print SHA·두 backend 명령은 ignored v453/original-edited-current-plan.json,
전체 TSV와 쪽수는 original-edited-native|fresh/<key>/silhouette.tsv/manifest,
현재 직접 판독/페이지 RGBA/대표 panel SHA는 original-current-qualification.json,
같은 좌표 확대는 original-current-zoom-provenance.json에 있다.
이 폴더의 PNG는 새 소스에서 실제 재출력한 패널이다. 입력·PDF·배율·한컴 Print 근거는 이전 계획과 동일하다.

## PNG 해시

- `child-offset-native-p001-overlay.png` — `aee72710d42456860651becc947e0dded1d5d786cd5ed3a787364dc6bf19d03c`
- `child-offset-native-p001-review.png` — `d57253aaf22b5aa27a9a2efd6a3f0e111021dec93007b156977138b7c6e1aeaf`
- `child-offset-fresh-p001-overlay.png` — `3548325240a706e84daafcb3039698d0ddabeb887706358e560a99175a2447f3`
- `child-offset-fresh-p001-review.png` — `90fc6e38b2bf2ce7b0f524a040f69ce483647a2116ff028e7d4df16f9f2544be`
- `current-04-6b4f6a375029-native-p001-overlay.png` — `0be1569111ce2b3d141bd4a55ab5570d5e2822d114929b61bbf162b6737545ce`
- `current-04-6b4f6a375029-native-p001-review.png` — `00e1ac05af071dc34710c822f0035aa91dccdc1ebaa6e07e2b55b6b461857992`
- `current-04-6b4f6a375029-fresh-p001-overlay.png` — `f36e22b93d4e30bfc96d12ad642c7bb6ff13dffe873d72462b45e728f944b9ea`
- `current-04-6b4f6a375029-fresh-p001-review.png` — `ea5565f7b313051f3a744c6cd6339af97677ae9fd98ce011bd9f9b358919a33a`
- `current-04-6b4f6a375029-native-p002-overlay.png` — `762d4cb96f8a508a64948d14a7734a7c165c3208d5765c1682aecff4c44b7d35`
- `current-04-6b4f6a375029-native-p002-review.png` — `354cf22885144a9320f1b67a7e9753f0e2d387d88909612d9014e2c35efb4a6e`
- `current-04-6b4f6a375029-fresh-p002-overlay.png` — `83dc850a28c6d1737f0a5b277191594f2e58c1c6880dd664d81463835861b186`
- `current-04-6b4f6a375029-fresh-p002-review.png` — `9941220c1a6d2b331472b2a6758dc85a4b76a33334d115c81b410e9d2d1cd9ff`
- `current-05-17a174162fd3-native-p001-overlay.png` — `a9b987e7986210df664b4fd5f686a36a2ff85b43575668c5a0d5a2c87a9ac097`
- `current-05-17a174162fd3-native-p001-review.png` — `39396c5b40b9660f01da023179f8dcf08a70d039a41e21d008966161e6246c13`
- `current-05-17a174162fd3-fresh-p001-overlay.png` — `f042b990e70404a6ea1f030f353843aef2e3e5119e4a20ae1317a1d15b7c6839`
- `current-05-17a174162fd3-fresh-p001-review.png` — `d9e31c4c97701c24f19720205f35d4cd033ddae952ae7ea511e6a69db778b315`
- `normal-save-hwp-native-p001-overlay.png` — `4e87d3ddaecd22b82fc5f888de44a35d0def445e176ba178602e42095c2c970d`
- `normal-save-hwp-native-p001-review.png` — `91b4ea1bf0c3aa0801d339fb0e6a0f812fc8a6def49c08935a34d506b5dd961d`
- `normal-save-hwp-fresh-p001-overlay.png` — `66a949f5e277aba6089c0e7c86e991e253f9e9b0052f96b551785c2bf425ba0f`
- `normal-save-hwp-fresh-p001-review.png` — `f80c7388eb9d7a576c87243b2e17561d0fea03a450008448a0297f3f6b7d2527`
- `pr7657-after-02-8ebbea4fa234-native-p001-overlay.png` — `2d92cada27a224d97ca80323dac622650064a2a32dd36d02174f5f90dcb9ce81`
- `pr7657-after-02-8ebbea4fa234-native-p001-review.png` — `fd97bb5525551ad88d2182abad8aeadafc4fad846afc0da80d1c43dd61b7d822`
- `pr7657-after-02-8ebbea4fa234-fresh-p001-overlay.png` — `2eb814f3cb04deeb3960a9a2e76c7f5dbe25a9c59c25a6dea354881f56e9a7b8`
- `pr7657-after-02-8ebbea4fa234-fresh-p001-review.png` — `999be97f7bafcc1b05cf1210ddea44e8a6c1d350ab2e38284cfc9deea10427b5`
- `pr7657-after-10-e788c42c854c-native-p001-overlay.png` — `1b2d54409bc89a8661a7d8c11efed0eaea2e30bd695cb0f2446b3edfeb853f8a`
- `pr7657-after-10-e788c42c854c-native-p001-review.png` — `773ca50954584911600be14ceed5a3d0403cd0c84cfcf46fcf05aa0c3ddc7c95`
- `pr7657-after-10-e788c42c854c-fresh-p001-overlay.png` — `67927290266348298b87325fe85a93b0bcf037e98dd90b4c228935b401a1e988`
- `pr7657-after-10-e788c42c854c-fresh-p001-review.png` — `eb6d319663db7332cfb8efeea04a0b2120c2c9cfcab23230522be2b03ee1b339`
- `pr7657-after-11-a1805d6c9409-native-p001-overlay.png` — `668bd0936c5475ddcf6628527048c1b4480be90472aabbd055fefa32b91958f7`
- `pr7657-after-11-a1805d6c9409-native-p001-review.png` — `88f1e2b0481b136c6963bec332f1e714e32ce0d7eac37df11c8c7c1b98cf1985`
- `pr7657-after-11-a1805d6c9409-fresh-p001-overlay.png` — `807fea5f8cbe6f3a8614953d2e732e6f90460736d0de876fad068f964c16932d`
- `pr7657-after-11-a1805d6c9409-fresh-p001-review.png` — `57635138c2ffc6e29fbf2a05525aff3dffcab6c6ba6626f6c3ea0b8cba6bf8eb`
- `current-04-6b4f6a375029-native-footnote-zoom.png` — `74cbe14df6d1d9883d8868919bdec1c409666d5afdcec0b7155f2146d465479a`
- `current-04-6b4f6a375029-fresh-footnote-zoom.png` — `74cbe14df6d1d9883d8868919bdec1c409666d5afdcec0b7155f2146d465479a`
- `pr7657-after-02-8ebbea4fa234-native-body-zoom.png` — `985fb95edfccc773721a8535678add7b926910595e70d77c3dd2a5888a43ce27`
- `pr7657-after-02-8ebbea4fa234-fresh-body-zoom.png` — `985fb95edfccc773721a8535678add7b926910595e70d77c3dd2a5888a43ce27`
