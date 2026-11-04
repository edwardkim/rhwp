# planet6897 문서 처리 PR 체리픽 통합 계획

- 작업 브랜치: `review/integration-20261026-document-updates`
- 기준: `upstream/devel` `0bb36e6f976e9ebeba0b1c48d86a2c5bdaee7f7c`
- 사용자 지시: 충돌 없는 열린 PR을 체리픽 통합한 뒤 검토한다. #8237도 편람 수정이 아니면 포함한다.
- 편람을 성능측정 대조군으로 사용하는 것만으로 제외하지 않는다. 성능 PR #8205·#8209·#8210·#8213·#8215·#8217은 포함한다.
- 실제 편람 수정 #8268과 충돌 상태 #8161은 이번 통합에서 제외하고 열린 상태를 유지한다. 두 PR에 안내 댓글이나 close를 수행하지 않는다.
- 2026-10-27 사용자 지시로 이미 체리픽한 아래 20건으로 범위를 고정한다. 이후 열린 PR은 추가로 체리픽하지 않는다.
  별도 HWP 시각 검증은 원 PR의 직접 대상·재현 입력에 한정하며 불필요한 다른 문서 대조를 추가하지 않는다.
- 기본 경로는 collaborator 매개 외부 PR의 주 작업공간 체리픽 통합이다. 원 PR별 reviewer `jangster77` 지정·API 확인 후 조사·fetch했다.
- 각 원 head는 고정 ref로 fetch해 SHA를 확인했다. 최신 기준과 각 원 head의 `git merge-tree --write-tree`는 20건 모두 exit 0이다.
- 통합 전체에 대응하는 단일 대표 이슈가 없어 작업 시작일을 브랜치 식별자로 사용한다.

## 고정 원 PR

