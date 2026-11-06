# 실제 fresh WASM 로드 글꼴

시각 검증은 시스템 전체 1,906 face catalogue와 입력별 대체 설정을 사용한다.
프로그램 자체는 저장소나 배포물에 넣지 않는다. 회귀 테스트의 최소 공개 글꼴 5개 환경과 구분한다.
아래 ID는 measurements.md의 각 입력에서 실제 로드한 face에 대응한다.

| ID | family / 등록명 | index·weight·style | 원본 프로그램 SHA-256 | 실제 text 프로그램 SHA-256 | 정본 공급기의 수리 |
| --- | --- | --- | --- | --- | --- |
| F1 | Apple SD Gothic Neo / Apple SD Gothic Neo | 0·100·normal | `ac63f87d7713492ec587a2a2b8404518a748b4bf65d9ef30ff4bc61f309552a6` | `ac63f87d7713492ec587a2a2b8404518a748b4bf65d9ef30ff4bc61f309552a6` | 없음 |
| F2 | GulimChe / GulimChe | 0·400·normal | `93e4c77ebe4e4ca5f0b7405bec1b26e8b4465dd5ab2fc36aabdf9ca02f0ef23c` | `93e4c77ebe4e4ca5f0b7405bec1b26e8b4465dd5ab2fc36aabdf9ca02f0ef23c` | 없음 |
| F3 | HCI Poppy / HCI Poppy | 0·400·normal | `3fb64d42c25d035b7b7cc3bb286b54ae265eceb49f9f713030a32129edf68385` | `3fb64d42c25d035b7b7cc3bb286b54ae265eceb49f9f713030a32129edf68385` | 없음 |
| F4 | HCI Poppy / HCI Poppy | 0·400·normal | `ad78b2ee6458dcbef0b373b1ca7b3ddfb6ddd1610eda331ebd88d486e0556341` | `ad78b2ee6458dcbef0b373b1ca7b3ddfb6ddd1610eda331ebd88d486e0556341` | 없음 |
| F5 | HCR Batang Ext / HCR Batang Ext | 0·400·normal | `f88d957a26dd5a6360424e37b6d25b4192a83d922b7ac1fa9fb92d34e579afe4` | `f88d957a26dd5a6360424e37b6d25b4192a83d922b7ac1fa9fb92d34e579afe4` | 없음 |
| F6 | HCR Batang ExtB / HCR Batang ExtB | 0·700·normal | `594860a83ffb618c76f1a08ab1708b2ad62bf533c919065e2527d7fbe69c497d` | `594860a83ffb618c76f1a08ab1708b2ad62bf533c919065e2527d7fbe69c497d` | 없음 |
| F7 | HCR Batang ExtB / HCR Batang ExtB | 0·400·normal | `c9c8ec1f6c295be9254157fdbddef9df93f5f4b6c55e8683214b50368c50c381` | `c9c8ec1f6c295be9254157fdbddef9df93f5f4b6c55e8683214b50368c50c381` | 없음 |
| F8 | HCR Batang / HCR Batang | 0·700·normal | `0d355f9766ffd1c7ec7648260611ddea9e12ea3c610423e552d52b921fc0f29f` | `0d355f9766ffd1c7ec7648260611ddea9e12ea3c610423e552d52b921fc0f29f` | 없음 |
| F9 | HCR Batang / HCR Batang | 0·400·normal | `c5d544769d832477d48b7318222b9c3c2eced25455f7ba9acb23060dcecac104` | `c5d544769d832477d48b7318222b9c3c2eced25455f7ba9acb23060dcecac104` | 없음 |
| F10 | HCR Dotum / HCR Dotum | 0·400·normal | `1d0437906d52e96af36943008f144ad74d6e994493df3a4210a9761e077ea898` | `1d0437906d52e96af36943008f144ad74d6e994493df3a4210a9761e077ea898` | 없음 |
| F11 | HCR Dotum / HCR Dotum | 0·700·normal | `5b0b9fa5f3ba2f7b2e3576c6c2e2f2a7a92f68c98f8270e10791ceb363ce1e51` | `5b0b9fa5f3ba2f7b2e3576c6c2e2f2a7a92f68c98f8270e10791ceb363ce1e51` | 없음 |
| F12 | HYHeadLine M / HYHeadLine M | 0·400·normal | `a3f57b4cb4292977479c1511b52997daadaf04c37f180ff05ab152c4d12ccaf5` | `a3f57b4cb4292977479c1511b52997daadaf04c37f180ff05ab152c4d12ccaf5` | 없음 |
| F13 | HYHeadLine-Medium / HYHeadLine-Medium | 0·400·normal | `cb627bb12c6b1d2ef3583a24be6dd5159daf192cba10ee4d1e9fd02cab60e5d6` | `cb627bb12c6b1d2ef3583a24be6dd5159daf192cba10ee4d1e9fd02cab60e5d6` | 없음 |
| F14 | HY견고딕 / HY견고딕 | 0·400·normal | `82f70d5e221412b10df0431f353f095885888823f5d026694691237c9a3004e8` | `c370326241e3e11b250ff6429a79a6c1ec022629debbbe1cd9fbed3c04ee014f` | cmap-format4-terminal-notdef |
| F15 | HY견고딕 / __rhwp_ttf__HY견고딕 | 0·400·normal | `82f70d5e221412b10df0431f353f095885888823f5d026694691237c9a3004e8` | `None` | cmap-format4-terminal-notdef |
| F16 | HY견명조 / HY견명조 | 0·400·normal | `a101a7a219ffd57273374a423e734adfba1529e5eaf2e9a7476f7150708ea506` | `90926c28ef4b4f249a7548f9d52cb744e91998b46dfe373c6b39d85beb316b22` | cmap-format4-terminal-notdef |
| F17 | HY견명조 / __rhwp_ttf__HY견명조 | 0·400·normal | `a101a7a219ffd57273374a423e734adfba1529e5eaf2e9a7476f7150708ea506` | `None` | cmap-format4-terminal-notdef |
| F18 | HY헤드라인M / HY헤드라인M | 0·400·normal | `a3f57b4cb4292977479c1511b52997daadaf04c37f180ff05ab152c4d12ccaf5` | `a3f57b4cb4292977479c1511b52997daadaf04c37f180ff05ab152c4d12ccaf5` | 없음 |
| F19 | HY헤드라인M / __rhwp_ttf__HY헤드라인M | 0·400·normal | `a3f57b4cb4292977479c1511b52997daadaf04c37f180ff05ab152c4d12ccaf5` | `None` | 없음 |
| F20 | Haansoft Batang / Haansoft Batang | 0·400·normal | `35f84328500fc2c3ee0b148aa75de0ed384bf9eee8dee9148c01b0a11a27fe05` | `e391734bd989a8c45fe671ebcf3a4960985a0dfb1c5638209b55a3dd3f5517df` | cmap-format4-terminal-notdef, drop-embedded-bitmap-strikes |
| F21 | Haansoft Batang / __rhwp_ttf__Haansoft Batang | 0·400·normal | `35f84328500fc2c3ee0b148aa75de0ed384bf9eee8dee9148c01b0a11a27fe05` | `None` | cmap-format4-terminal-notdef, drop-embedded-bitmap-strikes |
| F22 | Malgun Gothic / Malgun Gothic | 0·400·normal | `6706a05313c7fafb3fd8fda9538c9ecabac7cede30076b3ea85be7716f356702` | `6706a05313c7fafb3fd8fda9538c9ecabac7cede30076b3ea85be7716f356702` | 없음 |
| F23 | Malgun Gothic / Malgun Gothic | 0·700·normal | `a541834fb9cdde9bd6d2c3ff1565cb0bb5ccf7a15e4e9daf2cb58d9b1cf282aa` | `a541834fb9cdde9bd6d2c3ff1565cb0bb5ccf7a15e4e9daf2cb58d9b1cf282aa` | 없음 |
| F24 | NanumGothic / NanumGothic | 0·600·normal | `454eb3f503f377782a99eb84fc4bb7dde22a0f075d41b47427737f2985053bc9` | `29cb423610c71bcbbbbbbc15f8cfd504ba923bf2b5af659f29ca2dca4311a97a` | empty-vtt-source-table:TSI3 |
| F25 | NanumGothic / NanumGothic | 0·700·normal | `e0288775c8f021f327fd77f9eb251eea80e4601bbbb068478c7878caa23e8f92` | `80cbc937f3412b56e0a90a0d959ac14ab46db7bbe915028fda1a7ab97ed06c2a` | empty-vtt-source-table:TSI3 |
| F26 | NanumGothic / NanumGothic | 0·400·normal | `c47182a81692bcbf83fb1a059e6b25b2b73027e8ca6b9585efece5160c8c4446` | `c47182a81692bcbf83fb1a059e6b25b2b73027e8ca6b9585efece5160c8c4446` | 없음 |
| F27 | NanumGothic / NanumGothic | 0·300·normal | `da39544c39be58124a4fcb296eafc2bf73530b3dbd1b19aa802c5c3fd227e31c` | `da39544c39be58124a4fcb296eafc2bf73530b3dbd1b19aa802c5c3fd227e31c` | 없음 |
| F28 | Noto Color Emoji / Noto Color Emoji | 0·400·normal | `93cdc4ee9aa40e2afceecc63da0ca05ec7aab4bec991ece51a6b52389f48a477` | `93cdc4ee9aa40e2afceecc63da0ca05ec7aab4bec991ece51a6b52389f48a477` | 없음 |
| F29 | Noto Sans Mono / Noto Sans Mono | 0·400·normal | `6b692c4b6d15ccf59f1c1fe8d11cb8a92f51960f3e9f1f523781755a3af7e29f` | `6b692c4b6d15ccf59f1c1fe8d11cb8a92f51960f3e9f1f523781755a3af7e29f` | 없음 |
| F30 | Noto Sans Mono / Noto Sans Mono | 0·700·normal | `b22563d5f03b345a62115e78d8789c9ed99c2cfa8059fad6e7a5c4ced288e21e` | `b22563d5f03b345a62115e78d8789c9ed99c2cfa8059fad6e7a5c4ced288e21e` | 없음 |
| F31 | Pretendard / Pretendard | 0·100·normal | `7cc3da25615621a88fa85af4e4556a5d30cf709ad41785e62ed821877ce6c4fa` | `7cc3da25615621a88fa85af4e4556a5d30cf709ad41785e62ed821877ce6c4fa` | 없음 |
| F32 | Segoe UI Emoji / Segoe UI Emoji | 0·400·normal | `7c0244dd8eeb7c6bdecdfc3f9e59833527fc18a66d0295ce47339069692a2b4f` | `4206bc9cf9da537d39d5154a324517bda2751fa600cb901daf58f3de4af961d8` | 없음 |
| F33 | 굴림체 / 굴림체 | 0·400·normal | `93e4c77ebe4e4ca5f0b7405bec1b26e8b4465dd5ab2fc36aabdf9ca02f0ef23c` | `93e4c77ebe4e4ca5f0b7405bec1b26e8b4465dd5ab2fc36aabdf9ca02f0ef23c` | 없음 |
| F34 | 굴림체 / __rhwp_ttf__굴림체 | 0·400·normal | `93e4c77ebe4e4ca5f0b7405bec1b26e8b4465dd5ab2fc36aabdf9ca02f0ef23c` | `None` | 없음 |
| F35 | 나눔고딕 / 나눔고딕 | 0·600·normal | `454eb3f503f377782a99eb84fc4bb7dde22a0f075d41b47427737f2985053bc9` | `29cb423610c71bcbbbbbbc15f8cfd504ba923bf2b5af659f29ca2dca4311a97a` | empty-vtt-source-table:TSI3 |
| F36 | 나눔고딕 / 나눔고딕 | 0·700·normal | `e0288775c8f021f327fd77f9eb251eea80e4601bbbb068478c7878caa23e8f92` | `80cbc937f3412b56e0a90a0d959ac14ab46db7bbe915028fda1a7ab97ed06c2a` | empty-vtt-source-table:TSI3 |
| F37 | 나눔고딕 / 나눔고딕 | 0·400·normal | `48a28e97b34fc8e5b157657633670cd1b7de126cfc414da65ce9c3d5bc8be733` | `badb9944d99fc61dc918977b6bf3f8989904665c1205afc1402ed75126c56d79` | empty-vtt-source-table:TSI3 |
| F38 | 나눔고딕 / 나눔고딕 | 0·300·normal | `da39544c39be58124a4fcb296eafc2bf73530b3dbd1b19aa802c5c3fd227e31c` | `da39544c39be58124a4fcb296eafc2bf73530b3dbd1b19aa802c5c3fd227e31c` | 없음 |
| F39 | 나눔고딕 / __rhwp_ttf__나눔고딕 | 0·600·normal | `454eb3f503f377782a99eb84fc4bb7dde22a0f075d41b47427737f2985053bc9` | `None` | empty-vtt-source-table:TSI3 |
| F40 | 나눔고딕 / __rhwp_ttf__나눔고딕 | 0·700·normal | `e0288775c8f021f327fd77f9eb251eea80e4601bbbb068478c7878caa23e8f92` | `None` | empty-vtt-source-table:TSI3 |
| F41 | 나눔고딕 / __rhwp_ttf__나눔고딕 | 0·400·normal | `48a28e97b34fc8e5b157657633670cd1b7de126cfc414da65ce9c3d5bc8be733` | `None` | empty-vtt-source-table:TSI3 |
| F42 | 나눔고딕 / __rhwp_ttf__나눔고딕 | 0·300·normal | `da39544c39be58124a4fcb296eafc2bf73530b3dbd1b19aa802c5c3fd227e31c` | `None` | 없음 |
| F43 | 맑은 고딕 / 맑은 고딕 | 0·400·normal | `6706a05313c7fafb3fd8fda9538c9ecabac7cede30076b3ea85be7716f356702` | `6706a05313c7fafb3fd8fda9538c9ecabac7cede30076b3ea85be7716f356702` | 없음 |
| F44 | 맑은 고딕 / 맑은 고딕 | 0·700·normal | `a541834fb9cdde9bd6d2c3ff1565cb0bb5ccf7a15e4e9daf2cb58d9b1cf282aa` | `a541834fb9cdde9bd6d2c3ff1565cb0bb5ccf7a15e4e9daf2cb58d9b1cf282aa` | 없음 |
| F45 | 맑은 고딕 / __rhwp_ttf__맑은 고딕 | 0·400·normal | `6706a05313c7fafb3fd8fda9538c9ecabac7cede30076b3ea85be7716f356702` | `None` | 없음 |
| F46 | 맑은 고딕 / __rhwp_ttf__맑은 고딕 | 0·700·normal | `a541834fb9cdde9bd6d2c3ff1565cb0bb5ccf7a15e4e9daf2cb58d9b1cf282aa` | `None` | 없음 |
| F47 | 한컴바탕 / 한컴바탕 | 0·400·normal | `35f84328500fc2c3ee0b148aa75de0ed384bf9eee8dee9148c01b0a11a27fe05` | `e391734bd989a8c45fe671ebcf3a4960985a0dfb1c5638209b55a3dd3f5517df` | cmap-format4-terminal-notdef, drop-embedded-bitmap-strikes |
| F48 | 함초롬돋움 / 함초롬돋움 | 0·400·normal | `1d0437906d52e96af36943008f144ad74d6e994493df3a4210a9761e077ea898` | `1d0437906d52e96af36943008f144ad74d6e994493df3a4210a9761e077ea898` | 없음 |
| F49 | 함초롬돋움 / 함초롬돋움 | 0·700·normal | `5b0b9fa5f3ba2f7b2e3576c6c2e2f2a7a92f68c98f8270e10791ceb363ce1e51` | `5b0b9fa5f3ba2f7b2e3576c6c2e2f2a7a92f68c98f8270e10791ceb363ce1e51` | 없음 |
| F50 | 함초롬돋움 / __rhwp_ttf__함초롬돋움 | 0·400·normal | `1d0437906d52e96af36943008f144ad74d6e994493df3a4210a9761e077ea898` | `None` | 없음 |
| F51 | 함초롬돋움 / __rhwp_ttf__함초롬돋움 | 0·700·normal | `5b0b9fa5f3ba2f7b2e3576c6c2e2f2a7a92f68c98f8270e10791ceb363ce1e51` | `None` | 없음 |
| F52 | 함초롬바탕 확장 / 함초롬바탕 확장 | 0·400·normal | `f88d957a26dd5a6360424e37b6d25b4192a83d922b7ac1fa9fb92d34e579afe4` | `f88d957a26dd5a6360424e37b6d25b4192a83d922b7ac1fa9fb92d34e579afe4` | 없음 |
| F53 | 함초롬바탕 확장B / 함초롬바탕 확장B | 0·700·normal | `594860a83ffb618c76f1a08ab1708b2ad62bf533c919065e2527d7fbe69c497d` | `594860a83ffb618c76f1a08ab1708b2ad62bf533c919065e2527d7fbe69c497d` | 없음 |
| F54 | 함초롬바탕 확장B / 함초롬바탕 확장B | 0·400·normal | `c9c8ec1f6c295be9254157fdbddef9df93f5f4b6c55e8683214b50368c50c381` | `c9c8ec1f6c295be9254157fdbddef9df93f5f4b6c55e8683214b50368c50c381` | 없음 |
| F55 | 함초롬바탕 / 함초롬바탕 | 0·700·normal | `0d355f9766ffd1c7ec7648260611ddea9e12ea3c610423e552d52b921fc0f29f` | `0d355f9766ffd1c7ec7648260611ddea9e12ea3c610423e552d52b921fc0f29f` | 없음 |
| F56 | 함초롬바탕 / 함초롬바탕 | 0·400·normal | `c5d544769d832477d48b7318222b9c3c2eced25455f7ba9acb23060dcecac104` | `c5d544769d832477d48b7318222b9c3c2eced25455f7ba9acb23060dcecac104` | 없음 |
| F57 | 함초롬바탕 / __rhwp_ttf__함초롬바탕 | 0·700·normal | `0d355f9766ffd1c7ec7648260611ddea9e12ea3c610423e552d52b921fc0f29f` | `None` | 없음 |
| F58 | 함초롬바탕 / __rhwp_ttf__함초롬바탕 | 0·400·normal | `c5d544769d832477d48b7318222b9c3c2eced25455f7ba9acb23060dcecac104` | `None` | 없음 |
| F59 | 휴먼명조 / 휴먼명조 | 0·400·normal | `d033df71fa41de48905407f7d39aca74ebd9ae87298c5420513acaa187fbefc6` | `e48a43385fc11e9460c905aa2e19780933307d85e56dcb0e3bfd9e2d4a9cfcf1` | drop-embedded-bitmap-strikes |
| F60 | 휴먼명조 / 휴먼명조 | 0·400·normal | `99760ec20c65744bb9ae997b3b9f6b28ae4eebac7a80b202e24a32430537c33d` | `99760ec20c65744bb9ae997b3b9f6b28ae4eebac7a80b202e24a32430537c33d` | 없음 |
| F61 | 휴먼명조 / __rhwp_ttf__휴먼명조 | 0·400·normal | `d033df71fa41de48905407f7d39aca74ebd9ae87298c5420513acaa187fbefc6` | `None` | drop-embedded-bitmap-strikes |
| F62 | Noto Serif CJK KR / Noto Serif CJK KR | 1·400·normal | `93069d8e9e45d515cc421c971a79e6a5777704b348e36a9ef86578bf58adef77` | `35ff7a6d2e4ac94f1a3198f880877000c032e93b8ae2b43b2b1a975e14754d10` | 없음 |
| F63 | Noto Serif CJK KR / Noto Serif CJK KR | 1·900·normal | `f9ea56d72ca0f7fb6cbb550eb40b1002686f10fb9173bbca80210caa12e7b861` | `365c5bb2e5b48738bcb4155dc01348375c96e73a7b2a9bff38af5114fd8d2683` | 없음 |
| F64 | Noto Serif CJK KR / Noto Serif CJK KR | 1·700·normal | `9d3c0d769c703cfe41c49cbe2b14d4718e26545593dc247485486335b783fe94` | `4004a4be65b59e7af18e0cc423149a2c84d4d9007d28790bf8bf367f86e3b8a5` | 없음 |
| F65 | Noto Serif CJK KR / Noto Serif CJK KR | 1·600·normal | `b313b12c4ca6bffd67fced7ace2050bc70c9d6768327157636f8cef7d5aa2c8d` | `54022ece535ff009d31d311b88d60c3764149f96a6b1172eaefefd319b53f34e` | 없음 |
| F66 | Noto Serif CJK KR / Noto Serif CJK KR | 1·200·normal | `f40f667457544a3e9621234080c0ba40e9626d51d00f84441ca6b05b65f921ee` | `6eb56f1210e2dde68240ef48610d4f20c4ac7d8ec0c6890c088dcb5379e4b355` | 없음 |
| F67 | Noto Serif CJK KR / Noto Serif CJK KR | 1·300·normal | `bba932b49cca42f41eb90ed699ec03832767cc1b743781ca070e2ac81263b0fb` | `829b1799214939cb60b9467869c8192815b40ed276bbe3832f295dc82b023568` | 없음 |
| F68 | Noto Serif CJK KR / Noto Serif CJK KR | 1·500·normal | `9a9bec9cdcbb187384748656fa8e63f3c29b30c84e20fb91ec0af944e2bc2a29` | `f879b85495b27bdd084ce0346657a250bdb421e30702a93b4eb747d32dddddb1` | 없음 |

## 같은 WOFF 재변환의 조건 검토

원본 WOFF SHA-256 `c9a1e3ac69994d680883b53b1fd83d520733b704792522a932bfaa276117de45`는 같다.
현재 SFNT의 19개 raw table을 독립 WOFF 복원 테이블과 대조했고 head의 checksumAdjustment·시각을
정규화한 뒤 모두 같았다. 이전 개별 변환 SFNT bytes가 남아 있지 않아 그것을 직접 열었다고 주장하지 않는다.
이 특정 재저장 metadata 차이만 검토했으며 다른 프로그램·face·대체 선택 변경을 면제하지 않았다.
Native 8개 조건 지문과 실제 시스템 catalogue의 동일성을 따로 확인했다.
