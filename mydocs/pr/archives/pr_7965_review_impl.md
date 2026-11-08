# PR #7965 구현·통합·후속 처리

## 현재 단계

로컬 필수 검증과 정확한 제출 code head `228db2f278b52bc73fe2c5b5e0bb9a09b2ef4ff7`의 CI 성공을 확인했습니다. 원 PR source는 수정하지 않고 통합 PR #8148의 문서 trailing으로 기록합니다. 남은 조건은 trailing head의 CI·merge·후속 처리입니다.

## 기준과 범위

- 원 PR #7965: `690290fa9d06c6f19f18a753900aa3555e1de1c0`; 현재 기능 tip `690290fa9d06c6f19f18a753900aa3555e1de1c0`.
- base `a6e9d18fc1061318055b4eb348daec201427ea03`; 제출 code head `228db2f278b52bc73fe2c5b5e0bb9a09b2ef4ff7` / 실제 검증 runtime `3cb5a4c290663343123074be1e2fc0dda18d3db4` / 새 검사 등록 `e25dbd0f5d47bc32686016c08054d2bf718be6a7`.
- 기본 경로: collaborator 매개 외부 PR. 보조 경로: intake_and_review, local_validation, visual_fixture_evidence, multi_pr_update_branch, post_merge. 같은 통합 PR의 trailing 기록에 포함하며 통합 PR 번호용 별도 review나 추가 문서 PR은 만들지 않습니다.
- 원 변경의 실제 범위·전쪽 TSV·직접 판독·기존 이관·최종 판정은 같은 번호의 review와 공통 검증 원장에 연결합니다.

## 원 기여의 현재 대응

