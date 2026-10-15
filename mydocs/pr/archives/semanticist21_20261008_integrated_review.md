---
kind: review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-14
---

# semanticist21 비 draft PR 통합 검토

## 현재 판정

**통합 완료 — 메인터너 보정 후 승인.** 통합 PR [#7882](https://github.com/edwardkim/rhwp/pull/7882)는 `86a3c88fc89de9a2306a10bdda9a31023f0e24a5`로 병합됐으며 `upstream/devel` 포함을 확인했다. code/trailing head의 CI와 최종 정책 status가 성공한 뒤 exact-head merge했다. 개별18개 기록을 보존했고 후속 issue·원 PR 처리와 안전 정리는 아래 통합 merge 기록의 범위로 진행한다.

- 브랜치: `review/semanticist21-20261008`
- 최신 단계(진행240): #7882 merge `86a3c88fc89de9a2306a10bdda9a31023f0e24a5`의 devel 포함 및 최종 정책 감사3 성공 확인. 이전 로컬 단계(진행238): #5877 CI 회귀의 최종 로컬 검증 완료. 생산 `6cbebc7457ea8899c3949e48ed1255d43f40bcb2` / base `22317614b03722e95349a1d3d71c86923363c6f5`. [원문 TSV180개](https://gist.github.com/jangster77/1efd96731b845480bc1b8f38344d6d6e/1d7a7fe25f0960cb43496b28bed1965a68e282cf)를 압축 없이 게시하고 API로 원문 바이트를 재확인했다. 이전191/178/978 결과는 당시 소스의 역사 실행이다.
- 범위 정정(진행129): `table_giant_cell_overfill.hwpx`는 #7445 하위 #7812에서 다른 브랜치의 보정 대상으로 이관됐다. 현재 직접 보정·전쪽 통과 요구에서 제외한다. KPS·가맹사업의 기존 3실패와 승인된 CGMP 메인터너 범위는 계속 확인한다. 이후 사용자 자동 승인에 따라 모든 필수 검증 통과 후 PR·CI·merge·후속처리를 진행한다.
- 통합 범위는 초기 18개다. 2026-10-10 사용자 지시에 따라 후속 #7661/#7662/#7663/#7670/#7671/#7673은 **현재 단계의 읽기 전용 코드 검토만** 수행한다. 이 6개와 #7615 최신 head에 들어간 #7662 의존 변경을 체리픽하지 않는다. 아래 진행78에 정확한 head와 검토 결과를 기록한다.
- 과거 진행158의 통합 base: `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`. 현재 base는 위 최신 단계의552f5228이며 이 문단은 당시 체크포인트 기록이다. 생산/test 소스는 `940ea1322617ccff5a7b47d6920ae96e94cfffad`로 커밋했고 Native v315/fresh v323의1111개 생산 소스 해시와 같다. 진행158의 변경은 잔여 API 쪽수 목록1행과 문서/시각 증적이다. 이후 증적 커밋은 생산 소스 변경으로 세지 않는다. 이전 `10282f63db5f671eca8554613342e20211a025ac` 검증은 진행137의역사 기록이며 현재 소스의 전쪽/전체 회귀 결과로 재사용하지 않는다.
- 직전 일반 커밋 체크포인트는 `a09fc8b968dd7bb67e74fa6ffb185c586bd19d56`이다. 설계 전7쪽 Native/fresh WASM 검증과 관계 회귀를 고정했으며 후속 #5877 보정은 진행100의 v118 이후 기록에 연결한다. 이전 체크포인트는 `004bab229877d6b8f05dff9d98410161de135e09`와 `da70ea0258ab094ed67750a813ea465840f427f4`이며 각 설계 첫 조각/CGMP 단계의 검증 근거를 따른다. 이전 체크포인트 `f56ceec135f28aa6a8eb424bd74551aba0c7831f`는 역사 단계다. 진행89의 표 뒤 쪽 경계·인접 원본 앵커 보정과 실제 검증을 커밋했으며 후속 보정은 진행90에 기록한다. 이전 체크포인트는 `c43b98e9b0bcdb8ddee6a9f049baf703f0babc21`이다. 진행88의 직접 HWPX 원점 경계 전달을 커밋하고 `upstream/devel` 동기화·리베이스를 확인했다. 후속 보정은 진행89에 기록한다. 이전 체크포인트는 `fd4e242c9e4fafb2441f8d743d01746e26d15078`이다. 진행81~87의 저장 표 보정·시험지 원본 보존과 회귀 연결 제거·주석 번역을 커밋했고, `upstream/devel` fetch와 로컬 `devel` 동기화·리베이스를 확인했다. 후속 CGMP 보정과 실제 검증 범위는 진행88에 기록한다. 이전 일반 커밋 `81ff2fe4a362b8585fe24f824282e6db7332c0bb`(보정77~79의 교육과정 이관/clip 진단/중첩 source frame 보정과 현재 시각 증거)이다. 이전 `12d96686cdd24017b44becead2c4dfeb46867bf3`는 #7700 이관 체크포인트다. 현재 단계의 동기화·rebase와 다음 편집 검증은 진행80에 연결한다. 이전 production 코드 체크포인트 `9b53bd699126205b7a93fff326167f575252f1ea`는 저장 표의 수용 행 원장과 기존 회귀 검토 증거 보정이다. 진행70의 v86 retry2 두 Rust 파일을 이 커밋으로 고정했다. 실제 Native/fresh WASM 빌드는 부모 `3874bc4be037e84998d548e7755bc689af9d871b` 위 당시 미커밋 변경에서 수행했고 커밋된 production source와 동일하다. 이후 변경은 #7700 검사·자산 경로의 이관이며 진행74에서 검증한다. `cbaf53f3fa8fc8511ac82e5bd7481cfa56f8d922`와 진행61–62는 v78/retry1의 역사 체크포인트다. `e8ae5204d1b8e61851142f22b229de38f1867816`은 진행59의 rebase 체크포인트다. 이전 `3505cb8cd8d09f3c125fdb08c9ccec6efece78ff`는 진행55–56의 v73 검증·커밋·동기화 체크포인트다. 이전 `f3c8cd7d370668e36322db71c1c2c820260124ae`는 진행54의 v72 커밋·리베이스 체크포인트다. 이전 `fe201fa05f25ee744fb465a8b8a7348d38576d18`의 v64 retry1 원본 표제/첫 조각 보정과 검증은 진행44의 역사 증거다. 이전 `eb29a7a9ce171c2cc5dfd9ff4689eb7a0d64eaae` 및 v61 결과는 역사 증거다.
- rebase 전 contributor 누적 head: `cd0eb2d25380f6db60a4b8bb99ed90f7e1d5c84c` (#7659 포함). 메인터너 보정 v44 source는 `64cc5b5e805e2b05ab3fd64fa5aa4803e0d6357a`이며, 이후 후보와 검증은 아래 단계별 기록을 따른다.
- 최초 17 PR 검증 code head: `134a646e5a5325daf52d614e0395094175e78301` (아래 최초 검증 결과의 역사 head)
- 최초 검토 head: `8a9f87300a47293d644f8b862906c989adf34f84` (새 source 적용 전 역사 결과)
- 최초 contributor 누적 diff(역사 후보):47 source/test 파일, 5421 추가 / 823 삭제. contributor 원 commit은 `cherry-pick -x`로 보존. merge commit 제외, #7615 중복 의존 commit 4개는 재적용하지 않음.
- draft 제외: #7559, #7572, #7573, #7588, #7589, #7598.
- 통합 PR #7882는 `86a3c88fc89de9a2306a10bdda9a31023f0e24a5`로 병합 완료. 후속 문서·issue·원 PR 처리·안전 정리는 아래 최신 기록을 따른다.


## PR별 검토 문서

사용자 지시에 따라 원 PR 번호별 검토 문서18개를 유지한다. 아래 각 문서의 현재 판정·검증 범위가 정식 PR별 기록이고, 이 파일은 누적 보정 이력이다. #7612의 기존 병합 review는 보존하며 나머지17개는 메인터너 보정 후 승인 및 통합 merge 근거를 반영했다.

[#7566](pr_7566_review.md), [#7612](pr_7612_review.md), [#7613](pr_7613_review.md), [#7614](pr_7614_review.md), [#7615](pr_7615_review.md), [#7616](pr_7616_review.md), [#7638](pr_7638_review.md), [#7641](pr_7641_review.md), [#7647](pr_7647_review.md), [#7648](pr_7648_review.md), [#7652](pr_7652_review.md), [#7653](pr_7653_review.md), [#7654](pr_7654_review.md), [#7655](pr_7655_review.md), [#7656](pr_7656_review.md), [#7657](pr_7657_review.md), [#7658](pr_7658_review.md), [#7659](pr_7659_review.md)

## PR별 최초 검토 근거

아래 표는 최초 누적 후보의 판정이다. 이후 메인터너 보정의 개선/남은 차이와 Native/fresh WASM 증거는 아래 source별 기록을 따른다.

| PR | source head | 적용 commit | 현재 근거와 남은 범위 |
| --- | --- | --- | --- |
| [#7566](https://github.com/edwardkim/rhwp/pull/7566) | `abf6f9259ac45ee25e1a2ccd4fd5b7c2073c8dcf` | `b4f958c42` `51aa04372` `0b5d0af07` | Native 셀 주소/활성화 계약 통과. fresh WASM UI 활성화는 미검증. |
| [#7612](https://github.com/edwardkim/rhwp/pull/7612) | `34ceeb0d067549ee0a8689b3b24231bfb1f23428` | `b1d7ccf33` `dbfb72a17` | Native 논리 캐럿·개체 슬롯 계약 통과. 영향 편집 저장본 Print/시각 범위 미검증. |
| [#7613](https://github.com/edwardkim/rhwp/pull/7613) | `eab4fcd8307556c8e8650a35eac8fa881ffce38a` | `0d47bdfba` `64f9f3d82` | 미충족: 두 연속 표의 1쪽 Native/fresh WASM 48.61432% / 38.42346%. |
| [#7614](https://github.com/edwardkim/rhwp/pull/7614) | `2d4148eb8bcb561b2eac2004c7a64741864e7431` | `093ccccb7` `28c97f2a4` | 현재 Native49 묶음의 HTML/문단 끝 번호 계약 통과; 공개 WASM 본문 논리캐럿7상태/왕복28PASS. 현재 편집 출력 Print 미검증; source CodeQL 실패와 통합CI를 구분. |
| [#7615](https://github.com/edwardkim/rhwp/pull/7615) | `ed9c2847523c18f658344ec89cb7ed1a426807ae` | `a2ef9eab3` `761499928` `ebbec90fb` `60385438c` `3addfced3` `6a8f77dab` `2be87053d` | Native 누름틀·정의·슬롯 보존 검사 통과. 편집 저장본 Print 대조 범위 미검증. |
| [#7616](https://github.com/edwardkim/rhwp/pull/7616) | `ca43e499d30e90789d5e662f9cb562252fe95367` | `428a71a9b` `841ecf256` | 미충족: 일반/누름틀/책갈피 HWPX 각 1쪽 Native/fresh WASM 14.16561%. HWP 인쇄 세 건 900초 timeout. |
| [#7638](https://github.com/edwardkim/rhwp/pull/7638) | `aa0c88be40be3ca29aabc95bc724c2ade29c333a` | `c68d0b6dd` `51fbf680c` | 현재 Native 스타일 계약 및 공개 WASM60단계 통과. 같은 파일명문맥20개 Native 전쪽100%/대표·확대 판독 완료. 동일 조건 WASM20개도전쪽100%/Native전체PNG동일. 전체통합gate는별도. |
| [#7641](https://github.com/edwardkim/rhwp/pull/7641) | `9b6f6c51c2e873da0aaf339cb51e9e60be33a0f1` | `d746acf6e` | 미충족: 좁은 셀 1218HU에 중첩 표 점유 7766HU. Native/fresh WASM 모두 재현. |
| [#7647](https://github.com/edwardkim/rhwp/pull/7647) | `e33be129632b4fb9369fcc76c9b5af617aec5be4` | `f78d84aff` `327a4bf2a` | Native 잘못된 21식·정상 45식·오류 시 칸 보존 계약 통과. 공개 WASM 오류/정상 합계 통과. 전체 gate 대기. |
| [#7648](https://github.com/edwardkim/rhwp/pull/7648) | `9703c5fc91b38ca0e4d80dece8d68c9014b006e2` | `5394ca686` `2e33dd6ca` | 현재 Native3계약 및 공개WASM5번호상태×3저장단계15PASS. 실제 같은 입력 번호 Print 대조는 미검증. |
| [#7652](https://github.com/edwardkim/rhwp/pull/7652) | `95e51ad453a7e083e842a9024353919e895affb1` | `6484cd815` `40ebd5be5` | Native 탭 뒤 쪽 번호 자리표·HF 저장 계약 통과. Print/fresh WASM 시각 미검증; source Build & Test 실패. |
| [#7653](https://github.com/edwardkim/rhwp/pull/7653) | `1c5f93a2309437f2b9770c8619c92c5dcc8fb84d` | `b89dfdc30` `e774138aa` | 원 head 단독의 뒤 문단 겹침은 #7656 누적 후 Native/fresh WASM에서 보정 확인. 독립 인쇄 범위 진행 중. |
| [#7654](https://github.com/edwardkim/rhwp/pull/7654) | `db92404bf4427fad3cac2e9a6fa300c6f94d7a65` | `338a825ab` `a5ddd6fa9` | Native/fresh WASM에서 세 문자열의 미주 캐럿 경계 대조 통과. paint 변경 없음(동일 공백/번호 판정 helper 추출). 전체 gate 대기. |
| [#7655](https://github.com/edwardkim/rhwp/pull/7655) | `51d44bffb68c9459cbf4eb470878cf0afb027ad1` | `0b211ca68` `8a9f87300` | Native 무늬 종류/색·전체 Fill 비교 검사 통과. 다섯 편집 상태 Print 성공; Native/fresh WASM 시각 실행 중. |
| [#7656](https://github.com/edwardkim/rhwp/pull/7656) | `268e40eb1daf448bf66541eebf570a69ef8ad6ef` | `f1cc14414` `cf43c3ca5` | 미충족: 다음 문단이 비면 100%, 글자가 있으면 두 경로 86.01714%. 겹침은 없어졌지만 글줄이 아래로 이동. |
| [#7657](https://github.com/edwardkim/rhwp/pull/7657) | `a64ab93cbce0f788e9812673e6add4ce0dd24dce` | `031d7ccee` `b7e559534` | Native 실제 한컴 sample 포함 스타일/원시 바탕쪽 보존 통과. 공개 WASM 셀 스타일/HWP/HWPX 보존 통과. 조판 비해당 범위 근거 추가 확인 필요. |
| [#7658](https://github.com/edwardkim/rhwp/pull/7658) | `8b2f95d6872fba94ff1bb96d40c2bdbe64d6b90d` | `a4b502e32` `134a646e5` | 미충족: 탭 100%, 테두리/배경 82.33902% (두 경로). 녹색 문단 배경 하단 9.37px 차이. |

| [#7659](https://github.com/edwardkim/rhwp/pull/7659) | `db6f70d9c523692080f5318e406a578c11e6c32e` | `47a304211` `cd0eb2d25` | 현재 Native3계약 및 공개WASM7주석style조건×3저장단계21PASS. 현재 독립Print 시각/Studio는 미검증. |

## 실행한 검증

| 명령 / 범위 | 결과 | 증적 |
| --- | --- | --- |
| `git diff --check upstream/devel...HEAD` | PASS | 원 source diff |
| `cargo build --locked --profile release-test --target-dir target/pr-review` | exit 0, 4분 21초 | `logs/native-build-134a646e5.log` |
| `cargo fmt --all -- --check` | exit 0 | `logs/fmt-check-134a646e5.log` |
| `scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt`, `CARGO_TARGET_DIR=target/pr-review` | exit 0, 9분 30초. Docker 없는 환경의 최적화 생략 진단 빌드; optimized 배포 artifact 검증 아님 | `logs/wasm-build-134a646e5.log` |
| Native 원 PR source assertions의 진단 복사/연결 실행 | 최초 86/86 PASS + 자동 번호 4/4 + #7265 대조군 2/2 = 92 source assertions PASS. 정식 nextest 실행 아님 | `logs/fixture-capture-all-134a646e5.log`, `logs/fixture-capture-auto_number_trailing_placeholder-134a646e5.log`, `logs/fixture-capture-issue_7265_hwp5_origin_hwpx_table_outer_top-134a646e5.log` |
| 진단 반례 2개 | 표 여백 뒤 문단 PASS; 좁은 중첩 표 FAIL. ignored 진단에만 존재하고 tests/cases에 신규 추가하지 않음 | Native 위 로그, `wasm-public-contracts-134a646e5.json` |
| 공개 WASM API 6개 범위 | 5 PASS / 1 FAIL(중첩 표). 셀 스타일, 탭, 오류 계산식의 대상 보존, 미주 캐럿, 여백 변경의 다음 문단 포함 관계 확인 | `logs/wasm-public-contracts-134a646e5.log` |
| manifest `--check --base-ref 60efc4b7...` | PASS: 1503 sources / 6477 static attrs / 48 integration targets | `logs/manifest-check-134a646e5.log` |
| unit-tier `--check --base-ref 60efc4b7...` | PASS: 4196 tests / 298 modules | `logs/unit-tier-check-134a646e5.log` |

Native/WASM 산출물 SHA-256은 `build-provenance-134a646e5.json`에 고정했다. WASM SHA-256은 `e097f1b2ed1328aefee359a7b32dfb680f12ceb9f34983069807c143e5698cb4`; pkg/public의 JS와 WASM 각각 해시 일치. Studio 브라우저 사용자 여정은 미실행이다.

세 Clippy, workspace build, 전체 nextest, Skia 추가 gate는 아직 실행하지 않았다. 작은 경계와 직접 시각 판독에서 실제 차이를 먼저 확인하는 순서를 따랐다. 현재 source head의 CI 성공을 통합 head CI로 재사용하지 않는다.

## 한컴 Print와 출력 대조

같은 저장본을 실제 한컴 2020 `11.0.0.9136` / `PrintMethod=0` / `hancom2020_pdf_driver_one_up`로 출력했다. 입력이 공개 API 또는 수동 모델 변형을 거쳐 생성된 경우 실제 한컴 원본 생성본과 구분한다. 5000HU column-top 사례는 source test의 모델 변형이고, 공개 WASM 여백 사례는 `setTableProperties`로 편집했다.

두 공급 font 디렉터리는 실제 존재하며 HCR Batang을 공급한다: `/home/tsjang/.cache/hnc/fonts`, `/home/tsjang/.cache/rhwp-review-hancom-fonts-20260811`. Sweep은 96dpi print profile / embedded fonts / 2px 이웃 관용 내용 실루엣으로 Native와 fresh WASM을 각각 실행했다. WASM은 실제 Chrome export helper를 사용했다.

| 입력 (전부 1쪽) | Native | fresh WASM | 직접 판독 / 판정 |
| --- | --- | --- | --- |
| `cell_background_pattern_change-0001-dd74ebaaa9ec` | 100.00000% | 100.00000% | 해당 review PNG / gate 확인 |
| `cell_background_pattern_change-0003-62aaf8b57f83` | 100.00000% | 100.00000% | 해당 review PNG / gate 확인 |
| `cell_background_pattern_change-0005-4ace892a7d1b` | 100.00000% | 100.00000% | 해당 review PNG / gate 확인 |
| `cell_background_pattern_change-0007-fd96b4836e2c` | 100.00000% | 100.00000% | 해당 review PNG / gate 확인 |
| `cell_background_pattern_change-0009-07084746450e` | 100.00000% | 100.00000% | 해당 review PNG / gate 확인 |
| `column_top_table_reserves_outer_margins-0015-0aad3cf3a347` | 100.00000% | 100.00000% | 해당 review PNG / gate 확인 |
| `column_top_table_reserves_outer_margins-0021-80d658ee147e` | 86.01714% | 86.01714% | 해당 review PNG / gate 확인 |
| `created_table_after_table_starts_below_it-0010-e93fb91c8b2c` | 48.61432% | 48.61432% | 해당 review PNG / gate 확인 |
| `created_table_after_table_starts_below_it-0011-bb7be43f0149` | 38.42346% | 38.42346% | 해당 review PNG / gate 확인 |
| `inline_table_after_field_or_bookmark-0244-1792e69cfaf2` | 14.16561% | 14.16561% | 해당 review PNG / gate 확인 |
| `inline_table_after_field_or_bookmark-0250-4536c5dce2d6` | 14.16561% | 14.16561% | 해당 review PNG / gate 확인 |
| `inline_table_after_field_or_bookmark-0257-dbaa6d12fde5` | 14.16561% | 14.16561% | 해당 review PNG / gate 확인 |
| `updated_style_keeps_tabs_and_borders-0545-e0a6bc5b14ba` | 100.00000% | 100.00000% | 해당 review PNG / gate 확인 |
| `updated_style_keeps_tabs_and_borders-0547-e1b58ace1b99` | 82.33902% | 82.33902% | 해당 review PNG / gate 확인 |

직접 판독: #7613의 두 번째 표 시작점 차이 지속. #7616은 일반·누름틀·책갈피 3개 HWPX의 앞 글자 간격과 표/뒤 글자 위치 차이. #7656은 표 괘선이 같은데 비어 있지 않은 뒤 문단이 아래로 이동. #7658은 문단 배경 하단이 Native 150.9333px / Print 160.3040px이며 색상 배경과 테두리 높이 차이. 동일 Native/fresh WASM 대표 review를 각각 직접 읽었다. 유입 회귀인지에 대한 변경 전 대조는 미완료이며, 현재 출력 gate의 실패와 구분한다.

기준 PDF와 입력 hash/job id는 아래 표에 연결한다. 변환 timeout은 출력 결함의 원인 입증으로 세지 않는다. inline HWP 3건은 최종 failed/900초이며 Print 대조 미검증으로 유지한다.

| 입력 | SHA-256 | Print PDF SHA-256 | job / 결과 |
| --- | --- | --- | --- |
| `output/pr-review/semanticist21-20261008/captured-fixtures-134a646e5/cell_background_pattern_change--changing_only_the_cell_pattern_changes_the_cell_background/0001.hwpx` | `dd74ebaaa9ecaefb5d754b434051feff168316c0afbf4aeadb944cd31b4ac21f` | `1f8b5fee1ae8fea30a9f66eeeff86c55370e0f0f27df66c38b4ac85608aa8bed` | `557b0a9f-e379-4161-b3ab-c181ce362ee8` / succeeded |
| `output/pr-review/semanticist21-20261008/captured-fixtures-134a646e5/cell_background_pattern_change--changing_only_the_cell_pattern_changes_the_cell_background/0003.hwpx` | `62aaf8b57f83516a59505515d6082cbf4511259970c6410def814d25e894033a` | `4b0d8c8d84db3315abe7f373957ab25c2cbd960398eeb13a09f2f2b6efdbf7ee` | `a399775d-0f15-4555-aa6c-8ac82691c033` / succeeded |
| `output/pr-review/semanticist21-20261008/captured-fixtures-134a646e5/cell_background_pattern_change--changing_only_the_cell_pattern_changes_the_cell_background/0005.hwpx` | `4ace892a7d1b80ffbcd9d22e79e3e1ab63e8b656f3085ec2210b8e2c591c371e` | `05e68ea6c5f46d56e1b71358829c050bb35e7cd137288ecf156691a2ed61df5f` | `8bd203c1-2c8c-4f09-be70-24fa3fd516fe` / succeeded |
| `output/pr-review/semanticist21-20261008/captured-fixtures-134a646e5/cell_background_pattern_change--changing_only_the_cell_pattern_changes_the_cell_background/0007.hwpx` | `fd96b4836e2c49ec6b1e748d55f887c48f81ad6b93a3f87626cd756049278fdc` | `53eb9dc719a2bb90a066fa4f57ff76fd1d72092cd8e90d2a6a5384fc8c997fa7` | `4c9d8033-6291-4fbe-8ba8-8b8b6f9fd5ec` / succeeded |
| `output/pr-review/semanticist21-20261008/captured-fixtures-134a646e5/cell_background_pattern_change--changing_only_the_cell_pattern_changes_the_cell_background/0009.hwpx` | `07084746450efe8e2e96a15ef35ea0ca6920b96ca6ae948915a1afa4dd901833` | `d899871d6f09d9d58a08936ee4a6429934e01e3366c88155768ffd3499f54e43` | `d43c3e07-8d0f-4825-b518-6a83be8d4e19` / succeeded |
| `output/pr-review/semanticist21-20261008/captured-fixtures-134a646e5/column_top_table_reserves_outer_margins--paragraphs_after_first_line_table_start_below_its_bottom_margin/0015.hwpx` | `0aad3cf3a34737e4593c81c10989899e224505bbfabe104a4c73667cc0155f4b` | `3a960780f6afb663ec83f0bf1b69e65001fcf68fc54a79c01c9b9914c9df6bfe` | `d872f979-3011-40bf-87b2-d1287ba5f56a` / succeeded |
| `output/pr-review/semanticist21-20261008/captured-fixtures-134a646e5/column_top_table_reserves_outer_margins--paragraphs_after_first_line_table_start_below_its_bottom_margin/0021.hwpx` | `80d658ee147e7ada087460808be94c6433abf7a0b4f08cd7126a6f159e632661` | `97559a36aa108672fd553edcea48ec8e6cd1d2f1e7c1b786ab924795babe5399` | `c8b4e1ed-da92-4fac-9a0d-a2ced4ad01ed` / succeeded |
| `output/pr-review/semanticist21-20261008/captured-fixtures/created_table_after_table_starts_below_it--a_table_created_right_below_a_table_starts_below_it/0010.hwp` | `e93fb91c8b2c28ff87744e8ef1f8be245f650fc3060f2c54eaebdb502e151ea6` | `a16a20a002f83d60f0cdf5ce1cfdd9488ead35bb5983439f7f55bba356420af1` | `0cf2e341-9c53-4a86-873e-13dc205b751d` / succeeded |
| `output/pr-review/semanticist21-20261008/captured-fixtures/created_table_after_table_starts_below_it--a_table_created_right_below_a_table_starts_below_it/0011.hwpx` | `bb7be43f0149db7981b3bebac7a8f002f7d617010c9e060517b37793d8cdbf0d` | `532a0728cf80ee150d1f1b0471f407c2368e6ebfb41fc1d9e0b26a39939e1e35` | `b9d014c2-e273-466d-96ca-18eab5dcb6bc` / succeeded |
| `output/pr-review/semanticist21-20261008/captured-fixtures/inline_table_after_field_or_bookmark--inline_table_after_bookmark_gets_its_own_row_after_merge/0243.hwp` | `74a49507963845d5e1603c9db50787ab26bd5762dbaab3d919fdf6fa8a918d0c` | `미산출` | `35cedd43-a4f1-4faf-9a82-29dbaa095c06` / failed |
| `output/pr-review/semanticist21-20261008/captured-fixtures/inline_table_after_field_or_bookmark--inline_table_after_bookmark_gets_its_own_row_after_merge/0244.hwpx` | `1792e69cfaf2daa3bb1ee12ec6dc28ffb28e85f56bcfdb87e28068e843f59cfc` | `96ec8c928a03b02f78952fffd5ae7ba57a34b468b56c40c3abcff3a1d17500d0` | `a3257aa5-eb48-4a11-a626-9ef46704b1b7` / succeeded |
| `output/pr-review/semanticist21-20261008/captured-fixtures/inline_table_after_field_or_bookmark--inline_table_after_field_gets_its_own_row_after_merge/0249.hwp` | `21819f0c2aaad0f4a2064df266a0f59ec80ee1d2233925e5c63e887f72e00b36` | `미산출` | `c604511f-5b0a-45d2-b334-100b08ccc1b7` / failed |
| `output/pr-review/semanticist21-20261008/captured-fixtures/inline_table_after_field_or_bookmark--inline_table_after_field_gets_its_own_row_after_merge/0250.hwpx` | `4536c5dce2d6bd9141c91440a21fcd47410bb141f44b8e7fb0d5fa3c7de9e382` | `0ae2eadcf1a1ad0bab34290333ac47caf488a71b800659630bd65b14e80eec4b` | `7fbb943c-b6f2-46c4-8221-6ad172fc29fb` / succeeded |
| `output/pr-review/semanticist21-20261008/captured-fixtures/inline_table_after_field_or_bookmark--inline_table_gets_its_own_row_after_merge/0255.hwp` | `4f1e524661775dfbc569166f75f2289f1402a020f77c33a7e0a3f9776874d423` | `미산출` | `0f87ea4b-57d9-4505-b5cf-9a1e8c7050ae` / failed |
| `output/pr-review/semanticist21-20261008/captured-fixtures/inline_table_after_field_or_bookmark--inline_table_gets_its_own_row_after_merge/0257.hwpx` | `dbaa6d12fde562d8f1a59448b2edd6946e601e6ccab36b72a415a23fb2ef4193` | `f888cfaeaa2e5b06e1d07c5d5cafb0272857f7059ab885accb8805cb6e7a6cd4` | `1c2a2207-7bdf-4294-9c27-592c748b5852` / succeeded |
| `output/pr-review/semanticist21-20261008/wasm-margin-following-134a646e5.hwpx` | `08d1e92569756587702448f82aefbf86337d7fdcfd900d82643a23a81bbae114` | `534bbfdbf1b68b9da6acb44269f0e20eebd2363e59fd4ab66b38d1b9fdcdc118` | `a5e07f52-3382-4c36-a374-bd9c1767bfc0` / succeeded |
| `output/pr-review/semanticist21-20261008/captured-fixtures/table_properties_outer_margin_sync--outer_margin_edit_moves_table_now_and_after_reopen/0522.hwp` | `c5d54efe1d6d10a58d200250c3a2fb409144efa05b332e8361f5bcd760c750ad` | `6a53375c45a68d6bf5fb53bdb8f7572e97fb04cd4955660d26def1c5aa6ec604` | `f2f342a0-4c50-4b10-baaf-af644a45cf17` / succeeded |
| `output/pr-review/semanticist21-20261008/captured-fixtures/table_properties_outer_margin_sync--outer_margin_edit_moves_table_now_and_after_reopen/0523.hwpx` | `c083046f7120f86b4e39cbd1e524fee6b1be789ca1680cc0f5e702fe52b610d4` | `3a6ccff438807eb571e9468312ba80fa2d68ac3f5fcb18c343653b3e39fb8e58` | `ff875439-f936-49ef-9302-e7b8006fb46e` / succeeded |
| `output/pr-review/semanticist21-20261008/captured-fixtures-134a646e5/updated_style_keeps_tabs_and_borders--a_style_tab_stop_moves_the_text_after_the_tab/0545.hwpx` | `e0a6bc5b14ba9c0da977b6319484ce65196199b05f00a16ee12396b22c48bbf3` | `3fac3a82586d718786990e7fc9d16738b993611f878388da37019600c995485d` | `0499a0e4-50b6-45a6-81cd-91f694324a44` / succeeded |
| `output/pr-review/semanticist21-20261008/captured-fixtures-134a646e5/updated_style_keeps_tabs_and_borders--style_borders_fills_and_spacing_reach_the_style_and_its_paragraph/0547.hwpx` | `e1b58ace1b9955a7aa984b580a557cba6d694c69533b89d9daecc09b47b147d0` | `5a9bd75bc850507f45394c240e14f11646ba0c133980e9b24abb9e9581ff0289` | `0f844619-ff94-45cb-89e6-269f1d090eeb` / succeeded |

## 실제 호출 경로와 규칙 판독

- #7656: `float_placement.rs::empty_table_anchor_ladder_lacks_outer_box`의 저장 line origin/단독 표/None split 판독 → `block/entry.rs`의 `ParagraphFloatPlacement {table_top, occupied_bottom}` → typeset table 배치/예약 및 `paragraph_float_placements` 저장 → `layout.rs`의 확정 table_top 소비와 후속 문단 흐름. 다음 문단의 실제 글줄에서 잔여 차이를 관측했다. 요구 높이·예산 실패·이월과 분할/마지막 유닛 경계 검증은 미완료.
- #7658: style JSON → TabDef/BorderFill 참조·borderSpacing → 해당 style을 쓰는 body/cell para shape 및 char shape 변경 → reflow/rebuild_section → renderer/paint에서 소비. 렌더러 파일 미수정만으로 조판 비해당이 아니며, 현재 배경 점유 끝이 독립 Print와 다르다.
- #7657: owned-tree walker가 body 외 table/caption/header/footer/note/master/ruby까지 스타일 번호를 옮기고, HWP master raw PARA_HEADER byte 10 및 section_def 사본을 별도로 보정. Native 실제 sample 저장/다시 열기 계약 통과. 렌더의 para_shape_id·char shape 소비와 style_id clone 수용 조건을 읽었으나 전체 조판 비해당 판독은 아직 완료하지 않았다.
- #7654: leading whitespace 판정(`space`, NBSP, figure space)과 marker prefix의 본래 식을 helper로 추출했으며 paint에 소비하는 문자열/char shape/UTF-16 처리 식은 유지. cursor query가 같은 helper로 원본 offset을 변환한다. Native/fresh WASM에서 한글·astral 문자·끝 공백의 그린 경계와 캐럿을 대조했다.

## 원 PR 피드백과 CI

| PR | 실제 게시 review | 상태 |
| --- | --- | --- |
| #7613 | [5445340620](https://github.com/edwardkim/rhwp/pull/7613#pullrequestreview-5445340620) | CHANGES_REQUESTED / API body·head 재확인 |
| #7641 | [5445376862](https://github.com/edwardkim/rhwp/pull/7641#pullrequestreview-5445376862) | CHANGES_REQUESTED / API body·head 재확인 |
| #7616 | [5445441420](https://github.com/edwardkim/rhwp/pull/7616#pullrequestreview-5445441420) | CHANGES_REQUESTED / API body·head 재확인 |
| #7653 | [5445441918](https://github.com/edwardkim/rhwp/pull/7653#pullrequestreview-5445441918) | CHANGES_REQUESTED / API body·head 재확인 |
| #7656 | [5445750827](https://github.com/edwardkim/rhwp/pull/7656#pullrequestreview-5445750827) | CHANGES_REQUESTED / API body·head 재확인 |
| #7658 | [5445751499](https://github.com/edwardkim/rhwp/pull/7658#pullrequestreview-5445751499) | CHANGES_REQUESTED / API body·head 재확인 |

원 PR head 17개를 live 재조회했다. #7614 exact source의 CodeQL `Perform CodeQL Analysis` 실패, #7652 exact source의 `Check Build & Test worker results` 실패와 queued worker, #7653 정책/reuse/adapter 실패는 별도로 보존했다. `gh run view --log-failed`는 4개 요청 모두 내용이 비어 있어 원인 로그를 확보했다고 보고하지 않는다. 세부 snapshot: `current-state-17.json`, `github-run-*.json`. CI 취소·재실행·설정 변경은 수행하지 않았다.

## 남은 완료 조건

1. 아직 미검증인 원 PR 편집/출력 경로, 반례, source-side 회귀 주장 판독을 완성한다. #7655 다섯 무늬 상태의 실제 패턴·색도 읽고 점수로 대신하지 않는다.
2. 2026-10-08 사용자 지시로 90% 미만 사례는 메인터너 보정으로 처리한다. 같은 누적 branch에서 독립 Print를 근거로 원인별 code 보정을 만들고, 변경한 소비 경로와 대조군을 재검증한다.
3. 최종 통합 code head에서 Rust lint 묶음, workspace build, 전체 nextest와 변경 범위의 필수 gate를 완료한다.
4. gate 충족 후 같은 통합 PR에 필요한 source input/Print PDF/대표 review·overlay/통합 기록/오늘할일을 넣고 정확한 head image URL을 실제 표시한다.
5. 통합 PR CI 모니터링·exact-head merge·upstream/devel 포함 검증·원 PR/이슈별 반영 및 credit·archive/assets·devel sync·소유 확인 뒤 cleanup까지 진행한다.

이 문서는 진행 기록이며 전체 목표의 완료나 승인 보고가 아니다. 실행 로그·중간 JSON·TSV는 ignored output에 보존하고 generated suite는 source PR에 stage하지 않는다.

## 승인된 메인터너 보정 계획 (2026-10-08)

- 승인 원문: “90% 미만인것들 메인터너 보정으로 처리”.
- 대상: #7613의 연속 표 원점, #7616의 inline 표 앞 글자 정렬·뒤 글줄, #7653+#7656의 표 뒤 문단 흐름, #7658의 문단 테두리·배경 점유. 좁은 중첩 표 반례(#7641)는 독립 Print 지표도 확보하여 기존 통합 blocker를 함께 판독한다.
- 규칙: 독립 Print의 실제 괘선/글줄/점유 끝과 입력의 저장 line 원점·높이·간격을 대조한다. 측정과 배치가 같은 결과를 소비하도록 고치며 문서 ID·절대 좌표 예외, 출력 숨김, clamp로 겹침만 제거하지 않는다.
- 생산/소비 추적: block/entry가 만드는 ParagraphFloatPlacement → typeset 요구·예약 높이/예산·이월 → layout의 table_top/후속 flow; composer 줄 구성 → paragraph_layout의 마지막 줄/양쪽 맞춤; paragraph_layout의 para_border_ranges → border_rendering의 최종 점유 상자.
- 검증: 작은 기존 경계와 동일 입력의 Native 직접 출력부터 보정 방향을 확인한다. code head별 fresh WASM을 새로 빌드하고 전체 관련 TSV·대표 review/overlay를 직접 읽는다. 90% 이상과 실제 결함 해소가 확인된 뒤 필요한 정식 관계 검사를 추가하고, lint/전체 회귀/통합 CI/merge/후속 처리로 진행한다.

### 보정 1: 문단 외곽선과 채움의 점유 상자

`paragraph_layout.rs`의 외곽선 점유 조건이 `fill_color.is_none()`을 요구해 테두리+채움인 문단의 후행 줄간격을 배경 상자에서 누락했다. 가시 외곽선의 존재로 점유 조건을 정하고, 같은 상자의 좌우 간격도 `para_border_ranges`에 넘기도록 보정했다. 새 문서의 식별자·절대 좌표 분기는 추가하지 않았다.

- 수정 전 Native/fresh WASM: 테두리/배경 82.33902%, 탭 대조군 100%.
- 수정 후 Native 진단 build: exit 0, 4분 19초. 두 입력 모두 100% (`native-updated_style_keeps_tabs_and_borders-{scores,review}-border-draft`).
- 직접 확대 판독: 녹색 문단 배경의 빠진 후행 영역과 좌우 범위가 개선됐다. Native 채움 하단 158.9333px / Print 160.3040px 및 stroke raster 차이는 잔여 차이로 기록한다. fresh WASM 재검증과 최종 판정 전 추가 판독이 필요하다.
- 기존 #5711 음수/양수 줄간격 관계 검사 2/2 PASS (`logs/fixture-capture-issue_5711_para_border_negative_spacing-border-draft.log`). 테스트 원본·기대값은 바꾸지 않았다. 새 정식 회귀 검사는 아직 추가하지 않았다.
- Native binary 및 source 변경은 현재 로컬 보정 단계이며 세 Clippy·전체 nextest·최종 WASM·CI·merge 증거가 아니다.

#7655 다섯 상태의 실루엣은 양 backend에서 100%였지만 직접 판독에서 patternType 2(세로줄)와 5(십자)가 대각선/대각 격자로 그려졌다. serializer와 공용 PatternFillInfo는 1~6, SVG/WebCanvas의 switch는 0~5를 소비하는 실제 불일치를 확인했다. 별도 원인으로 보정하며 점수 100%로 면제하지 않는다. Print의 무늬 bitmap은 100dpi에서 8 pixel 주기(96dpi 출력 약 7.68px)이고, 현 renderer tile은 6px라 주기도 대조 대상이다.

### 보정 2: 공용 무늬 번호 소비

SVG와 WebCanvas의 switch를 serializer/PatternFillInfo/CanvasKit과 동일한 1~6 번호로 맞췄다. Native 진단 build exit 0(4분 28초), 다섯 편집 상태의 review 생성 성공. 0003의 직접 판독에서 세로 방향은 Print와 일치한다. 6px tile과 Print 약 7.68px 주기의 차이는 남아 있으므로 무늬 전체 완료로 판정하지 않는다. fresh WASM 및 실제 Canvas backend 검증도 남아 있다. 테스트·기대값은 변경하지 않았다. 증적: `native-cell_background_pattern_change-review-pattern-draft/`.

### 보정 3: 비어 있는 표 앵커와 뒤 문단의 독립 점유

단 맨 위 `None` split 표의 저장 사다리가 표 바깥 상자를 담지 않은 경로에서, `ft.host_spacing.after`의 저장 앵커 lh+ls를 다음 글자가 있을 때만 표 뒤에 붙였다. 확정 `occupied_bottom`을 표 상단 + 실제 높이 + 아래 바깥 여백 + 문단 뒤 간격으로 정했다. `place_table_with_text` 예약과 layout의 `lane_flow_bottom`이 같은 확정 끝을 소비하며 후속 문단은 빈 줄도 자기 높이를 차지한다. 독립 Print의 두 입력에서 셋째 문단 baseline은 모두 332.117px, 글자가 있는 둘째는 310.860px다.

- Native build exit 0, 4분 26초(`logs/native-maintainer-empty-anchor.log`).
- 동일 기존 입력 0015/0021 review 최저 모두 100%: 이전 100%/86.01714% → 100%/100%. 전체 1쪽이며 표 괘선·뒤 문단 순서·간격을 `empty-anchor-enlarged.png`로 직접 비교했다.
- 기존 source 계약 1/1 PASS(`logs/fixture-capture-column_top_table_reserves_outer_margins-empty-anchor-draft.log`). 정식 nextest는 별도로 남아 있다.
- 증적 `native-column_top_table_reserves_outer_margins-review-empty-anchor-draft/`. fresh WASM, 예산 실패·이월 대조, 신규 정식 회귀는 아직 미완료.

### 보정 4: composer 줄 소속과 개체 상자의 우선순위

- #7616: 현재 문단의 줄 경계·높이·폭과 조합 결과가 같은 경우 UTF-16 제어 슬롯의 소유 줄을 `composed_tac_line_assignment`로 측정(`tac_control_indices_for_line`)과 paint가 공유한다. 저장 줄과 편집 재조판 줄은 각 출처 안에서 조회하며 혼합 사다리는 기각한다. 저장 캐시 수용 조건을 바꾸지 않았다. 경계 충돌이 있는 TAC 문단은 별도 inline painter 대신 일반 composed painter로 보내 줄별 정렬·높이·표 위치를 함께 소비한다.
- 공백 없는 양쪽 맞춤은 slack을 글자 사이 N-1곳에 배분한다. 기존 N 분배는 마지막 글자 뒤에 여분을 남겨 잉크 오른쪽이 안쪽에 놓였다.
- #7613: 표 공간을 담지 않는 빈 앵커의 확정 개체 상자를 saved-host 포맷보다 먼저 선택한다. 단 시작과 본문 중간 모두 `table_top/occupied_bottom`을 fit·예약·layout lane에 전달한다. 다음 표 앵커의 안내 줄간격 600HU(8px)를 앞 표 아래에 더하지 않는다.
- 중간 진단 `shared-rows-draft`: inline 73.33928%, 연속 표 두 경로 38.42346%; 부족한 실제 간격을 확인하여 추가 보정했다. 이 결과는 완료 근거가 아니다.
- 추가 보정 Native build exit 0, 4분 21초(`logs/native-maintainer-row-gaps.log`). 동일 입력 전체 1쪽 TSV: inline 일반/누름틀/책갈피 세 건 모두 100%, 연속 표 HWP/HWPX 두 건 모두 100%. `*-row-gaps-enlarged.png` 다섯 장에서 앞 글자 맞춤, 표 괘선/간격, 뒤 글자·끝 문단의 순서와 간격을 직접 대조했다.
- Native 진단 계약 95 PASS / 1 FAIL(알려진 좁은 중첩 표 반례), 원 source/기대값 변경 없음. 94 source assertions(기존 92 + #5711 2)와 여백 뒤 문단 반례 통과이며 정식 nextest 증거는 아니다.
- 아직 필요한 범위: fresh WASM/대표 review, 예산 실패·이월에서 안내 줄이 다시 추가되지 않는지 확인, 정식 관계 회귀, lint/전체 회귀.

### 추가 intake

진행 중 새 비 draft [#7659](https://github.com/edwardkim/rhwp/pull/7659), head `db6f70d9c523692080f5318e406a578c11e6c32e`를 발견했다. 신규 각주 스타일 번호와 모양 선택을 수정하며 원 head의 시각/Studio gate는 미실행이다. head fetch 및 jangster77 reviewer 지정 후 `47a304211`·`cd0eb2d25`로 누적 적용했다. 기존 source assertions 3개는 Native 진단에서 통과했으며 실제 인쇄와 fresh WASM 검증은 남아 있다. 기존 17 head는 변경 없음을 재확인했다.

### 진행 중 보정 5: 빈 앵커 예산과 HTML 중첩 표의 실제 점유

- `EmptyHost` 흐름을 명시하여 확정 개체 상자의 후보가 예산에 안 맞아도 더 작은 legacy 높이로 교체되지 않게 한다. 높이 요구·예약·paint 완료는 동일 흐름 판정을 소비한다.
- 독립 작은 페이지 입력(`maintainer_object_boxes-0001-a6554951e542`)은 body 175px이며 Native/Hancom 모두 표를 첫 쪽, 뒤 문단을 둘째 쪽에 놓았다. 아래 바깥 여백까지의 예약 끝은 body를 초과할 수 있으므로 전체 표의 실제 괘선/후속 내용 관계를 확인하고, 예약 높이 자체를 페이지 쪽수만으로 fit 증거라고 하지 않는다.
- 좁은 HTML 셀의 1인치 폭 하한을 제거하고 composer 토큰/자식 개체의 최소 폭을 열 그리드에 아래에서 위로 예약했다. [CSS 자동 표 배치](https://www.w3.org/TR/CSS2/tables.html#auto-table-layout)의 셀 최소 내용 폭 원칙을 적용한다.
- 첫 폭 보정 뒤 source assertions + 진단 반례 99/99 PASS. 재생성 input `f7d474366850...`, Print SHA `9a8ab74341e5182b95f5a77f68519d77cd1c48864024fd94fcd115068c1afe41`, Native 99.48187%였으나 확대 PNG에서 ‘안’의 오른쪽/아래가 잘렸다. **실제 결함 미해결**로 판정하고 정식 테스트 추가를 보류했다. 원본 입력의 Print도 잘리므로 재생성 대조군 통과를 원본 개선으로 바꾸어 보고하지 않는다.
- 실제 원인: import 폭/LineSeg는 cell.padding=141HU를 쓰지만 `apply_inner_margin=false`의 공통 측정·paint는 table.padding=510HU를 쓴다. 현재 `Cell::effective_padding`을 import 폭과 reflow에 소비하고 `HeightMeasurer::measure_table`의 행 크기를 cell/common/raw 저장 크기 및 host LineSeg에 함께 반영하는 보정을 진단 중이다.
- 첫 유효 셀 상자 build는 exit 143으로 종료되었고 성공으로 보고하지 않는다(컴파일 오류 출력 없음). 동일 범위에서 무늬 보정까지 묶은 build를 재실행했다.

### 진행 중 보정 6: 독립 Print의 여섯 무늬

- 1~6 모두 포함한 동일 입력 `216166ae3b7d760fd03d9ebb297e8bb268fca5a13e0534380a8a8465901a58ae`; Hancom job `eee5c48f-346a-4856-9dd9-67fa8707dd20`, PDF SHA `9ac1553a70c91cb86d890ea9d9e2b50cd5c34bc230420db3512a2a9f7e9b41e4`.
- 실제 100dpi bitmap은 8dot 주기, 쪽 원점 기준이다. 3번은 우상향 slash, 4번은 우하향 backslash로 기존 SVG/WebCanvas/CanvasKit의 대각 소비와 반대다. 기존 번호만 맞춘 보정을 전체 무늬 완료로 취급하지 않는다.
- 현재 공용 `PatternFillInfo.tile_size`에 576HU 반복 주기를 출력 DPI로 변환해 생산하고 SVG·WebCanvas·CanvasKit에 전달한다. CanvasKit도 원래 도형 path의 채움 shader를 사용하도록 해 줄이 도형 밖으로 그려지는 기존 경로를 제거한다. WebCanvas의 반복 변환은 [CanvasPattern 표준](https://html.spec.whatwg.org/multipage/canvas.html#dom-canvaspattern-settransform)을 따른다.
- Native/fresh WASM/실제 Canvas 두 backend의 직접 확인·최종 gate는 아직 남아 있다. Skia 기존 단색 fallback을 실제 무늬 출력으로 보고하지 않는다.

#### 보정 5/6 Native 재검증

- Native build v1은 drawing style에 DPI 전달을 빠뜨린 컴파일 오류로 exit 101. 호출 지점에 DPI를 전달한 v2는 exit 0(4분 26초).
- 101개 진단 실행 PASS: 97 source assertions + 2 기존 반례 + 2 관측용 입력 생성. 관측용 2건은 관계 assertion을 갖춘 정식 회귀가 아니다. source/기대값의 완화는 없다.
- 새 narrow input `af77438524a5...`, Print job `a7fbd9b2-df30-468f-b799-154ddacde385`, PDF SHA `b92e437f398ecf1b3243fec9ef34a7ad588f2e28a13072500f524fa1f6b71339`. `narrow-effective-cell-box-native-print-zoom.png`에서 ‘안’의 오른쪽/아래가 온전히 보이고 부모·자식·옆 셀의 순서와 위치가 Print에 대응한다. Native gate는 통과했으며 fresh WASM은 남아 있다.
- 무늬 6종의 `six-patterns-common-boxes-v2-zoom.png`에서 방향과 반복 주기의 큰 차이가 해소됐다. stroke raster 강도와 1~2px 위상 잔여 차이는 기록한다. Native 6종 gate 통과, Canvas 실제 경로/fresh WASM 검증은 남아 있다.
- 최초 작은 용지 사례는 Native 61.2844%였다. 독립 Print의 MediaBox가 314×595pt, 입력은 width59528/height31549/WIDELY로 가로세로 순서가 맞지 않았다. 한컴에서 종이가 세로로 선택되어 표 오른쪽이 잘린 것이다. 원 입력·PDF·실패 증거를 보존하고, width30000을 표 생성 전에 설정한 portrait 대조군을 별도로 생성했다. 원 입력 통과로 바꾸어 보고하지 않는다.
- Studio TypeScript `npx tsc --noEmit`: exit 0. 실제 브라우저 E2E 증거는 아니다.

### 보정 5/6 fresh WASM 및 실제 Canvas 확인

- 고정 code head `f58718f70436e2ee9899539648e1a72f4ea00864`. Native binary SHA `e67cc8e87f83354544aa22c5edfeecc0508678e4f98541305bcd2439734b7645`, WASM SHA `b32bb4711ffcf8092bcf9a3c6b51a713851c2edcd5c3794c81d4ebaf5a9e833a`, JS SHA `e5bd272394d3c54a433af0b90e2851a75c9b03f6acd9735b5c025c7e4738d4ec`. output의 `build-provenance-f58718f70.json` 및 고정 binary/package 사본으로 재현한다.
- 루트 `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt` exit 0, 9분 48초. 진단 빌드이며 최종 head 검증은 별도로 남아 있다. pkg/public JS·WASM hash 일치와 public contracts 6/6 PASS.
- fresh WASM 동일 입력 최저값: 연속 표 HWP/HWPX 100/100%, inline 일반/누름틀/책갈피 모두 100%, 단 맨 위 표 뒤 빈/가시 문단 모두 100%, style 탭/테두리·배경 모두 100%. 좁은 유효 셀 상자 새 입력 `af77438524a5...` 100%, 여섯 무늬 모두 100%. 이전 잘린 입력 `f7d474366850...` 99.48187%도 보존하며 새 입력 통과로 이전 출력을 개선했다고 하지 않는다.
- 실제 Chromium Canvas2D/CanvasKit software replay로 여섯 무늬를 확인했다. nearest shader는 소수 DPI에서 대각선이 점선처럼 보였고, 현재 TS Linear 보정 후 `patterns-canvas-linear-print-zoom.png`에서 방향·주기 큰 차이가 사라졌다. stroke AA/1~2px 위상은 잔여 차이다. CanvasKit 제목은 host font 미등록으로 Noto Sans KR fallback이며 글꼴 정합 증거가 아니다. Skia는 기존 단색 fallback이어서 무늬 지원으로 보고하지 않는다.
- #7566 실제 Studio의 `loadDocument → setActiveField → refreshDocumentAgentMutation → clearActiveField → refreshDocumentAgentMutation`을 1행/80행 중첩 표에서 실행했다. 소유 cellPath 보존, 안내문 1→0→1, pageCount 1/4, 실제 visible-page render 완료 경계까지 PASS(`browser-wrapper-activation-f58718f70.json`). 초기 harness의 없는 메서드 호출은 수정하여 재실행했으며 제품 결함으로 세지 않는다. 현재 최종 WASM 재실행은 남아 있다.

### 진행 중 보정 7: 주석 참조 크기·기준선과 미주 시작 경계

- 독립 Print의 8/10/20pt 본문과 각주/미주 여섯 public API 입력을 생성했다. 참조 번호는 nominal size의 약 75%, 본문 실제 기준선보다 약 0.21em 위였다. 선두 미주 control slot 10pt와 뒤 8/20pt text slot을 구분했다. 선두 빈 각주 참조가 숨겨졌던 기존 분기도 보정한다.
- 생산/소비: raw control slot char shape → nominal row font / 75% marker size → 공통 측정 line max와 실제 TextLine 최종 기준선 → FootnoteMarker baseline → SVG/WebCanvas/Skia/paint JSON/CanvasKit. 조기 provisional baseline으로 선두 번호를 발행했던 진단 v2는 실패해 최종 첫 TextLine에서 발행하도록 수정했다.
- Native note-flow-boundary-v2 binary SHA `6774ccd3358f87c44207478843e0f48cc5c5b19f48b9739aecddc2abb7cbed98`, build exit 0(4분 53초), 진단 101/101 PASS. 주석 크기 여섯 input 최저값은 98.42374~100%. 빈 각주/미주 동일 입력은 99.4152/100%. 선두 번호의 실제 확대 판독과 fresh WASM은 아직 남아 있다.
- 마지막 NoLS 일반 본문의 외곽선 없는 terminal occupied height를 미주 시작에 공유한다. 외곽선은 기존 점유 줄간격을 유지하며 저장 LineSeg/개체 host 경로를 새 계약으로 대체하지 않는다. 확정 separator flow_origin은 PageItem → layout이 소비한다. 새 origin을 pagination convergence 비교에 포함해 오래된 페이지 cache 재사용을 막는다.
- 혼재 각주+미주 여섯 독립 입력의 Print: FE는 두 쪽, F/E 단독은 한 쪽. 각주 footer 점유가 있으면 미주를 새 쪽으로 넘기도록 보정했으나 새 쪽에 불필요한 구분선·여백이 남아 v2 최저 30.50431%로 실패했다. 실제 새 쪽 첫 미주는 본문과 나눌 경계가 없음을 직접 확인하여 현재 예약/배치 공통 경로를 추가 보정 중이다. Footnote inline-after-text 미지원 경로를 검증했다고 보고하지 않는다.
- 실제 상공신문 편집 input `b3c9896ea898...` 88.42212%, HWP3 sample 편집 input `6b4f6a375029...` 최저 41.35911%는 여전히 실패다. 전자는 chart/data·본문 재배치, 후자는 둘째 쪽 본문 원점 차이를 직접 확인했다. 참조 번호 수정만으로 전체 출력 완료로 판정하지 않는다.

### 추가 발견과 현재 검증 범위

- 올바른 filename context(`보고서.hwpx`)로 HF template 20개를 재생성·Print했다. header/footer4는 86.61972%, header10 74.10072%, footer10 76.25355%. 직접 확대에서 tab 구간 밑줄 누락과 번호 끝 위치 차이를 확인했다. 파일명 mismatch인 이전 캡처도 보존한다. 현재 intra-run tab의 표시 문자열과 모델 placeholder를 구분하는 보정 중이다.
- field/control slot 23개 및 leading-definition 7개 전체 TSV를 추가 확보했다. 빈 누름틀 안내문은 인쇄에서 숨겨졌지만 내용 좌표를 옆으로 밀고 있었다. 현재 안내문 삽입 전 실제 내용 상자를 생산 위치에 보존해 인쇄 레이어가 같은 상자를 소비하도록 보정 중이다. editor 안내문/활성화와 저장 text/control 슬롯은 유지한다.
- inline equation이 든 셀에 여러 HTML 문단을 붙인 input `87a311ae9d91...` 최저 15.79909%: Native는 표를 앞 글자보다 위에, Print는 앞 글자 뒤에 놓았다. 원 source의 token/slot assertions 통과로 실제 순서 결함을 면제하지 않는다. wide TAC host 저장/재조판 경로를 추적 중이다.
- proper portrait budget input `55aa3cfd4cd8...`: 기존 Native p1 32.11845/p2 0% → EmptyHost의 실제 table body fit와 후속 occupied flow를 구분한 보정 뒤 두 쪽 100%. body-only miss/physical paper miss 대조군 및 fresh WASM 확인은 남아 있다.
- 위 추가 보정은 현재 dirty 진단 단계다. 새 정식 rendering 회귀/fixture/기대값을 추가하거나 완화하지 않았다. 통합 PR 생성·승인·merge·후속 처리 완료가 아니며 최종 code head Native/fresh WASM 직접 판독과 필수 lint/full gate가 남아 있다.

#### 보정 7 이후 Native 진단 결과

- note-guides-v3 build exit 0(4분 18초), binary SHA `5d0a828310cb226810739d66136898ea169d2aec43de8fe7cbd230ce3dd4adca`, 기존 진단 101/101 PASS. 혼재 주석 여섯 입력은 최저 99.71126~100%, 두 쪽 FE에서 불필요한 미주 구분선을 제거했다. 아직 fresh WASM/정식 관계 회귀는 남아 있다.
- 누름틀/컨트롤 23개는 v3에서 18개가 90% 이상, 다섯 실패는 넓은 TAC 표/그림 혼재다: 0079 8.88818%, 0114 12.74659%, 0132/0158 14.37790%, 0148 7.72191%. 처음 실패했던 0069는 0.55249→100%로 인쇄 내용 좌표가 개선됐다. 원 input을 바꾸지 않고 비교했다.
- filename-context HF 20개 v3: header/footer4만 86.61972%로 남았다. header/footer10 밑줄 누락은 개선되어 90% 이상이지만 `hf-header10-display-width-print-native-v3.png` 직접 확대에서 숫자와 밑줄 오른쪽 끝이 약 6px 밀렸다. 통과 점수로 면제하지 않는다.
- 8/10/20pt의 plain `L\tR`, filename/page field, field 뒤 X를 독립 Print했다. plain right tab은 실제 tab stop에서 끝나고 field 포함 block의 정렬 폭은 가시 glyph 폭보다 큰 것으로 관찰했다. 단순 6px 이동을 추가하지 않고 control slot/field box 출처를 추적 중이다. Hancom 저장 HWP 대조도 확보했지만 저장 파일명에 따라 PATH 값이 바뀌므로 최초 input의 정합 근거로 바꾸지 않는다.
- HTML cell-flow-v4 build exit 0(4분 54초), binary SHA `debbb9809350d5e83838aae737d696ab216cddf34db5b0a8c90cf7b47d59a9f5`, paste/caret 진단 5/5 PASS. 셀 문단 vpos/inline host LineSeg를 새 점유로 발행했으나 새 input `a100f93c3c2d...` / Print SHA `67b32e51f3907802a54be129a67b218a80c456e4dd6317df1eb19a0481b5ed63`의 Native는 12.24209%였다. 앞 글자 누락과 표 위치 결함이 남아 **보정 완료가 아니다**. 원 input `87a311ae9d91...`도 보존한다.
- TAC host-frame-v5 build exit 0(4분 25초), binary SHA `deba366df3e1b51116b581fe5a76647c5100fc6bd70b75791a7a22509cd12ad9`, 진단 101/101 PASS. NO_LS 본문 TAC frame에서 별도 머리말/꼬리말을 폭·높이 소비자로 오인하던 거부 조건을 제거하고 표 바깥 여백/실측 높이를 공용 줄 메트릭에 반영했다. 동일 0079 input은 8.88818→74.63732%로 개선했으나 표 뒤 글자가 같은 줄로 붙어 여전히 실패다.
- 추가 원인: 새 inline table factory가 `common.treat_as_char=true`인데 `table.attr`에 TABLE-record 값 `0x04000006`을 넣고 `common.flow_with_text`/CTRL_HEADER 흐름 flag를 누락했다. live/native, HWP 저장, 순수 HWPX 재열기의 flow 판정이 갈린다. 현재 실제 inline 생성 경로의 CommonObjAttr/CTRL_HEADER/cache를 같은 flags로 발행하도록 보정 중이다. 기존 TABLE record attr와 기존 저장 자료의 수용 조건은 바꾸지 않는다.


### 진행 중 보정 9: 표 생성·셀 편집의 실제 줄 소속과 독립 개체 슬롯

- inline table factory의 CommonObjAttr/CTRL_HEADER 흐름 flag를 일치시켰다. 삽입 전 host end를 고정하고 `reflow_paragraph → recalculate_section_vpos`로 뒤 문단 좌표를 갱신한다. TABLE record 속성은 별도의 기존 계약을 유지한다.
- HTML cell paste는 flat/deep 경로 모두 셀 문단 reflow/vpos 갱신 뒤 공통 HeightMeasurer의 `reserve_content_heights → host reflow → section vpos`를 소비한다. 저장 줄의 TAC 소속은 실제 raw control slot과 줄 경계를 사용하며 높이 매칭을 소속 증거로 삼지 않는다.
- Native v7 binary SHA `3fd77aaa4c49e1de094bf1e9182865da02e055cb05d42099c0e4a5afc0f74504`, build exit 0(4분 21초), 진단 101/101 PASS. paste API 재생성 input `433acb46d996...`의 독립 Print PDF SHA `4c6b83ee91a1b39989cee3a973f82fa2046ce70b1792be09ab41185cbf083be4`, job `04af4d5c-1ed7-444a-bf5c-932e3087390f`, PrintMethod=0. Native 98.41537%, 앞 글자/표/뒤 문단 순서 회복. `paste-html-inline-equation-cell-v7-zoom.png`에서 수식 글꼴·연산자 간격 차이가 남아 있으므로 완료 판정은 보류한다.
- NO_LS 혼합 표 host는 표 폭을 뒤 글자에 합치던 방식 대신 원시 control slot을 별도의 U+FFFC 프레임 토큰으로 투영한다. 원본 IR은 변경하지 않는다. 개체 폭/높이와 원시 경계는 공통 occupied-size 측정 → Frame → LineSeg → TAC 배치가 소비한다. 표 끝과 뒤 글자는 다른 물리 줄을 소유할 수 있다.
- Native v9 binary SHA `5717c2c2cd1efb9eb276d5c9b37e0a7e4a8510709fefd9d14c32714feaeb08df`, build exit 0(4분 30초), 진단 101/101 PASS. 기존 Print 입력 24개 중 23개가 90% 이상(최저 99.35414%). `0079-896269c1daf6` 및 API 재생성 `0079-f05117fd896e` 모두 100%. 원본 review PNG에서 앞 글줄 → 표 → 끝 글자/그림 순서와 외곽선을 직접 확인했다.
- 기존 `0148-9dee19f77e58`은 옛 factory가 만든 잘못된 한 줄 cache를 유지해 7.72191%다. 수정 API로 재생성한 `0148-f00e5cab1285`는 실제 두 줄을 저장하고 독립 Print와 Native v10에서 100%. Print job `81e31328-57c9-4252-aa62-5bcf5fd7841f`, PDF SHA `a90f5a82274104102221544a08530f5dfdcb59fb57aae2f61e4bcf454c1b09f6`. 새 산출물 통과를 옛 저장본 렌더 개선으로 보고하지 않는다.
- 추가 Native v7 확인: auto-number trailing placeholder 네 입력 최저 97.09229~97.61647%. leading definition 일곱 입력 중 `0378-39da46633749` 86.31922%는 남아 있고 나머지는 98.24937~100%. fresh WASM 및 최종 확대 판독은 미실행이다.

### 진행 중 보정 10: 본문 각주 소유와 실제 예약 상자

- 상위 문단에 표가 있다는 이유로 형제 본문 각주까지 등록하지 않던 `!has_table` 분기를 제거했다. `FootnoteSource::Body`와 table-cell fragment queue의 `TableCell` source는 별개이며 셀 각주를 중복 등록하지 않는다. bodyless 판정은 새 각주의 원인이 아니었고 그대로 유지했다.
- Native v8 SHA `1d7c1acb84dce29bf445816e4f1ea9e62abb92b8f3915ef3575b8b4e74cbeb7e`, build exit 0(4분 37초), 진단 101/101 PASS. HWP3 변환 입력의 누락 각주는 표시됐으나 2→3쪽 과분할이 발생했다. 이 결과를 통과로 세지 않는다.
- 단일 단의 등록된 본문 각주도 `LayoutEngine::estimate_footnote_area_height_with_metrics → reserve_painted_footnote_area`를 사용한다. 실제 구분선/위아래 여백을 포함한 상자에는 추정용 40px safety margin을 중복 차감하지 않는다. 진입 fit와 분할 루프가 같은 `available_height`를 소비한다. 다단 경로는 이번 공통 측정 적용 밖이며 검증을 승격하지 않는다.
- Native v10 SHA `f0bb932e113837181649f85ccf3c91515b823129b61e4d77ffd081bc8e01f6e1`, build exit 0(4분 34초), 진단 101/101 PASS. 혼재 주석 여섯 독립 입력은 99.71126~100%를 유지했다. HWP3 입력은 다시 2쪽이나 p1 78.05031%, p2 41.35911%: 삽입 전 저장 쪽 끝의 빈 문단을 감추는 cache/guide 경로가 남아 있다. 143E chart 입력은 88.42212%다.
- 각주 삽입으로 가용 높이가 달라진 뒤 문단의 이전 프레임 cache를 API에서 무효화하는 v12 보정을 검증 중이다. 문단 내용과 글자모양은 보존한다. 재생성 입력의 독립 Print/Native를 먼저 확인하며 옛 입력의 실패 증거는 보존한다.

### 진행 중 보정 11: 오른쪽 탭의 동적 필드 상자

- 8/10/20pt filename/page field, 일반 `L\tR`, field 뒤 `X` 대조군의 독립 Print를 생성했다. 필드 블록은 반각 종단 슬롯을 정렬 폭에 포함하지만 일반 문자 블록은 그렇지 않았다. 고정 px 이동 대신 source control/range와 해당 char shape의 글자 크기로 폭을 구한다.
- `right_tab_fields::tail_space/apply_block_width → right_tab_block_width/estimate_line_runs → emit_line_runs → compute_char_positions`로 측정·배치가 같은 정렬 폭을 소비한다. 현재 독립 증거는 한 개 동적 필드가 있는 RIGHT tab 블록이며 여러 필드/개체 혼재 블록은 새 지원으로 주장하지 않는다.
- Native v11 SHA `beda69a9db570d50dc1bbfc2bed219c8868f71d6dc59d50a14d8581a2273da9c`, build exit 0(4분 22초), 진단 101/101 PASS. 초기 컴파일에서 별도 float-host 관측형을 LineSeg로 잘못 취급한 추가 조회는 제거 후 다시 빌드했다. 현재 filename 20개, 크기 대조 6개, plain 3개, suffix 2개를 비교 중이며 최종 결과·확대 판독·fresh WASM은 아직 남아 있다.

현재 code head 뒤 보정은 모두 로컬 진행 중이다. fresh WASM은 여전히 `f58718f70`이므로 위 Native 결과를 최종 WASM 검증으로 사용하지 않는다. 새 정식 렌더링 회귀/fixture/golden은 추가하지 않았다. 남은 90% 미만, 수식 확대 결함, 최종 lint/full nextest/Studio/통합 CI가 해소되기 전에는 통합 PR 생성·승인·merge를 하지 않는다.

- Native v11 최종 filename 20개는 모두 90% 이상(최저 90.22403%). 다만 추가 크기 대조의 PAGE 20pt는 89.91884%로 실패했다. 이 경우 종단 AutoNumber의 표시 placeholder는 raw control의 가시 위치와 달랐다. `resolve_auto_number_placeholder_positions`를 표시 치환과 정렬 폭 조회가 공유하도록 v13에서 보정했다.
- Native v13 SHA `f23f2a6ff0d7923d8571b879b97759b0fcc3f9e1d625bd050a3d45429b346074`, build exit 0(4분 25초), 진단 101/101 PASS. 동일 Print의 8/10/20pt 파일명·쪽번호 여섯 입력은 99.7331~100%. 20pt review PNG에서 번호·underline 끝의 큰 위치 차이가 사라졌다. 최종 확대/전체 head/fresh WASM은 남아 있다.
- Native v12 cache 무효화 입력 HWP3 `0ca435b9dae9...`의 독립 Print job `cae7a3cd-ddde-4685-8cbd-2244c6c0715e`, PDF SHA `71451896fb854b306d2c948aa4b0c9d30488d7763d5ad6b7f6c3f6f681d36716`. p2는 92.06827%로 빈 줄 소속이 개선됐지만 p1은 51.1407%로 악화했다. 첫 control-only host의 참조 누락·표 앞 폭·뒤 줄간격이 남아 있어 이 cache 변경을 완료로 판정하지 않는다. v12 binary SHA `c6be6b2de1d6cedff2dd1bb1eea7bef4763b4e2523998d5d56800422cb13dfd2`, build exit 0(4분 36초), 진단 101/101 PASS.
- control-only 문단에서 개체마다 없는 가시 문자 위치를 늘리던 fallback을 가시 경계 0으로 투영하는 v14를 검증 중이다. 원시 UTF-16 control 순서는 보존한다.
- 143E OLE의 길이 prefix와 `Contents` 원문을 ignored 경로에 보존했다. 기존 bounded VtChart 6 reader는 객체 그래프를 끝까지 읽었지만 column type 1을 거절했다. 독립 source metadata에는 axis 0..2000/4 divisions와 계열 fill `#73b1d1/#cd6565/#ebc452`가 있었다. 일반 fallback의 1702 축·다른 팔레트와 구별되며 아직 제품 코드에 적용하지 않았다.

- v14 Native SHA `6044ab9a9e21e681e789aff274c97c71792d479f0526ed6d0d4f9d67ea45ad43`, build exit 0, 진단 101/101 PASS. 그러나 빈 문단의 모든 `control_text_positions`를 0으로 만들면 주석 삽입 명령이 같은 경계의 표 뒤에 주석을 넣는 부작용이 확인됐다. 새 입력 `9aa1284b66ce...`와 Print job `a0d3724a-208e-4686-950c-bda67f6aed5d`를 보존하고 해당 전역 변경은 되돌렸다. 기존 v12 입력 비교는 여전히 p1 51.1407%로 실패했다.
- v15는 TAC 표와 같은 저장 줄을 소유하는 비-TAC Square 도형 때문에 후행 간격을 버리는 paint 분기를 보정한다. 원시 control 슬롯의 실제 줄 소속을 확인하며, 선행 그림·표·TAC 도형의 별도 점유 경로는 유지한다. 측정은 `tac_reconcile::measure`의 소유 줄 간격을 이미 소비하고 있었다.

- v15 Native의 최종 실행 파일 SHA는 `98616861774255774e93b6cbbe497e7bc98f5e6f2e16e87b692ad72f93d9648b`다(build exit 0, 4분 46초). 빌드 완료 전 이전 실행 파일을 복사한 오류를 확인해 다시 고정했고, Cargo 종료 뒤 진단을 다시 실행해 101/101 PASS를 확인했다. 같은 v12 입력/Print p1은 51.1407%→77.15709%로 개선됐으나 실패다. `hwp3-host-trace-v15.log`의 Table 뒤 흐름은 162.6px였다. 원시 저장 각주/표 줄의 간격 절반만 소비되고, 뒤 문단의 이전 vpos를 무효화했으므로 나머지 간격을 복원할 소비자가 없었다.
- v16은 `tac_host_trailing_spacing_at_boundary`를 `tac_reconcile::measure`, `stored_tac::prepare`, 실제 `layout_table_item`이 함께 호출하게 한다. 다음 문단의 저장 줄이 없거나 무효인 재조판 경계에서는 마지막 원본 TAC 줄의 양의 간격 전량을 소비한다. 원래 저장 줄/현재 RowBreak 줄의 기존 절반 간격 계약과 음수 Fixed 경로는 유지한다. `note_reference::leading`은 일반 문단과 빈 표 호스트의 번호·스타일·전진폭을 공유하고, 표 호스트는 같은 원시 소유 줄에서 표보다 앞선 참조만 그린다. 코드/시각 검증 진행 중이다.
- v13 최종 머리말·꼬리말 파일명 문맥 20개 전체 TSV가 완료됐다. 모두 100%, 누락 없음(`native-hf_filename_context-scores-field-slot-metrics-v13/summary.json`). 추가 8/10/20pt 여섯 Print 대조는 99.7331~100%이며 `hf-tab-20pt-v13-zoom.png`에서 번호·파일명·밑줄 끝을 확대 확인했다. SVG 재조판 잔차와 fresh WASM 확인은 최종 head에서 별도 수행한다.

- v16 Native SHA `84552c6c155c5faa5deb1d4f71eafb0a13cd1f5d47ec90c4d2f13b9dd879ad71`, build exit 0(4분 21초), 기존 진단 101/101 PASS. 기존 `#6812/#6879/#6167/#6179/#7160/#5708` 검사 원문을 ignored driver에서 실행한 32/32도 PASS(기대값 변경 없음). v12 HWP3 입력/같은 독립 Print 두 쪽의 full review gate가 Native에서 통과했다. 원래 v10 입력과 fresh WASM, 참조·간격 확대 확인은 별도다.
- 추가 확인: v15에서 최신 각주 API를 거친 143E 차트 입력 `8394cca6978e...`는 81.87141%로 실패, 옛 입력의 88.42212%와 혼동하지 않는다. 선두 정의+HTML 표+종이 기준 개체 입력 `39da46633749...`도 86.31922%로 남았다. HTML 표 factory의 raw 속성 `0x082A2311`에는 flowWithText·Table numbering·Para 기준 등이 있지만 수동 생성 common은 이 값들과 달랐다. v17은 `parse_common_obj_attr(&raw_ctrl_data)`로 양쪽을 같은 source에서 해소한다. 새 편집 입력과 독립 Print를 다시 생성해 검증한다.

- v16/v17의 같은 HWP3 v12 입력 p1 95.51084%, p2 92.07237%. `hwp3-note-reference-v16-zoom.png`에서 선두 참조 번호·표 안 글자·첫 본문과 하단 번호/구분선을 확인했다. 표 소속과 선두 참조 누락은 해소됐고 글꼴 raster·기존 footer 폭 잔차는 남아 있다. FE/E/F 여섯 대조군은 v16 최저 99.71126%로 유지됐다. 반면 이전 API가 남긴 원래 입력 `6b4f6a375029...`는 v16 p1 82.00126%, p2 41.35911%로 실패한다. 새 입력 통과를 옛 입력의 해결로 보고하지 않는다.
- v17 Native SHA `10719d1c5fba335efd534fcb0582782f3aa6f972967dbec592fa7c12c1e8a8ca`, build exit 0(4분 21초), 진단 101/101 PASS. 수정 source 21개와 base `301840aec6df20cc1dac4cf2a9e20ccccabebf3a`의 hash manifest를 ignored `source-code-v17/`에 고정했다(기존 diff patch에 빠지는 새 모듈 원문 포함). 선두 정의+HTML 표+종이 기준 개체의 새 입력 `4a66fd38f7f6...`, Print job `2a5bdcdd-06c5-4fbb-b5ea-9c0656e3c4aa`, PDF SHA `6bbce4b616b7afb7daad3c5e23e302ca65afbd0d14664923bf64cc959121903d`를 보존했다. full Native review와 fresh WASM 검증은 진행 중이다.
- 차트 metadata 추가 진단에서 저장 VtFont 14byte payload의 크기(f32 8/9/10pt), legend 배치/section rect를 확인했다. `metadata_probe.log`와 `chart-metadata-v15-zoom.png`는 ignored 진단이며 product parser/renderer 확장은 아직 적용하지 않았다.

- e13595c90 fresh WASM build exit 0(9분 22초). `pkg`/Studio public의 JS SHA `e5bd272394d3c54a433af0b90e2851a75c9b03f6acd9735b5c025c7e4738d4ec`, WASM SHA `f80df687085fffaaec3fe1ebc20f95def81a5f30e587c01768afec8021a73368`가 각각 일치했다. ignored `wasm-pkg-e13595c90/`에 고정했고 API/시각 검증은 진행 중이다. v17 새 HTML 표 저장본 `4a66fd38f7f6...`의 Native full review는 100%다.
- 옛 HWP3 각주 입력 추적: `hwp3-old-note-cursor-trace-v17.log`에서 TAC paint 134.1..157.0px 뒤에 오래된 HWP3 높이 부족 보정이 160.7px로 흐름을 다시 늘리고, 후행 간격·여백 후 170.0px가 됐다. 실제 소유 LS는 1996HU로 표 본체 1716HU와 상하 여백 280HU를 이미 담았다. v18은 저장 LH가 실제 표 높이보다 작을 때만 그 fallback을 적용한다.
- 같은 옛 입력의 빈 문단 34..37은 원래 본문 프레임에 들어가지만 각주 예약 뒤 영역을 넘는다. 기존 tail absorber가 35..37을 숨기고 navigator(38)의 저장 reset을 즉시 적용했다. v18은 유효한 저장 빈 줄의 요구 높이를 실제 fit/이월로 소비하고, 그 빈 줄이 이미 연 다음 쪽에서 같은 reset을 중복 적용하지 않는다. `HeightCursor`도 실제 빈 줄에서 시작한 단 원점의 증거로 저장 reset 뒤 사다리를 재고정한다. 새 source 변화의 Native/시각/대조군 검증을 시작한다.

- e13595c90 fresh WASM: 공개 API 계약 6/6 PASS. 실제 Studio 새 브라우저에서 1/80행 중첩 표의 2단계 cellPath와 guide 활성화/복원 1→0→1, pageCount 1/4를 확인했다(`browser-wrapper-activation-e13595c90.json`). 새 HWP3 각주 입력은 WASM 최저 92.07237%, 새 HTML 표 입력은 WASM 100%로 Native와 같았다.
- v18 첫 build는 새 모듈 `Paragraph` import 누락으로 exit 101이며 수정했다. 재빌드 중 `HeightCursor.col_anchor_y`가 paint에서는 용지 좌표, typeset에서는 단 상대 좌표인 사실을 코드와 대조해 기준점을 중복 차감하는 계산 오류를 확인했다. 검증용 해당 빌드만 종료하고 `y - col_anchor_y`로 고친 뒤 재빌드한다. 잘못된 build를 검증 증거로 사용하지 않는다.

- v18 Native build exit 0(4분 30초), 실행 파일 SHA `ae3b5fc14e8ee09afc5513b16c988b706a234d9bd68d89904deea32ceb263ec2`. 진단 101/101, 기존 표·각주 경계 32/32 PASS(기대값 변경 없음). 원래 HWP3 저장본 `6b4f6a375029...`과 원래 Print의 full Native review가 p1 약 95.47%, p2 92.07237%로 개선됐다. `hwp3-old-pages-v18.json`에서 빈 문단 36/37을 실제 다음 쪽으로 이월하고 navigator(38)가 같은 쪽에서 뒤따르는 것을 확인했다. 두 review PNG에서 참조·표·본문·하단 각주·뒤 쪽 순서를 직접 대조했다. source-code-v18/에 새 모듈을 포함한 source manifest를 고정했다. 이 결과의 fresh WASM과 정식 관계 회귀는 아직 남아 있다.
- e13595c90 fresh WASM filename HF 20개 모두 100%, 크기별 필드 6개 최저 99.7331%, 누락 없음. 20pt 필드 review PNG를 직접 대조해 번호·파일명·밑줄 끝 배치를 확인했다. v16 주석 크기 대조 6개의 Native 최저는 98.42374%다. 이 검증을 v18 이후 source 전체 검증으로 승격하지 않는다.

### 진행 중 보정 12: HYhwpEQ Latin 글리프와 공통 전진폭

- ignored `equation_font_probe--hancom/`의 8/10/20pt, italic 소문자·대문자, rm, 숫자·연산자 독립 한컴2020 Print를 확보했다. 실제 변수는 소문자 E0E5..E0FE/대문자 E000..E019, 숫자는 E03D/E034..E03C, rm은 ASCII 글리프를 쓴다. 일반 Latin Modern 글리프와 다르다. `x+y`의 수정 전 보조값은 83.0303%, 기호 대조는 85.32189%였다.
- HYHWPEQ.TTF의 cmap/hmtx(1024 UPEM)와 Print 전진을 대조했다. 48dpi 논리 크기를 반올림한 뒤 glyph advance를 반올림하는 두 단계가 10pt/20pt 변수 전진과 일치했다. 고정 px 간격 대신 source 글꼴·AST 스타일·글자 크기로 선택·전진을 산출한다. 필요한 글꼴을 기존 review 공급 경로에 실제 제공했고 font hash는 별도 로그에 남겼다.
- 측정 producer `hancom_glyphs::measure → EqLayout::layout_hancom_glyphs`의 선택 글리프/개별 원점을 SVG·Canvas2D·Native Skia·CanvasKit layer JSON이 소비하도록 보정 중이다. 아직 Native/fresh WASM/Skia/CanvasKit 직접 검증 전이며 정식 회귀/fixture/기대값은 추가하지 않았다. glyph mapping이 확인되지 않은 문자와 bold/특수 font style은 기존 경로를 유지한다. 저장 수식 bbox의 최종 가로 scale 및 본문/셀 기준선 소비는 별도로 대조해야 한다.

- `3d2709c07bcf693e8ef7316048cb981f46abd192` fresh WASM build exit 0(9분 22초), WASM SHA `05ec62a17acb9856deb5e7c1f77eff643d986cca509a94cc9ebc51c3e04cf3bf`, root/pkg와 Studio public 일치. 원래 HWP3 저장본/원래 Print 두 쪽 fresh WASM 최저 92.07237%, full review gate passed, 공개 API 6/6 PASS. Native와 같은 문단 소속·본문·각주 두 쪽 출력을 확보했다. 현재 수식 dirty 변경의 검증으로 승격하지 않는다.

- v19 Native build exit 0(4분 14초), binary SHA `4baa6a66c3e9b580d91bbbf14bb63671115931809a61886a3e4bc9f4dfd343dd`, 진단 101/101 PASS, Studio TypeScript는 closure 내부 union narrowing 오류 수정 뒤 exit 0. `source-code-v19/` snapshot을 보존했다. 같은 `x+y` Print full review는 86.70886%로 여전히 실패한다. `inline-xy-v19-zoom.png`에서 마지막 bbox 압축과 operator baseline 차이를 확인했다.
- 독립 한컴 Save한 x+y HWP도 원래 1955x1000 상자를 유지했다. 앞뒤 본문을 추가한 Print 대조에서 다음 글자는 저장 control advance를 소비하고 수식은 자연 글리프 크기로 그려졌다. v20은 독립 검증된 unprotected linear Latin 경로에서 공통 `stored_horizontal_scale`을 SVG/Canvas2D/Skia가 사용하고 CanvasKit JSON에 같은 scaleX를 발행한다. 보호 크기·비선형/mixed 경로를 같은 증거로 확대하지 않는다. Native Skia의 공급된 custom Hancom face도 실제 paint에 전달한다. 현재 검증 중이다.

- v20 Native build exit 0(4분 7초), SHA `e52d51da566fc20ed881b98a55f16caa969707cfec7397a8ea8ae6ff2e658135`, 진단 101/101 PASS, Studio TypeScript exit 0. 같은 실제 HTML 셀 수식 input `433acb46d996...` Native full review 100%. 독립 x+y 8/10/20pt 및 앞뒤 본문 대조도 완료된 다섯 입력은 100%. 전체 대조 batch는 아직 실행 중이며 `inline-xy-v20-zoom.png`에서 operator baseline 잔차를 확인해 완료로 선언하지 않았다.
- 추가 Print: baseline70에서도 operator/variable의 상대 기준선 차이가 유지되고 X+X/단독+/rm 숫자를 대조했다. v21은 Roman face ascent(820/1024)와 formula baseline의 차이를 공통 GlyphRun에 발행한다. 이 수치 근거와 글리프 위치/확대 결과는 별도로 보존한다. 연산자의 pad는 같은 AST row의 실제 앞뒤 피연산자와의 경계에서만 계상하며 단독/연속 연산자에는 내부 이항 간격을 넣지 않는다. 알려진 ×÷≤≥의 Unicode 글리프도 같은 메트릭으로 선택한다.
- v12의 각주 뒤 모든 문단 cache 무효화는 2단 차트 host(문단7)의 유효한 저장 줄·단 소속까지 지우는 부작용이 있다. v18은 원래 저장 줄을 유지한 HWP3 입력에서 이미 Native/fresh WASM 통과했다. v21에서는 뒤 문단 무효화를 제거하고 host reflow/본문 pagination이 예약 프레임의 실제 수용을 계산하도록 되돌린다. 새 API 산출/같은 Print 비교는 아직 검증 전이다. 새 정식 회귀·fixture·기대값은 추가하지 않았다.

- v21 Native build exit 0(4분 7초), binary SHA `13ec4ecd88a34591e41e55f0dfde930e0a74242fde6d785d231b9d198979c806`, 진단 101/101 PASS, TypeScript exit 0, 실제 셀 수식 100%. 연속 연산자 대조는 86.83093%로 남았다. SVG 원문에서 `DIVIDE`를 ÷ 대신 여섯 변수로 출력하는 기존 명령 누락을 확인했다. 같은 script의 Print는 ÷를 출력한다. v22는 기존 DIV/DIVDE와 함께 DIVIDE를 같은 연산자로 해소한다.
- v21 새 각주 API가 만든 차트 입력 `0466.hwpx`의 hash가 원래 `b3c9896ea898...`로 돌아왔다. 원래 저장 줄·단 소속을 보존하며 v12 잘못된 무효화 산출물 `8394cca6978e...`를 해결된 원본으로 바꾸어 보고하지 않는다. 차트 원본의 88.42212% 실패는 여전히 남아 있다.
- v22의 수식 가로 여백: common.margin의 좌/우를 inline advance에 포함하고 같은 여백으로 glyph content box를 얻는다. `composer::collect_tac_controls`/`tac_control_occupied_size → Frame/tac width → paragraph_layout`의 두 실제 분기와 table/table_partial/textbox fallback이 같은 helper를 소비한다. 앞뒤 본문 Print에서 저장 상자와 실제 glyph 원점의 좌측 여백을 대조했다. 수식 상자의 높이·원시 control 소속/내용은 변경하지 않는다. 새로운 검증은 진행 중이며 정식 회귀/fixture/기대값은 추가하지 않았다.

- v22 native-skia 첫 build는 새 `Font::new` 호출에 Option을 전달한 타입 오류로 exit 101이며 실제 Typeface/기존 default 생성 분기로 수정했다. 이후 새 수식 font 수집기의 Footnote/Endnote와 Header/Footer를 같은 타입으로 묶은 오류도 exit 101이며 각각의 실제 모델 타입별 arm으로 수정했다. 실패 build는 검증에 포함하지 않는다. 문서 fontsUsed에는 본문/셀/주석/header/footer/textbox/캡션의 실제 수식 font name도 발행해 공급 단계에서 요청할 수 있게 한다. 실제 브라우저의 공급된 face·Canvas2D/CanvasKit 확인은 아직 남아 있다.

- v22 native-skia 정상 build exit 0(4분 13초), SHA `e787ea69b78f731d61f7e46c773d4e25928ee5f0891d5f28e8dbde236f8504cb`. optional feature 도입 뒤 진단 driver의 serde_json 후보가 build/runtime 둘로 늘어난 문제는 Cargo artifact JSON의 runtime opt-level3와 실제 native library 경로로 해소했고 진단 101/101 PASS. 연속 연산자 대조와 실제 HTML 셀 수식 Native full review 모두 100%. 독립 x+y 및 HTML 셀의 실제 Skia PNG도 산출했다. `inline-xy-skia-v22-zoom.png`에서 hinting으로 굵어진 stem 차이를 확인해 v23은 supplied face의 outline을 subpixel/no-hinting으로 출력한다.
- CanvasKit preflight에도 실제 HancomGlyphs 사용을 기준으로 필요한 HyhwpEQ face를 발행한다. Source 문서에 없는 바깥 face를 임의로 정하는 것이 아니라 수식 컨트롤이 지정한 face다. 실제 face 공급과 브라우저 replay 검증은 다음 검증에 포함한다.

- v23 native-skia build exit 0(4분 16초), binary SHA `2a4cc6ee2aafc09f3532683955a21c170bad65fa20377fed6996a71c652f7354`, 기존 진단 101/101 PASS(5.99초), Studio TypeScript exit 0. `source-code-v23/manifest.json`에 실제 source 24개를 고정했다. `inline-xy-skia-v23-zoom.png`에서 독립 Print와 supplied face의 실제 Skia 출력을 확대 대조했고 v22의 굵은 + stem 차이가 줄었다. 전체 15개 Native 수식 full review는 실행 중이며 fresh WASM/Canvas2D/CanvasKit은 아직 미검증이다. 이 checkpoint는 완료/승인 판정이 아니다.

- v23 수식 대조 15개 Native full review는 최저 94.92834%였으나 Roman20 확대에서 sans fallback을 확인했다. SVG font 수집기가 HancomGlyphs의 지정 face를 Latin Modern Math로 잘못 수집했으므로 v24에서 HyhwpEQ의 실제 face를 임베드하도록 보정했다. 점수 통과만으로 완료하지 않는다.
- d649eebe6 fresh WASM build exit 0(9분28초), SHA `c24cc52118df4713631d797cd41db9bd0ba6a032dfdfde964806bea51f38ad66`, pkg/public 일치. 실제 CanvasKit supplied face는 fallback 0으로 x+y/Roman20을 그렸다. Canvas2D는 원본 TTF의 U+FFFF→65535 매핑 때문에 OTS가 파일을 거부했다. renderer 사본의 잘못된 종료 매핑만 glyph0으로 정정했고 정상 Unicode cmap/hmtx/glyf와 모든 정상 테이블이 그대로임을 확인했다(`font-terminal-repair-contract-v24.json`, 사본 SHA `c00c5b74ee92c66212adc60dd54cb289266bec8b67c90ec9858537579a97e1b4`). 로드 뒤 Studio display chain이 실제 감지된 원 face까지 제외하는 별도 소비 경로를 확인해 보정 중이다.
- 옛 0148/0378 입력을 한컴2020으로 정상 저장한 독립 HWP SHA는 각각 `f1161a36fed5501a3eecadbaca017b53287be7e74eb3ebf3a3d1ae335d1cc00a` / `5573322c52bb768a99f7aceeae2b06f8c976130f8899d7c3ea7b8d9503c461a4`. 전자는 저장 한 줄→두 줄(ts18), 후자는 table1000→1282/hostLH1566→1848/후행vpos+282HU로 정정됐다. 이 근거로 source box 모순을 판정하는 v24 실험을 했으나 구성된 새 줄과 최종 표 소유/저장 cursor의 소비가 갈려 0148이 3.3694%, 0378은 86.31922%로 남았다. 해당 불완전한 cache 실험을 제품 source에서 복원하고 ignored source-code-v24/와 실패 증거를 보존했다. 새 정식 회귀/fixture/기대값은 추가하지 않았다.
- v24 Native native-skia build exit 0(4분12초), binary SHA `46751f62fdc8977978c391419919fb368581e4877ece39ed32c103557fba996b`, 기존 진단101/101 PASS. 이 수치로 옛 cache 결함을 해결했다고 판정하지 않는다. TypeScript와 기존 SFNT 검사는 통과했고, font chain 보정 뒤 실제 브라우저 재검증을 계속한다.

- v24 SVG 임베딩 보정 뒤 Roman20은 실제 HyhwpEQ face를 포함하고 같은 원본 Print와 Native full review 100%다. `roman20-font-supply-v25-zoom.png`에서 serif 형상을 직접 확인했다. Canvas2D의 뒤 소비 문제는 감지된 실제 원 face가 `fontFamilyCandidatesForDisplay`의 originalAllowed에서 누락된 것이었다. v25는 현재 호출의 installedFaceName 결과를 표시 체인에 포함하며, 명시적 빈 confirmedLocalFonts는 이전 감지를 사용하지 않는다. 새 유한 대체 mapping이나 font-rule registry/projection은 추가하지 않았다. 기존 font-substitution/SFNT/host-canvas 검사20/20 및 TypeScript PASS. 실제 브라우저 Canvas2D/CanvasKit에서 supplied/detected HyhwpEQ의 Roman20 serif 형상을 직접 확인했고 원본 TTF는 Git에 추가하지 않았다. Native v25 exact build와 최종 fresh WASM은 아직 진행 중이다.

- v25 Native build exit0(4분11초), SHA `1d08597b7707a2e5d9d7fc072b7cfda1e1de01cb71005fc026e68883a157ea10`. 같은 15개 수식 TSV 최저98.85057%/누락0, 실제 HTML 셀 full review100%. `5b46702643ed84326c3405f8a9cafc3ec7ef5b36` fresh WASM exit0(9분29초), SHA `d7f97245c2cffcf3731fe401c74ed329a72f68617c0c09dec5dff61cfc58c861`, root/pkg/public 일치. 이 head의 대문자는 확대 판독상 아직 미충족이었다.
- 전체 수식 확대(`equation-native-font-supply-v25-zoom-{1,2}.png`)에서 PUA 대문자만 정체인 것을 검출했다. 동일 Print의 text matrix는 대문자 `1 0 0.333 -1`이고 소문자/rm은 `1 0 0 -1`이다. Print subset과 설치 TTF의 대문자 glyph bounds/hmtx가 같아 glyph 교체 문제가 아니었다(`equation-font-independent-shear-v26.json`). v26은 대문자 italic의 shear를 GlyphRun에서 생산하고 SVG/Canvas2D/Skia/CanvasKit JSON이 같은 변환을 소비한다. 전진폭은 바꾸지 않는다.
- v26 Native build exit0(4분16초), SHA `05ed2e085652a764f8e315f8c58bdbd7bfc15c66f45042b00ca5ddacbc69df25`, 기존 진단101/101 PASS(5.43초), TypeScript exit0. 대문자26자 및 X+X full review 둘 다100%; `uppercase-shear-v26-zoom.png`에서 Print/SVG/실제Skia의 기울기를 직접 대조했다. build 로그 이름에 tac-cache가 있지만 해당 load-cache draft는 적용되지 않았고 제품 변경은 수식 shear뿐이다. 이 source의 fresh WASM은 아직 남아 있다.
- 실제 셀 브라우저 replay는 공급된 HyhwpEQ와 실제 HCR Batang face를 등록해 Canvas2D/CanvasKit에서 수행했고 CanvasKit 미등록 fallback0이다(`browser-equation-fonts-html-cell-v25-hcr.json`). HCR Batang 실제 설치본 SHA `8664669bd2d6a4fe8b7538706d2bb2a1004c074aa02dca59051e515e731294f5`를 기존 review 폰트 경로의 HANBatang.ttf로 제공했다. 이전 HBATANG.ttf(Haansoft Batang) 공급과 구분한다. 이후 영향되는 본문 비교는 이 실제 face로 재실행한다.


- v27 load producer의 첫 draft는 HWPX Table의 정상 `raw_ctrl_data.len()==0`을 제외하는 잘못된 조건 때문에 원본 둘을 보정하지 못했다. 원시 CTRL byte 존재는 캐시 수용 조건이 아니므로 제거했다. 공용 HTML 높이 producer도 `Table::sync_ctrl_height`로 common.height와 존재하는 원시 CTRL height를 함께 갱신한다. XML 입력에서 raw vector 범위 접근을 하지 않는다.
- v28은 `DocumentCore::from_bytes`에서 최초 compose/pagination 전에 source box가 한 저장 행과 모순되는 지원 TAC host를 정규화한다. 원래 paragraph_flow_end를 먼저 잡고 공용 HeightMeasurer/reserve_content_heights → reflow_line_segs → recalculate_section_vpos를 적용한다. 폰트 대체 폭을 캐시 무효 판정 근거로 사용하지 않는다. 지원하지 않는 여러 행/불균등 단/복잡한 셀은 추정하지 않는다. 실제 측정과 배치는 발행된 동일 모델을 소비한다.
- v28 Native native-skia build exit0, binary SHA `9230c317256180953b6a0fb42f4599b5311de98ec52f8d6b642c868334a2e9f7`; source-code-v28/manifest.json에 변경 source를 고정했다. 기존 진단101/101 PASS(5.67초). 실패했던 **원래 0148/0378 저장본과 원래 Print** full review 둘 다100%/passed다. `old-boundaries-v28-zoom.png`에서 표 외곽/본문/셀 글자/뒤 문단의 포함·순서를 직접 대조했다. 한컴 정상 Save 대조군으로 원본을 치환하지 않았다(정상 Save 0378의 별도 native 진단은86.04899%였음).
- 공개 loader 진단 `cache-load-contract-v28.json`은 다운로드된 독립 Print에 대응하는 저장본176개에서 본문 줄 캐시 변경8문단/표 host5개를 기록했다. 모든 변경 문단의 text와 char_offsets는 유지됐다. 추가3개 HTML/중첩표 원래 입력과 정상 대조군도 다시 시각 검증 중이다. 아직 fresh WASM/최종 전체 gate/새 정식 회귀는 완료하지 않았다.


- c62b44012의 root wrapper fresh WASM은 Rust9분36초 + wasm-opt를 포함한 총18분01초/exit0이다. WASM SHA `4e06c8c8f13433d092f418c011e307cc7e6cf2ce6b4e1cdc6ab1e50b5808e4d6`, glue SHA `7462392b9b4dadb36c0b60cc0afa9de3fc6e41a861b150010ea51f0a7974737a`, pkg/public bytes 일치. 후속 v29 dirty Rust는 이 컴파일에 포함되지 않았음을 verification-manifest.json에 명시했다.
- 원래0148/0378과 원래Print의 c62b44012 fresh WASM full review 모두100%/passed다. Native 정상 대조 누름틀25건29쪽 최저100%, 선두제어8건10쪽 최저98.15341%/누락0. scores만 산출한 대조군은 별도 직접 판독/최종 gate를 대체하지 않는다.
- `equation-browser-c62b44012-zoom.png`에서 독립 Print와 실제 Canvas2D/CanvasKit 대문자26자 slant 및 Roman20 serif를 확대 대조했다. `html-cell-browser-c62b44012-zoom.png`에서 동일 셀 수식/본문/두 줄 붙이기/셀 뒤 본문의 순서·간격을 대조했다. 세 browser replay는 실제 HyhwpEQ/HCR Batang 제공, face loaded, CK 미등록 fallback0/unsupported0/runtime gate 통과다. 전체 수식 TSV와 대표 WASM full review는 진행 중이다.
- 추가 old 좁은 중첩표 `wasm-narrow-nested-3f4591b1708c`는 v28에서28.91566%로 실패했고 같은 입력 v25도28.91566%여서 신규 회귀로 세지 않는다. 원본도 보정 대상으로 유지한다. 독립 한컴 정상 Save HWP SHA `02f7abe9121c467e08592b36974556a7f871f42956b587529a0f74276e9d0c12`는 첫 셀의 바깥을 sw1440 두 줄/중첩표 원점3200으로 저장했고 같은 원래Print와 Native100%다. 이 대조군 통과를 원본 통과로 바꾸어 보고하지 않는다.
- v29는 거부된 TAC host 안에서 현재 셀의 공용 padding/최소폭/ParagraphBox와 모순되는 한 저장 행의 폭 캐시를 공용 reflow_line_segs로 다시 발행하는 draft다. 중첩 표는 아래부터 공용 높이를 예약하고 기존 셀 편집의 recalculate_cell_paragraph_vpos를 공유한다. size_protect/세로쓰기/다줄·carve·indent의 별도 box는 이 단일행 판정에서 추정하지 않는다. ZERO WIDTH 문자만 있는 host를 실제 가로 점유로 오판하지 않도록 UnicodeWidth의 0폭을 제외한다. Native build 진행 중/아직 검증 전이다. 새 정식 회귀·fixture·기대값은 추가하지 않았다.
- 차트 표현은 기존 bounded reader 진단만 이어갔다. [한컴 공식 차트 형식 revision1.2](https://cdn.hancom.com/link/docs/%ED%95%9C%EA%B8%80%EB%AC%B8%EC%84%9C%ED%8C%8C%EC%9D%BC%ED%98%95%EC%8B%9D_%EC%B0%A8%ED%8A%B8_revision1.2.pdf)의 ChartType/축 범위/font/Fill/Location 정의와 reader가 확인한 source metadata를 연결했다. 진단 PDF/hash와 offset/field 기록은 chart-input-inspection/에 보존했다. 폰트가 고정된 resize/legend font 진단은 임의의 동일 비율 확대와 point10 맞춤 상수가 정당하지 않음을 보였다. 새 chart engine·제품 parser 확장은 적용하지 않았으며 원래 차트88.42212%의 실패는 남아 있다.


- c62b44012 fresh WASM 수식15개 TSV는 최저100%/90%미만0/누락0이며 `not_evaluated`와 full review를 구분한다. 별도 대표 대문자26자/Roman20/실제HTML셀 WASM full review 3건도 각각100%/passed다.
- v29 Native native-skia build exit0(4분35초), SHA `b1630662dc898d79477ca9275bbc738d3a5fc780455063a31656a7870d330c3f`, 기존 진단101/101 PASS(5.38초). **원래 좁은 중첩표/원래Print** full review는28.91566→100%/passed다. `narrow-nested-original-v29-zoom.png`에서 바깥의 두 줄/옆 셀/중첩 시작 원점을 확대 대조했다. 중첩 안의 일부 잘림은 독립 원래Print에도 있으며 원래 문서의 잘림을 해결했다고 주장하지 않는다. 새 HTML import producer의 너비 예약 검증과 구분한다.
- 추가 영향 원래0148/0378/old HTML narrow/old HTML equation/current HTML equation의 v29 Native full review는 각각100/100/100/96.99223/100%로 통과했다. 복수 행/float carve의 정상 저장 box를 이 비교로 재조판하지 않도록 v30에서 셀 내부 floating control과 non-TAC nested table은 이 single-row width 비교의 적용 경로에서 제외했다. float의 배제 프레임은 별도 근거가 필요하다. v30 Native build 검증 중이며 v29 결과를 v30 결과로 승격하지 않는다. 새 정식 회귀/fixture/기대값은 아직 보류다.

- v30 Native native-skia build exit0(4분12초), SHA `07e9710edf3dad361c14d31786953cc45c8444803d197269afe0654cfcf49b0a`, 기존 진단101/101 PASS(5.90초). 원래0148/0378/좁은중첩표 full review 모두100%/passed다. source-code-v30/manifest.json과 native-cache-originals-review-v30/에 고정했다. 이 checkpoint의 fresh WASM은 root wrapper `--no-opt`로 진행한다: Rust release 최적화 빌드는 유지하며, 앞 checkpoint에서8분 이상 추가된 선택 wasm-opt 단계는 렌더 계약 확인 범위에 필요하지 않다. 완료 전에 새 결과로 보고하지 않는다.


- df28acd633d4834da4489fbbc3264a187f860eed fresh WASM `--no-opt` build exit0(9분26초), SHA `a67796e8b104cf36971612c72096fdd9c0e13cb44c1c2ce694d764f03ac643c2`, glue SHA `7462392b9b4dadb36c0b60cc0afa9de3fc6e41a861b150010ea51f0a7974737a`, pkg/public 일치. 원래 좁은 중첩표/원래Print WASM full review도100%/passed다.
- 정상 한컴 Save leading HWP의 별도86.04899% 차이를 source→measurement→placement로 추적했다: 유효 hostLS1848/ctrl.height1282/child cellSz1000/childLS1000+padding282에서 HeightMeasurer의 required row는1282HU였으나 `resolve_row_heights_with_common_fit → trust_declared_row_heights`가 `every_row_is_declared_plus_cell_padding`만으로1000HU로 덮어썼다. Table bbox13.3px와 뒤 문단 원점이 실제 측정에서3.76px 줄었다. source 캐시 자체를 무효화할 근거가 아니었다.
- v31은 별도의 object box가 측정 행합을 HU 단위로 정확히 확인하는데 cellSz 행합과 다를 때, padding 중복이라고 추정해 paint만 줄이는 교정을 적용하지 않는다. 임의의 문서ID/point 상수/px 위치를 추가하지 않았다. 기존 issue1663의 `cellSz + padding` 보정은 object box가 선언 행합을 확인하는 대조 경로에서 유지한다.
- v31 Native native-skia build exit0(4분12초), SHA `e6133a90b87ab00a3268af6b4a8301f148ec4c871af6b39e059028c9b5b5e179`, 기존 진단101/101 PASS(6.00초). 원래0148/0378/좁은중첩표 full review 모두100%, 유효 한컴 Saved leading 대조군86.04899→100%/passed, 원래HWP3 두 쪽은 최저92.07237%/passed다. source-code-v31/와 native-measured-row-boundaries-review-v31/에 실제 source/출력을 고정했다. 이 측정 소비 보정의 fresh WASM과 정상 issue1663 반례는 다음 검증 범위다.
- 원격 목록 재확인에서 초기18개 head SHA는 유지됐고 새 비draft #7661/#7662/#7663이 추가돼 현재21개다. 신규3개는 아직 체리픽하지 않았으며 이번 통합 포함 여부를 별도로 확인 중이다. 초기18개 보정/검증은 계속한다.


- e38f6156f5f7cc86e55d16b6d6fe55a60e9e6ea0 fresh WASM `--no-opt` build exit0(9분23초), SHA `4ffb444ef670ab837a0ba990e8f8b05a8b22d329600fd61b73622edd496070de`, glue SHA `7462392b9b4dadb36c0b60cc0afa9de3fc6e41a861b150010ea51f0a7974737a`, pkg/public 일치. 원래0148/0378/좁은중첩표 및 정상Saved-leading full review 모두100%, 원래HWP3 두 쪽 최저92.07237%/passed다.
- 기존 issue1663 대조군은 Native v30/v31 모두 p1 87.31118%/p2 80.8944%다. 앞뒤 실제 PNG SHA가 두 쪽 모두 같고21행 경계도 동일하다. 새 guard 때문에 생긴 회귀라고 세지 않는다. 이 baseline의 미통과를 숨기거나 현재 전체 검증 통과로 보고하지 않는다. `coanchor-independent-v31-p2.png`에서 같은 행 수와 경계를 확인했고 검증 범위를 확대할 때 별도 기존 출력 차이를 확인해야 한다. 새 정식 회귀/fixture 추가는 아직 보류다.
- 전체 nextest를 위해 manifest --prepare 완료(1504 sources/28 harnesses+20 exceptions/48 targets). 최초 build에서 `tests/cases/equation_command_goldens.rs::dump_layout`가 새 LayoutKind::HancomGlyphs를 처리하지 못하는 exhaustive match 오류가 났다. 일반 golden 경로는 EqLayout::new로 Hancom 전용 노드를 만들지 않으므로 baseline을 갱신할 이유가 없다. 보조 dump arm을 보완한 뒤 이 검사와 전체 nextest를 재실행한다. 실행 전 compile failure이며 회귀 test FAIL 재현으로 세지 않는다.
- 기존 bounded-reader Plot136 payload의 후보float source field0/field1 및 tail120/124를 수동 변경한 한컴 Print 진단에서 현재4개 막대의 좌표/폭은 모두 그대로였다. 앞의1.0 또는 뒤0.5 값을 xGap이라고 추정해 제품의 막대 폭을 정하지 않는다. 각 source/PDF hash와 정상zip 검사/수동 변경 provenance는 chart_gap_probe--hancom/, chart_plot_tail_probe--hancom/ 및 독립 관측 JSON에 남겼다. 제품 차트 renderer는 아직 바꾸지 않았다.

### 진행 중 보정 22: 전체 검증에서 확인한 회귀 원인

- 전체 nextest 최종 exit100: 10605 실행/10580 PASS/25 FAIL/50 SKIP, 실행957.029초. 완료 로그와 실패 원문은 `logs/nextest-hancom-helper-retry-v32.log`, 목록은 `nextest-final-summary-v32.json`에 보존했다. 파생 manifest/suite와 기존 baseline은 변경하지 않았다.
- 정책 문서의 실제 CLI 조판은 f58718f70/v8에서215쪽, 실제 각주 예약 보정 v10 이후 v31에서216쪽이다(`policy-pagecount-stage-comparison-v32.json`). p67의 각주 소속과 표의 이월이 달라지는 실측 회귀를 확인했다. 측정 완료 상태와 본문 하단 앵커 선택을 하나의 bool로 표현한 경로를 분리하여 대조 중이며 아직 통과로 보고하지 않는다.
- 차트 범례font10→10.001/9.999의 독립Print는 표시 글꼴 크기9.96pt를 유지하면서 원래2행/22.70px marker에서1행/15.18px marker로 바뀌었다(`chart-font-cache-independent-observations-v32.json`). VtTextBlock::SetFont/LegendSize의 실제 호출도 대조했다. 원 source 캐시 재사용과 변경 뒤 재측정이 다르다는 증거이며 point10 예외로 구현하지 않는다.
- Contents header의 extent/DPI 후보만 바꾼 독립Print는 원래 막대·범례 좌표와 같았다(`chart-header-independent-observations-v32.json`). 이 필드에서 표시 배율을 추정하는 가정을 제품에 추가하지 않는다. VtFont14byte는 f32 Size+u16 Style+u16 Effects+inline VtColor(v1/u32 color)임을 설치 라이브러리 ReadFrom과 공식 속성 정의로 확인했으며 종전의 color/여섯 flag 추정과 구분한다.

- v33의 앵커/측정 플래그 분리만으로 정책 문서는 여전히216쪽이며, 관련 nextest7건 중1PASS/6FAIL(exit100)이다. `paragraph::place_split_paragraph`가 진입 fit과 달리 앵커 플래그로 다시 추정용 안전 여백을 차감하는 소비 지점을 확인해 v34에서 측정 완료 플래그를 사용하도록 연결했다. Native build 진행 중이며 아직 개선으로 판정하지 않는다.
- 원래 표 캐시3건의 같은 관계 검사를 보정 전/후 고정 출력에 실행했다(`stale-tac-relational-before-after-v31.json`). 전부 수정 전FAIL/후PASS: full-width 표가 본문과 같은 물리 줄에 놓임, 셀 글줄이 소유 셀 하단을 넘음, 좁은 셀의 본문 줄 나눔이 누락됨을 각각 검출했다. 수동 draft의 빈 TextRun을 내용 글줄로 세던 진단 오류는 고쳤고 실제 내용·줄·중첩 표의 순서를 검사한다. 새 정식 검사 추가는 아직 하지 않았다.
- 차트 캐시 높이만341→227로 바꾼 독립Print 두 진단이 완료됐다. 처음 저장LRect만 바꾼 경우와 textExtent/후행LRect도 바꾼 경우 모두 marker22.70→15.18px/범례2행→1행/plotBottom325.244→366.799px가 됐다. 실제 글자 기준선은 두 진단 사이3.84px 차이여서, 첫 상자와 최종 글자 원점을 같은 값으로 대체할 수 없다. 입력/PDF SHA 및 좌표는 `chart-cached-box-independent-observations-v33.json`에 고정했다. 수동 진단을 정상 fixture로 분류하지 않는다.

- 사용자 선택에 따라 신규 #7661/#7662/#7663은 초기18개 통합 뒤 별도 검토한다. 이번 누적 후보에 추가 체리픽하지 않는다.
- v34의 분할 예산 연결로도 정책 문서는216쪽이었다. 앵커/측정 플래그 분리 시도는 제거했다. v35는 Native 저장 HWP5의 편집하지 않은 각주 본문이 composer에서 원래 LineSeg를 그대로 소비할 때 기존 source page-cut/continuation owner 계약을 보존한다. 누락·합성·편집 줄은 실제 FootnoteArea 재측정 경로를 유지한다. 저장정보 수용 조건 자체를 완화하거나 기대값을 변경하지 않았다.
- v35 Native build exit0(4분16초), binary SHA `a565df8a0ff6e7e3ab1de255417069f3816a35055b46ea9a0aad3628a503c432`. 실제 정책 문서는215쪽으로 복원됐고 기존 각주·표 이월7검사 모두PASS(exit0,1.744초)다. API 진단101/101도PASS(7.27초). 진단 드라이버의 최초Serde linkage 오류는 default/native feature별 산출물 혼용이었으며 default library의 실제 Cargo artifact JSON을 다시 확보해 같은 feature의 의존성으로 연결했다. 제품 검사 FAIL로 세지 않는다.
- 원래 HWP3 각주0462/원래Print의 v35 Native full review는 두 쪽 최저92.07237%/passed다. 새 각주 보정이 유지되는 대조이며 freshWASM은 별도다. 정책 문서215쪽 전수 보조값은 `native-policy-stored-note-cuts-scores-v35/`에서 진행 중이고 쪽수 복원만으로 시각 통과를 선언하지 않는다.
- Title 저장LRect 높이/Section external·internal margin만 각각 바꾼 세 한컴 Print 진단에서 차트 title/axis/bar/legend 좌표는 그대로였다(`chart-section-box-independent-observations-v35.json`). 범례 cache 높이가 실제 배치에 적용된 앞 진단과 다른 결과다. Title 저장 상자를 그대로 재사용하거나 이 margin 필드로 현재 출력 위치를 추정하는 가정은 보류한다.


### 진행 중 보정 23: 저장 각주 복구의 fresh WASM과 남은 회귀

- `bd42c480cbc30a520755487e3564df1ced3e122a` fresh WASM wrapper `--no-opt` build exit0(10분06초), WASM SHA `cbda0b27e7dde0bb6c65202fd34584e8ce1bb975893e8c1ba62a0807d1d4d9bc`, JS SHA `7462392b9b4dadb36c0b60cc0afa9de3fc6e41a861b150010ea51f0a7974737a`. `wasm-build-bd42c480c.json`에서 root pkg/Studio public 일치를 확인했다. 공개 API 6/6 PASS. 원래 HWP3 0462/원래Print 두 쪽 fresh WASM full review 최저92.07237%/passed이고 p2 review PNG에서 navigator/screen 본문과 footer를 직접 대조했다.
- v35 정책 문서 Native 전215쪽 보조값 산출 완료: 쪽수와 누락 없음이 확인됐지만 p44 74.4937%, p158 74.72316%, p168 82.93414%, p176 89.0736%, p184 71.10305%, p208 86.19535%, p211 81.66205%로 보류다. 변환 오류가 아니며 전수 gate 통과로 보고하지 않는다. `policy-v35-p44/158/184-compare.png` 직접 판독에서 표·캡션 위치 및 후속 본문/줄바꿈 차이를 확인했다. fresh WASM 전215쪽 보조값도 별도로 진행한다.
- 남은 기존 검사24건 재실행 exit100(6.419초): 6PASS/18FAIL. issue6718의 같은 실제 issue5966 원본/143쪽/본문 하한 검사가 PASS로 복구됐다. 앞서 복구한 6건과 함께 전체25FAIL 중7건이 해결됐으며, 새 전체 nextest 통과를 선언하지 않는다. 목록/원문은 `logs/nextest-remaining-regressions-v35.log`에 보존했다.
- bounded chart reader의 ignored metadata 초안은 원래 Contents 전체9876바이트를 경계·객체 참조에 따라 끝까지 읽어 type1, 3series, 원래 제목/범례 font·CP949 문자열·두 캐시 상자를 회수했다. 숨긴 Z축의 null text 객체는 빈 문자열 객체와 구별해 허용했다. 제품 parser/renderer는 아직 변경하지 않았다.
- 차트 title wordWrap0→1을 바꾸면 title 저장 firstLRect가 실제 출력에 적용됐다. 같은 wordWrap1에서 height391→541은 title baseline132.72→136.44pt 및 plot top을 이동시켰고 legend/plot bottom은 그대로였다(`chart-title-wrap-independent-observations-v36.json`). 설치본의 `VtChartText::Size → SetWordWrap(1) → VtTextBlock::SetWordWrap`가 바뀐 속성에서 캐시를 무효화하는 호출과 일치한다. title 상자가 무조건 무시된다는 앞 가정을 수정한다. 수동 진단 입력은 정상 저장 fixture로 분류하지 않는다.
- issue5598의 두 기존 합성 입력은 원래 page width/height가0/0이며 한컴 Print가 용지 범위 오류로 거부했다. 원본과 실패 job을 보존했고 용지만 A4로 지정한 ignored 대조군 두 개와 실제 원본2995759(Downloads)의 Print를 확보했다. 실제 원본 SHA `1ad6ec8dc787b1e1bda60cfca63890d4d83190065945bb4851633a09f775f9ba`, Print SHA `2b7c4d5d71a79d35e1cdf191857ac93cbfd828d1e2e80413698801921330ec33`. 대조군 통과를 원래0/0 용지 입력의 한컴 일치로 바꾸어 보고하지 않는다.
- v36은 Frame에서 재조판된 projected row도 TAG_IMPLEMENTATION_PROPERTY/soft-wrap 말미 separator 계약으로 판정한다. physical_frame_rows라는 배치 경로를 이유로 separator의 전진을 빼지 않는다. 내부 공백이 없는 이 줄의 char-spacing도 보존 separator 슬롯을 포함해 나누며 다른 줄은 기존 N-1 글리프 경계를 유지한다. 원본5598 검사에서 양쪽 모두 수정 전 bbox 초과3.4px가 검출됐고 변경 뒤 검증 중이다. 새 정식 검사·fixture·기대값·baseline 변경은 없다.

- v36 기존 검사4/4 PASS(exit0,0.076초): 좁은 셀의 내어쓰기/무내어쓰기 두 경로와 issue4956 폭 변경, 음수 자간 마지막 글리프 대조다. header 자체를 수정할 필요가 없었으며 원래 본문의 tab+soft-wrap 말미 공백이 중복 점유하던 같은 원인으로 설명된다. `logs/nextest-projected-wrap-spacing-v36.log`에 수정 전/후 기존 기대값 유지 증거를 연결한다. 새 Native/fresh WASM 시각 확인은 아직 남아 있다.
- 사용자 추가 지시(2026-10-08): 시험지 관련 회귀 실패가 있으므로 관련 회귀 테스트를 모두 제거한다. 시험지 보정은 다른 작업에서 진행 중이다. exam_* 및 3-월별 교육/실전 통합, 시험 해설 발췌 eq-002 입력의 테스트/공유 코퍼스 항목을 AST·입력 연결로 확인하며 제거할 예정이다. 문서 입력·독립 PDF 및 이전 실패 증거는 보존한다. 이 지시는 일반 문서의 실패 기준값 완화나 검증 통과 선언의 근거가 아니다.

### 시험지 회귀 검사 이관에 따른 제거 (2026-10-08)

- 사용자 승인 범위에 따라 시험지 원본 또는 시험지 생성에 연결된 기존 검사 293개를 제거했다(1차268개+잔여25개). 시험지 전용 검사 파일53개를 삭제하고 혼합 파일의 일반 문서 검사는 유지했다. `exam-regression-removal-applied{,-residual}.json`에 소스·검사명·전후 SHA를 보존했다.
- 공유 코퍼스13개 수집 경로에서 `tests/fixtures/transferred_exam_regressions.txt`의 정확한52개 시험지 입력을 제외했다. `tests/common/corpus_scope.rs`는 그 목록만 적용한다. 이에 연결된 baseline221행만 제거했으며 남은 행의 값·허용치는 원래 값 그대로다. `exam-corpus-removal-applied.json`에 수집 경로와6개 baseline의 제거 행 수를 기록했다. module path를 사용하는14개 integration source의 suite-policy 예외를 등록했으며, 파생 suite/manifest는 제출 대상이 아니다.
- production AST 감사에서8개 혼합 Rust source의 실행 구현은 기존7d8f766455c8913a084c103dd7ace0cf8b314f6b와 같았다(`exam-removal-production-ast-audit.json`). 빈 시험지 builder cfg(test) module도 제거했다. 소스 HWP/HWPX/HML와 독립 한컴 PDF는 그대로 보존했다.
- 보조 helper 정리 때 implicit format capture `{BASE_SAMPLE}`를 사용하지 않는 상수로 잘못 판정한 컴파일 오류4개가 발생했다. 원래 상수를 복원하고 같은 검증을 다시 실행했다. 최종 `cargo check --locked -p rhwp --tests --target-dir target/pr-review` exit0/35.98초, `cargo fmt --all`, `git diff --check`, manifest/base 비교 및 unit tiers/base 비교 PASS. manifest는1451sources/6244staticattrs/48integrationtargets, unit tiers는4113tests/297modules다. 정책 base는60efc4b7b380b2a183c20c3ee0ab24a60bc2d832다. 최종 로그는 `logs/check-exam-removal-final.log`에 있다. 전체 nextest 및 push 직전 lint 묶음은 아직 남아 있으며 이 검증을 전체 통과로 보고하지 않는다.
- 사용자가 요청한 수식 포함 파일 목록은 실제 파싱된 Equation control을 본문·중첩 표·도형 글상자·각주/미주·머리말/꼬리말·바탕쪽에서 재귀 순회해 확인했다. 제외52개 중 HWP/HWPX36개, 별도 HML fixture1개에 수식이 있다. `removed-exam-files-containing-equations.tsv`에는 정확한 파일 경로와 수식 개수를, `.json`에는 입력 SHA-256을 기록했다. 파싱 실패0건이며 HWPX14개는 XML 수식 개수와도 모두 일치했다(`removed-exam-hwpx-equation-xml-crosscheck.json`). `removed-equation-regression-source-mapping.tsv`에는 해당 입력과 연결된 삭제 검사174개/소스42개의 대응을 기록했다. 목록은 회귀 검사의 이관 범위를 뜻하며 수식 출력 보정 완료의 근거가 아니다.

### 제거 후 일반 문서 재검증과 셀 선두 캐시 보정 (v37/v38)

- 52cc8ee94의 일반 문서 text-overlap/off-canvas 전체16partition씩 및5593 기존 검사를 실행했다. 시험지 제외로 크기 기반 partition 배정이 바뀌므로 이전 실패 partition만 선택하지 않았다. exit100, build8분33초,33검사/28PASS/5FAIL/203.696초(`logs/nextest-general-remaining-v37.log`). 남은 실패는5593, text-overlap partition1/7/12, off-canvas partition12다. 신규 겹침 입력은 basic/issue1994_behindtext_table_20200830.hwp(1), issue2217/20200830.hwp(1), task1716/table_scattered_header_rowbreak.hwpx(2), table_scattered_header_rowbreak.hwp(2), 페이지 밖은 basic/shortcut.hwp(6)다. 일반 문서 기대값·허용치는 유지했다.
- 원격 refresh에서 #7615 head가 ed9c2847523c18f658344ec89cb7ed1a426807ae →4a6c27c5ffa54e4988a7e0e7956115d933725aca로 바뀐 것을 확인했다. 새 head에는 후속 #7662의377bcb71e7404a741105a28303a3fae8b671fc9d 및 그 선행 검사·갭 예약 제거가 포함된다(`pr7615-refresh-v37.json`). 사용자 지시에 따라 초기18개 검증은 처음 고정한7615 SHA를 유지하고 이 추가 변경은 #7661–#7663 후속 검토와 연결한다. 최신7615 head 전체를 이미 검증했다고 보고하거나 조기 종료하지 않는다.
- 5593 원본 SHA3973715acdb0…를 Hancom2020 PrintMethod0으로 출력했다. 독립 PDF SHA e79fe1a94d53a2843e717cf134e0fce2367b29c92bce3a73528cecb8a69a1af3 /1쪽. 선언6400HU 셀은 Print에서 약85.2px 높이를 유지하고 개체는 가운데에 있다. v35 Native는 셀92.2px/개체y100.1px로 불필요하게 성장하고 뒤 문단도7px 내려간다. Native 보조점수94.15%인데 직접 review PNG에서 차이가 보여 완료로 판정하지 않았다(`native-5593-before-v37/cell-front-object/review/review_001.png`).
- 동일 원본을 한컴에서 재저장한 정상 HWP 대조군 SHA3335ccea0197135d4de35b50d895a8e01af0a31724c7a40af9deaf4b717f5ba2를 확인했다. 선언 셀/개체 높이6400/5830HU는 그대로이며 셀 첫 저장 줄 vpos800→0, 부모 표 줄높이3560→6966HU로 정정됐다. 한컴이 만든 실제 대조군과 수동 진단 입력을 구분한다. 원본은 수정하지 않았다(`hancom-save-5593-v37.json`, `hancom-normalized/5593-hancom-export.hwpx`).
- 정렬 속성만 Top/Bottom으로 바꾼 수동 진단 입력의 독립 Print에서도 선언 셀 높이가 유지됐다. 개체 top은 Top74.796875pt/Center76.238281pt/Bottom77.675781pt다(`5593-independent-alignments-v37.json`). 선두 저장 vpos만0으로 바꾼 별도 진단은 Native 셀85.3px, 개체Top100.1/Center102.0/Bottom104.0px를 보였지만 이를 원본 수정 완료로 바꾸어 보고하지 않는다(`5593-zero-initial-line-probe-v37.json`).
- v38은 저장 조판 정보 수용을 완화하지 않는다. 이미 물리적으로 거부한 부모 TAC 행의 셀에서, line grid/HWP3/흐름 개체가 없는 첫 문단의 저장 시작 띠가 문단 위 간격+저장 줄 내부 leading보다 큰 경우 캐시를 거부하고 공용 cell reflow로 새 LineSeg와 후속 사다리를 만든다. foreground/background 개체는 줄 흐름을 전진시키지 않는다. 실제 저장 위 간격·내부 leading·grid·flow 개체가 있는 경계는 보존한다. HU 정수 비교로 동등한 간격의 소수 오차를 피한다. `tac_cache_repair::repair_cell_width_caches → reflow_line_segs/recalculate_cell_paragraph_vpos → reserve_content_heights → parent reflow`의 생산 결과를 측정과 배치가 소비한다. source에 시험지 ID·절대 화면 좌표·특정 높이 예외는 넣지 않았다. compile check exit0/1분43초이며 기존 검사와 Native/fresh WASM 시각 확인은 이어서 진행 중이다. 새 정식 회귀·fixture·baseline 변경은 없다.
- v38 기존5593/5598 검사3개는2PASS/1FAIL(exit100, build8분27초,0.084초)이며5593은 여전히 같은 y100.1px로 실패했다(`logs/nextest-initial-cell-band-v38.log`). 실제 호출을 추가 추적해 `reflow_line_segs_impl`이 원래 첫 LineSeg.vertical_pos를 새 ParagraphBox frame에 보존하고, `recalculate_cell_paragraph_vpos → apply_cell_vpos_ladder`도 그 원점을 재사용하는 것을 확인했다. v39는 거부된 셀 사다리를 먼저 clear한 뒤 canonical reflow를 호출한다. 이는 최종 좌표를 clamp하거나 임의로 vpos를 덮는 보정이 아니다. 원본 캐시의 판정·공용 재구성·측정/배치 소비를 잇는다. 같은3검사 재실행 중이며 이전 실패를 통과로 보고하지 않는다.
- shortcut 원시 진단의6건은 모델 공백 바탕쪽이 아니라 실제 본문 메뉴 제목 런이다. `파일/편집/보기/입력/모양`은 bbox1812.36px/extra_char_spacing892.17px, `창기타`는1359.27px/439.08px다(`shortcut-run-details-v37.log`). 해당 문단의 저장 첫 줄은 제목, 다음 줄은 별도 TAC 표를 소유한다(ColumnDef 앞 슬롯8, 제목 뒤 Table 슬롯 시작10 또는11). 단순히 bbox만 줄이거나 진단 대상에서 빼지 않는다. 원본 Justify의 제목 줄 종료/표 줄 참여 계약을 독립 PDF와 대조하며 보정할 예정이다. 기존 pr_7382_review.md 보정293–297의 동일 입력 전체7쪽 최저96.12686% 결과와 현재 후보를 구분한다. 새 후보의 해당 검증은 아직 없다.
- v39 기존5593 및5598 양쪽 내어쓰기 대조는3/3 PASS(exit0,0.088초, `logs/nextest-initial-cell-band-v39.log`). 5593의 기대값·입력은 그대로 유지했으며 이전v37/v38 실패와 연결된다. 현재 코드에서 거부 캐시의 원점을 실제로 제거한 뒤 공용 재조판 사다리가 높이와 배치를 함께 공급한다. Native/fresh WASM 출력 검증은 다음 게이트로 남아 있다.
- shortcut의6개 parent 제목은 font color16777215(흰색)이고, 실제 회색 헤더 표의 셀에는 별도 같은 제목이 있다. 색만으로 런을 제외하지 않는다. 배분 간격은 글자 다음 시작 위치를 전진시키며 backend의 `TextStyle::glyph_fit_advance`는 양수 extra_char_spacing을 glyph 폭에서 제외한다. 진단 `glyph_band_horizontal`은 공백이 없는 런에 전체 advance bbox를 그대로 써 마지막 글자 뒤 배분 간격도 잉크로 센다. 동일 replay/ink 계약의 누락인지 독립 PNG와 나머지 겹침 입력에서 확인 중이다. 앞의 제목 종료 추정만으로 정렬을 변경하지 않았다.

### 진행 중 보정 24: 실제 글꼴 공급과 진단 글리프 끝 소비

- 코드 head41b9be7f2b014f73ddeb4cb9668524d06b5db94d의 Native binary SHA d127aee7500a8edca9b9eeb8da1bd510dd01a85a052b55184fd32b867b355ebd, fresh WASM SHA cb4fb73566972d7d16d740eaa7f025a6c42554fae7b8d4087e332914b192d5e2를 고정했다. root pkg/Studio public의 JS/WASM byte 일치와 공개 API6/6 PASS다. 5593 Center/Top/Bottom의 Native/fresh WASM full review는 각각99.90031/99.90143/99.90031%, 모두 passed/본문 끝 차이0이다. 같은 배율로 셀 외곽·개체·뒤 문단을 직접 확인했고 정상 셀 높이가 유지된다. 원본/기대값 변경은 없다.
- shortcut은 양재난초체M 바탕쪽 쪽번호를 SVG에 내장하지 않아 Sans 숫자가 출력됐다. 실제 YNCH05.TTF(family Yj NANCHO Medium/양재난초체M, SHA d398aac5b56ab85b49343a3815ddad857e8b4dee01bc4f6c07eabe518faf2be9)의 동일 byte 사본을 ignored verified-font-supply/양재난초체M.ttf로 공급했다. 41b Native/fresh WASM 전체7쪽 TSV가 동일하고 최저96.14775%(p4), 누락0/90미만0이다. Native p4/p7 full review gate passed이며 한컴Print의 쪽번호·표 외곽·제목·본문을 직접 대조했다. 이전 누락 글꼴 캡처는 실패 기록으로 보존한다. 저작권 글꼴 파일은 커밋하지 않는다.
- scattered HWP의 Haansoft Batang/HWPX의 한컴바탕은 실제 HBATANG.TTF로 동일 face임을 fc-scan으로 확인했다. 이전 캡처의 해당 embedded font set은 비어 있었다. HBATANG.TTF 및 HMKMM.TTF 원 파일의 byte 동일 사본과 SHA를 verified-font-supply/verified-fonts.json에 고정하고 전52쪽씩 다시 측정 중이다. 루트 글꼴 디렉터리만 지정하면 filename lookup이 하위 symlink 디렉터리를 검색하지 않는다는 실제 소비 경로를 확인했다. 낮은 이전 점수를 배치 결함이나 불가피한 글꼴 예외로 승격하지 않는다.
- 진단 보정의 독립 근거: scattered p18의 하배/수관 두 런은 glyph 원점185.7467/211.6267, 글자폭13.3333px이며 이웃 셀 시작232.5067px 안쪽에서 끝난다. logical bbox는51.76px로 마지막 배분 간격12.5467px를 포함한다(scattered-ink-details-v40.log 및 실제 SVG). shortcut의 6개 parent 메뉴 제목도 배분 advance가 마지막 글자를 늘리지 않으며 실제 SVG 원점/크기에서 페이지 안쪽이다. 색·문서명으로 제외하거나 원 레이아웃을 줄이지 않는다.
- glyph_advance::tracked_glyph_end는 backend의 split_into_clusters → replay_positions_for → glyph_fit_positions → TextStyle::glyph_fit_advance를 동일하게 소비해 양수 배분의 glyph 끝을 구한다. 음수 자간의 advance보다 넓은 glyph 여유는 보존하며, finite/폭 계약이 없으면 종전 상자를 유지한다. glyph_band_horizontal이 공백 trimming 결과와 이 끝을 소비하고 flow_extent_bbox의 줄 합집합 및 text-overlap이 같은 결과를 사용한다. 원본 renderer/model·bbox·baseline·허용치·기대값은 바꾸지 않는다. 기존 계약 검사 및 실패 corpus 재검증을 이어서 수행한다. 새 정식 회귀·fixture는 아직 추가하지 않았다.
- Native scattered p18의 같은96DPI 영역을 확대해 glyph가 이웃 셀을 침범하지 않음을 확인했다(scattered-supplied-p18-ink-detail-v40.png). 동시에 독립Print와 약 한 행의 시작 위치 차이를 직접 확인했다. 실제 글꼴 공급을 전체 조판 통과로 보고하지 않으며 전쪽 점수와 이어받기 경계를 확인한 뒤 별도 보정한다.
- 차트는 기존 bounded metadata 초안으로 원 Contents type1/축0..2000/4division/series3/title 및 cached legend text를 다시 회수했다(column-metadata-v40.txt). 설치본 legend-item-layout의50.0 및0.25 상수를 실제 ELF load segment에서 읽어 chart-layout-constants-v40.json에 보존했다. 이 값만으로 문서별 좌표를 고정하거나 TWIP 단위를 확정하지 않았으며 제품 parser/renderer는 아직 바꾸지 않았다.
- 실제 글꼴 공급 뒤 Native scattered HWP의 전체52쪽 TSV는 최저41.05675%(p51), 90%미만41쪽/누락0이다. 원 face 내장 성공을 배치 개선으로 보고하지 않는다. 이전41.03131%와 구분해 native-general-supplied-fonts-v40/scattered-hwp/silhouette.tsv에 보존했다. 현재 입력 두 개를 source SHA로 고정한 새 한컴 Print 작업으로도 대응 기준 PDF를 재확인한다.
- 진단 보정 v40 기존 검사43/43 PASS(exit0, build9분42초, 실행214.216초/slow2): glyph-band 계약10개, text-overlap16partition, off-canvas16partition,5593대조1개. v37의 일반 실패5개가 기존 입력·기대값을 유지한 채 모두 복구됐으며 새 정식 회귀·baseline·래칫 변경은 없다. 원본6개 off-canvas 및 일반 겹침6건이 같은 glyph-band 소비로 해소됐다(logs/nextest-glyph-advance-v40.log). 이 결과는43개 선택 범위이며 전체 nextest/최종 lint/시각 보류 해소는 아직 남아 있다.
- 최초 Native Clippy v40은 앞선 메인터너 수식 코드 hancom_glyphs.rs의 needless_character_iteration/redundant_guards 두 건으로 exit101이었다. ASCII scalar 전수 검사를 동치인 str::is_ascii로, Roman/Italic guard를 동치 enum pattern으로 바꿨다. glyph/advance/shear/범위·기대값은 변경하지 않으며 같은 lint를 다시 실행한다. 이미 통과한 검사에 이 실패를 숨기지 않는다.
- 두 원본의 새 독립 Print가 완료됐다: HWP SHA65bbf7f22c8a…/Hancom2024 13.0.0.3901/PrintMethod0/PDF SHA d28a3c73b6b411e0b797b529894f0bc1d9f73fcb88112ab954cf2d2ef56bd6a7; HWPX SHAa36d3effddd7…/Hancom2020 11.0.0.9136/PDF SHA3e02139ddf2e9b58aa8d7e43c4c29e38735acfb8280685e0d916e28a1e7d0601. pdf/semanticist21-20261008/ 및 hancom-jobs/에 source hash/job/실제 출력 계약을 보존했다. HWP 새52쪽 Print와 기존41b Native PNG를 진단 png-pair로 비교해도 최저41.05675%이고 이전Print 대비 쪽별 점수 차이는 최대0.06639%p다. 새 head 재렌더링 증거로 보고하지 않는다. 실제 행 배치·분할 차이는 남아 있으며 reference 버전 표기만으로 면제하지 않는다.
- Native Clippy 경고 보정 뒤 동일 명령 cargo clippy --locked --target-dir target/pr-review -- -D warnings가 exit0/2분29초로 통과했다(logs/clippy-native-glyph-advance-v40-retry.log). WASM 전용 Clippy/workspace build/all-targets Clippy/정책 base 비교와 최종 full nextest는 아직 완료되지 않았다.

### 진행 중 보정 25: 전체 target lint 및 일반 표 경계 추적

- 2c483ce7b0ec535729257cf026475207ab320be6에서 WASM 전용 Clippy exit0/187.394초, workspace build exit0/264.042초다. all-targets Clippy는 시험지 검사 제거 뒤 남은 배열 원소가 하나인 기존 두 loop를 검출해 exit101이었다. issue_3893_bindata_sequence_ref 및 split_caret_axis의 동일 입력/검사 본문을 일반 block+let으로 옮긴다. 검사·입력·기대값·허용치를 바꾸거나 warning을 면제하지 않는다. 같은 all-targets lint와 base 정책 검사를 재실행한다(lint-glyph-advance-v40.json 및 로그).
- 글꼴 공급 후 Native 전수는 scattered HWP52쪽 최저41.05675%, HWPX52쪽 최저42.17084%, note-overlay4쪽 최저87.45003%, behind-variant4쪽 최저87.45049%로 실제 시각 보류가 남는다. source41b의 Native/fresh WASM scattered HWP52쪽 TSV는 동일하다. 두 종료/진행 상태는 각각 별도 로그로 확인하며 코퍼스 검사43PASS를 시각 통과로 대체하지 않는다.
- 일반 표 경계 추적은 DocumentCore 공개 입력 파싱/원본 셀·LineSeg 조회 → compose_section/HeightMeasurer::measure_section → block::prepare cut_row_heights/whole_row_fit_heights → continuation::fragment::budget/scan → row::required_height/whole_row_height → table_partial 실제 배치로 연결했다. scattered p18에서 Native 시작64행/Print 시작63행(용출성)이며, 앞 p16의 표18 첫 제목 원점도 Native564.1px/Print586.69px로 먼저 어긋난다. table15의 이어받기와 선행 공간을 함께 조사한다.
- RHWP_TABLE_DRIFT 진단에서 표6/15/18/62의 cut_row_h와 실제 MeasuredTable 행 벡터는 동일(diff0)이다. 따라서 단순히 더 큰 측정 벡터로 바꿔 공간을 늘리는 가정을 채택하지 않는다. 원본 line/선언·SourcePrint 문단 소속·내용 컷·물리 공간을 대조한다(scattered-source-cells-v41.tsv, scattered-measured-rows-v41.tsv, scattered-cut-measure-drift-v41.log, scattered-pages-v41.json). 새 조판 source/정식 fixture 변경은 아직 없다.
- 최초 loop 정리는 같은 입력을 쓰는 앞 함수에 적용되어 뒤 pics_survive_hwpx_roundtrip의 단일 원소 loop가 남았다. 재실행으로 검출해 뒤 함수도 같은 block+let으로 고쳤다. 공유 corpus_scope.rs 외부 mod가 같은 파생 suite에 두 번 로딩되는 duplicate_mod도 검출했다. 14개 원본의 private inline module에 동일 helper를 include!하고 불필요해진 path/root_mod 정책 override14개를 제거했다. helper 함수 본문/시험지52목록/입력·기대값은 동일하며 warning 면제는 추가하지 않았다. prepare:1451sources/6244staticattrs/48targets, all-targets Clippy retry exit0/12.89초, manifest/base 및 unit-tier/base PASS다(corpus-helper-inline-loading-v41.json 및 로그).

### 진행 중 보정 26: 저장 첫 조각의 완전 내용 컷과 물리 빈 띠

- 독립 한컴Print p1 SVG의 실제 괘선을 추출했다. 같은 row0..21의 Native/Print 외곽 차이는 약0.7..1.5px지만 마지막22행 바닥은 Native1067.2px/Print1081.536px로14.336px 차이가 난다. Native 제목 글줄도 마지막 셀 가운데 정렬에서8.6px 위다(scattered-independent-row-boundaries-page1-v41.tsv). 실물 위치를 formal golden/픽셀 assertion으로 고정하지 않는다.
- budget의 원본 첫 frame 조회는 이미Some(829.8667px)로 수용됐지만 scanner가 보호 블록 앞에서23행을 완전 소비한 결과는816.1px이다. 최근접 source_end24는 원자 블록과 실제 선택 컷의 소유를 대신할 수 없다. source_end를 억지로 맞춰 다음 행을 앞당기지 않는다. 저장 Common.height가 본문 안에 들어가며, 선택한 온전한 행의 예약 합과 실제 소비가 일치할 때에만 그 frame의 나머지 물리 띠를 현재 마지막 행에 보존한다.
- 생산/소비 연결: 기존 prepare_table_fragment_budget의 원본 frame 인증 → scan_table_fragment의 완전 컷/누적 consumed+end_row_height_override → emit_table_fragment의 partial_height/PageItem → table_partial의 실제 마지막 행 상자. 첫 조각/원본 저장/비편집/무컷/미정정 높이에서만 적용하며, 내용 단위·행 소유·이월 cursor를 바꾸지 않는다. 편집·재조판·부분 cut·각주 예산 초과·온전한 행합과 소비 불일치는 종전 경로를 보존한다. 새 정식 검사/fixture 추가는 전쪽 Native/fresh WASM90% 및 직접 판독 뒤까지 보류한다.

- v41 Native build exit0/4분17초, parent f1f8f9be6b40b8d552cc11d881f862819b10e5eb+patch SHA16a40984b1b6b72a06aed4be1332d453e6af42142ae4d83408281044c03c6164, binary SHA6b559f7cc90220505aece4c0bf9a37fb735a681bccc09fff4c48f876412ba597다. 실제 첫 조각은0..23행/829.8667px로 예약·그리기가 같은 물리 끝을 소비하며52쪽 전체 항목의 행/내용 컷 소유는 이전 후보와 동일하다. Native p1/p2/p3 직접 review: p1 약99.07%, p2 약97.10%, p3 56.53165%로 gate exit1/보류다. p1 마지막 행 외곽·셀 가운데 글줄은 개선됐지만 p3 뒤 표 원점·행 높이 차이는 남았다(native-stored-frame-tail-review-v41/ 및 scattered-pages-stored-frame-tail-v41.json). 새 fixture/기대값은 추가하지 않았다.
- 최신 upstream/devel f0e7228f6dd2ea1437724e53ad640c40c56d204b(#7619/#7667/#7630)의 저장 셀/재조판/float host 보정을 초기18개 후보에 반영한다. merge-tree 사전 비교의 충돌은 stored_reset.rs 한 곳이며, 초기 후보의 consume_saved_empty_run_reset 조건과 upstream의 stored_page_boundary_invalidated_by_reflow 조건을 동시에 보존한다. 이전60ef 기반 lint/43검사 결과를 새 통합 head의 완료 증거로 재사용하지 않으며 새 base 비교/영향 검사와 실제 출력 검증을 재실행한다.
- 최신 devel 충돌 해결: stored_reset.rs의 trigger에 consume_saved_empty_run_reset 및 stored_page_boundary_invalidated_by_reflow 두 독립 guard를 함께 둔다. 앞 조건은 이미 소비한 empty-prefix reset의 중복 전환을 막고, 뒤 조건은 재조판 직후 저장 빈 줄에서 옛 자동 쪽 경계를 복원하는 것을 막는다. 명시적 쪽/단 경계와 새로운 저장 본문·개체 owner를 각각 보존한다. merge 후 기존 관련 검사와 upstream 신규 경계 대조를 함께 실행한다.

### 최신 devel 통합 후보 검증과 사용자 지적 영역

- source ff7d647db94afd37b764905ed668239d3c0ef5bc/base f0e7228f6dd2ea1437724e53ad640c40c56d204b: Native build exit0/254.752초, binary SHA85b83eeebebd851b70e72f69b2a2215a535b80b4bd79f888b6e20e9e0979a4f9다. manifest/base 및 unit-tiers/base PASS다. 기존 선택 검사61/61 PASS(exit0, 전체588.446초/실행7.954초): upstream7330/7548/7618/7621의14검사, 기존7379의14검사,2097/6639/1133/TAC/glyph-band/5593대조33검사다. 이름별 실제 실행수는 nextest-upstream-integration-v42-counts.json에 기록했다. 필터에 포함한5598 이름은 실제 실행0건으로 그 검증을 주장하지 않는다. 새 전체 nextest/최종 lint/freshWASM은 아직 남아 있다.
- 같은 ff7d Native의 upstream 공개9입력/11쪽 전수 TSV는 최저92.52230%(cell-overlay), 나머지100% 또는 body-single-symbol99.94321%이며 누락0/90미만0이다. 대표 cell-fragments와 body-single-symbol full review/overlay를 직접 확인했다(각각100%/99.94321%, native-upstream-boundary-controls-v42/ 및 native-upstream-boundary-review-v42/). 셀 그림 뒤 제목, 단독 기호와 별표 세 글줄, 빈 문단/후속 본문 소속이 유지된다. 이 결과를 일반 scattered52쪽의 미통과 해소로 보고하지 않는다.
- 사용자 첨부 review_003.png(2026-10-08)는 v41 p3=56.53165%로 명시적 미통과다. 첫쪽 개선만으로 승인하거나 통합하지 않는다. 독립 Print p3 첫 인장시험(61행)을 현재 Native는 p2에 놓고 p3을편평시험62행부터 시작한다. 원본61행에 cell-protect/keep-with-next/widow/keep-lines/page-break 플래그는 모두 없다(scattered-row-boundary-flags-v42.log).
- 실제 원본/Print의72행: native44.2133px, Print36.7604px. 69..72의 짧은4행 병합 선언9448HU와 선행89.3067px가 요구하는 마지막 높이는36.6667px다. 현재 더 큰38..72의35행 선언61900HU 잔여가 마지막 행을44.2133px로 키운다. 이전 조각의 물리 빈 띠와 종료 병합 요구의 소유를 대조한다(scattered-span-physical-reservation-evidence-v42.json). 진단 좌표를 정식 expected/golden으로 고정하지 않았다.
- 진단 반례: 원본 HWPX61행의 저장 말미 spacing600→0, 및 같은 두 셀의 실제 문단 lineSpacing160→100+기존LineSeg 제거를 각각 독립 Print했다. 두 경우 모두인장시험은 여전히 p3에 있으므로 말미 줄간격만으로 잘못된 p2 수용을 설명하는 가정을 채택하지 않는다. 원본과 수정 provenance/job/Print SHA는 scattered-boundary-probe-inputs-v42.json 및 hancom-jobs/에 보존했다. 수동 진단 입력을 새 정상 fixture/원본 대체로 쓰지 않는다.

### 진행 중 보정 27: 반복 제목행이 여는 표 조각의 바깥 프레임

- 독립 반례 Print(2020 11.0.0.9136, method0/one-up): 같은 원본 HWPX 첫 표의 바깥 위/아래 여백141→0HU에서 인장시험61행이 p3→p2로 이동한다. 위/아래 각각141→216HU에서는 p3에 남는다. 셀 내용/행·열 그리드/선언 높이는 동일하다(scattered-outer-margin-probe-inputs-v43.json, scattered-outer-margin-print-observations-v43.json). Native의 p2는 before/bottom 모두0으로 첫 행 공간을 과대 평가했다.2×141HU=3.76px 예약 후 남은20.8267px는 원래 행20.88px보다 작다. 임의 상수/문서ID 없이 원본 margin을 소비한다.
- 생산/소비: 새 stored_header_frame 공통 술어(유효 저장/비편집/문단-단 기준 TopAndBottom RowBreak/실제 반복 제목행) → partial_rowbreak_fragment_spacing_px의 위/아래 예약 → fragment_placement.table_top 및 fragment_outer_bottom_overhead → layout_partial_table의 같은 반복 바깥 프레임. 기존 별도 위여백 경로는 새 프레임이 소유할 때 중복 가산하지 않는다. 원본 줄 segmentWidth가 양수라는 이유로 제목행의 새 물리 프레임을 버리지 않는다. TAC/합성·편집/비반복/다른 앵커·wrap 경로는 비적용이다. 영향을 받는 일반 대조와 Native 출력부터 확인하고 freshWASM/최종회귀를 이어간다. 아직 개선 완료나90% 통과로 판정하지 않는다.
- 앞 probe의 빈 띠/줄간격/작은 rowspan 수정은61행 소속을 바꾸지 않았다. 이것을 새 guard 근거로 재인용하지 않는다. 반복 바깥여백과 종료 큰 병합의 물리 예약은 별도 검증 경로로 남긴다.

- v43 Native build exit0/252.406초, binary SHA8265d44ff6456e2a93cb71b59c7bcb70b4d88c700336b9b263df6056b5db6dd7. 기존2097/6639/7379/1133/TAC 검사36/36 PASS(exit0,7.517초)다. Native p3 직접 review는56.53165→70.45078%로 여전히 gate exit1/보류다(native-stored-header-frame-review-v43/). 인장시험61행의 p3 소속은 복구됐지만, 앞 조각의 빈 물리 밴드가 병합 선언 잔여에서 빠지지 않아72행 높이/후속 표 원점이 아직 다르다. p1..3 결과를52쪽 전체나 freshWASM 검증으로 대신하지 않는다.

### 진행 중 보정 28: 병합 선언 잔여와 수용된 물리 행 상자

- 독립 Print p2 마지막60행은41.5521px로 원래20.88px보다20.6721px 크고, p3의72행은36.7604px다. 반복 제목행/바깥 여백을 예약한 Native p2의 남은20.8267px는 내용 단위를 수용하기 부족하지만 앞 조각을 닫는 물리 공간이다. 이 공간을 앞 행에 수용하면35행 병합38..72의 선언825.3333px를 끝행에서 다시 채우지 않아야 한다. 짧은69..72 병합은 여전히36.6667px를 요구한다. 독립 좌표는 진단 증거이며 기대값으로 고정하지 않는다.
- 구현 계획: 반복 제목행의 저장 프레임에서 실제 수용한 완전 행 상자를 continuation cursor에 기록 → 다음 조각의 종료 병합 선언에서 이전 물리 상자를 차감 → 하나의 행 높이 계획을 scanner/emit/PageItem/paint에 전달한다. 부분 내용 컷·재조판·편집에는 기존 경로를 유지한다. 제목행 반복 높이는 병합 내용의 소비로 세지 않는다. 각주 재스캔 뒤에만 빈 공간을 확정하고, 다음 행이 이미 들어간 예산/미완료 내용/비병합 경계에는 빈 공간을 추가하지 않는다.

- v44 parent1169ca645d125e356b62ec0d6cc660d2b3cd6049, Native build exit0/252.303초, binary SHA37431af25d96eb2f1818853f574301054afa4648f93e96d309251b153bd8722b. 변경 source와 새 physical_rows 모듈은 native-build-physical-row-plan-v44.json/source-code-v44에 각각 SHA로 고정했다. p1/p2/p3 Native full review는99.72513/99.49893/99.05228%로 수치 gate passed/exit0다. 세 review PNG를 직접 확인했고 p3 확대 판독도 별도로 실행했다.52쪽 전체/freshWASM 통과는 아직 주장하지 않는다.
- 단위 컷은 바꾸지 않고 받아들인 완전 행의 물리 예약을 기록한다. scan/refit → 공통 row plan을 적용한 MeasuredTable/whole-row fit/cut → emit/PageItem.fragment_row_heights → 일반 layout 및 cursor-rect probe의 같은 행 높이로 연결한다. 기존 PageItem 직접 생성과 remap은 None/clone으로 옛 경로를 유지하며 검사 기대값·입력은 바꾸지 않았다. 기존 #6976 쪽 하단 상자 접기와 100HU footer band는 예약·흐름과 별도 paint 소비여서 원래 경로를 유지한다. 표 외곽과 마지막 셀 상자 좌표를 같은 숫자로 보고하지 않는다.
- 기존 검사46/46 PASS(exit0,7.666초,1090skipped):2097/6639/7379/1133/TAC 및 PageItem 직접 생성이 있는6935/7032 대조다. fmt check, unit-tiers/base(f0e7228f6dd2ea1437724e53ad640c40c56d204b) PASS. manifest 최초 실행은 source 변경 뒤 파생 harness drift를 검출했다. --prepare로 새 source fingerprint를 재생성한 뒤 같은 base check가PASS(1455source/6258attrs/48targets)다. 파생 파일은 제출하지 않는다. 새 정식 렌더링 회귀·fixture·golden은 추가하지 않았다.
- 독립 여백 대조 p2/p3: margin0는99.45708/80.62738%로 gate exit1, margin+75는99.44336/95.95205%다. 두 변형 모두61행의 앞/뒤쪽 소속은 독립Print와 같다. margin0 p3는 앞 표가 거의 겹치지만 후속 제목/표의 원점이 다르다. 수동 변경한 source LineSeg와 재인쇄의 관계를 대조할 범위이며 숫자90 미만을 면제하지 않는다(physical-row-plan-validation-v44.json).
- 수치 통과 후 직접 검출한 미충족: 원래 Print p3 비고는 “바다모래인”만 있고 “경우”는 p4 첫 행에 이어진다. Native는 두 줄을 p3에 넣었다(p3-last-note-rhwp_png/pdf_png-v44.png, pdftotext p3/p4). 실제 원본 HWP pi9,row9,col8의 같은 문단 두 저장 줄은 text_start0/6, vpos0/0, LH1000/LS600/tag393216이며 dirty/keep/page-break가 없다(scattered-last-note-source-v44.log). 전체 행 fast path와 이 내용 컷의 소유를 계속 보정한다. 점수99.05%로 이 차이를 면제하거나 통합 승인하지 않는다.

### 진행 중 보정 29: 저장 첫 행 프레임의 문단 내부 원점0 컷

- 실제 HWP pi9의9행 col8은 같은 문단의0/6 text_start가 각각 vpos0/LH1000/LS600/tag393216이다. 이는 수동 메타데이터가 아니며 원본 HWP와 대응 HWPX에 같다. 원본 raw 행0..8 합18246HU < 첫 common frame21399HU <9행 끝21412HU다. 독립2024Print는 두 줄을 p3/p4에 분리한다. 단순0→0을 전체 행으로 흡수하는 가정에 대한 실제 반례다.
- 생산/소비 계획: 같은 원본 문단의 순서가 있는 양수 줄 전진과 선언 첫 행 프레임 경계를 함께 인증 → SourceFrameQuery의 전체 행 fast path 제외 → CellUnit의 strict source-frame cut → 기존 saved_multirow_opening_frame의 선언 상자/내용 컷 분리 → continuation/paint의 같은 cut. 같은 두0 줄만 가진 합성/편집/재조판/첫 frame을 닫지 않는 행은 이 새 원본 경계의 근거로 사용하지 않는다. 뒤 병합 라벨의 보존은 기존 opening-frame 수용성 검사가 그대로 확인한다.

- 최신 GitHub 재조회(v45): 비 draft24/draft6, 초기18개 이후7661/7662/7663/7670/7671/7673은 별도 검토 대기다. 사용자 선택에 따라 초기18개 통합 전에는 추가 체리픽하지 않는다. #7615 최신head4a6c27c5ffa54e4988a7e0e7956115d933725aca는 deferred7662 의존 갱신이 포함된 별도 delta이며 이번 원 적용headed9c2847523c18f658344ec89cb7ed1a426807ae와 구분한다(github-open-pr-snapshot-v45.json). 최신 upstream/devel은 f0e7228f6dd2ea1437724e53ad640c40c56d204b로 바뀌지 않았고 HEAD 기준 behind0이다.

- v45 Native build exit0/254.386초, binary SHA085389c3e70b9f81eb379bafe5becd2b9b11b8ed385f99f7c9dbf30de14d2b61. 원본 HWP/HWPX Print 모두 p3“바다모래인”/p4“경우”임을 다시 확인했다. 실제 새 내용 컷은 p3 pi9 rows0..10/endCut[1,1,2,1], p4 row9/startCut[1,1,2,1]이고52쪽을 유지한다. p3 Native는99.69210/99.68701%이며 두 줄의 페이지 소속도 직접 일치한다. p4는69.39683/69.34749%로 여전히 gate exit1이다. 관련 기존 검사 실제36/36 PASS(exit0,7.809초); 로그 재대조: #7032의7건은 실제 포함됐고 #6935/2097/1133은 실행0건이다. 해당 미실행 범위의 통과를 주장하지 않는다(nextest-zero-origin-text-frame-v45-counts.json).
- p4의 실행으로 확인한 원인은 first_fragment_blank_band의 base_remaining 생산값이다. 원시 행3166HU−첫 물리 프레임3153HU=13HU(0.1733px)만 continuation.start_row_height_override로 넘겼다. scanner의 carried-row 조기 경로는 이 값을 그대로 예약하고, paint는 이 높이를 그대로 적용한 뒤 남은“경우”의17.0933px 글줄 상자만 확장해 row10과 겹쳤다. 단일 행 source cut의 다음 프레임은 물리 최소 잔여와 남은 내용+패딩의 큰 값을 생산하여 같은 cursor/scanner/PageItem/paint로 전달한다. 블록 컷은 multi-row accounting을 유지한다. 기존 기대값 변경은 없다.

- 사용자 지시(2026-10-08): 현재 보정을 checkpoint commit한 뒤 최신 upstream/devel로 rebase하고 계속한다. v45 수치/내용 컷 증거와 아직 빌드하지 않은 이어받기 내용 높이 보정을 구분해 보존한다. source tree 동일성과 저장 reset의 두 독립 guard를 rebase 후 확인하며, 새 producer의 Native/freshWASM 출력은 재검증한다.

- rebase 완료: 사용자 요청 전 checkpoint `cc8c1c4ec3d10ec947f66fb427c4764a0da055a0` → 최신 base `f0e7228f6dd2ea1437724e53ad640c40c56d204b` 위 새 head `ab46e1c7431d1010b0b183d7d43b40453a087aaa`. `git diff refs/codex/semanticist21-pre-rebase-20261008-v46 HEAD`는 빈 결과로 tracked tree 전체가 동일하다. 저장 reset의 두 guard를 모두 보존하고 생성 JS를 특정 stash에서 복원했다. v46 Native build를 새 head에서 시작했으며 이전 v45 결과를 새 내용 높이 생산값의 검증으로 재사용하지 않는다(rebase-checkpoint-v46.json).

- v46 rebase head `ab46e1c7431d1010b0b183d7d43b40453a087aaa` Native build exit0/253.506초, binary SHA `64dc4b71890ea628f0d488b3afebb06f6220ff67ed65b9e30f4237278beaaa59`다. 원본 HWP p3/p4는99.69210/99.15457%, HWPX는99.68701/99.13639%로 두 수치 gate exit0다(native-continuation-content-height-review-v46/). 실제 p4 row9 높이는0.1733→20.88px이며 row10은119.2px에서 시작해 “경우”와 겹치지 않는다. 이전 p4의 추가 응결54행도 사라지고 독립 Print와 같이53행 비표면적에서 끝난다. 좌표는 진단값이며 assertion으로 고정하지 않았다.
- v46 직접 판독의 별도 미충족: p4 마지막53행의 병합 셀 col0 “플라이애시” 및 col7 “·제조회사별”은 독립 Print에 있지만 Native에서 비어 있다. 두 셀은 실제 원본의5행 병합/선언8985HU이며 다음 저장 줄이 원점0에서 재개한다. 첫 보이는 줄은 들어가지만 `rowbreak_straddle_cut_units → cell_units_fitting_height`의 코드와 원본 메트릭을 대조하면 완전 유닛 높이에 말미 spacing600/552HU가 포함돼 첫 조각의0유닛 수용으로 이어지는 것으로 추정된다. 이 분기의 실행 trace와 재개 컷 보존을 후속 검증한다. 이 경계의 source와 독립 p4/p5 텍스트 증거를 scattered-row53-source-owners-v46.json/scattered-print-p4-p5-label-v46.txt에 보존했다. 정식 검사·기대값 변경 및 완료/승인은 계속 보류한다. fresh WASM/전체52쪽은 새 head로 이어서 검증한다.

- v46 기존 관련 회귀46/46 PASS(exit0,전체306.122초/실행7.776초,1470skipped). 실제 실행 모듈별 수량은 nextest-continuation-content-height-v46-counts.json이며6935/7032 및2097/1133도 포함한다. fresh WASM root wrapper 빌드와 원본 HWP/HWPX 전체52쪽 Native TSV를 시작했다. 최종 전체 회귀/lint 및 다른 미통과 문서의 범위는 남아 있다.

### 진행 중 보정 30: 병합 셀의 저장 첫 글줄과 다음 프레임 컷

- 독립 원본 HWP/HWPX Print p4/p5는53행의 “플라이애시”/“·제조회사별” 한 줄씩을 p4에, 남은 “시멘트 (KS L 5211)”/빈도 문단들을 p5에 둔다. Native v46 p4 trace에서는 두 셀의 문단 방출이 없고 p5는 su0으로 전체 문단을 다시 시작한다(row53-owner-trace-v46.log/row53-next-owner-trace-v46.log). 첫 프레임의 가시 내용과 뒤 간격이 다른 소유라는 실제 반례다.
- 적용 경로 추적: 실제 첫3개의 원본 저장 줄 0→0→LH+LS를 가진 동일 시작·끝의 병합 셀들이 첫 frame 경계를 공유 → native_first_line_restart_cut_trailing_trim의 독립 source spacing → cell_units_fitting_height의 같은 prefix end/su 선택 → rowbreak_straddle_cut_units 및 straddle_continuation_demand의 같은 소유 컷 → PageItem의 행 경계와 table_partial의 가시 줄. 단일 셀의 불완전 단서·다른 rowspan 경계·편집/합성/재조판은 인증하지 않는다. 다음 조각에서도 앞 physical 높이에 같은 fitting 함수를 적용해 eu==su를 보존한다.
- 구현 방향: 병합 셀의 shared restart는 같은 행에 우연히 존재하는1행 셀 전체에 강제하지 않고 같은 시작·끝을 가진 원본 병합 peer에서 증명한다. 말미 간격은 인증된 source cut의 첫 유닛 fit에만 제외하고, 유닛의 일반 높이를 다시 쓰거나 paint에서 내용을 복제하지 않는다. 기존 양수 되감김·TAC·단일행 viewport 및 다른 원점0 정상 대조를 다시 실행한다. source/WASM 빌드 종료 후 수정한다. 정식 검사/fixture 추가는 관련 전체 Native/freshWASM90%와 확대 판독 뒤까지 보류한다.

- v46 fresh WASM wrapper exit0/597.310초, WASM SHA `066c60e35fa592344ab5b8d0b864398b4c75dda205a46ea8533081ec420dd7f5`, JS SHA `7462392b9b4dadb36c0b60cc0afa9de3fc6e41a861b150010ea51f0a7974737a`로 root pkg/public이 동일하다. --no-opt의 Native 진단 빌드이며 optimized 배포 artifact와 구분한다. HWP p3/p4 fresh WASM은99.69210/99.15457%로 Native와 같고 p4 review 직접 판독에서도53행 두 셀의 내용 이동이 남는다. HWPX/전체52쪽은 실행 중이다.

- v47 원본 프로필 재조회: HWPX는 hwp5_origin이 아닌 genuine hwpx_stored_layout이며 실제 첫3개 LineSeg가 HWP와 같다(scattered-spanning-first-frame-hwpx-source-v47.log). 이 저장 병합 프레임에도 같은 source 계약을 적용하고 기존 단일행 HWPX 경로는 별도 계약을 유지한다. parent `32ce04e790cdf9b3b59a3464662f9d61cb9767ef`+source SHA `e40026715fdfb8ba80ddba7ebfc3264fac62e6695f6a1e187f202d28b0eb06da`, Native build exit0/281.461초, binary SHA `87daa4a670a4187a8739f0c70413b35992bb6a06da0a6c850cc873af6e079b87`다. 실행 trace p4의 두 셀은 cut(0,1)로 첫 줄을, p5는 cut(1,MAX)로 남은 줄을 방출한다. 다음 조각은 앞쪽의 첫 줄을 재방출하지 않는다. 같은 source의 재측정/내용 높이는 기존 shared trim을 소비하며 행 경계는 유지한다. 비교 이미지와 기존 회귀는 진행 중이며 완료/승인으로 판정하지 않는다.

- v47 Native p3/p4/p5는 HWP99.69210/99.72851/96.44533%다. p4/p5 직접 review에서 첫 표제의 소속·후속 줄 보존은 개선됐다. 다만 p5의 잘린 셀을 Top으로 강제하는 기존 effective_align 분기로 남은 “시멘트 (KS L 5211)”와 빈도 문단들이 위로 붙는 미충족을 확인했다. 관련 기존 회귀 50/50 PASS(exit0,전체344.583초); #6761 원점 재시작의 실제/부정 대조도 포함한다(nextest-shared-span-restart-v47-counts.json). fmt 및 manifest/base(f0e7228...) PASS, 파생 harness는 미제출이다. 이 checkpoint의 수치 통과를 정렬 차이의 통과나 승인으로 바꾸지 않는다.
- v46 Native HWP 전체52쪽 TSV가 완료됐으며 최저45.96876%/90미만29쪽/누락0이다(native-continuation-content-height-full-v46-hwp-summary.json). HWPX 전체 비교는 계속 실행 중이다. v46 freshWASM HWPX p3/p4는99.68701/99.13639%로 Native와 같지만 동일 표제 소속 문제를 가진다. 이 역사 후보를 v47/후속 source의 전체52쪽 검증으로 재사용하지 않는다.
- 다음 보정: authenticated shared restart의 cut(1,끝)과 완료된 물리 rowspan이 남은 프레임의 정렬 owner다. straddle cut을 원본 padding 직후 한번 산출해 composition window/line ranges/effective_align이 같은 결과를 소비하게 한다. 열린 꼬리 프레임·중첩 표·미인증 재시작은 원래 경로를 유지한다. 원본 모델의 Center에 따라 현재 프레임 안에서 남은 내용만 정렬하며 독립 Print p5의 위/아래 간격과 대조한다. 새 절대 px 기대값은 만들지 않는다.

- v47 대응 HWPX p3/p4/p5도99.68701/99.67973/96.61612%로 수치 gate는 exit0다. HWP p4는 새 표제/빈도 첫 줄을 포함해99.72851%이며 p5의 잔여 내용은 보존되지만 앞에 기록한 정렬 차이는 면제하지 않는다.
- v48 후보: rowbreak_straddle_cut_units를 composition 선택 전에 한 번 산출하고 같은 Option cut을 가시 줄 범위에 재사용한다. shared source 첫줄 restart에서 start1/모든 잔여 unit/완료된 물리 rowspan을 가진 마지막 프레임만 원래 valign을 소유한다. source 인증은 dirty·cell-format-vpos·controls가 없는 원본 병합 peer에서 확인한다. 새 native_first_line_restart_cut_trailing_trim의 반환값은 scanner/잔여 높이 측정과 같은 source spacing이며 별도 px 보정이 아니다. full cargo fmt/check와 후속 변경 파일 rustfmt/check는 exit0다. parent94cfcc9e8의 Native 새 빌드를 시작했다.

- v48 Native build exit0/275.837초, binary SHA `c6f2e8227af6bede6a7fa404a34a581c791a93e635887aa28d90cfbd5ef95c3a`다. source2개는 native-build-shared-span-alignment-v48.json/source-code-v48에 SHA로 고정했다. HWP p3/p4/p5는99.69210/99.72851/99.07224%다. p5 trace에서53행 표제/빈도의 원점은 각각131.8/110.5px로 원래 모델 Center의 남은 프레임 정렬을 소비한다. 아직 freshWASM/전쪽/최종 검증 통과를 주장하지 않는다.
- v46 원본 두 파일 전수52쪽 TSV 완료(exit1): HWP 최저45.96876%/29쪽 미통과, HWPX 최저46.40781%/29쪽 미통과, 둘 다 누락0이다(native-continuation-content-height-full-v46-summary.json). 이 기록은 이전 source ab46e1c74의 진단 범위이며 현재 후보의 gate 통과로 대체하지 않는다.
- 후속 직접 검출: p5 하단96행/9행 병합의 “콘크리트 혼입용 방수재”는 독립 Print에3줄이 있지만 Native는2줄만 놓고 “방수재”를 p6에 넘긴다. 실제 저장 줄은0/1600/3200/4800HU로 원점 재시작이 없는 연속 텍스트다. `cell_units_fitting_height`가 마지막 보이는 줄의 spacing600HU까지 요구해 선택한 물리 행밴드에서3번째 줄을 배제한다. Print p6는 “(KS F 4926)”만 남은 병합 프레임에 가운데 정렬하지만 Native는 두 줄을 위에 붙인다(scattered-print-p5-p6-bbox-v48.html/scattered-page6-owner-v48/). 단순 첫줄 재시작만 정렬 owner라고 가정한 범위도 이 실제 반례로 확대 검토한다. 일반 저장 plaintext rowspan의 가시 prefix 끝과 source advance를 구분하고 마지막 물리 프레임의 실제 잔여 내용/원래 valign을 공통 cut으로 소비하게 하는 후속 보정이 필요하다. 새 기대값/절대 px assertion으로 차이를 숨기지 않는다.

- v48 HWPX p3/p4/p5는99.68701/99.67973/99.02628%다. HWP p5의 direct review에서53행 잔여 표제/빈도는 독립 Print와 같은 가운데 정렬로 개선됐지만96행 “방수재” 및 빈도 후속 줄의 불필요한 이월 차이는 남는다. 관련 기존50/50 PASS(exit0,전체315.112초/실행8.028초,1677skipped), manifest/base PASS다. source SHA와 Native 비교는 위에 고정했고 freshWASM/최종 전체 검증은 미완료다. 사용자“메인터너 보정으로 해결” 지시에 따라 같은 scope의 출력 보정을 계속한다.

### 진행 중 보정 31: 일반 저장 병합 셀의 가시 줄 끝과 다음 줄 전진

- 원본96행의 연속0/1600/3200/4800HU는 마지막 보이는 줄의 glyph/LH와 그 뒤 source spacing600HU를 구분한다. Print p5는 “방수재”까지, p6는 “(KS F 4926)” 한 줄만 마지막 물리 프레임에 가운데 정렬한다. 초기 재시작 인증만으로 마지막 저장 프레임의 정렬을 정하는 가정을 제거한다.
- 생산/소비: 새 stored_span_content의 비편집·비재조판·비TAC RowBreak/원본 plaintext rowspan 인증 → cell_units_fitting_height의 prefix 끝에서 source 줄뒤 간격만 제외/전체 전진량은 유지 → 이전 eu와 다음 su에 같은 함수 사용 → straddle_continuation_demand의 잔여 내용 높이 및 table_partial의 한번 계산한 cut → 끝난 물리 rowspan/완전 잔여 창의 원래 valign과 실제 paint. 마지막 whole-cell unit은 이미 LS를 제외하므로 다시 차감하지 않는다. controls·synthetic LS·dirty·음수 spacing·없는 원본 줄은 비적용이다.
- 작은 원점 재시작에 맞추는 추가 수치·문서ID·픽셀 clamp는 없다. 새 source 모듈은55줄 수준이고 신규 formal 회귀/fixture/golden은 추가하지 않는다. 원본 HWP/HWPX p5/p6와 앞53행 경계의 무회귀, 기존50검사 및 이후 Native/freshWASM 전수·확대 판독으로 계속 검증한다.

- v49 Native 최초 컴파일은 새 child module의 인증 메서드 접근 범위 오류(E0624)로68.419초/exit101이었다. 실제 table_partial 소비 지점이 볼 수 있는 layout 내부 범위로 수정하고 재빌드했다. retry exit0/251.204초, binary SHA `0aa3d8af2d3d6ccb2c1d2576d62d24eae17c8b9f19c1fd9adf0d2540670fe18a`다. 실패 metadata/log 및 수정 후 source SHA를 분리 보존했다(native-build-stored-span-content-v49-failed-visibility.json/native-build-stored-span-content-v49.json/source-code-v49-retry/).
- v49 실제 원본 HWP/HWPX의 p5는96행 “콘크리트 혼입용 방수재”3줄과 빈도3줄을 보존한다. p6는“(KS F 4926)”/“때마다”만 남겨 같은 마지막 물리 셀 프레임의 Center를 적용한다(span-page-owners-v49.json). 독립 Print의 두 p6 단어 yMin115.654320pt와 Native 같은 영역의 줄 원점154.3px를 직접 대조했다. source53행 p4→p5의 컷과 Center도 유지한다. 진단 좌표를 새로운 기대값으로 고정하지 않는다. 3..6쪽 PNG/관련 기존 회귀/그 뒤 freshWASM 전수 검증은 진행 중이다.

- v49 원본 HWP p3/p4/p5/p6 Native는99.69210/99.72851/99.71529/99.81823%, HWPX는99.68701/99.67973/99.61458/99.72035%다. 두 파일 모두4쪽 선택 범위의 수치 gate exit0이며 관련 최저99.61458%다. HWP p5 하단과 p6 첫 병합 셀을 같은2배 크기 Native/Print crop으로 직접 확인했고 “방수재”/후속KS와 빈도 마지막 줄의 소속·센터·뒤 셀 포함이 일치한다(row96-p5-native-print-zoom-v49.png/row96-p6-native-print-zoom-v49.png). HWPX p6 full review도 직접 확인했다. 해당 plain 행 프레임의 컷을 확인했으며 p6 화학식 첨자와 μm의 직접 판독은 아래 추가 기록을 따른다. 알려진 줄간격 차이를 수치 통과로 면제하지 않았다.
- v49 기존 관련50/50 PASS(exit0,전체319.996초/실행7.862초,1677skipped); 원점 재시작 실제/부정 대조와 빈 문단/원자 창/이어받기/원래 외곽 대조를 같은 후보에서 실행했다. 실제 모듈 수량은 nextest-stored-span-content-v49-counts.json이다. source fmt/manifest/base PASS. 아직 Native/freshWASM 전체52쪽 및 다른 일반 문서의 미충족, 최종 lint/전체 회귀가 남아 있어 checkpoint로만 기록한다.

- v49 p6 화학식 MgO/SO3/SiO2의 아래첨자·정체, 45μm의 기준선·앞뒤 글자 간격을 Native HWPX/독립 Print의 같은3배 crop에서 직접 대조했다(chemical-subscripts-p6-hwpx-zoom-v49.png). 내용/첨자/간격의 가시 결함은 확인되지 않았으며 AA와 괘선 농도 차이는 남는다. HWP p3 최신 review도 직접 확인해 사용자 지적의 마지막 표/뒤 제목·비고 컷이 유지됨을 확인했다. source checkpoint77953f3c8에서 freshWASM wrapper 빌드를 시작했다.

- v49 fresh WASM wrapper exit0/591.983초, code head `77953f3c8a64b295f82c171a8dc4e26dde614a24`와 WASM SHA `76035a19d7ffe1626a41501c1be3c993ceca831838311b46767669895b7fc75a`를 고정했다. root pkg/public JS 및 WASM 해시가 각각 같으며 --no-opt의 Native 진단 산출물이다. 같은 pinned package/binary의 p3..6 full review와 HWP/HWPX 전수52쪽 fresh WASM TSV를 시작했다. Native 전수52쪽도 별도 pinned binary에서 실행 중이다. 새 formal 회귀/기대값 및 통합 판정은 전쪽 최저값과 직접 판독이 완료될 때까지 보류한다.

- v49 freshWASM p3..6 full review가 두 원본 모두 exit0으로 완료됐다. HWP99.69210/99.72851/99.71529/99.81823%, HWPX99.68701/99.67973/99.61458/99.72035%로 Native와 같다. HWP p3 및 HWPX p6 review를 직접 확인했다. 사용자 지적 p3의 큰 표 외곽·행/쪽 소속과 뒤 제목은 복구됐으며 범위 최저99.61458%다. Native HWP52쪽 TSV는 최저45.96876%/28쪽 미통과/누락0으로 완료됐고 HWPX 및 freshWASM 전수는 진행 중이다(native-stored-span-content-full-v49-hwp-summary.json). 선택 범위 통과로 전수·통합을 승인하지 않는다.

### 진행 중 보정 32: 앞 조각의 물리 행 상자로 병합 컷 이어받기

- 독립 Print p6에는138행 빈도 셀의 첫3줄이 있고 p7은 “: 1일 타설량마다”부터 시작한다. Native v49는 p6에서 이미 방출한 “세제곱미터 미만인 경우”를 p7에서 다시 방출한다. 실제 원본138행/span3의 저장 줄은0/1300/2600/3900HU이며 다음 문단에서1300HU로 되감긴다(scattered-row138-source-hwp-v50.log). source/출력 분류로 면제하지 않는다.
- 실제 호출 경로: record_stored_fragment_rows가 p6 마지막 물리 행 밴드를 원장에 기록 → stored_fragment_row_plan이 다음 MeasuredTable/PageItem로 전달 → rowbreak_straddle_cut_units는 이를 받은 뒤에도 단일행 셀이 있는 이전 행을 row_cut_content_height로 재측정해 더 작은 내용 높이로 덮어씀 → p6 eu3/p7 su2의 중복과 종료 셀 요구 증가 → 뒤 행/쪽 끝 내용의 이동이다. 저장 plaintext span의 이전 완전 행은 실제 원장/공통 resolved 행 상자를 소비하고, 합성·편집·중첩/controls 등 비인증 입력은 종전 내용 높이 경로를 유지한다. 부분 행 컷은 같은 source cut을 계속 사용한다.
- 보정은 별도 prior-row 높이 helper를 추가해 기본 straddle 및 시작 컷 걸침의 paint 구제에서 같은 결과를 소비하도록 한다. 새 formal 회귀/fixture/기대값 추가는 관련 전체 Native/freshWASM90% 및 직접 확대 판독 뒤까지 보류한다.

- v49 freshWASM p6의 SO3/SiO2 첨자·정체·글자 간격을 독립 Print의 같은3배 crop으로 직접 확인했다(chemical-subscripts-p6-hwpx-wasm-zoom-v49.png). Native와 freshWASM의 원본 두 파일 p3..6 RGB raster 비교는8쪽 모두 차이 bbox=None이다. 직접 확대 판독한 Native p5/p6의 소속·Center·MgO/45μm 영역도 같은 실제 raster임을 별도로 확인했다. p7 이전 source trace는138행 빈도 셀의 su2를 명시한다(scattered-p7-trace-v49.log). 후속 source v50은 별도 빌드 중이며 v49 결과를 v50의 검증으로 재사용하지 않는다.

- v50 Native build exit0/293.195초, parent77953f3c8+3개 source SHA 및 binary SHA `13c7898912aa13d841f55033aec9ca74a487ceab4d8fc0a52255c714d9a70b7b`를 native-build-stored-span-prior-height-v50.json/source-code-v50에 고정했다. 원본 p7 실행 trace에서138행 빈도 su3가 적용되어 중복 문장이 없어지고 조각139..170으로 쪽 끝169행을 보존한다. 다음 p8은170..212, p9는212..244이며 전체52쪽을 유지한다(scattered-pages-v50.json/scattered-p7-trace-v50.log).
- v50 HWP p3..8 Native는99.69210/99.72851/99.71529/99.81823/99.52407/99.62649%다. p7은67.27483→99.52407%, p8은61.24208→99.62649%로 개선됐다. 최신 p7/p8 review PNG를 직접 확인해 중복 제거·마지막 PC강연선 행·다음쪽 첫 인장하중 및 p8말미 외관검사 소속·내용 보존·표 외곽을 확인했다. p7 비고의 kg/m³ 위첨자와 ± 양쪽/글줄 간격을 같은3배 Native/Print crop에서 직접 대조했다(p7-spacing-superscript-native-print-zoom-v50.png). HWPX 선택 범위와 전체52쪽은 계속 검증 중이다.
- v50 기존50/50 PASS(exit0,실행8.768초/1677skipped); 이름별 실제 수량은 nextest-stored-span-prior-height-v50-counts.json, 전체 소요는 같은 exit JSON이다. source 추가로 파생 suite fingerprint를 prepare했으며 신규 formal 테스트/fixture/기대값은 추가하지 않았다. Native 전수52쪽은 새 pinned binary에서 시작했고 아직 통합/완료 gate는 보류다.
- v49 Native HWP/HWPX 전수52쪽은 exit1으로 종료됐다. 각각 최저45.96876/46.40781%와28쪽 미통과/누락0이다(native-stored-span-content-full-v49-summary.json). 같은 v49 freshWASM HWP 전수도 Native와 동일한52개 페이지별 수치다. HWPX freshWASM 전수는 진행 중이며 이 역사 후보의 결과를 후속 v50 head의 증거로 대체하지 않는다.

- v50 HWPX p3..8 Native도99.68701/99.67973/99.61458/99.72035/99.51656/99.59727%로 full review exit0이다. 두 원본 선택12쪽 최저99.51656%이며 HWPX p7/p8 review도 직접 판독했다. 중복 빈도/PC강연선 및 다음쪽 외관검사의 소속, 뒤 표 괘선/가시 줄은 독립 Print와 일치한다. source fmt 전체 check, manifest/base 및 unit-tiers/base(f0e7228...)는 PASS다. existing50 검증 총354.259초/실행8.768초이며 새 전체 Rust lint/전수nextest는 아직 남아 있다. source를 checkpoint 커밋하고 그 정확한 head에서 freshWASM을 다시 빌드한다.

- v50 전수 출력의 최신 raster를 --png-pair로 재사용해 조기 진단했다(재출력은 같은 pinned Native binary의 진행 중 전수 작업, 입력 PNG 해시는 각각 silhouette_manifest.json에 기록). p9=99.32384%, p10..15=91.20541/97.77033/99.67256/98.81025/99.09792/99.27556%, p16..19=99.64007/99.56841/99.10979/97.37686%다. p9 직접 대조에서212행 시작과244행 끝이 독립 Print와 같고 p10/p11 내용 소속도 크게 개선됐다. 단, p10 첫 partial 블록 조각의 하단 프레임은 Native 약497.1px/원본 Common.height507.28px로 짧고 독립 Print에서도 하단 괘선/셀 가운데 차이가 보인다. 91.20541% 수치로 면제하지 않고 후속 프레임 보정 대상으로 남긴다.
- source p10 trace는 table12 첫 source_frame=Some(507.28,988.16), source_end7, available508.6, 실제 cut row6/12개 슬롯 및 consumed489.3px다. paint는 마지막 셀 padding을 넣어497.1px로 늘리지만 저장 프레임의 나머지 물리 공간은 사라진다(scattered-p10-drift-v50.log/scattered-source-frame-v50.log). source frame의 선택 컷과 실제 물리 요구를 scanner→emit→PageItem→paint에서 같이 보존하는 후속 조사이며 아직 source 추가 변경은 없다.
- v50 조기 Native p20..24는59.15986/64.64593/48.03726/48.53776/58.86768%, p28/p29는55.49027/64.76842%로 미통과다. p25..27은99.84791/99.65724/96.08807%다. 전수 전체/직접 판독/freshWASM 결과는 계속 실행 중이며 일부 높은 수치로 완료·승인을 판정하지 않는다.

- 사용자 “review.png 로 보여줘야 내가 같이 보지” 지시를 반영했다. 이후 수정/보류 영역을 현재 head의 Native/Print/overlay 세 패널 review PNG로 대화에 함께 표시한다. 최신 전수 raster에서 canonical visual_sweep.make_compares/make_overlay_compares/make_review_panels로 p10/p19/p20을 생성했다(native-stored-span-prior-height-selected-review-v50/review/ 및 provenance.json, create-selected-review-v50.py). p20=59.15986%의 실제 review_020.png를 공유했고 미통과를 유지한다.
- v50 freshWASM build는 source dc18b952716db98d7c9aa758c5eb8055645f86f0에서 exit0/620.321초, WASM SHA b695a553c1b9e129934cfa0763d6034ef1ff67bba4d36a5ffd13d02df691de6a다. root pkg/public JS·WASM 해시는 각각 동일하며 --no-opt 진단 산출물로 구분한다. 같은 pinned binary/package의 p3..8 및 새 실패 경계p19/p20 review를 시작했다. Native p20에서 이미 미충족이므로 freshWASM 전수52쪽을 추가로 시작하지 않고 선행 경계 보정과 Native 전수를 이어간다.

- v50 Native HWP52쪽 전수 TSV가 완료됐으며 최저48.03726%/미통과15쪽/누락0이다(native-stored-span-prior-height-full-v50-hwp-summary.json). 직전v49의28쪽 미통과에서13쪽이 수치 기준을 넘었으나 p10의 직접 판독 미충족과 다른15쪽이 남는다. HWPX 전수는 계속 실행 중이다. v49 freshWASM HWPX52쪽도 최저46.40781%/28쪽 미통과/누락0으로 완료됐으며 역사 source의 두 backend 전수 수치는 같다.

### 진행 중 보정 33: 같은 행의 서로 다른 저장 소유자가 입증하는 원점0 경계

- 독립 Print p19는146행의 “IDDV”/“KS M ISO 2507-1,” 첫 줄을 마지막 밴드에, p20은 “(ISO 3633)”/“KS M ISO 2507-2”부터 둔다. 원본146행 col7은 같은 문단의 두 실제 LS가0/0·LH1000·LS0이며 text_start는 전진한다. col1/span3은 IDDV의0과 다음 ISO 문단의0→1580(LH1000+LS580)이다. 일반 문단 로컬0만으로는 판정하지 않고 서로 다른 source 소유자의 같은 row에서의 restart/뒤 사다리를 함께 확인한다(scattered-source-frame-v50.tsv 및 최신 review_019/020.png).
- 실행 경로는 advance_row_cut의 [1,1]/13.3px 콘텐츠 컷 → row_cut_content_height의 실제 패딩 포함20.9px → row_step의25px 고아 기각 →146행 통째 이월 → 앞145행에 남은 물리 띠 기록 → 다음 source rowspan의 불필요한 전진이다(scattered-p19-trace-v50.log). 일반 완전 행의 공간 원장은 앞 수정대로 유지한다.
- 새 공유 경계 인증은 비편집/비재조판/비TAC TopAndBottom RowBreak의 원본 단일행 같은 문단0/0 및 같은 시작행의 실제 병합 peer의0/0→LH+LS를 함께 요구한다. target의 spacing0은 경계 존재와 공간 크기가 다른 상태이므로 허용하되, 증명 없는 첫 조각 경로는 기존 양수 spacing+Common.height의 물리 교차를 그대로 유지한다. 인증을 CellUnit의 frame-break/whole-row 제외/내용 컷 및 orphan keep가 같은 helper로 소비하게 한다. 모든0·합성·dirty·편집·개체·뒤 사다리가 없는 peer는 인정하지 않는다. 기존6761의 실제/부정 대조를 함께 검증하고 새 formal 테스트/fixture 추가는 전쪽 Native/freshWASM90%와 확대 판독까지 보류한다.

- v51 최초 Native build exit0/282.366초, binary SHA0d1b516d8140f04b2036a392a450579911854d918e1ae9a2afb3cb6c0bd31850다. 실행에서53쪽으로 늘어나 원본52쪽 계약을 위반했으며 실패 후보를 보존한다(native-build-shared-zero-origin-frame-v51.json/source-code-v51/scattered-pages-v51.json). p19에서는146행을 계속 이월하고 p20에 첫 줄만 있는 별도 조각이 생긴다.
- 추가 추적으로 초기 의심한 패딩 중복은 원인이 아님을 확인했다. content13.3/padding7.5, 컷[1,1]은 맞지만 stored_zero_origin_rewind_keep가 이미 끝난 col4의1줄에도 원점0 되감김을 요구하고, 다른 plain-reset gate는 표 전체의 모든 행에 single-row 선언이 있어야 하므로 원본의 병합 전용 행에서 차단된다(scattered-p19-drift-v51.log). 경계 인증과 공간 크기는 그대로 두고, 이미 완료된 소유자와 실제 이어질 source 소유자를 구분하는 공통 row-cut 판정을 추가한다.

- v50 Native 원본 두 파일 전수52쪽은 exit1으로 종료했다. HWP/HWPX 최저48.03726/48.41629% 및 각각15쪽 미통과/누락0이다(native-stored-span-prior-height-full-v50-summary.json). 같은 code head의 freshWASM p3..8/p19/p20 review도 종료(exit1)했다. 두 backend의 선택 범위 페이지별 수치는 각 원본마다 동일하며 HWPX p19/p20=97.48563/59.68074%다. p3..8의 실제 개선은 유지되지만 p19→20의 잘못된 소속은 면제하지 않는다.


### 한컴 기준 PDF 보존: untracked 자료 정리

- 사용자 지시 “현재 untract 된 pdf 는 뭐임? 필요없으면 제거, 필요하면 커밋”에 따라 `pdf/semanticist21-20261008/`의210개 PDF(10,134,567바이트)를 확인했다. 원본 기준, 편집·합성 대조 및 진단·반증 대조이며 기존 추적 PDF/이번 폴더 내부의 바이트 동일 중복은0이다. 현재 검토의 독립 출력 증거로 보존한다.
- `references.tsv`에 입력 경로·입력 SHA256, PDF SHA1/SHA256·크기·쪽수, Creator/Producer, engine·job ID·상태·print_method 및 로컬 작업 기록을 연결했다.210개 입력 해시는 현재 파일과 모두 일치하고210개 변환 상태는 succeeded다. 출력 계약은 작업의 status.result.pdf_print_method에서 읽었으며210개 모두 method0을 확인했다. 원본 문서의 info.printMethod와 변환 결과 계약을 구분한다. 변환 서비스 주소/비밀 환경 값은 포함하지 않는다. 상세 검사 원장은 ignored `output/pr-review/semanticist21-20261008/pdf-reference-audit-v51.json`에 보존한다.
- 이 커밋은 기준 PDF 보존이며 새 formal 회귀 fixture/golden 추가나 시각 gate 통과 판정이 아니다. 편집·합성 입력은 현재 ignored 증거 경로에 보존되어 있으며 최종 제출 증적의 입력 재현성 정리가 아직 남아 있다. 진행 중 v51 source는 별도로 검증한다.


- v51 retry Native는 binary SHA4225ced5c10f1a2c3293668767dac27f224885b2d22c551d0fee11201e99cde7/source-code-v51-retry로 고정했다. 원본52쪽을 유지하며 HWP p3/p4/p19/p20=99.69210/99.72851/99.35269/96.98379%다. 실제 review_019/020.png를 대화에 표시했다. IDDV/KS M ISO2507-1은 p19에, ISO3633/KS M ISO2507-2는 p20에 보존되지만 p20의 ISO 병합 셀은 원본 Center보다 위에 있다. 높은 점수로 완료 판정하지 않는다.
- 다음 원인 경로: advance_row_cut의 일반 행 컷은 row_span1 소유자만 기록 → start_cut_covers_cell은 같은 시작행의 rowspan도 소관으로 판정 → cell_cut_window가 unrelated 단일행 슬롯을 rowspan의 su로 읽음 → is_in_split_row 때문에 공통 straddle cut/완료 프레임 Center 경로가 차단된다. 일반 행 공간과 블록 공간을 명시적으로 전달하고, rowspan은 앞 조각이 실제 소비한 물리 밴드로 공통 유닛 창을 계산하게 한다. reservation의 straddle_continuation_demand와 paint의 rowbreak_straddle_cut_units/resumes_inside_own_start_row에 같은 시작 컷 공간을 전달한다. 블록 소유자의 컷 슬롯은 유지한다. 새로운 px assertion/fixture는 추가하지 않는다.

- v51 retry 기존 경계 회귀50/50 PASS(exit0, 전체327.230초/실행7.447초,1677skip)다. Native 선택8쪽은 두 원본 모두 exit0/312.569초이며 HWPX p3/p4/p19/p20=99.68701/99.67973/99.46133/97.15814%다. freshWASM/전수 및 가운데 정렬 차이는 후속 보정에서 다시 검증하며 완료를 선언하지 않는다.


### 진행 중 보정 34: 일반 행 컷과 병합 셀의 소유 공간

- start_cut_covers_cell/일반 end 판정과 cell_cut_window의 행 슬롯은 row_span1만 소유한다. 같은 행의 rowspan은 start_row_height_override와 명시적인 start_cut_is_block으로 앞 조각 소비 높이를 복원한다. 기존 빈 컷/병합 전용 행과 BlockCut 슬롯의 소유 계약은 유지한다.
- 전달 경로: FragmentInput.start.start_cut_is_block → BlockRowScanVars → RowScanQuery.required_height → straddle_continuation_demand → rowbreak_straddle_cut_units/resumes_inside_own_start_row다. paint도 PageItem.start_cut_is_block에서 동일 helper에 전달한다. 시작 RowCut/끝 BlockCut이 섞인 조각은 끝 블록 슬롯을 보존하고 시작의 물리 prefix만 공유 cut으로 소비한다. 한 쪽 공간이 다른 쪽 공간을 대신하지 않는다.
- 최종 저장 plaintext span은 공통 가시 창의 마지막 내용이 완료되었을 때 기존 Center/Bottom 정렬을 적용한다. 절대 위치 clamp·문서ID·기대값 완화는 추가하지 않았다. source snapshot/source SHA는 source-code-v52/native-build-row-cut-span-ownership-v52.json에 남기고 관련 기존67개 및 원본 HWP/HWPX의 p3/p4/p19/p20부터 검증한다. 같은 행 회귀 #7226, 이어받기 요구 #6981, 시작/끝 보존 #6803, #6024, #6756을 기존50개에 포함한다.

- v52 첫 Native build exit0/256.852초, binary SHAf89d269356484284e05be8a2097f04c5b5c1ba924c87e7de6124dee1cb4584d5다. 실제 p20 트리에서 IDDV가 다시 보인다(scattered-p20-trace-v52/render_tree_020.json). 이 후보는 실패이며 원인/증거를 보존한다. RowCut 슬롯 오소유를 제거한 뒤 start_row_height_override가 있는 분기의 resolved 높이−잔여 높이에서 자기 상단 패딩을 뺀 예산이 첫 줄 점유보다 작아 공통 su0으로 돌아갔다. 저장 선언이 첫 물리 프레임만 나타내는 경우 그 차이만으로 앞 컷의 소비 내용을 복원할 수 없다. 실제 앞 RowCut의 padding 포함 점유 높이를 소비의 하한으로 함께 사용하고, 더 큰 원장의 빈 물리 밴드는 보존하는 공통 prefix 보정이 필요하다. 회귀와 최초 sweep은 실패 후보의 증거로 구분해 완료까지 보존하며 formal 기대값은 바꾸지 않는다.

- v52 기존67개는 exit100/전체349.258초, 실행20.526초로62PASS/5FAIL/2127skip이다. #6024 원본의 전체 쪽수106vs105, #7226 교육과정의414vs413, #6981 같은 원본의 종료 뒤 노동인권 소속, #6803의 이어받기와 완전 보존2검사에서 실패했다. 기대값은 유지한다. 실패 원본의 Print/Visual Sweep을 추가하며, 기존 #6803 5쪽 PDF 대조는 pinned 실패 binary에서 시작했다. #6024·#7226/#6981·#6803의 원본3개는 버전별 MCP Print 계약을 새로 확인한다.

- 실패 원본 #6803의 Native v52 전쪽5페이지 Visual Sweep은 exit1이며99.26183/93.32244/73.17339/34.27747/33.25166%다. 최신 실패 binary와 추적 독립 PDF의 출력 비교이고 최저33.25166%로 원본은90% 선행 조건을 충족하지 않는다. 테스트/기대값을 변경하지 않으며 실제 출력부터 보정한다(native-failed-6803-original-review-v52/6803-original/review/). 새 MCP Print와 freshWASM의 대응도 남아 있다.

- v52 retry1 build exit0의 실제 트리도 IDDV 중복을 유지하므로 실패 후보(source-code-v52-retry)로 보존한다. 앞서 적은 물리 차이만의 가설은 충분하지 않았다. 같은 행의 일반 RowCut은 별도 start_row_height_override 없이도 이미 시작한 병합 셀의 물리/내용 prefix를 소유한다. 빈 컷/병합 전용 행만 height override를 요구하고, 일반 비어 있지 않은 RowCut은 자체로 이어받기를 입증하도록 조건을 교정한다.
- #6803의 앞 eu와 뒤 su도 서로 다른 높이 공간이었다. 앞 helper는 cell_h를 사용했지만 뒤는 이전 행 점유와 실제 RowCut prefix를 사용한다. 새 rowbreak_cut_prefix_height는 동일 원점의 실제 이전 행 원장+accepted RowCut 점유를 산출하고 일반 걸침 helper의 끝 컷과 #6803 블록 시작/일반 끝 경로가 함께 소비한다. 끝 블록 컷 슬롯은 별도로 유지한다. 원본에 따라 조건을 좁히거나 px 기대값을 바꾸지 않는다.

- 원인 분리를 위해 pinned v51(보정33)과 v52 retry1에서 실패 원본 쪽수를 다시 읽었다. #6024는 두 후보 모두106쪽, 교육과정은 두 후보 모두414쪽, #6803는5쪽이다(failed-original-page-count-comparison-v52.json). #6024/#7226의 쪽수 실패를 새 컷 공간 보정이 유입한 회귀라고 단정하지 않는다. 기존 검토 후보에 이미 남은 실패이며 원본 출력 보정이 필요하다. 교육과정의 독립415쪽 기준 PDF는 기존 pr_7544_review.md에 같은 원문 대응과 쪽수 불일치가 명시되어 있다. 새 MCP가 queue 대기 중이어도 기존의 미충족·미검증을 그대로 보존한다.

- v52 retry2 Native build exit0/249.830초, binary SHAf5b1ac023d02adaf5deb90e1d195758d182d9d9f2f66b37a3050fc34abc3a5ca다. 원본 p20 실행 트리에서 IDDV 중복이 제거되고 ISO 두 줄 y112.5/133.5가 독립 Print112.286/133.406과 대응한다. 새 review PNG와 기존67개를 다시 실행한다. #6803 공용 JSON 탐색은 row_span/model 소유 정보가 없어 다른 표의 같은 row/col을 섞으므로 정식 소유 검증으로 사용하지 않는다. 원본의 typed 회귀2개로 확인하며 잘못된 탐색 집계를 not_evaluated로 교정했다.

- v52 retry2 HWP p3/p4/p19/p20 review는99.69210/99.72851/99.35269/97.44457%다. 최신 p20 review_020.png를 대화에 표시해 IDDV 중복 제거와 ISO 두 줄의 세로 Center를 확인했다. 같은 첫줄 Native “(ISO ”는 줄끝 공백을 포함해 x188.2/폭34.3인 반면 독립 Print 잉크는 x191.36/폭27.835다. 가시 텍스트·공백의 정렬 폭과 줄말 공백 소유를 후속 확인한다. 하단 괘선의 작은 차이도 남아 있어 수치만으로 확대 판독을 통과하지 않는다.

- v52 retry2 선택 Native HWP/HWPX8쪽 review는 exit0/336.111초이며 HWPX p3/p4/p19/p20=99.68701/99.67973/99.46133/97.58977%다. 기존67개는 exit100/전체346.771초/실행20.623초,64PASS/3FAIL/2127skip이다. #6803 이어받기·완전 보존2검사는 수정 전 v52에서 FAIL/수정 후 retry2에서 PASS로 복구했고 원본/기대값은 그대로다. 남은 #6024 쪽수와 #7226 쪽수/#6981 종료 뒤 소속은 앞 후보에도 남은 원본 실패로 계속 보정한다. typed 검사의 복구를 #6803 전체 피델리티 승인으로 승격하지 않는다.
- 정확한 retry2 binary에서 원본 두 파일52쪽 Native 전수 TSV를 시작했다(native-row-cut-span-ownership-full-v52-retry2). 소스 형상/입력/기준/명령/선택 PNG는 source-code-v52-retry2 및 명령 JSON/검토 폴더로 고정했다. freshWASM, 최종 Rust lint/전체 nextest/Studio, 원본3개의90% 선행 검증과 나머지 최초18개 범위가 아직 남아 있으며 PR 생성·승인·merge는 보류한다. source 보정의 로컬 체크포인트만 커밋한다.


### 진행 중 보정 35: 줄바꿈 구분 공백과 가운데 정렬의 가시 폭

- 원본 p20의 ISO3633 두 저장 줄과 독립 Print의 대응에서 첫줄 말미 ASCII 공백은 다음 단어로의 줄바꿈 구분자이며 가시 중심에 포함되지 않는다. Native는 글리프 뒤 공백 폭6.4px의 절반만큼 첫줄을 왼쪽에 둔다. 소스의 center_excludes_trailing_ws가 is_last_line_of_para만 허용한다는 가정이 원인이다.
- 경로: estimate_composed_line_width에서 원본 줄의 전진 폭 생산 → effective_text_width/기존 trailing_space_width_after_last_inline_object가 각 run의 실제 스타일로 구분 공백 점유를 계산 → x_start의 Center 정렬 원점 → emit_line_runs의 실제 glyph 원점이다. 문단 마지막 여부 제한을 제거해 기존 순수 텍스트 Center의 가시 폭 계약을 모든 줄에 적용한다. 문서ID/절대 px 보정은 넣지 않으며 텍스트·char offsets·줄 전진과 소유는 그대로 보존한다. TAC 개체가 있는 줄, 밑줄 말미/공백만 있는 줄은 기존 가드·helper를 따른다. #7081/#6173/#6028의 기존 회귀와 원본 Print 대조를 연결한다. 새 formal 검사/fixture/기대값은 추가하지 않는다.
- 앞 source6c041a692의 #6803 재출력 전쪽5페이지는 exit1로99.26183/93.32244/73.88708/34.42714/33.25166%다. typed 내용 보존2검사의 복구와 실제 배치 미충족을 구분한다. 이 원본도 먼저90% 출력 보정 대상으로 남기며 테스트를 바꾸지 않는다.

- 실패 원본3개의 MCP Print가 완료됐다. 모두 engine2020/11.0.0.9136, pdf_print_method0/hancom2020_pdf_driver_one_up이며 입력/다운로드 해시를 검증했다. 위임전결규정103쪽(PDF90eda86b2b4dfeabc601cd9b583408c6528a9efabf4e626fd6f2e28e3bcb2b97), 교육과정415쪽(PDF82fe8eefecbf3ad5176549976a8cd486ac0bb9c113e7ac34510a69ee6ccbf17f), #6803 5쪽(PDFb6941c58dde2b2af5c6d5103f549af11efb4e6e7b451436245d22f97ee0078cf)이다. references.tsv에 같은210개 형식으로3개를 추가한다. 실패 회귀의 원본103/415쪽과 현재Native106/414쪽 및 기존 기대105/413쪽을 각각 구분한다. 기대값이 Print와 다르다는 이유만으로 수정하지 않으며 실제 전체 출력을90% 이상으로 먼저 보정한다.

- v53 Native build exit0/270.637초, binary SHAdcbc04665f608394f57c8b83fc6a412697508ec23e4ffb0b43f9cfcf6b6aa524다. ISO 첫줄 x188.2→191.2px/독립 Print191.36px로 가시 중심 차이를 제거하고 줄 전진·문자열은 보존했다. HWP p19/p20=99.99671/97.72314%, HWPX=99.92627/97.86418%의 새 review를 직접 읽었다(native-center-wrapped-space-review-v53). 아직 하단 괘선의 작은 차이는 남는다. 기존7081 Center/Right·6173 TAC·6028 밑줄4/4 PASS(exit0/296.718초,실행0.113초/631skip)이며 검사·입력·기대값을 바꾸지 않았다.
- 앞 source6c041a692 HWP52쪽 전수 TSV 최저48.03726%/미통과13쪽/누락0이다. 미통과는22/23/24/28/29/30/31/39/48/49/50/51/52이며 v50의15쪽에서20/21쪽이 개선됐다. HWPX 전수는 계속 진행 중이다. 현재v53 재출력 p22는47.99365%의 직접 미충족이며 원인 조사는 다음 source 단계다.


### 진행 중 보정 36: 첫 프레임 밖의 원본 두 문단 행 컷

- p21/p22 독립 Print는237행의 “압축강도”/KS M ISO844를 p21, 다음 문단“(경질형 에폭시수지)”를 p22에 둔다. 실제 두 문단 LS는 각각0·LH1000·첫LS300HU이고 padding과 두 줄 점유/간격 합이cellSz2866HU를 정확히 닫는다. 현재p21 실행은 content budget14.1px에 first unit17.3px(글줄13.3+다음줄간격4)을 강제 선택하지만 가시 cutoff/paint에서도 간격을 남겨20.9px짜리 유효 줄 조각을 배제한다(scattered-p21-cut-trace-v53.log).
- 기존 native_saved_two_line_row_frame은 동일 shape를 Common.height의 첫 프레임 교차에만 허용해 후속 프레임의 컷을 놓친다. 첫 프레임 위치 질의는 유지하고, 실제 RowCut의 source 두 줄/정확한cellSz 계약은 별도 공유 결과로 산출한다. 저장 표가 첫 프레임을 넘는 근거는 각 행의 명시적 single-row 최대 선언 높이 합이라는 하한이다. single-row 없는 병합 전용 행은0하한으로 포함하고 새로운 높이를 추정하지 않는다. 이 하한이Common.height를 넘으면 소유된 프레임은 전체 표가 아니다.
- 공통결과는 native_saved_reset_cut_trailing_trim → advance_row_cut 예산 수용·row_cut_content_height/물리패딩 → row_step source-boundary keep → partial paint의같은 가시 컷으로 소비한다. 문단별0만으로 판정하지 않고 실제 원본dirty/합성LS/control/정확한선언상자/스타일 계약을 확인한다. HWP/HWPX 원본 둘과 기존6761/7379 등 경계 검사를 실행하며 새 fixture/기대값은 추가하지 않는다.

- source6c041a692의Native HWPX52쪽도 종료(exit1)됐으며 최저48.41629%/미통과13쪽/누락0이다. source 이전 후보의 전수 HWP/HWPX 정리는 native-row-cut-span-ownership-full-v52-retry2-summary.json에 있다. 현재source 단계의 최종 증거로 이전 전수를 재사용하지 않는다.
- 두 문단 행의 간격 trim은 부분 컷의 다음 unit이 남아 있을 때만 적용한다. 완료된 whole-cell 마지막unit은 이미 LS를 제외하므로 두 번 빼지 않는다. 기존 첫 프레임 위치 질의는 hwp5/원본 행-prefix 교차 계약을 그대로 유지하고 후속 RowCut helper와 별도로 호출한다. source-code-v54의 새 helper는90줄이며 source3파일+SHA를 새 build에 고정한다.

- v54 첫 build exit0/260.076초이나 원본 HWP/HWPX dump-pages에서 p21 end237/endCut없음, p22 start237/startCut없음으로 보정 효과가 없었다. readonly 원본 진단은 row237의 원본 두 LS, padding283+283, spacing_before/after0과 1000+300+1000+566=2866HU의 정확한 선언 상자를 확인했다(plain-row-frame-source-v54.log). 원인은 컷 trim의 호출이 아직 첫 선언 프레임 전용 native_saved_two_line_row_frame에 남아 있던 연결 누락이다. retry1에서 후속 RowCut용 stored_two_line_row_cut_frame을 호출하도록 수정한다. 첫 프레임 예약의 별도 인증은 유지하며, 효과 없는 v54를 시각 통과로 보고하지 않는다.

- v54 retry1 Native build exit0/255.321초, binary SHA cadc1391f0cb2750de5a3a5f50bc4892d69e3f9c27585494f40edd32beb44ec5다. 두 원본은52쪽이며 p21 end238/endCut[1,1], p22 start237/startCut[1,1]로 독립 Print와 동일한 원본237행의 문단 소유를 복구했다. table pi18의 일반 셀 row237에서 p21 압축강도/KS M ISO844, p22 (경질형 에폭시수지)/빈 완료 method를 직접 확인했다(scattered-row237-source-owned-cells-v54-retry1.json, independent-print-row237-text-pages-v54-retry1.json). JSON으로 rowspan 소유를 판정한 증거가 아니다.
- Native 선택12쪽 review exit1/469.618초다. HWP p3/p4/p21/p22/p23/p24=99.87432/99.96125/100.00000/94.98743/50.15023/56.14305%, HWPX=99.86229/99.92357/99.95350/95.71094/49.94848/55.94398%다. p22/p23 review PNG를 직접 표시했으며 p22 상단 표·뒤 제목은 복구됐지만 하단 첫 조각/다음 쪽 내용은 미충족이다. 앞 후보p22 47.99365%와 구분한다. fresh WASM과 원본 전체90%는 아직 미검증이다.
- 기존 경계67검사는64PASS/3FAIL/2127skip(exit100/339.041초,실행19.006초)다. 정상64개는 유지되며 실패도 기존 #6024(106vs105), #7226(414vs413), #6981 노동인권 소속 세 개 그대로다. 해당 원본의 전체90% 출력 선행 보정이 끝나지 않았으므로 검사·기대값·baseline을 변경하지 않았다.

### 진행 중 보정 37: 첫 저장 프레임의 잉크 뒤 빈 밴드와 문단 내부 이어받기

- 같은 원본 table pi26의 첫 선언 프레임은22340HU이고 row0..3의 독립 single-row 상자 합은16064HU다. row4 frequency/remarks의 같은 문단 저장 줄은4200→0HU로 재개하며 첫 프레임 마지막 잉크와 padding 끝은21830HU다. 선언 프레임에는510HU의 잉크 없는 밴드가 남고, 다음 줄을 순차로 소비하면23230HU여서 첫 프레임을 넘는다(logs/scattered-table26-declared-frame-v54-retry1.log). Print p22는 마지막 작업시/불가능시 줄을 p23 첫 행에 둔다. 직전 Native는 전체 행 fast path가 저장 reset을 무시해 p22에 모두 소비했다.
- 기존 stored_row_reset_closes_declared_frame은 ink+padding이Common.height에 정확히 닿을 때만 true여서 원본의 양수 빈 밴드를 부정한다. first frame의 마지막 가시 줄·선언 끝·다음 줄의 요구 끝을 구분하고, 실제 같은 문단 양수→0 reset과 원본 사다리·declared row 범위로 빈 밴드를 인증한다. 단순 좌표 clamp/문서ID/허용치 증가는 넣지 않는다. 생산 술어는 cell_units의stored_frame_break_before와 whole-row 우회에 공유되고 실제 advance_row_cut → saved_multirow_opening_frame_height의Common.height 예약 → physical row carry → partial paint로 대조한다. 첫 프레임을 실제 예산에 수용하지 못하면 정상 이월 경로를 유지한다.

- v55 Native build exit0/254.272초, SHA c44287969fb9090b78e2c9845d315970cca7ff5ba10eed17922389dad90b3a2e다. 원본 두 개52쪽이고 p22 table26 endCut[1,1,4,4], p23 startRow4/startCut[1,1,4,4]로 작업시/불가능시 줄의 Print 대응 쪽 소유가 복구됐다. 22–24쪽 Native review exit1/236.057초: HWP95.45728/96.77602/58.89865%, HWPX96.23749/97.06903/57.77669%다. p23/p24 review를 직접 표시했다. p23 하단 row22 물리 밴드는 여전히 작고 p24의 여러 행이 아래로 밀려 남은 원인이다.
- v55 기존67검사는62PASS/5FAIL/2127skip(exit100/339.763초,실행19.249초)이다. 앞64PASS/3FAIL을 그대로 말한 중간 집계 오류를 즉시 정정했다. 새 FAIL은 #6981 giant_straddle_cell_that_must_be_cut_is_not_grown(#6803 원본의 A9~10 칸 밖 글줄)과 #6935 real_collision_document_preserves_text_across_fragments(4vs3쪽)이다. 같은 원본/기준 PDF로 전쪽 Native Visual Sweep을 시작했고, 실패 검사·기대값·baseline은 유지한다(native-new-failed-originals-review-v55).
- readonly 원본 판정 진단(first-frame-blank-band-candidates-v55.log)은 새 범위 조건이 main table26 row4 col4/5뿐 아니라 #6803 pi6 row8 col4와 #6935 pi0 row4 col1에도 발동함을 확인했다. 원본 LS와 선언 높이가 한 셀의 경계를 설명하는 것만으로 행 전체의 물리 컷을 확정한 가정이 부족했다. retry1은 source ordinary-row의 미완료 owner들이 같은 ink/padding 끝과 같은 문단 내부 원점0 재개를 공유하는 결과를 확인한다. 완료 owner는 해당 source frame에 들어가며, 걸친 셀은 ordinary RowCut의 peer로 승격하지 않는다. 단일 미완료 owner의 느슨한 Common.height 대조는 기존 capacity/BlockCut 경로에 남긴다. 문서ID·수치 예외가 아닌 RowCut 소유 결과의 인증 변경이고, 기존 정확한 first-frame closure 계약은 유지한다. source-code-v55-retry1 두 파일+SHA에서 재빌드/동일 경계 재검증한다. 아직 최종 통과가 아니다.

- v55 retry1 build exit0/255.309초, binary SHA fff08b7b8e5807cf907247bb6db1155147bf3faa85670a7e12c76226f2c58f78이다. 같은 원본의 shared unfinished owners가 없는 #6935는3쪽으로 복원됐고 #6803는5쪽이다. 원본 main두 파일은52쪽과 p22 endCut[1,1,4,4]/p23 startCut을 유지한다(pages-*-v55-retry1.json). 67개 기존 검사 최종64PASS/3FAIL/2127skip(exit100/342.087초,실행19.191초)이며 새 유입2FAIL은 검사 변경 없이 PASS로 복구됐다. 기존 #6024/#7226 쪽수와 #6981 노동인권 소속 실패는 그대로 남아 있다.
- 새 main두 원본 p22–24의Native review exit1/224.973초이며 HWP95.45728/96.77602/58.89865%, HWPX96.23749/97.06903/57.77669%다(native-stored-first-frame-blank-band-review-v55-retry1). 앞 실패 후보 v55 이미지 재사용이 아닌 retry1 새 바이너리 재출력이다. source가 바뀐 두 실패 원본의 전쪽Native review도 exit1/163.269초다: #6803 p1..5=98.94637/93.43892/75.15690/32.30497/35.53280%, #6935 p1..3=85.79861/54.92011/94.06013%(native-new-failed-originals-review-v55-retry1). #6935의직전 e13 원본도같은85.79861/54.92011/94.06013%였으며 3쪽 복원과 피델리티 실패를 구분한다. #6935 p2직전 review를 직접 표시해 상단 남은 줄과 뒤 표가 밀리는 결함을 판독했다. 원본전쪽90%와fresh WASM 선행 검증은 미충족/미검증이고 formal검사/fixture/기대값/baseline은 바꾸지 않았다.
- 다음 main경계는 table26 row22에서 시작한 rowspan label/frequency의 물리 시작 밴드와 p24 이어받기다. raw single-row row22는1566HU이지만 Print p23 마지막 밴드와 Native의높이가 다르다. row22 col0 span15/col5 span13은 같은 문단두 LS0/0, col4 span9는두 문단LS0/1300이며 row23 이후 여러 행과 마지막 요구높이가 이를 소비한다. 아직 한 가지 원인으로 단정하지 않고 source-owned cut·physical row prefix·straddle_continuation_demand·최종paint를 추적한다. PDF의 반복되는 동일 문구를 좌표만으로 같은 paragraph owner라고 판정하지 않는다. 관련 direct PNG/trace는 기존폴더에 보존한다.

- source122dd1b62 fresh WASM wrapper --no-opt exit0/594.153초다. Docker 없음의 기존 진단 설정이며 optimized 배포 검증은 아니다. WASM SHA cf971b0729c348d407434c4934e8862f65e994aa8f355ad763f36bd4450b9aa4, JS7462392b9b4dadb36c0b60cc0afa9de3fc6e41a861b150010ea51f0a7974737a이며 root pkg/Studio public 각각 일치했다. pinned package wasm-pkg-stored-first-frame-blank-band-v55-retry1-122dd1b62에서22–24쪽 review와 원본52쪽 전수TSV를 시작했다.
- 같은 source의 Native HWP52쪽 TSV는 누락0/최저51.53884%(p52)/90%미만11쪽이다. 미달은24/28/29/30/31/39/48/49/50/51/52이며 앞6c 후보13쪽에서22/23쪽이 개선됐다. HWPX와fresh WASM 전수는 아직 진행 중이다. 현재 다음source 변경 뒤 이전 전체를 최신 결과로 재인용하지 않는다.

### 진행 중 보정 38: 저장 병합 셀의 첫 물리 밴드와 원본 내용 컷

- 독립 HWP Print p23 SVG의 괘선을 같은96dpi로 추출했다(independent-print-p23-row-bands-v56.json). 마지막 밴드는1041.578125..1080.260416=38.682291px이며 Native는20.88px만 예약했다. 원본 row22 col4의 연속 LS0/1300·LH1000·padding283+283의 첫 저장 내용은2866HU=38.213333px다. 같은 행에서 시작하는 col0/span15와col5/span13은 각각 같은 문단의두LS0/0·text_start 증가·동일column/width로 새 프레임 재개를 기록한다. 문단 간 원점0이나 한 owner만으로 row frame을 정하지 않는다.
- rowSpan9 col4의 선언15087HU는 현재 단일행 선언 합14094HU보다993HU 크다. 첫 물리 밴드를 single-row 선언1566HU로만 잡은 뒤 선언 잔여를 마지막 걸침 행에 더해 row29/30이 이미 늘어난다. paint D6981R row30의 need20.9/have180.3에서는 추가 성장이 없음을 직접 확인했다(scattered-p24-span-demand-trace-v55-retry1.log). paint에서 높이를 clamp/줄이는 보정으로 해결하지 않는다.
- source 공유 결과는 공동 첫 프레임에서끝나는 내용 prefix+실효 padding의 물리 최소와, 원점0에서 재개하는 각 source owner의첫단위 컷이다. HeightMeasurer의 ordinary-row 측정 뒤/rowspan 선언 배분 전에 같은 최소를 반영하고, LayoutEngine fallback도 같은 결과를 소비한다. straddle 컷·후속 요구높이·paint에는같은 source owner 컷을 적용해 밴드가커져도 다음 프레임의 글자를 미리 소비하지 않는다. 정상 단일 owner/합성LS/편집본/control혼재/전통적인닫힌rowspan 선언 경로는 원본 witness가 없으므로 이 source 결과가 없다. 작은 실제 경계와 기존67검사부터 대조하고, 관련 전체 Native/fresh WASM90% 전에는 formal검사/fixture/baseline을 추가·변경하지 않는다.

- v56 최초 Native build exit0/285.412초, p23–25 두 원본6쪽은 모두90% 이상이다(HWP97.96819/99.55059/99.99178%, HWPX98.07385/99.46439/99.92535%). 다만 직접 p23 review를 읽은 결과 row22 col0의 기층과col5의 정기 사용불가가 독립 Print p24 소유인데 p23에 함께 출력됐다. 점수 개선을 승인으로 판정하지 않는다. 기존67검사는64PASS/3FAIL/2127skip(exit100/362.370초,실행19.339초)이며 기존 원본 실패3개의 검사/기대값은 유지했다.
- retry1은 다른 시작 행의 컷이 새 병합 셀의 owner까지 소비했다는 가정을 제거한다. 첫 프레임 endRow=cellRow+1의 source owner 컷(0,1)은 앞쪽의 다른 행 startCut과 독립적으로 결정한다. 같은 셀의 시작 행에서 이미 컷/잔여높이가 있으면 기존 부분 소비 원장을 유지한다. 다음 행의 빈 startCut으로 재개할 때 su=1을 공유하고, 이후 끝 컷은 기존 소비 원장으로 계산한다. 정상 native 2행/2문단 계약은 그대로며 문서ID/px 예외를 추가하지 않는다. 실제 예약·배치 호출 모두 rowbreak_straddle_cut_units를 소비한다. 동일 원본 재출력과 기존67검사로 다시 확인하며 최초v56 캡처는 retry1 증거로 재사용하지 않는다.

- source122 Native 전수는 exit1/1973.968초, freshWASM 전수는 exit1/2212.610초로 둘 다 종료됐다. HWP/HWPX 각각52쪽/누락0, Native=freshWASM 전체 페이지 점수가 일치한다. HWP 최저51.53884%(p52), HWPX52.34972%(p52), 각각90%미만11쪽(24/28/29/30/31/39/48/49/50/51/52)이다. full-score-observation-122dd1b62-v56-complete.json 및 전수TSV로 연결하며 진행 중인v56 retry1의 검증과 구분한다.

- 다음 미달 구간의 원인 조사용으로 source122의 pinned Native를 다시 실행했다(native-scattered-next-boundary-review-v55-retry1, HWP28–31쪽). p28 review_028.png를 대화에 직접 표시했다. 독립 Print의 첫 모양 및 치수·포장용·시험방법 KS F4001에 대응하는 내용이 앞쪽에 없고, 하단 경계는 Native가 다음 시험항목을 먼저 소비하는 차이가 보인다. 점수55.49206%의 위치 차이만으로 분류하지 않고 table pi33의27→28쪽 저장 줄/셀 소유와 컷 원장부터 확인한다. 이 출력은source122 원인 조사이며 새v56 retry1의 검증으로 인용하지 않는다.

- v56 retry1 Native build exit0/265.239초, binary SHA 84757c57899e7d1106c22a501fd1f026dc95fba0cd084abb441e83e4a0ba8d10다. 두 원본23–25쪽 review exit0/208.641초: HWP98.38810/99.55059/99.99178%, HWPX98.52414/99.46439/99.92535%다. 새 p23/p24 review를 대화에 직접 표시했고, first owner는p23에만/다음 owner는p24에만 존재함을 table pi26,row22,col0/5로 연결했다(scattered-source-owner-observation-v56-retry1.json). 절대좌표를 formal 기대값으로 추가하지 않았다. CLI의-p가0기반이라는 점을 진단 command에 바로잡고, 페이지파일명으로 실제23/24쪽을 확인했다. freshWASM 및 기존67개 재실행은 아직 진행/대기이며 전쪽90% 승인은 보류한다.

- v56 retry1 기존67개 검사는64PASS/3FAIL/2127skip으로 종료됐다(exit100/344.754초,실행20.636초). 기존 원본 #6024 위임전결규정 쪽수, #7226 교육과정 쪽수, #6981 교육과정 노동인권 소속 세 실패가 유지됐고 추가 실패는 없다. 원본의 전쪽90% 선행 개선이 미완료여서 검사/기대값/baseline은 그대로다. Native 검증을 마친 source만 로컬 체크포인트로 고정하고 같은 head의 freshWASM 검증을 이어간다. PR 생성·승인·merge는 아직 하지 않는다.
- 같은 v56 retry1의 다음 경계 HWP p27 review는96.18082%지만 하단 row51의 새 포장용 콘크리트 평판 첫 행을 한컴보다 앞 쪽에 소비한다. 독립 Print는 row50에서 p27을 끝내고 row51을 p28로 이월한다. source Table pi33,row51의 rowspan11 owner는 문단/줄 원점이0/1500/3000으로 연속하고 KS F4001 owner는 한 줄이다. score와 직접 판독을 분리해 해당27쪽도 완료로 판정하지 않는다. scanner의 BLOCK_UNIT_MAX_ROWS=3과 실제 블록/예산을 원인 후보로 확인하며 행 수 제한을 곧바로 변경하지 않는다.

### 진행 중 보정 39: 저장 병합 셀의 연속 owner와 조각 최소 점유

- v56 retry1 HWP27–31쪽 새 review는 exit1/215.046초로96.18082/55.49206/64.77340/65.38166/69.69944%다. p27/p29 review를 직접 표시했고 p27 점수 통과와 소유 결함을 구분했다. 정확한 compiled source는 c06b6b32bc111906a47560565c24f5769edceb0d와5개 파일 SHA 모두 일치한다(source-commit-stored-span-opening-frame-v56-retry1.json). 새 head의 freshWASM 빌드는 같은 target/pr-review에서 순차로 실행 중이다.
- readonly 실제 MeasuredTable에서 pi33,row51 블록은51..62/229.68px이고3행 제한으로 보호 분류가 안 된다(scattered-table33-measured-blocks-v56-retry1.tsv 및 로그). 하지만 독립 Print p28→p29는row77..81의4행 블록을 실제로 쪼갠다. 따라서 페이지에 들어가는 모든 병합 블록의 통째 이월은 올바른 일반 규칙으로 입증되지 않았다. 행 수 제한을 단순히 확대하지 않는다.
- 독립 Print p27의 마지막 괘선은1060.119792px/마지막 row50, p28의 마지막 row77..79 조각은999.385416..1080.260416=80.875px이다. 그 안의 두 일반 행 경계는1020.322917/1041.098959이고 마지막 밴드는39.161457px여서 선언1566HU=20.88px와 다르다. row77 col0의연속 저장 문단LS0/1500/3000/4500,LH1000,padding283+283의 내용 점유는6066HU=80.88px다. 네 owner를모두 p28에 표시하고 p29의row80는본문 표면층 두께만 남는다. 단순 row합/각 unit의순차 높이로 내용 owner를 분할하면 독립 출력과 다르다. source owner prefix와실제 부분 조각의 물리 최소를같은 결과로 예약/배치해야 한다는 원인 후보로 연결한다(independent-print-next-row-band-observation-v56-retry1.json, scattered-next-owner-independent-bbox-v56-retry1.json). 이좌표는진단 근거이며formal px 핀이 아니다.
- row51의frequency 저장 prefix는LS0/1600/3100/4600,LH1000,padding566으로6166HU=82.213333px다. 첫ordinary row가남은 공간에 들어간다는 이유만으로 source prefix의첫 owner만 앞 쪽에서소비하면나머지 label/시험방법 소유가 어긋난다. 정확한 예약 후보는전체 블록 높이를강제하는것이 아니라 해당source frame의 unit소유와최소 점유다. 부분 조각의누적 높이·수용 실패컷/이월·최종paint분기에 실제로 전달되는지 분석하며 코드가 바뀌기 전까지 해결된 것으로판정하지 않는다.
- GitHub를 다시 조회해semanticist21 open24 non-draft/6draft, 신규·종료·head/draft 변경0임을 확인했다(github-open-pr-snapshot-v56-retry1.json). 초기18개와deferred6개 범위를 유지하며원격 변경은 하지 않았다.

- c06b6b32b freshWASM wrapper --no-opt exit0/575.819초, WASM230ba446f2a2466abd39898db7c0a66b4611a0961d7e11949f1e042ddac1488c, JS7462392b9b4dadb36c0b60cc0afa9de3fc6e41a861b150010ea51f0a7974737a다. root pkg/Studio public 해시가 각각일치했다. pinned wasm-pkg-stored-span-opening-frame-v56-retry1-c06b6b32b의두 원본23–25쪽 full review는 exit0/440.139초이며 Native와각페이지의 점수가동일하다(HWP98.38810/99.55059/99.99178%, HWPX98.52414/99.46439/99.92535%). 아직27–31등남은 배치 결함이 있으므로 전쪽 승인이나formal 회귀 추가·기대값 변경의근거로승격하지 않는다.
- 실제Studio 기존vite7700에서 새 Chromium context/캐시 비활성/goto+reload 후 원본HWP를로드했다. 응답buffer수집기가navigation 뒤끝나지 않은자체하네스 오류를관측해 동일하게살아있는브라우저에서CDP로문서52쪽/view52쪽/font loaded/canvas4와23/24쪽owner를직접읽었다. 실제http public WASM13,852,065bytes의SHA가pinned와일치했고동일owner분리가유지됐다(browser-studio-stored-span-opening-frame-v56-retry1-observed.json,oktrue). 최초수집기는제품FAIL증거로세지않으며원로그를보존했고,관측완료뒤자체browser만종료했다. 화면profile의실제로딩확인은Print Visual Sweep의시각승인과구분한다. 최종범위전체Studio/API/CI 검증을대체하지않는다.

- v57의 생산 결과는 같은 source cell의 모든원본LS가문단을넘어엄격히증가하는연속사다리의inkEnd+실효상하padding이다. 원점되감김/같은원점·dirty/reflow·control혼재·TAC·RowBreak외경로·선언cell.height와모순된사다리는새계약을입증하지않으므로기존컷경로를유지한다. 긴page-larger source도기존unit walker를유지하며,새 쪽의 유효 row budget에들어가는연속source frame만한 owner묶음으로예약한다. 빈 문단의LS도점유에포함한다.
- 실제경로: float_placement/stored_span_content_frame의원본점유 → LayoutEngine.stored_span_fragment_minimum의cell원점+내용최소 → scan_block_table_split_rows의행진입예산판정/최종fragment consumed+end_row_height_override → emit_table_fragment의partial_height/PageItem/record_stored_fragment_rows → stored_fragment_row_plan의수용행원장/renderer.layout의fragmentMeasurement → table_partial의마지막행높이/rowbreak_straddle_cut_units의같은실제prefix다. 원점계산에는cut cache대신paint의MeasuredTable 또는이미수용한fragment_row_heights를전달한다. record도원래MeasuredTable이아닌이번실제fragment계획을기록해닫힌span의잔여와다음owner가같은물리prefix를소비한다.
- 수용한마지막행에만source내용점유와실제행합의차이를기록한다. 모든중간row 후보에같은최소를더해중복예약하지않고,의미컷과물리높이Override를분리한다. 기존header의꼬리빈밴드fill조건은넓히지않으며,연속source가있는비header표의물리원장보존만같은계약으로확장했다. 먼저원본의실제27→28→29쪽컷/owner/빈밴드와기존67검사를보며,최종90%전formal회귀/fixture/기대값/baseline은변경하지않는다. 새분기의명시적start/end내용컷및page-larger경계는기존경로이고최종유효성검증은아직진행하지않았다.

- v57 최초build exit0/255.406초, binarya3f1241c14f03ee556f9f8e5035a1b19011183bc0219b569f5d3dcb4d7ef2711지만빠른원본컷검사에서main53쪽/교육과정416쪽으로서실패후보로보존했다. 최초경계차이는p3 pi9endRow9 vs기존10/endCut이며새p4가row9하나의partialcut만소비한뒤강제새쪽을만들었다. #6935는3쪽/#6803는5쪽/위임전결106쪽이다(pages-*-v57.json).
- readonly원본MeasuredTable대조에서pi6의61..84와pi9의0..9에는source 최소가실제physical을넘는owner가없다(continuous-span-first-mismatch-v57.log). 새p2 usedHeight는1009.1467→988.7733으로줄었다. floating 합순서의미세양수차이에end_row_height_override를만든것이숫자크기와무관하게기존stored header tail/frame선택을막은자체오류다. retry1은source 범위를좁히는대신원래비교의부동소수점합오차상한(EPSILON×두높이규모×합항수)을넘는실제점유증분일때만override를만든다. 실제source밴드18.24px를줄이거나px허용치/문서ID조건을넣지않는다. 새SOURCE_SPAN_FRAME진단을통해수용한증분을기록한다. 기대값·baseline은유지하며다시동일원본컷과smallNativeSweep부터실행한다.

- retry1 build exit0/265.443초(binaryd927d5d6512fc2e15b61e12c4657e4ce2b0bef370196f0c9fbb4b668e7bbe081)뒤같은53/416쪽을유지했다. 새진단은pi6 rows23..61의physical962.347/minimum962.800/last21.333으로실제증분0.453333px를보여서직전합산오차만의원인가설은불충분했다. 원인판정을이실행증거로정정한다. 부동소수점noise guard는수치로새override상태를만들지않는용도로유지한다.
- 실제누락소비분기는scan_table_fragment의stored header tail이다. end_row_height_override.is_none을내용완전소비의대용으로써서새source minimum의양수확장도압축/partialband처럼배제했다. retry2는빈endCut/완전행조건과기존독립sourceFrame/body예산검사를유지하고,override가없거나원래whole-row 높이이상의확장인경우그행의저장빈밴드도수용한다. 음수압축/실제unit컷은기존조건그대로다. 새source내용증분18.24px의row77..79는기존wholeRows/consumed관계에서빈tail대상이아니며새px/문서ID조건을추가하지않는다. source값→최종예약→PageItem→paint의override선택까지대조한소비지점수정이다.

- retry2 build exit0/257.224초/binary9bbfa9dcaf48ef25cf6e8b9dc318f5659ee0fd0bc5e69ae4f3c3dd5290865686에서HWP52쪽과p3/p4원소유가복원됐다. HWP의새p3/p4/p27..31은99.87432/99.96125/99.77061/99.14356/99.97446/97.15990/99.13203%이며p28 review를직접표시해첫label·KS F4001·하단row77..79의4개문단owner와다음쪽row80소유를확인했다. 기존67개64PASS/3FAIL/2127skip(exit100/350.207초,실행19.411초),검사/기대값변경없다. 하지만HWPX53쪽/교육과정416쪽이고freshWASM아직없으므로완료나새fixture근거로보지않는다.
- HWPX최초새차이는p5 pi9 row96에서나타났다. Native HWP의label/frequency마지막LS는0으로재개하지만HWPX원본XML에는4800HU로연속되어있다. 이는파서보정/수동LS가아니며version.xml은정식Hancom2020 11.0.0.2129,입력첫commit은7f6c770b8(Task1716)이다. styles59/61및대상row51/77의styles도widowOrphan/keepWithNext/keepLines=0이다. 원본저장LS의연속성만으로모든내용이한쪽source frame을소유한다는가정은두형식의독립Print와충돌한다. 형식/문서ID예외를붙이지않고그가정을제거한다(scattered-full-source-*-v57.tsv와원본XML).
- retry3의공통생산결과는연속source line의각prefix별ink/padding끝이다. 행진입은기존MIN_TOP_KEEP_PX정책을source내용점유에적용한최소prefix를예약하고,최종내용창은실제해당cell의남은fragment budget에들어가는최대prefix를선택한다. short owner는전체가같은최소이며page-larger/originalreset/dirty/reflow의기존walker계약을유지한다. 완료된cell을현재끝경계의걸친owner로사용하지않는다. 실제band의증분은같은마지막행override/누적예약/수용원장으로공유하며모든줄을첫프레임에강제하지않는다. 새source birth가설·수정방향이바뀌었으므로retry2의PNG를retry3의검증으로재사용하지않는다.

- v57 retry3 Native build exit0/256.813초, binary SHA59491481a5e833e0fd5c8b1f465388cbe3a2da83a5ebd2f6d965ffa7b4c07a6b다. 원본 HWP/HWPX가 모두52쪽으로 복구됐으며 p5 pi9 end99 / p6 start99로 실제 연속 HWPX 저장 줄의 쪽 경계 반례를 유지한다. p27 pi33 end51 / p28 51..80 / p29 80..126 / p30 126..171 / p31 171..214로 앞 HWP 개선도 보존했다(pages-*-v57-retry3.json). #6935=3, #6803=5, #6024=106, 교육과정=414쪽이며 독립 Print의3/5/103/415쪽과 구분한다. 기존 실패 기대값을 바꾸지 않았다.
- retry3 실제 RenderTree에서 pi9 row96 label은 p5 앞3줄/p6 마지막 규격 문단, pi33 row77 col0의4문단은 전부 p28/같은 셀 p29는빈내용이다(stored-continuous-span-page-ownership-v57-retry3.json). 독립 Print bbox도 실제 대응 셀의 글자를 확인한다(independent-print-prefix-page-ownership-v57-retry3.json). PDF 전체의 반복 규격 문자열 존재만으로 owner를 판정하지 않는다. p3 새 review99.87432%를 직접 표시·판독했고 표 외곽/행/하단 제목 위치가복구됐다. p3–6/23–25/27–31 선택 Native review와52쪽 전수TSV, 기존67검사가 진행 중이며 아직 최종 head의 freshWASM/전체90%·lint/전체회귀·원격 통합은 미완료다.

- retry3 기존67개 재실행은 exit100/137.222초/실행21.060초로64PASS/기존3FAIL/2127skip이다. 최초 handle75726은 컴파일 도중143 종료됐고 summary가없으며 Cargo/Rust가없음을확인한 뒤 rerun했다. 환경 종료를 test FAIL로 집계하지않는다(nextest-stored-continuous-span-reservation-v57-retry3-interrupted.json 및 rerun exit/log). 실패원본의검사·기대값·baseline은그대로다.
- retry3 p25는99.99178→94.64765%로낮아졌으며 새 review를직접표시해 마지막첫조각의밑괘선이18.72px 위로올라간실제차이를확인했다. pi29 row0..42의 source최소가949.907→950.360px/last21.333px로증가했고 saved_first_fragment_source_frame(1004.04px)을예약하는경로의end_override없음조건이complete row를잘못제외했다. continuation에서이미해결한것과동일하게 첫조각도minimum growth와compressed cut을구분한다. 기존source frame/내용컷/기존0.5비교는유지하며단순출력clamp나문서ID예외는추가하지않는다. retry4에서첫조각tail과두형식반례/기존67개를재실행하며retry3이미지를최신완료증거로재사용하지않는다.

- retry4 Native build exit0/282.480초, binary SHA6f21310e6cefc99be76ff8d5e8434cb0a603a6ff36781e3a5785196d66d098f7이다. 두원본52쪽을유지하며 p25점유가989.08→1007.8px로이전독립출력대응프레임을복구했다(pages-*-v57-retry4.json). 두형식p25새review는계속진행중이며기존retry3결과로최종승인하지않는다.
- 2026-10-09 새fetch에서upstream/devel=f0e7228f6→5168e3256ad58eda425d84fdd4d67e9b47bb74b1을확인했다. #7644/#7620의새상위저장표·중첩컷보정이실제소비경로에들어오므로현단계source/진단기록을체크포인트커밋후최신base로rebase한다. 기존Native/freshWASM결과는원래source의이력으로유지하고rebase후정확한head에서재검증한다. generated Studio rhwp.js의기존작업은보존한다.

- rebase완료head=3cfd4b6a2c39f13fc3b983834b85cfa4df5ba429/base5168e3256이다. pre/post82커밋, range-diff삭제/추가0이며변경표시1개는float helper의새upstream함수명주변context만바뀐것이다. Git본문의42개cherry-pick출처도순서/해시가모두유지됐다(rebase-v57-20261009.json, range-diff, rebase-v57-contributor-provenance.json). 기존StudioJS는7462392b9b4dadb36c0b60cc0afa9de3fc6e41a861b150010ea51f0a7974737a로정확히복원했으며stash4cc75327016aa2270367e1b73db331d6656b319b도보존했다. rebase초기동시git status의일시적index lock은이미없어진것과실행Git없음을확인한후동일rebase를continue했고commit누락없음을검증했다.
- 새base고정manifest/unit-tier정책검사PASS다(1458sources/6264static attrs/48targets, 4113units/297modules). 파생suite배정이달라졌으므로기존67함수의13모듈을새생성파일에서재매핑하고upstream의#7644/#7620/빈문단뒤charshape3모듈을추가해16모듈/13targets명령을준비했다(nextest-stored-continuous-span-rebased-v58-command.json). 과거suite번호를그대로재사용하지않는다. 새headNative v58빌드는진행중이며기존컴파일결과로rebasehead를통과판정하지않는다.
- 이전source retry3선택24쪽Native는최종exit0/961.799초이며두형식52쪽/p3–6/23–25/27–31측정누락0이다. HWP최저94.64765%(p25), HWPX최저95.36790%(p25);27–31쪽HWP최저97.15990/HWPX97.33927%다. p25의직접판독결함은retry4에서HWP99.99178%로복구했다. rebase중기준PDF가일시적으로없어retry4의HWPXp25실행은변환환경오류로미완료다. oldsource를최신head증거로인용하지않으며currenthead/원본해시/참조PDF를다시고정해재실행한다.

- rebase 후v58 Native build exit0/255.326초이며binary SHAee5c65148e9722b859be00e3e63e6f7e784ced271ee313b9192d7ff7c18c95c9이다. 컴파일된11개소스해시는정확한3cfd4b6a2commit과모두일치한다(source-commit-stored-continuous-span-rebased-v58.json). 새basehead에서기존67+상위3모듈의회귀와두형식p3–6/23–25/27–31Native review, 잔여직접차이p10/39/48–52review를재실행중이다. 이전retry3 HWP전수52쪽은미달6쪽(39/48/49/50/51/52),최저51.53884%이며그대로최종승인에사용하지않는다.

- 정확한 rebase head 3cfd4b6a2에서 73개 경계 검사가 exit100/417.677초/실행23.166초로 70PASS/기존3FAIL/2767skip 종료됐다. 이전 67개는 64PASS/3FAIL을 유지하며, 새 upstream의 세 모듈 6개 검사도 모두 PASS다. 실패는 #6024 쪽수와 #7226 쪽수/#6981 마지막 내용 소속 그대로다(nextest-stored-continuous-span-rebased-v58-exit.json 및 log). 원본의 독립 Print 전쪽90% 선행 조건을 아직 충족하지 못했으므로 테스트·기대값·baseline은 변경하지 않았다.
- 같은 v58 head에서 두 원본52쪽/p25 점유1007.8px, #6935=3/#6803=5/#6024=106/교육과정414쪽을 재확인했다(pages-*-v58.json). 현재 Native 선택 review와 잔여 직접 결함 review가 진행 중이며, Cargo 종료를 확인한 뒤 root wrapper로 fresh WASM을 순차 시작했다(wasm-build-stored-continuous-span-rebased-v58). 기존 7700 Studio의 package/public 동기화와 실제 reload도 새 package에서 확인할 예정이다.
- 잔여 p39의 독립 Print는 첫 data row가 source pi53 row15 “단열성”이고 현재 Native는 row16 “내화성”부터다. p38의 같은 표는 Native가 row15까지 앞 쪽에서 소비한다. 독립 bbox와 원본의 Common.height25152HU/행 높이, 실제 첫 조각 예산을 연결해 잘못된 소유를 추적 중이다(independent-print-page39-v58.txt, scattered-original-table-53-v57.dump.txt, pages-hwp-v58.json). 반복 규격의 텍스트 존재나 최근접 행 경계만으로 정답 컷을 인증하지 않는다. 아직 다음 보정 구현/검증을 마치지 않았으며 새 formal 검사·기대값은 추가하지 않는다.

- 새 v58 HWP 선택12쪽의 실루엣은 모두97.15990% 이상이고 p25=99.99178% 복구를 유지한다. 별도 잔여 review의 HWP p10/39/48/49/50/51/52는91.26492/59.42662/67.50142/77.18501/81.04910/53.91818/51.53884%다. p39 새 review PNG를 직접 표시·판독했으며 첫 data row 누락/뒤 표의 상향 이동을 확인했다. p10도90% 초과 점수로 이전 직접 차이를 면제하지 않는다. 두 sweep의 HWPX 단계와 fresh WASM은 계속 진행 중이므로 전수 완료·승인으로 보고하지 않는다.

### 진행 중 보정 40: 저장 첫 조각의 물리 프레임과 행 소유

- v58 Native 개선12쪽/두형식24쪽 review가 exit0/932.016초로 종료됐다. HWP 최저97.15990%, HWPX 최저97.33927%이며 p25는99.99178/99.92535%다. 잔여14쪽 review는 exit1/585.662초이고 p39 HWP59.42662/HWPX59.59609%, p48–52도 두형식 모두90% 미달이다. p39를 직접 표시·판독했다.
- 같은3cfd4b6a2의 freshWASM root wrapper --no-opt는 exit0/599.193초다. WASM d58b8545f12ef2b806a0534bb8b49a1874a4b233ee9275b330203c21855b1b8f, JS7462392b9b4dadb36c0b60cc0afa9de3fc6e41a861b150010ea51f0a7974737a이며 pkg/Studio public이 각각 일치했다. pinned package의 선택24쪽 review와 실제 Studio reload를 시작했다. optimized 배포 산출물 검증은 아니다.
- p38 pi53 원본 Common.height25152HU=335.36px인데 실제 row scanner 예산은336.7px/source_end16/overflow allowance20.2px다(scattered-first-frame-physical-budget-v58.log). 그래서336.333px의16행을 소비하고 p39를 row16부터 시작한다. 독립 Print p38 첫조각의 괘선은745.265625..1080.260416=334.994791px이고 row14 방음성 뒤 빈밴드를 포함한다. p39 첫행은row15 단열성이며, 저장 개체 프레임이 다음 행까지 소유한다는 최근접 추측이 틀렸음을 확인했다(independent-print-p38-row-bands-v58.json/svg, independent-print-page39-v58.txt).
- 기존 stored_row_boxes_exceed_first_frame의 원본 single-row 선언 합/spacing 하한이 Common.height를 넘으면 전체표가 그 상자 안에 있다는 가정은 성립하지 않는다. 인정된 저장 첫 프레임과 원본 무편집/무재조판·credible 조건을 연결해 그 물리 높이를 실제 row budget으로 소비한다. 행 소유는 현재 단일 row/unit walker가 정하며 최근접 행 강제 수용과 별도 overflow allowance를 사용하지 않는다. 내용 컷·최소 높이·선언 프레임의 빈밴드 예약은 실제 PageItem/paint에 전달한다. 전체 표가 선언에 들어가는 정상 대조군과 편집·각주·재조판 경로는 기존 계약으로 대조한다. 임의 px 허용치/문서ID/출력clamp/내용 숨김은 추가하지 않는다. 기존73개 및 원본 두형식/실패원본의 새 실행으로 방향을 확인하며 formal 회귀·기대값·baseline 변경은 계속 보류한다.
- GitHub 재조회는24non-draft/6draft로 범위와head가 유지된다(github-open-pr-snapshot-v58.json). 초기18개와 이후6개를 순서대로 처리하는 범위를 유지하며 원격 변경은 하지 않았다.

- v59 Native build exit0/262.397초, binary SHAaf53f124fa0b911267a22a209a9b7d416b2df86b4f31f7e6a3c2c7f6d23a25ca다. HWP/HWPX 각각52쪽이며 p38 table53 end15/p39 start15로 독립 Print의단열성 쪽 소유를 복구했다. p39 table56 end10/p40 start10과 기존 p3/4/23–31 내용컷도 유지했다. #6935=3/#6803=5/#6024=106/교육과정414쪽이다(pages-*-v59.json). 같은 소스의 p38–40 Native review와73개 기존 경계 검사를 진행 중이며 아직 최종 통과 판정은 아니다.
- v58 실제 Studio reload는 exit0/oktrue, servedWASM=d58b8545f12ef2b806a0534bb8b49a1874a4b233ee9275b330203c21855b1b8f/예상 해시 일치, 문서52쪽/글꼴loaded/23→24쪽 row22 label·frequency의 원본 소유 유지다(browser-studio-stored-continuous-span-rebased-v58.json). 별도 화면 동작 확인이며 Print Visual Sweep 승인과 구분한다. v58 package의 새 WASM 비교는 계속 진행 중이다.

- v59 선택6쪽 Native는 exit0/280.896초로 HWP38/39/40=99.75942/99.97808/100.00000%, HWPX=99.80002/99.97060/99.93943%다. p38/39 review를 직접 표시하고 선언 밴드/단열성 소유/뒤 표 위치가 독립 Print에 맞는지 확인했다. 하지만 기존73개는68PASS/5FAIL/2767skip(exit100/382.640초/실행19.929초)이고 새 FAIL은 #1133 두 형식의 첫조각 endRow4vs5다. tool session의 관측 exit143과 wrapper가 남긴 nextest exit100/명시summary를 구분하며 실행으로 확인한5FAIL을 기록한다. 검사·기대값은 유지한다.
- #1133은 합성이 아닌 실제 samples/issue_1133.hwp(ea03105fa7fa22398f3e370d222f37213e986ecca4b9b97d2847f1bbfe998d3b)/samples/hwpx/issue_1133.hwpx(2cf479d94a70d2a834ae7bc1f5ec5fa3d7ff895792cded48d0bf92ca19341bee)다. 독립3쪽 Print pdf/issue7382-regression-review/issue_1133-2020.pdf(46ca5b1abc40d9479e9eea7994b951ebb82d177ebc448cd09688170effd850ae), pdf/hwpx/issue_1133.pdf(ca066fdc5cda9fd8c47c9a29ba9a546e70c6acdaa96df4d6c66e0b8e02cdbc45)로 전쪽 Native 비교를 시작했다(native-new-failed-originals-issue1133-v59). 두 입력 모두3쪽이지만 첫조각 행 소유가 달라졌으므로 쪽수 유지로 통과 판정하지 않는다.
- #1133 첫5행 선언/측정 합614.0667px가 source602.7333px보다11.3333px 크지만 마지막 row4의 전체 내용은 남은58.2933px에 들어가는지 별도의 공통 complete-cut 높이로 확인할 수 있다. 첫 v59는 선언 상자와 내용 최소를 같은 것으로 취급해 이 unused physical space를 절대 불가로 보았다. retry1은 전부 소비 가능한 일반행의 물리 나머지를 source_complete_frame_last_row에 전달해 기존 scan/paint가 같은 높이를 사용한다. span crossing은 별도 continuation owner/cut의 요구이므로 완료행 상자에서 그 내용을 추측하지 않는다. Main row15의 최소20.88px는 남은19.9067px에 못 들어가므로 실제 이월을 유지한다. 조건만 더 붙여 대상을 좁히지 않고 내용 점유와 선언 공간을 구분하는 원인층을 수정한다. 원본 모든 관련 쪽의 Native/fresh WASM90%가 끝나기 전에는 formal 검사/기대값/baseline을 바꾸지 않는다.

- source3cfd4b6a2 v58 freshWASM 선택24쪽 review는 inner exit0/1124.250초이며 terminal session의외부143과구분한다. 두형식52쪽/측정누락0,12쪽씩의 Native/freshWASM 점수가모두동일하다(native-wasm-selected-score-parity-v58.json). 최저HWP97.15990/HWPX97.33927%이고새v59소스나미달잔여쪽을통과판정하는증거로재사용하지않는다.
- 첫v59의새실패원본#1133 전쪽Native는exit1/168.248초다. HWP p1/2/3=96.19594/94.11056/30.55666%, HWPX=96.21383/94.11150/30.55666%로실제 p3 내용/위치회귀를확인했다. 독립기대값이나px검사를바꿔실패를없애지않으며retry1의공통완료행최소/물리나머지소비연결로출력을복구한다. 실패PNG와원본은보존한다.

- retry1 Native build exit0/254.735초, binary SHA661381aa64bace8e058d4b01512afc1fb96b3752a4720587a17887b4e4251a4d다. Main두형식52쪽/p38 end15/p39 start15와#1133 두형식3쪽/first0..5/next5..8을복구했다. #1133새전쪽Native는 HWP96.19594/98.92603/99.86388%, HWPX96.21383/98.92668/99.86388%이며 main38–40 점수도앞후보와같다. 하지만교육과정은414→411쪽으로바뀌었고73검사68PASS/5FAIL/2767skip(exit100/360.678초/실행20.461초)이다. 새FAIL은#7226인접예산의실제본문넘침과#6981전남조례83쪽소유다. source입력변형/실문서검사를구분하고테스트·기대값은바꾸지않는다.
- retry1의잘못된가정은'선택한행뒤에span이안이어지면전체내용이끝났다'였다. row_complete_cut_content_height는현재행에서시작한single-row owner의요구만계산하며,그행에서끝나는이전rowspan owner의미소비유닛을포함하지않는다. 작은반환높이로끝나는span행을압축하면뒤block·glyph실제배치가본문을넘을수있다. retry2는실제active owner집합이전부현재행의ordinary owner일때만이완료행최소를소비한다. 끝나는/이어지는span의owner·유닛컷은기존walker의실제결과에남기며추측하지않는다. SourceFrame전체heightbudget규칙과1133ordinary행의물리나머지는유지한다. 실제원본/경계/독립Print재실행에서main39와1133복구를보존하면서새회귀가사라지는지확인한다.

- retry2 Native build exit0/255.425초, binary SHA9d7e743c0dbd1fe119f82976fa9b26ff70b674d41f347919024eb2f0cc852d12다. source-code-v59-retry2와4개소스해시로고정했으며,정확한후보에서기존73개/원본Main38–40/새실패원본#1133전쪽6페이지를재실행중이다. 수정전실패·이전Native개선자료를새후보통과로보고하지않는다. #1133절대px검사의의미교정은원본두형식의Native/freshWASM전쪽90%및독립기대값대조후에만진행한다. 정확한쪽/행/내용소유검사는유지한다.

- retry2 최종 기존73개는 exit100/364.897초/실행18.753초로70PASS/기존3FAIL/2767skip이다. 새#1133두검사와#7226인접예산/#6981빈밴드검사는구현수정만으로PASS복구됐다. 정확한검사·기대값·baseline은그대로다(nextest-stored-partial-frame-budget-v59-retry2-exit.json/log).
- 같은retry2의Main두형식38–40쪽review는exit0/223.104초, HWP99.75942/99.97808/100.00000%, HWPX99.80002/99.97060/99.93943%다. 새실패원본#1133두형식전쪽review도exit0/169.647초, HWP96.19594/98.92603/99.86388%, HWPX96.21383/98.92668/99.86388%다. 원본SVG와독립PDFSVG를같은96dpi좌표·200%배율로재렌더한vector-zoom-p39-v59-retry2.png에서단열성/시험방법/개폐반복성/시험빈도·괘선을직접판독했다. 큰위치·줄소유차이는복구됐고작은원점/획차이는별도계측과구분한다. 진단crop은formal절대px기대값이아니다.
- 교육과정414쪽과정상대조#6935=3/#6803=5, 위임전결106쪽을재확인했다. 실패원본의독립Print415/103쪽과모든쪽90%조건은아직미충족/미검증이므로검사·기대값변경및PR통합을보류한다. Main전체미달48–52쪽과나머지초기18개범위,최종Rustlint/전체nextest/editedAPI/원격CI·merge·후속처리는남아있다. 현재소스와검토기록을로컬체크포인트로커밋하고정확한head의freshWASM/원본3쪽및영향3쪽을재검증한다.


### 진행 중 보정 41: 후반 표의 원본 줄 경계 대조

- 지침 변경 여부만 확인하라는 요청에서는 `upstream/devel...e5f66600c`와 미커밋 diff의 AGENTS/CLAUDE/manual/tech 지침 변경이 없음을 읽기 전용으로 확인했다. 기존 visual_fixture_evidence의 원본 Native/fresh WASM90% 선행 조건과 의미 검사 계약을 유지한다.
- 같은 e5f66600c의 pinned Native9d7e743c로 Main 두 형식47–52쪽 review를 새로 실행했다(exit1/408.300초). HWP98.20888/67.50142/77.18501/81.04910/53.91818/51.53884%, HWPX98.14675/67.01272/77.04138/80.89954/53.24873/52.34972%이고 전체52쪽씩이다. 첫 조각 보정으로39쪽이 개선됐지만48–52쪽의 남은 배치 결함은 복구되지 않았다(native-stored-partial-frame-budget-remaining-review-v59-retry2/). 이전 v58 PNG와 최신 재출력을 구분했다.
- 독립 Print47쪽에는 row99의 ‘중금속 함량(6가 크로’ 첫 줄이 있고48쪽 첫 데이터는 ‘뮴)’이다. 현재 Native의47쪽 pi59는59..99/endCut=[],48쪽은99..144/startCut=[]여서 row99 전체를 뒤 쪽으로 이월한다. 두 genuine 원본의 row99는한 문단/두 저장LS로ts0→12, vpos0/0, 같은column/segmentWidth10612HU, LH1000/LS600을가지며 dirty 수동입력으로 만든 자료가 아니다(scattered-row99-boundary-original-evidence-v59.json 및 두 원본dump). 실제DIAG_SCAN은 content budget17.2px, padding7.5px에서firstunit21.3px/endCut[1]을선택하는 시도를 보였다. 선택된 내용 컷의 간격/물리 높이와 orphan/최종 예산 소비 경로를 원인 후보로 추적한다. 원본의 동일0 위치만으로 새 프레임을 선언하거나 허용치를 늘리지 않으며, 정확한 분기·일반성·비적용 반례를 확인하기 전에는 완료로 판정하지 않는다.
- #1133 Native 2/3쪽 canonical review를 직접 열어 앞 flow table, 전형표0..5, 다음쪽5..8, 중첩 유의사항, 뒤 내용과 표 외곽을 대조했다. 이 Native 판독은 freshWASM 선행 조건을 대신하지 않는다. 정확한 기존 소유 검사는 유지하며 절대px 기대값 교정은 같은 원본 두형식의 Native/freshWASM 전쪽90% 확인 뒤에 진행한다.
- GitHub 읽기 전용 재조회는 비draft24/draft6, 신규·종료·head/draft변경0이다(github-open-pr-snapshot-v59.json). 초기18개 우선과 deferred6 별도 검토 범위를 유지했다. 원격 push/PR/approval/merge/postmerge는 수행하지 않았다.

- e5f66600c freshWASM wrapper --no-opt은 exit0/578.307초, 소스해시14개 변화0이다. WASM69f22597370ec3172b5e0654531bd0223a0bab7e038507d4a98c1c4acac835c0/JS7462392b9b4dadb36c0b60cc0afa9de3fc6e41a861b150010ea51f0a7974737a가root pkg/Studio public과일치했고pinned wasm-pkg-stored-partial-frame-budget-v59-retry2-e5f66600c에보존했다. 실제Studio새브라우저/goto+reload/캐시비활성으로52쪽/글꼴loaded/canvas 및 servedWASM해시 일치를 확인했다(browser-studio-stored-partial-frame-budget-v59-retry2.json,oktrue). 이는Print시각통과와구분한다.
- 첫WASM두sweep는디스크ENOSPC로환경실패했다. 파일쓰기실패로stored_zero_origin.rs가비워진자체편집오류는즉시e5f66600c 원본과동일하게복구했다. 실패로그/빈exit기록의환경원인을보존했고제품회귀로세지않았다. 종료된과거WASM실행의중복SVG는같은크기와SHA-256을확인한뒤같은내용의hardlink로공유해원래경로/바이트를보존했다(space-recovery-identical-svg-v59.jsonl). 임시로압축한과거SVG1개도압축해제해원래경로/해시로복구했다(space-recovery-v59.json). 공유target/pr-review와현재작업자료는삭제하지않았다. 공간확보후e5pinned #1133전쪽과Main38–40쪽을새space-retry1경로로재실행중이다.
- v60후보의생산계약은같은원본문단의인접저장줄/전진하는textStart/동일양수폭·열의0→0재시작이다. 원본rowbox합이Common첫개체frame을넘는경우그첫box만으로뒤쪽줄프레임을배제하지않는다. 원본문단사이로컬원점/dirty/reflow/다른열·폭/완전개체상자/합성LS는새근거가아니다. source재시작→cell_units의stored_frame_break→ordinary advance_row_cut/native_saved_reset_cut_trailing_trim→row_cut_content_height의같은가시끝→row_step orphan판정및실제split_total예산→fragment의내용컷·물리높이→table_partial paint로연결한다. 같은내용의Native/HWPX인증줄은같은종료간격을소비한다. rowspan소유완료나꼬리blank예약을추측하지않고실제unit walker를유지한다. 새분기는미검증후보이며테스트·baseline을바꾸지않고기존원본컷/대조군부터재실행한다.

- #1133 e5pinned 두형식 전쪽 freshWASM은 공간재시도 exit0/265.294초다. HWP96.19594/98.92603/99.86388%, HWPX96.21383/98.92668/99.86388%로Native모든값과동일하고쪽수3씩이다(wasm-new-failed-originals-issue1133-v59-retry2-space-retry1/). 새WASM3쪽canonical review를직접표시했다. 이후v60 source는별도후보이므로이전e5결과를그후보의최종시각통과로승격하지않는다.
- 최초v60 Nativebuild는 exit0/276.773초, binary59b4731ad45a88689f33f02fe05a76ef5718bd9a25a6743a2f713bc41027b934다. 원본빠른덤프는Main52/#1133각3/#6935=3/#6803=5/교육과정414/위임전결105쪽이다. Main47→48은여전히row99통째이월이며출력복구를선언하지않는다. 원본row19에는single-row cell 선언이없어raw row높이하한0이들어있다. 기존prefix의0배제는row99가이미첫Commonframe을지났다는충분한하한도지워버린다. 독립선언prefix하한166511HU가첫frame24829HU보다이미크므로row19의모르는양수물리높이는이판정을뒤집지못한다. retry1은0하한을정확한opening컷인증에는쓰지않으면서알려진앞공간의하한에는보존한다. merged-onlyrow의0을실제빈공간이나내용완료로추측하지않는다. 수정후다시Nativebuild/원본컷/경계검증부터진행한다. 새formal검사·기대값변경없다.

- e5pinned Main38–40 freshWASM도 공간재시도 exit0/471.918초로두형식52쪽/선택3쪽씩 누락0이다. HWP99.75942/99.97808/100.00000%, HWPX99.80002/99.97060/99.93943%로Native와모두동일하다. #1133전쪽과합한12페이지 parity/최저90%를native-wasm-original-score-parity-v59-retry2.json으로대조했고Main39freshWASMcanonical review를직접표시했다. 후반미달이나미검증초기18개전체범위를통과로바꾸지않는다.

- retry1 Native build는 exit0/264.055초이며 binary SHA는 b6ad53abf25dbbdbc1babfc6f00ab124946c9b5524c2d18f1a4bfe4fe3f9df28이다. 소스 3개 해시를 source-code-v60-retry1에 고정했다. 원본 빠른 dump에서 두 형식은 52쪽이며, p47 pi59 59..100/endCut[1] → p48 99..145/startCut[1], p49 145..191, p50 191..237, p51 pi59 237..238/pi62 0..40, p52 pi62 40..54이다. 독립 Print의 row99 두 줄 소유 경계를 복구했다. #1133은 각각3쪽, #6935=3/#6803=5/교육과정414/위임전결105쪽이다(pages-original-countercheck-v60-retry1.json). 이는 쪽수·내용 컷 관측이며 시각90%나 정식 회귀 통과 증거와 구분한다. 현 후보의47–52쪽 두 형식 Native review와 기존73검사를 병행 실행 중이며 Cargo는 하나로 제한했다. 기대값/fixture/baseline 변경은 없다.

- retry1 기존73개는 exit100/365.046초/실행21.514초로69PASS/4FAIL/2767skip이다. #6024쪽수검사는106→105로다시PASS지만독립Print103쪽과전쪽90%는여전히미충족이므로출력완료로보고하지않는다. 기존#6981종료소유/#7226쪽수2개외에#6761의rewind_without_ladder_continuation_is_not_split_evidence/later_confirmed_rewind_does_not_authorize_an_earlier_cut이새로실패했다. 실제원본뒤사다리를일부러깨뜨린독립계약반례이며기대값변경으로없애지않는다. 실패테스트의기준원본samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp(SHA398d03a5d5e4d6e857086be532d6d9ed0cec9c8ad06f95c17bbb7f83056ae860)과독립PDF103쪽(pdffile명2020/실제CreatorHwp2022 0.0.0.0, SHA f8e5c0408e221080ede9a9a67b153d02d792d22961c738e46749641f32a32e79)을확인했고관련39–40쪽Native원본출력부터비교중이다. 수동변형의정답을원본PDF로대신하지않으며계약과원문피델리티를구분한다.
- retry2는선택한0→0쌍뒤의저장줄이있다면그재시작의LH+LS전진과일치하는지생산단계에서확인한다. terminal두줄은뒤사다리가없으므로그검사와구분한다. 원본row99두줄은그terminal경계다. 행의orphan수용은완료된형제하나가다른미완료셀을입증하지못하므로모든미완료ordinaryowner가실제선택컷의같은source경계를입증해야한다. cross-paragraph기존계약은유지하고in-paragraph0원점인증을공유한다. 더뒤의다른유효후보로현재선택컷을인증하지않는다. source/frame요구를정정하며문서ID·px허용치·추측span종료를추가하지않고검사·baseline을유지한다.
- retry1 Main47–52두형식Native review는 inner exit1/478.510초이며외부session143과구분한다. HWP99.81596/99.98427/100.00000/99.84891/80.03252/100.00000%다. HWPX의전쪽선택값도증적에보존했고미달은p51=82.38518%다. 48쪽canonical review를표시해첫‘뮴)’줄, 삭제표밴드와뒤래커도료의소유·괘선·간격복구를직접대조했다. 51쪽canonical review에서는pi62첫프레임의원점/누적괘선차이가남아있음을읽었다. 같은원본hostLS7396HU=98.613333px와현재flow97.28px의차이로savedfirstframe의인증이배제되는지실제측정/배치소비를추적한다. 허용치를넓혀인증하거나clamp로맞추지않는다. 최종쪽52의단독100%로51의미달을면제하지않는다.

- #6761 기준 원본 Native39–40쪽은 exit1/48.573초이며103쪽으로독립PDF와쪽수는같다. 관련쪽점수92.48720/71.53246%로40쪽미달이다(native-new-failed-originals-issue6761-v60-retry1/). 새40쪽canonical review를직접표시해이어받기표의반복머리행·글줄·후속문단의간격차이를읽었다. 정상원본검사PASS를90%피델리티로바꾸지않으며,수동사다리반례검사나px기대값을수정하지않는다. 같은원본freshWASM/원문40쪽출력보정은남아있다.
- retry2 Nativebuild는exit0/256.772초이며binarySHAaeaa58c73a1e52dda4fe55a7cd13026f8f21f886d0432bf5d90571b5b3c39067이다. source-code-v60-retry2/소스3개해시로고정했다. 빠른원본컷에서Main52쪽/row99 endCut[1]→startCut[1]을유지했다. 현후보의같은Main47–52쪽Native와기존73개를재실행해새반례FAIL이사라지는지확인중이다. 이전retry1시각개선은최신후보의검증으로재사용하지않는다.

- retry2의 기존73개 재검사는 exit100/358.857초/실행19.969초로71PASS/기존2FAIL/2767skip이다. 새#6761반례2개는검사·기대값변경없이PASS복구됐다. #6024가105쪽으로PASS지만독립Print103/전쪽90%는아직미충족이며검사PASS를원문시각승인으로보고하지않는다. 교육과정은415쪽으로독립Print와같지만기존#7226의413쪽핀과#6981종료쪽소유검사는실패한다. 같은원본의전쪽Native/freshWASM90%및독립소유대조가끝나기전까지그검사·baseline을바꾸지않는다.
- retry2 Main47–52 Native 재검증은진행중이며HWP6쪽의점수는retry1과같다. HWPX도완료되는값을해당최신run에보존한다. 현후보소스/검토기록을로컬체크포인트로고정하고같은정확한코드의freshWASM을검증한다. Main51/#6761원본40/나머지초기18개미달및최종lint·전체nextest·편집API·원격CI/merge·후속처리는남아있다. Source3개해시와컴파일snapshot을commit에대조해귀속시키며생성JS와진단/로그는stage하지않는다.

- retry2의Main47–52두형식Native최종review는exit1/477.744초이다. HWP99.81596/99.98427/100.00000/99.84891/80.03252/100.00000%, HWPX99.79842/100.00000/99.69468/99.57791/82.38518/99.99428%다. 두형식각6쪽중미달은51쪽하나이며52쪽전체소유를유지한다. source-commit-stored-later-zero-origin-frame-v60-retry2.json에서컴파일소스3개와commit eb29a7a9ce171c2cc5dfd9ff4689eb7a0d64eaae의바이트해시가모두같음을확인했다. 같은정확한head의freshWASM wrapper는공유target에서순차실행중이다. 독립Print51쪽괘선좌표는independent-print-p51-row-bands-v60-retry2.json/SVG로보존했고추가근거없이flow/paint를clamp하거나px상수로맞추지않는다.


### 진행 42 — 반복 머리행 조각의 실제 물리 하단과 수용 원장 대조

- eb29a7a9c fresh WASM은 exit0/574.324초, 소스16개해시불변이다. WASM SHA053c420ce12fad9ae5d05309099fd0416a8807f40587ad82f1a4b0a5d5fd2492 및 JS7462392b9b4dadb36c0b60cc0afa9de3fc6e41a861b150010ea51f0a7974737a가루트pkg/Studio public과일치한다. actual Studio 새브라우저/goto+reload/캐시비활성으로서빙해시·52쪽·글꼴loaded·canvas/오류0을확인했다(browser-studio-stored-later-zero-origin-frame-v60-retry2.json,oktrue). 이는독립Print시각승인과구분한다. 같은pin의Main47–52와#1133전쪽/#6761원본39–40Native/freshWASM비교를시작했다.
- Main51잔여원인을최종paint까지추적했다. 이전50쪽pi59 191..237의예약partial_height1005.3867px를accepted_row_boxes에기록하지만layout/table_partial.rs의최종page_cap_fold가1.3333px를접어실제상자는1004.0533px다. 이물리차이를선언33483HU의row218..237종료span에서이미사용한공간으로잘못차감해51쪽종료행이25.7067px로축소된다. 원본뒤LS3876HU/5476HU/7396HU와현재pi62flow97.28px가1.3333px어긋나첫Common frame인증도배제된다. native-physical-fold-before-v61.log 및 native-print-p50-row-band-drift-v61.json으로관측을보존했다. Print A4축척의일반누적차이와이별도수용/paint차이를구분한다.
- 보정후보는기존single_cell_page_fragment_bottom 생산결과를원본반복머리행의continuation행예산에서도소비한다. source/body하단→기존물리page cap→prepare_table_fragment_budget 행예산→scan의실제마지막밴드→record_stored_fragment_rows 원장→stored_fragment_row_plan 종료span잔여→PageItem/MeasuredTable→table_partial 최종높이순서다. footnote/zone의더작은가용은유지하고새100HU/px상수나허용치완화를추가하지않는다. 첫조각/편집/reflow/반복머리행없는경로는기존계약을유지한다. 끝유닛의내용컷과선언빈밴드는분리하며앞반례#6761두개/기존73개/원본전체쪽수와47–52의새Native로먼저대조한다. 아직미검증후보이며검사·기대값·baseline은변경하지않는다.

- 최초v61후보Nativebuild exit0/267.375초/binary3b0176019b23284f5d6baa50d180c3fba26d211e78ce64f4f263b5e84db52cee다. 빠른원본검사는Main HWP54/HWPX53으로독립52쪽과충돌했다. #1133각3/#6935=3/#6803=5/#6024=105/교육과정415쪽은유지됐다(pages-original-countercheck-v61.json). 이를보정완료로보고하지않는다. 실제logical advance가선택할내용컷예산에서physical inset을먼저빼는가정이잘못됐으므로제거한다. 해당무효후보의nextest컴파일은원본쪽수회귀가확인된뒤자체소유cargo-nextest/cargo/rustc만SIGTERM했고최종exit-15/107.021초이며test summary없다(cancel-reason.json). 출력공백때문에중단한것이나testFAIL이아니다.
- retry1은기존scanner의내용컷선택/요구높이를유지하고refit후내용완전소비행의빈tail예약에서만같은physical cap을쓴다. logical줄전진과실제빈밴드를분리하며원래높이를수용한뒤paint에서다시확장하지않는다. 행의최소요구보다작게압축하지않고기존컷·declared source·dirty/reflow인증은유지한다. 정확한physical마지막행override가수용원장과paint로함께전달되는지앞반례/새원본컷/시각검증부터재실행한다. 최초후보의page수/시각을retry1근거로재사용하지않는다.
- #6761원본40의eb29Native/freshWASM은둘다71.53246%다(39쪽92.48720%). 40쪽SVG에‘돋움’이Noto Sans KR ExtraLight(SHA67b4003e2be99ea44a7c957e4b35acde1b5e6e82a54fe195c5598c3c617bc2e3)로대체되어있고독립PDF에는Dotum이실제로embed되어있음을확인했다. win10-ted의Windows설치gulim.ttc를읽기전용복사하여독립face이름/해시를verified-ms-font-supply-v61/provenance.json에기록했다. 해당새경로를명시한출력으로글꼴공급차이를따로대조하며위치결함이나90%gate를면제하지않는다.

- eb29의20개대응페이지Native/freshWASM점수는모두동일하다(native-wasm-original-score-parity-v60-retry2.json). Main47–52각6쪽의freshWASM최종exit1/645.276초이며미달은51쪽두형식뿐이다. #1133각3쪽은Native exit0/170.867초,freshWASM exit0/253.546초로최저96.19594%다. #6761두쪽Native exit1/50.765초,freshWASM exit1/102.934초이며미달40쪽71.53246%다. 새WASM48쪽canonical review99.98427%를직접표시해상단‘뮴)’의소유와삭제밴드/뒤본문·괘선을대조했다. 후속v61후보의완료나초기18개전체통과로승격하지않는다.

- Windows gulim.ttc그대로인font supply는돋움export에반영되지않아기존점수그대로였다. TTCollection의Dotum원본face를독립sfnt TTF로추출한명시경로는실제SVG에Dotum(embedded SHA1c08ea8777fe0f775c4139d7ff4f68886d7f5608c52dd4581a22bf19175417cf)을embed했다. 공급파일SHAd090c4cb35ec18af0938ba0f0cf2c4d92e210a61c309eb533aaa122720ffb2a5와embed의glyf/hmtx/hhea/maxp/OS2동일·rawcmap차이및bestUnicodeCmap비교는issue6761-dotum-embedded-font-proof-v61.json에기록했다. 해당eb29 Native재비교 exit1/65.175초의39–40쪽점수는[(39, 92.43318), (40, 72.36214)]%다. 글꼴공급만으로해결됐다고보고하지않으며실제표원점/행/후속문단차이와freshWASM최종검증은남는다. TTC원본·face추출절차/해시는standalone-faces-provenance.json에보존했다.

- retry1 Native build exit0/254.956초/binarycf1536031259e270be069cf7b44fddcdbc3769f1a4270cf84a10fdf2248e2896이며소스2개해시불변이다. 두원본52쪽/47→48 row99부분컷및51 pi59 237..238/pi62 0..40을유지했다(pages-main-countercheck-v61-retry1.json). 직접RenderTree/드리프트에서pi59종료행25.7067→27.04px와pi62 typesetflow97.28→98.6133px,원본Common888.84px의source_frame인증복구를확인했다(native-physical-fold-after-v61-retry1/ 및log). 하지만실제pi62paint는y190.8/높이879.0px이며독립Print192.427/887.833과차이가남는다. 기존첫blank-band 수용의override없음조건/별도saved_opening_frame 생산조건과후속흐름의outerbottom소비까지추적중이다. source_frame인증복구만으로51쪽출력완료를선언하지않으며추가원점/높이소비보정과재검증이필요하다. 현재기존73개재검사/47–52새Native는진행중이다.

- retry1 HWP51쪽Native canonical review는80.03252→95.67443%로개선됐고직접표시했다. 하단괘선/빈밴드차이가남아95%점수로완료를면제하지않는다. HWP47/48/49/50/52는99.81596/99.98427/100/99.84815/100%다. HWPX전체선택과기존73개최종summary는진행중이며현코드freshWASM은아직없다.
- 최신fetch에서upstream/devel=6f66932a73ee6fd05a74a83bdc438c9499a89a12를확인했다(upstream-base-advance-v61.json). 초기대상#7612가83131ecd1으로별도병합됐고후속기록이upstream에들어왔다. 이는이번스레드의remote merge로보고하지않는다. 현재컴파일/원본sweep의기준5168e3256은유지해해당결과를정확하게완결하고체크포인트커밋후새base로rebase한다. 새base소스의최종gate는별도로재실행한다.

- retry1 최초 nextest handle81936은 외부exit143으로 컴파일 중 종료됐고 test summary가 없다. 재감사에서 Cargo/nextest/rustc 실행0을 확인한 뒤 남아 있던 running metadata를 terminal/environment_interrupted/innerexit 미상으로 정정하고 interrupted.json을 보존했다. 소스2개 해시 불변을 확인해 공유cache에서 동일73개 명령을 rerun1 재실행한다. 제품FAIL로 집계하지 않으며 실행 중인 작업을 중복 재시작하지 않았다.

- GitHub최신조회는open non-draft23/draft6,새PR0/head·draft변경0이며이전24개중#7612만닫혔다(github-open-pr-snapshot-v61.json). #7612 API state=MERGED/mergeCommit=83131ecd186bd7483671b7886d659ecb4fd380ca를확인했다(github-pr7612-merged-v61.json). 초기18개의기여자적용이력은유지하며아직열린초기17개검토와유보6개후속순서는바꾸지않는다.

- retry1 기존73개의동일command warm-cache 재실행은최종exit100/126.460초/실행20.661초로71PASS/기존2FAIL/2767skip이다(nextest-stored-header-physical-page-cap-v61-retry1-rerun1-exit.json 및log). #6761두negative계약도유지됐고새FAIL없다. 남은#6981종료소유/#7226쪽수는원본전체Native/freshWASM90%선행조건미완료로검사·기대값·baseline을변경하지않았다. Native47–52전체선택review의최종값을완결한뒤같은소스해시의로컬체크포인트와최신upstream rebase를고정한다.

- retry1 Native47–52두형식review는최종exit0/498.070초이다. HWP99.81596/99.98427/100/99.84815/95.67443/100%, HWPX99.79842/100/99.97037/99.85027/96.23354/99.99428%다. 두형식선택12쪽누락0/각전체52쪽이며새HWPX51 canonical review도직접표시했다. 90%통과와남은하단괘선·첫source빈밴드/뒤흐름차이를별도로기록한다. 현후보의freshWASM/전수TSV·최종lint/full nextest와다른초기PR미달은아직미완료다. 현단계2개Rust파일/통합기록을체크포인트커밋하며generated Studio JS/출력로그/진단산출물은포함하지않는다.

- 체크포인트89b1d00c8b202b267b27c43b5e84c9e8474cb4c8을6f66932a7위로rebase한head는b0678b4631a8627b32cc3f12464b824f8c7d0106다. pre/post85커밋·42개cherry-pick원출처가같은순서로유지됐다. src/crates/Cargo/build/tests/scripts의pre/post전체Gitdiff는0이며range-diff의삭제/추가2쌍은이미upstream에병합된#7612두기여패치가빈커밋으로보존된결과다(rebase-v61-range-diff.txt 및rebase-v61-20261009.json). Studio JS는원래SHA7462392b9b4dadb36c0b60cc0afa9de3fc6e41a861b150010ea51f0a7974737a로복원됐고stash4dce188fad06e67ecddf11de5c86ec8992725a9c도보존했다. 새base의manifest/unit-tier정책검사를준비하며51쪽직접차이/#6761원본40쪽과나머지미달보정을이어간다. 새head최종gate/원격통합은아직완료하지않았다.


### 진행43 — 인증된 첫 물리 프레임과 마지막 후속 흐름의 소비 연결

- 새base6f66932a7의prepare/manifest --check 및unit-tier --check는PASS(1458sources/6264attrs/48targets,4113units/297modules)다. 현재head b0678b463에서추가소비보정을시작한다.
- 독립원본pi62 Common66663HU=888.84px/hostLS7396HU와Print192.427..1080.260px프레임이근거다. 기존stored_partial_fragment_height가그첫상자를이미인증하고행예산888.84를반환하지만scan의내용완전소비prefix0..40은878.7733px다. 별도saved_multirow_opening_frame_height는intra-row endCut만수용해여기서는없고,emit이내용컷없는첫상자의10.0667px빈밴드를버린다. 또한원본반복머리행continuation의종료outerbottom은typeset이예약하지만확정placement없으면paint후속y에빠진다. 이전headerbody의원장이선택한실제행높이/후속물리흐름과다르게소비되는문제다.
- 기존producer의인증결과를FragmentBudget에명시적으로전달한다. refit완료후완전한행prefix이며내용minimum을늘리는unusedband만동일마지막행override/누적예약/수용원장/paint로보낸다. 내용컷을추가소비하거나현재원점을clamp하지않는다. 일반intra-row reset/중첩block컷의기존경로는유지한다. 첫개체프레임의확정tableTop과원본반복머리행continuation의tableTop/occupiedBottom은같은budget결과에서계산해기존ParagraphFloatPlacement로두소비지점에전달한다. 이미있는placement를재해석하거나배치뒤고정px로맞추지않는다.
- 현재별도후보이며기존73개의정상/negative계약·Main52쪽/47–52컷과두형식Native를먼저검증한다. 앞95%점수로남은괘선차이를면제하지않고새formal시험/fixture/baseline을추가·변경하지않는다.

- #6761원본40의직접vector대조에서Print 표상단136.010416px/Native132.28px,원본outer.top283HU=3.773333px를확인했다(issue6761-independent-row-origin-v62.json). table높이는Native599.7px/Print598.229168px로비슷하며전체frame원점차이가먼저다. 기존column_rowbreak_fragment_opens_outer_top의Para분기는object_only_saved_table_anchor와prior-rowspan없음을필요로하고,shared intra_paragraph_rowbreak_reopens_outer_top은host.textempty만허용한다. 실제원본은이전쪽에서소비한표제host가있고양쪽ordinaryowner의유효intra-para새프레임컷으로재개한다. 단지빈host가아니라는이유로개체의새세로프레임여백을잃는다. 기존의원본host-line유효성producer와현재선택된sourcecut을연결할후속보정을검토하며수동LS사다리반례/일반걸침rowspan에는새프레임을추측하지않는다. 현재v62Cargo가끝나기전Rust소스는수정하지않는다.

- v62 Native build exit0/257.102초/binary26c10fe2242f9e5e2eba288681e709a0c0e79da014f611a49397d82a23f9a182이며소스5개해시불변이다. 두원본52쪽/#1133각3/#6935=3/#6803=5쪽을먼저확인했고나머지빠른원본검사와두형식51쪽Native review를진행중이다. v62pin은후속원본호스트조건보정과구분해보존한다.
- #6761후속후보는현재선택된유효intra-para새프레임컷에원본호스트유효성을연결한다. host가빈단독Table일때의기존계약과,stored_head_host_lines_are_valid가인증한표제줄/한개의Table·notes만있는원본을같은세로프레임으로처리한다. 원본제어슬롯/줄기하가없거나선행글줄이개체와겹치는host는넓히지않는다. 실제source cut을가진ordinaryowner가새쪽에서재개한다는근거없이prior-rowspan전체에여백을추가하지않는다. 같은intra_paragraph_rowbreak_reopens_outer_top의결과를budget와paint가소비하고,그뒤resolved_table_top/확정placement의덮어쓰기도대조한다. 원본39–40의새Native/정확한Dotum공급및73개의사다리반례부터재검증한다.

- v62 빠른 원본8개은Main각52/HWP1133=3/HWPX1133=3/#6935=3/#6803=5/#6024=105/교육과정415쪽으로모두유지됐다(pages-original-countercheck-v62.json). 두형식51쪽Native review는exit0/83.911초로HWP99.91930/HWPX99.88665%다. 새HWP51 canonical review를직접표시해‘아.기타’제목과다음표원점/하단·마지막행중앙정렬이복구된것을대조했다. Native RenderTree의후속blank/title원점은127.3/148.6px,다음표는192.7px/높이889.1px이며독립Print의축척과같은영역에서읽었다. 아직현소스freshWASM및73개최종/전쪽TSV승인근거는없다. v62의전체52쪽Native TSV와v63의원본host확장후보빌드를진행한다.


- v63 Native build는 exit0/267.567초, binary `afb026b65e78d933658c8292a253978dfcd4dd2fa5ae8b23ed2e02fecc2b742e`이며 8개 source hash 불변이다(`native-build-stored-title-host-source-frame-v63.json`). 정확한 Dotum 공급에서 #6761 원본39–40 새 Native sweep은 exit0/55.172초, 92.43318/96.48485%다. 40쪽은 글꼴만 바로잡은 이전72.36214%에서 개선됐고, tabletop136.1px/독립Print136.010416px 및 뒤 본문을 canonical `native-issue6761-stored-title-host-source-frame-v63/issue6761-original/review/review_040.png`로 직접 표시·대조했다. 39쪽도 직접 표시했으나 첫 조각의 하단 괘선 차이가 남아 점수로 완료를 면제하지 않는다. 현source fresh WASM은 미실행이다.
- v63 기존73개 집중 nextest는 최종 exit100/385.524초/실행22.496초로71PASS/기존2FAIL/2767skip이다(`nextest-stored-title-host-source-frame-v63-exit.json`, 대응log). 실패는 #6981 `completed_terminal_cut_does_not_allocate_an_empty_page`와 #7226 `the_fix_does_not_add_a_page`이며 새 실패는 없다. #6761의 수동 사다리 negative 두 계약은 유지됐다. 원본전체 Native/fresh WASM90% 선행 검증 전 검사·기대값·baseline을 변경하지 않았다. fmt check도 통과했다.
- 같은v63 pin의 빠른 원본9개는 Main HWP/HWPX52쪽, #1133 HWP/HWPX3쪽, #6935=3/#6803=5/#6024=105/교육과정415/#6761=103쪽이다(`pages-original-countercheck-v63.json`). #6024의 Print103과 actual105는 미해결이며 기존검사105PASS를 fidelity 승인으로 사용하지 않는다. Main47–52 두형식과 #1133 전쪽 새Native 경계검사는 진행중이다. 이전v62의Main 전쪽 TSV도 별도pin으로 진행하며 현v63 결과로 바꾸어 쓰지 않는다.
- 최신GitHub read-only snapshot은 non-draft23/draft6이며 v61 대비 새PR/닫힘/head·draft변경 모두0이다(`github-open-pr-snapshot-v63.json`, `-delta.json`). 최초18개 적용이력/현재열린17개와 유보6개 후속검토 순서를 유지한다. 이 스레드의 원격 push/PR/merge/close는 미수행이다.
- #6761 39쪽 직접 판독 후 `native-issue6761-p39-drift-v63.log`에서 첫 상자 `source_frame=185.28`, 예산185.3, 실제내용174.7, row3종료컷과 40쪽 `start_cut=[1,1]`를 확인했다. 원본Common13896HU=185.28px의 빈 하단 밴드가 보존되지 않는다. 기존 `saved_multirow_opening_frame_height`의 원본 내용컷·선언상자 인증과 `first_fragment_blank_band`의 object-only/빈host 선택을 대조한다. visible-title host에도 새 in-paragraph stored frame 컷과 실제 원본host line 인증이 있으면 같은 빈 밴드 소유를 적용하고, 마지막행 override→수용원장→예약→paint가 함께 소비하도록 후속 보정한다. 유닛 소유는 바꾸지 않으며, 원본39–40와 기존negative/정상 경계를 다시 검증한다. 현재8개 source의 체크포인트 후 다음 단계에서 구현한다.


### 진행44 — 표제 호스트의 첫 저장 상자 원점과 빈 밴드

- 체크포인트 `8f3c1e0db94f38195b192a216817a3ea6e8a54c9`에 v63의8개Rust source와 통합기록을 고정했다. 컴파일한8개 SHA와 commit blob의 SHA가 모두 같다(`v63-build-to-commit-proof.json`). generated Studio JS는 포함하지 않았다. 이 커밋의 focused71PASS/기존2FAIL 및 Native 선택scope는 최종수용 판정이 아니다.
- #6761 원본39쪽 독립Print vector는 table top836.682292/bottom1021.760416px, 높이185.078124px다(`issue6761-independent-first-frame-v63.json`). 원본Common13896HU=185.28px와 대응하며 v63 실제174.9px의 하단 차이를 입증한다. Native 첫 원점833.8px에도 차이가 있다. TABLE_SPLIT_AVAIL의 동일 예약 원점은 cur702.7+outer.top3.773333+vert_offset_overhead−1.106667px인데 확정placement가 없어 paint가 outer.top을 잃는다.
- 기존 `saved_multirow_opening_frame_height`가 인증한 첫 상자와 `row_cut_starts_intra_paragraph_stored_frame`의 실제 새프레임 컷을 연결한다. 호스트는 `stored_rowbreak_outer_frame_host`의 원본제어슬롯/유효LineSeg 근거로 수용한다. 별도empty-text 추측은 제거한다. 기존 `first_fragment_blank_band`는 독립선언/원본컷/요구높이·가용예산을 그대로 검사하며, 인증된 완전prefix 또는 내용컷있는 첫상자 모두 같은 tableTop/occupiedBottom placement를 생산한다. 실제 내용유닛을 늘리지 않고 마지막행 물리override/원장/예약/paint/뒤흐름으로 전달한다. 기존placement가 있으면 보존한다.
- v64후보는 `fragment/emit.rs`만 후속변경하며 fmt check 통과 뒤 같은target/pr-review Native build 중이다. v63의기존검사·페이지수·시각을 새후보 통과로 재사용하지 않는다. 원본39–40/73개사다리negative와 Main/#1133/기타원본 경계를 새pin으로 재실행하고 fresh WASM을 이어간다. 검사·fixture·baseline 변경은 없다.

- 최초v64 Native buildはexit0/269.637초/binarye3d878a747addcb2f084ce06bde109777e8189884b00cb21ecdf9242d3b1a42e다. 빠른9개원본쪽수는v63와같고39쪽height185.5px로복구됐다. 다만top833.8px가남는다(`native-issue6761-p39-drift-v64/`,log). 예산에이미prepared.host_placement가있어emit fallback origin은미사용이며, `block/prepare.rs`의`unconstrained_host_placement → for_first_fragment`가originalouter.top을빼는실제생산지점을확인했다. 기존margin복구는body-filling/column-anchor만소비해valid원본head-title Para anchor를놓친다. v64 retry1은그생산지점에서기존`stored_head_host_lines_are_valid`로인증된원본RowBreak title anchor의outertop을보존한다. dirty/reflow session/합성줄은확장하지않는다. 행예산은같은준비placement를읽고paint는ctx의확정tableTop을읽으며뒤에`resolved_table_top`이선택되는것까지대조한다. 단순origin clamp나문서전용수치는추가하지않는다. 검사와freshWASM은retry1소스에서처음실행한다.

- v63 pin의두Native 경계작업은최종exit0으로종료됐다(`native-original-boundaries-v63-exit.json`): Main47–52두형식12쪽은최저99.79842%, 단계456.333초; #1133전쪽두형식6쪽은최저96.19594%, 단계160.815초다. source_unchanged=false는작업시작후v64소스를편집한관측값이며, 작업은불변v63실행파일만사용했다. 재검사한binary SHA와8f3c1e0db commit의8개source hash가build metadata와같다. 해당결과를v64후보에귀속하지않는다.
- v62별도pin의Main HWP전체52쪽TSV는누락0/최저93.25821%(10쪽)/90%미만0이다. HWPX전쪽작업은진행중이며v63/v64전체scope의최종승인근거로재사용하지않는다.
- 후속freshWASM/전쪽검증의공간확보를위해실행중인작업이없고summary가있는v49/v50의종료된4개검토폴더에서동일파일명·size·SHA의SVG296개만hardlink공유했다. 경로/bytes/증적을모두보존했고physical8,777,682,944bytes를확보했다(`space-recovery-identical-closed-svg-v64.jsonl`, 최종232.105초). 첫안전검사의self-shell오탐으로mutation전멈춘뒤actualscript argv를검사하도록고쳐재실행했으며다른경로/cache/source는정리하지않았다.

- v62 전쪽원본작업은timeout1800초에서최종exit124로종료됐다. HWP52쪽TSV는완결됐으나HWPX는38쪽raster만있고나머지는미측정이다. 종료handle/실제process종료를확인한뒤동일v62불변pin의39–52나머지14쪽만별도tail TSV로진행한다(`native-stored-partial-frame-physical-extent-tail-v62-exit.json`). 시간제한종료를제품FAIL로집계하거나미완료scope를90%승인으로표시하지않는다. 현v64 retry1 전쪽은별도3600초의새출력으로처음부터104쪽을실행한다.
- v64 retry1 Native build는최종exit0/263.203초/9개sourcehash불변,binary `4428c9fdbd4e24f5f3b59386f612aedf0572f3128fe1f8ac9158d214b8f4bf3c`다. 빠른9개원본쪽수는이전과같고#6761원본39쪽bbox는top837.6/height185.5px로독립Print836.682292/185.078124와같은영역에서대조한다. 생산된준비placement의outer.top보존이최종resolved_table_top까지전달됐음을확인했다(`native-issue6761-p39-drift-v64-retry1/`). 아직새PNG직접판독/점수와73개최종결과는미완료다. 새39–40Native,Main전체104쪽Native TSV(두형식각52쪽),같은73개nextest를실행중이며freshWASM은Cargo순차게이트이후다.

- v64 retry1 #6761원본39–40 Native sweep는최종exit0/55.513초/9개source hash불변이며39쪽98.94265/40쪽97.45412%다. canonical 두review를직접표시해첫표하단과마지막행/뒤쪽반복머리행·이어받는‘함유혼합물’줄소유/괘선·후속본문을같은영역에서대조했다. 40쪽bbox는136.1/599.7px로유지되면서첫행글줄의원점도개선됐다. `native-issue6761-stored-title-host-opening-frame-v64-retry1/` 및exit/log에보존했다. freshWASM은아직없으므로최종승인아니다.
- 사용자가처음지적한Main3쪽도현v64 retry1로새캡처했다: HWP99.89467/HWPX99.84971%. HWPcanonical `native-stored-title-host-opening-frame-p3-review-v64-retry1/scattered-hwp/review/review_003.png`를직접표시해‘인장시험’/첫표전체행·다음표/뒤문단과Print의원점을대조했다. 이전v41의56.53%이미지를현결과로재사용하지않았다. 나머지Main전쪽TSV/#1133전쪽은진행중이다.
- 동일v64 retry1 기존73개nextest는최종exit100/406.611초/실행22.003초,71PASS/기존2FAIL/2767skip이고9개소스해시불변이다(`nextest-stored-title-host-opening-frame-v64-retry1-exit.json`,log). #6761negative두계약포함새FAIL없다. 남은#6981종료소유/#7226쪽수의원본교육과정415쪽전수Native/freshWASM90%선행증거는아직없어검사·기대값·baseline을변경하지않았다. 현재두Rust파일과통합기록을체크포인트커밋하고rootfreshWASM으로다음출력경로를검증한다.

- 체크포인트 `fe201fa05f25ee744fb465a8b8a7348d38576d18`의9개소스SHA가v64 retry1 Native build의9개snapshot SHA와모두같다(`v64-retry1-build-to-commit-proof.json`). 이head로rootwrapper의freshWASM web/no-opt진단빌드를시작했다(`wasm-build-stored-title-host-opening-frame-v64-retry1.json`). 최적화배포산출물검증은아직없으며Native/freshWASM최종시각·Studio확인은별도남는다. 현재Rust/test소스는동결하고공유target의Cargo는한개만실행한다.
- v64 retry1 Main3쪽두형식Native작업과#1133전쪽두형식Native작업은모두최종exit0/107.573초및223.774초로종료됐고9개소스해시불변이다(`native-original-boundaries-v64-retry1-exit.json`). #1133의6쪽최저96.19594%/누락0이며앞v63결과와같다. 검사변경은없다.
- v62같은불변pin의HWPX tail39–52는exit0/302.271초다. 기존1–38/새39–52의PNG52쌍을전부Pillow.verify하고중복PDF39–52bytes도같음을확인해별도완결scope에hardlink보존했다(`native-stored-partial-frame-physical-extent-completed-v62/provenance.json`). 첫결합에서Poppler숫자padding을세자리로추정해찾기실패한자체오류는파일명숫자로쪽을대응시키고재실행해수정했다. Native/PDFbytes를변경하지않았다. 전체52쪽TSV/90%미만0을같은역사source의증거로기록하며새v64전체head승인으로대신하지않는다.

- fe201fa05의rootfreshWASM web/no-opt빌드는최종exit0/599.703초/9개source hash불변이다(`wasm-build-stored-title-host-opening-frame-v64-retry1.json`). pin은`wasm-pkg-stored-title-host-opening-frame-v64-retry1-fe201fa05`,WASM SHA`5da9a953cb1094c6f02a36038c7eee461ebddc3512ae7c3117d016457e00b044`,JS SHA`7462392b9b4dadb36c0b60cc0afa9de3fc6e41a861b150010ea51f0a7974737a`이며pkg/public이같다. 새WASM이publicJS의추가6개runtime import를실제로요구함을WebAssembly.Module.imports로확인했고node syntax도통과했다(`studio-js-required-wasm-imports-v64-retry1.json`). 필수binding이므로generated publicJS를검증기록과함께커밋한다. 아직실제Studioreload/시각비교와최적화배포산출물검증은없다.
- 현v64 retry1 Main HWP전체52쪽Native TSV는누락0/최저93.25821%(10쪽)/90%미만0으로종료됐다. HWPX전체는진행중이다. 새10쪽canonical NativeHWP93.25821/HWPX94.98450%도직접대조했으나하단괘선과마지막행글줄에남는차이가보인다. 90%점수로해결완료를선언하지않고Git동기화후그물리밴드소비경로보정을이어간다.
- 실패#6981의`build_page_render_tree(377)`/‘노동인권’가정은정본PDF의페이지375–381텍스트에서노동인권이380–381쪽에있는것과충돌한다. Native같은구간도노동인권380–381쪽이다(`education-terminal-page-pin-independent-text-v64-retry1.json`). 관련‘선언문작성’경계는별도시각대조가필요하다. 텍스트추출을fidelity승인으로대체하지않으며원본415쪽전수Native/freshWASM90%선행검증전테스트·기대값·baseline을수정하지않는다.
- 사용자의커밋→upstream/devel동기화→rebase→계속지시에따라새fetch를완료했고upstream/devel은기존6f66932a7 그대로다. local devel은local-only0/behind65,현재review는upstream-only0/local87이다. freshWASM이종료됐으며원본전체Native는고정실행파일에서계속된다. 소스checkout없이localdevelref를fast-forward fetch로갱신하고현재review의rebase를실행해각SHA/clean여부를기록한다.

### 진행45 — 완전한 행 내용 컷과 첫 표 상자의 물리 공간

- 요청한 Git 단계는 `9b658ed6563a9c51c39ea170d0df117fb485d5eb` 커밋 뒤 완료했다. local devel/upstream/devel 모두 `6f66932a73ee6fd05a74a83bdc438c9499a89a12`, review rebase는 already up to date이고 작업 트리는 clean이었다(`rebase-v64-20261009.json`). 원본18개/유보6개 검토 순서와 적용이력은 유지한다.
- v64 retry1 Main 전쪽 Native는 exit0/2061.454초/소스9개해시불변으로 끝났다. 두형식 각52쪽/누락0/미달0이며 최저 HWP93.25821/HWPX94.98450%(각10쪽)다. 실제10쪽의 하단 괘선 차이가 남아 최종승인으로 쓰지 않는다(`continuation-state-v65-after-sync.json`).
- 수정 전 실제 원본 HWP의 pi12 Common38046HU=507.28px와 독립Print10쪽 top573.614584/bottom1080.260416px는 같은 첫 물리 상자를 입증한다. Native top574.2/height497.1px, scanner 예약489.3px다. 첫6행의 실제 측정합345.28px 뒤 row6은 블록 컷 `[1,3,3,8,1,0,0,0,0,0,0,0]`로 남지만 현재 행의 모든 소유 유닛을 소비하고 다음 행 소유는0이다. 수동 입력이 아닌 원본 dump/vector/렌더 트리에서 관측했다(`main-p10-original-table12-v65.dump.txt`, `main-p10-independent-physical-frame-v65.json`, `native-main-p10-physical-frame-before-v65/`). 일반 plain-text reset은 없으므로 기존 reset 계약을 완화하지 않는다.
- 후보는 authenticated `stored_partial_frame_height`의 완전 prefix 판정을 컷 표현이 아니라 실제 블록 셀 순서/유닛 완전소비/후속소유0으로 확인한다. 이전 행들이 실제 paint와 같은 measured frame을 쓰는지 기존 `whole_fragment_row_uses_measured_height`/`row_block_cut_uses_measured_height`로 확인하고 같은 행 계획의 prefix를 뺀 마지막 물리 높이를 생산한다. scanner의143.99px 내용 높이에서 measured151.55px를 빼는 추정은 쓰지 않는다. 생산결과 → emit 예약/마지막행 override → PageItem/MeasuredTable → table_partial의 정확한 마지막행 높이와 resolved_table_top → 뒤흐름을 추적한다. 기존 내용 컷·이어받기 소유는 유지하며 빈 밴드가 다음 내용 유닛을 소비하지 않는다. 편집/reflow/유닛 일부소비/미인증 프레임은 비적용이다.
- 작은 원본9개 쪽수/73개 기존 계약/Main10–11쪽과 #6761 경계의 새 Native를 먼저 실행한다. fresh WASM/영향 전쪽90%와 직접 판독 전 테스트·fixture·baseline은 변경하지 않는다. 아직 후보 계획이며 보정 완료/원격 제출은 아니다.

- 최초v65 Native build exit0/251.684초/binary `bd3bf0c02adebf92b47532b60e81b59caf4af1689ac4464e0b8ac9706a628c2c`, 빠른 원본9개 쪽수 불변이다. Main10–11 Native4쪽은 exit0/141.566초: HWP97.63165/98.07847,HWPX97.62819/98.54269%다. #6761 39–40은 exit0/53.855초/98.94265/97.45412% 유지, 기존73개 nextest는 exit100/349.685초/실행19.857초/71PASS/기존2FAIL/2767skip이며 source10개 hash불변이다. 모든 작업 최종종료를 확인했다. 새로운 검사·기대값 변경은 없다.
- canonical Main10–11 review를 직접 대조했고10쪽에는 여전히 작은 하단 차이가 남는다. 예약/기본RenderTree는507.28px이나 실제Print SVG의 마지막 괘선은1076.28px다. 실제 SVG 경로의 `DIAG_FOLD pi=12 rows=0..7 trail=8.00 slack=5.23 fold=5.23`가 선언 빈 밴드를 뒤 줄간격으로 다시 접는 소비지점임을 입증했다(`native-main-p10-complete-block-frame-svg-drift-v65-command.json`, 대응log). 기본 트리 좌표를 실제 SVG 판독 대신 사용하지 않는다.
- retry1은 같은 원본 블록의 완전유닛/미소비 후속소유/원본 프레임 인증 query를 예약과 실제 paint가 함께 소비한다. 이미 생산한 첫 resolved origin/마지막행 물리override와 프레임 높이가 대응할 때 원본 가운데 정렬 공간을 보존하고 줄간격 fold에서 제외한다. 일반 부분 유닛·편집/reflow/미인증 상자·좁은 가용 예산에는 적용하지 않는다. 기존 source-frame producer와 내용 컷은 유지하며 임의 clamp나 문서별 숫자를 추가하지 않는다. retry1 소스의 새 Native/기존계약/freshWASM을 다시 검증한다.
- 종료된 v62/v64 전쪽 SVG의 파일명·size·SHA가 같은102개를 hardlink 공유해3,919,171,584bytes를 확보했다(`space-recovery-identical-full-main-svg-v65.jsonl`). 경로/내용은 보존하고 실행중인 출력·공용target은 변경하지 않았다.

- retry1 Native build는 exit0/255.751초/binary `57be27721b8e343a1e4042b05f1ee2ba55c61edc2a2f24883af055c5546b7310`/소스11개해시불변이다. 지침 갱신 직후 최초 작은 Native 작업은 tool handle이 사라지고 실제 대응 process도 없는 것을 확인했다. 미완결 artifact는 interrupted/exit미상으로 보존했으며 제품FAIL/PASS로 집계하지 않았다. 동일pin의 별도 rerun1은 최종종료까지 확인했다.
- Main10–11 Native rerun1은 exit0/145.552초: HWP99.95503/98.07847,HWPX99.80029/98.54269%다. 새canonical10쪽 두형식을 직접 표시해 마지막행 글줄/가운데 정렬/하단 괘선을 Print와 대조했다. 실제SVG 하단은 v64의1071.053333 → 최초v65의1076.28 → retry1의1081.506667px다. 후자는 원본Common507.28px와 같은 물리 상자이며 독립Print의A4 축척 하단1080.260416px와 대응한다(`main-p10-svg-physical-frame-after-v65-retry1.json`). 고정px 기대값은 추가하지 않았다.
- #6761 원본39–40은 exit0/56.377초/98.94265/97.45412% 유지,39쪽 새review도 직접 표시했다. 원본9개 쪽수는 모두 이전과 같다(`pages-original-countercheck-stored-complete-block-frame-v65-retry1-rerun1.json`). 기존73개 nextest는 공용 절대target `/home/tsjang/rhwp/target/pr-review`에서 exit100/375.705초/실행22.166초/71PASS/기존2FAIL/2767skip, 소스11개해시불변이다. 검사·기대값·fixture·baseline 변경은 없다.
- 현재Main 전체104쪽 Native TSV는 같은 불변pin으로 새출력 중이다(`native-main-complete-block-frame-full-v65-retry1-exit.json`). 별도 원본 #6803의5쪽은98.94637/93.43892/75.15690/32.30497/35.53280%, #6935의3쪽은85.79861/54.92011/94.06013%로 기존미달을 유지한다(`native-remaining-rowspan-originals-v65-retry1/`). 해당범위를 이번Main 보정으로 해결했다고 보고하지 않는다. 초기18개의 남은미달/교육과정 전쪽선행검증/최종lint·전체nextest·freshWASM·Studio/API·원격CI/merge·후속처리는 계속 남는다.
- 이4개Rust source와 통합기록을 로컬 체크포인트로 고정한다. 컴파일snapshot의11개 SHA를 commit blob과 대조하며, 같은코드의rootfreshWASM/실제Studioreload/Native·WASM 전쪽 비교를 이어간다. 현재대표PNG 통과는 전체scope 승인이나 원격제출이 아니다. 새지침에따라 이후Cargo/WASM은 기본workspace의 공용 절대target을 명시한다.

- `337671d13a70a48196fcb0e612412490be4b9613` 커밋 이후 최신 fetch/rebase를 다시 확인했다. local devel/upstream/devel은 `6f66932a73ee6fd05a74a83bdc438c9499a89a12`, HEAD 변화 없음/clean이다. Native104쪽은 최종 exit0/2018.050초/소스11개 해시불변이며 HWP 최저97.15676%(30쪽), HWPX 최저95.41221%(7쪽), 각52쪽/누락0/미달0이다. fresh WASM build exit0/599.444초, WASM SHA `e21a5e772506f9982aba41272d8163b36aa4a5a0e58c388ba332e01b726e6856` 및 pkg/public 일치/실제 Studio reload ok를 확인했다. Native/fresh WASM 대응6쪽 점수도 동일하다(`native-wasm-current-boundary-score-parity-v65-retry1.json`). full WASM104쪽은 아직 실행하지 않았다.
- 현재 최소값30쪽 canonical review를 직접 판독해 `·재질변화시마다`의 쪽 소유 차이를 발견했다. Print30쪽에는0개/31쪽에는1개, Native30쪽에는1개/31쪽에는0개다. 같은 r168c1의 `분` 글줄도 다음쪽 소유를 침범한다. 점수가97%여도 해결 완료로 판정하지 않는다(`native-main-current-minimum-review-v65-retry1/review/review_030.png`, `main-print-text-p30-31-v65.txt`, `main-native-text-p30-v65.json`, `main-native-text-p31-v65.json`).

### 진행46 — 걸친 셀의 저장 프레임과 내용 소유 경계

- 원본 pi33 r168c9의 저장 줄은 vpos0/1500/3000/0, r168c1은0/1500/3000/0/1500/3000이다. 실제 source 재시작과 독립 Print30–31쪽 소유가 근거이며 수동 합성 메타데이터가 아니다. 현재 straddle 경로는 물리 행 prefix를 `cell_units_fitting_height`에 전달하지만 저장 프레임 경계가 아닌 전체 높이만으로 소유를 결정한다. 일반적인 opt-in 유닛/수용 컷 진단으로 실제 높이·reset flag·원본 줄 연결을 확인하고 생산→예약→paint의 공통 소유 경계를 보정한다. 고정px, 문서ID, 임의 clamp 또는 첫 reset에서 무조건 멈추는 조건은 사용하지 않는다.
- #6803 원본 r8c4에는 같은 문단의 vpos2000→0 경계가 있고 Print2쪽의 마지막 글줄이 Native3쪽으로 밀린다. 실제 측정 앞8행891.786667px와 source first frame936.04px, 마지막줄 점유와 간격을 구분해 별도 원인을 확인한다(`issue6803-source-reset-cut-diagnosis-v65.json`). 현재 전쪽 미달이므로 기존 테스트/기대값/fixture/baseline은 유지한다.

- opt-in 진단 빌드는 exit0/280.157초/소스12개해시불변, binary `803069469954d85b390cd82d08ebf702b857b1c8c3a05e31e0a9831e67ded16d`다. 실제 r168c1/r168c9의 첫3개 unit은 각20px, 네 번째는20/13.333px이며 hard_break=true/stored_frame_break=false다. 원본1566HU 세행과 달리 수용된 물리 높이는78.693333px여서 기존 컷이0..4/다음쪽4..끝을 고른다. #6803 r8c4는26.666667px 단위이며 원본 intra-para 재시작은 unit2이다. 로그와 source snapshot을 보존했다(`native-main-source-unit-diagnostics-v66-command.json`, 대응log, `native-issue6803-source-unit-diagnostics-v66-command.json`).
- Main 후보는 원본 행 prefix가 바로 source 첫 재시작 cut을 선택하고, 같은 시작행의 독립적인 두 병합 셀이 같은 행 경계를 인증할 때 그 source owner를 사용한다. 첫줄0부터 저장 LH/LS대로 전진하다 양의 좌표에서0으로 돌아오는 원본만 대상이며, 원본 행이0/유효하지 않거나 단독 reset/편집·reflow/부분행컷·override는 기존 계약을 유지한다. 이 source cut은 물리 행 높이/빈밴드를 축소하지 않는다. `stored_span_row_boundary_owner → rowbreak_straddle_cut_units → straddle_continuation_demand 및 table_partial`의 동일 start/end owner로 전달한다. 기존 수용 원장의 물리 공간과 뒤흐름은 유지하며 `분`/마지막 빈도 줄의 소유만 바로잡는 후보를 먼저 실제 출력으로 검증한다.

- 최초 후보의 fmt/build는 Rust2021 미지원 let-chain으로 실패했다(build exit101/68.233초). 에이전트의 문법 오류로 보존하고 Option 사전 계산으로 수정해 재실행한다. 제품FAIL 재현이나 검증PASS로 집계하지 않는다.

- retry1 Native build exit0/255.763초/소스13개해시불변, binary `d35a60ab7d126deb5753197035ab30a33dec65cb6130ffac0df5ca44bb232530`이다. 기존 소유진단의 다음쪽4..끝이 실제3..끝으로 바뀌었고 앞쪽0..3과 이어진다. 원본9개 쪽수는 모두 유지됐다(`pages-original-countercheck-shared-source-span-owner-v66-retry1.json`). 새 Native30–31 두형식 sweep은 exit0/143.438초/소스불변이며 HWP30–31=97.83234/100%, HWPX30–31=98.21598/99.97839%다. HWP두쪽/HWPX30의 canonical review를 직접 표시·판독했다. `재질변화시마다`는 양쪽의 실제 트리에서30쪽0개/31쪽1개이며 원본Print의 소유와 일치한다.
- #6761 원본39–40의 같은 소스 Native 대조는 exit0/89.187초/98.94265/97.45412%로 이전과 같다. 정확한 base6f66932a7의 manifest/unit-tier 정책 검사도 PASS(1458source/6264attrs/48targets,4113unit/297modules)다. 기존73개 nextest와 Main10–11은 아직 실행 중이다. 새code의 전체Native/freshWASM/최종lint와 초기18개의 다른 미달 보정은 남으며, 이 경계 통과를 전체승인으로 바꾸지 않는다.

- 기존73개 nextest는 최종 exit100/381.285초/실행21.041초로71PASS/기존2FAIL/2767skip이며 소스13개 해시불변이다. #6761 두 negative 계약도 PASS이고 새FAIL은 없다. Main10–11 두형식 새Native도 exit0/210.070초이며 이전 점수를 유지했다(HWP99.95503/98.07847,HWPX99.80029/98.54269%). 모든 Cargo/관련 sweep의 terminal 결과를 확인했다. 교육과정의 기존2FAIL은 원본415쪽 전체 Native/freshWASM90% 선행검증 미완료로 검사·기대값·baseline을 변경하지 않았다.
- 현3개 Rust source와 통합기록을 로컬 체크포인트로 고정한다. opt-in 유닛 진단도 포함해 원본 줄→유닛→실제 start/end 소유를 재현할 수 있다. 새회귀/fixture 추가는 아직없으며 현재Native 경계 결과는 새code의 전쪽/freshWASM 승인과 구분한다. 이후 #6803의 첫 물리 프레임을 셀 좌표로 정규화하는 producer 누락을 보정한다.


### 진행47 — 첫 표 상자를 분할 셀 좌표로 정규화

- `17832b15e5c242333f7656f8c28227b3914d4779` 커밋에 저장 행의 source owner 보정을 고정했다. 컴파일한13개 source SHA와 commit blob이 전부 같다(`v66-retry1-build-to-commit-proof.json`). 최신fetch/local devel동기화/rebase 후 base6f66932a7/HEAD불변/clean을 확인했다(`rebase-v66-retry1-20261009.json`).
- #6803 원본8행 이전의 원시 행 합은66884HU=891.786667px이며 실제 수용된 측정값과 같다. 원본 첫 Common 상자는70203HU=936.04px다. 기존 `native_intra_para_saved_reset_trailing_trim`는 ‘첫 상자에 앞선 여러 행이 있을 수 있다’고 설명하지만 마지막 조건에서 셀 내부57.093333px만 전체936.04px와 비교한다. 앞8행을 빠뜨려 source unit2 컷의 마지막 간격 보정을0으로 만들고 Print2쪽 소유의 글줄을3쪽으로 밀어낸다. 고정allowance나 clamp가 아닌 원본 프레임의 셀좌표 정규화 누락이다.
- producer는 편집/reflow되지 않은 원본 plain-text 셀과 유효 원시 행 prefix가 첫 선언 상자를 가로지르는 경우 그 상자를 해당 셀 원점 기준으로 정규화한다. 같은 결과를 기존 컷 수용/가시 높이/paint의 마지막 간격 계산이 소비한다. 셀 시작전 상자·불명확한 행 높이·합성/dirty source는 새 경로로 인증하지 않는다. 첫 소유2개 유닛의 끝은53.333333px+패딩3.76px이며 셀 상자는44.253333px라서 마지막 줄간격 중12.84px를 다음 프레임으로 보낸다. 원본이 남긴0.493333px 밴드는 보존한다.
- 작은 원본9개 쪽수/73개 정상·negative 계약과 #6803 전체5쪽의 새Native를 먼저 확인한다. Native/freshWASM 전쪽90% 선행조건 전 기존검사·기대값·fixture·baseline은 변경하지 않는다. Main17832의 새로운 전체Native/freshWASM 및 전체scope 최종gate도 아직 남는다.

- v67 최초Native build exit0/260.094초/source15개불변/binary `96c76028fc58456145fe3dc52a3d1249df02ceb7c0faa9dde8117cab4880fef4`다. 원본9개 쪽수는 유지됐다. #6803 첫 endCut이1→2, 첫 수용936.04px로 Print2쪽의 마지막 글줄이 복구됐다. 새8쪽Native sweep은 최종exit1/170.137초다. #6803=98.94637/97.63115/55.40141/45.02854/35.53280%, #6935=85.79861/54.92011/94.06013%이며 새2–3 canonical review를 직접 표시했다.2쪽만 개선됐고 전체는 미충족이다.
- 기존73개 nextest는 exit100/351.165초/실행18.614초/source불변/70PASS/3FAIL/2767skip이다. 새FAIL `#6981 giant_straddle_cell_that_must_be_cut_is_not_grown`의 원본은 교육과정이 아니라 같은 #6803이다. `A9~10, `의 셀 밖 글줄1개를 검출했다. 실제 원본전체 sweep의 미달과 연결하고 검사·기대값은 유지한다. 기존 교육과정2FAIL도 그대로다.
- Print3쪽 PDFium 경로에서 반복머리행 이후 row8 하단648.87px, Native 실제SVG592.826667px를 확인했다(`issue6803-p3-independent-grid-v67.json`, `issue6803-p3-native-svg-grid-v67.json`). 첫 부분행이 소비한44.253333px 뒤 원본/측정행569.186667px의 물리 잔여524.933333px를 이어받아야 한다. 현재 scanner/paint는 남은 글줄의470.4px만 이어받아 빈밴드54.533333px와 가운데 정렬 공간을 잃는다. 기존 `next_start_row_height_override`의 저장 행 잔여 생산은 first_fragment_blank_band가 있을 때만 열려, 이미Common전체를 채운 이번 첫컷에서는 닫힌다. 내용끝과 물리공간을 같은 상태로 다룬 결함이다.
- retry1은 원본 첫 프레임 컷과 앞 행의 실제 측정 prefix가 대응할 때 그 행의 남은 물리공간을 continuation cursor의 정확한 startRowHeightOverride로 전달한다. 원본FirstCommon→선택sourcecut/이미수용된첫상자→원래측정행잔여→다음scanner예약/rowledger→실제cellheight/완료된내용의가운데정렬이 같은 값으로 이어진다. 부분내용이 다시 잘리는 경우 가운데정렬은 켜지 않는다. 임의 숫자로 행을 늘리거나 clip을 숨기지 않으며 새원본/기존회귀를 재실행한다.

- 회귀 적격성도 출력 회귀와 함께 심사한다. 새 #6981 실패의 원본 #6803은 이미 전쪽Native에서90%미만이므로 현재 정식 기준 문서로 적격하다고 보고하지 않는다. 검사 자체는 고정위치가 아닌 글줄의 셀 포함 관계를 확인하고, 새후보에서 실제 `A9~10, ` 침범을 검출했으므로 출력에서 고친다. 교육과정의 고정쪽377/쪽수413 검사는 Print의380–381 소유/전체415쪽과 충돌하는 독립 증거가 있다. 두 검사의 원본전체 Native/freshWASM 적격성을 먼저 충족한 뒤 소속·순서·보존 관계로 전환한다. 미달 원본의 기대값을 현재 출력에 맞춰 갱신하거나 삭제해서 통과시키지 않는다.

- 회귀 검사 조건의 추가 감사는 `regression-criteria-audit-v67-retry1.json`에 source hash/위치/독립 근거를 고정했다. #6981 all-page helper는 render 오류를 건너뛰며, ‘늘리지 않는다’는 검사는 포함 관계만 확인하고 #6795 반례는 탈출 건수3만 고정해 대상/크기 악화를 입증하지 못한다. #6803은 조각 안 `dedup`으로 중복 증거를 지우고, 연속 prefix만 확인해 원본 끝까지의 보존을 입증하지 못하며, ‘용지 밖 없음’ 검사에40px 탈출을 허용한다. #6935의1958자 총량은 글자별 보존/쪽 소유 증거가 아니다. 이는 코드 검토상 검사 부족이며 실행 FAIL과 구분한다. 기존 #7226의413쪽/#6981의고정377index/#6024의105쪽은 독립Print415/380–381/103과 충돌한다. 관련 원본 Native/freshWASM 전체90% 전 기대값·검사·fixture/baseline을 변경하지 않고 출력부터 보정한다.
- v67 retry1 Native build는 exit0/256.419초/source15개불변, binary `215c54b74b0135f2365ee3f8a31471087814d0703d4308464a9a3f8b7b40b588`다. 원본9개 쪽수는 모두 유지됐다. 같은 pin의 원본8쪽 Native sweep은 exit1/171.351초/source불변이며 #6803=98.94637/97.63115/91.62217/52.85687/31.86659%, #6935=85.79861/54.92011/94.06013%다. 새3–4 canonical review를 직접 표시·판독했다. 첫 행의 물리 잔여를 scanner/paint가 보존해3쪽은 개선됐지만 왼쪽 병합 셀의 source 첫 빈문단 컷/뒤쪽 작은intra-para reset과 쪽 소유 차이가 남는다. 전체 미충족이며 새code freshWASM 미실행이다.
- v67 retry1 기존73개 nextest는 최종 exit100/358.583초/실행20.295초/source15개불변/70PASS/3FAIL/2767skip이다. #6803 탈출 줄은 이전후보의`A9~10, `에서`형태 및 외관`으로 바뀌었고 아직 셀 포함을 충족하지 않는다. 나머지는 교육과정415≠고정413/고정후속쪽의`노동인권` 부재다. 실행 FAIL·기준값 충돌·코드상 약한 검사 조건을 각각 분리하고 미달 문서의 PASS를 적격성으로 쓰지 않는다. 체크포인트에 현재 보정과 실패/미검증을 함께 고정하고, 소스·기대값 해시를 연결한다.

### 진행48 — 잘못된 회귀 검사 의미의 교정

- 사용자 추가 지시 “회귀테스트가 잘못되었으면 회귀테스트를 수정해야함”에 따라 독립적으로 확인된 기준 오류와 검사 누락을 현재 branch에서 교정한다. 기존의 시각 선행 조건 때문에 오류를 계속 고정하지 않고, 검사 교정과 문서의 정식 적격성을 분리한다. Native/freshWASM 전쪽90% 미충족 문서는 계속 수용 보류이며, 검사 PASS를 시각 승인으로 쓰지 않는다.
- 직전 source 보정은 `733f723d604d1782f0a10de2f1e551bbf8c59564`에 고정했다. Native 컴파일15개 source hash가 commit blob과 모두 같다(`v67-retry1-build-to-commit-proof.json`). 최신fetch/local devel동기화/rebase 후 base6f66932a7/HEAD불변/clean을 확인했다(`rebase-v67-retry1-20261009.json`).
- 같은 원본의 tracked Print PDF를 다시 읽어 교육과정415쪽(SHA82fe8eef...)과 위임전결103쪽(SHA90eda86b...)을 확인했다. 위임전결 원본 SHAde43607f...의 두 라벨은 PDF/현Native 전체에서 각각1회다(`education-independent-page-count-v68.json`, `issue6024-independent-test-expectation-audit-v67-retry1.json`). 잘못된413/105 기준을415/103으로 교정하고, 고정쪽 기반 라벨 검사 대신 전쪽 본문의 두 전체 라벨 보존을 검사한다. 따라서 남은105쪽 실제 출력은 새103 기대값에서 FAIL이어야 한다.
- #6981 후속쪽은 원본의 유일한 최종표와 다음 구역의 유일한 제목 표 소속으로 찾아 쪽 순서/빈쪽/누락·중복을 검사한다. 고정377index를 제거하고 all-page helper의 render 오류를 실패로 처리한다. 거대 셀 검사의 이름·주장을 실제 셀 포함 검사와 일치시켰다. #7226 대상 닻의 유일성도 확인한다. #6803은 원본 표 소속을 선택해 전쪽 paint 순서의67문단을 검사하고, 조각 내부 dedup/선택쪽/마지막 prefix 누락을 제거하며 용지 포함 관계를 일반 정밀도0.5px로 검사한다. 정상 출력 고정좌표·SVG hash·fixture/golden 변경은 없다.
- 최초 v68 focused 실행은 파생suite의 새 배정 뒤 이전 target번호 제한을 재사용해46개만 포함됐다(exit100/82.982초/실행8.507초/45PASS/#6024의105≠103 FAIL). 이 결과를73개 전체 검증으로 쓰지 않는다. 마지막 fmt 후 파생suite를 다시 준비하고 번호 제한을 제거한 동일semantic filter로 재실행한다. 정확한 base의 manifest 검사는1458source/6264attrs/48targets로 PASS이며 최초 generated drift 실패 로그도 보존했다. generated suite/manifest는 커밋하지 않는다.
- v68 retry1의73개 재실행은 최종exit100/417.832초/실행22.137초/source19개불변/71PASS/2FAIL/10309skip이다. 수정한 교육과정의415쪽/원본소속 후속제목은 PASS, #6803 전쪽67문단·순서/용지 포함 강화도 PASS다. 실제FAIL은#6803의`형태 및 외관` 셀 침범과#6024의105≠103쪽으로 남는다. 결과를 통과시키기 위해 기준을 되돌리지 않는다. 후속제목 검사에는 마지막으로 해당원본제목 노드의 실제글자 보존과 같은쪽 중복 검출을 추가하고 그함수만 다시 검증한다. 다른72개와 production15개 source는 이 후속변경의 영향을 받지 않는다.
- 후속제목 노드의 실제`노동인권 교육` 글자 보존/같은쪽 중복까지 강화한 최종함수는 별도exit0/24.380초/실행6.452초/1PASS/source19개불변으로 확인했다(`nextest-regression-terminal-owner-v68-retry2-exit.json`). 새배정의suite010을 파일에서 찾아 실행했고 정확한base manifest도 PASS다. #7226의전쪽닻조회에서도render 오류를 건너뛰지 않도록 교정해 해당module3개를 마지막으로 재실행한다. production renderer는 이 검사 교정 단계에서 변경하지 않았다.
- #7226 마지막module 검증은 exit0/21.867초/실행6.720초/3PASS/source19개불변이다(`nextest-regression-page-count-owner-v68-retry3-exit.json`). 정확한base manifest도 PASS다. 73개 재실행 뒤 수정한 local 함수1개와 render오류 처리의 영향module3개를 모두 재검증했고 검증후 source 변경은 없다. 기존2FAIL은 실제#6803 셀 침범/#6024 전체쪽수 차이로 기록하며 원본90% 적격성/초기18개 최종lint·전체Rust·freshWASM·원격CI/merge·후속 처리는 남는다. 이번4개 기존검사의 의미 교정을 별도일반커밋으로 고정하고 source hash/명령/결과를 연결한다.
- 최종 fmt check와 git diff check는 PASS다(`logs/fmt-regression-criteria-correction-v68.log`). 지침 파일 변경은 없다.

### 진행49 — 빈 저장 줄의 첫 병합 셀 소유

- 검사교정은 `20fa595d664214af377c843dfbc92bad4c4ac73b`에 고정했고 검증snapshot19개 SHA/commit blob이 모두 같다(`v68-validation-snapshot-to-commit-proof.json`). 최신fetch/local devel동기화/rebase 후 base6f66932a7/HEAD불변/clean이다(`rebase-v68-20261009.json`).
- #6803 r8c0/r8c1은 원본빈문단의vpos0/2000/0, ordinary r8c4는 같은첫구간의vpos0/2000/0을 소유한다. Common936.04−실제앞8행891.786667=44.253333px이고 첫2개의빈줄 잉크+패딩은43.76px로 이상자에 들어간다. 현재 `cell_units_fitting_height`의빈spacer tail 제외로 병합owner는 첫조각0..1/다음1..끝, ordinaryowner는0..2/다음2..끝을 고른다. Print3쪽A1/A9 라벨과 Native의약22px 차이는 다음조각의소유가한빈줄늦는실제출력과연결된다(`native-issue6803-span-source-cut-v67-retry1/`, 대응log 및canonical3쪽). 빈줄은 저장 line box와 줄전진을 갖는 원본이므로 가시글자가없다는 이유로소유를바꾸지않는다.
- 첫 Common의셀원점정규화를 plain-text 병합셀에도공유하고, 원본첫lader/양수→0재시작/선언상자의잉크와마지막간격/독립형제owner/실제측정prefix·ordinarycut의동일상자를함께확인해첫sourceowner를생산한다. `rowbreak_straddle_cut_units`의start/end가동일query를소비하고예약의`straddle_continuation_demand`와실제paint가같은범위를쓴다. 물리행과빈band·내용유닛을분리하고일반emptyspacer의높이·컷조건을전역으로완화하지않는다. 다른첫상자/수동LS·편집·reflow/불명확rawrow·커진prefix는현재계약을유지한다. 원본5쪽의새Native와정상/negative 기존검사에서먼저확인하며새fixture·golden은추가하지않는다.
- v69 Native build는 exit0/261.615초/source19개불변, binary `bdaf9b7c91f6a1f40e65ce545fd976d3267f1a0a6d9f2ebc29edcd561d15e3f1`다. 원본9개의쪽수는유지됐다. 실제diagnostic의병합owner는첫0..2/다음2..34로같은source경계를소비한다(`native-issue6803-shared-empty-source-frame-owner-v69/`, 대응log). 원본8쪽sweep은최종exit1/171.153초/source불변이며#6803=98.94637/97.63115/93.32701/52.85687/31.86659%, #6935=85.79861/54.92011/94.06013%다. 새3쪽canonical review를직접표시·판독했다. sourceowner는복구됐으나마지막글줄의3→4쪽이월과뒤4–5쪽소유/표grid차이는미해결이며현code freshWASM은미실행이다.
- 28개현재파생target의동일semantic filter에서기존73개nextest는exit100/441.583초/실행20.222초/source19개불변/71PASS/기존2FAIL/6053skip이다. #6803 셀침범과#6024의105≠103을유지하며새FAIL은없다. 재작업전원본/기대값/fixture는변경하지않았다. SourceCommon의첫병합owner와물리구간을checkpoint로고정한뒤, 이후ordinarycell의작은reset과독립적인걸침셀의큰reset을같은원본프레임으로인증하는후속query를검토한다.

### 진행50 — 원인 분석과 수정·검증을 묶는 작업 방식

- 사용자 지시에 따라 개별 보정마다 Native 빌드→73개 회귀→커밋을 반복하는 방식을 바꾼다. v69의 Native 빌드261.615초와 집중 검증441.583초 중 실제 테스트 실행은20.222초였다. 같은 조판 경계의 수정 범위와 반례를 먼저 확정하고 한 묶음으로 구현한 뒤 Native/fresh WASM 빌드와 영향 범위 검증을 진행한다. 작은 경계의 직접 판독 뒤 필수 전체 검증을 진행하는 순서는 유지한다.
- 분석 기준은 clean HEAD `1071f044e1e267f66b2ce573d3015c31dbbdbb32`와 v69 pinned binary `bdaf9b7c91f6a1f40e65ce545fd976d3267f1a0a6d9f2ebc29edcd561d15e3f1`다. 이 실행파일로 #6803/#6935/#6024 전체 render tree와 #6024 전체 텍스트를 다시 확보했다. 네 명령은 각각 exit0/0.317·0.166·2.676·1.772초/source19개불변이며 `batch-analysis-issue*-v70-command.json`, 대응 `-exit.json`/`logs/`에 보존한다. 분석 중 production 코드 변경·재빌드는 없다.

| 대상 | 원본·독립 Print·현재 실행에서 확인한 차이 | 함께 추적할 실제 소비 경로 |
| --- | --- | --- |
| #6803 | ordinary r10c4의 원본 reset2 대신 cut1, r13c4의 reset6 대신 cut5. 병합 r8c0/c1은2→34→67/68을 소비해 후속 큰 source reset36/70과 어긋난다. p3/p4의 host overhead0과 독립 Print의 표 위 원점 차이도 있다. | `stored_intra_paragraph_frame.rs`의 source owner → `table_layout.rs:15084`의 마지막 간격과 ordinary cut → `emit.rs:1046`의 실제 행 잔여 carry → `table_layout.rs:20360/20536`의 예약·paint owner → `table_layout.rs:18138`의 최종 위 원점. |
| #6935 | 첫 컷[5,9] 때문에 독립 Print의 첫쪽 마지막 줄이 다음쪽으로 이동한다. 다음 block의 scanner cut460.8은 잔여458.9를 넘고 실제 paint927.9는 본문918.5를 넘는다. 원본 첫 행74.4133과 실제 측정121.68의 차이는 약47.27이며 raw prefix를 실제 좌표로 대용할 수 없다. | 첫 행 측정 → ordinary 마지막 줄·source Common 인증 → block cut의 실제 요구 높이 → 누적 예약·예산 실패시 컷 → paint. Square wrap이므로 #6803의 TopAndBottom 조건 완화만으로 해결됐다고 주장하지 않는다. |
| #6024 첫 초과쪽 | 전쪽 내용 대응에서 Native34쪽은 para70 r167의 `일반사항` 한 행으로 따로 생긴다. Print33쪽은 같은 행까지 포함한다. 실제 scanner는 마지막 행43.7467/잔여38.0/내용28.5·완전소비를 관측하고 이월한다. | `height_measurer.rs:3936`의 병합 선언 제약 → cut/whole-row 높이 → `scan/runner/row_step.rs`의 요구 높이·마지막 밴드 수용 → `emit.rs:64/109`의 완성 조각 extent → 최종 행 상자·후속쪽. |
| #6024 두 번째 초과쪽 | Native79쪽의 para132 표는 Print76쪽에 있다. Print77쪽은 para131 표 전체를 별도 배치한다. 원본 para131의31행 모두 header 속성이며 Native는 첫22행을 먼저 출력한 뒤 다음쪽에0..30을 다시 출력한다. 원본 floating 표와 뒤 TAC 표의 쪽 소유·중복을 함께 보정해야 한다. | 원본 header coverage/host LS·floating anchor → `leading_header_rows`/fragment budget의 반복 범위 → 첫 수용·이월과 뒤 TAC의 쪽 소유 → 실제0..30 행 출력 횟수. |

- #6024의103개 독립 Print와105개 최신 Native 전쪽을 whitespace/page-number 정규화5-gram으로 대응시켰다(`batch-analysis-issue6024-page-ownership-v70.json`). 내용 기반 대응은 진단이며 시각 일치율이 아니다. 34쪽/79쪽의 차이는 원본 para70/130/131/132 덤프, 최신 render tree, Print33/76/77쪽 직접 판독으로 교차 확인했다. 기존 overflow 로그의53/55쪽만 고치면 초과2쪽을 해결한다는 가정은 폐기한다.
- 독립 PDF의 벡터 괘선 좌표는 `batch-analysis-independent-grid-v70.json`에 보존했다. 저장 cellSz·raw 행합·실제 측정·수용한 물리 공간을 구분하고, 측정 증가분이 잘못됐다는 결론은 그 값이 생긴 규칙과 Print의 같은 경계를 대조한 뒤 정한다. source reset과 shared owner 인증·물리 잔여·원점은 같은 묶음에서 보정하고, 모든 줄 reset을 강제 쪽 나눔으로 바꾸지 않는다.
- 묶음 검증은 같은 소스 snapshot에서 원본9개 쪽수/내용 소유, 기존 집중73개와 정상·비적용 대조군, 영향 경계의 Native/fresh WASM review·overlay를 실행한다. 그 뒤 초기18개 전체 영향 범위의 전쪽90% 적격성 및 필수 lint·전체 Rust·fresh WASM/Studio 검증을 완료한다. 현재 두 기존 FAIL과90% 미달/미검증은 그대로 보류이며 초기18개 범위·후속6개 별도 검토·원격 CI/merge·후속 처리의 완료 조건을 줄이지 않는다.

### 진행51 — 공통 저장 프레임·전체 제목 표의 묶음 보정

- v70은 실제 측정 행의 원점 계획을 `LayoutEngine`에 공급하고, ordinary source 첫 컷과 겹치는 병합 셀의 독립 저장 프레임으로 작은 reset을 인증한다. 마지막 간격·유닛 소유·분할 행의 물리 잔여를 scanner/예약/paint가 같은 query에서 소비한다. 원본 빈 host가 재개하는 위 원점도 반영한다. 전체31행이 제목인 원본 표는 현재 쪽에 들어가는 일부를 먼저 출력·반복하지 않고 기존 deferred-frame 경로로 전체를 다음쪽에 배치하며 뒤 TAC는 원래 쪽에 남긴다. source ID/고정 쪽·픽셀 예외, 새 회귀 fixture/golden은 추가하지 않았다.
- v70 Native build는 exit0/259.632초/source23개불변, binary `7907a5bffc84bc71ed9a5d596be849c042bdddf2aa19d1ccebf32503212b6482`다. 원본9개 중 #6024만105→104쪽이고 나머지52/52/3/3/3/5/415/103은 유지됐다. 최신 #6024 para131은78쪽에31행의201개 셀을 각각1회 출력하며 이전의 첫22행 중복을 제거했다. 원본 para70 r167의34쪽은 남는다. `native-batch-issue*-v70/`, `pages-original-countercheck-shared-source-frame-batch-v70.json`, `shared-source-frame-batch-v70-result-audit.json`에 명령·source SHA·전체 관측을 보존한다.
- v70 Native 전8쪽 sweep은 exit1/162.338초/source불변이다. #6803=98.94637/97.63733/96.42149/94.76612/94.70697%로 전5쪽의최저94.70697%, #6935=90.11088/72.15378/94.06013%다. 두 문서의4쪽/2쪽 canonical review를 직접 표시·판독했다(`native-source-frame-batch-review-v70/`). #6803의 ordinary 컷2/6, 병합owner36/70과 셀 포함은 개선됐지만 현code freshWASM은미실행이다. #6935의 오른쪽 병합 셀이 다음source 첫줄까지 당긴 차이는 남아 재검토한다.
- 같은source23개의 기존73개 nextest는 exit100/449.327초/실행19.516초/72PASS/1FAIL/6053skip이다. #6803 셀침범 검사는 PASS로 바뀌고 #6024의104≠103만 FAIL이다. PASS를 원본 전체 Native/freshWASM 적격성으로 승격하지 않는다. 기준값·검사 파일을 바꿔 실패를 숨기지 않았다.
- 직접 대조에서 이번 구현의 두 오류도 확인했다. `split_end_limit`는 padding 없는 유닛 높이인데 이를 마지막 행의 상자 높이로 차감해 padding을 prefix와 마지막 상자에 이중 예약했다. 실제 상자 높이를 동일 query로 차감하고 독립 Print와 대응하는 기존 source-frame 하단 계약 안에서 물리 상자를 선택하도록 교정한다. 또 이월 제목표 뒤의 source0 제목 `25. 국민신문고과`가 표 아래에 수용돼 다음쪽 소유를 잃는다. source의 TAC 후 dense 빈줄→0 본문 경계에서 후속문단의 최소physical page를 명시적으로 전달한다. 표 높이를 body 높이로 늘리지 않는다.
- v71 묶음에는 위 두 자체 오류, #6935의 여러 완전 행을 가로지르는 첫병합owner, 원본 빈host의 독립source 경계 인증을 함께 반영한다. 첫Common이 ordinary 마지막 컷을 인증하고 병합 셀의 진짜 stored page reset이 같은 프레임을 소유할 때만 여분의 물리 빈밴드를 보존한다. 수정후 동일 전체 원본·경계·정상대조를 다시 확인하며 아직 lint·freshWASM/Studio·초기18개 전체시각·원격 통합 완료가 아니다.
- 전쪽 검증 공간을 위해 종료metadata가 있는 이전7개 실행의 SVG를 ignored `closed-svg-lossless-archive-v70/`에 무손실 분할 압축 보관한다. 반복base64를 content SHA로 공유하고 모든SVG의 정확한 바이트 재구성 SHA를 확인·recipe를 fsync한 뒤 원래SVG를 제거한다. review/overlay/compare PNG·PDF·TSV·원본과 Cargo 캐시는 보존한다. 복원은 `archive-closed-svg-v70.py --restore <recipe의상대SVG경로>`이며 종료 집계는 `summary.json`에 기록한다. 이 압축은 이전 시각 판정을 새head 검증으로 바꾸지 않는다.

### 진행52 — v72 묶음 보정의 독립 근거·소비 경로

- #6935 Print2쪽 벡터 괘선(`issue6935-p2-independent-all-grid-v72.json`)은 아래 표의 제목52.2627px·첫 내용행109.8px·다음82.4693px이다. Native의 첫 내용행109.9px 자체가 크게 짧다는 이전 해석을 정정한다. 실제 차이는 제목 r7의 content-cut48.5733와 measured52.3467 간3.7733px로, 원본2개 저장 줄·원래 안 여백을 선언 cellSz566HU로 줄이면서 생긴다. `stored_whole_row_padding_frame`으로 같은 원본·줄사다리·패딩 차이를 확인하고 prepare의 전체 행 예약과 partial의 block 밖 완전행 paint가 동일 measured 상자를 사용한다. 부분 컷·편집·중첩·불명 저장정보는 제외한다.
- 사용자가 지적한 하단 선은 Square RowBreak의 미완성 end_cut 조각에서 원본 cell bottom을 현재 조각 끝에 수집한 것이다. 실제 종료 조각의 괘선은 보존하고, 기존 Square continuation의 열린 윗변과 대응하는 내부 컷 하단도 원본 셀 경계로 그리지 않는다. 좌표 보정·내용 숨김으로 선을 없애는 처리가 아니다.
- #6024 Print33쪽 원본 마지막166/167행 괘선은949.201→987.08→1023.68px(`issue6024-p33-independent-tail-grid-v72.json`)이다. Native는166행을74.7px로 늘린 뒤167행을 다음쪽에 출력한다. source r167의38px·전체 내용28.4533은 현재 잔여38px에 들어가고, measured43.7467의5.7467 증가분은 내용이 아니라 r160 부모 선언 최소값이다. `stored_complete_terminal_row_frame`은 실제 마지막 행·원본38px 수용·모든 종료 부모가 현재 조각에서 시작·원본 저장 줄/개체 없음·모든 부모의 전체 내용이 각 실제 상자에 들어감을 확인한다. `row_step`에서 수용 전 동일 물리 높이를 선택하고 end override를 emit/paint에 전달한다. 중간행·이미 내용 컷을 소비한 부모·미수용 내용에는 적용하지 않는다. 허용치는 기존0.1px의HU 변환 경계이며 baseline/쪽수 기대값은 바꾸지 않는다.
- v71 결과: Native6803 전5쪽 최저95.40319%,6935 최저78.30120%(2쪽),6024 원본104쪽/Print103쪽. v70 기존73검사72PASS/1FAIL(6024); v71 코드의 테스트·fresh WASM 증거로 대용하지 않는다. v72는 위3개 수정의 같은 소스 묶음으로 빌드·원본9개·기존 집중검사·영향 review를 다시 실행한다.

- v72 Native 묶음 빌드exit0/256.607초/source31개 불변, binary `556f8ac5088546a24da33639f54e1b4794956276b2973f40a47c306168229298`(`native-build-shared-source-frame-batch-v72-exit.json`). 원본9개 쪽수는Main52/52,1133 3/3,6935 3,6803 5,교육415,6761 103 유지;6024는104→103으로 독립 Print와 일치한다. `issue6024-terminal-ownership-audit-v72.json`은 r167c7 `일반사항`이33쪽에1회, `25. 국민신문고과`가78쪽에1회, 사무분장조정/조사관교육훈련이전쪽1회임을 확인한다.
- v72 Native8쪽 sweep은exit0/170.684초/source불변(`native-source-frame-batch-review-v72-exit.json`),6803 전5쪽 최저95.40319%,6935 전3쪽 최저91.57542%,누락0/gate passed이다. 6935 2쪽 review (`output/pr-review/semanticist21-20261008/native-source-frame-batch-review-v72/issue6935/review/review_002.png`; 당시 로컬 증적, 저장소 미포함)를 직접 열어 사용자가 지적한 하단닫힘선 제거와94.30407%를 확인했다. 3쪽 실제 종료 선·주석 내용은 유지된다.
- 6935 3쪽 직접 판독에서 빈왼쪽칸 상단의 짧은 선은 남는다. 이미 소진한r9/r10의0높이가 내부 괘선을 조각 원점으로 옮겨, 첫grid index0만 비우는 기존 열린윗변 처리를 우회한다(`source-frame-batch-v72-native-audit.json`). 최저값통과만으로 이 차이를 완료 처리하지 않으며 다음 보정 묶음에서 실제 경계 소유를 검증한다.
- v72 기존73개 nextest는exit0/471.677초/실행21.579초/source31개 불변,73PASS/6053skip이다(`nextest-shared-source-frame-batch-v72-exit.json`). 이전v70의6024 104≠103 FAIL은같은원본·변경없는기대103에서PASS로 바뀌었다.6024 전103쪽 Native TSV와fresh WASM은 실행중이고 원본전체의회귀적격성·최종lint·전체Rust·원격CI/merge는 미완료다. 새 fixture/golden/회귀 검사는 추가하지 않는다.
- 이전SVG 보관은terminal1976개·원래52,010,668,175바이트·압축34,198,613바이트·실제회수25,315,713,024바이트로 끝났다. 첫SVG를 원래경로에 실제 복원해 정확한SHA 일치까지 확인했다(`closed-svg-lossless-archive-v70/summary.json`, `restore-audit.json`). 원본·PDF·PNG·TSV·Cargo캐시는 보존했다.

### 진행53 — v72 전쪽 실행이 드러낸 미달·다음 묶음 범위

- #6024 Native 전103쪽 TSV는exit1/1269.923초/source31개 불변·누락0이며32쪽이90% 미만,최저35쪽38.84165%다(`native-source-terminal-frame-full-v72-exit.json`, `issue6024/silhouette.tsv`, `summary.json`). 103쪽 일치·73기존검사PASS를 문서 전체의 적격성으로 승격하지 않는다. 실패쪽은12,14,19,26,27,35,36,40,43,47,48,49,51,53,55,56,59,62,66,67,68,76,83,84,85,91,92,94,96,97,98,99다.
- 수정 경계33/76/77/78의 직접review는exit1/195.675초·76쪽74.01609%이며33쪽98.80354%다(`native-source-terminal-frame-review-v72/`). 33쪽 r167의 같은쪽수용은 확인했지만 위쪽r137c1 라벨도 차이가 있다. 독립Print32쪽은 첫문단 `위원회`만 소유하고33쪽은 다음문단 `정보보안 및 개인정보 보호`를 소유한다. 실제원본p0/p1의0→0 저장쪽리셋이CellUnit index1에hard/page_frame_reset=true로 존재하나 straddle의물리높이43.3733→내용cut0..2가 그 경계를 넘겨 다음문단첫줄까지32쪽에 소비한다(`native-batch-zero-slot-label-v72-readonly`, 원본para70덤프). 저장reset검출추가보다 실제straddle컷·예약·paint가이미검출된동일owner를소비하도록보정하는것이 다음원인경로다.
- #6935 2쪽하단선은 수정전1→수정후0,3쪽실제종료선은보존임을 같은원본RenderTree관계로확인했다(`issue6935-fragment-edge-ownership-audit-v72.json`).3쪽의0높이행에서원점으로이동한상단선은별도미해결이다. 새회귀·golden추가와통합PR은보류한다.
- 원격base는79abd9e49675f4286eaed38197f7e837afb744e7의#7693으로1커밋전진했다(`upstream-base-readonly-v72.json`).Rust엔진변경없이문서·Visual Sweep/exporter가변경됐다. 현재증거를보존하고커밋→fetch/devel동기화→rebase뒤새도구의전체쪽수·compat·색상범례계약으로fresh WASM/Native증거를확인한다.사용자지침파일을이작업에서직접수정하지않았다.

### 진행54 — 리베이스·시각 경계와 WASM 세대 API 묶음

- 보정커밋e12cd3886을보존한뒤devel을79abd9e49675로fast-forward하고94개로컬커밋을충돌없이rebase했다. 새code head는f3c8cd7d370668e36322db71c1c2c820260124ae이다.1354개Rust소스/빌드입력바이트동일·upstream조상·clean을확인했다(`rebase-source-identity-v72.json`). 새도구Native8쪽도exit0/167.749초/동일소스·전체쪽수검사로재출력했다(`native-source-frame-batch-review-v72-rebased/`).
- root wrapperfresh WASM은exit0/1101.409초/source31개불변이다. pkg/public/snapshot WASM SHA는2dc5a5730f42e9395261bc23217699bfde10a7d5c65b0ae4717435d4332f48d9로일치하며JS도일치한다(`wasm-package-source-frame-batch-v72-audit.json`). 그러나새exporter가요구한setHangul2024Compat가WASM에없어실제8쪽비교는exit1/6.692초·未출력이다.재빌드만으로해결되었다고보고하지않는다. `DocumentCore::set_hangul2024_compat`는존재하지만Deref만으로JS공개API가되지않는것이원인이다. 새layout_generation모듈은그기존함수·기존캐시무효화/재조판에직접위임한다.2020 Print비교는명시적으로2022를선택한다.
- #6935의남은3쪽윗선은block start_cut=[4,16,3,3]에서소진된0높이행의내부edge가원점에겹친것이다. Square·반복제목없음·블록시작컷에서만원점의0높이edge를확인하고,해당행에cut0/알수없는새셀owner가있으면원래선을보존한다. 이미소비한owner만있을때재개조각의새윗변으로그리지않는다. 높이·내용·최종닫힘선을바꾸지않으며Native/fresh WASM묶음재검증은v73에서실행한다.

### 진행55 — v73 Native 경계 확인과 미달 페이지 묶음 진단

- 경로는 `maintainer_general`이며 보조 문서는 `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `visual_fixture_evidence`다. 최신 base의 90% 선행 조건·확대 판독·기존 회귀 이관 조건을 다시 확인했다. 사용자 승인된 메인터너 보정을 계속하며 지침 파일은 수정하지 않는다.
- v73 Native 빌드는 exit0/260.307초/source 불변이며 binary SHA는 `07258c8ee5fe2380dc030538778893b7d115fbdbfa4ea8bd25f77802cd7092c1`이다. 전9개 원본 쪽수는 v72와 동일한52/52,3/3,6935 3,6803 5,6024 103,교육415,6761 103이다. countercheck의 `unchanged=false`는 v64의6024 104쪽과 비교한 값이며 v72→v73 회귀가 아니다.
- Native6803·6935 전8쪽 sweep은 exit0/161.647초/source 불변,최저95.40319%/91.57542%,전체쪽수5/5·3/3·누락0이다. 6935 새3쪽 review (`output/pr-review/semanticist21-20261008/native-source-frame-batch-review-v73/issue6935/review/review_003.png`; 당시 로컬 증적, 저장소 미포함)를 직접 읽어 빈왼쪽칸 위 짧은선 제거와 최종종료선·뒤주석 보존을 확인했다. `issue6935-fragment-edge-ownership-audit-v73.json`은2쪽 경계가v72와 동일하고3쪽 원점내부선만 제거됐음을 기록한다. fresh WASM 빌드는 진행 중이며 Native 결과로 대용하지 않는다.
- #6024 미달32쪽의 전쪽내용 대응 진단에서 모두 같은번호Print와 가장 잘 대응했다(`issue6024-full-below90-alignment-v72.json`). 이는 전역 쪽밀림 해소의 진단이며 시각 승인 근거가 아니다. v73 대표12/14/35/51/55/96쪽 재출력은 exit1/162.242초, 점수71.48133/53.48369/38.84165/48.53494/65.96649/48.27492%다. 35쪽은 이전쪽 업무행 잔여가 첫행으로 넘어와 다음표 시작과 내용컷까지 달라지고,51쪽은 첫업무행 높이 차이가 이후 행 경계에 누적되며,96쪽은같은표의내용행소속이앞쪽으로차이난다. 원본13개표의저장구조를묶음덤프하고측정·컷·paint경로를분석한다. 같은source의Native/fresh WASM을마칠때까지코드변경을동결한다.
- 원격 읽기전용 재조회는 열린30개 중 비draft23개이며 초기18개 중#7612는외부merge되어열린17개,후속6개는별도검토로유지한다(`open-pr-current-v73.json`). #7615 최신head의후속#7662꼬리는아직수용하지않았다. 원격변경·통합PR·CI/merge/후속처리는미수행이다.

- 독립 Print의51쪽 벡터 괘선을96dpi좌표로 추출했다(`issue6024-print-fragment-edges-v73.json`). Print50쪽r5의물리높이는974.2969→1025.1198=50.8229px,51쪽첫r6은142.0833→183.7969=41.7136px이다. Native는각각45.8/28.5px이다. `row_step.rs`의BAND_CUT가content limit35.7을수용하지만physical override를주지않아paint가content30.7+padding15.1로45.8을선택한다. `emit.rs`는완전소비된end_cut도accepted_row_boxes기록에서제외한다. 다음조각은실제수용prefix대신원래row5box를소비한다. 같은기록에34/35쪽의r23전체내용이월과95/96쪽의r28–30행이월을독립Print와연결했다(`issue6024-source-frame-consumer-diagnosis-v73.json`). 다음묶음은내용완전소비·실제수용빈밴드·closing span잔여를같은결과로연결하는범위다. 아직이변경을production에적용하거나해결완료로판정하지않았다.

- v73 최적화 fresh WASM은 exit0/1080.079초/source 불변으로 완료됐다. WASM SHA `503d25a88872861ef878dab749534a9fc268d16a77838a979d4ce8b79b4e8f42`, JS SHA `4628f097cddafb073362cc2ff03d19744de3e037aa62d311b4bd4386528c7a20`가 root pkg/Studio public/snapshot에서 일치한다. 기존에 추적하던 `rhwp-studio/public/rhwp.js`에는 wrapper가 생성한 새API8줄이 추가됐다. ignored pkg/WASM/파생suite는stage하지않는다.
- 실제 Studio 새로고침·#6935원본로드에서새WASM served SHA일치·font loaded·canvas3개·pageerror0을확인했다(`browser-studio-shared-source-frame-v73.json`). setHangul2024Compat(false→true→false) 실행은쪽수3→3→3과2022 RenderTree원상복원을확인했다. 이는generationAPI공개·기존캐시무효화위임·브라우저반영의증거이며2024독립Print정합검증은아니다. freshWASM8쪽sweep과기존73개회귀는실행중이다.

- fresh WASM6803·6935전8쪽sweep도exit0/182.505초/source불변으로끝났다. 두문서최저95.40319%/91.57542%·전체쪽수5/5·3/3·누락0으로Native와같다. 같은8쪽Native/fresh WASM RenderTree 전체가동일함을확인했다(`source-frame-batch-v73-backend-ownership-audit.json`). WASM6935 2쪽review (`output/pr-review/semanticist21-20261008/wasm-source-frame-batch-review-v73/issue6935/review/review_002.png`; 당시 로컬 증적, 저장소 미포함)와 3쪽review (`output/pr-review/semanticist21-20261008/wasm-source-frame-batch-review-v73/issue6935/review/review_003.png`; 당시 로컬 증적, 저장소 미포함)를직접읽어이어받는열린변·실제종료선·뒤주석소속을확인했다. #6024전103쪽·교육415쪽및나머지문서,최종lint/전체Rust/원격CI/merge를이8쪽결과로대신하지않는다.

- v73기존73개집중nextest는exit0/442.636초/실행18.259초,73PASS/6053skip,source23개불변으로끝났다(`nextest-shared-source-frame-batch-v73-exit.json`). 전체Rust·최종lint는별도미완료다. 검사 뒤소스변경에이결과를재사용하지않는다. 다음#6024묶음전현재변경을일반커밋으로고정하고upstream/devel정합을재확인한다.

### 진행56 — 내용 완료 행과 실제 수용 밴드의 공통 기록 보정(v74)

- v73 커밋은 `3505cb8cd8d09f3c125fdb08c9ccec6efece78ff`이다. fetch한 upstream/devel과local devel은모두79abd9e49675이며rebase는up-to-date였다. 검증한Native 소스23개와커밋바이트·생성Studio JS·fresh pkg의동일성을확인했고커밋직후clean이었다(`commit-source-identity-v73.json`).
- 독립Print34/35·50/51·95/96의source/consumer진단에따라두경로를한묶음으로보정한다. 첫source partial frame의내용완료행은기존ordinary owner만허용하는조건대신같은행에서닫히는모든span owner의원본줄유닛이실제prefix+남은밴드에완전히들어가는지를확인한다. 아직이어지는span·편집/재조판/합성줄·줄리셋은이완료증명에서제외한다. 첫source frame인증은기존호출자조건을유지한다.
- 내용이완전히소비된원본rowspan BAND_CUT는남은물리행예산을padding·빈밴드까지같은end_row_height_override로예약·배치한다. emit의accepted_row_boxes에도완전소비된일반end_cut의이물리높이를기록하여다음closing span이실제수용prefix를한번차감한다. 일반행#5585빈밴드이어받기·블록컷·마지막표행의기존경로는그대로검증대조군으로둔다. #6024 r137의0→0문단owner는별도미검증이다.
- 수정전대조군후보#5585 원본17쪽의Native/fresh WASM전쪽TSV를v73고정binary/pkg로실행중이다. 이실행은v74현재소스검증이아니며code source hash가진행중변경됐더라도입력·binary/pkg는3505cb8cd의검증산출물로고정되어있다. v74묶음은Native빌드→원본9개·영향review와같은대조군→기존집중회귀·fresh WASM을묶음실행한다. 신규fixture/golden/회귀추가·통합PR은아직보류한다.

- #5585 v73고정Native 전17쪽TSV는exit1/208.395초,15쪽40.95189%·16쪽33.72728%로미달했다. 기존fixture를정상대조군으로자동판정하지않는다. runner의source_unchanged=false는실행중v74원인보정을시작한것이며v73binary/pkg의변경은없다. 이결과는3505cb8cd와byte동일이확인된v73검증산출물의수정전증거다. fresh WASM결과·미달쪽직접review·v74같은입력대조는후속검증으로남긴다. #5585회귀기대값은변경하지않는다. v74 fmt·fmt check는exit0이며Native빌드는진행중이다.

- v74 Native 묶음빌드는exit0/274.593초/source불변, binary SHA `27be2550264b5ec1b878abc66b906ded09be0fe6d28938139acbf644e9b57adf`로완료됐다(`native-build-shared-source-frame-batch-v74-exit.json`). 원본9개·6024영향16쪽review·6803/6935전8쪽review·5585전17쪽TSV를같은source/binary로묶음실행중이다. 영향경계의직접판독전에는전체검증완료로보고하지않는다.

- v74 원본9개는6024 103쪽·Main52/52·1133 3/3·6935 3·6803 5·교육415를유지하지만6761은103→105로회귀했다. 최초차이는62쪽(구역3/para0/ctrl2) 첫조각endRow6→5이며원본r5에그림bin45/46이있다(`issue6761-section3-para0-v74.dump.txt`, `batch-analysis-issue6761-v74`). 새plain-line certificate를기존일반행의완료content box까지강제한것이잘못된가정이다. 일반행은기존측정box에그림도포함한다. 기존ordinary_owners경로를복원하고새certificate는앞행에서시작한closing span의추가증명에만적용한다(v74 retry1). 임계값·기대쪽수는변경하지않는다.
- v74 6024 35쪽review를직접읽었으며38.84165→99.42%로앞업무행이월·표끝·뒤제목·다음표첫행소속이개선됐다(`native-source-terminal-frame-diagnosis-v74/issue6024/review/review_035.png`). 같은묶음의6803/6935 Native 전8쪽은exit0/166.508초/source불변이다. 전체6024·6761회귀해소·fresh WASM및최종gate로대용하지않는다. 현재v74 retry1을같은원본9개와영향경계·대조군에서다시확인한다.

- v74 retry1 Native 빌드는 exit0/258.262초/source 불변, binary SHA `0c527aa96db817c432b4464314d909c302ab292049536856e5e6385953f01c02`다. 원본9개 재확인에서6761은105→103으로복구됐고6024 103·Main52/52·1133 3/3·6935 3·6803 5·교육415를유지했다(`pages-original-countercheck-shared-source-frame-batch-v74-retry1.json`). 쪽수복구를내용/시각무회귀증거로대신하지않고영향review를재실행한다.
- 초기v74의6024 영향16쪽review는exit1/468.721초,8쪽이90%미만(12/14/53/55/96/97/98/99)이며누락0이다. 실행중retry1소스변경으로source_unchanged=false였고고정v74binary의진단증거로보존한다. #5585는v73 Native/fresh WASM과v74 Native 모두17쪽중15/16쪽40.95189/33.72728%미달로같으며,최종검증이나정상대조군으로승격하지않는다. 기존회귀의기대값·baseline은변경하지않았다.

### 진행57 — 완료된 병합 셀의 뒤 행 소유와 합산 경계(v75)

- v74 retry1 Native6803/6935 전8쪽은exit0/165.299초,source불변이다.6024 영향16쪽은exit1/458.651초/source불변으로8쪽미달이남는다. 같은문서의13개원본표와전103쪽RenderTree 진단은exit0/2.376초로수집했다.
- 독립Print13쪽은r28 일반사항까지,14쪽은r29 오보·왜곡 보도 대응부터소유한다. 현source row completion은r25의긴병합owner가r35까지이어진다는이유로r28을배제했다. 저장줄내용전체가현재prefix+밴드에들어가는것과병합셀의물리종료행은별개다. 원본줄의완전한수용은유지하고,뒤물리행을continuation이계속소유하도록완료내용인증에서동일종료행가정을제거한다. 뒤유닛이실제로필요한owner·저장reset·편집/재조판은전체내용fit인증으로계속기각한다.
- Print95쪽의r28–30묶음은측정prefix810.44px와완료마지막행29.7466666667px로source901.4933333333px에들어간다. 기존block/개별row 실제합산순서를독립진단에서재현하면2.2737367544323206e-13px초과로엄격비교가실패한다(`issue6024-p95-complete-source-block-fp-diagnosis-v74-retry1.json`, `diagnose-complete-source-block-fp-actual-scan-v74-retry1.rs`). block 수용은기존runner의f64::EPSILON·실제연산개수기준roundoff만인정한다. content·물리행예산·paint좌표·시각90% gate의허용치를늘리지않는다. 실제초과인블록은기존이월/분할경로를유지한다. 두원인보정을한번에빌드하고원본9개·영향경계·73기존계약을검증한다.
- v75 Native build는exit0/254.622초/source불변(binary `feb935e1e1acbe552b39c1df0cfeeab2191773ea7a836ea17e1380f3a6de77fa`)이며원본9개쪽수는v74 retry1과같다.73개집중nextest도exit0/505.636초(실행23.565초),73PASS/6053skip/source불변이다.6803/6935 전8쪽Native는exit0/163.000초다. 테스트통과를시각무회귀로승격하지않는다.
- v75 6024 영향18쪽직접출력에서95/96쪽은90% gate를넘었으나14쪽첫두행겹침·51쪽첫업무행물리높이소실이관측됐다.51쪽점수는v74 retry1의통과에서58.39647%로악화됐고52/53쪽도미달이다. 현재변경의실제회귀로기록하고구현을고친다. 전103쪽TSV는고정v75binary로계속실행한다.
- 55쪽독립Print SVG의윗변은127.21875px,Native는123.5px다. 원본anchor 덤프에서61은표제앞control(pos0),106은표제뒤control(pos9)임을확인했다(`issue6024-source-anchors-v75.txt`). 첫조각준비에서`for_first_fragment`가제거한outer top을저장head control에만돌려주는가정이tail control의원본위여백283HU를잃는다. 같은유효저장host 계약의head/tail placement가동일바깥상자를소비하도록수정한다. 교육·합성/편집·호스트와겹치는줄은기존저장host 유효성·profile 조건으로제외한다.
- 내용완료조건의확대에서발생한회귀는원본끝행의실제physical row plan과paint 소비를추적한다. 기존옵트인TABLE_DRIFT에선택한original/accepted/resolved행상자를남겨최초손실계층을확인한다(v76진단). 아직51쪽등회귀해소·fresh WASM·최종검증근거는없다.
- v76 Native build는exit0/280.544초/source불변이며binary SHA `cbae672d0312a12ab878633ccd3b0efcd9f189fcf354eb85adb8d3f79e523da9`다. 옵트인진단은exit0/2.927초로실행됐다.61/102의첫accepted row ledger는각각37.2133333333/50.8266666667px를정확히기록했다. 뒤continuation의`start_row_height_override`가있으면`stored_fragment_row_plan`이조기반환하여그ledger를소비하지않는다. `DIAG_5714`도5.3466666667/13.3866666667px를새행높이로이월함을확인했다(`issue6024-next-row-tail-owner-diagnosis-v75.json`, `logs/batch-analysis-issue6024-v76.log`).
- v77은저장첫source row의완료증명과scan의실제끝행이일치하고end cut이없는경우,그이전행의blank tail을새행override로만들지않는다. 이미수용된physical ledger를다음closing span이차감하고새ordinary row의원래측정상자를보존한다. 같은행의부분컷과기존명시블록이어받기는별도소유로유지한다. source row 완료인증·블록roundoff·표제뒤outer top·잘못된tail owner수정을같은후보로원본9개·기존73+#5714/#5585·영향PNG에서검증한다.
- 55쪽outer top 보정의새Native sweep은exit0/38.751초/source불변,65.96649→99.74833%다. 표원점은123.5→127.3px로독립Print127.21875px와같은영역에서대조했고`native-source-frame-top-margin-v76/issue6024/review/review_055.png`를직접표시했다. fullsource v77/fresh WASM의최종근거로대신하지않는다.


### 진행58 — v77 실제 소비 복구와 남은 전체 검증

- v75 고정 binary의6024 전103쪽 TSV는 exit1/1232.193초로 종료했다. 75쪽이90% 이상이고28쪽이미달(12/14/19/27/40/47/48/49/51/52/53/55/56/59/62/66/67/68/76/83/84/85/91/92/94/97/98/99),최저47쪽45.66481%이며누락0이다. 실행중v76/v77 source 보정으로source_unchanged=false이며현재후보의성공증거가아닌수정전전체증거다(`native-source-terminal-frame-full-v75-exit.json`, `native-source-terminal-frame-full-v75/issue6024/silhouette.tsv`). 75/103=72.8%는이후보의페이지gate통과비율이며초기18개작업의전체완료율이아니다.
- v77 Native 묶음빌드는exit0/254.220초/source불변, binary SHA `7890ad6fbf1a2a1fe23289a96610d4448cbc1d6fe671ca7f2f18c96799b275b4`다(`native-build-shared-source-frame-batch-v77-exit.json`). 원본9개쪽수는v73/v75와같다.6024 countercheck의false는v64의104쪽과비교한것으로새회귀가아니다. 전103쪽진단도exit0/2.825초/source불변이다.
- 실제다음조각에서61/102의accepted prefix가소비되며,14쪽r29/r30은126.3/154.7px의연속28.5px상자로복구됐다.51쪽r6은142.1px부터41.8px이고뒤r7은184.0px부터시작한다. 독립Print의142.0833→183.7969px와같은영역에서대조한다(`issue6024-tail-consumer-before-after-v77.json`, `logs/batch-analysis-issue6024-v77.log`). v77 14쪽review (`output/pr-review/semanticist21-20261008/native-source-terminal-frame-diagnosis-v77/issue6024/review/review_014.png`; 당시 로컬 증적, 저장소 미포함)를직접열어60.00982→99.78%와겹침해소·내용순서를확인했다. 전체18쪽의terminal결과및51쪽PNG는별도확인한다.
- v77 Native6803/6935 전8쪽sweep은exit0/166.109초/source불변,최저95.40319/91.57542%·전체쪽수5/5·3/3·누락0으로통과했다. 6935 2쪽review (`output/pr-review/semanticist21-20261008/native-source-frame-batch-review-v77/issue6935/review/review_002.png`; 당시 로컬 증적, 저장소 미포함)를직접읽어하단이어받기열린변·뒤내용보존을확인했다. 새v77 fresh WASM은아직실행하지않아v73의WASM8쪽통과로대신하지않는다.
- 기존5714의76076 원본과정확한독립2020 Print(82쪽)의35–37쪽대조도실행했다. v77은exit1/68.975초/source불변이며36쪽73.26187%로미달한다. 직접review의행경계차이를정상대조군으로승격하지않고수정전v73같은쪽과비교한다. 테스트기대값·golden·픽셀허용치는바꾸지않는다. 전체82쪽·fresh WASM의90%적격성은미검증이다.
- 6024의96쪽은점수가90%이상이어도Print의r59 `직제개정 요구`가97쪽으로이월된다. 현재block59..61의전체56.9px는잔여31.5px에안들어가지만ordinary r59의완전한28.4533px상자는들어간다. 재시도end_cut은첫행6owner완료/다음행4owner미소비이며cut_h13.3px라기존MIN_TOP_KEEP_PX기각경로에서emit전에이월된다(`issue6024-p96-complete-row-orphan-diagnosis-v77.json`). 미완료텍스트고아와완전한행상자를구분하는독립내용·물리높이근거를분석한다. 전역최소높이를낮추거나새정식6024회귀를추가하지않았다.
- 진행률보고에서실행완료·90%통과·현재head검증을구분한다. 현재집중78검사/6024영향18쪽은실행중이며,6024전쪽재검증·교육415쪽과6761등전체Native/fresh WASM·최종lint/전체Rust/Skia·통합PR/CI/merge/후속처리는남는다. 초기18개체리픽적용완료를초기18개검토완료로보고하지않는다.

- 76076 수정전v73의같은35–37쪽도exit1/128.268초로36쪽73.26187%다. 세쪽RenderTree SHA가모두v77과바이트동일하여이번묶음의새배치회귀는아님을확인했다(`issue5714-native-before-after-v77-audit.json`). 이것은upstream base 자체의적격성증거가아니며,90%미달의기존회귀를삭제/완화하거나정상대조군으로승격하지않는다.
- v77 51쪽review (`output/pr-review/semanticist21-20261008/native-source-terminal-frame-diagnosis-v77/issue6024/review/review_051.png`; 당시 로컬 증적, 저장소 미포함)를직접읽어58.39647→95.25%와첫r6·뒤업무행순서복구를확인했다. 하단r28의내용/빈밴드차이는남는다. 원본block27..29의hard reset으로end cut을수용한뒤r28부분을다음쪽에이어받는경로이며전체업무소속정확성의완료증거로대신하지않는다. 35쪽도99.42%의같은영역·표끝·뒤제목을직접대조했다.

- v77 집중nextest는exit100/474.332초(실행19.727초)/source불변,78개중76PASS/2FAIL/6048skip으로종료했다(`nextest-shared-source-frame-batch-v77-exit.json`). 두FAIL은기존5585의row3동일컷빈밴드이어받기와끝표뒤outer bottom이다. actual fragment는14쪽rows0..4/endcut없음→15쪽rows4..10/endcut[1,1]→16쪽rows9..10/startcut[1,1]이고뒤문단gap은-0.24px다. testcase가기대한row3빈밴드·아래1.87px와다르다. 기존73개및5714두검사는통과했지만전체묶음통과로보고하지않는다.
- 실패원본5585의17쪽전체TSV와14–16쪽review를현재v77고정binary로재실행한다. 수정전v73와현재v77의14/15/16쪽RenderTree는모두바이트동일이다(`issue5585-before-after-native-v77-audit.json`). 이는이번묶음의새회귀가아님을입증하지만기존실제배치결함또는테스트오류판정을해소하지않는다. 독립Print·현재출력·완전내용/빈밴드소유를직접확인하고원본을90%이상으로만든뒤의관계검사로검토한다. expectation/픽셀허용치/테스트제거는수행하지않았다.
- Cargo 집중검증의terminal을확인한뒤같은동결소스의root 최적화fresh WASM을공용target/pr-review로시작했다(`wasm-build-shared-source-frame-batch-v77-command.json`). 빌드가끝나고pkg/public/hash·실제브라우저·Native와동일영역sweep까지확인하기전에는WASM검증완료가아니다.

- v77 6024 영향18쪽sweep은exit1/638.469초/source불변·누락0·전체쪽수103/103으로끝났다.13쪽은90%이상이고12/53/97/98/99의5쪽은71.48133/50.65125/67.60932/74.76649/73.52941%로미달이다(`native-source-terminal-frame-diagnosis-v77-exit.json`, `native-source-terminal-frame-diagnosis-v77/summary.json`). v75의9미달쪽에서14/51/52/55가gate를회복했지만전체103쪽의최종적격성으로승격하지않는다.96쪽review도직접표시해96.83%에도마지막r59가빠지는실제차이를확인했다.
- v77 5585 실패영역14–16쪽review는exit1/86.978초/source불변이며15/16쪽40.95189/33.72728%다. 두PNG를직접열어15쪽에서다음row9 `【구글】Advertising Program`이앞당겨지고16쪽에는그제목이없으며뒤본문도위로이동함을확인했다. 기존문서최저90%조건을충족하지못하므로새회귀추가나기대값갱신은하지않는다.기존FAIL은소스동일v73와같은실제출력차이로기록한다.17쪽전체TSV는실행중이다.
- 이묶음은부분보정체크포인트로일반커밋한다. source byte identity와base ancestry는ignored검증기록에고정하며fresh WASM빌드·upstream동기화후소스가바뀌면해당영향검증을다시실행한다. 집중검사2FAIL·위미달/내용소속차이가남아push/PR검토요청/통합완료단계가아니다.사용자지침파일·테스트기대값·fixture/golden은이번묶음에서변경하지않았다.


### 진행59 — 최신 base와 완전한 내용·남은 물리 공간의 구분(v78 계획)

- 부분보정fbd7c7edb를보존한뒤upstream/devel을21ed64071f81f35f37618717693bd30f727392c9(#7697)로동기화하고96개로컬커밋을rebase했다. 현재head는e8ae5204d1b8e61851142f22b229de38f1867816이며clean/base조상이다(`rebase-source-identity-v77.json`). Skia두파일의충돌은upstream의공통EquationFonts자원컨텍스트를유지하고기존HancomGlyphs의custom→bundled→system선택을그컨텍스트로연결했다. 일시적인충돌처리오류도해당이력안에서바로잡아최종소스에marker가없고,그경로cargo check --features native-skia --lib은exit0/118.248초/source1080개불변이다. 이는실제SkiaPNG검증의대체가아니다.
- 새upstream의출력backend소비검증canonical문서를읽었다. SVG raster와Canvas/NativeSkia최종화면을구분하며실제소비경로의미실행은미검증으로남긴다. 새source에서NativeCLI빌드는exit0/257.049초/source불변(binary42c2bee7c1cea34634d07780ac376190a58f11b4a726a34748939ea3080a4ed5)이다. 원본9개쪽수유지,6803/6935 전8쪽sweep도exit0/167.268초/source불변·전체5/5,3/3으로통과했다. freshWASMv77은base변경으로내소유빌드만종료(exit143/240.818초)하고cache는보존했다.
- 5585의원본pi169dump는r3의두plain Top셀이14721HU=196.28px의독립선언공간을갖고source첫조각35880HU=478.4px에178.9px만수용함을확인했다. `stored_partial_fragment_height`의ordinary 완료인증→scan의row높이교체→emit의새row4진행이내용완료를공간완료로취급해17px빈밴드를없앤다. 한컴2020 Print15쪽은이빈밴드뒤row4부터시작하고16쪽은row9제목부터시작한다. PDF 벡터의최종표96.375→291.20313px와원본후속vpos14916HU를같은좌표로대조한다(`issue5585-print-boundaries-v77.json`, `issue5585-source-dump-v77-audit.json`). 원본SVG transform 전체행렬을적용해단순raw y해석을정정했다.
- 일반blank-band의기존line-unit/declared-height판정을scan공통row Query로이동해source첫frame 선택도같은분류를소비하게한다. Top정렬plain행은완료내용과별개로선언공간을유지하고기존ordinary end_cut으로빈밴드를이어받는다. 그림이있는6761행은line-unit-only가아니므로기존완료상자를보존한다. 정상대조5714베트남원본의제9/제10장두열은모두Center라같은Top blank-band계약을강제하지않는다(`issue5714-vietnam-source-table-v77-rebased.dump.txt`). 테스트기대값은바꾸지않는다.
- 6024 pi172 block59..61의band재시도는이미실행된다(잔여31.5≥기존25px). 그러나content-only13.3<25px라기각된다. 저장원본유닛이완전행끝에서종료하고다음행의모든유닛은미소비인기존certificate를확인한뒤,같은cut의실제물리prefix/마지막행높이를한번생산해scan예약과PageItem의end override에전달한다. 실제부분텍스트·hard reset·편집/재조판·미수용전체상자는기존기각을유지한다. orphan임계값/시각gate/행좌표를늘리지않고원본위치의완전상자만수용한다. 두원인보정을한묶음으로빌드·기존회귀/원본9개/실제PNG/정상대조에서확인한다. 새정식렌더링회귀/golden/fixture는아직추가하지않는다.

### 진행60 — v78 묶음 검증 종료와 남은 범위

- v78은 e8ae5204d1b8e61851142f22b229de38f1867816 위의 미커밋 후보이며 base는21ed64071f81f35f37618717693bd30f727392c9다. Native 빌드는exit0/251.524초/source불변이고 고정binary SHA는f2d9e40665bd7e771a4b5d1af4fdb212059f96860a764b7ee301481dd73b92c6다. 집중nextest는exit0/475.874초/source불변,실제실행20.363초에78개모두PASS/6050skip으로종료했다(`nextest-shared-source-frame-batch-v78-exit.json`). 이는전체Rust검증이나문서의시각적격성을대신하지않는다.
- 5585 Native 전17쪽은16쪽이90%이상이고15쪽51.51470%만미달이다.14쪽99.05509%,16쪽98.83322%이며16쪽의제목과후속본문복구를직접읽었다.기존3개회귀가통과해도15쪽물리빈밴드가아직7.56px크므로완료판정하지않는다.ordinary 밴드컷의content budget과visible top padding을합친override가전체수용물리높이보다작다.예약/paint/다음조각이동일한전체수용높이를소비하도록후속수정하며테스트기대값·허용치·golden은바꾸지않는다(`native-complete-band-countercase-v78/issue5585/silhouette.tsv`).
- 6024 Native 영향18쪽은exit1/638.211초/source불변·누락0·쪽수103/103으로끝났다.16쪽이90%이상이고12쪽71.48133%,53쪽50.65125%가남았다.97/98/99쪽은각각99.83253/99.81049/99.68546%로복구됐다. 96쪽review (`output/pr-review/semanticist21-20261008/native-source-terminal-frame-diagnosis-v78/issue6024/review/review_096.png`; 당시 로컬 증적, 저장소 미포함)를직접표시하고99.69259%와마지막r59`직제개정 요구`의원래쪽복구를확인했다.전103쪽의나머지과거미달쪽과freshWASM은아직최종검증하지않았다(`native-source-terminal-frame-diagnosis-v78/summary.json`).
- 6803/6935 Native 전8쪽은exit0/167.970초/source불변,최저95.40319/91.57542%,전체5/5·3/3,누락0이다.이번후보의freshWASM은미실행이므로과거WASM통과를현재증거로쓰지않는다.원본9개쪽수도유지됐다.
- 남은범위는현재미달5585의15쪽과6024의12/53쪽및전103쪽재확인,교육과정415쪽·6761의103쪽등나머지초기18개전체문서의Native/freshWASM,실제Canvas/Skia소비경로,최종lint/전체Rust회귀,통합PR·CI·merge·후속처리다.선택쪽통과율을전체작업진행률로표시하지않는다.현재Cargo/Native선택검증은모두terminal이며새렌더링회귀·fixture/golden·지침수정은없다.

### 진행61 — 일반 밴드의 수용 물리 높이 공유(v78 retry1)

- ordinary 밴드컷의내용budget은상하안여백을제외하지만끝행override가내용budget+위안여백만사용해첫조각이7.56px짧고다음빈밴드가같은만큼커졌다. 내용유닛의end cut/limit은유지하며,후속행이있는조각은실제잔여물리높이전체를scan의consumed와PageItem의end_row_height_override에같이전달한다. emit은동일override를전체선언행높이에서빼다음빈밴드를만들고table_partial의paint도동일override를소비한다. 마지막행은기존종료계약을유지한다.
- 빌드는exit0/257.545초/source불변,고정binary SHA abe26861605628393ff9160073b6764e18beeb1bb5e063c55d504c48deb6c07a다. 실제5585첫밴드는478.4px,마지막행178.8933px,다음빈밴드17.3867px로복구됐다. endcut[7,7]·다음rows3..9·끝rows9..10과Google제목소속은유지된다(`logs/batch-analysis-issue5585-v78-retry1.log`).
- Native5585전17쪽sweep은exit0/207.323초/source불변,누락0·쪽수17/17·전쪽90%이상이다. 영향14/15/16쪽review도exit0/80.217초/source불변,99.05510/100.00000/98.83322%다. 15쪽review (`output/pr-review/semanticist21-20261008/native-complete-band-countercase-review-v78-retry1/issue5585/review/review_015.png`; 당시 로컬 증적, 저장소 미포함)를직접표시하고같은본문영역의행경계·빈밴드·뒤내용소속을읽었다. freshWASM과최종head전체검증은아직미검증이다.
- Native6803/6935전8쪽sweep도exit0/180.964초/source불변으로끝났다.원본9개쪽수는52/52,3/3,3,5,103,415,103으로유지된다. 6024의countercheck false는v64의104쪽대비이며이번수정의쪽수회귀가아니다. 집중78검사와6024전103쪽Native TSV는실행중이다.
- 6024 53쪽의이전reset추정은원본dump재확인으로정정했다. r54c2두문단의vpos는0→1300으로정상증가한다. bandoffset walk는잔여18.0px에서content17.3px인첫유닛을수용하나실제셀cut의안여백15.12px는예산에서빠진다. 이부분의row단위물리높이와owner를추적한다(`logs/batch-analysis-issue6024-band-owner-diagnosis-v78.log`).12쪽은11쪽마지막r111의빈밴드가작아r112`조정·시행`이앞쪽에들어가는관측으로좁혔으며zero-origin단독원인으로단정하지않는다. 원본·기대값·정식회귀를아직변경하지않았다.
- 원격읽기재조회는open30/non-draft23이며초기17개와보류6개가남아있다(`open-semanticist21-v78-retry1.json`). #7615 최신head4a6c27c5ffa5의추가tail은기존기록대로초기통합후별도검토범위다. 새push·PR생성·approve·merge·close는아직수행하지않았다.
- retry1 집중nextest도exit0/491.961초/source불변·실제실행21.114초에78/78PASS/6050skip으로종료했다.5585전17쪽Native최저는96.10105%다. 기존5585세검사는기대값수정없이통과했고고정px출력/golden을새로추가하지않았다. freshWASM은이후순차빌드하며현재정식수용완료로보고하지않는다.6761전103쪽과교육과정전415쪽의Native TSV도동일고정binary로시작했다(timeout7200초; tracked입력/Print PDF의해시는`remaining-full-document-batch-inputs-v78-retry1.json`).


### 진행62 — 전쪽 미달과 fresh WASM 검증(v78 retry1)

- 현재 코드 head는 `cbaf53f3fa8fc8511ac82e5bd7481cfa56f8d922`, 검증 base는 `21ed64071f81f35f37618717693bd30f727392c9`다. 후보 빌드 입력 1084개와 커밋의 byte identity를 확인했으며, source 변경 없이 완료한 검증만 이 head의 증거로 수용한다(`candidate-commit-source-identity-v78-retry1.json`). 이번 검증 단계에는 Rust/test/fixture/golden/지침을 변경하지 않았다.
- Native #6024 전103쪽은 exit1/1194.152초/source 불변으로 종료했다. 84쪽이90% 이상, 19쪽이 미달이며 최저47쪽45.66481%·누락0·전체103/103이다. 미달은12/19/27/40/47/48/49/53/56/59/66/67/68/76/83/84/85/91/92쪽이다. 선택18쪽의 두 미달만으로 전체가 해결됐다고 판단하지 않는다. 전쪽 TSV SHA는 `17f4dde1db20cdb6f99a04fb11902aa3ea89b29d2a8827ba4f4228f9f793d8ff`다(`native-source-terminal-frame-full-v78-retry1/issue6024/silhouette.tsv`).
- 같은 고정 binary의 Native #6761 전103쪽 비교도 산출됐다. 85쪽이90% 이상, 18쪽이 미달이며 최저79쪽17.91891%·누락0·전체103/103이다. 미달은8/10/15/35/41/46/50/53/55/76/78/79/80/81/82/83/86/94쪽이다. TSV SHA는 `d0a02a7dfec16ddff48c3e6108b8e2bd309766fc2a429104cb2de6c6481e3a0f`다. 같은 batch의 교육과정415쪽은 아직 실행 중이므로 batch 전체의 terminal 성공이나 교육과정 적격성을 선언하지 않는다(`native-remaining-full-document-batch-v78-retry1/issue6761-original/silhouette.tsv`).
- 위37개 미달쪽의 compare/overlay/review PNG를 원 실행의 동일 PNG에서 canonical helper로 합성했다(exit0/447.465초). 새로운 재렌더로 기록하지 않으며 full TSV·run manifest·원 PNG SHA를 `native-full-failed-pages-review-v78-retry1/derivation.json`에 연결했다. 선택 review의 실루엣값은 별도 `review_silhouette_metrics.json`에 full TSV에서 가져왔고 strict-pixel 전체 점수를 새로 만들었다고 주장하지 않는다. 6024 91쪽 review (`output/pr-review/semanticist21-20261008/native-full-failed-pages-review-v78-retry1/issue6024/review/review_091.png`; 당시 로컬 증적, 저장소 미포함)와 6761 79쪽 review (`output/pr-review/semanticist21-20261008/native-full-failed-pages-review-v78-retry1/issue6761-original/review/review_079.png`; 당시 로컬 증적, 저장소 미포함)를 직접 읽어 업무행의 쪽 소속 차이와 그림 겹침·뒤 표 침범을 확인했다.
- 최적화 fresh WASM 빌드는 exit0/1137.125초/source 불변이다. 고정 package는 `wasm-pkg-shared-source-frame-batch-v78-retry1-cbaf53f3fa`, JS SHA `4628f097cddafb073362cc2ff03d19744de3e037aa62d311b4bd4386528c7a20`, WASM SHA `2e7a81731ae960e95fe461834b51ade3d55472e3513e58309dfda2f1364df8fc`이며 root pkg/public 각각 동일하다. 후속 worker의 세 sweep도 모두 exit0으로 종료했다(`wasm-after-build-shared-source-frame-batch-v78-retry1-exit.json`).
- fresh WASM #5585 전17쪽 TSV는 exit0/516.992초/source 불변, 최저96.10105%·전쪽90% 이상·누락0·17/17이다. Native 전17쪽 TSV와 바이트 동일하다. 영향14/15/16쪽의 fresh WASM 일반 sweep은 exit0/183.420초/source 불변,99.05510/100.00000/98.83322%이며 gate passed다. 15쪽 review (`output/pr-review/semanticist21-20261008/wasm-complete-band-countercase-review-v78-retry1/issue5585/review/review_015.png`; 당시 로컬 증적, 저장소 미포함)를 직접 읽어 빈밴드 뒤 row4 소속과 행 경계를 대조했다. 기존 세 회귀는 기대값 수정 없이 통과했으며 새 절대px/golden 검사를 추가하지 않았다. 실제 Canvas/Skia 소비자의 최종 PNG는 여전히 미검증이다(`native-fresh-wasm-source-frame-validation-v78-retry1.json`).
- fresh WASM #6803/#6935 전8쪽 일반 sweep은 exit0/180.937초/source 불변, 최저95.40319/91.57542%·전체5/5·3/3·누락0·각 gate passed다. 6935 2쪽 review (`output/pr-review/semanticist21-20261008/wasm-source-frame-batch-review-v78-retry1/issue6935/review/review_002.png`; 당시 로컬 증적, 저장소 미포함)를 직접 표시해 마지막 이어받기 행에 닫힌 하단 구분선이 없고 뒤 내용이 보존되는지 확인했다. 이8쪽 통과를 다른 문서의 미달 면제로 쓰지 않는다.
- #6024 다음 보정의 원인 후보는 row-offset cut의 셀 안여백 누락, 마지막 행의 빈밴드 소유, 앞 조각의 종료 점유와 뒤 표제/표 원점이다(`remaining-full-corpus-grouped-diagnosis-v78-retry1.json`). 원본 r110–113의 raw cell flags에는 protection flag가 없음을 확인했으며 짧은 선언행 높이만으로 특정 쪽 경계를 만들지 않는다. p53 원본 r54c2의vpos0→1300은 정상 저장 줄이며 zero reset 가정은 사용하지 않는다.
- #6761 79쪽 원본 셀은 같은 빈 문단에 세 TopAndBottom 그림(99/100/101)이 있으며 원문 PARA_TEXT에 세 slot이 모두 존재한다. flow_with_text=true/allow_overlap=false이고 독립 Print의 그림100은78쪽 y880.956px,99/101은79쪽 y137.769/302.708px에 배치된다. 현재 Native는 세 그림을79쪽 y620.8/620.8/614.1px에 놓는다. “100번 그림이 PDF에서 없다”는 추정은 앞쪽 배치를 확인해 기각했다. source 슬롯 순서·저장 줄 원점·flow 소비의 실제 연결을 추적하며 그림 삭제나 전역 vpos 보정으로 숨기지 않는다(`batch-analysis-issue6761-picture-flow-v78/picture-page-ownership-diagnosis.json`).
- #6761 고정 binary의 실제 호출 경로 진단은 exit0/1.472초/source 불변이다. pi126 row1은 잔여150.1px에서 `UNSPLITTABLE`로 기각돼 첫 조각rows0..1/cut없음 → 다음rows1..2/cut없음으로 넘어간다. 측정622.4667px와 원본선언357.8133px도 다르다. `height_measurer.rs::is_row_splittable`은 글줄/중첩행 수로 판단하며, 같은 문단의 여러 TopAndBottom control 범위는 `append_non_inline_units`의 하나의 atomic owner로 묶인다. paint는 각 그림에 동일 positive stored_flow_vpos를 재사용한다. 이 연결과 원본LS36386HU=그림99/101의선언높이합을 진단에 기록했으며, 이 등식만으로 저장원점 제거·특정 그림 순서 변경을 정당화하지 않는다. 같은줄의 여러 그림/개별이월과 병렬그림의 반례를 함께 설계한다(`batch-analysis-issue6761-picture-flow-v78/picture-page-ownership-diagnosis.json`).
- #6024 90–91쪽은 별도의 CellBreak 정책 문제다. 원본 pi160은35×7/CellBreak(attr0x0600000d)이며r15–19의5행 병합셀 선언높이14800HU=197.3333px가 다음 정상밴드889.8px에 온전히 들어간다. 독립Print90쪽은r14에서끝나고91쪽은반복머리행뒤r15부터시작한다. 현재Native는90쪽에r15/16을더넣고91쪽을r17부터시작해왼쪽업무label과두행소속이달라진다. `scan.rs::candidate`의block_size≤3 보호판정과`runner.rs`의ordinary dispatch, `snap_to_block_boundary`/first-block/legacy pagination의같은limit소비위치를연결했다. CellBreak의셀단위보존과fresh물리밴드fit을기준으로분류를공유하는보정이필요하며RowBreak일반정책/실제page-largerfallback은별도로대조한다. “상수3을전역증가”나paint만이동하는보정은하지않았다. 최신고정후보의실제전103쪽진단도exit0/2.723초/source불변으로끝났고90/91쪽RenderTree는fullsweep과바이트동일이다(`issue6024-cellbreak-protected-block-diagnosis-v78-retry1.json`, `batch-analysis-issue6024-v78-retry1-exit.json`).
- 디스크 여유를 위해 terminal인 내 #6024 전쪽 실행의103개 SVG만 정확한 byte 복원 검증 후 lossless archive로 보존했다. 원본/Print PDF/PNG/TSV와 실행 중인 교육/WASM 파일, 공용 target은 유지했다(`closed-svg-lossless-archive-v78-retry1/summary.json`, `archive-closed-svg-v78-retry1.py --restore <relative-svg-path>`).
- 사용자의 현재 순서는 실행 중인 교육과정 batch 종료 결과까지 수집 → 이 검증 기록 커밋 → upstream/devel fetch·local devel ff-only 동기화·review branch rebase → 다음 묶음 보정·검증이다. 현 단계에 새 remote push·PR 생성·approve·merge·close는 없다. 교육과정 종료 및 rebase 결과는 확인 뒤 추가한다.
- 사용자 지시에 따라 종료된 과거 실행16개의 SVG 2,005개를 삭제했다. 먼저 압축 처리했던680개의 새 archive도 제거했고 나머지1,325개는 직접 삭제했다. 해당 실행의 SVG 잔여0·새 archive 없음·대표 review PNG4개 SHA 불변을 확인했다. 디스크 여유는 최초5,885,353,984→41,473,310,720bytes이며, 실행 중인 교육과정·다른 소유자의 Chrome/Vite 경로·PDF/PNG/TSV·공용 target은 보존했다(`output-cleanup-direct-delete-result-20261009.json`). 위의 이전103개 archive 기록과 이번 사용자 교정에 따른 삭제를 구분한다.
- #7566 포함 여부를 현재 head와 GitHub에서 읽기 전용으로 확인했다. 원 PR의 비merge 기능/test commit3개는 각각 `685848301310f48410fbb61b2f44507e5f0c7439`, `e692a3895249b58bd18521c68c4de74755c40076`, `b077ad94b6d3cc28e6d2601b3421d4f4e7fc49b3`로 체리픽되어 모두 현재 head의 조상이다. 원 PR은 OPEN/head `abf6f9259ac45ee25e1a2ccd4fd5b7c2073c8dcf`이며 로컬 포함을 원격merge나 UI 전체검증 완료로 보고하지 않는다(`pr7566-current-inclusion-check-20261009.json`).
- 교육과정 원본 전415쪽을 포함한 Native batch는 exit1/6133.438초/source 불변으로 종료했다. 교육과정은116쪽90% 이상·299쪽미달·최저262쪽6.44087%·누락0·쪽수415/415이며 전쪽 TSV SHA는 `9f51f09914b90363f3aa4ae27996472b8174b1f2b46e882453a8e2e8b779812b`다. 원본은 `samples/task2287/1342000_edu_curriculum_map.hwp`, 독립2020 Print는 `pdf/semanticist21-20261008/task2287-1342000_edu_curriculum_map-623b00d56bef-2020.pdf`다. 전체 미달299쪽·gate·전쪽 수는 `native-remaining-full-document-batch-v78-retry1-terminal-summary.json` 및 원 실행 `education/silhouette_manifest.json`에 보존했다. #6761의18쪽 미달과 합쳐 batch가 실패했으며 timeout이나 중단 결과가 아니다. 교육과정 fresh WASM·직접 시각 완료는 아직 미검증이며 이 원본의 기대값 갱신/회귀 신규 추가/통합 승인도 보류한다.
- 교육과정2/26/262쪽 review를 원 전쪽 PNG에서 canonical helper로 생성했고 TSV와 같은 실루엣값임을 확인했다(`native-education-selected-review-v78-retry1/derivation.json`). 26쪽 review (`output/pr-review/semanticist21-20261008/native-education-selected-review-v78-retry1/review/review_026.png`; 당시 로컬 증적, 저장소 미포함)와 262쪽 review (`output/pr-review/semanticist21-20261008/native-education-selected-review-v78-retry1/review/review_262.png`; 당시 로컬 증적, 저장소 미포함)를 직접 열었다.26쪽은표·문단·바깥상자끝차이가보이며,262쪽은Native세로쪽의다문화교육도식과Print가로쪽의앞표잔여가대응하지않는다. 같은415쪽이라는합계만으로각쪽소속·방향이정상이라고판정하지않으며원본구역용지와앞표종료·다음내용의연결을추적한다.

### 진행63 — 검증 기록 고정·rebase 후 병합셀과 물리 컷의 묶음 보정(v79)

- 전쪽 검증 종료 뒤 기록을 `df887a2aa57466ee2e01cf14517f7a6635391569`로 일반 커밋했다. `git fetch upstream devel` → local devel의 `git merge --ff-only upstream/devel` → review branch의 `git rebase upstream/devel` 순으로 실행했다. base/local devel은모두 `21ed64071f81f35f37618717693bd30f727392c9`이고rebase는up-to-date·base조상확인PASS·작업트리clean이었다. 원격push·PR·merge는아직미수행이다.
- 다음묶음은#6024의CellBreak5행병합셀과row-offset셀컷의안여백예산이다. 같은CellBreak병합그룹의셀소유를행개수3으로끊지않으며,현재밴드fit·다음온전한밴드fit·실제page-larger의분할을구분한다. 측정된block경계→첫이월/scan분류→whole/cut예약→PageItem→table_partial배치와별도opt-in Paginator의동일정책소비를대조한다. RowBreak의기존작은블록/내부hard-break정책과None표의원자정책은대조군이다.
- row-offset컷은현재내용유닛합만예산에대조한뒤paint에서패딩을더한다. 같은유닛prefix의저장trailingtrim·패딩을포함한공통물리상자를후보fit·소비높이·paint의`cell_cut_visible_height`에서소비하도록한다. 기존orphan/hard-break/sliver와첫행강제전진은별도정의된계약으로확인하며임계값을완화하지않는다. #6024/교육/#6761원본·#6803/#6935/#5585/#5714정상대조군을한후보로묶음검증한다. 새회귀·기대값/golden·지침수정은하지않는다.
- 같은후보에서CellBreak분류를MeasuredTable의공통query로연결했다. 기본TypesetEngine의first-block/scan/terminal-caption재스캔은같은단위를소비하고실제page-larger는header예약을뺀온전한밴드로판정한다. opt-in`RHWP_USE_PAGINATOR=1`은같은분류의fresh-fit/snap을소비하되기존RowBreak/None분류를보존한다. 후자의실행검증은아직미검증이다. row-offsetwalk를239줄모듈로이동했고후보fit와예약·paint가같은`CellCutFrame`의trim/안여백계산을소비한다. first-row강제전진·hard-break정책의내용예산과실제물리높이를분리했다. fmt/fmt-check/diff-check는exit0이며Native후보빌드가진행중이다.
- 교육과정전415쪽의Native/Print용지방향전환과독립PDF본문을읽기전용으로대조했다. Native262쪽의`범교과 학습 주제(다문화교육)`은Print263쪽에있고,방향불일치는15쪽(167/195/211/220/262/268/275/281/288/293/300/307/314/328/345)이다. 가령가로구역Native120–166/Print120–167뒤세로구역Native167–172/Print168–172로구역별쪽수차이가상쇄된다. 전체415라는합계만으로서로다른내용소속을면제하지않으며,단순용지방향지정오류로확정하지않는다(`education-page-ownership-orientation-diagnosis-v78-retry1.json`). 이읽기진단은v79출력보정이나시각통과의증거가아니다.
- v79 Native빌드는exit0/255.809초/source불변·binary SHA `3afe9f4c7eda4c72c98e50225f76a6455fc7a2783b5adc8519a8aa2bdfb0a8bc`다. 새module2개도source hash에포함했다. 원본9개선행쪽수는#6761만103→104로늘고다른8개는직전후보와같다. #6024실제진단은exit0/2.776초/source불변이며pi160의90쪽rows0..15/91쪽rows15..35가독립Print소속으로복구됐다. 이것은전103쪽시각통과가아니다.
- #6761의새회귀는첫차이76쪽의section4/pi118/row2부터다(`issue6761-v79-first-render-difference.json`). 같은후보76–78쪽Native sweep은exit1/76.765초/source불변,59.24099/59.41232/4.76537%·전체104/Print103이다. 76쪽 review (`output/pr-review/semanticist21-20261008/native-issue6761-v79-first-regression-review/issue6761-original/review/review_076.png`; 당시 로컬 증적, 저장소 미포함)를직접읽어Print의미국/UL행이Native첫조각에서이월됨을확인했다. 기존미달과이번에추가한한쪽을구분하며기대쪽수/golden을바꾸지않는다.
- 원본문HU는그림높이5113+offset71+안여백141+141=행5466이고앞행2150+4462+5466=선언첫프레임12078로정확히닫힌다. f64에서기존`content+top+bottom`은72.879999999999995px,공유query의`content+(top+bottom)`은72.88000000000001px,첫프레임잔여는72.879999999999995px다. 1ULP차이가`stored_partial_fragment_height`의엄격한`content<=remaining`를거짓으로만든다(`issue6761-source-frame-f64-association-v79.json`,원본dump). 저장물리프레임의수용을bit동일성으로판정하는가정을고쳐f64합산roundoff만구분하고실제padding초과는계속기각한다(v79 retry1). 별도Paginator의다음행intra-cut에도남아있던row-count분류를같은fresh-fitquery로연결했다. 두교정후한후보로다시검증하며아직회귀해소로판정하지않는다.
- 2026-10-10 원격읽기재조회는open30/non-draft23이며이전snapshot대비새PR/변경head는0개다(`open-semanticist21-v79-retry1-20261010.json`). 초기18개중열린17개와후속6개의검토순서는유지한다. 현재HEAD는검증기록커밋df887a2aa이고v79 retry1은미커밋Rust후보다. 진행62의cbaf53f3fa검증을이후보에그대로적용하지않는다.
- v79 retry1 Native빌드는exit0/250.136초/source불변·binary SHA `6f9041376d309042138553216333b1df626e705210b6f3fbb478f15af578ea4d`다. 원본9개쪽수진단은exit0/8.294초/source불변이며#6761이103쪽으로복구되고나머지도직전후보와같다. #6024실제전103쪽진단도exit0/3.027초/source불변이다. 아직일치율과전체회귀를통과한것은아니다.
- 소비지점대조에서`RowCutResult.consumed_height`를물리높이로바꾸면부분컷MIN_TOP_KEEP판정까지패딩으로확대되는문제를확인했다. v79 retry2는행오프셋walk의공통`physical_height`를별도결과로생산하고,기존부분컷최소높이/저장restart수용계약과분리한다. scan의행끝·누적예약은`occupied_height()`를소비하고split_end_limit/부분컷기각은기존내용높이계약을소비한다. paint의셀상자는같은CellCutFrame을사용한다. 행합산과0.5px이내라는이유로예약결과를다시선택하던분기도제거했다. 다른RowCut생산자는기존높이계약을유지한다. 이교정은코드검토상필요조건이며새회귀가실행으로확인됐다고쓰지않는다. retry2의검증결과는실행뒤추가한다.
- 최신v79 retry2 Native빌드는exit0/253.757초/source불변·binary SHA `954391e66738f4ed9ac24bd02584dfc71938b99db4f4b81862f3144f7a9a0b73`다. 새module포함1086개입력을고정했다. 파생suite28개prepare도exit0이며새test source는없고tracked test/Cargo파일변경도없다. 원본9개쪽수진단은exit0/7.592초/source불변이며직전후보와동일한52/52,3/3,6935 3,6803 5,6024 103,교육415,6761 103이다. 이전v64의6024 104쪽과비교하는countercheck false는이번후보의쪽수회귀가아니다. 기존분할/원자정책/거대셀집중nextest와#6024영향24쪽·#6803/#6935전8쪽Native sweep은같은고정후보로실행중이다. fresh WASM·전쪽90%·최종lint/전체Rust·실제Canvas/Skia·원격CI/merge는아직미완료다. 현재후보를완료·승인으로보고하지않는다.
- 같은후보#6803/#6935전8쪽Native일반sweep은exit0/168.986초/source불변으로종료했다. 최저95.40319/91.57542%·전체5/5·3/3·누락0·gate각각passed다(`native-shared-cell-cut-normal-pages-v79-retry2/summary.json`). 6935 2쪽review (`output/pr-review/semanticist21-20261008/native-shared-cell-cut-normal-pages-v79-retry2/issue6935/review/review_002.png`; 당시 로컬 증적, 저장소 미포함)를직접열어마지막이어받기행의하단이닫히지않고앞뒤내용이보존되는지대조했다. 현재freshWASM의증거로대신하지않으며5585대표대조와6024영향쪽·집중nextest는아직실행중이다.
- 같은 후보의 집중 nextest는 exit0/467.678초/source 불변이며 실제 실행22.016초에 **86/86 PASS**,6042skip으로 종료했다. 기존78개와원자정책/거대셀대조8개를포함하며기대값을바꾸지않았다(`nextest-shared-cell-cut-batch-v79-retry2-exit.json`). 전체Rust회귀나한컴시각일치의증거로대신하지않는다.
- #5585 Native14/15/16쪽 대조도 exit0/117.941초/source 불변,최저98.83322%·gate passed다. 15쪽 review (`output/pr-review/semanticist21-20261008/native-complete-band-countercase-review-v79-retry2/issue5585/review/review_015.png`; 당시 로컬 증적, 저장소 미포함)를직접읽어빈밴드뒤본문/다음제목의소속을대조했다.17쪽전체와freshWASM은별도남긴다.
- #6024 현재까지완료된영향쪽의53쪽은50.65125→99.86337%,90쪽99.88779%,91쪽49.05735→99.90699%다. 53쪽 review (`output/pr-review/semanticist21-20261008/native-shared-cell-cut-boundaries-v79-retry2/issue6024/review/review_053.png`; 당시 로컬 증적, 저장소 미포함)와 91쪽 review (`output/pr-review/semanticist21-20261008/native-shared-cell-cut-boundaries-v79-retry2/issue6024/review/review_091.png`; 당시 로컬 증적, 저장소 미포함)를직접열어앞표종료/뒤제목과왼쪽업무라벨·전체병합행소속을대조했다.12/47/56쪽은71.48133/45.66481/61.89756%로미달인채남았다.24쪽실행의최종결과는아직미수집이므로이값을전103쪽완료나gate통과로표시하지않는다. 현재source를동결하고공용target에서최적화freshWASM빌드를순차시작했다. 이전WASM결과는현재후보의근거로재사용하지않는다.
- #6024 영향24쪽 실행은 exit1/830.524초/source 불변으로 종료했다.20쪽이90% 이상이고12/47/56/92쪽은71.48133/45.66481/61.89756/71.31535%로미달이다.전체103/103·선택24/24·누락0·gate `re_review_required`다. 같은원본·Print의수정전전쪽TSV와새overlay측정값을대조하면52/53/90/91쪽만달라졌고나머지선택20쪽은기존값과같다(`issue6024-boundaries-before-after-v79-retry2.json`).90쪽은기존93.49124%였지만추가두행의잘못된쪽소속이있었으며,이번99.88779%와직접소속대조를분리해기록한다. 전103쪽의다른기존미달과교육299쪽을이24쪽결과로면제하지않는다.
- 현재같은소스의최적화freshWASM빌드가실행중이다. 빌드성공/source불변뒤root pkg/Studio public의JS·WASM SHA를대조하고package를고정한후#5585전17쪽/대표3쪽,#6803/#6935전8쪽,#6024같은24쪽을순차검증하는worker를시작했다(`run-wasm-after-build-v79-retry2.py`). Native #5585전17쪽도같은고정binary로재실행중이다. 각job의terminal/exit/source불변은확인뒤기록하며source를바꾸거나이전package를현재증거로대신하지않는다. 마지막전체lint·전체Rust·전문서90%·실제Canvas/Skia·통합PR/CI/merge/후속처리는계속남아있다.
- Native #5585 전17쪽 TSV는 exit0/327.759초/source 불변으로 종료했다.최저96.10105%·전쪽90% 이상·17/17·누락0이고같은source의14/15/16쪽직접review대조도완료했다(`native-complete-band-countercase-v79-retry2/issue5585/silhouette.tsv`). 이원본의현재freshWASM과최종출력소비자의검증은계속실행/미검증으로구분한다.

- v79 retry2 최적화 fresh WASM 빌드는 exit0/1066.628초/source 불변으로 종료했다. root `pkg`와 Studio `public`, 고정 package `wasm-pkg-shared-cell-cut-batch-v79-retry2-df887a2aa5`의 SHA가 각각 일치한다. JS SHA는 `4628f097cddafb073362cc2ff03d19744de3e037aa62d311b4bd4386528c7a20`, WASM SHA는 `a2fb7d8023f0212bdac0c266950fa2fb4093ff258c338faa9aebaaaf4f9501f3`다. 실제 브라우저 Canvas PNG 검증으로 대신하지 않는다. 후속 worker는 source 동결 상태로 같은 package를 소비한다.
- 같은 fresh WASM의 #5585 전17쪽은 exit0·누락0·최저96.10105%이고 Native 전쪽 TSV와 바이트 동일하다. 대표14/15/16쪽 일반 sweep은 exit0/186.316초/source 불변,99.05510/100.00000/98.83322%다. 현재 대표 PNG 직접 판독은 확인 뒤 기록한다. #6803/#6935 전8쪽과 #6024 영향24쪽은 아직 실행/대기 중이다.

- fresh WASM #6803/#6935 전8쪽도 exit0/184.788초/source 불변으로 종료했다. 최저95.40319/91.57542%·5/5와3/3·누락0·두 gate passed다. 6935 2쪽 review (`output/pr-review/semanticist21-20261008/wasm-shared-cell-cut-normal-pages-v79-retry2/issue6935/review/review_002.png`; 당시 로컬 증적, 저장소 미포함)와 5585 15쪽 review (`output/pr-review/semanticist21-20261008/wasm-complete-band-countercase-review-v79-retry2/issue5585/review/review_015.png`; 당시 로컬 증적, 저장소 미포함)를 직접 열어 열린 이어받기 하단과 빈밴드 뒤 내용의 소속을 대조했다. #6024 같은24쪽은 계속 실행 중이며 아직 gate 통과로 표시하지 않는다.
- 다음 묶음의 읽기 전용 원인 분석에서 #6024 40/47/56쪽의 앞 표 조각은 원본 위 여백140/283/283HU가 실제 Print 원점과 대응하지만 Native에서는 빠진다. 공통 반복 제목행 프레임 query가 `HorzRelTo::Para`를 제외하고 있고, 그 결과를 예산·accepted-row ledger·최종 paint가 소비한다. 가로 앵커와 세로 프레임 여백을 같은 조건으로 제한한 가정을 재검토한다. 교육과정83행 표도 원본section28/pi2의141HU와 Print377–379쪽의원점이대응하며, “83행이면여백을열면안된다”는기존설명만으로예외를유지하지않는다. section15의83행표는제목행이없고쪽소속도달라별도계약으로분리한다. 프레임top여백과page-fillingbottom/마지막병합행최소높이는동일개념이아니므로소비범위를따로입증한다(`repeated-header-outer-frame-independent-diagnosis-v79-retry2.json`). 현재source는검증종료까지동결했으며이진단은새코드의통과증거가아니다.

- #6935 원본의 실제 Studio Canvas2D 출력3쪽도 같은 fresh package로 캡처했다(`capture-studio-issue6935-canvas-v79-retry2.mjs --mode=headless`, exit0). 초기 HTTP 응답의 WASM SHA가 고정 package와 같으며, 초기 JS는 Vite의sourceMap제거와WASM URL한곳의변환을역변환하면원pkg와동일하다. 별도fetch의해시로초기로드를대신하지않았고응답원문을보존했다. 최초스크립트는Vite변환/sourceMap을원pkg불일치로오판했으며정확한변환관계를대조하도록수정후재실행했다. 최종pageErrors0·3/3·2022조판을확인했다(`studio-issue6935-canvas-v79-retry2/diagnostics.json`).
- 실제 Canvas 최종PNG와같은독립Print3쪽을canonical helper로비교하면92.04579/94.91000/95.17925%·최저92.04579%다. 실제 Canvas 2쪽 review (`output/pr-review/semanticist21-20261008/studio-issue6935-canvas-v79-retry2/review/review_002.png`; 당시 로컬 증적, 저장소 미포함)를직접열어마지막이어받기행이하단을닫지않고다음내용/쪽소속을보존하는지확인했다. 기본Studio font-loader이며공급된SVG font환경과동일하다고쓰지않는다. 이결과는현재#6935의Canvas2D출력증거이고다른원본·NativeSkia·CanvasKit·편집사용자여정이나전체통합gate로확장하지않는다. 새test/fixture/golden/기대값은추가하지않았다.
- #6761 section4/pi126의원본gso슬롯3개를raw record에서다시확인했다. 세슬롯의control ID뒤8byte는모두0이고CTRL_HEADER의각instance ID는서로다르다. 슬롯의instance ID가다른순서를지시한다는근거는없다(`batch-analysis-issue6761-picture-flow-v78/source-slot-instance-binding-v79-retry2.json`). 독립Print의100번먼저/99·101다음쪽배치는큰그림을이월대기하고잔여에들어가는작은그림을현재쪽에수용한계획과도일치한다. 이는가설이며z-order정렬이나원문순서변경의구현근거로승격하지않는다. 부분소비가non-prefix인원본owner를표현할수있는지와한컴대조군의관측이후공통float프레임계획을정한다.

- v79 retry2 fresh WASM #6024 영향24쪽도 exit1/817.913초/source 불변으로 종료했다. Native와페이지별실루엣값24개가모두같고20쪽90%이상·12/47/56/92쪽미달·누락0·전체103/103·gate `re_review_required`다. WASM53쪽 review (`output/pr-review/semanticist21-20261008/wasm-shared-cell-cut-boundaries-v79-retry2/issue6024/review/review_053.png`; 당시 로컬 증적, 저장소 미포함)와 WASM91쪽 review (`output/pr-review/semanticist21-20261008/wasm-shared-cell-cut-boundaries-v79-retry2/issue6024/review/review_091.png`; 당시 로컬 증적, 저장소 미포함)를직접열어내용/병합셀소속·앞표종료와뒤제목을대조했다. after-build worker도4실행의최종exit0/0/0/1을수집해terminal(exit1/2094.184초)로끝났다. 현재1086개source입력이빌드고정값과같고fmt-check/diff-check는exit0이다.
- 이묶음의구현과검증기록을일반checkpoint커밋한뒤사용자지시의upstream/devel동기화·rebase를수행한다. 전체문서90%·최종lint/전체Rust·남은실제출력/편집여정·통합PR/CI/merge/후속처리는미완료다. 부분보정을전체승인으로표시하거나baseline/golden/회귀기대값을바꾸지않는다. 다음묶음은반복제목행의원본바깥프레임query와실제예산·배치소비의정합이다.

### 진행64 — rebase 후 반복 제목행의 가로 앵커와 세로 프레임 계약 분리(v80)

- v79 구현/증거 checkpoint는 `2a4add88ae84b42d1ca9b5c5fefc75c547473728`이다. `git fetch upstream devel` → local devel `merge --ff-only` → review branch `rebase upstream/devel`이모두exit0으로끝났다. base/local devel은 `21ed64071f81f35f37618717693bd30f727392c9`이고head변경없음·source입력불변·새Rust입력없음·base조상PASS·작업트리clean을확인했다(`rebase-shared-cell-cut-batch-v79-retry2-20261010.json`). remote push/PR/merge는아직미수행이다.
- 위반규칙은반복제목행을다시여는저장물리프레임의세로바깥여백을가로앵커가Column일때만소유한다고판정하는것이다. 독립Print의#6024 40/47/56쪽과교육section28 376–379쪽을원본HU/원점에연결했다. 현재보정은같은저장원본프레임query에서Para와Column의가로앵커를수용한다. producer `stored_header_frame.rs::stored_repeated_header_rowbreak_outer_frame` → `budget.rs` host_before/outer_bottom과`physical_rows.rs`원점·실제수용행부기/예산 → `PageItem.resolved_table_top` → `table_partial.rs`의최종원점및margin중복방지분기다. 부분컷/정상전체행·마지막병합행의물리잔여·뒤제목까지같은후보로확인한다.
- 반복제목행없음/글줄참여표/비자리차지/편집·수동저장정보는이query의비적용경로다. 첫프레임의상자와계속되는상자의bottom/마지막행부기까지영향받으므로top한곳의그림이동만으로통과를선언하지않는다. #6024 영향쪽과전103쪽·교육반복제목행쪽/전쪽수·#6761 전쪽수·#6803/#6935/#5585정상대조를한Native후보로묶고,방향확인뒤freshWASM을빌드한다. 새회귀/fixture/golden/기대값·지침파일은수정하지않는다. 기존v79의통과를이후보의최종검증으로재사용하지않는다.

- v80 Native빌드는exit0/257.660초/source불변·binary SHA `856773a720c8ec328c1feb3d80e69c1121e15ab3863bf216f0bce2da33f84c4b`다. 원본9개쪽수/실제6024전103쪽RenderTree진단은exit0이며교육과정만415→416이다. 전체교육RenderTree도exit0/6.488초/source불변으로확보했다. #6803/#6935 전8쪽은exit0/166.246초·최저95.40319/91.57542%·gate각각passed, #5585 대표14/15/16쪽은exit0/81.539초·최저98.83322%·17/17·gatepassed다. #6024 선택범위는11–14/33–40/45–47/50–56/90–92/95–99의 **30쪽**이며실행중이다. 앞서진행메시지의31쪽은범위합산오류였고실제command/manifest범위는30쪽이다. 아직후보의전체시각통과/freshWASM은미완료다.
- 교육쪽수변경은source section12의15→16쪽에서발생했다. 독립Print의section12제목은220쪽,다음section13제목은236쪽으로실제16쪽을소유한다. 이전Native는221–235의15쪽으로앞구역의과다한한쪽을상쇄했으며새후보는221–236의16쪽으로계속한쪽늦다. source section9의제목은Native/Print모두183쪽이지만section10제목은Native196/Print195쪽이어서앞source section9종료의과다점유가남아있다. 새후보의전체416쪽은미달이며정상판정하지않는다. 독립소속을복구하기전에15쪽으로되돌려전체415만맞추거나기대값을416으로갱신하지않는다(`education-section12-page-count-ownership-v80.json`).

- v80 #6024 선택30쪽은 exit1/925.503초/source 불변으로 종료했다. **28/30쪽 90% 이상**,12쪽71.48133%·92쪽71.31535% 미달,전체103/103·누락0·gate `re_review_required`다.40쪽59.69290→99.64437%,47쪽45.66481→99.90985%,56쪽61.89756→99.64763%이며 세 review를 직접 열어 앞표 끝/뒤제목과 병합업무셀의 소속·행경계·외곽을 대조했다. 56쪽 review (`output/pr-review/semanticist21-20261008/native-repeated-header-boundaries-v80/issue6024/review/review_056.png`; 당시 로컬 증적, 저장소 미포함)를 사용자에게도 실제 이미지로 표시했다. 이 선택결과는 전103쪽 TSV 또는 fresh WASM의 통과 증거가 아니다. v80 after-build worker도 exit1/1136.219초로 종료됐고 현재1086개 빌드 입력이 고정값과 같다.
- 교육 source section9의 초등학교 표는 Native/Print 모두183–187쪽이며 중학교 제목은 양쪽188쪽이다. 초등학교 표 뒤의 원본 PageBreak나 잔여 공간 비율이 원인이 아니다. 중학교52행 표(section9/pi5)는 Native188/189/190쪽인데 Print188/189쪽이고, 마지막행51이 Native190쪽에 남는다. 원본행27의 마지막셀은 같은문단의 저장LineSeg `200→0`으로 실제 첫줄188쪽/둘째줄189쪽을 소유한다. 현재 Native는 행27 전체를 이월하고 앞 행26을61.76px로 늘려 첫프레임을 채운다. Print는 행26을 약37.08px로 보존하고 행27 첫줄을 약24.61px의 첫조각에 남긴다. 원본 paragraph shape45/47의 raw spacing_before400은 정상 파서의 variant_div2로200HU이며 잘못된 정렬 여백이나 무효 저장본으로 분류하지 않는다(`education-section9-para-shapes-original-v80.json`, `batch-analysis-education-source-cut-v80`, exit0/11.200초/source 불변). 전체 교육 문서의 시각 적격성과 나머지 경계는 미검증이다.

### 진행65 — 원본 첫 프레임의 문단 앞 간격과 비대칭 셀 컷 계약(v81)

- v80 Native 보정/기록 checkpoint는 `733d9ea473051cc7aa495cd3409804191cae6d1b`이다. commit뒤 upstream fetch → local devel ff-only → review rebase를모두exit0으로수행했고 base는계속`21ed64071f81f35f37618717693bd30f727392c9`,head변경없음·작업트리clean이다. v80 fresh WASM은 미실행이므로 이checkpoint를최종수용으로판정하지않는다.
- 구현전근거: 원본 저장본/독립Print188–189쪽은같은문단에서첫줄200HU→둘째줄0HU의컷을확인한다. 초기200HU는실제문단앞간격이며유닛첫높이20px에포함된다. 생산query `first_source_frame_owner`는이초기원점을0으로제한하고첫상자의물리끝을줄이송끝으로제한해정상원본을놓친다. 별도MIN_TOP_KEEP25px기준도원본의첫조각24.6667px를거부한다. 교정계획은실제앞간격과저장상자를인증한같은query의소유유닛·물리높이를컷측정/scan/paint가소비하고,불완전셀의컷과완전셀의소비가모두일치할때만인증한첫프레임을보존하는것이다. 일반고아최소값·실제예산·전체쪽수기대값은변경하지않는다.
- 적용/반례: 원본첫상자가실제측정행중간에서끝나고원문한문단내다음줄이0으로재시작하는경우만해당한다. 편집/수동LineSeg·일반로컬문단전환·완전표상자·첫간격불일치·미소비형제셀은비적용이다. 공통생산결과는`CellCutFrame.height`와`row_cut_content_height`의물리점유에연결하고scan의원본컷선행조건에같은owner인증을사용한다. 교육전쪽RenderTree/쪽수와188–190쪽직접sweep,기존86집중검사·정상8쪽/5585대표·6024영향쪽을한Native후보로확인한다. 새회귀/fixture/golden/기대값·지침은수정하지않고다른남은원인은같은후보실행중읽기전용분석한다.

- v81 Native빌드는exit0/257.340초/source불변이며 binary SHA는 `e8dcce8d2c08282f76a090d7e3947de9a035c99b18f573d11e59abb35e696af2`다. 원본9개쪽수와교육전414쪽RenderTree를확보했고다른8개원본쪽수는v80과같다. 교육은416→414쪽이지만415쪽기준과아직불일치다. section9 중학교표의실제컷은첫188쪽rows0..28/end_cut[1,1]→189쪽rows27..52/start_cut[1,1]로복구되었고고등학교제목은190쪽이다. 188쪽 review (`output/pr-review/semanticist21-20261008/native-first-source-row-education-review-v81/education/review/review_188.png`; 당시 로컬 증적, 저장소 미포함),189/190쪽을직접열어첫줄·둘째줄·뒤제목의소속을대조했다. 점수99.05052/97.72234/99.14230%·누락0이며189쪽하단의작은높이차이는남는다. 실행은exit1/111.005초/source불변,전체414/415쪽불일치로gate `re_review_required`다. 대표3쪽통과를전쪽통과로바꾸지않는다.
- #6803/#6935 전8쪽은exit0/173.546초/source불변·최저95.40319/91.57542%·5/5와3/3·두gatepassed이고,#5585 대표14/15/16쪽도exit0/101.401초/source불변·최저98.83322%·17/17·gatepassed다.현재#5585전17쪽TSV와새freshWASM은아직미검증이다. #6024 선택30쪽은같은고정binary로실행중이며최종결과를아직기록하지않는다.
- 집중nextest는exit100/463.846초/source불변·실행20.686초에 **85/86 PASS**·6042skip으로종료했다. 실패는기존 `issue_7226_rowspan_only_row_cut::page_count_matches_independent_print` 하나이며독립415쪽과현재414쪽차이를정확히검출했다. 기존내용소유/중복/겹침검사와다른85개는PASS지만묶음통과로표시하지않는다. 사용자의실패회귀원본우선확인절차에따라동일교육원본/Print를대조했고,새기대값이나test/baseline변경으로실패를가리지않았다.
- 교육의34구역위치를원본소속RenderTree·독립Print제목으로대조하면section9는13→12쪽(독립183–194의12쪽),section25는12→11쪽(독립346–356의11쪽)으로바뀌었다. 현재section12는Native/Print220–235의16쪽이다. 남은한쪽부족은section14 인권교육추출의Native247–261(15쪽) 대 Print247–262(16쪽)으로분리된다(`education-original-section-title-locations-v81.json`, `education-section-lengths-before-after-v81.json`).제목텍스트위치는진단용이며내용완전성/시각승인의근거로대신하지않는다.
- 같은원본 인권교육248/259쪽추가sweep은exit1/142.426초/source불변·71.21793/71.01420%다.248쪽review를직접열어앞행잔여의쪽소속과본문하단초과·꼬리말중첩을확인했다. 실제scan은section14/pi2의여러조각에서consumed가avail_for_rows를넘어`fits=false`인데도수용되며,가로용지의legacy36/260px whole/short 허용분기를거친다. 반복제목행의물리닫힘query는scan뒤빈밴드를늘릴때만소비되고scan자체의예산초과를기각하지않는다. 최종paint닫힘과원본유닛소속을함께고치는다음보정의원인근거이며단순페이지수복구/픽셀허용치완화로처리하지않는다.
- #6024 92쪽의읽기전용대조에서원본nested표는비TAC/TopAndBottom/Column/Left/offset0이다. Print는부모의실효안여백510HU와nested바깥여백141HU뒤x86.033px에서시작하는데Native의depth>0가로배치가가운데로바꾸어x89.5px로약3.47px늦다. 마지막행15의저장줄3900+lh1000HU와실효표안여백1134HU는80.4533px의점유하한이고Print행높이80.392px와대응하지만Native행은65.4px이고다음행을90.4px로늘린다. `cell_row_grows_with_padding`의거의동일한line/decl높이성장기각분기와후속tail배분을추적하며실제producer계측/기존#3308countercase증거는아직필요하다(`issue6024-p92-nested-anchor-and-padding-diagnosis-v81.json`). 이진단을새보정의검증성공으로기록하지않는다.
- 집중Cargo의terminal과실행중Cargo/Rust없음을확인한뒤동결한같은1086개입력으로root최적화freshWASM을공용target에서시작했다(`wasm-build-first-source-row-frame-batch-v81-command.json`). 빌드뒤pkg/public SHA일치·package고정·Native와같은대표교육3쪽/정상8쪽/5585전17쪽및대표3쪽/6024선택30쪽을순차실행하는worker를준비했다. 아직실행중/대기이며이전v79package의증거를현재후보에적용하지않는다. 최종전체Rust/lint·전문서90%·실제backend/편집여정·통합PR/CI/merge/후속처리는미완료다.
- 같은후보의Native #6935 2쪽과#5585 15쪽review도직접열어열린이어받기하단과빈밴드뒤표본문/다음제목의소속을확인했다.전자는94.30%,후자는100.00%인선택쪽이며각문서최저값과구분한다. 현재default Native binary는`native-skia` feature를포함하지않으므로이SVG→browser PNG증거를실제NativeSkia출력으로보고하지않는다.

- v81 Native #6024 선택30쪽은 exit1/1097.533초/source 불변으로 종료했다. 28쪽은90% 이상이고12쪽71.48133%·92쪽71.31535%가 남는다. 선택30쪽의 값은v80과같고전체103/103·누락0이다(`issue6024-boundaries-before-after-v81.json`). Native after-build worker도exit1/1478.229초의terminal이며교육쪽수414/415와기존회귀1FAIL을숨기지않는다.
- 같은1086개빌드입력의root fresh WASM은exit0/1074.967초/source 불변으로끝났다. root pkg와Studio public의JS SHA는`4628f097cddafb073362cc2ff03d19744de3e037aa62d311b4bd4386528c7a20`,WASM SHA는`99cebfcbbac95469f0ca02b374b67cfae70006a049ea09f711d21d9fffb03f7d`로일치하며`wasm-pkg-first-source-row-frame-batch-v81-733d9ea473`에고정했다. 첫정상8쪽sweep은exit0/177.003초/source 불변·두gate passed다. 교육대표3쪽·5585전17쪽/대표3쪽·6024선택30쪽은동일worker가순차실행중이다. 실제Canvas/CanvasKit/NativeSkia화면검증과구분한다.
- #6024 원본을같은Windows 한컴2020 호환Print로재출력했다(job `9f30467b-8cec-4b15-ae1d-5c17fc6d9ac7`, Hancom `11.0.0.9136`, PrintMethod0, `hancom2020_pdf_driver_one_up`). 새로운PDF는103쪽이며기존기준PDF와11/12/92쪽의text·용지·drawing수·96dpi raster bytes가동일하다. 기존원본/기준을교체하지않았다. 원본Section0의중첩표속성4bytes만각각바꾼진단사본3개도동일한Print경로로출력했다. 원본/다른CFB stream불변을검사했으며수동사본은정식fixture나원본적격성증거가아니다(`nested-anchor-hancom-diagnosis-v81/manual-mutation-manifest.json`, `independent-print-observations.json`).
- 진단Print92쪽의nested왼끝은원본Column/Left/offset0과Para/Left/offset0이모두86.0333px이고두raster가동일하다. Column/Center/offset0은88.432px,Column/Left/offset6226HU는169.0293px로바뀌며후자는오른쪽내용이실제쪽밖으로넘친다. 네출력의진단 review.png (`output/pr-review/semanticist21-20261008/nested-anchor-hancom-diagnosis-v81/review.png`; 당시 로컬 증적, 저장소 미포함)를직접열었다. Column이라는이유만으로Left/offset을가운데로대체하는현재가정은이원본에서성립하지않는다. 기존#3308 Print 반례와의차이를조사중이며가로위치수정은아직적용하지않았다.
- #3308의기존HWPX를새Print로확인하는선택진단은MCP macro guard `HANCOM_MACRO_MANUAL_PROCESSING_REQUIRED`에서중단됐다. macro실행·자동승인·내용제거로우회하지않았고기존독립Print와원본속성자료를보존했다. 다른원본/진단Print및v81 WASM검증은계속가능하므로전체작업의진행불능으로분류하지않는다. 새진단PDF·원본과수동사본의출처는ignored 기록으로구분하며아직회귀fixture/golden으로수용하지않는다.

- 추가한컴진단에서#6024의PageBreak만Row→Cell로바꾼사본도92쪽왼끝86.0333px와행15높이80.392px를유지했다. 표상하안여백567+567HU를0으로바꾼별도사본의행15는65.2093px여서원본과15.1827px차이다(원본여백1134HU=15.12px). Hancom이직접HWP→HWPX로Save As한새대조군도동일한Print에서103쪽/92쪽왼끝86.0333px다. 이재저장본의PageBreak=Cell/noAdjust=1로원본속성이바뀐것을공개하며원본의같은gate로바꾸어기록하지않는다(`cellbreak-and-padding-print-observations.json`, `genuine-hwpx-print-observation.json`).
- #3308의현재원본은9쪽이며기존독립Print도9쪽이다. 현재v81 Native7쪽을직접sweep하고review를열었으며exit0/29.712초/source불변·96.35607%·선택쪽gatepassed다. 실제Native부모왼끝109.7px/중첩표왼끝213.7px·첫셀폭28911HU는보이는직인경계를599.18px에두지만Print는582.2467px다. 원본부모표안여백283HU·중첩바깥여백140HU·가로offset6226HU를같은부모원점에적용하면경계583.8333px(약1.59px차),제목글줄시작은약200.24px(독립200.2115px)다. 과거199.5px가운데계산은현재부모원점의근거로사용하지않는다(`issue3308-current-visible-border-anchor-diagnosis-v81.json`). 앞진행메시지의0.3px경계차이는부모표안여백을141HU로잘못넣은계산이라1.59px로정정했다. 전9쪽Native/freshWASM은아직미검증이며이한쪽의높은점수로17px배치결함을면제하지않는다.
- v81 실제Studio `PageRenderer`의6935전3쪽을새headless Chrome/print profile/zoom1/DPR1로DOM전체쪽캡처했다. 최초초기화JS/WASM응답이위고정pkg와일치하고실제backend=canvas2d·쪽수3·pageerror0·DOM이미지누락0을확인했다(`studio-issue6935-page-renderer-v81/diagnostics.json`). 같은독립Print와의실루엣은91.99433/95.07682/96.33170%다. 2쪽 실제Studio review (`output/pr-review/semanticist21-20261008/studio-issue6935-page-renderer-v81/independent-print-review/review/review_002.png`; 당시 로컬 증적, 저장소 미포함)를직접열고사용자에게표시해이어받기하단열림/후속본문을확인했다. 별도로저장한filtered Canvas의toDataURL은진단용이며전체쪽의입증으로대신하지않는다. CanvasKit/NativeSkia는다른미실행경로다.
- v81 freshWASM 교육188/189/190쪽은exit1/1044.141초/source불변으로종료했고각99.05052/97.72234/99.14230%로Native와같다.189쪽review를직접열어둘째소스줄·전체남은행/마지막하단의작은차이를대조했으며전체414/415로gate는계속`re_review_required`다. Native와WASM모두동일한쪽수회귀를확인했으므로기대값을바꾸지않는다.5585전17쪽/대표및6024선택30쪽은동일worker가계속실행중이다.
- 사용자승인output정리에서종료된v40/v46/초기filename-context의SVG708개를직접삭제했다. 완료summary/실행중command·open fd없음/tracked없음을먼저확인하고PNG/PDF/TSV/JSON의전후SHA를확인했다(`output-cleanup-ended-svg-v81-20261010.json`). 현재동결WASM source/pkg/다른사용자작업/공용Cargo target은보존했으며새압축보관본을만들지않았다.
- 교육WASM실행의terminal뒤그실행을참조하는live command/open fd없음을확인하고추가SVG1242개(논리32,908,365,569bytes)도직접삭제했다. 필요한PNG/PDF/TSV/JSON전후SHA와source/pkg를보존했고SVG잔여0·free42,783,539,200bytes를확인했다(`output-cleanup-ended-education-svg-v81-20261010.json`). 실행중인5585·후속6024 SVG는삭제하지않았다.
- v81 freshWASM #5585 전17쪽TSV는exit0/526.499초/source불변·누락0·전체17/17이고최저11쪽96.10105%다. 이실루엣전용실행의gate는`not_evaluated`여서직접review완료로승격하지않는다. 영향14/15/16쪽의별도review는동일worker가계속진행한다. 현재빌드1086개입력불변·diff-check exit0을다시확인했다. 실제Studio6935의1/3쪽review도직접열어앞쪽열린연속표와마지막쪽닫힘/후속용어3개의소속을확인했다.
- 원격읽기재조회(`open-semanticist21-current-v81-20261010.json`)는계속open30/non-draft23이며초기17개와통합후별도검토6개를유지한다. 현재source를동결한순차WASM검증의마지막worker가끝난뒤이부분보정과필요한진단PDF/검증기록을checkpoint하고사용자지시의upstream fetch→local devel ff-only→review rebase를진행한다. 그전에는새Rust수정이나재시작으로현재실행의증거를무효화하지않는다. 최종통합PR/CI/merge/후속처리완료로보고하지않는다.
- v81 freshWASM #5585 대표14/15/16쪽review도exit0/180.823초/source불변·누락0·전체17/17·최저98.83322%·gatepassed로끝났다.15쪽review를직접열어같은원본빈밴드와뒤Daum제목·본문소속을확인했다. 같은후보마지막#6024선택30쪽만같은worker에서실행중이다.
- #6761 원본SHA `398d03a5d5e4d6e857086be532d6d9ed0cec9c8ad06f95c17bbb7f83056ae860`을Windows Hancom `11.0.0.9136`/engine2020로직접HWPX재저장했다(job `1980ba99-d8e6-481a-a5d0-64a2ed87fd0b`, HWPX SHA `4bc66a1f9f4cc4b80cdd40e4328d98a182ffd5bdd504d94786f4e8d8ce0c17f9`). 그림99/100/101의동일한순서·instance id·소스크기·음수offset·단일저장줄vpos36386을그대로보존한다(`picture-flow-hancom-diagnosis-v81/genuine-hancom-picture-properties.json`). 수동LineSeg나orphan-control로분류해원본을지우거나이순서를zOrder로정렬할근거가아니다.
- 같은진단에서온전한gso기록묶음의순서만[100,99,101]/[101,100,99]로바꾼두수동CFB사본을만들었다. 개별기록묶음bytes·외부Body·다른stream/bitmap·원본은불변이며정식fixture/golden이아니다(`manual-control-order-manifest.json`). 한컴의정상PrintMethod0/one-up으로genuine HWPX와두사본모두103쪽을출력했다. 세출력모두작은100번그림을78쪽y880.956px에둔다. 첫둘을바꾼사본의79쪽은원본처럼99(y137.769px)→101(y302.708px)이고,첫셋째를바꾼사본은101(y137.769px)→99(y457.579px)다(`picture-order-print-observations.json`). 저장순서앞의미수용큰개체는이월되어도뒤의수용가능작은개체가현재프레임에남고,이월된개체끼리는그순서를유지한다는좁은관측이다. 임의재정렬·한그림분할·원본소스줄전체0치환으로대신하지않는다. 공유물리배치계획의구현과정상옆배치/독립그림반례는아직미검증이다.
- 6761 한컴진단 review.png (`output/pr-review/semanticist21-20261008/picture-flow-hancom-diagnosis-v81/review.png`; 당시 로컬 증적, 저장소 미포함)를직접열어79쪽에서두큰그림의순서가바뀌어도뒤해외인증표와쪽소속이보존되는것을확인했다. 비교PNG는진단Print끼리의관측이며새rhwp보정결과나원본전103쪽90%통과를뜻하지않는다. 새진단PDF/원문/명령/job·SHA는보존했고Rust/기존회귀/기대값/지침은추가로변경하지않았다.
- v81 freshWASM #6024 선택30쪽은exit1/995.847초/source불변으로종료했다. Native와같은28쪽이90% 이상이고12쪽71.48133%·92쪽71.31535%가미달이며전체103/103·누락0이다. 두미달review를직접열어앞쪽행의소속과중첩표원점/마지막행높이차이를확인했다. after-build worker도전체5실행의exit0/1/0/0/1을수집해exit1/3888.777초로종료했고실행중해당worker/Cargo/Rust없음·빌드1086개입력불변·diff-check exit0을확인했다. Native85/86의올바른교육쪽수FAIL과두backend의미달을유지한부분checkpoint이며승인/PR제출단계가아니다.
- 재현근거로필요한한컴진단PDF10개(가로앵커/안여백7개·그림소속3개)는위입력/명령/job/SHA기록과함께일반checkpoint에포함한다. 수동진단사본을새회귀fixture/golden으로수용하거나기존원본/기준PDF를교체하지않는다. 그림진단의78/79쪽text/그림경계는기존기준과대응하지만96dpi raster bytes는같지않아pixel identity로보고하지않는다(`picture-flow-hancom-diagnosis-v81/original-and-diagnosis-pixel-identity.json`). 다음묶음의일반구현과전체시각/실제backend·최종Rust/lint·통합PR/CI/merge/후속처리는계속미완료다.

### 진행66 — 검증 checkpoint와 실제 물리 예산·앵커·안 여백의 공동 보정(v82 계획)

- v81 부분보정과필요한진단PDF10개/기록을일반커밋 `771ec45e15d35a66895f6cb4e1e81db8c9aa19d1`로고정했다. upstream fetch→local devel ff-only→review rebase가모두exit0이고base/local devel은`21ed64071f81f35f37618717693bd30f727392c9`다. head변경없음·작업트리clean·base조상·실제빌드1086개입력불변을확인했다(`rebase-first-source-row-frame-batch-v81-20261010.json`). 기존각각의미달/gate/올바른회귀FAIL은유지하며remote push/PR/merge는아직수행하지않았다.
- 구현전독립근거는교육section14의실제본문하단초과,6024의11→12쪽조기행소비및92쪽원본Column/Left·실효안여백이포함된줄점유,현재3308의직인경계다. 반복제목행의기존공통물리닫힘query는scan뒤빈밴드부기에만쓰고스캔예산초과에는사용하지않았다. 같은query가scan예약과수용행/paint닫힘을연결하게하고,인증된닫힘의바깥까지수용하는가로용지legacy bleed를허용하지않는다. 첫저장프레임과편집/재조판의비적용계약은보존한다. 이어받기의부분컷·완전행·종료/뒤내용은같은후보로확인한다.
- 중첩TopAndBottom 표는문단/단기준의선언Left/Center/Right·offset을같은부모안쪽영역에서해석한다. 현재원점producer `compute_table_x_position` 뒤에별도inline원점덮어쓰기나폭정규화가없는지nested 호출도대조한다. 현재3308의가운데예측은독립경계를17px어긋나게하므로기존오래된좌표설명을근거로Column을제외하지않는다. 다른wrap의독립계약은구분해확인한다.
- 저장줄의점유와선언높이가가깝다는이유로실효안여백을빼는가정은92쪽의실물문자로반증됐다. 인증된plain 저장줄의내용최소상자는활성셀/표여백만포함하며셀의비활성raw padding을원인없이읽지않는다. 빈저장프레임의줄/간격과개체가소유한LineSeg extent는줄콘텐츠와같게재분류하지않는다. 생산최소상자→행성장/병합셀배분→측정된행의실제셀원점/clip을연결하고,2211의빈프레임/그림혼재·6442의비활성여백·6030의줄끝·3308을정상대조로검토한다. 원본전쪽90%이전에는새정식회귀/fixture/golden/기대값을추가·갱신하지않는다.
- 위세원인을한묶음으로수정·빌드하고,교육쪽수/인권교육경계·6024의12/92쪽및기존경계·정상6803/6935/5585·기존집중회귀에서먼저방향을확인한다. 6761의비연속그림소유는독립Print로확인한별도공통배치계획이필요하고아직구현하지않았다. 이번세보정으로6761까지완료했다고보고하지않으며초기18개전체문서의Native/freshWASM90%와최종Rust/lint/실제backend/PR/CI/merge/후속처리범위는유지한다.

- v82 native-skia 빌드는exit0/255.560초/source불변이고binary SHA `79322b63627b13687dd128dab7264ed51ed7e8f5572a9381dc70b96f0396716c`다. 원본9개쪽수의산재HWP/HWPX는54/53쪽(독립52),6024는104쪽(독립103),교육은422쪽(독립415)으로늘었다. 실제전체measured JSON/log의교육37표·6024 26표원래행높이벡터는v81과같다. 이실행실패를기대값오류로바꾸지않고실패후보를보존했다(`physical-budget-candidate-rejection-v82.json`, `scan-physical-frame-before-after-v82.json`).
- 중첩6024의뒤로밀린93쪽(독립92쪽)의원점86.1px·15/16행높이80.5/75.4px는독립Print86.0333px·80.392/75.277px와대응한다. 이는개별기하진단이며원본전체쪽소속/90%승인을뜻하지않는다. 앵커와활성padding보정을유지하고잘못된사전예산대체를제거했다. 물리닫힘의미사용밴드를logical CellUnit전진예산으로쓰고strict-following-text/orphan/terminal-note까지동일상태로확대한가정이잘못됐다. landscape query전체를닫아넘침흡수뿐아니라정상boundary split소속도배제했다. 이세확대는철회하고원래독립계약을복구했다. 올바른물리컷/행소속보정은계속미완료이며원본90%·최종통합목표에서제외하지않는다. 수정한원점/최소높이를한묶음으로다시빌드하고기존정상대조와실제SkiaPNG/Native review부터확인한다.
- 사용자교정에따라남아있던종료SVG압축보관본v70/v78의두폴더도직접삭제했다. 실제잔존자료는1976+103개SVG분량·1060개압축/recipe파일46,799,923bytes였으며새보관본은없다. live command/open fd/tracked없음과대표PNG4개의SHA불변을확인했다(`output-cleanup-remove-old-svg-archives-20261010-result.json`). 원본/PrintPDF/PNG/TSV/현재source/pkg/공용target은보존했다.

- v82 retry1 native-skia 재빌드는exit0/255.862초/source1087개불변·binary SHA `534e87373e857c9f17fbecd621cef833c4f687ebe594116c687668bb0d39bb38`다. 원본9개쪽수는v81과모두동일해산재52/52·1133 3/3·6935 3·6803 5·6024 103·교육414·6761 103을보존했다. 교육독립415쪽과의차이는미해결이다. 실험v82의새쪽수증가만해소했고전체검토통과로표시하지않는다(`pages-original-countercheck-v82-retry1.json`).
- 같은후보Native #6024의12쪽은71.48133%로미달인채,92쪽은71.31535→99.92761%로개선됐다. 실제103/103·선택2/2·누락0이며gate는12쪽때문에 `re_review_required`다. 92쪽review (`output/pr-review/semanticist21-20261008/native-anchor-padding-boundaries-v82-retry1/issue6024/review/review_092.png`; 당시 로컬 증적, 저장소 미포함)를직접열어중첩원점과마지막병합칸/일반행높이·앞제목과뒤꼬리말소속을대조했다. 실제CLI Native Skia의같은원본92쪽PNG도99.75324%이며 Skia review (`output/pr-review/semanticist21-20261008/skia-anchor-padding-issue6024-v82-retry1/independent-print-review/review/review_092.png`; 당시 로컬 증적, 저장소 미포함)를직접읽었다. SVG raster를실제Skia로대신하지않고별도command/binary/referenceSHA를기록했다. 첫Skia PNG파일명10857을쪽번호로읽어review가생기지않았던진단도원본바이트동일의092명사본으로교정해실제review92를다시만들었다. 실제글꼴두께/바깥괘선색상의차이를부분위치보정의전체승인으로면제하지않는다.
- #3308 Native 전9쪽TSV는exit0/37.492초/source불변·최저97.36291%·9/9·누락0이다. 대표7쪽sweep는exit0/27.411초·gatepassed이고직인상자의좌우원점과이후하단/꼬리말을직접대조했다. #6442 전3쪽은99.70387/99.49044/99.19302%·3/3·gatepassed이며2쪽의좌우뒷면카드내용·외곽/직인칸을직접확인했다. 활자두께와청색카드의가느다란안쪽선차이는점수와구분해남긴다. 전체검토/freshWASM충족을뜻하지않는다.
- #2211 빈프레임대조전4쪽은87.65555/97.35583/87.68642/96.18096%로1/3쪽미달이다. pinned v81 원본바이너리로같은1/3쪽·독립Print·글꼴을재출력하면점수와PNG bytes/SHA가둘다정확히같다(`empty-frame-countercase-before-after-v82-retry1.json`, before exit1/101.399초). 1/3쪽review를직접읽었으며이번padding보정의새회귀라는근거는없다. upstream base의시각비교로대신하거나기존미달을승인하지않고추가보정/전쪽freshWASM미완료로남긴다. 기존검사/원본/Print/baseline/golden/지침은제거·갱신하지않았다.
- 기존6442 세검사·6030 한검사의focused nextest는exit0/443.177초/source불변이고4/4 PASS·6124skip이다. 검사실행자체는0.238초며컴파일/링크시간과구분했다. 6030 기존px검사의PASS를독립Print시각90%충족의증거로쓰거나기대값을갱신하지않는다. Native6803/6935 전8쪽도exit0/205.288초/source불변·최저95.40319/91.57542%·5/5와3/3·두gatepassed다. 새6935 2쪽review에서이어받기하단이열린것을직접확인했다. 전체nextest/최종3Clippy/편집여정/다른실제backend/통합PR/CI/merge/후속처리는계속남아있다.
- 위Native worker는실제미달두실행을포함해exit1/470.953초로terminal이다. 모든Cargo/해당worker종료를확인한뒤같은source1087개를동결하고공용target에서최적화freshWASM빌드를시작했다(`wasm-after-cargo-anchor-padding-v82-retry1-exit.json`). root/pkg와Studio/public의JS·WASM동일성확인및고정package이후3308전9쪽/대표7쪽·padding대조·6024 12/92쪽·정상8쪽을같은후보에서순차검증한다. 현재package성공이나WASM시각통과는아직미검증이다. 원격재조회는open30/non-draft23으로초기잔여17+별도검토6을유지했다. PR/push/merge는미수행이다.
- 종료확인된v78 Native전쪽검증의SVG518개(논리9,696,360,677bytes)를직접삭제했다. 해당폴더live command/open fd/tracked없음과PNG/PDF/TSV/JSON전후SHA를대조했고SVG잔여0·새archive없음을확인했다(`output-cleanup-ended-native-full-svg-v82-20261010.json`). 실제여유공간36,505,554,944→46,176,137,216bytes이며현재WASM source/pkg와다른소유자의작업/공용target은보존했다.

- 6030 기존검사의원본 `samples/issue6030/2386771_agritech_review_form.hwp`(SHA `e1f413ce7fea8f4b08a0c13df31a9c6743b3111bdc68b65716919d529c3a2117`)에대한독립Print가현재pdf목록에없어원문으로한컴2020 Print를추가산출했다. job `b1350d67-ab4b-4c5f-8254-0cf868447f67`은succeeded이며실제 `11.0.0.9136`·PrintMethod0·`hancom2020_pdf_driver_one_up`을status에서확인했다. `pdf/semanticist21-20261008/issue6030-2386771_agritech_review_form-e1f413ce7fea-2020.pdf`는76,926bytes·SHA `79045eccbd6f712310b2b0fcc8f370cb4c64b37c14c75a51ba78699e79c8ae8c`다. 원본/기존검사/기대값은불변이며필요한기준PDF는다음checkpoint에포함한다. 같은후보Native전쪽sweep를시작했고현재그결과와freshWASM은미검증이다(`padding-countercase-hancom-v82-retry1/`).

- 6030 새독립Print의Native 전2쪽sweep는exit0/43.312초/source불변·98.69917/99.83262%·2/2·누락0·gatepassed로끝났다. 두review를직접열어평가항목행의줄끝과괘선/페이지하단·다음쪽의내용소속을대조했다. 기존절대px검사를새기대값으로잠그지않았고freshWASM전2쪽을현재package pin뒤실행하는별도worker에연결했다(`wasm-negative-trailing-after-package-v82-retry1-exit.json`). Native통과를WASM/기존검사교정의완료로대신하지않는다.

- Native의6442 전3쪽·2211 전4쪽·6030 전2쪽PNG를같은원본출력run_manifest/Native-export/PDF쪽수증거와연결해별도전쪽TSV도산출했다(`native-padding-countercase-tsv-v82-retry1-*`, `native-negative-trailing-tsv-v82-retry1-issue6030`). 6442/6030 exit0와2211 exit1을구분하며기존PNG재사용을새렌더로보고하지않는다. TSV의png-pair가독립적인원본총쪽수증거를추가한것은아니다. 6030 Print의실제저장metadata는한컴2020 `11.0.0.5409`로engine2020선택과대응한다. 두review에서확인한활자두께차이는잔여차이로남긴다.

- v82 retry1 fresh WASM 빌드는 exit0/1055.996초, source 1087개 불변으로 끝났다. root `pkg/`와 Studio `public/`의 JS/WASM SHA가 일치하며, 고정 package의 JS는 `4628f097cddafb073362cc2ff03d19744de3e037aa62d311b4bd4386528c7a20`, WASM은 `e903dadd4d668f62bcbd11c42efc6dd6916c32d04f728f78b94708849a6f5134`다. 실제 browser 새로고침/Canvas 검증 완료로 대신하지 않는다.
- 같은 package의 #3308 전9쪽 TSV는 최저97.36291%/누락0이며 대표7쪽 sweep도 exit0다. #6030 전2쪽 sweep는 exit0/53.836초, 98.69917/99.83262%/2대2쪽/gatepassed다. #6442 전3쪽은 최저99.19302%/3대3쪽/gatepassed, #2211 전4쪽은 최저87.65455%/4대4쪽/1·3쪽 미달이다. 각 실행의 terminal/source불변을 확인했고 #3308 7쪽, #6030 1·2쪽, #6442 2쪽, #2211 1·3쪽의 새 WASM review를 직접 읽었다. 활자두께, #6442 내부 보조선 및 #2211의 줄/가사 차이를 점수 통과나 기존 차이라는 이유로 면제하지 않았다. WASM PNG 재사용 전쪽 TSV도 별도 산출해 원본 출력 manifest와 연결했다(`wasm-padding-countercase-tsv-v82-retry1-*`, `wasm-negative-trailing-tsv-v82-retry1-issue6030`).
- #6024 새 WASM 12/92쪽 sweep는 exit1/178.967초/source불변, 전체103대103쪽/선택2대2/누락0이다. 12쪽71.48133%와92쪽99.92761%를 각각 직접 판독했다. 92쪽 review (`output/pr-review/semanticist21-20261008/wasm-anchor-padding-boundaries-v82-retry1/issue6024/review/review_092.png`; 당시 로컬 증적, 저장소 미포함)의 중첩 원점/병합칸/마지막 일반행 높이가 독립 Print와 대응하며, 12쪽 review (`output/pr-review/semanticist21-20261008/wasm-anchor-padding-boundaries-v82-retry1/issue6024/review/review_012.png`; 당시 로컬 증적, 저장소 미포함)는 `조정ㆍ시행` 한 행의 쪽 소속 차이가 그대로 남는다. 따라서 현재 전체 gate는 보류다. Native/fresh WASM의 시각 선행 조건이 없는 #6024 신규 회귀나 baseline을 추가하지 않았다.
- 실패 v82 분할 실험을 표별로 분리하니 #6024 pi60의 조각 수는6개 그대로이고 마지막 시작행만113→112로 이동했다. 104번째 쪽은 별도 pi70의 마지막 r167이 기존6조각에서7조각으로 늘어나 생겼다(`physical-frame-fragment-count-isolation-v82.json`). r167은 rowspan-touched여서 terminal-note 형상 변경이라는 설명은 성립하지 않는다. 원본 단일행 셀 높이는2850HU=38px, scanner의 논리 잔여는38px이나 물리 잔여는36.7px이고 전체 가시 내용28.5px는 들어간다. 독립 Print33쪽의 마지막 괘선987.080→1023.680px와 원본/로그 SHA를 `stored-terminal-logical-physical-budget-proof-v83.json`에 연결했다. 후속 보정은 원본 행 소속을 검사하는 논리 예산과 실제 상자를 예약하는 물리 예산을 함께 전달하고, 가로 용지 경계의 행 내부 컷 자격을 보존하면서 통째 초과 수용만 분리한다. source 전체가 동결된 현재 WASM 검증 종료 전에는 이 후속 source 변경을 적용하지 않는다.

- v82 retry1 WASM 정상 대조6803/6935 전8쪽도 exit0/177.833초/source불변으로 끝났다. 각각 최저95.40319%/91.57542%, 전체5대5쪽/3대3쪽, 누락0/gatepassed다. 6803 3쪽의 병합칸/경계와 6935 2쪽의 열린 이어받기 하단을 직접 읽었다. 활자와 일부 괘선색 차이는 잔여로 남긴다. 전체 WASM worker는 실제 미달 #2211/#6024를 포함해 exit1/1778.337초로 종료했으며 source1087개 SHA 불변을 별도로 재확인했다. 이 단계의 source/독립6030 Print/통합 기록만 checkpoint commit하고 사용자가 요청한 최신 upstream/devel 동기화·rebase 뒤 후속 분할 보정을 계속한다. 최종 lint/전 회귀/전체 시각 gate/실제 UI/PR/CI/merge/후속 처리 완료를 의미하지 않는다. live PR 재조회도 open30/non-draft23으로 초기 잔여17+후속6이 유지됐다.

## 67. v83 — 마지막 저장 행의 소속 예산과 물리 상자 예산을 분리

- v82 retry1 보정을 `3874bc4be037e84998d548e7755bc689af9d871b`로 commit했다. fetch 이후 upstream/local devel은 모두 `21ed64071f81f35f37618717693bd30f727392c9`이며 local devel ff-only 동기화와 검토 branch rebase가 끝났다. HEAD는 그대로이고 source1087개 SHA도 불변이다(`rebase-anchor-padding-batch-v82-retry1-20261010.json`).
- 독립 근거는 위 원본 pi70/r167과 Print33쪽이다. 저장 반복 제목행의 이어받기에서만 기존 논리 예산을 보존하고, 실제 행 fit에는 현재 paint가 닫는 물리 하단을 사용한다. 마지막 원본행의 선언 상자가 논리 예산에 들어가고 모든 단일행·닫히는 병합 셀 내용이 물리 예산에도 들어갈 때만 실제 마지막 행 상자를 물리 잔여로 확정한다. 이미 앞 쪽에서 잘린 병합 owner, 저장 텍스트가 바뀐 셀, hard break/frame reset, 큰 실제 내용은 이 경로에서 수용하지 않는다. 문서 ID·행 번호·새 허용 px 값으로 대상 범위를 좁히지 않는다.
- 실제 연결: `physical_rows.rs::stored_header_fragment_row_budget` → `fragment/scan.rs`에서 논리/물리 예산 전달 → `scan/runner/row_step.rs`의 마지막 rowspan 행 검사 → `stored_terminal_row_frame.rs`의 원본 소속/전체 내용 확인 → scan의 consumed와 end-row override → `emit.rs`의 예약·PageItem → `table_partial.rs`의 같은 마지막 행 높이 소비. 가로 용지 query는 경계의 intra-row 컷 자격을 계속 계산하되, 물리 상자를 넘는 whole-row 수용만 금지한다. following-plain-text의 strict 계약과 terminal-note 형상은 변경하지 않는다. 각주 refit과 종료 캡션/여백 refit도 두 예산에 같은 추가/차감을 전달한다.
- 한 번의 Native 빌드 뒤 원본9개 쪽수·#6024 11/12/33쪽과 교육14구역 경계·정상6803/6935의 actual cuts/내용 보존을 묶어서 먼저 확인한다. Source 전체와 입력 SHA를 고정한다. 실패 실험 v82와 비교해 pi60 행 소속 개선과 pi70 불필요한 마지막 쪽 해소를 각각 확인하고, 교육 총쪽수·가로 경계 컷도 독립 Print와 대조한다. 후속 코드가 적용되면 v82 시각 증적을 v83 출력 근거로 재사용하지 않는다. 90% 미만인 원본의 회귀/golden/baseline 추가·완화는 보류하며 최종 Native/fresh WASM/확대 판독과 전체 필수 게이트는 계속 남아 있다.

- v83 Native Skia 빌드는 exit0/260.951초/source불변, binary SHA `8f761b682731a903aa16d738211dc5b4ab0aa369e4660be035de02ea73bab576`이다. 마지막 source/physical 분리로 #6024는103쪽을 유지했으나 산재 HWP/HWPX는52→54/53쪽, 교육은414→422쪽이다. 원본9개 쪽수와 두 문서 전체 render-tree/scan 진단 worker는 terminal exit1로 끝났으며 정상 쪽수 변화 때문에 예정된 큰 시각 묶음을 보류했다(`native-after-build-logical-physical-v83-exit.json`). 이 후보는 통합 불가다. source/fixture/baseline/회귀 기대값을 변경해 증가를 승인하지 않았다.
- 보정 방향만 확인하는 #6024 Native 11/12/33/92쪽 별도 진단은 exit0/115.860초/source불변, 94.63039/99.77932/99.47253/99.92761%, 전체103대103쪽/선택4대4/누락0으로 끝났다. 12쪽의 `조정ㆍ시행` 쪽 소속과33쪽의 마지막 병합 owner/괘선/꼬리말을 직접 대조했다. 이 부분 개선을 산재/교육 회귀의 통합 승인으로 쓰지 않으며 fresh WASM도 미검증이다.
- 종료된 WASM boundary 출력의 SVG309개/4,177,447,087bytes를 직접 삭제했다. live argv/open-fd/tracked 없음을 확인했으며 PNG/PDF/JSON/TSV234개 SHA를 모두 보존했다(`output-cleanup-ended-wasm-boundary-svg-v82-20261010.json`). archive는 새로 만들지 않았다.
- 추가한 산재 원본 A/B 진단은 실제 pinned v82 retry1/v83 binary SHA와 원본 SHA를 기록한4실행이 모두 exit0다. HWP pi18의7→8조각, pi26의3→4조각 및 HWPX pi26의3→4조각을 분리했다(`scattered-logical-physical-cut-diagnosis-v83.json`, `scattered-fragment-count-isolation-v83.json`). 최초 차이는 pi9 continuation의 마지막 r53을 old에서는 수용하고 v83에서는 다음 쪽으로 미룬 데서 생겼다. 그 tail이 커져 후속 첫-fragment의 저장 source-frame 선택까지 달라졌다. 원본 r53의 단일행은1566HU이고 상단 시작 rowspan은5행이다. 현재 쪽 마지막 상자의 blank reduction과 해당 span의 source content cut을 구분하지 않은 것이 원인이다.

## 68. v84 — 물리 하단에서 같은 source-unit 컷을 보존하는 행 상자

- v83의 전역 물리 cap이 전체 source 행 높이를 그대로 요구한 가정을 보정한다. 마지막 ordinary row의 논리 상자는 기존 source budget에 들어가되 물리 상자가 더 작은 경우, 해당 행의 단일행 전체 내용과 모든 영향 rowspan의 보정 전후 `(su, eu)`가 같고 그 내용·활성 여백이 실제 상자에 포함될 때만 물리 잔여를 공유 row height로 확정한다. 단일행의 hard break/reset, 편집/무효 source, 객체를 가진 셀, 내용 컷 변경, 실제 점유 초과는 수용하지 않는다. 이전 쪽의 source 컷과 첫 부분행의 이미 수용한 높이도 기존 paint helper로 함께 소비한다. 특정 문서·행·숫자 조건으로 우회하지 않는다.
- 생산은 `stored_row_closing_frame`이고 소비는 row scan의 consumed/override와 기존 emit/partial paint다. rowspan의 cut 확인에는 실제 paint와 같은 `rowbreak_straddle_cut_units`를 사용하고, 줄 점유/활성 여백은 `cell_cut_visible_height`의 공통 결과를 검사한다. 행의 끝 공간을 접는다는 이유로 글줄이나 owner를 숨기지 않는다. 같은 Native 빌드 한 번으로 원본9개/산재 A-B/교육과6024 경계를 다시 묶어서 확인하며, 현재 시각 증적을 후속 source의 성공 증거로 재사용하지 않는다.

- 첫 v84 Native 빌드는 exit0/257.953초/source불변이나 산재54/53쪽·교육422쪽으로 v83의 증가를 해소하지 못했다. 따라서 worker는 두 full render-tree 진단 뒤 terminal exit1로 끝났고 새 WASM/큰 sweep/fixture는 실행하지 않았다. 기존 이름의 `visible_height`를 실제 내용 점유로 추정한 가정도 재검토했다.
- 실제 v84 `RHWP_DIAG_CELL_UNITS` 실행은 exit0이고 원본 pi9/r53/c0의 논리20.880px와 물리20.613333px에서 둘 다 동일한 `(su,eu)=(0,1)`임을 보여 준다(`scattered-source-unit-trace-v84`). 이후 거부는 `CellCutFrame::height`가 줄+여백에 `first_source_frame_owner`의 최소 상자 높이를 다시 max한 데서 발생했다. 이 값을 실제 내용 점유로 비교한 것이 잘못이었다. 기존 source minimum은 유지하되, 그 max 이전의 줄/활성 여백 점유를 `occupied_height`로 공통 생산해 닫는 상자의 fit 검사에 사용한다. 빈 줄의 점유도 이 공통 계산에 포함하며 가시 글자 없음으로 공간을 삭제하지 않는다.
- v84 retry1은 RSPAN 경로의 실제 `cut_row_h`와 일반 경로의 `whole_row_fit_h` 생산자를 구분하고, 이미 수용한 row ledger는 paint와 같은 `mt.row_heights`를 소비한다. 가로 용지에서도 기존 query가 통째 source row 소속을 허용했던 경계만 논리 자격으로 보존하고, 동일 source-unit 컷/실제 점유의 물리 fit 검사는 계속 필수로 둔다. 새 px 허용값이나 document/row 조건은 추가하지 않았다. 두 번째 Native 빌드/원본 countercheck/진단/영향 PNG는 새 source snapshot에서 다시 실행한다. v83/v84의 부분 통과를 최종 후보의 근거로 바꾸지 않는다.

- v84 retry1 빌드는 exit0/258.074초/source불변, binary SHA `e0b3bd4d26fed58a4dd3f77294ac2213356feb1390eaa9ef1a0d4fa352a628db`다. 교육422→421쪽의 한 쪽만 줄고 산재54/53쪽은 그대로여서 실패 후보다. worker는 원본9개/두 full render-tree 진단 뒤 terminal exit1이다. `scattered-source-unit-trace-v84-retry1` exit0에서도 r53/c0의 source/physical 컷은 둘 다0..1이나 row closing 성공은0건이다. 따라서 source-opening max 하나만 제거하면 된다는 앞 가정은 충분하지 않았다.

## 69. v85 — 저장 span의 같은 prefix에서 줄 끝 간격을 공통 계산

- 추가 추적에서 `cell_units_fitting_height`는 저장 plain span의 partial prefix 마지막 줄에서 다음 owner로 가는 간격을 제외하지만, `CellCutFrame::occupied_height`는 특별한 reset trim만 제외하고 그 일반 간격을 다시 점유로 요구했다. r53/c0의 첫 유닛은21.333333px, source line은1000HU+spacing600HU이며 두 높이에서 같은 첫 유닛을 선택했다. current helper가 이 동일 컷 뒤에 거부되는 실행 경로와 실제 계산을 대조했다. cached opening max 제거만으로 해결됐다고 보고하지 않는다.
- 동일한 기존 판정을 `stored_span_cut_line_tail`로 생산해 capacity와 `CellCutFrame`의 점유가 함께 소비한다. full-cell 마지막 유닛, 빈 spacer/빈 줄, 객체·편집/재조판·무효 source는 기존 계약을 유지한다. source-opening 최소 높이와 source-unit 소속 검사는 그대로 보존한다. 초기 반복 제목행·이어받기·닫는 rowspan의 실호출 경로에서 실제 Cut/점유/paint를 다시 대조한다. 새 허용값·baseline·fixture·검사 기대값을 추가하지 않는다.
- 새 Native 빌드 한 번 후 원본9개 및6024/교육 full 진단을 먼저 다시 수행한다. 정상 source의 추가 쪽이 사라진 뒤 영향 PNG와 정상 대조를 진행하며, 최종 fresh WASM/전쪽90%/필수 회귀·lints·실제 backend/PR·CI·merge·후속 처리는 여전히 완료 조건이다.

- v85 Native 빌드는 exit0이며 source 불변, binary SHA `619c8645213c701ca0bb00a2384f5c65e83d4528f013b710a14bf9df8045390d`다. 원본9개 countercheck와6024/교육 full JSON 진단이 끝났으나 산재54/53쪽·교육421쪽으로 실패했다. 실제 v85 unit trace도 r53/c0의두 컷0..1을확인하지만내용+아래안여백20.880px가물리예산20.613333px보다커서수용하지않는다. 따라서 일반 줄 끝 간격 공유만으로 해결했다는 주장은 성립하지 않는다.

## 70. v86 — 고정 하단 선행 cap을 철회하고 부분 조각의 온전한 내부 행을 보존

- 실제 paint `table_partial.rs`의 `fold_slack` 생산·소비를 대조했다. 모든 마지막 셀의 내용·아래안여백을 지키는 여분의 최솟값을 구한 뒤, 저장 줄간격/쪽 하단 초과 중 큰 값을 그 여분 한도에서만 접는다. `single_cell_page_fragment_bottom`은 모든 조각에서 무조건 달성되는 상한이 아니다. 이 하단을 일률적으로 scan에 선행 cap한 v83–v85의 가정이 잘못되어 해당 실험 source를 검증된3874bc4be로 복원했다. 원본·실패 binary·전체 source snapshot·원인 로그는 그대로 보존했다(`retired-global-physical-cap-candidates-v86.json`).
- 후속 구현은 기존 실제 행 높이 원장만 보정한다. `record_stored_fragment_rows`가 시작/끝에 부분행 하나가 있다는 이유로 중간의 모든 온전한 행까지 버렸고, `stored_fragment_row_plan`도 부분행으로 시작하면 앞 조각의 확정된 행 높이를 소비하지 않았다. 이제 per-row 부분 시작행은 제외하되 뒤의 완전 행은 보존한다. 부분 끝 블록 전체와 실제 마지막 높이가 명시되지 않은 끝 행은 미확정으로 남긴다. 블록 시작 컷의 물리 행 분배는 이 변경에서 추정하지 않는다. 빈 물리 밴드나 미확정 부분행의 합을 원본 높이로 새로 인증하지 않는다.
- 연결: 기존 scan/emit의 fragment 측정과 명시된 마지막 높이 → `StoredFragmentEnd`로 컷 범위를 전달 → `accepted_row_boxes`에 완전 내부 행 기록 → 다음 fragment의 measured-table/누적 높이·scan과 PageItem paint에 동일 계획 전달. 부분 시작행의 컷/remaining-band와 원본 source-owner 선택은 유지한다. 새 document/row ID, 허용 px, baseline·회귀 기대값 변경 없이 한 Native 빌드 후 원본9개·두 큰 문서 전체 JSON·영향/정상 PNG를 묶어 확인한다. 산재 증가가 해소되는지와 교육415쪽 계약을 별도로 판정하고 전쪽90%/freshWASM/최종 게이트를 아직 완료로 보고하지 않는다.

- v86의 첫 빌드는 실행 중인 compiler를 코드 검토상 누락 때문에 명시적으로 중단했다(exit101/SIGTERM/source불변,115.551초). 출력 공백·관찰 timeout으로 중단한 것이 아니며 렌더링 결함 재현이나 검증 성공으로 세지 않는다. 블록 시작 컷에서 미래 행의 개별 물리 분배가 미확정인데 기존 plan의 차단 조건을 제거했던 부분을 보완했다. record와plan 모두 block-start를 유지하고, per-row 부분행 뒤의 온전한 행만 소비한다. 최종 source를 다시 동결한 v86 retry1을 새 태그로 실행하며 첫 실행 결과를 덮어쓰지 않는다.

- v86 retry1 Native 빌드는 exit0/254.620초/source불변, binary SHA `3737276c867a329d46d7a1351a8617c59e57cbfd7353d378631db1159971e48c`다. 산재52/52쪽과1133각3쪽·6935 3쪽·6803 5쪽·6024/6761각103쪽은 유지됐지만 교육은416대415쪽이다. v81 동일 원본 전체 진단과 비교해 S1/pi3과 S25/pi11에 추가 조각을 분리했다. S14의 기존 미달은 그대로이며 두추가쪽을개선완료로보지않는다(`education-fragment-count-isolation-v86-retry1.json`).
- 원본 Native full 정상 대조에서6935는전3쪽/최저91.57542%/누락0으로통과했지만6803 5쪽은52.48057%로새회귀다(앞4쪽98.94637/97.60800/96.43145/95.82690%).5쪽review를직접열고이전조각소유의A9-10가넘어온것과셀의세로성장을확인했다.6024선택4쪽은12쪽71.48133%로계속보류다.두visual job 및전체worker는terminal exit1이며WASM·fixture/golden·기대값갱신을진행하지않았다.
- 초기 편집축9개를같은source로새Native sweep했다. 연속표2개/column-top·뒤문단2개/inline 일반·누름틀·책갈피3개/스타일 탭·배경2개가모두전1쪽/100%/누락0/gatepassed이고worker exit0/482.064초(빌드대기포함)다.대표4개의review를직접열어표원점·뒤문단·inline앞뒤텍스트·녹색배경하단을대조했다(`native-original-edit-axes-terminal-evidence-v86-retry1.json`).Native 괘선은Print보다연한회색이며활자·작은테두리차이는남는다.ignored API capture인입력을정상한컴생성본/committed fixture로승격하지않고,현재freshWASM·실제API/Studio·원PR전체완료와구분한다.
- 실제6803 전체JSON·scan 추가진단은exit0/0.115초/source불변이다. 중간r10의ordinary내용이끝났다는`complete_physical_end_cut`로39.786667px를온전한행높이로원장에기록했지만다음fragment는여전히r10에서시작한다.물리행원본156.386667px와구분하지않아뒤cursor13의prefix를116.6px작게복원했다(`issue6803-accepted-row-unknown-prefix-v86-retry1.json`).교육에서도미확정부분행이있는prefix를원본높이와섞고S25 r161의선언잔여를11.973333px더했다.같은미확정합문제를한꺼번에고친다.
- v86 retry2에서는내용완료와행기하완료를분리한다.끝cut/limit이있는행은ordinary내용이끝나도완전행원장에넣지않는다.걸침시작owner와닫히는span의선언잔여를계산할때그prefix의모든물리행높이가확정된경우에만원장을소비한다.모르는부분행을원본높이로새로인증하거나새px예외로대체하지않는다.부분행전체높이의합을직접확정하는공통결과가없는범위는기존계약과미검증을유지하며완료범위를축소하지않는다.새source에서원본/전체진단/6803·6935전쪽/6024대표를다시검증한다.

- v86 retry2 Native 빌드는exit0/256.179초/source불변이며binary SHA `a41dbbebdf51f7c2fa928c6f2a7d5bd80a80a91bb9c904b3320fbd1afed8f2e2`다.원본9개쪽수는v82 retry1과모두같아졌다(산재52/52·교육414·6024/6761각103·나머지3/3/3/5).Native6803전5쪽최저95.40319%/6935전3쪽최저91.57542%/누락0/gatepassed로회귀가사라졌다.6803 5쪽과6935 2쪽review를직접읽어A9-10의앞쪽소유·마지막세로상자및이어받기열린하단을확인했다.6024 12쪽71.48133%와교육414대415쪽은여전히남아전체worker exit1이다.이는새WASM/전회귀/lints/전체gate/통합승인이아니다.
- 최신fetch에서upstream/devel은계속 `21ed64071f81f35f37618717693bd30f727392c9`다.부적합한기존회귀의이관여부는feature후보를base증거로대체하지않고이SHA에서이미회귀·원본이등록됐는지와독립Print미달을직접검증한다.진단용detached worktree `/home/tsjang/rhwp-review-base-20261010`를만들었고clean/HEAD를확인했다(`upstream-base-visual-audit-plan-20261010.json`).고정공용 `target/pr-review`를순차재사용하며새Nativebase binary를SHA고정해동일원본·PDF·font·범위로비교한다.아직이관판정·이슈등록·테스트/자료이동은하지않았고,현재수정의별도회귀를기존미달이라는이유로제외하지않는다.이검증worktree는조사끝에live작업과용도를확인한뒤정리한다.

- 정확한upstream base Native build는exit0/259.818초/source불변이며binary SHA `b5fa531bdf40ce2b80692b11d9451b1f96ab2579260f644ddbffdff3ac5a98e0`다.원본9개모두base commit blob과현재입력SHA가같다.쪽수는base6024=105/현재103/독립103,base교육=413/현재414/독립415이며나머지7개는base/head동일이다.이count만으로같은출력결함이나90%미달/이관요건을확정하지않고base의같은영역Native review를계속산출한다(`upstream-base-native-followup-v87-exit.json`).base binary는build record의21ed64071 source이며호출workspace HEAD3874bc4be/dirty root guard와혼동하지않는다.
- 현재v86 retry2 source의Native새회귀해소를확인했으므로별도의freshWASM도순차공용target에서빌드하고pkg/public바이트일치와고정pkg를확인한뒤초기편집9개·정상대조8쪽·6024대표를검증한다.immutablebase Native출력은병행하며두source를별도로기록한다.현재code/head가변하지않도록동결했고완료전큰source변경·이관·기대값수정으로증거를무효화하지않는다.전체PR검토/전90%/최종회귀·lint/CI·merge/후속처리완료를의미하지않는다.

- base Native 세 visual job는 모두미달을실제로검출해exit1로종료했다.6024 base11/12/33/92쪽은96.18033/71.46986/47.54771/27.62657%,교육base247/248/259/260/261/262쪽은51.70013/71.17023/64.17688/66.50120/15.17174/16.96349%,6761 base78/79쪽은50.22524/17.91891%이며선택쪽누락은없다.완료원본9개SHA·basebinary·고정SHA와현재Native를`upstream-base-vs-current-native-boundaries-audit-v87.json`에연결했다.호출workspace HEAD는basecodeSHA로대체하지않고별도binarybuildsource로설명한다.
- base6024 12쪽·6761 79쪽·교육261쪽review를직접열었다.6024는base도맨위`조정ㆍ시행`행이다른쪽소유이며12쪽의현재71.48133%와같은기존증상을보인다.6761 base도다수그림이겹치며교육base는261쪽에서기준표대신다음마당흐름도를보인다.이로써기존미달이존재함을확인했지만,아직각실패회귀함수·base등록commit·검사적합성과현재의별도회귀를전수대조한것은아니므로테스트/원본의이관이나이슈종료를완료로보고하지않는다.새이슈/기존이슈연결도아직수행하지않았다.현재source의freshWASM은실행중이다.


## 71. v87–v88 — 부적합한 기존 회귀의 함수별 적합성 및 정확한 base 대조

- 기본 경로는 maintainer_general, 보조는 intake_and_review·local_validation·multi_pr_update_branch·visual_fixture_evidence다. 모 workflow/선택표와 해당 자식 절차를 읽었다. 현재 remote PR/push/merge는 미수행이며, 새 회귀의 시각 선행 조건과 기존 미달 이관 조건을 구분한다. 지침 파일은 수정하지 않는다.
- `per-function-rendering-test-suitability-audit-v88.json`은4개 source의 base/current SHA와 `.rs.txt` 원문을 보존한다. #6024 기존105쪽 pin은 독립Print103쪽과 다르고, 현재103쪽+전쪽 라벨 보존으로 바꾼 검사도 전쪽90% 전에 복원한 유효 회귀로 인정하지 않는다. #5679는 고정11쪽·셀 오른쪽451.8px 및8/14px 한계를 사용하는 렌더링 검사이며 순수parser 검사로 분류하지 않는다.
- 교육의 `partial_retreat_inside_a_page_does_not_break`는 현재와base 모두413쪽을 고정하지만 독립Print는415쪽이다. 이 assertion은 해당 문단의 부분 후퇴/쪽 소속을 직접 확인하지 않으므로 정확한 검사로 인정하지 않는다. 현재415쪽 검사 `page_count_matches_independent_print`의 v81 실제FAIL414대415도 보존하며 기대값을414로 낮추거나 잘못된413 pin으로 복원하지 않는다. 같은 파일의 MARKET 원본 검사와 #6981의 정상6803·다른6795 대조는 교육미달만으로 제거하지 않는다. 교육과 다른3개 문서를 함께 쓰는 floating-anchor line-metric/edit 계약도 source filename만으로 통째 제거하지 않는다.
- GitHub 최신 #7445의 댓글을 직접 읽었다. 교육의 동일 원본/413쪽 검사 FAIL은 기존댓글6063814441, 화학제품 전체원본의103쪽 미달은 기존댓글5981264035에서 이미 추적한다. 새 중복 이슈를 만들지 않는다. 화학제품19함수를 과거 이관했다는 문서는 보정105에서 정상함수를 복원한 정정도 포함하므로 과거 제거 목록을 현재 제거 권한이나 결과로 재사용하지 않는다. #7226/#7531/#6761도 현재OPEN이지만 각각의 원인 범위를 실제body와 대조하며 broad filename 일치만으로 연결하지 않는다.
- 위임전결규정의 exact filename/한글명으로 열린 이슈를 검색했다. #5941의 서베이 총괄은 현재12쪽 행 소속 결함의 동일 이슈가 아니며 전용 이슈는 발견하지 못했다. 신규 등록 전 최종 base/current 전쪽 Native TSV를 추가 대조한다(`native-base-current-6024-full-suitability-worker-v88-exit.json`). 실제 immutable base/current binary와 같은303104byte 입력·103쪽Print·font·96dpi를 사용한다. 선택쪽 TSV나 점수 개선만으로 현재 별도 회귀를 면제하지 않는다.
- 현재12쪽review를 다시 직접읽었다. 독립Print 맨위 `조정ㆍ시행`이 현재 앞쪽에 남고 이어지는 결과보고/계획수립 및24–30행 경계가 위로 어긋난다(현재71.48133%, base71.46986%). PNG를 사용자에게 표시했다. 전쪽대조/freshWASM은 실행중이며 아직 이슈등록·active source/test/fixture 이동은 수행하지 않았다. 이관은 제품결함 해결·통합완료·이슈종료가 아니다.


## 72. v88 — 현재 fresh WASM 공개 API와 실제 Studio 출력/포인터 검증

- v86 retry2 source의 최적화 fresh WASM build는exit0/source불변으로끝났다. JS SHA `4628f097cddafb073362cc2ff03d19744de3e037aa62d311b4bd4386528c7a20`, WASM SHA `ab5e01324ad024dd34979f03a08d2da6844d26ff31c49069c17de0fe849779b6`이며root pkg와Studio public 둘다같다. immutable pkg는 `wasm-pkg-current-ledger-v86-retry2-3874bc4be0`다. Native같은frozen Rust source이며 HEAD3874bc4be만으로dirty v86 code를표현하지않는다. 초기편집9개/정상8쪽/6024경계의canonical freshWASM sweep은계속실행중이다.
- 공개API6범위를동일pkg로재실행해6PASS/exit0/2.581초/source불변을확인했다(`wasm-public-contracts-current-ledger-v88.json`): 7653+7656표뒤문단포함,7641중첩표의이웃칸비침범,7658탭+HWP/HWPX왕복,7657셀스타일보존,7647잘못된4식의칸보존+합계3,7654한글/emoji/말미공백미주캐럿의paint경계다. 이를21개잘못된식/45개정상식전체검증이나전쪽Print통과로확대하지않는다.
- #7566의기존source 계약4개를공개WASM API로실행해모두통과했다(`wasm-clickhere-wrapper-current-v88.json`). 2열비투명대조/투명1×1래퍼의FIRST·SECOND guide 둘다같은2단계cellPath와서로다른host소속을보존했다. 첫field활성화만숨기고둘째guide는남으며clear가원래SVG를복원했다.80행부분래퍼의첫칸/마지막guide도서로다른쪽에서같은논리소속을보존하고마지막guide활성화가통과했다. ignored API-generated HWPX를새정식fixture/한컴생성본으로승격하지않는다.
- 실제Studio 마우스로첫guide를클릭하는독립사용자입력경로도exit0/7.347초/source불변이다. 활성화API를test가직접부르지않고mouse.click을보냈다. 클릭전guide2개·hitTest2단계cellPath,클릭후FIRST만숨고SECONDGUIDE가남는다. 실제응답WASM SHA는위ab5e0132와같고browserpageerror0이다(`studio-wrapper-field-pointer-v88/evidence.json`,before/after screenshot). 이는첫투명래퍼의실제포인터증거며80행GUI전체여정까지실행했다고하지않는다.
- 같은freshWASM의6935 실제Studio PageRenderer Canvas2D 전3쪽을새캡처했다. 독립한컴Print와각쪽91.99433/95.07682/96.33170%,전3대3이며actualmodule response 일치/pageerror0/DOM이미지미완료0이다. capture exit0/16.377초/source불변이고비교도exit0이다.2쪽review를직접열고사용자에게표시하여하단이어받기표의열린구분선·앞완료표닫힘·다음내용소속을확인했다. 작은활자획/회색괘선차이는남는다(`studio-issue6935-page-renderer-v88/independent-print-review/`). CanvasKit출력또는최종전체gate로확대하지않는다.
- base6024 전쪽canonical실행은105쪽SVG/Tree와103쪽Print의쪽수불일치를실제로검출해TSV전단계exit1이다. 이를전105쪽측정완료로보고하지않는다. 같은basebinary를그대로사용해공통1–103쪽을새태그 `native-upstream-base-6024-common103-suitability-v88`로비교하며원문105대103쪽차이와비대응104/105쪽을별도로유지한다. 관찰timeout으로재시작하거나gate공차/코드를바꾼것이아니다. 현재103쪽전쪽TSV도실행중이다. 2211의base동일입력SHA를추가확인하고현재6442/6030/3308정상대조전쪽을앞Native job종료뒤묶어서실행하도록준비했다.아직관련source/test이관·새이슈등록·커밋/rebase는하지않았다.

- 초기편집9개의현재freshWASM canonical sweep이exit0/source불변으로종료했고각1쪽100%·누락0·gatepassed다(`fresh-wasm-original-edit-axis-terminal-evidence-v88.json`). inline책갈피0257/스타일배경0547의새review를직접읽어앞뒤텍스트·표소속·녹색배경하단/테두리를확인하고사용자에게표시했다. 괘선회색/검정과작은glyph획차이는남는다.
- 같은실행의현재Native font-policy export SVG9개는이전v86 retry1 Native SVG와모두SHA동일이다. 따라서이전Native PNG는명시재사용하고현재freshWASM PNG는새출력으로구분했다. 각PNG SHA와Native 현재export동일성·실제source build를고정한보조TSV18개는exit0/각1쪽100%다(`native-original-edit-axis-current-svg-identity-v88.json`, `initial-edit-axes-tsv-current-proof-v88-exit.json`). 새Native raster실행으로보고하지않고원본전쪽수와direct review는canonical manifest에별도로연결한다.
- 현재freshWASM 정상6803전5쪽은98.94637/97.60800/96.43145/95.82690/95.40319%,6935전3쪽은91.57542/94.30407/94.58717%이며누락0·쪽수5대5/3대3·두gatepassed·worker job exit0이다. Native와같은source에서검증했다.6024경계job는여전히실행중이고이를전체worker종료/통합승인으로세지않는다.
- 위임전결규정전용이관이슈의원본/Print/job/SHA/base등록source/검사별적합성/이전·새경로/관계형복원조건/명령을초안으로구체화했다(`issue-10857-legacy-rendering-fidelity-v88-draft.md`). #5679/#6024는CLOSED임을재조회했고동일현재잔여의OPEN이슈가없다. 초안의실행중전쪽대조와아직원격미게시인PDF/PNG를등록완료나안정공개URL로보고하지않는다. 현재active검사/원본이동은수행하지않았다.

- freshWASM 전체worker는terminal exit1/1833.623초(빌드대기/빌드/visual포함)다. build·편집9개·정상8쪽은exit0이며6024경계4쪽만exit1이다.6024각쪽96.17576/71.48133/98.80354/99.92761%,전체103대103/누락0/re_review_required로Native와같은기존12쪽미달을확인했다.정상6803 5쪽freshWASM review도직접읽어새회귀였던A9-10이마지막쪽에되풀이되지않고마지막행/닫힘과내용소속이Print에대응함을확인했다.이를부적합한기존회귀이관이나초기전체PR승인으로확대하지않는다.
- 현재WASM 실제CanvasKit 무늬6종의첫관찰은함초롬바탕미등록→Noto Sans KR1fallback이었다(`browser-patterns-current-v88.json`). readiness통과를글꼴/시각통과로세지않고실제존재하는HCR Batangface를fc-scan으로확인해두Canvas에직접등록했다. font SHA `8664669bd2d6a4fe8b7538706d2bb2a1004c074aa02dca59051e515e731294f5`,입력SHA `216166ae3b7d760fd03d9ebb297e8bb268fca5a13e0534380a8a8465901a58ae`다.
- 글꼴공급후Canvas2D/실제CanvasKit software모두독립한컴2020 Print전1쪽100%이며기준PDF SHA `9ac1553a70c91cb86d890ea9d9e2b50cd5c34bc230420db3512a2a9f7e9b41e4`다. CanvasKit registeredface1·fallback0·unsupported0·rendererror0·hiddenCanvas2dOverlayfalse이며capture exit0/43.894초/source불변이다. 같은한쪽review를새로생성하고CanvasKit review를직접읽어6종색·방향·격자·대각주기를확인하고사용자에게표시했다. 작은AA/선두께차이는남는다(`actual-pattern-backends-supplied-font-v88/metrics.json`). 최초진단파일명v88을쪽88로읽어review를누락하던진단alias를바이트동일rhwp_001.png로고쳐쪽1review존재도확인했다. 제품코드·원본·PDF는바꾸지않았다.
- 최신GH조회는semanticist21 OPEN30/비draft23개로이전범위와같다(`semanticist21-open-pr-current-v88.json`).초기17잔여와후속6개는그대로전체목표에남아있다.기존base/head문서전체Native대조및queued정상대조는실행중이며각liveprocess를직접확인했다.완료뒤검사/원본의정식이관·current단계커밋/upstream동기화/rebase·최종Rust/lint/전쪽시각/통합PR/CI/merge/후속처리를계속한다.


## 73. v89 — 위임전결규정 전쪽 base 대조와 별도 이슈 등록

- 현재Native 전103쪽은exit1/1141.211초/source불변으로종료했다.12/19/27/59쪽71.48133/74.28775/63.15748/87.48894%,최저27쪽63.15748%이며누락0·103대103이다.직접세쪽review를추가생성해읽고사용자에게표시했다.기존terminal PNG를해시변경없이재사용한review이며새export/raster로보고하지않는다.
- 정확한base21ed64071 Native의공통103쪽은exit1/1418.451초/source불변이다.105대103쪽차이와비대응base104/105쪽을유지하고공통103쪽을측정했다.현재미달4쪽은모두base미달이며19/27/59쪽은71.75853/67.71964/47.28116%다.전체base미달목록/최저50쪽8.62142%와base통과→현재미달0쪽을`native-base-current-6024-full-comparison-v89.json`에연결했다.전체미달의개선이나0 crossing만으로승인하지않는다.
- 27쪽은현재점수가낮아진항목이므로독립PDF long-rule좌표와실제RenderTree를대조했다.표상단base94.5/current96.4/Print96.3747px,표높이base/current339.1px다.현재1.9px 이동은독립상단에가까워졌으며`주요통계관리`의기존잘못된쪽소속과내부행차이는둘다남는다.19쪽의전체높이base920.9/current929.1/Print928.744px도현재가근접한다.59쪽은base앞표한개대current앞표/다음정책표두개로독립Print의소속에가까워졌으나행차이는미달이다.현재점수하락을기존미달만으로면제하지않고같은의미/경계를직접대조했다.
- 동일원본·잔여출력결함의OPEN이슈검색및CLOSED5679/6024body조회후 [#7700](https://github.com/edwardkim/rhwp/issues/7700)을실제로등록했다.담당jangster77/OPEN이며API로본문UTF-8·전송원문byte일치/담당을재조회했다(`issue7700-created-api-verified-v89.json`).기존105쪽pin·고정8/9·66/67쪽및5679고정px검사의적합성·같은원본/Print/job/SHA·복원조건·현재미검증/원격미게시증적을구체적으로적었다.아직active검사/원본의이관이나이슈종료는수행하지않았다.
- issue7700/delegation-rules에원본303104byte·Print769432byte·base/current대표PNG·두검사base/current.rs.txt를SHA동일한보존사본으로준비했다.새정상fixture/golden이아니며active경로는이단계에서유지한다.ignored TSV/로그/중간JSON은커밋하지않는다.현재보정과검증checkpoint를커밋하고사용자지시대로fetch/devel동기화/rebase후검사별이관을진행한다.
- 정상대조현재Native는3308전9쪽최저97.36291%,6442전3쪽99.19302%,6030전2쪽98.69917%이며쪽수일치·미달0이다.직접review/현재freshWASM의해당범위는별도완료조건이며TSV not_evaluated를승인으로바꾸지않는다.2211은base/current전4쪽최저87.65555%,미달1/3쪽이고PNG4개가모두byte동일이다.새회귀가아닌기존차이임을직접출력으로확인했으나90% 정상대조로보고하지않으며기존4068의다른잔여와원본/증상을구분해후속판정한다(`native-final-countercases-and-base2211-audit-v89.json`).두counterjob도terminal/source불변이며전체worker exit1은2211기존미달때문이다.


## 74. v90 — 체크포인트 동기화와 #7700 검사·원본 이관

- 진행73의 보정·증거를 `9b53bd699126205b7a93fff326167f575252f1ea`에 일반 커밋한 뒤 `git fetch upstream devel`, 로컬 `devel`의 `merge --ff-only upstream/devel`, review branch의 `git rebase upstream/devel`을 순서대로 완료했다. 모두 exit 0이며 base/local devel은 `21ed64071f81f35f37618717693bd30f727392c9`, review HEAD는 9b53으로 유지됐다. 동기화로 production source를 변경하지 않았다.
- #7700의 이관 요건과 진행73의 같은 원본 base/head·독립 Print 직접 대조에 따라 활성 `issue_6024_band_continuation_does_not_redraw_merged_cell_content` 및 `issue_5679_justified_trailing_space_does_not_take_word_distribution`을 제거했다. 정식 검사 감소는 두 개다. 변경 전후 본문은 자동 수집되지 않는 이슈 자산 `.rs.txt`에 보존한다.
- 원본 HWP와 대응 Print PDF를 활성 samples/일반 PDF 경로에서 제거했다. 이미 커밋한 `mydocs/pr/assets/issue7700/delegation-rules/` 사본과 9b53의 이전 파일이 각각 바이트 동일함을 재검증했고 SHA·bytes를 유지한다. body_overflow의 해당 2건 행, text_overlap의 1건 행, PDF references의 대응 행만 제거했다. 이관은 #7700의 제품 결함 해결·시각 통과·이슈 종료가 아니다.
- 같은 HWP의 HWP5 roundtrip·IR field sweep은 `tests/common/preserved_format_inputs.rs`로 이슈 자산을 원래 논리 이름으로 계속 읽는다. 기존 IR baseline 3206건 행은 바이트 동일하게 보존하고 정상 원본·기준값·다른 검사에는 손대지 않는다. 이슈 원본은 렌더링 samples 자동 수집에서는 빠지며 parser/serializer 검사만 별도로 유지한다. `#[ignore]`·새 제외 분기·누락 시 조용한 return을 추가하지 않았다. 같은 원본의 현재 출력에서 새롭게 발생한 회귀를 이관으로 해소했다고 보고하지 않는다.
- 정확한 base 비교의 manifest 및 unit-tier 정책 검사, fmt check가 source 불변/exit 0이다. 세 Clippy·workspace build 및 보존 계약/정상 대조군 nextest를 고정 공용 `target/pr-review`에서 순차 실행한다. 모든 step의 실제 terminal exit/source hash는 ignored `issue7700-relocation-validation-v90.json`에 보존한다. 진행 중인 결과는 통과로 세지 않으며 이 진단 범위는 최종 전체 nextest/Visual Sweep/통합 head CI를 대신하지 않는다.
- #7647의 기존 Rust 독립 입력/기대값 21개 오류식·45개 정상식을 현재 공개 WASM API로 모두 실행해 **66 PASS / 0 FAIL**, 실제 Node terminal exit 0을 확인했다. 오류식 21개는 write_result=true로 호출해도 매 호출 후 전체 HWPX 문서 바이트가 변경 전과 동일하다. 정상식은 A5를 평가 위치로 하고 A1=11…E5=55의 같은 값으로 독립 기대 결과 45개와 정확히 일치한다. 단순 SUM 한 건만으로 전체 범위를 주장하지 않는다. ignored `wasm-formula-complete-contract-v90.json`에 식별 결과/소스 SHA를 기록했다.
- 위 WASM 패키지는 실제 v86 retry2 fresh build SHA `ab5e01324ad024dd34979f03a08d2da6844d26ff31c49069c17de0fe849779b6`를 확인해 재사용했다. 이번에는 rendering code/package를 바꾸거나 새 rendering fixture를 추가하지 않았으며 새 fresh build로 보고하지 않는다.
- 최신 GH 조회는 OPEN30/비draft23개다. 초기18개 통합 검토와 뒤 6개 별도 검토의 순서를 유지한다. education 원본의 잘못된 413쪽 고정과 현재 독립415쪽 미달, chemical/기타 잔여·최종 gate·통합 PR/CI/merge/후속처리는 여전히 미완료다. 정확한 기존 관계 검사·다른 원본 검사는 문서의 미달만으로 일괄 제거하지 않는다.

- 위 9개 편집 상태의 input HWP/HWPX·Native 재사용 review·최신 WASM review/overlay 36개 파일을 [초기 편집 증적](../assets/semanticist21-20261008/initial-edit-axes-v86-retry2/README.md)에 바이트 동일하게 보존했다. 입력/Print SHA·실제 생성 방식·재사용 범위·남은 괘선/AA 차이를 기록하고 PDF는 이미 커밋한 같은 기준 파일을 링크한다. 새 test/golden/정상 한컴 생성본으로 분류하지 않는다. 최신 WASM 9개 review를 모두 직접 판독했고 자료의 상대 파일 링크 45개가 실제 파일에 대응한다. 원본 복사·PNG SHA 기록은 ignored `initial-edit-axis-assets-preservation-v90.json`에만 둔다.

- 정상 대조 #6442/6030/3308의 현재 WASM 전3/2/9쪽은 terminal exit0/162.907초/source불변으로 종료했다. 각 최저99.19302/98.69917/97.36291%, 미달0·누락0·Print 쪽수일치이며 Native v88의14개 PNG와 모두 SHA동일하다. 최저쪽과 마지막쪽 review5개를 새로 합성해 직접 판독하고 [증적](../assets/semanticist21-20261008/normal-countercases-v90/README.md)에 보존했다. 마지막 표·셀 포함·내용 순서에는 큰 새 이동/누락을 관측하지 않았으나 #6442의 옅은 내부격자, 글자굵기/괘선색상 및 #3308의 작은 줄간격 차이는 남는다. 원인/base동일성은 미검증이며 높은 점수로 전체 피델리티 해결이나 새 회귀 면제를 주장하지 않는다.
- 같은 무늬6종 입력의 현재 Native SVG+webfont 인쇄 sweep은 terminal exit0/48.041초/source불변, 전1대1쪽100%/gatepassed다. 새 Native review를 직접 읽어 6종의 색상/방향/셀 소속을 확인했고 현재 WASM 실제 Canvas2D/CanvasKit 두 경로의 기존 v88 100% 직접 판독 증거와 함께 [입력/대표PNG](../assets/semanticist21-20261008/patterns-six-v90/README.md)에 바이트 동일하게 보존했다. Native Skia 직접 PNG paint까지 지원한다고 확대하지 않는다.
- 직접 판독의 글자굵기 차이를 별도로 추적해 embedded TTF name table에서 #6442 선언HY견고딕/한컴돋움과 #6030 한양견고딕이 실제 Noto Sans KR ExtraLight/Regular/weight400임을 확인했다. 기존 공급 디렉터리의 존재와 CSS 이름만으로 실제 face 공급 완료로 판정하지 않는다. `/usr/local/share/fonts/hancom-office-2020/Hwp/HYGTRE.TTF`(실제HYgtrE/HY견고딕, SHA82f70d5e…) 및 `All/HDOTUM.TTF`(실제Haansoft Dotum/한컴돋움, SHA2989f379…)를 fc-scan/실제bytesSHA로 검증하고 해당 두 실제 경로를 명시해 같은3문서 전쪽Native→WASM을 순차 재실행한다. 이전14쪽 고득점과README/PNG는 대체font 포함 진단으로 구분한다. production source나baseline을 바꾸거나 글꼴예외로 면제하지 않는다. 실제전체SHA는 ignored `verified-actual-hancom-face-supply-v90.json`에 있다.
- #7700 본문에 실제 이관 범위·base공통103쪽/현재전103쪽 최종대조·31PASS와필수lint/정책결과·복원조건을 갱신했다. `gh api`로본문UTF-8전송원문일치/OPEN/담당을재검증했다(`issue7700-local-relocation-validated-api-v90.json`). 미게시자산을공개rawURL로표시하지않고최종전체gate/통합PR완료로확대하지않는다.
- #7615 원PR의 최신head를 별도 조회하니 `4a6c27c5ffa54e4988a7e0e7956115d933725aca`이며, 초기검토pin `ed9c284…` 이후 #7662의두commit과gap예약제거commit이포함돼있다. 초기18개의검토pin과최신authorhead를같다고보고하지않는다. 사용자순서대로초기18개후후속6개/해당의존변경을검토하며, 최신head범위미포함상태에서원PR전체를완료/close하지않는다. 실제commit목록은 ignored `pr7615-current-head-and-commits-v91.json`에있다.
- #7700 이관의 순차 검증이 terminal exit0/1458.539초로 종료했다. 각 step source불변이며 fmt/manifest-base/unit-tier-base/기본Clippy/WASM-libClippy/workspacebuild/workspace-alltargetsClippy가 모두exit0이다. 관련 nextest는 build포함707.979초/실제검사168.261초, 31PASS/0FAIL/10351skip(2slow)다. HWP5 roundtrip21·IRfield4·6803소유/지면4·6935관련2개를 실제 실행했다. 이 결과는 최종전체nextest와통합headCI가 아니다.
- #6442 옅은 내부격자의원인을같은actualHY/HDOTUM공급조건의고정upstream21edNative3쪽으로대조했다. Base/currentPNG가모두SHA `2638e4bad47b22a7c2192b8c053764f1f2fbeb302b96d011fecd3723dd3b9ebb`로byte동일이다. 따라서관측한3쪽그림의격자와paint차이는현재보정의새회귀가아니지만Print와의완전일치/해결을뜻하지않는다. Native선택3쪽만검증했고baseWASM이나base전체3쪽검증으로확대하지않는다(`base-current-6442-p3-paint-identity-v90.json`).
- 추가embeddedfont검사에서기존9편집Native정책SVG의선언함초롬바탕이실제Haansoft Batang/한컴바탕이었던점을확인했다. 대응Print의embeddedface는HCRBatang이다. 실제patternCanvas2D/CanvasKit에공급해검증했던HCRBatang SHA8664669bd2d6…와같은원본TTF, 실제HY견고딕·한컴돋움3파일을ignored `verified-declared-faces-v91/`로byte동일하게복사/fc-scan했다. Glyph내용·입력을변조하거나font파일을Git에커밋하지않는다. 이경로를우선해같은9편집·3정상대조·Native무늬6종을묶어재검증한다.
- v91의첫Native9개canonical실행은terminal exit0/361.659초/source불변·각1쪽100%/gatepassed다. 단지CSS명칭이아니라9개SVG의실제font바이트모두SHA `8664669bd2d6a4fe8b7538706d2bb2a1004c074aa02dca59051e515e731294f5`로기대한HCRBatang과동일함을검증했다(`initial-edit-axes-genuine-hcr-face-proof-v91.json`). 이것은기존NativePNG재사용이아닌새Nativeexport/raster이며같은font조건의WASM과나머지batch는진행중이다. 이후반드시새PNG를판독하고이전대체face진단과구분한다.
- v91 편집9개WASM도terminal exit0/399.087초/source불변·각1쪽100%/gatepassed이며9개Native/WASM raster가모두byte동일하다. 실제HCR 바이트고정뒤의최신9개review를모두직접열어순서/셀포함/표뒤문단/탭/배경하단을판독했다. [초기편집증적](../assets/semanticist21-20261008/initial-edit-axes-v86-retry2/README.md)의PNG27개를새출력으로갱신했고현재Native는재사용이아닌새raster로표시한다. 이전대체face진단과구분하고GlyphAA/회색괘선차이는남는다.
- 같은실제face의3정상대조첫Native job는6442 SVG109–159MB에서Chrome의실제navigation30초제한으로측정전exit1/91.817초다. 관찰timeout이나코드결함으로분류하지않는다. 실패로그를보존하고지원된 `RHWP_VISUAL_RASTER_TIMEOUT_MS=180000`만늘려새retry1태그로재실행했다. 기존입력/Print/font/source/metric공차는변경하지않았다. 재Native전14쪽은terminal exit0/310.051초/source불변,6442/6030/3308최저96.92594/99.15282/97.34742%·미달0·누락0·3대3/2대2/9대9쪽일치다. 최저/마지막6개review를새로합성해모두직접판독했고[정상대조증적](../assets/semanticist21-20261008/normal-countercases-v90/README.md)의새NativePNG를갱신했다. 동일조건WASM/Native무늬후속은실행중이다. 제목굵기는properface공급뒤개선했으나일부굵기/괘선/옅은격자/미세간격차이는남는다.

- v91 정상대조 WASM 전14쪽도 terminal exit0/402.187초/source불변으로 끝났다. 6442/6030/3308 최저96.92594/99.15282/97.34742%, 쪽수3대3/2대2/9대9·미달0·누락0이며 Native14개 raster와 모두 SHA동일하다. 이미 직접 판독한 같은 raster의 대표6개 review는 두 경로의 관측을 공유하며 이전 대체font 진단과 구분한다(`genuine-faces-normal-native-wasm-audit-v91.json`).
- 실제 HCRBatang을 우선 공급한 Native 무늬6종도 terminal exit0/39.365초/source불변·1대1쪽100%/gatepassed다. 새 SVG에 원본 HCRBatang TTF SHA8664669bd2d6…가 실제 embed돼있음을 확인했고 새 review를 직접 표시·판독했다. 색/방향/셀 배열은 유지하며 제목 자간·AA·선 두께/위상 차이는 남는다. [무늬 증적](../assets/semanticist21-20261008/patterns-six-v90/README.md)의 Native PNG2개를 새 출력으로 갱신했고 실제 Canvas2D/CanvasKit의 기존 같은 face/package 캡처는 재사용으로 구분한다. NativeSkia 직접 PNG 지원 또는 전체 통합 gate 완료로 확대하지 않는다.
- 이 단계의 #7700 부적합 검사2개 이관·형식/IR 계약 보존·31PASS/필수lint와 실제font 시각 재검증 결과를 커밋한 뒤 사용자 지시대로 upstream/devel 동기화와 rebase를 진행한다. education/chemical/나머지 초기18개 및 후속 PR 검토, 최종 전체 gate, 통합 PR/CI/merge/후속처리는 계속 남는다.


## 75. v92 — 보존 계약 체크포인트와 교육과정 검사 적격성

- 진행74의 이관·형식/IR 계약 및 실제font 시각 증거를 일반 commit `12d96686cdd24017b44becead2c4dfeb46867bf3`에 고정했다. Cargo/시각 작업이 종료된 상태에서 fetch → 로컬 devel ff-only → review rebase를 완료했다. base/local devel `21ed64071f81f35f37618717693bd30f727392c9`는 전진하지 않았고 review source도 그대로다. 작업트리는 동기화 직후 clean이었다.
- 최신 조회는 semanticist21 OPEN30/비draft23개이며 초기18개 중 별도 merge된7612와 후속6개를 구분한다. #7615 최신4a6c27…의 후속의존/추가수정은 초기pin과 다르므로 후속 검토에 포함한다. 원PR sourceCI를 통합head CI로 대신하지 않는다.
- 교육과정 같은 원본 SHA623b00d56beffc45d27c5bf23911bdc49d3a541ded8aecbb323d0716a2bc9f4e의 base등록/현재입력 바이트 동일성을 확인했다. 독립 현재Print는415쪽이다. 기존6761의 `partial_retreat_inside_a_page_does_not_break`는 총413만 고정하고 실제 문단 되감김의 소속을 검사하지 않는다. 7226의 기존 `the_fix_does_not_add_a_page`를415로 바꾼 현재 `page_count_matches_independent_print`도 전쪽90% 전 복원한 검사이며 actual414대415 실패를 보존한다. 기대값을414로 바꾸거나 틀린413으로 되돌려 통과시키지 않는다.
- 이전 고정base Native6쪽의 미달 근거가 있으나 일부font의 대체 문제가 새로 확인됐으므로 같은 실제font 우선조건으로 고정base binary와 현재production binary의247/248/259–262쪽을 순차 재검증한다. 아직 진행 중인 결과를 이관완료/통과로 세지 않는다. 다른문서 MARKET 함수와 정확한 셀 포함·소유 관계 및 원본 저장line metric 계약은 유지한다. 원본HWP/기준PDF5개와 두파일 base/현재본문의 바이트동일보존 사본을 먼저 준비했으며 활성source/원장은 아직 이동하지 않았다.

- 시각 대조 중 추가 검증으로 #7638 머리말/꼬리말20마당×현재/HWP/HWPX의 실제 공개WASM layout/SVG60개, #7659 이름기반주석style/문단모양 및 이름없는대조7개×3저장단계21개, #7648 독립번호/첫삭제/다른종류보존/시작번호3의5상태×3저장단계15개를 실행해 **96PASS/0FAIL**, Node terminal exit0이다. Source12d966/packageab5를 확인했다(`wasm-hf-and-note-contracts-v92.json`). XML 검사 기대값은 기존source계약에서 읽고 제품의계산값을복사하지 않았다. 독립Print시각/Studio사용자여정 검증으로 확대하지 않는다.
- 첫진단의HF24개FAIL은 저장 후 자동번호자리표rawtext 공백을 `trim()`으로제외한 진단선택오류였다. 실제SVG는번호1을그렸고공개layout도bold/underline을유지했다. 원본실패JSON을보존하고 HF 가상문단소속의run과실제로보이는SVGglyph를함께검사하도록 진단을수정해96개를모두재실행했다. Production/test기대값을바꿔실패를숨기지않았다.

- 현재 HF20의첫시각job는파일명문맥이맞지않는예전번호파일Print를사용한자체입력선택오류였다. 실제Native SVG의보고서.hwp와Print0193.hwpx차이를확인하고그내소유process14개만SIGTERM으로종료했다(toolterminal143). 이일부95–100%를완료근거로사용하지않는다. 별도로기존올바른보고서.hwpx문맥20개를현재공개API로재생성해각바이트SHA가기존동일입력Print쌍과모두같음을확인했다(`captured-current-public-hf-context-v92.json`). 원본/PDF/실패자료/코드/다른작업process를보존하며같은filename문맥Native→WASM새batch로정정했다. 기존v13/v18시각결과는역사검증이며현재실행의대체가아니다.
- 같은실제font의upstream21ed교육과정Native247/248/259–262쪽은terminal exit1/555.895초/source불변·6/6측정으로종료했다. 각각51.88058/71.40332/64.23382/66.64074/15.22185/17.20583%,전체413대415쪽이다. 실제embeddedface는HYGulim/HCRBatang regular·bold이며전체font의Print동일성은주장하지않는다. 261쪽review를직접표시해Native다문화교육도식과Print앞표의내용소속이서로다름을확인했다. 단순글꼴문제가아닌기존쪽소속차이를확인했으며현재같은조건6쪽의종료·직접대조뒤검사별이관을판정한다.

- #7614의기존본문논리캐럿7상태(각주앞/뒤/끝5·인라인수식뒤1·다문단붙이기1)를현재공개WASM API로추가실행했다. 반환caret7개와현재/HWP/HWPX본문의컨트롤순서21개를독립적인기존기대문자열로확인해 **28PASS/0FAIL**, Node terminal exit0이다(`wasm-html-logical-caret-contract-v92.json`). 셀붙이기·문단끝자동번호·독립Print시각까지확대하지않는다. 첫XML진단은run의직접equation자식을누락해3개를오판했다. 실제HWPX의equation존재를확인하고독립XMLreader를정정해전체28개를재실행했으며처음실패자료도보존했다. 제품코드/정식회귀기대값은바꾸지않았다.

- Native 공개편집source계약10module집중검사가terminal exit0/594.423초/source불변으로종료했다. build후실제검사0.454초·49PASS/0FAIL/10333skip이다. field/control슬롯·논리캐럿·문단끝자동번호·HF/새주석style/독립번호·표뒤문단을포함하며전체nextest/CI를대신하지않는다.
- 교육과정현재Native도terminal exit1/634.113초/source불변·선택6쪽모두측정했다. base/current각413/414대Print415쪽이며현재247/248/259–262는73.30430/71.45219/72.98027/64.80141/68.90873/6.42706%다. base261/current261·262와base262 review를직접읽었다. 도식은base261→current262로한쪽늦게이동했고Body subtree의전체내용/기하가동일하다. 같은쪽점수의감소만으로새도식손상/무회귀를판정하지않는다. Print262의기존앞표꼬리밴드를현재도식이대신하는소속차이는남고전체무회귀는미검증이다.
- [#7445 추가등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-6091651108)에정확한base/head/input/Print·Native범위·두검사판정·유지계약·복원조건을게시하고API로UTF-8본문일치를확인했다. 관련실행이모두종료된뒤부적합쪽수2함수와원본1/PDF5의활성경로및전용원장각1행을이관했다. 원문/base·현재검사와대표review4개는[이슈자산](../assets/issue7445/education-curriculum-map/README.md)에바이트동일하게보존했다. 정확한MARKET/셀포함/내용소유/이어받기/종료/저장line/2287/2291공유계약은유지하고새ignore/제외/조용한return은없다. IR16626/216행은변경하지않고필수lint/관련검증을순차실행한다. 이관을제품해결이나전체최종gate로보고하지않는다.

- 파일명문맥을맞춘현재HF20개Native sweep은terminal exit0/858.564초/production불변·20개각1대1쪽100%·미달0·누락0으로완료했다. 머리말/꼬리말4/6/10의대표review6개와10번두영역의4배잉크경계확대를직접판독했다. 파일명·번호·탭밑줄의큰끝점이동은관측하지않고glyphedge/굵기/AA·작은선위상차이는남는다. [동일입력20개·Print·대표PNG](../assets/semanticist21-20261008/hf-template-filename-context-v92/README.md)에보존하며같은조건WASM은진행중이다. source/privateNative계약49PASS·공개WASM96PASS와독립Print측정을서로구분한다. StudioCanvas/NativeSkia/초기18개전체승인으로확대하지않는다.

- 이관 후 fmt/manifest-base/unit-tier-base/기본Clippy/WASM-libClippy/workspacebuild/workspace-alltargetsClippy가 실제terminal exit0이다. 최초prepare의변경은의도된ignored generated suite/manifest갱신이며production/지침파일변경은없다. 관련nextest62개가실행중이며최종summary전에는PASS로세지않는다(`education-relocation-validation-v92.json`). 현재HF Native20개의같은입력/Print/source를보존한WASM동일조건실행도진행중이다.

- 이관 후관련nextest의최종결과는terminal exit100/59PASS/3FAIL/10318skip(실제237.649초)이다. 보존한HWP5/IR/교육과정정확한관계검사는통과했고text-overlap partition2/6/10에서다른원본3건이실패했다. table_giant_cell_overfill.hwpx16→26, rowbreak-problem-pages.hwpbaseline없음18, issue3837/stored_vpos_rewind_form.hwp3→4다. 통합수용gate는보류이며baseline을완화/추가하지않았다.
- 고정base/current CLI의실제anomaly를확인하니giant48/48쪽·16→26, rowbreak18→19쪽·0→18,3837은4/4쪽·3→4다. 특히rowbreak의issue7445보존본과active원본이바이트동일하게중복돼있지만현재수정으로생긴새회귀는기존이관으로면제하지않는다. 세원본의독립Print/base/current/원인분기를묶어검토해구현/검사오류를구분한다. 3837의예전PDF한개는cairo이며기존보고서의한컴표기를새Print근거로사용하지않는다. 다른기존HancomPDF와현재원본대응/출력방법을확인하고새actualPrint job도시작했다.

- 같은파일명문맥HF20개WASM도terminal exit0/958.237초/production불변·각1대1쪽100%·미달0·누락0으로종료했다. Native/WASM20개raster가모두byte동일하므로직접판독한대표6개/확대2개의관측을공유한다(`hf-context-20-current-native-wasm-audit-v92.json`). #7638의현재해당범위증거를갱신하되text-overlap의실제3FAIL과전체통합gate보류를면제하지않는다.


### 진행 중 보정 76: 실패한 세 원본의 고정 base 대조와 검사 의미 재검토

- 현재 code HEAD는 `12d96686cdd24017b44becead2c4dfeb46867bf3`, 고정 upstream base는 `21ed64071f81f35f37618717693bd30f727392c9`다. production Rust는 v86 retry2와 동일하며 이번 단계에서는 아직 구현·baseline을 바꾸지 않았다. GitHub 재조회는 open30/non-draft23, v92 대비 head 변경0으로 초기 잔여17+후속6을 유지한다(`open-pr-snapshot-v93.json`). 최종 PR/CI/merge/후속 처리는 계속 미완료다.
- 이미 보존된 binary를 순차 실행해 원인 구간을 확인했다. #3837은 v44까지3건/v53부터4건이며 가운데 정렬 말미 공백 보정과 연결된다. rowbreak HWP는 v58까지18쪽/겹침0, v59 retry2부터19쪽/겹침18이다. giant HWPX는 v81까지48쪽/16건, v82 retry1부터48쪽/26건이다. binary·원본별 원시 JSON과 범위 기록은 `existing-binary-overlap-regression-boundaries-v92.json`, `earlier-binary-overlap-boundaries-v93.json`, `narrow-overlap-source-boundaries-v93.json`, `rowbreak-narrow-source-boundaries-v93.json`에 있다. 일반 binary의 건수를 실제 글꼴/Print 일치 판정으로 대신하지 않는다.
- rowbreak의 같은 원본/기존 독립 한컴2024 Print에서 base Native 7–9쪽은92.52618/98.14008/99.28141%, 현재는46.24879/50.59834/32.86536%이고 전체18→19쪽이다. base/current8쪽 review를 직접 읽어 현재 제26조 재시작과 제27조 겹침/뒤 문단 이월의 실제 회귀를 확인했다. 이 입력은 제거·이관으로 면제하지 않고 구현을 수정한다. Native 선택3쪽만 실행했고 전18쪽/fresh WASM은 아직 미검증이다.
- 해당 원본 pi21의 shared trace는 측정행3=366.013333px/내용 컷 행3=477.32px를 구분한다. 첫 source frame813.08px에서 scanner는 content budget362.3px에 `[3,16]`/363.8px 컷을 얻지만 최종 end_row3·end_cut없음으로 거부하고 다음 조각은 start_cut없이 행3 전체에서 시작한다. 기존 base는 같은 `[3,16]`을 전달해 행3 잔여에서 시작한다. 후속 보정은 source frame의 실제 수용 컷/물리 점유·전달과 온전한 nested 행의 측정/paint 소유를 함께 검토한다. 예산·threshold를 늘려 같은 컷을 강제하지 않는다(`rowbreak-{base,current}-split-trace-v93.log`, `rowbreak-table21-source-v93.dump.txt`).
- giant HWPX의 같은 원본/기존 독립 한컴2024 Print에서 base Native41/42쪽은0.26231/29.00198%, 현재는25.46988/30.58735%다. 두 쪽은 양쪽 모두 원래 미달이며 현재42쪽 review를 직접 읽어 source 표의 행/쪽 소속 차이를 확인했다. count16→26만으로 새 가시 결함이라고 확정하지 않는다. 원시 진단은 다른 쪽 viewport 밖 y1938/2978px의 런과 셀 경계 밖 글줄도 세므로 실제 clip과 독립 Print를 대조한다. 현재 기하 변화가 실제 새 결함이면 구현을 수정하며, base의 기존 미달만으로 면제하지 않는다. 전48쪽과 WASM은 미검증이며 새 제외 조건/이관/기준 완화는 하지 않았다.
- #3837의 새 독립 Hancom2020 Print는 actual job `0d9f5d55-71af-4d8a-bd45-6981c812dbbc`, runtime11.0.0.9136/PrintMethod0/1-up/전4쪽/input preprocess없음이다. 원본 SHA `b1609c633619fdc5203c33ab542eca97b00211717d618f55ee2869106c895652`, PDF SHA `535c46fa7174c2efe0772ba147f67c85fd8a9c6ca1c1bafcd72c852328d7fa78`/205849bytes를 검증했다. cairo PDF를 기준으로 사용하지 않았다. 고정 base Native 전4쪽은99.56102/98.43065/99.68445/98.29366%로 최저98.29366%/누락0/4대4쪽이며 실행exit0/445.265초/source불변이다. 현재 Native와 fresh WASM 전4쪽은 진행 중이다.
- #3837 Print1쪽에는 사진 설명의 `2매`와 글앞 날인 `㊞`가 실제 겹친다. 독립 PDF word bbox의 첫 `2매` x506.605440pt=675.47392px에 대해 Native base 숫자x672.313333/current674.82px이고, 날인 원점은 이동하지 않았다. 현재 정렬이 기준에 가까워진 것을 겹침 count 증가로 오인할 가능성이 확인됐다. 기본 count 원장의 "모든 글자 교차=확정 결함" 가정을 수정하려면 현재 Native/fresh WASM 전체90%와 확대 판독, source overlay/사진 설명의 소유·내용 관계를 먼저 확인한다. 관련없는 본문 충돌은 계속 검출하며 count 허용치를3→4로 늘리지 않는다.
- 대표 PNG는 [분석 자산](../assets/semanticist21-20261008/overlap-regression-analysis-v93/)에 보존했다. Native TSV/실행 명령/원시 tree/trace는 ignored `output/pr-review/semanticist21-20261008/`에 있다. 테스트 감소·skip·지침 변경은 이번 단계에서 없고 교육과정 이관의59PASS/3FAIL 결과도 아직 최종 수용 gate로 바꾸지 않았다.


### 보정 77: 실제 clip·사진 날인과 중첩 source frame의 점유 계약

- 진행76의 현재 Native/fresh WASM #3837 전4쪽 검증은 각각 exit0/431.419초와 exit0/592.896초로 종료했다. 두 경로 모두99.56102/98.43065/99.68445/98.29366%, 최저98.29366%/누락0/4대4쪽이다. 4개 raster가 경로별 바이트 동일하다(`issue3837-current-native-wasm-raster-audit-v93.json`). 전4쪽 review와 사진 셀 두 곳의 base/current/Print 확대를 직접 판독했다. 사진 설명의 2매와 원본의 글앞 날인이 Print에도 겹친다. 현재 정렬은 Print에 가까우며 원형 선 종류·활자 획/색 차이는 남는다. 이관·허용치3→4 증가 없이 원본 Group의 InFrontOfText·같은 문단 소유·사진 셀 포함·설명 런 관계로 원본 교차만 구분하는 검사 보정을 준비했다. 다른 본문 교차는 계속 검출한다. 새 실제 Print PDF와 입력 SHA·job은 references.tsv에 등록하고 PDF를 보존했다.
- giant HWPX의 실제 공개 WASM print layer에서 기존 원시 교차의 해당 TextOp와 부모 Body/TableCell/TextBox clip을 추적했다. 영향41/42/43/45쪽의 원시26건 모두 clip 교집합 이후 가시 교차0이다(`giant-overlap-actual-paint-clip-audit-v93.json`). 이를 전48쪽90%/제품 결함 해소로 승격하지 않는다. 독립 Print와 다른 기존 행·쪽 소속은 그대로 미달이며 active 원본/기준값은 유지한다. 진단의 후보 생산을 실제 paint clip 교집합으로 수정했고 overflow/offcanvas/flow 판정은 유지한다. 새 합성 검사도 렌더링 golden이 아닌 공개 진단 기하 계약이며 잘린 교차 비검출·부분 가시 교차 검출·TextBox clip·일반 Group 비clip을 검증한다. 저장된 수정 전 release-test rlib SHA64c9a3575cb1246146ec3077dc97e639649b3a5fcb1066ade05e39e412d3823d에 같은 공개 반례를 직접 링크해 compile exit0, 실행exit101/잘린 교차1≠독립 기대0을 재현했다(`diagnostic-clip-before-cached-library-v94.json`).
- rowbreak pi21의 독립 Print7쪽 SVG 긴 괘선으로 첫 열린 행 조각365.520833px를 확인했다(`rowbreak-print-page7-grid-v93.json`). 기존 논리 컷363.8+부모 top/bottom padding3.76=367.6은 실제 source 잔여366.013333보다 크다. 중첩 저장 프레임의 열린 컷은 부모 종료 bottom padding1.88을 소유하지 않으며, 첫 조각의 점유363.8+top1.88=365.68은 Print 관측에 대응한다. padding의 소유를 공통 trailing trim 결과로 전달하도록 보정했다. 유닛 컷/예산·공차는 늘리지 않으며 terminal 전체 종료·임의 용량 컷·편집/reflow는 비적용이다. `mixed_nested_fragments`의 stored reset 생산→CellUnit 소유→컷 scanner의 요구 높이→fragment 예약/paint가 같은 trailing trim을 소비한다. 실제 실행과 전18쪽 Native/fresh WASM은 아직 미검증이다.
- source/test 보정 전체를 준비한 뒤 고정 target에서 경계·진단·원장 검사를 묶어 빌드/실행한다(`overlap-correction-boundary-tests-v94-command.json`). 아직 최종 summary가 없으므로 PASS나 표 회귀 해결로 보고하지 않는다. 지침/프로젝트 MEMORY는 수정하지 않았다. 최종 lint·통합 PR·CI·merge·후속 처리는 계속 남는다.

- 같은 공개 진단 반례의 수정 후 실행도 확인했다. 현재 release-test rlib SHA`96d61d58900d1af5da260526445db4cb8d9f4dc8c10b260d6d8d46f7c799df16`, compile exit0/실행exit0이며 잘린 교차/부분 가시 교차/TextBox/일반 Group의 네 독립 기하 기대가 모두 통과했다(`diagnostic-clip-after-current-library-v94.json`). 정식 nextest는 여전히 실행 파일을 빌드 중이며 이 진단의 통과를 전체 gate로 대체하지 않는다.

- 수정한 production library의 공개 API로 실패 원본3건을 다시 실행해 giant48쪽/진단0건, rowbreak18쪽/진단0건, #3837 전4쪽/원시4건·원본 날인 구분 후0건을 확인했다(`overlap-originals-public-api-v94.jsonl`). 이는 기본font/API 진단이며 새 Print 시각 통과로 대체하지 않는다. 같은 current CLI artifact SHA`c6b4492b91d7b5ed6b8bb5df627538df0aa65539c7cf5bfcf5fea3a4d52a3299`도 원본 rowbreak18쪽을 반환했고 같은 실제font Native7–9쪽 sweep을 시작했다.

- 정식 경계·진단·원장 nextest는 terminal exit100/745.035초(빌드8분53초, 검사204.699초), 71개 중70PASS/1FAIL/10310skip이다. 원래 실패 partition2/6/10과 6803/6935/7470/6123/5880/6756/7620/6981/7226 관계 경계는 PASS다. 유일 실패는 shortcut의 이미 검증된 장식쪽번호 endpoint를 full bbox exact equality로 찾는 기존 검사다. 진단이 paper clip 이후 부분 bbox를 반환해 해당 동일 런83건을 본문 충돌로 오인했다. source ownership·master layer·독립 Print의 장식 계약은 유지하며 현재 Native/fresh WASM 전7쪽 재검증 뒤 clip 내부 포함 관계로 수정한다. baseline83 추가/제외/skip은 하지 않았다. production library를 다시 고칠 사유로 판정하지 않으며 현재 WASM 빌드를 시작했다.

- 현재 Native 같은 실제font의 rowbreak 영향7/8/9쪽은93.01408/98.18550/99.63160%, 전체18대18쪽이며 terminal exit0/237.071초/source불변이다. 새 세쪽 review를 직접 읽고 사용자에게 표시했다. 제26조의 마지막 중첩 문장이8쪽에서 이어지고 제27조와 겹치지 않으며9쪽 시행령 표의 시작 순서가 유지된다. 활자 획·회색/점선의 작은 차이는 남는다. `TABLE_SPLIT_RESULT pi21 sec0`은 첫컷end_row4/consumed812.8/avail813.1/fits=true, 다음조각cursor_row3/start_cut[3,16]이며 allowance0이다. 예산이나컷 관용을늘리지않고실제컷과점유를함께소비한다. 전18쪽Native/freshWASM은아직실행중/대기다.
- 기존 정식 `issue_7470`의 두 함수를 동일 test executable에서 CLI만 수정 전/후 artifact로 지정해 직접 실행했다. original_page_count_and_cut_labels_keep_their_owners는 before19≠Print18로exit101/afterexit0, nested_terminal_line_is_owned_once_inside_the_continuation_cell는 before문장소유0≠1로exit101/afterexit0이다(`rowbreak-existing-contract-before-after-v94.json`). 환경/빌드실패가아닌두기존검사의의미대로재현됐으며새fixture/검사/px기대값을추가하지않았다. before는실제로보정전v86retry2CLI, after는현재defaultCLI다.
- shortcut 현재Native 전7쪽은99.95490/99.33520/97.27506/96.30135/99.95499/99.63341/99.94124%,최저96.30135%/누락0/7대7쪽/exit0/44.185초/source불변이다. 4쪽review를직접읽고사용자에게표시해실제Print에도장식4가본문뒤로교차하는것을확인했다. 원래source audit의유일한바탕쪽런·페이지번호표시·master뒤쪽layer를그대로실행해83건모두이동일장식런의clip부분상자에속하며본문끼리의교차0임을추가확인했다(`shortcut-clip-ownership-probe-v94.json`). 정식검사의bbox비교보정은현재freshWASM전7쪽확인뒤진행하며baseline/제외목록은유지한다.
- 대표 현재 PNG는 [보정77 Native 자산](../assets/semanticist21-20261008/overlap-correction-v94/README.md)에 source/binary/PDF/관측과함께보존했다. rowbreak전18쪽Native와같은productsource의optimizedWASMbuild를실행중이며완료결과전에는통과·커밋완료·rebase완료로세지않는다. AGENTS/CLAUDE/프로젝트MEMORY의HEAD/current SHA256도동일함을다시확인했다(`instructions-unchanged-v94.json`).

- 같은 product source의 fresh WASM build를 최적화까지 완료했다. terminal exit0/1073.667초/source불변이며 JS SHA`4628f097cddafb073362cc2ff03d19744de3e037aa62d311b4bd4386528c7a20`, WASM SHA`a2cb686c42193dbb9572f7609b11a4875c5b5eaa2df91dfec15aa59e5a620747`다. rootpkg/Studiopublic/immutable`wasm-pkg-open-nested-frame-v94`가 각각동일하다(`fresh-wasm-package-provenance-v94.json`). Wrapper가Studio public에동기화했음을확인했으나이단계의브라우저GUI사용자여정을실행했다고하지않는다.
- shortcut freshWASM도전7쪽최저96.30135%/누락0/7대7쪽/terminal exit0/49.804초/source불변이며Native와7개PNG모두바이트동일하다(`shortcut-current-native-wasm-raster-identity-v94.json`). 양경로의전쪽90%와이미직접읽은4쪽Print/장식뒤쪽layer를확인한뒤유일한master런의full bbox exact equality를paperclip후상자포함관계로교정했다. 소유·번호·레이어검사및다른본문충돌검출은유지한다. 기준값증가/샘플제외/skip은없다. 실패partition3만관련재검증하며다른70PASS와Rustsource불변을구분한다. 첫필수NativeClippy도exit0/96.930초이며다른lint단계는미완료다.


### 보정 77 검증 갱신: 현재 source의 전쪽 범위와 검사 교정

- Native rowbreak 전18쪽은 terminal exit0/1816.170초/source 불변이다. 최저12쪽92.75031%, 전18쪽90% 이상/누락0/18대18쪽이며 `native-current-rowbreak-full18-v94/rowbreak/`에 TSV를 보존했다. 현재 fresh WASM 영향7–9쪽도 terminal exit0/646.448초, 93.01408/98.18550/99.63160%/18대18쪽이며 Native와 세 PNG가 바이트 동일하다(`rowbreak-current-native-wasm-boundary-raster-v94.json`). fresh WASM의 이미 내보낸 전18 SVG 중 나머지15쪽 raster·전쪽 TSV는 계속 검증 중이다. 첫 raster 명령은 작업자가 `RHWP_VISUAL_RASTER_TIMEOUT_MS=180000`을 누락해 Chrome 기본30초 navigation timeout으로 실패했다. 해당 작업만 종료하고 동일 immutable SVG/정식 helper/96dpi에서 설정을 교정한 retry1을 실행했다. 환경 실패를 제품 회귀나 시각 통과로 세지 않는다.
- #3837 현재 fresh WASM 전4쪽도 terminal exit0/614.362초/source 불변, 최저98.29366%/누락0/4대4쪽이다. 현재 Native 전4 SVG가 v93 Native와 SHA256 동일하고, 새 WASM 전4 PNG가 기존 Native 전4 PNG와 바이트 동일함을 각각 `issue3837-native-current-svg-identity-v94.json` 및 `issue3837-current-qualified-native-wasm-raster-identity-v94.json`에서 확인했다. Native PNG는 이 동일성 증거로 명시적으로 재사용했고 fresh WASM PNG는 새로 산출했다. 전4쪽 및 사진 날인 확대 판독의 관측은 진행77과 같다. 원본 사진 셀/글앞 Group 소유·날인 관계를 분류한 뒤 불필요한 baseline3 행을 제거해 의도하지 않은 교차 기대0을 강제했다. 교정 후 partition10은 terminal exit0/21.295초, 1PASS/10380skip이다. 허용치 증가가 아니다.
- shortcut의 clip 후 endpoint 포함 관계 교정 후 partition3도 terminal exit0/39.493초, 1PASS/10380skip이다. 앞서 경계 묶음70PASS와 교정한1PASS를 합쳐 71개가 각각 통과했으며, 최종 한 번의71개 재실행으로 표현하지 않는다. #3837 baseline0 강화는 위 partition10에서 별도로 확인했다. Native Clippy exit0/96.930초, WASM-lib Clippy exit0/88.289초, workspace build exit0/184.962초, workspace-all-targets Clippy exit0/129.710초이며 모두 product source 불변이다. 전체 Rust suite/Native Skia/Studio GUI/초기18개 최종 시각 gate/통합 PR/CI/merge/후속 처리는 미완료다.

### 진행 78: 후속 6개는 현재 단계에서 코드만 검토

사용자가 후속 6개의 체리픽을 금지하고 현재 단계에서만 검토하도록 범위를 정했다. 초기18개 통합 범위는 유지한다. 이번 검토는 `gh pr view`/`gh pr diff`로 정확한 원 PR head의 변경 코드·검사·본문·CI를 읽고 현재 소비 경로와 대조한 결과다. source head를 로컬 브랜치에 적용하거나 별도 PR 실행 검증을 하지 않았다. 원 PR의 PASS 수치는 작성자 보고이며 현재 통합 후보의 검증으로 승격하지 않는다. reviewer assign/approve/comment/push/close/merge 등 원격 변경도 이번 범위에서 수행하지 않았다.

| PR / 정확한 검토 head | 코드에서 확인한 계약·의존성 | 현재 판정과 필요한 근거 |
| --- | --- | --- |
| [#7661](https://github.com/edwardkim/rhwp/pull/7661) `f1da6189670357b4a2394c73d7106137ab4b38c4` | `fill_json_values`가 gradient/image를 구분하고 `create_border_fill_from_json`은 원 `borderFillId`의 채우기를 보존한다. 정의 재사용의 Fill 전체 비교는 초기 #7655에 의존한다. 실물3개 셀/2개 쪽 표본 검사도 원본·HWP·HWPX의 Fill 모델 보존을 비교한다. | **머지 보류/필수 증거 미검증.** 실제 채우기 출력의 같은 편집본 Print/Native/fresh WASM 전쪽90%가 없다. 현재 Studio 셀·쪽 대화상자는 여전히 확인 시 `none`을 생성하므로 UI 문제 해결로 확대할 수 없다. cellzone이 조회/저장 대상을 달리하는 잔여7셀은 원문에도 별도 문제로 남아 있다. |
| [#7662](https://github.com/edwardkim/rhwp/pull/7662) `377bcb71e7404a741105a28303a3fae8b671fc9d` | `Paragraph::split_at`에서 moves를 한 번 생성해 UTF-16 원점과 control 이관에 공유한다. gap slot과 자동번호 자리표를 구분하고 field end/title mark hidden slot을 제외한다. 검사6개는 개체 앞/뒤 분할·합치기 복원·복사·번호·title mark 소유를 비교한다. 초기 #7615의 `reserve_split_gap_slots`와 같은 원인 계층을 변경하며 최신 #7615는 이 PR을 합친 뒤 그 보정을 제거했다. | **머지 보류/필수 증거 미검증.** 합성 XML/편집 API 결과를 한컴 편집 저장본/Print로 확인하지 않았고 Native/fresh WASM 전쪽 증거가 없다. 각주 뒤 번호의 HWP 왕복은 이 검사에서 의도적으로 미검증이며 #7671 범위와 연결한다. 최신 #7615 전체를 이번 통합 완료로 close할 수 없다. |
| [#7663](https://github.com/edwardkim/rhwp/pull/7663) `9afbee93ec46dd6911dc11cad5ee6b524cc9a553` | `estimate_text_width_exact`도 `tab_ext_is_placeholder`를 걸러 `compute_char_positions_walk`와 같은 TabDef fallback으로 내려간다. fallback 분기에서도 tab ordinal을 소비하며 실제 저장 폭이 있는 탭은 재사용한다. 초기 #7652의 HF 탭 저장 출력에 영향이 있다. | **머지 보류/필수 증거 미검증.** 새 검사3개가 전후 x/width의 절대0.01px 동일성을 요구한다. 독립 Print가 없는 HWP3 영향2쪽과 편집 저장본 Native/fresh WASM 전쪽90%·관계형 검사 근거가 필요하다. 작성자가 남긴 오른쪽/가운데 정렬 측정·배치 원점 불일치를 이 변경의 완전한 공통 결과로 판정하지 않는다. |
| [#7670](https://github.com/edwardkim/rhwp/pull/7670) `4967286840897ed2ab611f8240d93c2131f1493a` | `tab-units.ts`의1pt=200을 탭 추가/현재·삭제 목록/기본 간격 표시/구역 읽기·쓰기에 공유한다. 현재 `style_resolver.rs`의 TabItem.position÷2 소비와 일치한다. columnSpacing은 기존 일반 HWPUNIT 변환을 유지한다. 초기 #7658 스타일 대화상자에도 적용된다. | **머지 보류/필수 증거 미검증.** 환산 단위와 source regex 검사는 실제 Studio 입력·읽기·적용·저장·재열기의 증거를 대신하지 않는다. 작성자 임시 엔진 측정과 frontend CI만 있으며 동일 편집본 Print/fresh WASM/브라우저 소비 경로는 미검증이다. |
| [#7671](https://github.com/edwardkim/rhwp/pull/7671) `67764a121708efc6f71ed58f28993bae5718df44` | HWP serializer의 gap 후 두 번째 번호 판정에 공백 자리표를 허용하되 다음 offset 간격의8배수 조건으로 실제1유닛 공백을 구분한다. 기존 첫 판정/FFFC는 유지한다. 검사2개는 각주·미주·그림 뒤/구역 앞 번호의 text+control 소유를 HWP/HWPX에서 비교한다. #7662가 범위 밖으로 남긴 HWP 번호 저장을 다룬다. | **머지 보류/필수 증거 미검증.** 수동 autoNum XML 입력과 실제 정상 한컴 저장본의8유닛 출처를 대조하고 #3495 실제 공백 대조군·같은 번호 편집본 Print/Native/fresh WASM의 영향을 확인해야 한다. 작성자의 corpus 무변화 보고를 독립 출력의 정확성으로 승격하지 않는다. |
| [#7673](https://github.com/edwardkim/rhwp/pull/7673) `aeb6399f42b9cb8029d3e3cf640a5a66c5447998` | 쪽 break 후 새 PageLayoutInfo의 단 x/width·방향·separator와 기존 zone을 비교해 동일 단 수/종류라도 설치한다. x순 정렬로 mirrored 단 순서만 다른 경우를 제외한다. 검사3개는 RIGHT 방향·구분선 포함 관계·단 너비 비를 확인한다. | **머지 보류/필수 증거 미검증.** 입력은 공개 API 저장본의 수동 colPr XML이며 정상 한컴 생성본/Print 전쪽 Native/fresh WASM90%가 없다. `force_new_page` 후 `install_zone_layout`이 그리는 배치와 생성 시 page metadata의 `getPageInfo.columns`가 달라지는 잔여 소비 경로를 작성자도 명시했다. 이를 전체 출력 소비 계약 충족으로 판정하지 않는다. |

- 6개 모두 baseRefName=devel/non-draft/open이다. 조회 시 Build & Test는 SUCCESS이며 실패/취소/timeout conclusion은 없다. Rust5개 lint는 SUCCESS이고 Studio #7670의 해당 Rust lint는 SKIPPED/frontend package는 SUCCESS다. **6개 모두 WASM Build는 SKIPPED**이므로 fresh WASM 검증으로 사용하지 않는다. 정확한 CI·본문·diff 원문은 ignored `pr{번호}-read-only-current-v94.json`/`.diff`에 보존한다.
- 현재 브랜치 HEAD는 여전히 `12d96686cdd24017b44becead2c4dfeb46867bf3`다. 원 6개 head commit object는 로컬에 없고 base 이후 로그에 해당 SHA의 cherry-pick -x 기록도 없다. 이번 조사에서는 fetch/cherry-pick/rebase를 실행하지 않았다. 초기18개의 기존 보정 및 승인된 검증은 계속하되 후속 변경의 실행 통과/수용으로 혼동하지 않는다.


- 전쪽 Native의 최저12쪽 및 앞 경계11쪽/마지막18쪽 review·overlay를 새로 생성해 직접 판독하고 사용자에게 표시했다. 11쪽 목차 표→12쪽 첫 표 조각/뒤 본문과18쪽 마지막 두 절의 소속·내용 순서·끝 문장 보존을 확인했다. 11쪽 표 머리의 파랑/회색 채우기와 가는 괘선·회색 띠/작은 기호·활자 획 차이는 남는다. PNG는 [보정77 자산](../assets/semanticist21-20261008/overlap-correction-v94/README.md)에 보존하며 Native 전18쪽 TSV 통과를 완전 동일이나 전체초기18개 승인으로 확대하지 않는다.

- 최종 fmt check/고정 base unit-tier check는 exit0이다. 마지막 manifest check에서 generated harness drift를 검출해 `--prepare`로 파생28개만 다시 준비한 뒤 같은 base 비교를 재실행해 exit0/1459sources/6265attrs/48targets를 확인했다(`logs/overlap-correction-manifest-final-v94.log`). 파생 suite/manifest는 커밋 대상에서 제외한다. AGENTS.md/CLAUDE.md/프로젝트 MEMORY는 HEAD/current SHA256이 모두 동일하며 HEAD도12d96686 그대로다.


### 진행 79: 현재 Native Skia·Studio와 편집 표본의 최종 source 연결

- 현재 Native Skia build는 terminal exit0/318.525초/source 불변이다. binary SHA`72ab96b23d36a7ea954c7120f716f8112fdac0a67b9f332f26caab8b78f785b7`를 별도 immutable 사본에 고정했다(`native-skia-current-artifact-v94.json`). 공식 lib/picture-placeholder/direct-PDF 3종은 같은 공용 target에서 순차 실행 중이며 최종 결과 전에는 PASS로 세지 않는다.
- 현재 default Native CLI로 초기 편집 원본9개와 정상대조6442/6030/3308의 전14쪽을 동일 Print·font·2022 조건에서 다시 export했다. 12개 입력/전23 SVG가 v91의 적격 출력과 모두 SHA256 동일이며 입력/Print SHA도 manifest와 일치한다(`current-native-edit-counter-svg-identity-v94.json`, terminal exit0). Native PNG의 명시적 재사용 근거이며 새로운 Native raster 실행으로 표현하지 않는다. 같은 현재 frozen WASM 패키지의 편집9개 canonical 전쪽 export/raster는 진행 중이다.
- 새 Studio PageRenderer actual Canvas2D 전3쪽은 terminal exit0/44.806초/source 불변, JS/WASM의 실제 HTTP 응답과 현재 rootpkg/frozen package가 일치하고 pageerror0/DOM 이미지 미완료0이다. 독립2020 Print 일치율91.99433/95.07682/96.33170%, 전3대3쪽이며2쪽 review를 직접 읽고 사용자에게 표시했다. 이어받기 표의 열린 하단/첫 표 닫힘/본문 소속을 확인했고 작은 활자획·회색 괘선 차이는 남는다. 새 캡처는 이전v88 세 PNG와 바이트 동일하지만 실제 현재 런타임에서 새로 산출한 것이다(`studio-issue6935-current-raster-identity-v94.json`). 대표 review/overlay는 보정77 자산에 보존한다. 최초 브라우저 명령은 helper 기본 외부CDP 주소 EHOSTUNREACH로 exit1/3.828초였고, 허용된 `--mode=headless`의 로컬 Chrome으로 교정한 retry1만 현재 실행 통과다. foreign Chrome/서버를 변경하지 않았다.
- 현재 공개 WASM에서 #7566 wrapper/nonwrapper/활성화·복원/80행 분할 소유4계약을 다시 실행해4PASS/terminal exit0이다. 실제 Studio mouse로 중첩 FIRSTGUIDE를 클릭하는 경로도 terminal exit0이며 GUIDE2→1/SECONDGUIDE 보존/actual WASM 응답일치/pageerror0이다. `wasm-clickhere-wrapper-current-v94.json`과 `studio-wrapper-field-pointer-v94/evidence.json`에 새 package hash를 연결한다. 인쇄 프로필 GUIDE 비출력과 편집 활성화 검사를 구분한다.
- #7614 공개 HTML 논리캐럿7상태/양저장형식의 기존 독립 기대28개를 현재 WASM에서28PASS로 재확인했다. #7638/#7659/#7648의 기존96개도96PASS다. 새 렌더링 회귀나 baseline은 추가하지 않았다. 모델/직렬화 관측으로 독립 Print를 대신하지 않고, 같은 현재 API로 만든 HTML7개와 번호5개의 실제 HWPX 저장본에 대해 한컴2020 Print 비동기12job을 시작한다(`additional-edit-print-jobs-v94.json`). job 시작·Print 성공·시각 검증 완료를 각각 구분한다.
- rowbreak WASM 나머지쪽 raster 중11쪽만180초 navigation timeout으로 실패했고 다른 처리 중인 쪽은 계속 진행한다. 동일 입력/source의 나머지 결과를 폐기하거나 재출력하지 않으며 실패 쪽만 후속 보완한다. 전쪽 TSV/최저90%와 실제 terminal 결과 전에는 전18쪽 fresh WASM 통과로 보고하지 않는다. 현재 보정·증거는 아직 미커밋이며 commit/fetch/devel ff/rebase를 완료했다고 하지 않는다.

- 추가 Print12입력 중9job이 실제 queue에 수용됐고10번째 시작은 서버의 queued 최대8개 제한으로 명시 거절됐다. 잘못된 입력/변환 실패로 분류하지 않으며9개의 기존 job ID를 status로 관찰·다운로드한 뒤 미수용3개만 제출한다. 시작 스크립트 exit1과 거절 로그를 보존하며12job 모두 시작됐다고 보고하지 않는다.


- Native Skia 공식3종은 모두 terminal exit0/source 불변으로 완료했다. lib525.724초(root3868PASS/12기존ignore + workspace182PASS), picture-placeholder305.703초/2PASS, direct-PDF17.417초/4PASS다. lib의 기존 ignore12개를 새 skip이나 통과로 세지 않는다. rowbreak 영향7–9쪽의 Native Skia 직접 print PNG도 각각90.46831/97.79471/99.17244%다. 새7/8쪽 review를 직접 판독해 제26조 꼬리→8쪽/제27조 본문 소속·표 외곽·중첩 문장 순서·겹침 없음과 작은활자획·회색/점선차이를 확인했다. SVG raster이 아닌 실제 Native Skia 산출물이고 대표3쪽은 보정77자산에 보존했다. 첫 compare는 CLI 단쪽 출력의 번호 없는 파일명을 사용해 review pairing이 성립하지 않았으므로 원 PNG를 바이트 그대로 페이지번호 alias로 복사해 `independent-print-review-numbered`를 새 생성했다. 잘못된 pairing을 직접 판독으로 보고하지 않는다.
- 현재 fresh WASM 초기편집9개 canonical 실행은 terminal exit0/464.146초/source 불변·각1대1쪽/gatepassed다. 기존 적격 Native9개와 PNG가 모두바이트동일하며 current Native9개 SVG 동일성도별도확인했다. Native명시재사용/fresh WASM 새 raster의보조전쪽TSV18개도모두exit0/최저100%다(`initial-edit-current-native-wasm-raster-proof-v94.json`, `current-initial-edit-png-pair-tsv-v94-exit.json`). 이전직접판독과동일한9개raster의관측을공유하며새로직접열었다고하지않는다.
- 현재 Native Skia 원본9개 dump-pages는 모두terminal exit0, 산재HWP/HWPX52/52·1133 HWP/HWPX3/3·6935 3·6803 5·보존6024 103·보존교육414·chemical103쪽으로이전v86retry2와동일하다(`current-original-page-counts-v94.json`). 쪽수동일성을문서전체90%나잔여결함해결로승격하지않고이관원본진단과활성정상대조를구분한다.
- 추가 실제 Print는9job의성공/다운로드후미수용3개만새로접수해12개전부succeeded·다운로드SHA검증을완료했다. 번호입력4개는실제PDF2쪽이며전체12입력총16쪽이다. 첫보존helper의일괄1쪽가정이assert로기각됐고실제pdfinfo/job의쪽수를기준으로보존·references를교정했다. 원본·독립PDF의쪽수를맞추려고수정하지않았다. [추가편집입력과독립Print](../assets/semanticist21-20261008/additional-edit-inputs-v94/README.md)에12개현재API저장본을SHA동일하게보존했다. 이12개Native/freshWASM대조는커밋/rebase뒤다음검증단계이며현재시각승인이나새fixture/baseline이아니다.
- rowbreak fresh WASM 전18개의실제PNG는Native전18개와모두바이트동일하다(`rowbreak-current-native-wasm-full18-raster-v94.json`). 첫나머지15쪽raster batch는11쪽180초navigation실패로terminal exit1이며다른14쪽은exit0이다. 11쪽만같은SVG·helper·96dpi에서허용된timeout360초로재실행해terminal exit0/236.462초/source불변을확인했다. 실패를지우거나batch전체exit0으로바꾸지않는다. 전18쪽embedded font검사/최종TSV는별도live작업에서진행중이다.

- rowbreak 전18쪽 fresh WASM의 embedded font검사/TSV가 terminal exit0으로 완료했다. font cmap검사135개/18쪽passed, 최저12쪽92.75031%/전18쪽90%이상/누락0/18대18쪽이다. 실제PNG18개가현재Native와바이트동일하므로이미직접판독한7/8/9/11/12/18쪽의관측을공유한다. raw font검사는cmap만의근거이며실제face/내용소속을대신하지않는다. 보정77~79와교육과정이관·추가Print준비를현재일반commit으로고정한뒤fetch/devel동기화/rebase를진행한다. 전체Rust/추가편집12개16쪽/최종통합CI와merge는계속남는다.


### 진행 80: 보정 체크포인트 커밋과 동기화

- 보정77~79와 교육과정 검사/원본 이관·필수 lint/현재 Native·fresh WASM·Skia·Studio 증거·추가 Print 준비를 일반 commit `81ff2fe4a362b8585fe24f824282e6db7332c0bb`에 고정했다. ignored output/generated suite와 지침은 stage하지 않았고 필요한 PDF/현재 API 입력/대표 PNG는 보존했다. Stage diff 검사에서 새 helper의 EOF 빈 줄을 검출했으나 commit은 이미 진행됐으므로 기능 변경 없이 EOF만 교정하고 이 기록과 함께 별도 일반 commit으로 고정한다. amend나 기존 증적의 source 변경은 하지 않는다. 이후 fetch/devel ff/rebase 결과를 기록한다.

- 새 include! helper는 cargo fmt --all이 직접 순회하지 않는 파일이어서 별도 rustfmt --check에서 줄 구성 차이를 검출했다. 해당 파일에 rustfmt를 적용하고 직접 check/전체 diff check의 exit0을 확인했다. 기능·기대값·product source는 바뀌지 않았으며 별도 형식 교정 commit으로 보존한다. 이후 PR/push 전에 고정 base lint/policy를 갱신한다.

- fetch → 로컬 devel ff-only → review rebase는 모두 exit0이다. 현재 head `9b37446556273d2dae1d1a089645687defb3215b`, base/local devel `21ed64071f81f35f37618717693bd30f727392c9`를 확인했다. upstream은 전진하지 않았고 v94 Native/WASM build의 product source hash가 현재 commit과 모두 동일하다. 동기화 직후 worktree는 clean이었다. 추가 Print12입력/16쪽의 Native → fresh WASM 전쪽 비교를 같은 frozen runtime에서 시작했다. 후속6PR은 체리픽하지 않았다(`checkpoint-commit-sync-rebase-v94.json`).

- 추가12입력의현재Native dump-pages는독립Print와모두쪽수가같다(7개HTML1쪽,번호4개2쪽/1개1쪽;총16쪽). 쪽수만으로승인하지않는다(`additional-edited-axes-page-counts-v95.json`). 현재Native 전쪽TSV는완료된HTML7개와첫번호1개(2쪽)가모두100%이며나머지입력과fresh WASM은진행중이다. 고정현재head의전체default Rust nextest도한번에실행중이며최종summary전에는전체PASS로세지않는다(`whole-default-rust-after-rebase-v95-exit.json`).

- 후속6개는 현재 단계의 코드 검토만 유지한다. 현재 `9b37446556273d2dae1d1a089645687defb3215b`와 base `21ed64071f81f35f37618717693bd30f727392c9`에서 각 원 PR의 모든 commit을 다시 대조해 source ancestry 및 cherry-pick trailer가 없음을 확인했다(`deferred-six-no-cherry-pick-audit-v95.json`). 진행78의 검토 head·의존성·미검증 판정은 그대로이며 승인·통합으로 확대하지 않는다.
- 추가12개/전16쪽 비교는 Native와 fresh WASM 모두 terminal exit1이며 product source는 불변이다. 15쪽은100%, `additional-edited-note-number-after-delete-second-footnote-v94`의2쪽은 양쪽85.90604%다. 직접 review에서 번호1)/2)는 맞지만 두 번째 미주의 세로 간격이 Native15.6267px, 독립 Print 약11.9869px로 다름을 확인했다. 번호 재계산 결함으로 분류하지 않는다. 삭제 없는 공개 API 입력의 빈 미주/짧은 본문/복수 문단/명시적 미주 간격500HU 네 대조군을 생성하고 각각 실제 Hancom2020 Print 작업을 시작했다(`endnote-boundary-counters-v95-retry1.json`, `endnote-boundary-print-jobs-v95.json`). 새 회귀 fixture·기대값으로 추가하지 않았으며 출력 간격의 독립 근거를 먼저 확보한다.
- 전체 default Rust는 계속 실행 중이다. 중간 로그에서 overflow-cell partition6개와 #5966/#6718의143대142쪽, #4882 왕복216대215쪽, #3820 Q8 소속, #6035 두 검사 등12실패를 확인했다(`whole-default-rust-running-failures-v95.json`). 최종 실패 수나 전체 완료로 표현하지 않는다. 실패 원본과 독립 Print를 확보해 고정 base/현재 Native·fresh WASM을 대조할 대상이며 baseline 완화·쪽수 기대값 갱신·검사 제거는 아직 하지 않았다. #6718 실패 원본은 시험지가 아니라 #5966 가맹사업 보고서다.

### 진행 81: 전체 회귀의 원본 대조와 열린 저장 프레임 분석

- 전체 default nextest는 terminal exit100, wrapper1586.375초/실제 test summary1028.605초이며10336개 중10299PASS/37FAIL/기존45skip이다. product source는불변이고 모든37개의 원문은 `whole-default-rust-terminal-failures-v95.json`에 보존했다. 중간12실패 목록을 최종결과로 사용하지 않는다. 최종통합 gate는보류다.
- 원본8개를 고정 base/현재 immutable CLI의 layout-anomaly로 대조했다(`default-rust-failure-originals-base-current-v95.json`). #5966은143→142쪽이며 독립 후보 Print도142쪽이다. #4882 원본은 양쪽215쪽이고 실패는 현재 export→reparse216쪽이다. #5877은32→31쪽(Print31), #1891은74→70쪽(Print70)이지만 셀넘침0→2/10→110을 쪽수 개선으로 면제하지 않는다. giant는48대48이나76→230이다. 행정편람HWP381→381/HWPX383→381이며 Print384쪽이다. #6035는48대48이며 source clipping7건도같다. 후보 PDF의 출처/정상출력 재확인과 영향 PNG를 별도로 대조하며 파일명의 연도를 Creator나 실제 Print 출처로 추정하지 않는다. #6035의 기존 `local2020.pdf`는cairoProducer/11쪽이므로 독립 Print로 사용하지 않는다.
- 원본 가정통신문 한국어/중국어·#3308·#7207의 base Native 전34쪽도 terminal exit1/362.813초/source불변이다. 한국어2쪽의최저96.99695%, #3308 전9쪽96.34221%, #7207 전21쪽93.23809%이며 모든쪽90%이상이다. 중국어2쪽은80.27051/79.55697%로base부터미달이다(`default-rust-short-base-visual-minima-v95.json`). 중국어 미달을 한국어의새3쪽회귀와혼동하지 않는다. 현재 한국어는3쪽이며첫조각이내용컷52를받지못하고row0/1만남긴다.
- 한국어실제row2의단위51→52는원본줄`(vpos63879,lh800,ls480)`→다음문단`(vpos0,lh1500,ls900)`의저장프레임reset이다(`newsletter-current-cell-units-v95.log`). 현재scanner는내용868.8+부모padding22.7이첫프레임잔여886.3을넘어실제컷을거절하고1007.8px빈밴드를예약한다. 독립PDF1쪽외곽상·하단58.0156/1064.7552px은실제첫프레임을증언한다(`newsletter-independent-print-grid-v95.json`). 첫 후보는 중첩 컷의 bottom padding 제외를 직접 저장줄 컷에도 확장했으나, #5966 원본을142→145쪽으로 악화시켜 기각하고 해당 확장을 전부 되돌렸다(source-bottom-padding-candidate-rejected-v95.json). 현재 후보는 기존 padding을 유지하면서 다음 프레임에 그림·표가 있다는 이유로 앞 문단 종료 줄간격 trim을 막던 조건만 제거한다. 종료 줄의 내용·실제 저장 reset·기존 적용 경로의 조건은 유지하며 같은 trim을 scanner의 내용 높이·예약·paint가 소비한다. 예산·공차를 늘리지 않는다. Native빌드후기존실물소유검사와전쪽Native/fresh WASM을확인하기전에는회귀해결로판정하지않는다.
- 시험지이관으로유니코드정상코퍼스는7개가남았는데기존최소8개가드를그대로둔것은이관정정누락이다. 모든목록항목을성공적으로읽도록`checked==CLEAN_CORPUS.len()`와빈목록거부로교정했다. 기존오탐0검사는그대로이며포맷/파싱실패를조용히면제하지않는다. 해당정식검사재실행은terminal exit0/24.127초/1PASS/10380skip이며product source불변이다(`unicode-clean-corpus-after-exam-transfer-v95-exit.json`). 렌더링baseline/기대쪽수를완화하지않았다.
- 새미주대조4개의실제Print를다운로드하고Native/fresh WASM 전4쪽을완료했다(각148.949/127.338초/exit0/source불변). 점수100/100/99.63964/94.91525%와별개로복수문단/명시적500HU미주간격의확대review에서차이를직접확인하고사용자에게표시했다. 같은미주안문단은약15.6629px,다음미주는약11.9869px,500HU설정은약18.6996px이라는독립Print관측을확보했다. 현재renderer의본뒤미주구분선floor→HeightCursor vpos덮어쓰기와별도미주쪽전진경로가다르므로숫자재계산만고치지않는다. 4개API입력/실제PDF는[진단자산](../assets/semanticist21-20261008/endnote-boundary-v95/README.md)에보존하며새회귀fixture/완료증거로추가하지않았다.

- 후속6개를 읽기 전용으로 다시 조회해 모두 OPEN/non-draft/devel이며 검토한 head SHA와 동일함을 확인했다(`deferred-six-current-stage-refresh-v96.json`). 초기18개 누적 범위에 추가하지 않고 진행78의 필수 근거 미검증 판정을 유지한다. 원격 상태 변경은 수행하지 않았다.
- 좁힌 저장 reset 종료 줄간격 후보의 Native build는 terminal exit0/252.155초/source 불변이다(`native-source-reset-trailing-gap-v95-exit.json`). 바이너리를 별도 immutable 사본에 고정한 실제 원본 진단에서 한국어 가정통신문2쪽/rowbreak18쪽/#5966 142쪽이며 앞 두 입력의 셀넘침0이다. #5966의 셀넘침31개는 남아 있으며 쪽수 유지로 면제하지 않는다(`source-reset-trailing-gap-page-counts-v95.json`). 기존 소유 검사와 원본 전쪽 Native 비교는 진행 중이며 fresh WASM/최종 수용으로 확대하지 않는다.

### 진행 82: 종료 컷의 padding과 물리 프레임 소비 대조

- 좁힌 reset 후보의 기존검사35개는 실제 nextest summary34PASS/1FAIL(기존 skip10346)이며 가정통신문·6935·6803·유니코드 목록 검사는 통과했다. 남은 실패는7207의 원본 첫 프레임/쪽 소유 검사다. nextest 종료 로그는 남았지만 wrapper가143으로 끝나 terminal 메타데이터를 쓰지 못했으므로 wrapper exit100이나 그 terminal source불변을 주장하지 않는다. 다음 source 수정 전에 hash 비교가 true임을 별도 확인했다(`source-reset-trailing-gap-focused-observed-terminal-v95.json`).
- 같은 후보의 Native 원본34쪽은 terminal exit1/429.978초/source불변이다. 한국어 최저93.41485%, outline3308 전9쪽97.34742%, 중국어79.55141%, eogu7207 전21쪽46.89563%(2~4쪽미달)다. 중국어는base부터미달이나 eogu는base 최저93.23809%에서 악화한 실제 회귀다(`source-reset-trailing-gap-short-native-minima-v95.json`). 원본/기대값/baseline은 변경하지 않는다. 한국어 전2쪽과 eogu2쪽의 canonical review를 새 현재 raster에서 합성해 직접 판독하고 사용자에게 표시했다. 한국어1쪽 괘선은 약21px 높게 끝나며 eogu2쪽 다음 행 원점도 어긋나므로 점수 통과만으로 한국어 보정을 완료라고 하지 않는다.
- 한국어 trace는 source첫프레임1007.7733px/예약1007.8px이나 paint985.1px다. `stored_row_frame`의 `consumed-split_end_limit`가 마지막 행의 padding을 앞 행 공간에 더해 버린 것이 원인이다. `row_cut_content_height`가 생산한 내용+padding 요구 높이를 앞 행 차감·예약 증가분·end_row_height_override·accepted row box에 함께 전달하도록 보정했다. source 최소 상자·내용 유닛·padding의 소유를 유지하고 공차나 예산을 늘리지 않는다. 현재 Native 재빌드 중이며 아직 시각/정식 검사 통과로 판정하지 않는다.
- eogu의 첫 컷은 base/head 모두 [1,1,1,1,9]이지만, 다음 조각의 첫 행은 base204.1px/head125.9px다. source 첫 프레임이 내용 높이에 이미 반영되어 예약과 같아지면 `first_fragment_blank_band`의 엄격한 증가 조건이 물리 높이 전달을 생략한다. 그 결과 첫 조각의 괘선이12.8px 접히고 다음 조각이 원본 행의 빈 잔여를 잃는다. 같은 원본/source frame 소비 경로를 확인 중이며7207 검사를 완화하지 않는다(`logs/eogu-base-fragment-trace-v95.log`/`logs/eogu-source-reset-gap-fragment-trace-v95.log`).
- padding 보정만 적용한 Native build는 terminal exit0/263.035초/source 불변이다. 현재 immutable CLI에서 한국어2쪽의 첫 표 상자는1008.5px로 복원됐고 #5966은142쪽을 유지한다. eogu21쪽의 첫 표992.9px/이어받기 차이는 그대로여서 별도 원인을 구분한다(`row-cut-padding-artifact-and-geometry-v96.json`). PNG90%나 eogu 해결로 확대하지 않는다.
- 저장 첫 프레임은 내용과 예약 높이가 같아도 그 물리 높이를 paint/다음 조각에 전달해야 한다. 독립적인 저장 경계·host·원본 frame 수용 조건을 유지하고, 그 프레임이 이미 컷 높이에 포함된 경우에도 첫 행 override를 전달하도록 엄격한 증가 조건을 교정했다. 같은 source 프레임의 빈 잔여를 다시 사용하며 유닛 컷/예산/공차를 바꾸지 않는다. padding 보정과 합쳐 Native build·기존소유검사·전쪽 Native/fresh WASM을 진행한다.
- 중국어 기준은 실제 Hwp2024/한컴PDF2쪽이다. 앞 혼합 batch의 일괄 compat2022와 구분해 고정 base 바이너리/같은 입력·Print·font를 compat2024로 재실행했다. terminal exit1/111.189초/source불변이며80.27051/79.55697%로 동일하다(`native-base-newsletter-zh-compat2024-v96-exit.json`). 원본의 기존 미달을 다시 확인했으며 current/fresh WASM도 해당 버전으로 비교한다. 올바른 공급 가능성 확인 없이 글꼴 예외나 검사 이관으로 처리하지 않는다.
- 첫 프레임 보정의 Native build는 terminal exit0/255.685초/source 불변이다. 해당 artifact에서 한국어2쪽의 첫 상자1008.5px, eogu21쪽의 첫 상자1005.7px가 복원됐고 #5966은142쪽을 유지했다. 다만 아직 수용하지 않는다(`row-cut-physical-frame-before-control-counter-v96.json`).
- 기존 private 단위검사 `test_saved_reset_trailing_trim_requires_plain_text_owners`의실행은101/1FAIL이다. 앞 줄간격을 제거하는 후보가 개체-only 문단의 로컬reset에도8px trim을 적용한 반례다. 기존 기대0을유지하고, 개체 문단에서는 `page_frame_reset_before`가 증언하는 실제저장쪽프레임일때만허용하도록소유조건을교정했다. 한국어 실제 unit52는 stored/page reset 둘다 true이며, 로컬reset 반례는 page reset false다. 테스트를 수정하거나 새 시각fixture를추가하지 않았다(`local-control-reset-counter-before-v96.json`).
- 같은 immutable CLI를기존7207 harness에명시적으로전달한3개검사는2PASS/1FAIL이다. 원래첫프레임/표제소속 검사는PASS가됐으나17쪽 지도첫조각의가운데정렬공간이사라진반례가남았다. parser/test source는그대로이며이실행을현재 direct lib 검증으로확대하지 않는다(`eogu-physical-frame-formal-cli-override-v96.json`). `stored_row_frame`에서도물리높이==이미선택한컷높이면override를생략하는조건이같은소유정보를잃는다. 원본최소상자판정은유지하고높이가같아도행높이·정렬소유를항상전달하도록교정했다. 국소reset 반례와지도정렬을합친Native retry1과관련검사/fresh WASM검증이진행중이다.
- retry1 Native build는terminal exit0/258.506초/source불변이나7207 지도검사는 여전히2PASS/1FAIL이다. 이를수용하지않는다(`row-cut-physical-frame-final-artifact-v96.json`). 실제17쪽 첫지도행을대조하니base797.0px/current451.5px이고표제도454.8→284.0px로위로갔다. 새 row height조회에앞행67의start_cut=[267]을뒤행68의end_cut=[4]과함께넘긴것이자기보정의오류다. start/end 컷의행소유를구분해종료행이시작행과같을때만start_cut을적용하도록바로교정했다. 기존지도검사/기대값/원본은유지한다.
- 이종류의소유반례를release-test전체링크전에확인하려고고정 `target/pr-review`의dev Native CLI를진단용으로빌드한다(`native-row-cut-owner-diagnostic-v97-command.json`). 진단통과를최종release-test·Native/fresh WASM검증으로대신하지않는다. 앞의formatter가바꾼ignored generated harness를`--prepare`로재생성했으며1459sources/6265attrs/28suites+20exceptions=48targets다. source/Cargo/지침변경은추가되지않았고파생파일은stage하지않는다.
- v97 dev Native 진단 빌드는 terminal exit0/222.208초/source 불변이다. 새 CLI를 그대로 둔 기존7207 harness에 전달한 원본 검사3개는 terminal exit0/3PASS이며 첫 프레임·10쪽 주석·17~21쪽 지도6개와 표제 순서가 모두 통과했다(`row-cut-owner-diagnostic-artifact-v97.json`, `logs/eogu-row-cut-owner-diagnostic-v97.log`). 시작 컷의 행 소유 교정 후 관측이며 정식 release-test/fresh WASM 전쪽 시각 통과로 대신하지 않는다. 관련 private 경계44개는 별도 dev nextest에서 최종 결과를 확인한다.
- 관련 private 경계 dev nextest도 terminal exit0/315.240초/source 불변이다. 최종 summary는44run/44PASS(1slow)/3961기존skip이며 `test_saved_reset_trailing_trim_requires_plain_text_owners`의 기존 기대0도 변경 없이 통과했다(`row-cut-owner-private-counters-diagnostic-v97-exit.json`). dev 실행이며 release-test 및 전쪽 Native/fresh WASM 최종 판정은 별도로 남는다.
- 사용자 범위 지시를 재확인했다. 후속 #7661/#7662/#7663/#7670/#7671/#7673은 현재 단계의 읽기 전용 코드 검토만 수행한다. 최신 GitHub 조회에서6개 모두 OPEN/non-draft/devel이고 진행78의 정확한 검토 head와 같다. 현재 HEAD `9b37446556273d2dae1d1a089645687defb3215b`의 ancestry 및 cherry-pick trailer에 이6개 head가 없음을 다시 확인했다(`deferred-six-current-stage-refresh-v97.json`). 코드 검토의 필수 증거 미검증/머지 보류 판정을 유지하고 초기18개의 보정·검증을 계속한다. 이번 재확인에서 체리픽이나 원격 상태 변경을 수행하지 않았다.

### 진행 83: 원본 전쪽 검증으로 v97 후보의 남은 회귀 확인

- 정식 release-test Native build는 terminal exit0/258.563초/source 불변이다. immutable CLI SHA256 `3ff8a9c27695f0c74e861a95300ac8e7511f4580c2573ca5f855f5dfbe241bbf`와 당시 product diff를 `row-cut-owner-qualified-native-artifact-v97.json`에 고정했다. Native 원본32쪽 batch는 terminal exit1/242.125초/source 불변이다. 한국어2쪽 최저96.91159%, outline3308 전9쪽97.34742%, eogu7207 전21쪽44.04673%이며 eogu17쪽88.05153%/20쪽44.04673%만90% 미달이다. 앞서 악화한 eogu2~4쪽은97.18599/99.23951/97.39314%로 복원됐지만 전체 원본 통과는 아니다(`row-cut-owner-current-visual-minima-v97.json`).
- 한국어 전2쪽 canonical review를 새 raster에서 합성해 직접 표시·판독했다.1쪽의 외곽선 끝이 약21px 위로 접히던 차이는 해소됐으며 본문/증상 표/손씻기 그림/2쪽 네 그림과 아래 안내 상자의 소속을 대조했다. 제목 활자 굵기와 그림·작은 획 차이는 남는다. eogu1/2/3/4/17/20/21쪽 review를 생성하고17/20쪽을 직접 표시·판독했다.17쪽 지도/표제가 약2px 내려가고20쪽은 약12.2px 내려간 실제 회귀다. 원본 소유 검사3PASS를 시각 배치 통과로 확대하지 않는다(`native-row-cut-owner-direct-review-v97/`).
- 중국어 전2쪽을 올바른 compat2024로 비교한 결과는 terminal exit1/116.600초/source 불변,79.93664/79.55141%다. 새 전2쪽 review를 직접 표시했고 제목 글꼴뿐 아니라 표·그림·본문 위치 차이도 남음을 확인했다. 고정 base의80.27051/79.55697%와 구분해 기록하며 글꼴 예외·테스트 변경을 수행하지 않는다. fresh WASM 빌드는 같은 source에서 진행 중이다.
- #6035 회귀 원본 `samples/issue6035/cgmp_evaluation_table.hwpx`(SHA256 `0f03119d6ac3e19f74fc9fcec5315ac6ed436bab5abffd0804e9cf3350dd747a`)는 고정 base에도 같은 바이트다. 실제 Hancom2020 `11.0.0.9136`/managed direct DLL/PrintMethod0/1-up/input preprocess none으로48쪽 Print를 새로 확보했다. job `7d4c4ecb-a9c6-409e-b21b-a59a0c0313ae`, PDF SHA256 `08ae67334ed2cb690dc25f5447d9c7204044a5b313e026fb20f4147de0182d4a`, 재현 기록은 `cgmp-original-actual-print-v97.json`이고 PDF는 `pdf/semanticist21-20261008/cgmp-original-v97/`에 보존한다. 독립 Print 텍스트에서 제목은35쪽/첫 하위 항목은36쪽으로, 기존 두 검사의 "같은 쪽이어야 한다" 및 "제목만 남으면 빈 쪽" 가정과 다르다. 고정 base/현재 Native 전48쪽 비교를 시작했으며 시각90%/fresh WASM/직접 판독 전에는 회귀 검사를 변경하지 않는다.
- v97 fresh WASM build는 terminal exit0/1137.800초/source 불변이며 최적화까지 완료했다. immutable package/Studio public/실제 HTTP JS·WASM SHA256이 각각 일치한다(`row-cut-owner-wasm-artifact-v97.json`). Native 지도 회귀가 있는 후보이므로 통합 수용하지 않고, 이 package로 전쪽 재검증을 했다고도 보고하지 않는다. 이후 보정 source에는 새 빌드를 적용한다.
- #6035 Native 전48쪽은 base/current 각각 terminal exit1/1075.841초·1072.698초/source 불변이다. 양쪽 모두48대48쪽/누락0이지만 최저46쪽0%이며 base 미달11쪽/current 미달13쪽이다. 기존 기준미달과 별도로35쪽96.65234→83.37607%,38쪽96.98017→49.44828%의 현재 후보 악화를 확인했다.36쪽은47.54287→87.07284%로 개선됐지만90% 미만이다.35/36쪽 canonical review를 직접 표시해 제목과 하위 항목이 분리되는 독립 Print를 확인했다. 잘못된 기존 assertion과 실제 배치 결함을 구분하며, 현재의35/38쪽 회귀를 이관으로 면제하지 않는다(`cgmp-original-base-current-native-minima-v97.json`).
- 새 v97 RowBreak 후보의 영향인지 분리하려고 이전 frozen v94 Native로 같은 전48 SVG만 내보냈다. terminal exit0/30.209초/source 불변이며 v97 전48 SVG와 모두 SHA256 동일하다(`cgmp-prior-v94-current-v97-svg-identity-v98.json`).35/38쪽 악화는 이번 v97 후보 이전부터 존재한 현재 통합 후보의 문제다. 이 동일한 진단 SVG48개는 종료·소유·해시를 확인한 뒤 직접 삭제해1.52GiB를 확보했고 압축하지 않았다. 현재/base 출력·PNG·실물 원본·새 Print·binary·로그·공유 target은 보존했다(`redundant-cgmp-svg-cleanup-v98.json`).

### 진행 84: 물리 프레임의 공통 경계와 접은 상자의 소비 보정

- 일반 종료 행의 padding 재해석/동일 높이에서 무조건 override를 추가한 자기 보정은 정상 지도17/20쪽을 악화시켜 철회했다. v98에서는 기존 일반 행 경로를 복원하고 저장 첫 프레임의 동일 높이 전달을 유지했다. 진단 CLI의 eogu 첫 상자1005.7px/17쪽 지도 원점527.6px/20쪽 지도 원점81.2px와 기존 원본3검사의PASS를 확인했지만, 한국어 첫 상자는987.0px로 짧아진 채였다. 더 좁은 후보의PASS를 전체 해결로 보고하지 않는다(`opening-frame-equality-only-artifact-and-geometry-v98.json`, `opening-frame-owner-artifact-and-geometry-v98.json`).
- 선언 최소 상자와 실제 저장 첫 프레임의 우선권/누적 예약을 대조했다. 실제 저장 프레임은 scanner가 선택한 내용 요구 높이와 비교해야 하며, 더 큰 fallback 최소 상자를 새 내용 요구 높이로 비교하지 않는다. 실제 선택한 종료 행 상자를 continuation 누적값에도 전달했다. 다만 v98 추적에서 한국어는 required1006.5333/fallback1007.7733/`saved=None`이라 우선권만으로 해결되지 않음을 확인했다(`logs/opening-frame-demand-ko-v98.log`).
- 실제 원본은 그림으로 시작하는 다음 저장 쪽이었다. 종료 줄간격 측정은 이미 `page_frame_reset_before`와 원본 줄·tag·공유 프레임을 검증했지만 첫 프레임 생산/paint 정렬은 개체가 없는 문단만 인정해 같은 저장 경계를 누락했다. 측정의 판정을 `native_multirow_saved_reset_frame_cut`의Option 결과로 공유했다. Some(0)은 유효한 경계지만 종료 간격이 없는 경우이고, 로컬reset의None과 다르다. source 첫 프레임/배치 정렬도 실제 page-frame owner와 이 결과를 함께 소비하며 일반 row-tail/sliver의 plain-text 계약은 유지한다. 원본 또는 검사 기대값을 바꾸지 않았다.
- v99 dev build는 terminal exit0/69.675초/source 불변이며 한국어의 저장 첫 프레임1007.7733px가 다시 생산·수용된다. 기존 eogu3검사는3PASS/5.35초이며 private경계44개는 terminal exit0/201.096초/source 불변/44PASS(1slow)/3961기존skip이다. 진단 Native 전32쪽은 terminal exit0/281.413초/source 불변, 한국어 전2쪽95.24687%, outline3308 전9쪽97.34742%, eogu 전21쪽93.26292%가 각각최저이며 미달/누락0이다. dev 실행이며 정식 release-test/fresh WASM 통과로 승격하지 않는다(`shared-saved-boundary-diagnostic-minima-v99.json`).
- 한국어2쪽에서는 접힌 셀 끝과 하단 선이 달랐다. `layout_table_partial`의 terminal Square 복원 분기가 trailing-band 접기 후에도 `partial_table_height`의 접기 전 높이를 다시 사용해 이미 버린 약6.67px를 복구했다. 복원을 현재 `table_node.bbox.height`에서 시작하고 실제 recovered child가 요구한 공간만 늘리도록 고쳤다. v100 dev build는 terminal exit0/54.260초/source 불변이며 전2쪽 모두 하단 선bbox y1062.5/h3.8(중심1064.4)과 셀끝1064.5가 일치한다. 원본 Print 하단 약1064.7552px와도 대조했다(`post-fold-frame-artifact-and-geometry-v100.json`). 정식 Native/fresh WASM 전쪽·정상 대조군·필수 lint·최종 전체Rust/통합CI는 계속 남는다.

- 후속6개는 다시 최신 원격 head와 현재 브랜치 ancestry/trailer를 읽기 전용으로 확인했다. #7661/#7662/#7663/#7670/#7671/#7673 모두 진행78의 검토 SHA 그대로이며 OPEN/non-draft/devel이다. 현재 HEAD `9b37446556273d2dae1d1a089645687defb3215b`에 적용되지 않았고, Build & Test SUCCESS / WASM Build SKIPPED 상태다(`deferred-six-current-stage-refresh-v100.json`). 현재 단계의 코드 검토·머지 보류 판정을 유지하며 초기18개에만 보정을 적용한다.
- v100 정식 release-test Native build는 terminal exit0/266.763초/source 불변이다. CLI SHA256 `ebcf0e43eef67aa73c9f575e249ecd0390c1a280a85d9118530f57bb3a31b613`와 product diff를 `shared-frame-qualified-native-artifact-v100.json`에 고정했다. 같은 source에서 Native 원본32쪽/중국어2쪽 비교 및 fresh WASM 빌드를 시작했다. 실행 시작을 시각 통과로 세지 않는다.
- v100 Native 원본32쪽 비교는 terminal exit0/241.900초/source 불변이다. 한국어 전2쪽 최저96.91813%, outline3308 전9쪽97.34742%, eogu7207 전21쪽93.26292%이며 쪽수 각각2/9/21로 독립 Print와 같고 미달/누락0이다(`shared-frame-current-visual-minima-v100.json`). silhouette-only gate는 직접 판독이 필요한 `not_evaluated`이며 초기18개 전체 승인으로 확대하지 않는다. 한국어1/2쪽과 outline7쪽 새 review를 직접 표시·판독했다. 한국어 외곽 하단/본문·증상 표·예방 그림·아래 안내 상자의 소속을 확인했고 제목 글꼴 굵기/그림 획·작은 행 간격 차이는 남는다. outline 직인 셀은 원본의 왼쪽 기준 오프셋에 배치되며 부모 셀과 글자 포함 관계를 유지한다. 기존 가운데 정렬 assertion은 fresh WASM 전9쪽과 독립 기대값 확인 전까지 변경하지 않는다.
- 중국어 Native 전2쪽은 terminal exit1/115.083초/source 불변,80.03919/79.55141%다. fresh WASM 빌드는 계속 진행 중이며 새 code의 WASM 시각 통과를 주장하지 않는다. AGENTS.md/CLAUDE.md/프로젝트 MEMORY는 현재 HEAD의 바이트와 같고 수정하지 않았다.
- eogu7207의 새1/2/3/4/7/17/20/21쪽 canonical review를 모두 직접 표시·판독했다(`native-shared-frame-direct-review-v100/eogu7207/review/`). 자기 보정이 악화했던17/20쪽은97.39710/96.63530%로 복원됐고 지도·표제 순서와 첫 지도 뒤 표제, 마지막21쪽 지도 소속을 확인했다. 표의 앞쪽 조각과 마지막 지도의 내용 누락·중복은 이 대표쪽에서 관찰되지 않았다. 작은 활자·그림 획, 일부 줄바꿈 및1/2/4/7쪽 표 하단 모양·위치 차이는 남아 전21쪽의 완전 동일이나 최종 시각 승인으로 보고하지 않는다. 관계형 검사 추가·기대값 변경·최종 lint·일반 commit/rebase는 아직 수행하지 않았다.

### 진행 85: fresh WASM 전쪽 대조와 테두리 소유 원인 분리

- v100 fresh WASM build는 terminal exit0/1077.072초/source 불변이다. 새 WASM SHA256 `677ffc8c528b53627f267510dd53cd8ba3d18a9ef4a20f553e2631ec695cc2a3`와 JS SHA256 `4628f097cddafb073362cc2ff03d19744de3e037aa62d311b4bd4386528c7a20`가 immutable package/rootpkg/Studio public/실제 HTTP에서 각각 일치한다(`shared-frame-wasm-artifact-v100.json`). 전32쪽 WASM 비교도 terminal exit0/255.082초/source 불변, 한국어2쪽96.91813%/outline9쪽97.34742%/eogu21쪽93.26292%가 각각 최저다. 중국어2쪽은 별도 terminal exit1/125.372초/source 불변,80.03919/79.55141%다. 전34개 새 Native/WASM PNG가 모두 바이트 동일하며 앞서 Native 직접 판독의 관측을 공유한다. W PNG를 새로 직접 열었다거나 중국어·초기18개 전체를 승인했다고 하지 않는다(`shared-frame-native-fresh-wasm-raster-parity-v100.json`).
- 정상 rowbreak 원본 전18쪽을 같은 font/profile/compat2024로 현재 frozen Native에서 다시 export했다. terminal exit0/197.184초/source 불변이며 기존 적격v94 Native 전18 SVG와 전부 SHA256 동일하다(`shared-frame-native-rowbreak-full18-svg-identity-v100.json`). 이는 Native 기존 PNG의 명시적 재사용 근거이며 새 raster/fresh WASM 전18쪽 실행으로 표현하지 않는다.
- #3308 기존 검사를 현재 immutable Native CLI에 연결해 그대로 실행했다. 실제 등록된 release-test suite11 harness를 조회해 찾았으며 terminal exit101/0.172초/source 불변, 가운데 정렬 assertion에서 좌88.6/우117.6으로 실패했다(`outline3308-existing-wrong-centering-v100-exit.json`). 원본 LEFT/COLUMN/offset 및 독립 Print를 확인한 원인과 일치한다. 처음에 독립 target와 현재 generated suite24를 추정한 두 helper는 harness/검사명 확인 단계에서 실패해 검사를 실행하지 못했다. 환경·탐색 실패를 결함 재현으로 세지 않는다.
- eogu 원본 dump에서 자리차지 RowBreak/69행·223셀을 확인했다. 현재1쪽은 행12 안에서 endCut `[1,1,1,1,9]`,4쪽은 rowspan 블록 내부에서 끝난다(`eogu-row-fragment-cuts-v100.json`). 같은 좌표에서1쪽Native 하단선은624픽셀 연속이고4쪽은650픽셀 연속이지만 독립 Print에는 같은 긴 선이 없다(`eogu-source-border-raster-diagnosis-v100.json`).2/3쪽의 실제 닫힌 행 경계는 Print에도 존재한다. 내용 존재·90% 점수로 이 차이를 면제하지 않는다.
- 실제 호출 경로는 두 셀 배치 경로의 `collect_cell_borders`가 조각으로 축소한 행 span에 원본 하단 테두리를 수집하고, `render_edge_borders`가 그것을 쪽 내부 컷에 그리는 것이다. 기존 Square의 전체 마지막 edge 제거는 자리차지 경로에 적용되지 않았다. v101 보정은 `fragment_cell_borders`에서 원본 셀 끝/물리 컷/남은 유닛을 구분해 두 호출부에 같은 테두리 소유를 전달한다. 셀의 측면과 실제 종료된 이웃 셀 경계는 유지하고 본문 위치·높이를 보정하거나 덮지 않는다. 마지막 조각과 CellBreak의 기존 경로를 구분한다. 정식 관계형 검사 추가는 보정 뒤 원본 전쪽 Native/fresh WASM·직접 판독을 먼저 확인한 뒤 진행한다.
- v100 Native Clippy는 Option 반환으로 바꾼4개의 let-else에 `question_mark`를 검출해 terminal exit101/83.375초/source 불변이다. 해당 자기 lint 오류를 동등한 `?`로 교정하고 위 테두리 소유 보정을 함께 적용했다. v101 Native Clippy는 terminal exit0/85.830초/source 불변이며 고정 공용 target의 dev CLI 진단 빌드를 시작했다. 나머지 lint·최종 원본 검증·commit/rebase는 아직 완료하지 않았다.
- v101 dev build는 terminal exit0/84.581초/source 불변, WASM-lib Clippy도 terminal exit0/84.219초/source 불변이다. dev Native 전32쪽은 terminal exit0/285.227초/source 불변이며 한국어2쪽95.70785%/outline9쪽97.34742%/eogu21쪽94.04543%가 각각최저다. eogu1쪽99.56047%/4쪽98.52237%로 개선되지만 한국어1쪽의 실제 Print에 있는 닫힌 프레임 선을 제거했다. 새 한국어1쪽 review를 직접 표시·판독해 이 반례를 확인하고 보정을 철회했다. `fragment_borders.rs`와 두 호출부의 적용을 모두 제거했으며 보존된 v101 CLI/PNG/로그는 거절 후보 증거다. RowBreak 내부 컷만으로 테두리 소유를 판단하는 가정은 맞지 않으며, 문서명·선 색상·수치 예외를 추가해 대상을 좁히지 않았다.
- #6035의 historical v94 Skia/default는 같은 head `12d96686cdd24017b44becead2c4dfeb46867bf3`의 immutable artifact다. 같은 source/Print/font/2022 조건에서 Skia35/38쪽 SVG를 다시 export해 이전 current-default35/38 SVG와 모두 SHA256 동일함을 확인했다(`cgmp-native-feature-pair-v101.json`, terminal exit0/6.239초/source 불변). v94-default와 v97-default 전48 SVG 동일성도 별도로 존재한다. 관측한35/38쪽 악화를 backend feature 차이로 면제하지 않는다. 이는 historical feature 분리 진단이며 현재v102 시각 승인으로 확대하지 않는다.
- 중국어 v100 전2쪽 review를 직접 표시·판독했다. 첫 프레임 외곽 하단과2쪽 본문·그림·안내 상자 위치 차이가 있어 글꼴 예외로 처리하지 않는다. source p39의 빈 줄은 같은 vpos63119, 서로 다른 column_start0/41362인 좌우 저장 조각이며 CellUnit은 기존 `stored_seg_is_row_fragment`로 한 물리 줄로 합친다. 그러나 저장 종료 경계 판정은 empty spacer의 raw LineSeg가 정확히1개여야 한다고 요구해 같은 줄을 거부한다. 현재 trace의 required1007.0667/fallback1007.7733/`saved=None`도 보존했다(`newsletter-zh-first-frame-demand-v101.log`). v102는 그 단일-segment 가정을 제거하고 CellUnit과 동일한 공통 판별자로 **한 물리 줄인지** 확인한다. 다줄·같은 column_start의 reset·implementation tag 조각을 한 줄로 인정하지 않는다. 원본 metadata·기대값을 바꾸지 않았고 dev CLI 진단 빌드 뒤 실제 source 프레임 수용 및 전쪽 개선을 확인한다.


### 진행 86: 시험지 회귀 연결 제거와 두 조판 파일 주석 번역

- 최신 사용자 목표는 **현재 체리픽된 초기18개만** 검토·보정·통합하고 추가 체리픽을 하지 않는 것이다. 후속6개는 진행78의 읽기 전용 코드 검토로 유지한다. 검토 완료 뒤에는 최신 지시대로 **개별 원 PR review 문서와 오늘할일**을 작성하고 통합 PR/정확한 head CI/merge/후속처리를 진행한다. 이전 통합 기록 우선 선호를 최신 개별 문서 지시보다 우선하지 않는다.
- 사용자가 마지막 전체Rust의37FAIL을 지적하고 삭제 대상 시험지 원본을 포함하는 회귀도 제거하도록 지시했다. 현재 후보는 **PR 준비 완료가 아니며 머지 보류**다. v95 전체 summary `10336 run / 10299 PASS / 37 FAIL / 45 기존 ignore`를 현재 보정의 통과로 재사용하거나 제거 건수만큼 실패를 임의 차감하지 않는다.
- v103에서 회귀 연결 제거 지시를 원본 삭제까지 확대 적용해 HWP/HWPX56개와 HML3개, 총59개를 잘못 삭제했다. 최신 사용자 지시로 **진행87에서 전부 복원**했다. 아래 v103 제거 기록은 당시 실행 이력이며 현재 원본 파일의 상태가 아니다(`removed-exam-forbidden-inputs-v103.json`, `removed-exam-consumer-changes-v103.json`).
- Gym11개 pack의 시험지 과제86개와 대응 reference86개를 제거했고 남은 과제의 정식 inventory·lookup 검사도 갱신했다. 실제 남은 extraction16개를 계속 검사하며 존재 확인이나 과제↔풀이 계약을 skip하지 않는다. Studio의 시험지 E2E3개와 눈금자 검사 중 시험지 브라우저 진입점을 제거했다. 독립 눈금자 driver 계약은 유지하고 실행했다. HML 상수3개는 이미 소비 함수가 제거된 뒤 남은 unused 참조였으며 상수도 제거했다.
- 글자모양 원본 코퍼스의 시험지 유래66개 행을 Rust 목록과 JSONL 양쪽에서 제거했다. 다른 원본의 동일-id 경계 계약은 유지한다. replay/capsule의 시험지3개 시나리오, unicode 진단1개, Oracle 도구의 관련36개 검사와 판단 봉투3개 검사를 제거하고 목록·개수·생성 경로를 함께 정리했다. 기존 Oracle 등록 목록은 시험지 관련72개 pair를 제거해1030→958로 바뀌었고 새 일반 문서를 임의 추가하지 않았다. 사용 중인 쪽수 cache에서 관련PDF63개 record를 제거하고 남은 측정값으로 집계만 재산출했다. 기존PDF의 내용·쪽수 측정값을 바꾸지 않았다.
- v103 조회에서 검색한 `tests/`, Studio E2E, Gym, replay/capsule, Oracle/판단 fixture 경로의 직접·별칭 참조는0개였으나, 이 조회는 **회귀가 읽는 mydocs JSON 두 경로를 놓쳤다**. 이를 전수 제거 완료의 증거로 사용하지 않으며 진행87에서 간접 소비 목록과 자동 수집을 보완했다(`removed-exam-active-reference-audit-v103.json`). `transferred_exam_regressions.txt`는 재유입을 막는 **제외 목록**이며 검증 입력이 아니다. source의 과거 관측 주석과 실행 입력을 구분한다. 입력이 없는 검사를 조용히 return/ignore해서 통과시킨 결과가 아니다. Visual Sweep의 기본 시험지 등록15개도 제거했으며, 빈 대상 실행은 exit2로 실패해 0건을 PASS로 세지 않는다(`removed-exam-empty-sweep-guard-v103.json`).
- 사용자 요청에 따라 `src/renderer/typeset/table/continuation/fragment/emit.rs`와 `src/renderer/layout/table_partial.rs`의 영어 설명 주석49개 블록을 한국어로 바꿨다. 경로·변수·코드블록 같은 기술 표기는 유지했다. 번역 직전과 번역/format 뒤의 **주석 제외 코드 바이트가 두 파일 모두 동일**하다(`korean-comment-code-identity-v103.json`). 이전 단계의 실제 조판 보정 diff와 이번 주석 번역을 혼동하지 않는다.

| 제거와 연결된 검사 | 실제 최종 결과 | 로그 |
| --- | --- | --- |
| Gym pack/추출/개체·조판·진단/render-tree 계약 | 134 PASS / exit0 | `logs/removed-exam-gym-unit-v103-retry1.log` |
| Oracle 공개 도구 계약 | 103 PASS / exit0 | `logs/removed-exam-oracle-unit-v103-final.log` |
| 작업 영수증·replay/capsule 계약 | 37 PASS / exit0 | `logs/removed-exam-work-receipt-unit-v103.log` |
| Visual Sweep·fidelity/agent 등록·생성 계약 | 196 PASS / exit0 | `logs/removed-exam-render-tools-unit-v103-retry1.log` |
| inspect fixture/목록 계약 | 22 PASS / exit0 | `logs/removed-exam-inspect-fixtures-unit-v103.log` |
| 남은 눈금자 driver의 실제 PNG 포함·표시·정렬 반례 | 4 PASS / exit0 | `logs/removed-exam-ruler-driver-v103.log` |

- 위 관련 Python/Node는 합계496PASS다. 확대 실행한 별도 `oracle_vs_self` 전체39개에는38PASS/1FAIL이 있다. 실패는 `test_visual_sweep_schema_version_live`가 producer schema1을 고정한 기존 검사다. 고정base `21ed640…`와 현재 producer는 모두schema2여서 시험지 제거로 발생한 실패와 구분한다. 이 도구 전체를 PASS라고 하지 않는다(`logs/removed-exam-decision-envelopes-unit-v103.log`). schema2의 실제 소비 계약은 별도 확인이 남았다.
- 파생 Rust suite를 준비했고 manifest는1459source/6265정적test/28suite+20exception=48target이다. `--check --base-ref 21ed640…`와 unit-tier4113test/297module 검사가 통과했다. 생성 파일은 ignored이며 커밋하지 않는다. 포맷과 `git diff --check`도 확인했다. v103 최종 Rust lint/Native/fresh WASM 시각 gate는 아직 완료하지 않았다.
- 시험지 제거·주석 번역 뒤 전체Rust를 **실제로 다시 시작**했다: `cargo nextest run --locked --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast`, wrapper timeout3600초. 이 실행은 진행87에서 terminal exit100/10309PASS/27FAIL로 종료됐다(`whole-default-rust-after-exam-removal-v103-command.json`, `logs/whole-default-rust-after-exam-removal-v103.log`). 입력59개는 종료 직전 복원됐으므로 동일 입력 snapshot의 최종 통과 증거가 아니다. Cargo가 종료됐음을 확인한 뒤 다음 checkpoint를 진행하며 공유 target을 초기화하지 않는다.

- 앞 단계 v102 진단은 한국어 전2쪽 최저96.91813%/outline 전9쪽97.34742%/eogu 전21쪽93.26292%이며 모두90%이상/누락0이다. 중국어 전2쪽은83.19177/81.06842%로 미달이며 새1/2쪽 review를 직접 표시·판독했다. p39의 좌우 저장 조각을 한 물리 빈 줄로 인정해 source 첫 프레임1007.7733px 수용은 복원됐지만 본문·그림·글꼴 차이는 남는다. Windows의 실제 Malgun Bold를 해시 확인해 공급한 진단도83.15734/81.05742%로 미달이며 공급만으로 해결되지 않았다. private경계44PASS, NativeClippy exit0/85.477초, WASM-libClippy exit0/76.649초, workspacebuild exit0/154.878초는 **v102 당시 source**의 결과다. 현재v103 최종 head의 필수 gate를 이 과거 결과로 대신하지 않는다.


### 진행 87: 시험지 원본 복원과 간접 회귀 소비 경로 교정

- 최신 사용자 지시는 **시험지 원본은 존재해야 하며 삭제하지 않는 것**이다. 회귀 연결 제거 요청을 원본 삭제까지 확대 적용한 것은 에이전트의 오류다. 지정한59개(HWP/HWPX56개·HML3개)를 모두 원래 경로로 복원했다. `git show 9b374465…:<path>`의 바이트가 삭제 직전 기록된SHA256과 모두 같음을 먼저 확인하고 복원했으며, 복원 뒤에도59/59일치를 확인했다(`exam-originals-restored-v104.json`). 시험지 회귀·등록을 다시 활성화하지 않는다. 복원한 원본의72개 historical Markdown 링크도 기존 로컬 목적지로 되돌렸고20개 문서에 추가한 잘못된 삭제 안내를 제거했다(`exam-original-document-links-restored-v104.json`).
- v103 전체Rust는 실제 terminal exit100/1577.601초이며 test summary는**10336 run / 10309 PASS(14slow) / 27 FAIL / 45 기존 skip**, test 시간1038.155초다. product source 불변은 확인했지만 입력59개는 마지막 tail 중 복원됐으므로 최종 head의 동일 입력 전체 통과로 주장하지 않는다. v95의37개와 실제 검사 이름을 비교하면11개가 더는 실패하지 않고,26개가 계속 실패하며, 원본을 잘못 삭제한 #4962 하나가 새로 실패했다(`whole-default-rust-terminal-failures-v103.json`, `whole-default-rust-failure-delta-v104.json`). 남은 실패를 숨기거나27개를 임의로PASS로 바꾸지 않는다.
- 누락된 간접 소비는 `mydocs/tech/investigations/issue-4961/`의WASM/Nativetrace 문서 목록·public fixture 목록과 `issue-4962/font_metric_coverage_public_fixtures.json`의 시험지HWP/HWPX 두 항목이다. 해당 항목과 시험지 포맷 parity 비교만 제거했다. #4962의 남은 일반 수출입 문서 분류/hash/개수 검사는 유지하고 검사명을 실제 범위에 맞췄다. 기존 Native에 명시된 이관 핀 계약을 WASM 소비자도 동일하게 읽게 했으며, 살아 있는 exact/missing/substFont 프로필과 나머지 핀·backend 검사는 유지했다.
- W0font-rule의 시험지 유래4개 parity 등록과 재실행 command만 제거했다. 폰트메트릭600개·401face·lookup/hash는 바꾸지 않았고 W6metadata의 해당 입력 파일 해시만 다시 고정했다. char-shape 추출기·Oracle resolver/coverage/sweep/page-smoke의 자동 수집에는 기존 제외 목록을 적용했다. mini-repo의 원본3개도 존재하며 로컬 제외 목록으로 활성 수집에서 빠진다. 일반 문서는 유지되고 다른 상대 경로의 동일 basename까지 제외하지 않는다.
- 원본이 실제 존재하는 상태에서 자동 수집을 실행해 세 Oracle 수집기는 각각 main1169개/mini7개, page-smoke도 main1169개/mini7개만 수집하고 지정 입력은8경로 모두0개임을 확인했다(`exam-original-preserved-automatic-discovery-v104.json`). 직접 활성 참조와 회귀가 소비하는 네 investigation JSON의 참조도0개이며, 제외 목록과 과거 실행 증빙은 입력 등록으로 세지 않는다(`exam-originals-preserved-regression-reference-audit-v104.json`).
- 관련 Oracle 전체는**104PASS/exit0**, font-rule/WASMtrace는**13PASS/exit0**다(`logs/restored-exam-oracle-unit-v104-retry1.log`, `logs/restored-exam-font-e2e-v104-retry1.log`). 변경된 #4961/#4962/봉투 Rust20개는**20PASS/exit0/42.755초**, 실제 test10.594초/source 불변이다(`restored-exam-indirect-consumer-rust-v104-exit.json`). 파생 suite/base 정책과 format/diff check도 통과했다. 이 focused PASS를 전체27실패 해소나최종freshWASM 조판 gate로 확대하지 않는다.
- 확대 Node35개 실행은23PASS/12FAIL이었다. W6lineage10개 실패는 고정base21ed에서도10/10FAIL로 재현되며 LatinRange8/9 수집과 연결된다(`logs/base-font-lineage-contract-v104.log`). source-digest drift 검사1개와 이관 핀을 읽지 않던 WASMtrace1개도 있었다. 마지막 것은 기존 이관 계약을 실제 소비해 위13개 재검증에서PASS가 됐으며, 나머지 도구 전체를 통과했다고 쓰지 않는다. v103의판단 도구schema1/2 실패도 별도로 남아 있다.
- CGMP #6035의 악화 구간을 재빌드 없이 immutable Native로 분리했다. v42의35쪽은61행부터 시작하지만 v44부터 이미 소진된60행 컷 `[2,1,1]`의 빈 잔여를 이월한다. 현재 잔여는1.7px이며 후속 행도 내려간다.38쪽은v53까지84행 시작 컷 `[1,1,1]`/40.5px인데v55부터`[2,1,1]`/19.7px로 바뀌어 한 줄을 앞쪽에서 더 소비한다. 현재v102도 같은소속이다(`cgmp-historical-geometry-v104.json`, `cgmp-historical-geometry-narrow-v104.json`, `cgmp-historical-cuts-v104/`, `cgmp-regressed-row-source-v104.json`). 원본84행은 같은 column_start의vpos0/0/1560 세 줄이며, 단순 원점0을 새 쪽 경계와 동일시하지 않는다. 기대값이나 baseline은 바꾸지 않았다.
- 기존v97PNG로38쪽 review를 새 합성하고 current를 대화에 표시·직접 판독했다. base96.98017%/current49.44828%이며 후보는 첫 줄을 빠뜨리고 다음 행을 약20.8px 올린다. 독립Print의마지막 `제4장 품질 보증` 소속도 후보에 없다. 이는 과거 raster의 명시적 재판독이며 새 head 시각 검증이 아니다(`cgmp-page38-direct-review-v104.json`, `cgmp-current-page38-review-v104/review/review_038.png`). 실제 악화를 보정하기 전 PR준비·승인·merge는 계속 보류한다.


## 88. v105 — 직접 HWPX 원점 경계의 생산·소비 불일치 보정

- 검증 기반은 `fd4e242c9e4fafb2441f8d743d01746e26d15078`, 고정 base는 `21ed64071f81f35f37618717693bd30f727392c9`다. 최신 fetch에서도 base는 같고 GitHub #7697/#7693/#7612는 병합 완료다. 그래프의 한 부모 연결은 미병합 상태가 아니며, 현재 검토 branch의 #7612 체리픽 사본은 원본과 다른 SHA다. 추가 체리픽과 원격 변경은 수행하지 않았다.
- 원본은 수정하지 않은 `samples/issue6035/cgmp_evaluation_table.hwpx`(SHA256 `0f03119d6ac3e19f74fc9fcec5315ac6ed436bab5abffd0804e9cf3350dd747a`)다. 기준은 이미 커밋된 독립 한컴 2020 Print PDF 전48쪽이다. 실제 PDF의37쪽은84행 첫 줄에서 끝나고38쪽은 그 문단의 뒤 두 줄로 시작하며,38쪽 마지막에는97행의 `제4장 품질 보증` 제목이 있다. 35쪽의 물 공급 제목과36쪽 하위 항목의 분리는 기준 출력에도 있으므로 기존 #6035 같은 쪽 assertion을 정답으로 취급하지 않는다.
- `cgmp-cut-trace-v105/units-instrumented.log`에서 공통 `stored_zero_origin_text_frame_break`는84행 경계를 **true**로 판정하지만 해당 `CellUnit.stored_frame_break_before`는 **false**였다. 앞단 `is_stored_frame_rewind`가 직접 HWPX의 선언 span 휴리스틱을 통과하지 못한 입력을 조기 반환하여, 이미 입증한 공통 결과까지 차단했다. 이 조기 차단 전에 공통 판별 결과를 전달하도록 순서를 보정했다. 문서 ID·행 번호·좌표·예산·공차는 구현 조건에 넣지 않았으며, 편집/합성/서로 다른 column의 비적용 조건은 기존 공통 판별자에 남는다. 임시 상세 로그 코드는 제거했고 production 변경은 이 전달 순서뿐이다.
- 소비 연결은 `stored_zero_origin.rs::stored_zero_origin_text_frame_break → table_layout.rs::is_stored_frame_rewind → CellUnit.stored_frame_break_before → advance_row_cut_inner의 strict_saved_frame_break → scan_ordinary_row_step → layout_partial_table의 같은 start/end 컷`이다. 같은 함수 이름만 확인한 것이 아니라 실제 유닛 플래그와 컷을 대조했다. 새 진단에서84행의 두 번째 유닛 플래그는true, 앞 조각 종료 컷은 `[1,1,1]`이다.38쪽84행 표시 높이는19.7→40.5px,85행 시작은186.0→206.8px이며, 마지막97행 첫 유닛도38쪽에 남는다(`cgmp-shared-zero-frame-trace-v105/trace.log`, `cgmp-shared-zero-frame-pages-v105.json`).
- 수정된 source에서 새 dev Native를 빌드하고 immutable 사본으로 고정했다(`shared-zero-frame-artifact-v105.json`, `logs/cgmp-shared-zero-frame-build-v105.log`). table_layout.rs SHA256은 `6fc34978e2e222b9afa58ca1c48d44e91af86851fa24d170a093249f89da02e0`이다. 기존 경계 검사44개는 **44PASS/exit0**(실제 test92.712초, compile 포함173.559초), format/diff check도PASS다. 새 렌더링 회귀나 golden은 아직 추가·변경하지 않았다.
- 정상 대조 Native는 한국어 전2쪽 최저96.91813%, outline 전9쪽97.34742%, eogu 전21쪽93.26292%로 모두 누락0/미달0이다(`native-shared-zero-frame-counters-v105-exit.json`, 각 silhouette.tsv). 이는 현재 보정의 Native 진단이며 과거 fresh WASM을 이번 보정의 검증으로 승격하지 않는다.
- CGMP 전쪽 실행은 **exit1**이다. SVG와 render tree는47쪽, 독립 Print는48쪽이라 도구가 전체 TSV 산출 전에 불일치를 거부했다(`native-cgmp-shared-zero-frame-v105-exit.json`). 부분 점수를 전체 통과로 바꾸지 않는다. 같은 새 SVG와 같은 PDF raster에서 표준 webfont rasterizer·compare·overlay·review로35/38/42/45쪽을 별도 판독했다(`cgmp-shared-zero-frame-direct-review-v105.json`).38쪽은 **49.44828→98.54216%**이며 첫 줄·이어받는 두 줄·후속 행과 마지막 제목의 소속이 복원됐다. 새 `cgmp-shared-zero-frame-review-v105/review/review_038.png`를 대화에 직접 표시했다.35쪽83.37607%,42쪽48.58196%,45쪽37.72150%는 여전히 미달이다. 선택 PNG는 직접 진단 자료이며 문서 gate는 `re_review_required`다.
- 표는 이제 기준과 같은42쪽에서 끝나지만41쪽의 마지막 제목/42쪽 시작 내용은 아직 다르다. 35쪽의 소진된 가운데 정렬 행60 잔여1.7px도 남는다. 뒤 서식에서 기준46쪽의 `제2호서식 삭제` 문단이 Native45쪽에 있다. 원본 XML의389문단 첫 vpos는0인데 재구성된 dump는136221HU다. `document.rs::reflow_zero_height_paragraphs`의 `orig_plain_tail/source_plain_reset`은 직전 Table control을 plain tail에서 제외하므로 원본 표 끝의 물리 점유를 소비하지 못하는 경로가 다음 분석 대상이다. 독립 관측과 소스의 연결은 확인했지만 아직 이 경로를 수정하거나 회귀 해소로 판정하지 않았다.
- 필수 최종 전체Rust·세 Clippy·workspace·release Native/fresh WASM 및 남은 원본 출력 보정은 미완료다. CGMP 기존 검사·기대값을 통과 목적으로 수정하지 않으며, 통합 PR 생성·CI·merge·후속 처리는 계속 보류한다. 시험지59개 원본은 보존하고 회귀 연결 제외를 유지한다.


## 89. v106 — 표 뒤 원본 문단의 쪽 경계 보정 계획

- 입력은 진행88과 같은 수정하지 않은 CGMP HWPX와 독립 Print48쪽이다. XML388문단은 단일 비글자취급 TopAndBottom/문단 기준 표이며 source host vpos31360HU, 세로 offset390HU, 바깥 위/아래 여백141HU를 기록한다. 셀 마지막 줄33816+높이1200+활성 안 여백283+283은 표 선언 높이35582HU와 정확히 같다. 다음389문단의 source vpos0/높이1200은 기준46쪽의 삭제 서식 안내다.
- 원본 표의 잉크 프레임 끝은67473HU, 바깥 아래 여백을 포함한 전진 끝은67614HU다. 본문 높이67918HU에는 다음1200HU 글줄을 이어 담을 수 없다. 단순 `vpos=0`이나 문서 번호로 쪽을 추가하지 않고, 원본 줄로 닫힌 표의 점유 끝을 `orig_plain_tail → source_plain_reset → 재구성 첫 줄 원점 → typeset 저장 reset → 해당 문단 소속`에 전달한다.
- 적용은 편집/합성 줄이 없는 단일 plain 저장 셀의 실제 줄 끝이 선언 프레임을 닫는 문단 기준 TopAndBottom 표다. 다른 정렬/좌표계·중첩 개체·원본이 아닌 줄·재조판된 문단·선언과 내용 프레임 불일치는 원본 표 프레임의 증거로 사용하지 않는다. 기존 plain 글줄 계약과 일반 재조판은 보존한다.
- 구현 전 public parser/DocumentCore 관측을 고정하고, 보정 뒤 같은 원본의389문단 원점과 쪽 소속·뒤 증명서 앞/뒤 면 보존을 재확인한다. CGMP 전체48쪽 Native와 관련 정상 대조를 확인하며, 90% 미만/누락 쪽과 직접 PNG를 남긴다. 해당 원본의 Native/fresh WASM 전체90% 이전에는 새 렌더링 회귀/golden이나 기존 기대값을 추가·갱신하지 않는다.

- 첫 Native 후보는389문단 원점을0으로 복원했으나49쪽/해당 문단47쪽으로 실패했다(`source-tail-pages-v106.json`). 실제 downstream `stored_adjacent_line_table_frame`은 원점을 복원한 후속 줄과 재구성된 앞 두 줄을 서로 다른 축으로 읽어1332px 부근의 표 원점을 만들었다. 인접 줄 프레임도 기존 `ladder_vpos`로 세 원본 위치를 함께 소비하도록 보정한다. 이 후보 실패를 보존하며, sidecar가 없으면 기존 좌표를 그대로 쓴다. public 진단의 최초 after 실행은 루트 rlib가 오래된 사본이라 제품 변경 검증에 사용하지 않는다. 실제 bin이 링크한 `debug/deps/librhwp.rlib`로 다시 확인한다.

- 두 소비 경로를 함께 보정한 retry1 Native dev 빌드는 exit0/53.77초다. immutable binary SHA256 `bcaff5bff98a7ae20788cddcdca37be0be36220f8ce88b098967afb3ddc6fc10`와 세 production 파일 해시는 `stored-table-tail-artifact-v106-retry1.json`에 고정했다. 실제 bin이 링크한 deps rlib의 public probe에서도389문단은 parser/Core 모두0이고 셀 줄·패딩은 변하지 않았다(`cgmp-source-tail-public-probe-v106.json`).
- 같은 원본의 출력은 **48쪽**으로 복원됐다.388표는45쪽,389 삭제서식안내는46쪽,419/420 증명서 앞/뒤는47/48쪽이다(`source-tail-pages-v106-retry1.json`). 전체 Native sweep도48쪽을 export했고 현재 전쪽 raster·TSV 실행 중이다. 쪽수 복원만으로 시각 통과라고 쓰지 않는다.
- 기존 관련 source 검사 **80run/80PASS/3925filtered skip/exit0**다. 실제 test100.889초, compile 포함207.903초이며 명령·로그·source불변 검증은 `source-table-tail-related-counters-v106-retry1-exit.json`에 남겼다. 정상 Native 대조는 한국어2쪽 최저96.91813%, outline9쪽97.34742%, eogu21쪽93.26292%로 총32쪽의 미달/누락0이다(`native-source-tail-counters-v106-retry1-exit.json`, 각 silhouette.tsv). 최종 전체Rust/lint/release Native/fresh WASM 검증과 구별한다.
- 최신 Native PNG로35/38쪽 compare·overlay·review를 산출했다(`cgmp-source-tail-direct-review-v106-retry1.json`).35쪽83.37607%는 미달이며38쪽98.54216%는 첫 줄·나머지 줄·후속 행·끝 제목을 복원한 상태를 유지한다.38쪽 review를 대화에 직접 표시하고 첫 행·표 외곽·끝 제목을 판독했다. 새 renderer 회귀/golden·기대값은 추가·변경하지 않았다.

- 전쪽 Native 실행은900초 timeout으로46쪽 raster에서 종료됐다. terminal session과 해당 종료를 확인하고, 정확한 이번 output profile의 고아 Chrome10개만 종료했다. 동일 immutable SVG의47/48쪽을 표준 webfont rasterizer로 마저 산출한 뒤48쌍 PNG의 전쪽 TSV를 완성했다. 원래 timeout을 통과로 바꾸지 않고 원문 export48/독립 PDF48/PNG48·번호누락0과 세 source 해시 불변을 따로 확인했다(`native-cgmp-source-tail-v106-retry1-timeout.json`, `cgmp-source-tail-completion-pages-v106.json`, `native-cgmp-source-tail-complete-validation-v106.json`).
- 전48쪽은40쪽90% 이상/8쪽미달이며 최저48.58196%다. 미달은28쪽61.98717%,35쪽83.37607%,36쪽87.07284%,37쪽51.53933%,39쪽50.70337%,42쪽48.58196%,44쪽83.09709%,45쪽80.35438%다(`native-cgmp-source-tail-complete-scores-v106/silhouette.tsv`, exit1).46쪽 안내와47/48쪽 증명서의 소속은 복원됐으며47/48쪽 점수는94.00342/98.58632%다. full Native gate는 `re_review_required`이며 새 PR·승인·통합은 보류한다.
- 45쪽 최신 review를 대화에 직접 표시했다. 구비서류 표의 위치는 복원되어37.72150→80.35438%지만 위 처리절차 표의 글자·상자가 빠져 있다. 원본374문단에는 첫 TAC 표를 소유한67767HU 줄 뒤에1400HU/vpos0 줄이 있고, 글뒤 표는 두 번째 줄을 소유한다. Core는 이 원점을68607HU로 바꾸며 현재 typeset은 두 표를44쪽에 함께 발행한다. 단순 좌표 clamp 대신 원본 TAC 점유 상자·소유 줄의 물리 경계와 실제 컨트롤 쪽 전이를 함께 추적한 후 보정한다(`cgmp-tac-carrier-probe-v107.log`).
- 시험지/HML 원본59개는 현재 HEAD의 바이트·Git mode와 모두 일치한다(`exam-originals-preserved-after-cgmp-v106.json`). 회귀 소비 제외를 유지하며 파일은 삭제하지 않았다. format/diff check 통과. 이번 결과는 진행89의 로컬 진단이고 필수 최종 release/fresh WASM·전체Rust·lint 묶음과 개별 최종review·오늘할일·원격 CI/merge/후속 처리의 완료 증거가 아니다.


## 90. v107 — 글자취급 표 뒤 저장 줄의 실제 쪽 소속 보정 계획

- 진행89의 보정과80개 관련 검사·32쪽 정상 대조·CGMP 전48쪽 Native 결과를 `f56ceec135f28aa6a8eb424bd74551aba0c7831f`로 커밋했다. fetch 후 upstream/devel과 로컬devel은 기존 `21ed64071f81f35f37618717693bd30f727392c9` 그대로였다. 불필요한 no-op rebase가 실행됐지만 HEAD는 변하지 않았다(`cgmp-source-tail-checkpoint-sync-v106.json`). 사용자 지시에 따라 이후에는 마지막 리베이스 기준과 fetch된 upstream/devel HEAD를 비교하고, 변경이 없으면 checkout·rebase를 생략한다.
- 수정하지 않은 CGMP 원본374문단의 첫 글자취급 표67485HU+바깥 여백141+141은 원본 첫 줄67767HU와 정확히 같다. 다음 원본 줄은 text_start8/vpos0/높이1400HU이며, 첫 줄의 잉크 끝67767HU는 본문67918HU 안에 있으나 전진 끝68607HU에 다음1400HU 줄을 이어 담을 수 없다. 첫 표는Print44쪽 신청서 앞면, 두 번째 글뒤 표는Print45쪽 처리절차 표를 소유한다. 이 쪽 구분은 원본 저장 줄·개체 소유와 독립 Print를 함께 근거로 삼는다.
- 공통 결과는 원본 첫 TAC 상자에 이어지는 물리 줄 경계와 그 앞 개체 소유자다. 문단 재구성 전의 원본에서 입증한 줄만0 원점을 보존하고, typeset의 실제 앞 개체가 현재 쪽에 있는 경우에만 그 경계를 전진시킨다. 동일 새 줄에 여러 개체가 있으면 쪽 이동을 반복하지 않는다. 편집·합성 줄·무효 저장 캐시·같은 줄 형제·첫 상자가 이미 본문을 넘는 경우는 이 계약의 근거로 쓰지 않는다.
- 소비 연결은 `원본 첫 TAC 줄/표 높이 일치 → 재구성 전 경계 스냅샷 → document.rs 내부 줄 원점 보존 → control_line_seg_index의 실제 소유 줄 → 현재 쪽의 앞 개체 존재 → prepare_table_control의 쪽 전이 → 새 쪽의 PageItem::Shape → layout_shape_item/뒤 shape pass의 해당 쪽 원점`이다. paint의 별도 앵커도 직접 확인하며 clamp·출력 숨김·px 기준값 갱신으로 처리하지 않는다.
- 작은 경계 확인은 같은 원본의44/45/46/47/48쪽 소속과45쪽 처리절차 상자·문구, 뒤 구비서류 표를 포함한다. 먼저 한 번에 두 소비 경로를 수정·빌드하고 해당 페이지 review와 기존 관련 검사를 실행한다. CGMP 전체 Native/fresh WASM90% 전에는 새 renderer 회귀/golden·기존 기대값을 추가/변경하지 않는다. 통합 gate와 남은 미달 범위는 그대로 보류한다.

- `stored_control_frame.rs::following_first_tac_frame`은 원본 첫 TAC 표의 높이·바깥 여백과 소유 줄이 일치하고, 같은 가로 프레임의 다음 원점0 줄이 현재 본문에 들어가지 않는 경우에 앞 컨트롤/다음 줄의 공통 결과를 반환한다. Core는 캐시 보정 전 그 결과를 고정해 해당 내부 경계만 보존한다. typeset은 실제 후속 표가 그 줄을 소유하고 앞 개체가 아직 현재 쪽에 있을 때만 한 번 전진한다. 이미 새 쪽으로 넘어간 같은 줄 형제는 그 앞 개체가 없어 반복 이월하지 않는다. source sidecar와 재구성 좌표를 혼용하지 않으며 문서 번호·px 예외는 없다.
- 새 dev Native 빌드는 **exit0/168.312초**이고 binary SHA256은 `1443b2801cdaafcd1c7788c66bb0da8a609ea17a62581f2619bf27fb1744b358`이다(`control-frame-artifact-v107.json`). 실제 dump는48쪽이며374문단의 신청서 표는44쪽, 처리절차 글뒤 표는45쪽이다.388 구비서류 표도45쪽,389 삭제 안내46쪽,419/420 증명서 앞뒤47/48쪽을 유지한다(`cgmp-control-frame-pages-v107.json`).
- 기존 관련 검사 **80run/80PASS/3925filtered skip/exit0**다. 실제 test89.245초, compile 포함262.716초이며 실행 뒤 source 해시 불변을 확인했다(`control-frame-related-counters-v107-exit.json`). format/diff check도PASS다. 정상 문서32쪽을 같은 print/compat2022/실제 글꼴 공급으로 새 export했으며 직전v106 SVG와 바이트가 모두 같다(`native-control-frame-svg-counters-v107.json`). 이것은 새 Native SVG 진단이며 과거 PNG를 새 raster나 fresh WASM으로 보고하지 않는다.
- CGMP44/45/46/47/48쪽의 최신 Native compare·overlay·review는 **5쪽완료/누락0/exit0**다. 원문 전체48쪽/기준PDF48쪽을 별도로 확인했고 선택 범위의 점수는94.73514/95.54654/100.0/94.00342/98.58632%다(`native-cgmp-control-frame-review-v107-exit.json`, 해당 summary/overlay_metrics.json).44쪽은83.09709→94.73514%,45쪽은80.35438→95.54654%다. 선택 범위 gate의 `passed`를 CGMP 전체48쪽이나 fresh WASM 통과로 승격하지 않는다.
- 최신44/45쪽 review를 대화에 실제 표시하고 직접 판독했다. 신청서 앞면의 처리절차 표 겹침이 없어지고 뒤면의 처리절차 표·문구와 구비서류 표가 제 쪽에 있다. 두 쪽에서 테두리 농도와 미세 위치 차이가 보이며, 이 잔여를 자동 점수로 면제하거나 문서 전체 완료로 쓰지 않는다. 마지막 전쪽v106의 미달8쪽 중44/45쪽만 이번 선택 진단에서90%를 넘었고28/35/36/37/39/42쪽은 여전히 미해소다. 새 전체 Native/fresh WASM·필수 전체Rust/lint/Skia 게이트와 최종 기록·통합PR/CI/merge/후속 처리는 미완료다.
- 초기18개 원 PR의 현재 metadata만 재조회했다(`initial18-live-metadata-v107.json`). #7612는MERGED, 나머지17개는OPEN이다. 기존 검토 head와 현재 원 head를 혼동하지 않으며 #7615 후속 의존 등 새 contributor commit을 fetch·체리픽하지 않았다. 원격 상태 변경은 수행하지 않았다.

## 91. v108 — 쪽 첫 저장 여백과 후속 문단 원점의 공통 판정

- 진행90의 코드·검증을 `3de5f70580282fc0d8698751df1500d163307e3c`로 커밋했고, upstream/devel은 `21ed64071f81f35f37618717693bd30f727392c9`로 변하지 않아 checkout/rebase를 생략했다(`cgmp-control-frame-checkpoint-sync-v107.json`). 기존 원본과 그 커밋의 immutable Native로 남은6쪽을 다시 산출했다.6쪽 완료/누락0/exit1이며28/35/36/37/39/42쪽은61.98717/83.37607/87.07284/51.53933/50.70337/48.58196%다(`native-cgmp-remaining-review-v108-exit.json`). 현재 미달을 역사적 결과로만 추정하지 않는다.
- 28쪽 원본333문단은 빈 글줄이지만 앞 간격500HU·vpos500HU·높이1400HU·줄간격1820HU를 가진다.334문단의 앞 간격은0이고 vpos3720HU는 앞 줄의 전진 끝500+1400+1820과 정확히 같다. 원본 XML과 독립 Print에서 이 첫 여백은 쪽 안의 점유이며 쪽 원점 자체가 아니다. 실제 배치 trace는333문단을 올바르게 그린 뒤334문단에서 page_base500HU를 빼6.6667px 위로 이동한다(`cgmp-remaining-vpos-trace-v108.log`).
- 기존 `stored_first_margin_is_page_relative`의 다음 앞 간격이 양수여야 한다는 가정은 간격0인 연속 저장 사다리도 배제한다. 다음 앞 간격이0이어도 원본 줄 전진 끝과 정확히 이어지는 경우 같은 공통 판정을 사용한다. 명시적 쪽나눔·합성 줄·TAC 도형 제목·overlay 앵커의 기존 제외 조건은 유지하며 문서 번호나500HU 수치로 분기하지 않는다.
- 소비 연결은 `첫 원본 vpos=앞 간격 / 다음 원본 vpos=앞 줄 끝+다음 앞 간격 → 공통 원점 판정 → typeset/section/vpos의 record_vpos_page_origin → HeightCursor의 후속 측정 → layout의 vpos_page_base_init → paint HeightCursor::vpos_adjust`다. 큰 layout 파일에서 판정 helper만 작은 공통 모듈로 옮기고 두 호출자가 같은 결과를 소비하게 한다. Native28쪽과 영향을 받는 원점 대조군·기존 관련 검사를 함께 확인한다. 다른 표 분할 미달5쪽은 별도 원인 분석 중이며 이번 변경으로 해결됐다고 쓰지 않는다.
- 전체 원본 Native/fresh WASM90% 전에는 새 렌더 회귀·fixture/golden·기존 기대값을 추가/갱신하지 않는다. 최종 전체Rust/lint/Skia·fresh WASM·개별 최종review·오늘할일·통합PR/CI/merge/후속 처리는 미완료다.

- 공통 helper를70줄 이하 `stored_page_origin.rs`로 이관했고 두 소비자가 같은 판정을 사용한다. 다음 앞 간격0인 정확한 저장 사다리도 쪽 상대 원점0으로 판정하며 기존 명시 쪽나눔/합성/TAC 도형/overlay 제외는 유지했다. dev Native build **exit0/94.984초**, immutable binary SHA256 `29c360ec04233e5b7f7a51213bd7e5bed71823bca33526c44fb6b233162ab281` 및 네 production 파일 해시는 `page-origin-artifact-v108.json`이다.
- 최신 Native28쪽 compare·overlay·review는 **1쪽완료/누락0/exit0/33.141초**, 일치율 **61.98717→95.71002%**다(`native-cgmp-page-origin-review-v108-exit.json`, 해당 overlay_metrics.json). 새 `review_028.png`를 대화에 실제 표시했고 부칙 제목·뒤 본문·마지막 줄의 세로 원점 이동이 해소됐음을 직접 확인했다. 글자 모양·미세 간격 차이는 남으며 이 선택 쪽 결과를 전체 문서/fresh WASM 통과로 쓰지 않는다.
- 원본 전체48쪽을 같은 print/compat2022/font 명령으로 새 SVG export했다. 독립Print48쪽과 같고, v106 전쪽 + v107 변경 쪽의 정확한 기존 SVG와 비교해 **28쪽만 변경/나머지47쪽 동일**이다(`native-cgmp-page-origin-full-svg-comparison-v108.json`). 전체 SVG 비교·SHA 기록이 끝난 검토 전용48개 SVG는 압축하지 않고 삭제해 **1,629,601,295바이트**를 회수했다. 선택 review/overlay/PNG·원본·독립PDF·전체쪽 SHA manifest는 보존한다(`page-origin-completed-svg-cleanup-v108.json`).
- 정상 Native32쪽(한국어2/outline9/eogu21)과 명시 쪽나눔1853의10쪽·TAC 제목6797의7쪽·양수 앞간격사다리PrEP90쪽을 새 export해 기존SVG와 모두 동일함을 확인했다(`native-page-origin-svg-counters-v108.json`). SVG 진단이며 fresh raster/WASM 측정으로 보고하지 않는다.
- 기존 관련 검사 **145run/145PASS/4344filtered skip/exit0**다. 실제 실행90.981초, compile 포함304.662초이며 source 해시 불변을 확인했다(`page-origin-related-counters-v108-exit.json`). HeightCursor와 기존 표/문서 검사, #7345 첫 간격 원점 및 #6797 유효/합성/바깥 앵커 반례를 포함한다. format/diff check도PASS다. 시험지/HML 원본 **59개 바이트·Git mode 모두 보존**이다(`exam-originals-preserved-after-page-origin-v108.json`).
- 남은 표5쪽은 미해소다. 독립 Print의 실제 vector 경계를 추출해35쪽에는60행 빈 잔여가 없고,42쪽에는41쪽에서 글자를 소비한135행의 빈 밴드21.896px가 있음을 구분했다.37쪽78행의 이어받기 밴드는177.2467px인데 현재165.3px로 작다(`cgmp-independent-print-row-bands-v108.json`). 동일한 빈 꼬리 제거로 모든 경우를 처리할 수 없으며, 원본 물리 공간과 이미 소비한 유닛을 분리해 후속 계획에 연결한다. 전체Rust/lint/Skia/fresh WASM·최종 기록·원격 절차는 여전히 미완료다.

## 92. v109 — 판단 도구의 실제 계약과 잘못된 기존 검사의 대조

- 진행91을 `a51a4b0ae1a6e21dc86b75691919e86bc4b6d17a`로 커밋했다. upstream/devel은 기존 `21ed64071f81f35f37618717693bd30f727392c9` 그대로여서 checkout/rebase를 생략했고 worktree는 clean이었다(`cgmp-page-origin-checkpoint-sync-v108.json`).
- 기존 Node source-digest 검사는 W1의 역사적 source SHA와 다른 경로를 정확히 나열하지만 `rhwp-studio/src/core/local-fonts.ts`의 현재 drift를 누락한다. W1 candidate의 source SHA와 실제 파일·고정base 바이트를 직접 대조한 후 누락된 경로만 기대 목록에 반영한다. 역사적 candidate/source SHA를 갱신하거나 drift를 숨기지 않는다. 기존 고장 주입 검사는 계속 추가 drift와 상대 경로를 검출해야 한다.
- `oracle_vs_self`의 schema1 검사에는 실제 producer와의 불일치뿐 아니라 runtime binder도 schema2를 거부하는 결함이 있다. 현재 `visual_sweep.py`는 run schema2/page schema1이며 run schema2는 기존 소비 필드에 `font_mismatch_evidence`를 추가한다. 판단 도구의 contract/snapshot/live 검사를 run2에 맞추고 binder는 저장된 schema1과 현재 schema2의 같은 provenance/완료 상태를 소비한다. 알 수 없는 schema와 incomplete·기준PDF 없는 자료는 계속 거부한다. 실제 최신 run manifest를 읽는 확인과 기존 Python 전체 검사를 함께 수행한다.
- 이 단계는 제품 parser·font metric·조판·paint가 아니라 source-drift/판단 도구 계약과 그 검사만 변경한다. 새 렌더링 회귀·시각 fixture/golden·baseline은 추가/변경하지 않는다. Native/fresh WASM 미달을 통과로 바꾸는 데 사용하지 않는다. 별도 W6 lineage는 #7697의 범위 override와 parser/first-match/역사적 의미 해시를 더 대조해야 하며 여기서 통과라고 쓰지 않는다.

- `local-fonts.ts`의 W1 역사 SHA256은 `25d0e0d2e29f4d758b32c9ec20343fb43e4b022c4168cae1831d31fa93f437cc`, 현재/고정base는 모두 `78228d2db58ea6467271bfe3bd5018ae7fb16b17480cc23f453cd0dc186fe1c8`이다. 누락된 기대 경로만 보충했고 candidate·source·digest 구현은 그대로다. 기존 source-digest 고장 주입·상대 경로 검사 포함 **Node12/12PASS/exit0**다(`logs/source-digest-contract-v109.log`).
- 판단 도구는 run schema2/page schema1의 현재 contract와 fixture metadata를 기록하고, binder는 schema1/2의 `complete`만 수용한다. 현재 schema2 완료본을 거부하는 새 검사에서 수정 전 **FAIL**을 실행했다(`logs/oracle-schema-before-v109.log`). 수정 후 legacy/current 완료, unknown schema, incomplete/failed/빈 상태, 기준PDF 없음의 검사와 기존 Python 전체 **42/42PASS/exit0/7.837초**다(`logs/oracle-schema-all-v109.log`). metadata fixture 갱신이며 문서 시각 fixture/golden 갱신이 아니다.
- 실제 최신 두 run manifest도 수정 전에는 schema2라 `cheap_ok=false`, 수정 후에는 공통 envelope 필드를 읽는다. 기존 schema1의 `failed`가 잘못 수용되던 사례는 true→false다(`oracle-schema-live-manifest-v109.json`). 이 값은 envelope 필드 가용성이고 전체 쪽수·90% gate·시각 승인 증거가 아니다. 남은 CGMP5쪽의 `re_review_required`와 필수 최종 검증·W6 lineage 미검증을 해제하지 않는다. product Rust/WASM source는 진행91과 같으며 이번54개 도구 검사 결과를 전체Rust 통과로 쓰지 않는다. diff checkPASS다.


## 93. v110 — CGMP 회귀 검사 제거 판단 철회

- 현재 메인터너 보정 중인 CGMP에 #7445의 기존 이관 기록을 적용해 두 회귀 검사를 제거한 것은 범위 판단 오류다. 사용자의 정정에 따라 미커밋 제거 변경을 철회하고 `tests/cases/issue_6035_cell_split_empty_band.rs`를 HEAD와 바이트 동일하게 복구했다. 이번 삭제를 위해 만든 `.rs.txt` 사본도 제거했다. 이 단계에서 CGMP의 추가 이관·제외나 원격 issue 변경은 하지 않았다.
- CGMP는 현재 수정 대상으로 유지한다. 원본 HWPX·독립 Print48쪽을 보존하고 남은35/36/37/39/42쪽의 출력 차이를 보정한다.35쪽의 base96.65234→현재83.37607% 회귀도 계속 해결한다. 현재 전체 Native/fresh WASM90% 미충족이며 완료·통합으로 보고하지 않는다.
- 독립 Print에서 제목은35쪽, 소항목은36쪽이므로 기존 두 검사의 같은 쪽 강제와 제목 단독 쪽 금지 기대값은 재검토 대상이다. 검사 삭제로 해결하지 않고 원본·독립 출력과의 관계를 확인한 뒤 보정한다. 필요한 시각 선행 조건을 충족하기 전에는 기존 기대값을 변경하지 않는다.
- 일반적인 #7445 중복 부적합 회귀 정리와90% 이상 문서의 px→관계형 보정 지시는 유지하되, 현재 작업 중인 CGMP의 해결 범위를 축소하는 데 적용하지 않는다. 파생 suite를 다시 준비하고 두 검사가 복구된 등록 상태와 고정base 정책 검사를 확인한다.
- 복구 후 manifest `--prepare`와 `--check --base-ref 21ed64071f81f35f37618717693bd30f727392c9`는 모두 exit0이다. 원본 검사 파일은 HEAD와 바이트 동일하며 `regression_suite_028`에 다시 등록됐다.1459 source/6265 static test attrs로 두 검사 복구를 확인했다(`logs/cgmp-regression-restored-prepare-v110.log`, `logs/cgmp-regression-restored-check-v110.log`). 이는 등록·정책 확인이며 회귀 실행 통과나 시각 완료 판정이 아니다.


## 94. v110 — #3308 원본 오프셋 관계 검사 보정

- 현재 production head `388989c6c02f20ba379f35b74d5e576b44b94519`의 변경하지 않은 원본 `samples/task3307/issue3307_outline_number.hwpx`(SHA256 `28a60bd05e152fd5292bdc8b75c6dbd980a04375c0d8890b752e32017fd1731d`)와 독립 한컴2020 Print9쪽 `pdf/task3307/issue3307_outline_number-2020.pdf`(SHA256 `0b176d3020e89d6a896b4efcab87bb9683298ea10b8593ad799bf99009668acb`)를 대조했다. 고정base21ed의 Native 전9쪽 최저96.34221%이며 현재 Native/fresh WASM은 모두 전9쪽 최저97.34742%(7쪽), 미달·누락0/9대9쪽이다(`outline-source-offset-visual-qualification-v110.json`, 두 전쪽 TSV). 평균·대표 점수만으로 판정하지 않았다.
- fresh WASM 표준 root wrapper/공용 target 빌드는 exit0/1077.010초/source불변이며 새 WASM SHA256 `4b6fd08845979cbbb94a54fa4a5f99ffa20cad2c4e6d411ea3bb428e49e55b73`이다. immutable package/rootpkg/Studio public의 두 공급 파일은 각각 해시 일치다(`wasm-build-current-v110-exit.json`). 빌드·공급 확인을 Studio 편집 동작 확인으로 확대하지 않는다. fresh Native9쪽 raster와 WASM9쪽 raster도 모두 바이트 동일하다.
- 최신 Native/fresh WASM7쪽 compare·overlay·review를 새로 산출하고 직접 판독했다(`native-outline-current-v110/outline3308-direct/review/review_007.png`, `wasm-outline-current-v110/outline3308-direct/review/review_007.png`). 직인 표·오른쪽 셀·글자 포함을 확인했으며 미세 괘선·글자 간격 차이는 남는다. 기존 컴파일된 동일 검사에 현재 immutable Native renderer를 공급해 잘못된 가운데 assertion의 실제 FAIL/exit101을 재현했다(`outline-source-offset-before-v110.json`, 해당 로그).
- 원본 중첩 표는 LEFT/COLUMN 오프셋6226HU, 바깥 왼쪽 여백140HU, 부모의 유효 왼쪽 안 여백283HU를 가지며 부모 셀 폭48096HU/중첩 표 폭32635HU다. 가운데 배치를 강제할 근거가 없다. 기존 검사에 원본 파서의 폭·왼쪽 기준·오프셋을 연결하고 선언 폭과 오프셋의 상대 비율 및 기존 부모/직인 셀/글자 내부 포함 관계를 검사한다. 넓은 폭 비율 범위와 가운데 assertion을 원본 속성 기반 관계로 바꾸며 fixture·PDF·renderer·baseline을 바꾸지 않는다.
- CGMP는 여전히 현재 구현 보정 대상으로 유지한다. 두 기존 CGMP 검사도 그대로 보존하며 전체90% 미충족 상태에서 기대값 변경·통합을 진행하지 않는다. 남은 CGMP 물리 행 프레임 보정의 생산→스캔→예약→이어받기→paint 소비 계획과 반례는 `cgmp-ordinary-frame-correction-plan-v111.json`에 연결한다. 해당 계획은 아직 구현·검증 완료가 아니다.
- 수정 검사 실제 실행은 dev profile에서 **1run/1PASS/256filtered skip/exit0**이다(검사0.972초, compile 포함15.531초; `outline-source-offset-focused-v110-retry1-exit.json`, 해당 로그). 첫 선택은 정책 검사와 파생 suite 갱신 순서가 겹쳐 기존 suite013에서0run/exit4였으므로 통과로 세지 않았다. 정책 검사 종료 뒤 실제 등록된 suite008을 찾아 순차 재실행했다. prepare/고정base manifest check와 fmt/diff check도 통과했다. 전체Rust·필수 세 Clippy와 전체시각 gate 통과로 확대하지 않는다.
- 시험지/HML59개 원본의 현재 바이트·실행 권한이 커밋된 원본과 동일한지 재확인했다(`exam-originals-preserved-v110.json`). 모든 WASM production source hash도 fresh build와 일치한다. 이번 단계는 기존 회귀 기대값 보정이며 CGMP 구현 계획은 아직 적용하지 않았다.
- 수정한 기존 검사를 포함한 Native Clippy `--test regression_suite_008 -- -D warnings`도 exit0이다(`logs/outline-source-offset-test-clippy-v110.log`). 한정 Clippy이며 제출 전 필수 세 lint 단계를 대체하지 않는다.


## 95. v111 — CGMP 일반 행 쪽 프레임의 공통 점유 보정

- 진행94를 커밋한 뒤 fetch한 upstream/devel은21ed64071 그대로여서 checkout·rebase를 생략했다(`outline-source-offset-checkpoint-sync-v110.json`). 현재 출발 head는03616d394이며 CGMP는 현재 보정 범위를 유지한다.
- 독립 Print의34/35쪽에는60행의 빈 이어받기 조각이 없지만 현재35쪽에는1.7px 꼬리가 생겨61행 이하를 민다(`cgmp-ordinary-frame-before-cells-v111.json`, 고정base의 실제 `base-cgmp-row60-trace-v110` 출력).60행 마지막 원본 글줄의 다음 줄 간격360HU/4.8px보다 작은 물리 꼬리는 새 내용·독립 빈 밴드의 소유자가 아니다. 반대로 #5585의 약17px 빈 밴드는 이 원본 줄 간격을 초과하며 기존 비반복제목행 계약을 유지한다.
- 독립 Print36/37쪽의78행 상자는385.4974/177.2467px이고 현재는373.3/165.3px다.39쪽의97행 잔여도 선언된 물리 높이를 잃는다. 내용 컷·글줄 수만으로 선언 잔여를 대체하지 않고 원본 반복제목행의 쪽 상한에서 수용한 앞 상자와 남은 내용 포함 잔여 상자를 함께 전달한다.135행의 단일 줄도 글자를 나누지 않고 선언 높이의 빈 꼬리를 다음 쪽에 보존해야 한다. 일반 다중줄 가운데 행의 전체 이월 규칙을 완화하지 않는다.
- 생산은 `stored_ordinary_row_frame`의 원본 행/컷 조회이며, 실제 컷과 각주 refit이 끝난 `scan_table_fragment`가 공통 끝 행 높이와 consumed를 수용한다. continuation은 같은 컷·앞 높이로 잔여를 계산해 start override로 전달하고 다음 스캔과 paint가 그 정확한 높이를 소비한다. paint의 정렬과 마지막 줄 간격 접기도 원본 프레임 소유를 소비한다. 소유 유닛 컷은 바꾸지 않는다. TAC·편집/reflow·중첩 controls·rowspan·원본이 아닌 줄 사다리는 이 경로에서 제외한다.
- 먼저 하나의 구현 묶음과 Native build 뒤 CGMP 경계35/36/37/39/41/42쪽·기존 분할 대조 검사를 실행한다. 같은 원본 전48쪽 Native/fresh WASM90% 전에는 CGMP 기존 기대값·golden을 변경하지 않는다. 현재 단계의 구현·실행 결과는 아직 미검증이다.
- 첫 묶음 dev build는 exit0/109.868초/source불변이며 frozen Native SHA256 `435bcb692cdd4f3a3745a8f4f9eee5c3d17e1e5b541a3bb1d863f32df2192723`다. Native 대조32쪽은 exit0/398.130초, 한국어2쪽 최저96.91813%, outline9쪽97.34742%, eogu21쪽93.26292%로 미달·누락0이다. 관련 기존 검사158개는 exit0/전부PASS(검사95.181초, compile 포함480.238초;1slow/6033filtered skip)이며 #5585의 실제 빈 밴드·#5714·#3308 대조를 포함한다. 전체Rust/freshWASM 완료로 확대하지 않는다.
- 첫 CGMP 선택8쪽은 exit1/577.670초/source불변이며35쪽98.46451%,37쪽97.65544%로 개선됐지만38쪽76.14151%,39쪽74.83172%,42쪽48.58196%가 미달이다.35/37/38/39쪽 새 review를 직접 표시·판독했다. 전48쪽 fresh SVG는48대48쪽/exit0이며32~41쪽이 달라32/33/40쪽도 후속 확인 범위에 추가한다. 개선만으로 전체 승인하지 않는다.
-38쪽 회귀는84행의 작은 선언 잔여36.6px가 일반 fallback을 통해 먼저 수용돼 남은 두 줄의 실제 요구40.5px를 잃은 것이다. 일반 per-row 컷의 이어받기 생산에서 선언 잔여와 같은 컷의 내용 요구 중 큰 값을 전달하도록 보정한다. 블록 컷의 별도 인덱스 공간과 완전히 소비된 #5585 빈 밴드는 유지한다.42쪽 단일 줄 진입은 기존 whole-row 측정이 이미 선언 높이를 포함해 `선언 > 내용`을 늘 거짓으로 만들었다. 같은 원본 유닛의 잉크+여백 점유로 판정하도록 수정한다. 현재 두 후속 보정은 새 build/검증 전이다.
- retry1 build는 exit0/110.018초/source불변이며 관련158검사도 전부PASS/exit0(실제96.019초/총385.921초)다. 그러나 Native 영향11쪽은 exit1/530.379초이며33/38/39/40/41/42쪽이 미달이다. 공개 DocumentCore에 현재 cached Native rlib를 연결해 원본과 실제 page items를 직접 읽었다(`cgmp-public-source-diagnostic-v111-retry1.rs`, 해당 로그).84행의 세 셀 모두 row_span1이며 dirty/vposdirty/controls는 없었다. 병합 블록 소비라는 앞선 추정은 틀렸다.
- 실제 누락은 생산자가 이미 입증한 문단 내부 원점0 경계를 일반 행 프레임 소비가 문단 간 리셋으로만 제한한 데 있었다.84행의0/0/1560,123행의0/1560/0은 독립 Print와 기존 저장 경계 생산 계약에 대응한다. 같은 원본 경계 판별자를 종료 상자·잔여·정렬에서 공통 소비하고, 선언 잔여의 최소 요구에는 같은 유닛의 잉크+유효 안 여백을 포함하도록 수정한다. 90% 미달을 이관하거나 기대값으로 덮지 않는다. 이 후속 수정은 아직 새 build/시각 확인 전이다.

- retry2 build는 exit0/107.843초/source불변, frozen Native SHA256 `bf5148720d526ed9727d1c3e0716065d7f4f5e0d9cbef9b9bf75acffe21d514d`다. CGMP 영향32~42쪽11개는 모두90% 이상/exit0/515.601초이며 최저39쪽95.37856%다.38쪽98.52483%,42쪽98.77406%로 잔여 상자와 빈 이어받기 프레임의 위치가 복구됐다. 관련 기존158검사도 전부PASS/exit0(실제99.736초/compile 포함279.080초)다. 최신 Native 대조32쪽은 최저한국어96.91813%,outline97.34742%,eogu93.26292%로 미달·누락0/exit0/359.626초다.
- retry2의33/35/37/38/39/41/42쪽 compare·overlay·review를 새로 생성해 직접 표시·판독했다(`native-cgmp-ordinary-row-frame-direct-v111-retry2/cgmp6035/direct-review-provenance.json`).35쪽에는 마지막 줄 전진만 남은60행의 독립 빈 꼬리가 없고,42쪽에는 원본135행의 실제 빈 밴드가 남아 뒤 본문이 Print와 대응한다.39쪽의 글자·작은 경계 차이는 남으며 픽셀 완전 일치로 보고하지 않는다. 기존 CGMP2검사는 HEAD와 같은바이트/sha256 `da83a6fb3c0e296b0e52ca72bda14362bbf1f083964afccc8c528329281ad4a2`로 유지한다. 전48쪽 Native와 새production의 표준fresh WASM build를 시작했으며 전체90%/최종gate는 아직 미검증이다.


- v111 retry2의 같은 원본/Print 전48쪽 Native sweep이 exit0/1051.978초/source불변으로 종료됐다. 실제 TSV는48쪽 모두 누락·90%미달0이며 최저47쪽94.00342%다(`cgmp-ordinary-row-frame-native-whole-summary-v111-retry2.json`). 이는 Native 전체 확인이며 진행 중인fresh WASM 및 필수 최종gate를 대신하지 않는다. 기존2검사 기대값 변경은 아직 하지 않았다.

- 같은 current production으로 표준 WASM wrapper가 exit0/1126.432초/source불변으로 종료됐다. frozen/rootpkg/Studio public의 wasm SHA256은 모두 `ef22409aad9a830d2cec2b4650fc2c4cdb7aba71f15dca03865276a548ec869a`다. 이는 package 공급 확인이며 Studio 실제 편집 동작 확인으로 보고하지 않는다. frozen fresh WASM의 CGMP48쪽과 일반대조32쪽 비교를 시작했다. Native 전쪽의1/47쪽 review도 새로 생성해 직접 표시·판독했고, 외곽·내용 소속은 대응하나47쪽 증명서의 제목·서명 글자 굵기/폭 차이는 남아 있다. 기존 CGMP2검사 기대값은 아직 변경하지 않았다.

- 최신fresh WASM 일반대조32쪽은 exit0/272.432초/source불변, 미달·누락0이다. 최저한국어96.91813%,outline97.34742%,eogu93.26292%이며32개 실제 raster는 최신 Native와 바이트 동일하다(`wasm-ordinary-row-frame-counter-minima-v111.json`, `ordinary-row-frame-normal-native-wasm-raster-parity-v111.json`). CGMP 전48쪽 fresh WASM은 진행 중이며 이 대조32쪽으로 전체 CGMP 통과를 주장하지 않는다.

## 96. 2026-10-10 — #7445 대표 이슈와 문제별 하위 이슈 정리

- 사용자가 #7445를 대표 이슈로 바꾸고 첫 본문·전체 코멘트를 하위 이슈로 연결하도록 지시했다. 실제 기존 본문1개/코멘트103개/직계 하위 이슈0개를 snapshot으로 보존했다. GitHub의 직계100개 한도를 확인한 뒤 사용자가 같은 문제의 후속·정정 코멘트를 묶고 모든 원문을 연결하는 방식을 선택했다.
- 기존 본문과103코멘트를 누락 없이77개 문제로 정리했다. 복수 문제를 다룬86/87/93/96번 등은 해당 하위 이슈들에 함께 연결한다. 입력 형식이 다른 문서는 독립 검증 범위를 보존하고, 원문·명령·수치·원 author/time·출처 링크를 각 하위 이슈에 그대로 보존한다. 당시 로컬 완료/이관 철회/미검증은 최신 완료로 승격하지 않는다. 기존 첫 본문은 #7716에 원문 전체를 등록했다.
- CGMP는 #7764에서 현재 `review/semanticist21-20261008`의 메인터너 보정으로 추적한다. 과거 셀 넘침 원장 한 축의 이관 기록과 현재 작업의 범위를 구분했고, 잘못 제거한2검사 복구·Native 경계11쪽90% 및158검사PASS·전48쪽/fresh WASM 진행 상태를 명시했다. CGMP 코드·검사를 추가 제외하거나 이관하지 않는다.
- GitHub 구조 변경 실행 원장은 ignored `issue7445-hub-remote-ledger-20261010.json`, 전체 source snapshot과77개 묶음은 `issue7445-hub-{source,comments,group-plan}-20261010.json`에 보존한다. 하위 이슈 생성→실제 부모 연결→본문·title·parent API 재확인 뒤 기존103코멘트에 연결번호를 추가하고 마지막으로 대표 본문을 목차로 갱신한다. 현재 실행 중이며 전체 완료·검증 결과는 terminal 후 이어 적는다.

- 사용자 후속 요청에 따라 하위 이슈에 보존한 기존 코멘트103개도 삭제한다. 코멘트에 링크를 추가하던 자체 runner를 중단했고,77개 실제 하위 이슈에 본문1개+코멘트103개의 원문이 모두 남아 있음을 API로 먼저 대조했다. 대표 본문은 삭제된 댓글 URL 대신 하위 이슈와 원기록 순번의 목차로 바꿨다. 현재103개 원댓글을 시간순으로 정리 중이며, 삭제 후 전체 source-preservation/실제77개 부모 관계/댓글 잔여를 다시 검증한다.

- #7445 구조 정리는 실제 API 재확인 후 완료했다. 대표 제목은「[대표 이슈] 문서 전체 피델리티 개선과 회귀 검사 재구축」이며77개 실제 직계 하위 이슈(#7716~#7792)를 연결했다. 기존 본문1개와103코멘트의 원문104개가 해당 하위 이슈들에 모두 남아 있음을 삭제 전후 대조했고, 원 코멘트103개는 삭제/잔여0이다. parent title/body·77개 실제 관계·원문 보존을 모두 재확인했다(`issue7445-hub-remote-ledger-20261010.json`의 audit, 최종parent/subissues/comments snapshots).
- 추가 사용자 정정에 따라 대표 본문의「현재 메인터너 보정」섹션도 제거했고 API로 부재를 확인했다. 대표 본문은 문제별 하위 이슈 목차와 공통 완료 기준으로 유지한다. 이는 #7764의 현재 CGMP 보정 범위나 기존2검사의 유지 상태를 바꾸지 않는다. GitHub 구조 정리 완료를 CGMP fresh WASM·전체Rust·필수lint·통합PR/CI/merge 완료로 쓰지 않는다.

## 97. v111/v112 — CGMP 전쪽 fresh WASM과 관계 회귀 보정

- 같은 v111 retry2 production과 독립 Print의 fresh WASM 전48쪽 비교가 exit0/1858.12초/source불변으로 종료됐다. 최저47쪽94.00342%, 미달·누락0이며 Native48쪽과 TSV SHA `f08521aa545c81c87a9a33a74194cbaa004227153eae3c82d331b6a46dff1478` 및 모든48 raster가 바이트 동일하다(`cgmp-ordinary-row-frame-wasm-whole-summary-v111.json`, `cgmp-ordinary-row-frame-native-wasm-whole-parity-v111.json`). 전체 쪽 수48도 독립 Print와 같다. 이 결과는 SVG 경로의 확인이며 Studio Canvas·Native Skia와 최종 전체 회귀를 대신하지 않는다.
- 최신 fresh WASM35/38/39/42/47쪽 compare·overlay·review를 별도로 생성했다(`wasm-cgmp-ordinary-row-frame-direct-v111/cgmp6035/direct-review-provenance.json`).35/39/42쪽 `review.png`를 직접 표시·판독했고 앞서 확인한 Native 경계와 같다.35쪽에서60행의 작은 줄 전진 꼬리가 제거됐고,42쪽에서135행의 실제 빈 밴드와 뒤136행은 보존된다.39쪽의 일부 글자 폭·작은 간격,47쪽 증명서 제목·서명의 글리프 두께 차이는 남는다. 점수 통과를 픽셀 완전 일치·최종 승인으로 쓰지 않는다.
- 독립 Print35/36쪽의 실제 텍스트와 원본75행을 대조했다(`cgmp-print-water-supply-v112.txt`, `cgmp-public-source-diagnostic-v112.rs`와 로그). 물 공급 설비 제목의 원본 첫 문단은35쪽, 세 하위 항목은36쪽이다. 첫 문단과 다음 문단의 유효한 저장 원점0 경계를 같은 쪽으로 강제하던 기존2검사는 정본과 달랐다. 기존 검사를 제거하지 않고 제목/세 항목의 실제 셀 소속·내용 순서·완전 보존·셀 내부 포함, 앞 평가 행들의 존재와 같은 소유 컷 이어받기로 보정했다.
- 같은 정식 `tests/cases/issue_6035_cell_split_empty_band.rs`에 실제 출력의 물리 관계 검사3개를 독립적으로 추가했다.60행의 완전한 내용 컷 뒤 다음 행으로 바로 전진하고,84행의 문단 내부 경계와97행의 문단 간 경계에서 원본 내용이 두 조각에 빠짐없이 나뉘고 글줄이 소유 셀 안에 있으며 두 조각이 원본 최소 행 높이를 잃지 않는지 확인한다.135행은 앞 제목과 뒤 빈 밴드 높이의 합이 원본 선언 높이를 정확히 한 번 소비하고, 뒤136행이 그 밴드 아래에 이어지는지 검사한다. 내부 helper 계산값이나 절대 px 기대값을 재인용하지 않는다.
- 현재 코드의 공식 suite020에서3검사 모두PASS/exit0(실제2.295초/전체18.345초/199필터skip), production불변이다(`cgmp-relational-current-v112-exit.json`, 해당 로그). 최초 컴파일의 반환 타입 `&Box<Table>` 불일치는 `&Table`로 바로잡았다. 이는 구현 결함의 수정 전FAIL 증거로 세지 않는다. 고정base21ed 대조는 build/compile exit0 뒤 source 종료 컷 부재로3FAIL이다(`cgmp-relational-before-v112-exit.json`). 이 결과만을 v111 보정의 개별 결함 검출로 쓰지 않고, 아래의 보정 직전036 대조를 별도로 수행했다.
- v111 production의 포맷 검사 exit0/27.963초, Native Clippy exit0/128.813초, WASM32 Clippy exit0/115.136초다. production SHA inventory는 불변이며 root/package/Studio 공급 SHA는 진행95의 표준 build 기록을 따른다. workspace build·all-target Clippy·최종 전체 회귀와 다른 문서 보정은 아직 남아 있다.
- 시험지/HML 원본59개의 실제 바이트·실행 권한을 HEAD와 다시 대조했고 전부 보존됐다(`exam-originals-preserved-v112.json`). CGMP 원본과 독립 Print도 HEAD에 커밋된 내용과 바이트 동일하다. #7445 본문의 사용자 요청 문단 제거는 CGMP 보정 대상 제외를 뜻하지 않는다.

### 보정 직전 대조와 최종 단계 확인

- 보정 직전036 production을 소유한 별도 검증 worktree에서 같은 공유 `target/pr-review`의 release-test library로 빌드했다(exit0/218.615초). 동일 정식 검사 원본을 `rustc --test`로 연결했으며 build/compile 실패를 재현으로 세지 않는다. worktree는 확인 뒤21ed로 복구했다(`cgmp-relational-precise-before-v112-exit.json`). 강화한 최종5검사의 before 대조는 기존 의미 보정2PASS/새 물리 반례3FAIL이다(`cgmp-relational-precise-before-five-v112-retry1-exit.json`, 해당 로그). 실패 원인은60행의 빈 줄 전진 꼬리,135행의 한 줄 물리 분할 컷 부재, 원본 reset 행의 최소 물리 높이 손실이다. 기존2검사의 정상 기대값 정정과 새3검사의 구현 결함 검출을 구분한다.
- 현재 production의 공식 suite008에서 강화한5검사 모두PASS/exit0, 실제2.227초/전체107.482초/223필터skip다(`cgmp-stage-gates-v112-retry1-exit.json` step3). before와 after의 정식 검사 원본 SHA는 동일한 `2f52aba0906860e852c7216668b4a3c915aeda9db4d61db3f8626e9956f4964a`다. 파생 suite의 배정은 준비 시점별로 달라지므로 과거 suite020/012를 현재 배정으로 재사용하지 않는다.
- 필수 workspace build는 exit0/96.777초다. 전체 target Clippy에서 과거 시험지 HML 검사 제거 뒤 남은 `RELSIZE`·`ShadeColor`·`RATIO` fixture 설명 주석의 세 오류가 차례로 검출됐다. 해당 고아 주석만 정리했고 기존 정상 계약·원본·생산 코드는 변경하지 않았다. 같은 패턴을 전체 test source에서 함께 검색했으며 실행되지 않는 legacy pilot의 유효한 문서 주석을 삭제 대상으로 확대하지 않았다.
- 최종 재실행 `cgmp-stage-gates-v112-retry2-exit.json`의 prepare/fmt/fmt-check/workspace all-target Clippy/고정base manifest/고정base unit-tier는 모두exit0이다. Clippy는15.571초, manifest5.683초, unit-tier26.854초이며 production과 최종 CGMP 검사 원본 모두불변이다. Native·WASM32 Clippy는 진행97의 같은 production 성공 기록을 결합한다. 최초 manifest drift는 마지막 검사 편집 뒤 파생 harness를 다시 준비해 해소했으며 정책 완화나 파생 파일 커밋은 없다. 최종 전체 release-test/Native Skia/다른 문서와 편집 증적은 계속 남아 있어 통합 준비 완료로 보고하지 않는다.
- 종료된 CGMP Native/fresh WASM의 SVG143개를 해시 기록 뒤 삭제해6.092GiB를 확보했다(`cgmp-completed-svg-cleanup-v112.json`).47쪽 Native SVG 한 개는 글꼴 조사용으로 남겼고 실제 TSV·PNG·review·입력·PDF·실행 및 export provenance는 보존했다. 압축 보관하지 않았다.
-47쪽 Print의 실제 GungsuhChe title은18pt, SVG는24px이며 첫 글자 원점은116.64/116.5267px, 기준선은234.88/234.8667px로 대응한다(`cgmp-certificate-print-font-transforms-v112.json`, `cgmp-certificate-source-font-attributes-v112.json`). 글리프 차이 원인은 이 좌표만으로 확정하지 않는다. 임베딩 CSS의 별칭으로 SVG 글꼴 실체를 식별하는 시도는 불완전하므로 글꼴 누락·해결 불가능 예외로 주장하지 않는다.
- 정책연구 #7753의 최신 본문을 다시 읽어 이전 이관 철회와 #7505의 전215쪽90.01587% 확인 기록을 확인했다(`issue7753-policy-current-v112.json`). 현재 #4882 원본/왕복 쪽수 차이는 오래된 미달 기록만으로 제거할 수 없다. 같은 최신 고정base와 원본/독립 Print의 직접 증거를 확보하며 조판·저장 차이를 계속 조사한다.

## 98. v112 — 남은 구조 회귀의 묶음 재현과 원본 시각 검토

- 역사 전체 v103 실패 중 구조·쪽 소속 검사11개를 현재 공식 suite에서 한 묶음으로 다시 실행했다. exit100/98.604초, 실제50.441초/11FAIL/0PASS/2183필터skip이며 production불변이다(`pending-structural-regressions-current-v112-selection.json`, command/exit/log). 이11개만으로 남은 전체 실패 건수를 단정하지 않는다. 기존27FAIL 전체를 최종 head에서 다시 확인하는 검증은 남아 있다.
- 설계업자 평가기준의 `samples/issue5885/3171199_design_capability_criteria.hwp`와 issue6126 경로는 SHA `6980eda2b04765cc8051da8581ee7f29fc9cb4780bb42ae14ab98883c1e90a69`로 동일하다. 독립2020 Print SHA `dabd9f8a992a80ff69916ff91ac0dcf65949739875712f470362aa472bec86d9`의 전7쪽 Native/fresh WASM 모두 최저5쪽96.39917%, 미달·누락0/exit0다.1/2/5쪽 대표 review를 양쪽으로 생성했고 Native1쪽·WASM2쪽을 직접 표시했다.2쪽 하단 외곽이 Print보다 짧게 남으므로 수치 통과와 별도로 구현 보정을 계속한다.
- #5885 실패는 독립 PDF 상단·하단 비교가 아니라 실제 paint보다 긴 본문 점유 검사에서 발생한다. 첫 바깥 표의 bbox 끝1013.3px와 실제 괘선 끝1003.7px 사이에9.6px 차이가 남으며, 같은 바이트를 쓰는 issue6126/5885 본문 넘침 실패와 함께 추적한다(`design-physical-frame-diagnostic-v112.json`). `table_partial.rs`의 배치 후 줄간격 접기, 이후 자식 포섭으로 높이를 다시 확장하는 소비 위치와 pagination 예약을 함께 대조한다. 그리기만 숨기거나 실패한 관계 계약을 약화하지 않는다.
- 고용지원 평가기준 `samples/issue6025/3232693_employment_support_criteria.hwpx` SHA `b53e9a83a79903b9c7cf69521519dcdf7b128348ac26ec21e6ac7b8fc6c0e441`, 독립 Print SHA `985dab08280f0cb7a2489c796614337e230871428bc0ac98b5cd3be4b5abd579`를 같은 production의 Native/fresh WASM 전4쪽으로 비교했다. 두 경로 모두1쪽89.49447%·2쪽81.14196%로 미달/exit1이다. #6950의 원본 외곽 위여백141HU 누락 의심은 아직 검사 변경 근거로 채택하지 않는다. 고정base의 같은 원본 시각 증거와 동일 기존 하위 이슈도 확인되지 않았으므로 검사를 임의 제거·이관하지 않는다.
- 양 문서의 실제 TSV·해시·범위는 `cgmp-and-pending-regressions-qualification-v112.json`에 연결한다. 이후 구현을 바꾸면 영향받은 Native/fresh WASM 전쪽과 대표 review를 새 production에서 다시 산출한다. 초기18개 통합 범위와 후속6개 체리픽 금지는 유지한다.


## 99. v113 — 설계업자 평가기준의 공통 첫 줄 컷과 물리 예약 보정

- 실제 원본의4행0열 병합 셀과 같은 시작 행의 일반 형제들은 첫 세 저장 줄이 `0 → 0 → 1920HU`이고, 줄 높이1200HU·줄간격720HU·위아래 안여백 각141HU다. 같은 source 쪽 경계인데 병합 셀에 같은 rowspan 형제를 추가 요구해 종료 줄간격9.6px이 예약에 남았다. PDF 첫 조각 하단1002.902667px·source 컷 `[1,1,1,1,0,0,0]`와 실제 로그를 함께 대조했다. 입력·독립 Print SHA는 진행98과 동일하다.
- `stored_first_line_restart.rs`로 기존 판정을 분리하고 Native HWP5에서는 같은 시작 행의 진짜 저장 첫 줄 재시작을 공유하도록 잘못된 rowspan 동일성 가정을 제거했다. 첫 줄 다음에 전진하는 저장 사다리를 확인하므로 원점0만 반복하는 입력·끝난 형제·개체 로컬 원점은 근거로 승격하지 않는다. 편집/재조판/글자처럼취급 제외와 HWPX의 동일 span 소유자 확인은 유지한다.
- 실제 소비 연결은 `native_saved_reset_cut_trailing_trim → CellCutFrame::height → offset 컷 walk의 physical_height → SelectedBlockCut::occupied_height → block_step의 누적 consumed`이며, paint는 같은 trim을 `row_cut_content_height/cell_cut_visible_height → table_partial의 per-row/병합셀 요구 높이 → grid·bbox·반환 높이`에 적용한다. 내용 유닛 전진25.6px와 물리 점유19.76px를 구분했다. 수정 후 모든 해당 시작 셀이19.76px을 예약하며 paint 후 접기 로그가 사라졌다. 첫 표 bbox는823.7px에서814.3px(획 포함)으로 바뀌고 실제 외곽선·글자 raster는 전7쪽 모두 이전 정상 출력과 바이트 동일하다. 그리기 숨김이나 bbox clamp 변경은 없다.
- 기존 정식 `issue_5885_nested_host_row_ladder_end`의 두 검사를 수정하지 않고 공식 suite020으로 실행해2PASS/exit0/전체147.204초·실제0.958초·202필터skip을 확인했다. 수정 전 현재 production의 첫 조각 검사는 paint보다 긴 본문 점유에서FAIL이었다. 수정 후 예약 끝과 실제 괘선의 관계, 동일 cut의 이어받기, 중첩 표 포함·뒤 행 순서·마지막 문단 보존까지 기존 검사가 통과한다. 새 fixture/golden/기대값 완화는 없다.
- 새 Native binary SHA `dabfbdd5ef9346f40c516d7d58a73e9255d594583ef00a756177dff5843c22de`, 빌드 exit0/107.272초다. 새 전7쪽 Native Visual Sweep은 exit0/71.617초·최저5쪽96.39917%·미달/누락0이며1/2/5쪽 review를 새로 만들고1쪽을 직접 표시했다.2쪽의 짧은 하단 외곽선은 여전히 미충족이다. 독립 PDF의 해당 행882.394667→1002.902667px과 현재980.2px 하단을 기록했고, 내용 종료 이후 실제 빈 물리 밴드의 소유를 먼저 조사한다. 점수로 이 차이를 면제하지 않는다.
- 표준 fresh WASM 빌드는 공유 target에서 진행 중이다. 그 결과·새 WASM 전쪽·정상 대조군·필수 lint·최종 전체 Rust와초기18개 통합PR/CI/merge/후속처리는 아직 완료가 아니다. 검증은 `design-shared-first-line-stage-validation-v113.json`·명령/exit/log와 Native 직접 review provenance에 연결한다. Rust 검증 도중 파생 suite를 갱신하거나 다른 Cargo를 동시에 실행하지 않는다.


### v113 fresh WASM와 후속 프레임 근거

- 표준 fresh WASM build가 exit0/1092.476초로 끝났다. Rust compile9분41초 후 wasm-opt까지 완료했으며 입력 source hash는 Native와 같다. 새 package·root `pkg`·Studio public의 WASM SHA는 모두 `91bbf9f43ff7d1314dde1d12ef85fac6b2ab1df7e087c0b68a97593412575b18`다. package 공급을 실제 Studio 편집/Canvas 실행으로 보고하지 않는다.
- fresh WASM 전7쪽 비교는 exit0/80.014초·최저5쪽96.39917%·미달/누락0이다. Native와 전7쪽 raster와 TSV가 바이트 동일하다(`design-shared-first-line-native-wasm-parity-v113.json`).2쪽의 하단 외곽선 차이는 양쪽에 그대로 남는다.
- 후속 source frame의 근거를 원본 공개 API 진단으로 확인했다. `host.vpos6720 + object.offset129 + outMargin.top283 + common.height61085 = 68217HU`, 원본 본문 높이68601HU에서 기존 바깥 아래여백283HU와 기존 쪽 경계 inset100HU를 빼면68218HU다. 차이는−1HU다. 원본 용지는59528×84188HU이고 독립 Print의841pt 용지 크기에 대응시키면 예측 하단1002.990497px과 실제1002.902667px의 차이는0.087831px다. 구현에서 PDF 좌표를 기준값으로 주입하는 것이 아니라 원본 첫 프레임이 본문 경계를 소유함을 입증하는 근거다(`design-body-closed-source-frame-plan-v114.json`, `logs/design-body-closed-source-diagnostic-v114.log`). 같은 source 컷 뒤의 실제 빈 밴드 예약·paint와 각주 refit/종료/비적용 경계는 아직 구현·검증 전이다.
- 정상 대조군 묶음은 과거 suite target 목록을 먼저 실행했으나 현재 case 배정과 대조해5개 integration target 누락을 발견했다. 현재 manifest에서 도출한 누락 target 추가 검증을 준비했으며 기존 묶음이 끝난 뒤 순차 실행한다. 전체158PASS로 미리 보고하지 않는다(`shared-first-line-related-counters-v113-coverage.json`).
- #5877 현재 원본은31쪽/off-canvas1/본문 넘침3, #1891 원본은70쪽/off-canvas6/본문 넘침15/글자 겹침2를 최신 immutable CLI로 다시 확인했다(`pending-overflow-originals-current-v113.json`). 과거 v95의 같은 두 원본·고정base 관측과 구분하며, 아직 동일 #7445 하위 이슈 이관 근거를 확인하지 못했으므로 기존 검사 제거의 근거로 쓰지 않는다. #5877의 독립 Print는 기존 `pr_7275_review.md`에 실제2020 MCP job·무전처리·원본/SHA/31쪽 출처가 기록돼 있음을 재확인했다. 현재 전31쪽 Native Sweep은 진행 중이다.

- 정상 대조군은 현재배정의 누락16개까지 실제 실행해 합계158/158PASS다. 첫142개 exit0/전체216.151초·실제89.376초/6026필터skip, 추가16개 exit0/전체41.813초·실제13.674초/1086필터skip이다. 두 실행과2개 #5885 검사 원본/생산 code의 SHA는 불변이며 phase99 전체 Rust10k 통과로 확대하지 않는다. 추가 target을 실제 실행한 뒤에만158PASS로 기록한다.

## 100. v114–v116 — 공동 문단 원점과 후속 쪽의 물리 프레임

- v113 이후 실제 본문 넘침 원장 partition3/8/12를 현재 suite005에서 재실행했다. #5885/#6126의 같은 설계 원본 두 실패는 해소됐고 partition8/12는 PASS다. partition3의 실제 남은 증가는 `issue5877/fragment_ghost_vrules.hwp`의2→3건이다. 다른 문서의 실패로 잘못 대체하지 않는다(`design-body-overflow-post-fix-v113-command.json`, exit100/96.761초·실제93.942초).
- #5877의 동일 원본 SHA `263a96fa24b4070a4a22cc4a62d878c50b7586d0741e039c58aa18615273d5a3`와 실제2020 Print SHA `71dd147ab8d9be9f0d79b6e8d42c62d293cbb88e9a7cb645715aa6896343e22f`를 Native 전31쪽으로 비교했다. 최저9쪽44.89766%, 미달9/11/14/18쪽, 누락0이다. exit1/1146.891초이며 전체 실패를 대표 쪽 통과로 바꾸지 않는다(`ghost-whole-native-qualification-v113.json`). physical9가 layout-anomaly의0-based8이며, 정상 physical8의98.58925%를 실패 쪽의 점수로 사용하지 않는다.
- 실제9쪽의 pi19에는 TAC 작은 표 ci1과 Para 기준 부동 표 ci0이 같은 원본 줄에 있다. ci0의17211HU 오프셋은 문단 원점을 기준으로 하지만 이전 paint는 ci1이 소비한 흐름 높이 뒤에 다시 가산했다. `FragmentInput.prepared.budget_para_start_height → coanchored_first_fragment_origin → FragmentBudget.fragment_placement.table_top/page_avail → scan → emit의 확정 배치 → layout_partial_table_resolved.resolved_table_top`으로 같은 원점을 전달한다. 이미 이 단에 실제 놓인 같은 줄의 TAC만 증거로 쓰며, 다른 문단/줄·미배치 형제·이어받기·편집/합성/재조판은 제외한다.
- v114 후보의 #5877 영향4쪽 비교는9쪽99.54633%로 개선됐지만11쪽60.44945%·14쪽70.55769%·18쪽70.75674%가 남아 exit1이다. 새9쪽 `native-source-frame-direct-v114/ghost5877/review/review_009.png`를 실제 표시해 큰 공백 제거, 앞 작은 표와 뒤 큰 표의 소유·내용 보존을 대조했다. fresh WASM 전31쪽 및 전체 판정은 아직 없다.
- 설계 원본의 첫 Common 상자가 원본 본문 경계를−1HU로 닫는 근거는 phase99와 같다. 후속 공동 source 컷의 실제 빈 띠를 스캔에서 수용하고 `consumed/end_row_height_override`로 예약·paint에 전달한다. 각주 refit도 같은 scan closure의 실제 예산을 대조하며, 종료 컷이 없는 조각·병합 소유자를 표현하지 못하는 일반 행 컷에는 이 공동 종료를 추정하지 않는다.
- v114는 새 물리 프레임을 전체 행의 압축 높이로 읽어 일반 `full - limit` 잔여를 다음 쪽에 전달했고, 설계4쪽24.08327%와 관련163개 중3FAIL을 만들었다. 해당 후보는 거부했다. v115는 쪽의 빈 물리 공간과 행의 잔여 내용을 구분해 이 잘못된 전달을 제거하고, 시작 블록 컷과 현재 종료 일반 행 컷을 각각 판정했다. 두 후보의 명령·source hash·exit·실패 로그를 보존했고 기대값·baseline·fixture를 바꾸지 않았다.
- v115 Native 설계 전7쪽은 최저3쪽96.17193%, 미달/누락0이며4쪽99.40835%로 회복됐다. 현재 공식15개 suite+관련 unit/2개 #5885/본문 원장3개는 **163/163PASS**, exit0/303.009초·실제115.733초/7107필터skip이다(`source-frame-anchor-related-v115-command.json`, exit/log). v114의 synam001 쪽수·#5875/#5782 본문 증가 세 실패도 실제 재실행으로 해소됐다. 이를 전체 Rust 회귀 통과로 확대하지 않는다.
- 설계2쪽 외곽선은 v115에도 짧게 남아 시각 완료가 아니다. v116은 실제 미소비 셀들의 종료 컷을 대조해 서로 다른 줄 높이의 공동 재시작을 확인하며, 논리 유닛 전진량 `split_end_limit` 대신 안여백을 포함한 공통 `row_cut_content_height`를 마지막 행의 물리 높이로 소비하도록 보완 중이다. 기존 개별 인증 컷은 유지한다. v116 Native/fresh WASM·정상 대조군·필수 lint·전체 회귀는 실행 전/진행 중으로 남긴다.
- 종료된 #5877 v113 SVG26개를 해시 기록 뒤 삭제해1329.2MiB를 확보했다. 조사 쪽8/9/11/14/18 SVG와 모든 PNG/TSV/JSON, 원본/Print, shared target은 보존했으며 압축 보관하지 않았다(`ghost-completed-svg-cleanup-v114.json`).

### v116 확대 판독과 v117 공동 소유 교정

- v116은 Native 전7쪽·관련163개가 모두 통과했지만 실제3쪽 review에서 외곽선이 PDF보다 아래로 늘어난 것을 확인해 후보를 거부했다. 원본 첫 Common 상자의 본문 종료는 이후 모든 내용 컷의 본문 종료를 증명하지 않는다. 완료 셀을 후속 쪽의 공동 소유자로 취급한 가정을 제거했다. 현재3쪽의 `[7,36,35,36]` 중 앞 세 셀은 내용을 완료했고 마지막 셀만 이어진다.2쪽 `[4,4,4,4]`는 실제 미소비 소유자 네 개가 새 원본 쪽을 공유한다.
- v117은 실제 미소비 소유자 둘 이상이 같은 종료 컷을 공유할 때만 본문 빈 띠를 소유하게 한다. 개별 종료 간격의 원본 인증과 실제 공동 컷을 대조하며 글꼴 줄 높이의 동일성을 요구하지 않는다. `split_end_limit`은 내용 전진량이므로 안여백을 포함한 `row_cut_content_height`를 마지막 행의 물리 점유로 사용한다. 행 전체를 압축한 잔여와 쪽 빈 띠의 구분은 v115 교정을 유지한다.
- v117 immutable Native SHA `0269e34de874a2242e0844de4ed75dfb7829e919e2780272baefb877b4cd3d5f`, build exit0/84.034초·source불변이다. 같은 설계 원본/독립 Print 전7쪽은 exit0/70.936초·최저5쪽96.39917%·미달/누락0이며2쪽99.35517%,3쪽98.50665%다.2쪽의 짧은 외곽선은 보정하고3쪽을 내용에서 닫는 원래 위치는 복구했다(`native-design-body-frame-v117/design5885/silhouette.tsv`, `native-source-frame-direct-v117/design5885/review/`).
- 현재 공식 suite/관련 unit/2개 #5885/본문 원장3개는 v117에서 **163/163PASS**, exit0/269.581초다. 전체 Rust 회귀·다른 실패 원본 통과로 확대하지 않는다. #5877 영향4쪽은 여전히9쪽99.54633%·11쪽60.44945%·14쪽70.55769%·18쪽70.75674%이며 exit1/218.175초다. 남은 세 쪽의 원본 선언 최소 높이와 종료/시작 컷을 조사한다.
- v117 표준 fresh WASM build는 같은 source의 공용 target에서 시작했다. Native/fresh WASM 전7쪽 조건과 직접 판독을 모두 확인한 뒤 새 관계 회귀를 등록한다. 제안한 원본2/3쪽 외곽선의 종료 관계 검사는 아직 ignored 후보이며 정식 테스트·fixture·baseline은 이 단계에서 변경하지 않았다. 최종 lint/전체 회귀, 초기18개 개별 기록·오늘할일·통합 PR/CI/merge/후속 처리는 계속 남아 있다.


### v117 fresh WASM·관계 회귀 완료 (2026-10-11)

- 표준 `scripts/wasm-pack-locked.sh --target web --out-dir pkg`는 exit0/1107.402초·production source 불변이다. frozen `wasm-pkg-source-frame-anchor-batch-v117`과 루트 `pkg`·Studio public의 JS/WASM SHA가 각각 같다. WASM SHA256은 `9c053566c48d2df1643641080b845095f4d383fb96be716750b3402cd6b772c1`이다. 브라우저 새로고침·편집 검증을 대신한 결과는 아니다.
- 동일 원본 SHA `6980eda2b04765cc8051da8581ee7f29fc9cb4780bb42ae14ab98883c1e90a69` / 독립 Print SHA `dabd9f8a992a80ff69916ff91ac0dcf65949739875712f470362aa472bec86d9`의 fresh WASM 전7쪽은 exit0/74.852초·최저5쪽 **96.39917%**·미달/누락0이다. Native 전7쪽과 TSV 값 및 PNG 바이트가 모두 동일하다(`design-body-frame-qualification-v117.json`, `wasm-design-body-frame-v117/design5885/silhouette.tsv`).
- 현재 fresh WASM `wasm-source-frame-direct-v117/design5885/review/review_002.png`·`review_003.png`를 직접 판독했다. 2쪽의 하단 공동 프레임을 복원하고 3쪽의 단독 소유 내용 종료를 유지한다. 표 외곽·내부 표·글줄 순서에 큰 위치 차이/누락/겹침이 없으며 괘선 명도와 글꼴 raster 차이는 남는다. Native와 같은 실제 PNG이므로 두 backend의 이 직접 비교는 동일한 출력에 대한 판독이다. 초기 PNG 생성의 출력 폴더 누락을 고쳤고 overlay 제목의 zero-based 페이지 인덱스도 보정해 재생성했다.
- 위 선행 조건 이후 `tests/cases/issue_5885_continuation_body_frame.rs`를 등록했다. 독립 Print에서 2쪽 외곽 끝이 3쪽보다 아래라는 관계를 검사하며 절대 px 기대값/고정 영역/해시를 assertion으로 사용하지 않는다. 동일 최종 case를 frozen 수정 전 v113 CLI / 수정 후 v117 CLI에 실행한 결과는 **의도한 종료 관계 FAIL(exit101) → PASS(exit0)**이다. 수정 전 관계 `[(98.2,980.2),(98.2,986.8)]`는 실패 원인 확인용 관측값이며 기대값이 아니다. 원본 새 fixture·golden·baseline은 추가/완화하지 않았다(`design-body-frame-before-after-v117.json`).
- 정식 generated `regression_suite_014`의 신규 관계 회귀는 **1/1PASS**, exit0/15.984초이다(`design-body-frame-official-v117b-exit.json`). 최초 정식 빌드에서 직접 dev dependency가 아닌 tempfile 사용이 실패해 표준 라이브러리의 전용 임시 경로/Drop 정리로 고치고, 동일 최종 case의 수정 전후 증거와 정식 suite를 다시 실행했다. 최초 빌드 실패는 결함 재현 증거로 세지 않는다. 파생 suite/manifest는 stage하지 않는다. 앞서 관련163/163PASS와 함께 이 단계의 증거이며 전체 Rust 회귀 통과로 확대하지 않는다.
- #5877의 실제 v117 호출 경로를 원본/Print와 대조했다(`ghost-frame-trace-v118-exit.json`, `ghost-remaining-consumption-analysis-v118.json`). 11쪽의 `complete_block_next_height`가 병합 셀 잔여47.6px으로 미소비 새 완전 행의 원본 최소4960HU/66.133px을 제한한다. 독립 Print 첫 행 높이는66.008px이다. 다음 보정은 병합 잔여와 새 행의 요구 높이를 같은 예약/paint 경로에서 계상한다. 14쪽의 두0→0 소유자와 걸침 셀, 18쪽의 일반 셀 완료/병합 셀 마지막 유닛의 소유는 계속 분석하며, 빈 꼬리를 단순 제거하지 않는다. 이 분석만으로 #5877 완료 또는 회귀 변경을 선언하지 않는다.
- 최종 Native/WASM32/workspace Clippy·정책 검사·전체 Rust 회귀·실제 편집 검증과 초기18개 개별 기록/오늘할일/통합 PR/CI/merge/후속 처리는 미완료다. #5877 11·14·18쪽의90% 미달은 여전히 보정 대상이다.


### v118 적용 전 원인·반례와 배치 보정 범위

- v117 완료 커밋 `a09fc8b968dd7bb67e74fa6ffb185c586bd19d56` 이후 fetch한 `upstream/devel`은 `21ed64071f81f35f37618717693bd30f727392c9`로 동일하므로 rebase하지 않았다.
- #5877 새 완전 행의 최소 높이와 이어받기 본문 프레임 원점 두 원인을 함께 보정한다. 빈 `next_cut`의 `complete_block_next_height`는 앞 병합 칸의 잔여를 반환하지만, 새 완전 행의 내용/선언 높이는 미소비다. 같은 `row_cut_content_height(row,[],[])` 요구 높이와 병합 잔여 중 큰 값을 커서에서 생산하고 행 스캔·paint가 그대로 소비하게 한다. 컷이 있는 부분 행에는 이 완전 행 규칙을 적용하지 않는다.
- 원본의 `common.height`·저장 host·바깥여백이 기존100HU 본문 프레임 하단을 닫는 Native HWP5의 이어받기에서도 HWPX와 같은 상단 바깥여백을 예약/paint가 한 번 연다. #5877 pi20 common66800HU + outer141HU는 기존 본문 종료66942HU에 정수 반올림1HU로 맞고, 독립 Print 14·18쪽 표 원점115.233px는 현재113.373px + 원본 outer141HU/1.88px에 대응한다. 원본 인증 helper를 budget/paint 양쪽에 같은 조건으로 사용한다. 이미 여백을 연 Design 및 반복 제목행/중첩/편집본은 기존 분기와 중복 계상하지 않는다.
- 이 두 보정만으로 #5877 14·18쪽의 종료 컷/병합 소유 문제가 해결된다고 주장하지 않는다. 다음 후보의 영향9·11·14·18쪽과 Design 전7쪽, 관련 의미/정상 대조군을 한 묶음으로 실행한다. #5877 whole31 Native/fresh WASM90% 선행 조건 전에는 그 기존 회귀를 변경하지 않는다.


### v118 Native 묶음 결과와 v119 물리 소유 분석

- Native immutable SHA `4273b8f0b134ef2174d2ddaded74d2c3e825575794202e62c820dc37a3090ff0`, build exit0/89.772초·source불변이다. Design 전7쪽은 exit0/63.624초·최저5쪽96.39917%이며 v117의 TSV를 유지했다. 관련 정식/의미/대조군은 신규 관계 검사까지 **164/164PASS**, exit0/296.360초다. 전체 Rust 회귀 통과는 아니다.
- #5877 영향4쪽은 exit1/232.987초·9쪽99.54633%, **11쪽98.68404%**, 14쪽80.64127%,18쪽82.75637%다. 직접 `native-ghost-direct-v118/ghost5877/review/review_011.png`·`review_014.png`·`review_018.png`를 대조했다. 11쪽 첫 미소비 행은47.6→66.1px로 복구하고 뒤 본문을 보존한다. 14쪽은 첫 행과 그 뒤 행의 높이 차이가 남으며18쪽은 첫 일반 행의 빈 꼬리와 병합 최소 높이 잔여가 어긋난다. 현재 후보를 전체 문서 시각 통과로 판정하지 않는다.
- 원본/Print와 실제 Page/Cell 출력으로 내용을 확인했다. 14쪽의 row18 col0 병합 내용은13쪽에 모두 존재하고14쪽은 비어 있다. 18쪽의 row76 col0 병합 셀은17쪽에 `11./소화/용수`,18쪽에 마지막 `설비`를 소유한다. 모든31쪽 텍스트 multiset은 v117/v118 동일하다. 따라서 일반 행의 완료와 병합 셀의 내용 완료를 분리해야 한다(`ghost-v118-geometry-preservation.json`).
- 다음 묶음은 본문 하단 상한을 **스캔 전에** 같은 원본 인증 결과에서 제한하고, 실제 일반 컷의 공동0→0 재시작/병합 소유자의 양수→0 저장 재시작을 대조해 해당 컷만 본문 물리 밴드를 소유하게 한다. 걸침 셀은 같은 `rowbreak_straddle_cut_units`의 실제 종료 유닛을 조회한다. 이미 완료한 걸침 내용은 컷의 미소비 소유자로 세지 않는다. 일반 행이 완료됐지만 병합 마지막 유닛이 다음 쪽에 남는 경우에도 실제 수용한 마지막 행 상자를 원장에 기록하고 다음 완전 행으로 전진시켜 병합 마지막 유닛을 보존한다.
- 경로는 `원본 host/common 본문 종료 인증 → budget 상한 → scan 컷/실제 마지막 행 높이 → accepted_row_boxes → continuation 실제 잔여 → 동일 measured row의 paint`다. 첫0→0 컷의 원본 행 최소 잔여는 유지하고, 완료된 일반 행은 이전 행의 빈 꼬리로 새 행을 대체하지 않는다. Design 3쪽의 단독 일반 셀 재시작·중첩/편집본은 본문 채우기의 독립 증거가 없으므로 기존 동작을 유지한다. #5877 whole31 Native/fresh WASM·관계 회귀 선행 조건은 계속 미완료다.


### v119 생산 결과와 v120 소비 연결 보정

- immutable Native SHA `3fdf5b456cfc96177557bbff67d15abb85a1e932481b6c88778c1bbaee5a8e9e`·build exit0/122.386초·source불변이다. Design 전7쪽은 exit0/67.384초로 min96.39917%를 유지했고, 관련 **164/164PASS** exit0/327.560초다.
- #5877 6쪽은9=99.54633,11=98.68404,13=94.76563,14=81.64369,17=96.17432,18=73.52430%다(exit1/304.749초). 이 후보는 통과가 아니다. 실제 스캔은13쪽 last27.027/17쪽 last58.493px의 본문 종료 상자를 생산했지만 후속 소비 연결은 미완료다.
- 14쪽 row23은 원본 높이7676HU/102.347px에서 앞 조각27.027px을 뺀75.320px을 소유해야 한다. 기존 `first_fragment_blank_band`가 일반 잔여 전달을 제외해 남은 내용71.947px만 그렸다. 반대로 row37 원본4682HU/62.427px은 다음 쪽의 내용 높이보다 작으므로, 동일 RowCut의 미소비 내용 요구를 잔여보다 큰 하한으로 유지해야 한다(`ghost-partial-minimum-source-v120.log`). 이 조건을 실제 첫0→0 컷에 한정해 명시적으로 전달한다.
- 18쪽 전환의 일반 종료 컷[1,1,1,1,3]은 모든 일반 셀을 끝냈지만 `StoredFragmentEnd.complete_cut`이 빈 내용 컷만 완료로 세어 실제58.493px 행 상자를 원장에 넣지 않았다. 걸침 유닛의 원본 양수→0 재시작과 독립 Print를 확인한 본문 종료 상자의 완료 행만 기록한다. `accepted_row_boxes → stored_fragment_row_plan`가 다음 완전 행의 최소/병합 종료 잔여를 다시 계산하고, 다음 쪽의 `설비` 유닛을 그대로 보존하도록 한다. 실제 소유 컷을 건너뛰거나 glyph만 숨기지 않는다.
- 다음 candidate는 이 두 소비 연결을 함께 보정하고 동일6쪽/Design7쪽/관련164개를 확인한다. 전체31쪽 Native/fresh WASM·정식 관계 회귀 선행 조건과 최종 lint/전체 회귀/PR 절차는 계속 남아 있다.


### v120 선행 소비 경계 관측과 v121 완료/정렬 계약

- Native SHA `dbdd47e9c5da5f40c4d9275c3232e9d84ec3b22daf0c9363db466f7cf910a40f`·build exit0/105.587초·source불변이며 CLI 전31쪽 진단은 exit0/5.791초다. 18쪽은 row77 원점115.3, row80 높이69.8로 복구하고 `설비`를 보존했다. 14쪽 row23은 여전히71.9px이며 가운데 정렬 한 줄이 셀 머리115.3px에 남는다. 이 후보는 시각 통과로 판정하지 않았고 비용이 큰 묶음 회귀는 아직 실행하지 않았다.
- `advance_row_cut(start=cut,budget=MAX).fully_consumed`는 **남은 유닛이 무제한 예산 안에 다 들어간다**는 뜻이지, 그 컷에서 유닛이 이미 없다는 뜻이 아니다. 기존 `complete_physical_end_cut`은 이를 혼동했고 v120의 미소비 잔여 전달을 건너뛰었다. 실제 컷 인덱스가 각 일반 소유자의 전체 유닛 수와 같은지 확인하는 완료 계약으로 바꾼다. 빈/개체 유닛도 인덱스로 검사해 `보이지 않음/높이0`을 완료의 대용으로 사용하지 않는다.
- 동일 원본의 `rowspan_declared_overflow_shrink`는 전150행0이다(`ghost-grid-shrink-source-v121.log`). 14쪽의 선언 높이 누락 원인은 선언을 줄이는 모델 보정이 아니라 위 완료 의미의 혼동이다. 원본 값에 대한 예외를 추가하지 않는다.
- 원본 본문 프레임·실제0→0 소유 컷·전달된 시작/끝 높이가 확인된 일반 행은 해당 물리 조각의 원본 `Center` 정렬도 소유한다. 이 같은 행 소유를 셀 composition window와 `effective_align` 소비 지점에 전달한다. 기준 Print 14쪽의 한 줄은 셀 중앙,13쪽의 첫 줄도 앞 조각 상자 중앙이다. 중첩/편집본·일반 양수 되감김의 단독 셀은 이 계약에 포함하지 않는다. 이 두 소비 경계를 한 번에 보정한 뒤6쪽/Design7쪽/관련164개를 묶어 확인한다.


### v121 선행 좌표·정렬 대조와 최종 경계 묶음

- v121 Native SHA `179a8c1fc88e10b33fa8e1b682cfff2e19a34bb7ba202a2dc0e86540e01eb850`·build exit0/86.323초, 전31쪽 CLI 진단 exit0/5.839초·source불변이다. 14쪽 row23 높이는75.3px, 선택한 한 줄의 원점146.9px로 복구했다. 독립 Print의 같은 글줄 bbox는147.371px이다. 18쪽 row80 높이69.8px과 마지막 `설비` 유닛도 보존했다.
- 독립 `pdftotext -bbox`를 원본 Print의13·14·17·18쪽에 실행했다(`ghost-independent-text-alignment-v121.json`, 각 html). Glyph bbox와 line box의 미세 메트릭 차이를 구분하며, Native14쪽 머리115.3px vs 독립147.371px의 큰 정렬 차이는 해소됐다.
- 17쪽의 실제 원본 병합 컷은 다음 쪽의 마지막 `설비`를 남기므로 기존 마지막 병합 조각 정렬 경로에 포함되지 않았다. 이 앞 조각도 확인된 본문 프레임과 양수→0 소유 컷을 공유한다. Native `11./소화/용수` 원점947.4/966.5/985.6px vs 독립949.921/969.100/988.119px의 차이를 확인했다. 같은 종료 행 인증을 병합 셀의 실제 종료 유닛 조회와 연결해 선택한 앞 조각만 원본 Center로 정렬한다. 다음 유닛을 앞 조각 구성에 포함하지 않고 마지막 조각의 기존 정렬을 보존한다.
- 선행 CLI 경계 진단 뒤 이 최종 정렬 연결까지 묶어 Native6쪽/Design7쪽/관련164개를 확인한다. 부분 점수 통과로 Ghost 전31쪽 또는 fresh WASM 완료를 주장하지 않는다.


### 원 PR 상태 재조회 (2026-10-11, v122)

- `gh pr list --repo edwardkim/rhwp --author semanticist21 --state all --limit100 --json ...`의 API 결과를 초기18개 번호에 대조했다(`semanticist21-pr-states-v122.json`). #7612는2026-10-08T21:00:28Z에 이미MERGED이며 나머지17개는OPEN/비draft다. 통합 scope는 초기18개 고유41개 contributor commit 그대로다.
- #7615의 현재 remote head는 `4a6c27c5ffa54e4988a7e0e7956115d933725aca`로 최초 선택 head와 다르다. 최신 head의 후속 의존 변경을 새로 적용하지 않으며, 원 PR의 최종 close 판정은 실제 통합한 원 commit 범위와 남은 최신 diff를 대조해야 한다. 이 조회는 remote 변경이나 추가 체리픽이 아니다.


### v122 최종 선행 묶음 통과·전쪽 검증 시작

- immutable Native SHA `0047b6492df08f132042aa93b58096acc382fa852eb0f60481c802b2295df0c7`·build exit0/84.616초·production불변이다. 공식 관련/정상 대조군 **164/164PASS**, exit0/397.314초이며 전제 base/생성 suite를 바꾸지 않았다. Design 전7쪽은 exit0/67.132초·최저5쪽96.39917%·미달/누락0으로 v117 점수를 유지한다.
- #5877 영향6쪽은 exit0/279.171초·9=99.54633%,11=98.68404%,13=95.02698%,14=96.93008%,17=96.15809%,18=93.45386%이며 전부90% 이상이다. 현재 Native13/14/17/18쪽 `native-ghost-direct-v122/ghost5877/review/`를 직접 표시/판독했다. 상단·하단 외곽과 다음 행 위치,14쪽 한 줄의 Center,17쪽 앞 세 유닛과18쪽 마지막 `설비`/뒤 내용의 보존을 대조했다. 괘선 명도/글리프 메트릭과 미세한 내부 열/줄 차이는 남고 기준 PDF와 픽셀 동일하다고 주장하지 않는다.
- actual CLI 전31쪽은 source불변/exit0이며 모든4570 텍스트 노드 multiset이 v117과 동일하다(`ghost-content-preservation-v122.json`). 새 유닛 소유·스캔/예약/paint 연결의 결과이며 독립 Print의 시각 통과를 대신하지 않는다.
- ignored `issue_5877_source_frame_relations_candidate_v122.rs`의3개 관계 후보는 같은 case/원본을 frozen 수정 전 v113 CLI / 현재 v122 CLI에 실행해 **3FAIL→3PASS**를 확인했다(`ghost-source-frame-before-after-v122.json`, 각 compile/test log). 본문 내 같은 문단의 두 표, 새 완전 행 vs 이전 잔여, 선택한 한 줄의 가운데 포함·병합 유닛의 쪽 소속·뒤 종료 행의 상대 높이를 검사한다. 절대 위치/고정 영역/해시 pin을 기대값으로 쓰지 않는다. 아직 정식 regression source·fixture·golden을 등록하거나 기존 #5877 px 검사를 변경하지 않았다.
- 같은 production을 고정해 Native 전31쪽 비교와 표준 fresh WASM build를 시작했다. 현재 source hash와 이미 성공한6쪽만으로 전31쪽/fresh WASM90%를 단정하지 않는다. source 파일을 변경하지 않은 채 결과를 기다리고, 필요한 대표/미달 쪽 PNG를 대조한 뒤 관계 회귀를 등록한다. 최종 lint·전체Rust·실제 편집 검증·초기18개 개별 기록/오늘할일/통합PR/CI/merge/후속 처리는 계속 남는다.

## 104. v122 전쪽 결과와 v123 물리 행 누적 보정 근거

- Native31쪽은 exit1/1071.363초·누락0·미달1이며15쪽79.49738%가 새로 검출됐다. 나머지30쪽은90% 이상이다. 영향6쪽 통과를 전쪽 통과로 승격하지 않는다. `native-ghost-whole-v122/ghost5877/silhouette.tsv`와 새 `native-ghost-direct-whole-v122/ghost5877/review/review_015.png`를 직접 표시/판독했다.
- fresh WASM 빌드는exit0/1128.892초·production 불변이며 frozen package/root pkg/Studio public JS·WASM의 SHA-256이 모두 같다. 실패15쪽을 fresh WASM으로 재현했고exit1/154.077초·79.49738%·Native와PNG 바이트 동일이다. Native에서 전쪽 gate가 이미 실패했으므로 같은 후보의WASM31쪽을 반복하지 않고 원인 수정으로 진행한다. v122의fresh 전쪽 통과나Studio 실제 동작 검증을 주장하지 않는다.
- 독립 Print15쪽의 `pdftocairo -svg` 괘선(`ghost-independent-p15-grid-v122.json`)은 시작115.234px, 첫 행 끝167.974px, 병합 종료 행 끝237.021px이다. 원본34행 병합 높이24522HU=326.960px인데 앞 조각37행 실제21.387px과 뒤 조각52.853px을 함께 차감하지 않아 종료38행에 기존 전역 잔여71.333px을 재사용했다. 실제 합74.240px을 쓰면 종료 요구는69.040px이며 독립 Print의69.047px에 대응한다.
- 생산/소비 연결: `emit.rs`가 확정한 원본 재시작 컷의 실제 마지막 상자 → continuation cursor의 아직 완료되지 않은 같은 행 누적 → `physical_rows.rs`의 이어받기 첫 상자와 이후 병합 종료 행 요구 → 동일 fragment_row_heights를 소비하는 scan/측정/paint. 미완료 컷은 완료 행 장부와 분리하며 큰 예산에 들어가는지로 완료를 추정하지 않는다. 신규 회귀 추가 및기존 #5877 픽셀 검사 변경은 Native/fresh 전31쪽90%와직접 판독 전까지 보류한다.
- v123 첫 후보는 양수 reset의 공동 컷 계약과 달리 첫0→0 뒤 정상 저장 줄 사다리로 입증된 재시작에서 미소비 소유자 수가1이어도 프레임을 보존하도록 수정했다. 동시에 넣은 동일 문단 제한은 아래 실행에서 원본 문단 간 전역 사다리를 거부한 것으로 확인되어 v124에서 제거했다. 특정 쪽이나 문서 ID로 분기하지 않으며 코드 변경 뒤에는 v122 결과를 현재 결과로 재사용하지 않는다.
- v123 첫 빌드의 새 StoredFragmentEnd 초기자 누락은 보완해 재빌드exit0/111.032초로 해소했다. 이어받기 트리는exit0/5.682초지만 같은 문단 여부로0→0을 제한한 새 가정 때문에 원본의 문단 간 셀 전역 저장 재시작도 거부해14쪽 잔여가71.9px로 되돌아갔다. 이 가정을 제거한 v124로 재검증한다. 앞 행의 실제 유닛이 두 번째 원본 저장 줄을 소유하는지와 뒤 양수 사다리/원본 본문 프레임 증거는 유지하며 문단 인덱스 동일성으로 쪽을 추정하지 않는다. v123 트리나빌드 성공을 시각 성공으로 보고하지 않는다.
- v124 Native 빌드는exit0/47.555초·production 불변이며CLI SHA256 `2f725838570d232614f5cd6f2a4970ca5c3561bd979ad04d3e5b06439d41ec8c`다. 실제31쪽 트리에서14쪽 첫 잔여75.3px을 유지하고15쪽38행은69.0px,34행 병합 앞/뒤 상자205.1+121.9px로 원본24522HU 최소 높이를 보존한다.15쪽 마지막49행은27.1px로 실제 저장 재시작 프레임을 수용한다. 아직 점수·시각 통과의 대용으로 쓰지 않는다.
- 같은최종 관계 후보4개(`issue_5877_source_frame_relations_candidate_v123.rs`)를 frozen CLI로 실행했다. v113은3FAIL/1PASS, v122는이번 누적 검사1FAIL/3PASS, v124는4PASS이며모든compile exit0·run101/101/0이다(`ghost-source-frame-four-before-after-v124.json`). 이는 ignored 진단 후보이고 전쪽 Native/fresh gate 전에는 정식tests/cases로 추가하지 않는다.
- 설계평가기준 Native7쪽은exit0/54.673초·최저5쪽96.39917%·미달/누락0이며 기존v117과TSV 값이 같다. 현재production의fresh WASM7쪽은 별도 실행할 예정이다.
- 종료 증적을 확인한 이전 #5877 SVG106개를 삭제해5.099GiB를 확보했다(`completed-ghost-svg-cleanup-v124.json`). 실패15쪽 조사용SVG3개와 PNG/TSV/원본/Print/실행기록은 보존했고 압축하지 않았다.
- v124 영향8쪽 Native는exit0/209.368초·9=99.54633%,11=98.68404%,13=95.02698%,14=96.93008%,15=96.15582%,16=95.99031%,17=96.15809%,18=93.45386%·미달/누락0이다. 새 `native-ghost-direct-v124/ghost5877/review/review_015.png`와`review_016.png`를 실제 표시/판독해 병합 종료 높이와 뒤 내용·하단 프레임을 대조했다. 나머지6쪽도현재재출력PNG가v122와바이트동일이다. 괘선 명도·미세한 열/글리프 차이는 남으며픽셀 동일을 주장하지 않는다. Native 전체31쪽과현재fresh WASM31/설계7쪽이 아직 남아 있다.
- 기존 #5877 고정 x/y 검사 대신 실제 첫 밴드의 셀 경계와 출력 괘선의 대응을 검사하는 ignored 관계 후보는1PASS/exit0/6.462초다(`ghost-column-grid-relations-v124-exit.json`). 원본 지오메트리를 고정 숫자로 복제하지 않으며전쪽 gate 전에는 기존 정식 검사를변경하지 않는다. Native Clippy `-- -D warnings`도exit0/97.806초·production 불변이다. WASM32/워크스페이스lint 및최종전체Rust는별도필수검증으로남는다.
- Native31쪽과새WASM 빌드를 시작했고현재production을v124 SHA inventory로 고정했다. 진행중작업의로그/TSV/PNG는새v124경로에보존하며과거전쪽을현재통과로쓰지않는다.
- v124 원본 내용 보존: 31쪽 전쪽 트리의 각 쪽 텍스트 multiset4570개가v122와 모두동일하며 트리 변경은15/16쪽뿐이다(`ghost-content-preservation-v124.json`). 실제 입력/독립Print도수정하지않았다. 내용 보존은 시각 gate를 대체하지 않는다.

## 105. 고용지원기준 기존 회귀의 원본 진단 — v124 읽기 전용

- 실패 #6950의 실제 입력은 `samples/issue6025/3232693_employment_support_criteria.hwpx`이며 같은파일의다른 합성 검사가읽는20260909-para-table과구분했다. 고정base21ed Native4쪽은86.51725/81.14196/93.69061/95.86258%·exit1,현재v124는89.49447/81.14196/93.69061/95.86258%·exit1/48.515초다. 기준에서도미달인원본이지만동일 #7445 이관을 확인하지못해기존검사를유지했다. 현재1/2쪽 review PNG를 실제 표시/판독했다(`native-employment-direct-v124/employment6025/review/`).
- 원본ZIPXML(`employment-source-cuts-v124.json`),독립Print벡터/텍스트(`employment-independent-p1/p2-v124.svg/html`),실제경로trace(`employment-source-frame-trace-v124`·exit0/0.617초)를대조했다. 첫표common69352HU=924.693px,7행16170HU=215.6px·CENTER·10800→0뒤1600원본사다리다. Print7행앞175.489px+뒤39.797px=215.286px인데현재앞161.1px+뒤38.4px이며앞글은911.7px대Print918.625px이다. 현재다음쪽원점75.6px대Print77.354px이다.
- 예산은926.6px라첫저장924.693px이들어가지만trace에서source_frame=None이다. budget.rs는글줄저장원점과preemit뒤문단원점을비교하고,stored_frame_budget.rs는일반미완료컷의물리확장을거부한다. 문단앞간격20px과실제해석된host_placement의좌표계를추적한뒤같은원점/요구/수용/정렬결과로고치는것이원인후보다(`employment-shared-first-frame-diagnosis-v124.json`). 부분컷의미소비내용과빈띠는분리하고다음최소상자는실제앞상자를차감한뒤남은내용요구를하한으로삼는다. 본문/다음조각위여백도예산과paint가같은술어를써야한다.
- #5877 검증 source는v124로고정중이므로이진단에서production을수정하지않았다. 고용지원기준의원본/기존테스트를삭제하거나회귀기대값을완화하지않으며먼저같은원본Native/fresh4쪽을90%이상으로만든뒤독립관계로검사를변경한다.
- GitHub 현재 #7445 직계 하위 이슈79개의본문/제목을읽기전용으로재조회했고3232693/employment/6950/취업취약/고용지원/취업지원서비스/수급요건의직접일치는없었다(`issue7445-subissues-employment-check-v124.json`·`issue7445-employment-check-v124-result.json`). 하위이슈코멘트는전수검색하지않았으므로동일이관이없다는확정판정은하지않는다. 기존검사를삭제할근거로쓰지않는다. 현재79개와기존77개스냅샷의차이는 #7803/#7804이며이번읽기에서추가수정/삭제하지않았다.

## 106. v124 Native 전쪽 통과와 fresh WASM 비교 시작

- Native31쪽은exit0/1075.438초·production 불변·최저18쪽93.45386%·미달/누락0이다(`native-ghost-whole-v124/ghost5877/silhouette.tsv`). 동일원본과31쪽독립Print를끝까지비교했다. 앞서직접표시한현재영향쪽review와연결하며Native전쪽을WASM통과로대신하지않는다.
- fresh WASM build는exit0/1116.358초·production 불변이다. frozen/root pkg/Studio public의JS SHA256은모두4628f097cddafb073362cc2ff03d19744de3e037aa62d311b4bd4386528c7a20,WASM은모두bb879fa64ea41da9521f2f1c9a00eb94b4b52b6c06c3cbf710988029532ad063이다. Studio실제브라우저동작이나WASM전쪽gate통과의대용이아니다.
- 같은frozen package로fresh WASM #5877 전31쪽과설계평가기준전7쪽비교를시작했다. 신규관계4검사추가/기존격자픽셀검사관계변경은이최종전쪽gate와직접PNG확인뒤에수행한다. Native Clippy완료뒤현재WASM32libClippy를순차검증하며Cargo동시실행은없다.

## 107. v124 fresh 전쪽 완료와 관계 회귀 등록

- #5877 fresh WASM31쪽은exit0/1062.349초·production 불변·최저18쪽93.45386%·미달/누락0이다. Native전31쪽과TSV/전31PNG바이트가동일하다(`ghost-native-fresh-wasm-whole-v124-parity.json`). 현재fresh15/18쪽 review PNG를실제표시/판독했고17쪽도생성했다(`wasm-ghost-direct-v124/ghost5877/review/`).15쪽병합종료와뒤행,18쪽마지막병합유닛과후속내용소속을대조했으며미세한선/글리프차이는남는다.
- Native/fresh전쪽선행조건확인뒤 `tests/cases/issue_5877_source_frame_relations.rs`에ignored진단의동일4검사를등록했다. v113의3FAIL/v122의추가1FAIL/v124의4PASS를검출증거로연결한다. 기존 `issue_5877_fragment_ghost_vrules.rs`의고정GHOST_X/REAL_X/y상한은실제첫밴드의셀경계와그밴드를가로지르는괘선의쌍방향대응으로교체했다. 격자검사이전오류의수정전검출을이번4검사로대신주장하지않는다.
- 설계평가기준fresh7쪽은exit0/60.705초·최저5쪽96.39917%·미달/누락0이며Native7쪽과TSV/전7PNG바이트동일이다. 현재fresh2/3쪽review도실제표시/판독했다. `issue_5885_nested_host_row_ladder_end.rs`의고정위치/선택밴드/마지막본문y값을원본행소유·실제부모셀의두중첩표포함·다음행접속·선택글줄상자+원본안여백의첫종료·예약/paint공통끝·실제Body포함으로변경했다. 원본안여백값은컷helper반환값으로재계산하지않는다. 같은최종원본의직접컴파일/2검사는exit0/9.727초이다. 최초진단compile의Body구조패턴과standalone serde중복타입문제는보완했고그실패를결함재현으로세지않았다.
- WASM32libClippy는exit0/89.201초·production불변이다. Native/WASM32Clippy성공만으로workspaceall-targets/최종전체Rust/통합PR승인을선언하지않는다. 이제최종정식suite·필수lint·고정base정책검사를검증한뒤보정단계를커밋한다.
- 최종 전체 Rust 검증 시작: `cargo nextest run --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast`를현재production/최종정식검사로시작했다(`whole-default-rust-source-frame-v124-command.json`,timeout5400초). 기존v103의27실패목록을현재결과로재사용하지않고종료summary를확인한다. Cargo/Rust실행중에는production/검사/파생suite를변경하지않는다.

- 같은최종source의고정base21ed manifest정책은exit0/6.361초,unit-tier정책은exit0/32.386초다.59개시험지/HML원본은v112SHA와전부동일하며삭제하지않았다(`exam-originals-preserved-v124.json`).현재Rust AST감사에서직접원본참조와같은파일helper/상위상수의전이참조후보는0개다(`exam-regression-consumer-audit-v124.json`).이감사는동적전수탐색의모든경로를입증하지않으므로기존전수탐색제외정책과구분한다.전체nextest는진행중이며대형#2063은444.937초뒤PASS,기존격자1/설계관계2도현재정식suite에서PASS다.현재fresh#5877최저18쪽review를다시표시/판독했고하단선/글리프의미세차이도남는다고기록한다.

- 종료exit0을확인한현재Native/fresh31쪽비교의SVG124개를정확한두output경로에서삭제해4.616GiB를확보했다(`completed-whole-ghost-svg-cleanup-v124.json`).압축하지않으며전쪽PNG/TSV/실행기록/대표직접review/원본/Print는보존한다.공유Cargo산출물은이정리대상이아니다.


## 108. v124 전체 Rust 종료 — 남은 17실패 유지

- 정식 전체 nextest는 exit100/1509.650초이며 production과 최종 검사는 불변이다. `Summary [ 966.843s] 10344 tests run: 10327 passed (13 slow), 17 failed, 45 skipped`. `whole-default-rust-terminal-failures-v124.json`에 모든 실패명, 현재 #5877·#5885 검사 8개의 실제 PASS, 실행 명령·exit·log를 연결했다. 전체 실행을 통과로 보고하지 않는다. v103의 27개와 비교하면 같은 검사명으로 현재 PASS인 6개와 새 실패명 0개다. 과거 실패명 4개는 현재 실행 목록에 없으므로 통과로 세지 않는다. v103은 원본 복구 시점도 달랐으므로 이 이름 비교를 동일 입력의 수정 전후 검출 증거로 승격하지 않는다. 파싱은 Summary 뒤 반복 실패 목록을 제외해 10,344개의 고유 실행을 확인했다.
- 남은 실패: overflow-cell partition1/2/7/8/9, #6860 수동 대조군, #4882 HWPX 왕복, #6950 고용지원기준, #5966 각주 fresh page, #7009 편람 2개, oracle 쪽수 partition0, #1891 외부 링크, #1156 KPS, #3820 저장 쪽 꼬리, #6718 되감김, #1133 쪽번호 각주다. 새 관계 검사를 위해 기존 실패를 삭제하거나 완화하지 않았으며 원본 Native/fresh 90%와 같은 #7445 이관 여부를 각각 판정한다.
- 전체 Rust 프로세스 종료를 확인한 뒤 같은 source/검사에서 fmt/workspace build/all-target Clippy를 순차 실행한다. 현재 단계 커밋은 보정 범위와 남은 실패를 함께 고정하는 체크포인트이며 통합 PR 준비 완료가 아니다.

- 최종 lint: fmt 실행 27.196초·fmt check 26.554초, workspace build 168.070초, workspace all-target Clippy 125.610초가 모두 exit0이며 production/정식 case는 불변이다. Native Clippy 97.806초·WASM32 lib Clippy 89.201초와 고정 base21ed의 두 정책 검사를 함께 `source-frame-checkpoint-gates-v124.json`에 각 exit 기록으로 연결했다.
- 실제 보정 원본 2개와 독립 Print PDF 2개는 HEAD의 커밋된 blob SHA256과 모두 일치한다(`source-frame-original-print-commit-verification-v124.json`). CGMP 기존 관계 검사 5개도 이번 전체 정식 실행에서 모두 PASS다. 새 CGMP 전쪽 검증으로 확대하지 않는다. 초기18개 최종 전체 gate, 실제 편집/Studio, 남은 실패 원본의 보정과 Native/fresh 시각 증거, 개별 PR 기록/오늘할일/통합 PR/CI/merge/후속 처리는 계속 남아 있다.


## 109. 고용지원 평가기준 원본 프레임 보정 계획 — v125

- 직전 보정 단계는 `54f93fea0e24688503a8f59adf44400735559f96`에 커밋했다. fetch한 upstream/devel과 local devel은 모두 `21ed64071f81f35f37618717693bd30f727392c9`이며 변동이 없어 rebase하지 않았다. 같은 커밋의 원본/검사 bytes가 v124 검증 inventory와 일치하고 작업 트리는 깨끗했다(`source-frame-checkpoint-fetch-state-v124.json`).
- 원본은 `samples/issue6025/3232693_employment_support_criteria.hwpx`와 기존 독립 Hancom Print 4쪽이다. v124 Native 1/2쪽은 89.49447/81.14196%이며 기존 #6950 검사를 유지한다. 원본의 첫 조각 높이 69352HU, 7행 최소 높이 16170HU, 실제 저장 컷 10800→0과 뒤 정상 줄 사다리, Print의 앞/뒤 행 상자를 근거로 삼는다. 원본은 수동 수정한 합성 입력이 아니다.
- 호출 경로: `block/entry.rs`의 원본/계산 호스트 앵커 → `block/prepare.rs`의 `for_first_fragment` 및 실제 배치 계획 → `fragment/budget.rs`의 저장 첫 상자 인증·행 예산 → `stored_frame_budget.rs`의 선택한 컷과 실제 마지막 행 상자 → `emit.rs`의 점유/원본 최소 잔여 → 다음 `FragmentStart` 예산과 `table_partial.rs`의 전달된 높이·정렬. 제목 선방출 뒤 문단 시작을 저장 가시 줄 앵커와 비교하는 별도 추정은 실제 공유 배치 앵커로 대체한다.
- 선택한 내용 컷을 그대로 보존하고 인증된 첫 물리 상자의 빈 부분을 계상한다. 미완료 일반 행의 저장 경계를 확인하며 병합 블록의 기존 소유 계약은 유지한다. 실제 첫 상자를 수용한 사실을 커서에 전달해 다음 쪽의 위여백을 예산과 배치가 한 번만 소비하게 한다. 다른 여백 경로와 중복하지 않는다.
- `advance_row_cut(..., MAX).fully_consumed`는 남은 내용이 무한 예산에 들어가는지를 뜻하며 현재 컷이 이미 끝났다는 증거가 아니다. 일반 행의 완료/정렬과 잔여 요구에서 이 가정을 실제 소유 유닛의 끝과 대조한다. 원점0이라는 숫자만으로 쪽을 추정하지 않고 실제 원본 컷/줄 사다리를 사용한다.
- 첫 실행은 묶음 수정의 Native 4쪽과 실제 트리/trace, #5877·설계·CGMP 정상 대조군이다. 원본 전4쪽 Native/fresh WASM 90%와 직접 review 판독 전에는 #6950 기대값이나 정식 회귀를 변경하지 않는다. 생산 코드에 문서/쪽 번호나 절대 좌표 예외를 넣지 않는다. 이후 필수 lint/전체 Rust/초기18개 최종 검증과 원격 lifecycle은 그대로 남는다.

- v125 Native 빌드 exit0/118.502초·source 불변, CLI SHA `a5c874b2a15a9c2e3ed5d349f4d25257e576e361f5e9c81cc8e828ccb79df4a1`이다. 실제 trace exit0/1.077초에서 source_anchor=45.6, placement.anchor_y=45.6, flow_host=25.6을 확인했다. 첫 저장 프레임 인증은 복구됐으나 nearest 완전 행 끝이 None이라 기존 전체 행 신뢰 조건이 행 내부 조각 처리를 막았다. Native 전4쪽은 v124와 같은 89.49447/81.14196/93.69061/95.86258%·exit1/72.893초이므로 개선 완료로 세지 않는다(`employment-source-frame-first-candidate-v125.json`). 대조군 Cargo가 종료되면 잘못된 전체 행 조건을 부분 예산과 분리한다.

- v125 집중 검증은 exit100/444.235초·실행114.527초, 204개 중 200PASS/기존2FAIL/새2FAIL이었다. 기존 #6860 수동 대조군과 #6950 원본 px 기대값은 유지했으며 새 body overflow partition8/12에서 #5875 중첩 표 캡션·#5782 프레스·#6551 HWPX의 넘침이 검출됐다. 같은 frozen v124/v125 CLI의 #5875 trace에서 첫 조각/예산은 같지만 다음 한 조각이 661.7px에서1060.5px로 늘어 가용971.9px를 넘는 것을 확인했다. 남은 전체 내용 높이를 현재 한 조각의 하한으로 사용한 제 오류를 고치며 baseline이나 원본을 변경하지 않는다.
- v126은 원본 텍스트 프레임이 확인된 `stored_host_origin`을 앵커 비교에 사용한다. opaque 개체 앵커와 가시 텍스트 앵커를 같은 좌표로 추정하지 않는다. 행 내부의 엄격한 저장 프레임 예산은 완전 행 끝을 필요로 하지 않으며, 완전 행 압축의 기존 신뢰 조건은 유지한다. 새 일반 부분 컷의 물리 앞 행 합은 실제 paint도 소비하는 측정 상자와 소유 술어에서 얻고, 내용 스캔의 전진/패딩을 다시 더하지 않는다. 원시 잔여 fallback은 현재 한 조각의 물리 하한으로 유지하며 다음 스캔이 실제 내용을 선택한다. 유효한 일반 원본 컷의 완료 여부는 정확한 소유 유닛 끝으로 확인한다.

- v126 Native build exit0/96.076초·source 불변, CLI SHA `19c56bc391c603598e1cd8914457bf9e53d568a2b62967b2b42542e4abb4ba7b`다. 원본 전4쪽 Native는 exit0/70.567초·92.57173/95.02929/97.44742/98.76769%·미달/누락0이다. 첫7행 앞175.7px+뒤39.9px로 원본215.6px를 보존했고 둘째쪽 원점77.5px·다음8행117.4px로 Print77.354/117.151px와 대응한다. 이 Native 결과를 fresh 또는 전체 대조군 성공으로 확대하지 않는다.
- #5875 cold frozen CLI의 v126 대조군은 여전히 넘침을 보였다. 한 칸의 일반 저장 리셋을 같은 행의 중첩 개체들 전체 소유의 증거로 삼은 것이 원인이다. v127에서는 실제 활성 소유자 전부가 원본 일반 글줄일 때만 새 일반 부분 프레임 계약을 적용한다. 중첩 개체는 기존 실제 블록/개체 컷의 별도 물리 상자를 유지한다. 글줄 하나의 좌표만으로 중첩 개체의 소유를 추정하지 않는다. 별도 문서 ID/쪽/절대 좌표 분기는 없다. v126 Native 적격성이 있어도 새 코드의 재출력과 대조군·fresh가 끝나기 전 기존 #6950 검사를 변경하지 않는다.

- v127 Native build exit0/79.941초·source 불변, CLI SHA `367c39a497d883caa1cba8b2818bc61f4708021f3f80e80cc115e5872ee6af33`다. #5875 원본12쪽 RenderTree가 v124와 전쪽 byte 동일하여 제가 만든 큰 넘침 경로가 해소되는 방향을 확인했고, 고용지원 원본4쪽 트리는 v126과 전쪽 동일하다. 이것을 아직 정식 body overflow 검사 PASS나 시각 전체 승인으로 세지 않는다.
- v126 Native 1/2쪽 review PNG를 생성·표시·직접 판독했다(`native-employment-direct-v126/employment6025/review/`). 2쪽 마지막 일반 행과 하단 괘선은 95.02929%이어도 Print보다 짧아 보정 대상으로 유지한다. 3쪽도 마지막 컷의 본문 점유를 확인한다. 프레임 상한만 숫자로 통과시키지 않는다.
- v128은 실제 첫 상자의 확정 원점+물리 높이+원본 바깥 아래 여백이 본문 아래와 저장 HU 반올림 범위에서 닫힌 사실을 커서에 전달한다. 이후 같은 상한에서 먼저 컷을 선택하고, 미완료 원본 일반 글줄 컷의 실제 되감김이 확인된 조각만 같은 본문 아래까지 점유한다. 빈 띠를 내용 컷으로 소비하지 않는다. 시작 조각은 이전 emit의 실제 물리 높이를, 완전 중간 행은 측정/paint의 같은 상자를 사용하며 새 종료 조각의 요구와 점유를 같은 결과로 전달한다. 끝까지 소비한 마지막 조각, 개체/병합 블록, 본문 구역/각주 예산이 맞지 않는 경우는 이 계약으로 채우지 않는다. 캡션이 있는 상자는 별도 원점 계약이므로 새 본문 종료 계약을 적용하지 않는다. 양수 원점200의 원본 재시작도 좌표를0으로 바꾸지 않고 실제 앞/뒤 저장 글줄과 소유 유닛으로 확인한다.

- v128 Native build exit0/106.498초·source 불변, CLI SHA `cff9c935ae1079671b0259e9129e4359934d3ee88a00e7185c6c67383a570a1c`다. Native 전4쪽 exit0/65.082초·92.57173/98.40602/95.83454/98.76769%·미달/누락0이다. 현재2/3쪽 review를 실제 표시/판독하여2쪽 마지막 칸과 하단 괘선,3쪽 하단 종료/다음쪽 내용 소속을 확인했다.3쪽 점수의 이전97.44742% 대비 하락은 물리 상자와 마지막 칸 정렬을 바꾼 결과이며 숨기지 않는다. 경계 괘선은 Print와 대응하고 glyph/미세 간격 차이는 남는다. #5875 전12쪽 트리는 v124와 계속 byte 동일이다.
- 코드 소비를 대조해 양수200의 실제 문단 간 저장 재시작은 기존 CellUnit의 `hard_break_before`와 `row_cut_ends_at_original_plain_text_reset`에 이미 표현됨을 확인했다(`table_layout.rs` reset_before 생산→CellUnit→동일 일반 프레임 소유 검사). v129은 새 좌표 되감김 helper를 추가로 해석하지 않고 이 기존 공통 컷 계약을 그대로 사용한다. 또한 일반 물리 프레임의 내용/잔여/정렬에 붙어 있던 repeat_header 요구를 제거한다. 실제 헤더가 없는 현재 문서에서도 그 flag는1이지만, 일반 행 상자의 소유는 원본 부분 프레임·소유 유닛/저장 컷에서 결정되며 제목 반복 flag로 달라지지 않는다. 모든 일반 frame helper가 같은 원본 셀 소유 계약을 사용하도록 하고 편집/합성/중첩/병합 배제 근거는 그대로 유지한다. 제목 반복 동작의 기존 호출 조건을 바꾸지 않는다.


- v129 Native 전4쪽은 exit0·92.57173/98.40602/95.83454/98.76769%·미달/누락0이며 현재2/3쪽 review PNG를 새로 생성·실제 표시/판독했다(`native-employment-direct-v129/employment6025/review/`). 해당 결과는 Native만의 검증이다. 관련 정식 검사204개는201 PASS/3 FAIL/6946 skipped·exit100/395.619초다. 기존 #6860 합성 경계와 #6950 절대 오프셋 외에 body overflow partition8의 #6551 평가요령이1→2건으로 증가했다. partition12와 #5875 대조군은 통과했으나 전체 무회귀로 확대하지 않는다.
- #6551 frozen v124/v129 실제 전46쪽 트리를 비교해3/4쪽의5행 이어받기를 추적했다. v129 스캔은1278px 이상의 이전 물리 잔여를 현재952.5px 예산 확인 없이 통째로 수용하여 다음6행까지 배치했고 실제 표가 본문427px 아래로 넘쳤다. 원인은 `scan/runner.rs`의 시작 행 override 무조건 전진이다. v130은 물리 잔여도 현재 예산에 들어갈 때만 통째로 소비하고, 큰 잔여는 기존 내용 컷 스캔으로 나눈다. 일반 원본 잔여는 커서의 이전 실제 물리 높이에서 현재 컷의 패딩 포함 점유를 차감해 넘긴다. 원본 전체 높이로 되돌아가 이미 소비한 공간을 다시 계상하지 않는다. `원본 컷/물리 잔여 → 행 스캔 예산 → 수용 컷/높이 → emit 잔여 차감 → table_partial의 시작/끝 행 높이` 경로를 함께 수정하며 baseline·fixture를 바꾸지 않았다. Native build 진행 중이며 새 코드의 회귀/fresh 결과는 아직 미검증이다.


- v130 Native build exit0/105.548초·source 불변·CLI SHA `108facbb25e2c16d435dcc14b72c72759c3d6df9ea85d5d5054a1e9630695956`다. 고용지원 전4쪽은 exit0/69.849초·92.57173/98.40602/95.83454/98.76769%이고 실제 트리도 v129와 전쪽 동일하다. #6551 전46쪽 트리에서3쪽 표 외곽 높이만 달라졌으며4쪽 이후는 v124와 동일하다. 영향3/4쪽 Print 비교 exit0/32.417초·92.33164/94.60777%이며3쪽 review를 실제 표시/판독했다. 정식 관련204검사 exit100/300.444초·202 PASS/2 FAIL/6946 skipped로 #6551 증가와 partition8이 해소됐다. 남은 #6860/#6950은 기존 두 실패이며 전체 최종 승인/회귀 무실패가 아니다. #6551 제목 검사의 필터를 덧붙였으나 그 source는 suite010이고 이번 명령의14 suite에 포함되지 않아 별도 PASS로 세지 않는다.
- v130 소비 대조에서 다시 자른 시작 행은 스캔에서936.7px를 수용했지만 paint가 전체 start override를 복원한 뒤 페이지 상한에서947.4px로 접는 경로가 남았다. v131은 재조판 경로가 이미 쓰는 동일한 `end_row_height_override=consumed` 생산을 실제 원본 일반 글줄 소유자에도 적용한다. 현재 컷의 수용 상자가 시작 override보다 뒤에서 최종 행 상자를 정하며, 큰 원본 잔여를 그린 뒤 접어 숨기지 않는다. 새 조건은 실제 일반 셀 소유·컷 공간에서 판정하고 병합/개체 블록을 제외한다. 이 변경 뒤 Native/fresh/정식 회귀는 재확인하며 #6551 기존 제목 검사 suite010도 실제로 포함한다. 아직 새 렌더링 테스트/fixture는 추가·변경하지 않았다.


- v131 Native build exit0/85.856초·source 불변·CLI SHA `ebda727a48394f8720cf7edc4a973585f0fc685c5066b9c48a573dff82721de7`다. 고용지원 원본 전4쪽은 exit0/72.287초·92.57173/98.40602/95.83454/98.76769%·미달/누락0이고 v130과 트리 동일하다. #6551 영향3/4쪽은 exit0/46.645초·92.30181/94.60777%이며 현재3쪽 review를 실제 표시/판독했다. 큰 시작 잔여를 복원한 뒤 페이지 상한에서 접던 경로 대신 실제 수용936.7px를 마지막 행 override로 전달해 예약/실제 상자가 일치한다. 전체46쪽 중 그3쪽 외에는 v130과 동일하다. 기존 제목·단 상단 검사도 suite010에 실제 포함해 PASS를 확인했다.
- 관련 정식205개는203 PASS/기존2 FAIL/7156 skipped·exit100/317.716초다. 도로조명 한컴 재저장 원본2쪽 Native는93.46641/99.85838%·exit0/145.796초, 설계 원본7쪽 Native 최저5쪽96.39917%·exit0/159.637초·미달/누락0이다. #5877 전31쪽 Native와 fresh WASM build가 진행 중이며 새 source의 전체 시각 승인으로 확대하지 않는다.
- #6860 수동 대조군은 원본 XML의 vertpos=0을1로54개 치환한 입력이다. 한컴 Print 직접 인쇄로 그 기대를 대조한다. 첫 Python 생성 대조 ZIP SHA `412793cc8e83c89750d467c0597ebd79e72ccc91aa035fa1d4f64992ec88f59f`, job `08c8ef53-2fea-4313-aaa8-47e5d6fffba5`, engine2020·실제11.0.0.9136·PrintMethod0·hancom2020_pdf_driver_one_up, PDF SHA `5dbd70a4e4ec3ec8e5d892821b2521e59ba8083eabef72f0833b5c494306a984`·2쪽이다. Print1쪽에 첫 줄 `Tilt=<filename> 또는 INCLUDE 또는 NONE`이 남고2쪽에는 동일 전체 줄이 없다. 기존 수동 검사의 “양수이면 첫 줄을 다음쪽으로 이월” 가정과 실제 Print가 다르다. archive 생성 메타데이터를 포함해 정식 Rust writer의 동일 입력을 확보해 검증하고 Native/fresh가 통과한 뒤 검사·계약 설명을 수정한다. 수동 메타데이터만으로 새 수용 조건을 완화하지 않는다.


- #6860 기존 검사와 같은 zip crate·Rust writer 루프에서 생성한 입력은 SHA `9a067f15b71250c36b45c07be826fb7dc859d2b400504994956d6b40fb815804`다. Python 대조와 모든 archive entry 내용 byte가 동일하나 ZIP 메타데이터까지 같은 입력을 보장하려고 다시 Print했다. job `c5ced948-038e-4330-bb4f-9a4331df31ae`·engine2020·실제11.0.0.9136·PrintMethod0·hancom2020_pdf_driver_one_up·PDF SHA `a63ea7029cc7fd946f5a24a86f6d8189d4ca0b367ba731826c023c398ca4f8dd`·2쪽이다. Native 같은 입력/Print 전2쪽은93.46641/99.85838%·exit0/57.988초·미달/누락0이다. 입력과 Print는 재현 진단용이며 원본 fixture를 덮어쓰지 않았다. 아직 fresh WASM이 없으므로 정식 수동 검사·fixture 계약을 바꾸지 않았다.
- 고용지원 원본28개 셀의 출력 내용을 원본 XML과 전쪽 순서로 대조해 누락/중복0을 확인했다. 원본7행 최소16170HU=215.6px에 대해 v124 앞161.1+뒤38.4=199.5px로 미달이고 v131은175.7+39.9=215.6px를 보존한다. 첫3쪽의 마지막 칸과 본문 아래 사이 간격은 v124 16.5/18.8/11.0px와 달리 v1311.9/1.9/1.8px로 원본 바깥 아래141HU와 대응하며 마지막4쪽은 내용 끝에서 멈춘다. JSON bbox는0.1px 반올림이라 이 진단의0.15px 허용은 표현 오차이고 fixture 절대 위치 기준값이 아니다(`employment-independent-relational-before-after-v131.json`, `employment-semantic-source-contract-v131.json`). 같은 의미의 정식 수정 전FAIL/수정 후PASS 검사는 Native/fresh 시각 선행 조건을 모두 확인한 뒤 반영한다.


- v131 Native #5877 전31쪽은 exit0/1074.690초·최저18쪽93.45386%·미달/누락0이다. 모든31쪽 PNG와TSV가v124 Native와 byte 동일하며 current source의 재출력을 따로 확인했다(`ghost-native-whole-v131-summary.json`). 종료된 Native 조각 SVG51개1,890,347,979byte를 사용자 정리 지시에 따라 삭제하고 PNG·TSV·입력·Print·로그는 보존했다. 공유 target은 변경하지 않았다.
- v131 fresh WASM build는exit0/1121.818초·source 불변이다. frozen/pkg/Studio public의JS SHA는 `4628f097cddafb073362cc2ff03d19744de3e037aa62d311b4bd4386528c7a20`, WASM SHA는 `99690fcc641f585550ddc21dc94e432f5f85ca8b2871d6287f421e359ca859da`로세경로일치한다. Studio실제브라우저검증과전체시각gate로확대하지않으며 고용지원4·도로조명원본2·정식Rust writer변형2·설계7·#5877전31쪽 fresh비교를시작했다.
- 남은 #7009 두 검사는 고정base21ed 동일 원본에서도 Visual Sweep312쪽HWP69.53447%/HWPX61.83606%로미달이며 각각전체381/383쪽vs독립Print384쪽이다. 최초384쪽요청은 HWP381쪽 범위초과로exit2/24.881초종료하여실패기록을보존하고존재하는312쪽만새job으로비교했다(재시도exit1/66.846초). 선택1쪽 실패는전쪽90%를부정할수있지만전체384쪽검증통과를증명하지않는다. 두312쪽review를현재대화에실제표시/판독하여같은표제아래본문누락/잘못된소속을확인했다. 같은원본·같은문제의 #7445 직계하위 #7751/#7752를GitHubAPI로재읽어OPEN/본문 SHA `424982679bd051a272298dfad0a17ee420a86c3004989962b11f96587475142c`/`ca1f5769c774933dcfcb4796c20e47a653a58bfd0951fb6375646d3d134c0390`을확인했다. 사용자지침에따라`issue_7009_handbook_structure_landmarks.rs`의기존“-3편차보존”/“부록구간쪽수”두검사만제거했다. 파일의나머지는그두검사전용helper이며다른정식검사를포함하지않는다. 원본2·정상Print2·다른원장/함수/기대값은변경하지않았고이를렌더러개선으로세지않는다. 파생suite준비와삭제후집중검증은다음단계에서실행한다.


- 고용지원 원본4쪽 fresh는exit0/61.359초·Native와전쪽PNG/TSV byte동일·최저1쪽92.57173%·미달/누락0이다. 도로조명 원본2쪽 fresh exit0/69.538초와 같은Rust writer변형2쪽 fresh exit0/71.396초는각각최저1쪽93.46641%·미달/누락0이며Native와전쪽byte동일이다. 설계7쪽 fresh exit0/62.604초·최저5쪽96.39917%·전쪽byte동일이다. 고용지원 fresh2쪽과Rust대조본 fresh1쪽 review를실제표시/판독했다. 작은글리프/괘선편차는남으며픽셀완전일치를주장하지않는다. #5877전31쪽 fresh와#6551영향2쪽fresh는별도진행중이다.
- 위Native/fresh선행조건뒤#6950의`first_fragment_real_fixture_matches_paragraph_offset_without_extra_margins` 절대0.02px핀을제거하고`issue_6950_original_fragment_relations.rs`의3관계검사로옮겼다. 기존1326줄파일에는줄을더하지않고그원본검사만새255줄모듈로분리했다. 독립Print전체4쪽·제목앞뒤/셀·글줄의본문포함·첫3조각바깥아래여백·마지막내용끝·분할7행원본최소높이·원본모든28셀내용의순서/누락/중복을직접출력으로검사한다. JSON bbox 네값의각0.1소수표현반올림에따른0.2끝점오차만허용하며절대fixture좌표기준은없다. 같은정식source SHA `2539dee6a4146c9bbd42be12578c49fe59ceb54e14dd140854b722727d53307e`를frozen v124CLI에연결한독립before실행은exit101/1.826초·2 FAIL/1 PASS다:1쪽gap16.5vs원본1.88,7행199.5vs원본215.6. 최초진단컴파일의Box<Table>형식오류와없는binary127실행은결함재현으로세지않고수정후컴파일/실행결과만연결했다. current정식suite실행은아직진행중이다.
- #6860 기존수동대조함수는양수원점이저장경계를없앤다는잘못된가정대신독립Print의첫줄1쪽소속/두쪽유지/중복방지를검사하도록수정했다. production은이수동대조군을근거로변경하지않았다. 같은입력의ActualPrintSHA와방법·Native/fresh최저93.46641%를기존MANIFEST/README계약에연결했다. 변형HWPX를정상한컴저장원본이라고부르지않으며원본binary는보존한다. 독립Print는필요한회귀근거이므로최종커밋에포함하고Python중간ZIP용중복PDF는정리한다.
- 파생suite준비는성공했으며28 suites+20exceptions·1461 sources·6273 static attrs다. file변동으로suite배정이다시바뀌어정식after명령은현행18 suite를조회해선택했고신규#6950관계3·#6860수정·기존관련범위를함께실행한다(`source-frame-corrected-regressions-v131-command.json`). production파일은fmt/prepare후에도v131QualifiedSource와SHA동일임을확인했다. source회귀삭제/이동만으로current전체RustPASS또는PR준비완료를선언하지않는다.


- 정식 after 집중207검사는exit0/249.853초·실행116.601초·207 PASS/0 FAIL/7704 skipped다. 신규관계3과#6860수정함수모두PASS이며같은newcaseSource SHA는before진단과같다. 전31쪽fresh #5877도exit0/1051.178초·최저18쪽93.45386%·미달/누락0이고Native와모든31PNG/TSV byte동일이다. #6551영향3/4쪽fresh exit0/348.562초·92.30181/94.60777%·Native와두PNG/TSV byte동일이다. #6551전체46쪽fresh통과로확대하지않는다. currentfresh#5877최저18쪽review를실제표시/판독했다. 검사추가선행조건과before/after검출은이범위에서충족했으나최종전체Rust는아직재실행하지않았다.
- 현재production+수정case들을고정해필수7gate를순차실행중이다(`source-frame-corrected-stage-gates-v131.json`). fmt --check는exit0/30.553초·source/cases불변이고다음Rust lint는진행중이다. stage필수gate완료뒤커밋하고fetch된upstream/devel SHA가달라졌을때만rebase하며, base가변하면policy비교와영향검증을다시한다. 기존v124전체17실패는이source의current총실패수로재인용하지않으며PR준비완료/전체회귀PASS를주장하지않는다.


- 종료된 fresh WASM6개 작업의 SVG276개(13,410,058,911byte·약12.49GiB)를 삭제해 디스크 가용 공간을 약15GB로 확보했다. 각 작업의 terminal exit0와 source 불변을 확인했고 원본·독립Print·PNG·TSV·manifest·로그 및 공유target은 보존했다(`finished-fresh-source-frame-svg-cleanup-v131.json`). 압축 보관은 하지 않았다. 필요 없는 Python ZIP 대조 중간PDF는 제거했으며 같은 Rust writer 입력의 실제Print PDF만 회귀 근거로 보존한다.
- 고정source/cases 필수gate의 최초 실행에서 fmt·Native Clippy·WASM32 lib Clippy·workspace build·workspace all-targets Clippy가 모두exit0이었다. manifest 검사는 최신 case bytes에 맞지 않는 생성 harness28개로exit1이었고 기대값·정책을 완화하지 않았다. 원본source/cases SHA 불변을 확인하며 최종 case bytes에서 `--prepare`를 다시 실행해exit0/3.145초로 생성물을 갱신했다. retry1 fmt도exit0/28.031초이며 all-targets Clippy와 base 고정 정책 검사는 순차 재확인 중이다(`source-frame-final-cases-reprepare-v131.json`, `source-frame-corrected-stage-gates-v131-retry1.json`).

- retry1은 terminal exit0로 완료했다. 최종 case bytes 재생성 뒤 fmt28.031초·all-targets Clippy15.323초·고정base21ed manifest2.094초·unit-tier26.876초가 모두PASS이고 source/cases SHA는 불변이다. 최초 gate의 Native/WASM32 Clippy·workspace build 통과와 함께 필수7항목을 충족했다. 집중207 PASS 및 Native/fresh 범위별 수치는 위 기록과 같다. 전체Rust·초기18개 최종통합/시각/Studio 검증은 남아 있어 PR 준비 완료를 주장하지 않는다. 이 단계는 일반 커밋으로 고정하고 fetch 뒤 upstream/devel HEAD가 변경된 경우에만 rebase한다.


## 110. v131 완료 뒤 — 전체 실패9개와 원본 시각 비교

- 보정 단계는 일반 커밋 `ebb3d8337dcb6b8ebd8563bd453ad91ee7aff1ac`으로 고정했다. fetch 결과 upstream/devel과 local devel 모두 `21ed64071f81f35f37618717693bd30f727392c9` 그대로여서 rebase하지 않았다. 새 체리픽·원격 push·PR 생성·merge는 하지 않았다(`stage-source-frame-commit-sync-v131.json`).
- 위 고정 커밋의 전체 기본 nextest는 terminal exit100/1603.460초다. 실제 Summary1019.822초·10344 run/10335 PASS/9 FAIL/45 skip이며 production/cases SHA는 불변이다(`whole-default-rust-terminal-failures-v131.json`). 이전17 실패 이름 중8개가 현재 실패 목록에서 빠졌고 새 실패 이름은 없다. #7009 두 검사 삭제, #6950 절대 핀의 관계3검사 대체, #6860 잘못된 수동 기대값 보정도 포함되므로 전부 렌더러 개선으로 세지 않는다. 기존 overflow partition1/2, #1891 개별 상한 검사, oracle partition0은 현재 정식 전체 실행에서PASS했다.
- 현재 실패는 overflow-cell partition7/8/9, #4882 원본215쪽/내보낸 HWPX216쪽, #3930 편람 HWP의 Q8 고정쪽, #5966/#6718 가맹산업 보고서144대143쪽 핀, #1133 소상공인 각주 고정쪽, #1156 KPS 표의0..16행 핀이다. 셀 원장의 실제 관측은 external-bindata32→43, giant-overfill76→230, 편람HWP32→35다. 기준값이나 공차를 완화하지 않는다.
- 가맹산업 원본 SHA `a1b8299b2554a83873b780474defb02efa949b43c86c63bcc164967f96091e55`와 기존 독립Print SHA `4cf22800e29baddb9b84023462c75252efd4efb04d22e9e64a8e3d9313076009`는 유지했다. PR #6098의 동일PDF 기록 및 Creator/Producer를 재확인했다. 고정base143쪽/현재144쪽/Print142쪽이며, 선택60쪽 base38.12514%·current46.87310%로 둘 다미달이다. current60쪽 review를 실제 표시/판독했고 독립Print59쪽의 내용이 나타나는 쪽 소속 차이를 확인했다. 현재 전144쪽 tree와 원본 model table을 읽어 말미12행과 ‘8. 보복조치 행위’가143~144쪽에 남음을 확인했다. 텍스트 근방 대응은 진단일 뿐 전쪽 시각 통과가 아니다(`franchise5966-base-current-p60-v131.json`, `franchise5966-current-body-page-correspondence-v131.json`). 대화에서현재142쪽이라고 한 최초 설명은 v124의 과거값을 잘못 재인용한 것이므로 즉시 현재144쪽으로 정정했다. 검사143을142/144로 바꾸거나 원문을 제거하지 않았다.
- #7751/#7752/#7753/#7732의 최신 본문을 API로 다시 읽어 동일 원본/축의 이관과 유지 범위를 확인했다(`remaining-regression-exact-transfer-live-v132.json`). #7753은 #4882 왕복 등식을 유지하고 #7751은 Q8 계약을 유지한 기록이 있어, 문서명만으로 추가 검사를 일괄 제거하지 않는다. #7769의 synam-001은 실패한 kps-ai와 다른 원본이다.
- #7732에는 동일 원본 `issue1937_rowbreak_footnote_overpagination.hwp`의 #1133 각주 쪽번호 함수가 제거 대상으로 명시되어 있다. 현재 고정base51/Print50쪽, current52/Print50쪽이며 base25쪽98.30030%·43쪽40.95946%, current25쪽40.14877%·43쪽32.81172%다. current25쪽 review를 실제 표시/판독해 다른 표 내용과 본문의 쪽 소속 차이를 확인했다. 기존 함수의 부적합 이관 근거와 현재25쪽의 추가 악화를 구분하며 코드의 소비 경로도 검토한다(`footnote1937-base-current-selected-v132.json`, `footnote1937-base-current-direct-v132.json`). 이 단계에서는 해당 검사를 아직 삭제하지 않았다.
- CGMP는 현재 커밋의 Native 전48쪽을 재출력해 terminal exit0/1160.087초·최저47쪽94.00342%·미달/누락0을 확인했다. 모든48PNG는 v111 Native와 byte동일하나 실제 현재 실행을 별도로 기록했다(`cgmp-current-native-whole-summary-v131.json`). 현재 frozen fresh WASM 전48쪽은 이어 진행 중이며 과거 fresh 결과로 최신 실행을 대신하지 않는다. CGMP의 현재 메인터너 보정 범위와 원본/정식5검사는 유지한다.
- 전체 Rust는 종료했으므로 다음 보정 단계의 source/cases 편집이 가능하다. 관련 원본의 시각 판독과 실제 값의 생산→예산/스캔→커서 잔여→paint 소비를 대조한 뒤 한 묶음으로 보정한다. 현재 KPS의 독립Print37/38쪽 base/current 진단은 진행 중이다. Native Skia·Studio·초기18개 최종 시각·개별review/오늘할일·통합PR/정확한head CI/merge/후속처리는 아직 남는다.

- KPS 원본 `samples/kps-ai.hwp` SHA `9b0fceb3d96956f27c893e15a72a1ad94f7ee005bd581381a1aadfcb1f57a7b9`, Print `pdf/kps-ai-2022.pdf` SHA `7c064fd290368369a3c8eaa7d7b03668c46fb4dfe0fc18ba52d00456ffe01d28`를 같은 글꼴/DPI로 비교했다. base/current/Print 모두77쪽이며, 선택37/38쪽 base96.09153/50.04329%·exit1/88.583초, current99.81896/98.70875%·exit0/110.668초다. current37쪽 review를 직접 표시/판독해 Print와 내용·표 경계가 대응함을 확인했다. 기존0..16 고정행 기대값의 정당성을 독립Print와 다시 대조하며 검사를 아직 변경하지 않았다. 선택2쪽 Native만의 근거이고 전77쪽/fresh WASM 통과가 아니다(`kps1156-base-current-selected-v132.json`, `native-current-kps1156-direct-v132/kps1156/provenance.json`).
- 완료한 CGMP Native48SVG1,588,680,589byte(약1.48GiB)를 삭제했다. Native terminal exit0와48PNG/TSV 보존을 확인했고 현재 실행 중인 WASM의font_policy/svg·rendered/svg 경로와 구분했다(`finished-native-cgmp-svg-cleanup-v131.json`). 삭제 뒤 가용 공간은8.5GB이며 source·입력·Print·공유target은 변경하지 않았다. KPS의 full font SVG는 선택2개에도약0.149GiB이므로 전77쪽 Native/fresh 작업 전에는 작업별 공간 소비를 확인한다. 공간 때문에 검증 범위를 축소하거나 완료로 보고하지 않는다.


## 111. v132 — 첫 프레임 높이의 소유 판정과 추가 회귀 추적

- 이전 goal turn은 v131 전체10344검사 terminal 및9실패 확정, 원본/Print baseline 비교와KPS 선택2쪽 직접 판독을 완료한 진행이다. 현재 Cargo와 v131 전체 nextest는 종료했으며 CGMP frozen fresh만 별도 계속하던 상태를 확인했다.
- 같은 소상공인 원본25쪽의 단계별 frozen Native tree를 대조했다. 최초 통합 f58718 및v2/v20/v41/v58/v77/v78/v79/v80에는 Print와 같은 본문·각주가 있고 v81부터 이전 표의 행이 남으며 각주가 없다. v94/v95/v111/v124/v131도 같은 소속 차이가 유지된다. 따라서 v131의 마지막 일반행 재컷만으로 이 차이를 설명하지 않는다(`footnote1937-early-frozen-p25-bisection-v132.json`, `footnote1937-v78-v85-bisection-v132.json`, `footnote1937-frozen-version-p25-bisection-v132.json`).
- v81의 일반 원점 재시작이 임의의 첫 상자 빈 밴드까지 인증하는 가정을 제거했다. 실제 원본 줄 전진과 안여백의 끝을 포함한 경계만 허용하며 별도로 인증된 병합 상자는 유지한다. 앞간격200HU·첫조각24.6667px의 기존 근거를 포함한 끝점 equality를 허용하고 문서ID/고정좌표 분기를 추가하지 않았다. 생산 `first_source_frame_owner` → `CellCutFrame::height` 후보 높이와 `row_cut_content_height`의 행 상자 → scan/emit 소비를 추적한다.
- 첫 v132 Native build는exit0/79.104초·source 불변·SHA `28c6104d78ef37bdb7c2877371c739db8f8cf9b9b140a278cd53a1432a7f556a`다. 그러나 소상공인25쪽 tree는 여전히 이전 표·각주0이며 이 수정만으로 해결을 주장하지 않는다. KPS37쪽과가맹산업60쪽도 quick tree만확인했으며시각승인으로세지않는다(`first-frame-boundary-quick-counters-v132.json`). 일반 첫 프레임의 실제컷·높이·소유를 진단하려고 환경변수로만 활성화되는 trace를 추가해 후속Native를빌드중이다. 정식 회귀 기대값·baseline은 아직 변경하지 않았다.
- 소상공인 원본164문단7×3표의 common26965HU와 raw행1282/17722/8762/2882/2882/2882/2882HU 및 실제 저장리셋을 public DocumentCore로 읽어 기록했다(`footnote1937-source-row-metrics-v132.log`). 첫 standalone 진단의 library glob 오류는 실제비해시 `debug/librhwp.rlib` 경로로바로잡았다. 이것은 구현 결함의 수정 전FAIL 증거가 아니다.
- frozen `ebb3d8337` CGMP fresh 전48쪽은terminal exit0/1843.162초·최저47쪽94.00342%·미달0이며 모든48PNG가동결Native와byte동일하다(`cgmp-frozen-native-fresh-whole-v131-summary.json`). 새 v132의 source는 변경됐으므로 그48쪽 성공을 새source검증으로 재인용하지 않는다. 해당 종료 작업의SVG144개는 해시를보존하고사용자지침대로삭제한다.


## 112. v132 — 첫 프레임 인증과 실제 컷의 유효 안여백 통일

- v80/v81/current frozen CLI의 동일 원본 scan을 실행해 최초 차이가 소상공인99문단 표의4행 컷 `[1,2,7,1,1,1,1]` 뒤에서 발생함을 확인했다. v80의 다음 쪽16행이 v81의14행으로 이동한다(`footnote1937-v80-v81-scan-diff-v132.txt`). 마지막25쪽만의 각주 배치 문제가 아니다.
- 실제99문단4행 셀의 선언282HU와 원본 안여백566+566HU는 현재 `resolve_cell_padding`에서 유효 총141HU=1.88px로 축소된다. 그러나 첫 프레임 소유 인증은 원본1132HU=15.09333px를 더해 실제 컷에 없는 높이를 인정했다. 상자142.29333px, 내용 전진134.4px에서 원본 여백으로 인증되던 프레임은 실제 유효 여백과 일치하지 않는다. 같은 원본·독립 Print를 유지하며 값을 고정하는 문서 예외를 추가하지 않는다.
- 생산 `first_source_frame_owner`와 완결 형제 높이 → `row_cut_content_height`/`CellCutFrame::height` → scan 예산/재컷/예약 → 이어받기 물리 높이의 소비를 대조했다. 첫 프레임 소유·완결 형제·첫 프레임 trailing trim 모두 기존 실제 컷과 같은 `resolve_cell_padding`을 소비하도록 통일한다. 원점 reset만으로 임의 빈 밴드를 소유하는 v81 가정도 제거하며, 독립 형제 행의 증명이 있는 rowspan 경로와 문단 앞 간격 인증은 보존한다.
- 첫 유효 여백 진단 Native는 build exit0/49.402초, SHA `e9a450f86adfcee77b618308d12df66103a06fed7eb08051ab6cf8b252ed172c`다. 같은 원본25쪽 render tree의 각주는0→1로 복구됐다(`footnote1937-resolved-padding-quick-v132/`). 이는 내용 소속 진단이고 전체50쪽·Native/fresh WASM90% 증거가 아니다. 잘못 호출한 export-render-tree `--json`은 usage exit2였으며 올바른 `-o` 재실행 exit0으로 고쳤고 수정 전 결함 검출로 세지 않는다.
- 진단 trace를 제거하고 첫 프레임 trim까지 동일한 여백 소비로 정리한 최종 후보를 fmt/build 중이다. 이어 같은 원본25/43쪽·정상 첫 프레임/이어받기 대조와 기존 정식 검사를 한 묶음으로 확인한다. source/cases와 원본/Print는 검증 중 동결하며 기존 검사 기대값·baseline을 변경하지 않는다. 새 회귀 추가와 통합 PR/CI/merge는 최종 Native/fresh·필수 gate까지 보류한다.

- 최종 후보 Native build는 exit0/51.973초·source불변·SHA `2eafdfa74af6c1ac0a2c7ce850b5b2ed5a272b886eb57e4b669a3b7b03ad2cbb`다. 기존 검사는 최초198개와 현재 파생suite에서 선택 누락을 확인한 추가16개, 총214개 모두PASS다(96.338초/100.425초, compile포함291.823초/150.934초). #1133의 기존25쪽 각주 검사도PASS이며 기대값을 변경하지 않았다.

- 최종 후보의 소상공인 Native25쪽은40.14877→98.29507%로 복구됐고 실제 각주·표 마지막 행·뒤 본문 순서를 직접 review로 확인했다.43쪽은32.81172→39.81618%로 개선되지만 여전히 미달이며 실제 전체51/Print50쪽이다. 고정base도43쪽40.95946%·51/50쪽이지만 이번25쪽 개선으로 전체 미달을 면제하지 않는다. 원본 전체90%·fresh WASM 완료로 보고하지 않는다(`native-effective-padding-footnote1937-retry2-v132/footnote1937/silhouette.tsv`). 가맹산업 원본의 최신 실제쪽수도144로 유지된다(`first-frame-effective-padding-final-quick-counters-v132.json`).
- 첫 Chrome WS 시작30초 시간초과는 제품 결함과 구분한다. 실루엣 전용에서 지원하지 않는 `--resume`의usage exit2도 보존했으며, 같은 입력·후보·글꼴·Print로 새출력/Chrome 대기180초를 지정한 재실행은 두쪽 raster/TSV를 완료했고 exit1은43쪽/전체쪽수 미달에 따른 gate결과다. 같은 완료SVG를 별도Google Chrome for Testing152.0.7977.75로 캡처한25쪽 진단 review도 표시·판독했다(98.29512%). 캐시Chrome 진단과 기본raster의98.29507%를 구분한다. 원본·글꼴·좌표·일치율 허용값을 바꾸지 않았다.
- 정상대조 Native는5585의14/15/16쪽98.86635/99.86181/97.84322%, 고용지원 전4쪽93.26573/98.66593/96.40553/98.89367%, CGMP35/39/42/47쪽98.46451/95.37856/98.77406/94.00342%로 모두90% 이상이다(`first-frame-effective-padding-native-controls-v132.json`). 고용지원만 해당문서 전쪽이고 CGMP/5585는 선택 경계이며 fresh로 확대하지 않는다.
- 교육과정 첫 프레임 대조의 과거 active samples/PDF 경로는 이미 같은바이트의#7445 asset으로 이관된 상태였다. 실제 실행자료의원본SHA623b00d56bef…/PrintSHA82fe8eefecbf…를 확인해 현재 보존 경로로 재실행한다. 첫 경로없음exit1은제품회귀가 아니며 새source·fixture로 대체하지 않는다. 교육과정 검사를 복원하지 않는다.

- v132의 fmt/Native Clippy/WASM32 Clippy/workspace build/all-target Clippy/고정base manifest/고정base unit-tier7단계는 모두exit0이며 source/cases불변이다(30.984/92.891/79.944/109.955/114.573/6.523/28.032초). 그러나 이 lint 성공으로 뒤의 실행 회귀를 면제하지 않는다.
- 끝난 중간 v114–123/v125–128 바이너리24개를 실행 중인 `/proc/*/exe`와 대조하고SHA를 기록한 뒤 삭제해8,082,372,592byte(약7.53GiB)를 확보했다(`finished-intermediate-binary-cleanup-v132.json`). v80/v81·v113/v124/v129/v131/v132와고정base/현재후보, 원본·PDF·PNG·TSV·공유target은보존했고 가용공간은17GB다. 압축하지 않았다.

## 113. v133 — 첫 프레임 빈 물리 공간의 반례 확인과 재보정

- 같은 이관 원본의 교육과정 대조188/189/190쪽은96.57268/61.77494/21.36546%이며 전체416/Print415쪽이다. 바로직전 frozen v131의 같은 바이트/compat2022 실제layout은415쪽이었다(`education-v131-before-effective-padding-page188-v132.json`, exit0/44.633초).188쪽 render tree도byte동일하지 않다(exit0/41.794초). 따라서 과거#7445 미달만으로 이번415→416 실행회귀를 제외하지 않으며 후보를 커밋하지 않았다. 최종 WASM도 아직 시작하지 않았다.
- 내용 전진과 실제 물리 상자의 빈 띠를 구분한다. 저장Common 상자의 높이를 내용전진의 상한으로 항상 제한한 v132조건은 원본상자가소유하는 빈물리공간도 기각할 수 있다. 현재후보는 실제컷의유효여백과원본여백이같은좌표를쓰는지를먼저검사하고, 일치하는상자의인증된같은문단쪽프레임reset에는기존물리공간소유를보존하도록재보정한다. 안여백축소로좌표가달라진상자는Common높이를그대로덧씌우지않는다. 문서ID·쪽번호·임의픽셀핀을추가하지않는다.
- 소상공인/교육과정 두원본의실제쪽수·영향경계를같이확인한뒤관련214검사와필수gate·Native/fresh를같은새후보로재실행한다. v132의214PASS/7gatePASS를바뀐v133source의성공으로재인용하지않는다. 기존모든원본/Print·검사기대값·baseline은그대로보존한다.

- 첫 v132의 줄 전진 상한만 적용한 동결후보도 같은교육과정에서416쪽이었다(exit0/45.990초, `education-bound-only-causal-counter-v133.json`). 원본안여백을유효여백으로바꾼후보와구분해원인을분리했다. v133 Native build는exit0/75.616초·source불변·SHA `c78594a23d03adc0d45c84ad347d78fcda53089e440c83f0faebf6267e26c373`다. 실제쪽수는소상공인51/교육과정415로동결전후의수량을보존했다(4.086초/41.095초, `first-frame-padding-coordinate-contract-quick-counters-v133.json`). 쪽수보존을원본전체시각통과로쓰지않는다.
- 현재파생28suite와lib의같은필터214검사를한번에선택하고, 종료성공뒤필수7gate를순차실행하는worker를시작했다. 별도의동결Nativeworker는소상공인25/43쪽·5585 14–16쪽·고용지원전4쪽·CGMP35/39/42/47쪽·교육과정188–190쪽을같은새후보로재출력한다. 안정된Google Chrome152 실행경로와180초대기만명시하며90%기준·글꼴·원본·PDF·좌표를완화하지않는다. 두worker는현재실행중이며fresh WASM/커밋/새base동기화는아직미실행이다.

- 최신v133후보에서도원본25쪽은98.29512%·실제각주/표마지막행/뒤본문의순서를새review.png (`output/pr-review/semanticist21-20261008/native-coordinate-contract-footnote1937-direct-v133/footnote1937/review.png`; 당시 로컬 증적, 저장소 미포함)에서직접표시·확인했다.43쪽39.81618%·전체51/Print50쪽미달은남는다. 관련214기존검사는최신source의같은28suite/lib에서전부PASS/exit0(실행118.806초, compile포함318.116초, 3slow)다. 기대값변경이나새회귀추가는없다. 필수7gate와남은Native대조는같은source에서이어진행하며이전후보의성공을전용하지않는다.


## 114. v133 — Native 대조·필수 lint 종료와 남은 실패 재확인

- 현재 source/cases에서 필수7단계 모두terminal PASS다(`first-frame-padding-coordinate-contract-stage-gates-v133.json`): fmt33.632초, Native Clippy98.552초, WASM32 Clippy87.132초, workspace build125.846초, all-target Clippy117.602초, 고정base21ed64071 manifest7.684초, unit-tier25.388초. 전체기간 source/cases불변이며 파생suite나baseline을커밋하지않는다.
- 동결Native5문서/16선택쪽대조도terminal이다(`first-frame-padding-coordinate-contract-native-controls-v133.json`). 고용지원 전4쪽최저1쪽93.27661%,5585선택3쪽최저16쪽97.84322%, CGMP선택4쪽최저47쪽94.05301%다. 교육과정188/189/190쪽은99.02821/97.68479/99.13336%·실제415/Print415쪽으로v132의416쪽회귀가복구됐다.189쪽새review를직접표시·판독해행소속/본문순서/쪽이어받기를확인했고,Native하단표괘선이PDF보다약5px낮은차이를남긴다. 교육과정전체415쪽미달해소로확대하지않는다. 소상공인은25쪽98.29512%복구,43쪽39.81618%와51/50쪽의기존미달을남긴다.
- 최신후보의이전실패8개를정확한전체함수명으로다시실행했다(`remaining-eight-terminal-results-v133.json`): 실제119.554초/compile포함226.074초,1PASS/7FAIL,exit100/source불변이다. overflow-cell partition7은새후보에서PASS다. 나머지는partition8/9,정책연구원본215/내보낸HWPX216쪽등식,Q8고정쪽,KPS고정행,가맹산업144대143두검사다. 전체release-test10344재실행으로해석하지않고이번관련214PASS와남은7FAIL을구분한다.
- 정책연구원본을새후보의같은Nativeexport로진단HWPX에내보냈다(`policy-roundtrip-forensics-v133/run.json`). 원본SHA50094a3db2b2…불변,생성SHAf2d662b1ad5e…,CLI `--verify-pages` exit4/75.831초에서215/216쪽을재현했다. `ir-diff`도exit3/19.270초이며정규화차이는쪽수1건과표셀1건,합계2건이다. 이생성HWPX는진단자료이고독립Print/정식fixture가아니다. 원본이나원래왕복등식을제거하거나기준값을완화하지않는다.
- 이번단계는첫프레임여백좌표보정의Native·기존검사·lint체크포인트다. fresh WASM·현재source의전체필수검증·남은7실패·초기18개개별review/오늘할일/통합PR/CI/merge/후속처리는완료되지않았다. 종료검증뒤현재변경을일반커밋으로고정하고fetch된upstream/develHEAD가달라진경우에만rebase한다.


### 진행115 — HWP5 저장 각주의 왕복 예약 프로파일 보정과 잔여 실패 분석(v134)

- `d3bccc16b4f6b0779520f49c2e33e571f262c7df` 체크포인트 뒤 fetch한 `upstream/devel`은 기존 `21ed64071f81f35f37618717693bd30f727392c9`와 같아 rebase하지 않았다. v133 fresh WASM 빌드는 exit0/1082.741초/source 불변으로 완료됐고, 고정 package·루트 pkg·Studio public의 JS/WASM SHA가 일치한다(`wasm-build-first-frame-padding-coordinate-contract-v133-exit.json`). 현재 v133 경계·정상 대조군 비교는 이 고정 package와 CLI를 사용하며, 이후 작업 트리의 v134 변경을 v133 검증으로 보고하지 않는다.
- 정책연구 #4882의 fixed base 왕복은215/215쪽, v133은215/216쪽이며, 두 export ZIP의 모든 entry가 바이트 동일하다. 정규화 IR 차이2건은 전체 raw IR 차이2건을 뜻하지 않는다. 실제 첫 차이는 물리9쪽의 pi225 표/뒤 빈 문단이 왕복에서10쪽으로 밀리는 것이며, 앞 각주 등록 뒤 표의 가용 높이는821.0→817.4px이다(`policy-roundtrip-forensics-v133/base-current-export-entry-comparison.json`, `page9-scan-diff.txt`).
- 원인은 `notes/footnotes/measure.rs::reserve_current_body_note_area`가 Native HWP5의 유효한 저장 각주만 기존 쪽 컷을 보존하고, 같은 줄을 가진 HWP5-origin HWPX에는 실제 조판 예약 높이를 다시 적용하는 조건이다. `hwp5_stored_pagination_layout()`으로 원 HWP5와 origin marker HWPX의 계약을 공유했다. 단일 단·미편집·비어 있지 않은 진짜 저장 줄·양의 줄 높이·dirty가 아닌 텍스트 분할이라는 기존 유효성 조건은 보존했다. 순수 HWPX/HWP3와 편집·합성·누락 줄은 기존 실제 조판 예약을 소비한다. 문서 ID나 픽셀 예외를 추가하지 않았다.
- 실제 소비 경로는 `section/controls.rs::Control::Footnote → register_body_footnote → reserve_current_body_note_area → state/notes.rs::reserve_painted_footnote_area → current_footnote_height/body bottom 예약 → 다음 본문·표 예산`이다. 실제 각주 배치의 유효 저장 줄은 `layout/picture_footnote.rs::footnote_area_has_stored_lines`로 확인한다. 공통 프로파일을 사용한 각주 등록 경로와 예약 후 값을 덮어쓰는 지점을 함께 대조했다.
- v134 Native 빌드는 exit0/81.437초/source 불변, CLI SHA `58107f0957c06280aaa48e7143d6d0f501b50e67f1bcc88d8ff6df13a46eecf2`다. 같은 원본의 `export-hwpx --verify-pages --json`은 exit0/41.553초에서215/215쪽으로 복구됐다. 원본 SHA50094a3db2b2…와 export ZIP entry는 수정 전후 불변이다(`saved-note-pagination-profile-artifact-v134.json`, `policy-roundtrip-forensics-v134/run.json`). 생성 HWPX는 진단자료이며 독립 Print를 가진 새 fixture로 등록하지 않았다.
- Native 원본/왕복의 물리9·10쪽은 보정 전 원본99.87989/99.97364%, 왕복65.12338/18.32114%에서, 보정 후 양쪽 모두99.87989/99.97364%로 복구됐다. `native-policy-roundtrip-after-v134-exit.json`은 exit0/393.713초/source 불변이다. 새 왕복 `direct/review/review_009.png`와 `review_010.png`를 실제 표시해 뒤 그림·각주·본문 순서와 같은 쪽 소속을 확인했다. 그림 배경색과 활자 굵기 차이는 남아 있으며, 전215쪽 일치나 fresh WASM 완료로 확대하지 않는다. 원본 Print와 생성 HWPX의 비교는 왕복 진단이고 생성본 자체의 독립 한컴 Print 증거와 구분한다.
- 기존214검사에 #4882/#5128·각주/미주·편집 대조를 포함한424개 검사를 실행했다. 최종423PASS/1FAIL/9715skip, 실제204.659초/compile 포함400.422초, exit100/source 불변이다(`saved-note-pagination-profile-focused-v134-exit.json`). #4882 기존 왕복 등식은 수정 전 FAIL/수정 후 PASS이며 기대값·fixture를 바꾸지 않았다. 실패1개는 v131 전체검증에도 있던 #5966의144대143쪽 핀으로, 새 각주 프로파일 보정의 회귀로 재분류하거나 성공으로 숨기지 않는다. 전체 release-test는 아직 v131 기록이며 새 head 전체 무실패라고 보고하지 않는다.
- 편람의 실제 Q8 본문은 search의0기반284(물리285쪽)에 있고, 기존 Q8 검사는0기반283을 지정한다. 독립 Print285·286쪽과 Native는97.33137/96.42398%이며285쪽 review에서 Q7 응답 tail 뒤 Q8의 같은 쪽 표시를 확인했다. 전체382대384쪽 차이 때문에 gate는 보류다(`handbook-q8-pin-forensics-v134/address-analysis.json`, `native-handbook-q8-direct-v134-exit.json`). 검사 삭제·새 위치 핀·관계 검사 교체는 아직 하지 않았다.
- giant HWPX 원본의 default2022 셀 넘침은 fixed base76건(42쪽56/43쪽20), v134230건(41쪽144/42쪽66/43쪽20), 양쪽48쪽이다. 첫 진단의 지원되지 않는 `--embed-fonts=none` exit2는 입력/렌더 회귀가 아니며 옵션을 제거한 재실행 exit0의 결과를 사용했다(`giant-cell-overflow-forensics-v134-retry/results.json`). frozen v80/v81은76, v82 retry1부터230이며 v94/v111/v124도230이다(`giant-cell-overflow-history-v134/results.json`). v81→v82의 실제 소스 차이는 table_layout과 stored_plain_cell_minimum이며 커밋3874bc4be의 저장 줄+실효 여백 최소값과 중첩 가로 앵커 소비를 조사한다. 기존 raw overlap clip 검증을 셀 넘침 내용 보존의 증거로 바꾸지 않는다. 기존 이슈 목록의 giant 언급은 PR 설명이며 동일 #7445 하위 이슈 이관의 증거가 아니다. 원장/허용치를 바꾸지 않았다.
- 종료된 ignored 시각 SVG2484개62,444,596,125bytes(58.156GiB)를 삭제했다(`finished-svg-cleanup-v134.json`). 실행 중인 v133 WASM와 v134 Native 경로는 제외했으며, 원본·Print PDF·PNG·TSV·로그·JSON·고정 package·공용 target은 보존했다. v133 각주 비교 종료 뒤 해당153SVG도9,334,640,314bytes(8.694GiB)를 삭제했다(`finished-footnote1937-wasm-svg-cleanup-v134.json`); SVG를 압축하지 않았다.
- v133 fresh 각주25/43쪽은98.29512/39.81618%로 Native와 같고51대50쪽이다.25쪽의 새 `wasm-coordinate-contract-footnote1937-direct-v133/footnote1937/review.png`를 실제 표시해 표 꼬리·뒤 본문·각주/꼬리말의 소속을 읽었다.43쪽과 전체 쪽수의 기존 미달은 유지한다. v134의 최종 lint/fresh WASM, 나머지 기존 실패와 초기18개 전체 review·개별 문서·오늘할일·PR/CI/merge/후속처리는 진행 중이다. 추가 체리픽은 없다.

- v134의 필수 lint/정책7단계도 모두 exit0/source·cases 불변으로 완료했다(`saved-note-pagination-profile-stage-gates-v134.json`): fmt 27.757초 / clippy-native 97.175초 / clippy-wasm 81.824초 / workspace-build 120.369초 / clippy-all-targets 115.826초 / manifest-base 5.534초 / unit-tiers-base 24.331초. 정책 비교 base는21ed64071 고정이며 generated suite/manifest는 stage하지 않는다. 원본/fixture/기대값 변경은 없다. 잔여8개 정확한 함수의 새후보 진단과 fresh WASM 확인은 별도로 계속한다.

- 이전 실패8개의 v134 진단은 최종 Summary [ 126.513s] 8 tests run: 2 passed (1 slow), 6 failed, 1111 skipped이며 compile 포함240.767초/exit100/source 불변이다(`remaining-eight-terminal-results-v134.json`). partition7과 #4882 등식이 PASS, 나머지6개는 셀 원장partition8/9·편람Q8·KPS 행 핀·가맹사업2검사다. 기존424개 검사 중 가맹사업 실패는 이6개와 중복이며 합산하지 않는다. 최종 전체 Rust 무실패가 아니므로 PR을 준비 완료로 판정하지 않는다.
- v134 원본의 전체215쪽 pagination은 v133 원본과 동일하며, 왕복은 전체 쪽/항목/내용 컷 소속이 복구됐다. 왕복13쪽의 `usedHeight`만833.7733대821.7733px로 차이가 남는다(`policy-pagination-forensics-v134-retry/comparison.json`, `page13-differences.json`). 별도 진단에서 존재하지 않는 `dump-pagination`을 호출한 exit2는 결함 증거가 아니며, 기존 성공 명령인 `dump-pages --compat 2022 --json`으로 교정해 양쪽 exit0 결과를 비교했다. 전체 PNG/Print 일치의 증거로 바꾸지 않는다.
- giant v81→v82 pagination은 같은48쪽이며41쪽 종료 컷1242→1240,42쪽 시작 컷도1242→1240으로 바뀐다.41쪽 점유1002.0933→907.8267px,42쪽819.6933→927.5333px다(`giant-pagination-cut-history-v134/boundary-differences.json`).39쪽의 부동소수 끝자리 차이와 실제 컷 변화를 구분했다. 추가 줄144건이 생긴41쪽의 중첩 표 선택 컷/유닛과 실제 배치 소비를 다음 조사 경계로 삼으며, 올바른 안여백 자체를 없애 이전 카운트에 맞추지 않는다.
- v133 fresh 정상 대조도 #5585의14/15/16쪽98.86635/99.86181/97.84322%, 고용지원 원본 전4쪽93.27661/98.69101/96.42142/98.93194%로 Native와 같고 각 exit0다. CGMP 선택35/39/42/47쪽도 exit0/303.4초이나 현재 전48쪽으로 확대하지 않는다(`first-frame-padding-coordinate-contract-wasm-controls-v133.json`). 종료된 정상 대조군63SVG3.475GiB는 PNG/TSV/로그를 보존하고 삭제했다. 교육 경계의 동일 frozen package 비교는 계속 실행 중이다.


### 진행116 — frozen5cc WASM 확인과 직접 HWPX 자식 컷 보존(v135/v136)

- 5ccbc4d005bdc8e037b316e4726cff624c98e1e2의 fresh WASM은 exit0/1106.798초/source 불변으로 완료됐다(`wasm-build-saved-note-pagination-profile-v134-exit.json`). package/root pkg/Studio public의 JS·WASM 해시는 일치하며 WASM SHA는e03855bbeb46…다. 이후 v135/v136 작업 트리와 frozen5cc 실행 결과를 구분한다.
- frozen5cc 정책연구 원본/왕복의 fresh WASM9·10쪽은 양쪽99.87989/99.97364%, 전체215/215쪽, exit0/270.060초다(`wasm-policy-roundtrip-v134-exit.json`). 새 왕복9쪽 review를 실제 표시해 표·그림·각주·본문 소속을 확인했다. 선택 쪽 실행 보조 driver는 public WASM의 전215쪽 raw SVG/tree를 먼저 만들고, 요청 쪽만 같은 Native CSS와 canonical 측정·게이트로 래스터한다. 각 selected-wasm-font-policy-audit는 style 이외 SVG가 원본과 동일함을 검사하며 좌표·내용·허용치를 변경하지 않는다. canonical 명령의 원래 전체 font-policy/전체 styled SVG와 실행 비용을 구분했고, 전215쪽 PNG 일치로 확대하지 않는다.
- frozen5cc CGMP Native 전48쪽은 exit0/741.585초/source 불변, 최저47쪽94.00342%, 미달·누락0이다(`native-cgmp-full-v134-exit.json`). fresh WASM 전48쪽의 raw SVG/tree/font policy/PDF PNG는 만들어졌으나 기본 Chrome의30초 WS 시작 timeout으로 첫 래스터 전에 exit1이었다. 문서 회귀로 판정하지 않고 동일 해시 산출물의 canonical 일반모드 --resume에 Chrome152/180초를 명시해 계속한다(`wasm-cgmp-full-raster-retry-reason-v134.json`). 실루엣 전용+resume은 canonical parser가 금지하므로 일반 compare/overlay/review 재개를 사용한다. CGMP 원장 제외는 fresh 전체 확인 전에는 아직 복구하지 않았다.
- 직접 HWPX 투영은 자식 CellUnit 높이를 가져오면서 recursive=false로 내용 컷을 버렸다. 측정의 같은 유닛을 자식 배치에 전달하도록 기존 분기의 recursive만true로 바꾸고, HWP5 강제 hard/stored 쪽 경계 플래그와 수용 조건은 유지했다. `nested_table_mixed_fragment_heights → mixed_nested_recursive → mixed_nested_split_from_cut → NestedTableSplit.recursive_cut → 자식 partial table`의 실제 소비를 대조했다(`direct-hwpx-child-cut-ownership-diagnosis-v135.json`). 문서 ID·여백 삭제·좌표 clamp·baseline 완화는 없다.
- v135 Native 빌드는 exit0/89.766초/source 불변, CLI SHA7d79e9dbbc72…이다. giant는48쪽을 유지하고 셀 넘침230→0, external1891은74쪽/10건, virtual6776은75쪽/0건, water6776은3쪽/0건, CGMP48쪽/0건이다(`direct-hwpx-child-cut-ownership-counters-v135/results.json`). #3637 press 원본은13쪽이며 반복31쪽 계약의 regulatory 원본과 다르다. regulatory는 before/after31쪽/0건을 유지했다(`regulatory3637-before-after-v135/results.json`). 진단 건수·쪽수만으로 시각 보정 완료를 선언하지 않는다.
- v135 giant Native1/41/42/43/48쪽은90.22786/20.43977/24.98504/20.16949/98.83715%, 전체48대48쪽으로 gate 보류다(`native-giant-child-cuts-v135-exit.json`). 새41쪽 review를 직접 읽으니 Native는고장 배제 표, Print는부속서I 소개여서 실제 내용의 쪽 소속이 다르다. 컷 보존은 넘친/선택하지 않은 자식 유닛 배치의 문제를 해결했지만 원본의 기존 source pagination 차이는 남아 있다. 새 fixture/PNG golden/입력 교체는 하지 않았다.
- v135 관련439검사는436PASS/3FAIL/9716skip, 실제235.084초/compile 포함414.933초다. 기존 가맹사업 핀·원장partition9 외에 synthetic direct-HWPX unit이 recursive=false를 요구해 실패했다. 이 검사는 로컬 reset의 강제 쪽 경계와 자식 내용 컷 전달을 혼동했다. 같은 수동 합성 입력을 유지하고, source 글줄 수 보존과 hard/stored 강제 경계가 없다는 독립 계약으로 교정했다. 사양 입력/실물 Print 증거와 합성 계약을 혼동하지 않고 이 합성 검사로 한컴 일치를 주장하지 않는다.
- v136 재검사는437PASS/2FAIL/9716skip, 실제246.564초/compile 포함697.672초, exit100/source 불변이다(`direct-hwpx-child-cut-contract-focused-v136-exit.json`). 기존 원장partition7/8과 새 로컬 reset 계약은PASS, 가맹사업143쪽 핀과 Native 편람의 원장partition9는FAIL이다. 최종 전체 Rust 무실패가 아니며, source-side test 변경의 base 고정 unit-tier 검사와 필수 lint7단계를 계속한다. 기대값을 현재 좌표/카운트로 잠그거나 원장 허용치를 바꾸지 않았다.
- v133 교육 경계 WASM은 전체 raw415쪽과 Native font-policy415쪽을 만든 뒤 반복 CSS 처리에서1800초 timeout으로 coordinator exit1이었다. 이전 완료4개 run은 보존했고 교육 PASS/TSV는 없음을 기록했다. 검증된 terminal 뒤 frozen5cc의 같은 원본·Print·188/189/190쪽을 선택 보조 driver와2400초 제한으로 다시 실행한다. 부분 산출물을 성공으로 보고하거나 단순 관측 timeout만으로 작업을 재시작하지 않았다.
- 종료된 Native CGMP와 timeout 교육의 Native font-policy463SVG를22.715GiB 삭제했다(`finished-native-cgmp-and-failed-education-policy-svg-cleanup-v136.json`). 재개 중 CGMP와 새 교육 산출물, 원본/PDF/PNG/TSV/tree/manifest/공용 target은 보존했다. KPS 회귀의77쪽수 검사는 전쪽 Native/fresh90%가 필요해 Native 전77쪽 비교를 먼저 시작했고 기존 행/쪽수 기대값은 아직 바꾸지 않았다.

- frozen5cc 교육 경계의 재실행은 exit0/387.094초, 고정 CLI/package 해시 불변으로 종료됐다(`wasm-education-selected-v134-exit.json`). 같은 원본과 Print의188/189/190쪽은99.02821/97.68479/99.13336%로 Native와 일치한다. 문서 쪽수415/415와 선택3쪽의 확인을 전415쪽 일치로 확대하지 않는다. 새 `wasm-education-selected-head5cc-v134/education/direct/review/review_189.png`를 실제 표시·판독했고, 표 행·본문 순서는 대응하지만 하단 괘선이 Print보다 약5px 낮은 차이가 남는다. 지표가90%를 넘었다는 이유로 이 차이를 해소됐다고 기록하지 않는다.
- giant의 부속서I 원본 XML과 Print40–43쪽을 추가 대조했다(`giant-appendix-frame-mapping-v136/source-paragraphs.json`, `print40-43.txt`). 원본 자식 셀의19번째 문단은표 제목/vertpos71020,20번째 문단은26×9 자식 표/vertpos30744이고 명시적 pageBreak/columnBreak는 모두0이다. Print41쪽은소개 문단과표 제목까지,42쪽은표로 시작한다. 단일 vpos reset과현재 hard 표식만으로 강제 쪽 경계를 복구하지 않으며, 부모 조각의 시작·자식 내용 소속과 일반 로컬 reset 대조군을 함께 확인한다.
- v136의 필수7단계는 모두exit0/source·cases 불변으로 종료됐다(`direct-hwpx-child-cut-contract-stage-gates-v136.json`): fmt29.379초, Native Clippy94.333초, WASM32 Clippy82.782초, workspace build126.167초, all-target Clippy126.019초, manifest-base5.871초, unit-tier-base25.038초. 정책 비교 base는21ed64071이다. 기존 합성 검사 교정은 같은 수동 입력의 내용 보존과 강제 경계 비승격 계약이며, 새 실물 fixture/한컴 시각 회귀/허용치를 추가하지 않았다. 실제 giant 시각 미달과 다른 잔여 실패는 그대로 보류한다. 종료한 보정 단계를 일반 커밋으로 고정한 뒤, fetched upstream HEAD가 달라진 경우에만 rebase하고 해당 base 정책 비교를 재실행한다.

### 진행117 — 변경된 upstream 동기화와 giant 첫 분할 차이의 원인 추적(v137/v138)

- v136 종료 단계를d6843de379e2c9594b41a7c1e04af0881f1dfce0으로 일반 커밋한 뒤 fetch했다. upstream이21ed64071→19245412a43a19baa9ca03a7ef5ca99013fc3242로 실제 전진한 경우여서 local devel을fast-forward하고 review branch를충돌 없이rebase했다. 새review HEAD는ccea4910222bcf7cc92f8913ff9ff28ea6754e21이며 Rust source·cases·Cargo 파일은리베이스 전과바이트 동일하다(`direct-hwpx-child-cut-contract-rebase-v137.json`). 추가체리픽은없고 새base의#7712 누름틀 삽입 기능이포함됐다. 기존frozen5cc/v135 시각 산출물은그원래source의증거로보존한다.
- 새base19245412a의manifest와unit-tier 비교는모두exit0(1.977/23.699초)다. rebased Native build도exit0/116.764초이며 별도동결CLI를보존했다(`rebased-rust-confirmation-v137.json`, `rebased-child-cut-contract-artifact-v137.json`). 이전실패8개의정확한함수재검사는132.782초/compile 포함272.212초,3PASS/5FAIL/1111skip,exit100/source불변이다. partition7/8과정책연구왕복은PASS,편람원장partition9·Q8쪽핀·KPS행핀·가맹사업2검사는FAIL이다. 전체release-test결과로확대하지않는다.
- giant 전48쪽Native RenderTree와같은Print의내용소속을대조했다(`giant-appendix-frame-mapping-v136/content-page-correspondence.json`). 내용문자비교는시각90%점수가아니다. 17쪽까지본문이대응하지만18쪽에절연저항표일부가먼저들어가며,Print18쪽은앞소개문단으로끝나고19쪽이표로시작한다. Native40쪽의부속서I 소개는Print41쪽내용과대응한다. v135의48쪽새review도직접판독했고끝본문은대응하지만이를앞쪽분할차이의면제근거로쓰지않는다.
- 원본main 셀의첫줄vpos2000HU는header의paraPr54 case 문단위간격2000HU와같다. 저장323문단끝62862+1200HU→324표host500HU라는되감김이있지만,실제CellUnit554의hard/stored표식은둘다false다(`giant-parent-reset-diagnosis-v137.json`, `giant-parent-reset-trace-v137/trace.log`). 원시첫vpos≤500조건이문단위간격을셀좌표원점으로오인하여되감김검출을꺼버린것이다. 스캐너조건만의문제라는초기가설을실행자료로수정했다.
- v138 후보는셀원점판정에배치가이미소비하는`stored_cell_initial_frame_origin_hu`를재사용하고,기존HWPX 쪽스케일되감김분할을표전체1×1이아닌해당행의단독전폭셀소유로판정한다. 기존70%점유조건·원본좌표·로컬reset투영플래그는유지한다. 실제경로는`cell_units_uncached의원점→reset_before→첫nested_row CellUnit→advance_row_cut_inner→scan/row.rs 및 row_entry.rs→continuation fragment의같은내용컷배치`다. rowspan 블록경로의별도조건까지충족했다고주장하지않는다. 현재후보Native빌드/시각대조전이며,new fixture·회귀기대값·원장허용치를바꾸지않았다.
- v138 Native 빌드는exit0/Cargo표시114초이며동결CLI와source/case 해시를기록했다(`paragraph-leading-frame-origin-candidate-v138.json`). 같은원본의`dump-pages --compat2024 --json`은48쪽을유지한다. 전쪽내용소속진단에서18–25쪽의본문대응은복구됐지만28쪽에다음쪽그림일부가들어가고40쪽의부속서I는여전히Print41쪽내용이다(`giant-leading-origin-pagination-v138/content-page-correspondence.json`). 이문자진단을90%시각일치로보고하지않는다.
- 같은후보의관련439개와기존잔여8개를합친442검사는437PASS/5FAIL/9713skip,실제268.933초/compile포함476.587초,exit100/source·cases불변으로종료됐다(`paragraph-leading-frame-focused-v138.json`). 실패는기존Q8·KPS·가맹사업2개·편람원장partition9이며추가실패는검출되지않았다. 전체release-test최종결과를대신하지않는다.
- v138 Native giant 전48쪽은exit1/871.939초,동결binary/source·cases불변이며36쪽이90%미만,최저40쪽3.85364%다(`native-giant-leading-origin-full-v138-exit.json`, `giant/silhouette.tsv`). 새18쪽review를직접표시했으며같은쪽내용소속은복구됐어도그림·글립위치가달라59.29030%다. 쪽수48과내용대응으로시각미달을면제하지않고새회귀·PR준비완료를보류한다.
- frozen v135 KPS 원본 전77쪽은exit1/2149.350초이며11쪽이90%미만,최저55쪽45.38915%다(`native-kps-full-v136-exit.json`, `native-kps-full-v136/kps1156/silhouette.tsv`). 미달쪽은15/16/17/19/21/27/55/60/65/66/73이다. 새55쪽review를직접판독했으며첫내부표가아래로밀려주석·뒤표와겹친다.37–38쪽의높은점수로전체쪽수검사의교정선행조건을충족한것으로쓰지않고,기존77쪽/행검사는유지한다. 종료된77SVG3.158GiB를PNG/TSV/tree/로그를보존하고삭제했다(`finished-kps-native-svg-cleanup-v139.json`).
- frozen5cc CGMP 일반모드재개는45쪽까지완료후실제2400초timeout으로종료했다(`wasm-cgmp-full-raster-retry-v134-terminal-timeout.json`). 리베이스후main checkout에서의재개는git_head provenance가달라정상거부됐다. 원manifest를수정하지않고실제5cc source bytes와sweep script가같은읽기전용checkout에서재개했으며45개checkpoint재사용을확인했다. 이checkout의누락된puppeteer-core는같은lock의기존Studio node_modules를공유해복구했고마지막46–48쪽을이어처리한다(`wasm-cgmp-frozen-checkout-resume-v140-preflight.json`). 별도Cargo target이나새compile은만들지않고현재v138 검증과frozen5cc 검증을구분한다.
- giant18쪽의`HYSinMyeongJo-Medium`은SVG에서임베드되지않고`src:local(...)`로만남았으며호스트fc-match는Noto Sans를선택했다. 임베드된다른face의cmap통과는이누락face의공급증거가아니다. 기존win10-ted의H2MJSM.TTF에서family/full name은`HYSinMyeongJo-Medium`/`HY신명조`,PostScript는PDF의`H2mjsM`과같음을확인했다. SHAf78f538885b29f956cf84286e430898462d11430604625857ad587c0ea73925b,cmap17368과한국어probe를기록했다(`giant-hys-font-supply-forensics-v140.json`). 폰트는저장소밖별도private 디렉터리에보존하고시스템/기존공급디렉터리는바꾸지않았다. 같은v138 CLI의18/28/41쪽에정확한face를추가해재비교중이며폰트예외나전체48쪽통과로판정하지않는다.
- v138 필수7단계도고정base19245412a에서모두exit0/source·cases불변으로종료됐다(`paragraph-leading-frame-stage-gates-v138.json`): fmt30.712초/Native Clippy96.902초/WASM32 Clippy85.339초/workspace build140.982초/all-target Clippy116.197초/manifest-base6.378초/unit-tier-base27.940초. 종료한원점·행소유보정의부분단계를커밋하고남은실제시각미달을계속보정한다. Native/fresh WASM 전체90%·잔여5FAIL·최종전체Rust/NativeSkia/Studio·초기18개개별review/오늘할일·통합PR/CI/merge/후속처리는완료되지않았다.

### 진행118 — frozen CGMP 전쪽 종료와 정확한 글꼴 공급 확인(v140/v141)

- 원점·행소유단계를2328e0801으로커밋후fetch했다. upstream이19245412a→90fa2f8955c2dd21e9f0ea8fd0c6ee8dc5996a47로실제전진하여local devel을ff하고review를충돌없이rebase했다. 현재code checkpoint는f8b68b6b7ff1b718027ff57043243c1eea76a119이다. Rust source·cases·Cargo bytes는v138과동일하며새base의manifest/unit-tier 비교도각exit0/3.176초·27.217초다(`paragraph-leading-frame-rebase-v141.json`, `paragraph-leading-frame-newbase-policy-v141.json`). #7706 IME 변경은새base의병합내용이며추가체리픽은없다.
- actual frozen5cc checkout의CGMP 일반모드재개는exit0/470.904초/동결산출물해시불변으로종료됐다(`wasm-cgmp-frozen-checkout-resume-v140-exit.json`). 전48쪽metrics의최저47쪽94.05301%,미달/누락0,48/Print48이며canonical gate는passed다. frozen5cc Native전48쪽최저94.00342%와짝을이룬다. 새47쪽 `direct-final/review/review_047.png`를실제로표시·판독했으며표외곽·본문순서·쪽소속은대응하지만제목과일부글자의획·글꼴차이는남는다. 이것을현재f8 code head의fresh검증이나모든glyph의일치로보고하지않는다. 현재CGMP원장복구는최신동일source의Native/fresh전쪽확인후진행한다.
- 완료한CGMP의144SVG3.256GiB를압축없이삭제하고PNG/metrics/manifest/trees/로그를보존했다(`finished-cgmp-full-svg-cleanup-v141.json`). 이번에만든동결checkout도tracked clean·사용중인cwd프로세스0을확인후제거했다(`finished-frozen-cgmp-checkout-cleanup-v141.json`). shared target/node_modules,기본repo,다른worktree와원본입력은보존했다.
- H2MJSM 원파일추가만으로는giant SVG의HYS face가계속local()여서지표가동일했다. 실제`plan_svg_font_file_lookup`는SFNT name table 대신font family/alias 파일명후보를검색한다. 같은font의nameID1/4가요청family와정확히같다는근거로저장소밖파일명을`HYSinMyeongJo-Medium.ttf`로hardlink했다. 원본바이트/글립/name table을편집하지않았고SHA는동일f78f5388…이다(`giant-hys-font-supply-forensics-v140.json`).
- 정확한파일명공급뒤HYS face가실제SVG에임베드됨을확인했다. 18/28/41쪽은59.03128/65.87195/20.28270%,exit1/155.484초/동결CLI·source·cases불변이다(`native-giant-hys-font-selected-v141-exit.json`). 글꼴공급문제는해소했지만실제위치/분할미달은남으며폰트예외로면제하지않는다. 새18쪽review와남은원본28쪽의단일TAC표프레임/29쪽그림의소속을이어대조한다. 기존기대값·허용치·fixture는변경하지않는다.

### 진행119 — 저장 TAC 줄의 표 전체 소유를 보존한 후보와 후속 분할 미달(v142)

- 원본 giant의 문단483은 직전 저장 줄43580+1000HU 뒤에서500HU로 되감긴다. 한 줄67941HU가 표67661HU와 바깥여백140+140HU를 정확히 소유한다. 독립 Print28쪽은 소개까지,29쪽은 그림 표 전체를 담지만 기존 Native28쪽은 표의 첫 두 행을 먼저 선택했다. 원본 XML·기준 쪽 경계·실제 유닛846–851과 호출 경로는 ignored `output/pr-review/semanticist21-20261008/giant-inline-frame-diagnosis-v142/`에 보존했다.
- `stored_inline_row_frame.rs`는 원본의 한 줄 상자가 단일 TAC 표 전체와 일치하는지 확인하고, 기존 행 유닛의 합을 일반 행 scanner에 반환한다. 그 합이 새 밴드에는 들어가고 현재 잔여에는 안 들어갈 때 저장 되감김을 흡수하지 않는다. 편집·재조판·합성 줄·다중 쪽 표·불완전한 자식 행 그룹은 제외한다. 부모는 전폭의 비병합 셀이어야 하며 앞 행의 rowspan 덮임도 제외하므로 블록 scanner의 동등성을 주장하지 않는다. 새 높이·픽셀 컷을 만들지 않고 기존 `CellUnit → advance_row_cut_inner → scan/row.rs, row_entry.rs → continuation`의 동일 컷을 유지한다.
- Native 빌드는 exit0/125.962초, source 불변이며 동결 CLI SHA `ce2ce4c0c96f8f14eb5f968779d41fa752b4bfa7ed530137e7e5cb861d656470`이다(`stored-inline-row-frame-artifact-v142.json`). 관련442개는437PASS/5FAIL/9713skip, 실제241.461초·compile 포함462.948초, exit100이다(`stored-inline-row-frame-focused-v142.json`). 실패는 기존 Q8·KPS·가맹사업2개·편람 원장partition9이며 원장32→35도 유지된다. 원본 입력·기대값·fixture·허용치는 변경하지 않았고, 기존928개 검사 파일 해시의 불변도 별도로 확인했다. 이 결과를 전체 Rust 검증으로 확대하지 않는다.
- 정확한 HYS 공급의 Native 선택18/28/29/40/41/42쪽은59.03128/89.11841/59.70224/27.59721/92.31478/33.87343%, exit1/445.334초다(`native-giant-hys-font-selected-v142-exit.json`). 28쪽의 표 시작 컷은846으로 복구됐지만 전체48→50쪽으로 늘어 Print48쪽과 다르다. 41쪽이20.28270→92.31478%로 개선돼도 직접 표시한 새 `giant/direct/review/review_041.png`에서는 마지막 설계지침과 표 제목이 누락되고42쪽으로 넘어간다. 새42쪽 review도 직접 표시했으며 그 잔여 내용 뒤에 표가 시작해 실제 위치와 다음 내용 소속이 맞지 않는다. 점수 통과만으로 의미 검증을 면제하지 않는다.
- 실제42쪽 경로는 부모 컷1223..1239 → 자식 컷28..44 → 자식 문단17–20 → 손자 표0..12행이다. 다음 보정은 저장 프레임에 남은 빈 밴드·자식 행/rowspan 점유와 paint의 소비를 함께 추적한다. 빈 문단을 지우거나 좌표 clamp로 숨기지 않는다. #7751의 최신 본문도 API로 재확인했으며 Q8 검사를 유지한 이관 기록이므로 문서명만으로 그 실패를 제거하지 않는다(`handbook-q8-exact-transfer-recheck-v142.json`). 현재 후보의 fresh WASM·lint7·최종 전체 검증과 통합 PR 단계는 미완료이며 추가 체리픽은 없다.

### 진행120 — 중첩 행의 초과 허용과 재귀 컷 예약을 함께 보정(v143)

- v142 후보·실패 증거를 일반 커밋 `28bbc6f25c359439dc84013100836f4fa4b02cff`으로 고정한 뒤 fetch했다. upstream은 기존 `90fa2f895`와 같아 rebase하지 않았다. 원본70행 표의33행은 Print38쪽 말미,34–68행은39쪽,69행의 주석은40쪽이다. 주석 행은 원본11450HU·저장7줄을 가진 온전한 표 행인데, `visible_tail_fits_before_spacer`가 가시 꼬리 글줄처럼 예산1005.4px를 넘어 수용했다. 실제 scanner의 컷1173/높이1103.2px 뒤에 paint 높이 재시도가 컷1170으로 되돌려, 들어가는67–68행까지 주석과 함께 다음 쪽으로 밀었다(`giant-inline-frame-diagnosis-v142/after-scan.trace.log`, `mechanical-source-rows.json`).
- 그 초과 허용에서 `nested_row`를 제외했다. 일반 글줄·캡션 꼬리의 기존 조건·공차는 그대로다. 또 직접 HWPX가 자식 RowCut을 전달하도록 바뀐 v135 이후에도 `mixed_nested_flow_extra_from_cut`가 종전 scalar 첫 유닛을 추가 예약하던 소비를 고쳤다. 원본 HWPX의 전부 재귀인 run도 HWP5처럼 같은 자식 컷의 실제 높이로 예약하며, 편집·비재귀의 기존 계약을 바꾸지 않았다. 컷·높이·배치 연결과 반례는 `stored-nested-row-budget-diagnosis-v143.json`에 기록했다.
- Native 빌드는 exit0/83.832초, 동결 CLI SHA `05d7524c9147d6b33482738d73d7a95f18e13af5be50bca0b8f48e2547594638`이다. 기존6원본 진단은 giant48쪽/넘침0, press3637 13/0, external1891 74/10, virtual6776 75/0, water6776 3/0, CGMP48/0이다(`stored-nested-row-budget-counters-v143/results.json`). giant50→48은 회복됐지만 이 건수를 시각 통과로 쓰지 않는다. 관련442개 재검사는437PASS/기존5FAIL/9713skip, 실제233.973초·compile 포함425.150초, exit100/source 불변이다. 기존928개 검사 파일 해시도 불변이며 새 검사·fixture·기대값·원장 허용치를 추가·변경하지 않았다.
- 정확한 HYS 공급의 Native18/28/29/39/40/41/42/45/46/48쪽은59.03128/89.11841/59.70224/45.82993/37.58126/98.70584/53.66499/22.83808/83.80848/99.68394%, exit1/648.712초, 전체48/Print48이다(`native-giant-hys-font-selected-v143-exit.json`). 새41쪽 review를 표시·판독해 마지막 설계지침과 표 제목의 같은 쪽 복구를 확인했다. 새39쪽은 같은34–68행이지만 위로 밀린 괘선·본문과 말미 빈 밴드 누락이 남는다. 새42쪽에는 소개 잔여가 없고2.4항도 표시되지만 병합 셀 제목이 Print의 세로 가운데와 달리 위에 놓이며, 괘선·말미 내용 차이가 남는다. 점수 통과와 실제 의미를 구분한다.
- 다음 실제 경계는 첫 저장 TAC 줄의 원점·부모 물리 안 여백과 병합 셀의 조각 정렬이다. 18쪽 실제 Print PDF 이미지 행렬과 Native print SVG를 직접 읽어 이미지 상단98.132/87.9px, 본문5.5.1.3 기준선816.38780/813.5px의 차이를 기록했다(`diagram18-print-svg-geometry-v143.json`). 이 좌표를 새 offset·clamp·기대값으로 주입하지 않고 저장 줄·측정·paint 소비를 추적한다. 종료된 이전 giant60SVG 약1.44GiB를 압축 없이 제거하고 PNG·TSV·tree·원본·Print를 유지했다(`finished-giant-native-svg-cleanup-v143.json`). 현 후보의 lint7/fresh WASM·전체 최종 회귀와 초기18개 PR 문서/오늘할일·PR/CI/merge/후속 처리는 아직 완료되지 않았다.


### 진행121 — 아래쪽 행 컷과 내용 잘림을 구분해 병합 셀 정렬을 보정(v144)

- 실제 경로를 GDB로 확인했다. giant 원본의26×9 표 row12/col0/rowspan4에는 `CENTER`와 한 줄의 “2.4 광커플러”가 저장돼 있다. `layout_table_cells`의 `row_filter=Some((0,15))`는 아래쪽 행만 자르지만, 실제 현재 셀 안 높이225.760px에 전체 내용14.667px가 들어간다. 기존 코드는 행 범위만으로 `effective_valign=Top`을 강제했다. 부분 셀 함수의 추정 경로 대신 이 실제 소비 지점을 보정했다(`giant-align-forensics-v144/full-gdb.log`).
- 최종 저장 흐름·그림 프레임·문단 간격을 반영한 `total_content_height` 이후에 정렬을 결정한다. 아래쪽 행 컷은 내용이 현재 안 높이를 넘을 때만 Top으로 수렴한다. 위쪽 이어받기·scalar continuation·실제 부모/페이지 viewport 잘림은 기존 조건을 유지한다. 현재 셀 상자와 컷·예약 높이는 바꾸지 않고, 같은 내용 높이가 정렬 여유와 실제 `text_y_start`에 소비된다. 문서 ID·좌표 offset·새 clamp·공차·기대값 변경은 없다.
- 수정 후 같은 실제 호출에서 `effective_valign=Center`이며 라벨 줄 상단은841.3→946.8px, 현재 셀 상자는837.5/233.3px로 동일하다. 긴 조건 본문의 줄 위치도 동일하다. 이 관측값을 회귀 기대값으로 추가하지 않았다. 독립 Print42의 라벨 기준선961.76px도 별도로 읽었다(`after-gdb.log`, `label42-before-after-geometry.json`, `label42-independent-print-text.json`). 새42쪽 review를 사용자에게 표시하고 직접 판독해 라벨과 짧은 기호의 가운데 정렬을 확인했다. 표 괘선·본문의 위쪽 밀림과 활자 차이는 남아 있다.
- Native 빌드는 exit0/101.719초, source 불변, 동결 CLI SHA `1c21d1b8c91ce8c504660194efb3e26c97ee19bc1d2740a59827873adc5a8f5c`다. 빌드 출처는 `02b534eb0` 위 이번 한 production 파일 변경이며 커밋될 소스 해시를 `row-filter-content-align-artifact-v144.json`에 고정했다. 관련 기존442개에 정렬·실제 clip 대조9개를 합친451개는446PASS/기존5FAIL/9704skip, 실제244.998초·compile 포함444.509초, exit100이다. 원본 case 파일929개(README 포함)는 실행 전후 불변이다. #4068의 부모/페이지 clip·완전 셀 정렬과 #7066/#5593/#6569의 기존 정렬 대조도 통과했다. 기존 실패는 편람 셀 원장32→35, Q8/KPS 쪽·행 pin, franchise 두 쪽수 pin으로 이전과 같다. 새 회귀·fixture·baseline·허용치를 추가/변경하지 않았다(`row-filter-content-align-focused-v144.json`).
- 정확한 HYS 공급의 Native 선택39/40/41/42/43/45/46쪽은45.82993/37.58126/98.70584/58.89286/36.19821/22.83808/83.80848%, exit1/445.039초, 전체48/Print48이다. 42쪽은53.66499→58.89286%로 개선했으나6/7쪽이90% 미만이다. 최저45쪽22.83808%이며 `re_review_required`다. 새39/42/43쪽 `direct/review/review_*.png`를 표시·직접 판독했다. 39쪽의 말미 빈 밴드 누락과43쪽의 반복 표 머리 누락·행 이동은 남아 있다. 동결v143 CLI로 새43쪽 tree를 별도 출력해 v144와 전체 tree 및173개 text run/순서/좌표가 동일함을 확인했다. 위쪽 이어받기의 이번 무변경과 기존 결함을 구분하며 새 golden으로 삼지 않는다(`giant-align-forensics-v144/page43-before-after.json`).
- 다음 보정을 위한18쪽 첫 TAC 실제 경로도 별도 추적했다. root5×1 row4 para318은 cut547..554에서 시작하고 원본 host 줄의vpos500HU를 `frag_vpos_origin`으로 잡는다. 선형 vpos 보존/앞 텍스트/float frame이 없는 경로는 `nested_y=inner_area.y=77.47333px`로 그려 원본 첫 슬롯의 문단 앞 간격이 사라진다. source49542HU 줄은 child49260HU와 바깥141+141HU로 닫히지만, para652의1200HU 줄은 큰 자식 표 전체와 일치하지 않으므로 같은 가정을 적용할 수 없다. 후속 문단 스냅과 실제 flow 소비를 함께 확인할 다음 근거이며, 이번 단계에서 원점·패딩 보정을 추정으로 넣지 않았다(`host18-gdb.log`, `row-filter-content-align-diagnosis-v144.json`).
- `cargo fmt --all -- --check` 및 `git diff --check`에서 오류가 없었다. 이번 소스의 필수 lint7/fresh WASM·전쪽 최종 회귀/시각 승인, 초기18개 개별 review·오늘할일과 PR/CI/merge/후속 처리는 아직 미완료다. 선택쪽 개선을 전48쪽 또는 전체 작업 승인으로 확대하지 않는다.


### 진행122 — 첫 원본 TAC 호스트에 예약된 앞 간격을 배치에서 소비(v145)

- 18쪽 root5×1 row4/para318의 실제 측정·배치·뒤 문단 소비를 한 실행에서 대조했다. 원본49542HU 줄은 child49260HU와 바깥141+141HU로 닫힌다. 측정 유닛676.82667px에는 앞 간격6.66667px와 표660.56px·저장 줄간격9.6px가 들어 있지만, 실제 첫 표는 `inner_area.y=77.47333px`부터 배치됐다. 표 뒤 흐름738.03333px는 다음 저장 문단 스냅754.3px보다 작다. 이 결과로 전체 조각 원점을 바꾸거나 뒤 본문을 함께 옮길 필요가 없음을 확인했다(`giant-host-slot-forensics-v145/before18-gdb.log`).
- 기존 행 프레임 판별의 원본 줄/객체 닫힘 검증을 `stored_inline_owner_frame`으로 공유했다. 그 행 유닛 합·쪽 시작 판별은 기존 계약을 유지한다. 부분 배치는 실제 컷의 첫 소유 유닛에 예약된 앞 간격을 읽어, 앞 텍스트가 없는 표 호스트에만 소비한다. 저장 줄 상단이 해당 간격으로 설명되는 원본 슬롯만 수용하며, 원본이 dirty/reflow/synthetic이거나 첫 내부 행을 이미 소비했거나 mixed 조각이면 제외한다. 기존 선형 vpos 스냅도 재가산하지 않는다. atomic 유닛은 원본 줄 높이와 앞 간격을 실제 예약 높이가 수용하는지 확인한다. 문서 ID·프레임 원점 변경·공차 확대·추정 패딩/좌표 offset은 없다.
- 수정 후 같은 실행에서 lead=6.66667px, 표 시작84.14px/뒤 흐름744.7px이며 원본 측정 유닛676.82667px는 동일하다. 다음 문단 스냅754.3px와 뒤 본문 시작793.23333px도 동일하다. 실제 최종 tree의 내용/순서는 보존됐고 바뀐13개 leaf는 모두 이 표 호스트의 그림·설명에 속한다. 다른 본문 leaf의 좌표는 동일하다(`after18-gdb.log`, `page18-before-after.json`). 진단 결과를 새 fixture/golden/assertion으로 추가하지 않았다.
- Native 빌드는 exit0/113.130초, source 불변, 동결 CLI SHA `d0a0b4f2e749aca5d0a57f5abe85f662e47dbfb57dbd87209e557fd98002d4fc`다. 출처는 `62530502c` 위 이번 두 production 파일 변경이며 `stored-inline-owner-lead-artifact-v145.json`으로 해시 고정했다. 기존451개 검사는446PASS/기존5FAIL/9704skip, 실제244.218초·compile 포함457.237초, exit100/source 및929개 case 파일 불변이다. 실패는 이전과 같은 편람 셀 원장32→35, Q8/KPS 쪽·행 pin, franchise 두 쪽수 pin이다. 원본·회귀·baseline·공차 변경은 없다(`stored-inline-owner-lead-focused-v145.json`, `stored-inline-owner-lead-case-snapshot-v145.json`).
- 6원본 쪽수/셀 넘침 진단은 giant48/0, press3637 13/0, external1891 74/10, virtual6776 75/0, water6776 3/0, CGMP48/0으로 이전과 동일하다. 이 export는 compat2022의 runtime 진단이며 독립 Print 시각 승인과 구분한다(`stored-inline-owner-lead-counters-v145/results.json`).
- 정확한 HYS 공급·독립 Print2024의 Native 선택18/28/29/42쪽은76.02306/89.11841/88.28464/58.89286%, exit1/343.733초, 전체48/Print48이다. 역사 v143의18쪽59.03128%,29쪽59.70224%에서 개선됐다. 28쪽은 같은89.11841%이며, 42쪽은 v144 새 raster와 바이트 동일하다. 새18/28/29쪽 review를 사용자에게 표시하고 직접 판독해 그림·설명과 본문 순서 보존을 확인했다.18쪽 회로 그림과 본문 위치 차이,29쪽 작은 오른쪽 표식 잘림·그림/본문 위치와 활자 차이는 남는다. 비교4쪽 모두90% 미만으로 `re_review_required`이며 원본 전체48쪽/fresh WASM 승인으로 확대하지 않는다.
- 다음 진단을 위해29쪽 실제 SVG clip/그림 ancestry와 같은 Print의3개 이미지 행렬을 별도 추출했다(`page29-svg-clips.json`, `page29-picture-print-svg-ancestry.json`). 현재 child6×2 표는 root para483이 소유하며 두 번째 그림의 가시 오른쪽 끝과 셀 clip을 대조할 근거다. 원인을 확정하거나 crop 수정 완료로 기록하지 않는다. 첫 표 위치·소유 관계가 다른 source para652의1200HU 줄은 큰 자식 표 전체를 닫지 않으므로 이번 앞 간격 계약의 근거로 사용하지 않았다. 부모의 실제 물리 패딩과 다음 이어받기 표 머리도 여전히 별도 미해결이다.
- formatter 후 빌드했고, 검증 종료 후 `cargo fmt --all -- --check`와 `git diff --check`도 exit0이다. 이번 source의 lint7/fresh WASM·최종 전체 회귀·전쪽 시각 승인, 초기18개 개별 review/오늘할일·PR/CI/merge/후속 처리는 미완료다.


### 진행123 — 반복 제목행의 실제 예약 검증과 원본 행 컷·열 폭 추적(v146/v147)

- v146은 원본 `leading_header_rows`와 최종 행 높이를 함께 소비해 반복 제목행 공간을 예약하고 배치했다. 기존 상위 recursive reserve 재시도가 이 물리 공간을 전파한다. 실제 GDB에서 grandchild26×9의 제목행0/1 높이는87.09333px이며 독립 Print43의86.6253→173.7293px, 약87.104px와 대응한다. 실제 선택 범위는15..23→15..22, root 원본 컷1242..1250→1242..1249로 바뀌었다. root의 raw flow882.0px에 제목행을 한 번 반영한 실제 child969.09333px와 root978.52px 상자도 확인했다(`giant-repeated-header-forensics-v146/after43-gdb.log`, `after-root43-gdb.log`, `print43-header-rule-geometry.json`).
- Native 빌드는exit0/178.899초, source 불변, 동결 CLI SHA `c4840671f2bb297b0eeebbc5a9b941eb5c04359a38d476a0ab9530700b7ed1c7`다. 기존451개는446PASS/동일5FAIL/9704skip, 실제238.826초·compile 포함724.779초, exit100/source 및929개 case 불변이다. 같은6원본의 쪽수/셀 넘침 진단도v145와 동일하다(`nested-header-frame-artifact-v146.json`, `nested-header-frame-focused-v146.json`, `nested-header-frame-counters-v146/results.json`). 종료된 counter261SVG75,033,523바이트만 압축 없이 제거했고 다른 산출물은 불변이다.
- 정확한 HYS와 같은 독립 Print2024의 새42/43/44/45쪽 Native는58.89286/44.76949/32.51681/25.54481%, exit1/313.552초, source 및동결CLI 불변이다. 새43/44/45쪽 review PNG를 실제 표시·판독했다.43쪽의 반복 표 머리와44쪽의3.4 퓨즈 소속은 복구됐지만, 제목행 공간 예약 뒤44쪽의 PCB 행 전체가45쪽으로 이동해44쪽 하단이 비었다. v145의새44/45쪽 tree도 따로 출력해 변경 전44쪽은23/24행,45쪽은25행이었음을 확인했다. 이 새 시각 문제를 기존 무변경으로 분류하지 않는다.42쪽 새 PNG는v144와 바이트 동일하다. 모든 선택쪽이90% 미만이며 전체48쪽/fresh WASM 승인은 아니다.
- 원본 PCB 행24/col6의 저장 문단10은 같은 문단의 textpos98에서vpos40960→0HU로 재시작한다. 실제 셀 유닛33개 중 index27에는 `hard_break_before=true`, `page_frame_reset_before=true`, `vis_start=5`가 이미 있다. 그러나 부모 행 생산의 `row_is_auto_height=false`, `driver_is_nested_table_rows=false` 조건이 이 행을 단일 원자로 묶어 저장 경계를 상위 컷에서 감췄다. 문단마다0에서 시작하는 로컬 좌표, synthetic/편집 줄과 구분한 원본 문단 내부 재시작을 기존 `NestedTableUnitCut`에 연결하는 v147 추가 보정을 진행한다(`source-row-cut-contracts.json`, `source-row24-driver-units-gdb.log`). 기대값·fixture·공차·새 회귀는 추가하지 않았다.
- 같은 Print의 열 끝711.2973px와 원본 grandchild 선언폭46493HU를 대조했다. `layout_table`의 기존 nested fit은 부모의 안쪽 내용 폭을 기준으로 source 열 구획을 줄이는 반면 측정은 원본 열 폭을 사용한다. 원본 표가 동일한 최종 원점에서 부모 물리 셀 안에 들어가는 경로는 원본 폭을 보존하고, 부모 셀을 넘거나 편집/재조판한 경로는 기존 fit을 유지하도록 v147에서 함께 보정한다. 원점은 기존 계산 결과를 재사용하며 PDF 좌표·문서ID·추정 offset을 코드에 넣지 않는다. source 여백/물리 셀 차이와43쪽 누락된1000·범주III 표식, 이후 모든 쪽의 소속·누락·중복은 추가 확인 대상이다(`source-header-owner-chain.json`, `print42-44-top-border-segments.json`).
- v147은 아직 빌드/시각/관련 회귀 전 후보다. v146 결과를 이 추가 변경의 통과 증거로 쓰지 않는다. 현재 단계는 미완료이며 최종 전체Rust·lint7·Native/fresh WASM 전쪽90%와 초기18개 개별review/오늘할일·통합PR/CI/merge/후속 처리는 계속 보류한다.

- v147 묶음 빌드는exit0/114.484초, source 불변, 동결CLI SHA `66cbfa6c25baddbadabe8e85a8034dab34486760bd2bc6af718a2748f86f14c1`다. 원본 행/폭·클립·정렬 경계의 기존38개가38PASS/2860skip, 실제83.176초·compile 포함250.461초, exit0/source와929개 case 불변으로 종료됐다. v146의451개 결과를 v147 전체 검증으로 재인용하지 않는다(`stored-nested-row-cut-width-artifact-v147.json`, `stored-nested-row-cut-width-boundary-v147.json`). 6원본 runtime 쪽수/셀 넘침 진단도 이전과 동일하다. CGMP 원본48쪽/0넘침을 전쪽 Native/fresh 시각 통과로 확대하지 않는다.
- 실제 GDB에서 행24의 원본 내부 컷 수용은true, crossing rowspan은false, driver6이다. original width 수용도true이며 원본619.90667px와 이전 fit615.69333px, 부모 셀83.10667/631.16px 및 내용 영역89.90667/617.56px를 한 호출에서 확인했다(`giant-repeated-header-forensics-v147/row-cut-width-gdb.log`). 같은 원점에서 원본 표 오른쪽은 부모 물리 셀 안에 있으며, 실제 paint 폭도619.9px로 보존됐다.3.6 PCB 행을 새44쪽에서 일부 소비하고45쪽에서 남은 내용을 이어받는다.43쪽의1000 표식도 새tree/PNG에 복구됐다(`selected43-45-owned-cells.json`).
- 정확한 HYS·독립 Print2024의 새42/43/44/45쪽은60.04321/54.24175/41.45521/40.95507%, exit1/242.800초, source/동결CLI 불변이다. 새43/44/45쪽 review PNG를 표시·직접 판독했다. 모든 선택쪽이90% 미만이다.43쪽의범주III 반복 표식,43→44쪽 비고 끝의 “한다.” 소속과44→45쪽 PCB 문단의 원본 재시작 첫 줄 소속은 아직 다르다. 표의 위 원점/물리 안 여백과 조각 끝도 Print와 어긋난다. 이 차이를 폰트·같은 쪽수·경계38PASS로 면제하지 않는다. `cargo fmt --all -- --check`와 `git diff --check`도 통과했다. 부분 보정 단계를 일반 커밋으로 고정한 뒤 새 실제 HEAD 변화가 있을 때만 rebase하고 후속 실제 차이를 계속 보정한다.


### 진행124 — 실제 자식 컷이 소유한 문단 내부 재시작을 상위 투영에 보존(v148)

- v147 부분 단계를7b5e5f68e62b491f954e40ee35a699874291c19c으로 커밋 후fetch했다. upstream/devel과local devel은90fa2f8955c2dd21e9f0ea8fd0c6ee8dc5996a47로 같아rebase하지 않았다. 커밋된source bytes는v147 artifact와 같고 당시 작업 트리는clean이었다. 종료된v146/v147 Native와v147 counters의269SVG232,514,875바이트를 압축 없이 제거했고,4쪽 clip 기하와PNG/TSV/tree/로그는 보존했다(`finished-stored-nested-row-cut-width-checkpoint-v147.json`).
- v148의 실제 원본 추적은exit0/21.303초다. root5×1 row4/para652 중 grandchild 표를 투영한 source_para20 유닛은hard/stored/reset=false였다. 소개 문단의1195 유닛은이전부터hard=true이며이번에변경되지않았다. 새44쪽 컷은1257..1288, raw flow911.66667px, root 상자1008.18667px이며, 원본PCB 줄의 재시작을 넘어 다음 문단 첫 줄까지 소비한다(`giant-source-reset-ownership-forensics-v148/root44-reset-units-gdb.log`). 같은 모델의grandchild 행24/col6 leaf index27은문단10/vis5 원본40960→0HU,hard/reset=true다. 생산 시사실이direct-HWPX `nested_table_mixed_fragment_heights`의고정false 대입에서사라졌다. 독립Print44/45의그첫줄소속과직접확인한v147 review 차이를근거로이경로를보정한다.
- 새 helper는이미선택한한자식유닛의실제`NestedTableUnitCut` 또는recursive RowCut 시작을원본글줄까지추적한다. 같은원본의textpos가전진하면서동일문단의vpos가0으로재시작하는지,단일행의원본/dirty/synthetic 상태를기존행분해판별과공유한다. 블록컷인덱스를셀유닛으로추측하지않는다. direct HWPX 투영은이원본시작컷에서만hard/stored 표식을보존하고,문단마다0에서시작하는로컬좌표와재조판경로의계약은그대로유지한다. 흐름높이·본문좌표·예산공차·문서ID를변경하지않는다. 경로는원본leaf→nested typed cut→1×1 direct 투영→root cut 선택→같은recursive cut의paint이며추가깊이도동일실제cursor를소비한다.
- 아직빌드/경계/새Native 비교전후보다. 새회귀/fixture/golden이나기대값/허용치는추가·변경하지않았다.43→44 비고끝소속과44→45 PCB문단첫줄소속,본문점유·뒤내용·원본누락/중복을다시대조한다. 남은표원점·패딩과전체Native/fresh90%·전체회귀/lint 및초기18개review/오늘할일·PR/CI/merge는미완료다.

- v148 Native 빌드는exit0/113.455초,source 불변이며동결CLI SHA `fd48ee37d66180afcab9bda7368dfbd0bdfebe0215b1c43abbb792ba617454e4`다. 기존경계38개는38PASS/2860skip,실제81.699초·compile 포함246.422초,exit0/source 및929개 case 불변이다.6원본runtime 쪽수/넘침도전부직전과같다(`stored-nested-source-reset-artifact-v148.json`, `stored-nested-source-reset-boundary-v148.json`, `stored-nested-source-reset-counters-v148/results.json`).
- 같은호출의root 유닛비교에서1256/1286의hard/stored만false→true로바뀌고모든유닛의높이·서수·소스소유는동일하다. 소개문단1195의원래hard=true는그대로다. 새44쪽컷은1256..1286,raw flow886.10667px/root 상자980.74px다(`root-source-unit-flags-before-after.json`, `after-root44-reset-units-gdb.log`). 새tree에서44쪽비고의“한다.”,45쪽PCB문단의“다.”및다음문단이원본소속으로복구됐다.
- 정확한HYS·독립Print2024의새42/43/44/45쪽Native는60.04321/54.17450/44.46360/48.99701%,exit1/232.936초/source 및동결CLI 불변이다. 새43/44/45쪽review를실제표시·판독했다.43쪽의범주III 표식과위쪽원점·활자·행끝차이는남아있다. 모든선택쪽이90% 미만으로최종통과가아니다.38개경계결과를이전451개또는전체Rust/freshWASM승인으로확대하지않는다.
- 다음여백보정의실제생산→소비도별도추적했다(`giant-source-padding-forensics-v149/measure-paint-padding-gdb.log`): root는원래283+283HU 안여백에대해측정의추가예약3.77333px를계상하지만실제부분paint는상하각1.88667px만쓴다. 실제셀980.74px·원래논리행43512.61333px에도선언566HU만으로여백을절반으로축소한다. middle1×1도실제셀975.08px·원래논리행4591.53333px인데선언282HU로141+141HU를각0.94px로줄인다. 두상자는실제확장한행의안여백을예약·배치에서함께소비해야한다. 근거없는offset을넣지않는다.
- 원본bodyhost는폭0/vpos0/lh1200HU의개체앵커이고표본체는73450HU,위·아래바깥여백은각141HU다(`source-root-host-box.json`). 이앵커의작은줄상자가큰표의바깥프레임을소유한다고보는첫줄조건과,중첩recursive cut의바깥여백예약·실제원점소비를다음단계에서대조한다. 아직여백수정전이며현v148시각차이를해결됐다고보고하지않는다.


### 진행125 — 확장된 원본 중첩 행의 안 여백을 컷과 배치에서 공유(v149)

- 직전 v148을07806b486으로 커밋했고 작업 트리 clean 및 Cargo 종료를 확인했다. 현재 giant 원본/Print의44쪽 측정 예약과 부분 배치에서 실제 안 여백 차이가 확인된 범위를 수정한다. root의283+283HU, middle의141+141HU는 이미 확장된 원래 논리 행 안에 들어가지만 부분 배치가 작은 선언566/282HU로 다시 축소했다. 기하 근거는 `giant-source-padding-forensics-v149/measure-paint-padding-gdb.log`와 독립 Print42–44 외곽선 비교다.
- 공통 helper는 원본 RowBreak 단일 행에서 실제 저장 줄을 가진 중첩 표 호스트만 선택하고, 동일 행 생산자의 확정된 전체 물리 높이를 소비한다. 편집/dirty/synthetic/재조판 및 병합 셀은 제외한다. 원래 행에도 들어가지 않는 비정상 여백은 기존 방어를 유지한다. 조각마다 달라지는 현재 조각 높이를 사용하지 않는다. 원장이 아직 없으면 같은 원래 행 resolver로 한 번 마련하고 호출자의 선언 신뢰 정책을 복구한다.
- 생산 `resolve_row_heights_trusting_declared`/원래 행 원장 → `row_cut_content_height`·남은/위 여백 예산 → 전체/부분 셀 배치로 연결한다. recursive 부모의 별도 여백 보충도 같은 유효 여백을 빼므로 이중 예약을 피한다. 현재 행/블록 유닛의 내용 및 source cut 소유를 변경하지 않는다. 새 회귀·fixture·기대값·공차는 추가하지 않았다.
- 후보 빌드와 기존 경계38개·6원본 수량 및 정확한 HYS/Print2024 선택42–45쪽을 한 묶음으로 실행한다. 아직 검증 전이며 기존 결과를 이번 source의 통과로 쓰지 않는다. 바깥 프레임 여백과 범주III 표식, 최종 전체 Rust/lint/Native/fresh 및 초기18개 review/오늘할일·PR/CI/merge는 남아 있다.

- v149 Native 빌드는exit0/107.308초,source 불변이며 동결CLI SHA `e2db2e6b0f90ffd8bb1652ddcd7ab4ed6abe26f08e78c44618c858b8ac2f1c82`다(`stored-nested-padding-artifact-v149.json`). 기존451개와38개 경계 필터를 중복 제거한476개를 한 묶음으로 실행했고471PASS/기존5FAIL/9679skip,실제281.398초·compile 포함623.143초,exit100/source 및929개 case 불변으로 종료됐다(`stored-nested-padding-focused-v149.json`). 실패는Q8/KPS 고정 쪽·행,가맹산업 두 고정 쪽수,편람 셀 넘침32→35로 이전과 동일하다. 전체release-test를 이번 결과로 대신하지 않는다.
- 실제44쪽 부분 배치는root 상하각3.77333px와middle각1.88px를 사용하고root의예약상자는980.74→982.62px다. 원래 논리행43512.61333/4591.53333px는 같으며재귀부모의별도여백보충은0이됐다(`giant-source-padding-forensics-v149/after44-measure-paint-padding-gdb.log`). 처음진단의CLI `--page44`는0부터세어45쪽을출력했으므로그로그를44쪽증거로쓰지않고 `--page43`으로재실행했다. 이명령오류는제품회귀와구분하고두실행모두보존한다.
- 6원본runtime 쪽수/넘침은giant48/0,press3637 13/0,external1891 74/10,virtual6776 75/0,water6776 3/0,CGMP48/0으로전부직전과같다(`stored-nested-padding-counters-v149/results.json`). 독립Print시각승인이아니다.
- 정확한HYS·독립Print2024의새42/43/44/45쪽Native는60.00330/65.01285/51.86651/62.98848%,exit1/280.980초/source 및동결CLI 불변이다. 새43/44/45쪽review를실제표시·직접판독했다.43쪽범주III가Print와같은쪽으로복구됐고44·45쪽문단끝소속은유지됐다.44/45쪽은TextRun의내용·소유·순서가직전과동일하며42→43은표식1개이동으로204/165→203/166개다(`text-owner-order-before-after.json`). 모든선택쪽은90% 미만이며,위쪽원점·글줄/활자차이와표외곽차이가남는다. 원본전체48쪽/freshWASM승인으로확대하지않는다.
- 다음바깥프레임진단의실제host를원본ZIP에서추출했다(`giant-source-padding-forensics-v149/source-frame-hosts.json`). root는빈폭0/vpos0/lh1200HU,중간표host는빈폭0/vpos500/lh1200HU,grandchild는공백2개/폭46316/vpos30744/lh1200HU다. 셋모두para상단·flowWithText이며각각141/140/140HU 바깥여백을가진다. 작은호스트줄을큰표프레임의여백소유자로대신해도되는지와recursive커서에서누락한바깥여백을추적한다. 이추출은수정완료증거가아니다.
- 종료된v148 Native/counters와v149 counters의526SVG228,754,431바이트,추가로종료된v149 Native4SVG78,741,822바이트만압축없이제거했다. 기하·해시를보존했고원본/Print/PNG/TSV/tree/로그/동결CLI/공유target은유지했다(`finished-stored-nested-padding-svg-cleanup-v149.json`, `finished-native-stored-nested-padding-svg-cleanup-v149.json`). 필수lint7/freshWASM·최종전체회귀/시각검증및초기18개개별review/오늘할일·PR/CI/merge/후속처리는미완료다.

- 비정상 여백 방어·정상 조밀 행·선언 높이의 안 여백 소유를 확인하는 기존5751/3386 대조3개는3PASS/470skip,exit0/9.477초(실제0.590초),source 및929개 case 불변이다(`stored-nested-padding-guard-controls-v149.json`). `cargo fmt --all -- --check`와`git diff --check`도exit0이고현재source 전체해시는빌드artifact와같으며새82줄helper도해시원장에포함돼있다. 현재단계만일반커밋으로고정하며최종전체통과로쓰지않는다.


### 진행126 — 원본 중첩 컷의 바깥 프레임 예약과 원점 소비(v150)

- 직전v149는bc01c5903으로커밋했고fetch된upstream/devel은90fa2f895로동일해rebase하지않았다. 작업트리clean·Cargo종료를확인하고다음보정에들어갔다. 직전476개471PASS/기존5FAIL과새42–45쪽Native미달을현재수정전증거로보존한다.
- 독립Print42–44의root/middle/grandchild위괘선과원본바깥여백141/140/140HU,직전실제배치의원점을대조했다. 원본작은호스트줄은표전체프레임을담지않으며중첩typed/recursive컷배치는원본host없이바깥위여백을열지않았다. 원본유닛생산은첫/마지막유닛에만위/아래여백을싣는다(`source-frame-hosts.json`, `print42-44-top-border-segments.json`). 전체표배치는별도로바깥여백을소비하므로컷경로의누락을대조한다.
- 새공통판별자는원본직접HWPX·빈/공백단일호스트·실제저장줄·비TAC/문단상단흐름표의작은앵커와큰표프레임을구분한다. HWP5유래HWPX·편집/dirty/synthetic·본문텍스트·표전체를담는저장줄은기존계약이다. root본문상단의예산/배치는같은판별자를사용한다. 중첩은실제선택원본run의첫/마지막유닛이이미소유한여백을중복하지않고중간컷이새로여는몫을추가예약한다.
- 생산 `cell_units_uncached` 첫/마지막여백 → `mixed_nested_flow_extra_from_cut`의같은source run·recursive부모예약 → 실제child RowCut·`resolved_table_top`의위바깥여백 → 부분표최종원점으로연결한다. 이미stored float계획을소비한원점은재가산하지않는다. root는prepare/continuation budget/부분paint의동일판별을연결하고captionedwhole-fit조회도같이유지한다. 문서ID·추정offset·기대값·공차·새회귀/fixture는추가하지않는다.
- 후보빌드후직전476기존검사와3여백대조·6원본runtime·정확한HYS/Print2024선택42–45쪽을같은source로묶어확인한다. 아직검증전이며이번source의전체Rust/lint/freshWASM또는18개검토완료로쓰지않는다.

- 첫v150 Native 빌드는exit0/120.777초·source불변·동결CLI SHA97621775c5270c6797b5bcd69f7be9e2008e62aef567e63acedc2a51b3b12c54다. 아직회귀/시각검증전이며다음추가연결의검증으로쓰지않는다. 전체paint실제호출을대조해일반 `layout_table_cells`의안여백소비가v149에서빠졌음을확인했다. v149는TAC전체셀과부분배치를연결했지만일반전체셀8887경로는선언가드를다시썼다. 이번묶음에같은helper를연결하고,block컷frame·block높이·원본열린컷의종료여백회수도같은유효여백을사용하도록추가연결한다.
- 추가연결을포함한후보를v151로구분한다. 한묶음의기존필터476+여백대조3=479개·6원본runtime·Native42–45쪽을새후보에서실행한다. 새source/회귀/fixture/기대값을고정하기전Native/fresh90%와직접판독조건은그대로이며새회귀는추가하지않는다. 원래선언여백방어와원본유닛의내용생산은전역변경하지않는다.

- v151 Native는exit0/51.781초·source불변·동결CLI SHA cb5c9764ed7e18d8a8bf225936f7dc90e8ab6596a95ed670389ede84c3c3eaed다. 이어block잔여높이의실제scan호출까지대조해동일안여백helper를한곳더연결한v152로최종묶음을동결한다. 이번원본의바깥프레임회계는일반RowCut·typed/recursivechild경로의실제증거로판정하며,rowspan블록의깊은중첩바깥프레임회계는독립반례없이는검증완료로쓰지않는다.

- v152 Native 빌드는exit0/48.725초·source불변이며 동결CLI SHA `b6e84ce4e7dad762502e611cadd8331ff74f93a3cc4a71b6e82c72d3c0d3404d`다. 기존479개 묶음은474PASS/직전과같은5FAIL/9676skip, 실제286.351초·compile포함567.8초,exit100/source및929개case불변으로종료됐다(`stored-source-outer-frame-artifact-v152.json`, `stored-source-outer-frame-focused-v152.json`). 편람Q8/KPS행핀/가맹사업두쪽수/편람넘침partition9가실패하며새실패는없다. 최종전체회귀/lint/fresh를대신하지않는다.
- 같은원본SHA5d7eb4a21e46…와독립Print2024/정확한HYS의새42/43/44/45쪽Native는88.49199/95.60783/79.43550/99.83296%,전체48/48쪽,exit1/279.065초·source/동결CLI불변이다. 같은canonical232마스크·2px반경·32색차이며이진화경계보정픽셀은전부0이다. `native-giant-stored-source-outer-frame-selected-v152/giant/direct/review/review_042.png`부터045까지새로생성해실제표시·판독했다.43쪽범주III와45쪽PCB꼬리·다음본문소속은대응하지만42쪽아래경계·44쪽PCB조건문의줄바꿈/아래글줄차이가남는다. 선택4쪽을전48쪽Native/fresh승인으로확대하지않고gate보류를유지한다.
- 6원본runtime쪽수/넘침은giant48/0,press3637 13/0,external1891 74/10,virtual6776 75/0,water6776 3/0,CGMP48/0으로직전과같다(`stored-source-outer-frame-counters-v152/results.json`). CGMP의선택35/39/42/47쪽Native도98.46300/95.37392/98.77537/94.05301%,exit0/102.217초·source/CLI불변이다. 같은Print2020·compat2022를사용한정상대조일뿐전48쪽/fresh 또는새직접보정완료가아니다.
- root바깥위여백의미소비를실제44쪽GDB로확인했다(`giant-outer-frame-forensics-v152/root-paint-contract-gdb.log`). 원본/배치호스트의저장줄·dirty·profile·표속성은같고source작은앵커1200HU/표73450HU이나,원본컨트롤은SectionDef/ColumnDef/**PageNumberPos**/Table이다. 새판별자의구조컨트롤허용목록에서쪽번호위치를누락해본문상단75.58667px에서위여백이열리지않았다. 쪽번호위치는본문표의줄흐름개체가아니며기존 `stored_body_filling_rowbreak_frame`도같이허용한다. 첫GDB의Vec직접첨자오류exit1은제품회귀가아니며Rust필드포인터로교정한exit0/20.703초의결과로판정한다. 원본/기준/기대값을바꾸지않고다음단계에서같은메타데이터계약을연결한다.
- 종료된위3개진단의SVG269개310,143,254바이트는기하·해시를보존한뒤압축없이삭제했다(`finished-source-outer-frame-svg-cleanup-v152.json`). 원본/Print/새PNG/TSV/tree/로그/동결CLI/공유target은유지했다. fmt검사와diff검사는exit0이며현재source해시는v152빌드와같다. 이단계는부분개선커밋이며90%미달·기존5실패·최종전체검증및18개개별review/오늘할일·PR/CI/merge/후속처리는미완료다.


### 진행127 — 실제 upstream 전진 리베이스와 쪽번호 메타데이터의 프레임 소유(v153)

- v152는666bfe8e440a로커밋했고fetch에서upstream/devel이90fa2f895→4020c0428로실제전진했다. 완료된모든Cargo/시각진단뒤131개로컬커밋을새base위에rebase했고현재head는be5fa75faa4adf8fb430ce2b07e3d54514d867b8다. local devel도같은4020c0428이며추가PR체리픽은없다. old head는전용backup ref로보존했고이전검증을새head의검증으로쓰지않는다(`rebase-upstream-4020c-{preflight,complete}-v152.json`).
- 가운데정렬충돌은upstream의유효한저장중간줄폭보존/재조판구분공백제외조건을유지했다. #7698의독립Print/#3837 실제겹침4→3이근거이며,기존로컬ISO3633 p20의가시중심보정과는경계가달라scattered HWP/HWPX19/20쪽을새소스로재캡처한다. 검증전에는양쪽모두충족으로보고하지않는다(`rebase-4020c-center-conflict-v152.json`).
- WASM조판세대공개API는upstream이같은기능을추가해중복됐다. upstream한국어JS선언과src/wasm_api.rs의단일`set_hangul2024_compat → DocumentCore`를유지하고,참조하지않는기존로컬중복모듈을이번보정에서제거했다. fresh package재생성으로루트pkg/Studio public동일성을검증할대상이며기존bundle을최신결과로쓰지않는다(`rebase-4020c-wasm-conflict-v152.json`).
- root여백누락은같은원본호스트/표/저장줄/dirty/profile에도쪽번호위치컨트롤때문에공통프레임판별이false인실제실행으로확인했다. `collect_header_footer_controls → Page.page_number_pos`가이컨트롤의생산/소비이며본문줄높이를갖는개체가아니다. 기존body-filling원본프레임도이메타데이터를허용한다. 새작은앵커프레임판별도같이허용해측정예산·prepare·최종부분원점이모두같은결과를사용하게한다. 문서ID/임의offset/클램프/fixture/기대값/공차를추가하지않는다.
- 최신suite를prepare한뒤Native빌드·기존479개와가운데정렬대조/새upstreamparser영향검사·6원본runtime·giant42–45및scattered19/20을한후보로확인한다. root위바깥여백141HU의예약/배치와쪽소유·뒤내용보존은실행으로확인하며,fresh/전체회귀/lint/전쪽90%와초기18개개별review/오늘할일·PR/CI/merge/후속처리는미완료다.

- v153 Native빌드는exit0/131.850초·source불변이며동결CLI SHA `9f8499869f9278d074d31ab6a0c975815b68267912438d28545490eb2b665603`다(`stored-source-outer-frame-artifact-v153.json`). 같은동결CLI의실제44쪽원점추적은exit0/20.195초·source불변이고공통root프레임판별false→true,최종원점75.58667→77.46667px로원본141HU위여백을한번소비했다. tree의root/middle/grandchild위끝은75.6/81.2/85.0→77.5/83.1/86.9px이며같은독립Print관측77.356/82.9493/86.6253px와대응한다(`giant-outer-frame-forensics-v153/root-paint-contract-gdb.log`, `root44-physical-frame-before-after.json`). 임의픽셀offset을정답으로고정하지않고동일원본의바깥여백생산/최종소비를확인한진단이다. 새Native점수/직접판독·관련회귀는진행중이며전쪽/fresh승인을주장하지않는다.

- v153 기존 집중496개는491PASS/직전과같은5FAIL/9671skip, 실제254.580초·compile포함720.813초,exit100/source및930개case불변이다. 가운데정렬4개와upstream parser13개는실제로통과했다. overlap/IR 원장의처음exact필터는module접두사가빠져두검사를선택하지않았으므로통과로세지않고별도2개묶음을재실행했다(`stored-source-outer-frame-focused-v153.json`, `stored-source-outer-frame-rebase-baselines-v153.json`).
- 같은독립Print2024·HYS·compat2024로새giant42/43/44/45쪽Native는89.73662/93.67989/86.98285/99.92837%,전체48/48쪽,exit1/291.349초·source/CLI불변이다. 새review_042–045를실제표시·직접판독했다. root와중첩표의시작위치는대응하지만42쪽하단2.4조건과44쪽PCB조건문·하단경계차이가남고43쪽도아래경계/간격차이가남는다. 선택42/44가90%미만이므로전쪽/fresh승인·PR준비를주장하지않는다(`native-giant-stored-source-outer-frame-selected-v153/giant/direct/`).
- 리베이스가운데정렬대조는scattered HWP19/20쪽99.35269/97.44457%(Print2024/compat2024,101.283초),HWPX99.46133/97.58977%(Print2020/compat2022,186.969초),각exit0/source불변이다. 두새20쪽review를표시·판독했으며ISO3633첫줄의작은가운데정렬과아래표경계차이를점수만으로해결완료로쓰지않는다(`native-center-rebase-controls-v153/`).
- CGMP의정상대조선택35/39/42/47쪽은98.44317/95.35326/98.74964/94.05301%,exit0/134.762초,같은Print2020/compat2022·source/CLI불변이다. 6원본runtime쪽수/넘침은giant48/0,press3637 13/0,external1891 74/10,virtual6776 75/0,water6776 3/0,CGMP48/0이다. CGMP는#7764의현재메인터너보정범위이며이선택대조나기존이관을제외근거로사용하지않는다. 전48쪽/fresh최종자격검증은별도다.
- 다음실제원인의독립근거를43쪽에서확인했다. 원본row16/col0의`2.5 `는Native가문자를중복하지않지만2/dot/5의x가97.01333/126.28444/150.66222이며같은Print는96.96/139.999808/178.080448이다. 저장양쪽정렬의문자간분배경로가끝공백을글자수와자연폭에포함하는반면낱말간분배는제외한다. `compute_line_extra_spacing`의원본저장줄경로와fresh synthetic-wrap끝advance계약을구분해추적한다(`giant-outer-frame-forensics-v153/page43-label-svg-fragments.json`, `page43-label-print-ink-boxes.json`). 좌표를기대값으로주입하거나수정완료로보고하지않는다.
- 종료된v153의SVG273개462,419,120byte는root/clip/해시와43쪽글자속성을보존한뒤압축없이삭제했다. 원본/Print/PNG/TSV/tree/log/동결CLI/공유target은유지했다(`finished-source-outer-frame-svg-cleanup-v153.json`).


### 진행128 — 현재 보정 범위 밖의 이관된 편람 실패 검사 제거(v154)

- 사용자께서 현재 직접 보정중인 문서가 아니고#7445로이미이관돼있으면회귀검사를제거하라고지시했다. 86개직계하위이슈의현재본문을읽고편람HWP는#7751로연결됨을확인했다. 현재직접대상giant원본과현재메인터너범위CGMP(#7764)는계속유지한다. KPS `kps-ai.hwp`와가맹사업 `1130000-202100008_franchise_review_report.hwp`는하위이슈에서같은원본을찾지못했다. #7769의 `synam-001.hwp`를KPS의이관근거로혼동하지않으며기존3실패를임의제거하지않는다.
- 현재v153실패묶음중 `issue_3820_hwp5_qa_rowbreak_tail_reduces_page_count`의Q8배치함수와편람HWP의셀넘침원장검사를제거했다. 과거#7751에서Q8정상검사를유지한기록과이번사용자의처리범위를구분한다. 원본HWP SHA40d6d05e…와독립Print2024 PDF SHA34db2aee…는기존경로에바이트그대로보존한다. HWPX의저장·바탕쪽/IR계약과다른문서/파서/보안검사는변경하지않는다.
- 제거전함수와원장의32줄기준은 `mydocs/pr/assets/issue7445/handbook-current-failures-20261011/`에자동수집되지않는텍스트로보존했다. 활성원장의다른기준값은변경하지않는다. 기존수집목록의partition배정후해당편람행만실행목록에서제거하므로다른입력의partition도변경하지않는다. 검사기준을현재35줄로완화하거나ignore/조용한return으로통과시키지않는다. 현재집중실패5개에서해당2검사를분리한것이며전체최종회귀통과나편람렌더링해결로보고하지않는다.
- 이전IR/overlap묶음은compile이끝난동결바이너리에서계속실행된다. 이번변경은그검사들의production/IR원장/930개case입력을바꾸지않으며새Cargo작업은기존실행종료후시작한다. 제거범위의manifest·남은편람저장검사·셀원장partition9재실행결과를확정한뒤이단계를커밋한다.
- fetch에서upstream이4020c0428→8761ac902로실제로전진했다. 차이는PR#7698병합·보정검증을기록한문서커밋한개다. 이단계커밋뒤새base로rebase하고production동일성과정책base를확인한다. 변경이없는head에는rebase하지않는다.
- 리베이스뒤독립IR/글자겹침2검사는terminal exit0,실제1735.277초/compile포함1748.991초로2PASS/434skip이다. production및930case해시는불변이며전체Rust검증은아니다(`stored-source-outer-frame-rebase-baselines-v153.json`).
- 사용자추가지시: PR은승인받고진행한다. 로컬보정/검증/커밋/실제head변경에따른리베이스를계속하며,필수검증과90%시각gate를충족한구체적인변경내용·대표review PNG·검증결과를제시한뒤PR생성승인을요청한다. 아직PR을생성하거나승인을요청할후보가아니다.
- 제거뒤prepare는1462source/6284static test attrs로기존보다검사함수1개가감소했고생성28suite/20exception을유지했다. fmt/fmtcheck exit0이다. 셀넘침partition9는1PASS/233skip·실제102.921초·exit0이며편람HWP가실행에서빠졌다. 기존suite013을지정한첫명령은prepare재배정뒤저장검사2개를선택하지않았으므로해당통과로세지않는다. prepare뒤실제suite003을읽어다시실행한보존저장/HWPX2개는2PASS/202skip·실제61.091초·compile포함77.583초·exit0/소스불변이다(`transferred-current-failures-focused-v154.json`, `transferred-handbook-save-focused-v154.json`). 합계기존검사3개PASS이며제거한검사/입력은성공건수에포함하지않는다. 원본/Print해시와다른원장행의동일성을다시확인했다. 전체회귀/fresh/전쪽90%완료가아니며현재이관근거없는3실패를계속보정한다.

### 진행129 — giant의 이관 범위 정정과 활성 회귀 분리(v155)

- 사용자 지적으로 최신 #7445와 #7812 본문을 다시 읽었다. `samples/table_giant_cell_overfill.hwpx` SHA `5d7eb4a21e46d9ad01a0f631eea2b1f2ec8a71750b4d448868e944e1b95042f4`는 다른 브랜치에서 보정하는 이관 원본이다. 초기18개에 적용한41개 contributor commit의 실제 diff를 검사했으며 이 원본의 변경·언급이 하나도 없었다(`initial18-giant-scope-audit-v155.json`). 최근 진행에서 이 문서를 현재 직접 대상으로 계속 잡은 판단을 정정한다. giant의 추가 조판 보정과 전48쪽/fresh 자격 검증을 현재 완료 조건으로 계속 요구하지 않는다. 이미 존재하는 실패 증적을 통과·결함 해결로 바꾸지 않는다.
- 사용자가 이관됐으면 회귀를 제거하고 KPS·가맹사업의 기존3실패 및 보정 후 전쪽 Native/fresh·전체 회귀/lint를 진행하라고 지시했다. #5908의3함수, #7140 HWPX 쪽수1함수, #6976 giant 높이1함수, layout-anomaly의 giant 의존7함수(합계12)를 활성 검사에서 분리한다. #7140의 중첩 행 검사는 별도 HWP counterpart만 남긴다. 혼합 파일의 clean CLI7함수·별도 문서 정렬 검사와 parser/serializer/security 계약은 보존한다.
- 원장6곳에서 해당 HWPX 행만 제거한다(본문·셀 넘침/용지 밖/글자 겹침/한컴 쪽수/렌더 쪽수). 자동 수집4경로는 원래 partition 배정 후 이 원본을 작업 목록에서 제외한다. 다른 입력의 배정·기준값·허용치를 바꾸지 않는다. 관련 실제 partition은 본문9/용지밖10/겹침2/셀8이다(`giant-transfer-target-resolution-v155.json`). 공통 parser 코퍼스와 HWP counterpart를 제외하는 전역 술어는 추가하지 않는다.
- 변경 전 검사4파일과 제거한 원장6행은 `mydocs/pr/assets/issue7445/giant-regression-scope-20261011/`의 수집되지 않는 `.rs.txt`/`.tsv.txt` 및 `transfer-inventory.json`에 보존한다. 원본 HWPX·같은48쪽 독립Print·별도 HWP SHA를 확인했고 바이트를 유지한다. 이전 편람 이관 inventory의 잘못된 current_direct_target도 정정한다. 향후 #7812에서 전쪽 Native/fresh 최저90% 이상과 직접 판독 후 관계형 회귀를 복원한다.
- 이번 단계는 production 코드를 수정하지 않는다. v153 동결CLI와 현재1102 production 파일 해시가 같음을 확인했다. 앞서한 giant GDB 진단에서 저장 경계 술어를 추가로 걸었지만 breakpoint는 맞지 않았으므로 반환값의 입증으로 쓰지 않는다. 새로운 가운데 정렬/공백 패치는 적용하지 않았다. 기존 공통 보정의 다른 승인 대상 검증을 이 범위 정정으로 면제하지 않는다.
- prepare는1461source/6272static test attrs/28suite/20exception으로 기존 검사12개 감소를 확인했다. fmt/fmtcheck는exit0이다. 실제prepare뒤 재배정된suite를읽어 보존CLI/fixture/HWP/혼합검사와 해당4partition을 실행한다. 제거된 검사·원본은 성공건수에 포함하지 않는다. 최신 KPS37/38/55와 가맹사업60쪽의 진단 재출력도 같은 동결 production CLI로 진행한다. 이는 전쪽/fresh 최종 자격 검증이 아니다. 완료 결과를 다음 항목에 연결하며 PR은 아직 생성하지 않는다.

- 관련 기존20개 실제 결과는19PASS/1FAIL/1366skip,실제48.044초/compile포함104.509초·exit100이다. FAIL은giant가아닌 `issue1949_giant_cell_nested_tables_perf.hwpx`의본문원장9→11이다. 최신#7813에서동일원본의고정base전115쪽미달과이관을확인했고현재직접대상밖이라는사용자지시에따라해당실패입력/기존9행을활성본문원장에서분리했다. 다른브랜치27FAIL·4 API검사를이번브랜치실패로가져오거나제품결함해결로보고하지않는다. 추가제거행도같은자산에보존했다.
- 분리뒤본문partition9는1PASS/225skip,실제28.756초/compile포함81.365초·exit0이다(`giant-transfer-body-remaining-v155.json`). 처음의19PASS와재검사1PASS를각각기록하며제거한검사·입력은성공건수로세지않는다. 소스/기대값을변경하지않은보존CLI7/fixture6/별도HWP1/혼합정렬2/용지밖·겹침·셀partition3의통과도첫묶음에포함됐다.
- 최초prepare뒤fmt로원본의가중치가변해manifest check에서파생harness drift가검출됐다. source전량fmt → prepare → fmtcheck → 고정base8761ac manifest/unit-tier를순차재실행해5단계모두exit0을확인했다(`giant-transfer-post-format-policy-v155.json`). 파생파일은커밋하지않는다. 모든다른원장행/원본SHA/1102production파일동일성도확인했다. 새렌더링회귀·golden·기준값갱신은없다.
- KPS77쪽과가맹사업144쪽의현재Native전쪽비교는고정v153CLI로실행중이다(`remaining-whole-native-v155-command.json`). 사용자추가지침에따라최종전쪽TSV의90%미만/누락/쪽수차이와직접확대판독을확인하고freshWASM도새소스로확인한다. 이전부분진단은도구세션및실제OSworker가사라졌지만terminal결과가없어중단기록으로남겼으며성공으로세지않는다. 새전쪽작업은독립supervisor PID/start tick을고정해관측한다. 전체회귀·세Clippy/workspace build·PR승인·CI/통합단계는아직미완료다.

## 130. v155 전체 종료와 v156 issue1949 검사 이관 보완

- 사용자 지시에 따라 현재 직접 보정은 `samples/kps-ai.hwp`와 `samples/issue5966/1130000-202100008_franchise_review_report.hwp`에 집중한다. 최초18개 적용41커밋과 원 PR 본문을 다시 읽었으며 두 원본은 원 PR의 직접 언급 대상이 아니고 통합 회귀 조사에서 추가된 보정 대상이다. CGMP는 앞서 승인된 메인터너 보정 범위를 유지하며 최신 최종 검증을 남긴다. `table_giant_cell_overfill.hwpx`는 #7812로 이관되어 제외한다.
- `rhwp-review-v155-rust.service`의 실제 terminal 결과는 exit100, 전체 실행1741.136초·빌드 포함4072.177초, **10343검사 중10334PASS/9FAIL/45skip**다. 실행 전후 source 해시는 모두 같고 MainPID0/inactive를 확인했다. 기존 단기 실행이나 실행 중 상태를 전체 PASS로 쓰지 않는다. 원문은 `whole-rust-transferred-scope-v155-retry1-exit.json`과 해당 로그다.
- 실패6개는 `issue1949_giant_cell_nested_tables_perf`에 의존한다: source API의 page-local bbox·scoped cache 각1개, `issue_2214_page_local_repaint`의 warm/cold2개, `issue_2215_selection_page_range`의 normal/cross-page2개다. 나머지3개는 기존 KPS 행 핀과 가맹사업의 두 쪽/되감김 검사다. 이 구분을 보정 뒤 최종 전체 결과로 확대하지 않는다.
- #7813의 현재 본문에서 정확히 같은 HWP/HWPX SHA와 고정renderer base `90fa2f8955c2dd21e9f0ea8fd0c6ee8dc5996a47`의 전115쪽 Native 미달을 확인했다. HWP 최저23.54542%, HWPX 최저23.78253%, 각115쪽 모두90% 미만이다. 원본과 Studio HWP도 해시가 같으며 이 원본은 현재 직접 보정 대상이 아니다. 이관 이슈의 fresh WASM은 아직 진행 중이라는 기록 그대로이며 성공으로 쓰지 않는다.
- 해당 원본의 배치 회귀를 활성 경로에서 분리했다. source6함수(bbox/scoped cache/focused geometry/resumable3)와 해당 원본만 쓰는 두 integration 파일의6함수, 합계12검사 및 전용 컷 helper를 보관했다. 이 중 이미 통과했던 검사를 포함하며 제거를 PASS로 세지 않는다. 일반 입력의 API/cache/serializer 검사, 조사·프로파일링과 모델 지문 계약은 유지한다. 원문·목록·SHA·고정base 근거·변경 전 전체 결과는 `mydocs/pr/assets/issue7445/issue1949-regression-scope-20261011/`에 보존한다. HWP/HWPX/PDF 파일은 삭제하지 않았다.
- Native 전쪽 비교도 종료됐다. KPS는77/Print77쪽이며11쪽이90% 미만(15·16·17·19·21·27·55·60·65·66·73), 최저55쪽45.38915%, gate exit1이다. 가맹사업은Native144/Print142쪽 불일치로 exit1이며 TSV를 산출한 것으로 보고하지 않는다. 동결 CLI와 현재 production의 동일성은 보존하되 이 결과는 보정 전 Native이고 새 코드나 fresh WASM의 자격 검증이 아니다.
- KPS 미달11쪽의 실제 `review.png`를 만들고 대표21·27·65·73쪽을 직접 읽었다. 실제 GDB의 `kps206-tac-flow-gap-retry1-gdb.log`는 조판이0..5 소유 줄을 쓰는데 paint의 첫 두 표가 모두 마지막5번 줄을 조회함을 보여준다. `kps-multiple-tac-source-slots-gdb.log`는 원본 선행 공백4개/offset0..3과 각 제어 슬롯을 확인했다. 27쪽의 진단은 원본 저장 줄 가로 시작0·폭50752HU와 달리 legacy가 내어쓰기48.8px를 표의 왼쪽 하한으로 강제함을 보여준다. 같은 원본 Print 괘선과 실제 원점은 `kps-root-table-print-coordinate-evidence.json`에 연결했다. 실제 호출되지 않은 breakpoint와 실패한 GDB 시도는 실행 근거로 쓰지 않는다.
- 저장 개체 줄의 원시 소속·기존 글줄 메트릭·펜·전진 끝을 공유하는 KPS 초안과, 가맹사업 각주 소유/각주 표 출력/같은 행의 잔여 물리 높이 초안을 묶어 준비했다. production 보정은 아직 적용·빌드 전이다. 관련 모든 페이지의 보정 후 Native/fresh WASM90%와 직접 확대 판독 전에는 기존 KPS·가맹사업 기대값을 변경하지 않는다. 최종 전체 회귀·세Clippy/workspace·원래 편집 사례/Studio·개별PR review/오늘할일·승인 후 통합PR/CI/merge/후속 처리도 미완료다.
- 이관 보완의 정적 검증은 fmt → prepare → fmtcheck → 고정base8761ac manifest → unit-tier5단계 모두exit0이다(`issue1949-transfer-stage-policy-v156-retry1.json`). 삭제한 source를 옛 파생 suite가 import해 최초fmt가 실패했으므로 파생 suite를 먼저 다시 준비한 뒤 재실행했다. 최종1459source/6266static attrs/28suite/20exception이며 파생 파일은 stage하지 않는다. production renderer와 다른 case는 변경하지 않았고 이관 뒤 전체 Rust·필수Clippy 성공을 주장하지 않는다.


### 진행131 — KPS·가맹사업 묶음 적용과 저장 개체 원점 소비(v156/v157)

- issue1949 이관 단계를 `85ba765d02fe6f1df6f6ec8299f368777e600d80`으로 일반 커밋한 뒤 실제 upstream 전진을 확인했다. 새 base790a6f8f3로 리베이스한 HEAD는774a6e366이며 리베이스 전후 src/crates/tests diff는 비어 있다. 새 base 고정 manifest/unit-tier 검사는 각각exit0이다(`issue1949-transfer-rebase-v156.json`, `issue1949-transfer-rebase-policy-v156.json`). 추가 체리픽·push·PR 생성은 없다.
- 진행130의 초안을 실제 적용했다. v156 fmt와 Native debug 빌드는exit0/35.221초·167.483초이며 고정 target/pr-review를 재사용했다. 동결CLI SHA는 `631159367077d4a1e962146fff275d7a37556d3d530ef438c1185773ae3c48d8`다. 실제 dump-pages는KPS77쪽·가맹사업142쪽(기존144→독립Print142와 일치)을 확인했다. 쪽수만으로 전쪽 시각 통과나 내용 보존을 선언하지 않는다(`kps-franchise-batch-native-v156-result.json`, `kps-franchise-batch-quick-pagination-v156.json`).
- 같은 원본 KPS 미달11쪽의 새 Native 비교는exit1이다. 55쪽은45.38915→99.95%,27쪽76.12177→99.34%로 개선되어 새review PNG에서 중첩 표의 주석·뒤 표 겹침과 중앙 정렬을 직접 확인했다. 나머지9쪽은90%미만이며,15/16/17/19쪽에는 제가 추가한 개체 줄 펜이 뒤 줄의 내어쓰기를 누락한 오류가 있다. 이를 기존 원본의 차이로 취급하거나 원 PR 검증 완료로 보고하지 않는다(`native-kps-batch-boundary-v156/kps1156/analysis/metrics.json`, 같은 경로 `review/review_015.png`, `review_027.png`, `review_055.png`).
- v156 가맹사업 비교는동결CLI로계속실행중이다. 새10쪽review에서 각주8의 실제 표가독립Print와대응하고95.12%,47쪽review에서 현재본문·각주43 앞부분소속이대응하고97.19%를확인했다. 뒤부분이월·109–111쪽조각·마지막142쪽과전쪽검증은아직완료하지않았다(`native-franchise-batch-boundary-v156/franchise5966/review/review_010.png`, `review_047.png`).
- 새 실제 GDB는21쪽뒤빈줄의format이높이20/간격16px를이미소비함을보여준다. 잘못추정한뒤빈줄의절반간격추가를하지않고원래표소유줄의다음저장원점이LH+LS전량과일치하는경계를공통trailing helper에서판정한다.60쪽원본마지막그림은저장vpos59430HU+offset158HU+그림2880HU로물리끝62468HU를갖는다. 기존셀정렬은줄끝60830HU만신뢰하고실제그림배치도센터가반영된text_y_start184.92대신content_top155.4533을선택해최종y949.96으로덮어썼다. 실패한GDB시도와73쪽NOT HIT는실행근거로쓰지않는다(`remaining-source-frame-forensics-v155/shared-origin-diagnostic-results-v157.json`, 원본 `kps60-cell-source-lines-and-styles.json`).
- v157은줄별내어쓰기·같은문단의인증된전체후행간격·Square Para 표의가로바깥여백·저장그림의물리상자를묶어적용했다. 새 `stored_cell_flow_picture.rs`의원점/끝을HeightMeasurer의셀개체높이→layout 셀콘텐츠정렬→같은그림의paint가소비한다. 원본이아닌줄·dirty·저장리셋·첫그림이밀어낸빈앵커·캡션은제외한다. 전체행높이의임의증가·절대좌표clamp·원본ID분기는추가하지않았다. Native 빌드는진행중이며이보정의시각통과는미검증이다.
- 실행중인v156비교는원래동결CLI를끝까지쓴다. v157을적용하기직전모든source hash가v156빌드와일치하고동결CLI가불변임을확인했다. 따라서v156worker의마지막worktree source_unchanged가false여도그결과를v157검증으로올려보고하지않으며원래frozen산출물의비교로보존한다(`kps-next-batch-controlled-source-change-v157.json`). 최종전쪽Native/fresh90%·확대판독전에는기존KPS·가맹사업회귀의기대값을변경하지않는다. 전체회귀/lint·원18개편집저장본·Studio·개별PR review/오늘할일·승인후PR/CI/merge/후속처리는아직미완료다.

- v156 가맹사업109쪽의 새review는70.14%로미달이다. 실제본문이Print보다수px아래이며줄간격/원점의확대판독을계속한다. 앞쪽10/47쪽의개선과142쪽수복구로전체문서를통과했다고보고하지않는다.48쪽review에서도각주43그림의후속쪽소속·44–47번호순서를대조했으나전쪽내용보존검사를대신하지않는다. v157선택비교는성공빌드와v156비교의terminal기록을확인한뒤같은원본11+7쪽을실행하도록별도사용자service에서대기한다(`kps-franchise-batch-selected-native-v157-command.json`, `kps-franchise-batch-selected-native-v157-wait.json`).

- v157 fmt는exit0/70.710초, Native debug 빌드는exit0/175.640초로종료됐다. 새동결CLI SHA `25b1fc0ef8e67081b64901e117285e92e612834675da2efd99e080992be8cc5d`이며빌드뒤source hash를기록했다. 실제dump-pages는KPS77쪽(exit0/8.696초)·가맹사업142쪽(exit0/13.569초)을유지하고source불변이다(`kps-franchise-batch-native-v157-result.json`, `kps-franchise-batch-quick-pagination-v157.json`). 선택Native는앞v156동결비교종료를기다리는service가살아있음을확인했으며새시각결과는아직없다. 빌드성공을전체회귀·lint·fresh WASM/90%통과로보고하지않는다.


### 진행132 — 남은 실제 쪽 소속·바깥여백 소비 재검토(v158/v159)

- 현재 직접 보정은 KPS `samples/kps-ai.hwp`와 가맹사업 `samples/issue5966/1130000-202100008_franchise_review_report.hwp`다. 최초18개41커밋의 직접 언급 원본과 다르며 전체 통합 회귀의 기존3실패 조사에서 추가됐다. 승인된 CGMP 범위의 최종48쪽 검증은 유지한다. giant/#7812·issue1949/#7813은 이관 범위이며 원본을 삭제하지 않는다. 추가 체리픽·원격 PR/push/merge는 수행하지 않았다.
- v157 KPS 선택11쪽 Native는8쪽(15/16/17/19/21/27/55/60)이99.34034–99.98150%이고,65/66/73은83.91100/78.36627/81.27601%로 미달이다. 새review15/60/65/73을 실제 표시·대조했다. 전77쪽 및 fresh WASM 통과로 확대하지 않는다(`native-kps-batch-boundary-v157/kps1156/overlay/overlay_metrics.json`).
- v158 fmt/Native 빌드는exit0/33.141초·96.230초이며 동결CLI SHA `557ae8f01f90108c7893ab341067dcdae3c88b65cbe7c6720f5298ef03b2a1db`다. 선택 Native도 terminal/source불변이다. KPS3쪽은 여전히 미달이고 가맹사업109/141/142는99.99874/35.07474/31.88529%, 각exit1이다.109쪽은 v15770.14398→99.99874%로 개선됐다. 같은 `review_109.png`에서 본문·줄 간격·뒤 문단/하단 쪽번호를 실제 대조했으며, 같은 문단 내부의 원본 저장 쪽 원점이 소유한 조각에 셀 중앙정렬4.4px를 더하던 잘못된 가정을 제거했다.141/142는 앞쪽 내용의 소속이 다르므로142쪽수 일치만으로 내용 보존을 선언하지 않는다(`kps-franchise-batch-selected-native-v158-result.json`, `native-franchise-batch-boundary-v158/franchise5966/overlay/overlay_metrics.json`).
- KPS73의 새 빈 호스트 helper는 제가 단일control만 허용해 실제 표+종이 기준 전면 장식 도형2개의 원본을 잘못 배제했다. 정상 종료한GDB에서 후보placement=None/legacy whole fit=true 및 before283HU를 확인했다. 보이는 장식과 글줄 흐름 소유를 구분하며, 단독 흐름 표와 nonTAC·flowWithText=false·줄간격 비참여·종이/쪽 기준 전면/배경 장식만 허용하도록 고친다. 인라인/본문 어울림 형제는 제외한다(`remaining-source-frame-forensics-v155/kps73-final-margin-consumers-v159-gdb.log`, `kps73-host-control-kinds-v159-gdb.log`).
- KPS65/66의 어울림(Square) 분할 표는 실제 첫/뒤 조각의host_before=0·placement=None이고 실제paint 원점128.50667px에outer top283HU가 없다. 독립Print 괘선과 기존 통째Square 경로는 같은 바깥 위여백을 소비한다. v159는 HostSpacing 생산→prepare 첫행 적합성→fragment budget 위여백 차감/결정된table_top→emit 동일placement 기록/물리 흐름→layout_partial_table의resolved_table_top으로 전달한다. 여백을paint만에서 추가하거나선택한컷의높이를다시늘리지않는다. 가시 텍스트 호스트는 기존 앵커 소속을 유지하고 이미 결정된 다른placement는 덮어쓰지 않는다(`remaining-source-frame-forensics-v155/kps-square-shared-budget-origin-v159-gdb.log`, `kps-flow-frame-controlled-change-v159.json`). 새빌드와 영향3쪽 비교로 이 경계의 실제 결과를 확인하며 전쪽/fresh 및 정상 대조 검증 전에는 완료로 판정하지 않는다.
- 가맹사업 전142쪽 Native 진단은 동결v157CLI SHA `25b1fc0ef8e67081b64901e117285e92e612834675da2efd99e080992be8cc5d`의 별도service로 실행 중이다. PID3379694와 실제자식worker를 확인했다. 이미 적용한v158/v159 source와 구분하며 이 진단을 최종head의 자격 검증으로 보고하지 않는다. 최종page141 앞쪽row10에서 원본paragraph23의마지막줄 대신paragraph22가 시작해 다음row11이 약124px 늦는 원인을 컷유닛·원본저장줄과 대조한다. 같은142쪽에서 Print는paragraph15로시작하나 Native는11부터시작하므로 단순쪽수회귀의기대값을바꾸지않는다.
- 사용자 승인된 불필요SVG 정리로 종료된v155 Native 전쪽산출물의SVG만221개/17,720,723,829byte를 삭제했다. 원본·PDF·review/overlay PNG·TSV·render tree·명령/로그·실행중인작업·공용target은보존했다. 정확한목록과 종료/ignored/비사용 확인은 `finished-native-whole-svg-cleanup-v158.json`에 남겼다. 디스크가 약20GiB→36GiB로 확보됐으며 압축보관하지 않았다.
- 기존 KPS1156·가맹사업5966/6718 검사와 기대값은 유지한다. 보정후 전쪽Native/fresh90%·확대판독, 최종전체Rust회귀·세Clippy/workspace·원18개편집저장본/Studio·개별PR review/오늘할일·사용자승인후PR/CI/merge/후속처리는 아직 미완료다.

- v159 fmt/Native 빌드는exit0/34.647초·112.154초이며 동결CLI SHA `84ae40a89ef07435f9eed29e84e53f4070465ab9965b393cb2a5b8f02bdae164`다. KPS65/66/73 선택Native는99.37215/96.42159/97.62566%,exit0/159.612초·source불변으로종료했다. 새review65/73을 실제표시해 `<붙임1>`·중첩표외곽/내부행·체크박스·표뒤쪽번호와73의전면장식/표위끝·내용순서를대조했다. 이전v157에서개선된8쪽과합쳐보정전미달11쪽의선택Native개선을확인했지만같은최종빌드의전77쪽/fresh통과로대체하지않는다. 현재v159의전77쪽 Native TSV를새로산출하며다음source변화와동결CLI를구분한다(`kps-franchise-batch-native-v159-result.json`, `kps-franchise-batch-selected-native-v159-result.json`, `native-kps-batch-boundary-v159/kps1156/overlay/overlay_metrics.json`).
- 가맹사업의최초큰누적차이는row4의118→119쪽이다.117쪽까지이구간의본문소속이대응하지만118말미의작은중첩표가Native에서통째223.96px유닛으로예약되어119로밀리고,119쪽에는서로다른원본저장프레임의앞/뒤문단이같은원점에서겹친다. 같은독립Print의해당표상자는118쪽909.565–996.031px/119쪽141.605–278.575px로확인했다. 원본child는CELL1×1/선언첫상자6488HU/셀물리높이16231HU이며첫문단vpos300/1800/3300후다음문단이300으로되감긴다. 실제문단prev여백300HU가저장원점에포함되므로단순원점0판별과큰높이임계로이경계를지우는가정을재검토한다. 부모다음저장원점10848HU는자식마지막잉크끝10000+상하패딩282+상하바깥여백566과일치한다.124쪽이후제목의약124–156px이동은이앞선원본소속상실과연결되며마지막쪽만별도보정하지않는다. 성공한실제GDB유닛원장은 `remaining-source-frame-forensics-v155/franchise-row4-unit-ledger-v159-gdb.log`, 원본줄/페이지소속/전12행제목비교는같은폴더 `franchise-row4-source-opening-v159.json`·`franchise-row4-page-boundaries-v159.json`·`franchise-whole-row-heading-origins-v159.json`에보존한다. 이원인진단의후속코드보정과정상대조·Native/fresh검증은아직미완료다.

- v159의66쪽review도실제표시했다. 점수96.42159%와별도로상단이어받은병합셀의‘시장침해 가능성’이Native에서위쪽에있고Print에서는가운데놓인다. 표바깥원점의개선만으로이셀의정렬차이를완료로판정하지않는다. 이외뒤날짜/서명·하단표외곽과부속용지표기띠의미세차이도최종확대대조에포함한다. KPS전77쪽Native·가맹사업전142쪽v157Native진단·가맹사업117–119쪽v159직접review는각각실제live service/PID3390248/3379694/3390531로확인했으며최종fresh/회귀/lint를대체하지않는다.


### 진행133 — 선택된 셀 정렬·작은 자식 표의 원본 쪽 프레임 공통화(v160)

- v159 가맹사업117/118/119 선택Native는99.99883/88.01124/37.00942%,exit1/263.396초·source불변으로종료했다. 같은119쪽review를실제표시해앞쪽에서이미소비해야할문장이조건(1)과겹치고뒤본문도밀리는것을확인했다.117쪽대조군의일치나142쪽수로이소속결함을면제하지않는다(`franchise-nested-frame-native-v159-result.json`, `native-franchise-nested-frame-v159/franchise5966/review/review_119.png`).
- KPS66의실제호출은 `table_partial.rs` 셀정렬이아니라 `table_layout.rs`의중첩통째배치+row_filter다. 첫breakpoint가호출되지않은기록을진단근거로쓰지않는다.정상종료한교정GDB는원본5문단에서선택한뒤2문단·셀173.53333px/안높이165.98667px/내용41.6px·row_filter20..29·다른viewport 잘림없음을보여준다. 원래Center인데행18<20만으로Top을덮어쓰므로Native시장침해는141.7px/독립Print텍스트상자는205.549px에서시작한다(`remaining-source-frame-forensics-v155/kps66-actual-span-tail-alignment-v160-gdb.log`).
- v160은이미선택한온전한원본문단조각에대해원래정렬을유지한다. original Native/미편집·비재조판·dirty없음·컨트롤없는원본저장줄·실제양의줄원점이있는조각으로확인하며,수용높이를넘는아래쪽잘림/다른실제viewport잘림/재조판·합성원점은기존Top계약을유지한다. 행범위로앞쪽내용을다시선택하거나독립PDF의px좌표를구현상수로쓰지않는다.
- 작은중첩표의실제프레임증거는 `render_normalization/stored_nested_text_frames.rs`에서원본문서기준으로한번생산한다. 현재수정범위의NativeRowBreak부모/빈단독표호스트·원본CELL1×1텍스트자식·작은첫선언상자보다큰셀·정확히한번의정상사다리되감김·문단위간격을제외한재시작·문단간간격등식·부모다음원점의마지막자식상자종료등식을확인한다. 수정된줄·로컬재시작·어울림이아닌개체·캡션/추가개체·직접HWPX는이새증거로승격하지않는다. 첫상자의빈물리띠도6488HU에포함하며같은프레임유닛이쪽마다패딩/바깥여백을소유한다. 새로운문서ID나픽셀offset조건은없다.
- 공통결과의소비는 `RenderNormalizationOverlay::from_document_reusing`→부모`cell_units_uncached`의1×1자식투영→측정/행컷이보존하는원본유닛서수/쪽경계→`mixed_nested_split_from_cut`의같은원본child cut와물리점유→`layout_partial_table_resolved`의선택된원본행높이/위여백원점→같은컷을소비하는셀의원래정렬이다. canonical자식유닛의para/line서수가저장프레임과동일한지먼저확인하며대체scalar전체표와혼합하지않는다. 다른기존프레임placement를덮어쓰거나선택컷뒤paint만높이를늘리지않는다.
- v159동결CLI와각전체Nativeworker는기존source버전을끝까지쓴다. source변화와CLI불변·새모듈목록은 `source-nested-physical-frame-controlled-change-v160.json`에남겼다. 새묶음의fmt/Native빌드는전용service에서실행중이며새출력/쪽수·전쪽/fresh·정상반례·기존회귀/lint는미완료다. 기존기대값·시험지원본·체리픽범위·PR승인경계는유지한다.

- v160 fmt/Native 빌드는 exit0/38.433초·208.273초로 종료했고, 동결 CLI SHA는 `13589d69f34ccad33e63c08ff8358fb0b83c2800588c50d8e64c6bbe937d274c`다. source 불변의 dump-pages는 KPS77쪽/가맹사업142쪽이다. 결과는 `kps-franchise-batch-native-v160-result.json`·`source-nested-frame-quick-observations-v160.json`에 보존하며 전체 자격 검증과 구분한다.
- KPS66의새v160review를실제표시했다. ‘시장침해 가능성’의같은TextLine원점은141.7→203.9px이고독립Print글자상자상단205.549px와대응한다. 전체점수96.42159%가같더라도실제PNG해시는서로다르고표시된셀정렬은변했다. old_partial셀의NOT HIT와GDB에서초기화전변수/Vec포인터를잘못조회한exit1시도는성공실행근거로올리지않는다. 정상v160render tree와실제review/NativePNG를근거로한다.
- 가맹사업v160의컷은117–142경계에서이전과같고원본쪽프레임생산이아직적용되지않았다. 실제GDB에서child의내부page_break가RowBreak임을확인했다. 제가독립HWPX의CELL표기를내부CellBreak로해석한오류로생산조건에서배제했다. 현재serializer/parser계약은HWPX CELL/ROW→RowBreak, TABLE→CellBreak이므로v161에서실제내부RowBreak로고친다. 다른실패/누락조건을조건추가로가리거나원본기대값을바꾸지않는다(`source-frame-enum-controlled-change-v161.json`, `remaining-source-frame-forensics-v155/franchise-stored-text-frame-entry-retry2-v160-gdb.log`). 진행133앞항목의‘CELL1×1’은원본HWPX문자열이며내부enum의미와구분한다. 새빌드후같은원본물리프레임·쪽소속·확대review를재검증한다.

- v161 fmt/Native 빌드는exit0/65.57초·131.561초로종료했고 동결CLI SHA는 `6300c4934323268570bed933eff346c0a93e3b4c40c5a99e0a0152416cda22f9`다. source불변의빠른출력은KPS77쪽/가맹사업142쪽을유지하며118쪽종료컷55→58/119쪽시작58을확인했다. 새실제트리에서118말미에원래앞부분3줄이남고119첫줄이원래조건(1)로시작한다. 정상대조117/124제목은Print352.902/621.702px에Native352.6/621.4px로대응한다. 같은142쪽전체트리export는exit0/17.035초·142파일/source불변이다(`source-nested-frame-quick-observations-v161.json`, `franchise-all-frame-trees-v161-result.json`, `remaining-source-frame-forensics-v155/franchise-whole-row-heading-origins-v161.json`). 쪽수·트리관측을새Native 점수·fresh통과로보고하지않는다.
- 이후경계는아직남았다.125쪽다음row6제목은Print967.142/Native946.2px이며,133쪽row7/138쪽row9/141쪽row11이Native상단으로일러진다. 마지막142쪽은원래문단의중간줄부터시작하므로내용소속보정완료가아니다. 원본row5선언셀1432HU와마지막빈문단vpos59040/LH1300/LS780,다음row6의제목0/다음표호스트0을구분하며,빈글자와실제물리띠·쪽경계의소비를추가대조한다. 초기118쪽원인의해소로이뒤쪽원본소속차이를면제하거나기존두회귀기대값을바꾸지않는다.
- KPS전77쪽Native v159는exit0/2096.898초로종료했다. 전체77/Print77쪽TSV의최저는42쪽90.93343%이고미달/누락0쪽이며gate는직접판독을남긴not_evaluated다(`native-kps-whole-diagnostic-v159/kps1156/silhouette.tsv`, `silhouette_manifest.json`). 이작업은동결v159CLI를끝까지썼으며이후v160/v161 source변화때문에source_unchanged=false인기록을지금빌드의전쪽검증으로올리지않는다. v160에서직접확인한66쪽정렬과최종source의전77쪽/freshWASM은별도다.
- v160선택Native의후속가맹사업과v161선택Native는원래각동결CLI로진행한다. 이미끝난KPS3쪽은v160점수99.37215/96.42159/97.62566%·exit0/206.657초이고새66쪽review의실제정렬을확인했다. 아직필수freshWASM·최종전쪽Native/확대판독·정상반례/전후회귀·전체Rust/세Clippy/workspace·원18개편집/Studio·개별PRreview/오늘할일·승인후PR/CI/merge/후속처리는미완료다. 추가체리픽/원격조치는없다.

### 진행134 — 빈 저장 쪽 원점의 실제 점유와 작은 셀 선언값 구분(v161/v162)

- 직전 질의 응답 단계는 범위 확인이며 구현 완료 단계가 아니다. 현재 직접 보정은 KPS·가맹사업 두 원본, CGMP는 승인된 최종 검증 범위로 유지한다. 최초18개41커밋의 직접 편집 사례와 통합 회귀에서 추가된 두 원본을 구분하며 추가 체리픽/원격 조치는 없다.
- v161 선택 Native는 실제 terminal이며 가맹사업117/118/119/124쪽99.99883/99.99852/95.22284/99.99843%,139/141/142쪽33.69504/32.70079/26.77695%다. 새119쪽review를 실제 표시·대조해 앞 문장/조건의 겹침 해소와 본문 소속을 확인했다. 작은 표 안의 간격·활자 차이는 남으며 이 개선을 전142쪽/fresh 통과로 보고하지 않는다. KPS65/66/73의 v161 비교도exit0/232.409초다(`kps-franchise-batch-selected-native-v161-result.json`, `native-franchise-batch-boundary-v161/franchise5966/overlay/overlay_metrics.json`).
- 동결v157 가맹사업 전쪽 진단은 실제5400.306초 timeout/exit124로 종료했다. SVG142쪽을 만들고74쪽만 래스터했으므로 전쪽TSV·성공 근거가 아니다. 원래service의MainPID0/inactive와동결CLI불변을확인했고관측timeout만으로재시작하지않았다(`franchise-whole-native-diagnostic-v158-result.json`).
- 실제GDB에서행11의빈저장재시작유닛23직전h698.9866667px/예산869.32px·최소셀선언90.7066667px를확인했다. 저장50664+1100+660HU와동일한점유인데선언최소상자를전진높이상한으로써물리경계판정이false가됐다. 이후내용을앞쪽에넣어142쪽이문단중간줄로시작한다. 최초breakpoint의NOT HIT와row6의일반스캔관측을교정한정상GDB실행과구분한다(`remaining-source-frame-forensics-v155/franchise-empty-reset-scan-retry1-v161-gdb.log`, `franchise-terminal-row-unit-diagnostic-v161.log`).
- v162의`stored_empty_frame_restart.rs`는원본빈0재시작줄과뒤저장줄의정확한LH+LS전진,현재프레임의원본유닛합과마지막저장전진끝/문단뒤간격의일치를함께확인한다. 선언셀높이는최소공간이며내용점유상한으로사용하지않는다. edited/dirty/reflow·합성줄·중첩/개체경계·양수로컬재시작·전진불일치는새근거로승격하지않는다. 기존쪽규모검사와정상대조조건을완화하지않고이독립원본일치를같은일반행스캐너가소비한다. 이어받는빈재시작유닛의저장전진은`empty_unit_owns_flow_line_box`에도공유해무높이소비와실제배치의차이를줄인다. 기대값·baseline·원본·새회귀는바꾸지않았다.
- 같은소스묶음의fmt/Native빌드와원본142쪽트리/쪽컷·KPS·정상eogu/rowbreak·CGMP쪽수진단을별도service로진행한다. 검증전후소스/동결CLI를기록하며새각쪽점수와최종Native/fresh90%·정식회귀/lint통과는아직미검증이다(`stored-empty-source-frame-controlled-change-v162.json`, `stored-empty-frame-observations-v162-result.json`).
- 종료한v159KPS전쪽77SVG와timeoutv157가맹사업142SVG만총219개/17,514,319,870byte를압축없이삭제했다. 해당service와동결CLIworker가없고ignored경로임을확인했다. KPS77PNG/TSV와가맹사업74PNG,모든PDF/원본/tree/로그/manifest·현재검증·공유target을보존했다. 가용공간29→46GiB다(`finished-whole-diagnostic-svg-cleanup-v162.json`).

- v162 fmt/Native빌드는exit0/32.184초·104.046초,동결CLI SHA `460f8aecabafdc4995502870b056f1dab9b573ea299d7b41cb3dbc1ac101da79`다. source불변의전142트리및5원본진단도모두exit0이며KPS77/가맹사업142/eogu21/rowbreak18/CGMP48쪽이다. 가맹사업133/135/138/139/141의제목은Print205.382/852.422/269.222/446.822/306.022px에Native205.1/852.1/268.9/446.5/305.7px로대응한다.142쪽시작컷은30→23으로복구돼원래빈0줄과뒤문단첫줄을소유한다. 각쪽점수/fresh보정완료로확대하지않는다(`stored-empty-frame-observations-v162-result.json`, `franchise-whole-row-heading-origins-v162.json`).
- 125쪽첫제목은여전히Native946.2/Print967.142px다. 원본Row6앞제목0·다음TAC호스트0은별도프레임이며,자식4282+바깥283+283=호스트LH4848HU,뒤줄5508=4848+LS660HU로저장전진이정확하다. 현재최소셀33132HU중앞조각은제목만소유한다. v163은현재본문/바깥아래여백경계까지수용한물리밴드를스캔에서예약해동일end-row override를paint에전달하고뒤자식유닛을원래컷1로유지한다. 선언전체행의빈잔여를다음내용에다시씌우지않는다(`stored-heading-physical-frame-controlled-change-v163.json`).
- v163 fmt/Native빌드는exit0/33.258초·79.820초,동결CLI SHA `4d4d0697ea0b63e34a2ad46ec2e9133d107241b7babe1cbe3accbed5801b2e0f`다. 같은원본전142트리/5문서쪽수는v162와동일·source불변이며뒤쪽제목대응도유지한다.125제목은961.2px여서아직차이가있다. 실제GDB에서scan과paint의같은행상자61.013333px·실제제목콘텐츠14.666667px·Center를확인했고,제가`cell_cut_visible_height`의패딩/전진포함27.226667px를잉크정렬높이로잘못선택한오류를발견했다. budget원점은정상3.773333px/offset0/placementNone으로확인했으므로잘못된앵커가설을패치하지않는다(`franchise-heading-final-consumers-v163-gdb.log`). v164는이미선택한제목의실제줄구성높이를사용하도록이소비를고친다. 다른span경로의기존helper/예약높이는그대로다.
- v163동결CLI의KPS3쪽및가맹사업앞/뒤17쪽비교는원래service로계속한다. 이후v164소스변경과별도로보존하고현재소스의최종증거로올리지않는다. v164의성공빌드/전쪽트리·정상대조뒤125쪽실제review를우선만든다. 기존회귀기대값·원본·90%기준변경/추가체리픽·원격조치는없다(`stored-heading-ink-consumer-controlled-change-v164.json`, `franchise-heading-native-v164-result.json`).

- v164 fmt/Native빌드는exit0/28.315초·49.415초,동결CLI SHA `bba5489dcb2e7853108f49410f04904723b818cbdeaeb6ee2cd00ee88de2788e`다. 새전142트리/5원본쪽수진단도모두exit0·source불변이고뒤쪽제목대응을유지한다.125쪽제목은967.5px로복구됐으며독립Print글자상자967.142px와대응한다. 같은실제Native125쪽review는exit0/103.318초·99.99868%·source불변이다. `native-franchise-heading-v164/franchise5966/review/review_125.png`를사용자에게실제표시하고앞본문·나/다제목·마지막구입강제제목과그앞빈밴드·뒤쪽번호를직접대조했다. 활자획/굵기의차이는점수와구분한다. 전체142쪽/fresh통과나시험지제거·KPS기대값수정을이한쪽으로대신하지않는다(`franchise-heading-native-v164-result.json`, `stored-empty-frame-observations-v164-result.json`).
- 기존KPS1156·가맹사업5966/6718와정상7207/6756/6923검사를파생7suite에서한필터로실행한다. Native빌드와진단의terminal/source불변·빌드serviceMainPID0을확인한뒤고정target/pr-review에서Cargo를순차로시작했다. 현재검사는실행중이며기존기대값그대로다(`stored-source-frame-focused-v164-result.json`). 아직전체Rust·세Clippy/workspace·최종전쪽Native/fresh·원18개편집저장본/Studio·개별PRreview/오늘할일·사용자승인후PR/CI/merge/후속처리는미완료다.

### 진행135 — 앞쪽 미달 쪽의 재발견·원본 종료 상자와 각주 표시 소속 보정(v165–v168)

- 위 v164 기존 검사 실행은 종료했다. 24개 중 21PASS/3FAIL, exit100, 396.446초이며 source는 불변이다. KPS1156의37쪽 행 컷, 가맹사업5966/6718의143쪽 기대값이 실패했다. 정상7207/6756/6923 검사도 함께 실행했으나 전체 Rust 통과를 의미하지 않는다. 기존 검사·기대값은 그대로이며 전쪽 Native/fresh90%와 독립 기대값 확인 전에 변경하지 않는다(`stored-source-frame-focused-v164-result.json`, 해당 로그).
- 동결 v163 가맹사업17쪽 Native는 exit0/2174.514초로 종료했다. 최저10쪽95.12192%이고 뒤쪽133/135/138/139/140/141/142도 모두90% 이상이다.125쪽은 v16399.58803%와 실제 제목 위치 차이가 있었고 v164 별도 재출력99.99868%로 교정했다. v163의 source_unchanged=false는 뒤 v164 수정과 구분하는 기록이며 최종 전142쪽/fresh 자격으로 올리지 않는다(`kps-franchise-batch-selected-native-v163-result.json`, `native-franchise-batch-boundary-v163/franchise5966/overlay/overlay_metrics.json`).
- v165 fresh WASM은 optimized native wrapper 경로로 exit0/1604.261초, source불변·pkg/public JS/WASM 해시 일치로 종료했다. WASM SHA는 `155fda37a470dc7455efa867d1f784f6a4a156a93288906bb6698e2e94b023c2`이며 동결 패키지는 `wasm-pkg-source-frames-v165`다. Docker executable이 없는 서버의 cached wasm-opt 경로이며 Docker 빌드라고 보고하지 않는다. 이후 v168 코드 변경 때문에 최종 source의 fresh 증거로 재사용하지 않는다(`fresh-wasm-source-frames-v165-result.json`).
- timeout한 v157 전체 진단의 보존 PNG74쪽을 같은 표준 실루엣 계산으로 측정했다. 최저71쪽33.61299%이고 미달은1/2/9/19/38/39/41/42/43/56/57/68/69/70/71의15쪽이다. 이는 역사 빌드의74/142쪽 진단이며 현재 출력이나 전쪽 gate 통과가 아니다.9/39/43/57/71의 canonical review를 생성해 실제 표시·대조했다.9/43은 종료 표 상자와 뒤 본문이 일러지고,39는 앞 쪽 본문 두 줄을 잘못 소유하며,57/71은 표 내부·각주 소속 차이도 남는다(`frozen-franchise-prefix-v157-silhouette-v165-result.json`, `frozen-franchise-prefix-reviews-v166/review/`). 뒤쪽 경계의 개선만으로 문서 보정 완료를 판정하지 않는다.
- 9쪽 원본은 셀10788−첫개체4452+바깥140+140=뒤 빈 문단6616HU,43쪽은19650−2443+283+283=17773HU로 각각 정확히 닫힌다. 실제 v164 emit에서 이어받기 컷은 `[2]`와 원본 재시작을 소유하지만 single_cell_closing_frame은 둘 다None, 본체는70.96/218.10667px였다. 기존 생산 술어가 바깥 아래 여백0·공백도 없는 빈 문자열만 요구하고 소비가 HWPX만 허용해 유효한 Native 종료 상자를 버렸다. v168은 원본 종료식의 본체만 반환하고 바깥 여백은 기존 예약·배치가 별도로 소유하도록 고쳤으며, dirty/편집/재조판·원본 재시작 없는 컷을 제외한다(`remaining-source-frame-forensics-v155/franchise-early-closing-source-frames-v166.json`, `franchise-early-closing-consumers-v166-gdb.log`).
- 38→39쪽 원본447문단은 저장 줄4에서0으로 재시작하지만 v164 실제 분할은0..2/2..6이었다. GDB의 현재 높이727.41333px·각주31.39333px·줄 예산70.04667px와 두 줄66.61333px, 강제 경계None을 확인했다. v168은 모든 각주 표시의 원시 UTF-16 줄 소속이 뒤 프레임에 있고 앞 프레임 잉크가 현재 실제 각주 영역 위에 있을 때, 그 표시 때문에 앞 본문의 원본 앵커/쪽 소속을 버리지 않도록 같은 강제 경계·스캔 조회를 연결한다. 표/그림·앞 프레임 각주·잉크 겹침·dirty/edited는 이 새 근거에 해당하지 않는다. 첫 GDB 시도의 Deref 멤버 직접 조회 오류는 재시도에서 실제 `st.data`로 교정했고 실패 시도를 성공 증거로 세지 않는다(`franchise-early-paragraph-budget-retry1-v167-gdb.log`).
- v168은 위 두 원인을 한 묶음으로 수정했고 고정 target/pr-review의 fmt/Native 빌드, 전142쪽 트리와 KPS/eogu/rowbreak/CGMP 대조, 앞 미달15쪽과 인접 정상 쪽을 포함한25쪽 Native TSV를 시작했다. 원본·기대값·baseline을 변경하지 않았다. 실제 출력 개선, 앞뒤 내용 소유·간격 판독과 최종 전쪽 Native/fresh·회귀/lint는 아직 미검증이다(`early-franchise-physical-frame-controlled-change-v168.json`, `kps-franchise-batch-native-v168-result.json`, `stored-empty-frame-observations-v168-result.json`, `early-franchise-native-silhouette-v168-result.json`).
- SVG의 반복 full-font 임베딩 비용을 줄이는 private scratch 사본은 글자 cmap/GID/outline/advance/hint 검사와 실제 래스터를 구분했다. v165 retry2 및 원본 raw 테이블을 보존한 v167 retry1 모두125쪽/KPS65쪽 PNG가 원 글꼴의 같은 v164 출력과 동일하지 않아 채택하지 않았다. 원 글꼴 파일·기본 공급 경로는 변경하지 않았으며 v168은 기존 원 글꼴을 사용한다. 실패한 최초 raw-glyph 진단의 glyph 이름 조회는 GID/원본 cmap으로 교정해 다시 실행했고 어떤 실패도 글꼴 예외나 렌더 통과로 올리지 않는다(`private-font-raster-equality-v166-result.json`, `private-font-raster-equality-v167-result.json`, `private-raw-glyph-fonts-v167-result.json`).
- 최초18개 외 추가 체리픽·원격 게시·push/PR 생성은 없다. 현재 직접 보정 KPS/가맹사업과 유지한 CGMP 최종 검증, 원18개 편집/Studio·개별 review/오늘할일, 사용자 승인 후 PR/CI/merge/후속처리는 계속 미완료다.

- v168 fmt/Native 빌드는 exit0/33.614초·94.353초, 이후 전쪽 트리와5원본 진단도 exit0·source불변으로 종료했다. KPS77/eogu21/rowbreak18/CGMP48쪽은 유지하지만 가맹사업은 **143쪽/Print142쪽**이므로 보류다.9쪽 표 본체70.96→84.48px, 뒤139문단205.1→220.5px·140문단235.1→250.5px로 원본 종료식에 대응한다.38→39의447문단 컷은0..2/2..6→0..4/4..6으로 독립Print와 맞지만,41→42의472문단 소속도 교정된 뒤42쪽 표477의 앞 조각이43으로 밀렸다. 이143쪽 출력을 성공·새 기대값으로 고정하지 않는다(`kps-franchise-batch-native-v168-result.json`, `stored-empty-frame-observations-v168-result.json`, `franchise-all-frame-trees-v168/`).
- 실제 한컴 Print41/42를 표시해472의 앞5줄/뒤1줄·31/32와33/34/35/36 각주 소속을 대조했다. 성공한 v168 GDB의42쪽 표 진입은 현재 높이632.82667px·각주162.71333px이며 기존 internal-reset 조회false다. 실제 원본 셀 줄은 **0,0,1872,…**이고 별도 source_vpos 사본은None이다. 따라서 원시 좌표를 다른 캐시에서 읽는 패치를 추정으로 넣지 않는다. 양수→0만 찾는 기존 조회가 이 두 상자의0→0 경계를 놓쳐40px 안전 여유를 유지하고, 앞 줄+패딩 최소 요구18.42667px에 약0.93px 모자라 다음 쪽으로 이월했다. 최초 GDB의u8 포인터 직접 멤버 조회 실패는 typed Python lookup으로 교정했고 후속 정상 exit0 관측만 근거로 쓴다(`franchise-first-frame-footnote-query-retry1-v168-gdb.log`, `franchise-split-budget-v168.log`).
- v169는 **독립 cellSz·첫 선언 상자·뒤 빈 문단의 종료식이 정확히 맞는 두 프레임**에서 첫0→0도 조회한다. 이 증거를 표 진입의 실제 각주 경계와 첫 조각 예약에 같이 연결하고, canonical 유닛의 다음 저장 프레임 컷을 보존한다. 첫 선언 본체는 이미 수용한 내용 이상·실제 예산 이하일 때만 동일 end-row override로 paint에 전달하며 끝 조각의 종료식도 유지한다. 일반 로컬0·여러 재시작·dirty/편집·각주 영역 초과는 새 근거로 승격하지 않는다. 셀 내부 다른 컨트롤 경계의 반례는 별도 검증 전까지 미검증으로 남긴다. 기존 전체 셀 최소 높이나 절대px 오프셋으로 내용을 고정하지 않는다. 새 묶음 fmt/Native·전쪽 트리/5원본·25쪽 TSV는 실행 중이며142쪽 복구나 시각 통과는 아직 미검증이다(`first-and-closing-physical-frame-controlled-change-v169.json`, 대응 v169 결과 파일).

- v169 fmt/Native 빌드는 exit0/32.899초·93.506초, 동결CLI SHA `eb1812f88c08e09b5cd401a43cdb339c25033020d1f628bbf9959d7875ab8471`다. 전쪽 트리와5원본 진단은모두exit0·source불변이며 **가맹사업142쪽/Print142쪽**으로 복구됐다. KPS77/eogu21/rowbreak18/CGMP48쪽도 유지한다.42쪽은472의뒤1줄→474/476본문→477앞컷`[]→[1]`,43쪽은477뒤컷`[1]→[]`→뒤본문의 원래 소속이며, 앞뒤 유닛을 임의로 더 소비하지 않았다. 시각/전체회귀 통과는 별도다(`kps-franchise-batch-native-v169-result.json`, `stored-empty-frame-observations-v169-result.json`, `franchise-pages-empty-source-frame-v169.json`).
- 쪽수 미달이 확인된 v168의 나머지 선택 래스터는 목적을 잃었으므로 제가 만든 해당 unit만 명시적으로 종료하고 취소 이유·전후 실제 handle을 기록했다. 관측 timeout 때문의 중단/재시작이 아니며 원본·실패 SVG/로그·트리와 v169 실행은 보존했다(`early-franchise-native-silhouette-v168-result.json`). v169의25쪽 Native TSV와 기존24개/정상 saved-frame 회귀는 현재 실행 중이다. 기대값 변경·추가체리픽·원격조치는 없다.


### 진행136 — 첫 저장 상자의 최종 배치 소비 확인과 실제 upstream 전진 동기화(v169/v170)

- v169 기존 집중 회귀는 실제 종료했으며 **24개 중21PASS/3FAIL**, exit100/348.376초·source불변이다. 기존 KPS1156의37쪽 행 컷과 가맹사업5966/6718의143쪽 기대값이 실패했다. 정상7207/6756/6923 검사는 통과했지만 전체 회귀 통과가 아니다. 기존 검사·기대값을 변경하지 않았다(`stored-source-frame-focused-v169-result.json`, 대응 로그).
- 같은 v169 새 직접 review의9/38/39/42/43쪽은95.12142/91.57016/97.23559/90.00948/95.39884%다.9/39쪽은 원본 종료 상자와 앞뒤 본문 소속 개선을 직접 확인했다. **42쪽은90% 이상이어도 표 첫 글줄이 각주 영역에 겹쳐 실패**로 판독했다. 점수 통과로 결함을 면제하지 않는다(`franchise-early-direct-review-v169-result.json`, `native-franchise-early-direct-review-v169/review/`).
- 실제 v169 GDB에서 scan/emit의42쪽 첫 상자는32.573333px이나, paint의 `owns_declared_opening_frame=false` 뒤 일반 쪽 하단 고정이 행 높이를220.186667px로 덮어쓰는 것을 확인했다. v170은 같은 독립 두 프레임 종료식과 canonical 다음 저장 쪽 컷을 최종 paint에서도 소비하게 하고, 수용된 첫 상자가 정렬·클립·테두리를 소유하도록 연결했다. 문서 ID·임의 위치 보정·원본·baseline을 추가하거나 변경하지 않았다(`franchise-first-frame-final-paint-v169-gdb.log`, `first-physical-frame-final-paint-controlled-change-v170.json`).
- v170 fmt/Native 빌드는exit0/27.404초·93.335초이며 동결CLI SHA는 `c9f36b74e361497be21a0bd2ae3b33323e18e6b785330f3f642a46eff6557ac6`다. 빠른 전쪽 트리 및5원본 진단도 모두exit0·source불변이며 KPS77/가맹사업142/eogu21/rowbreak18/CGMP48쪽을 유지했다.42쪽 표 상자는32.6px, 첫 글줄792.8px이며 독립 Print793.062px와 대응한다. 예약된 원본 상자가 paint에서 다시 늘어나지 않는다(`kps-franchise-batch-native-v170-result.json`, `stored-empty-frame-observations-v170-result.json`).
- 새 v17042쪽 Visual Sweep은exit0/110.780초·source불변, **94.27957%**다. 실제 `native-franchise-first-frame-v170/franchise5966/review/review_042.png`를 사용자에게 표시하고 본문→표→구분선→각주33–36의 순서와 표/각주 비겹침을 직접 확인했다. 작은 활자·간격 차이는 남으며 이 한 쪽을 전142쪽/fresh WASM 자격으로 확대하지 않는다. 앞쪽25쪽 v169 진단은 동결CLI에서 계속 실행 중이고 v170 및 다음 리베이스의 최신 증거와 구분한다.
- 사용자가 upstream 동기화 후 리베이스를 지시했다. fetch에서 실제 `upstream/devel`은 `790a6f8f388445e7bff2bcb165aa7b384f9ece80`에서 `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`로 전진했다(#7832 통합). 이 부분 보정·증거를 먼저 커밋하고 새 base 위로 리베이스한다. 기존 결과를 리베이스 뒤 source의 검증으로 재사용하지 않으며, 원18개 외 추가 체리픽/원격 게시·push/PR 생성은 없다.
- 남은 작업은 가맹사업 앞쪽 미달 쪽과 중첩 표/각주 소속 보정, 현재 source의 KPS77·가맹사업142·CGMP48 전쪽 Native/fresh 및 직접 확대 판독, 필수 전체 회귀·lint/정책·편집/Studio 검증, 개별18개 review/오늘할일과 사용자 승인 후 통합 PR·CI·merge·후속 처리다. 이번 커밋은 부분 개선을 고정하는 단계이며 완료·승인 후보가 아니다.


### 진행137 — 실제 upstream #7832 통합 뒤 리베이스와 공통 소비 경로 결합(v171)

- 사용자 지시로 보정 단계 `fa5852140c12f05f461bf235c74a4265b3143fc6`을 먼저 고정하고, 실제 전진한 `upstream/devel` `bfa28a77a2714482f647ee6a7b9aa7f7876d8404` 위로136개 로컬 커밋을 리베이스했다. 리베이스 직후 head는 `f4d7e899651c1486359b1de251c63b4b3e3a7a17`이며 local devel도 bfa28a77a다. 새 base가 head의 조상임을 확인했고 이전 head는 `backup/semanticist21-before-bfa28a-v170`으로 보존했다. 원18개 밖의 추가 체리픽이나 원격 게시·push/PR 생성은 없다(`rebase-upstream-bfa28a-v170-{preflight,complete}.json`).
- 충돌에서는 upstream의 유효한 빈 저장 호스트 상자와 현재 보정의 빈 개체 앵커를 순서대로 보존했다. 배치의 쪽 원점/탭 우단과 글리프 클러스터 점유는 upstream의 공통 좌표·glyph-fit 계약을 유지하며, 현재 보정의 후속 런 폭/중첩 표 앞 원본 상자/실제 수용 행 목록을 연결한다. 걸침 셀의 빈 띠 차감과 수용한 행 상자는 서로 다른 소유자이며 두 상태를 함께 보존한다. 같은 각주 뒤 조각에 중복 큐를 만들지 않고 실제 새 쪽에서 한 번 등록한다. 이 병합 방향을 아직 시각/회귀 충족으로 판정하지 않는다.
- 기존 검사 충돌은 upstream에서 이미 이관한 원본의 검사·원장 행을 되살리지 않는 방향으로 해소했다. 현재 범위 밖 편람 Q8과 이관된 issue1949의 배치·편집 검사는 사용자 승인 범위에 따라 계속 분리하며 다른 정상 검사·독립 합성 입력은 보존한다. 시험지/HML 원본59개는 리베이스 전 backup과 현재 바이트 SHA가 전부 같고 모두 존재한다(`rebase-bfa28a-exam-originals-preserved-v171.json`).
- 오래된 파생 suite가 upstream에서 삭제한 원본 test를 참조해 첫 fmt가 실패했다. 소스 파일을 복구해 검사를 되살리는 대신 최신 manifest `--prepare`로 현재1460source/6228static attrs의28harness/20exception을 재생성했다. 파생 결과는 커밋하지 않는다. v171 fmt는exit0/26.976초이며 첫 Native 빌드는exit101/94.764초, 중복 glyph 점유 정의·종료 상자 함수 이름·탭 우단 인자·제거된 컷 필드의6개 통합 오류였다. 이것은 제품 회귀 검사 결과가 아니며 Native 통과로 보고하지 않는다(`kps-franchise-batch-native-v171-result.json`, 대응 로그).
- 통합 오류는 한 묶음으로 정정했다. 여러 앞 조각 뒤의 종료 **하단 좌표** 계약은 upstream을 유지하고, 독립된 두 상자의 종료 **본체 높이** 계약은 별도 이름으로 보존하여 좌표/높이를 서로 바꾸어 소비하지 않는다. 공통 쪽 원점 함수와 각주 큐는 단일 구현을 쓰며, 탭 필드 소비자에도 같은 text-start 원점을 전달한다. v171 retry1 Native 빌드와5원본/전쪽 트리·같은Print42쪽 직접 review는 진행 중이다. 리베이스 전 PNG/TSV·fresh 패키지를 새 source의 최종 검증으로 재사용하지 않는다.
- 동결 v169 선택25쪽 진단은exit1/1950.224초로 종료했고 source_unchanged=false다. 미달은1/2/19/41/56/57/68/69/70/71의10쪽이다. 이 역사 진단의142/142쪽과 다른 선택15쪽 점수는 새 리베이스 head의 전쪽 통과가 아니며 남은 원인 분석의 입력으로만 쓴다. 현재 필수 전쪽Native/fresh90%·직접 확대 판독·전체 회귀/lint·개별18개review/오늘할일·승인 후PR/CI/merge/후속 처리는 미완료다.

- v171 retry1 fmt/Native 빌드는 exit0/27.959초·174.660초이며 동결 CLI SHA는 `d76ae6a671a4205de477cdb18fed509fe6936c3931e385e233b2c026ee455d68`다. 그러나 전쪽 트리는118개 출력 뒤 exit101/10.348초로 중단했다. 중첩 표의 빈 본문 문단 배열에서 `original_body_closed_table_frame`의 호스트를 직접 인덱싱한 통합 오류이며, 뒤42쪽 Visual Sweep은 의존 검사 실패로 실행하지 않았다. 빌드 성공과 실제 렌더 실패를 구분한다(`stored-empty-frame-observations-v171-retry1-result.json`, 해당 stderr).
- retry2는 본문 종료 프레임 보정에 기존 최상위 소유 조건과 실제 호스트 존재를 함께 요구한다. 중첩 조각에는 이 본문 계약을 적용하지 않으며 해당 조각의 기존 셀 경로를 유지한다. 시험지 원본·회귀 기대값·baseline은 변경하지 않았다. 고정 target의 fmt/Native와 전쪽 트리/5원본·독립 Print42쪽 재검증은 진행 중이다.

- v171 retry2 fmt/Native는 exit0/28.571초·82.403초, 동결 CLI SHA `123e656246f62ef5f82c4ee560e4674fcc24e6bb05c563fcc2b12f1de767c7c9`다. 전142쪽 트리와5원본 쪽수 진단은 모두 exit0·source불변이며 KPS77/가맹사업142/eogu21/rowbreak18/CGMP48쪽을 유지했다. 새42쪽 Native Visual Sweep도 exit0/113.979초·source불변·94.27957%다. 실제 `native-franchise-first-frame-v171-retry2/franchise5966/review/review_042.png`를 표시하고 본문→표→구분선→각주33–36의 순서와 비겹침을 직접 확인했다. 리베이스 전과 점수가 같아도 새 source에서 재출력한 증거이며 전쪽/fresh/전체 회귀 충족을 의미하지 않는다. 이번 통합 오류 수정과 기록을 일반 커밋으로 고정하고 남은 보정을 계속한다.


### 진행138 — 초기18개 범위의 PR 준비와 현재 source 최종 검증(v172)

- 사용자 지시 `PR 준비`에 따라 초기18개 통합 scope를 유지하고 본문·원 PR별 review의 로컬 초안을 준비한다. 라우팅은 `collaborator_external_pr`이며 보조 문서는 `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `visual_fixture_evidence`다. 새 원 PR 추가 체리픽·source head 갱신·원격 게시·push/PR 생성은 하지 않았다. base `bfa28a77a`, code head `10282f63d`를 고정했다.
- 읽기 전용 API 확인에서 #7612는 이미 merged이며 기존 archive review를 보존한다. #7615의 현재 원격 head는 `4a6c27c5f`로 초기 수용 head `ed9c28475`와 다르므로 새 의존 변경을 가져오지 않는다. 초기 나머지17개는 open이며 초기 적용과 현재 원격 상태는 `pr-preparation-original18-remote-state-v172.json`과 metadata에 구분했다.
- 실제 전142쪽 트리에서57쪽 이어받은 행은102.3px이고71쪽 표는340.5px다. 새 upstream의 중첩 각주 경로가 반영되어67–70의 내용도71쪽 각주 영역에 모인다. 이 구조 진단을 실제 Print 일치로 승격하지 않고 전체 Native/fresh TSV와 필요한 직접 review에서 대조한다.
- 고정 target에서 manifest prepare·fmt check→Native/WASM/workspace 세Clippy·workspace build·base 비교 정책→최적화 fresh WASM→전체 nextest12threads/no-fail-fast→Native Skia3종을 순차 실행한다. 독립 Native 전쪽(KPS77·가맹사업142·CGMP48)과 fresh 패키지 성공 뒤 같은 원본의 WASM 전쪽 TSV를 연결했다. live service는 `rhwp-review-v172-rust-validation`, `rhwp-review-v172-native-whole`, `rhwp-review-v172-wasm-whole`이며 ignored 결과/로그의 실제 상태와 끝 exit를 수집한다. 실행 시작을 통과로 보고하지 않는다.
- 디스크 확보는 종료한 과거 Native 실행의 임베딩 SVG123개만 제거하여10.880GiB를 확보했다. 경로·크기·SHA와 종료 상태를 `completed-native-svg-cleanup-v172.json`에 보존했으며 원본·PDF·PNG·TSV·공용 target은 그대로다. 현재 전쪽 실행도 종료한 뒤 임베딩 SVG만 같은 방식으로 정리한다. 압축 보관하지 않는다.
- 정식 개별18개 최종 review·오늘할일은 필요한 검증 종료 뒤 실행 결과로 확정한다. 현재 초안은 ignored `output/pr-review/semanticist21-20261008/pr-preparation-v172/`에 두며 통과·승인·PR 준비 완료를 주장하지 않는다. 전쪽 미달/실패가 나오면 원본·Print·base/head를 대조해 승인된 원인 보정 또는 기존 이관 범위의 부적합 검사 제거를 진행한다. 현재 직접 보정 KPS·가맹사업·CGMP를 이관 근거로 제외하지 않는다.

- v172 PR 준비의 manifest prepare와fmt check는exit0/2.677초·30.680초, Native/WASM32 Clippy는exit0/154.098초·139.575초다. workspace build와후속 workspace Clippy/정책·fresh WASM·전체 nextest/Skia는 아직 종료 전이다. 단일 Cargo 체인 뒤release build/lib·doc test·Studio TypeScript/unit/production·public binding 계약도 순차 연결했다(`pr-preparation-composite-validation-v173-result.json`). tracked source/test/helper4309개의 SHA를 보존하여 원 head·후속 문서 변경과 생산/검사 입력 바이트를 구분한다. 로컬 review 초안18개는원 적용41개 모두현재 rebase commit에 연결했고, 정식 완료 판정/오늘할일은 최종 검증 뒤 확정한다.

- v172 workspace build는exit0/264.459초이나 all-target Clippy는exit101/295.406초다. `tests/envelope_integrity_contract.rs`의 승인된 시험지 제외 selector가 `corpus_scope`를 참조하면서 module 선언을 빠뜨린 리베이스 통합 오류를 확인했다. 기존 공통 helper를 연결하는 선언만 추가했다. 시험지/HML 입력·기대값·renderer source는 변경하지 않으며, 기존 Native/WASM Clippy와workspace build의 생산 source 동일성을 유지한다. 실패한workspace Clippy와fmt/정책부터v173 순차 체인으로 재검증한 뒤fresh/전체 회귀/Skia를 계속한다. 뒤composite v174는그종료뒤 실행한다. v172 오류로fresh미생성일때 종료된 대기 worker는 실패·미실행 기록을 남겼으며, WASM전쪽은v174 service에서fresh성공과Native전쪽 종료/중간SVG정리뒤 실행한다. 디스크 확보를 위한 의존 순서 조정이며 시각 관측 timeout에 따른 재시작이 아니다.

- 2026-10-12 사용자 요청으로 원 PR별 review18개를 저장소에 분리했다. 기존 merged #7612 기록을 보존하고 나머지17개에는 최초 원 commit→적용→현재 rebase 매핑과 PR별 변경 파일·현재 계약 함수 목록을 연결했다. 판정은 검증 진행에 맞춰 머지 보류이며, 종료 결과와 원본/Print·전체Native/fresh/확대 판독을 받은 뒤 각 PR의 검증 범위와 수용 여부를 확정한다. 오늘할일·승인 후PR/CI/merge/후속 처리는 여전히 미완료다.


## 139. 초기18개 메인터너 보정: KPS 전쪽 Native와 현재 호출 경로 확인 (2026-10-12)

사용자의 메인터너 보정 지시를 유지한다. 초기18개 외 추가 체리픽·원격 변경은 수행하지 않았다.
production source는 `10282f63db5f671eca8554613342e20211a025ac`의 Native 빌드와 동일하며,
모듈 선언 복구 `2d6f82fbba427035eccfddaa818b5de4bb878e31` 뒤 v173 fmt·workspace/all-target Clippy와
base `bfa28a77a2714482f647ee6a7b9aa7f7876d8404` 대비 manifest/unit 정책을 각각 exit0으로 재확인했다.

| 현재 실행 증거 | 결과 | 판정 범위 |
| --- | --- | --- |
| `pr-preparation-native-kps-whole-v172-result`의 KPS 단계 | exit0 / 2401.043초 / source unchanged | Native 전체77쪽 산출·비교 명령 종료 |
| `pr-preparation-native-kps-whole-v172/kps1156/silhouette.tsv` | 최저42쪽90.91935%, 미달0·누락0, RHWP/PDF77/77쪽 | Native 전쪽 실루엣 선행 기준 충족; fresh 및 전체 직접 판독은 별도 |
| `native-kps-critical-page-review-v175/review/review_042.png` | 같은 현재 PNG를 재출력 없이 canonical compare/overlay/review로 구성 | 본문 줄바꿈·문단 순서·쪽번호 위치 보존. 글자별 폭·공백 차이는 남음 |
| `row-plan-composition-kps-cgmp-observer-v175-result.json` | KPS exit0/20.516초, CGMP exit0/9.311초, relief event0·observer error0 | 가맹사업 v173과 합쳐 세 원본의 실제 호출에서 중복 보정 발동은 관측되지 않음. 일반적인 상호 배타성의 입증으로 확대하지 않음 |

상대 증거 경로의 공통 prefix는 ignored `output/pr-review/semanticist21-20261008/`다.
KPS TSV의 `pr_review_gate=not_evaluated`는 직접 판독/본문·각주 소유 검토가 별도로 필요하다는 뜻이며,
PR 승인 또는 전체 검증 완료로 바꾸어 해석하지 않는다. 최저42쪽 review와 이미지 SHA는
`native-kps-critical-page-review-v175/evidence.json`에 연결했다.

fresh WASM은 v173 실제 unit의 최적화 단계에서 계속 실행 중이다. 성공한 새 package를 기다려
v175 public API·현재 저장본 capture·Studio 실제 mouse activation과 동일 SHA Print가 있는 편집 축의
Native/fresh 비교를 실행하도록 연결했다. 현재 저장본에 같은 입력 SHA의 독립 Print가 없으면
옛 PDF로 승인하지 않고 새 Print를 산출한다. 현재 각주 style7조건도 저장본을 capture하며 검사 기대값을
변경하지 않는다. 각 단계의 명령·종료 결과·package hash·source 동일성을 기록한 뒤 개별 PR 판정을
갱신한다. 전체 nextest·Native Skia·전쪽 fresh·Studio 복합 검증 및 남은 직접 판독은 **미검증**이다.


## 140. 같은 source의 fresh WASM·공개 API·Studio 실제 클릭 완료 (2026-10-12)

v173 optimized wrapper 빌드는 exit0/1400.614초로 종료했다. Native 빌드 이후 production source1107개는
동일하며, root `pkg/`와 `rhwp-studio/public/`의 JS/WASM은 byte 동일하다.
`pr-preparation-fresh-wasm-v172-result.json`의 frozen package와 실제 Studio 응답을 SHA로 연결했다.

- WASM SHA-256: `ed5248ba4b2a6e765029411021cfd67a55d6c5dded80ca60f2bd971126430581`
- JS SHA-256: `691e4fd480cf4e04f9e599ca7c7a4802ad87b2c141fba93bf94111f4144dcb3b`

| 개별 PR | 새 package의 관측 | 증거 파일 |
| --- | --- | --- |
| #7566 | 공개 API4조건 PASS; 실제 Studio mouse click에서 guide2→1, SECONDGUIDE와 전체 셀 경로 유지, pageerror0, served WASM SHA 일치 | `wasm-clickhere-wrapper-current-v175.json`, `studio-wrapper-field-pointer-v175/evidence.json` 및 before/after PNG |
| #7614 | 논리 caret의 본문 삽입7조건×반환/현재/HWP/HWPX28PASS | `wasm-html-logical-caret-contract-v175.json` |
| #7638 | header/footer10조건×현재/HWP/HWPX60PASS, 현재 FileName 문맥 저장본20개 capture | `wasm-hf-and-note-contracts-v175.json`, `captured-current-public-hf-context-v175.json` |
| #7641 | 좁은 중첩 표가 인접 셀 앞에 포함됨 PASS | `wasm-public-contracts-v175.json` |
| #7647 | malformed21개 거부와 입력 byte 보존, valid45개 독립 기대 결과:66PASS | `wasm-formula-complete-contract-v175.json` |
| #7648 | 주석 삭제·혼합·시작번호5조건×현재/HWP/HWPX15PASS | `wasm-hf-and-note-contracts-v175.json` |
| #7653·#7656 | 표 바깥 여백과 다음 본문 점유 관계 PASS | `wasm-public-contracts-v175.json` |
| #7654 | 표시한 주석 글자 경계와 caret의 관계3조건 PASS | `wasm-public-contracts-v175.json` |
| #7657 | 삭제 대상 이외의 중첩 셀 스타일을 현재/HWP/HWPX에서 유지 PASS | `wasm-public-contracts-v175.json` |
| #7658 | style tab의 현재/HWP/HWPX roundtrip PASS | `wasm-public-contracts-v175.json` |
| #7659 | 새 각주/미주7조건×현재/HWP/HWPX21PASS, 현재 저장본7개 capture | `wasm-hf-and-note-contracts-v175.json` |

공개 API runner6개는 모두 exit0이며 production source 동일성을 각각 다시 확인했다.
이 계약 관측을 독립 Print 일치나 모든 원 PR 적용 경계의 완료로 승격하지 않는다.
특히 #7614의 셀 paste·자동번호 뒤 슬롯, #7658의 기타 border 조건 등 원 검사 범위는
전체 Rust 회귀와 해당 시각 증거에 별도로 연결한다.

편집 시각 v175는 현재 후보43개 중42개의 입력 SHA에 정확히 대응하는 독립 Print를 검증했으며
Native/fresh를 새로 실행 중이다. 새 wrapper 입력과 현재 note-style7조건은 같은 Print가 없는 경우
`pr-preparation-current-input-print-v175-result.json`에서 실제 engine·제품·Print method0·출력 mode를
확인해 새 PDF를 산출한다. 옛 PDF의 이름 또는 유사한 모양으로 입력 대응을 대신하지 않는다.
전체 회귀·전쪽 fresh·남은 확대 판독이 완료되기 전 판정은 여전히 보류다.

사용자 지시: 모든 필수 테스트·전쪽 Native/fresh 비교·직접 판독이 완료되고 실패/미달이 해결되면,
개별 PR 문서의 최종 판정을 **메인터너 보정으로 수용 가능**으로 갱신한다.

## 141. 최종 검증에서 확인한 실패와 글꼴 환경 정정 (2026-10-12)

검토 브랜치는 `review/semanticist21-20261008`, base는 `bfa28a77a`다. production source는
`10282f63d` 이후 동일하고 검증 입력·기대값·baseline을 변경하지 않았다. 사용자 지시대로 모든
필수 테스트와 검증을 완료한 뒤 PR과 후속 처리를 진행한다. 현재는 다음 실제 실패가 있어
**머지 보류**를 유지한다. 진행 중인 실행 건수와 마지막 종료 결과를 구분한다.

| 검증 | 현재 관측 | 다음 처리 |
| --- | --- | --- |
| 전체 nextest v173 | 총10,287개 중8,115개 종료 시점에서21FAIL 관측; 전체 실행 중, 최종 summary 아님 | `pr-preparation-failures-observed-v185.json`에서 원본·함수별 실패를 모아 실제 회귀와 부적합 기존 검사를 대조한 뒤 일괄 보정 |
| 현재 note-style HWP 각주 저장본 | Native/fresh1쪽88.40005%, 독립 Print와 차트·두 번째 단의 본문 흐름 차이 | `pr-preparation-current-print-{native,wasm}-remaining-visual-v178/current-03-b3c9896ea898/`의 같은 입력·Print·PNG 보존, 원인 수정 후 재검증 |
| 이름이 각주인 스타일이 없는 저장본 | Native/fresh3쪽, 같은 입력 SHA의 독립 Print2쪽; 측정 불가/쪽수 불일치 | `pr-preparation-current-print-{native,wasm}-unmeasured-visual-v182/current-04-6b4f6a375029/`의 실패 보존, 기대 쪽수를 바꾸지 않고 출력 보정 |
| CGMP Native 전쪽 | 원래 raster43쪽 Chrome30초 timeout으로 중단. 기존42쪽을 보존하고 같은 source의43–48쪽만 올바른 환경에서 완료 | `cgmp-native-completed48-v180/silhouette.tsv`:48/48쪽·최저47쪽94.05301%·미달/누락0. fresh 전48쪽과34/35/47쪽 직접 판독은 별도 |
| 가맹사업 원래 글꼴 환경 | 첫 font directory의 함초롬바탕 대체가 실제 휴먼명조보다 먼저 선택됨. 원래2쪽88.91905% | 실제 설치 HMKMM을 먼저 공급하고 독립 Print의 TrueType 실현을 기존 `FontEnvironment.trueTypeFaces` 계약으로 지정 |
| 가맹사업 정정 환경1·2쪽 | Native/fresh 각각99.60405/99.93459%, 두 backend 동일. 여는 따옴표와 다음 글자 겹침이 사라짐 | `franchise-original-human-font-{native,wasm}-v179/` 및 환경 JSON. 선택2쪽 결과를 전142쪽 통과로 승격하지 않음 |

가맹사업의 HMKMM 원본 SHA-256은
`d033df71fa41de48905407f7d39aca74ebd9ae87298c5420513acaa187fbefc6`이며 설치 글꼴 바이트를
수정하지 않았다. 기존 generic SVG outline 정규화와 실제 TrueType 프로그램의 문장부호 폭을
사용한다. 임의 폭 상수·문서 ID 분기·굵은 글꼴 교체를 추가하지 않았다. 원래 굵기는 이미 Print와
같은 stroke 방식이므로 굵기 변경 가설은 부정 진단으로 남겼다.

잘못된 환경의 Native 가맹사업은32쪽 raster를 보존하고 종료했다. 아직 렌더하지 않은 이전
WASM 전쪽 대기 job도 잘못된 환경을 반복하지 않도록 교체 대상으로 기록했다. 별도 예전 휴먼명조
WASM 진단은 실제1800초 subprocess 제한으로 먼저 실패했으며 완성된 시각 증거가 아니다.
이 상태·실제 handle과 종료된 SVG428개/24.687GB의 SHA·경로를
`known-invalid-font-environment-superseded-v181.json`에 보존한 뒤 SVG만 삭제했다. PNG·TSV·tree·
원본·PDF·공용 target은 보존한다. 출력 공백이나 관찰 timeout 때문에 실행을 재시작한 것이 아니다.

남은 현재 저장본은 v183에서 입력별 canonical gate를 그대로 실행하며 미달을 기록하고 다음
입력을 계속 확인한다. CGMP fresh 전48쪽은 v184에서 실행 중이다. 같은 작업의 매 수정마다
빌드하지 않고 실패 원본·소비 경로·독립 기준을 먼저 모아 보정한 뒤 영향을 받는 최종 source를
한꺼번에 검증한다. 정확한 기존 관계 검사를 미달 문서라는 이유만으로 일괄 제거하지 않는다.
이관 후보는 현재 보정 대상인지와 고정 base의 실제 미달·기존 #7445 범위를 함께 확인한다.

v183의 추가 Native 관측은 wrapper1쪽35.95134%, HWP 미주1/2쪽88.39849/4.39631%,
HWPX 미주1/2쪽88.68987/4.39631%, HWPX 각주1쪽88.69135%다. wrapper 원본과 같은 SHA의
새 Print를 직접 대조하니 Native의 뒤 두 중첩 표에 여분의 외곽선과 세로 점유·뒤 문단 위치 차이가
보인다. #7566의 실제 Studio 클릭 PASS를 이 저장본의 Print 일치로 바꾸지 않는다. 미주2쪽의
점수와 쪽/번호 원점, HWP/HWPX 차트와 단 흐름을 실제 출력으로 분석하며 해당 기대값·기준 PDF는
유지한다. 추가 fresh 결과는 아직 실행 중이다.

Native Skia3종·복합 release/lib/doc/Studio 검증·전쪽 fresh·남은 직접 판독·PR/CI/merge/후속 처리는
아직 완료되지 않았다. 새 head로 source가 바뀌면 관련 이전 결과는 역사 증거로 남기고 최종
검증에 재사용하지 않는다. 조건부 수용 지시를 현재 승인 판정으로 앞당기지 않는다.


## 진행142 — 전체 회귀 종료와 기준 브랜치 대조, 원점 보정 묶음

검증 기준은 `upstream/devel`의 `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`로 유지한다.
전체 nextest v173은 exit100으로 종료했다. 최종 결과는 **10,287개 실행: 10,258 PASS,
29 FAIL, 45 SKIP**, 테스트 실행 2,819.236초다. 진행141의 21실패는 중간 관측이며
최종 실패 수가 아니다. `whole-nextest-terminal-failures-v197.json`에 고유 검사29개와
원 로그를 연결했다. 후속 Skia·release/lib/doc·Studio 전체 검증은 이 실패 뒤 미실행이다.

동일 입력 SHA19종(경로20개)을 frozen base Native와 frozen 통합 Native에서 각각
실행한 `failed-original-anomaly-base-head-v188-result.json`의38명령은 모두 exit0이다.
이는 진단 명령 성공이며 출력 무회귀 판정이 아니다. 기준에서는 없던 K-water24쪽47겹침,
inner-table2쪽38겹침과 보도자료 첫쪽 넘침이 현재 출력에서 새로 발생했다. 기존 미달 이관이나
래칫 완화로 처리하지 않고 실제 생산·소비 원점을 추적했다.

| 원인과 독립 근거 | 보정한 생산·소비 경로 | 현재 검증 상태 |
| --- | --- | --- |
| K-water24쪽의 부모 셀은 그대로인데 자식 표가484.5px에서172.8px로 이동. 부분 셀 누적 커서를 저장된 표 뒤 호스트 줄로 해석하고 표 높이를 다시 뺌 | `layout_partial_table_cells`의 저장 줄 원점 소비 여부를 기록하고, `stored_nested_float_placement`의 `NextLine` 결과는 실제 저장 호스트 원점에서만 수용. 전체 셀의 기존 원점 계약과 일치시킴 | 수정 전 frozen CLI의 실제 호출·값은 `kwater-inner-final-origin-forensics-v190`. 수정 후 검증은v194/v196에서 진행 |
| 미주2쪽의 원본 page fill은SHOW_ALL이며 독립 Print에 워터마크가 있음. 기존 painter는 이미지 배경을 첫쪽으로만 제한 | `layout.rs` page background image도 색·그라데이션과 같은hide_fill/SHOW_FIRST 조건을 소비 | 수정 전 Native/fresh2쪽4.39631%. 수정 후2쪽 진단·PNG 대조 대기;1쪽 차트 차이는 별도 미해결 |
| 공백4자를 소유한 보도자료 문단19는 실제8px 글줄이 따로 출력됨. 빈 앵커 상자가 줄간격을 담고 painter가 다시 더해 하단3.81333px 넘침 | `stored_empty_host_margin::prepare`가trim 후 빈 문자열을 글줄 없는 앵커로 간주하던 조건을 제거. 실제 빈 문자열만 표 포맷 상자로 처리 | 실제 호출의표 끝1015.57333px·공유 흐름 끝1020.65333px·호스트 줄1023.85333px는 `first-body-overflow-origin-v193`. 수정 후 전체 원본 재측정 대기 |

v194는 네 source 파일의 조건·한국어 주석 변경을 한 번에fmt/Native build하고 생산 소스
해시를 고정한다. v196은 그 성공한 frozen CLI로 동일 원본19종 진단과 대표 영향 페이지
비교를 수행한다. source/test 기대값·baseline은 이 진행에서 변경하지 않았다.

CGMP의 frozen v172 fresh WASM48쪽은v184에서exit0/2,145.332초로 종료했고 최저47쪽
**94.05301%**, 미달0이다. Native48쪽도 같은 최저값이다. 실행 중 새 원점 보정이 시작돼
`source_unchanged=false`이므로 이 결과는 **보정 전 소스 증거**로 보존하며 새 보정의 통과로
재사용하지 않는다. KPS77쪽과 편집 저장본의 같은 frozen WASM 작업은 실제 실행 핸들을
확인하며 계속 관측한다.

현재 저장본의 wrapper35.95134%, 각주·미주1쪽88%대와 스타일 저장본3쪽/Print2쪽 불일치는
여전히 보류 사유다. 미주 배경 보정으로 이 차이까지 해결됐다고 판정하지 않는다.
차트 Contents의기존 parser는VtChart6 혼합형9만 지원하고 해당 저장본은컬럼형1이다.
기존 진단 parser로 같은Contents의제목·범례·색·축 메타데이터를 확인했으며,
`note-legacy-presentation-inventory-v195.json`·`note-legacy-full-metadata-v195.txt`에 보존했다.

**판정은 머지 보류를 유지한다.** 모든 필수 검증과 직접 확대 판독이 끝난 뒤에만 각PR의
보류 사유를 “메인터너 보정으로 수용 가능”으로 변경한다. 아직 push·PR 생성은 하지 않았다.


진행142의 묶음 Native 빌드는exit0/313.826초, fmt는exit0/51.674초다.
frozen CLI의SHA-256은 `543a3c46744a7ad558368ff35e5b8cfc77423f318dc825c680a1d2846e6e7add`이며
`batch-origin-native-v194-result.json`에 생산 source 해시를 연결했다.
v196의19개 원본 재측정에서 보도자료7종의 새 본문 하단 넘침1→0,
K-water 두 입력의 새겹침47→0·47→0, inner-table의38→0을 확인했다.
API·화학연구·#7333·59043·none-image-fill의 기존 관측 실패는 이 묶음에서 해소되지 않았다.

대표 Native TSV는K-water24쪽97.63240%, inner-table2쪽97.64176%다.
K-water의 `batch-origin-native-selected-visual-v196/kwater2024/review/review_024.png`를
직접 대조했다. 자식12×2표는y172.8에서484.5로 돌아와 부모 셀 안에 있고,
요구사항 행의 내용·순서는 PDF와 대응한다. 부모 외곽 아래와 가운데 정렬 문구에는
높이 차이가 남는다. 이 페이지의97% 점수를 표 외곽 완료 근거로 사용하지 않는다.
보도자료1쪽·미주2쪽의 새출력 비교는 같은 frozen CLI로 계속 실행한다.
회귀 기대값·baseline은 아직 변경하지 않았으며, 전체 문서·fresh WASM·최종lint의 통과로
이 대표 Native 결과를 확대하지 않는다.

## 진행 143 — 저장 표 뒤 호스트 줄과 물리 하단·쪽 원점의 소비 경로 보정

기준은 `upstream/devel`의 `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`,
보정 전 커밋은 `8b9a4ce6eb943510d47ee4b416a8cd302b816a7b`다.
이번 진행에서는 회귀 기대값·baseline·원본 입력을 변경하지 않았다.
최종 전체 회귀의 마지막 종료 결과는 여전히 진행142의 **29실패**이며,
아래 진단 결과를 전체 nextest 통과나 현재 실패 수로 바꾸어 보고하지 않는다.

| 원본·독립 근거 | 실제 생산·소비 연결과 보정 | 관측 결과·한계 |
| --- | --- | --- |
| K-water24쪽의 중첩 표 앞 줄 끝, 표의 바깥 상자, 표 뒤의 빈 호스트 줄과 다음 줄이 같은 저장 프레임을 닫음. 마지막 원본 줄 끝45253HU와 독립 Print의 부모 하단은 기존 측정보다 호스트1952HU만큼 큼 | `stored_block_cell_flow`가 짧은 **표 뒤** 간격을 저장 표 높이의 누락으로 오인해 합산 흐름을 선택하고 그 호스트 줄을 빼던 조건을 교정. `stored_nested_before_host_frame`의 원본 인증을 측정·배치가 공유하여 완전한 사다리는 저장 셀 높이 경로를 유지 | v204 Native24쪽99.95378%, inner-table2쪽99.95784%. 선택 페이지·보정 중간 소스의 증거이며 최종 전체/fresh 검증은 대체하지 않음 |
| 취업취약계층 수급 요건 원본의 바깥 아래 여백141HU. 독립 Print의 앞3쪽은 미완료 표 조각이 이 여백을 남기고, 마지막쪽은 내용 끝에서 닫힘 | `prepare_table_fragment_budget`에서 물리 하단을 먼저 산출하되, 뒤의 `fragment_placement`가 기본 예산을 대체한 **후** 실제 `table_top`으로 제한. 스캐너가 수용한 물리 높이→예약→부분 셀 외곽선이 같은 끝을 소비 | v210·v217의 실제 출력 진단은 수정 전FAIL/수정 후PASS. 전4쪽, row7의 두 조각 최소높이215.6px, 모든 원본 셀 내용의 순서·단일 소유와 셀/글줄 포함 관계를 확인. 정식 nextest는v220에서 별도 실행 중 |
| 화학제품 연구76쪽의 첫 문단107은 실제 쪽 나눔이며 저장vpos4000HU=문단 앞간격. 다음 문단108의7320HU=4000+1400+420+1500이 독립된 앞간격 사다리를 증명. Print의 표 윗변은861.2px | 새 `stored_page_origin`이 명시적 쪽 나눔만으로 유효한 원점을 제외한 가정을 제거. `vpos_snap_current_height`의 쪽 기준과 `build_single_column`의 같은 원점 판단을 공유하고 캡션 표의 예약·실제 원점에 연결 | v219 실제 출력 진단: 앞 문단 하단806.9px는 보존, 캡션780.6→834.0px, 표808.0→861.3px. 앞뒤 순서와 첫/다음 문단의 원본 사다리 거리 검사는FAIL→PASS. 전체·fresh 및 직접 PNG 판독은 아직 완료되지 않음 |

첫 두 시도(v204/v207)의 고용지원 하단 여백은 **FAIL→FAIL**이었다.
실행된 분기의 반환값을 GDB로 확인한 결과 기존 물리 프레임 인증은 이미true였으며,
인증을 넓히던 중간 `table_layout.rs` 변경은 제거했다. 실제 원인은 뒤의 원점 선택이
줄인 예산을 덮어쓰는 소비 경로였다. `physical-frame-boundary-v208/gate-v204.log`,
`gate-fileline.log`, `budget-overwrite.log`와 `budget-trace.log`에 실패와 호출 값을 보존했다.
적용 조건을 계속 덧붙여 화면을 맞추는 예외는 남기지 않았다.

v209는 fmt27.517초·Native build86.600초로exit0이며,
CLI SHA-256은 `94cbd4944679804024f327cf6746a0cbd14ad80e1ff3a501f9d08b32128ce7c3`다.
이 중간 소스의 고용지원 Native 전4쪽 최저는95.83454%다(v211).
쪽 원점 보정을 포함한 v216은 fmt30.029초·Native build63.264초로exit0,
CLI SHA-256은 `2151ffa87d859bf5ab4c9554dc1d69ad654b2f7a1c9a80e636ed1cb37b08450c`다.
각 빌드 기록의 생산Rust1107개 파일 해시와 `source_unchanged=true`를 확인했다.
검증 이후 커밋 SHA와 이 파일 해시의 연결은 별도로 확인한다.

같은19종 원본의v218 재측정에서 이전 보정의 보도자료7종 넘침0,
K-water 두 입력·inner-table의 새 겹침0은 유지됐다.
화학제품 두 원본은 각각 본문 하단 넘침5→4, 텍스트 겹침15→6으로 줄었다.
API·59043·none-image-fill·#7333의 남은 관측은 해소되지 않았으며,
화학제품의 나머지 차이도 완료 또는 기존 이관으로 분류하지 않는다.

진단·실행 기록은 ignored `output/pr-review/semanticist21-20261008/`의
`physical-frame-boundary-v210/`, `physical-frame-boundary-v217/`,
`chemical-caption-relations-v219/`, `remaining-regression-source-diagnosis-v212/`,
`batch-physical-frame-native-v216-result.json`,
`batch-physical-frame-validation-v218-result.json`에 보존한다.
v218은 화학제품 영향5쪽·issue1853 대조4쪽 및 고용지원/K-water/inner-table 전체
Native TSV를 산출한다. 선택 비교는 원본 전체 통과 근거가 아니다.
기존 관계 검사9개는 `physical-frame-focused-nextest-v220-result.json`으로 실행 중이다.

v213 fresh WASM은 화학제품의 추가 원점 결함을 확정한 뒤 보정 묶음에 포함하기 위해
중단했다. 자기 소유 unit의MainPID0/inactive 및 Cargo/Rust 작업 종료를 확인했고,
`fresh-wasm-source-frames-v213-result.json`에 중단 이유를 남겼다.
패키지 통과·실패나 최신 fresh WASM 증거로 세지 않는다. 공용target은 삭제하지 않았다.
KPS·CGMP의 이전 frozen package 전쪽 결과도 최신 생산 소스의 통과로 재사용하지 않는다.

**전체 검증·직접 판독 미완료이므로 머지 보류를 유지한다.**
개별 PR의 “메인터너 보정으로 수용 가능” 전환, push와 새 PR 생성은 아직 수행하지 않았다.

진행143의 v220 정식 검사도 종료했다: 기존 #6950/#7552의 **9검사 전부PASS**,
397개 선택 제외, nextest exit0/409.395초(실행 summary2.518초)이며 당시 생산 소스는
v216과 같았다. 이후 뒤 간격 보정을 추가했으므로 이 결과는 새 v224 소스의 검사로
재사용하지 않는다.

v218의 중간 Native 영향 비교는 화학제품5쪽40.96947%,45쪽97.91849%,
57쪽57.04931%,76쪽98.07514%,81쪽99.83384%였다.
76쪽 `physical-frame-selected-review-v221/chemical-study/review/review_076.png`를
직접 표시·판독했다. 캡션은 앞 문단 뒤에 있고 표 외곽·머리행·첫 두 데이터행은 Print와
대응한다. 목록 원형 기호의 크기 차이는 남아 있어 미세 모양까지 완료로 판정하지 않았다.
5쪽 `physical-frame-selected-review-v222/chemical-study/review/review_005.png`도
직접 표시했다. 첫 제목 자체는 맞지만 다음 문단·표는26.7px 위에 있고 다음쪽 제목과
본문까지 미리 소비하여 아래로 넘친다. 두 미달 쪽을 평균으로 상쇄하지 않았다.
issue1853 대조8/10/13/37쪽의 같은 중간 소스 Native 값은
97.60638/96.34071/97.41296/96.39927%다. 전체·fresh 통과로 확대하지 않는다.

추가 원인은 같은 공통 원점 helper로 옮기며 **첫 문단의 뒤 간격을 빠뜨린 것**이었다.
원본42의 vpos2000·줄높이2000·줄간격1200·뒤 간격1000HU와 다음43의 앞 간격4000HU는
다음vpos10200HU를 닫는다. 고정 base의 `layout.rs` 기존 함수도 이 뒤 간격을 포함한다.
`stored_page_origin`에 그 항을 복원하여 빈 간격으로 무효 판정하던 사다리를 교정했고,
측정과 배치는 같은 결과를 계속 소비한다. 문서ID·고정 화면 좌표·내용 숨김 조건은 추가하지 않았다.

이 보정을 포함한 v224 fmt51.092초·Native build117.976초는exit0이며,
frozen CLI SHA-256은 `8700ac5a21dda40429be0c46f599bbd571732c7508b353ce42993bc9e9507133`다.
v225의 고용지원 전4쪽 내용/행 최소/하단 관계, v226의 캡션 앞뒤·사다리 관계는 각각
수정 전FAIL/수정 후PASS를 유지했다. v228은 원본dump와 독립 Print로 첫/다음 문단 거리와
쪽 소유를 별도로 대조했다. 첫/다음 거리82.7→109.4px는 원본109.33333px와 대응하고,
5쪽의 탈출 문단53–56은 없어졌다. 독립 Print6쪽의 `3. 연구의 주요내용`도6쪽으로 돌아와
같은 관계 검사가FAIL→PASS였다. 진단의 통과를 신규 회귀 추가나 전체 시각 통과로 바꾸지 않는다.

v227의 같은19종 재측정에서 화학제품 두 원본은 각각 본문 하단 넘침4→0,
지면 밖2→0, 텍스트 겹침6→0이다. #7333 겹침은4→3이며 완전히 해소되지 않았다.
API의 새 넘침, 59043의 추가쪽·넘침, none-image-fill의 새 넘침은 여전히 남는다.
v227 Native 영향/대조/전체 TSV와 v229의 세Clippy·workspace build·고정 base 정책 검사·
기존9개 nextest 재검증은 진행 중이다. 최신 fresh WASM과 전체 회귀·전체 시각 검증은 미완료다.

v227의 최신 v224 Native 영향5쪽도 종료했다: 화학제품5/45/57/76/81쪽은
**99.50561/97.91849/99.23289/98.07514/99.83384%**, 미달0이다.
이는 원본103쪽 중 영향5쪽이다. 전체 원본 통과로 확대하지 않는다.
새5쪽 `physical-frame-selected-review-v230/chemical-study/review/review_005.png`를
직접 표시·판독했다. 제목 다음 간격·표 외곽·뒤 본문과 다음쪽 내용의 소속은 Print와
대응하며 잘못 추가한 하단 제목은 없어졌다. 목록 기호의 작은 크기 차이는 별도로 남긴다.
issue1853 대조4쪽도 중간 소스와 같은 수치로90% 이상을 유지했다.

v229는 최신 v224와 동일한 생산 소스에서 **모든9단계 exit0**로 종료했다.
Native Clippy151.477초, WASM Clippy101.145초, workspace build152.836초,
workspace all-targets Clippy128.018초 및 fmt·고정 base manifest/unit 정책을 통과했다.
기존 #6950/#7552 정식 nextest는427.160초(실행 summary1.125초), **9검사 전부PASS**,
397개 선택 제외다. 모든 단계의 `source_unchanged=true`를 확인했다.
전체 nextest·Skia·Studio·fresh WASM 완료를 뜻하지 않는다.

다음 검증은 v232의 화학제품 두 원본 각각103쪽, CGMP48쪽, KPS77쪽 전체 Native다.
실제 unit을 확인하며 실행 중이고 종료 전 SVG는 삭제하지 않는다.
API18쪽은 고정 base와 최신 실제 출력의 표·뒤 본문을 대조했다.
기준은 표149.6px/Example989.3px, 최신은153.4px/995.9px이며,
추가한 빈 호스트 fallback이 anchor17.33333/위3.77333/표742.32/뒤13.57333px를
배치에 전달한다(`remaining-regression-source-diagnosis-v212/api18-saved-host.log`).
이 값은 원인 조사 자료이며 아직 API 수정·해결로 보고하지 않는다.
원본의 유효 저장 앵커와 fallback의 여백 소유 근거를 다음 보정에서 대조한다.


### 진행144 — 빈 저장 호스트의 여백 소유를 원본 사다리로 인증(v235–v245)

사용자가 승인한 메인터너 보정을 계속한다. 검토 base는
`bfa28a77a2714482f647ee6a7b9aa7f7876d8404`, 이 단계 직전 head는
`0cce247c5d909be7b77bccf4a34d07861723606f`다. 추가 체리픽·원격 push/PR 생성은 없다.
현재 라우팅은 `collaborator_external_pr`와 `intake_and_review`·`local_validation`·
`multi_pr_update_branch`·`visual_fixture_evidence`다. 개별 PR 판정은 머지 보류를 유지한다.

#### 독립 근거와 실제 소비 경로

추가했던 `stored_empty_host_margin::prepare`는 빈 원본과 선언/측정 높이의 일치만으로
상자를 수용했다. 다음 저장 줄이 그 여백을 소유하는지는 확인하지 않고 표 위 여백과
문단 뒤 간격을 다시 더했다. API18쪽과 #7333의9/24쪽에서 실제 이 경로를 GDB로 확인했다.
음수 좌표 clamp·내용 숨김·문서ID 분기는 추가하지 않는다.

| 원본 저장 증거 | 기대 계약과 수정 |
| --- | --- |
| API의 앞 문단 마지막 줄은 새 쪽vpos0+1000+300=1300HU. 표 host1800은 문단 앞 간격500을 포함. 표 바깥 상자56475HU와1300을 더하면 다음 원본 줄57775를 정확히 닫음 | 사다리 인증에는 표 위 여백283이 아니라 문단 앞 간격500을 사용. 다음 줄의 원점이 이미 닫은 상자에 문단 뒤 간격500을 더하지 않음. 비대칭 여백의 기존 그림 원점과 전체 흐름 점유를 구분 |
| KPS73쪽의 앞 줄 끝5640, host6640, 문단 앞 간격1000, 표 offset3085+위283+본체33632+아래283=37283HU. 다음 줄42923=5640+37283 | 종이/쪽 기준의 nonTAC·흐름 및 줄높이 비참여 장식은 단독 흐름 표를 배제하지 않음. 문단 앞 간격·원본 offset·양쪽 여백과 다음 줄이 닫는 물리 상자를 유지 |
| #7333의 다음 저장 줄은 새 쪽으로 되감고 NONE-fill15의 다음 줄 델타1980은 표 offset433+높이3397을 닫지 않음 | 높이가 같다는 이유만으로 저장 원점을 대체하지 않음. 인증되지 않은 후보는 기존 분할·배치 경로가 소유 |

기존 `from_empty_stored_ladder_host`의 원본 인증을 작은
`float_placement/stored_empty_host_frame.rs`로 옮기고 흐름에 참여하지 않는 장식을 대조한다.
높이만 확인하던 별도 fallback과 모듈을 제거했다. 비대칭 여백의 기존 원점 판별은
`stored_float_anchor::stored_outer_box_table_top_hu`로 공유하며 기존 원본 앵커 동작을 유지한다.
최상위는 `entry.rs`의 같은 결과를 제약·예약·기록하고 최종 배치가 소비한다.
중첩 경로 `table_layout.rs::stored_nested_float_placement`도 같은 사다리 helper를 소비한다.
호스트의 문단 앞 간격과 표 바깥 위 여백은 별도 값으로 전달한다.

#### 실행과 실제 출력

v237 fmt33.654초·Native build123.381초는exit0이다.
동결 CLI SHA-256은 `58d9e265ff4715f6a2c7851ebf6bebf8a0464357d11ff1477d049ef5fbfe7d4a`이며
새 파일까지 포함한 생산Rust1107개 해시와 `source_unchanged=true`를 보존했다.

v239는 같은19종 원본 전체 geometry를 재검사했다. API의 본문 하단 초과는2→1로
고정 base의 기존1건과 같고, NONE-fill은1→0, #7333 텍스트 겹침은3→0이다.
화학제품 두 원본·보도자료7종·K-water 두 원본·inner-table의 앞 단계 개선도 유지했다.
59043은38쪽·새 하단 초과1건으로 남는다. 집계의 일치를 전체 시각 완료로 세지 않는다.

v241의 실제 출력 관계 검사에서는 API18쪽 Example의 본문 포함, #7333의9/24쪽
제목 뒤 표의 비겹침, NONE-fill1쪽 후속 본문의 포함이 각각 수정 전FAIL/수정 후PASS다.
같은 네 비교의 모든 TextRun 내용과 순서도 유지됐다.
KPS의 원본9008HU 표 윗변·42923HU 다음 줄은 각각 실제248.6/700.8px와 대응한다.
이 자료는 진단이며 정식 회귀 신규 추가·기대값 변경으로 대체하지 않는다.

| v237 Native 영향/대조 범위 | 페이지별 실루엣 |
| --- | --- |
| API18/25/32쪽 | 58.93130 / 98.30699 / 99.43698%;18쪽 미달 |
| KPS73쪽 | 97.62566% |
| #7333 9/24/40쪽 | 99.71248 / 99.66730 / 99.88917% |
| NONE-fill1/9쪽 | 98.80053 / 96.96571% |

API18쪽은 같은 원본·Print·글꼴로 고정 upstream 바이너리에서도 **58.93130%**이며,
base/현재 PNG의SHA-256까지 동일하다(v244/v245). 실제 새 넘침은 고쳤지만 원본의
기존 표 행 경계 차이는 남는다. 현재 원격 #7731의 같은 입력SHA와 #7445 이관 본문을
읽기 전용으로 확인했다. 이 문서는 초기 직접 보정 대상으로 추가하지 않으며,
원본과 정확한 검사 범위를 대조한 뒤 사용자 승인에 따른 부적합 회귀 처리를 진행한다.
이 단계에서는 검사 삭제·기대값 갱신·원본 이동을 하지 않았다.
`stored-empty-frame-selected-review-v243/api/review/review_018.png`를 표시·직접 확인했다.

v240의 fmt·Native/WASM/workspace-all-targets 세Clippy·workspace build·고정base
manifest/unit 정책은 모두exit0·source불변이다. #6950/#7552/#7203/#7333/#7235/
#7620/#6378/#1921의 기존 집중 nextest는 실제 unit이 live인 상태로 컴파일 중이다.
원래 기대값은 변경하지 않았다. 전체 nextest·Skia·fresh WASM·Studio의 완료가 아니다.

직전 v224의 K-water2024 전체27쪽 Native는최저3쪽96.61647%·미달/누락0으로 종료했다.
`physical-frame-selected-review-v236/kwater2024/review/review_024.png`를 실제 표시하고,
중첩 표·부모 마지막 행 외곽·뒤의 `끝`·쪽번호를 Print와 대조했다(24쪽99.95378%).
작은 획/AA 차이는 남는다. 이 증거를 v237의 최신 whole/fresh로 재사용하지 않는다.
v227의 원래 K-water 전쪽과 v232의 화학제품103쪽은 고정 v224로 계속 실행 중이다.
소스가 바뀐 사실과 다음 과제의 source 검사를 `source-correction-invalidates-v238.json`에
남겼으며, 종료 또는 관측 timeout을 이유로 재시작하지 않는다. 새 source의 qualification은 별도다.

59043은 최초 다른 컷을26쪽 pi163에서 찾았다(base86,현재83).
실제 `scan_rowspan_block_step → advance_row_block_cut_with_row_offsets → CellCutFrame`
호출의 row offset50.34667·padding3.76px와 앞/뒤 원본 줄의 페이지 되감김을 수집했다.
한컴 Print26/27쪽의 오른쪽 본문은77.12/77.44px에서 시작한다.
이미 소비한 앞 행의 원점·패딩을 스캔/예약/paint가 어떻게 공유하는지 다음 보정에서 확인한다.
48px sliver 허용치를 늘리거나 새 쪽수 회귀를 삭제해 통과시키지 않는다.

상대 증거 경로는 ignored `output/pr-review/semanticist21-20261008/`다.
`stored-empty-margin-source-v235/`, `stored-empty-frame-{native-v237,validation-v239,
focused-nextest-v240}-result.json`, `stored-empty-frame-boundaries-v241/`,
`59043-fragment-source-v242/`, `stored-empty-frame-api-existing-fidelity-v245.json`에 연결했다.
전체 Native/fresh·직접 확대 판독·전체 회귀가 미완료이므로 PR 준비 완료나 수용 전환은 보류한다.


v239의 모든19종 geometry와6개 Native 명령도 종료했다.
inner-table 전2쪽은99.82283% 이상이며, API·59043 명령은미달로exit1,
나머지4명령은exit0이다. 전체 미달0이나 PR gate 통과로 보고하지 않는다.
모든 관측의 `source_unchanged=true`를 확인했다.

#7333의 새9쪽 `stored-empty-frame-selected-review-v246/7333/review/review_009.png`도
실제 표시·판독했다. 스크린샷 뒤 `3.1.1 메인화면`과 다음 표의 순서·간격은 Print와
대응하며 앞 단계의 제목/표 겹침은 없어졌다. 작은 활자·그림 주석·꼬리말 그림의
잔여 차이는 전체50쪽/fresh 검증에서 계속 대조한다.
59043 선택30/31/32쪽은31.01006/22.68149/18.24037%로 쪽 소속 차이가 실제로 남는다.


v240 집중 회귀도 종료했다: **37검사 실행,36PASS/1FAIL,10295 선택 제외**,
실행 summary3.606초·nextest exit100이다. 유일한 실패는 기존
`issue_1921_59043_pagination_pin::regulatory_59043_page_count_pin`(37 기대/38 실제)이며
새 쪽수 회귀를 그대로 검출한다. #7203/#7333의 기존 실패와 #7620의 정상 대조군은
현재 검사 그대로 통과했다. 모든9단계 source불변이고 앞8단계는exit0이다.
전체 nextest 통과나 남은 실패 수1개로 확대하지 않는다.

소스는 `25bef3ef0c9ef3080f216903c8d81a7ba4a8170a`로 커밋했다.
`stored-empty-frame-committed-source-v247.json`에서 `git cat-file`로 생산Rust1107개를
동결 v237 빌드와 전부 대조했으며 불일치0이다. 미추적 Print PDF4개는 필요한 기준
자료로 보존하고 원본·시험지/HML·회귀 기대값·baseline은 변경하지 않았다.


### 진행145 — 소비한 앞 행의 높이를 다음 조각 예산에 중복 계상하지 않음(v249–v258)

검토 base는 `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`다. 현재 단계 직전
head는 `b766bde7e5ccc814aebf81006cfd6f3c545ab241`이며 추가 체리픽·원격 push/PR은 없다.
59043의 실제 새 쪽수 회귀를 먼저 구현에서 보정했다. 기존 검사·공차·baseline은 변경하지 않았다.

#### 독립 근거와 실제 소비 경로

원본 pi163의 앞 행 짧은 셀은 시작 컷2가 마지막 유닛2를 모두 소비했다.
그런데 기존 행 offset은 선언 높이3776HU(50.34667px)를 다음 조각에서도 더했다.
실제 partial paint는 같은 컷의 그 행을0높이로 처리하므로 스캔의 오른쪽 셀 예산만
50.34667px 줄었다. 독립 Print26/27쪽 오른쪽 본문 시작77.12/77.44px에는 반복 머리행이 없다.

`scan.rs::row_offsets_for_cut`는 시작 컷이 있을 때 기존
`row_block_cut_physical_height → row_block_cut_row_content_height` 판정의 같은 행 높이를
소비한다. `fragment_height`도 이 기존 판정을 작은 `row_cut_height`로 공유한다.
인증된 남은 물리 행 상자는 기존 helper가 보존하며, 높이가 양수인 남은 행 사이에만
간격을 계상한다. 시작 컷 없는 처음 조각의 원래 행 높이 경로는 유지한다.
`block_step.rs`의 주 스캔과 band retry가 모두 같은 컷별 원점을 전달한다.
paint의 per-row 컷은 이미 소비된 자기 행을0으로 만들고, rowspan의 남은 높이 차액은
마지막 표시 행에 실으므로 소비한 앞 행을 다시 원점에 더하지 않는다.

실제 `scan_rowspan_block_step → advance_row_block_cut_with_row_offsets → CellCutFrame`
GDB 관측에서 오른쪽 start39의 offset은50.34667→0, available969.42667,
content budget915.32→965.66667px다. padding3.76px와 기존48px sliver 계약은 그대로다.
요구 높이를 작은 값으로 수용한 뒤 paint에서 늘리거나 좌표를 clamp하지 않았다.

#### 실행 결과와 한계

v249 fmt33.050초·Native build97.492초는exit0이다. 동결 CLI SHA-256은
`f434172eed3dba1903780b17d8e80c1ff0b3e6d5b4eb0317adce6b77b73908b1`이며 생산 소스 불변을 확인했다.
59043은38→37쪽, 새 본문 하단 초과1→0이다. 전체37쪽의 문단·표·개체 소속과
시작/끝 컷은 고정 upstream 출력과 전부 같으며 다른 쪽0이다.
처음 달라졌던26쪽 pi163 끝 컷은83→86으로 복구됐다. 이 관측은 전체 시각 통과가 아니다.

v251은19종 원본의 전체 geometry를 완료했다. 고정 base보다 증가한 본문 하단 초과·
지면 밖·텍스트 겹침은0이다. API/#7333/NONE-fill·화학제품 두 원본·K-water·보도자료 등의
앞 단계 개선도 유지했다. v257의 별도 정상 대조에서 CGMP48쪽/KPS77쪽의 전체 item 소속과
컷도 수정 전후 같았다. 기존 CGMP의 Table 하단 초과1건(46쪽7.4px), KPS 텍스트 겹침2건은
증가하지 않았으나 이를 없어진 결함이나 독립 Print 판독 완료로 보고하지 않는다.

v252 prepare·fmt check·고정 base manifest 정책·집중 nextest는 전부exit0/source불변이다.
기존 #6950/#7552/#7203/#7333/#7235/#7620/#6378/#1921의 **37검사 전부PASS**,
10295개 선택 제외다. nextest 명령885.445초(컴파일 포함), 실행 summary4.079초다.
직전 같은 검사의37개 중1FAIL은 `regulatory_59043_page_count_pin`의37기대/38실제였으며,
같은 기존 검사를 변경하지 않고 PASS로 복구했다. 전체 nextest·최신 세Clippy·fresh WASM·
Skia·Studio는 이 실행에 포함되지 않는다. 마지막 전체 회귀29FAIL을1FAIL이나0FAIL로 바꾸지 않는다.

v251의 완료된 Native 선택 쪽은 API18/25/32쪽58.93130/98.30699/99.43698%,
KPS73쪽97.62566%, #7333의9/24/40쪽99.71248/99.66730/99.88917%다.
API18은 고정 base와 최신PNG까지 동일하다. 이미 #7445/#7731에서 추적하는 부적합
API 전용 렌더링24함수를 정확히 구분했고 모두 검토 base에 등록되어 있음을 확인했다(v255).
순수IR·다른 원본 검사·공통 helper는 유지 대상으로 구분했다. 이 단계에서는 아직 검사를 삭제하거나
원본을 이동하지 않았다. CGMP·KPS·가맹사업은 현재 보정 대상으로 유지한다.

이전 소스0cce의 v232 화학제품 연구 전체103쪽도 종료했다. 10쪽이90% 미만이고 최저79쪽
17.91785%다. 부모 runner는 소스 불일치 검사에서 종료해 다음 문서를 실행하지 않았다.
이 결과는 이전 소스의 실제 미달 근거이며 최신 head의 전체 결과가 아니다.
v250의15/79/85쪽 historical review PNG를 표시·직접 확인했다:15쪽은 그림 뒤 캡션/본문이
밀렸고79쪽은 로고 묶음 일부 소속·상단 위치가,85쪽은 제목/그림과 본문의 위치가 다르다.
최신 v251의 영향 쪽 및 정확한 base v258의15/79/85쪽 비교를 계속한다.
v258은 bfa28의 기존 v187 동결 바이너리SHA와 같음을 별도 provenance로 확인했다.

완료된 v232의SVG103개5,920,935,946byte(약5.514GiB)를 삭제했다(v254).
MainPID0/failed, 열린SVG handle0과PNG 양쪽103쪽·TSV 보존을 확인했고 압축하지 않았다.
원본·PDF·PNG·TSV·로그·공용 target은 보존했다. 공간 여유는 약38GiB다.

ignored `output/pr-review/semanticist21-20261008/`의
`remaining-row-frame-{native-v249,validation-v251,focused-nextest-v252}-result.json`,
`remaining-row-frame-boundaries-v253/`, `remaining-row-frame-positive-controls-v257/`,
`chemical-whole-historical-review-v250/`, `api-existing-transfer-audit-v255.json`,
`chemical-base-native-v258-{result,build-provenance}.json`에 실제 명령·원본SHA·관측을 연결했다.
현재 실행 중인 v251/v258은 종료 후 결과를 이어서 기록한다.
개별 PR의 머지 보류와 최종 전체 Native/fresh·확대 판독·회귀·lint gate는 유지한다.


진행145의 소스는 `5eda5d45f100f9d0d4635c9e65a09a341145d3b3`로 커밋했다.
v259에서 생산Rust1107개가 동결v249 빌드와 전부 같고 불일치0임을 확인했다.
커밋 뒤 fetch는exit0이며 upstream/devel은bfa28 그대로다. 사용자 지시대로 rebase하지 않았다(v260).
59043의 최신26쪽 `remaining-row-frame-selected-review-v262/review/review_026.png`를
직접 표시·판독했다(96.99%). 표 윗변과 앞/뒤 내용 순서·하단 이어짐이 Print와 대응하며
작은 획/AA 차이는 남는다. 전체37쪽/fresh의 시각 통과를 뜻하지 않는다.

v258의 정확한 upstream Native15/79/85쪽은69.53622/17.91785/22.66846%로 종료했고,
세PNG가 기존0cce v232의 같은쪽 출력과 바이트까지 같다. #7739의 원격OPEN 본문에 같은
화학제품 원본SHA가 있음을 읽기 전용으로 확인했다(v261). 최신v251 영향 비교는 계속 진행 중이다.

필요한 미추적 한컴 Print PDF4개를 대응 edited HWPX4개 및 입력/PDF SHA·쪽수·Creator/Producer와
함께 보존했다. 기준PDF는 `pdf/semanticist21-20261008/pr-preparation-current-inputs-v175/`,
입력 및 작은 출처JSON은 `mydocs/pr/assets/semanticist21-20261008/current-print-inputs-v175/`다.
모두 기존 v175의 실제 Hancom2020 변환 종료/다운로드 기록과 해시가 같다.
해당 편집 사례의 시각 보정은 아직 미완료이고 회귀 추가·기대값 변경·PR 수용 전환은 보류한다.
기존 시험지/HML59개와 모든 원본 파일의 내용·경로는 이 단계에서 변경하지 않았다.


### 진행146: API 기존 미달 회귀24개 이관과 보존 계약 검증

고정 base `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`의 API18쪽은58.93130%이며,
최신 생산 소스 `5eda5d45f100f9d0d4635c9e65a09a341145d3b3`도 같은 값이다.
같은 원본/독립 Print/글꼴 조건의 양쪽 PNG는 SHA-256까지 동일하다.
직접 review_018.png를 판독해 표 행 높이·누적 세로 위치와 글자 획 차이가 남음을 확인했다.
이미 #7445의 하위 #7731에서 같은 원본을 추적하므로 중복 원격 이슈를 만들지 않았다.
앞 단계에서 새 본문 초과를 구현으로 먼저 해결한 뒤, base에 이미 등록돼 있던 API 전용
부적합 렌더링 함수24개만 제거했다. 이는 API 출력 해결이나 시각 통과가 아니다.

전용5파일16검사를 제거하고 공유5파일의 API 검사8개만 제거했다.
공유 파일의 다른 원본 검사·정상 대조군·helper는 유지했다. 전용 renderer manifest1행과
text overlap/page count baseline 각1행을 제거했으며 다른 기준값·공차는 바꾸지 않았다.
활성 samples의 중복 API 원본은 기존 issue7445 자산의 동일262144byte/해시 원본으로 보존한다.
독립 Print PDF3개는 바이트/해시 그대로 `mydocs/pr/assets/issue7731/api-reference/`로 옮겼다.
이 경로의 `transfer_provenance.json`에는 검사 이름·변경 전 commit 링크·이전/새 경로와
해시를 기록했다. `review_018.png`/`overlay_018.png`는 이관의 부정 증거이며 새 golden이 아니다.
순수 structure-auto IR 계약은 이관 원본을 읽고, 공통 preserved-format helper로 전체 HWP5
roundtrip·IR field sweep의 기존 검사 및 IR baseline2행을 유지했다. ignore/조용한 return은 없다.

v267은 prepare·fmt check·Native Clippy·WASM lib Clippy·workspace build·workspace all-targets
Clippy·고정 base manifest/unit 정책·집중 nextest9단계가 모두exit0이다.
유지한 관련 문서 검사와 전체 HWP5 roundtrip16분할/IR sweep은47개 전부PASS다.
nextest230.211초(컴파일 포함), 실행 summary172.893초이며 선택 대상 내1550개 제외다.
검증 소스/test 해시를 v271에서 현재 작업트리와 재대조했고 생산Rust1107개는5eda와 전부 같다.
runner의 원래 head 필드는 v249의 이전 메타데이터를 상속했으므로 수용 SHA로 쓰지 않는다.
실제 검증 작업트리03c5e62b26ed181e74ebd82ff9895f72dc1e4ce3와 생산 SHA·해시 대조를
별도 필드로 정정했다. 최초 fmt는 stale generated suite를 준비하지 않아 실패했으며,
정식 prepare 후 fmt/lint를 전부 통과했다. 이 준비 오류를 제품 회귀로 세지 않는다.
시험지/HML 원본59개는 기존 v112와 내용/실행모드가 모두 같고 경로도 보존했다(v271).

v251 Native 선택 쪽 비교는 모두 종료했다. 화학제품 미달10쪽은 정확한 upstream의
같은 쪽과 PNG 바이트까지 동일함을 v258/v268/v269로 확인했다. 최저79쪽17.91785%이며
동일 원본SHA를 열린 #7739에서 확인했다. 59043의25/28/30쪽48.57985/78.26698/79.45617%도
정확한 base/current PNG가 전부 같다(v263/v270). 이들 이관 여부는 현재 보정 범위와
검사의 의미를 분리해 계속 검토하며, 이전 제거목록을 일괄 재적용하지 않는다.

ignored output의 `api-existing-transfer-{audit-v255,applied-v264}.json`,
`api-transfer-validation-v267-result.json`, `exam-originals-preserved-v271.json`과
각 단계 logs에 명령·원본 SHA·결과를 연결했다. 최신 전체 nextest/Native/fresh WASM,
Skia·Studio·독립 edited Print 사례의 차트/중첩표 보정은 아직 미완료다.
마지막 전체29FAIL을 이번47PASS로 대체하지 않으며 개별 PR 보류를 유지한다.


### 진행147: 표 뒤 OLE의 단 소유와 본문 흐름 분리 — 집중 검증 완료

API 이관 단계는80908c7cb88cc3e5c0cb3a90cb23ddfb2ddd8a17로 커밋했다.
이후 fetch도exit0이고 upstream은bfa28 그대로여서 rebase하지 않았다(v272).

독립 edited HWP-endnote Print의 차트 상단은136.01066px인 반면 이전 출력은113.37333px다.
v273의 실제 GDB 호출은 `controls::place_table_host_shape`에서 왼쪽 current_height745.44px,
기존 요구332.58667px로 전체 단을 이동해 오른쪽 current_height0을 확정했다.
`layout_shape_item → layout_column_shapes_pass → layout_shape`도 그113.37333px 원점을 소비했다.
원본은 비TAC OLE/TopAndBottom/문단Top/단Center, 위1417HU·아래2267HU 바깥 여백이다.
독립 Print와 이전 review를 직접 대조하면 차트만 오른쪽 단으로 이월되고 뒤 본문의 첫2줄은
왼쪽 표 아래에 남아야 한다. 기존 코드는 본문까지 오른쪽으로 옮겼다. Square로 분류하지 않는다.

보정은 `column_ole_placement`에서 세로 오프셋·위/아래 여백을 포함한 단 상대 상자를 확정하고,
현재 본문 커서를 남긴 채 다음 단에 개체만 예약한다. 일반 단 이동과 저장 다단 문단 분할의
실제 두 경로가 `activate_next_column_stored_frames`를 소비한다. 다음 단의 본문 예산과
ColumnContent는 같은 occupied_bottom을 쓰고, OLE paint는 같은 ink top을 소비한다.
layout_shape_item은 이미 확정된 예약을 다시 높이로 더하지 않는다. 쪽 나눔/구역 종료에서도
뒤 본문 유무와 관계없이 큐의 개체를 원래 다음 단에 발행한다. 여러 이월 개체는 순서대로 예약한다.
단이 없거나 대상 단에 전체 상자가 안 맞는 경우 및 캡션/다른 앵커 계약은 기존 경로다.
문서ID·특정 화면 좌표·새 clamp·baseline 공차 변경은 없다. 아직 해당 경계 전부의 실행은 미검증이다.

최초 v275 컴파일은 method visibility 오류로 실패했고 현재 v276에서 고쳤다.
v276 fmt31.431초·Native build126.806초는exit0/source불변이며 동결 CLI SHA-256은
62f2f44730d95ab5c82899694565534f292708b659272c0a8552b887273cb2fe다.
v279의 같은 문단9는 수정 전6줄 전부 오른쪽으로 독립 기대와 달라FAIL,
수정 후 첫2줄 왼쪽/나머지4줄 오른쪽으로 PASS다. 여섯 줄의 내용/순서가 전후 같고
누락·중복은 없다. 이는 진단 관계 검사이며 새 정식 회귀는 아직 추가하지 않았다.
차트 상단은132.3px로 개선됐지만 Print와 약3.7px 차이가 남는다.

v278의19종 전체 geometry 감사는 모두 종료했고 고정 base보다 증가한 본문 하단 초과·
지면 밖·텍스트 겹침은0이다. 쪽수도 같다. API는 보존한 이슈 원본의 부정 대조이며 활성 회귀로
복원하지 않았다. CGMP/KPS/가맹사업 최종 전쪽 시각 gate를 이 감사로 대신하지 않는다.

v280/v282에서 독립 edited Print8쌍을 모두 확인했다(v285 집계): blank endnote/footnote는100%,
HWP footnote는91.23518%, HWP endnote는91.23437/99.07373%, HWPX endnote는
91.50201/99.07373%, HWPX footnote는91.50275%다. Missing-named-style은3쪽/기준2쪽으로
미달이며 ClickHere 중첩 wrapper는35.95134%로 미달이다. 페이지 불일치에서 단일 batch가
중단하므로 남은4쌍은 각각 실행해 실패해도 다음 쌍을 계속 비교했다.
v277 최초 실행은 systemd의 Chrome 경로 누락으로 실패했고 v280/v282는 실제 설치 경로를
VISUAL_SWEEP_CHROME으로 명시했다. 환경 오류를 시각 회귀나 수용 증거로 세지 않는다.

`column-ole-direct-v284/review/review_001.png`를 표시·직접 판독했다(91.23437%).
왼쪽 표 아래 두 줄과 오른쪽 후속 내용 소속은 개선됐다. 차트의 축 최대값/plot 크기/범례 배치,
글꼴·색상·불투명 배경과 작은 원점 차이는 여전히 Print와 다르므로90%를 수용으로 바꾸지 않는다.
Native19 감사·진단 관계 검사·8쌍 점수를 최종 fresh WASM/전체 회귀 통과로 보고하지 않는다.

v281은 prepare·fmt check·Native Clippy·WASM lib Clippy·workspace build·workspace all-targets
Clippy·고정 base manifest/unit 정책·기존 frame/OLE/다단 집중 nextest9단계 모두exit0으로
종료했다. 집중 nextest는42개 전부PASS(1465개 선택 제외), 컴파일 포함537.378초이며
실제 검사 실행2.570초다. systemd MainPID0/inactive/ExecMainStatus0과 소스 불변도 확인했다.
이는 최종 전체 nextest나 fresh WASM 시각 gate의 완료 증거가 아니다. source SHA·원본/Print SHA·실제 명령과
source 불변은 ignored output의 `column-ole-native-v276-result.json`, `column-layout-source-v273/`,
`column-ole-{original-geometry-v278,boundaries-v279}/`, `column-ole-native-eight-pairs-v285.json`,
`column-ole-validation-v281-result.json`에 보존했다. 새 정식 검사/fixture/golden/기대값 변경은
시각 보정과 최종 선행 조건을 완료할 때까지 보류한다. 개별 PR의 보류 판정도 유지한다.


### 진행148: 중첩 표의 흐름 제한과 부모 상자 소유 — 구현·검증 중

진행147은4ceebf852543522e927db7b67230f8bc22f24662로 커밋했다. fetch 뒤에도
upstream/devel은bfa28a77a2714482f647ee6a7b9aa7f7876d8404로 같아 rebase하지 않았다(v288).

원본 ClickHere wrapper17a174162fd3fcd1c51bd04a25a577396c44bd51ad1c8ab001f9184b16dc4de3을
한컴2020에서HWP로 저장하고 다시HWPX로 저장했다(v287). 표 크기·여백·앵커 속성은
원본과 같으며, 저장 과정의 본문 LineSeg 위치 갱신과 안내 문자열 실체화만 따로 기록했다.
재생성본을 원본의 일치 증거로 대체하지 않는다. 같은HWP의 Print도 원본HWPX Print와
세 표의 외곽선 및 앞/뒤 텍스트 좌표가 같다.

원본의 복사본에서 자식 오프셋·부모 안여백·flowWithText·TAC·부모 높이를 각각 한 종류씩
바꾸고 실제 한컴2020 Print6건을 산출했다(v289/v290). 저장본과 모든 Print의 해시·job·
pdf_print_method0/hancom2020_pdf_driver_one_up을 기록했다. 최초 실행은venv에lxml이
없어 시작 전에 실패했고 실제 설치된system Python으로 고쳤다. 제품 실패로 세지 않는다.
부모 안여백이 커져도 flowWithText=false/nonTAC/Para Left·Top 자식은 부모 외곽 원점을
유지했다. 부모 높이를 크게 하고 CENTER 정렬을 유지해도 자식은 위 원점을 유지했다.
자식 오프셋만 바꾸면 자식만 그 오프셋으로 이동하고, 부모 테두리는 원래 높이를 보존하며
뒤 본문은 자식의 넘친 끝을 반영했다. flowWithText=true 또는TAC로 바꾸면 부모 여백과
자식 바깥여백 안쪽에 놓이고 부모 높이도 늘었다.

기존1×1 펼침은 모든 자식을 부모 안여백+자식 바깥여백 안쪽으로 놓고 부모 테두리도
그 합계로 늘렸다. 새table_wrapper 공통 프레임은 정상 저장 빈 앵커 줄과 셀 선언 높이로
부모 상자를 계산하며, 제한 없는 자식의 원점·오프셋·점유 끝을 독립적으로 반환한다.
measure_table_impl의 펼침 선택→typeset::table::format/fit_measured_for_host의 예약→
layout_table_with_wrapper_margin의 확정 원점/부모 테두리/뒤 흐름이 같은 판정을 소비한다.
TAC·흐름 제한·다른 앵커·쪽 분할·저장 줄이 없는 편집 입력은 기존 경로다. 문서ID·상수
화면 좌표·테두리 숨김·clamp·baseline 변경은 없다. 이 경계 밖은 미검증이며 범위를 확대해
수용하지 않는다. 현재 Native 빌드와 원본/독립 대조군 실행을 준비했고 결과는 종료 후 기록한다.
새 회귀/fixture/golden/기대값 갱신은 전체 Native/fresh90%와 직접 판독 전까지 보류한다.
증적은ignored output의wrapper-normal-save-v287/와wrapper-anchor-counters-v289/ 및
result.json, Print는pdf/semanticist21-20261008/wrapper-anchor-counters-v289/에 보존했다.

진행148의v292 fmt28.614초·Native build130.446초는exit0/source불변으로 종료했다.
동결Native와 원본19종의v294 감사도 전체 종료했고 base 대비 쪽수·본문 초과·지면 밖·
텍스트 겹침의 증가0을 확인했다. v293의 원본+독립 대조6쌍도7건 전부 실행했다.
원본HWPX30.10657%, 자식offset18.83610%, 부모padding23.34119%, child-flow48.91420%,
child-inline40.15141%, parent-height21.80119%로 아직 미달이다. 원본의 기존35.95134%를
이번30.10657%로 정정하며, 관계 일부 개선을 전체 fidelity 개선으로 주장하지 않는다.
한컴 정상HWP 저장본은Native100%지만 원본HWPX의 대체 증거가 아니다. 정상HWP의
수정 전 동결Native는v296에서exit1이고, 같은 저장본의수정 후100%와 구분해 기록한다.

최신wrapper-anchor-direct-v295/review/review_001.png를 직접 표시·판독했다. 부모/자식
가로 원점과17.1px 테두리는 겹쳐 독립 Print 관계로 복구됐으나2번째/3번째 표의 세로
원점과 뒤 본문이 약3.2/6.3px 위에 남는다. 원본 전체90%·fresh·최종 회귀·수용은 미완료다.
최초review 합성은 출력 하위 폴더 준비 누락으로 실패했고 폴더를 준비해 고쳤다.
도구 라벨의0-based 쪽 인덱스도 정정했다. 두 도구 오류를 제품 결함으로 세지 않는다.
기존 GDB 소스 줄 breakpoint는 실행을 잡지 못했고 정확한 함수 breakpoint로 현재 호출
y_start157.37333/221.37333/227.02667/299.98667/305.64를 확인했다. 최적화로 소실된
resolved_table_origin 값은 미관측으로 남기며 함수 호출 자체를 원점 공유 완료로 바꾸지 않는다.
v296 대조는 종료했다. 정상HWP의 수정 전Native38.99645%→수정 후100%,
정상HWPX의 수정 후Native도100%다. 두 정상 저장본의 통과를 원본HWPX30.10657%
통과로 바꾸지 않는다. 남은 차이를 원본과 정상 저장본의 앵커 재사용 경로에서 계속 분석한다.

원본의 복사본에서 루트 문단 LineSeg만 생략한 진단은Native에서2번째 표224.7px,
3번째 표292.0px로 돌아왔고3번째 앞 글줄266.9px도 독립 Print266.92588px와 대응했다.
자식 셀의 저장 줄·원본 내용·표 속성은 그대로다. 이는 저장 루트 앵커가 뒤 흐름에
다시 적용되는 경로를 원인 후보로 좁힌 실행 증거이며, 캐시를 지운 복사본의 관측을
원본 수정/시각 통과로 보고하지 않는다. 다음 단계는 원본의 객체 점유 끝과 다음 저장
줄의 수용 조건·편집 저장 캐시의 생성 출처를 같은 실제 경로에서 대조한다.
파일은wrapper-anchor-counters-v289/root-cache-omitted-diagnostic.{hwpx,json},
root-cache-omitted-native-nodes.json에 보존했다. 새frame 보정은 원본HWPX가 미달이라
아직 커밋하지 않았으며 원격 push/PR 생성/보류 해제도 하지 않았다.


### 진행149: 객체 점유를 담지 못하는 일정한 저장 사다리의 공통 수용 조건

v297의 실제 raw-top 호출은 표1/4/7에서anchor153.60/217.60/281.60px, 위 바깥여백
3.77333px를 소비했다. 저장 top 특례는None이다. RHWP_VPOS_DEBUG는 공통HeightCursor가
표 뒤의 순차끝45.97/109.97/173.97px를 저장 끝42.67/106.67/170.67px로 되돌림을
입증했다. 다음 문단도 저장 좌표의3.30667px를 재소비했다. paint만의 위치 문제가 아니며
조판의stored_paragraph_flow_ends도 같은 낡은 끝을 전파했다.

원본은 모든 루트 줄의높이/간격이 동일하고 걸음도 정확히 같은 사다리다. 글꼴은 그
걸음과 같지만 자리차지 표의 선언 높이+양쪽 바깥여백은 다음 저장 줄까지의 걸음보다
크다. 한컴 정상 저장본은 그 차이를 후속 vpos에 누적했고 원본과 같은 Print를 냈다.
기존uniform filler 판정이 글꼴의 모순만 세어 객체의 점유 모순은 놓쳤다.

공통stored_line_ladder 모듈로 기존 판정을 옮기고 동일 걸음의 다음 줄이 있는 빈
문단의Para Top/T&B/비TAC/비분할/무offset/무caption 표의 점유 띠도 독립 대조한다.
기존 최소 문단 수·동일 튜플/걸음 조건·모순 비율·공차는 변경하지 않는다. 다른 앵커는
이 점유 계약으로 판정하지 않는다. formatter는 구역의 판정을 결과에 포함하고
stored_successor_flow는 무효 사다리를 문단 끝 좌표로 다시 적용하지 않는다. 기존
HeightCursor와paragraph layout이 같은 구역 판정을 소비한다. 원본 입력/LS는 수정하지
않으며 새 좌표 clamp·숨김·baseline 변경은 없다. Native 원본/대조군과19종 geometry를
먼저 실행해 판단하고 아직 새 회귀나 수용 판정을 추가하지 않는다.


### 진행150: 펼친 자식 표의 측정·배치 깊이 정합

v301 Native 빌드는 fmt와 함께 통과했다. v302 원본/독립 대조7건은 종료했으며
원본100%, offset100%, padding100%, flow98.77153%, inline70.10563%,
parent-height100%, 정상HWP100%다. v303 원본19종 감사도 exit0이며 기준 대비
쪽수·본문 초과·지면 밖·텍스트 겹침 증가0이다. 원본100%를 전체 통과로 확장하지 않는다.
v304의 최신 원본·inline review_001.png를 직접 표시·판독했다. 원본의 세 표와 뒤
글줄은 Print와 대응하지만 inline 대조의 세 번째 표와 앞 글줄은 약3.8px 위에 남았다.

v305 실제 호출 진단에서 inline 자식의 측정 행13.3px·예약 프레임24.6px와 배치 행17.1px가
달랐고 flow 자식은17.1px·28.4px로 맞았다. HeightMeasurer의 투명1×1 펼침은
자식을 depth+1로 측정했지만 layout_table_with_wrapper_margin의 같은 펼침은 depth를
보존했다. 측정의 relaxed_pad_mirror가 가짜 중첩 때문에 안여백을 제외한 원인이다.
펼침 측정도 실제 배치와 같은 깊이를 전달한다. 실제 셀 안의 일반 중첩은 기존 깊이를
유지한다. 새 좌표 보정·여백 상수·기대값 변경은 없다. 새 Native7쌍/원본19종으로
반례와 영향을 확인하며 최종 lint·fresh WASM·회귀·수용은 아직 미완료다.

같은 v301 Native 라이브러리를 사용하는 ignored Rust 진단에서 실제 공개 편집 API로
원본과 동일한 표/누름틀을 생성했다. 내보낸 HWPX는 원본과 바이트가 동일하고
SHA17a174162fd3fcd1c51bd04a25a577396c44bd51ad1c8ab001f9184b16dc4de3이다.
편집 상태의 Print SVG/논리 글자 경로도 보존했으나 직접 시각 비교는 별도로 수행한다.
정식 회귀 추가·fixture/golden 변경과 원격 조치는 아직 하지 않았다. 증적은ignored
wrapper-inline-flow-diagnostic-v305/와wrapper-ladder-direct-v304/에 보존한다.


v306 fmt·Native build은exit0/source불변으로 종료했다. v307 원본/독립 대조7쌍은
전부exit0이며 원본·offset·padding·parent-height·정상HWP100%, flow/inline98.77153%다.
같은 inline 원본/Print에서70.10563%→98.77153%로 개선됐다. v308 원본19종도
exit0/source불변, 기준 대비 쪽수·본문 초과·지면 밖·겹침 증가0이다. v312에서
새 원본/inline compare·overlay·review를 만들고 빌드/source/PNG 해시를 연결했다.
수정 전v301의 실제 공개 편집 API Print도 독립 원본Print와100%이고 동일HWPX를
내보냈다(v305); 수정 후API 경로는 집중 회귀와fresh 실제 실행에서 별도로 확인한다.
v310의 필수lint/정책/집중 회귀와 그 성공을 조건으로 한v311 fresh WASM을 실행했다.
아직 종료하지 않은 검증을 통과로 기록하지 않으며 새 회귀·수용·PR 생성도 보류한다.

별도v309 진단에서 missing-named-style 편집본은 원본62문단의34번 본문이
current741.4px+fit12.6px인데가용749.6px여서 먼저 이월됐다. 실제 각주영역34.1px를
차감하고도Native 저장 각주 경로의 추가 안전 여백40px가 남았다(DIAG_AVAIL).
다음 저장reset 앞 빈35–37문단은 새 쪽에 놓이고38번reset으로 세 번째 쪽이 생긴다.
독립Print는34번location.replace 사용예를 첫 쪽에, navigator를두 번째 쪽에 둔다.
추정 여백과 저장/편집 각주 경계의 소유를 더 대조하며 이 진단만으로 안전 여백을
제거하거나 쪽수 기대값을 변경하지 않는다. 처음시도한systemPython에는PyMuPDF가
없어 진단이 시작 전 실패했고 설치된venv Python으로 실행했다.


v312의 같은 좌표3배 확대 판독은flow/inline 자식의 오른쪽 끝이 부모 외곽을
넘는 차이를 검출했다. 전체98.77153%가 이 차이를 면제하지 않는다. v314 독립PDF
콘텐츠 스트림에서flow/inline은 자식 괘선을 부모 셀 영역의W*로 자르고, 제한 없는
offset/padding은 용지 영역W*로 그리는 것을 확인했다. 부모의 물리 셀 소유가
펼침 과정에서clip으로 전달되지 않은 별도 결함이다. 높이 보정의 개선은 유지하되
시각 완료·새 회귀·수용 판정은 보류한다. v311은 검증 대기Python만 실행하던 상태에서
중지했고WASM/Cargo 빌드는 시작하지 않았다. v310 실행 결과는 현재 깊이 보정
소스에만 귀속한다. 종료 결과를 보존한 뒤 펼친 셀의 물리clip을 공통render tree로
보존해 모든backend가 같은 영역을 소비하도록 보정하고fresh를 새로 실행한다.

v313 missing-named-style의 실제 한컴2020 정상HWP 저장과HWPX 재저장은 모두
성공했다. 두 저장본의 v306 Native 쪽수는2쪽이다. 원본3쪽의 대체 통과로 보고하지
않으며 원본/정상 저장본의 각주·본문 저장 줄 차이는 saved-metadata-comparison.json에
연결했다. 최초 메타데이터 JSON 기록에서lxml 속성 객체를dict로 바꾸지 않아 실패했고
정정해 기록했다. 제품 회귀로 세지 않는다. 원본은 변경하지 않았다.


### 진행151: 부모 셀 표시 영역과 각주 캐시 폭의 실제 소비 경계

v310의9단계는 모두exit0/source불변으로 종료했고 기존46개는 전부PASS다.
prepare/fmt·세Clippy·workspace build·고정base manifest/unit 정책을 포함한다.
컴파일 포함 집중368.611초, 실제1.371초이며 전체 회귀나 fresh 완료가 아니다.

v314의독립PDF W*와 확대 판독을 근거로, 온전한 투명1×1부모의 흐름 참여 자식이
배치된 노드들만 부모의 확정 셀 영역으로 감싼다. 기존TableCell.clip은공통paint builder가
ClipKind::TableCell로 바꿔SVG/Canvas/Skia가 같은 물리 영역을 소비한다. 부모 테두리·
점유 끝·자식 원점·이미 명시한 논리 셀 주소를 변경하지 않는다. 제한 없는 자식은
영역 밖 표시를 유지하고 쪽 조각은 기존 컷 경계를 사용한다. 좌표 보정이나 축소로
출력을 맞추지 않으며 actual parent cell owner가 사라진 원인을 보정한다.

각주의추가40px를 광역 제거하지 않는다. 진행115의정책연구215쪽 왕복 보호는 정확한
기존 계약이므로 유지한다. 원본의새 각주는segment_width39684HU로 제본여백을
빼기 전 폭을 저장했다. 실제각주영역과 한컴 정상저장본은37420HU다. 정책연구558개
저장각주줄은45352HU로가용45354HU 안이다. 실제각주영역폭을저장줄 수용조건에
전달해 양의 캐시 폭·끝이 그 영역 안인지 확인한다. 모순 폭의 줄은기존 실제
FootnoteArea 예약과저장빈줄 소유 경로를소비한다. 유효한 원본의저장컷을 유지한다.
본문 p34/넘친 빈 p36–37/뒤 navigator와 각주 소유를 새 출력에서 직접 대조한다.
새 회귀·fixture/golden/기대값은 아직 추가·변경하지 않았다. v315 Native로 두 보정을
한꺼번에 빌드하고 wrapper7쌍·편집8쌍·원본19종·정책연구 원본/왕복을 확인한다.


v315 fmt43.421초·Native build147.243초는exit0/source불변으로 종료했다.
v316–318의 의존 대기가 종료된 이전v306서비스를 참조해 실행 전 실패했다.
실제MainPID0을 확인하고 원 기록을 보존한 뒤v320–322로 정정했다. 제품 실패로
세지 않으며 Native도 다시 빌드하지 않았다. v321 원본19종은 전부exit0이고 기준
대비 쪽수·본문 초과·지면 밖·겹침 증가0이다. v320의7쌍은 모두100%이며 오른쪽
클리핑 차이도v325의새review와 확대 판독에 연결했다. v322의missing-named-style
원본은3→2쪽,95.47353%/92.07644%다. p34사용예는1쪽, navigator는2쪽으로
복구했다. 원본은 변경하지 않았다. v319정책연구 원본/왕복은215/215쪽이고,
독립Print9쪽99.54423%,10쪽99.97364%로 두 컨테이너의TSV가 같다.

v325에서wrapper/inline, missing-style전2쪽, policy9/10의새review를 만들고
실제로 표시했다. 최초Python은 스크립트 디렉터리에서repo를import하지 못해
실행 전 실패했고 검색 경로를 정정했다. policy10은 이미지 색 차이가 남으며
직접review의전체silhouette68.75%와 구조를 쓰는TSV99.97364%를 구분한다.
215쪽의소유·쪽수무회귀를 해당문서전체의시각통과로 바꾸지 않는다. 기존policy의
정확한 의미검사를 보존하며 이번clip/각주예약 보정을 전체PR수용으로 바꾸지
않는다. v324에서 전체lint/기존wrapper46검사와policy/spec왕복을 실행 중이다.
성공 후같은 소스의fresh WASM을v323에서 빌드하도록 연결했다. 새 회귀·fixture/
golden과개별PR판정은 시각·최종조건 완료전까지 보류한다.


### 진행152: 최신 집중 회귀 종료와 차트 저장 정보의 독립 출력 대조

v324는 실제 systemd MainPID0/inactive/ExecMainStatus0으로 종료했다. prepare·fmt
check·Native/WASM lib/workspace all-targets 세 Clippy·workspace build·고정 base
manifest/unit 정책·집중 nextest의9단계 모두 exit0이며 소스 해시도 현재 파일과 같다.
집중 summary는57개 실행/57 PASS/1875 선택 제외, 실제 실행7.269초다. 이전 계획의
46개라는 수로 결과를 보고하지 않는다. 정책연구215쪽 왕복과 명세서 원본의 기존
의미 검사를 포함한다. 전체 nextest의 실패0 또는 최종 시각 수용을 의미하지 않는다.
성공을 확인한 v323이 실제 fresh WASM 컴파일을 시작했고 v326/v327은 그 결과를 기다린다.

v328은 같은 edited HWP-endnote 원본의 길이 접두부/CFB Contents를 읽고 기존 bounded
진단 probe의 메타데이터와 독립 한컴 Print의 실제 벡터·글꼴·범례를 연결했다. 입력/Print/
Contents/probe 해시와 명령은 `chart-column-primary-analysis-v328/result.json`에 보존한다.
수동 값 축은0–2000/4분할, 계열색은#73b1d1/#cd6565/#ebc452이며 실제 Print와 같다.
Print의 차트 글꼴은GulimChe9.96pt로 저장10pt와 대응한다. 제목 캐시 높이341을 글꼴
크기로 대신 쓰지 않는다. 실제 범례는 첫 줄 적립금·지출/둘째 줄 수입이며, Native
라이브러리가 읽는 최대 캐시 label900/marker341/고정 gutter50 및 section margin으로
두 열·두 줄이 예상된다. 좌표 단위와 다른 chart path는 추가 입증 전 추론으로 남긴다.
현재 원점의 Print/Native 차이는3.710661px이며 원인과 무관한 상수 보정을 추가하지 않는다.
기존 type1은 production presentation reader가 수용하지 않아 generic renderer로 빠지는
실제 경로를 확인했다. 배경·제목·축·범례의 저장/재배치 계약을 함께 분석해 배치 보정을
준비하며 이 진단을 production parser 검증이나 최종 완료로 바꾸지 않는다.

첫 진단은venv의olefile 누락/CFB 길이 접두부/Debug 출력의후행 쉼표 처리로 실패했고,
설치된 시스템 모듈 사용·입력 프레이밍 검증·진단 parser를 정정한 뒤 종료했다. 제품 실패
또는 원본 변경은 아니다. 생산 코드는 v323/v326/v327의 소스 불변을 위해 유지한다.
새 formal 회귀·fixture·golden·기대값·개별 PR 보류 판정은 변경하지 않았다.


v329의최신Native와독립Print에서 같은 원 좌표를3배 확대한 review를 실제 표시했다.
inline 오른쪽 선은 부모 경계에서 끝나며, missing-style의location.replace는1쪽,
navigator는2쪽에 보존된다. 선의농도와약1px괘선 차이·본문의글꼴/간격차이는 남아
있다.3배 이미지의2px관용값은원96dpi전쪽2px값과같은 gate가 아니므로 확대의점수를
전쪽 통과의근거로바꾸지 않는다. 원PNG해시·영역·배율은
`wrapper-cell-note-zoom-v329/result.json`에 고정했다.

v330의bounded class trace와설치된HncChartBase.so의실제Section/Location reader·
layout switch를 대조했다. 차트 좌표는Cartesian이므로 높은y의제목과낮은y의범례를
PDF의위→아래좌표와같게해석하면상하소속을뒤집는다. 실제native type1은높은y에서
margin/height를빼고가로중앙, type6은낮은y에서margin을더하고가로중앙을택한다.
공개COM location번호와동일하다고추정하지 않는다. 기존독립진단v35는Title 저장
상자높이/외부·내부margin만바꿔도Print 좌표가같음을입증했으므로, 새renderer에
저장title box나margin을그대로재사용하는가정도채택하지 않는다. 유효legend 캐시와
title의word-wrap변경후재측정을구분하는계약을계속추적한다. 코드보정은아직없다.

v332는마지막전체29FAIL의검사이름을최신57PASS로그와대조했다. 같은이름7개가
최신집중검사에서PASS이며나머지22개는이번선택에서실행하지않았다. 22개를현재
실패수로보고하거나7개의같은이름만으로기대값동일/전체회귀해소를주장하지않는다.
macro로생성한baseline partition은source의fn검색부재로제거됐다고판정하지않는다.
현재원본/독립Print의의미와실제최종전체실행을기준으로후속작업한다.
`historical-failure-current-audit-v332.json`에원래suite/name·현재source해시·실행포함을
연결했다. fresh v323은release컴파일10분40초를종료하고wasm-opt가실행중이며
v326/v327의검증완료로보고하지않는다. PR 생성/push/merge도계속보류한다.


### 진행153: fresh WASM 완료와 기존 회귀 검사의 실제 범위 정정

v323은exit0/source불변으로1607.265초에종료했다. repo root의locked web wrapper로
release컴파일10분40초와wasm-opt를실행했으며 `--no-opt`는사용하지않았다.
동결package는 `wasm-pkg-wrapper-cell-note-v323/`이고 JS SHA-256은
691e4fd480cf4e04f9e599ca7c7a4802ad87b2c141fba93bf94111f4144dcb3b, WASM은
f16609ba3bac35ab0292a0c15c2bc36d8becb562834f98090fb1831cc2d517f9다.
pkg/public 두파일해시는일치한다. Studio 브라우저조작을이해시확인으로대신하지않는다.
v327의실제공개API는4.640초/exit0/source불변이다. 불투명/투명/분할wrapper의
논리셀주소와누름틀활성화·해제를대조했고 다시생성한입력도보존원본과바이트동일한
17a174162fd3fcd1c51bd04a25a577396c44bd51ad1c8ab001f9184b16dc4de3다.
완료된API를독립Print/Studio/final visual통과로확대하지않는다. v326의브라우저
fresh14쌍전쪽비교는실행중이며처음blank note2쌍은각100%로종료했다.

v333에서#7739의현재OPEN본문을API로다시읽어#7445대표이슈·원본/Print SHA·
기존전체원문이관을확인했다. 원본398d03a5d5e4d6e857086be532d6d9ed0cec9c8ad06f95c17bbb7f83056ae860와
Print f8e5c0408e221080ede9a9a67b153d02d792d22961c738e46749641f32a32e79를유지한다.
고정base bfa28의전쪽미달근거는v258/v268/v269와같고10미달쪽이기존current와PNG동일하다.
현재v315의19종geometry감사에서도이원본103쪽의기준대비오류증가는없다.
`issue_6761_side_by_side_objects_do_not_stack::horizontally_overlapping_objects_still_stack`은
특정패널의소유를주장하면서전체103쪽을page_bottom1122.5px/공차1px로검사했다.
직접보정3문서에포함되지않고이미이관된미달전체범위이므로이검사1개와전용
max_off_canvas helper만제거한다. 같은모듈의나란한그림/동일물리원점Right정렬2검사와
다른영향쪽·정상대조군·입력·Print·baseline공차는보존했다. 검증후승인으로기록하지않고
`chemical-whole-scope-transfer-v333/result.json`에근거와정확한변경을고정했다.

v334는실제v323 Cargo작업의종료를기다린뒤공유target에서prepare·fmt·세Clippy·
workspace build·고정base정책과기존57+보존2검사를순차재검증한다. 소스1111개와
tests의별도해시를고정하며완료전PASS로기록하지않는다.

v335는#6972의같은높이줄대조군의기존기대값을정밀검토했다. HWP5 확장그림슬롯은
0–7유닛,보이는글자첫offset과두번째줄시작은8이다. 한컴사양표6/62와이입력의
원시축은그림의첫줄소유를입증한다. 기존None기대는글자의둘째줄소속을그림에대입한
오류다. 현재공통raw-slot조회가반환한Some((0,0))을구현회귀로세지않으며, 기존순수
소유helper검사의수정안을준비했다. Render/실물입력/fixture/golden을소비하는검사가
아니며시각개선의증거도아니다. v334의test해시보호를위해아직실제tests에는적용하지
않았다. 독립사양해시·생성조건·실제소비경로·기대·수정안은
`raw-slot-regression-review-v335/`에연결했다. 전체29실패해소·직접보정3문서최종전쪽·
backend·개별PR수용·원격PR생성의완료로바꾸지않는다.


### 진행154: Native/fresh 전체17쪽과 보존·정정한 회귀61개 완료

v326은실제MainPID0/inactive/ExecMainStatus0으로14쌍전부exit0/source불변종료했다.
모든17쪽의최저91.23437%,90%미달/누락0이며현재Native와fresh의decoded RGB가
전부같다(v338). original wrapper와독립대조6개는양쪽각100%다. Missing-named-style
편집원본의같은2쪽은양쪽95.47353/92.07644%다. 실제fresh의대표review4개를v339로
새로만들어원PNG해시와연결했고inline은실제표시·직접판독했다. 원본입력은보존한다.
이점수로남은차트의축/범례/색/배경·3문서최종전체·Skia/Studio·전체회귀를완료로
바꾸지않는다.

v334의9단계는exit0이며실제집중summary는47PASS/2089선택제외/8.075초다.
prepare로모듈의suite번호가달라져고정suite선택에서이전57중12개가빠졌다.
완료된47개를59개나전체회귀PASS로보고하지않으며누락을v337에서정정했다.
v337은prepare후실제module소속을찾아suite를선택하고이전57+보존2+raw-slot2의
기대61개이름과실제PASS이름을비교한다. 현재9단계는모두exit0/source/test불변,
집중61개전부PASS/2303선택제외/5.227초며기대대상누락0이다. 실제MainPID0/inactive/
ExecMainStatus0도확인했다. #6972의원시슬롯기대값정정도실제tests에적용했으며
시각fixture나생산behavior는바꾸지않았다. 전체nextest의마지막29FAIL을61PASS로
대체하지않으며초기18개PR의보류를유지한다.

v336은현재v315소스의화학제품원본79쪽을고정base와같은입력/Print/font로재출력했다.
둘다17.91785%이고decoded RGB도완전히같다. PNG컨테이너해시는Chrome버전/인코딩
차이로다르므로바이트SHA동일이라고보고하지않는다. 현재103/기준103쪽으로누락/
쪽수차이는없고,exit1은의도한미달의re_review_required gate다. v333제거의기존미달
근거를최신소스에서도확인했으며새구현회귀를숨기는제거가아니다. 원본·Print와두
영향영역검사는보존했다. `chemical-whole-scope-transfer-v333/current-negative-check.json`에
근거를연결한다.

필요한독립대조6입력을SHA파일명으로
`mydocs/pr/assets/semanticist21-20261008/wrapper-anchor-counters-v289/`에바이트그대로
보존하고기존untracked6Print PDF와연결했다. 생성방식/한컴버전/Print계약/입력·PDF·
Native·WASM SHA/원본과대조군구분은같은README에기록한다. qualified fresh review4개는
`../assets/semanticist21-20261008/wrapper-cell-note-fresh-v339/`에보존한다. TSV/실행로그/
중간JSON/SVG는ignored output에남기며Git에추가하지않는다. 새조판정식회귀는
현재qualified 원본/대조군의관계형검사와수정전후검출을갖추기위해다음단계에서작성한다.
PR제출·원격push·merge·새6PR체리픽은아직하지않았다.


### 진행155: 최종 인쇄 관계 검사4개와 실제 수정 전후 검증

검증 head는 `643812fab0a2c6f82f1a74b49bc52b19f6f88e2b`, pinned base는
`bfa28a77a2714482f647ee6a7b9aa7f7876d8404`다. production1111파일은 Native v315와
fresh v323/v326/v338의 해시 목록과 동일하다. 이번 변경은 검사 원본·지원 helper·누락 입력 보존이며
렌더 구현이나 기준값을 변경하지 않았다. 신규 관계 검사의 원본/독립 Print는 앞 단계의 전체
Native/fresh17쪽 중7개 래퍼 입력각100%, 같은 missing-named-style 입력2쪽
95.47353/92.07644%로 먼저 검증했다. v329의본문 끝/다음 쪽 확대를 다시 직접 표시·판독했고,
글꼴 raster·가는 선의 진하기 잔차를 전체 일치나 최종 수용으로 보고하지 않는다.

`tests/cases/wrapper_cell_print_relations.rs`는 실제 `export-svg --profile print --compat 2022`를
소비한다. 글자 `뒤`가 각각 한 번만 남는 두 자식 셀의 클립과 실제 부모 괘선을 찾아,
flow/TAC 자식의 저장 폭은 축소하지 않고 최종 표시 끝은 부모 괘선 안에 있는지 검사한다.
클립ID·문서의 절대px 위치를 고정하지 않는다. 흐름 제한이 없는 원본/offset/padding/height/
정상한컴HWP 대조군은 자기 표시 영역을 유지해야 한다. 기존 누름틀 전체 셀 주소·활성화·분할
검사4개도 그대로 실행했다. `footnote_stored_width_body_ownership.rs`는 동일 편집 저장본의
첫 쪽 `location.replace("URL")`, 둘째 쪽 `navigator`와 마지막screen속성 목록의 소유·순서·
누락/중복 및독립Print전체2쪽을 확인한다. 쪽 소속 검사를 글꼴/차트/모든backend의 증거로 확장하지 않는다.

원인 소비 경로는 진행151의 `TableWrapperVerticalFrame → render_transparent_borders`와
`wrapper_cell_viewport::retain → TableCell clip → PageLayerTree → SVG clip-path`,
`footnote_area_has_stored_lines(actual area_width) → 무효폭 재조판/예약 → 본문 쪽 소유`를 잇는다.
새 검사들은 마지막 실제 출력에서 위 계약을 확인한다. source모듈4개와그정상대조를 이번4검사로
모두 입증했다고 주장하지 않으며, 저장사다리·부모높이 세부계약의 기존 직접Print대조와 진단은
해당 기록으로 구분한다.

v343의9단계 prepare/fmt-check/Native·WASM·workspace-all-targets Clippy/workspace build/
pinned manifest·unit policy/집중nextest는 모두exit0이다. 실제65PASS/2772skip/7.593초,
컴파일 포함174.644초이며 기대실행65개와실제실행65개를대조해누락0을확인했다.
초기 `#[path]`외부모듈 구성은 integration target50>48로 prepare에서 실패했다.
동일 지원 파일을 inline module에서 포함하는 기존 suite 패턴으로 바로잡아28suite+20exception,
48/48예산을 유지했고 새 Cargo target·정책완화·registry 예외를 추가하지 않았다.
파생 suite는커밋하지않는다. 실제 source/test해시와모든명령은 ignored
`wrapper-note-formal-validation-v343-result.json`에보존했다.

v344는 동일 새 검사 실행 파일에 런타임 `CARGO_BIN_EXE_rhwp`를 지정해 실제 CLI를 대조했다.
수정전 v306 Native SHA `059c7aa34e2018311cfb9558a250afdf62e6917f9439447bf511f3c0caa77e84`,
수정후 v315 SHA `3d50e1311eb579afd41d0dcf3bafade916b8b763a633083c8f89d2b001340ae3`다.
inline/flow는 수정전 부모right676.54667 밖으로자식 표시right687.12가남아 관계 검사exit101,
각주 원본은 같은Print2쪽대신3쪽이라exit101이다. 정상 unrestricted대조는수정전exit0,
수정후4개모두exit0을 실제확인했다. 좌표는 실패 원인 진단값이며 검사 기대값은 아니다.
`wrapper-note-formal-before-after-v344-result.json`의실행파일/CLI해시·명령·각로그를보존했다.

검증에 쓴 missing-named-style 입력은 이미Git에있다는이전가정과달리 output에만있었다.
같은10443바이트/SHA `6b4f6a375029dcb0c9def5b4443683d1b55cf09acbdb9cb9ab87ac2d94888edb`를
변경없이 `mydocs/pr/assets/semanticist21-20261008/current-print-inputs-v175/`에 보완했다.
같은내용의tracked사본은없고대응독립Print는기존pdf경로에있다. README에생성·출처·해시를연결했다.
새 입력·검사가 시험지/HML 제외원본을 사용하지 않는지확인했다.
이번65개를전체회귀0FAIL로바꾸지않는다. 마지막전체29FAIL은역사결과이며현재전체를다시실행해
실제남은실패를확인한다. 차트내부·KPS/가맹사업/CGMP최종전체·backend·개별18PR수용·원격PR는계속보류다.


### 진행156: 현재 전체 회귀 실행과 실제 Studio 두 backend 확인

검증 source head는 `940ea1322617ccff5a7b47d6920ae96e94cfffad`, 고정 base는
`bfa28a77a2714482f647ee6a7b9aa7f7876d8404`다. v345 전체 nextest와 v347 KPS77쪽 /
CGMP48쪽 / 가맹사업142쪽의 Native·fresh WASM 전쪽 비교는 해당 systemd MainPID를
확인했으며 계속 실행 중이다. 실행 중 소스 및 검사 원본은 변경하지 않았다.
부분 로그에는 쪽 배경·각주 예약·CGMP 종료 행·KPS37쪽·TAC 캡션 그림·이관 API의
잔존 쪽수 목록·차트 단 이동 검사 실패가 있다. 최종 실패 수는 종료 뒤 확정한다.

v349/v350은 실제 Studio Canvas2D/CanvasKit에서 새 탭을 열고 초기 JS/WASM 응답을
기록했다. 같은 입력의 논리 누름틀2개와 깊이2의 소유 경로, 실제 마우스 클릭 후 뒤
누름틀1개 유지, 페이지 오류0을 확인했다. 각각36.366초 / 46.813초·exit0이며 소스
해시가 그대로다. v350의 요청·실제 backend는 CanvasKit이고 fallbackReason은 null이다.
실제 화면도 직접 열어 확인했다. 검증 전용 서버는 각각 정상 종료했다.

첫 v348은 클릭 동작을 완료했으나 Vite 변환 JS를 원본 바이트/소스맵만으로 식별하려던
진단 assertion이 실패했다. v349에서 초기 응답 원문·소스맵을 먼저 보존하고 소스맵
제거 및 정확한 WASM URL 치환 하나만 복원하여 원본 JS와 일치함을 확인했다.
초기 WASM 응답 SHA는 `f16609ba3bac35ab0292a0c15c2bc36d8becb562834f98090fb1831cc2d517f9`다.
식별 오류를 제품 클릭 실패로 보고하지 않으며, 다른 JS 차이를 허용하지 않았다.
[실제 화면과 입력·빌드 식별 기록](../assets/semanticist21-20261008/wrapper-studio-v349-v350/README.md)을
연결한다. 상세 실행 기록은 ignored `studio-wrapper-field-pointer-v349-result.json` 및
`studio-wrapper-field-pointer-v350-result.json`에 있다.

v351은 동일 원본의 현재 분할 JSON과 독립 Print를 다시 읽었다. KPS37쪽은 성능
요구사항으로 끝나고38쪽은 데이터 요구사항으로 시작하며 현재 행 경계도 이에 맞는다.
CGMP34쪽은 원자재 입고시 행을 완전히 소비하고35쪽은 다음 행으로 이어진다.
기존 CGMP 검사는 이미 완전히 수용한 행에도 비어 있지 않은 종료 컷을 요구하고,
KPS 검사는 실제 Print 경계 대신 고정 endRow16을 요구했다. 전쪽 Native/fresh90%를
확인하기 전 검사 기대값을 바꾸지 않았다. 단순 쪽수·텍스트만으로 전체 시각 통과를
선언하지 않는다. v351의 JSON·Print 경계 단어·PNG·명령 및 실제 exit0을 보존한다.

쪽 배경 합성 검사는 SHOW_ALL 기본 설정에도 뒤 쪽 이미지가 없어야 한다고 기대한다.
생산 경로는 구역 fill visibility를 색·이미지에 함께 적용하며 기존 SHOW_FIRST/SHOW_ALL
계약과 일치한다. 각주 합성 검사는 폭0인 LineSeg를 유효한 저장 줄로 가정한다.
양쪽 검사 입력·독립 계약을 바로잡을 후보를 준비하며 현재 실행 소스는 유지한다.
이관한 API 원본은 issue7445 자산에 보존되어 있으나 활성 render_page_samples.tsv에
옛 samples 경로가 남아 있어 해당 목록의 정밀 정리가 필요하다. 원본을 삭제하지 않는다.
TAC 그림과 차트는 추가 독립 출력 대조 및 원인 분석이 남아 있다.

v346은 제목 여백 대조군이 실제 소유 section을 변경했음을 확인했다. v352는 원래 차트
Contents의 backdrop prefix를 끝까지 읽고 소유 위치를 기록했다. 해당 진단 reader의
최종 지원 조건은 type9이므로 type1의 supported=false를 제품 parser 성공으로 보고하지
않는다. 본 단계에서 차트 생산 코드는 변경하지 않았다.

실제 Studio 두 backend의 해당 누름틀 통과를 Native Skia·다른 입력의 clip/그림/폰트·
최종 차트·직접3문서 전쪽·전체 회귀 완료로 확대하지 않는다. 개별18PR 보류 판정은 유지한다.


진행156 추가 확인: v356에서 Canvas2D의 요청·실제 선택 backend 및 fallbackReason=null을
직접 읽은 뒤 같은 클릭을 재실행했다(9.595초·exit0). 초기 JS/WASM 식별과 소스 해시도
유지된다. v349의 URL 요청만으로 실제 선택 신원을 대신하지 않았다.

v355/v357은 #6575 축소 입력과 실물 HWP의 base/현재 출력을 대조했다. 축소 입력의
TAC 그림과 소유 TextLine은 base에서도 같은 y444.7, 현재에서도 같은 y458.3이다.
현재 그림만 기준선으로 내려간 현상은 아니며 앞 내용의 흐름 변화가 함께 반영됐다.
원본 HWP 첫 그림은 base/current y234.98667로 같고 둘째는544.53333→532.28이다.
이 관측을 독립 Print 일치로 바꾸지 않는다. 같은 축소 입력의 Print와 원본 시각 범위를
확인하기 전 검사를 수정하지 않았다. v354의 잘못된 embed-fonts none 인자는 CLI usage
exit2이며 v355에서 옵션을 생략해 네 실제 export가 모두 exit0이었다.


### 진행157: TAC 축소 입력의 독립 Print 확보와 동일 원문 비교 시작

v358/v359에서 기존 `samples/issue6575/tac_picture_line_top.hwpx`를 수정 없이 한컴2020과
2024에서 PrintMethod0의1-up PDF로 출력했다. 두 job은 실제 succeeded이고 download도
exit0이다. 입력 SHA `ae4c01001405de8deb2332cab5f469896a3b5ca12b18c1c28a48ff9719cadf85`,
두 결과 모두1쪽이며 세 그림 상자가 동일하다. 각 job·version·Print 경로·PDF SHA와
[보존한 실제 PDF](../../../pdf/semanticist21-20261008/tac-caption-independent-print-v358/README.md)를
연결한다. 이 자료는 실패 원인의 비교용이며 새 회귀 fixture 또는 정상 baseline이 아니다.

Print의 마지막 그림 top476.479pt는 기존 검사의 y444.68px 및 현재 y458.32px와도 다르다.
그림과 소유 글줄의 상대 정렬만으로 앞 내용의 흐름을 정상이라고 판정하지 않는다.
v360은 동일 입력·한컴2020 Print를 고정하고 base bfa28a / 현재 Native / 현재 fresh WASM의
전체1쪽을 순차 비교하고 있다. 실제 Print 부족을 예외로 두고 검사 기대값을 바꾸지 않았다.
원본 HWP8쪽의 첫 그림 위치가 같다는 관측을 이 축소 입력의 전체 일치로 대신하지 않는다.

실행 source는 계속940ea132이며 이후 두 문서/화면 증적 커밋은 Rust/Studio 소스를 바꾸지 않았다.
전체 회귀 v345 및 직접3문서 전쪽 v347은 같은 live MainPID로 계속 실행 중이다. 필수 검사와
개별18PR 판정은 미완료 상태를 유지한다. 신규 원격 이슈·PR·review·push는 수행하지 않았다.


### 진행158: 전체 회귀 실패 분리·이관 잔여 목록 보정·실제 Native Skia

생산/test 소스는 `940ea1322617ccff5a7b47d6920ae96e94cfffad`, 검증 시작 head는 `bb93ab7f1`,
고정 base는 `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`다.
v345 전체 nextest는 terminal/exit100이다. 실행 10,266개 중 10,254 PASS·12 FAIL이고 45개는 제외됐다.
테스트 실행은 2691.894초, 컴파일 포함 3597.38초이며 생산/test 소스 해시는 변하지 않았다.
반복 summary를 중복 제거한 실제 실패는 아래 12개다. 집중 65 PASS로 전체 실패를 대신하지 않는다.

| 실제 실패 | 현재 처리 근거 / 상태 |
| --- | --- |
| page_bg_image_only_on_section_first_page | SHOW_ALL인데 이미지가 첫 쪽에만 있다고 기대한 합성 계약이다. SHOW_ALL/SHOW_FIRST의 이미지·색상 대조 수정안을 준비했으며 아직 적용하지 않았다. |
| footnote_area_reserve_uses_section_shape_metrics | 저장 LH는 있지만 폭이 0인 합성 줄이다. 유효 저장 줄과 재조판 계약을 분리해야 한다. |
| issue_6035_complete_row_does_not_emit_a_line_advance_only_tail | CGMP의 완결 행 컷과 내용 보존을 같은 Print와 대조한다. CGMP는 직접 보정 범위다. |
| kps_ai_page37_defers_overflowing_split_row_slice | KPS 37/38쪽의 실제 행 소유를 확인했다. fresh 전쪽 90% 확인 후 관계형 검사로 보정한다. |
| issue_6575_tac_picture_sits_on_line_top_when_line_is_taller | 축소 원본의 Print 1쪽은 base 25.31052%, 현재 Native/fresh 27.01645%다(v360). 출력 해결·검사 변경·이관으로 판정하지 않는다. |
| every_document_path_exists_in_the_repository | 이미 이관한 API의 쪽수 목록 잔여 1행이다. 해당 행만 제거하고 6계약 PASS를 확인했다(v371). |
| chart_moves_to_second_column_and_text_does_not_overlap | 차트의 실제 원점과 내부 글꼴·축·범례 보정이 남는다. 고정 y 기대값만 완화하지 않는다. |
| oracle_page_count partition15 | 실제 실패 입력은 k-water HWPX의 27→28쪽 증가다. 다른 k-water HWP의 이관 기록으로 제외하지 않는다. |
| issue_5833_stacked_cell_pics_flow_without_overlap | 같은 축소 입력의 독립 Print 1쪽을 확보했다. 고정 y 범위·셀 소유·빈 그림의 기준 출력을 대조한다. |
| oracle_page_count partition13 | 실제 실패 입력은 issue157 HWP의 2→3쪽 증가다. 같은 원문의 독립 Print 2쪽을 확보했다. giant의 slow 출력을 실패로 세지 않는다. |
| continuation_fragment_receives_the_remaining_paragraphs | KPS의 市場 글자는 Native baseline 217.5px와 Print top 205.55px로 양쪽 모두 기존 y≤200 범위 밖이다. 실제 셀 소유·내용 보존으로 보정할 근거를 확보했고 fresh 전쪽은 대기다. |
| issue_2291_rowspan_declared_residual_expands_last_spanned_row | 이관된 교육과정의 정확한 table 244×10/pi8/r183c8를 추적했다. 현재 183행이 두 쪽의 조각에 걸린다. 고정 첫 조각 높이와 전체 행 계약을 구별해 이관 적합성을 확인한다. companion 재래핑 검사는 PASS다. |

v371은 이미 #7445/#7731로 보존한 `hwpctl_API_v2.4.hwp`를 가리키는 쪽수 목록 1행만 제거했다.
2026-10-12에 #7731 OPEN 본문에서 같은 원본명과 #7445 연결을 읽기 전용으로 재확인했다.
기존 compiled regression_suite_014의 같은 fixture 계약 6개를 수정 전후 실행해
5 PASS/1 FAIL → 6 PASS를 확인했다. 입력 파일·다른 쪽수 값·공차·정확한 semantic/parser 검사는
바꾸지 않았다. 고정 base의 manifest 정책 검사도 exit0이다. 최종 전체 재실행은 아직 남는다.

v364의 실제 `native-skia` 빌드는 590.681초/exit0이다. 바이너리 SHA는
`9120fe0a56d99b2742c4a229888195ee3c927d7142749c6c2a014d08b1cbf85f`이고 생산 1111개 소스는 현재와 같다.
v369에서 실제 Native PNG 11문서 14쪽을 내보냈고 v372에서 같은 원본·한컴 Print의 전체 14쪽 TSV를
산출했다. 최저 91.21234%, 미달·누락 0이다. 래퍼 7문서는 모두 100%다.
대표 review 4개를 열고 원본·흘림·글자취급의 오른쪽 경계와 앞/뒤 글자를 3배 확대했다.
자식 폭·부모 clip·앞뒤 래퍼 순서·글자 기준선이 대응한다. Print stroke 0.36pt와 원본 0.12mm가
대응하며 raster AA 농도 차이는 남는다. 각주 뒤 본문 쪽 소속은 대응하지만 Latin 획·자간 차이는
남아 최종 수용으로 면제하지 않는다.
[대표 review·overlay·확대 PNG와 출처](../assets/semanticist21-20261008/wrapper-native-skia-v369-v372/README.md)를 보존한다.
진단 import 등록 오류와 최초 overlay caption의 0-based 인자 오류를 고쳤고 정상 종료·정확한
쪽 라벨로 다시 판독했다. 도구 오류는 제품 회귀로 세지 않으며 새 source-side 검사를 추가하지 않았다.

v366/v367에서 같은 issue157/5833 원본을 바꾸지 않고 실제 한컴2020 Print로 출력했다.
두 job은 succeeded/download exit0이다.
[PDF 2개·입력/PDF 해시·version·Print 출처](../../../pdf/semanticist21-20261008/remaining-failures-independent-print-v366/README.md)를 커밋해 보존한다.
k-water HWPX는 서버의 macro/script 자동 처리 거절로 job이 생기지 않았다. 원본의 스크립트를 실행하거나
삭제하지 않았다. 기존 대응 독립 Print의 출처를 별도로 확인하며 이 거절을 제품 출력 실패로 세지 않는다.

v370은 issue157의 base/current 흐름을 실제 진단했다. pi2~24의 누적 흐름은 같다.
새로 수용하는 빈 표 frame이 현재 누적 772.52px를 쓰는 반면 유효 저장 원점은 737.32px다.
현재 표 상단 856.1067px는 Print 615.047pt(820.063px)보다 아래이며, 저장 원점+offset+outer-top의
820.9067px가 독립 Print에 대응한다. 같은 검증된 저장 단의 원점을 fit/paint/점유 하단에 공유하는
후보 patch를 준비했으며 production에는 아직 적용하지 않았다. v373의 GDB 변수 변경은 실제 반환 frame에
전달되지 않아 3쪽 그대로다. 수정 후 통과 증거가 아니며 후보의 입증도 미검증이다.

v374는 교육과정의 정확한 원본을 base/current에서 dump-pages했다. default DocumentCore 경로의 전체
418/422쪽을 관측했으며 독립 Print 415쪽과 다르다. v375에서 같은 s5pi8/r183c8의 소유자를 대조했다.
base 139쪽의 완전한 행은 56.3px, 현재 139쪽의 앞 조각은 41.9px, 140쪽의 물리 꼬리는 14.5px다.
합계 56.4px이며 문장은 앞 조각에 보존되고 꼬리에 중복 텍스트가 없다. 첫 조각만 55~58px라고
요구하는 검사의 가정을 다시 검토한다. 제거에 앞서 같은 고정 base·원본·독립 Print의 해당 두 쪽을
Native/현재 fresh로 비교하는 v376을 시작했다. 전쪽 415쪽의 통과를 주장하지 않는다.

생산 소스를 바꾸지 않아 v347의 KPS 77/CGMP 48/가맹 142쪽 전쪽 검증을 이어간다.
KPS Native 77쪽은 최저 90.91935%, 미달·누락 0으로 종료했고 fresh WASM은 진행 중이다.
CGMP/가맹의 이번 전쪽 비교는 아직 시작 전이다. 코드 변경 후 최종 head의 영향 시각·전체 회귀·lint는
다시 확인한다. 신규 원격 issue/comment/PR/review/push는 없고 개별 18PR 보류를 유지한다.

ignored output의 whole-regression-audit-v345, direct-three-current-whole-v347,
api-transferred-inventory-correction-v371, wrapper-native-skia-png-v369,
wrapper-native-skia-compare-v372, remaining-failures-independent-print-v366,
issue157-flow-advance-base-current-v370, issue157-certified-origin-diagnostic-v373,
education-source-row-owner-v374, education-row-fragments-v375, education-exact-row-visual-v376에
명령·로그·관측을 연결한다.


### 진행159: 디스크 부족 복구와 선택 쪽 WASM 내보내기의 중복 저장 수정

교육과정139/140쪽만 요청한 v376의 WASM 경로가 Native 글꼴 정책 SVG를 전쪽 생성하고,
최종 WASM SVG도422쪽 복사해 정책 디렉터리만 약22GB를 사용했다. 디스크 부족으로
v347/v376 서비스가 실제 실패 종료했고 집계 JSON 두 개가 잘렸다. 완료 표식이나 빈 진단 스크립트의
exit0을 검증 성공으로 간주하지 않는다. 특히 디스크 부족으로 작성되지 않은 v382의 빈 스크립트는
KPS 경계 검사 증거가 아니다.

수정 경로는 `export_wasm_target`의 전체 raw SVG/render tree·pageCount 확인 → 문서 전체의
글꼴 별칭 CSS 수집 → 정책용 중간 Native SVG 정리 → 요청 쪽의 최종 SVG/트리 복사다.
다른 쪽에 저장된 글꼴도 공급하며 WASM 소유 텍스트·좌표·전체 쪽수를 바꾸지 않는다.
`render_target`은 완료 표식 외에 요청한 SVG와 트리의 존재를 확인해 다른 부분 내보내기를
잘못 이어받지 않는다. 중간 SVG의 경로·크기·CSS SHA는 작은 JSON으로 남긴다.
측정 지표·90% 기준·문서 쪽수 gate는 유지한다. 같은 결과가 필요한 전체 내보내기는 여전히
전쪽을 복사한다. 새 도구 계약은 선택 쪽만 저장되는지, 비선택 쪽의 글꼴이 공급되는지,
비선택 raw/트리 누락도 실패하는지, 전체 쪽수·이어받기·범위 밖 요청을 검사한다.

`venv/bin/python -m unittest scripts.tests.test_visual_sweep
scripts.tests.test_visual_sweep_generation scripts.tests.test_visual_sweep_document_pages
scripts.tests.test_visual_sweep_wasm_pages`는100검사/24.866초/exit0이다.
실물 재실행으로 저장량 감소를 확인하기 전이므로 이 결과만으로 실물 검증 완료를 주장하지 않는다.

`visual-recovery-v384/result.json`에서 잘린 집계의 원래 바이트·SHA를 보존하고1111개 생산 소스와
Native 바이너리 해시를 확인했다. 각 완료 `silhouette_manifest.json`의 전체 PNG 입력 해시를
다시 검사한 결과는 다음과 같다. 잘린 집계의 process exit code를 새로 만들어 채우지 않았다.

| 재확인한 완료 출력 | 비교 쪽 | 최저 일치율 | 전체 쪽수/남은 판단 |
| --- | ---: | ---: | --- |
| KPS Native | 77 | 90.91935% | 77/77, 미달·누락0. 경계 직접 판독·기존 검사 보정 남음 |
| KPS fresh WASM | 77 | 90.91935% | 77/77, 미달·누락0. 직접 판독 남음 |
| CGMP Native | 48 | 94.05301% | 48/48, 미달·누락0. fresh 재개 필요 |
| 교육과정 base Native139/140 | 2 | 75.29239% | base418/Print415, 이관된 고정 높이 검사의 적합성 증거 |
| 교육과정 현재 Native139/140 | 2 | 74.39013% | 현재422/Print415, 현재 fresh 확인 남음. 전쪽 통과 아님 |

종료된 작업의 중간 SVG와 더 이상 필요한 검증에 사용하지 않는 초기 frozen 바이너리만
정확한 경로·프로세스 미사용 확인 후 제거했다. 원본·Print·PNG·TSV·트리·현재 Native/fresh·공유
`target/pr-review`를 보존했고 현재31GB 공간을 확보했다. 삭제 경로·크기·해시는
`completed-education-native-svg-cleanup-v383.json`, `obsolete-frozen-binary-cleanup-v383.json`,
`terminal-incomplete-svg-cleanup-v383.json`에 남겼다. 원격 GitHub 변경은 없다.


### 진행160: KPS 전쪽 RGB 대응·경계 확대 판독과 실제 상단 괘선 제거 추적

v386 retry1은 전쪽77개의 Native/fresh PNG를 decoded RGB로 비교해 모두 동일함을 확인했다.
37/38/65/66쪽 review·overlay와 셀 소유 bbox에 대응하는 판독용3배확대를 생성하고 직접 읽었다.
37쪽은 성능 요구사항으로 끝나고38쪽은 데이터 요구사항부터 시작한다. 65쪽 라벨은 앞부분을,
66쪽은 `시장침해/가능성`을 소유한다. 기존 dump의 행16 기대 및 y≤200 선택은 실제 한컴의
소유 경계와 맞지 않는다. 다만66쪽의 바깥/라벨 프레임 상단이 한컴 Print와 다르게 빠진다.
전쪽90% 이상이라는 이유로 이 배치·괘선 차이를 완료로 면제하지 않는다.
[review·overlay·3배확대와source/입력/PDF/PNG해시](../assets/semanticist21-20261008/kps-qualified-boundaries-v386/README.md)를 보존한다.

v387은 현재Native의실제66쪽`export-svg --profile print --compat2022 -p65`를 읽기 전용GDB로 실행했다.
원본의 바깥674/0 표r2c0와table-level Fill4는4방향Solid/검정/width1이다.
`layout_partial_table_resolved`에서 collect_cell_borders로 수집한 상단6칸은Solid/검정이다.
6293/6304줄에서is_continuation=true,start_cut=[20],header_rows=[],Square를 확인했고,
6332줄에서는상단6칸이모두None이다. 같은조건의새물리조각을일괄열어두는분기가정상프레임을지운다.
이 source-level producer→h_edges→이어받기분기→render_edge_borders→최종SVG 연결은 실제 호출 증거다.
열린상단이정답인#6756/#6549와프레임소유를구분할 독립 근거·반례를묶어보정하며 조건을더붙여
한문서에만맞추지않는다. 생산코드와기존KPS검사는아직수정하지않았다.

선택쪽도구수정커밋후fetch는exit0이고upstream/devel은bfa28a77로동일해rebase하지않았다.
`post-selected-wasm-commit-fetch-v384.json`에기록했다. v385는교육과정현재fresh139/140→CGMPfresh48→
가맹Native142/fresh142순서로실제서비스에서재개했다. 완료한기존세전쪽출력은다시실행하지않는다.
Atomic JSON으로집계하고 각완료출력의SVG만정리하며 실패·쪽수차이·90%미달을그대로기록한다.
교육과정선택2쪽도Native문서전체글꼴정책이필요하므로임시정책생성단계의디스크량을감시한다.
과거생성된큰SVG를정확한경로와processcmdline/open-fd미사용확인후추가정리하는v388을실행했다.
현재v385산출물·원본·PDF·PNG·TSV·트리·frozen바이너리·공유target은정리대상에넣지않았다.

최초v386의진단compare폴더누락을고쳐retry1에서exit0과정확한4쪽review/확대산출물을확인했다.
이진단오류는제품회귀로세지않는다. 공식판정은지표와직접판독을별도로기록하고현재상단결함을보류한다.
신규원격변경·PR/push/승인/통합은없다.


### 진행161: 원래 체리픽 문서 우선과 교육과정의 부적합 고정 높이 검사 정리

2026-10-12 사용자 지시로 초기18개PR에서 언급한 문서와 편집 동작의 검증을 먼저 마무리한다.
PR별 검토 문서를 실제 원 PR 범위·현재 검증 증거·남은 항목에 맞춰 정리하고 승인용 PR을 준비한다.
원래 범위 밖의 입력 진단을 추가로 확장하지 않으며, 현재 실행 중인 승인된 전쪽 비교의 결과는 보존한다.
전체 회귀·lint의 필수 gate와 알려진 실제 회귀를 통과한 것으로 처리하지 않는다.

교육과정의 s5/pi8 244×10 표 r183c8는 고정 base bfa28a77에서139/140쪽이
77.52827/75.29239%이며418/Print415쪽이다. 현재 Native/fresh는74.39013/76.06998%이고
422/Print415쪽이다. 같은 원본 SHA·독립 Print·96dpi/compat2022/글꼴 조건으로 비교했다.
두 쪽 Native/fresh의 decoded RGB가 동일하며 base/current의 네 review를 직접 판독했다.
행 내용과 쪽 소속이 base부터 미달인 기존 입력이며 현재의 추가 차이도 그대로 남긴다.
전쪽 개선 완료·현재 회귀 면제·새 정상 baseline으로 판정하지 않는다.

`issue_2291_rowspan_declared_residual_expands_last_spanned_row`는 표의 소유를 확인하지 않고
첫 r183c8 조각에 전체 행 높이55–58px를 요구한다. 실제 현재 조각은41.9px와 글자 없는
잔여14.5px로 나뉜다. 이미 열린#7445/#7743으로 이 부적합 함수와 전용 helper2개만 이관했다.
[원문/base·원본/Print·네 review·정확한 판정과 복원 조건](../assets/issue7445/education-curriculum-map/README.md)을 보존한다.
기존 재래핑 내용 검사·다른 정확한 공유 검사·원본·Print·baseline·허용치는 유지했다.

v394의 prepare/fmt/fmt-check/Native Clippy/WASM-lib Clippy/workspace build/
workspace-all-targets Clippy/고정base manifest/unit-tier 정책9단계는 모두exit0이다.
첫 집중 실행은 prepare가 소스를suite026에서025로 재배치한 뒤 이전 target을 선택해
0건/exit4였으므로 PASS가 아니다. 실제 생성 모듈 소속을 다시 찾아 retry1에서
`issue_2291_stored_single_seg_paragraph_rewraps_full_text` 1건을 실행해1PASS/6.406초/exit0을 확인했다.
생산1111개 소스와 수정한 검사 소스는 검증 중 불변이다. 관련 JSON과 로그는 ignored
`education-exact-height-migration-v394-result.json`, `education-exact-height-migration-v394-retry1-result.json`에 남긴다.
이 집중 결과를 최종 전체 회귀로 대신하지 않는다.

v385의 교육과정 선택2쪽과 CGMP fresh48쪽은 실제 terminal 결과를 확보했고 다음 가맹사업
Native/fresh142쪽 비교가 진행 중이다. 미달·쪽수 차이와 직접 판독은 별도로 기록한다.
Visual Sweep의 범례/일치율 설명은 포함돼 있어 사용자가 해당 수정 요청을 취소했고 도구 변경은 없다.
fetch 결과upstream/devel은bfa28a77로동일하므로 rebase하지 않았다. 새 원격 게시/push/PR/승인/통합은 없다.


### 진행162: 초기 체리픽 문서 우선·48쌍 전쪽 비교 완료와 #7657 실제 저장 원점 결함

사용자 지시에 따라 원래18개PR의 기존 편집 입력과 원본을 먼저 확인한다.
v396은 기존48개입력/독립Print52쪽을 현재Native v315/fresh v323으로 전쪽 재출력했다.
Native/fresh 각각exit0/48문서52쪽/최저99.62848%이며90%미달·누락·쪽수차이0이다.
v402에서52쪽의decoded RGB가전부동일함도확인했다. 실제현행소스의재출력이며 이전PNG의재사용이아니다.
다만 이일치율·RGB동일성만으로확대판독이나 초기18개전체수용을완료로바꾸지않는다.
기존초기9축/파일명맥락20/논리캐럿·번호12/무늬6종1/바깥여백2/미주경계4의48쌍이다.
7개논리캐럿입력은현재v175저장본과기존v94입력의바이트동일성을확인했다.

현재원격18개를읽기전용으로조회해#7612는MERGED,나머지17개는OPEN/비draft임을확인했다.
#7615후속head변경은조회만하고추가체리픽하지않았다. 원PR문서의계약78개고유함수는
v345전체로그의실제PASS행과현재검사소스의940ea132바이트동일성으로연결했다.
17개개별review의실행대기표현과현재생산소스/원격스냅샷을정정했고#7612기존병합기록은보존했다.
로그추출·해시는ignored `original-pr-contract-readiness-v396.json`이며이78PASS를전체
v345의10254PASS/12FAIL또는최종통합성공으로확대하지않는다.

#7657의독립편집Print가부족해기존정식3개검사와같은순서로저장본6개를생성했다.
캐시의같은production library를standalone rustc로읽었고Cargo target은변경하지않았다.
정식실물SO-SUEOP/143E/덧말입력과빈문서셀을구분하며덧말의수동스타일참조설정도공개했다.
6개한컴2020 job이모두성공했고runtime11.0.0.9136/PrintMethod0/1-up/input_preprocess none을
확인했다. Print는합계53쪽이며현재Native가53쪽전체완료됐다.
빈셀HWP/HWPX1쪽은모두41.80704%;143E1쪽91.24177%;덧말각2쪽94.02610/96.62673%다.
SO-SUEOP46쪽은1/43쪽미달,최저44.20171%다. 현재fresh v398은진행중이다.

첫셀review를실제표시·판독했고상단/너비는맞지만높이25.1대41.1px와뒤문단16px차이를발견했다.
동일생성HWP의독립한컴HWPX저장대조군은LineSeg vertpos1200,원생성본은0이다.
줄height1600/spacing960/폭40932/flags393216·셀높이선언282·안여백141은같다.
한컴의문단ID/문단모양재번호부여도기록해바이트동일입력으로보고하지않는다.
그대조군의현재Native는100%이며Cell높이41.1/내부빈TextLine y175.3/뒤문단y202.2다.
원생성본은25.1/y159.3/y186.2로같은16px차이가난다.
`apply_cell_style_native`→cell reflow→기존vpos보존→저장LineSeg→저장앞간격포함을전제로한
측정/배치의실제연결을확인했다. renderer guard를완화하지않고서식변경의저장원점생산을보정한다.
[원생성본6/독립Print6·독립한컴저장대조군/직접review/상세경로](../assets/semanticist21-20261008/pr7657-cell-style-before-correction-v399/README.md)를남긴다.
대조군100%를원생성본41.80704%의해결로바꾸지않는다. SO-SUEOP미달쪽도직접판독/보정이남는다.

승인된v385기존전쪽비교도실제종료결과를확인했다. CGMPfresh48쪽은최저94.05301%·미달/누락0·48/48·exit0이다.
가맹Native142쪽은19쪽86.41272%·쪽수142/142·누락0·exit1이며후속fresh는시작하지않았다.
집계에실제실패조건을보존하고원본이나허용치를바꾸지않는다. 현재우선순위는원래체리픽문서의보정이다.
교육과정이관커밋75bd77e09후fetch는exit0/base bfa28a77로동일해rebase하지않았다.
새GitHub원격게시/push/PR/승인/통합은없으며필수최종gate완료후승인요청을진행한다.


### 진행163: SO-SUEOP 1쪽 제목 보정·관계형 회귀와 초기 체리픽 검증 우선

v406은 셀 서식의 저장 첫 줄 원점·미주 번호의 원시 슬롯 글자 모양·WMF OLE 두 축을
일괄 보정했다. fmt/fmt-check/Native build와 fresh WASM wrapper는 실제exit0이고
생산1112개 소스 해시 불변·pkg/Studio public 해시 일치다. 최종 lint/전체 회귀를 대신하지 않는다.

SO-SUEOP 원본 HWPX1쪽은 Native/fresh 모두96.97415%,46/46이며 해당쪽RGB도같다.
제목의높이·폭·원점을 실제review와동일좌표2배확대로직접읽었다.
HWP3 Native1쪽은84.44393%로미달이며학교명/교사명/학년용의폭·원점차이가남는다.
원본과형식별독립Print의SHA는#7807원문과같다. 다른형식PDF로점수를맞추지않았다.
양형식제목진단의같은소스는수정전2FAIL/exit101,수정후2PASS/exit0이며컴파일은모두성공했다.

사용자는#7807전체해결을요청한뒤최신지시로1쪽보정확인→회귀생성→현재체리픽중심PR검증을
우선했다. 정식`tests/cases/so_sueop_hwpx_first_page_ole_title_regression.rs`는
시각선행조건을충족한HWPX1쪽제목만검사하며실물px핀없이독립usvg최종투영영역과
원본OLE소유영역을대조한다. 미달HWP3나전체쪽수회귀를새로고정하지않는다.
동일정식소스의수정전FAIL/수정후PASS및파생integration실행은v413기록으로확인한다.
나머지#7807미달은해결완료/이슈종료로바꾸지않고원래18개PR의검증을우선한다.
진행중v409전쪽Native진단은종료결과를보존하며새전체fresh진단은추가시작하지않는다.

첫셀원점갱신만으로출력은해결되지않았다. 한컴저장대조군은원점외표Common높이도
1282→3082HU로바뀌어초기100%대조군의원인분리가부족했다. 초기README를정정했다.
v406의첫줄y175.3에도셀높이25.1/뒤문단186.2px가남아독립Print41.1px에못미친다.
8개반례중unchanged/둘째문단/복구3개는100%,나머지5개는41.08527–49.78778%이며
원래셀HWP/HWPX는각41.80704%다. 저장앞간격의줄합산/표예약/실제배치/뒤흐름소비를
같이보정해야하며선언높이강제확대·기대값완화로이차이를없애지않는다.

[부분보정·반례18개입력·전후Print16개·실제1쪽review·회귀진단·프로그램추적](../assets/semanticist21-20261008/pr7657-batched-correction-v406/README.md)을보존한다.
새untracked PDF16개는유효한독립Print증거여서입력SHA/job과함께커밋대상으로정했다.
임시JSON/TSV/SVG/로그와상용글꼴바이트는커밋하지않는다. 개별#7657review의보류도유지한다.
전체회귀v345의12FAIL·직접보정KPS/가맹실패·초기18개최종시각gate는아직해소되지않았으며
새GitHub게시/push/PR/승인/통합/종료는없다.


추가 지시: HWPX만 포함하면 안 되고 HWP 원본도1쪽 회귀에 포함해야 한다.
HWPX-only 등록은 중간 상태다. HWP3 1쪽84.44393%의 실제 출력을 먼저 보정하고
Native/fresh90% 이상·직접 판독 뒤 같은1쪽 제목 관계 검사를 정식 추가해 양형식을 마무리한다.
HWPX 정식 최종 포맷 소스의 독립 before/after 실행은 컴파일0/0, 실행101/0으로
의도한1FAIL→1PASS를 확인했다. 최초 focused는 fmt 실패로 테스트0건이며 PASS로 세지 않는다.
포맷 수정 뒤 같은 의미의 source로 두 파생suite에서새1개·원래#7657의3개를 실행중이다.
HWP3 원시 줄의sx 대응은 진단 관측이다. 공식HWP3표12는 그6바이트를 예약 영역으로
정의하므로 정상 대조·편집후 유효성 근거 없이 좌표권위를 확대하지 않는다.

## 진행164 — v406 보정 단계의 실제 종료 확인

v413 retry1 nextest4/4 통과: 원래 #7657 의미 검사3개와 HWPX OLE 첫쪽 관계 검사1개.
v409 전체 Native의 HWP3 최저42.64803%(34쪽 미달), HWPX 최저85.59465%(43쪽만 미달),
양형식46/46쪽으로 누락·쪽수 불일치 없음. HWP3 첫쪽 전진폭 SVG-only 진단97.44400%는
생산 코드 및 fresh 검증 전이다. HWP/HWPX 양형식 첫쪽 회귀 완료를 계속 진행하며
현재 단계의 source·입력·독립 Print·확인 결과를 일반 commit으로 보존한다. PR 최종 준비는 미완료다.

## 진행165 — 양형식 첫쪽 정식 검사와 가맹사업 결측 글리프 공급

현재 head는 `cba5fc76ef363cec7e1253d9e2062a7f98a7876f`이며,
base는 `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`다. 추가 보정은 dirty 상태이며
검증 생산 소스는 v423의 파일별 해시로 별도 고정한다. 원격 PR 생성·push는 아직 하지 않았다.

HWP의 첫쪽 정식 회귀를 `tests/cases/so_sueop_hwp_first_page_regression.rs`에 추가했다.
원본 HWP의 1쪽만 검사하는 OLE 소유영역·독립 HFT 전진·학년 숫자 전진의 세 검사다.
기존 HWPX 첫쪽 검사는 유지하며 파생 suite는 prepare로 등록했다.
v430은 이 세 검사·HWPX 한 검사·원래 #7657 세 의미 검사, 합계 일곱 검사를
실제 실행해7/7 통과·exit0을 확인했다. 고정 base manifest 정책도 exit0이다.
파생 suite·manifest는 커밋하지 않는다.

원 휴먼명조를 다른 face의 대체 파일보다 먼저 공급하는 v423에서 Native 첫쪽은
HWP99.79294%·HWPX99.95935%다. 성공한 fresh WASM 빌드를 재사용한 비교도
각각 같은 점수이며 두 정상 대조군을 포함한 네 입력 모두 Native/fresh RGB가 같다.
양형식 제목·학년·교사명·학교명의 같은 좌표 확대 판독과 현재 review를 보존했다.
전체 #7807의 해결이나 최종 PR 수용으로 확대하지 않는다.

가맹사업 19쪽의 기존86.41272%는 원 휴먼명조에 없는 괄호의 공급·전진폭 문제였다.
독립 Print와 실제 cmap/hmtx에 따라 기존 FontEnvironment의 glyphFallbacks를 선언하니
동일 Native 생산물에서99.98153%가 됐다. 새 review를 직접 읽었다.
[공급 계약·독립 글꼴 증거](../assets/semanticist21-20261008/franchise-glyph-fallback-v428/README.md)를 보존한다.
v429는 후속 셀·OLE 소스 보정을 모아 적용하기 위해 중단했다. Native PNG 27쪽과
512개 글꼴 검사는 생성됐으나 전쪽 TSV·fresh WASM은 미완료다.
종료된 불필요한 SVG 133개를 제거해 약 12.39GiB를 확보했고 대표 9쪽과 기존 증거는 보존했다.
최종 보정 소스에서 공용 target과 루트 pkg로 전체142쪽 Native/fresh를 다시 검증한다.

셀 높이·KPS·CGMP·남은 전체 회귀 실패·최종 lint와 초기18개 전체 시각 gate는 아직 미완료다.
각 개별 PR의 보류와 승인 후 원격 PR 생성 원칙을 유지한다.

v434는 정식 HWP 첫쪽의 정확히 동일한 세 검사 소스를 기존 공용 v406/v423 library에
직접 링크했다. 컴파일0/0, 이전1PASS/2FAIL, 이후3PASS이며 새 Cargo/WASM 빌드는 없다.
학교명·숫자 전진 두 계약의 결함 검출을 확인했고 OLE는 v406에서 이미 보정돼 이전에도 PASS다.

v432의 현재 소스 재실행은12검사 중2PASS/10FAIL·exit100이다.
문서 경로 검사와 원 face 조회 검사는 PASS이며, 나머지 실제 실패와 로그를 보존했다.
과거 CGMP rowspan 이름은 현재에 없으므로 해당 필터를 실행 성공으로 세지 않는다.
복구된 현재 CGMP 의미 검사는 v433에서 별도1/1 PASS·exit0이다.
이는 선택 검사 진단이며 현재 전체 회귀의 성공/실패 건수로 확대하지 않는다.

## 진행166 — 사용자 제공 OLE 분리 입력과 공통 프레임 보정

사용자가 OLE 차트만 복사해 저장한 다음 세 파일을 현재 보정 입력으로 사용한다.
기존 단 배치 원본의 검증을 대신하지 않으며 수정 후 두 범위를 다시 비교한다.

| 파일 | SHA-256 |
| --- | --- |
| `samples/hwpx/143E433F503322BD33-ole.hwp` | `9f7975a341f9bf0be89e3d6dc1385b59de5e61e079eeb0048db0090c932ad14c` |
| `samples/hwpx/143E433F503322BD33-ole.hwpx` | `83815ff764b8ab40e6e232e57bc20b6dcb6f0b2c824f7673148fe63d965e1255` |
| `pdf/143E433F503322BD33-ole-2024.pdf` | `584902f845c4b0d88d4e827a8a7dde23bf3ee3d92bd843f4c1f925b7a7498b20` |

PDF는 Hwp 2024 13.0.0.3901 / Hancom PDF 1.3.0.550의 1쪽이다.
독립 출력의 축은0–2000·4분할, 계열은 `#73b1d1`·`#cd6565`·`#ebc452`다.
기존 production reader는 혼합형(9)만 수용해 두 분리 입력의 막대형(1)을 거부했다.
저장 종류별 painter와 해당 축·색상을 연결했다. 독립 parser 진단은 이전 두 입력 모두
unsupported, 이후 두 입력 모두 독립 축·색상과 일치한다. 시각 수용 증거와는 구분한다.

OLE 일반 배치에서는 위 바깥여백1417 HWPUNIT가 누락됐다.
`column_ole_placement`의 `anchor_y → table_top → occupied_bottom`을
일반 `shape_layout` 및 본문 전진에서 소비하고, 이월 OLE도 원래 앵커를 전달해
offset·위 여백을 중복 가산하지 않는다. 임의 좌표 상수를 더하지 않았다.

공용 `target/pr-review` Native build는 exit0이다. 생산 소스는 ignored
`ole-isolated-v439/batch-native-plan.json`, 바이너리 해시는 build-result에 고정했다.
Native 방향 검증에서 OLE 원점은 y132.3→151.2로 이동했고, 독립 PDF는 약151.0이다.
전쪽(각1쪽) 일치율은 HWP/HWPX 모두48.44923%→51.99896%로 **미달**이다.
제목·범례 글꼴과 크기 및 플롯 내부 배치는 아직 기본 renderer 경로이며 보정 중이다.
새 실물 회귀·기대값 갱신·최종 fresh WASM·PR 수용은 보류한다.

셀 앞 간격은 흐름 간격과 저장 첫 줄 앞의 물리 띠를 공통 결과로 분리했다.
측정 세 경로·전체 줄 높이·첫 분할 유닛·범위별 높이가 같은 점유량을 소비한다.
부분 줄 범위는 첫 유닛이 선택될 때만 앞 띠를 포함하며, 중첩 표 offset은 소비한 몫을 뺀다.
10개 셀/편집 경계의 Native 방향 검증(v440)은 exit0으로 종료했다.
최저99.76088%·미달/누락0이며, 원래 셀 서식 삭제 HWP/HWPX 두 입력은 각각100%다.
대표 셀·서식 변경 대조군의 review/overlay를 새로 만들고 실제 셀 외곽·텍스트·뒤 내용을 대조했다.
이 수치는 **별도 셀 문서들의 결과**이며 OLE 차트51.99896%와 구분한다.
fresh WASM과 최종 소스 영향 범위 검증 전이므로 최종 수용은 미검증이다.

기존 source unit도 독립 계약으로 재검토했다. 쪽 배경은 첫 쪽 제한 플래그를 명시하고
뒤쪽의 흰 종이 및 제한 없는 배경을 확인한다. 폭0 합성 각주를 저장 LH로 고정하던 검사는
같은 두 각주의 구역 여백 차이(위1000·아래700·사이900 HWPUNIT)를 확인한다.
글꼴 조회 검사에서는 시스템/환경 글꼴을 섞지 않고 임시 두 루트만 비교한다.
unit-tier는 신규 cfg support helper를 제거한 뒤 고정 base 비교 exit0이며 manifest도 exit0이다.
이 검사들과 기존 OLE unit의 정식 focused nextest는36/36 PASS·3960 skipped·exit0으로 종료했다.
run ID는 `131d2bb8-f971-4dd8-906c-db0e55652b02`이며 production source hash guard는 변경 없음이다.
전체 회귀·lint 완료는 아니다.

동일 분리 HWP/HWPX의 독립 한컴2024 Print도 각각 완료했다.
job `0c72375e-1e5c-4adb-8e16-6b9baeb0196a` / `adc0d17b-9f9a-4ce3-be82-44ab526607ac`의
실행 증거는 build13.0.0.3901, PrintMethod0, `frame_print_to_pdf_ex_one_up`이다.
`pdf/semanticist21-20261008/ole-isolated-independent-print-v439/`에 두 출력 PDF를 보존한다.
두 독립 출력의 96dpi RGB는 동일하다. 사용자 PDF와는 일부 글자 원점 약1.2pt 차이가 있어
같은 바이트·래스터로 간주하지 않으며, 이후 보정에서 두 출력을 직접 대조한다.

## 진행167 — OLE 저장 글꼴·제목 원문과 실제 SVG 글꼴 공급

`hwp_ole_spec.md`는 컨테이너 구조의 역사 자료임을 확인하고 현행
`chart_ole_v1_boundary.md` 및 이미 확보한 한컴 차트 revision1.2와 연결했다.
기존 bounded reader에서 소유 객체의 `VtString → VtFont → VtTextBlock`을 보존한다.
축 첫 참조는 제목이며 둘째 참조의 레이블 배열이 실제 눈금/범주 글꼴을 소유한다.
단일 글꼴로 표현할 수 없는 다단 레이블은 하나의 face로 덮어쓰지 않는다.
제목/범례 표시 플래그와 제목 전각 공백을 보존하고 저장 포인트 크기는 layout DPI로 변환한다.
임의 문서 ID·위치 맞춤 상수·캐시 사각형을 글꼴 크기로 쓰지 않았다.
미검증 글꼴 스타일 플래그는 이번 plain-font 소비 범위로 확대하지 않는다.

`RawSvg → SVG text의 명시적 속성/그룹 상속 → font_codepoints 및 bold face 수집 →
기존 generate_font_style`로 실제 공급을 연결했다. 단순 family 속성 추가로 끝내지 않았다.
분리 입력 양형식과 원래 차트 Contents의 제목/범례/축은 모두 굴림체10pt로 추출된다.
기존 혼합형 `samples/issue6938/mixed_chart.hwp`/`.hwpx`도 type9·두 축과 두 계열을
계속 수용한다. 이는 parser 진단이며 혼합형 전체 시각 검증의 대체 증거가 아니다.

공용 target Native 묶음 빌드는 exit0/155.839초이며 바이너리는
`defefad25332a9d23d31d60c56c924218080edfb44ccb34039951b9034793aab`다.
생산 소스·명령은 ignored `ole-font-metadata-v442/native-build-plan.json`에 고정했다.
빌드 뒤 기존 조각 원문 보존 unit을 확장한 test-only 변경은 별도로 기록했다.
동일 source 모듈의 공유 rlib 링크 검사는19/19 PASS·exit0이며 정식 전체 nextest는 아니다.
처음 추가했던 source unit은 base19 상한에 걸려 기존 계약 검사 확장으로 수정했다.
unit-tier/base 및 manifest/base 최종 exit0이며 기준선 상향은 없다.

| 실제 Native 전쪽 비교 | 최저 실루엣 | 판정 |
| --- | ---: | --- |
| 분리 HWP / 사용자2024 PDF | 52.22510% | 미달 |
| 분리 HWPX / 사용자2024 PDF | 52.22510% | 미달 |
| 분리 HWP / 같은 원본의 독립2024 Print | 52.70762% | 미달 |
| 분리 HWPX / 같은 원본의 독립2024 Print | 52.70762% | 미달 |
| 원래 단 배치 HWP / 대응2020 Print | 91.72377% | 수치 통과, 차트 영역 결함 유지 |
| 원래 단 배치 HWPX / 대응 독립2020 Print | 91.96023% | 수치 통과, 차트 영역 결함 유지 |

모두1/1쪽이며 분리 입력은 실제 Print 세대에 따라 compat2024로 재검증했다.
v439 방향 검증은 compat2022였으므로 그 점수를 최종2024 검증으로 재사용하지 않는다.
원래 HWPX 기준은 job `530ba761-3985-4838-9c71-3a28494f86bf`의
2020/11.0.0.9136·PrintMethod0·one-up 출력이며 compat2022를 사용했다.
입력/기준/전체 source hash·명령·TSV·비교 PNG는 `ole-font-metadata-v442/`에 보존한다.
새 분리 review를 직접 확인했고 Native SVG에서 실제 GulimChe 프로그램
`790131a9f24edef28f95f90a10bacebab649b137c8096fb5b785922f0ca650fa`의 임베딩과 cmap을 확인했다.
제목/플롯 원점·범례 마커/간격·격자선·외곽선 차이는 남는다.
원래 문서의91%로 분리 차트52% 결함을 상쇄하지 않으며 머지 보류를 유지한다.
저장 section/여백·레이블 점유에 따른 공통 배치, fresh WASM 및 최종 전체 회귀/lint가 남는다.

### 공식 차트 사양의 저장 간격 대응 확인 — v443

[한컴 차트 파일 형식 revision1.2](https://cdn.hancom.com/link/docs/%ED%95%9C%EA%B8%80%EB%AC%B8%EC%84%9C%ED%8C%8C%EC%9D%BC%ED%98%95%EC%8B%9D_%EC%B0%A8%ED%8A%B8_revision1.2.pdf)의
Location·Plot.AutoLayout·BarGap/xGap·Pen/Tick 정의를 실제 owning object와 대조했다.
공식 PDF SHA는 `e014db3e4b55bc57d93b3aba0b186151b3487575e3a6397a2983715b43beeeb1`이다.
설치본의 ReadFrom/SetBarGap/SetXGap 읽기 전용 대조로 Plot prefix의8/12 byte 필드를 확인했다.
분리 HWP/HWPX와 원래 단 배치 Contents는 BarGap0/xGap1이며 혼합 차트 대조군은1.5/1이다.
앞서 prefix의 마지막 float를 간격 후보로 추정한 가정은 제거했다.
생산 reader에서 이 값을 읽고 column renderer의 계열/항목 간격 계산에 전달했다.
현재 reader/renderer를 기존 실제 parser IR과 연결한 진단 컴파일 및 양형식 출력은 exit0이다.
source/input SHA와 명령·진단 출력은 ignored `ole-gap-spec-mapping-v443/production-diagnostic-result.json`에 보존한다.
이 진단은 정식 Native/fresh WASM·독립 Print 비교를 대체하지 않는다.
v442 점수는 이 변경 전 결과이며, 일반적인 비영 간격의 배치와 나머지 플롯/범례/축 보정은 미검증이다.


## 진행168 — OLE 소유 배치·축 정렬·범례 간격 보정(v444/v445)

사용자가 제공한 `/home/tsjang/mydocs/tech/한글문서파일형식_차트_revision1.2.md`
(SHA256 `7c844d8846162b62b3363adcb10cf7efb6addff54fce9900bddb7452a4f95676`)의
Location, TextLayout, HorizontalAlignment/VerticalAlignment를 owning archive 객체와 대조했다.
`legacy_presentation`에서 root/Section Backdrop, 축 Pen/Grid/Tick, TextBlock의 점유/그리기 상자와
정렬, 계열 LegendLabel을 보존하고 `legacy_column_renderer`가 같은 root 변환으로 소비한다.
저장 캐시 좌표와 글꼴·펜의 물리 포인트 크기를 구분한다. 마커 packing 여유와 실제 글자 여백도 구분했다.
설치된 차트 라이브러리의 읽기 전용 필드/소비 위치 대조는 진단 근거이며 Native 라이브러리 호출이나
새 런타임 의존성은 추가하지 않는다. 특정 문서 ID나 절대 픽셀 보정은 사용하지 않는다.

v445 현재 Native 빌드는 exit0/104.160초이며 source guard는 unchanged다.
바이너리 SHA256은 `a5b0a186a71ae0db9074e77bee7d59690756183c8b197e53d4346417f5d4e3fd`다.
검증 명령·전체 Rust source hash와 입력/독립 Print hash는 ignored `ole-text-position-v445/`의
build/sweep plan, 각 run_manifest와 TSV에 보존했다. Native 두 묶음은 각각 exit0이다.

| 실제 Native 전쪽 비교(각1/1쪽) | v442 | v445 |
| --- | ---: | ---: |
| 분리 HWP / 사용자2024 PDF | 52.22510% | 98.94751% |
| 분리 HWPX / 사용자2024 PDF | 52.22510% | 98.94751% |
| 분리 HWP / 같은 원본의 독립2024 Print | 52.70762% | 99.02533% |
| 분리 HWPX / 같은 원본의 독립2024 Print | 52.70762% | 99.02533% |
| 원래 단 배치 HWP / 대응2020 Print | 91.72377% | 97.06291% |
| 원래 단 배치 HWPX / 대응 독립2020 Print | 91.96023% | 97.78619% |

전체 TSV에 미달/누락/쪽수 불일치는 없다. silhouette-only의 gate `not_evaluated`는 승인 판정이 아니다.
Native review/overlay와 동일 좌표3배 확대 `native-zoom-before-after-review.png`를 직접 확인했다.
플롯·막대 간격, gray frame, 저장 grid 표시, 숫자의 가운데 정렬, 범주 레이블 위치, 범례 간격은 개선됐다.
범례 기준선 및 한글 전진폭의 작은 차이는 별도 추적하며 수치 통과로 면제하지 않는다.
PDF와 공급 TTF 모두 GulimChe의 동일 hhea/advance를 확인했으므로 글꼴 누락 예외로 분류하지 않는다.
PDF 원문 TJ는 한글 -36.1/숫자 -18.1의 전진 보정을 포함한다. 사용자 PDF의 숫자 기준선은
독립 Print와1.2pt 다르지만 범례/제목 기준선은 같다. 원문을 `user-independent-text.json`,
`user-pdf-content.txt`, `independent-pdf-content.txt`, `remaining-text-position-evidence.json`에 보존했다.

범례 TextBlock의 own paint 상자를 사용하는 후속 방향 검증은99.83461%다.
이는 실제 Native 페이지의 해당 차트 조각만 교체한 **진단**이며 정식 Native 또는 fresh WASM 결과로 보고하지 않는다.
`legend-owned-paint-provenance.json`에 실제 parent SVG와 진단 source hash를 고정했다.
진단에서 확인한 상자 소비 수정의 생산 반영·최종 Native/fresh WASM·기존 관계형 회귀·lint는 남는다.
새 렌더 회귀/golden은 이 선행 증거를 완료하기 전 추가하지 않는다. 머지 보류와 원격 PR 생성 보류를 유지한다.


### 범례 own paint 상자의 생산 반영 — v446

후속 진단을 생산 renderer에 반영했다. 범례 packing의 물리 마커 높이와 TextBlock의
root paint 상자를 구분하며 실제 문구도 계열의 owning LegendLabel에서 가져온다.
지원하는 제목/범주/범례 정렬은 명시하고 그 외 정렬은 기존 경로를 유지한다.
plot node의 큰 메타데이터는 Box로 소유하며 파일 길이는 column renderer299/reader879줄이다.
실제 Native 빌드는 exit0/89.572초·source unchanged이며 바이너리 SHA256은
`70adad665c81d17d5dd6109c088fd2fc066aeab5f3334fc1c72125fff88d7d6d`다.

| 실제 Native 전쪽 비교(각1/1쪽) | v446 최저 실루엣 |
| --- | ---: |
| 분리 HWP / 사용자2024 PDF | 99.83461% |
| 분리 HWPX / 사용자2024 PDF | 99.83461% |
| 분리 HWP / 독립2024 Print | 99.91386% |
| 분리 HWPX / 독립2024 Print | 99.91386% |
| 원래 단 배치 HWP / 대응2020 Print | 97.05704% |
| 원래 단 배치 HWPX / 독립2020 Print | 97.78626% |

양형식의 새로운 review/overlay를 직접 확인했다. 원래 단 배치 전체 review와 분리 입력의
같은 좌표3배 확대 `native-zoom-review.png`를 보존했다. 범례의 아래 처짐은 개선됐으며
원래 문서의 차트·뒤 문단·표·각주도 직접 대조한다. 한컴 PDF의 글자 전진 보정과 세대별
숫자 기준선의 작은 차이는 앞 절의 독립 근거와 연결하며 전체 PR 완료로 확대하지 않는다.
증거는 ignored `ole-text-layout-final-v446/`에 보존한다.
Native Clippy와 고정 base unit-tier 검사는 exit0이고 WASM Clippy도 통과했다.
최종 최적화 WASM·workspace/all-target lint·같은 입력의 fresh 전쪽 비교는 진행 중이다.

기존 #1156 검사를 그대로 실행한 진단은1 PASS/1 FAIL이며 실패 원인은 `chart_y < 130px`다.
최종 Print의 차트와 뒤 문단이 비겹침이어도 이 고정 원점으로 실패한다.
고정 영역 대신 원본 OLE 참조·Column(1)·최종 Print paint 외곽·같은 단의 뒤 문단 포함/순서를
소비하는 관계형 후보는 HWP/HWPX 모두2/2 PASS다. 이는 독립 rlib 링크 진단이고 정식 nextest가 아니다.
후보는 ignored output에만 있으며 **활성 회귀 검사는 아직 수정하지 않았다**.
최종 Native/fresh 시각 선행 증거와 정식 회귀 확인 뒤 검사 변경을 반영한다.
이 단계에서 신규 렌더 회귀/golden 추가 또는 PR 생성은 수행하지 않았다.

v445 WASM 최적화는 v446의 확정된 범례 상자 수정으로 대체되어 소유 프로세스만 중단했다.
그 실행은1152.441초/exit1이며 성공 증거로 사용하지 않는다.
중단 사유·프로세스 소유와 종료 결과를 `ole-text-position-v445/superseded-wasm-optimization.json`,
`wasm-build-result.json`에 보존한다. v446의 새 최적화는 같은 공용 target과 루트 pkg를 재사용한다.


### v446 Native/fresh 교차 검증과 관계형 회귀 반영

최적화 WASM 빌드는 exit0/1117.671초·production source unchanged다.
루트 pkg와 Studio public의 JS/WASM SHA-256 일치도 확인했다.
Native와 fresh WASM의 위 여섯 전쪽 비교는 모두 같은 점수이며,
모든 실제 PNG의 RGBA가 완전히 같다. 분리 입력 Native/fresh review를 각각 직접 판독했다.
WASM 두 sweep은94.932/78.185초·exit0이고 미달/누락/쪽수 불일치는 없다.
[새 대표 PNG와 producer/입력/TSV 증거](../assets/semanticist21-20261008/ole-owned-text-layout-v446/README.md)를 보존한다.

시각 선행 조건을 확인한 뒤 `tests/issue_1156_chart_column_flow.rs`의 고정px 선택/원점을
원본 OLE 참조·두 번째 단·최종 Print 외곽·같은 단의 뒤 문단 포함/비겹침으로 변경했다.
원래 HWP/HWPX 모두 검사한다. 워터마크 opacity의 기존 의미 검사는 유지했다.
`tests/cases/ole_chart_stored_text_layout.rs`에는 분리 양형식의 값축 가운데 정렬과
범례의 마커 오른쪽/위쪽 반 정렬 관계를 추가했다. 절대 좌표나 SVG 해시는 고정하지 않는다.

정렬 검사 before/after는 커밋 `cba5fc76ef363cec7e1253d9e2062a7f98a7876f`의 실제
OLE reader/combo/기본 painter를 복원하고 같은 원본 Contents·변하지 않은 데이터 IR로 실행했다.
컴파일/실행 성공 뒤 동일 검사에서 `end` 대 `middle`로 FAIL했고, 수정 후 실제 공개 API의
HWP/HWPX는 PASS했다. `before-painter/provenance.json`에 old source hash를 고정했다.
이 비교는 독립 painter 진단이며 이전 전체 Native 문서 재출력으로 과장하지 않는다.

Native/WASM Clippy와 workspace build는 각각 exit0이다. 처음 all-target Clippy는
기존 SO-SUEOP HWP 검사에 남은 단일 항목 for loop 하나로 실패했다.
반복문을 단일 바인딩으로 바꾸었고 기대값/검사 항목은 유지했다.
후속 fmt·fmt check·prepare·고정 base manifest·all-target Clippy retry는 모두 exit0이다.
최종 관련 nextest는 exit0/561.383초이며 6/6 통과했다(빌드9분15초, 실행1.787초).
새 양형식 OLE 정렬1개, 원래 양형식 #1156 관계/워터마크2개, SO-SUEOP HWP 1쪽3개다.
기존 SO-SUEOP HWPX 1쪽 검사도 별도로 exit0/5.275초·1/1 통과했다.
실행 명령과 로그는 `ole-text-layout-final-v446/qualified-test-validation-plan.json`,
`focused-nextest.log`, `hwpx-first-page-nextest-plan.json`, `hwpx-first-page-nextest.log`에 보존했다.
production source는 시각 검증 뒤 바뀌지 않았고 test source hash를 별도로 보존했다.
전체 PR의 나머지 원본/편집 저장본/회귀 게이트는 이 OLE 범위의 결과와 구분하며 머지 보류를 유지한다.

### v446 단계 종료와 PR 검토 재개

최종 확대 review를 다시 직접 열어 범례·제목·값축·범주 위치를 대조했다.
사용자는 원래 HWPX97.79%에서 추가 보정을 중단하고 커밋 후 현재 PR 검토를 재개하도록 지시했다.
분리 차트 HWP/HWPX는 Native/fresh99.83461%이며 이 보정 단계를 마무리한다.
현재 누적 셀/글꼴/OLE 코드·정식 회귀·입력/PDF·대표 이미지·실행 기록을 함께 로컬 커밋한다.
원래18개 체리픽 범위의 검토를 이어가며 후속6개는 체리픽하지 않는다.
전체 회귀와 남은 시각 범위는 아직 완료가 아니므로 개별 PR의 최종 판정은 일괄 변경하지 않는다.
최신 upstream/devel은 fetch 후 SHA가 바뀐 경우에만 rebase하며 원격 push/PR 생성은 별도 승인 대상이다.

## 진행169 — 보정 커밋 후 초기 PR 검토 재개(v447)

사용자의 커밋 후 검토 재개 지시에 따라 누적 셀/글꼴/OLE 보정·입력·PDF·대표 증거를
`3cdcb2e20a47dc23dfdd6d1b399ad4d247430b44`로 커밋했다. 당시 작업 트리는 clean이었다.
fetch 후 upstream/devel은 기존 `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`와 같아
rebase하지 않았다. 원래18개 최신 source head도 개별 기록과 같았으며 #7612는 MERGED다.
후속6개는 체리픽하지 않았다. 원격 push/PR 생성/승인/merge는 수행하지 않았다.
메인터너 누적 검토 경로의 intake/local_validation/multi_pr_update_branch/visual_fixture_evidence를 적용한다.

### 커밋된 head의 실제 전체 회귀

공용 `/home/tsjang/rhwp/target/pr-review`에서 release-test 전체 nextest를 실행했다.
결과는 **10258 PASS/12 FAIL/45 skipped**, exit100/1687.018초다.
가장 긴 거대 셀 검사도936.269초 뒤 PASS로 종료했으며 quiet/SLOW 출력 때문에 중단하지 않았다.
현재 production source는 실행 계획의 모든 hash와 일치한다.
초기 PR별 계약은 실제 로그에서 **78/78 PASS**를 확인하고 개별17개 review에 연결했다.
이미 머지된 #7612의 기존 독립 검토 판정은 유지했다. 전체 통합 성공은 아직 미충족이다.

명령·head/base·source hash·실패 본문은 ignored `pr-review-resumed-v447/full-nextest-plan.json`,
`full-nextest-result.json`, `full-nextest.log`, `original-pr-contract-results.json`에 보존한다.

### 셀 원본/편집 경계의 최신 Native/fresh 완료

기존10개 저장 입력과 독립2020 Print를 실제 커밋 바이트와 대조했다.
전체 각1/1쪽 Native/fresh는 최저99.76088%·미달/누락/쪽수 불일치0이며
원래 HWP/HWPX 두 입력은100%다. 10개 실제 RGBA 쌍이 모두 동일하다.
최신 compare/review/overlay의 셀 외곽·글자·뒤 내용 위치를 직접 확인했다.
[대표 이미지와 전체 TSV·생산/입력 hash 연결](../assets/semanticist21-20261008/cell-stored-lead-final-v447/README.md)을 보존한다.
기본 CLI 시각 검사는 commit timestamp guard로 거절되어 출처가 고정된 절대 경로로 재실행했다.
전체 회귀와 겹친 Native 캡처의 navigation timeout 뒤, 전체 회귀 종료를 기다려 마지막2개만
새로 캡처했다(exit0/75.631초). fresh10개는 새로 실행해 exit0/384.813초로 완료했다.
같은 공용 target과 루트 pkg를 사용했고 production source와 package는 바뀌지 않았다.

### 전체 실행에서 찾은 OLE 검사·신규 입력 래칫 보정

- 기존 #1251/#1283 두 전체 문서 SVG 검사는 데이터 IR의 ASCII 공백 제목을 화면에도 요구했다.
  원래 HWP/HWPX의 저장 TextBlock과 실제 새 출력은 `연금\u3000재정\u3000전망`이며 공식 사양의
  소유 텍스트를 보존한다. 같은 원본 Native/fresh 전쪽97.05704/97.78626% 및 직접 판독 후
  두 화면 검사만 전각 공백 원문으로 교정했다. parser/standalone의 정규화된 IR 검사는 유지했다.
- 새 `samples/hwpx/143E433F503322BD33-ole.hwpx`의 IR sweep은 baseline 미등록0에 대해
  `sections[].paragraphs[].raw_header_extra[]`4건으로 실패했다. 같은 원본을 clean한 고정
  devel worktree의 실제 base library로 실행한 대조군과 현재 release-test library로 실행한
  결과는 모두4건이고 값/경로도 같다(raw_header_extra[6..9]:210/148/9/186→0).
  base lib build는 공용 target·exit0/182.428초이며 Native CLI는 변경되지 않았다.
  base library SHA256은 `405e7ca23d4d09e0db95a95e1108a1d5ceaf4770106b2fdba9fc86de06beccee`다.
  이 입력은 원래 Native/fresh99.83461%로 검증됐다. 신규 입력의 기존 왕복 차이4건만
  기준행으로 등록하고 다른 문서·경로의 허용값과 기존267개 개선 행은 변경하지 않았다.
  대조군 source/명령/library hash와 실제 payload는 `base-codec-build-*`,
  `base-codec-probe-*`, `current-codec-probe-*`에 보존했다.

이 두 검사와 신규 기준행의 보정 뒤 prepare·fmt/fmt check·Native/WASM32/all-target Clippy·
workspace build·고정 base manifest는 모두 exit0이다. 관련 정식 nextest는 **18/18 PASS**,
exit0/451.967초(빌드4분47초, 실제 실행161.044초)다. IR 전수1197입력 sweep도 이 실행에서 PASS했다.
production source는 보정 전후 동일하다. 검사 이름·실행 명령·최종 결과는
`ole-ir-correction-validation-plan.json`, `ole-ir-*-result.json`, `ole-ir-ole-ir-focused-nextest.log`에 연결한다.

### 남은 실패와 최종 제출 조건

위 전체12 FAIL 중 OLE 제목2개와 신규 입력 IR1개를 보정·재검증했다.
나머지9개는 아래와 같으며 전체 nextest를 다시 통과했다고 보고하지 않는다.

| 실패 | 현재 확인/다음 검토 |
| --- | --- |
| CGMP #6035 complete row tail | 현재 직접 보정 범위. 독립 Print와 완결 행의 컷/내용 보존을 대조 |
| #6639 rowbreak fragment origins | 수동 작성 LineSeg 입력의 원점 보존과 실제 서식 변경 계약을 구분해 검토 |
| oracle partition15 | 동일 k-water HWPX27→28쪽. #7773의 다른 HWP 원본으로 제외하지 않음 |
| oracle partition13 | issue_157.hwp2→3쪽. 원본/독립 Print의 동일성·base/current 비교 |
| #4514 ECR-004 ownership | 동일 원본의 #7808 이관은 확인. 고정 현재 base의 미달 근거와 검사 이관 조건 확인 |
| #6575 TAC picture line top | 같은 원본의 기존 Native/fresh 미달과 일치하는 #7445 이관은 아직 확인되지 않음 |
| KPS #1156 page37 cut | 직접 보정 범위. 최신 전쪽 Native/fresh 뒤 실제 행 소유 검사 검토 |
| KPS #4698 continuation | 직접 보정 범위. 실제 셀 포함/내용 보존과 부적합 고정 영역 검토 |
| #5833 stacked cell pictures | 같은 원본 독립 Print의 실제 흐름과 base/current 미달 근거 대조 |

동일 원본 문자열을 #7445의 최신90개 하위 이슈에서 확인했다. 이미 이관된 다른 형식/원본을
현재 실패의 제외 근거로 사용하지 않았다. 시험지 원본은 보존한다.
남은 실제 실패·시각/backend 범위·최종 통합 gate를 해결한 뒤 개별 PR 판정과 제출 후보를 확정한다.


## 진행 갱신170 — 저장 원점의 두 소비 경로 보정 및 devel 동기화 준비 (2026-10-13)

검토 HEAD `dc5b73f60448404e8db18e8ebeb0912cf6b90599`에서 기존 실패9개를 이어 검토했다.
사용자가 다시 devel 동기화·리베이스를 지시했으며 fetch exit0으로 새 base
`552f5228d2dbfd6046c838f14241704f1bcb8a05`(#7861)를 확인했다.
기존 base `bfa28a77a2714482f647ee6a7b9aa7f7876d8404` 대비 HEAD가 실제로 바뀌었으므로
현재 검증 종료·보정 커밋 뒤 새 base로 리베이스한다. 이전 검증을 새 base의 통과로 재사용하지 않는다.

### 구현의 공통 원점 계약

- `cell_format_origin::restamp_cell_format_origin`은 논리 셀 첫 문단의 저장 앞 띠만 변경한다.
  `cell_paragraph_spacing::paragraph_before`의 `para_index == 0` 소유 계약과 맞춘다.
  이후 이어받기 조각의 reset 원점은 고정하고 문단 흐름 간격은 기존 vpos 사다리에서 소비한다.
  기존 #6639의 batch/non-batch·undo·원점 보존 검사 기대값을 완화하지 않았다.
- 표 block entry의 빈 저장 호스트는 검증된 연속 저장 단의 `source_host_origin`이 있을 때
  문단 앞 간격을 뺀 프레임 원점을 사용한다. `from_empty_stored_ladder_host`가 생산하는
  동일 `ParagraphFloatPlacement`의 `occupied_bottom`을 whole-fit에 사용하고
  `record_paragraph_float_placement`로 실제 paint에 전달한다.
  편집·합성 줄·불연속·다단·부분 표의 원점 거절 조건과 중첩 셀 호출은 유지했다.

### 동일 원본의 base/current 및 후보 비교

새 source를 만들기 전 실제 clean bfa worktree의 CLI를 공용 target에서 빌드했다
(exit0/91.023초, SHA256 `8312404e26a74d317da14054b2bae22c460625a1928848302fe1d5d445184e47`).
현재 Native CLI도 별도로 hash 고정하여 같은 원본·Print·글꼴·compat2022·DPI96으로 비교했다.

| 원본/범위 | bfa Native (%) | 보정 전 dc5 Native (%) | 판정 |
| --- | ---: | ---: | --- |
| issue_157.hwp 전쪽 | 1쪽99.74326 / 2쪽74.52729 | 3쪽, Print2쪽과 불일치 | 기존 미달과 현재 쪽수 회귀 모두 남음 |
| issue4514/sample1-repro.hwp 10쪽 | 50.32920 | 14.47312 | #7808의 동일 원본·미해결 소유 문제. 새 악화를 기존 차이로 면제하지 않음 |
| issue6575/tac_picture_line_top.hwpx 1쪽 | 25.29586 | 26.99995 | 같은 축소 입력의 이관 근거 부족. 검사 기대값 변경 보류 |
| issue5833/cell_multi_para_float_pics.hwp 1쪽 | 6.62281 | 6.61305 | 같은 축소 입력의 원점·Print 계약 미달. 검사 기대값 변경 보류 |

두 보정의 Native CLI build는 exit0/200.966초다. #157 후보는 원본을 바꾸지 않고2쪽으로 복원했으며
전쪽 TSV는1쪽99.74326/2쪽78.74413%다. review2쪽을 직접 확인했다.
하단 표는 현재 y820.9px/Print820.063px로 대응하지만 상단 표는 현재232.8px/Print224.395px로
차이가 남는다. **부분 개선이며 90% 통과·완료로 판정하지 않는다.** fresh WASM은 아직 실행 전이다.
정식 #6639 및 CGMP complete-row 검증은 현재 실행 중이며 다음 기록에서 최종 exit/함수별 결과를 연결한다.

명령·원본/PDF/CLI/source SHA·TSV·PNG·상세 관측은 ignored
`regression-root-cause-batch-v448/{implementation-plan,paired-native-plan,candidate-issue157-plan}.json`,
각 `*-result.json`·log와 review PNG에 보존한다. 원본·PDF·기존 정확한 assertion은 삭제하지 않았다.
사용자가 요청한 output 정리로 종료된 v1~v99 진단 바이너리97개(5,325,788,648bytes)만 제거했다.
`/proc/*/exe` 사용 여부·파일별 SHA·크기는 같은 폴더의 `old-completed-binary-cleanup.json`에 남겼으며
공유 `target/pr-review`와 현재/base/candidate CLI 및 검증 증거는 보존했다.


## 진행 갱신171 — 최신 devel 리베이스 완료와 검토 재개 (2026-10-13)

보정 commit `d9a2a63657a80511c0e47c204e7a048fec97475d`를 고정한 뒤
최신 `upstream/devel` **`552f5228d2dbfd6046c838f14241704f1bcb8a05`** 위로165개 로컬 commit을 리베이스했다.
리베이스 후 code HEAD는 **`df7574f29db1a98a34c3cd9043f724922313ffeb`**다.
`merge-base`가 위 base와 같고 작업 트리는 clean이다.
셀 원점·표 block entry·OLE 두 production module·cell spacing 및 SO-SUEOP HWP/OLE 정식 회귀의
7개 파일 바이트가 리베이스 전 commit과 같음을 실제 대조했다.

충돌은 시험지·이관 corpus 선택과 GPU module 경계에서 해결했다.
HWP5/IR/envelope 수집은 최신 시험지 제외와 기존 보존 codec corpus 조건을 함께 적용했다.
시험지 golden 제거·기존 gym 과제 집합·upstream GPU 폐기는 유지하고 legacy font helper를 보존했다.
세 standalone Studio 스크립트는 upstream의 명시적 실행 거절 guard다. 일반 문서 검사의 통과로 집계하지 않는다.
원본 시험지와 범위 밖의 정상 검사는 보존한다. source 전부를 어느 한쪽으로 덮어쓰지 않았다.

리베이스 직전 기존 정식 nextest4개는13분04초 빌드 뒤 **3PASS/1FAIL**로 종료했다.
#6639의 `rowbreak_fragments_keep_origins_and_only_changed_fragments_move`,
`line_spacing_reflow_preserves_rowbreak_origins`, `rowbreak_origin_survives_shrink_then_restore`가 PASS다.
기존 실패에서 원점250→200을 복원했으며 검사 기대값은 바꾸지 않았다.
CGMP `issue_6035_complete_row_does_not_emit_a_line_advance_only_tail`은 종료 컷을 요구하는
기존 전제로 FAIL이 남는다. 전체 회귀 통과나 CGMP 검토 완료로 보고하지 않는다.

새 base의 충돌 영향 Python 검사는4모듈 **134/134PASS**(2.896초)다.
prepare·fmt check·base 고정 manifest·source unit tier 검사도 모두 exit0이며,
공용 `target/pr-review`에서 새 dependency/base의 Native CLI를 다시 빌드하고 검토를 재개했다.
현재 Native build·관련 출력 비교와 남은 회귀/lint/fresh/backend 최종 게이트는 진행 중이다.
기존 bfa 결과를 새552f base의 검증으로 승격하지 않는다.

ignored `regression-root-cause-batch-v448/rebase-result.json`, `rebase-conflict-policy.json`,
`rebase-python-conflict-validation-result.json`, `rebase-validation-plan.json` 및 단계별 log/result에
SHA·명령·관측을 연결한다. 최종 전체 검증 뒤 개별 PR 판정을 확정하며 원격 PR/push/review/merge는
사용자의 구체적인 승인 후 진행한다. 이번 동기화는 원격 읽기와 로컬 커밋/리베이스만 수행했다.

## 진행 갱신172 — 동일 devel 확인과 저장 표 뒤 호스트 보정 (2026-10-13)

사용자의 동기화·리베이스 지시로 `git fetch upstream devel`을 다시 실행했다.
base는 `552f5228d2dbfd6046c838f14241704f1bcb8a05`로 변하지 않았고 이미 HEAD의 조상이다.
같은 base에 추가 리베이스하지 않고 현재 메인터너 보정을 계속했다.

리베이스한 production의 Native 전쪽 비교는 KPS77/77쪽 최저42쪽90.85010%,
CGMP48/48쪽 최저47쪽93.64014%로 미달·누락0이며 독립 Print와 전체 쪽수도 같다.
이는 아래 추가 보정 전 source의 결과다. 추가 보정의 KPS/CGMP 전쪽 Native와
공용 target의 fresh WASM build는 현재 진행 중이며 이전 캡처로 대신하지 않는다.

`issue_157.hwp`의 상단 표는 공백 호스트 줄 **앞**에 놓인 저장 프레임이다.
앞 문단의 마지막 줄 끝11760HU에 표의 offset302HU·위 여백283HU·본체12507HU·아래 여백283HU를
더하면 호스트25135HU이고, 호스트 자신의 줄1200HU·간격720HU를 더하면 뒤 줄27055HU다.
이 독립 저장 메트릭과 한컴 Print에서 표→공백 호스트→위임 제목 순서를 확인했다.
절대 화면 좌표나 문서별 예외로 배치를 고정하지 않았다.

생산·소비 경로는 `entry.rs`의 검증된 현재 단 시작/호스트 원점→
`stored_table_before_host_frame`의 같은 단 slice와 앞·현재·뒤 저장 줄/개체 바깥 상자→
뒤 호스트까지 포함한 whole-fit→`record_paragraph_float_placement`→
`place_table_with_text`의 post-text 1회 전진→같은 frame의 table paint다.
앞쪽 구역 시작 문단까지 검사하던 잘못된 frame 범위를 제거했고,
중첩 셀은 기존 셀 소유 범위와 dirty 검사를 유지한다.
음수 offset·caption·편집·불연속 저장 줄·부분 조각은 이 전체 frame을 쓰지 않는다.

Native build는 exit0/78.798초, 같은 원본·독립2020 Print의 전2쪽 TSV는
1쪽99.74326%·2쪽99.82668%이며 미달·누락0이다(45.604초).
2쪽 review를 직접 열어 두 표의 외곽, 앞뒤 본문·위임 제목·날짜·마지막 안내문의 위치를 대조했다.
수정 전 공백 호스트 줄은 표 본체와 겹쳤고 수정 후에는 표 뒤에 포함 없이 놓인다.
공개 최종 RenderTree의 같은 소유 관계에서도 수정 전FAIL/수정 후PASS를 관측했다.
이는 정식 회귀 suite의 PASS가 아니다. 정식 회귀 초안은 ignored 경로에 두었으며,
fresh WASM의 전2쪽90% 이상·직접 판독 후 `tests/cases/`에 추가하고 실행할 예정이다.

ignored `regression-root-cause-batch-v448/user-sync-check-20261013.json`,
`scoped-frame-plan.json`, `scoped-frame-consumption-review.json`,
`scoped-frame-before-after-relations.json`, 각 build/result와
`scoped-frame-native-issue157/issue157/{silhouette.tsv,review/review_002.png}`에
입력·source hash·명령·출력 근거를 연결한다.
전체 회귀·lint·최종 Native/fresh 검증과 원격 PR 승인은 아직 완료하지 않았다.

## 진행 갱신173 — #157 양 backend 선행 조건과 정식 회귀 검출 (2026-10-13)

Production 보정은 `58b63a927587ebaf1ee5e60dd497d7d208868905`에 고정했다.
새 source의 KPS Native77/77쪽 최저42쪽90.85010%, CGMP Native48/48쪽 최저47쪽93.64014%도
모두 미달·누락0으로 종료했다. 각 해당 독립 Print와 전체 쪽수가 같다.
KPS37·38·65·66 및 CGMP34·35쪽 review를 생성했으며, CGMP34와 KPS66을 직접 대조해
완전한 입고 행과 병합 라벨의 다음 쪽 내용이 보존되는 것을 확인했다.
이 두 문서의 fresh 전쪽 검증이 끝나기 전에는 기존 검사 전제를 바꾸지 않는다.

공용 target·저장소 루트 wrapper의 fresh WASM build는 exit0/1298.272초다.
최적화 WASM SHA256은 `25faec9f8f337cbcdaf439a564ffceea60b7237baf7abcec8bb41ec6275a58d9`이고,
루트 pkg/Studio public의 JS와 WASM이 각각 같은 hash임을 확인했다.
빌드 중 production source hash는 변하지 않았다.

#157 원본은 Native/fresh 모두 전체2쪽, 독립 Print2쪽이며 각1쪽99.74326%·2쪽99.82668%다.
각 쪽의 RGBA와2쪽 review/overlay 파일 바이트가 Native/fresh 사이에서 완전히 같다.
fresh review도 직접 열어 상·하단 표, 앞뒤 본문, 공백 줄, 위임 제목, 날짜와 마지막 안내문을 대조했다.
[Native/fresh 공통 최종 증적](../assets/semanticist21-20261008/stored-before-host-frame-v448/README.md)을
한 벌만 보존하고 동일 이미지를 중복 커밋하지 않는다.

위 선행 조건 뒤 `tests/cases/issue157_stored_before_host_frame.rs`를 정식 원본으로 추가했다.
원본 참석자 표의 소유 문단을 찾고 앞 본문→표→공백 호스트→뒤 제목 및 같은 쪽의 마지막
안내문 보존을 검사한다. 고정px 영역·전체 SVG hash로 배치를 선택하거나 기대값을 복사하지 않는다.
동일한 실제 테스트 소스(SHA256 `836689ffebdb976f0bd624fdc9d653f52bb7c7646e273e5f1bb51964e743f8a3`)를
새 base552f의 실제 library와 보정58b63의 library에 직접 링크했다.
컴파일 양쪽exit0, base 실행1FAIL/exit101(공백 호스트와 표의 겹침), 보정 실행1PASS/exit0이다.
이 개별 정식 소스의 실행을 전체 Cargo/nextest 통과로 확대하지 않는다.
fmt check는exit0이고, formatter 뒤 파생 suite를 다시 준비해 base 고정 manifest를 검사한다.
포맷 전 prepare의 오래된 파생 harness 때문에 발생한 drift는 policy 완화 없이 재생성으로 처리한다.
파생 파일은 커밋하지 않는다.

새 base의 실제 Native 대조도 완료했다: #4514 10쪽50.32920%, #157 전쪽 최저2쪽74.52547%,
#6575 1쪽25.29586%, #5833 1쪽6.62281%다. 이전 bfa 결과로 대신하지 않았다.
#4514는 동일한 overlay 컷/대기열을 생산하지만 보정 head의 마지막 행 요구 높이가
463.8→470.4px로 늘어나 다음 표 전 슬롯에 들어가지 못하면서 paint에서 행이 사라짐을 관측했다.
기존 미달을 현재 악화의 면제 사유로 삼지 않으며 구현 보정 대상으로 남긴다.

K-water HWPX는 기존 `pdf/hwpx/k-water-rfp-hwpx-2020.pdf`(독립27쪽)를 찾았다.
PDF 추가 commit `ce808b588fd43698fd105f0be0320921c83a829c`의 원본과 현재 원본 SHA256이
`036ae73c4982696b9aad50c43a1c47f50871d1447aef3209e058f1b30fa2fc14`로 같다.
새 base는27쪽·현재는28쪽이며 auto/compat2022 모두 같다. 첫 소유 차이는24쪽의 문단255 표가
마지막 행을 이월하는 경계다. 새 MCP job이 macro guard로 거절된 사실과 기존 Print의 존재를 구분한다.
원본/스크립트는 바꾸거나 실행하지 않았으며 기대 쪽수를28로 완화하지 않는다.

종료된 이전 v396/v398 출력의 SVG315개만 삭제하여16,434,615,532bytes를 확보했다.
`/proc/*/fd`에서 사용 중이 아님을 확인했고 PNG·TSV·JSON·원본·PDF·현재 출력은 보존했다.
새 source·base·test/lib hash·명령·exit와 상세 관측은 ignored
`regression-root-cause-batch-v448/`의 `scoped-frame-*`, `issue157-formal-*`, `new-base-*`,
`kwater-existing-print-intake.json`, `old-completed-svg-cleanup-20261013.json`에 연결한다.
KPS/CGMP fresh 전쪽 비교, 남은 회귀 보정과 전체 회귀·lint·최종 PR gate는 계속 진행 중이다.
원격 push/PR/review/merge는 아직 수행하지 않았다.


## 진행 갱신174 — 개별18개 PR 기록 갱신과 자동 승인 (2026-10-13)

작업지시자의 “개별 pr 문서도 생성 후 진행” 지시에 따라 기존18개 개별 review의 존재와
현재 기록을 확인하고 갱신했다. 통합 보고서만으로 개별 기록을 대신하지 않는다.
17개 미병합 원 PR의 적용39개 commit은 최초 적용 patch와 현재 rebased commit의 stable patch-id가
같고 모두 현재 HEAD의 조상임을 확인했다. 각 문서의 적용 SHA를 새 base552f 이후 SHA로 고쳤다.
#7612의 기존 병합 SHA는 현재 base의 조상임을 확인하고 승인·병합 기록을 유지했다.

각 문서는 원 source와 적용 SHA, 이슈·입력·변경 파일·검사 함수, 실제 v447 계약 PASS와 당시 SHA,
현재 production 보정58b63, 남은 최종 검증과 원 PR 후속 처리 계획을 따로 기록한다.
현재18개 기록의 탐색 표는 다음과 같다.

| 개별 리뷰 문서 | 현재 상태 |
| --- | --- |
| [#7566](pr_7566_review.md) | 머지 보류; v447 개별 계약 4/4 PASS |
| [#7612](pr_7612_review.md) | 기존 병합 완료; base 포함 확인 |
| [#7613](pr_7613_review.md) | 머지 보류; v447 개별 계약 2/2 PASS |
| [#7614](pr_7614_review.md) | 머지 보류; v447 개별 계약 8/8 PASS |
| [#7615](pr_7615_review.md) | 머지 보류; v447 개별 계약 33/33 PASS |
| [#7616](pr_7616_review.md) | 머지 보류; v447 개별 계약 3/3 PASS |
| [#7638](pr_7638_review.md) | 머지 보류; v447 개별 계약 1/1 PASS |
| [#7641](pr_7641_review.md) | 머지 보류; v447 개별 계약 3/3 PASS |
| [#7647](pr_7647_review.md) | 머지 보류; v447 개별 계약 3/3 PASS |
| [#7648](pr_7648_review.md) | 머지 보류; v447 개별 계약 3/3 PASS |
| [#7652](pr_7652_review.md) | 머지 보류; v447 개별 계약 1/1 PASS |
| [#7653](pr_7653_review.md) | 머지 보류; v447 개별 계약 4/4 PASS |
| [#7654](pr_7654_review.md) | 머지 보류; v447 개별 계약 2/2 PASS |
| [#7655](pr_7655_review.md) | 머지 보류; v447 개별 계약 2/2 PASS |
| [#7656](pr_7656_review.md) | 머지 보류; v447 개별 계약 1/1 PASS |
| [#7657](pr_7657_review.md) | 머지 보류; v447 개별 계약 3/3 PASS |
| [#7658](pr_7658_review.md) | 머지 보류; v447 개별 계약 2/2 PASS |
| [#7659](pr_7659_review.md) | 머지 보류; v447 개별 계약 3/3 PASS |

사용자는 “자동 승인. 모든 테스트 완료되면 pr 후 머지 후 후속처리”를 지시했다.
이 지시는 이후 같은 범위의 PR/push/merge/후속 처리 권한에 적용한다. 테스트 실패·시각 미달·
미검증을 승인으로 면제하지 않으며, 모든 필수 검증 완료 뒤17개 개별 판정을
“메인터너 보정 후 승인”으로 변경하고 정확한 통합 head의 원격 CI와 머지를 진행한다.
후속6개 PR은 조회/별도 검토만 유지하며 이번 통합에 추가 체리픽하지 않는다.

후속 지시로 최종 판정 문구는 **“메인터너 보정 후 승인”**을 사용한다. 변경 시점은
모든 필수 테스트·검증이 완료되고 미해결 문제를 해소한 뒤이며, 현재 보류를 미리 승인으로 바꾸지 않는다.


## 진행 갱신175 — KPS 전쪽 양 backend 완료와 부적합 검사 정정 (2026-10-13)

Production58b63의 KPS Native/fresh는 각각77/77쪽·독립 Print77쪽, 최저42쪽90.85010%,
미달·누락0으로 완료했다. fresh는exit0/1876.442초이며 생성 중 source hash가 변하지 않았다.
전체77쪽 RGBA가 Native/fresh 사이에서 같으며, 기존 Native37·38·65·66쪽 review를 직접 판독했다.
[공통 최종 이미지와 입력·PDF hash](../assets/semanticist21-20261008/kps-owned-fragments-v448/README.md)에 연결한다.

독립 Print37쪽은 `4. 성능 요구사항`으로 끝나고38쪽은 반복 머리행 다음 `5. 데이터 요구사항`부터 잇는다.
기존1156의 고정 `rows=0..16/16..32`는 이 정상 출력에서 실제FAIL이었다.
원본 작성방법 표의 소유자와 데이터 항목의 행을 찾아 실제 조각의 내용·본문 단 포함을 검사하게 정정했다.
초기 수정에서 같은 제목의 다른 원본 표를 선택한 오류는 `작성방법` 머리행 소유를 대조해 고쳤고,
최종 실제1156 소스의2개 검사(KPS와 그대로 유지한synam 대조군)가 모두PASS했다.

4698의 기존 `x<175/y<200`도 정상 라벨이 그 좌표 밖에 있다는 이유로FAIL이었다.
새 검사는 원본 바깥 표→소유 셀→안쪽 표→병합 라벨의 경로를 따르고,3840HU→0HU 저장 줄
재시작의 앞/뒤 내용을 각각65/66쪽에서 확인한다. 실제 글줄의 셀 포함·순서·누락·중복도 검사한다.
기존 두 정식 검사 소스는 같은 current library에서 실제FAIL을 재현했고 정정한 두 파일은4PASS다.
이는 정식 테스트 원본을 `rustc --test`로 링크한 실행이며 전체 Cargo/nextest 완료를 뜻하지 않는다.
원문 로그·test/lib SHA와 기존/정정 결과는 ignored `kps-contract-before-after-intake.json`,
`kps-relational-validation-results.json`, `kps-*-previous-*`, `kps-*-relational-*`에 연결한다.
formatter 후 prepare와 새 base 고정 manifest 검사는 exit0이고 파생 suite는 커밋하지 않는다.

CGMP fresh 전48쪽은 진행 중이며 검사 전제는 아직 변경하지 않았다.
K-water 진단에서는 새 base의 stored block plan이577.34667px를 반환하지만 현재는None으로 떨어져
저장 호스트417.34667px에 이미 호스트 앞에 있는 중첩 표311.74667px를 다시 더해729.09333px로
측정함을 실제GDB로 확인했다. 새 before-host witness를 확인한 뒤 기존 fallback에 반환한 연결 오류다.
닫힌 저장 표 프레임·호스트·뒤 문단을 측정과 실제 배치가 함께 소비하도록 보정하며, 쪽수 기대값을
완화하거나 큰 높이를 clamp하지 않는다. base whole raster의 Chrome30초 timeout은 기존120초
환경 설정으로 같은 SVG의 나머지 쪽을 이어 캡처 중이다. 완료 전 TSV 통과로 보고하지 않는다.


## 진행 갱신176 — 닫힌 저장 셀 프레임 보정과 대형 SVG 로딩 분석 (2026-10-13)

`stored_closed_cell_frame.rs`에서 모든 표가 호스트 앞의 원본 프레임으로 닫히는 셀의 저장
문단 원점·표 앵커·점유 끝을 공통 결과로 반환했다. `stored_block_cell_flow`의 원래 조합 검증과
앞뒤 LineSeg/여백/실제 표 높이 witness를 유지하고, 전체 셀 사다리의 역행·명시적 새 쪽/단을 배제한다.
이 결과는 HeightMeasurer의 셀 끝과 `sequential_nested_cell_layout`의 같은 문단/표 원점으로 소비된다.
잘못된 큰 높이를 clamp하거나 내용을 숨겨 쪽수를 맞추지 않는다.

Native build exit0(compiler 보고3분19초), K-water 원본의 전체 쪽수28→27, #157은2쪽을 유지했고
현재 library로 다시 링크한 기존 #157 정식 검사1PASS다. K-water 영향24/25쪽 Native는
98.20912/99.64755%이며 전체 쪽수27/27이다. [실제 review·남은 차이](../assets/semanticist21-20261008/closed-cell-frame-v448/README.md)를
직접 열어 중첩 표와 끝 표시·다음 쪽 소유가 복원됐음을 확인했다.24쪽 표 하단 높이 차이가 남으므로
전쪽/fresh·최종 회귀·lint 완료와 승인으로 보고하지 않는다.

CGMP 이전 production58b63의 fresh capture는46쪽 뒤 Chrome30초 navigation timeout으로 exit1이었다.
진행 중 로컬 source가 바뀌어 source guard도false다. 전체 시각 통과로 판정하거나 기존 검사를
바꾸지 않았다. K-water base의 같은 오류는120초 설정으로19쪽까지 복구됐으나20쪽에서 다시 timeout이다.
실물 렌더링 실패와 로딩 비용을 구별했다:20쪽 SVG는270,438,235bytes(약258MB)이고,
글꼴 데이터 URL만 원본과 같은 바이트·SHA의 공용 파일로 참조한 진단 proxy는367,031bytes,
기존 Chrome rasterizer에서5.45초·exit0으로 캡처됐다. 별도의 원본 #157 2쪽 대조군도4.18초·exit0이며
기존 원본 SVG 캡처와 RGBA가 완전히 같다. 형상/글꼴 공급 정책을 변경하지 않고 검증 비용을 줄이는
방식을 추가 대조 중이며 원본 SVG·PDF·글꼴 hash와 proxy 기록을 보존했다.

사용자가 승인한 불필요 output 정리 범위에서 종료된 `binaries/`의 v430 이전 ELF snapshot69개,
24.54GiB를 추가 제거했다. 실제 프로세스 exe/fd와 사용 경로를 먼저 대조했고 현재 v430+,
base/current 후보, 공용 Cargo target·원본·PDF·PNG·TSV는 보존했다.
ledger는 ignored `old-completed-binary-cleanup-v100-v429.json`이다.
모든17개 미병합 원 PR의 최종 판정은 계속 보류이며 사용자 지시대로 전체 필수 검증과 해결이 완료된
뒤에만 **메인터너 보정 후 승인**으로 바꾸고 통합 PR·CI·머지·후속 처리를 진행한다.


## 진행 갱신177 — 같은 글꼴 payload 공용 cache와 중단된 전쪽 비교 복구 (2026-10-13)

`rasterize-svg-webfonts.mjs`의 캡처용 HTML에서만 내장 font-face의 data URL을 같은 font program
바이트·SHA256의 공용 파일 URL로 바꾼다. 원본 SVG·face 이름·별칭·format·실제 좌표는 바꾸지 않는다.
cache는 저장소의 고정 `output/pr-review/font-raster-cache`다. hash 불일치는 실패시키고 완성된
payload만 원자적으로 게시해 병렬 캡처도 같은 바이트를 읽게 했다. 장시간 timeout을 수용 조건
완화로 바꾸지 않으며 `document.fonts.ready`와 실제 Chrome screenshot을 유지한다.

기존 Node 검사7/7PASS·exit0이고 실제 #157 2쪽과 KPS65쪽의 새 canonical cache 캡처는
기존 data URL 캡처와 RGBA가 완전히 같다. 이전에 timeout 난258MB K-water20쪽도 기본30초
한도에서 정상 캡처됐고, 같은 바이트 proxy의 이미지와 RGBA가 같다.
`font-cache-validation-result.json`과 control identity·Node log·font payload hash에 기록했다.

중단됐던 새 base552f의 K-water Native 전27쪽은 빠진20~27쪽을 같은 글꼴 바이트로 보완해
27/27쪽·PDF27쪽, 최저24쪽94.69429%·미달/누락0으로 완료했다.
**이 K-water HWPX는 base에서 이미90% 이상인 정상 회귀 대상**이므로 기존27쪽 기대값을
유지하고 이번28쪽 증가를 구현에서 보정한다. 다른 HWP2024의 #7773을 이 원본의 이관 근거로 쓰지 않는다.
보정d7a13의 Native 전27쪽 비교는 새로 진행 중이다. 선택24/25쪽과 전쪽 완료를 구분한다.

이전 immutable production58b63/pkg의 CGMP fresh도 남은47/48쪽을 복구했다.
Native/fresh 각각48/48쪽·PDF48쪽, 최저47쪽93.64014%·미달/누락0이고 전48쪽 RGBA가 같다.
이는 중간 source 변경으로 guard가false였던 이전 캡처의 불변 package/source에 대한 결과다.
현재 d7a13의 fresh 완료로 승격하지 않으며 최종 code head의 필수 fresh gate는 다시 확인한다.
현재 d7a13 library에 다시 링크한 KPS 두 정식 소스4개와 #157 1개는5PASS다.
전체 nextest·lint 또는 개별 PR 승인으로 확대하지 않는다.

복구 실행·page별 명령·실제 exit·전체 TSV와 입력 hash는 ignored
`kwater-new-base-whole-cached-completion-*`, `scoped-frame-cgmp-fresh-cached-completion-*`,
`scoped-frame-cgmp-native-fresh-identity.json`, `kwater-current-native-whole-*`,
`kps-*-closed-cell-*`에 보존한다. 아직 모든 검증이 끝나지 않아 원격 PR/push/merge는 수행하지 않았다.


## 진행 갱신178 — 현재 K-water 전쪽 완료와 부적합 축약 회귀 두 개 이관 (2026-10-13)

현재 Rust production `d7a13ef73`/도구 `f1461af1f`의 K-water Native 전쪽은 27/27쪽,
독립 PDF 27쪽, 최저3쪽 **96.87092%**, 미달·누락0, exit0(492.29초)로 완료했다.
이전 선택24/25쪽 결과와 구분하며 fresh WASM·남은 하단 괘선 판독·최종 전체 회귀는 미완료다.
이 문서의 올바른27쪽 회귀는 유지한다.

수동 축약 입력은 같은 글꼴·원본·독립2020Print로 고정 base552f와 현재d7의 Native 전1쪽을 비교했다.
base에 입력·회귀가 이미 등록된 사실을 확인했다. base/current review PNG를 모두 직접 열었다.

| 공유 축약 입력 | base552f / 현재d7 최저 | 직접 판독 | 이관 |
| --- | --- | --- | --- |
| `samples/issue5833/cell_multi_para_float_pics.hwp` | 6.62281% / 6.61305% | 양쪽 표·캡션 원점이 Print보다 위, 현재 하단 밴드도 미일치 | [#7863](https://github.com/edwardkim/rhwp/issues/7863) |
| `samples/issue6575/tac_picture_line_top.hwpx` | 25.29586% / 26.99995% | 양쪽 머리 표·분홍 본문·캡션이 Print보다 크게 위 | [#7864](https://github.com/edwardkim/rhwp/issues/7864) |

두 축약 fixture의 fresh WASM은 미실행이다. Native 기존 미달 증거로 이관하고 출력 해결·통과로 보고하지 않는다.
원래 닫힌 #5833/#6575는 전체 원본의 다른 해결 범위여서 이관 근거로 대신하지 않았다.
동일 파일·미해결 범위의 열린 이슈 검색 결과가 없어 정확한 입력 hash·PDF·명령·직접 비교·복원 조건을
별도 이슈에 등록하고 #7445 하위 이슈로 연결했다(직계90→92). 게시 본문과 부모 연결을 API 재조회했다.

사용자가 승인한 진행·후속 처리 범위에서 재현 자산만 `upstream`의
`evidence/semanticist21-cropped-fixtures-20261013`에 게시했다.
asset-only commit `932a56b09814704be856464e9f30f9cb73a965b3`은 base552f 위의 PNG·입력·PDF·이전검사 `.rs.txt`·Markdown만 포함한다.
Rust·workflow 변경이나 원격 PR·merge는 없다. 원본 기여자의 범위 밖 후속 PR6개는 적용하지 않았다.
안정 증적은 [#7863 입력·Print·base/current 비교](../assets/semanticist21-20261008/legacy-cropped-fixtures-v448/5833/README.md),
[#7864 입력·Print·base/current 비교](../assets/semanticist21-20261008/legacy-cropped-fixtures-v448/6575/README.md)에 보존했다.

활성 검사 이관 범위:

- `issue_5833_stacked_cell_pics_flow_without_overlap` 및 `issue_6575_tac_picture_sits_on_line_top_when_line_is_taller` 두 함수·원본 `.rs`만 제거했다. 이전 본문은 이슈 자산 `.rs.txt`로 보존했다.
- #7490 `default_column_rebuild_preserves_content_and_object_geometry`의 #5833 입력 하나만 제외했다. 다른10개와 편집 계약은 유지했다.
- #6840 같은 입력의 SVG 좌표 비교를 이관했다. 원본 그림 소유 경로·참조 payload와 43→7 BinData pruning의 독립 데이터 계약으로 확인했다. 내장 글꼴·전체참조 대조군은 그대로 유지했다.
- 공유 `samples/` 원본 두 개, 독립PDF, parser·codec의 IR baseline5행은 그대로 보존했다. 저장된 source 좌표와 실제 paint 좌표를 혼동하지 않는다.
- 파생 suite를 재준비했다(1459sources, static test attrs6208; 기존2검사 감소). 파생 파일은 stage하지 않는다. #6840 정식 소스를 현재 library에 링크한 실제 실행은3PASS·exit0이고 최종 nextest로 확대하지 않는다. #7490 정식 소스의 남은10개 입력 기본단 재구성 검사도1PASS·exit0(13.90초)다.

이관은 결함 해결이 아니며 Native/fresh 전쪽≥90%·확대 판독 뒤 관계형 회귀 복원이 필요하다.
#4514 현재 변경의 이어받기 행 소실은 이관으로 면제하지 않고 구현 보정 대상으로 유지한다.
CGMP·KPS·가맹사업은 직접 보정 대상이며 제거하지 않는다. 아직 전체 검증이 완료되지 않았으므로
미병합17개 원 PR은 머지 보류다. 모두 해결·필수 검증 후에만 **메인터너 보정 후 승인**으로 바꾼다.


## 진행 갱신179 — 장식 표의 부분 행 소유·물리 높이 전달 보정 후보 (2026-10-13)

#4514의 독립 Print는 ECR-004 앞쪽 마지막 행의 `도입수량`을 앞쪽에, `기능 및 구성`부터
다음 쪽에 소유한다. 원본 셀의 두 첫 문단은 저장 vpos500HU에서 새로 시작한다. 현재는
전체 행470.42667px을 다음 표 위 예산464.12px에 수용하지 못하면 통째로 버려지는 실제 회귀다.
base는463.8px을 통째로 그려 기존 내용 컷 오류도 갖는다. 합성 입력이나 전체 점수로 이 손실을 면제하지 않는다.

보정 후보의 생산→소비 연결:

- `typeset/controls/decoration_table/row_fragment.rs`: 일반 분할의 `LayoutEngine::advance_row_cut`과
  `row_cut_content_height`를 동일 프로파일·column 환경·정규화·원본 행 높이로 조회한다.
  boundary에 걸친 rowspan은 기존 경로로 남기고 RowBreak의 실제 패딩 포함 앞 조각이 예산에 맞을 때만 수용한다.
- `decoration_table::continuation`은 prefix 행·행간격 뒤 예산에서 선택한 유닛 컷과 앞/뒤 물리 높이를 함께 반환한다.
- `state::finish_decoration_table`은 앵커 `OverlayCut`과 이어받는 `OverlayContinuation`에 같은 컷을 복사한다.
  기존 필러 흐름은 reserve0을 유지하고 저장 사다리 reset 후 흐름만 잔여 높이를 예약한다.
- `layout.rs` shapes pass는 부분 마지막 행을 `layout_partial_table`로 같은 end_cut·높이로 그린다.
  이어받기 pass는 같은 start_cut·잔여 높이로 예산을 확인한 뒤 기존 부분 표 경로에 전달한다.
  단순히 선언 전체 행 높이를 반복하거나 허용 px를 늘려 소실을 숨기지 않는다.
- 부분 행이 없는 장식 표의 기존 whole-row paint 경로는 유지한다. 문서 ID나 임의 좌표로 분기하지 않는다.

최초 `cargo check --locked --lib --target-dir /home/tsjang/rhwp/target/pr-review`는 exit0(1분35초)다.
뒤 행간격 회계와 주석을 함께 정리했고 현재 source의 CLI 빌드·기존 실제 실패 검사·경계 Visual Sweep으로 검증한다.
아직 출력 보정 완료·전쪽≥90%·최종 전체 회귀로 보고하지 않는다. 신규 실물 회귀는 선행 시각 검증 뒤 추가한다.


K-water 하단 괘선의 소비 지점도 이어 추적했다. 현재d7 24쪽의 Table bbox 끝은980.4px이나
실제 최종 수평선은964.95px이다. 독립 Print의 같은 선은735.875pt=981.16667px이다.
원본 마지막 빈 문단은vpos44053HU+lh1200HU까지의 점유를 갖는다. `row_cut_content_height`
whole-row가 중첩 셀의 컷 없는 행을 별도 CellUnit 합으로 다시 계산하며 닫힌 원본 프레임의 빈 꼬리를 잃는다.
새 후보는 `StoredBlockCellFlow::closed_before_host`에 생산한 프레임 종류를 보존하고,
온전한 행의 높이 계산도 같은 `flow.bottom`을 소비한다. 부분 start/end컷은 기존 선택 유닛 회계를 유지한다.
어긋난 선을 clamp하거나 높이 숫자만 덧붙이는 방식이 아니다. Native/fresh 경계·전쪽으로 검증할 예정이다.

장식 표 후보의 기존 #4514 검사는 ECR-001~005가3쪽이라는 assertion에서 FAIL했다(실제4쪽).
독립 Print10/11쪽을 대조한 결과 ECR-005의 뒷 본문은11쪽에서 계속되므로 원본 범위는4쪽이다.
현재 후보가 기존에 누락하던 ECR-005 이어받기를 복구했는지 직접 배치와 함께 확인 중이다.
이전 test PASS는 그 뒷 본문 누락으로 얻은 것일 수 있어 정답지와 비교하며, 미달 상태에서 기대값을 먼저 바꾸지 않았다.


## 진행 갱신180 — 7749d 보정 후 최종 검증 진입 (2026-10-13)

생산 코드 `7749d712ca338d1883cc092d8020133b312082f4`를 일반 커밋으로 고정했다.
그 뒤 `git fetch upstream devel` 결과는 기존 `552f5228d2dbfd6046c838f14241704f1bcb8a05`와 같다.
사용자의 지시대로 base head가 바뀌지 않아 추가 rebase하지 않았다. 원 PR18개를 원격 API로
다시 조회했다: #7612만 MERGED이고 나머지17개는 OPEN·비draft다. 현재 적용39개 SHA가7749d의
조상임을 다시 확인하고 각 PR 기록의 현재 production/code head를 갱신했다. **머지 보류는 유지한다.**

- 현재 K-water Native 선택24/25쪽은99.94889%·99.64755%, PDF/실제 쪽수27/27·미달/누락0이다.
  새24쪽 review를 직접 열어 상단 이어받기·중첩 표·`끝`·하단 빈 밴드와 외곽선을 함께 판독했다.
  하단 괘선은 현재982.35px, 독립Print981.16667px으로 기존16px 차이가 없어졌고2px 이웃 안이다.
  전27쪽 재검증은 새7749d 실행으로 별도 진행한다.
- #157 Native 전2쪽은99.74326%·99.82668%, 미달/누락0이다. 이전 결과를 현재 코드 완료로 재표기하지 않고
  새 immutable CLI로 실제 재출력했다. 최종 fresh는 새 package의 실행을 기다린다.
- #4514 현재 Native9/10/11쪽은98.06674%·82.91036%·84.03715%다. 현재변경으로 잃었던ECR-004
  이어받기와 앞/뒤 소유는 복구했고 ECR-005 뒷 본문도 나타난다.10/11쪽의 기존 간격 차이는
  [#7808](https://github.com/edwardkim/rhwp/issues/7808)의 미해결 범위이며 전쪽 완료/90% 통과로 보고하지 않는다.
  Native9/10쪽 review를 직접 열었으며 ECR-004 `도입수량`은9쪽, `기능 및 구성`부터10쪽에 있고 중복이 없다.
  고정 base의 기존 미달·동일 열린 이슈·독립Print11쪽의ECR-005를 근거로 부적합한3쪽 핀만 이관했다.
  정확한 전46쪽 무겹침·연속 소속·ECR-004 두 조각/이웃 관계는 유지했고 실제 정식 소스1PASS·exit0(6.41초)다.
  layout 기대값을4쪽으로 갱신하거나 올바른 소유 검사를 제거하지 않았다.
- 현재 CGMP Native 전48쪽은48/48·PDF48쪽, 최저47쪽93.64014%·미달/누락0·exit0(214.64초), source guard=true다.
  이는7749d의 새 실행이며 이전58b63 결과와 구분한다. CGMP 실패 검사 변경은 현재fresh도 확인한 뒤 진행한다.
- `--prepare`, `cargo fmt --all`, `cargo fmt --all -- --check`는 exit0이고 tree의 Rust source는7749d와 같다.
  NativeClippy exit0(164.24초), WASM32libClippy exit0(157.00초), workspacebuild exit0(307초)를 확인했다.
  workspaceall-targetClippy·base대비정책검사·공용target의freshWASM build·전체nextest/Skia와 후속 비교는 아직 미완료다.

ignored `final-code-validation-v449/`의 Native/순차lint·fresh 계획에 고정 base/code SHA,
각 입력·Print hash·immutable CLI hash·전체 source hash·실제 argv·timeout을 보존한다.
전쪽 Native는 CGMP48→KPS77→K-water27→가맹사업142 순서로 진행한다. 종료된 각 Native SVG의
비대표 쪽만 실제 열린fd를 확인하고 SHA/size ledger를 먼저 남긴 뒤 삭제하며 PNG·TSV·원본·Print·공용target은 보존한다.
시험지/HML 명시 입력의 active Rust 참조를 다시 검색했고 제외 helper 이외 참조는 관측되지 않았다.
원본 시험지 파일은 삭제하지 않았다. 원격 이슈/asset 증거 게시와 PR/CI/merge는 구분하며 아직 통합 PR은 만들지 않았다.
필수 검증과 해결이 끝나야만 개별17개의 보류를 **메인터너 보정 후 승인**으로 변경하고 자동 승인된
통합 PR→최신head CI→머지→후속처리로 진행한다.


## 진행 갱신181 — zip9 검사 보조 호환성과 세 문서 Native 전쪽 확인 (2026-10-13)

7749d의 workspaceall-targetClippy 최초 실행은101이었다. 새 base552f의zip9에서
`ZipFile::name()`은`Result<Cow<str>>`인데 초기 PR의 두 테스트 보조 함수가 문자열로 사용했다.
`inline_table_after_field_or_bookmark.rs`와`field_paragraph_control_slots.rs`에서 이름을
검증해 소유 문자열로 받은 뒤 같은 section0.xml 선택·동일 이름 재포장에 사용하도록 보정했다.
렌더 구현·fixture·기대값·허용치는 바꾸지 않았다. 실제 보조 테스트 실행은 최종 전체 nextest에서 확인한다.

같은 공용target의 전체 workspaceall-targetClippy를 `-j2`로 재실행해exit0(1분25초)을 확인했다.
이 호스트는RAM15GiB라동시 compiler 부하를 줄였으며 대상을 축소하지 않았다.
앞서 통과한Native/WASM32libClippy·workspacebuild와 합쳐 세 lint 경로가 모두 통과했다.
base552f대비manifest확인0(1459sources,6208staticattrs),unit-tier확인0(4103tests),최종fmtcheck0이다.
source쪽cfg(test)인벤토리는진단증적이며stage하지 않는다.

현재Rust생산7749d의Native 전쪽:

| 원본 | 전체쪽수/Print | 최저 | 미달·누락 | 실행 구분 |
| --- | --- | --- | --- | --- |
| CGMP |48/48|47쪽93.64014%|0·0|새실행exit0,214.64초|
| KPS |77/77|42쪽90.85010%|0·0|초기Chrome시작timeout후누락55~77쪽만복구|
| K-water |27/27|3쪽96.87092%|0·0|새실행exit0,571.27초|
| 가맹사업 |142쪽대상|미완료|미검증|현재전체SVG내보내기진행중|

KPS최초실행은ChromeWS endpoint시작30초timeout으로54쪽뒤exit1,TSV미생성이었다.
이를출력회귀나완료로보고하지않고같은immutable7749dSVG로누락23쪽만60초시작한도에서
canonical webfontChrome캡처했다(23개모두exit0). 기존54PNG는정상파일을확인하고보존했다.
전체77쪽TSV/manifest를다시계산했으며sourceguard=true,missing0,전쪽≥90%다.
복구후종료된비대표SVG59개517,026,034bytes만fd/해시ledger확인후삭제했다.
원래실패와후속복구기록을구분하며fresh통과로확대하지않는다.

[7749dNative경계review·overlay](../assets/semanticist21-20261008/closed-row-overlay-v449/README.md)를
안정경로에보존했다. K-water24쪽의직접판독과#4514부분복구/10·11쪽미달을함께기록했다.
현재세Native문서와이전원PR계약의통과를최종전체nextest·fresh/Skia·원격CI완료로승격하지않는다.
CGMP검사보정은현재fresh48쪽확인뒤에진행하며초기17개의최종판정은여전히머지보류다.


## 진행 갱신182 — 현재 Native 경계 판독과 검증 입력 누락 보완 (2026-10-13)

Rust 생산 소스는7749d로 유지하며 현재 HEAD c0fb6734e의 실제 CGMP 정식 소스5개를
같은 debug library에 링크해 실행했다.4PASS/1FAIL·exit101이며, 실패는
`issue_6035_complete_row_does_not_emit_a_line_advance_only_tail`의 `split_pair`가
완료된 행에도 비어 있지 않은 endCut을 요구하는 전제다. 다른4검사는 통과했다.
현재 Native review34/35/36/47쪽을 직접 열었다. 원자재 입고 행은34쪽에서 끝나고
35쪽은 다음 항목부터 시작하며 물 공급 설비 제목35쪽/세 항목36쪽의 소속을 보존한다.
47쪽은 표 외곽·제목/본문/발급자 위치가 대응하며 기준의 굵은 글꼴과 raster 차이가 남는다.
현재 fresh 전48쪽을 확인한 뒤에만 완료 행의 잘못된 전제를 교정하며 아직 검사 source를 바꾸지 않았다.

KPS 현재 Native review37/38/42/65/66쪽도 직접 확인했다.37쪽 성능 요구사항의 끝과
38쪽 다음 데이터 요구사항의 소속,65/66쪽 병합 제목·본문의 이어받기와 뒤 내용이 대응한다.
42쪽은90.85010%이며 글꼴/글줄 내부 raster 차이가 남지만 본문 순서·줄바꿈·쪽 끝은 보존된다.
대표 review 생성은 기존 현재 Native PNG/독립Print PNG를 canonical compare·overlay·review 함수로
합성했으며 새 렌더 실행과 구분한다. 명령·실패/성공·이미지는 ignored v449에 보존한다.

초기 체리픽 편집 사례의 최종 재검증 준비에서 독립Print의 입력3개가 ignored output에만
있었음을 발견했다. [현재 편집 입력 자산](../assets/semanticist21-20261008/current-print-inputs-v175/README.md)에
blank endnote/blank footnote/Hancom HWP footnote의 실제 기존 바이트를 추가 보존했다.
각 입력 SHA와 같은 입력의 독립2020 Print를 연결하며 새로 생성하거나 기준을 교체하지 않았다.
이 커밋은 입력 보존 보완이고 현재 시각 통과·새 회귀 추가·PR 승인으로 보고하지 않는다.

현재 frontend source의 TypeScript 검사는exit0,Studio 단위 검사는1836개 중1834PASS/2SKIP/0FAIL·exit0이다.
동일 frontend 파일 hash guard를 확인했으며 Studio production build·현재 JS/WASM 실제 browser
검증을 대신하지 않는다. 공용target의 현재 freshWASM은 Rust 컴파일12분59초를 마치고
같은 wrapper/cache의 wasm-opt 단계다. 가맹사업 Native142쪽은 전체SVG/render tree 내보내기
이후 임베딩 글꼴 검증 중이다. 전체 회귀·fresh·NativeSkia·원격PR/CI/merge는 여전히 미완료다.
개별17개는 머지보류를 유지하며 모든 필수 검증과 해결 뒤 **메인터너 보정 후 승인**으로 변경한다.


## 진행 갱신183 — 현재 fresh WASM과 실제 Studio 활성화 확인 (2026-10-13)

공용target·저장소 루트 wrapper의 현재 freshWASM build는exit0·1716.277초다.
빌드 기록의 HEAD는c0fb6734e이고 이후b49d666b1은입력/문서만 추가했다. Rust생산은
7749d이며source경로의해시는유지된다. 최적화WASM SHA256은
`6741509894054ff51881f01d78603ba364a5f670c45ef93fa12171b30651c09b`,
JS는`691e4fd480cf4e04f9e599ca7c7a4802ad87b2c141fba93bf94111f4144dcb3b`이고
각각루트pkg/Studio public과일치한다. 현재전쪽fresh비교는CGMP48→KPS77→K-water27→
가맹사업142→#157두쪽순서로시작했으며완료전이다.

같은현재package로임시Studio7791서버와실제Chrome에서#7566누름틀의mouse activation을
실행했다. guide2→1·SECONDGUIDE보존·전체셀경로·JS/WASM 실제응답해시·pageerror0을확인했고
exit0·13.809초다. 임시서버는종료확인했다. 이실제Canvas2D UI검증을독립Print/fresh전쪽이나
다른backend의완료로확대하지않는다. 공개WASM계약6묶음과Studio production/binding검사는진행중이다.

가맹사업의전142쪽검증에서는같은전체글꼴바이트의Unicode cmap을쪽마다다시파싱한다.
후속검증을위해`visual_sweep_font_cmap.py`에프로세스내성공결과캐시를분리했다.
실제font바이트의SHA256을매번계산하며해시가다르면다시검사한다. 캐시는해시→매핑수만
최대128개저장하며폰트바이트·실패결과·픽셀/점수/게이트를저장하거나완화하지않는다.
손상/빈cmap은계속실패하며실패보고의fontSHA도유지한다.

동일바이트파싱재사용·바이트변경/손상/빈cmap차단·캐시상한의3검사와기존Visual Sweep
83검사를합쳐86/86PASS·exit0(최종재실행30.157초),py_compile0,diffcheck0이다.
실제동일가맹사업SVG의수정전후보고서동등성과반복시간도별도측정중이며아직수치개선을주장하지않는다.
이미실행중이던Native비교는이전validator를계속사용하며중단/재시작하지않았다.
새fresh실행은새helper를사용하지만Rust구현/입력/Print/글꼴과실루엣계산식은같다.
진행중인실행의scriptSHA와각source/package/입력hash는ignored v449의실제기록으로구분한다.

독립Print와커밋된입력의해시를확인한원PR편집/정상대조24쌍의새Native비교도진행중이다.
먼저완료한5쌍은전쪽미달0이고blank미주/각주/누름틀100%,Hancom HWP각주97.04843%,
스타일이름없는두쪽원본최저92.07644%다. 나머지와fresh/최종전체회귀는완료전이다.
개별17개최종판정은아직머지보류이며,모든필수검증과해결후에만**메인터너 보정 후 승인**으로갱신한다.


## 진행 갱신184 — 현재 공개 API/Studio 통과와 글꼴 검사 실측 (2026-10-13)

현재67415098…WASM으로공개API검사5묶음과현재HF컨텍스트capture1묶음을모두exit0으로
확인했다. same source/package guard와scriptSHA/원래검사cloning근거는v449에있다.
검사함수·입력·기대값은이전계약그대로이며예전package를재사용하지않았다.
Studio production build와frontend binding검사도exit0이다. 앞서현재TypeScript0·
unit1834PASS/2SKIP/0FAIL과actualpointer0을확인했다. 전체Rust/Skia/전쪽fresh게이트는여전히남는다.

가맹사업첫쪽118,525,186바이트SVG의수정전후font검사보고서를직접동등성비교했다.
첫cmap캐시만적용했을때old8.620/8.742초,cache8.587/8.782초로시간개선을확인하지못했다.
이결과를성능개선으로보고하지않는다. cProfile에서정규식findall2.830초와공백제거sub2.569초가
큰비중임을확인했다. 실제base64바이트의엄격검증은유지하면서Unicode공백제거를
`str.split`으로처리했다. 일반/Unicode공백이섞인정상payload와잘못된base64의실패를
추가확인했고새4검사+기존83검사87/87PASS·exit0(37.003초)다.

동일SVG·4글꼴의기존보고서와새보고서는완전히같았다. 최종실측은
old첫9.339/반복10.362초,new첫7.893/반복6.744초다. 이는이한SVG의진단시간이며
전142쪽의완료시간·전체검증완료로확대하지않는다. 현재Native/cgmp-fresh로이미실행중인
프로세스는시작당시validator를계속사용하며중단하지않았다. 이후시작되는검증에만
새helper/payload처리가적용된다. Rust생산7749d·최적화WASM바이트·실루엣계산·90%gate는동일하다.

각프로세스의실제scriptSHA와현재HEAD를구분한다. 최종Rust source 검사 변경은CGMP의
현재fresh48쪽과독립Print의선행조건을확인한뒤진행하며,개별17개머지보류는아직유지한다.


## 진행 갱신185 — CGMP 양 backend 전쪽 확인 후 기존 의미 검사 교정 (2026-10-13)

현재Rust7749d의CGMP Native/fresh WASM은각48/48쪽·Print48쪽,최저47쪽93.64014%,
미달/누락0이다. fresh실행exit0·693.020초·sourceguard=true이며48쪽전체decoded RGBA가
같다. Native34/35/36/47쪽과현재fresh34/35쪽review를직접열어소유·분할·내용보존을
대조했다. fresh34/35쪽의원자재입고마지막행/다음행시작은독립Print와같으며
글꼴굵기·raster의기존차이는별도로남긴다. [현재 양 backend review](../assets/semanticist21-20261008/cgmp-complete-row-v449/README.md)에대표PNG를보존한다.

이선행조건을확인한뒤`issue_6035_complete_row_does_not_emit_a_line_advance_only_tail`의
완료행에도부분endCut을요구하던전제만교정했다. 전체원본내용의앞쪽소유셀보존/포함,
다음조각이바로다음원본행부터시작,완료행의모든셀을다음쪽에중복표시하지않는관계를
검사한다. `split_pair`와다른4개기존검사는유지한다. 원본·Print·허용치·기준값은바꾸지않는다.
기존5개4PASS/1FAIL에서최종포맷된정식소스5/5PASS·exit0(4.66초)다.
formatter와겹친최초직접compile기록은pre-format진단으로따로보존했으며,formatter종료뒤
다시compile/run하여source SHA 전후동일·최종파일일치를확인했다. 전체nextest결과로승격하지않는다.

초기체리픽편집/정상대조24쌍·전27쪽의새Native비교는exit0(951.827초),
최저현재스타일이름없는원본2쪽92.07644%,미달/누락0·sourceguard=true다. 현재fresh재비교는남는다.
가맹사업Native142쪽의임베딩글꼴512개검증은passed이며현재142쪽캡처중이다.
KPS fresh77쪽은진행중이며K-water/가맹사업/#157fresh와최종Rust/Skia는미완료다.

디스크여유확보를위해종료된이전r448 scoped-frame CGMP/KPS의비대표raw/normalized SVG
204개4,679,530,058bytes만정리했다. 각fd와실행중명령의경로참조를확인하고SHA/size ledger를
먼저기록했다. 대표경계/최저쪽SVG와전체PNG/TSV/JSON·입력·Print·공용target은보존했다.
개별17개의최종판정은머지보류이며,모든필수검증과해결뒤에만**메인터너 보정 후 승인**으로변경한다.


## 진행 갱신186 — KPS/원PR 양 backend 확인과 실제 표 프레임 consumer (2026-10-13)

KPS current7749d Native/fresh는각77/77쪽·Print77쪽,최저42쪽90.85010%,미달/누락0다.
CGMP48쪽과합쳐현재두직접보정문서의양backend전쪽선행조건을확인했다. KPS75쪽은
RGBA가같고22쪽331픽셀/42쪽1픽셀이달랐다. fresh22/42쪽review를직접열어세품질요구표의
본문/병합제목·내용순서,42쪽글줄/줄바꿈/쪽끝을독립Print와대조했다. 각각99.96707/90.85010%다.
동일좌표내글자raster차이를정확히기록하며전77쪽RGBA동일로보고하지않는다.

원PR편집/정상대조24쌍·27쪽의현재fresh는exit0·1087.117초·sourceguard=true다.
Native/fresh모두최저스타일이름없는원본2쪽92.07644%,미달/누락0이며전체27쪽RGBA는같다.
등록입력/Print의커밋바이트·SHA와현재67415098…pkg/public을확인한새실행이다.

현재coupledproducer `stored_block_cell_flow → row_cut_content_height → partial table
physical bbox/paint`의제품consumer도검증했다. 실제StudioCanvas2D에서K-water24/25쪽을
캡처해같은독립Print와비교했다.24쪽99.94940%,25쪽99.66933%,쪽수27/27,실제JS/WASM응답
해시일치·pageerror0·exit0(32.182초),임시7791서버종료확인이다. 두쪽review를직접열어
중첩표/완료된행의바닥과뒤내용,다음첨부의소속을확인했다.
[현재backend review](../assets/semanticist21-20261008/current-backend-v449/README.md)에PNG를보존한다.

capture스크립트의첫두시도는미설치localfont도font.load가성공해야한다는전제로
NetworkError·exit1이었다. 실제제품회귀/완료로보고하지않고시도로그를보존했다.
같은현재NativeFont선언을공급하고font상태를기록하며canonical webfont helper로
로드된맑은고딕regular의synthetic bold를복구한뒤재캡처했다. 한양신명조local부재와
실제webfont/fallback을구분한다. 임의glyph/좌표보정·실루엣계산변경은없다.

최종Rust code/test기준HEAD c771d233ebaedb1a473a07c2758770c250ef0d21에서
prepare/fmt·Native/WASM32lib/all-targetClippy·workspacebuild·base552f대비manifest/unit-tier가
모두exit0이다. 공용target·build-jobs2의전체nextest는컴파일중이며이후NativeSkia3종/doc검사를
순차연결했다. 가맹사업Native142쪽캡처와K-water/가맹사업/#157fresh는아직미완료다.
이기록/asset만의commit은Rust생산/검사바이트를바꾸지않으며실행중hashguard를유지한다.
개별17개머지보류를유지하고전체필수검증과해결뒤**메인터너 보정 후 승인**으로확정한다.

## 진행 갱신187 — 가맹사업 전쪽 미달 검출과 저장 앞 띠 추적 (2026-10-13)

current7749의 가맹사업 Native 전142쪽 비교는 exit1·6038.239초로 종료했다.
Print와142/142쪽이며 55쪽84.64501%,57쪽88.24553%,79쪽85.28897%가 미달이다.
세 쪽 review·overlay를 같은 현재 PNG에서 생성하고 실제로 열었다.
[미달 PNG와 생산 기록](../assets/semanticist21-20261008/franchise-failed-pages-v449/README.md)을 보존한다.
직접 보정 대상이므로 이관하거나 이 결과로 승인하지 않는다. 가맹사업 fresh 전쪽은 시작하지 않았다.
앞서 완료한 CGMP/KPS/K-water fresh는 그대로 보존하고 #157 전2쪽 fresh만 독립적으로 완료했다.

K-water Native/fresh27쪽과 #157 current7749 Native/fresh2쪽은 각각 전쪽 RGBA가 같다.
K-water 최저3쪽96.87092%,#157 최저1쪽99.74326%이며 모두 누락·미달0이다.
원본/Print·source·패키지 hash와 실제 종료 결과는 v449의 whole-fresh-results와
kwater-issue157-current-pixel-identity에 연결한다. #157 Native의 실제 생산 경로는
r448/issue157-closed-row-native-whole이며 존재하지 않는 v449 Native 경로로 집계하지 않는다.

가맹사업55쪽의 대응 문장 기준선은 오른쪽 약4px,왼쪽 약2.7px 위에 있다.
원본 첫 LineSeg300/200HU와 실제 ParaShape 앞 간격600/400(2배 저장)이 각각 이 띠에 대응한다.
`cell_paragraph_spacing::paragraph_before.stored_lead → 측정/CellUnit → 부분 표의
layout_composed_paragraph`를 대조했고, 온전한 Center 셀에서 앞 띠를 억제하는 호출을 확인했다.
79쪽 마지막 분할 행은 별도로 물리 조각 높이·다음 조각·정렬을 추적한다.
GDB 실제 호출은 first cut(0,2),45.8933px,Top/windowed 경로이며 진단을 수정 후 증거로 쓰지 않는다.

최종 Rust 전체 nextest는 컴파일을 마치고 실행 중이다. 현재 HML 파서 성능 검사 한 건이
500ms 한도에서569.922ms로 실패했다. 구현의 row_sizes는 여전히 셀을 한 번 순회한다.
전체 종료 후 같은 코드의 단독 실행과 nextest 자원 배정을 대조하며 상한을 완화하지 않는다.
OLE 원본/분리 차트의 현재 Native/fresh는 점수를 확인했고 SO-SUEOP 첫쪽 fresh는 진행 중이다.
모두 해결·필수 검증 후에만 개별17개의 보류를 **메인터너 보정 후 승인**으로 변경한다.

upstream/devel을 다시 fetch했으며552f5228… 그대로여서 리베이스하지 않았다.
종료된 가맹사업 비대표 SVG는 기존 driver의 fd/hash ledger 후 정리했고,
원본·Print·PNG·TSV·JSON·대표 SVG·공용target은 보존했다. 이 commit은 문서/증적만 변경한다.


## 진행 갱신188 — 저장 앞 띠·반복 프레임의 일괄 보정 후보 (2026-10-13)

이전 c771d 검증은 최종 종료했다. 전체 nextest는10267건 실행,10266PASS/1FAIL/45SKIP이며
실패는 HML row_sizes 성능 guard의569.922ms(500ms 상한) 한 건이다. 초기 원PR 계약78건은
현재 source hash와 전체 로그에서78PASS/0FAIL/미관측0으로 대조했다. 이후 NativeSkia3종과
문서 검사는 driver가 실패 뒤 멈춰 미실행이다. 이 결과로 개별17개를 승인하지 않는다.

동일 구현·검사·상한의 HML 단독 실행은1PASS/0SKIP,테스트88ms·exit0이었다.
파서의 셀 단일 순회와500ms 상한은 유지하고 nextest default/ci-duration-observation에서
해당 검사가 모든 슬롯을 점유하도록 배정했다. 새 전체 실행의 통과는 아직 미확인이다.

가맹사업55/57/79쪽의 원본·독립 Print·실제 GDB 호출을 정밀 대조한 뒤 아래를 함께 수정했다.
문서 ID·고정 픽셀 이동·허용치 완화는 추가하지 않는다.

| 규칙 / 생산 결과 | 실제 측정·예약 소비 | 실제 배치·종료 소비 / 반례 |
| --- | --- | --- |
| `cell_paragraph_spacing::paragraph_before`의 첫 저장 앞 띠 | 부분 표의 선택 줄 높이에도 동일한 `occupied_before` 사용 | 원본 일반 첫 문단은 column-top에서 같은 띠를 소비. 저장 스냅/선형 셀·개체 문단·첫 줄 미소유는 기존 원점 경로 유지 |
| 기존 형식별 `rowbreak_fragment_frame_bottom` | `FragmentBudget.source_frame_bottom`에 반복 프레임 하단을 한 번 확정. 첫 중간 조각의 기존 fit 예산은 별도로 유지 | 첫 수용 프레임의 종료 인증·후속 예산·`stored_body_closing_fragment_extent`가 같은 하단을 사용. 기존2HU rounding만 유지 |
| 원본 재시작 컷이 소유하는 물리 빈 띠 | 선택 유닛·컷을 늘리지 않고 수용한 행 상자의 높이로 예약 | plain/단일행 소유자·실제 저장 재시작·내용보다 큰 실제 end-row override가 입증되면 같은 선택 줄을 원본 정렬로 배치. 임의 예산 컷·병합/개체는 제외 |

79쪽의 이전 첫 조각은 본문 전체 하단으로 종료를 인증했지만 이어받기에서는 기존100HU inset의
하단을 사용해 종료 사실이 전달되지 않았다. 반복 프레임 하단을 공유하며 inset이나 허용치를
새로 만들지 않는다. 남은80쪽의 논리 유닛·줄바꿈은 이미 Print와 일치하므로 그 컷을 바꾸지 않는다.
수정 전55/57/79쪽의 실제 Native 미달과 GDB cut(0,2)/45.8933px/Top-windowed 기록을 보존한다.

새 Native CLI는 공용 `target/pr-review`에서 build-jobs2로 빌드 중이다. 먼저55/56/57/78/79/80쪽의
경계를 새 출력으로 확인하고 직접 판독한 뒤 fresh·전142쪽 및 직접 보정 대조 문서를 검증한다.
시각 선행 조건을 통과하기 전에는 새 정식 회귀나 기존 배치 기대값을 추가·변경하지 않는다.
계획·working source hash·명령은 ignored `franchise-source-frame-batch-v450/native-boundary-plan.json`에
보존한다. 현재 후보는 검증 중이며 개별17개의 최종 판정은 **머지 보류**다.

이전7749d의 OLE 원본/분리 및 SO-SUEOP 첫쪽/대조군10쌍은 Native/fresh 모두 통과했고
각 쌍의 decoded RGBA가 같다. 이 결과를 새 표 보정 후보의 재검증으로 바꾸어 보고하지 않는다.
종료된 이전 K-water 비대표SVG46개3,632,085,473bytes만 fd/hash ledger 뒤 삭제했다.
입력·Print·PNG·TSV·JSON·대표/미달SVG·공용target은 보존한다.


## 진행 갱신189 — Native 경계 통과 후 저장 프레임 소유의 직접 전달 (2026-10-13)

f8495824b의 Native CLI build exit0·12분1초다. 새 경계6쪽 비교는exit0·181.551초,
55/56/57/78/79/80쪽 각각99.99889/99.99893/99.99882/99.99907/99.53600/99.99655%,
미달·누락0·sourceguard=true다. 실제6쪽review를 모두열어독립Print와대조했다.
[새 경계 PNG와 생산 증거](../assets/semanticist21-20261008/franchise-source-frame-boundary-v450/README.md)를
보존한다. Native 포맷·Clippy와WASM32lib Clippy도 같은f849에서각exit0이다.

55/57쪽의 완료행 앞 띠는 정합했고79쪽 반복 프레임의 하단도 복구됐다.
80쪽 전체render tree는이전7749d와동일하며남은12줄의쪽소속·줄바꿈·기준선이유지된다.
다만79쪽 마지막오른쪽두줄은이전위치를유지한다. 높은점수로이차이를면제하지않는다.
첫보정의배치경로가프레임소유를다시판독하던부분을제거하고,예약에서인증·수용한
`stored_body_closing_fragment_extent`의사실을`PageItem.end_row_owns_stored_frame`으로
전달하도록보완한다. 선택줄구성·Center·마지막행접기·cursor의실제소비경로가같은사실을읽는다.
논리컷과다음쪽내용은바꾸지않는다. 구형paginator·overlay·재귀·합성item은인증producer가
없으므로false를넘기며원래동작을유지한다. 주소변환/미주item복사도이소유정보를보존한다.
기존생성자의필드기본값만추가했으며검사기대값·허용치나새렌더fixture는등록하지않는다.

처음기계적라우팅검사에서overlay caller의새인수누락으로cargo check가실패했다.
해당호출을수정했고최종source로재실행중이다. f849 CLI의release-test 프로파일은
debug정보가없어GDB breakpoint가pending으로끝났다. 내부변수를관측했다는근거로사용하지않는다.
실제관측은새PNG·render tree와독립Print의대조다. 이추가보정도새source에서빌드/경계/전쪽을
재확인하며,개별17개는모든필수검증과해결전까지 **머지 보류**다.


## 진행 갱신190 — 실제 정렬 높이 덮어쓰기 확인과 교정 (2026-10-13)

c1747cc7a Native 경계6쪽은다시exit0·184.871초,동일한6쪽점수로종료했다.
새포맷·Native/WASM32lib Clippy도각exit0이다. 그러나79쪽마지막오른쪽두줄위치는
그대로여서해결완료로판정하지않았다. 디버그CLI를공용target에서추가빌드해
실제호출의소유/높이/분기를확인했다. release-test의누락된debug정보로추정하지않는다.

GDB실측은`end_row_owns_stored_frame=true`,windowed=false,원본Center,
부분셀높이52.1600px,같은선택줄높이42.1333px인데최종정렬높이는54.9333px다.
`ordinary_source_frame_owns_alignment=true`인뒤분기가같은컷의물리전진량으로
올바른선택줄높이를덮었다. 종료프레임의예약/소유는정상전달됐으며문제는후속소비다.
로그/스크립트는ignored v450의trace-owned-closing-row/trace-owned-alignment-dispatch에있다.

인증·수용한종료프레임에는선택한가시줄높이를먼저소비하도록교정했다.
물리예약과논리컷을줄이거나기준값을완화하지않고,다음쪽으로전진하는마지막줄간격/
문단뒤간격을이조각의정렬높이로다시넣지않는다. 종료프레임소유가없는나머지기존분기는
유지하며80쪽·앞쪽인접경계와같은source에서재확인한다. 새회귀는여전히양backend의
전쪽선행조건뒤에추가한다. 현재다음빌드/경계/fresh/전쪽/전체회귀는미완료다.

실행중인명령·fd·exe와ELF형식을확인하고종료된이전검토실행파일21개
7,376,214,296bytes를hash ledger뒤삭제했다. 최신7749/552기준·#157수정전후실행파일과
입력·Print·PNG·TSV·로그·공용target은보존했다. 디스크여유는약25GiB다.
개별17개는필수검증과해결후에만 **메인터너 보정 후 승인**으로바꾸며현재 **머지 보류**다.


## 진행 갱신191 — 가맹사업 Native 경계 정합과 최종 전쪽 검증 준비 (2026-10-13)

7f04c48cd의공용dev Native빌드는exit0·51.39초다. 새경계6쪽은exit0·296.619초,
55/56/57/78/79/80쪽각99.99889/99.99893/99.99882/99.99907/99.99881/99.99655%,
미달·누락0·sourceguard=true다. 새79쪽review를실제로열어독립Print와마지막두줄/셀바닥을
대조했고알려진정렬차이가해소됐다. 같은새source로다시캡처한나머지5쪽의전체RGBA와
render tree는직전판독한출력과같다. 이전캡처를새source의실행결과로재사용하지않았다.
[새Native review와해시](../assets/semanticist21-20261008/franchise-selected-visible-frame-v450/README.md)를
보존한다. 고정px이동·원본/Print변경·논리컷변경없이선택줄높이의소비순서만바로잡았다.

최신7f04c의포맷·Native/WASM32lib Clippy는각exit0이며workspacebuild/전체targetClippy/
base552대비manifest와unit-tier를순차실행중이다. 현재snapshot의여유공간과실행중Cargo를
확인하고latest Native 전142쪽및직접문서/대조,root wrapper의fresh WASM과전쪽비교를이어간다.
실행계획/생산소스·입력해시는ignored final-code-validation-v451에있다.

원PR78계약의이전c771d全PASS와현재6쪽통과를최신전체회귀/전쪽fresh완료로확대하지않는다.
새정식관계형회귀는같은원본의Native/fresh전쪽선행조건뒤에추가한다. 그뒤최종전체Rust,
NativeSkia3종/문서검사·Studio실제consumer와필수gate를완료한다.
모든필수검증/해결후개별17개를 **메인터너 보정 후 승인**으로바꾸고자동승인된PR/CI/merge/
후속처리를진행한다. 현재개별최종판정은여전히 **머지 보류**다.

## 진행 갱신192 — 최신 fresh 패키지와 실제 Studio 소비 경로 통과 (2026-10-13)

생산 코드 `7f04c48cd395086ed59c08565f343d3f9e1b013d`의 root wrapper fresh WASM은
exit0·1426.585초이며 source guard가 유지됐다. 공용 `target/pr-review`와 공용 wasm-opt를 사용했다.
`pkg/rhwp_bg.wasm`과 `rhwp-studio/public/rhwp_bg.wasm`의 SHA-256은 모두
`460a928d8423b17b150eee98f4408af330682512f0d0806a78d4cc709316586a`다.
JS 두 경로도 같은 해시다. 실제 패키지 반영 후 Studio production build·frontend bindings·
공개 API 소비자·누름틀 마우스 동작 검증은 각각 exit0이다. 임시 Studio 서버는 종료했다.
TypeScript와 Studio 단위 검사도 새 실행에서 exit0, 1834 PASS·2 SKIP·0 FAIL이다.
이는 전체 조판 비교나 Rust 회귀의 완료를 뜻하지 않는다.

최신 가맹사업 전142쪽 Native는 임베딩 글꼴 검사를 마치고 페이지별 캡처 중이다.
직접 보정 CGMP/KPS/K-water/#157과 원PR 편집·정상 대조24쌍, OLE/SO-SUEOP 범위도
동일 생산 소스의 새 Native/fresh 비교를 실행한다. OLE 원본·분리6쌍의 양backend 비교는
모두 exit0이며 원본 HWP/HWPX는97.05704/97.78626%, 분리 차트는 독립 Print99.91386%,
사용자 PDF99.83461%다. SO-SUEOP 뒤 단계와 직접 문서의 전체 gate는 아직 완료 전이다.
전체 실행 명령·입력/Print/소스/패키지 해시·TSV·로그는 ignored
`output/pr-review/semanticist21-20261008/final-code-validation-v451/`에 보존한다.

앞선 lint driver의 파생 harness drift는 prepare 뒤 base552 대비 manifest와 unit-tier를
재실행해 각exit0, 준비된 전체 target Clippy도 exit0으로 교정했다. 포맷·Native Clippy·
WASM32 lib Clippy·workspace build와 함께 최신 소스의 lint 묶음을 충족했다.
최신 전체 Rust 회귀·Native Skia3종·doc 검사는 별도로 순차 실행 중이다.
신규 sample 보안 검사에는 이번 base 대비 추가된 분리 OLE HWP/HWPX 두 입력을 명시해 전달한다.

이전 v449에서 캡처가 완료되고 대응 PNG/TSV가 있는 중복 SVG146개6,708,924,192bytes만
fd 확인·SHA ledger 뒤 삭제했다. 과거 가맹사업 미달55/57/79쪽 SVG, 모든 원본/Print/
PNG/TSV/render tree/로그와 공용 target은 보존한다. 정리 스크립트의 첫 실행은 파일명
변형에서 IndexError가 발생해 삭제 전에 중단됐으며, 이름 검사를 교정한 재실행이 exit0이다.
종료된 idle 검증 대기 프로세스만 종료하고 공용 Cargo 작업 종료·공간 확보 후 전체 회귀를 시작했다.

개별17개 판정은 여전히 **머지 보류**다. 새 저장 앞 띠·종료 프레임 관계 검사는 ignored
초안으로만 준비했으며 전쪽 Native/fresh 선행 조건 전에는 활성 회귀에 추가하지 않는다.
모든 필수 검증·해결 뒤에만 각 PR의 보류 사유를 **메인터너 보정 후 승인**으로 갱신한다.
이후 자동 승인된 통합 PR 생성·정확한 head CI·merge·후속 처리를 진행한다.

## 진행 갱신193 — 원 PR 양쪽 대조 완료와 timeout/입력 보존 보완 (2026-10-13)

최신7f04c의 원 PR 편집·정상 대조24쌍·27쪽은 Native exit0·873.268초,
fresh exit0·1324.191초다. 양쪽 최저92.07644%, 미달·누락0이며 새27쪽의
Native/fresh decoded RGBA가 모두 같다. 이전에 직접 판독한 Native와도27쪽 모두 같다.
낮은 쪽과 셀 서식 보존 대조군의 새review를 직접 열어 내용 소속·순서·본문/꼬리말·
색/밑줄/테두리를 대조했다. 글꼴 획·폭과 미세한 테두리 래스터 차이는 남는다.
[최신 review·overlay와 입력/Print/생산 해시](../assets/semanticist21-20261008/original-pr-current-proof-v451/README.md)를 보존한다.

OLE/SO10쌍의 현재 검증은 첫5단계exit0 뒤 SO fresh4개 묶음의 마지막 사례에서
1800초 timeout(exit124)으로 끝났다. HWP/HWPX 첫쪽과 일반 글꼴 대조군3개는 이미
99.79294/99.95935/99.84371%로 완료됐다. 미완료 #5821 대조군만 같은소스/패키지/입력으로
독립 재실행해exit0·51.532초,99.75702%를 확인했다. 최종10쌍의 Native/fresh RGBA는
모두 같다. 최초exit124와 원로그를 별도 보존하고 `ole-so-current-qualified-after-retry.json`에
분할 완료 근거를 기록했다. 최초 명령을exit0으로 바꾸어 기록하지 않는다.

입력 commit 확인에서 일반 글꼴 대조군의 실행HWP가ignored 위치에만 있는 것을 찾았다.
[동일 바이트의 대조 입력과 범위](../assets/semanticist21-20261008/so-first-page-normal-font-input-v451/README.md)를
커밋에 보존한다. Native/fresh 첫쪽만 검증한 입력이며46쪽 전체 수용이나 #7807 종료를
주장하지 않는다. 원본/기준PDF/실행입력은 바꾸지 않는다.

K-water Canvas 초기 실행은 최신Native의24쪽 SVG를 만들기전에 시작해ENOENT/exit1이었다.
렌더러 결함으로 분류하지 않는다. 최초결과를 보존하고 Native 전쪽exit0·sourceguard 확인뒤
같은current package를 검증하도록 순서를 고쳤다. 실제 재검증은 아직 완료전이다.
추가로 종료된 v449의 번호 없는 단일쪽 중복SVG28개1,225,328,008bytes를 fd/hash ledger뒤
삭제했다. 입력·Print·PNG·TSV·render tree·로그·공용target은 보존한다.

원 PR 시각 대조와 OLE/SO 범위의 완료를 공통 문서의 전쪽·전체 Rust·Skia/doc·Canvas·
원격 CI 완료로 확대하지 않는다. 최신 전체회귀와 가맹사업/KPS 등 전쪽 비교는 실행중이다.
개별17개는 모든필수검증과해결뒤에만 **메인터너 보정 후 승인**으로 바꾸며 현재 **머지 보류**다.

## 진행 갱신194 — 가맹사업 Native 전쪽 통과와 실제 영향 쪽 확인 (2026-10-13)

최신7f04c immutable CLI의 가맹사업 Native 전142쪽 비교는exit0·5780.035초다.
최저68쪽93.16816%, 미달·누락0, sourceguard=true이며 독립Print142쪽과 쪽수가 같다.
55/57/60/68/78/79/80쪽의 새review·overlay를 만들어 실제로 열었다.
55/57쪽 완료 셀의 앞 띠와 가운데 정렬,78/79쪽 반복 프레임 하단·마지막 두줄,
80쪽의 다음 문단부터 이어지는 내용과 외곽선을 대조했다.60쪽 본문도 소유 상자 안에 있다.
최저68쪽은 이전에 판독한 Native와 전체RGBA/render tree가 동일하다. 글자 획·폭과
밑줄 래스터의 잔여 차이는 있지만 이 보정에 따른 내용 누락·중복·행 소속 변화는 없다.

새전쪽과 이전7749의 비교에서 실제RGBA/render tree 변경은
8/25/26/55/56/57/58/59/67/72/73/78/79의13쪽이다. 나머지129쪽은 모두같다.
위변경을경계6쪽만의변화로축소하지않는다. 추가9쪽의review를생성중이며직접판독뒤
변경영역·내용소속·앞뒤순서·보존을확정한다. 전체자료는ignored v451의
`franchise-current-native-prior-comparison.json`과현재PNG/TSV에보존한다.
전142쪽fresh는동일소스/검증package로실행중이다. 새관계형회귀는양쪽전쪽선행조건뒤에등록한다.

최신KPS fresh 전77쪽은exit0·2124.239초,최저42쪽90.85010%·미달/누락0이다.
이전fresh와77쪽render tree가모두같고76쪽RGBA도같다.22쪽은212픽셀·최대채널차2의
래스터차이만있으며새22/42쪽review를열어표/문단/번호소속·내용과간격을대조했다.
최신K-water fresh 전27쪽도exit0,최저3쪽96.87092%·미달/누락0이며이전27쪽
RGBA/render tree가모두같다. 두문서의최신Native/Canvas뒤단계는아직미완료다.

현재전체Rust회귀는실제실행중이다. 이전유일실패였던
`issue_2833_hml_adapter_row_sizes::inflated_row_count_does_not_slow_down_parsing`는
이번전체에서98ms로PASS했다.500ms상한과검사구현은변경하지않았다.
전체최종exit/summary·Skia3종·doc검사완료를앞서선언하지않는다.
개별17개의최종판정은여전히 **머지 보류**이며필수검증/해결후에만
**메인터너 보정 후 승인**으로바꾸고승인된통합PR/CI/merge/후속처리를이어간다.

## 진행 갱신195 — 전 회귀 통과와 72쪽 정렬 높이 소비 경로 보정 (2026-10-13)

7f04c 소스의 전체 Rust 회귀는 exit0, 10,267개 PASS·45개 SKIP다.
Clippy 세 단계·workspace build·fmt·base 정책 검사도 통과했다. 이후 시작한 Skia와
진행 중인 수정 전 시각 검증은 아래 배치 결함 보정으로 대체하며 완료로 세지 않는다.
완료 결과와 중단한 소유 프로세스는 v451의
`superseded-p72-correction-process-ledger.json`에 보존했다. 공용 target은 유지한다.

추가 영향 쪽을 직접 판독한 결과 가맹사업 72쪽 하단 표의 왼쪽 셀 글줄이 독립 Print보다
위에 있다. 96.90530%라는 점수로 이 차이를 면제하지 않는다. 원본 table661/r2c0의
저장 줄과 GDB 실행을 대조했다. `full_width_frame_owns_alignment=true`로 인증된
선택 글줄 높이는 73.6이지만, 먼저 실행한 일반 행 분기가 물리 전진량 86.4로 덮어써
가운데 정렬 원점을 올렸다. 같은 행 오른쪽 셀은 Print 기준선과 일치한다.

소비 경로는 저장 전체 행 프레임 인증 → 선택 컷의 `line_ranges` → 마지막 가시 줄의
뒤 전진량을 제외한 `total_content_height` → `centered_content_height` →
`text_y_start` → 실제 글줄 배치다. 인증된 전체 행 프레임도 이미 수용한 종료 프레임과
같이 선택 가시 높이를 먼저 소비하도록 우선순위를 보정한다. 일반 행 fallback,
재시작 병합 셀, 임의 예산 컷의 인증 조건·예약 높이·유닛 소유는 변경하지 않는다.
문서 ID나 고정 오프셋을 구현 조건으로 추가하지 않는다.

v452에서 실제 영향 13쪽과 정상 대조 60/68/80쪽을 먼저 재출력한다. 현재 보정은
아직 빌드·시각 검증 전이며, 수정 전 7f04c의 통과 결과를 새 소스에 적용하지 않는다.
양쪽 시각 선행 조건 이후 관계형 회귀를 추가하고 최종 필수 검증을 다시 완료한다.
개별 17개 PR은 여전히 **머지 보류**다. 모두 해결·검증된 뒤에만
**메인터너 보정 후 승인**으로 변경하고 승인된 원격 후속 절차를 진행한다.

## 진행 갱신196 — 새 소스 빌드와 변경 전후 실제 정렬 관계 확인 (2026-10-13)

현재 production은 `b4ff8c81b5d986baf47bad9a3eee83cbd9d8367b`다.
공용 `target/pr-review`의 Native 빌드 exit0·50.03초, fmt check exit0을 확인했다.
새 immutable CLI로 같은 원본 72쪽을 실행 추적했다. 전체 행 프레임 인증과 선택 컷은
유지되며, 정렬 높이는 선택 글줄 높이 73.6을 그대로 소비한다. 왼쪽 세 줄은 6.4만큼
내려가고 오른쪽 셀 및 셀 외곽은 변경되지 않는다.

원본 첫 저장 앞 띠와 셀의 위·아래 점유 관계를 같은 검사로 대조했다. 왼쪽은 수정 전
FAIL / 수정 후 PASS이며 오른쪽 대조군은 전후 PASS다. 실제 render tree와 독립 Print
기준선은 v452의 `p72-before-after-alignment.json`, 관계 판정은
`p72-centering-before-fail-after-pass.json`에 보존한다. 이는 진단 증거이며 정식 회귀
등록·Native/fresh 시각 통과를 대신하지 않는다. 관계형 검사 초안은 ignored output에
유지하고 양쪽 시각 선행 조건 이후에만 활성 검사로 등록한다.

원 PR 계약 78개는 수정 전 7f04c 소스의 전체 회귀에서 모두 PASS다. 개별 17개 문서에
각 PR의 실제 계약 수와 현재 b4ff8c 재검증을 구분해 기록했다. Native 영향 16쪽 비교와
공용 wrapper의 fresh package 빌드는 실행 중이며 전쪽·최종 회귀·Skia·doc·Canvas·
원격 CI는 현재 소스로 아직 완료되지 않았다. 승인 및 원격 PR/merge는 수행하지 않았다.

## 진행 갱신197 — Native 영향 쪽 통과와 디스크 실패 복구 (2026-10-13)

현재 b4ff8c 소스의 Native 영향 16쪽 비교는 exit0·1103.805초다. 최저68쪽
93.16816%, 미달·누락0, source guard=true다. 72쪽은96.90530%에서99.95995%로
개선됐다. 새72쪽 review와 동일 좌표 확대 비교를 직접 열어 첫 두 문단의 글줄·기준선,
오른쪽 분류명·행 구분선·셀 외곽을 확인했다. 나머지15쪽의 새RGBA와 render tree는
이미 판독한7f04c 출력과 동일하다. 현재 Native 자산은
[`franchise-full-width-selected-lines-v452`](../assets/semanticist21-20261008/franchise-full-width-selected-lines-v452/README.md)에 보존한다.

공용 root wrapper의 fresh package 빌드는 exit0·1502.211초다. root/pkg와
Studio/public의 WASM 및 JS 해시가 각각 동일하다. WASM SHA-256은
`94fe70f13609c5a006b8e9044463fac7ae96fe5fe03ae33bc0c0153b19f7867c`다.
upstream/devel을 fetch했으며552f5228d와 동일해서 리베이스하지 않았다.

이후 Native 전쪽 SVG와 fresh 경계의 글꼴 정책 SVG를 병행해 실제ENOSPC가 발생했다.
이 운영 오류는 검증 성공으로 처리하지 않는다. Native/fresh 및 Rust 드라이버의 초기
실패 로그와 부분 JSON은 v452에 보존했다. 초기 Rust의 fmt·Native/WASM Clippy·
workspace build는 통과했고 all-target Clippy는 임시 파일 생성 실패였다. 이 실행을
전체lint 또는 회귀 통과로 확대하지 않는다. Studio 재검증 준비도 ENOSPC로 실패해
미실행이며 이전 source 결과로 대신하지 않는다.

terminal 확인 뒤 실패한 중간SVG272개21,768,288,046bytes를 제거했다.
별도로 현재 출력과 열린 파일을 제외하고, 해당 쪽PNG와TSV가 존재하는 구 증적의
중복SVG531개12,661,724,517bytes를 제거해33GB를 확보했다. 원본·독립Print·
PNG·TSV·render tree·로그·immutable CLI·공급 글꼴·공용target은 보존한다.
앞서 정리한 종료v451 중복219개와v450 경계18개도 각각 별도ledger에 기록했다.

검증 결과 JSON은 원자적으로 게시하도록 ignored 드라이버를 고쳤다. 대용량 출력은
순서를 조정하고 완료된SVG는 PNG/TSV 확인 뒤 정리한다. 필요한K-water24쪽 SVG만
Canvas 검사 때까지 보존한다. 이미 검증한 package를 다시 빌드하지 않고 동일 CLI/
package/input/source guard로 fresh 경계와 실패한 Rust 게이트를 재시도 중이다.
Native 전쪽·fresh 전쪽·관계형 회귀 등록·최종 backend/UI·원격CI는 아직 완료 전이다.
개별17개는 **머지 보류**이며 모두 해결·검증된 뒤에만 **메인터너 보정 후 승인**으로
바꾸고 승인된 PR/merge/후속처리를 이어간다.

## 진행 갱신198 — 디스크 복구 뒤 최신 필수 lint 통과 (2026-10-13)

동일 b4ff8c 소스의 재시도에서 prepare·fmt·Native Clippy·WASM lib Clippy·
workspace build·workspace/all-target Clippy·고정 base manifest·unit-tier 검사 모두
exit0이다. 전체 nextest는 현재 실행 중이며 새 소스의 전체 PASS로 앞서 보고하지 않는다.
Studio 재검증 준비를 복구했고 TypeScript/단위 검사·production build·frontend binding·
현재 package의 public consumer6종·실제 PR7566 pointer 검사는 모두exit0이다.
actual pointer는 새WASM 해시를 확인했고 임시 서버를 종료했다. K-water Canvas는 별도 미완료다.
fresh 경계 재시도는 실행 중이다. Native 전쪽 재시도는 fresh 경계의 완료·직접 판독 뒤
시작하도록 대기시켜 동일한 두 대용량 중간 출력이 다시 겹치지 않게 했다.
필수 시각·회귀·backend/UI·원격 CI의 완료 판정은 여전히 보류한다.

## 진행 갱신199 — 현재 소스 OLE 6개 Native/fresh 통과 (2026-10-13)

b4ff8c 소스와94fe70 WASM package의 OLE6개 비교는 Native/fresh 모두exit0이다.
분리4개는 독립Print99.91386% 또는 사용자PDF99.83461%, 원본HWP97.05704%·
HWPX97.78626%다. 각 Native/fresh의RGBA와 이미 판독한7f04c Native의RGBA가
6개 모두동일하다. 실제 비교 결과는 v452의 `ole-current-pixel-identity.json`에 보존한다.
사용자가 수용한 원본OLE 범위를 추가로 보정하지 않는다. SO-SUEOP4개는 실행 중이고
전체10개 완료로 보고하지 않는다. 새소스의 전체Rust는 컴파일 진행 중이다.
현재fresh 경계의 글꼴 정책은 처리 중이며, 필요한K-water24쪽 SVG/Native27쪽
완료를 기다리는 actual Canvas 검사도 준비했다. 전쪽·최종 회귀·관계형 회귀 등록과
원격PR/CI/merge는 아직 완료되지 않았다.

## 진행 갱신200 — fresh 영향16쪽 통과·직접 판독 완료 (2026-10-13)

b4ff8c/94fe70 소스·package의 fresh 영향16쪽 비교는 exit0·1807.749초,
source guard=true, 최저68쪽93.16816%, 미달·누락0이다. 현재 Native와 fresh의
decoded RGBA16쪽 모두 동일하다. fresh72쪽 review와 독립 Print의 같은
표 영역2배 확대를 직접 열어 선택 두 문단·오른쪽 분류명·기준선·행 구분선과
외곽을 확인했다. 잔여 차이는 획/불릿 크기/래스터이며 배치 결함은 관측하지 않았다.
render tree의 차이는 picture_footnote.rs의 usize sentinel159개64/32비트 차이이며
좌표·텍스트·소유 관계는 같다. fresh 대표 review/overlay/zoom도 기존 v452 자산에
추가했다. ignored `fresh-boundary-direct-reading.json`을 단계 통과로 게시해
Native 전쪽과 fresh 전쪽 대기 작업을 이어간다. 대용량 가맹사업 Native/fresh의
동시 중간 출력은 피하고 완료SVG 정리 뒤 fresh를 실행한다.

SO-SUEOP4개 Native는 exit0, HWP1쪽99.79294%·HWPX1쪽99.95935%·
일반 글꼴 대조군99.84371%·issue5821 99.75702%다. fresh4개는 아직 실행 중이다.
전체 Rust는 실제 cargo-nextest/Rust 컴파일 프로세스가 실행 중이며 최종 통과 전이다.
전쪽·관계형 회귀 등록·최종 backend/Canvas·원격CI 미완료를 승인으로 바꾸지 않는다.
개별17개 HOLD와 승인된 원격 PR/merge/후속처리 순서는 유지한다.

## 진행 갱신201 — OLE/SO10개 최신 양쪽 비교 완료 (2026-10-13)

현재 b4ff8c/94fe70의 OLE6개 및 SO-SUEOP4개는 Native/fresh 모두exit0,
source guard=true, `ole-so-current-result.json`의 all_pass=true다. SO 첫쪽 점수는
양쪽 동일하게 HWP99.79294%·HWPX99.95935%·일반 글꼴 대조99.84371%·
issue5821 99.75702%다. SO4개의 새 Native/fresh RGBA 및 직접 판독한7f04c
Native와의 RGBA가 모두 동일하다. OLE6개도 같은 관계를 앞서 확인했다. 사용한
원본·Print17개 실제 바이트가 현재 HEAD blob과 동일함을 검증했고, 결과는 v452의
`so-current-pixel-identity.json`, `ole-so-current-input-head-proof.json`에 보존한다.
SO 검증 범위는 첫쪽이며 #7807 전체46쪽 해결로 확대하지 않는다.

fresh 경계 직접 판독 통과 뒤 Native 가맹사업 전142쪽과 fresh CGMP 전48쪽
검증 프로세스가 실제 시작됐음을 확인했다. 전체 nextest는 Cargo/Rust 컴파일 중이다.
전쪽·정식 관계형 회귀·backend/Canvas·원격CI는 완료 전이며 개별17개 HOLD를 유지한다.

## 진행 갱신202 — 원본 보존과 정식 회귀 초안의 사전 대조 (2026-10-13)

사용자가 열거한 시험지/HML59개는 실제 파일이 존재하고 현재HEAD blob과 동일하다.
원본 보존 감사의 경로·SHA는 v452의 `deferred-exam-original-preservation-audit.json`에
보존한다. 이는 활성 회귀의 제외 판정과 별개의 원본 보존 확인이다. 기존 공용
corpus_scope/deferred_exam_scope/exam_fixtures 및 이관 목록은 유지한다. 후속 사용자
지시로 추가한 SO-SUEOP HWP/HWPX 첫쪽 정식 검사는 원본59개 목록과 구분한다.

보존된 출력에 정식 초안과 같은 가운데 정렬 관계를 사전 적용했다. 원본 첫 저장
앞 띠를 잃던557표의55쪽0/1행·57쪽4행 양열6개는7749d 출력FAIL/현재b4ff8c
PASS다. 첫 원본 문단을 소유하지 않는57쪽3행1열은 전후PASS다.719표79쪽
마지막 행의 선택 줄 관계는 c1747 전후에 오른쪽FAIL→PASS·왼쪽PASS→PASS다.
이 진단은 독립 원본 메트릭 dump의Center/LineSeg와 실제 tree SHA를 연결하며
`stored-frame-draft-before-after-relations.json`에 보존한다. 72쪽의7f04c→b4ff8c
FAIL→PASS 진단과 함께 정식 검사 검출 범위를 확인하는 자료이며, 정식 Rust 실행을
대신하지 않는다. ignored 초안의 원본 절대 경로·출력 이름 선택·nonce/Drop 정리도
보완했다. 원본 전142쪽 양쪽 시각 선행 조건 전에는 활성 검사로 등록하지 않는다.

전체 nextest는 컴파일 이후10267개 실행에 들어갔으며 최종 summary/exit 대기다.
Native 가맹사업 전쪽과 fresh CGMP 전쪽도 실제 실행 중이다. 개별17개는 계속HOLD다.

## 진행 갱신203 — 최신 CGMP fresh 전쪽 통과와 입력/Print 커밋 확인 (2026-10-13)

b4ff8c/94fe70의 CGMP fresh 전48쪽은 exit0·1010.918초, 최저47쪽
93.64014%, 미달·누락0·쪽수48/48·source guard=true다. 현재48쪽 RGBA는
이전7f04c fresh와 동일하다. 새47쪽 review와 제목2배 확대를 직접 열어
외곽·문단 시작·줄바꿈·내용을 확인했고 획/굵기/일부 자간 차이는 별도로 남긴다.
[fresh 대표 PNG와 재현 범위](../assets/semanticist21-20261008/cgmp-current-proof-v452/README.md)를
보존한다. 해당 TSV의not_evaluated를 자동 승인으로 바꾸지 않으며 현재 Native
전쪽 재실행은 아직 대기 중이다. fresh KPS 전77쪽 비교가 이어서 실제 시작됐다.

전쪽/원PR 편집 대조에 사용한 입력·독립Print58개도 실제 바이트가 현재HEAD와
모두 일치하고 실행계획의 입력해시와 같다. 결과는 v452의
`whole-original-input-head-proof.json`이다. 전체nextest 완료 후 원PR별 계약78개
실제PASS를 추출하는 감사를 준비했다. 현재 전체회귀·전쪽·관계형 회귀·backend/
Canvas·원격CI가 미완료이므로 개별17개는HOLD를 유지한다. 사용자의 PR 준비
지시를 따라 최신 자산과 본문을 준비하고, 필수 통과 뒤에만 PR 생성·CI·merge한다.

## 진행 갱신204 — 원PR 상태 재확인과 통합 PR 본문 준비 (2026-10-13)

GitHub에서 초기18개 상태와 최신 source head를 재확인했다. #7612는MERGED,
나머지17개는OPEN이며 후속6개를 통합 대상에 추가하지 않는다. 개별17개 review의
현재 시각 진행 행을 영향16쪽/OLE·SO10개/CGMP fresh48쪽 완료로 갱신했고
미완료 전쪽·전체회귀·backend와 HOLD 판정은 유지했다. 누적 문서 첫머리의 과거
진행162/base 기록을 현재b4ff8c/552f5228 상태와 구분해 정리했다.

통합 PR 한국어 제목·본문 초안은 ignored v452의 `integration-pr-body-draft.md`와
`integration-pr-preparation.json`에 준비했다. 원PR별 문서와 실제 Markdown
review/overlay 이미지, partial Refs/미해결 범위, 완료/진행/대기 검증을 구분한다.
아직 원격PR을 게시하지 않았고 필수 통과 뒤 최종 제출SHA·시각URL·개별승인 판정으로
갱신해 제출한다. 제출code head CI 뒤 review/오늘할일 trailing commit 및 trailing CI,
exact head merge와 후속처리 순서를 따른다. user 자동승인은 유지하며 다시 요청하지 않는다.

upstream/devel fetch 결과552f5228d2dbfd6046c838f14241704f1bcb8a05로 이전과
동일하다. head가 바뀌지 않아 rebase하지 않았다. 현재 Cargo/시각 프로세스는
정상 실행 중이며 완료하지 않은 항목을 통과로 표시하지 않는다.

## 진행 갱신205 — 현재 전체 nextest·원PR78계약 통과와 devel 동기화 (2026-10-13)

b4ff8c 소스의 전체 nextest는 **10267PASS·45SKIP·exit0**, 실제 테스트2089.664초,
컴파일 포함4088.686초로 종료했다. source guard=true다. 같은 로그에서 원PR17개
계약78개를 정확한 qualified name·source SHA로 추출했고 전부1회PASS, FAIL/SKIP/
미관측0이다. 결과는 v452의 `original-pr-contract-results.json`이며 개별17개 문서에
각 계약의 현재 통과 수를 갱신했다. 원 기여 적용39개가 현재HEAD 조상임도 확인했고
이전 stable patch-id equality와 연결했다. 이 전체 결과는 아직 등록하지 않은 저장
프레임 관계3검사의 통과를 포함하지 않는다. Native Skia/lib 빌드는 이어서 실행 중이며
Skia 두 integration·doc·최종 시각·Canvas가 남아 현재 승인/HOLD를 바꾸지 않는다.

upstream/devel을 다시 fetch했고552f5228d로 기존 검증 base와 같다. 로컬devel은
bfa28a77a에서552f5228d로 fast-forward했다. 현재review HEAD/source를 보존했고
새base의 ancestry를 확인했다. upstream HEAD가 바뀌지 않아rebase하지 않았다.
`devel-sync-current-result.json`에 전후 SHA와 사유를 보존한다. 기존 검사 입력
59개 원본 보존과 활성 제외 helper의 source 참조 감사도 유지한다.

현재 #7445 하위 이슈를 다시 조회해 실제 보정한 CGMP/KPS/K-water/고용지원과
타 브랜치 이관 기록을 구분했다. 이슈 종료는 이번 PR 통합 완료 뒤 정확한 범위로
갱신하며 현재 원격 종료를 수행하지 않았다. 사용자의「남은 사항 모두 검증 후PR 준비」
지시에 따라 남은 필수 검증을 모두 마치고 개별승인/PR/CI/merge/후속처리를 이어간다.

## 진행 갱신206 — 머지 보류 사유를 실제 남은 gate로 정정 (2026-10-13)

전체nextest와원PR계약은통과했는데「전체 회귀 진행 중」이라는 포괄적 문구가
남아 오해를 만들었다. 개별17개 최상단/현재판정과 누적 첫머리를 다음처럼 정정했다:
**전체 nextest10267개/원PR78계약 PASS; 전쪽 Native/fresh·신규 관계 회귀·
Native Skia/doc/Canvas 미완료로 보류.** 과거 실패나 미완료를 현재 전체회귀 실패로
표시하지 않는다. 남은 gate 종료 뒤 승인 문구로 갱신한다. 현재source/검사 기대값은
변경하지 않았으며 새 렌더링 회귀는 여전히 전쪽 양쪽 자격 뒤에 등록한다.

## 진행 갱신207 — KPS fresh全77쪽·Skia lib와 도구 계약 보정 (2026-10-13)

현재b4ff8c/94fe70 KPS fresh 전77쪽은exit0,최저42쪽90.85010%,미달/누락0이다.
새42/68쪽review를직접열었다.76쪽RGBA는앞선7f04c fresh와동일하며68쪽의
차이는한픽셀RGB2단계(253↔255)뿐이다.68쪽전체render tree와실루엣98.69518%는
동일하고본문/날인칸/내용배치차이는관측하지않았다. Native현재전쪽은대기중이다.
Native Skia/lib는exit0이며root3859+workspace15/165/2,합4041PASS·12ignored다.
실제root 실행298.13초이며Native Skia placeholder/directPDF/doc는아직진행전/중이다.

변경된검증도구의시각/Python221개,oracle_self42개,font/rasterNode19개,manifestNode23개는
통과했다. oracle_public/work_receipt137개묶음은초기135PASS/2FAIL이었다. 독립
메타데이터fixture의pairs11개/일반문서8개는이미정확했으나요약pairCount6/
matched5/link6및2020/2022건수2/2가남아있었다. 기존pairs목록과문서/PDF를
보존하고요약11/8/11·연도4/5로정정했으며같은137개재실행은137PASS·exit0이다.
이합성목록은시험지원본렌더회귀가아니며시험지목록을활성화하지않았다. 최초
2FAIL 로그와보정근거/새fileSHA는v452의affected-tool-contracts-qualified-after-retry에
보존한다. 현재5묶음총442개PASS이며Rust기대값/생산소스/시각기준은변경하지않았다.

사용자의「Python만수정이면전체Rust재실행불필요」지시에따라Python/JSON보정에
대해서는관련검사만재실행했다. 이미통과한현재production10267개전체회귀를
다시시작하지않으며진행중Skia는이전Rust보정의기존필수후속게이트다. 향후정식
관계형회귀등록후에도생산코드가그대로라면추가/변경검사와필수Rustlint/정책을
검증하고같은생산소스의전체PASS와연결한다. code변경으로인한재실행과구분한다.

종료된Native/fresh영향16쪽중복SVG32개4,711,622,430bytes를fd/PNG/TSV확인과
SHA ledger뒤삭제했다. 원본/Print/PNG/TSV/tree/immutableCLI/package/공용target은
보존한다.현재가맹사업Native전쪽캡처와freshK-water및나머지필수검증을계속한다.

## 진행 갱신208 — K-water fresh 전27쪽 판독 및 최신 대표 PNG (2026-10-13)

현재 b4ff8c/94fe70 K-water fresh 전27쪽은 exit0, 최저3쪽96.87092%,
미달·누락0, 원본/Print27/27이다. 3·24·10쪽의 현재 review.png와 24쪽
동일 좌표 2배 확대를 직접 대조했다. 문단 시작·목록·이어받기 표 외곽·
중첩 표의 행/내용 순서·뒤 끝 표시는 보존된다. 글자 획·자간·괘선 래스터
차이는 남는다. 이전7f fresh와 26쪽 RGBA 동일, 10쪽 한 픽셀의 회색
1단계 차이만 있고 전27쪽 render tree는 동일하다. 현재 Native/Canvas는
아직 대기 중이므로 양쪽 완료로 확대하지 않는다.

현재 KPS42/68쪽 및 K-water3/24/10쪽 fresh review/overlay·24쪽 확대를
각 `mydocs/pr/assets/semanticist21-20261008/kps-current-proof-v452/`,
`kwater-current-proof-v452/`에 보존했다. 각 README의 입력/Print/소스/
패키지·PNG 해시, v452 direct-reading/provenance와 전쪽 TSV를 연결한다.
Python/JSON 보정은94bb1cf70에 고정했고 관련137개 PASS·전체 도구442개
PASS다. Rust source guard는 그대로이며 전체10267개 검증을 재시작하지 않았다.

## 진행 갱신209 — Skia3종 통과와 원 체리픽 문서 비교 우선 실행 (2026-10-13)

현재 b4ff8c의 Native Skia3종은 모두 exit0/source guard=true다. lib4041개
PASS·12ignored, placeholder2개 PASS, 직접 PDF 검사 exit0을 확인했다.
`final-rust-gates-result.json`과 세 단계의 실제 로그에 연결하며 doc-test는
아직 진행 중이다. 이 종료 결과는 Python 수정에 따른 전체 회귀 재실행이 아니다.

원래 체리픽 문서를 우선하라는 지시에 맞춰24쌍27쪽의 Native/fresh 비교를
먼저 실행했다. 기존 두 대기 스케줄러는 자식 프로세스/캡처/출력 없음과 정확한
PID·명령을 확인한 뒤 종료했으며 렌더링 실행을 중단·중복하지 않았다.
`original-pr-priority-scheduler-change.json`에 기록했다. 같은 승인된 명령,
입력/Print 해시, 현재 immutable CLI와 최적화 WASM을 사용한다. 입력/Print
총1,534,489bytes, 당시 여유18GiB 이상이며 가맹사업 전쪽/Cargo는 유지한다.
현재 원문 로그에서 Native SVG/트리와 fresh WASM 내보내기 시작을 확인했다.

## 진행 갱신210 — 현재 Rust13단계 전체 완료 및 남은 검증 범위 정정 (2026-10-13)

현재 검증 driver는최종exit0/all_pass=true/terminal이다. 필수8개lint/정책,
전체nextest10267PASS·45SKIP, Native Skia lib4041PASS·12ignored/placeholder
2PASS/직접PDF4PASS, doc8PASS·3ignored가모두완료됐다.현재tracked Rust/검사
소스해시는검증시점과모두동일하며Cargo/rustc/rustdoc/wasm-opt실행없음을확인했다.
`final-rust-gates-result.json`과`rust-current-final-qualification.json`에각실제
명령/exit/기간/실행head와현재바이트동일성을연결한다. skip/ignored는PASS로세지않는다.

개별17개최상단과현재상태에서Skia/doc를남은항목으로표시하던문구를정정했다.
보류는전쪽Native/fresh·실제Canvas·시각선행조건뒤의정식관계회귀3개및
기존본문포함검사보완에따른다. 기존Rust전체를Python/문서때문에다시돌리지않는다.
관계회귀등록뒤에는변경검사/필수lint/정책/최신headCI를검증한다.

## 진행 갱신211 — 원 PR24쌍27쪽 양쪽 검증 및 현재 대표 확대 판독 완료 (2026-10-13)

현재 b4ff8c/94fe70의 원 PR편집/정상24쌍27쪽 Native/fresh는각각exit0,
892.431초/1021.503초다. 양쪽최저current-04 2쪽92.07644%,미달/누락0,
모든원본/독립Print의전체쪽수일치및입력/Print/source해시를확인했다.
전27쪽Native/fresh및이전직접판독7f소스의RGBA는모두동일하다.
현재대표16개review와최저원본본문/각주의동일좌표2배/3배확대를직접열었다.
누름틀정상/자식offset·HWP저장의외곽/셀내용/뒤문단,스타일삭제의가시대조군·
원본HWP/HWPX,최저원본문/각주번호/구분선/쪽소속/내용순서를확인했다.
글자획·폭·자간의기존차이는남으며Print와완전픽셀일치를주장하지않는다.

정확한명령/TSV/로그/guard/현재PNG해시와판독은v452의
`original-edited-{native,fresh}-result.json`, `original-current-qualification.json`,
`original-edited-zooms/provenance.json`이다. 대표review/overlay/확대및24입력/Print
해시표는`mydocs/pr/assets/semanticist21-20261008/original-current-proof-v452/`에
보존했다. 다른직접문서전쪽·실제Canvas·정식관계회귀의미완료로는확대하지않는다.

## 진행 갱신212 — 전142쪽 완료 후 118쪽 실제 정렬 회귀 확인 (2026-10-13)

b4ff8c Native전142쪽은exit0·5599.170초,최저68쪽93.16816%,미달/누락0·
142/142다.전쪽새RGBA와7f출력을대조하니140쪽은동일하고72/118쪽이달랐다.
72쪽은96.90530%→99.95995%이나118쪽은99.99852%→96.18473%다.
현재/이전118쪽review를같은Print와직접열어하단중첩표3줄의5.4px하향이동을
확인했다.외곽/내용순서는동일하다.점수90이상이나현재실제배치회귀이므로수용하지않는다.

Debug실제호출에서둘다cell_was_split/full-width/ordinary=true,closing=false다.
full-width owner가원본전체행과중첩1×1원본프레임의다른인증결과를합쳐선택
줄높이를같이소비시킨것이원인이다.원본전체행과중첩셀의인증결과를분리해
각높이계약을유지하고특정문서/좌표조건을추가하지않는다.새수정과검증은다음단계다.

v452의완료된Rust13단계/원PR24쌍등은b4바이트에대한실행사실로보존한다.
추가대형Native/fresh와Canvas대기는정확한소유root/자식PID를확인해중단했다.
`superseded-p118-correction-process-ledger.json`에기록하며실행오류를제품실패로세지않는다.
현재118쪽회귀PNG·GDB/트리/점수/입력근거를
`mydocs/pr/assets/semanticist21-20261008/franchise-p118-alignment-regression-v452/`와
ignoredv452에보존한다.종료NativeSVG141개14,267,740,082bytes,완료된원PR
SVG81개2,389,651,075bytes를ledger/열린fd/PNG/TSV확인후삭제했다.
원본/Print/PNG/TSV/tree/immutableCLI/WASM/공용target은보존한다.

## 진행 갱신213 — 프레임 출처 분리 보정 후보 구현과 Native 검증 시작 (2026-10-13)

공통 마지막 행 컷 조건을 유지하고 원본 전체 행 인증과 중첩 단일 셀 인증을
각각 생산한다. `frame_owns_alignment`는 두 출처를 모두 소비하므로 예약/물리
상자/컷/원본 정렬 소유는 유지한다. `selected_frame_owns_alignment`는 원본
전체 행/종료 프레임의 선택 줄 높이를 소비하고, 중첩 단일 셀은 기존 원본 유닛
높이 소비로 이어진다. 서로 다른 인증을 같은 높이 계약으로 합친 가정을 제거했으며
문서ID/셀개수/고정오프셋을 새 조건으로 넣지 않았다.

Native 빌드 검증 당시 미커밋 `table_partial.rs` SHA-256은`d15dbc30b69549c03b6a2627f9fcd34791bd852ab6b11ad3aebb9a0ef34f4efd`다.
fmt check exit0이고 공용target의 실제 Native build/전142쪽 트리 대조/경계17쪽
Native 비교는v453에서실행중이다.118쪽과72쪽을동시에확인하며전쪽/
fresh/회귀/최종lint의검증완료는아직주장하지않는다. v452의b4검증은역사
바이트에대한증거로유지하고개별17개보류사유를실제118쪽회귀로정정했다.

중단한v452의정책용/중간SVG218개7,665,141,771bytes를정확한root/종료PID/
열린fd확인후삭제했고공간27.75GiB를확보했다. 기존b4 fresh web package는
검증된JS/WASM해시를확인해v452/fresh-web-package-snapshot에바이트보존했다.
이는새빌드/wasm-opt경로가아니며공용target/루트pkg빌드계약을유지한다.

## 진행 갱신214 — 보정 Native 빌드와 전142쪽 실제 트리 대조 통과 (2026-10-13)

v453 Native build는exit0·522.651초/source guard=true다. fmt check도exit0이다.
현재source의전142쪽render tree를실제immutableCLI로내보내대조했다.
직전b4와다른쪽은118쪽뿐이고,정상118쪽을가지던이전7f와다른쪽은
개선한72쪽뿐이다.118쪽의세줄/중첩표/뒤내용은이전정상트리로복구됐으며
72쪽개선과나머지140쪽은보존된다.트리동일성을실제Print/픽셀통과로대신하지않는다.

Native sourceSHA/빌드명령/exit/기간/CLI해시는v453/native-build-result.json,
전쪽관측은whole-franchise-before-after-tree-comparison.json이다.
immutableCLI SHA-256은`bcf415c03b79c2e37627d558628136ca05e356e7cd0b54b0980426a9c292fd69`다.
현재경계17쪽Native이미지/TSV검증은진행중이다.새freshWASM·전쪽/
관련회귀/필수lint/Canvas/CI의검증완료는아직주장하지않는다.프레임분리보정
코드와이관측을일반커밋으로함께고정하고다음검증을계속한다.

## 진행 갱신215 — Native17쪽·118쪽 복구 직접 확인과 fresh 준비 (2026-10-13)

보정production은ddf17f864다. Native17쪽exit0/source guard=true,최저68쪽
93.16816%,미달/누락0이다.72쪽99.95995%를유지하고118쪽99.99852%로복구됐다.
현재72/118 review와동일좌표확대를독립Print와직접대조했다.원래첫16쪽은
b4의통과/판독RGBA와동일하며118쪽은정상7f의RGBA와동일하다.표외곽/글줄/
각주/뒤내용은보존되고글자획·자간·불릿크기·괘선래스터차이는남는다.

정확한source/명령/TSV/PNG/직접판독은v453의native-boundary-plan/result/
direct-reading/prior-pixel-comparison이다.현재대표PNG와구현/소비연결은
`mydocs/pr/assets/semanticist21-20261008/franchise-nested-frame-source-v453/`에보존한다.
새fresh빌드는Native경계/판독완료뒤공용target·루트wrapper로진행하도록준비했다.
새 fresh/전쪽/관련회귀/필수lint/Canvas/CI는미완료다. upstream/devel fetch는
552f5228d로동일해사용자지시에따라rebase하지않았다.

## 진행 갱신216 — fresh 빌드·Native125쪽 확인과 관계 회귀 초안 검출 (2026-10-13)

ddf17f864의 root wrapper fresh 빌드는 exit0·1342.306초/source guard=true다.
공용 target/pr-review와 공용 wasm-opt cache를 사용했고 pkg/Studio public의 JS/WASM 해시가 각각 같다.
WASM SHA-256은 `4299eb204ec8283d75446b61f2183d3c4f3cd41acc0acabfd7be7dc065f01ae0`,
JS는 `691e4fd480cf4e04f9e599ca7c7a4802ad87b2c141fba93bf94111f4144dcb3b`다.
빌드/환경/패키지 증거는 v453/fresh-package-result.json이다.

현재 Native CGMP48/48쪽 최저47쪽93.64014%, KPS77/77쪽 최저42쪽90.85010%로
각각 exit0·미달/누락0이다. 현재 CGMP47/KPS42·68 review를 독립 Print와 직접 열어
외곽·본문 줄바꿈·항목 순서·뒤내용·서명칸을 확인했다. 글자 획/굵기·미세 자간 차이는 남는다.
대표 PNG6개는 `mydocs/pr/assets/semanticist21-20261008/small-native-current-proof-v453/`에 보존했다.

정식 등록 전 관계 회귀 초안4개를 같은 원본의 고정 CLI에서 실제 실행했다.
새 소스4PASS, 원본 앞 띠/반복 프레임/전체 행/중첩 셀의 각 보정 전 바이너리에서
해당 검사1개씩 실제 assertion FAIL(각 exit101)이다. 빌드/환경 오류가 아닌 정렬 관계 실패를
`draft-formal-before-after-results.json`과 원문 로그에 보존한다. 기대값은 원본 Center/LineSeg,
내용 순서와 독립 Print에 따른 셀·문단 소속이다. 현재 구현 helper의 높이나 절대 위치를 핀하지 않는다.
이 초안은 ignored 디렉터리에만 있으며 전142쪽 Native/fresh 시각 선행 조건 뒤 정식 등록한다.

사용자의 Python-only 검증 지시에 따라94bb1cf70의 Python/JSON 보정은 관련137개 PASS로
끝냈으며 그 때문에 전체 Rust를 다시 시작하지 않았다. 이후 실제 Rust 소스 ddf17f864가 바뀌어
새 경계·전쪽 검증이 필요하다. 새 정식 검사를 등록한 뒤 같은 최종 소스의 필수 Rust 검증을 한 번 실행한다.
현재 가맹사업 fresh17쪽, K-water Native전쪽 및 초기 PR24쌍 Native/fresh가 실제 실행 중이다.
가맹사업 Native전142쪽은 fresh 경계 직접 판독과 여유20GiB 뒤 진행하고 fresh전142쪽은
Native 전쪽/중간 SVG 정리 뒤 진행하도록 대기 작업을 준비했다. actual Canvas도 현재 Native 글꼴/
fresh package 준비 뒤 실행한다. 새 전체 통과·승인·PR 생성·merge 완료로 보고하지 않는다.

종료/Native 판독 완료된17개 SVG1,871,818,314bytes만 열린fd/해시 ledger 확인 뒤 제거했다.
입력·독립 Print·PNG·TSV·tree·공용 target·패키지는 보존한다.

## 진행 갱신217 — 새 fresh 경계·초기 PR 전쪽·actual Canvas 확인과 종료 이슈 범위 (2026-10-13)

ddf17f864 fresh 경계17쪽은 exit0·1517.642초/source guard=true, 최저68쪽93.16816%,
미달/누락0이다. 전체 문서 쪽수142/142이며 선택17쪽의 RGBA는 현재 Native와 모두 같다.
현재72/118 review와 동일좌표 확대를 독립 Print와 직접 열어72쪽 개선과118쪽 중첩 셀
세 줄 복구·외곽·내용/각주/뒤 문단의 보존을 확인했다. 글자획·불릿·괘선 래스터 차이는 남는다.
새 fresh PNG6개를 franchise-nested-frame-source-v453에 추가했고 실제 전142쪽 비교를 재개했다.

새 소스의 초기 PR 편집 입력24쌍은 Native/fresh 각27쪽 전쪽에서 최저92.07644%,
미달/누락0·Print 쪽수 일치·exit0/source guard=true다. 현재16개 review와 같은 좌표
확대4개를 직접 열었고, 전체27쪽의 Native/fresh 및 이전 b4 RGBA가 같다.
표/본문/끝내용/각주·구분선/편집색/빈누름틀 소속을 확인했고 글자획/장평/미세 자간과
각주 번호 크기 차이는 별도로 남는다. original-current-proof-v453에36개 PNG를 보존했다.

Native K-water27/27쪽 최저3쪽96.87092%, #157은2/2쪽 최저1쪽99.74326%로
각각 exit0·미달/누락0이다. 이는 fresh 전쪽 완료로 대신하지 않는다.
actual Studio Canvas2D K-water24/25쪽은99.94940/99.66933%다. 최신 pkg/Studio public과
실제 브라우저 응답 해시 일치, Canvas metric 활성, renderer fallback/pageError없음,
27쪽·exit0·21.817초·임시 서버 종료를 확인했다. 현재 두 review를 직접 열어 표 외곽/
내부 행/끝글자/다음 서식·그림 보존을 확인했고 작은 불릿/글자획/선 래스터 차이는 남는다.
kwater-canvas-current-proof-v453에4개 PNG 및 실제 호출/패키지 증거를 연결한다.

사용자의2026-10-13 추가 지시: **모든 테스트 완료 뒤 #7445 하위 이슈 중 이번 브랜치에서
전체 해결이 입증된 이슈는 PR 본문의 Closes에 포함한다.** live API로92개 하위 이슈와
후보 원문을 읽었고 파일명·입력 SHA·원본/독립 Print·완료 범위를 대조했다.
단순히 제목/문서 계열이 같다는 이유로 이슈를 종료하지 않는다.

| 이슈 | 원본 대응과 PR 종료 판정 |
| --- | --- |
| #7764 CGMP | 같은 원본48쪽의 직접 보정. 현재 Native 통과, 현재 Native/fresh48쪽 모두 통과(최저93.64014%)·직접 판독 완료. 최종 필수 회귀 완료 뒤 Closes 후보 |
| #7759 공개 OLE HWP 폰트 추적 | 같은 원본/Print1쪽 Native/fresh97.05704%이나 현재 직접 연 패널에 원본 HWP legacy plot/범례 배치 차이가 남고 counts/hash 핀도 이관 상태다. 사용자 수용한 현재 분리 OLE/97.79% 범위를 전체 해결로 확대하지 않으며 Refs 유지 |
| #7769 synam KPS | 원본 synam-001.hwp35쪽, 현재 검증한 kps-ai.hwp77쪽과 SHA가 다르다. AI 결과로 종료하지 않는다 |
| #7773 K-water | 원본 k-water-rfp-2024.hwp와2024 Print, 현재 HWPX와2020 Print 검증은 다른 입력이다. 동일 HWP 원본 검증 없이 종료하지 않는다 |
| #7807 SO-SUEOP | 현재 승인된1쪽 범위만 보정/회귀. 전체46쪽 종료로 확대하지 않는다 |
| #7822 고용 기준 | 이관된4쪽 원본의 새 전쪽 검증이 없다. 종료 대상으로 삼지 않는다 |
| #7445 대표 이슈 | 다른 미해결 하위 이슈가 있으므로 Refs 유지 |

현재 source의 필수 Rust13단계는 fresh 경계 판독/공용 Cargo idle 확인 뒤 시작했다.
Python/문서 변경 때문에 다시 실행하는 것이 아니며 실제 Rust 보정 ddf17f864의 새 검증이다.
정식 회귀 추가/기존 검사 보완은 시각 선행 조건 뒤 수행하고, 생산 소스가 그대로면
추가·변경 검사와 필수 lint/정책을 검증하며 같은 생산 소스의 전체 회귀 결과와 연결한다.
로그/완료 원문은 v453/final-rust-gates-result.json이다. 아직 최종 통과·승인·PR·merge는 주장하지 않는다.

현재 새 필수 lint/정책8단계는 모두 exit0이고 전체 nextest가 실제 진행 중이다.
OLE/SO10쌍20개 Native/fresh 비교도 모두 exit0/전체 source guard=true로 종료했다.
이는 #7807 전체46쪽이나 #7759의 남은 legacy 차이를 해결했다는 의미가 아니다.

종료된 초기 입력 SVG81개2,389,651,075bytes는 전쪽 자격·직접 판독·열린fd·해시 ledger 확인 뒤 제거했다.
입력/Print/PNG/TSV/tree/공용 target과 패키지는 보존한다. 원격 변경은 아직 수행하지 않았다.


## 진행 갱신218 — #7445 종료 범위와 현재 CGMP·KPS 양쪽 전쪽 검증 (2026-10-13)

사용자의 지시에 따라 최종 테스트 완료 후 #7445 하위 이슈 중 전체 해결을 입증한 항목을 PR 본문의 `Closes`로 등록한다. 현재 후보는 **#7764 CGMP**이며 같은 원본48쪽 Native/fresh 모두 최저93.64014%, 미달/누락0, 쪽수48/48 및 현재 대표 직접 판독을 확인했다. #7445 대표 이슈와 부분 개선 #7759/#7807은 Refs로 남기며, 다른 원본인 #7769/#7773을 현재 결과로 종료하지 않는다. 미게시 현재 PR 초안과 종료 guard는 v453에 준비했다.

KPS-AI77쪽도 현재 Native/fresh 모두 최저42쪽90.85010%, 미달/누락0,77/77이며 전77쪽 RGBA가 동일하다. 새 fresh42/68쪽 review를 직접 열어 줄 소속·내용 순서·뒤 내용·날짜/서명을 확인했고 획/굵기·미세 간격 잔여 차이는 별도로 남겼다. 현재 [CGMP·KPS fresh 증적](../assets/semanticist21-20261008/small-fresh-current-proof-v453/README.md)을 보존한다. production은 ddf17f864이고 소스 변경은 없다. 전체 nextest와 가맹사업142쪽 Native/fresh 및 후속 필수 검증은 계속 실행 중이므로 개별17개 승인과 PR 게시/merge는 아직 하지 않았다.


## 진행 갱신219 — PR 완료 범위 대조와 현재 Studio·남은 소형 문서 검증 (2026-10-13)

현재 production ddf17f864/WASM4299eb20의 Studio production build·frontend bindings·공개 계약6묶음·실제 #7566 누름틀 mouse activation은 모두 exit0다. 임시 서버 종료·package/source guard·실제 JS/WASM 응답을 확인했다. TypeScript/1834 PASS·2 SKIP의 frontend source는 직전 결과와 모두 같은 SHA로 확인해 재사용하고, 변경된 package 소비 경로만 현재 실행했다.

K-water HWPX27쪽과 #157 두쪽의 Native/fresh 전쪽 비교는 모두 exit0·쪽수 일치·미달/누락0이다. 최저값은 각각3쪽96.87092%,1쪽99.74326%다. 현재 K-water3/10/24/25쪽과 #157 1쪽 review를 직접 열어 본문·줄 소속·표 외곽·중첩 표·뒤 내용·서식/그림을 확인했다. 양쪽29쪽은 K-water10쪽 단일 래스터 픽셀을 제외하고 RGB 동일하며 그 차이도 기록했다. [현재 대표 증적](../assets/semanticist21-20261008/remaining-small-current-proof-v453/README.md)을 보존한다.

완료 범위 대조에서 현재 편집24개와 기존 초기48개 입력의 SHA 집합이 다름을 확인했다. 현재24개의 통과를 초기48개 통과로 대체하지 않는다. 기존48입력/독립Print의 SHA를 모두 확인하고 현재 Native/fresh 전체 비교를 실행 중이다. 그중2개 입력이 ignored capture 경로에 있으므로 같은 바이트의 게시 자산도 보완한다. 시험지/HML 원본59개는 현재 HEAD와 이전 원본 모두 일치함을 다시 확인했다.

전체 Rust nextest는 컴파일33분45초 뒤10,267개 실제 검사 실행 단계다. 16논리CPU·사용 가능 메모리 약12GiB에 비해 기존 CARGO_BUILD_JOBS=2가 낮았음을 확인했고 다음 검증 호출은6부터 메모리를 확인한다. 기존 공용target과 release-test 최적화 계약을 유지하며 현재 전체 실행은 재시작하지 않았다. 가맹사업 전142쪽 Native/fresh·초기48개 양쪽·최종 Rust 후속·정식 관계 회귀와 제출 head 검증이 아직 남아 있으므로 개별 판정은 완료 전이다. 새 PR 게시·승인·merge를 완료했다고 보고하지 않는다.


## 진행220 — 디스크 확장과 완료 검증 보존

- 현재 전체 nextest 로그 `final-full-nextest.log`: 10,267 passed, 35 slow, 45 skipped, exit0. 원PR 계약78개가 이 실행에 포함돼 통과했다. Skia lib도 exit0이며 뒤 검사 실패 원인은 ENOSPC 컴파일 환경이다.
- 같은 원본48개·52쪽 Native/fresh 모두 최저99.62848%, 쪽수 일치·미달/누락0. 현재 양쪽과 이전 v396 Native의 전52쪽 RGBA가 같다. [대표 review/overlay와 확대 판독](../assets/semanticist21-20261008/initial-48-current-proof-v453/README.md)을 커밋에 보존한다.
- 가맹사업 전142쪽 Native는 최저68쪽93.16816%, 미달·누락0. 이전 b4와118쪽만 다르며 보정 방향을 직접 확인했다. fresh 경계17쪽/72·118쪽 판독은 완료했고 전체 fresh는 재개했다.
- 디스크 `/dev/sda`250→300GiB, 파티션/PV/LV 및 ext4 루트248→298GiB를 재부팅 없이 확장했다. 가용52GB 확인. 파티션/LVM 백업과 실행 기록은 `v453/disk-expansion/`에 보존했다.
- 디스크 장애 당시 Rust10단계와 fresh4문서 exit0를 보존하고 중단된 범위만 재실행한다. 전체 회귀를 반복하지 않는다. 결과 JSON의 환경 중단과 실제 시험 실패를 구분한다.
- TSV 제출 압축 의무는 `.github/pull_request_template.md`와 Visual Sweep 가이드에서 제거했다(`386fdc0cb`). Native/fresh TSV 원본 링크와 대표 PNG 의무는 유지한다.
- 현재 승인·PR·merge는 아직 실행 전이다. 생산 소스 변경 없이 남은 필수 게이트를 완료한 뒤 사용자 자동 승인 범위에서 계속한다.


## 진행221 — Rust13 완료와 원PR 후속 증적 준비

- Rust13단계 최종 JSON은 `v453/final-rust-gates-result.json`의 terminal/all_pass=true다. 디스크 확장 후 placeholder2·직접PDF4·doctest8 검사도 실제 exit0이며, 이미 통과한 전체10,267개를 반복하지 않았다.
- [초기 원본 현재 증적](../assets/semanticist21-20261008/initial-48-current-proof-v453/README.md)에 원PR #7613/#7616/#7658/#7614/#7654의 대표 양쪽10review를 보완했다. 연속 표·줄 소속·탭 간격·캐럿 뒤 문자와 주석·빈 미주 소속을 직접 확인했으며 각 대표100%다. 전52쪽 최저99.62848%와 구분한다.
- 기존 #6718 검사에서8px 공차를 제거하고 원본 셀 소유/내용 보존을 추가한 ignored 초안1개는 현재 실제 검사에서 PASS했다. 새 프레임 관계 초안4개와 함께 시각 조건 충족 후 정식 등록한다. 초안 통과를 활성 회귀 등록 완료로 보고하지 않는다.
- 선택 경로는 `collaborator_external_pr`9.1.1의 체리픽 통합이다. 개별 원PR 문서18개를 같은 통합 PR에 유지하고, 채번된 통합 PR용 별도 review 파일은 만들지 않는다. CI 녹색 뒤 개별 기록/오늘할일 trailing 변경만 추가한다.
- 원PR 후속 계획은 초기 수용 source만 대상으로 한다. #7615의 초기ed9c 이후 #7662 변경은 이번 적용에서 제외했고 후속6개는 읽기 전용이다. 통합이 병합되기 전에는 댓글·원PR close·#7764 종료를 수행하지 않는다.


## 진행222 — 내장 글꼴 중복 검사 단축

- 사용자 요청으로 Python 검사 도구만 보정했다. `visual_sweep_font_cmap.py`는 @font-face 규칙 전체의 SHA-256으로 성공 결과를 최대128개 재사용한다. 캐시에 원문·글꼴 바이트·실패 결과를 보관하지 않으며 같은 family라도 헤더·payload가 바뀌면 다시 검사한다. 디코딩한 바이트 SHA의 중복 계산도 제거했다.
- `visual_sweep.py`는 기존과 같은 평면 규칙을 추출하는 보조 함수를 호출하고 글꼴 검사 진행 쪽수를 출력한다. Unicode 공백·대소문자·중첩/미완성 괄호의 규칙 선택을 기존 정규식과 대조했다. 손상 글꼴·없는/빈 cmap·엄격한 base64 검증과 실패 보고는 유지한다.
- 관련 Python117개 PASS(27.495초), 최종 focused8개 PASS다. 실제205,024,100바이트 SVG의 기존9.953초·보정 cold9.349초·반복3.735초를 구분한다. 단일 쪽의 cold 측정을 다쪽 HWP 전체 단축으로 확대하지 않는다.
- 같은 가맹사업 HWP의142쪽 SVG와920개 글꼴 항목을 모두 다시 검사해 기존 보고서와 정확히 일치했다. 기존 export 완료 표식→글꼴 보고서 파일시각 구간은1275.065초, 보정 검사 실측은590.064초다. 글꼴 구간 절감은685.002초(53.72%)이며 변경 후 전체 export·PNG·TSV 총시간의 실측과 구분한다. 증거는 `v453/font-optimization-comparison.json`, `embedded-font-optimized-whole-result.json`, 두 테스트 로그다.
- 생산 Rust·WASM 패키지와 승인된 source guard는 그대로다. 전체 Rust10,267개를 재실행하지 않았다. 이미 진행 중인 fresh142쪽 실행은 기존 checker의 통과 보고서로 PNG 비교를 계속하며 중단하거나 SVG를 재생성하지 않았다. 전쪽 fresh와 정식 관계5개·마지막 lint/정책은 아직 완료 전이다.

## 진행223 — 가맹사업 전쪽 완료와 Chrome 재사용·Windows 검증

- 현재 가맹사업 전142쪽 Native/fresh는 모두 최저68쪽93.16816%, 미달/누락0, 독립 Print 쪽수142/142다. 전142쪽 RGBA가 같으며 60/68/118쪽 양쪽 review6개를 직접 열어 셀/본문 소속·내용 보존·중첩 시작/끝과 후속 상자를 대조했다. 68쪽의 인용문 밑줄/획 표현 차이는 남는다. [최종 전쪽 대표 증적](../assets/semanticist21-20261008/franchise-whole-dual-current-proof-v453/README.md)에 입력·PDF·명령·판독을 연결했다.
- SVG의 내장 글꼴을 같은 바이트의 공용 파일로 먼저 바꾸고 규칙을 탐색하도록 순서를 보정했다. 실제1/68/118쪽의 전후 PNG와 글꼴 메타데이터가 같고 총63.55→21.77초다. 진행 중이던 전쪽 실행은63쪽 완료 뒤 새 Node 호출부터 적용했고 SVG를 다시 만들지 않았다. 최종 전142쪽 Native/fresh 동일성이 확인됐다.
- 후속 다쪽 캡처는 문서당 Chrome 하나를 재사용한다. 시작의 빈 탭을 사용하고 각 쪽 캡처 뒤 탭을 닫아 다음 쪽을 연다. 쪽 완료 JSON과 기존 분석/checkpoint 순서는 유지한다. 실제4입력 비교는 같은 browser PID·최대1탭·완료 뒤0탭·브라우저 종료 및 PNG/글꼴 기록 동일을 확인했고23.38초였다. 처리 순서 보정 후의 개별 호출30.47초와 구분하며, 문서 전체 실행시간으로 확대하지 않는다. 기존 전142쪽 실행의 Python은 이 배치 경로를 사용하지 않았다.
- Python117개·Node7개는 통과했고 새 프로토콜/실제 Chrome 계약4개와 실제 Python bridge2쪽도 통과했다. Snap Chrome이 `/tmp` 입력을 읽지 못한 환경 실패는 저장소 아래 TMPDIR로 재실행해 해소했다. Windows 중단 시 강제 terminate가 정리 코드를 건너뛰는 문제는 stdin 종료 요청으로 보정했다. wrapper 제거에는 force를 지원하는 rmSync를 사용한다.
- `win10-ted`의 Windows/Python3.12.10/Node24.19.0/설치 Chrome에서 글꼴8개·캡처2개·프로토콜2개가 통과했다. 한글/공백 경로와 실제 중단 종료143, Chrome 종료·임시 profile/HTML 제거·최대1탭을 확인했다. Unicode 공백 입력 테스트에는 UTF-8을 명시해 cp949 실패를 고쳤다. 기본 Windows 저장소는 변경하지 않았고 별도 probe만 사용 후 원격에서 삭제했다. Linux 후속 글꼴8개·계약4개·Node7개도 통과했다. 증거는 v453의 raster-cross-platform 로그, windows-raster-contract-tests.log, windows-raster-probe-result.log 및 source/probe 해시 원장이다.
- 생산 Rust/WASM은 ddf17f864와 동일하다. 이미 통과한 전체10,267개는 반복하지 않는다. 전쪽 선행 조건이 충족됐으므로 다음 단계에서 새 관계4개와 기존 #6718 관계1개를 정식 등록하고 해당 focused 검사 및 필수 lint/정책을 마무리한다. 개별17개 최종 승인과 원격 PR 게시/통합은 그 종료 결과 뒤에 진행한다.

## 진행224 — 관계 회귀 정식 실행과 실제 base 전진

- 전142쪽 Native/fresh 최저93.16816%·쪽수 일치·직접 판독 뒤 `tests/cases/stored_partial_frame_alignment.rs`의 관계4개와 기존 `issue_6718_zero_rewind_in_split_paragraph.rs`의 본문/셀 소유·내용 순서·포함 검사1개를 정식 등록했다. 실제 nextest는4 PASS와1 PASS였다. 기존 검사의8px 공차를 제거했으며 내용이 비어 포함 검사를 통과하는 상황도 막는다.
- 새 정식4개는 같은 컴파일된 test binary와 각 보정 전 CLI를 사용해 모두 해당 assertion에서 FAIL(exit101)했다. CLI와 test binary 해시 및 정확한 함수·로그는 `v453/formal-relations-final-result.json`에 연결한다. #6718 변경1개는 현재 PASS 증거이며 새 결함의 before FAIL이라고 주장하지 않는다.
- 첫 prepare 뒤 fmt가 새 source의 바이트 수를 바꿔 suite 배정이 달라졌고 첫 focused 실행은0 tests/exit4였다. 이를 통과로 세지 않고 fmt 후 prepare를 다시 실행해 실제4/1개를 확인했다. nextest가 `CARGO_BIN_EXE_rhwp`를 자체 주입하므로 부모 환경의 historical CLI가 적용되지 않았던 시도도 before 검출 증거에서 제외했다. 최종 before4개는 같은 정식 binary를 직접 실행해 실제 결함을 검출했다.
- 원PR17개는 아직 OPEN이고 #7612는 이미 MERGED, 후속6개는 그대로 미적용이다. 원본 TSV176개는 원 바이트·SHA·범위·독립Print를 대조해 로컬 제출 payload를 준비했다. 경계17쪽은 완료된 silhouette manifest의 명시적 쪽 범위를 쓰며 기존 capture manifest의 incomplete 상태와 구분한다. 아직 gist/PR을 게시하지 않았다.
- 실제 원격 `upstream/devel`이552f5228d에서22317614b03722e95349a1d3d71c86923363c6f5로 전진했다. #7877의 표 분할·저장 원점/흐름 소유 및 #7639 성능 변경106파일을 확인했다. 현재 최종 통합 preview는22파일72충돌이며 공유 helper·height 소비 경로가 겹친다. 이 단계의 code/test를 커밋한 뒤 실제 rebase를 수행한다. source SHA가 바뀐 결과에는 이전 전체/시각 완료를 그대로 적용하지 않으며 영향 경계·원 PR 입력의 Native/fresh와 필수 lint/정책을 새 head에서 확인한다. 통과한 이전 전체10,267개는 보존하고 불필요하게 반복하지 않는다.

## 진행225 — #7877 리베이스 완료와 우선 회귀 대조

- 실제 변경된 base `22317614b03722e95349a1d3d71c86923363c6f5` 위의 rebase228단계를 완료했다. 완료 head는 `110e4749e1dd3ac3b312510b5bcb9588a2a7106d`이고, 이전 head는 `backup/semanticist21-pre-223176-20261013`으로 보존했다. 각 충돌의 원래 commit·경로·해결 근거는 `v453/upstream-223176-rebase-resolutions.jsonl`에 기록했다.
- upstream의 typed `StraddleCellCarry`·물리 예산·후속 저장 원점·중첩 종료 띠 소유를 유지하면서 현재 보정의 원본 행 상자·RowCut 인덱스·OLE 호스트 원점·정렬 출처를 함께 전달한다. compile에서 확인한 중복 임시 helper와 overlay의 새 carry 인자 누락을 수정했다. 이 변경의 현재 PASS를 아직 선언하지 않는다.
- 사용자 지시에 따라 #7877에서 변경한 integration10파일45개를 가장 먼저 실행한다. source 물리 예산 검사1개도 별도로 확인한다. 다른 전체 회귀를 먼저 반복하지 않는다. current/source 입력 SHA와 원격 #7877 metadata, 실행 명령·결과는 `output/pr-review/semanticist21-20261008/upstream-7877-rebase-v454/`에 보존한다.
- #6972의 같은 높이 합성 검사는 삭제되지 않았으며 기존 메인터너 보정에서 `ambiguous_same_height_lines_keep_raw_control_slot_ownership`으로 이름과 기대값이 바뀌었다. 원시 확장 제어의8유닛 소속을 보존하는 계약이며, upstream의 글자 위치 projection 기대값과 의미가 다름을 명시한다. 다른44개 함수 이름은 그대로 보존되어 있다.
- #7877이 giant-overfill 진단 양성 fixture를 정상 검증된 `issue2004_cell_image_stack.hwpx`로 교체한7개 검사는 유지했다. 과거 giant 입력의 검사 제외 기록을 이 정상 대체 fixture의 삭제 근거로 쓰지 않는다. 원본 시험지 파일과 범위 밖 검사는 보존한다.
- 새 source의 우선 회귀·Native/fresh 시각 대조·lint와 원격 CI가 남아 있다. v453의10,267 PASS 및 이전 시각 증적은 이전 source의 역사 기록이며 새 통합 head의 통과 증거로 승격하지 않는다.

## 진행226 — #7877 우선 회귀46개 완료

- #7877에서 변경한 integration10파일45개를 현재 소스에서 실행해 **45 PASS/0 FAIL**을 확인했다. 각 실제 실행 함수와 inventory를 대조해45/45개 일치·누락0을 확인했다. 기존 물리 예산 source 검사 `test_advance_row_cut_rowbreak_true_tail_obeys_physical_budget`도 별도로 **1 PASS**였다. 대상46개는 SKIP0이며, nextest의 다른 함수 필터/기존 ignore 수를 이 대상의 생략으로 세지 않는다.
- integration compile은8분50초, 검사 실행은3.509초였다. source1개의 warm 검사 실행은0.052초였다. 처음 JSON 집계에 필터 밖 ignore5개가 포함되어집계 assertion이 실패했고, 실제 PASS/FAIL 이벤트와 대상 함수의 정확한 목록으로 수정해 확인했다. 해당 집계 오류로 Rust 검사를 반복하지 않았다.
- 다음 빌드부터 사용자 지시대로 `CARGO_BUILD_JOBS=8`을 사용한다. 실제 진단 시 사용가능 메모리 약12GiB, active swap-in/out0, I/O wait0–1%여서 메모리·디스크 압박은 확인되지 않았다. 마지막 source libtest의 `lto cgu.00` 작업이 관측됐다. `lto=false`에도 local Thin LTO가 수행되므로 jobs 증가만으로 이 단일 crate의 마지막 최적화 구간을 단축한다고 주장하지 않는다. 현재 프로필·toolchain은 변경하지 않았다.
- 입력/PDF208개 항목·고유205파일 모두 기존 입력 SHA 및 현재 commit 바이트와 일치했다. 원PR17개의 기능 commit39개는 새 head의 조상이며37개는 stable patch-id가 같고2개는 upstream 변경 문맥 차이를 별도 range-diff로 보존했다. 후속6개는 적용하지 않았다.
- 완료 증거는 `v454/pr7877-priority-result.json`, `pr7877-priority-nextest.log`, `pr7877-source-budget-nextest.log`, `current-input-pdf-audit.json`, `original-application-rebased-audit.json`이다. #7877 회귀 우선 검사는 완료됐고, 현재 source의 나머지 focused/Native·fresh WASM·필수 lint·원격 CI는 계속 진행한다. 전체10,267개를 다시 실행한 결과로 보고하지 않는다.

## 진행227 — 리베이스 뒤 가맹사업 정렬 회귀 두 곳의 직접 확인

- `d9befb30381a7c29ce7cc17e37510ef910351d48`에서 초기 PR/관계 focused83개는81 PASS/2 FAIL이었다. 실패는 `stored_partial_frame_alignment`의 완전한 가운데 정렬 셀과 중첩 시작 문단 관계다. 이 문서는 현재 메인터너 보정 대상이며 검사를 이관하거나 허용치를 완화하지 않는다.
- 같은 가맹사업 원본·독립 한컴 Print로 Native 경계17쪽을 재출력했다. 55쪽99.90891%는 이전 Native/fresh99.99889%보다0.08998%p 낮다. 빈 문단 뒤 본문이 이전보다2.6667px 위로 이동했다. 118쪽89.30514%는90% gate 미달이며, 하단 중첩 표의 첫 문단이 이전보다8px 위로 이동했다. 두 review/확대 PNG를 직접 열고 사용자에게 표시했다. `v454/franchise-failure-visual.json`, `native-boundary-result.json`, `native-boundary/franchise5966/review/`에 연결한다.
- 첫 원인 경로는 `stored_empty_successor_advance`의 저장 줄 원점 차이 → partial/whole의 문단 앞 간격 적용 전 커서 덮어쓰기다. 현재 문단의 흐름 앞 간격은 측정에 있지만 덮어쓰기에서 빠진다. 공통 helper가 현재 문단의 앞 간격을 포함한 전진량을 반환하도록 보정한다. 첫 원본 문단의 저장 띠는 흐름 앞 간격과 구분하며 기존 zero-origin 계약을 유지한다.
- 둘째 원인은 partial 셀의 첫 저장 줄 스냅에서 Center/Bottom의 앞 띠까지 선차감하도록 범위가 바뀐 것이다. 가맹사업 첫 source 줄은 그 띠를 선택 문단 정렬에 소유하며, 선언 상자와 잉크 끝이 일치하는 #7877 인증 프레임과 다르다. 선언 프레임은 새 계약을 유지하고 독립 문단 정렬은 기존 소유를 복구한다. 측정/실제 스냅/최종 문단 앞 간격 소비를 함께 대조한다.
- 같은 head의 초기48개 Native52쪽·편집24개 Native27쪽은 전쪽90% 이상·쪽수 일치였다. 필수 lint/정책7단계도 통과했다. 이 결과는 보정 전 증거로 보존한다. 보정 뒤 관련83개/#7877의46개, 영향 시각 범위와 필수 lint를 새 source에서 확인하며, 로컬 전체10,267개는 반복하지 않는다. 개별 PR 최종 승인·원격 PR은 아직 보류다.

## 진행228 — 가맹사업 정렬 복구와 전 문서 리베이스 전후 하락 검사

- 보정 head `558eb66085f1437b5cce1f0b7c42ee9aaa2e2b74`에서 원PR/관계/#7877 focused129개가 실제129 PASS/누락0이었다. Native/WASM/all-targets Clippy·fmt·workspace build·base223176 정책 검사를 포함한 필수7단계도 통과했다. 전체10,267개는 다시 실행하지 않았다.
- 가맹사업 Native 경계17쪽은 모두 이전 v453의 RGBA와 같다. 55쪽99.99889%·118쪽99.99852%로 복구됐고, 두 쪽의 전체/확대 review를 직접 열어 셀 소속·문단 앞 간격·뒤 내용을 확인했다. 현재 fresh 패키지는 루트 공용 target에서 새로 빌드해 pkg/Studio public SHA 일치를 확인했으며, fresh 경계17쪽은 계속 비교 중이다.
- 사용자 요청으로 리베이스 전 production `ddf17f86476293b114b6236bfa6cf1ebc9d9d474`와 현재의 전체 범위174 TSV·770쪽/backend를 페이지별로 비교한다. 같은 입력/PDF SHA·글꼴·DPI·출력 계약·쪽수와 완료 manifest를 대조하며, 평균이나 최저값으로 개별 하락을 숨기지 않는다. 현재 완료233쪽/backend는228동일·2상승·3하락이고 나머지는 미검증이다.
- 초기48사례 Native52쪽 및 편집24사례 Native27쪽은 이전 점수와 RGBA가 모두 동일하다. CGMP48쪽, K-water27쪽, #157 2쪽 Native도 점수 하락 없이 쪽수 일치·미달/누락0이다. KPS Native77쪽에서는12쪽99.63501→98.54681%,13쪽94.29582→76.41154%,14쪽99.55754→71.34227% 하락을 검출했다. 47쪽99.23336→99.49643%,60쪽99.48664→99.52969% 상승은 그대로 수용한다.
- KPS12~14쪽 review를 생성하고13/14쪽을 직접 열었다. 상세 장 제목과 기능 제목이 앞 쪽 끝에 남으며 해당 표만 다음 쪽으로 넘어간다. 리베이스 전후 dump-pages·원본 LineSeg를 대조했다. TAC 종료 뒤 좌표 축을 닫는 동작은 보존하되, 완전한 원본 개체 줄과 새 제목/후속 줄 사다리가 입증하는 저장 쪽 경계는 별도로 보존해야 한다. `저장 개체 상자 → 쪽 경계 판별 → advance_column_or_new_page → 문단 fit → PageItem 쪽 소유 → 실제 배치`를 같은 경로로 확인한다.
- 사용자 요청에 따라 KPS 제목/표의 같은 쪽 소속·순서와 앞 표 뒤 새 장의 쪽 경계를 검사하는 관계 초안2개를 ignored output에 준비했다. 같은 draft binary와 frozen CLI를 직접 실행해 이전 v453는2 PASS, 회귀 head558은 두 의도한 assertion에서2 FAIL(exit101)이다. 픽셀 공차나 절대 위치로 기존 출력을 고정하지 않는다. 현재 Native/fresh 전77쪽90%와 확대 판독 후 정식 등록하며, 검출된 회귀를 이관하거나 기대값을 완화하지 않는다.
- 실행·비교·입력/패키지 해시·검사 초안의 증거는 `output/pr-review/semanticist21-20261008/franchise-rebase-alignment-v455/`에 보존한다. fresh 실행의 이전 frozen CLI 파일명 잔재로 발생한 첫 실행 실패는 경로를 바로잡아 재실행 중이고, 실패 로그를 보존했다. 최신 전체 시각 비교와 KPS 보정이 완료되기 전에는 개별 승인·원격 PR·merge를 진행하지 않는다.

## 진행229 — KPS의 독립 저장 쪽 경계 보정

- 가맹사업 head558 fresh 경계17쪽도 exit0·최저68쪽93.16816%·미달/누락0이며, 이전 fresh와 현재 Native의 RGBA와 모두 같다. KPS 회귀를 발견했으므로 미실행 중이던 head558의 후속 큐는 다음 source에서 검증하도록 종료했다. 현재 진행 중이던 경계 캡처와 완료 결과는 보존했다.
- `stored_page_origin::stored_inline_frame_restarts_page`가 완전한 TAC 줄의 높이=원본 표 높이+바깥 여백과 새 제목의0 원점/다음 줄의 정확한 저장 전진을 함께 확인한다. `apply_stored_paragraph_boundary`는 바로 앞 완전한 Table 항목이 현재 단을 소유할 때만 이 쪽 경계를 수용한다. dirty/합성/문단별 로컬0/부분 이어받기를 제외하고, TAC 좌표 축 종료 및 #7877 물리 컷 계약은 그대로 소비한다.
- 이 함수의 결과는 문단 fit 전에 단/쪽 전환에 사용되고, 그 결과의 PageItem 소속을 실제 배치가 소비한다. 표 height나 화면 좌표를 clamp하지 않으며, 임시4px 안전마진을 되살리지 않는다. 원본 제목/표 소속을 검사하는 관계 초안2개는 아직 정식 경로에 넣지 않았다.
- 다음 검증은 현재 보정 source의 KPS 영향 쪽·초안2개/#7877·기존 관계 검사·필수 lint와 전 문서 Native/fresh 리베이스 전후 비교다. 이 단계는 구현과 검증 계획이며 아직 보정 후 PASS로 보고하지 않는다.

## 진행230 — KPS 전77쪽 검증과 정식 관계형 검사2개 완료 (2026-10-14)

- 생산 source `6995bc5d35b14f598e03cf0f5cc99bd957a42843`, base `22317614b03722e95349a1d3d71c86923363c6f5`에서 KPS Native/fresh 전77쪽은 모두 최저42쪽90.85010%·미달/누락0·독립 Print 쪽수77/77이다. 리베이스 전보다 낮은 쪽은 없고47/60쪽 개선을 수용한다. fresh13/14/22 review를 직접 열어 제목과 표의 같은 쪽 소속·본문 순서·셀 포함을 확인했다. Native와 fresh76쪽 RGBA가 같고22쪽 라벨의 raster 차이는 따로 판독했다.
- 선행 검증 뒤 `tests/cases/kps_heading_page_ownership.rs`에 최종 render tree의 제목/표 쪽 소속·순서·제목 내용 보존 관계2개를 등록했다. 절대 쪽 번호·px 좌표·SVG 해시를 검사하지 않는다. 같은 정식 test binary와 frozen CLI를 실행해 리베이스 전2 PASS → 회귀558의 소속 assertion2 FAIL(exit101) → 보정699의2 PASS를 입증했다. `formal-kps-before-after-result.json`에 연결한다.
- 새2개와 기존 중요 문서·원PR/#7877 계약을 포함한 focused146개가 실제146 PASS·누락/예상 밖 실행0이었다. 초기 fmt는 새 cases 파일을 generated suite에 등록하기 전에 실행돼 새 파일을 발견하지 못했고, 이후 fmt-check가 실패했다. generated 경로를 준비한 뒤 다시 fmt했으며 canonical rustfmt 바이트가 같아 검사 의미가 바뀌지 않았음을 확인했다. 전체10,267개를 다시 실행하지 않았다. 필수 lint7단계는 별도로 계속 진행한다.
- Native 직접 문서5종은 KPS77/CGMP48/K-water27/#157 2/가맹사업142쪽 모두 미달/누락0이다. 최초48입력52쪽·편집24입력27쪽도 Native 통과했다. Native batch 부모는 exit143으로 중단됐지만 실행 중이던 whole 자식이 계속 실행해5종을 모두 exit0으로 완료했다. 이를 부모 exit0으로 바꾸어 보고하지 않는다. 실제 child 결과·완료 manifest·source guard로 범위를 재대조하며 이미 완료된 출력을 반복 생성하지 않는다.
- 현재 source의 Studio4단계와 실제 K-water Canvas24/25 소비 검사가 완료됐다. 최신 원격 devel은 여전히223176이어서 추가 rebase를 하지 않았다. 초기18개 중17개 OPEN/7612 MERGED, 후속6개는 미적용으로 유지한다. 전체174 TSV·770쪽/backend와 나머지 fresh·lint가 완료되기 전 개별 승인·원격 PR은 아직 진행하지 않는다. 증거는 `output/pr-review/semanticist21-20261008/kps-source-page-boundary-v456/`다.

## 진행231 — Visual Sweep 작업자 옵션과 실제77쪽 성능 대조 (2026-10-14)

- `--page-workers N`을 추가했고 CLI 기본값은4, 1은 순차 실행이다. 글꼴 검사와 점수 계산은 Windows/Linux 공통 spawn 프로세스에서, PNG는 별도 Node 작업자별 Chrome 하나·탭 하나로 수행한다. 원래 쪽 순서/checkpoint와 worker 소유를 보존하고 종료/중단 시 이 실행의 자식·profile·임시 manifest만 정리한다. 원문 내보내기·PDF 래스터화·일반 상세 overlay는 순차 구간이다. [사용법과 실측](../../manual/verification/visual_sweep_guide.md#쪽-작업자-수와-chrome-캡처-재사용)에 기록했다.
- 같은 `samples/kps-ai.hwp`·독립 `pdf/kps-ai-2022.pdf`·생산699의 frozen CLI·글꼴·96dpi로 전77쪽을 실제 실행했다. 작업자1개382.79371초 →4개147.70903초, **235.08468초/61.41289% 단축·2.59154배**였다. 원문 export를 포함하지만 Rust/WASM 빌드 시간은 포함하지 않는다. 다른 검증이 실행 중인16 논리 CPU/15GiB 호스트의 관측이며 일반적인 문서 전체 단축률로 확대하지 않는다.
- 실제1/4 실행의77 PNG RGBA, TSV 원 바이트, 글꼴 내용 보고서가 모두 동일했다. 최저42쪽90.85010%·쪽수77/77·미달/누락0이고 각 Chrome의 동시 탭은1이었다. 별도 과거 Native 캡처와는22쪽 라벨 raster가 다르나 SVG 원 바이트와 render tree는 같고 점수도 같다. 현재1/4 출력은 현재 fresh22와도 같은 RGBA다. raw benchmark의 이 추가 역사 비교 실패를 보존하고 `actual-file-benchmark-reviewed.json`에서 실제 병렬화 비교와 과거 raster 진단을 분리했다. 과거77 PNG까지 모두 같다고 주장하지 않는다.
- 기존 Python119개·Node7개, 새 spawn/실제 Chrome/픽셀/순서 검사6개가 통과했다. Linux Snap Chrome의 private `/tmp` 때문에 첫 실제 캡처가 실패해 작업용 output 아래 TMPDIR로 다시 실행했고 실패 로그를 보존했다. Windows는 기본 Python의 Pillow/fonttools 미설치 실패 뒤 저장소 venv로 실행해16 PASS 및4개 작업자 중단/한글·공백 경로/브라우저·profile·wrapper·manifest 정리를 확인했다. Windows 임시 검증 외곽 디렉터리와 archive 삭제는 후속 SSH가 exit255로 돌아와 미완료로 기록한다.
- 원문 SHA/Native/fresh 패키지는 변경하지 않았고 전체 Rust10,267개를 반복하지 않았다. source1395개와 frozen CLI/패키지 해시의 동일성을 유지한 채 나머지 문서 검증은 새4작업자 옵션을 사용한다. 증거·명령·도구 SHA·원본 로그는 `output/pr-review/semanticist21-20261008/visual-sweep-page-workers-v457/`에 보존한다. 필수 lint와 전770쪽/backend 완료 후 개별 PR 최종 판정·통합 PR·CI·merge·후속 처리를 진행한다.


## 진행 갱신232 — 최종 로컬 검증과 제출 증적 완료 (2026-10-14)

현재 브랜치는 `review/semanticist21-20261008`, 조판 생산 소스는 `6995bc5d35b14f598e03cf0f5cc99bd957a42843`, 고정 정책 base는 `22317614b03722e95349a1d3d71c86923363c6f5`다. fetch 뒤 base SHA가 같아 불필요한 rebase를 하지 않았다. 현재 소스의1395개 production/Cargo 파일 및 공용 pkg/Studio JS·WASM 바이트를 검증 자료와 대조했다.

| 실행 범위 | 실제 결과와 증거 |
| --- | --- |
| 집중 Rust 회귀 | 146 PASS·누락/예상 밖0. 원 PR 계약 및 #7877, 현재 중요 문서 관계, 새 KPS2개 포함. `v456/focused-result.json` |
| KPS 새 정식 회귀의 검출 | 실제 release-test binary·같은 CLI 원본으로 rebase 전2 PASS →558 회귀2 assertion FAIL →699 보정2 PASS. 단순 helper 계산값이 아닌 최종77쪽 RenderTree의 소유·순서·보존 관계. `formal-kps-before-after-result.json` |
| 필수 lint·정책 | fmt, Native/WASM32 Clippy, workspace build, all-target Clippy, base 고정 manifest/unit-tier 모두 exit0. KPS 소스 fmt 뒤 shard weight가 바뀐 파생 목록을 정식 prepare로 갱신하고 fmt·policy·all-target 재확인. 실패 원문 보존, 생성28개 harness는 PR에 stage하지 않음 |
| 전체 시각 비교 | 174 TSV /770쪽/backend, 전쪽90% 이상·누락/쪽수 차이0, 리베이스 전 대비762 동일·8 개선·하락0. 원본/Print/공급 글꼴·DPI provenance 차이0. `before-rebase-comparison-result.json` |
| 변경 부위 판독 | 이전 대비759 PNG 전체 RGBA 동일,11쪽 변경. 현재 양 backend385쌍 중382 동일; 다른3쌍은 KPS22 및 K-water3/10의 기록된 글꼴 raster·1px 부위를 개별 판독. 가맹사업50/119는 Native 판독 후 fresh RGBA 동일성으로 연결. `final-current-direct-reading-qualification.json` |
| 직접 문서 전쪽 Native/fresh | CGMP48쪽 최저47쪽93.64014%, KPS77쪽 최저42쪽90.85010%, K-water27쪽 최저3쪽96.87092%, #157 2쪽 최저1쪽99.74326%, 가맹사업142쪽 최저68쪽93.16816%; 양쪽 미달·누락0 |
| Studio·공개 소비 | 현재 공용 fresh 패키지로 production build, bindings, 공개 계약6종, 실제 누름틀 클릭 및 K-water Canvas24/25 완료·exit0·source/package guard 유지. 기존 TypeScript/unit1834 PASS·2 SKIP은 변경 없는 source 범위의 기록으로만 재사용 |
| Visual Sweep 도구 | 기존 Python119·Node7, 새 spawn/실제 Chrome6 PASS; Windows 실제16 PASS와4worker 중단·한글/공백·최대1탭·소유 browser/profile/wrapper/manifest 정리 PASS. Windows probe directory/archive 없음 및 owned process0을 재접속 후 실제 확인 |
| 성능 비교 | 같은 KPS77쪽 전체 pipeline1worker382.794초 →4worker147.709초,235.085초(61.41%) 단축·2.59배. 전체77 PNG RGBA·TSV 바이트·글꼴 검사 결과 동일. 과거 캡처22쪽의 raster 차이는 별도 진단으로 보존하며 이 실험의1/4 결과와 혼동하지 않음 |

전체 로컬10,267개는 반복하지 않았다. 기존 ddf17f864의10,267 PASS·45 SKIP·Skia4041 PASS 등의 실행은 역사 소스에 연결하며, 현재 추가 생산 보정699는 focused·fresh WASM·전쪽·필수 lint와 통합 PR exact-head CI로 검증한다. Native batch parent가143으로 끝난 사실은 `native-current-batch-reconciliation.json`에 보존했고 실제 완료한 child5종 exit0·296쪽 및 선행 초기/편집 완료 결과로 판단했다. parent를 exit0으로 정정하지 않았다.

TSV 원문 게시 및 재조회 확인: [Native/fresh TSV174개](https://gist.github.com/jangster77/1efd96731b845480bc1b8f38344d6d6e). 같은 원문의SHA와 입력·Print·쪽 범위는 `manifest.json`이다. 압축 파일 없이 원문을 제출하며, JSON/로그/TSV 작업 파일은 ignored output에 보존한다. 현재 대표 PNG는 [현재 생산 소스 증적](../assets/semanticist21-20261008/current-source-page-proof-v456/README.md)에 연결한다. 회귀 이관 증적 `.rs.txt`의 불필요한 마지막 빈 줄1개도 제거해 제출 whitespace gate를 정리했다.

Closes 대상은 동일 CGMP 전체48쪽으로 확인한 #7764다. 대표#7445 및 #7759/#7807/#4514는 Refs로 남긴다. #7769의 synam-001, #7773의2024 HWP는 현재 KPS/K-water 입력과 다르므로 종료하지 않는다. 후속6개 및 #7615의 초기 수용 이후 #7662 의존 변경은 적용하지 않았다. 현재 원격 PR·CI·merge는 다음 순차 단계이며 이미 완료한 것으로 보고하지 않는다.


## 진행 갱신233 — Render Diff 계약 검사 실패 보정과 실제 로컬 재검증 (2026-10-14)

통합 PR#7882의 첫 code candidate `b318a881d71ffa3f401beb12d3b2e06ab876680c`에서 [Render Diff job115401467632](https://github.com/edwardkim/rhwp/actions/runs/38427681153/job/115401467632?pr=7882)가 실패했다. WASM/Native CLI 빌드는 성공했으며 화면 비교 전에 `npm run e2e:renderer-contract`의 테스트용 CanvasKit 모형에서 `ColorType.RGBA_8888`을 읽지 못했다. 이 단계 실패를 문서 출력 회귀 또는 CI 통과로 보고하지 않는다.

- 테스트용 이미지 API·enum·shader 생성/해제를 별도21줄 helper에 구현했다. 실제 렌더러를 실행해 직렬화한 무늬/배경색이 이미지에 있고 shader가 원래 도형에 그려지는지, Repeat/Linear API·행 stride·이미지 길이·그리기 이후 자원 해제를 검사한다. 실제 렌더러나 출력 공차를 바꾸지 않았다.
- 이어 드러난 각주 baseline의 옛 source-regex `op.fontSize ??7` 핀은32줄 helper의 실제 `renderOp → renderTextRun` 소비 검사로 교체했다. 저장 기준선 및 유효0의 보존, 누락 시 기존 fontSize/기본값 fallback, bbox/본문/색 보존을 확인했다. 같은 실제 모듈에서 기준선 보정을 진단용으로 제거하면 의도한 assertion FAIL(10 대4), 현재 모듈은 PASS다. 합성 API 계약이며 실물 문서의 px 배치를 고정하지 않는다.
- manifest에서 `hwpctl-api`는 이미 `bce6b28088b684f79aa4f1d514bda4453ac6b05f`로 #7445/#7731에 이관됐지만 계약은 대표 표본23개를 기대했다. 남은22개로 정확히 맞추고 해당 이관 ID가 활성 목록에 없음을 추가 확인했다. 다른 표본을 지우거나 허용치를 완화하지 않았다.
- 로컬 수정 전 동일 TypeError/exit1을 재현했고 보정 뒤 `e2e:renderer-contract` exit0을 확인했다. CI의 해당 step 전체를 그대로 실행해 모든 Node syntax 검사, Native/CanvasKit provenance self-test, renderer 계약 및 `e2e:canvaskit-font-coverage`가 모두 exit0이었다.
- 변경은 e2e 계약·새 모형 helper2개와 이 기록이다. Rust1395개 생산 소스와 실제 CanvasKit renderer 바이트가 같아 이전146 PASS·lint·WASM·174 TSV/770쪽 및 직접 판독은 같은 범위로 유지한다. 전체 Rust 또는 전쪽 재출력을 반복하지 않았다. 실패 원문·로컬 before/after·진단 결과는 ignored `v456/render-diff-contract-fix-result.json`과 각 로그에 보존한다.

사용자의 개선 후 remote push 지시에 따라 새 test code candidate를 같은 PR에 제출한다. 새 head의 전체 원격 CI 성공 뒤 개별17개 승인·오늘할일을 같은 PR의 trailing docs-only commit으로 반영한다. 첫 candidate의 실패를 무시하거나 진행 중 CI를 완료로 표시하지 않는다.


## 진행 갱신234 — CI Archive B/C 재현과 빈 띠 소유 보정 (2026-10-14)

- 실제 실패 run38430133271/head edd8dc49e: Archive B oracle partition10/14의 `samples/table_scattered_header_rowbreak.hwp` 및 `samples/task1716/table_scattered_header_rowbreak.hwpx`가 기준52쪽 대비54쪽. Archive C text-overlap partition8의 `issue1921/59043_regulatory_analysis.hwp`는4→6건이다. 이전 ddf 전체 로그의 같은 세 검사 PASS를 확인했고, 최신146선별에 이 원장 검사가 없었다는 검증 공백을 기록한다.
- 고정 base223176 바이너리 및 리베이스 전 보존 바이너리는52쪽, 리베이스 직후 d9befb303와6995bc5d3는54쪽이다. 첫 차이는3쪽 pi6 최종 예약626.373→632.587px, 뒤 pi9 내용 예산34.493→28.307px로 두 번째 글줄이 밀리는 경계다.
- 첫 패딩 인증 후보는 dev 빌드 후54/54로 실패해 해당 변경을 원복했다. 실제 GDB 소비 추적으로 pi6/23..61의 마지막 행 상자에 이미 예약한 빈 띠19.493px가 빠지고 새 확장1.333px만 다음 조각에 전달됨을 확인했다.
- `stored_span_content::reserved_straddle_end_band`는 배치의 `ordinary_rowbreak_fragment_heights`에서 마지막 상자 override만 제외한 자연 높이를 얻고, 실제 수용 상자의 차이를 빈 띠로 생산한다. `emit_table_fragment`는 같은 하단의 source_frame_band와 겹치는 몫을 한 번만 합치고 새 확장은 따로 더해 다음 걸침 셀 장부에 전달한다. 문서ID·px 핀·허용치·52쪽 기준선 변경이 없다.
- 보정 dev 빌드는exit0/1분17초이며 HWP/HWPX 모두52쪽이다. 이 쪽수 복구만으로 시각·회귀·lint 완료를 주장하지 않는다. 관련 원장 모든partition, 기존 집중·정상 대조군 및 전쪽 Native/fresh 검증은 다음 단계다. 로컬 전체10,267 재실행은 사용자 지침에 따라 생략하고 새 PR head의 전체 GitHub CI를 확인한다.
- #7778 현재 OPEN/대표#7445 연결과 같은 원본 SHA da9f9c98…/Print fade38ba…를 API로 재조회했다. 같은 font/profile/DPI의 base223176/current edd25쪽은48.36347%이고 PNG 바이트 동일,37/37쪽이다. 영문 칼럼 줄바꿈과 아래쪽 내용 소속 차이를 직접 판독했다. 정상 golden이 아닌 이관 부정 증거를 [59043 증거](../assets/issue7445/ci-regression-20261014/README.md)에 보존한다. 현재4→6 로그도 보존하며 기준값을6으로 늘리지 않는다.
- ignored `output/pr-review/semanticist21-20261008/ci-regression-rebase-v458/`에 실제 job 로그, base/current 바이너리·입력 해시, 첫 실패 후보, GDB 예약/전달, 독립 PDF review/overlay 및 추가 검증 결과를 남긴다. 아직 새 remote push·CI 성공·개별 승인·merge·issue 종료는 수행하지 않았다.


## 진행 갱신235 — 물리 행 계획의 이중 차감 제거 (2026-10-14)

- 진행234 후보는 두 형식52쪽을 복구했지만 Native 전쪽 HWPX의3/7/42/46쪽이90% 미만이었다. 해당 후보의 실제 로그·TSV를 v458에 보존하고 승인·push하지 않았다. Native 실패 확인으로 그 후보의 fresh WASM 빌드만 중단했다.
- 실제 배치 소비를 다시 추적했다. `stored_fragment_row_plan`이 앞 조각의 수용 상자를 반영해 종료 행을 이미36.667로 계산한 뒤, 별도로 만든 기존 `straddle_row_relief`가 paint에서7.546을 다시 빼29.120으로 만들었다. 선택은 원래44.213에서 같은 차감을 한36.667을 사용하므로 예약과 배치가 달랐다. 이 수치는 진단 관측값이며 회귀 기준의 px 핀으로 추가하지 않는다.
- `fragment.rs`에서 유효한 공통 물리 행 계획을 먼저 생산하고, 그 결과가 있으면 별도 기존 띠 차감을 적용하지 않는다. 계획이 없는 경로는 기존 차감을 유지한다. 생산 계획 → source.measured_table/fragment_row_heights → 선택과 emit/paint의 연결을 공유하며 문서별 조건을 추가하지 않았다.
- dev 빌드exit0/55.59초. 새로운 HWPX3쪽 render tree의 해당 행36.7 및 표 외곽649.3은 독립 기준에 대응하는 고정 base223176의 같은 영역과 같으며, 실패 후보의29.1/641.8에서 복구됐다. 이는 구조 진단이다. 전쪽 Native/fresh 일치율·직접 PNG 판독·관련 회귀·필수 lint는 별도 실행 결과로 판정한다.
- 새 증거는 ignored `output/pr-review/semanticist21-20261008/shared-row-plan-v459/`에 생산 소스/바이너리 해시와 함께 보존한다. 전체 로컬 회귀를 반복하지 않고 기존 집중 검사에 oracle/text-overlap의 모든 파티션과 관련 정상 경계를 보완한다. 최종 원격 head의 전체 CI는 다시 확인한다.


## 진행 갱신236 — CI 회귀 보정의 최종 로컬 검증 (2026-10-14)

- 생산 `9591883177d902ed3e2fa239fe047f602c05c7d3` / base `22317614b03722e95349a1d3d71c86923363c6f5`. 수용한 행의 빈 띠 보존(78bc05873)과 물리 행 계획에 대한 중복 relief 차감 제거(959188317)를 검증했다. 문서 ID·고정 px 예외나52쪽 기대값 완화는 없다.
- `cargo nextest` 공용 `target/pr-review`, jobs8, release-test 집중191개 PASS·6 slow·9043 filter skipped·303.367초. 초기 PR 계약78/78·oracle16분할·overlap16분할 모두 포함한다. 첫 실행은 libtest-json-plus 환경 설정 오류로 테스트 시작 전에 종료했으며 소스 실패로 세지 않는다. 재실행 로그와 각 계약 함수·source hash를 v459에 연결한다. 로컬 전체10,267은 재실행하지 않았다.
- 새 package를 실제 소비한 공개 WASM API6종은 모두 exit0이다. 이 결과는 당시 소스의 Studio/Canvas E2E 기록과 구분한다.
- fmt·Native/WASM/workspace-all-targets Clippy, workspace build, manifest/unit-tier 정책(base SHA 명시), fresh package·Studio byte 일치 모두 PASS. 최종 Rust/test 바이트는 이후 asset/docs commit에도 동일성을 유지한다.
- Native/fresh WASM178 TSV·978쪽/backend(각489쪽): 모든 선택 범위90% 이상, 누락0, 독립 PDF 쪽수 일치. 산재 표 HWP/HWPX각52쪽은 양 backend 최저37쪽92.60853%·3쪽99.88979%. 3/4/7/37/42/46쪽의 행 종료·이어받기·외곽·뒤 제목/표·내용 보존을 직접 판독 또는 같은 현재 Native raster의 RGBA 동일성으로 확인했다.
- CGMP48쪽 최저47쪽93.64014%, KPS77쪽42쪽90.85010%, K-water27쪽3쪽96.87092%, #1572쪽1쪽99.74326%, 가맹사업142쪽68쪽93.16816%. 초기48입력52쪽·편집24입력27쪽·OLE/SO10개도 양 backend 선택 범위에서 통과한다. SO-SUEOP는1쪽 회귀 범위이며46쪽 전체 개선으로 확대하지 않는다.
- 현재 Native 가맹사업142쪽·CGMP48쪽·#1572쪽은 이전 판독 raster와 전체RGBA 동일하다. KPS22쪽의 작은 raster 차이는 독립 PDF/전후 확대 crop과 동일 render-tree로 판독했고 K-water3/10/14쪽의1/1/2픽셀 차이도 배치 변화가 아니다. fresh CGMP2쪽의1픽셀 차이는 확대 crop에서 같은 글자 위치·획으로 판독했고 K-water3/10쪽의2/1픽셀 차이도 raster 차이로 구분했다. 중요 문서 fresh 최저값 하락0 및 가맹사업142쪽 전체 Native/이전 fresh RGBA 동일성을 확인했다. fresh 결과와 중요 문서 점수 비교는 `important-fresh-current-comparison.json`, 원 PR 대표 현재/이전 raster 연결은 `original-representative-current-replay.json`을 따른다. 이전 source 이미지를 새 캡처로 가장하지 않는다.
- 가맹사업 fresh의 Native 글꼴 CSS 생산을 같은 소스의 공용 target release-test CLI로 전환했다. 원 dev CLI를 보존하고 source proof 및 전체 p3 render-tree 동일성을 확인했다. 실제 WASM 본문은 새 package로 재생성하며 Native 본문을 대신 사용하지 않는다. 중단한 dev font-policy 시도·로그는 역사 기록으로 보존한다.
- 초기 입력 aggregate는 독립 문서 병렬 처리로 전환하기 위해 SIGINT로 중단했다. 완료된 현재 소스38개 TSV/PNG는 보존하고 미완료44개(나머지 초기10·편집24·OLE6·SO4)는 독립 출력에서 문서4개·문서당 페이지 작업자1개로 실제 실행했다. 중단한 aggregate의 exit-2와 개별 완료 문서 및 새 coordinator exit0을 구분한다. 실제 캡처 경로를 보존한 동일 hardlink를 정식 group 경로에 연결했고 source/pkg/입력/Print·쪽수·최저값을 각 문서에서 검증했다. `document-parallel-plan.json`/`document-parallel-results.json`에 실제 명령·관측·소요 시간을 보존한다.
- 59043은 현재 base/current25쪽48.36347%·동일 PNG이고 같은 원본/증상#7778(#7445 하위)이 이미 등록돼 있어 겹침 원장의 활성 참조/기준선1행만 이관했다. 원본/PDF와 다른 의미 검사는 보존하며4→6허용치 완화는 없다. #7778을 해결로 닫지 않는다.
- 증적은 ignored `output/pr-review/semanticist21-20261008/shared-row-plan-v459/`의 source/입력/Print/font/pkg hash·실행 명령·TSV·로그·직접 판독 JSON 및 committed `ci-correction-v459`의 HWP/HWPX Native/fresh review·overlay8개다. 완료 SVG는 압축하지 않고 삭제했다. 새 exact-head CI는 push 후 확인하며 failed edd8dc49 또는 이전 green 결과로 대신하지 않는다.


## 진행237: #5877 CI 실패 재현과 완료 행 원장 보정 (2026-10-14, v460)

- [실패 CI Archive B](https://github.com/edwardkim/rhwp/actions/runs/38458366449/job/115512064094)는 기존 관계 검사 `issue_5877_source_frame_relations::restart_frames_align_selected_content_and_preserve_the_spanning_tail`의 종료 행 높이 관계에서 실패했다. 로컬에서도 같은 production9591883으로4개 중3 PASS/1 FAIL을 재현했다. 생성 suite 번호는 환경별로 달라질 수 있으므로 원본 module/function으로 연결한다. CI에서는028, 로컬 준비 inventory에서는015다. 검사 삭제·기대값 완화는 없다.
- 원본 `samples/issue5877/fragment_ghost_vrules.hwp`, 독립 `pdf/fragment_ghost_vrules-2020.pdf` 전31쪽을 동일 Print profile/compat2022로 다시 비교했다. 수정 전18쪽76.80448%로 실제 회귀였으며 독립 PDF의 종료 행 높이52.263pt는 약69.684px다. 수정 전 종료 행과 앞 새 행이 모두64.2px로 잘못 같아졌다.
- 생산·소비 경로는 `emit이 완료한 일반 행의 실제 상자 → accepted_row_boxes → stored_fragment_row_plan의 앞 물리 높이 차감 → fragment_row_heights → scan 요구·예약 → 실제 paint`다. 걸침 셀의 미소비 유닛이 남으면 커서는 일반 칸을 이미 소비한 같은 행에 머문다. 기존 계획이 cursor 이전 행만 복사해 완료한 현재 행의 실제58.493px를 원장에 넣지 않았고, 추정64.16px를 다시 사용했다. 일반 컷의 모든 소유 유닛 소비와 실제 accepted 상자가 확인된 경우 이 상자도 포함한다. 미소비 일반 행·분할 재시작 상자·알 수 없는 prefix는 기존 계약을 유지한다. 문서/쪽/px 예외나 clamp를 추가하지 않는다.
- 보정 후 종료 행69.8px로 복구했고 전31쪽 render-tree에서18쪽만 변경됐다. 새 Native 전31쪽은 최저26쪽91.70631%,18쪽95.40188%,17쪽96.49612%, 미달/누락0·독립 PDF31쪽과 일치한다. 현재17/18/26쪽 review·overlay를 직접 표시해 앞/뒤 병합 유닛·종료 행·뒤 제연 설비·본문/외곽/뒤 문단 보존을 확인했다. 글리프 굵기·미세한 기준선 차이는 남는다.
- 증적은 ignored `output/pr-review/semanticist21-20261008/complete-row-prefix-v460/`의 source proof·patch·before-failure·Native 명령/TSV/manifest·현재 트리·직접 판독 기록이다. 필수 lint·집중 회귀(기존191개+위4개)·fresh WASM·영향 문서 확인은 진행 중이며 현재 Native 선행 통과를 최종 승인으로 바꾸지 않는다. Cargo는 공용 `target/pr-review`·jobs8에서 순차 실행하고 로컬 전체 회귀는 반복하지 않는다.


## 진행 갱신238 — #5877 CI 회귀 보정의 최종 로컬 검증 (2026-10-14)

- 생산 `6cbebc7457ea8899c3949e48ed1255d43f40bcb2`, 고정 base `22317614b03722e95349a1d3d71c86923363c6f5`. 완료한 시작 행의 실제 상자와 미소비 걸침 유닛을 분리해 `accepted_row_boxes → stored_fragment_row_plan → scan/예약/paint`가 같은 물리 행 계획을 소비한다. 독립 Print의 종료 행/앞 행 관계는 정확하므로 기존4개 검사와 기대값은 유지했다. 수정 전 동일 함수 FAIL, 수정 후 PASS를 실제 Native CLI 호출로 확인했다.
- 공용 `target/pr-review`·jobs8의 release-test 집중195/195 PASS·1 slow·9699 filter skipped·187.026초, 컴파일10분30초. 기존191개와 #5877 관계4개를 함께 실행했으며 원 PR 계약78/78·oracle16분할·overlap16분할 모두 포함한다. 로컬 전체10,267은 반복하지 않았다.
- fmt·Native/WASM32/workspace-all-targets Clippy·workspace build·고정 base manifest 정책 모두 exit0. 이번 source 변경에는 `#[cfg(test)]` 변경이 없어 unit-tier 추가 검사는 비해당이다. 파생 suite/manifest는 검증용이며 Git에 포함하지 않는다.
- 새 WASM 빌드·최적화20분18초(컴파일11분40초), 공용 pkg/Studio public/frozen package의 JS/WASM 바이트 동일. 실제 새 package의 공개 API6종 모두 exit0. 첫 API orchestration은 필수 expected WASM hash 환경 변수를 빠뜨려 검사 시작 전에 guard에서 종료됐으며 source 실패로 세지 않는다. 실제 hash를 전달한 재실행과 원 로그를 함께 보존했다.
- Native/fresh180 TSV·1040쪽/backend(각520쪽),90% 미만·누락·독립 Print 쪽수 불일치0. 기존89개 입력 범위를 양 backend에서 현재 source로 다시 출력했고 #5877 원본의 전31쪽도 추가로 확인했다. SO-SUEOP는1쪽 한정이며 전체 해결로 확대하지 않는다. 원문 TSV와 입력/Print/source/font/package hash·실제 명령은 위 revision 고정 Gist manifest에 연결한다.
- #5877 수정 전18쪽76.80448% → 현재 Native/fresh95.40188%, 전31쪽 최저26쪽91.70631%. 같은 쪽의 Native/fresh 전31 PNG는 RGBA 동일하다. 현재17/18/26쪽의 앞/뒤 병합 소유·마지막 설비·종료 행·뒤 제연 설비·외곽/뒤 문단을 독립 Print와 직접 대조했다. 전후 트리는18쪽만 변경됐다. 글리프 굵기와 미세한 기준선 차이는 남는다.
- 산재 표 HWP/HWPX 각각52쪽·최저37쪽92.60853%를 유지한다. Native 전104쪽은 이전9591883 이미지와 RGBA 동일하다. 중요 문서의 Native/fresh 각 쪽 점수 하락0을 확인했고 원 PR별 대표32개 현재 재출력도 이전 판독 raster와 동일하다. KPS Native22쪽의543픽셀 획 차이는 트리/점수99.96707%가 같고 전후/독립 Print 확대 crop에서 배치·줄바꿈·표 경계 보존을 판독했다.
- 증적은 ignored `output/pr-review/semanticist21-20261008/complete-row-prefix-v460/`의 source proof·실행 로그·현재 TSV/manifest·직접 판독·전후/동일성 비교와 committed `ci-correction-v460`의 현재 review/overlay PNG다. source1117개 파일 hash를 고정했고 dev/optimized CLI의 #5877 전31 트리 바이트 동일성도 확인했다. 완료 SVG는 압축하지 않고 삭제하며 입력/PDF/PNG/TSV/로그·공용 target은 보존한다.
- 가맹사업 fresh는 전142쪽 WASM/SVG 및 내장 글꼴 검사 완료 뒤 페이지1작업자 캡처를4작업자로 전환했다. 기존26쪽의 완료 PNG/기록을 보존하고 같은 source/package/입력/Print의 남은116쪽만 실제 재캡처했다. 원 coordinator의 의도적 SIGINT·exit-2/전체exit1을 원본 기록으로 보존하고, 새 Node4작업자 캡처·전142쪽 실루엣 비교의 exit0을 별도로 확인했다. 중단한 시도를 source 실패나 전체 성공으로 바꾸지 않는다. 전환용 임시 도구의 multiprocessing main guard와 Snap wrapper 실행 경로 오류도 보존했으며 main 진입점·같은 Chromium155의 직접 실행 경로를 지정한 최종 재캡처/비교만 성공 근거로 사용한다. 실제 명령·완료 기록·동일성은 `franchise-raster-speedup-plan/result.json`과 원/후속 raster JSONL을 따른다.
- 이전 ef9712a77 CI의 실제 실패와 현재 로컬 검증을 구분한다. 새 exact-head GitHub CI가 성공하기 전에는 개별 PR 승인·통합을 적용하지 않는다. 59043의 이관은 진행236의 base/current9591883 미달·기존 #7778 등록 근거를 유지하며 원본/PDF는 보존한다. #7445 전체 또는 #7778/#7807을 해결로 닫지 않는다.


## 진행 갱신239 — 정확한 code head CI 성공과 개별 승인·오늘할일 trailing 기록 (2026-10-14)

통합 PR#7882의 `0d59e7f73cc366897891329a21d353458db7776e` 실제 CI를 모두 확인했다: [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/38475033894) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38475033796) · [CI](https://github.com/edwardkim/rhwp/actions/runs/38475033776) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/38475033750) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38475033733). code head에 대한 결과가 성공한 뒤 원PR17개를 **승인 — 메인터너 보정 후 승인**으로 갱신하고#7612의 기존 병합·명시 예외를 보존했다. 현재 적용39개의 조상 관계와 실제 focused 원 계약78/78 PASS를 재확인해 각 문서에 원/보정 SHA·자기 계약·원문 입력/Print 범위·후속 댓글 계획을 연결했다. 오늘할일은 현재 날짜 파일에 이 PR 항목만 추가하며 최종 merge-tree에서 최신 base의 기존 기록과 신규 기록 보존을 검증한다. 같은 PR의 문서-only trailing commit이고 통합번호용 별도 review 문서를 만들지 않았다. 최신 trailing head 검사와 merge SHA·devel·issue/원 PR 후속처리는 다음 단계다.


## 진행 갱신240 — 통합 merge 확정값의 후속 기록 (2026-10-14)

## 통합 merge 확인 — 2026-10-14

통합 PR [#7882](https://github.com/edwardkim/rhwp/pull/7882)는 merge `86a3c88fc89de9a2306a10bdda9a31023f0e24a5`로 실제 병합됐으며 `upstream/devel` 포함을 확인했다. code candidate `0d59e7f73cc366897891329a21d353458db7776e`의 전체 CI, 문서-only trailing head `34cc7ddefa246ecb7780c748dc7eb449bde89e6b`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/38478418053)·CodeQL·Render Diff·Adapter·Proptest 및 [정책 감사3](https://github.com/edwardkim/rhwp/actions/runs/38478674616)를 확인했다. 최종 `CI Impact Policy=success`, `MERGEABLE/CLEAN`, 모든 최신 head check 성공/정상 skipped 상태에서 exact-head merge했다. API의 완료 step 메타데이터 지연으로 첫 정책 감사가 실패/대기했으며 실제 step이 모두 종료된 뒤 Controller만 재실행했다. 검사를 완화하거나 전체 로컬 회귀를 반복하지 않았다.

195 집중 PASS·180 Native/fresh TSV·1040쪽/backend의 범위와 독립 입력/Print·생산 SHA는 이전 최종 검증에 유지한다. 이번 후속 기록은 확정 merge SHA만 보완하며 source/test/fixture/workflow·시각 결과는 바꾸지 않는다. 이미 제출된 오늘할일은 반복 갱신하지 않는다.

후속 범위는 CGMP 하위 [#7764](https://github.com/edwardkim/rhwp/issues/7764)의 완료 증거와 원 PR별 contributor comment/supersede 처리다. 원 PR 16개는 머지 후 원격 head 재확인 뒤 처리한다. #7612의 기존 merge는 보존하고 #7615의 초기 수용 이후 변경과 후속6개는 제외한 상태로 유지한다. 대표 #7445 및 Refs #7759/#7807/#4514는 전체 해결로 닫지 않는다. 실행 결과는 원 PR·해당 이슈의 실제 comment에 남기며 이 문단을 미실행 close의 증거로 사용하지 않는다.


## 진행 갱신241 — 요청한 두 표 파일의 추가 영문 주석 한글화 (2026-10-14)

통합·원 PR 후속처리의 최종 점검에서 보정 과정에 추가된 영문 주석17줄을 발견했다. 사용자께서 지정한 `src/renderer/typeset/table/continuation/fragment/emit.rs`와 `src/renderer/layout/table_partial.rs`의 일반 `//` 설명만 한글로 바꾼다. 코드 줄과 전체 줄 수를 그대로 유지하며, 변경 전후 비주석 원문 바이트 동일성을 확인한다. 함수·조건·저장 정보·테스트·fixture 및 출력 계약을 바꾸지 않는 주석 한글화이므로 새 조판 개선이나 시각 재출력으로 주장하지 않는다. 이 변경 범위의 Visual Sweep·회귀 신규 추가는 비해당이다. 프로젝트의 Rust source 변경 필수 lint 묶음은 같은 공용 target/jobs8로 순차 수행하고, 현재 전체 로컬 회귀를 반복하지 않는다. 결과와 별도 주석 PR의 정확한 head CI는 아래에 기록한다. 기존 두 파일의 줄 수를 늘리거나 이 언어 수정에 기능 분할·리팩토링을 섞지 않는다.


### 주석 보완의 실제 로컬 검증

정책 base `07e7e813d06d3f8031e86eb7c27233b829964907`, 공용 `target/pr-review`, Cargo jobs8. 구현 코드/줄 수 동일성은 포맷 뒤 다시 확인했다. 아래 필수 명령은 모두 exit0이며, source-side cfg(test) 변경은 없으므로 추가 unit-tier 검사는 비해당이다. 파생 suite/manifest는 PR에 포함하지 않는다.

| 실제 단계 | 결과/초 |
| --- | --- |
| `prepare` | exit0 / 2.912 |
| `fmt` | exit0 / 31.177 |
| `fmt-check` | exit0 / 27.815 |
| `clippy-native` | exit0 / 105.519 |
| `clippy-wasm` | exit0 / 87.232 |
| `build-workspace` | exit0 / 114.054 |
| `clippy-all-targets` | exit0 / 117.909 |
| `manifest` | exit0 / 2.784 |

원 통합 PR #7882 및 문서 후속 #7887, #7764 완료 증거·원 PR16개 종료/17개 댓글과 전용 branch/worktree 정리는 실제 확인한 기존 완료 범위다. 이번 주석 보완 PR의 GitHub CI·merge는 이후 별도로 확인하며, 기존 CI나 시각 자료를 새 실행으로 주장하지 않는다.


## 진행 갱신242 — 주석 보완 PR #7889의 정확한 code CI 성공과 trailing 기록 (2026-10-14)

**승인 — 요청한 주석 한글화 보완.** [PR #7889](https://github.com/edwardkim/rhwp/pull/7889)의 code candidate `7b6cdc6e235553e3df676c6be31848b14ab03544`, 정책 base `07e7e813d06d3f8031e86eb7c27233b829964907`에서 필수 로컬 lint 묶음과 정확한 head의 GitHub CI를 완료했다: [CI](https://github.com/edwardkim/rhwp/actions/runs/38483170236) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/38483170161) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/38483170109) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38483170088) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38483170052). 최종 `CI Impact Policy=success`도 확인했다. 이전 통합 PR의 CI를 이번 head의 결과로 대신하지 않았다.

`emit.rs` 일반 주석8줄과 `table_partial.rs` 일반 주석9줄만 한글화했고 포맷 뒤/commit 뒤의 비주석 원문 바이트·파일 줄 수 동일성을 확인했다. 코드·검사·fixture·출력 계약의 변경은 없어 조판/Visual Sweep 범위는 비해당이다. 기존 Native/fresh 자료는 해당 생산 SHA의 역사 증거로 유지하며 새 source 재출력으로 주장하지 않는다. 이 두 파일은 기존 크기를 유지하고 언어 수정에 리팩토링을 섞지 않았다.

같은 PR에 이 review와 오늘할일만 single-parent trailing commit으로 반영한다. 새 trailing head의 CI·정책 status·mergeability를 확인한 뒤 병합하며 현재 merge 완료로 기록하지 않는다. 원 PR #7882/#7887의 issue·contributor 후속처리는 이미 완료된 범위로 보존하고 반복하지 않는다. 이 주석 PR에는 추가 종료 이슈가 없다. 병합 후 duration 갱신 결과·devel 포함과 전용 브랜치 정리만 확인한다.


## 진행 갱신243 — 주석 보완 PR #7889 통합 완료 (2026-10-14)

**주석 한글화 통합 완료.** [#7889](https://github.com/edwardkim/rhwp/pull/7889)는 merge `79f66b6c470c61f8e08883c33215ba5e0fd668b1`로 병합됐고 `upstream/devel` 포함을 확인했다. code candidate `7b6cdc6e235553e3df676c6be31848b14ab03544`의 full CI 및 문서-only trailing `7b0f79cb8ff96fdbb3462b2be5781c933d4d4f29`의 실제 CI·정책 status 성공, `MERGEABLE/CLEAN` 확인 뒤 exact-head merge했다. trailing 실제 실행은 [CI](https://github.com/edwardkim/rhwp/actions/runs/38485743742) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/38485743701) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/38485743690) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38485743669) · [Cancel stale PR runs](https://github.com/edwardkim/rhwp/actions/runs/38485743620) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38485743584). 이 문단이 주석 보완의 현재 상태이며 진행241~242의 당시 대기 상태는 역사 기록이다.

두 파일의 일반 영문 주석17줄은 한글로 바뀌었으며 비주석 원문 바이트·파일 줄 수 동일성을 최종 head에서도 확인했다. 기존 source/test/fixture/출력 계약과 원 통합 PR #7882/#7887의 증거·#7764/원 PR16개 종료·17개 댓글을 유지한다. 오늘할일은 #7889에 이미 포함된 trailing 기록을 보존하고 반복 갱신하지 않는다. 추가 종료 이슈는 없다. 이 문서는 완료된 원 PR의 확정 SHA만 보완하는 review-only 후속 기록이며 별도 code 검증·issue/contributor 후속처리를 반복하지 않는다. merge 후 duration 갱신 결과와 전용 브랜치 정리의 실행 증거는 해당 PR의 완료 comment 및 ignored 운영 로그에 남긴다.