| PR | 고정 원 head | 통합 축 |
| --- | --- | --- |
| [#8115](https://github.com/edwardkim/rhwp/pull/8115) | `a47a41d3004a0458873cf1a00e90a356f19f2fa5` | 표·각주 경계 |
| [#8128](https://github.com/edwardkim/rhwp/pull/8128) | `21c40f21c5ee019fdf1ee4e68bdfeed35d3ae26c` | 이어지는 쪽 저장 reset·자기 각주 |
| [#8133](https://github.com/edwardkim/rhwp/pull/8133) | `465d85f13cab961628ec07247ea6ade82e0cbc46` | Native/Canvas/CanvasKit 획 |
| [#8142](https://github.com/edwardkim/rhwp/pull/8142) | `e1abc4c7f149268ea63995eeedeb02cfcc92e5cd` | 병합 칸 안쪽 행 |
| [#8184](https://github.com/edwardkim/rhwp/pull/8184) | `567ff9d18ac1f7e702e94042eb38a03a44f95943` | 빈 host 자리차지 표 |
| [#8202](https://github.com/edwardkim/rhwp/pull/8202) | `d0f6adbeef179e17d0431eb84c38443d318a887a` | 글자처럼 도형 |
| [#8205](https://github.com/edwardkim/rhwp/pull/8205) | `5de6de6a1586e4ce714ee4c8e51a92a3b4812c97` | 수식·미주 측정 캐시 |
| [#8206](https://github.com/edwardkim/rhwp/pull/8206) | `dd3ad0b29bb55893caf8d618e42a24776d64673f` | 글자처럼 그림과 위아래 개체 |
| [#8209](https://github.com/edwardkim/rhwp/pull/8209) | `01588f1e997b37795d2dbafecdf1f015341d553c` | 글꼴 판정·봉인 다이제스트 |
| [#8210](https://github.com/edwardkim/rhwp/pull/8210) | `276d569c2f5abce5b77bfb89aed016951e314bd3` | 큰 표 스캔·높이 측정 |
| [#8213](https://github.com/edwardkim/rhwp/pull/8213) | `d9aab2292be0da4c71dc2c28665fd9ddc8c07a59` | 중첩 표·제목행·행 상자 |
| [#8215](https://github.com/edwardkim/rhwp/pull/8215) | `31e5e700791c89c743feacd84acc55bc034569a3` | HWP5 그림 저장 스트림 |
| [#8217](https://github.com/edwardkim/rhwp/pull/8217) | `b717326395c5e9556f6ccb0ec3dc1c127c2daf49` | HWPX 그림 deflate |
| [#8219](https://github.com/edwardkim/rhwp/pull/8219) | `bfe5d44cf1d08806d9ac12aff1096096ed0853c0` | HWP3 BinData 순서 |
| [#8221](https://github.com/edwardkim/rhwp/pull/8221) | `1a69db15352e171d2a16b4195d78cc445af2e9e6` | PDF 문단 경계 |
| [#8227](https://github.com/edwardkim/rhwp/pull/8227) | `a34d1fb70dcdc2e1f692efaa63bfd2d77ee0e74c` | PDF 편집 우선 배치 |
| [#8229](https://github.com/edwardkim/rhwp/pull/8229) | `195ce6297541f2136ea02fb55cdc531320d2d714` | Studio PDF 배치 설정 |
| [#8237](https://github.com/edwardkim/rhwp/pull/8237) | `f58e857b1294dd1ed0ea1f355f6323fbc5dbcd51` | PDF 머리·꼬리 띠 분리 |
| [#8265](https://github.com/edwardkim/rhwp/pull/8265) | `65196bb3f97293763b7736438feb6d47a5072922` | Studio AI 제공자 |
| [#8283](https://github.com/edwardkim/rhwp/pull/8283) | `43d7eb24e84c84e6fd7a817dad579b9baebaaae3` | HWPX 재열기와 그림 host |

## 적용과 검증 순서

1. 원 PR 번호 순서로 기능·테스트·증적 commit을 `git cherry-pick -x`한다. merge commit은 제외한다.
   PDF·성능·그림 host의 적층 PR은 이미 적용한 원 SHA와 patch-id를 대조해 중복 commit을 다시 적용하지 않는다.
   원 저자·원 SHA·적용 SHA·순서·중복·보정은 원장에 기록하며 원 fork head는 변경하지 않는다.
2. 체리픽 전 기준 Native CLI와 최적화 WASM을 고정 공용 `target/pr-review`에서 순차 준비하고 source/build/font provenance를 기록한다.
   기준 코드·후보 코드의 같은 입력·페이지 비교에 사용하며, 원 PR의 첨부 수치를 통합 head의 직접 검증으로 대신하지 않는다.
3. 통합 후 각 변경의 실제 소비 경로·독립 기대값·반례·focused 검사를 검토한다. 각주·표 분할·저장 줄·획·그림 저장과 PDF 두 배치를 구분한다.
   이미 해결된 이슈를 보정할 때 원 해결 PR·통합/메인터너 commit·독립 근거를 먼저 추적한다.
4. Rust fmt·세 Clippy·workspace build·suite/unit tier 정책·전체 nextest·Native Skia 3종을 순차 실행한다.
   Studio TypeScript·단위·실제 브라우저, 두 제공자·PDF 설정, HWP/HWPX 그림 보존·재저장 계약을 검증한다.
5. 실제 출력 영향 입력의 전쪽 Native/fresh 최적화 WASM TSV와 대표 확대 판독을 수행한다.
   최소 90%와 사용자 지정 기준 대비 페이지별 0.1%p 하락 gate를 모두 적용하고, 기존 목표 문서의 개선도 보존한다.
   편람 수정 자체는 #8153의 별도 작업으로 유지한다. 성능 대조군 언급과 편람 조판 수정 범위를 혼동하지 않는다.
6. 최신 사용자 지시에 따라 승인된 로컬 검증·필요한 메인터너 보정을 마친 뒤 원 PR별 archive review를 작성해
   같은 통합 PR에 포함한다. 문서에는 실제 로컬 결과와 미실행 원격 CI를 구분한다. 통합 PR의 정확한 head CI를 확인하고
   각 원 PR의 수용/보류와 실제 검증 범위를 기록한다.
7. 현재 작업은 로컬 검증·원 PR별 검토 문서·통합 PR 제출까지 진행한다. 이후 CI 결과를 보고하며, 원격 머지·원 PR 종료는
   이 작업에 대한 별도 사용자 지시를 따른다. #8268·#8161은 제외한 열린 상태를 유지하고 원 contributor fork branch는 보존한다.

## 산출물과 자원 경계

- 작업 중 임시 파일·렌더 결과·검증 로그: `output/pr-review/planet6897-20261026/`.
- 선정·원 SHA·검토 gate·명령/결과 운영 기록: `/home/tsjang/rhwp-pr-evidence/planet6897-20261026/`.
- Cargo는 `/home/tsjang/rhwp/target/pr-review` 하나를 순차 재사용한다. 아직 실행 중인 빌드·테스트 산출물은 삭제하지 않는다.
- 정식 입력·한컴 기준 PDF·전쪽 TSV·대표 PNG는 역할별 정식 경로에 반영한다. 대용량 PDF는 별도 로컬 보관 규칙을 따른다.
- 검토·후속 처리 종료 후 output 전체와 사용 종료된 review debug를 별도 지시·백업 없이 정리한다.
  다른 profile과 다른 작업의 worktree를 보존하며 유휴 셸 cwd만으로 삭제를 보류하지 않는다.

## 2026-10-27 통합과 첫 보정

- 원 PR 20건의 고유 commit 94개를 `-x`로 적용하고 적층 중복 35개를 제외했다.
  원본·적용 SHA와 순서는 [적용 원장](assets/review_integration_20261026/cherry_pick_ledger.json)에 기록한다.
  체리픽 완료 head는 `be1de8e41ed32f461706f9c8ddcefcfd392b0e5c`다.
- 각 PR 적용 결과는 누적 기준에 원 head의 변경을 병합한 예상 tree와 모두 일치했다.
  #8227·#8237의 사전 tree 비교는 이미 체리픽한 적층 부모를 각각 `1a69db153`·`a34d1fb70`으로
  명시했다. 중복 도입에 따른 함수 공개 범위·검사 파일의 가상 충돌이었으며 실제 체리픽 충돌은 없었다.
- 기준 Native 빌드와 합성 입력 21쌍의 전쪽 비교를 완료했다. 기준 WASM은 원본 head에서 Rust
  컴파일을 끝낸 뒤 바이너리 최적화와 코드 체리픽을 병행했다. 기준 패키지/public의 JS·WASM 해시와
  복사한 기준 모듈의 `WebAssembly.compile`을 확인했다. 후보 비교는 남았다.
- 사용자가 불필요한 범위 확장을 지적해 sample16 별도 시각 재검증과 그 자식 프로세스를 종료했다.
  이 별도 재검증은 재실행하지 않는다.
- #8265 원 head의 `readSse`는 CRLF의 CR과 LF가 서로 다른 청크에 오면 LF를 빈 줄로 처리했다.
  event 이름과 여러 data 줄이 사라지는 실패를 원 코드와 실제 새 검사에서 각각 확인했다.
  [SSE 해석 규약](https://html.spec.whatwg.org/multipage/server-sent-events.html#event-stream-interpretation)에
  따라 청크 끝 CR을 다음 청크 또는 EOF까지 보존했다. 모든 바이트 분할·UTF-8·LF/CR/CRLF 검사와
  기존 제공자 검사를 합쳐 22개가 통과했고 Studio TypeScript 검사도 통과했다.
  보정 commit은 `57abd92f0`이며 실제 API 키는 사용하지 않았다.
- #8167·#8168·#8271 입력의 마지막 저장 제품을 확인하고, 누락됐거나 다른 저장본·세대의 PDF만 있던
  8개 입력을 자체 원본에서 MCP 2020 Print로 출력했다. 입력·PDF 해시, 실제 한컴 버전,
  `pdf_print_method=0`·1-up 출력 모드는 [Print 근거](assets/review_integration_20261026/hancom_print_inputs.json)에 기록한다.
  이 PDF와 기존 PR의 2024 비교는 세대를 구분한다.
- 현재 수용 판정은 미완료다. Rust 전체 검증, Native/fresh WASM 후보 전쪽 비교, 관련 Studio 브라우저
  검증과 확대 판독을 마친 뒤 PR을 준비한다. 최신 사용자 지시대로 로컬 검증 후 원 PR별 archive review를 먼저 작성하고
  통합 PR을 만든다. 지침의 기본 CI 후 trailing 순서보다 이 명시적 지시를 우선한다.
- Studio 전체 단위 검사는 2,599개 통과·기존 skip 2개·실패 0개로 완료했다.
- 첫 통합 Native 빌드는 `stored_tac.rs`의 E0614로 실패했다. #8184의 `9ec584a15`는 다음 문단을
  `*first + 1`로 조회했고, 이후 #8205의 `9e72098ba`가 같은 함수의 패턴을 값 복사로 바꿔 `first`가
  `usize`가 됐다. 두 원 변경과 `d254c9bac` 원 commit을 추적했다. 추가된 다음 문단 조회도
  `first + 1`로 맞췄으며 조판 정책·기대값은 바꾸지 않았다. 보정 head에서 Native 빌드부터 다시 검증한다.


## 기존 미달 페이지 보정과 CanvasKit 검토

- 사용자는 기존 미달 오류도 이번 브랜치에서 보정하도록 지시했다. #8133의 sungeo 21·22쪽,
  #8184의 hwpspec 23·54쪽, #8213의 중첩 표 입력 2·8·9·10·11·12·14쪽을 포함하며,
  해당 PR을 제외하거나 입력·기준값을 바꿔 회피하지 않는다. 추가 PR은 체리픽하지 않는다.
- 이 보정 전에 진행하던 전체 Rust 검사는 새 코드가 필요해 중지했다. 6,769개 부분 통과·실패 0개는
  완료 결과가 아니다. 기존 후보의 Native 빌드·Clippy 3종·workspace build·suite/unit-tier 정책은
  통과했으며, 최종 보정 head에서 필요한 전체 검증을 완료한다.
- #8133 CanvasKit의 화면 배율만 사용하는 선 두께 하한은 축소된 자식 그룹에서 실제 1픽셀을
  보장하지 못했다. 실제 CanvasKit 소프트웨어 raster에서 반배율 도형 경계가 회색 127로
  그려졌고, 원래 두께 0.48의 HWP 점선 주기도 보정된 획 두께로 계산돼 `[6.46, 3.62]`가 됐다.
  Native·Canvas2D와 동일하게 현재 canvas의 역변환으로 장치 1픽셀을 환산하고, 점선 주기는
  원래 두께를 사용한다. 화면용 SkPicture 기록에서는 외부 변환도 반영하고 cache key에 포함하며
  실패 시 변환 상태를 복원한다. 인쇄 기록은 기존 실제 장치 CTM을 사용한다.
- 실제 raster 경계와 점선 검사 2개는 수정 전 실패·수정 후 통과했다. 항등·축소·확대·회전 변환과
  기록 실패 복원까지 총 8개가 통과했다. TypeScript 검사와 Studio 전체 단위 검사도
  2,607개 통과·기존 skip 2개·실패 0개다. HWP 전쪽 Native/fresh WASM 검증은 아직 남았다.
- 실행 로그는 `output/pr-review/planet6897-20261026/`의 `canvaskit-contract-before.log`,
  `canvaskit-contract-after.log`, `canvaskit-tsc.log`, `studio-unit-after-canvaskit.log`에 있다.


### sungeo 저장 프레임 후속 경계

- 원본 2구역의 문단 43은 저장 위치 64,594HU를 가진 표제어지만 다음 본문과 함께 새 쪽에
  수용된다. 뒤 문단 44부터 새 저장 프레임의 본문을 소비해도, 이전 prefix 판정이 첫 표제어의
  옛 위치만 보고 문단 65·85의 후속 쪽 경계를 계속 무효화했다.
- 원 수정 `8cc5042c1`과 보완 `7f7cb6086`, 관련 #7642의 stage2649·2666·3429 기록을
  대조했다. 기존 whole-fit이 사용하는 실제 소유자·저장 끝 대조 결과를 prefix 유효성 조회에도
  사용한다. 실제 흐름이 저장 프레임 끝에 도달하면 첫 항목의 옛 좌표를 쪽 전체의 무효 상태로
  사용하지 않는다. 명시적 경계와 TAC 원점 계약은 그대로 소비한다.
- `legacy-v2` Native는 수정 전 21쪽 26.13435%·22쪽 38.97308%에서 각각 99.94110%·
  99.93512%로 개선됐다. 전체 94쪽 최저 99.85165%, 90% 미만 0쪽, PDF와 쪽수 일치다.
  fresh WASM·최종 전체 Rust 검증은 아직 남아 있으며 이를 완료 전 전체 수용으로 판정하지 않는다.
- 빌드와 전쪽 로그: `output/pr-review/planet6897-20261026/legacy-v2-native-build.log`,
  `sungeo-v2-all.log`. 고정 바이너리와 source diff 해시는 `legacy-v2-producer.json`에 기록했다.
- OpenAI·Gemini 제공자 E2E, AI panel·설정·side panel E2E도 통과했다. 모의 API이며 실제 키는
  사용하지 않았다. 브라우저가 소비한 Rust 모듈은 기준 WASM이므로 조판 후보 검증과 구분한다.


### hwpspec 공동 앵커·후속 표 간격

- 2구역 문단 126의 첫 표는 본체 8,046HU + 양쪽 여백 282HU + 아래 캡션 1,802HU가
  빈 호스트 줄의 저장 위치 10,130HU를 정확히 닫는다. 첫 표만 남은 공간에 넣으면 표는
  22쪽에 남고 호스트·형제 표는 23쪽으로 갈라졌다. 첫 표 앞 공간을 포함한 저장 호스트 줄과
  앞 문단의 끝을 기존 `saved_positive_frame_reset`에 전달해 개체 방출 전에 쪽 소유를 확정한다.
  `is_textless_coanchored_float_host`의 같은 공동 앵커 경로가 배치를 계속 담당한다.
- 54쪽 문단 347은 나누지 않는 표이며 후속 빈 호스트 문단 348은 저장 위치가 되감기는 새 쪽이다.
  `host_spacing`의 기존 source reset 판정이 RowBreak만 허용해 347→348을 같은 쪽 표 묶음으로
  오인했다. 분할 정책과 후속 쪽 소유를 구분하고, pagination·paint가 같은 원본 되감김 조회를
  사용하도록 공유했다. 표 116의 앞 간격·후행 줄간격 중복이 제거됐다.
- 선행 원 해결 #8184의 `9ec584a15`와 기존 간격 판정 `1d6bc70767` 및 실제 소비 경로인
  `section::stored_reset` → `controls::flow_table` → 빈 호스트/공동 앵커 → `layout`을 대조했다.
  첫 표 상자와 뒤 호스트 줄의 소유를 바꿨으며 내용 숨김·좌표 clamp·문서 ID 분기는 추가하지 않았다.
- `legacy-v3` Native는 hwpspec 전체 178쪽 최저 92.45586%, 미달 0쪽이며 쪽수가 유지됐다.
  23쪽은 86.58353%→99.99389%, 54쪽은 89.58093%→99.99765%다.
  같은 경로를 사용하는 HWP5 공식 사양 71쪽도 최저 95.27413%를 유지했다.
  원문 공식 PDF는 변경하지 않았다. 최종 fresh WASM과 전체 회귀 검증은 계속 남았다.
- 로그: `output/pr-review/planet6897-20261026/hwpml-v3-all.log`, `hwp5spec-v3-all.log`.
  빌드 원본 diff와 고정 바이너리의 해시는 `legacy-v3-producer.json`에 기록했다.


### 중첩 표의 빈 줄 쪽 소유·세로 오프셋 1차 보정

- 같은 원본을 MCP 2020 Print로 다시 출력했다. 저장 제품 revision `7.5.12.754`는 2022 이전이며,
  job `cc627da2-d599-49f6-95d2-72f0693a4f08`의 실제 Hancom `11.0.0.9136`, Print method 0,
  `hancom2020_pdf_driver_one_up`, 17쪽을 확인했다. 기존 `-hwp-2020.pdf`의 Creator는
  `Hwp 2022 0.0.0.0`이므로 새 Print와 버전·출처를 구분한다. 파일명만으로 기존 출력을
  2020 제품으로 판정하지 않는다. 원본·기존 PDF는 변경하지 않았다.
- 원 해결 #3820 stage56과 #6697 및 #7445의 중첩 offset 계약을 대조했다. `CellUnit` 98(문단 73)의
  10.6667px 빈 줄은 저장된 새 쪽의 첫 슬롯이지만, 직전 TAC 표에 float placement가 없다는
  이유로 프레임 종료 인증이 거절돼 앞쪽 컷에 흡수됐다. 완전한 원본 개체 줄은 이미 저장 줄
  상자를 소유하므로, 기존 `owns_saved_object_line` 결과를 같은 body 종료 조회에 전달한다.
  인증된 빈 줄 경계는 재귀 투영에도 전달하며 일반 빈 overlay 리셋은 기존 계약을 유지한다.
- 부분 표 배치는 저장 줄이 있다는 사실만으로 세로 오프셋이 이미 소비됐다고 간주했다.
  실제 확정 float frame의 유무로 판단하고, 셀 끝 호스트의 첫 행 예약과 실제 배치가 같은
  `para_relative_float_table_lead`를 소비하도록 맞췄다. 2쪽 선언 offset 713HU를 복원했다.
- `legacy-v4` Native 17쪽은 쪽수를 유지했다. 새 Print 기준 2쪽 68.22392%→98.82900%,
  9쪽 49.65136%→94.27405%, 10쪽 61.16692%→90.26331%로 개선됐다.
  11·12·14·16쪽 미달은 아직 남아 있어 보정을 계속한다. 새 회귀의 기대값으로 고정하지 않는다.
- 로그: `output/pr-review/planet6897-20261026/nested-v4-all.log`, `nested-mcp-finish.log`,
  `nested-units-v2.log`. 고정 바이너리는 `candidate-v4-rhwp`, 빌드 출처는 `legacy-v4-producer.json`이다.

### 중첩 셀의 물리 끝 프레임과 재귀 유닛 후속 보정

- 선행 `3e9de474086707cf7c0b0ebdf60801a840d66859`의 `saved_nested_row_frames`는
  첫 물리 프레임은 `common.height`로 예약하지만 뒤 프레임은 글줄 높이만 사용했다.
  원본 셀 높이는 두 물리 조각의 합이므로 `cell.height - common.height`의 남은 상자를
  같은 예약·컷·부분 배치 조회에 전달한다. 남은 실제 내용이나 전진이 그 상자에 들어가지
  않으면 인증하지 않는다. 가운데 정렬의 빈 공간을 새 텍스트 유닛으로 세지 않는다.
- 작은 저장 호스트 줄은 큰 개체의 바깥여백을 대신 소비하지 않는다. 직접 HWPX에서 쓰던
  같은 원본 앵커·미편집·단일 개체 조건을 Native HWP5에도 적용해 원본 컷의 여백 예약과
  실제 원점이 같은 결과를 읽도록 한다. 첫 줄이 전체 개체 상자를 소유한 경우는 제외한다.
- `legacy-v5`는 17쪽을 유지하며 11쪽 62.77638%→90.18496%,
  16쪽 89.82594%→95.38000%다. 12·14쪽은 미달이고 15쪽은 92.10444%→82.35367%로
  하락했다. 이 후보를 수용하지 않으며, 다음 후보에서 재귀 자식의 마지막 예약 유닛을
  상위 cursor가 버리는 경로를 보정한다. scalar fallback의 합성 꼬리와 실제 원본 자식
  cursor의 빈 물리 밴드를 구분한다. 원본·PDF·기대값·공차는 변경하지 않는다.
- 로그: `output/pr-review/planet6897-20261026/legacy-v5-native-build.log`,
  `nested-v5-all.log`, `nested-fragment-alignment-trace.log`, `nested-p12-alignment-trace.log`.

### 재조판 그림·부동 글상자의 원본 control 줄 소유 조사

- #8168의 두 여러 줄 입력을 같은 원본에서 Hancom 2020으로 임시 HWPX 재저장했다.
  원본 생성 절차와 마지막 control의 원시 슬롯 위치를 대조했다. 원본을 대체하거나
  임시 저장본을 새 회귀 입력으로 사용하지 않는다.
- `above`의 실제 저장 줄은 세 번째 줄을 6,647HU→13,182HU로 옮긴다.
  차이 6,535HU는 offset 300 + 도형 높이 5,669 + 바깥 위·아래 여백 283씩과 같다.
  `below`의 offset 9,000HU에서는 같은 세 번째 줄이 6,647HU를 유지한다.
  두 입력의 `vertRelTo=PARA`, `horzRelTo=PAPER`와 도형 offset은 그대로다.
  배제 후 밀린 최종 글줄을 다시 개체 앵커로 쓰지 않고 배제 전 control 소속을 유지해야 한다.
- 실제 job은 `71dc25d1-fc74-4984-bf53-20512bd9b684`·
  `e08d5684-f3a9-4457-b533-350c7f932f88`이며 결과는
  `output/pr-review/planet6897-20261026/multiline-hancom-save-results.json`에 기록한다.
  이 두 저장본은 조사 자료이며 아직 구현·시각 검증을 마친 것으로 판정하지 않는다.
- `legacy-v6`는 마지막 재귀 예약 유닛을 보존해 12쪽을 57.03039%→99.24307%로 개선했다.
  전체 17쪽 중 15쪽은 90% 이상이며 14쪽 75.65324%·15쪽 82.35367%가 남는다.
  같은 원본의 자식 표는 첫 물리 높이 35,362HU, 셀 총높이 70,018HU를 저장한다.
  남은 선언 34,656HU보다 실제 종료 줄·안여백 점유 34,682HU가 커서 프레임 조회가 거절된다.
  첫 상자가 유효해도 마지막 상자가 내용의 최소 점유를 담지 못하면 끝 행은 성장해야 한다.
  다음 후보에서 선언된 잔여를 최소로, 실제 내용 점유를 성장 하한으로 함께 소비하도록 한다.
  이 26HU 차이를 별도 px 허용치로 면제하지 않는다. 로그는 `nested-v6-all.log`,
  `nested-v6-p14-detail.log`, `nested-v6-p15-detail.log`다.
- `legacy-v7`의 종료 행 최소 높이 보정으로 중첩 표 **전체 17쪽이 90% 이상**이다.
  최저 90.18496%, 14쪽 97.88003%·15쪽 98.31085%, 쪽수 17을 유지한다.
  선언된 잔여 높이와 실제 내용 점유 중 큰 값으로 종료 행을 성장시키며, 같은 source reset과
  첫 물리 프레임 인증은 유지한다. fresh WASM·확대 판독·전체 회귀는 아직 남았다.
  이 바이너리에는 진행 중인 그림·도형 계획도 들어 있지만, 중첩 원본의 해당 본문에는
  새 NO_LS 그림 계획이 적용되지 않는다. 정확한 전체 source diff·바이너리 해시는
  `legacy-v7-producer.json`, 실행은 `v7-nested-v7.log`에 연결한다.
- hwpspec의 MCP Print PDF도 `pdf/review-integration-20261026/hwpspec-2020.pdf`에 정식 보존하고
  기존 Print 근거 JSON에 job·입력/PDF 해시·실제 runtime·Print method를 추가했다.
- 진행 중인 그림 계획은 배제 전 실제 control 줄에서 앵커를 확정하고, 그 도형 띠로
  다시 구성한 글줄과 흐름 끝을 `InlineFlowPlan`으로 전달한다. Native 첫 후보에서
  세 줄 문단의 이동·뒤 문단 위치는 한컴의 6,535HU 이동과 대응했으나, 가로 원점을
  소비자가 기대하는 종이 절대 좌표 대신 단 상대 좌표로 전달했고 BehindText 도형을
  기존 control 경로가 중복 등록했다. 그 후보는 수용하지 않는다. 다음 후보에서 좌표계와
  단일 개체 소유 전달을 보정하며 별도 backend 보정·기대값 변경을 하지 않는다.

### 재조판 그림·부동 글상자 공통 계획의 Native 검증

- `InlineFlowPlan`이 배제 전 control 소속 글줄에서 부동 도형 앵커를 한 번 정하고,
  같은 배제 띠로 재구성한 글줄·흐름 끝·개체 위치를 측정과 배치에 전달한다.
  지원하는 도형의 캡션·각주 등 별도 소유자가 필요한 내용은 기존 경로에 남긴다.
- `AbsolutePageFrame`의 가로 원점은 기존 소비 계약대로 종이 절대 좌표를 유지하고,
  세로만 단 상대 좌표로 전달한다. 확정 계획이 등록한 도형은 기존 control 경로에서
  다시 등록하지 않는다. 실제 배치는 같은 RenderTree를 사용하는 모든 backend에 전달한다.
- `legacy-v8` Native는 새 MCP Print 입력 8개 모두 90% 이상이다. 사각형·글상자 TAC,
  종이 기준 TopAndBottom, HWPX TopAndBottom·BehindText 5개는 100%다.
  여러 줄 above·below는 각각 94.91910%·94.94234%, 기존 저장 줄을 가진 대조군은
  93.52218%다. 여러 줄 확대 비교에서 그림 줄·글상자·셋째 줄·후속 문단을 확인했다.
  중첩 원본도 전체 17쪽 최저 90.18496%를 유지했다.
- 로그와 빌드 출처는 `output/pr-review/planet6897-20261026/v8-focused-results.json`,
  `legacy-v8-producer.json`이다. fresh WASM·전체 회귀는 아직 남았으며,
  새로운 Rust 회귀나 기존 기대값은 시각 선행 조건 충족 전 추가·변경하지 않는다.
- 11쪽 두 번째 중첩 표의 확대 비교에서는 별도 결함을 확인했다. 원본 TAC 호스트 줄
  26,177HU는 표 25,895HU + 바깥여백 141HU씩을 정확히 소유하지만, 바로 다음 문단의
  reset 때문에 표를 scalar 글줄로 분해해 최소 높이·가운데 정렬 여유를 잃는다.
  선행 `81777a6a99b63d320d10b7103905f676d40ba445`의 프레임 꼬리 분해 조건을 대조했다.
  기존 `owns_saved_object_line`로 전체 상자 소유를 재사용해 이 불필요한 분해만 막는다.
  실제 여러 쪽인 자식·내부 source reset의 분해 조건은 유지하며 다음 후보에서 검증한다.

### WASM 검증 공급 글꼴의 컨테이너 보정

- 수정 전 WASM의 sungeo 전쪽 검증은 `invalid native PUA SFNT face`로 중단됐다.
  계측 API에 전달된 함초롬돋움은 765,892바이트의 `wOF2` 컨테이너였다.
  Canvas FontFace는 이를 읽지만 공통 PUA 계측기는 SFNT를 요구한다. 엔진 결과를
  숨기거나 글꼴을 바꾸지 않고, 공급 카탈로그에서 같은 프로그램을 SFNT로 풀어 전달한다.
- 기존 테이블 복사기를 재사용하며 원본 글꼴 파일은 보존한다. 원본·공급본 SHA와
  `woff-to-sfnt` 또는 `woff2-to-sfnt` 변환을 카탈로그·검증된 캐시에 기록한다.
  신규 검사는 WOFF·WOFF2 모두 수정 전 컨테이너 계약 위반으로 실패했고,
  수정 후 SFNT·글꼴 테이블 동일성·원본 보존·캐시 재사용 검사를 통과했다.
- `venv/bin/python -m unittest scripts.tests.test_visual_sweep_png`: 29건 통과.
  로그는 `output/pr-review/planet6897-20261026/woff-sfnt-before.log`,
  `woff-sfnt-after.log`, `pua-sungeo-diagnostic.log`다. 바뀐 공급 절차로 base/head가
  같은 실제 글꼴 프로그램을 사용하도록 전쪽 비교를 다시 산출한다. 이전 실패를
  통과로 간주하지 않으며, 이 검사는 픽셀 좌표 기대값을 추가하지 않는다.

### 저장 TAC 개체 줄의 최소 표 상자 복원

- `legacy-v9`는 다음 문단 reset만으로 완전한 TAC 개체 줄을 분해하지 않는다.
  여러 쪽의 물리 표·실제 자식 내부 reset은 기존 분해 조건을 유지한다.
  공통 `owns_saved_object_line`의 원본 줄·단일 개체·미편집·정확한 전체 높이 인증을
  재사용하며 별도 문서 식별자나 허용 오차를 만들지 않았다.
- Native에서 해당 내부 표 본체는 337.7px→345.3px로 원본 25,895HU의 높이를 복원했다.
  확대 비교에서 아래 테두리·가운데 정렬 내용과 뒤 원본 소유를 확인했다.
  중첩 표 11쪽은 90.18496%→98.82287%, 전체 17쪽 최저는 94.38594%다.
  같은 후보의 새 Print 입력 8개는 모두 통과하며 `legacy-v8` 수치를 유지했다.
- 고정 바이너리 SHA는 `336c7467562b53cf6968fdb211da1b311767569e08a763119404e7974488f543`이다.
  실행·전체 source diff·바이너리 출처는
  `output/pr-review/planet6897-20261026/v9-focused-results.json`,
  `legacy-v9-producer.json`에 기록했다. 다음 단계는 같은 production source의
  fresh WASM·전체 대상 전쪽 검증이며, 시각 선행 조건 뒤 회귀 검사를 정정·보강한다.

### 전쪽 검증 완료와 원본 소유 회귀 보강

- production `3ae26ef0a1a20a8810813a085179220f56d2561a`의 fresh WASM 최적화·최종 exit 0·
  `WebAssembly.compile`·pkg/public 해시 일치를 확인했다. WASM SHA는
  `987a000459ddec38d0095348d425016c30e2b0b173c39c31c212a6e15e814bee`다.
  빌드가 갱신한 기존 공개 JS 바인딩도 같은 산출물로 동기화한다. 생성 바인딩을 수작업으로
  수정하지 않았으며, 1000줄 원칙의 일반 구현 파일이 아닌 기존 자동 생성 공개 파일이다.
- 최종 Native·fresh WASM 각각 44개 조합·813쪽 비교가 모두 90% 이상이다.
  전체 최저는 Native 90.81277%·WASM 91.32974%다. 이는 여러 입력·모드의 비교 횟수이며
  하나의 문서 쪽수가 아니다. 같은 글꼴 경로·새 Print로 기준 Native 41개 조합도 다시 수집했다.
- 중첩 표 17쪽은 Native 최저 94.38594%·WASM 94.22950%,
  11쪽은 각각 98.82287%·98.83280%다. 이 선행 조건 뒤 기존 잘못된 Top 핀을 제거했다.
  실제 부모 clip 대조군과 함께 정렬·내용 포함 검사 5개가 통과했다.
- 신규 `source_frame_control_ownership` 6개는 같은 컴파일된 검사 실행 파일에서
  보정 전 CLI에 대해 6개 모두 배치 assertion으로 실패했고 현재 Native에서 모두 통과했다.
  직전 v8의 전체 TAC 표 높이도 337.7px 대 원본 345.2667px로 별도 실패를 확인했다.
  초기 검사 코드의 컴파일 오류·수집 오류는 재현 통과로 세지 않는다.
- 브라우저 실제 WASM의 PDF 3종 × 2모드와 비 PDF 대조군, fresh WASM AI 제공자 E2E,
  Visual Sweep Python 192건, actionlint가 통과했다. WOFF2 검사에 필요한 Brotli 1.1.0을
  기존 CI Python 설치 단계에 추가하며 별도 테스트 중복 호출은 추가하지 않는다.
- CanvasKit의 sungeo 전체 문서 전환은 bounded preflight 상한·shaping 제약에 따른 정책이다.
  전체 문서 CanvasKit 통과로 세지 않고 실제 34쪽 선 연산의 두 모드 × 두 배율을 검사했다.
  Skia 실제 최소 굵기 1 장치 화소와 기존 Native·독립 Print의 점선 내용 화소 기준을 충족했다.
- 완료한 시각·focused 증거는
  [정식 검증 경로](assets/review_integration_20261026/validation/README.md)에 반영했다.
  기준과의 십진수 하락 비교·전체 Rust 필수 검사·개별 원 PR 검토 문서·통합 PR 제출은 남았다.
  최신 upstream은 여전히 고정 base이며 본인 계정의 열린 PR은 없음을 API로 확인했다.

### 전체 Rust 검증에서 발견한 보정의 적용 경계

- 검증 head `f34157026afad07c423e58a9b3018b9c46593325`에서 fmt·Clippy 3종·workspace build·
  suite/unit-tier 정책이 통과했다. 전체 nextest는 11,621개 실행 중이며 현재 실패 7건을 관측했다.
  신규 소유 회귀 6개는 이 전체 실행에서도 통과했다. 부분 통과를 전체 완료로 표시하지 않는다.
- 온새미로 HWP/HWPX의 46→47쪽은 자체 보정 `7ccf8dffc`에서 시작한다. base·보정 전 CLI는
  두 입력 모두 46쪽이고 v2 이후는 47쪽이다. 처음 달라지는 42쪽의 실제 첫 문단 116은 원본
  위치 1,800HU부터 시작하며, 앞 저장 줄을 이전 실제 쪽이 이미 소비했다. 끝점이 한 줄 공차로
  가까운 것만으로 이 실제 prefix 소유를 버리면 문단 138의 첫 줄을 다음 쪽으로 이월한다.
  선행 `6f59addc82f0d1c1d3752b4615d9e5631384d351`의 확정·임시 원점 구분과 대조했다.
  현재 쪽에서 실제로 수용한 원본 문단들의 새 저장 프레임 시작을 공통
  `height_cursor::paragraph_origin_rewinds`로 확인해야 하며, 숫자 끝점만으로 프레임의
  동일성을 대신하는 가정을 제거한다. 기존 46쪽·꼬리·제목 소유 기대값은 유지한다.
- 가정통신문의 두 실패는 첫 쪽 중첩 표 오프셋과 중국어 끝 프레임이다. `8c699b0c6`의
  부분 배치에서 확정 float frame만 오프셋 소비로 세어, 이미
  `stored_square_table_anchor_offset`를 적용한 원점에 21.36px를 다시 더하는 경로를 확인했다.
  원점을 만드는 분기와 최종 lead 가산을 함께 추적했으며, 완전/부분 배치에서 실제로
  오프셋을 소비한 공통 결과를 전달하는 보정과 기존 두 검사의 재실행이 필요하다.
- 민원 서식 원본은 보정 전·현재 Native가 독립 PDF의 전 2쪽 100%다. 실패한 검사는
  `b26709b8a`에서 추가한 수동 여백 편집 뒤의 무이동 가정이다. 원본을 HWPX로 내보낸 대조군과
  같은 문서의 첫 표 `outMargin.top` 141→0만 변경한 대조군을 새 MCP Hancom 2020 Print로
  각각 산출했다. 두 결과 모두 2쪽이며 둘째 표의 위 경계는 887.508→885.5907px로 이동했다.
  변화 −1.9173px는 원본 여백 141HU=1.88px의 한 번 소비와 일치하며, 0 이동 기대값과 다르다.
  편집 대조 검사를 유지하고 독립 Print에 근거한 방향·원본 여백 관계로 정정한다.
  생성 HWPX를 회귀 fixture로 커밋하지 않으며 실제 Print·입력 변경 recipe·해시는
  [독립 편집 근거](assets/review_integration_20261026/validation/civil-form-independent-margin-edit.json)에 있다.
- 민원 서식의 fresh WASM 추가 진단은 Chrome 시작 30초 timeout으로 렌더 전 실패했다.
  이를 엔진 결함이나 통과로 세지 않는다. 입력의 양쪽 시각 선행 조건을 실제 완료한 뒤 검사를 정정한다.
- 전체 실행 결과를 수집하고 보정한 다음 새 production source에서 영향 입력·전체 회귀·fresh WASM을
  다시 검증한다. 현 단계에서는 개별 최종 검토 문서나 통합 PR을 제출하지 않는다.

- 이전 후보의 전체 실행은 실제 보정으로 source가 바뀌기 전에 중단했다. 최종 수집 결과는
  8,854 PASS·assertion FAIL 7개·SIGINT 3개·미실행 2,757개·기존 skip 45개다.
  3개의 중단을 제품 실패로 세지 않으며 이 실행을 전체 통과로 보고하지 않는다.
  Native Skia 후속 단계는 아직 실행하지 않았고, 새 후보의 필수 검증을 다시 완료한다.

### 저장 프레임 식별 보정의 Native 확인

- 숫자 끝점만으로 prefix를 무효화하는 조건을 보완했다. 현재 실제 쪽에서 온전히 수용한
  원본 문단 사이에 paint와 같은 원점 되감김이 있고 끝점도 대응할 때만 새 프레임으로 본다.
  이미 앞쪽에서 소비한 저장 prefix는 그대로 유지한다. 문서 식별자·새 수치 문턱은 추가하지 않았다.
- v10 Native에서 온새미로 HWP/HWPX 모두 46쪽, HWPX 2024 호환도 46쪽을 확인했다.
  같은 실행 파일을 사용한 CLI 소유 회귀 6개가 통과했고 sungeo 전 94쪽 최저99.85165%를 유지했다.
  초기 검사 스크립트의 stdout/stderr 혼합 JSON 읽기 오류는 수정했으며 제품 실패로 세지 않는다.
- [실제 실행·producer](assets/review_integration_20261026/validation/v10-prefix-focused-results.json)와
  [94쪽 TSV](assets/review_integration_20261026/validation/tsv/stage-v10/sungeo.tsv)를 연결한다.
  이는 해당 Native 보정의 선행 확인이며 전체 Rust·새 production fresh WASM 완료는 아니다.
- 민원 서식도 같은 원본의 Native/fresh WASM 전 2쪽100%를 실제 확인했다. 독립 편집 Print와
  함께 기존 편집 기대값을 정정할 선행 근거로 보존했다. 이제 중첩 원점의 오프셋 소유 보정을 진행한다.


### 중첩 표 원점의 오프셋 중복 보정

- 전체 Rust 검사에서 드러난 가정통신문의 중첩 표 위치·마지막 프레임 실패를 기존 #6712 검사와 대조했다.
  `stored_square_table_anchor_offset`가 이미 소비한 세로 오프셋을 중첩 표 배치에서 다시 더한 것이 원인이다.
- 문단 원점과 이미 확정된 표 원점을 `NestedTableOrigin`으로 구분해 전체·분할 배치가 같은 결과를 소비한다.
  문단 원점은 자신의 시작을 소유할 때 원본 오프셋을 한 번 적용하며 확정 표 원점에는 재적용하지 않는다.
  문서 식별자·임의 높이 분기를 추가하거나 기존 검사 기대값을 바꾸지 않았다.
- 기존 `issue_6712_cell_visual_row_units` 18개가 Native Skia에서 모두 통과했다. 앞서 실패한 두 검사도 포함한다.
  [실행 기록](assets/review_integration_20261026/validation/v11-nested-origin-focused-results.json)을 남겼다.
- 온새미로 두 원본·가정통신문 두 원본은 마지막 저장 제품을 확인한 뒤 해당 2020/2024 MCP Print로 직접 출력했다.
  [입력·출력 근거](assets/review_integration_20261026/validation/safety-reference-prints.json)에 해시·제품·Print mode를 기록했다.
- 추가 대조의 중국어 원본은 새 2024 기준 PDF에서 1쪽 81.20203%이며 고정한 devel도 같은 값이다.
  현재 수정의 하락과 구분하고 기존 오류까지 보정하는 승인 범위에서 해결을 계속한다.
  원본·기준을 교체하거나 미달을 회귀 기대값으로 등록하지 않는다. 최종 전쪽 Native/fresh 최적화 WASM 검증은 미완료다.


### 저장 가로 글줄의 분할 셀 정렬 높이 보정

- #6299 `bc695cd6c`, #6712 통합 #6813 `6a193a648`, 컷 정렬 분리 `b56b6b50f`와
  #8191 `609725346`의 뒤 간격 보정을 추적했다. 같은 vpos·다른 column_start의 좌우 조각은
  이미 CellUnit 생산과 실제 글줄 전진에서 한 물리 줄로 처리한다. 분할 셀의 정렬 합산만 조각 수대로
  다시 더했고, 가로 조각 둘을 소유한 마지막 유닛의 뒤 줄간격도 재소비했다.
- 두 정렬 합산은 기존 `stored_seg_is_row_fragment` 판정으로 생산자와 같은 마지막 조각만 소비한다.
  컷 정렬의 마지막 간격 분리도 같은 물리 줄에 속한 유닛 범위와 마지막 조각의 원본 메트릭을 사용한다.
  새 문서 식별자·수치 상수·backend 분기를 넣지 않고 기존 원본 출처·편집·합성 줄 검사를 유지한다.
- 중국어 가정통신문 Native 1쪽은 81.20203%에서 99.21476%로 개선됐고 2쪽은 93.55035%를 유지한다.
  한국어 두 쪽도 97.56934%·93.09499%로 유지한다.
  [수정 전후 출처와 전쪽 수치](assets/review_integration_20261026/validation/v12-horizontal-row-alignment-native-proof.json)를 기록한다.
- 두 문서의 새 WASM 확인과 현재 전체 원본 49조건의 최종 검증은 진행 중이다.
  아직 새 물리 배치 회귀를 추가하거나 기존 기대값을 변경하지 않았다.


### 기존 기대값 정정·새 글줄 정렬 회귀와 전체 검증 착수

- `56a638b99`의 Native 49조건·911쪽이 전부 90% 이상이다. 최저 90.81277%이며
  기준과 대응되는 46조건·865쪽에서 수치 하락은 0쪽이다. 기존 44조건 TSV의 바이트도 이전
  정식 증적과 같음을 확인했다. [전쪽 실행](assets/review_integration_20261026/validation/v12-native-review.json)을 남긴다.
- 같은 생산 코드의 새 release WASM 진단 빌드(`--no-opt`)로 중국어·한국어·민원서식·중첩 표
  전쪽을 대조했다. 최저는 각각 92.66758%·91.58176%·100%·94.22950%다.
  [WASM 진단과 출처](assets/review_integration_20261026/validation/v12-diagnostic-wasm-review.json)는
  최종 최적화 패키지를 대신하지 않는다.
- 이 선행 검증 뒤 민원서식의 여백 편집 검사를 정정했다. 원 도입 `b26709b8a`·설명 보완
  `ee874b038`(#7538), #4068의 원 해결 #7007→#7016과 후속 그림 축 종료 기록을 대조했다.
  기존 0px 변화 전제는 독립 MCP Print 쌍의 -1.9173px와 맞지 않는다. 원본 위 여백
  141HU(1.88px)를 후속 흐름에서 한 번 소비하도록 기대 관계를 수정하고 기존 0.3px 허용치를
  유지했다. 원본 표의 본문 관계·비겹침을 보존하고 편집 뒤 표 소유도 검사한다.
- `source_frame_control_ownership`에 물리 글줄의 가운데 정렬 관계 1개를 추가했다. 원본 줄
  프레임의 양끝·셀 padding에서 위·아래 여유가 같은지 검사하며 절대 현재 출력 좌표나
  JSON 골든을 쓰지 않는다. 같은 컴파일된 검사에서 수정 전 CLI는 정렬 관계 때문에 실패하고
  수정 후 CLI는 7개 모두 통과했다. 소수 한 자리 JSON의 양자화 허용치 0.25px를 일관되게 적용했다.
- 현재 집중 검사는 중복 실행을 빼고 33개(소유 7·가정통신문 18·여백 3·정렬/clip 5)가 통과했다.
  [수정 전 실패·수정 후 통과와 실제 명령](assets/review_integration_20261026/validation/v12-focused-proofs.json)을 보존한다.
- 다음은 이 code/test head의 필수 Rust lint·전체 nextest·Native 공식 검사다. 최종 최적화 WASM
  전쪽·실제 UI와 모든 로컬 필수 게이트를 완료한 뒤 원 PR별 검토 문서와 통합 PR을 작성한다.

### 전체 검사 후반의 그림 줄 마지막 간격 수용 오류 조사

- `59c7fef16`의 실제 전체 nextest는 11,622개 실행, 11,619 PASS·3 FAIL·기존 skip 45개로 끝났다.
  중단·미실행은 없다. [실제 실패 기록](assets/review_integration_20261026/validation/rust-before-inline-picture-fit-correction.json)과
  [해당 실행 결과](assets/review_integration_20261026/validation/rust-before-inline-picture-fit-correction.json)를 보존한다. 원시 로그는 임시 자료로 최종 PR에 포함하지 않는다.
- 실패는 #6312 공개 슬라이스의 기준 4쪽이 5쪽으로 늘어난 결과다. 첫 본문 원점·머리 표 높이는
  유지되며, 저장 LineSeg가 없는 14번 그림 문단만 첫 쪽 꼬리에서 별도 쪽으로 넘어간다.
  원본 XML의 해당 문단은 빈 글자가 아니라 글자처럼 그림 높이 3805HU를 소유한다.
- 원 #6312, 저장 축 보정 `cd9412d7e`, #7382의 `c685efb11`·`e12d99b91`과 독립 2020 Print 4쪽을
  대조했다. 고정 base CLI는 4쪽이며 보정 v7·v9·v10·v12는 모두 5쪽이다.
- 저장 줄 없는 그림의 원 도입 `946007823`과 이번 `a2129e3f7`의 합성 줄 재조판 연결을 추적했다.
  `inline_flow::plan → finish_row의 물리 row 끝 / next_row_top → typeset_inline_flow의 fit →
  commit_inline_flow의 흐름 끝 → layout inline paint`를 대조했다. 일반 경로는 마지막 줄 간격과
  문단 뒤 간격을 포함한 `plan.end`로 fit을 판단하지만 표 경로는 이미 `plan.occupied_end()`를
  소비한다. 마지막 줄 상자가 들어가도 다음 줄 전진 때문에 그림을 새 쪽으로 옮기는 불일치다.
- 같은 공통 plan의 물리 점유 끝을 일반 fit에도 연결한다. 흐름 전진 `end`는 유지하고,
  실제 부동 도형 점유와 별도 text_rows 소유 경로를 빠뜨리지 않는다. 문서별 조건·좌표 clamp·
  4쪽 기대값이나 허용치 변경은 추가하지 않는다. 기존 #6312와 no-LS 그림의 마지막 간격/이월
  대조군, 독립 전쪽 Native·fresh WASM과 필수 게이트를 다시 확인한다. 아직 보정 완료가 아니다.

- 보정 후 실제 CLI가 기준과 같은 4쪽을 내보냈다. #6312 원본 6개, 기존 no-LS 그림 관계
  4개, 공통 프레임 7개가 모두 PASS다. [실제 17개 집중 검사](assets/review_integration_20261026/validation/v13-focused.json)를 연결한다.
  기존 4쪽·좌표·간격 관계의 기대값과 허용치는 변경하지 않았다.
- 재정동향 Native 전쪽 raw는 99.99813/99.94031/99.87901/99.97917%다. 1–3쪽의
  [직접 비교](assets/review_integration_20261026/validation/images/stage-v13/fiscal-p003-native-compare.png)에서
  첫 쪽 끝 그림, 후속 표와 본문 소유, 빈 줄 뒤의 채무·국고채 문단이 복원됐음을 확인했다.
- 추가 정상 대조 13개는 독립 Print 26쪽이며 Native·WASM 기준 실행이 모두 통과했다.
  [14개 입력·30쪽씩의 기준](assets/review_integration_20261026/validation/v13-additional-base-review.json)과
  [기존 독립 PDF의 바이트 출처](assets/review_integration_20261026/validation/v13-additional-reference-provenance.json)를 보존한다.
  새 후보의 추가 14개 Native도 통과했고 기존 조합의 전쪽 재검증을 계속한다.
- 이 단계의 source와 집중 검사·실패 이력을 일반 커밋으로 고정한 뒤, 같은 source의 최종
  최적화 WASM을 빌드한다. 전체 Native 63개·fresh WASM 63개와 최종 Rust 게이트는 아직
  완료되지 않았으며 모두 실제 완료한 뒤 검토 문서와 PR을 작성한다.

- #6312의 전체 댓글 5개와 원 해결 경로도 재확인했다. 원 PR #6445의 source
  `222c81ef2fbc244087bfe0791959124f95a2930b`는 #6485의 `cd9412d7e`로 반영됐고,
  통합 merge는 `fccec08dd`다. 이전 병행 해결 #6473(`2ca9aa90`)의 기록과 구분한다.
  [#6485 기록](../pr/archives/pr_6485_planet6897_visual_sweep.md)의 줄 전진 축과
  #7382의 후속 머리 표·빈 줄 간격 보정을 함께 보존한다. 이번 오류는 그 기대값을
  변경할 근거가 아니라 합성 그림 줄의 수용 경로가 소비한 물리 끝을 보정할 근거다.

### 그림 줄 수용 보정의 물리 소유 범위 재검토

- 코드 검토에서 `occupied_end()`에 추가한 모든 부동 도형 외곽의 합산을 제거한다.
  BehindText·InFrontOfText 장식은 보이는 것과 본문 흐름을 점유하는 것이 다르다.
  이 합산은 기존 행 하단 계약을 재사용하는 데 필요하지 않으며, 현재 정상 문서 통과를
  근거로 장식의 외곽을 줄 수용 예산으로 바꾸지 않는다. 실제 실행 실패를 새로 검출했다고
  보고하는 항목은 아니며 검토상 소유 범위 보정이다.
- 원래 helper의 행 하단을 그대로 소비하고, 별도 text_rows 소유자는 기존 측정 끝을 유지한다.
  실제 TopAndBottom 배제는 기존 row_geometry·배치 metadata가 처리한다.
  일반 fit 3곳의 공통 하단 소비와 기존 flow end 전진은 유지한다.
- 제 v13 WASM optimizer만 SIGTERM으로 종료했다. 컴파일은 통과했지만 최적화가 완료된
  패키지로 세지 않는다. [종료 이유](assets/review_integration_20261026/validation/v13-wasm-corrective-stop.json)를 보존한다.
  source 변경 뒤 Native·관련 회귀·최종 최적화 WASM과 필수 검증을 다시 실행한다.

- v14 Native 빌드는 exit 0, 9분 05초에 완료됐다. 원본 재정동향의 전 4쪽 render tree가
  바로 앞의 독립 Print 검증을 마친 v13 결과와 바이트 단위로 동일하다.
  [현재 CLI의 4쪽 소유 확인](assets/review_integration_20261026/validation/v14-fiscal-geometry-proof.json)을 보존한다.
- 같은 고정 CLI로 Native 63개·941쪽 비교를 새로 시작했다. 테스트 소스와 독립 기대값은
  변경하지 않았으며 관련 17개는 이후 현재 source의 전체 nextest에 포함해 실행한다.
  앞 단계의 17개 통과를 현재 전체 회귀 완료로 바꾸어 기록하지 않는다.

### 최종 Native와 최적화 WASM 빌드 고정

- 생산 source `f272b81c28b2525c5f6e520ef1e5d499b16672d3`의 Native 63개·941쪽이 전부 90% 이상이다.
  최저 90.81277%, 대응 기준 60개·895쪽의
  수치 하락은 0쪽이다. 실제 [완료 원장](assets/review_integration_20261026/validation/v14-native-review.json)을 연결한다.
- 같은 source snapshot 1828개로 최종 release WASM과 wasm-opt를 완료했다.
  `pkg`·Studio public·고정 패키지 JS/WASM 해시가 같고 WebAssembly.compile이 통과했다.
  [실제 빌드·해시](assets/review_integration_20261026/validation/final-wasm-build.json)를 보존한다.
- 생성 binding과 Native 증거를 커밋한 뒤 그 clean head에서 전체 Rust lint·회귀·Native
  공식 검사를 순차 실행한다. 실제 fresh WASM UI·63개 전쪽 비교도 남아 있으며
  모두 완료하기 전에는 PR을 제출하지 않는다.

- 검토 중 잘못 추적된 원시 로그 15개는 최종 PR 트리에서 제거했다. 임시 원시 로그 사본도 제거하며, 실제 명령·종료 코드·요약·SHA는 정식 실행 영수증에 보존한다. 기존 단계 커밋은 재작성하지 않는다.

## 검증 종료 후 새 devel 확인

- 전체 필수 로컬 검증을 완료한 뒤 fetch한 devel은 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`다. #8116의 저장 줄 없는 표·빈 host 배치가 새로 반영됐다.
- 기존 검증 head `1cd656fdf9a09e88cd1ee9eb92f4c43358ea2559`와 0bb 기준의 Native/WASM 전쪽·전체 회귀 결과를 먼저 고정한다. 새 base 위 리베이스와 양쪽 기능 보존 검증은 아직 남았으며 PR 제출 완료로 처리하지 않는다.

## #8116 리베이스 뒤 기능 보존 검증

- 새 base `faa885e8b54ffc7cdf4196cd43a3103f362499a2` 위로 118개 커밋을 리베이스했다. 이전 head는 `refs/codex/integration-20261026/pre-8116`에 보존했다.
- `line_breaking.rs` 두 진입점의 한 충돌에서 새 글앞/글뒤 표·도형 수용과 기존 no-LineSeg 인라인 그림 수용을 모두 유지했다. [커밋별 대조](assets/review_integration_20261026/validation/rebase-8116-commit-map.json)에서 117개 patch 동일·1개 결합과 원 저자 보존을 확인했다.
- 충돌이 있었으므로 이전 전체 회귀를 새 후보의 전체 성공으로 바꾸지 않는다. 공통 경로·새 base 기능을 함께 확인하고 새 후보의 필수 검증을 완료한다.
- 기존 63개 조합에 incoming #8116의 공개 HWP 4개·15쪽을 추가해 Native/fresh WASM 각각 67개·956쪽을 확인한다. 새 baseline은 editable 3모드를 제외한 64개·910쪽이며 같은 PDF·글꼴로 대조한다.
- #8116의 4개 기준 PDF는 원 기여자가 같은 HWP를 한컴 웹한글기안기에서 인쇄한 원본이며 PDFium 메타데이터를 가진다. 새 MCP Office Print로 부르거나 원 출처를 바꾸지 않는다. 새 base 원본과 PDF는 그대로 보존한다.
- 기본 작업공간의 현재 개선을 유지하면서 다른 source의 producer를 고정하기 위해 ignored `output/pr-review/planet6897-20261026/base-8116`에 detached baseline worktree를 사용한다. 공용 Cargo target은 기본 작업공간의 `target/pr-review` 하나이며 모든 Cargo 호출은 순차다.

### 리베이스 후 Native 보존 확인

- 현재 Native 67개·956쪽이 전부 90% 이상이다. 새 base 64개·910쪽의 같은 PDF·글꼴을 대조해 2%p 이상 하락은 0쪽, 양의 하락 0쪽이다.
- 기존63개·941쪽 중 941쪽의 실제 화소가 리베이스 전과 같다. 새4문서의 표·제목 경계는 독립 Print와 직접 읽었다. [Native 비교](assets/review_integration_20261026/validation/post-rebase-8116/native-comparison.json)와 [판독](assets/review_integration_20261026/validation/post-rebase-8116/native-direct-reading.json)을 연결한다.
- fresh WASM과 리베이스 후 전체 Rust 검증은 아직 남아 있다.

### 리베이스 뒤 최적화 WASM 빌드 고정

- 현재 source의 locked release WASM·wasm-opt가 완료됐고 Native와 source snapshot이 같다. pkg/public 해시와 WebAssembly.compile을 확인했다.
- 이 증거를 커밋한 clean head에서 lint·새 base/충돌 경로 focused·전체 nextest·Native 필수 검사를 순차 수행한다. 실제 fresh WASM 브라우저와67개 전쪽도 별도로 확인한다. 모두 완료 전 PR을 제출하지 않는다.

### 공유 target의 잘못된 WASM 재사용 발견과 재검증

- 첫 리베이스 후 후보 WASM은 cargo 단계가 0.61초에 끝났고 새 기준과 바이너리 SHA가 같았다. 실제 중국어 원본 첫 쪽도 82.98138%로 보정 전 출력이어서, 이 후보 producer와 브라우저·전쪽 결과를 무효로 분리했다. 앞의 빌드 종료 기록은 현재 소스의 유효한 WASM 검증으로 사용하지 않는다.
- 기준 worktree 빌드 뒤 현재 checkout의 이전 파일 시각 때문에 공유 target의 기준 산출물이 재사용된 문제다. 소스 내용과 target을 삭제하지 않고 현재 Rust 소스의 mtime을 갱신했다. `/home/tsjang/rhwp`의 본체·내부 crate가 실제로 재컴파일되는 것을 확인했다.
- 잘못된 후보의 출력과 부분 검사 원장은 임시 경로에 분리했다. 전체 nextest 시작 전 후속 controller를 중단했고 실행 중이던 Cargo 작업은 종료까지 기다렸다. 실제 빌드·검사 실패와 의도적 무효화를 구분한다.
- 새 최적화 WASM이 기준 바이너리와 다르고, 실제 중국어 출력이 90% 이상으로 복원되는지 확인한 뒤 clean head에서 필수 Rust 검사를 다시 진행한다. Native의 고정 producer·67개/956쪽 결과와 올바른 기준 producer는 유지한다. 최종 전쪽 WASM·브라우저 결과는 재빌드한 패키지로 새로 산출한다.

### 리베이스 뒤 최적화 WASM 빌드 고정

- 현재 source의 locked release WASM·wasm-opt가 완료됐고 Native와 source snapshot이 같다. pkg/public 해시와 WebAssembly.compile을 확인했다.
- 이 증거를 커밋한 clean head에서 lint·새 base/충돌 경로 focused·전체 nextest·Native 필수 검사를 순차 수행한다. 실제 fresh WASM 브라우저와67개 전쪽도 별도로 확인한다. 모두 완료 전 PR을 제출하지 않는다.

### 새 base 변경 영역의 실제 WASM 확대 판독

- 올바르게 재컴파일한 WASM으로 네 원본의 여섯 비교 패널과 중첩 셀의 기준/후보/독립 Print 확대 영역을 직접 확인했다. 제목·작은 표·본문·중첩 셀·후속 내용의 소유와 겹침 방지 기능을 보존한다.
- title-between-shapes의 장식 색상 영역 크기와 일부 셀 줄바꿈 등 독립 PDF와의 차이는 새 base에도 있는 외양 차이다. 해당 쪽은 기준/후보 PNG가 동일하다. 새 15쪽 중 Native/WASM 각각 14쪽은 기준과 바이트 동일하며, 중첩 셀 1쪽의 RGB 차이는 별도 확대 대조했다. raw 지표와 내용·행 경계를 유지하며 픽셀 완전 일치를 주장하지 않는다.
- `validation/post-rebase-8116/wasm-direct-reading.json`과 두 incoming-base-output-continuity 원장에 실제 이미지 해시·독립 reference·관찰을 기록했다. 기존 Native 판독도 글꼴이라는 포괄적 표현에 숨기지 않고 장식·줄바꿈 차이를 구체화했다.

### PR 생성의 사전 승인 조건

- 작업지시자의 최신 지시 “pr 은 승인 받고 진행”에 따라 로컬 검증·커밋·검토 기록·실제 PR 본문 준비까지 완료한 뒤 생성 승인을 받는다. 승인 전 원격 push·PR 생성은 하지 않는다. 원 PR comment·close·머지도 현재 범위에 넣지 않는다.
- 승인 대기 시 검토 가능한 최종 커밋과 정식 증거, 본문안을 제시한다. 임시 출력·유휴 debug 정리는 필수 증거를 정식 경로에 반영한 뒤 기존 지침대로 수행한다.

### 최종 로컬 검증 완료와 승인 대기 준비

- code/test 검증 head `2e902abacf328bccdd7780606666ac3b1c5c1ce9`에서 필수 Rust 18단계를 모두 exit0으로 완료했다. 전체 nextest 11,637 PASS·기존 skip 45개, Native lib 합계 4,040 PASS·기존 ignore 12개, 필수 Native 4개 모듈을 실제 실행했다.
- Native/fresh 최적화 WASM은 각각 67개 조합·956쪽 전부 90% 이상이며, 기준 64개·910쪽에서 양의 하락과 2%p 이상 하락 모두 0쪽이다. 리베이스 이전 63개·941쪽은 양쪽 경로 모두 PNG 바이트 동일하다.
- 현재 입력·독립 PDF 125개와 Git blob을 대조했다. 실제 PDF import 중간본 12개는 생성 영수증으로만 식별한다. 소유 관계 17개·새 base 15개는 완료된 전체 실행의 실제 PASS 항목으로 확인했으며 별도 실행 수로 더하지 않는다.
- 원 PR 20건의 head·reviewer·미해결 스레드를 재조회해 고정 head 유지·reviewer 20건 유지·미해결 0건을 확인했고 review/impl 40개와 오늘할일을 갱신했다. 문서 43개·내부 링크 770개는 누락·임시 전용 증거 링크 없이 통과했다.
- 최신 devel fetch 결과는 기존 기준 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`와 같아 추가 리베이스·회귀 반복은 필요 없다. 본인 열린 #8319는 조회 시 CI 진행 중·실패 없음이며 다른 작업을 수정하지 않았다. PR 생성 승인 후에는 상태를 다시 확인한다.
- 최종 로컬 기록을 커밋한 뒤 본문안을 생성한다. 사용자의 사전 승인 전 원격 push·PR 생성·머지·원 PR comment/close는 진행하지 않는다.

## #8319 반영 후 리베이스 재개

- 사용자가 PR 생성을 승인한 뒤 최신 devel을 확인했을 때 #8319의 merge
  `af6bb05b26887135fb9da0fa4e54ca7205564fca`가 반영돼 있었다. 이후 지시대로
  #8319 후속 처리를 중단하고 기존 20건 통합 브랜치의 리베이스를 재개했다.
- 이전 head `3a2a081a418a38a2fb696136835d0f721b216972`는
  `refs/codex/integration-20261026/pre-8319`에 보존한다. 123개 커밋과 원 저자를 유지했으며
  추가 PR은 체리픽하지 않았다. 리베이스 후보는 `0508d3afcdd233b59ea1f0ff04013291d8b0c254`다.
- [충돌 해소·현재 검증 상태](assets/review_integration_20261026/validation/post-rebase-8319/README.md)와
  [커밋별 대응](assets/review_integration_20261026/validation/post-rebase-8319/commit-map.json)을 기록했다.
  자동 보정 오류를 정정한 중간 후보는 검증 근거로 사용하지 않는다.
- suite 파생 경로를 현재 source로 준비하고 fmt를 완료했다. 기존 전체 검증은 이전 기준의
  이력이며 새 기준과 결합한 현재 source의 기능 보존·전체 회귀·전쪽 검증을 대신하지 않는다.
  현재 검증을 완료하기 전에는 원격 push·PR 생성·머지를 진행하지 않는다.

### 리베이스 후 중첩 이어받기 정렬 재진단

동일 51개 글꼴과 이전 시스템 대체 조건에서 issue2007 3쪽을 비교했다. 이전 및
현재 버전의 상위 셀은 모두 y=113.386667px, 내용 상자 높이 715.280000px이고,
이전 정렬 내용 높이 711.506667px는 상위 원점 115.273333px을 만들었다. 현재
정렬 내용 높이는 715.280000px로 원점 113.386667px이다. 차이 3.773333px는
이어받기 위 띠와 같으며 가운데 정렬에서 자식 원점이 1.886667px 올라간다.

이는 #8319의 #7518·#8060 이어받기 띠 정렬 보정과 같은 코드 경로다. 해당 보정의
독립 Print 근거와 검사를 유지하면서 원본 저장 프레임과 정렬 소유를 구분해야 한다.
현재는 원인 진단만 완료했으며 수정이나 최종 통과로 판정하지 않는다. 한컴 PDF에서
자식 표 왼쪽 바깥 테두리의 시작은 약 117.15px이고, 이전 119.04px·현재 117.16px다.
개별 글줄 위치와 전체 페이지 차이도 함께 검증해야 한다.

임시 실행 로그는 `output/pr-review/planet6897-20261026/rebase-8319/issue2007-old-origin-build.log`,
`issue2007-old-origin.log`, `issue2007-old-alignment.log`, `issue2007-new-alignment.log`다.

### 하이픈 선분과 한컴 Print 추가 대조

중첩 표 3쪽의 큰 픽셀 차이는 한양신명조의 연속 하이픈을 겹치지 않게 짧은 선분으로
출력하는 기존 공통 경로에서 발생한다. SVG에는 선분과 검색용 투명 텍스트가 함께 있어
투명 텍스트의 기준선만으로 인쇄 잉크 위치를 판정할 수 없다.

독립 한컴 Print PDF를 96dpi로 래스터화해 같은 14pt 연속 하이픈을 확인했다.
#3308의 8쪽은 PDF 기준선 358.97px, 검정 선분 행 352px이고, issue2007의
3쪽은 기준선 812.39px, 검정 선분 행 805px이다. 두 PDF의 글꼴 크기는 약
18.70px이다. 현재 SVG 선분을 변환한 위치는 각각 약 350.98px, 804.31px다.
두 입력은 같은 방향으로 선분이 높다. PDF 잉크 행의 중심과 대조해 공통 인쇄
선분을 보정하는 후보를 검증하되, #3308 전쪽 및 관련 Native/fresh WASM 하락
검사와 기존 회귀를 함께 통과해야 한다. 아직 수정이나 통과 판정은 하지 않았다.

진단 로그는 `output/pr-review/planet6897-20261026/rebase-8319/print-dash-baseline-evidence.log`,
`dash3307-current.log`, `issue2007-current-svg.log`이고 확대 판독 자료는 같은 폴더의
`issue2007-lines-comparison.png`다.

### 연속 하이픈 선분 보정과 Native 결과

`f5c397284`는 일반 한양신명조의 겹치는 연속 하이픈 선분을 두 한컴 Print에서
확인한 잉크 위치에 맞췄다. 굵게·기울임의 기존 위치는 유지한다. 원본 프레임
회귀에 두 인쇄 선분의 실제 위치를 보호하는 검사를 추가했다. 보정 전 SVG는 두
독립 잉크 행에서 각각 약 1.52px·1.19px 벗어났고, 보정 후 검사를 통과했다.

새 Native 실행 파일의 issue2007 전 17쪽은 독립 Print와 쪽수가 일치하고 최저
94.47057%다. 리베이스 전보다 0.1%p 이상 하락한 쪽은 없으며, 3쪽은
99.79139%에서 99.84704%다. #3308 원본의 전 9쪽도 보정 전 대비 0.1%p 이상
하락이 없고 최저 96.67154%이며, 8쪽은 99.87867%에서 99.95277%로 개선됐다.
[쪽별 TSV와 해시 기록](assets/review_integration_20261026/validation/post-rebase-8319/dash-fix-native-interim.json)을 따른다.

원본 프레임 8개, #8060 4개, #3308 1개 집중 검사는 통과했다. #3308을 처음
독립 Cargo 타깃으로 잘못 호출한 실행은 오류였으며, 생성된 묶음 타깃으로 재실행해
통과했다. 새 최적화 WASM·전 범위·전체 Rust 검사와 #8188의 최종 WASM 정수 픽셀
검사는 진행 중이므로 현재 결과만으로 통합 PR 승인 준비 완료로 판정하지 않는다.

### 과거 WASM 빌드 조건 추가 확인

과거 956쪽 결과에 대응한 `post-rebase-8116/wasm-build.json`은 release
`codegen-units=16`을 기록한다. 현재 진행 중인 새 WASM 빌드 프로세스에는 해당
환경 변수가 설정되지 않았고 Cargo.toml의 release 기본값은 1이다. 이전 소스
재빌드의 코드 생성 단위도 기존 진단 JSON에 기록돼 있지 않아 동일하다고 단정할 수
없다. 이 차이가 계산 결과나 과거 TSV 재현에 영향을 주는지는 아직 미확인이다.
과거 producer head `523af99acf2c53265e390a4e982eb07dd1071099`와 리베이스 전
`3a2a081a418a38a2fb696136835d0f721b216972` 사이에는 `src`, Cargo.toml,
Cargo.lock의 차이가 없다. 과거 산출물 SHA와 현재 재빌드 SHA가 다르다는 사실만으로
렌더 회귀를 판정하지 않는다. 실제 동일 빌드 조건으로 재현해 원시 페이지 값과 함께
확인해야 한다.

### 공유 target의 이전 차트 의존성 정정

이전 소스 worktree를 같은 `target/pr-review`에서 진단 빌드한 뒤 생성한 Native
실행 파일 `d4fc9e...`와 `4be33b...`가 이전 차트 라이브러리를 재사용했다.
#8319 기준과 통합 전 실행 파일은 차트 124·125·128·129 사례에서 약 99.7%이나
해당 실행 파일은 약 89%로 떨어졌다. 같은 원본을 SVG로 내보내 실제 확인한 결과,
실행 파일의 차트 바깥선 `#8c8c8c`/0.75와 제목 원점은 이전 worktree의 차트
renderer에 해당하고 현재 소스는 `#868686`/1 및 다른 원점을 사용한다.

따라서 이 실행 파일로 기록한 67건 Native·sample16 정수 픽셀·선분 Native 결과를
현재 소스의 최종 통과로 인정하지 않는다. 384쪽 비교 결과 자체는 실제 출력에 관한
관측으로 보존한다. WASM의 차트 의존성도 별도 대조한다. 진행 중인 전체 Rust 검사의
실행 출처를 확인하고, 공유 target의 현재 workspace 소스와 의존성을 명시적으로
다시 빌드한 뒤 필요한 모든 전쪽·회귀를 재실행한다. 이전 출력의 기준값을 내리거나
차트 수치만 별도로 제외하지 않는다.

WASM 차트 대조에서도 같은 문제가 확인됐다. `incoming-124`의 새 WASM 출력은
87.98250%, #8319 기준은 99.66483%다. 빌드 target의 차트 `.d`는 이전 소스의
단일 renderer 파일만 참조하며 현재 소스의 분리된 renderer 하위 모듈을 포함하지
않았다. Native뿐 아니라 WASM 빌드와 그 산출물의 브라우저·전쪽 결과도 현재 소스
최종 검증에서 제외한다. 실행 중인 전체 Rust 컴파일 역시 실제 `--extern`이 이전
차트 `.rlib`을 참조하는 것을 확인했으므로 중단했다. 소스 내용은 바꾸지 않고 현재
차트 crate 소스 시각을 갱신해 의존성을 명시 재빌드한 다음 차트 출력부터 검사한다.

### 현재 소스로 다시 빌드한 Native 중간 검증

차트 의존성을 현재 소스로 재빌드한 Native 실행 파일 SHA-256은 `3a7b0c02eb624c38e75ef18240abe2c6c39c2aabb24f43d38be4dc0757444b0a`다. 실제 차트 SVG의 테두리 색·두께는 `#868686`/1이며 해당 쪽 점수는 99.80761%다. sample16 세 문서의 Native 192쪽 PNG를 독립 Print PNG와 양방향 반경 2px로 직접 비교해 미대응 픽셀 0을 확인했다. 기존 범위 완료분 56건·693쪽에서 90% 미만과 0.1%p 이상 하락은 없었다. 이는 중간 결과다. WASM 전쪽, 나머지 비교, 전체 Rust 및 브라우저 검사는 완료 전이다. [중간 기록](assets/review_integration_20261026/validation/post-rebase-8319/corrected-native-interim.json)

### 현재 소스로 재빌드한 WASM 하락 조사

`release` codegen-units 16으로 새 빌드한 WASM SHA-256은 `dc7dd9a1a9a725a486064356ad583032d02394ad75cb64de8f1809ccfe87248b`다. 차트 검증 쪽은 #8319 기준과 동일한 99.66483%지만 과거 통합 TSV와 비교한 여러 쪽에서 0.1%p 이상 하락했다. 중첩 셀 1쪽은 98.57262%에서 98.35045%로 떨어졌다. 같은 51개 글꼴 프로그램을 같은 순서로 공급했으며 힌팅을 끄거나 최대로 바꾸고 ExtraLight family를 제외한 별도 진단은 해당 PNG를 바꾸지 않았다. 과거 WASM 바이트는 남아 있지 않아 원인을 단정하지 않는다. 기준을 낮추지 않고 조사·보정한다. 새 WASM의 AI 제공자 브라우저 검사 34항목과 PDF 가져오기 UI 검사 24항목은 통과했다. [하락 진단](assets/review_integration_20261026/validation/post-rebase-8319/corrected-wasm-decline-diagnostic.json)

### #8188 새 Native·WASM 전쪽 픽셀 확인

차트 의존성을 재빌드한 Native 및 WASM으로 sample16 3개 문서를 각각 64쪽씩 다시 렌더링했다. 독립 Print PNG와의 양방향 반경 2px 정수 비교에서 384/384쪽의 미대응 픽셀이 0이다. 실제 PNG별 SHA-256과 입력·실행 신원은 [픽셀 계약](assets/review_integration_20261026/validation/post-rebase-8319/final/sample16-pixel-contract.json)에 기록했다. 이는 #8188 범위의 확인이며 다른 기존 문서의 WASM 하락을 해결했다는 뜻은 아니다.

중첩 셀 1쪽을 브라우저 글꼴 추적으로 확인한 결과 일반 환경에서는 시스템 HCR Dotum을 실제로 사용한다. 독립 PDF의 HCR Dotum CID 글꼴에서 확인한 256개 문자의 윤곽과 시스템·한컴 설치 글꼴의 윤곽은 모두 일치하고 힌팅 프로그램도 같다. 시스템 글꼴을 임의 제한하면 Noto Sans KR ExtraLight가 선택돼 점수가 올라가지만 같은 PDF 글꼴의 윤곽과 일치하지 않으므로 정답이나 최종 통과 근거로 사용하지 않는다. 이전 소스와 이전 글꼴 등록 도구를 함께 사용해도 98.35047%였으며 Chromium 155와 156에서 PNG가 같았다. 동일 글꼴의 과거 출력 차이는 계속 조사한다. [시스템 글꼴 조사](assets/review_integration_20261026/validation/post-rebase-8319/corrected-wasm-system-font-investigation.json)

### 전체 회귀에서 발견한 가맹사업 쪽수와 표 뒤 문단

현재 Native 실행 파일 및 전체 Rust 검사에서 가맹사업 문서가 독립 Print·기준의 142쪽과 달리 143쪽이다. 첫 차이는 118쪽이며 중첩 표의 이전 컷 `[58]`이 실제 높이 재검사 후 `[57]`로 바뀌어 119쪽에 작은 조각이 추가된다. 7개 기존 검사가 이 쪽수에서 실패한다. 별도의 `split-rows-60-policy-1-body-false-flow-false.hwp`는 기준·Print의 표 뒤 본문 y=310.7px가 현재 307.0px로 바뀌어 px 소유 검사가 실패한다. 표 자체 괘선은 일치한다. [재현 기록](assets/review_integration_20261026/validation/post-rebase-8319/franchise-float-regression-diagnostic.json). 기대값을 바꾸지 않고 저장 컷의 물리 프레임과 마지막 표 조각 이후 간격을 좁혀 보정해야 한다.

### #8328 반영 리베이스와 후속 검증 (진행 중)

리베이스 전 base는 `af6bb05b26887135fb9da0fa4e54ca7205564fca`, 새 base는
`db5cd5bcc60a69e42f419f18a7f57a33b6fb7fdc`(#8328), 리베이스 직후 후보는
`acc95761e7`이다. 작업 브랜치 `review/integration-20261026-document-updates`의
173개 커밋을 내용 충돌 없이 재적용했다. 처리 중 인덱스 잠금 충돌은 해제 확인 후
재개했다. 새 base의 조판·EMF 변경과 원본/대체 입력 구분, 독립 Print를 같은 조건의
Native/fresh WASM으로 대조하고 현재 작업의 개선도 보존해야 한다.

위 가맹사업·표 뒤 간격의 실패는 이후 보정해 리베이스 전 후보에서 독립 Print의
가맹사업 142쪽과 사양 문서 71쪽을 복원했고, 관련 회귀 57개가 통과했다. 문제 문서의
11·16쪽은 각각 95.56383%, 98.57242%이고 전18쪽 최저 94.1568%, 기준 대비
0.1%p 이상 하락은 없다. sample16 Native 192쪽은 양방향 반경 2px 미대응 0이다.
이는 리베이스 전 결과이며 새 조판 코드에 대한 시각 확인은 아직 필요하다.

위에서 과거 WASM 바이트가 없다고 적은 것은 당시 상태다. 원 소스를 같은 codegen 설정으로
재빌드해 기존 SHA-256 `c5b25dd8b22efce0d4ecb08291069e3ec625345c40957cbe457cc431f69737d8`와
일치하는 WASM을 복원했다. 같은 바이너리도 현재 브라우저 환경에서 과거 TSV 일부를 재현하지
못하므로 해당 환경 차이는 미해결이다. Native 뉴스레터 2쪽은 과거 글꼴 선택 조건을
프로세스에 재현했을 때 이전 PNG와 SHA-256까지 일치했다. 기준을 낮추지 않고 동일 조건에서
전쪽을 재검증한다. 진행 중인 리베이스 전 실행과 새 후보 검증은 별도로 기록한다.

과거 WASM TSV 대비 0.1%p 이상 차이가 난 중첩 표 1쪽, 온세미로 HWP·HWPX 각 4쪽,
뉴스레터 1쪽과 미주 문서 2쪽을 추가 대조했다. 과거와 동일한 해시의 최적화 WASM과
새 후보를 같은 현재 글꼴 설정에서 비교하면 이 12쪽의 최대 하락은 0.01347%p다.
[선택 쪽별 세 수치와 입력·PDF·바이너리 해시](assets/review_integration_20261026/validation/post-rebase-8328/historical-wasm-spot-comparison.json)를
기록했다. 이는 원래 TSV의 실행 환경이 복원됐다는 증거가 아니며 전체 WASM 범위의
0.1%p 기준 통과를 뜻하지 않는다. 환경 차이와 나머지 쪽의 비교는 계속한다.
보존된 과거 WASM 원본 PNG 세 장(중첩 표 1쪽, 9월 미주 20쪽, 11월 미주 21쪽)을
현재 기준 PDF PNG와 직접 재비교해 과거 TSV의 98.57262%, 96.35945%, 91.32974%를
각각 재현했다. 과거 PNG와 현재 환경의 같은 과거 WASM PNG를 서로 비교한 수치는
각각 100%, 100%, 99.91143%다. [이미지 해시와 직접 비교](assets/review_integration_20261026/validation/post-rebase-8328/historical-representative-png-comparison.json)는
수치 차이의 공간적·환경적 원인이나 다른 아홉 쪽의 과거 이미지를 대신하지 않는다.
세 이미지의 과거 PNG와 현재 환경 재산출 PNG에 대해 전체 이동 ±3px을 탐색한 결과
모두 이동 0px에서 이진 마스크 차이가 가장 작았다. 중첩 표는 반경 1px에서 100%,
9월·11월 미주는 각각 98.92553%·99.02440%이며 반경 2px 결과는 위와 같다.
이는 단일 전역 위치 이동으로 차이를 설명하기 어렵다는 진단이며 글꼴 또는 장치
래스터가 원인이라는 확정은 아니다.

기존 통합 범위 fresh WASM 실행은 67개 입력·956쪽을 완료했다. 90% 미만과 실행
오류는 없지만 과거 TSV와 직접 대조하면 12쪽이 0.1%p 이상 낮아 **미통과**다.
[전쪽 비교 원장](assets/review_integration_20261026/validation/post-rebase-8328/original-wasm-comparison.json)에
입력·PDF·WASM 해시와 전쪽 수치를 보존한다. 선택 쪽의 동일 환경 비교만으로 이
미달을 통과로 대체하지 않으며 원래 환경 차이를 계속 조사한다.
선행 보정 WASM에서 한컴 HCR 글꼴을 추가 공급한 진단은 중첩 표 1쪽의 과거
98.57262%를 98.57264%로 재현했지만 나머지 선택 7쪽의 미달은 해소하지 못했다.
[글꼴 공급 선택 쪽 진단](assets/review_integration_20261026/validation/post-rebase-8328/historical-hcr-spot-diagnostic.json)은
최종 후보 검증이 아니며 단일 글꼴 추가로 전쪽 환경이 복원됐다는 근거가 아니다.
이후 실제 과거 WASM과 최종 후보 WASM 양쪽에 동일한 한컴 HCR 글꼴을 공급해 중첩 표
1쪽을 재검증했다. 과거 WASM PNG의 SHA-256이 보존된 과거 PNG와 완전히 같고 과거
98.57262%, 후보 98.57264%로 0.1%p 하락이 없다. [정확한 과거 이미지 재현과
후보 비교](assets/review_integration_20261026/validation/post-rebase-8328/historical-exact-hcr-cell.json)는
이 한 쪽의 근거이며 나머지 열한 쪽의 과거 조건 재현까지 의미하지 않는다.
미주 PDF의 한컴바탕 윤곽은 9월 문서에서 대응 가능한 271개, 11월 문서에서 324개가
서버 원본과 같았다(9월 유니코드 하나는 원본에서 찾지 못함). 한컴돋움은 각각
25·27개 윤곽이 달랐지만 대상 쪽의 추출 텍스트에는 그 글자가 없었다.
한컴바탕·돋움을 명시 공급한 과거 WASM 9월 20쪽은 95.32996%로 과거 96.35945%를
재현하지 못했다. [PDF 윤곽·글꼴 공급 진단](assets/review_integration_20261026/validation/post-rebase-8328/endnotes-hancom-font-diagnostic.json)은
추출되지 않은 표시에 대한 판정이나 원인 확정이 아니다.
한글만 대상으로 하던 Canvas 글꼴 표본 탐색을 수식 문자까지 확장하자 9월 20쪽은
`HyhwpEQ` 요청이 가장 많았다. PDF 수식 글꼴의 대응 글리프는 두 문서에서 각각
133개·132개 모두 한컴 2020 설치본 윤곽과 일치했다. 같은 이름의 다른 서버 조사
글꼴은 일부가 다르거나 빠졌다. 해당 한컴 설치본을 과거 WASM에 명시 공급한 결과
9월 20쪽 PNG는 기존 현재 환경 출력과 바이트가 같았고 95.30078%로 과거 PNG를
재현하지 못했다. 위 미주 진단에 근거를 추가했으며 나머지 원인을 계속 확인한다.
9월 20쪽에서 과거 WASM과 과거 Canvas exporter를 현재 글꼴 조건으로 함께 실행한
PNG는 현재 exporter로 같은 WASM을 실행한 PNG와 바이트가 같았으며 95.30078%다.
보존된 과거 PNG·96.35945%는 재현되지 않았다. 위 미주 진단에 두 exporter와 WASM,
PNG 해시를 기록했으며 exporter 변경만으로 차이를 설명하지 않는다.
미달이 있는 중첩 표·온세미로 HWP·뉴스레터·9월 미주 문서의 기존과 현재
명시적 글꼴 공급 목록은 각각 51개 파일의 해시와 순서가 모두 같다.
[글꼴 공급 비교](assets/review_integration_20261026/validation/post-rebase-8328/wasm-font-supply-comparison.json)는
시스템 글꼴 선택까지 같다는 증거가 아니므로 환경 차이는 계속 조사한다.
현재 `fontconfig` 진단에서는 sample16용으로 추가된 시스템 글꼴 220개가 검색되며,
해당 경로를 제외하는 조사 설정에서는 0개다. 두 설정의 `fc-match` 결과는 한양신명조와
함초롬돋움에서 달라진다. [설정·선택 결과](assets/review_integration_20261026/validation/post-rebase-8328/fontconfig-selection-diagnostic.json)를
기록하되, 이는 Chromium의 실제 글꼴 사용이나 과거 환경 전체의 복원을 입증하지 않는다.
미달 문서 네 건의 선행 Chromium 글꼴 진단에서 관찰된 family 8종을 두 설정에서
`fc-match`로 조회하면 모두 같은 파일이 선택됐다. [관찰·조회 결과](assets/review_integration_20261026/validation/post-rebase-8328/wasm-platform-font-diagnostic.json)는
두 설정에서 Chromium 전체를 실행한 비교나 과거 선택의 증거는 아니다. 이전에 확인한
일부 이름의 시스템 선택 차이만으로는 열두 쪽의 차이를 설명할 수 없다.
온세미로 HWP의 독립 PDF에는 KoPub 계열 세 글꼴이 포함돼 있다. 같은 문서의
선행 Chromium 표본 글자 탐색에서 `KoPubWorld돋움체_Pro Medium`을 요청한 항목은
`Haansoft Batang`으로 선택됐으며 명시적 공급 목록에는 KoPub family가 없다.
[PDF·탐색·목록 근거](assets/review_integration_20261026/validation/post-rebase-8328/onsemiro-kopub-font-diagnostic.json)는
표본 결과이므로 전체 글자나 과거 실행의 선택을 대신하지 않는다. 실제 원본의
글꼴 대응을 추적하기 전 임의의 글꼴 매핑으로 수치를 맞추지 않는다.
원본 HWP 글꼴 목록에도 KoPubWorld 돋움·바탕과 `_Pro` 이름이 있다. PDF에서 실제
사용한 KoPubWorld 돋움 Light·Bold, 바탕 Light 세 글꼴의 ToUnicode·Identity
CID를 통해 각각 82·12·855개 글리프를 현 서버의 대응 글꼴과 대조했고, 분해된
윤곽 명령과 좌표가 모두 같다. [독립 윤곽 비교](assets/review_integration_20261026/validation/post-rebase-8328/onsemiro-kopub-outline-identity.json)에
PDF와 글꼴 해시를 기록한다. 글리프 윤곽 동일성만으로 전체 메트릭이나 과거
Chromium의 선택까지 같다고 판정하지 않는다.
후보 WASM의 온세미로 HWP 19·26·33·38쪽에 이 세 글꼴을 명시 공급한 진단은
네 쪽 수치가 기존 후보와 모두 같았다. [공급 진단](assets/review_integration_20261026/validation/post-rebase-8328/onsemiro-kopub-supply-diagnostic.json)에
글꼴 목록과 입력·PDF·WASM 해시를 기록한다. 폰트 목록에 추가된 것만으로 실제
글자별 사용 여부나 과거 환경 재현을 주장하지 않는다.
글꼴 추가 후 26쪽 Chromium 표본의 KoPub 요청은 여전히 `Haansoft Batang`을 선택했다.
독립 PDF의 19·26·33·38쪽 텍스트 추출에도 해당 글꼴이 나타난다. 같은 진단에
쪽별 추출 글꼴 수와 실제 표본을 추가했다. 추출되지 않는 표시와 다른 글자까지
판정하지 않으며, 요청 이름만으로 KoPub 강제 치환을 정당화하지 않는다.

#8319 반영 범위 Native 재검증은 197개 입력·256쪽을 완료했다. 실제 입력·PDF와
TSV 해시, 쪽수와 리베이스 전 후보 대비 각 쪽 수치를 대조했으며 실행 오류,
0.1%p 이상 하락 및 90% 미만이 모두 없다. 결과는
[전쪽 비교 원장](assets/review_integration_20261026/validation/post-rebase-8328/incoming-native-comparison.json)에
보존한다. 같은 범위의 fresh WASM 검증과 전체 Rust 검사는 아직 진행 중이다.
사용자의 지시로 원본 `gulim.ttc`를 시스템에 등록하는 동안 기존 #8319 WASM 검사가
진행 중이었다. 등록 전 산출된 입력 15의 3쪽을 같은 WASM·명시 글꼴 목록으로 등록 후
재실행한 결과 raw 수치는 양쪽 99.95693%, PNG SHA-256도
`0fb3ae994c1af6581df82077eb9a4ac37638bf15046356ee1728d865778ac12d`로 일치했다.
한 쪽 대조는 다른 쪽의 글꼴 선택까지 보증하지 않으며 최종 실행 출처를 함께 확인한다.
#8319 관련 WASM 197개 입력·256쪽의 추가 표 간격 보정 전 실행은 비교 기준
`f5c3972844` 대비 0.1%p 이상 하락, 90% 미만, 실행 오류가 없었다.
[보정 전 전쪽 원장](assets/review_integration_20261026/validation/post-rebase-8328/incoming-wasm-before-float-fix.json)에
실제 WASM·원본·PDF 해시와 쪽별 수치를 보존한다. 이것은 아래 추가 보정 전의 결과이고
최종 코드의 검증을 대신하지 않는다.

#8328 기준과 후보의 fresh WASM은 55개 입력·329쪽을 같은 조건으로 비교했다.
입력·PDF·WASM 해시, 쪽수와 쪽별 수치를 확인했고 0.1%p 이상 하락과 실행 오류는
없다. 다만 `claim-8045` 1쪽, `claim-8049` 10·82쪽, `claim-8056` 11쪽,
`claim-8062` 30·31쪽은 기준에서도 미달인 기존 예외로 90% 미만이다.
[전쪽 비교 원장](assets/review_integration_20261026/validation/post-rebase-8328/fixed-wasm-matrix.json)에
이 여섯 쪽을 표시하고 90% 통과로 세지 않는다.
진행 중인 WASM 비교 임시 원장의 `binary_sha256`은 WASM이 아니라 `--rhwp-bin`으로
함께 지정된 Native 보조 파일의 해시다. 최종 근거를 만들 때는 실제 `--wasm-pkg`의
`rhwp_bg.wasm` 해시를 따로 계산해야 한다. 위 선택 비교 기록의 잘못된 WASM 해시는
실제 패키지 해시로 정정하고 Native 해시도 별도 필드에 보존했다. base와 후보의
WASM 해시는 각각 `266dc44808d31a7fb49eb025be0b4b052f77fe7e114edeaa52fd8534d34c4322`,
`3faf7e1bca135d57bf70049d3ee63290c4892cea5be30c2ca6fdf2f772faa01a`다.

#8328 기준과 리베이스 후보의 Native 동일 조건 비교는 55개 입력의 329쪽을
완료했다. 쪽 수 불일치나 실행 오류는 없으나 #8057 공식 대체 입력의 15쪽이
90.49374%에서 90.36600%로 0.12774%p 하락해 허용 기준을 넘는다. 동일 글꼴
설정으로 재실행해 같은 값을 확인했으며, 렌더 트리와 SVG는 같고 Native 표 선
래스터에서 차이가 관찰됐다. #8125의 한 장치 픽셀 하한과 기존 괘선·점선 개선을
유지하는 보정 및 전체 비교가 필요하다. #8328에서 명시적으로 기존 미달을 유예한
#8045·#8049·#8056·#8062의 여섯 쪽은 90% 미만 그대로이며 통과로 기록하지 않는다.
sample16은 새 Native에서 192쪽 모두 양방향 반경 2px 미대응 0이며 fresh WASM은
아직 검증 전이다. 전체 Rust 회귀검사는 실행 중이다.

#8057 15쪽 표의 원본 `noAdjust=1`, 저장 행별 높이 합과 전체 높이는 모두
`45570` HWP 단위이고, 반복 행 높이 `1965`는 한컴 PDF의 약 26.2px 간격과
일치한다. 기존 후보는 일부 행을 약 37.1px로 늘리고 다른 행을 약 24.6px로
줄였다. 표 점선 위치 차이는 #8328 기준에도 존재했으며 통합 브랜치의 가는 획
보정으로 차이가 더 뚜렷해졌다. 저장 프레임이 닫히는 인라인 병합 표의 행
경계와 독립 PDF의 두 위치를 보호하는 보정을 준비했으며, 이 보정은 아직
컴파일·회귀·전쪽 시각 검증을 통과한 상태가 아니다.

전체 Rust 검사 중 #5128의 HWP→HWPX 쪽수·표 이어받기 검사 두 개가 실패했다.
리베이스 직전 최종 Native 바이너리 `6eb6a59609d9311d4254b9074c14f589809cae2335ae7ebf32d6b66ae720b4c8`에서도
원본 71쪽과 왕복본 74쪽으로 재현되므로 #8328 리베이스 이전에 이미 생긴 문제다.
최초 차이는 원본 57쪽의 문단 372 표가 왕복본에서 다음 쪽으로 이동하는 것이다.
제목 간격 복원은 HWP5 유래 HWPX에 원본 HWP 계보를 적용하도록 조판·렌더 양쪽의
조건을 보정 중이며, 기존 왕복 검사의 기대값을 유지해 재검증해야 한다.

리베이스 직후 코드의 전체 `nextest`는 11,897개 실행 중 11,895개 통과,
#5128 두 개 실패, 45개 제외로 종료했다. 마지막 #6981 전쪽 행 검사 자체는
1,794.409초에 통과했으나 이전 실행의 795.009초보다 오래 걸렸다. 서로 다른
실행 시점과 동시 부하를 고려해야 하며 이 수치만으로 성능 회귀를 확정하지 않는다.
이 전체 실행은 위 두 보정의 적용 전 결과다. 보정 뒤 실패 검사와 인쇄 경계,
관련 문서 전쪽 비교를 별도로 실행해야 한다.

HWP 유래 HWPX의 표 뒤 간격 보정 `db79f8d2c7`과 인라인 noAdjust 병합 표
저장 행 보정 `94a868b34d` 적용 후 #5128의 기존 검사 다섯 개와 #8057의
독립 Print 행 경계 검사 한 개가 모두 통과했다. #8057 전쪽 Native 및 WASM
수치, 원래 통합 문서·sample16의 전쪽 보존은 아직 재확인 중이다.

#8057 공식 입력 15쪽은 같은 글꼴 설정과 Hancom 2020 PDF로 Native를 다시
측정해 raw 99.16179%다. #8328 기준 90.49374%, 보정 전 후보 90.36600%와
비교하면 0.12774%p 하락은 해소됐다. 이는 해당 한 쪽 결과이며 나머지 55개
입력과 sample16의 전쪽 검사 결과를 대체하지 않는다.

보정 후 기존 가는 실선·점선의 Native 화소 검사 두 개, sample16 세 문서의
64쪽 내용·소유 검사 세 개, 저장 표 분할 검사 세 개는 모두 통과했다. sample16
Native 세 문서 192쪽은 실제 PNG와 독립 Print를 양방향 반경 2px로 다시 비교해
미대응 0이며 정식 증거는 `post-rebase-8328/sample16-fixed-native/`에 있다.
fresh WASM과 다른 입력의 전쪽 비교는 별도로 진행 중이다.

사용자 추가 요청 #7723: `samples/hwp3-sample16-hwp5-2010.hwp`는 기본 이름
HWP와 SHA-256이 같은 별칭이며, 실제 한컴 2024 PDF를 기준으로 수정 후 Native
전 64쪽 미대응 0을 확인했다. 근거는 `post-rebase-8328/issue7723/`에 보존했다.
2026-10-29 작업지시자는 새 MCP 재출력과 다른 **기존 2024 MCP PDF**를 #7723의
PR 검증 기준으로 선택했다. 선택한 기준의 최종 후보 Native·fresh WASM 전쪽과 기존
별칭 검사를 확인한 뒤 PR의 `Closes #7723` 및 통합 후 #7445 행 갱신 여부를 판정한다.

추가로 새 HWP 2024 MCP Print 0 job `5cc99c9a-f703-4c27-9073-7a2f777eefc8`으로
64쪽 PDF(SHA-256 `4b03e8f1325654b423f4e2975890309c44b70cf3a1b1083879097aff2ad09c6a`)를
산출했다. 새 기준 Native 결과는 64쪽 중 2쪽만 raw 100%, 최저 22쪽 98.298%다.
이전 PDF 기준의 64쪽 100%를 새 PDF의 통과로 쓰지 않는다. 동일 Native PNG는
두 실행에서 64쪽 모두 바이트가 같고 PDF는 글꼴 프로그램 구성이 다르다. 새 PDF와
동일한 글꼴 공급·출력 경로를 확인해 원인을 분석하고 fresh WASM도 검증해야 한다.
새 PDF의 명목 `한양신명조` 글꼴 중 한 프로그램은 독립 서버 글꼴 Gulim
SHA-256 `6ff0f49540efc2a6fd980d95e3d930635635b082dd1c10d31fe81b7971d7fdab`의
87개 비교 글리프와 윤곽이 같고, 휴먼명조 프로그램은 로컬 원본 446개 글리프와 같다.
22쪽 한컴 출력은 기존 PDF보다 줄 끝의 한글 소유도 달라진다. 전역 Hanyang→Gulim
또는 HCI Poppy→Gulim을 페인트/메트릭에 일괄 적용한 선택 4쪽 진단은 100%에
이르지 못하거나 다른 쪽을 악화시켜 채택하지 않았다. 새 PDF 기준 fresh WASM도
64쪽 중 2쪽만 raw 100%, 최저 22쪽 98.31053%다. Native·WASM 양방향 실제 PNG
미대응 증거를 `post-rebase-8328/issue7723/`에 보존했다. 새 PDF 기준 미달이
결과는 선택한 기존 기준과 구분해 기록한다. 새 PDF 기준의 100%를 주장하지 않는다.

2026-10-29 추가 대조: 원 PR #8188은 저장 줄이 없는 22쪽의 끝 자간과 원본
HCI Poppy 폭을 함께 반영했다. 현재 폭 구현은 원 PR 커밋과 동일하며, 원 PR의 HWPX
기준 PDF와 #7723 기존 PDF는 `응용프` 뒤에서 줄을 나눈다. 새 MCP PDF는 `응용프로`
뒤에서 나눈다. 별도 진단에서 물리 구간의 4 HWPUNIT 초과를 허용하면 새 PDF의
22쪽은 98.24607%에서 99.90314%로 오르지만 기존 PDF는 100%에서 98.26082%로
떨어져 채택하지 않았다. 1·4·22·38쪽 비교, 동일 래스터 해시와 조건은
[`page22-font-line-diagnostic.json`](assets/review_integration_20261026/validation/post-rebase-8328/issue7723/page22-font-line-diagnostic.json)에 기록했다.
원 PR의 동일 HWPX를 현재 MCP에서 한컴 2024 Print로 재출력해 22쪽만 추가 대조했다.
기존 HWPX PDF와 기존 #7723 PDF의 22쪽 래스터는 서로 같고, 새 HWPX와 새 #7723의
22쪽 래스터도 서로 같다. 기존과 새 출력의 줄 구성·글꼴 자원은 달라 입력 형식만의
차이로 설명되지 않는다. 같은 버전·출력 방식과 서버의 원본 Poppy 파일 존재는 확인했지만
실제 글꼴 선택이 바뀐 원인은 아직 확정하지 않았다. HWPX 전쪽 시각 재검사는 하지 않았다.
작업지시자는 이후 변환에 기존 HWP 2024 MCP만 사용하도록 지정했다. 별도 호스트의 이전 MCP나
다른 변환기를 사용하지 않는다. 새 시스템 글꼴을 사용한 #7723 Native 1·4·22·38쪽은
각각 99.28159%, 99.38157%, 98.29509%, 99.14395%이며 모두 100% 미만이다. 같은
PDF 래스터의 이전 명시적 글꼴 공급 결과와 현재 Fontconfig 조회는
[시스템 글꼴 진단](assets/review_integration_20261026/validation/post-rebase-8328/issue7723/installed-system-font-spot-check.json)에
구분해 기록한다. 해당 진단을 현재 시스템 글꼴의 전쪽·WASM 검증으로 사용하지 않는다.

기존 전체 회귀는 보정 전 두 실패로 종료했으므로 수정 후 전체 회귀를 새로
실행한다. 공유 `target/pr-review`의 Cargo/WASM 빌드와 겹치지 않게 순차 실행하고,
#7723 별칭의 내용·쪽 경계 검사 및 최종 종료 코드를 확인한다.

sample16 세 문서의 #8328 리베이스 후 fresh WASM 전쪽 검증을 완료했다.
HWPX·2024 HWP·2022 HWP는 각각 64쪽 TSV 모두 raw 100%이며 Native와 WASM의
총 384쪽 실제 PNG에서 양방향 반경 2px 미대응 픽셀은 모두 0이다. 새 WASM의
SHA-256은 `3faf7e1bca135d57bf70049d3ee63290c4892cea5be30c2ca6fdf2f772faa01a`다.
[WASM 전쪽 TSV와 통합 픽셀 계약](assets/review_integration_20261026/validation/post-rebase-8328/sample16-fixed-wasm/combined-pixel-contract.json)에
각 페이지의 이미지 해시와 미대응 수를 보존한다. 이는 위 #7723의 다른 새 MCP PDF
기준 미달을 해결했다는 뜻은 아니다. 전체 Rust와 나머지 WASM 검사는 계속 진행한다.

보정 뒤 전체 Rust `nextest`는 11,898개 중 11,897개 통과, 한 개 실패, 45개
skip으로 종료했다. 실패한 `source_owned_float_fragment_frames` 검사는 HWPX 표 뒤
문단의 독립 Print 좌표를 보호한다. 36개 입력을 별도 대조한 결과 HWPX 8개에서
뒤 문단이 약 3.72px 위에 있고 HWP는 일치했다. 표의 `flowWithText` 양쪽 값에서
실패하므로 이를 구분 조건으로 쓰지 않는다. 실패한 HWP 유래 HWPX는 표 호스트와
후속 문단에 저장 줄이 없고, #5128 사양서의 줄 없는 제목 앞 표 호스트는 저장 줄이
있다. 이 원본 구조를 반영하는 보정을 준비했으며 해당 36개 사례와 #5128 왕복,
#8149 및 저장 행 경계 검사를 함께 실행한다. 이 보정은 아직 검증 완료가 아니다.
보정 후 기존 36개 사례의 좌표·소유 검사, #5128 사양서 왕복, #8149 및 저장 행
경계 검사 총 22개가 모두 통과했다. 서식 검사도 통과했다. 이는 기존 100% TSV의
새 실행이나 전체 회귀 완료를 의미하지 않으며 이어서 검증한다.
기존 회귀의 36개 입력·독립 PDF와 Native·WASM TSV는 각각 provenance에 기록된
해시와 일치하며 양쪽 64쪽의 이전 raw 값은 모두 100%다. 원래 1,314개 글꼴도
해시·순서를 대조해 준비했다. 원래 경로에서 누락된 굴림은 사용자 지정
`rhwp-local-verification/issue8160`의 동일 해시 원본을 찾아 시스템에 등록했다.
이는 보정 후 전쪽이 100%라는 뜻은 아니며 새 실행 결과를 확인해야 한다.

표 뒤 문단 보정 `81f2428cf8`의 네이티브 출력으로 기존 36개 입력의 독립 Print
대조를 다시 실행해 총 64쪽 모두 raw 100%를 확인했다. sample16 HWPX·2024 HWP·
2022 HWP도 각각 Native와 fresh WASM의 64쪽 모두 raw 100%이며, 여섯 실행의
실제 PNG 384쪽을 독립 Print와 양방향 반경 2px로 재비교해 미대응 0을 확인했다.
정식 TSV는 `post-rebase-8328/sample16-floatfix-native/`와
`post-rebase-8328/sample16-floatfix-wasm/`에, 페이지별 PNG 해시와 픽셀 결과는
[픽셀 계약](assets/review_integration_20261026/validation/post-rebase-8328/sample16-floatfix-wasm/combined-pixel-contract.json)에
기록한다. Native 실행 파일 SHA-256은
`2a4915dfda39a74efbc903297d51e85a3d71634127ec01c47f65124624beedb0`, WASM은
`05bab1b50e9a17b380582e562e992c7189fc3d2e9c85a01187ef069c24ff56e7`이다.
이후 추가한 회귀 지침 커밋은 문서만 변경했다. 표 이어짐 WASM, 나머지 입력 비교,
전체 Rust 회귀와 #7723의 새 MCP 기준 100% 검증은 아직 완료되지 않았다.

같은 보정 후 #8328 관련 Native 55개 입력·329쪽을 보정 직전과 비교한 결과
0.1%p 이상 하락과 실행 오류는 없다. 기존부터 90% 미만인 여섯 쪽은 그대로
미달로 표시한다. 기존 통합 범위 WASM 67개 입력·956쪽도 보정 직전 대비
0.1%p 이상 하락은 없지만, 과거 TSV 대비 12쪽의 차이는 남아 있다. 따라서
보정 직전 대비 유지와 과거 기준 재현을 구분하며 후자를 통과로 기록하지 않는다.
[Native 비교](assets/review_integration_20261026/validation/post-rebase-8328/floatfix-native-matrix.json),
[기존 통합 WASM 비교](assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-wasm-comparison.json).

기존 통합 범위 Native 67개 입력·956쪽은 보정 직전 대비 가정통신문 2쪽 한 곳이
93.09501%에서 92.81548%로 0.27953%p 하락해 미통과다. 같은 입력·PDF·현재
시스템 폰트에서 보정 전 실행 파일을 다시 돌리면 보정 후와 두 쪽 PNG 해시 및
수치가 모두 같다. 새로 추가된 두 폰트 경로를 격리해도 보정 전 과거 수치는
복원되지 않았다. 시스템 폰트 선택 환경의 변화 가능성을 조사하되 특정 원인으로
단정하지 않으며, 0.1%p 기준 통과로 처리하지 않는다.
[Native 비교](assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-native-comparison.json),
[폰트 환경 진단](assets/review_integration_20261026/validation/post-rebase-8328/newsletter-font-environment-diagnostic.json).

시험 HWPX `samples/hwpx/exam_kor.hwpx`의 원본 용지는 A3인데 기존
`pdf/hwpx/exam_kor-hwpx-2020.pdf`는 A4로 출력돼 페이지별 비교 기준으로 맞지 않았다.
사용자 지시에 따라 해당 PDF를 한컴 2020 MCP의 원본 용지 Print 1-up으로 재출력해
덮어썼다. 새 PDF는 A3·20쪽이며 입력·이전 PDF·새 PDF의 해시와 job 정보는
[MCP 증거](assets/review_integration_20261026/validation/post-rebase-8328/exam-hwpx-mcp-print.json)에
기록한다. A4 PDF를 기준으로 측정한 낮은 수치는 유효한 비교 결과로 사용하지 않는다.
시험지 원본 55개를 추가 점검해 모든 구역이 A3인 12개와 대응 PDF의 크기를 확인했다.
국어 HWP에 대응하는 `samples/2020-exam_kor.pdf`도 A4여서 같은 절차로 A3·20쪽
MCP Print로 교체했다. [용지 점검](assets/review_integration_20261026/validation/post-rebase-8328/exam-paper-audit.json),
[HWP MCP 증거](assets/review_integration_20261026/validation/post-rebase-8328/exam-hwp-mcp-print.json).
별도의 `samples/2010-exam_kor.pdf`는 원래 A4·31쪽인 한컴 2010의 역사 자료였다.
현 MCP는 2010을 지원하지 않지만 작업지시자가 2020 출력으로 덮어쓰라고 명시하여 같은
2020 MCP 출력으로 교체했다. 파일명과 달리 현재 내용은 한컴 2020의 A3·20쪽이며,
이전과 현재 해시 및 명시적 지시는 위 MCP 증거에 기록한다.

보정 `81f2428cf8`의 전체 `cargo nextest`는 종료 코드 0, 11,898개 통과,
45개 skip, 52개 slow로 완료됐다. 기존에 실패한
`source_owned_float_fragment_frames::completed_pages_keep_independent_print_grid_and_body_origins_within_two_pixels`도
통과했다. 이 실행은 이후 작업지시로 추가한 고정 좌표 assertion을 컴파일하기 전의
결과다. 새 assertion은 별도 대상 검사로 확인한다. 같은 보정의 기존 표 이어짐
36개 입력은 Native·WASM 각각 64쪽 모두 raw 100%다.
[전쪽 기록](assets/review_integration_20261026/validation/post-rebase-8328/float-fixture-raw100.json).

### 2026-10-28 회귀 근거·개별 PR 검토 후속

- 사용자 지시에 따라 전체 회귀는 재실행하지 않는다. 98% 이상 검사 대상의 독립 Print 좌표를 보강하고, 98% 미만은 관계 검사로 유지한다. 다른 원본이나 편집 전 결과를 편집본의 측정값으로 바꾸어 기재하지 않는다.
- 독립 Print 좌표 보강 대상 검사 25개와 #8168 검사 2개가 정상 통과했다. 명령·로그 해시는 [기록](assets/review_integration_20261026/validation/post-rebase-8328/targeted-coordinate-tests.json)에 있다.
- 최소 시스템 글꼴로 제한한 Skia 검사는 4개 통과했다. 테스트에서 직접 지정한 저장소 글꼴은 별도이며 [구성과 해시](assets/review_integration_20261026/validation/post-rebase-8328/skia-minimal-font-test.json)를 기록했다.
- 한글 원본 보고서 PDF와 합성 PDF를 대상으로 한 fidelity 출력 측정은 [원본별·전쪽 결과](assets/review_integration_20261026/validation/post-rebase-8328/pdf-gap-visual-evidence.json)에 있다. 이 결과는 편집 후 문서의 측정값이 아니다.
- `newsletter-korean` 2쪽 Native는 기존 비교보다 0.27953%p 하락해 사용자 기준 0.1%p를 초과한다. 같은 현재 글꼴 환경에서 구·신 바이너리가 동일 출력이라는 진단은 기존 기준 회복이나 원인 확정을 뜻하지 않는다. 이 항목을 통과로 처리하지 않는다.
- 지정된 26개 파일 중 `hwpx_precompressed_entry.rs`는 문서가 아닌 합성 ZIP 바이트 비교이므로 독립 문서 PDF의 수치를 대입하지 않는다. `issue_8218_hwp3_link_bindata_order.rs`는 사용자 지정 sample4의 실제 측정 근거로 갱신했다.
- 첫머리 주석 추가와 98% 이상 전환은 별도 완료 조건이다. 캐시·그림 교체·PDF 편집처럼 테스트가 입력을 수정하는 경우 현재 원본 수치는 수정본의 독립 Print가 아니므로, 해당 편집본의 측정과 고정 좌표 전환을 계속 확인한다. 주석이 있다는 이유만으로 26개 파일의 검사 유형 전환을 완료로 판정하지 않는다.
- #8328 보존 비교의 fresh WASM 원장에는 이전에도 90% 미만이었던 6쪽이 기록돼 있다. `status: pass`는 0.1%p 하락 비교에 대한 판정이며 이 6쪽의 절대 90% 기준 통과를 뜻하지 않는다. 절대 기준과 예외 여부는 별도로 해결한다. [원장](assets/review_integration_20261026/validation/post-rebase-8328/floatfix-wasm-matrix.json).
- #8319 보존 대상 197개 입력·256쪽은 Native와 fresh WASM 모두 직전 기준 대비 0.1%p 이상 하락이 없고 90% 미만도 없었다. [Native](assets/review_integration_20261026/validation/post-rebase-8328/incoming-floatfix-native-comparison.json) / [WASM](assets/review_integration_20261026/validation/post-rebase-8328/incoming-floatfix-wasm-comparison.json).
- 사용자가 #8219의 원본을 내장 그림 문서 `samples/hwp3-sample4.hwp`로 변경했다. 기존 sample10 검증은 더 이상 #8219의 수용 근거가 아니다. 별도 요청으로 구현한 Visual Sweep 연결 그림 제외 기능은 명시적 해시·영역 증거가 있을 때만 사용하며, 제외 영역과 겹친 내용은 검증하지 않는다.
- #8219 sample4는 36쪽이며 새 MCP 2020 Print를 `pdf/hwp3-sample4-2020.pdf`에 덮어썼다. 21쪽 Native·fresh WASM 각 100.00000%, 1쪽 및 전쪽 최저는 Native 64.32418% / WASM 64.81033%다. 고정 base와 후보의 Native 및 WASM PNG는 36쪽 모두 동일하며 0.1%p 이상 하락이 없다. 1쪽의 기존 90% 미달은 그대로다. 원본의 내장 그림 및 메모리에서 만든 별도 연결 그림 입력의 검사 4개가 통과했다. [입력·PDF·TSV 및 검사 근거](../pr/assets/issue8219/sample4/evidence.json).
- 작업지시자는 sample4 1쪽의 글꼴 제약에 대해 추가 보정 없이 진행하도록 했다. 동글 Regular를 `중간공한`에 명시적으로 치환한 Native 실험은 41.54053%로 기존 64.32418%보다 낮아 적용하지 않았다. 1쪽은 실제로 90% 미달이며 이 지시를 점수 통과나 모든 차이의 원인 확정으로 기록하지 않는다. 다른 입력의 미해결 기준은 별도로 확인한다.
- #8213 중첩 셀 편집 검사와 같은 편집을 저장·재열기한 별도 입력은 한컴 2020 MCP Print 18쪽과 비교 시 최저 Native 26.59257% / fresh WASM 26.64294%다. 2~4쪽도 98% 미만이므로 관계 검사를 유지한다. 편집본의 결과를 편집 전 문서나 메모리 내 캐시 출력의 직접 측정으로 혼동하지 않는다. [측정·전쪽 TSV](../pr/assets/pr8213-nested-cache-edit/evidence.json).
- #8227의 PDF 편집 가능 모드에서 기존 측정이 98% 이상인 2010 PDF 7쪽의 독립 PDF 글줄 x·y를 ±2px로 검사한다. 편집 전 좌표 검사와 편집 후 관계 검사를 구분하며 해당 파일 검사 4개가 통과했다. [실행 기록](assets/review_integration_20261026/validation/post-rebase-8328/pdf-editable-coordinate-test.json).
- #8227 구현은 반영됐으며 편집 가능 가져오기·원본 저장 왕복·메모리 편집 관계 검사는 통과했다. 별도 추가 실험으로 같은 편집을 HWP로 저장·재열기한 입력은 한컴 2020 Print와 비교해 4쪽 Native 65.17416% / fresh WASM 64.97946%, 전16쪽 최저 1.58239% / 1.63218%다. 5쪽에는 글 내용·쪽 소속 차이가 있다. 같은 저장 HWP·PDF·현재 환경에서 새 base와 후보의 Native 4~6쪽 PNG는 바이트 단위로 같으며 다른 쪽이나 WASM의 동등성을 주장하지 않는다. 이 추가 실험을 원 PR의 편집 전 시각 검증이나 메모리 편집 검사의 미해결로 잘못 판정하지 않는다. [독립 Print·측정](../pr/assets/pr8227-editable-after-save/evidence.json).
- 가정통신문 2쪽에서 임시 Fontconfig 설정으로 설치된 맑은 고딕 Bold 한 파일만 제외하자 기존 rhwp PNG가 해시까지 동일하게 재현됐다. 다른 전역 설정과 시스템 폰트는 변경하지 않았다. PDFium이 출력하는 기준 PDF PNG도 이 설정의 영향을 받아, 과거 PDF PNG를 고정하여 비교하면 과거 93.09501%, 현재 글꼴 출력 92.81548%다. 동일 조건 구·신 코드의 결과가 같다는 사실과 과거 점수 대비 0.27953%p 하락을 구분하며, 후자는 통과로 처리하지 않는다. [글꼴 격리 기록](assets/review_integration_20261026/validation/post-rebase-8328/newsletter-font-environment-diagnostic.json).
- 후속 대조에서 같은 글꼴 격리를 이전 원본 실행 파일과 최종 후보에 모두 적용하자 두 쪽 렌더 PNG가 과거 PNG와 각각 동일했다. 과거 PDF PNG를 고정한 직접 재계산은 1쪽 97.56961%, 2쪽 93.09501%로 과거 수치와 같으며 동일 조건에서 0.1%p 이상 하락이 없다. 현재 전체 시스템 글꼴에서의 92.81548% 및 새 PDF 래스터 점수와는 별도다. 과거 실행 파일 해시가 다른 대조 바이너리의 해시로 잘못 기록된 것도 원래 manifest에 맞게 정정했다. [두 쪽 대조](assets/review_integration_20261026/validation/post-rebase-8328/newsletter-historical-font-recovery.json).
- #8049의 82쪽 미달은 본문 제목이 이전 81쪽으로 이동하고 표가 위로 당겨지는 물리 배치 차이를 포함한다. 제목은 원본 0구역 1023 문단이며 저장 첫 줄 vpos=0, 다음 표 문단은 vpos=4220이다. 제목 문단은 `keep_with_next`와 `page_break_before`가 꺼져 있다. 이 저장 위치를 새 쪽의 단서로 검토하되 하나의 사례만으로 일반 조건이나 해결을 주장하지 않는다. [원본·독립 PDF 및 기록](assets/review_integration_20261026/validation/post-rebase-8328/claim8049-title-diagnostic.json).
- 작업지시자 확인: #8049 82쪽은 리베이스 전 `upstream/devel`의 [원본 WASM TSV](../pr/assets/pr8319_evidence_alignment_20261027/source-claims/pr8049/wasm/silhouette.tsv)에서도 26.63057%였으며 이번 PR의 보정·검토 범위에서 제외한다. 시험 코드 변경은 반영하지 않았다. 새 base 대비 기존 기능이 0.1%p 이상 후퇴하지 않는지의 리베이스 보존 검증은 별도로 유지한다.
- #8319에서 가져온 WASM 전쪽 비교의 기존 90% 미만 6쪽은 upstream이 커밋한 각 원본 TSV에서도 모두 90% 미만이다. 새 base와 후보는 같은 현재 측정 조건에서 6쪽 모두 수치가 같았다. 과거 측정 환경의 수치를 현재 실행과 동일시하지 않는다. 특히 #8049 10쪽은 과거 87.94137%, 현재 새 base·후보 각각 86.53418%로 당시 측정 대비 차이가 있어 그 차이를 숨기지 않는다. 사용자의 명시적 범위 제외는 #8049 82쪽에만 기록한다. [원본 TSV 해시·수치 대조](assets/review_integration_20261026/validation/post-rebase-8328/upstream-existing-low-pages-audit.json).
- #8217의 `hwpx_precompressed_entry.rs`는 ZIP 엔트리의 압축 바이트 동등성을 보호하는 별도 합성 검사다. 시각 근거가 없다는 이유로 HWPX 그림 저장 검사와 병합하려 했으나, 작업지시자의 지적을 받은 뒤 단순 중복이라는 근거가 부족하다고 판단해 원래 파일과 검사를 복원했다. 문서 쪽·Print 측정값은 이 합성 검사에 적용하지 않으며 임의 수치를 기재하지 않는다. suite 수집 경로도 다시 확인했다.
- #8217 두 검사 파일은 복원 후 현재 suite에서 4개 모두 통과했다. [검사 실행](assets/review_integration_20261026/validation/post-rebase-8328/pr8217-restored-tests.json).
- suite 생성 전 점검에서 #2225가 `suite-policy.json`에 그룹 통합 허용과 독립 target 양쪽으로 남아 target 수 49/48이 됐다. 기존 `rust-test-suite-manifest.test.mjs`는 #2225의 그룹 실행을 명시하고 실제 현재 Cargo 구성도 그룹만 사용한다. 오래된 독립 target 정책을 제거해 실제 수집 계약과 생성기를 일치시킨 뒤, 정책 검사와 native-skia 최소 글꼴 대상 검사를 확인한다. 생성 실패가 기존 대상의 누락으로 이어지지 않도록 기존 generated suite는 보존했다.
- 2026-10-28 원 PR 20건을 재조회한 결과 head가 모두 바뀌었다. GitHub API의 head와 원 저장소 `refs/pull` 일부가 달라 기여자 저장소에서 API가 지정한 head SHA를 직접 가져왔다. 각 PR의 과거·현재 merge base 이후 커밋을 stable patch-id로 비교한 결과 #8227·#8217 등은 동일하고, #8184에는 기존 24개 외 3개 보정 커밋이 있다. 그중 호출 호환 수정은 현재 코드에 있으나 나머지 두 커밋은 별도 물리 간격·첫 조각 예약 조건을 바꾼다. 이미 통합 브랜치에서 수행한 독립 Print 회귀 보정과 수용 범위를 대조해야 하며, 이 항목은 미검증이다. 다른 patch-id 차이도 주변 devel 변경과 실제 의미를 각각 확인한 뒤 각 PR 리뷰를 갱신한다. [현재 head·리뷰어](assets/review_integration_20261026/validation/post-rebase-8328/source-pr-metadata-refresh-20261028.json) / [커밋별 비교](assets/review_integration_20261026/validation/post-rebase-8328/source-pr-patch-refresh-20261028.json).
- #8184 추가 보정의 대표 사례를 기존 후보에서 확인했다. `hcar-001.hwpx` 1쪽 Native는 74.87266%이며 표 윗선은 독립 PDF 152.47px, 후보 147.9px이다. 문제 표 HWPX 11쪽은 95.74925%, 연구보고서 HWPX 174·175쪽은 각각 99.99397%·99.23434%다. 추가 보정은 현재 브랜치의 소유 규칙과 충돌 여부를 먼저 대조하고, 구현할 경우 이 입력과 기존 36개 이어짐 사례·사양서 43쪽의 전후 Native/fresh WASM을 검증한다. [hcar 입력·좌표](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-new-hcar-native-diagnostic.json).

### 2026-10-28 체리픽 순서 정리와 후속 검증

- `58063983b6`, `6506bd2aee`를 기존 #8149 묶음 뒤로 이동했다. 대화형 리베이스는 충돌 없이 끝났고 이전·이후 전체 트리는 `c9722a0cc2027a69c51be9f71fcede1bca1020d9`로 같다. 이후에는 문서와 검증 증거만 커밋했다. 예전 SHA와 현재 SHA는 [대응 원장](assets/review_integration_20261026/cherry_pick_current_ledger_20261028.json)에 기록했다.
- #8184 후속 체리픽 이후 sample16 세 원본의 Native와 fresh WASM 각 64쪽을 검증했다. 기존의 같은 PDF 래스터를 사용하면 전쪽 raw 100%다. 현재 방식으로 다시 만든 PDF 래스터를 사용한 측정도 별도로 기록했으며 일부 99.98%다. [원본·PDF 및 쪽별 래스터 해시와 TSV](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-after-sample16/evidence.json). 기존·현재 PDF 바이트 해시가 같은 것은 확인했지만 두 래스터의 수치를 같은 결과로 주장하지 않는다.
- 36개 표 조각 입력은 Native·fresh WASM 각각 전 64쪽 raw 100%다. [입력·PDF 및 쪽별 결과](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-float-fragments/evidence.json). 공식 사양서 전 71쪽은 기존 PDF 래스터 기준 두 경로 모두 보정 전과 수치가 같다. [전쪽 비교](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-spec-all/evidence.json). #7723의 별도 새 MCP PDF 기준 100% 미달과 해당 PR이 제시한 모든 쪽의 최신 근거 확인은 남아 있다.
- 원 PR 20건의 head는 최근 재조회 시 이전 기록과 같았다. 리베이스 후 체리픽 기록과 추가 2건의 현재 SHA, 대표 원 PR의 패치 비교, 검토 문서 파일 링크 검사를 별도 보존했다. 이 대조는 최종 시각 검증과 사용자 승인 대신이 아니다.

### 2026-10-29 #7723 새 인쇄 차이 추적

- #7723 새 MCP 재출력은 이전 새 MCP 출력과 전 64쪽의 래스터 해시가 같았다. 22쪽에서 차이가 큰 문단 0.446은 원시 레코드에 저장 줄이 없고, 기존 PDF의 영문 실행은 Type3 글꼴, 새 PDF는 Type0 글꼴 자원을 쓴다. 새 PDF와 기존 PDF는 해당 문단 줄 끝도 다르다. HCI Poppy를 Gulim 메트릭으로 일괄 치환하면 다른 줄이 틀어져 채택하지 않았다. 이는 원인 계층을 좁힌 진단이며 100% 해결이 아니다. [재출력 전쪽 비교](assets/review_integration_20261026/validation/post-rebase-8328/issue7723/mcp-repeat-all-page-check.json), [원본 레코드·글꼴·줄 진단](assets/review_integration_20261026/validation/post-rebase-8328/issue7723/page22-font-line-diagnostic.json).

### 2026-10-29 기존 통합 입력 후속 보정 대조

- 공식 사양서 71쪽과 별도 기존 입력 66건·885쪽을 합친 67건·956쪽에서 후속 보정 전후 Native와 fresh WASM 렌더 PNG가 각 경로에서 모두 동일했다. 입력·PDF SHA도 맞췄다. 같은 기존 PDF 래스터 기준 전쪽 raw 차이는 0이고, 두 경로 모두 90% 미만이 없다. 새 방식의 PDF 래스터를 쓴 직접 측정값과 동일하다는 뜻은 아니다. [Native](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-original-native-png-comparison.json), [WASM](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-original-wasm-png-comparison.json), [사양서](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-spec-all/evidence.json). 임시 PDF 가져오기 중간본 6건은 파일을 보존하지 않고 생성 출처와 해시를 남겼다. 행정업무편람은 제외했다.

### 2026-10-29 후속 검증 진행

- 후속 표 보정에 직접 연결되는 `issue_8149_empty_host_float_page_bottom`, `source_owned_float_fragment_frames`, `issue_8160_saved_page_boundaries` 세 묶음의 선택 nextest는 종료 코드 0, 24개 통과다. 필터에서 제외된 794개는 실행하지 않았고 전체 회귀도 다시 실행하지 않았다. [명령·검사명·로그 해시](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-after-targeted-nextest.json).
- #8319 보존 대상 197개 입력·256쪽은 후속 보정 전후 Native와 fresh WASM 렌더 PNG가 각 경로에서 모두 동일하다. 같은 이전 PDF 래스터 기준 최저는 Native 93.22230% / WASM 93.61735%이며 90% 미만과 0.1%p 이상 하락이 없다. WASM 집계 헤더의 이전 패키지 SHA는 실제 실행 인자와 개별 manifest 197개를 확인해 정정했다. [Native 전쪽](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-incoming-native-same-raster-comparison.json), [WASM 전쪽·패키지 해시](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-incoming-wasm-same-raster-comparison.json). 새 PDF 래스터 직접 측정과 구분한다.
- #7723 공백 설정 통제 인쇄에서는 변환 HWPX의 charPr 55 `useFontSpace`만 0에서 1로 바꾸면 문제 문단의 첫 줄이 원래 인쇄 결과보다 길어졌다. 원본 속성은 0이므로 임의 변경하지 않는다. [인쇄 대조와 원본 글꼴 진단](assets/review_integration_20261026/validation/post-rebase-8328/issue7723/page22-font-line-diagnostic.json).

- 회귀 근거 정리 시작점 이후 변경된 Rust 검사 26개 파일을 다시 대조했다. 첫 줄의 유형과 합성 ZIP 검사 외 날짜·raw 96dpi·Native/WASM·전쪽 최저 표기, 첫 주석의 정식 근거 29개 경로의 존재와 Git 추적을 확인했다. 이 기계 검사는 수치의 의미나 98% 기준의 적합성을 대신 판정하지 않는다. [파일별 주석·해시 목록](assets/review_integration_20261026/validation/post-rebase-8328/regression-26-header-inventory.json).

## 2026-10-29 중간공한 설치 후 1쪽 확인

이전 중간공한 파일(SHA `adb0346c…`) 설치 뒤 sample4 1쪽만 같은 독립 PDF 래스터로 측정했다. Native 64.32418% → 66.60007%, WASM 64.81033% → 66.44902%다. 새 글꼴 환경에서 후속 표 보정 전후 PNG는 양 경로 모두 같다. WASM은 새 글꼴을 명시 공급해 해당 글꼴 SHA의 실제 로드를 확인했다. 양쪽 모두 90% 미만이며 글꼴 공급 조건이 달라 결과 차이의 유일한 원인을 단정하지 않는다. 사용자 결정에 따라 이 쪽을 추가 보정하지 않고, 다른 35쪽은 이 실험으로 재측정하지 않았다. [1쪽 측정·해시](<../pr/assets/issue8219/sample4/final-konghan-page1.json>).

- 26개 검사 주석 중 원본별 Native·WASM 원장과 같은 파일명 및 명시적 쪽별 수치가 대응되는 52개 쌍을 재대조해 모두 일치함을 확인했다. 다른 형식의 수치와 최신 출력까지 검증한 결과는 아니다. [대조 대상·원장 해시](assets/review_integration_20261026/validation/post-rebase-8328/regression-header-original-score-crosscheck.json).

### 2026-10-29 #8319 보존 대상 Native 완료

- 후속 표 보정 뒤 197개 입력·256쪽의 Native 렌더 PNG가 이전과 모두 동일하며, 입력·PDF SHA도 각각 확인했다. 기존 PDF 래스터로 산출한 전쪽 최저는 93.22230%, 90% 미만과 0.1%p 이상 하락은 없다. 새 PDF 래스터의 직접 측정은 아니며 fresh WASM도 전쪽 동일함을 별도로 확인했다. [전쪽 원장](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-incoming-native-same-raster-comparison.json).

### 2026-10-29 시스템 글꼴 비의존 검증

- Fontconfig 검색을 빈 디렉터리로 제한해 `fc-list` 결과 0개인 상태에서 Native Skia 소수 원점·가는 선 검사 4개가 통과했다. 검사에 필요한 Noto Sans KR은 시스템 설치본이 아니라 저장소에 추적된 파일을 직접 공급했다. 이는 디렉터리 자체를 숨긴 검사나 26개 파일 전체의 결과는 아니다. [실행·글꼴 해시](assets/review_integration_20261026/validation/post-rebase-8328/fontless-skia-precheck.json). 별도로 시스템 글꼴 디렉터리까지 가린 Linux 격리 환경에서 이 브랜치의 26개 파일을 선택 실행해 110개 모두 통과했다. 각 파일에서 적어도 한 검사가 실행됐고 필터로 제외된 4,357개는 이 실행에 포함되지 않았다. [파일별 검사·글꼴 공급과 로그 해시](assets/review_integration_20261026/validation/post-rebase-8328/fontless-branch-regression.json).

- #7723은 새 MCP PDF F1 글꼴을 임시로 재구성하고 동일 입력·기준·원본 폭 보존 설정의 대조군과 비교했다. 22쪽은 Native 98.24609% → 98.28007%, 1·4·22·38쪽 모두 100% 미달이다. 임시 표시 치환만으로 해결되지 않았고 전체 쪽·WASM을 확인하지 않아 적용하지 않았다. 글꼴과 임시 스크립트는 회귀에 추가하지 않는다. [대조 조건과 쪽별 수치](assets/review_integration_20261026/validation/post-rebase-8328/issue7723/page22-font-line-diagnostic.json).

- 시스템 글꼴 비의존 원칙을 AGENTS.md·CLAUDE.md와 회귀 검증 지침에 반영했다. 문서 전용 경로의 CI 영향 분류 45개, fast-pass 관련 검사 25개, 신뢰된 검토 연결 검사 83개가 로컬에서 통과했다. 통합 브랜치 전체가 문서 전용이거나 원격 CI가 끝났다는 뜻은 아니다. [명령·로그 해시](assets/review_integration_20261026/validation/post-rebase-8328/fontless-guidance-ci-checks.json).

- 개별 PR 20건의 review·구현 문서 40개를 최근 증거 링크 반영 후 다시 검사했다. 인식한 상대 파일 링크 863개에 누락·미추적 파일·임시 output 경로가 없다. 외부 링크·앵커 및 증거 내용 적합성을 이 검사로 판정하지 않는다. [문서별 해시와 결과](assets/review_integration_20261026/validation/post-rebase-8328/individual-review-local-links-20261029.json).

- #7723의 기존 PDF와 새 MCP PDF 22쪽을 동일 PDFium 설정으로 재래스터화해 내용 마스크를 직접 비교했다. 4px 사각 이웃에도 대응하지 않는 화소가 기존 203개·새 PDF 177개다. 같은 렌더 PNG가 두 PDF에 각각 2px raw 100%를 동시에 만족할 수 없다는 뜻이며, 새 MCP 기준만의 해결 가능성을 부정하지 않는다. [조건·래스터 해시·대조](assets/review_integration_20261026/validation/post-rebase-8328/issue7723/old-vs-new-pdf-page-comparison.json).
- Poppler의 1쪽 `물` 글자 추출 bbox 상단은 두 PDF에서 약 15.38pt 다르지만 같은 글자 영역의 96dpi 잉크 행은 모두 248~262px다. PDF 텍스트 bbox 상단만으로 실제 잉크 위치나 글줄 이동을 단정하지 않는다. 이는 선택 영역의 진단이며 새 기준의 100% 해결을 뜻하지 않는다. [좌표·래스터 해시](assets/review_integration_20261026/validation/post-rebase-8328/issue7723/poppler-bbox-ink-check.json).

- 2026-10-29 upstream/devel을 다시 fetch한 결과 `db5cd5bcc60a69e42f419f18a7f57a33b6fb7fdc`로 기존 기준과 같았다. 확인 시 로컬 HEAD `b15fa728e853bcca51b60327b2cab5cbb4372382`이며 원격에만 있는 커밋은 0개라 추가 리베이스는 필요하지 않았다. fetch 로그 SHA-256 `5df049c409ba55f303db1c87c1704adc35971902ab9f2be478362df2a03b2373`. 제출 전에는 다시 확인한다.

### 최신 작업 지시와 제출 상태

- 작업지시자가 새로 설치한 글꼴은 내부 검증 전용으로 지정했다. 글꼴 파일을 저장소·패키지·배포물에
  포함하지 않는다. 2026-10-29 현재 base 대비 변경 파일과 변경된 HWPX·ZIP·EPUB 내부에서
  글꼴 확장자 파일은 발견되지 않았다. [검사 범위와 고정 head](assets/review_integration_20261026/validation/post-rebase-8328/font-distribution-audit.json).
  이는 PDF에 포함된 글꼴 자원까지 검사한 결과는 아니다.
- 이후 변환은 기존 MCP의 한컴 2024 경로만 사용한다. 별도 호스트 MCP는 사용하지 않는다.
- 전체 회귀는 다시 실행하지 않는다. 완료한 Rust 정적 검사와 Studio 선택·브라우저 검증은
  [PR 본문 준비안](review_integration_20261026_pr_body_draft.md)의 실행 근거를 따른다.
- 작업지시자는 필수 검증 완료 뒤 PR 생성과 CI 완료 후 머지·후속 처리를 승인했다. CI 변경을
  포함하므로 PR review와 오늘할일은 옵션 2의 별도 후속 기록 PR로 처리한다. PR 생성 전 최신
  base·원 PR·로컬 검증을 다시 확인하고, 머지 전 정확한 head의 필수 CI를 확인한다.
- #7723은 작업지시자가 기존 2024 MCP PDF를 최종 기준으로 선택했다. 해당 PDF의 기존
  래스터 기준으로 최종 Native·fresh WASM 64쪽 모두 raw 100%다. Native는 과거 글꼴
  검색 조건에서 100%를 측정한 PNG와 전쪽 해시가 같으며, 현재 PDF 재래스터 결과의
  세 쪽은 99.98~99.99%로 구분한다. 기존 별칭의 내용·소유 검사도 통과했다.
  [Native](assets/review_integration_20261026/validation/post-rebase-8328/issue7723/selected-reference-latest-native.json),
  [WASM](assets/review_integration_20261026/validation/post-rebase-8328/issue7723/selected-reference-latest-wasm.json),
  [별칭 검사](assets/review_integration_20261026/validation/post-rebase-8328/issue7723/existing-alias-regression-status.json).
  새 PDF는 별도의 미달 진단으로 남긴다. 통합 PR에 `Closes #7723`을 포함하고 머지 후
  #7445의 해당 행을 갱신한다.