| 원 source SHA | 현재 통합 SHA | 실제 제목 |
| --- | --- | --- |
| `eec9ee49283298a076e66f2b94832ca47440147b` | `a14ffe9f91ab10003f9c4870634d2bccaa9ca695` | fix(조판): 첫 개체 프레임 뒤 행의 원점0 재시작은 그 행이 쪽 끝에 걸릴 때만 자른다 |
| `1d23ddebe288bd39147b3f68216dfc2e870545a6` | `27ae5a1ce06aaf0c8ed51aefe7b4c9bdaaca18cc` | fix(조판): 첫 분할 행의 이어받기 물리 잔여는 선언 행 상자를 넘지 않는다 |
| `0f0d9f5346dd1e39602d099f209f6296bbd4b627` | `8a64eea4f3e31705c013dc99afb9c1f9aa9bb526` | style(조판): 원점0 인증의 let-else 를 ? 로 정리 (clippy) |
| `c382d42a18e9558704c43006a3bf3b9547a92f09` | `6cd1fb72aec633d6257910bbb1f4c4860a70a4d0` | 문서: devel 표 이어받기 회귀 수정 대표 쪽 review·overlay PNG (Native review · fresh WASM overlay) |
| `2ca278d61a1c4434519cd5b1051585c277b77741` | `afb18538a28ce4b9d0083d572fdb9367ee1972b5` | fix(조판): 저장 vpos 가 되감긴 atomic TAC 그림은 하단 여백으로 스필하지 않는다 |
| `9853e8a1b8cf68f1d663540eca1378e1e1ea21b5` | `54c01b3420335d39507fd6252537e824118a7a91` | fix(조판): RowBreak 표의 0행 rowspan 묶음은 0행만 들 때 0행 뒤에서 자른다 |
| `0b5b529124be228ff9dc648cdc8027e5d0fb7b13` | `1a2a27c940e1e6aaa32fa3b015780f0f1c80e10a` | style(시험): tac_picture_stored_rewind_next_page rustfmt 줄바꿈 |
| `ec006b0357ebee98313c6e00c2cda223671d11aa` | `c2847bc6efbccf4752f6a60514c4b456a244dfba` | fix(조판): 저장 근거가 증언한 행의 빈 띠를 다음 쪽으로 넘기고 그 끝 조각의 아래 여백을 예약한다 |
| `b2db4f211de5980f03d88b447adb9d9bd0ca4b20` | `2500a13db53cf22b07d2b8d9e493f7d817906a6f` | fix(조판): 걸침 칸 없는 짧은 쪽 끝 행은 통째로 넘기고, 블록 끝 행의 빈 띠는 쪽 경계에서 닫는다 |
| `9477a974af5e36d7a590d60e2bbc06dbf8add643` | `e21f05d652fdb3640a1c4a6628feb8dddd94b68e` | fix(조판): 블록 끝 행 닫기는 선언 초과가 말미 행 압축 허용치 이내일 때만 |
| `48a310eac8c924838b39974c288c06e91be49305` | `90f3fa3e6a2b6effebdacdc8d35cacf6bec04c92` | fix(조판): 쪽 마지막 행이 셀 저장 줄의 새 원점 재시작을 가지면 그 줄에서 자른다 |
| `40ef3c285a03e16c42f2ce8cdbcca7a6e01339d0` | `b08304df1f2857a458658a3ebd565ff7a7677f46` | fix(조판): 분할 표 뒤 빈 줄은 다음 문단의 저장 위치가 자리를 남기면 그린다 |
| `f1d2608f9af9c80745284639fb5e94f64f75efe3` | `f863563c100da4b7ee5d3d774bf70e9f35097efb` | fix(조판): 걸침 칸의 미지 행은 가장 먼저 끝나는 제약이 정하고, 쪽을 넘어온 칸의 남은 높이를 끝 행에 준다 |
| `737ffc061c5824c505365a3ede15cb3812ee5d6e` | `d08e940987cc458f38a3852bb60053137cba0293` | fix(조판): 문단 경계 저장 쪽 재시작으로 끝나는 분할 행도 원본 조각으로 보고 칸 정렬을 따른다 |
| `36e861af2c8036a9b1cb733d80c6138cf403f566` | `2e6ac3c673aaa86cd4ddda9a3b57268f5cb14946` | 문서: devel 표 이어받기 회귀 수정 대표 쪽 review·overlay PNG 와 전쪽 TSV 갱신 (head 0971bb232) |
| `af74e9bdbab5eb1ed6397b86d46ed0950e1c9d86` | `89bfb3d988f4673c2ab538a42b13965b0eb5c67f` | fix(조판): 한/글 저장 쪽 경계에서 끊긴 걸침 행도 쪽 끝 띠를 갖고, 다음 조각이 그 상자를 이어받는다 |
| `237be13f701c3285e37a83da19bf4fd56d65f76b` | `6c9de5483b6ae5a0dc8d43ffe4a4b0ccb16de0fb` | 문서: devel 표 이어받기 회귀 수정 대표 쪽 review·overlay PNG 와 전쪽 TSV 갱신 (head c23a1fa7f) |
| `6b4e2baa908256ee3145fa6e78b6048a6796d0e5` | `c3109146936a340b423b87bb075fb2a01f8f41ee` | fix(조판): 끝 행을 지나는 걸침 칸의 쪽 끝 띠는 컷 종류와 무관하게 남는 몫까지 준다 — 1342000 416→415쪽 |
| `dd54037422ec0c8f0df00b654921d35f79f14a88` | `e57e11f308d08c9228dcec3bd1e27892ef081fbd` | 문서: 대표 쪽 review·overlay PNG 와 전쪽 TSV 갱신 (devel 515bd72dc 위 head 6b4e2baa9) |
| `f95faf3a57cccc1304ab253fa8db40b3e8442ea4` | `e0c2ca84f845b727815b2ae892e45625601685cf` | style(조판): 문단 유지 계약 판정의 let-else 를 ? 로 정리 (clippy::question_mark) |
| `27bd84bf31357583eb1fab956aa0d3f1b65c15ab` | `ca81783205b7e5176808663fc51b00e39350fabe` | fix(조판): 행 경계를 닫는 자란 행은 줄이지 않고, 끝 조각 뒤 문단은 저장 줄을 따른다 — 1613000 185–190쪽 |
| `4c83c6112a038cf0e30ee164e12c4c0a98f4e6af` | `e9ad31d033380c496165faaa50cd81614acfecc1` | 문서: 1613000 186·188쪽·issue1880 1쪽 대표 PNG, 전쪽 TSV, issue1880 한컴 2024 PDF (head 27bd84bf3) |
| `5e7ee4f2d5d8a40b6f45fc84d84f898a1fc8964f` | `49a7346fdb0451af77d20a4ac3bd91d652382b3c` | fix(조판): 새 쪽에 통째로 든 표도 끝 조각처럼 뒤 문단을 저장 줄에 둔다 — 1613000 191–193쪽 |
| `06ae02fd2235c5acb14a342371039d8c25ec8af7` | `e560389ab4da6805f87d59cf2de67e7ef9d7546b` | 문서: 1613000 191·192쪽 대표 PNG, 전쪽 TSV 갱신 (head 5e7ee4f2d) |
| `3bb5ca91390f23740e68cb08c555ab8410b5c46a` | `96b14feb9302ee7b1c1e8b51c17e804e763a127e` | fix(조판): 저장 재시작 컷의 쪽 끝 행은 선언·컷 내용 상자로 닫고, 끝 조각 캡션은 통째 표 간격을 쓴다 — 1613000 126–128쪽 |
| `bb8dff08d3b9e2ca883e1ecd69950e1e84a0ccc1` | `c136c6c3cee0c6094033bc0b842429b0fae2de27` | 문서: 1613000 127·128쪽 대표 PNG, 1613000·1480000 전쪽 TSV 갱신 (head 3bb5ca913) |
| `dcf10726ccac60d5753fb2aaccd5e3519126d5c1` | `88f3aa3f05d3af6b341d0a589fad3fc84a3d69ed` | fix(조판): 걸침 칸 안 쪽 끝 행은 내용이 비끝 조각 닫는 선 안에 들면 그 쪽에서 닫는다 — 1613000 195–202쪽 |
| `25676d3a5d66de198171d0e3abda80a51ab0067c` | `3b3abd9ae49727a5d1fa370e0bb0adb301a2774c` | 문서: 1613000 196·201쪽 대표 PNG, 1613000·1342000 전쪽 TSV 갱신 (head dcf10726c) |
| `67081c132bf95671c9f35151756e1fc58c285922` | `01b1dea1b650d2e59c9f3f9c3bb8072cb09a2c88` | fix(조판): 통째로 받은 끝 행의 쪽 끝 띠는 비끝 조각 닫는 선에서 그리고, 걸침 몫 이월은 줄이기 전 띠로 센다 |
| `b1c09d949f64ada061b5b27abd3ee98336b9c103` | `d8b8c6345207af107f97bf9ed8f70f7c68b56742` | 문서: 1613000 195쪽 대표 PNG, 1613000·1342000·80168 전쪽 TSV 갱신 (head 67081c132) |
| `9503498d13560409e34284fc67c223e911e7d0d0` | `0024df763a009c146aabf4960662aa9c34a290b4` | fix(조판): 걸침 칸 안 쪽 끝 행 상자는 max(내용, 선언, 걸침 칸 몫)을 닫는 선까지 — 1613000 139–183쪽 |
| `90b8a2dcac0443826d1bfb4828839906958c80bd` | `2e286309dc491301b8d11528f20d77dd8ab91089` | 문서: 1613000 144·154쪽 대표 PNG, 1613000·1342000 전쪽 TSV 갱신 (head 9503498d1) |
| `f932843889a7dae1034d629042540a72056557e8` | `893ad5822f7148e93d871422629f429b77eceec9` | fix(조판): 이어받은 행은 셀 저장 줄의 다음 새 원점 재시작에서 닫는다 — 1613000 123–125쪽 |
| `38eb5be034b231acbe55fc1ea3ec78d7c6b949e5` | `687d7e303c74b78ed953d8bfe64f677ae51f27ce` | fix(조판): 빈 호스트 표 끝 조각 뒤 문단도 상자 아래와 같은 저장 줄에 놓는다 — 1613000 125쪽·task2070 |
| `13e49119559e9030cc93f72f331fc84244655e2c` | `e6851a676dadfd9a0f32303e99b40533108f2132` | 문서: 1613000 124·125쪽 대표 PNG, 1613000 전쪽 TSV 갱신 (head 38eb5be03) |
| `81f7bad00bfc236ca30aee4f4caa9b3c5065d06a` | `9fb4c62024e4995f7cef63bd1cacc9223d422227` | fix(조판): 1×1 상자 안 중첩 표가 쪽을 넘으면 상자를 가르고 저장 첫 조각 경계에서 자른다 — 1613000 58·59쪽 |
| `1a30b64a134d0b033b1ca598a88add86a6414238` | `8d2366528562bcfc9c186b8af109fa3dec2be42a` | 문서: 1613000 58·59쪽 대표 PNG, 1613000 전쪽 TSV 갱신 (head 81f7bad00) |
| `741a869f062d36baaea7cb035c6e34bb1d4ff1cb` | `52ee896770f8f31ad1052395202df6dcbdecb682` | fix(조판): 빈 꼬리말 밴드가 각주를 모두 받으면 각주 보수 여백이 각주 높이를 넘어 본문을 깎지 않는다 — 1613000 85쪽 |
| `b84e205002f8492b5b645423dc47692b070a8408` | `1d193bc337c90eedd8e761aee2400d885ee3f5ab` | 문서: 1613000 85쪽 대표 PNG, 1613000 전쪽 TSV 갱신 (head 741a869f0) |
| `9914b3ce949078a834c363b507c7b881c8a35623` | `2e1e501a446ff215bf2b7c2be782d9298926ad82` | fix(조판): 첫 조각 프레임만 저장한 원본 표에는 가로 용지 경계 행 허용치를 쓰지 않는다 — 1613000 198쪽 |
| `30d75e6ee1df030d2fcba3435c0f6b141f75c40f` | `eafbb040dcb17f2b235708a9ab6e1727bb61aaea` | 문서: 1613000 198쪽 대표 PNG, 1613000 전쪽 TSV 갱신 (head 9914b3ce9) |
| `39a5b201a69dfb75bbe72125da2c466e0ee08f98` | `856b9467b84f17bcd2ef520f3c3ac2a4c0d8b81d` | fix(조판): 저장 글줄이 없는 셀도 문단 끝 컷은 행 분할 경계로 쓴다 — 80168 61쪽 |
| `21ab9623df80fb2619264060831d0c706e9de4e4` | `fb7874d3a991bab7419e3bfcf47effdb49c7b88e` | 문서: 80168 61쪽 대표 PNG, 80168 전쪽 TSV 갱신 (head 39a5b201a) |
| `50dcb007820cae9f273f758d34e09cd369bef502` | `9bca16c4a76c7f0ff52a36e7f30d71d5a82dc468` | fix(조판): 리베이스 후 쪽 끝 띠의 방을 실제 예약 높이와 프레임 inset 에서 잰다 |
| `f8fa0f02ac8eaaf4f9ab28ff604463937986684a` | `7b585a6120f429576fcb59667167221bce9a266d` | fix(조판): 병합 칸 HWP5 저장 표의 예산은 inset 없이, 쪽 끝 띠·걸침 예약은 그린 상자와 같은 값으로 |
| `065125a9479107706577f91bdf5e307affda0f4a` | `0c479111667a63c378307ff60d96674251bd46b5` | fix(조판): 한/글 2020 이전 저장 줄의 낱말 꼬리를 condense 흡수로 다시 끊는다 (#7702) |
| `da062b3aa24d50a5670b239519afd5606e2f1264` | `4347454f35f5cc7568d2ac94636016c8b54e2a8e` | 문서: #7702 시각 증적 — 1342000 114·151·154쪽 Native/fresh WASM, 전쪽 TSV (code head 065125a94) |
| `8e1811336dc9d1279fd3e72dcd964749c6cbae67` | `e1585c493280b5c6d3a5f64e9a2e4531b456db1d` | 문서: #7965 시각 증적 재생성 — devel efc52afb7 리베이스 뒤 code head 065125a94 |
| `478da825029b876b6b54da777a9b269b8822aa4d` | `758d2a34b16f2b1ba5057fa386f008e5435d7149` | fix(조판): 쪽 끝 행을 내용 컷으로 가를 때 그 행에서 끝나는 걸침 칸 줄은 닫는 선까지 남긴다 |
| `e90586ecce4a6097603e1fde38264c994584c04a` | `40816fc776222e51066f69280de94d954e116054` | 문서: #7965 시각 증적 재생성 — code head 478da8250 (1342000 199·200·203쪽 추가) |
| `f296dbc21cf4f586e73e5cb5efaf9db69d975f8d` | `c799a1bc3b0470c59a4bfba0afaa15c2a8100cdb` | fix(조판): 휴먼명조 ASCII 전진폭을 글꼴 hmtx 로, 공백 줄여 들어가는 문단 끝 줄은 끝 글자 자간 없이 (#7702) |
| `e4aa55ad0a2899576d3f08a7769f7a0acfb1dc32` | `8f644dfebc241ec817b1df8e6d47cbb8e67493e5` | 문서: #7965 시각 증적 재생성 — code head f296dbc21 (1342000 189쪽·issue5169 4쪽·휴먼명조 합성 입력 추가) |
| `1cf3b879b61a9f8fd00a6b1aa0aac85a2c588033` | `247791e1111dab9cc199cf23cbf289850a8d6273` | fix(조판): noAdjust 표의 비끝 조각은 본문 아래 − 바깥 아래 여백에서 닫는다 (#8101) |
| `d0338d4c3599aa3f93f82c477cd4ec766d1ec9e7` | `0acf42680d7bb117ee8579b36feca176dd2155ed` | 문서: #7965 시각 증적 재생성 — code head 1cf3b879b (1342000 126·127쪽 추가) |
| `e64b513ed5faf7392ab187b7cb70bb112cbe05d1` | `d0f0b38830ef04940b83122e170e58dbda5e9fdc` | fix(조판): 쪽 나눔 표 조각의 바깥 위 여백과 저장 되감김 경계의 끝 줄간격 (#8105) |
| `b02433b21e7c38034c218f4624b1ef4ef1c9750e` | `064caf58bd24e9309e3974108879e058ba81478f` | 문서: #7965 시각 증적 재생성 — code head e64b513ed (1342000 51·56쪽, hwpx_sample2 6·7쪽 추가) |
| `51a628363b51d01a0bf4009552dc298f168307dc` | `8aa560acc2feaed0b8a9359f61619cb45ad1047f` | fix(#8105): 분할 행의 남은 선언 높이 — 빈 띠 상한과 저장 줄 행의 닫는 선 |
| `f67f6410d5cc0d577837e90cd398b9e5b9030b8e` | `fb893248f561b93baedaecea8539de20f8b37d5c` | 문서: #7965 시각 증적 재생성 — code head 51a628363 (1342000 59·61·62·307쪽, #8105 빈 띠 픽스처 추가) |
| `9f2b241500e2de48f5e5acc4ae232e836da1b8fa` | `fa7aefdf10d8e71404350a36a2a5bb364d166e67` | fix(#8105): 번호 문단의 저장 줄 낱말 꼬리 — 한/글이 다시 나누는 문단 |
| `690290fa9d06c6f19f18a753900aa3555e1de1c0` | `361c049311f828c954f3887b9832ef24b255396d` | 문서: #7965 시각 증적 재생성 — code head 9f2b24150 (1342000 250·251쪽 추가) |

## 보정과 검증의 실행 순서

1. 최신 devel 위에서 기여를 먼저 적용하고 source 124개와 공통 보정 순서를 고정했습니다. source·현재 SHA·커밋 제목 전체는 [실제 통합 순서 원장](../../../mydocs/pr/assets/planet6897_20261018/documents/integration_commit_order.md)에 연결합니다.
2. 원본·독립 Print·source별 claim을 대조해 공통 컷 소유·여백·줄/표 흐름·수식/폰트/편집을 보정했습니다. 구체적인 생산/소비 코드·반례·전후 관측은 통합 계획과 문서별 증거를 재사용합니다.
3. runtime `3cb5a4c290663343123074be1e2fc0dda18d3db4`의 전체 로컬 검증과 `e25dbd0f5d47bc32686016c08054d2bf718be6a7`의 새 5건·lint를 완료하고 code `228db2f278b52bc73fe2c5b5e0bb9a09b2ef4ff7`로 Open PR #8148을 제출했습니다. [실제 검증 명령과 결과](../../../mydocs/pr/assets/planet6897_20261018/documents/validation_commands.md)와 [같은 번호의 최종 검토](../../../mydocs/pr/archives/pr_7965_review.md)에 실제 결과를 연결합니다.
4. 정확한 code head의 [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39705756548) · [CI](https://github.com/edwardkim/rhwp/actions/runs/39705756552) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39705756369) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39705756537) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39705756582) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39705756438)가 모두 성공했습니다. 원 PR 27건의 review/implementation와 오늘할일을 같은 PR의 문서 trailing으로 기록하며, 그 최신 head의 CI·병합 결과·링크를 push 전후 확인합니다.
5. 정확한 trailing head로 merge한 뒤 merge SHA 증거 comment·원 PR open/closed 처리·완료 이슈/하위 이슈 정리·devel 동기화·branch/worktree/output 정리를 수행합니다. 공유 target·사용자 자료·기여자 fork는 보존합니다.

## 보정·되돌림 경계

- 이 원 PR의 기여 SHA는 위 표로 고정하고 공통 보정 및 전후 관측은 [통합 계획·생산/소비 경로·보정 기록](../../../mydocs/plans/pr_review_planet6897_20261018.md)에 연결합니다. 문서별 임계값 완화나 source branch 강제 재작성으로 결과를 맞추지 않습니다.
- 원 PR source는 유지합니다. 미병합 통합 후보의 결함은 해당 공통 계층의 추가 보정으로 처리하고, 이미 통합한 변경을 되돌릴 때는 표의 source SHA 및 후속 공통 보정 의존성을 함께 검토합니다.
- 최종 판정과 GitHub approve/push/merge/close를 구분합니다. 사용자의 기존 승인 범위 안에서 실제 최신 상태와 필수 결과를 확인해 후속 처리를 이어갑니다.
