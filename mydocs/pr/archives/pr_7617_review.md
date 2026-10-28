---
kind: investigation
status: historical
canonical: mydocs/pr/archives/pr_7617_review.md
last_verified: 2026-10-10
---

# PR #7617 리뷰 — 표가 민 첫 글줄의 문단 앞 간격

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 기여와 누적 메인터너 보정을 포함한 통합 후보의 판정이다.
최종 통합은 아래에 기록한 남은 검증 조건을 완료한 뒤 진행한다.

표가 밀어낸 첫 글줄의 앞 간격과 측정·배치 소비 경로를 보정했다. 이어받는 각주의 원본 소유·그림·중첩 셀 각주를 보존하며, 가맹사업 보고서의 독립 Print142쪽과 Native/fresh WASM 전142쪽 최저92.20721%를 확인했다.

## 현재 검증 근거 — 2026-10-11

| 항목 | 확인한 상태 |
| --- | --- |
| 검토 branch | `review/planet6897-20261008` |
| 통합 PR | [#7832](https://github.com/edwardkim/rhwp/pull/7832) → `devel`; 원 기여와 메인터너 보정 함께 제출 |
| rebase 후 통합 code head | `9b4817473b2a3515ea27eefef9d20731258433e4` |
| 최신 upstream base | `920dc29284e8a6e624861bfccf68903e19ae309b` |
| 전체 회귀 | 검증 source `7dd9572b7fecbc602c9e5ab47c4a6c9d6422b4bd`: 10,215PASS·실패0·skip45 |
| lint·정책 | fmt·Native/WASM/all-target Clippy·workspace build·base 고정 manifest/unit-tier PASS |
| Studio | 1,834PASS·실패0·skip2, TypeScript 검사 PASS |
| optimized fresh WASM | code source `03a987f4a`: 빌드exit0, `pkg/`·Studio `public/` JS/WASM SHA-256 일치 |
| 시각 선행 조건 | table-text·stored-vpos·hwp-multi·해외투자3입력의 Native/fresh WASM 전쪽 TSV 모두90% 이상. table-text·stored-vpos 변경 영역 확대 판독 완료 |
| 승인된 이관 | #7813 giant-cell, #7830 sample2 HWP, #7831 가정폭력 조사. 각 출력 문제는 후속 이슈에서 해결 |
| rebase 후 재검사 | 오늘할일만 충돌했고 양쪽 기록을 보존했다. 사용자 지시에 따라 통과한 전체 회귀·lint·Studio를 반복하지 않음 |

전체 결과·시각 점수·source와 로그는 [남은 회귀 보정 기록](../../plans/planet6897_20261008_remaining_regressions.md#현재-시각-검증-상태-정정--2026-10-11)에 연결한다.
위 시각 결과는 해당 입력과 기록한 renderer source의 증거이며, 원 PR 전체 영향 목록의 전수 통과를 주장하지 않는다.

## #7768 재확인과 통합 조건

#7768 원본 전1쪽의 Native/fresh WASM 최저92.73966%, 누락0을 확인했다.
양 경로 raster bytes와 review/overlay의 canonical gate `passed`를 확인하고 캡션·표·
체크박스3개·후행 용지 규격 줄을 확대 판독했다. [공개 시각 증적](../assets/issue7768/README.md)을
통합 PR의 `Closes #7768` 근거로 연결한다.

- 최신 upstream의 확장 프로그램 #7703·기존 오늘할일을 보존해 rebase했다. 충돌은 오늘할일 한 파일뿐이며 Rust/test/Cargo tree는 rebase 전과 같다.
- 사용자 지시에 따라 rebase 후 추가 로컬 테스트를 하지 않는다. 별도 로컬 Native Skia3종은 미실행으로 기록하고 통합 PR의 Native Skia CI 결과를 확인한다.
- 남은 통합 조건은 제출 head의 GitHub CI·CodeQL·Native Skia 및 mergeability 확인이다. 원격 CI 완료·merge를 미리 보고하지 않는다.

<details>
<summary>이전 검토 이력 — 당시 판정과 단계별 실행 기록</summary>

초기 검토에서 확인한 결함·증거 부족과 단계별 재검증 기록이다. 당시 보류 판단은 아래 이력의 source와 실행 범위에 한정하며, 현재 판정은 문서 상단을 따른다.

## 접수·적용 범위

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·base | [#7617](https://github.com/edwardkim/rhwp/pull/7617) / planet6897 / devel |
| 원 head | `c19f5b805e4be92d9b51f561b2e47e0c587e3145` |
| 최초 적용 commit | `64f0a293212070210550021e64ecd5a1b7c5876c` |
| 이 PR의 적용 뒤 head | `3e7bacd2c9204d5bc3048b32628b8fc70a753bec`; 기능 commit 6개, 원 저자·cherry-pick -x 보존 |
| 누적 검증 source head | `90e2c6fd2c614ba0ea7667355c4ed2f3617f6563` |
| 최신 base 정렬 뒤 code head | `ae786f75b2cee72bd5576f6473bd51d9a8216f7c`; 제품/검증 입력 tree 동일 확인 |
| 검토 branch·reviewer | `review/planet6897-20261008` / jangster77; 원 PR reviewer assign 선행 |
| 원 PR 참고 상태 | draft=False, MERGEABLE / CLEAN; 작성 시점 snapshot이며 merge 직전 판정을 대신하지 않음 |
| 경로 | collaborator 외부 PR의 주 작업공간 누적 cherry-pick 9.1.1; intake/template/local_validation/visual_fixture_evidence 및 대형 PR 보조 절차 |

관련 이슈와 해결 표현은 원 PR 본문의 범위를 유지한다. 이번 보류 기록에 새로운 closes 표현을 추가하지 않는다. 원 PR 제공 before/after와 corpus 수치는 참고이며 통합 head의 독립 실행과 구별한다.

## 구현 주장·소비 경로·검증 판정

`typeset/paragraph.rs:223`의 float 표 원점과 `fmt.spacing_before` 차감 → `typeset/paragraph/metrics.rs:flow_trimmed_spacing_before / flow_advance_height` → `paragraph/flow.rs:place_fitted_paragraph`의 실제 배치를 대조했다. 표가 밀어낸 첫 글줄의 앞 간격 중복과 표 없는 정상 간격은 서로 다른 상태다.

공개 `float-push` 1쪽과 `column-top` 4쪽의 Native/fresh WASM 전쪽 최저는 각각 99.89324%, 99.09774%다. 대표 p1/p2에서 표 아래 첫 글줄·후속 문단 순서와 정상 대조군 간격을 직접 확인했다. **미검증:** 원 PR의 전체 영향 corpus에 있는 90% 미만 쪽, 대응 Print PDF가 없는 비공개 사례의 실제 저장본/재조판 경로. 공개 대조군의 충족만으로 원 PR 전체를 승인하지 않는다.

공통 빌드·회귀·글꼴 등록·383쪽 시각 결과와 입력/기준 해시·증적 보존·merge 후 댓글 계획은 [#7611 공통 통합 검증](pr_7611_review.md#공통-통합-검증과-증거-보존)에 연결한다. 이 PR의 조건부 수용이 아니라 미충족·미검증 해소 후 재검토 계획이다.

![native float-push p1 비교](../assets/planet6897_20261008/native_float-push_p001_review.png)

![wasm float-push p1 비교](../assets/planet6897_20261008/wasm_float-push_p001_review.png)

![native column-top p2 비교](../assets/planet6897_20261008/native_column-top_p002_review.png)

![wasm column-top p2 비교](../assets/planet6897_20261008/wasm_column-top_p002_review.png)

## 최신 보완의 경계 입력 검증

원 PR의 후속 구현 `7802047fb2682a2c97b35a3045dc76ddfac343fe`는 통합 후보에서 원 contributor 구현 뒤·메인터너 보정 앞의 `6230b09b6486e07cf3775580e9d9ff8f796f890d`로 반영했다. 아래 결과의 source head는 `59e1756d59b8d261b7d525772f991fe92e7edf9a`다. 최신 원 PR의 공개 경계 입력·기준 PDF·생성기·7개 회귀 source는 `1ed091daf421f92a9e2536645f9178877d54be40`의 정확한 Git blob을 ignored scratch로 추출해 선행 검증했으며, 아직 통합 commit에 추가하지 않았다.

공통 결과의 소비 경로는 `float_placement.rs:float_pushed_line_spacing_before` → `typeset.rs`의 첫 post-table 글줄 조건 → `state/commands.rs:consume_float_pushed_line_spacing`의 누적 높이·문단별 map → 단 전환의 map 전달 → `layout.rs`의 `start_line == 0` map 조회 → `paragraph_layout.rs:layout_partial_paragraph_with_spacing_before`의 확정 앞 간격이다. 저장 host 원점이 있는 경우는 생산 조건에서 제외한다. 앞 간격은 문단 상단 기준 거리이며, 표 점유 끝 위에 남는 몫만 한 번 예약한다. endnote 계측의 새 `EndnoteColumnPlacements`에도 global→local 문단 remap으로 map을 전달하는 호환 보정을 포함했다. 이 경계 입력은 endnote를 포함하지 않으므로 해당 endnote 실행 경계는 미검증이다.

공개 합성 HWPX(저장 줄 없는 생성본)와 이를 한/글 2020에서 저장한 HWP를 같은 HWP Print PDF 4쪽에 대조했다. Native/fresh WASM 각각 HWP 최저 99.43997%, HWPX 최저 96.45793%이며 총 16쪽 모두 `pr_review_gate=passed`, 누락·90% 미만은 0쪽이다. 16개 review PNG를 직접 열어 앞 간격이 표보다 큰/작은 host, 표 없는 대조 문단, 쪽 맨 위, 마지막 host 세 줄과 뒤 문단을 확인했다. 글꼴 획·농도와 HWPX 재조판의 일부 글자 간격 차이는 남으며 완전한 시각 일치로 보고하지 않는다.

독립 Print PDF의 텍스트에서 얻은 2쪽 채움 1~39·3쪽 채움 40~42를 네 출력의 실제 render tree와 대조해 순서·누락·중복 및 마지막 host 세 줄을 확인했다. 결과·입력/build hash·명령·manifest·PNG는 ignored `output/pr-review/planet6897-20261008/pr7617-boundaries-59e175/`의 `verification.json`·`page-ownership.json`과 네 하위 sweep에 보존한다. 이 검증은 공개 경계 입력의 신규 회귀 추가 선행 증거이며, 아직 실행하지 않은 해당 7개 정식 회귀의 PASS나 전체 영향 corpus·통합 PR 승인으로 기록하지 않는다.

## 편람 HWPX의 384쪽 불일치와 별도 메인터너 보정

후속 구현 `7802047` 적용 전 `1a1625ebdbbaf9369f142991fed0b4086f5db4d6`의 보존 WASM과 적용 후 `59e1756d59b8d261b7d525772f991fe92e7edf9a`의 보존 WASM을 같은 Chrome·원문·옵션으로 실행했다. 두 출력 모두 384쪽이고, 279~283쪽 SVG와 render tree 해시도 같았다. 따라서 이 마지막 구현이 해당 추가 쪽을 만들었다는 가설은 재현 결과와 맞지 않는다. 이 A/B는 이전 패키지를 사용한 원인 분리이며 최종 fresh WASM 승인 증거가 아니다. 기록은 `output/pr-review/planet6897-20261008/pr7617-source-ab-wasm/comparison.json`에 있다.

원본 HWPX 구역 10 문단 3의 제목 묶음 도형은 문단 기준·오프셋 0·자리차지이고, 빈 host의 유효 저장 줄은 폭 0, 높이 1000HU, 줄간격 600HU다. 도형 높이 6315HU와 아래 여백 1417HU의 합은 다음 목차 host의 저장 시작점 7732HU와 같다. 독립 Print PDF의 목차는 279~281쪽이며 뒤 질의·답변은 282쪽에서 시작한다. 도형은 보이지만 빈 host 줄은 도형 띠 안에 포함되므로 별도 공간으로 더하지 않는 계약이다.

기존 공통 판별 `renderer/mod.rs:empty_host_line_absorbed_by_topbottom_float` → `typeset/section/controls.rs`의 추가 예약 높이 → 가로 컬럼별 누적 예약과 `advance_flow_by` → `layout.rs:layout_shape_item`의 실제 도형 하단 전진을 추적했다. 그림 분기는 문단 측정에서 이미 소비한 host 줄을 차감했지만 도형 분기는 차감하지 않아 목차 예산에 1600HU(96dpi에서 21.33px)를 중복 계상했다. 도형 분기에도 같은 공통 판별을 적용했다. 문서 ID나 목차 항목 수로 분기하지 않으며, 글자·유효 줄 폭·세로 오프셋·구현 생성 줄 등 기존 판별의 반례 조건을 유지한다.

`59e1756d59` 위의 해당 source 수정 후보를 공유 `target/pr-review`에서 `cargo build --locked --jobs 8 --bin rhwp`로 새로 빌드했고 종료 코드 0을 확인했다. 같은 바이너리의 `dump-pages --compat 2024 --json`에서 HWPX 384→383쪽, HWP 대조군 383→383쪽이다. HWPX 목차 첫 예약 원점은 124.4→103.1px, 첫 두 조각의 끝 컷은 27/60→29/62로 바뀌어 원본 저장 줄의 쪽 경계와 일치한다. 마지막 조각 뒤 질의·답변도 283→282쪽이다. HWP는 전쪽 항목 소속이 같고 5쪽의 예약 높이만 변했다. 바이너리·source patch·원문 hash와 실행 로그는 `output/pr-review/planet6897-20261008/shape-host-absorption-v1/verification.json` 및 같은 디렉터리의 두 형식 페이지 계획·drift 로그에 보존했다.

첫 도형 보정 후보의 Native 279~283쪽 Visual Sweep 최저는 96.97055%, 자동 gate는 `passed`다. 5개 review PNG를 직접 열어 목차 항목 1~18/19~37/38~55의 소속과 뒤 본문 시작을 대조했고, render tree에서도 항목 1~55의 순서·누락·중복을 검사했다. 최초 실행은 Windows의 manifest 원자 교체 시 파일 접근 오류로 종료됐고, 동일 provenance의 `--resume` 실행 종료 코드 0을 확인했다. 원 실패 로그와 재개 로그를 모두 보존했다. **직접 판독으로 남은 배치 결함:** 명시적 우측 탭 쪽번호의 가로 이동과 281쪽 마지막 표 테두리의 부족한 높이. 높은 점수로 이 차이를 면제하지 않는다.

사용자가 지적한 281쪽 목차 쪽번호 18개의 시작은 Native x=596.27px, PDF advance box x=439.92pt(96dpi에서 586.56px)로 약 9.7px 오른쪽이다. 본문 `38.`/`39.`의 시작은 Native 120.5px, PDF 90.36pt(120.48px)로 맞는다. 원본 paraPr 742의 RIGHT 탭 정지는 38272HU이고, 단 시작에 더한 우단은 약 608.59px다. 기존 `find_next_tab_stop`이 이 정지를 생산했지만 교차 run 배치/폭 추정과 글자 위치 측정이 리더를 이유로 셀 콘텐츠 우단으로 덮어썼다. 후속 보정 후보는 명시적 탭 정지를 유지하고 자동 탭만 본문 우단을 생산하도록 생산자의 좌표계를 통일했다. `text_measurement.rs:find_next_tab_stop` → 문자 위치/폭 측정 → `paragraph_layout.rs:resolve_last_tab_pending` → 교차 run 폭 추정/방출의 실제 소비 경로를 함께 바꿨다. inline 저장 확장 탭의 별도 경로는 아직 이 보정의 검증 완료로 세지 않는다.

후속 탭 보정 후보도 `--jobs 8` Native 빌드 종료 코드 0과 HWP/HWPX 각각 383쪽을 확인했다. 새 281쪽 Native에서 쪽번호 시작점은 586.59px로 이동해 PDF 586.56px와 맞고, Visual Sweep 종료 코드 0·실루엣 99.12348%를 확인했다. 새 review PNG를 직접 열어 쪽번호·줄바꿈과 남은 표 아래 테두리 차이를 확인했다. 기록과 이미지는 `output/pr-review/planet6897-20261008/shape-host-and-right-tab-v2/`에 보존했다.

후속 탭 보정 후보의 fresh WASM 빌드와 실제 Chrome 281쪽 비교도 종료 코드 0을 확인했다. Native와 같은 쪽번호 위치·실루엣 99.12348%였고 새 review PNG를 직접 확인했다. 당시 표 아래 테두리 차이는 남아 있었으며 이 결과는 다음 종료 프레임 보정이나 전쪽 WASM 검증에 승계하지 않는다. 기록은 `shape-host-and-right-tab-v2/wasm-toc-panel/verification.json`에 보존했다.

종료 프레임 보정은 위 도형·탭 변경과 함께 `e469c4486e5e5d7b6de5a6e4a74d938e89dab188`에 커밋했다. 원본 단일 셀 선언 높이는 159642HU로 세 쪽을 포함하지만 기존 종료 helper는 첫 object 높이 47726HU 하나만 빼서 후속 빈 문단의 vpos와 비교했다. 실제 뒤 빈 문단의 저장 원점은 55458HU다. 새 `stored_single_cell_closing_frame_bottom`은 유효·비편집·비합성 저장 host/후속 줄을 확인하고 그 하단을 생산한다. 종료 `emit`은 실제 재개 컷이 원본 쪽 프레임에서 시작한 경우에만 현재 조각 원점을 빼고, 내용 요구 높이 이상·가용 예산 이하인지 확인한 같은 상자를 `partial_height`와 행 높이 override로 전달한다. 반복 머리행 높이는 override에서 한 번 제외한다. 내용 컷을 더 소비하거나 paint에서 별도로 상자를 늘리지 않는다. 편집·reflow·caption·각주·일반 셀 분기는 이 근거의 대용으로 쓰지 않는다.

이 커밋의 새 Native 279~282쪽 compare·standalone overlay·review를 산출했다(exit 0). 최저 98.49306%, 90% 미만·측정 불가 0쪽, focused gate `passed`다. 281쪽은 99.92654%이며 마지막 셀 높이는 709.3867→739.44px로 변했다. 네 review와 281쪽 standalone overlay를 직접 열어 쪽번호·표 아래 테두리·뒤 282쪽 본문을 대조했다. 실제 TextLine을 결합한 render tree 검사에서도 목차 1~18/19~37/38~55가 원래 279~281쪽에 한 번씩 순서대로 소속됐다. 글꼴 획·농도·AA, 제목 괘선 두께와 일부 본문 장식 경계는 남는다. source·binary·원본/PDF hash, 명령·PNG hash·직접 판독은 `committed-e469c4486-native-toc-provenance.json`, 소속은 `committed-e469c4486-toc-ownership.json`, PNG는 `committed-e469c4486-native-toc/handbook-hwpx-e469c4486/`에 보존한다. 이 자료는 Native 선택 쪽 증거이며 전쪽·fresh WASM·정식 새 회귀의 완료를 뜻하지 않는다.

**현재 미검증:** 종료 프레임 보정의 fresh WASM·전쪽 시각 검증·반례와 필수 lint 묶음. 전체 회귀는 [최신 커밋의 공통 결과](pr_7611_review.md#최신-커밋의-전체-회귀와-base-판정)처럼 28개가 실패했으며 최신 base의 90% 기준으로 이관/현재 보정을 구분 중이다. 첫 후보의 파생 suite를 준비한 review worktree의 `cargo fmt --all -- --check`와 fresh WASM 빌드는 통과했다. 주 작업공간에서 먼저 실행한 fmt 확인은 오래된 파생 목록의 누락 module 때문에 실패했으며 제품 결함 재현으로 세지 않는다. 후속 source 변경에는 첫 후보의 검증을 그대로 적용하지 않는다. 최신 원 PR head `85ea9a90bd0cd348b820db0eb9cea8d95356a15d`의 7개 경계 검사·자산은 선행 진단 scratch에만 있으며 아직 통합 commit에 추가하지 않았다.

## HWPX 전쪽 비교와 저장 경계 후보의 소유 보정

f971 제품 source의 HWP 편람 전383쪽은 Native/fresh WASM 최저90.08363%·누락0을 확인했다. 같은 source의 HWPX 전383쪽 Native는 최저27.09624%이며16쪽(116,126,135,140,152~156,223,287,289,290,370,373,375)이90% 미만이다. 같은 쪽수로 내용 소유의 차이를 면제하지 않는다.116/140/290쪽 compare·overlay·review를 직접 판독해 인접 표/뒤 본문 간격, 이어받기 문단 및 중첩 표의 실제 배치 차이를 확인했다. `candidate-f97144651-native-handbook-hwpx-full/provenance.json`과 `candidate-f97144651-hwpx-failure-panels/`에 보존하며, HWPX fresh WASM 전쪽 결과나 후속 코드의 통과로 승격하지 않는다.

140쪽의 원본(section3, host 문단64,1×1 RowBreak 셀)은 저장 줄이 문단16에서 새 원점으로 돌아간다. 독립 Print는 다음 문단17을140쪽에 두지만 Native는 그 문단들을 앞쪽으로 가져와140쪽을 문단20부터 시작한다. 실제 `CUT_BASE`는21유닛/714.507px, 기존 `stored_frame_cut_for_row`가 인증한 source 후보는16유닛/639.627px이며 동일 행 예산735.160px에 들어간다. `source_frame::resolve → source_tail → scan/runner/row_step`은 이 후보를 계산하고도 height extension이 양수가 아니라는 이유로 버렸다. 음의74.880px는 화면 위치를 맞출 보정 상수가 아니라 관측한 두 컷의 내용 높이 차이다. 원본과 소스 좌표, 진단 로그는 `pr7617-hwpx140-source-frame-closure.json`, `candidate-46de329e5-hwpx140-scan/`에 보존한다.

후속 후보는 기존 인증·profile·행 소유 gate 안에서, 실제 hard-break source 컷이 모든 셀의 capacity 컷보다 같거나 앞서며 적어도 하나가 앞설 때 그 source 컷을 선택한다. 해당 컷의 중첩 점유 예약까지 현재 행 예산에 들어가야 한다는 기존 판정은 유지한다. 예산이 source 경계에 못 미친 경우의 기존 양수 확장 계약·상한은 유지한다. 선택 결과는 같은 `res.end_cut/consumed_height`로 fit/retry와 emit을 거쳐 다음 continuation 시작 컷 및 실제 배치가 소비한다. source 인증을 완화하거나 임의로 컷 인덱스/좌표를 고정하지 않는다. 빈 물리 밴드·누적 예약과 최종 괘선/뒤 본문의 소비 경로도 새 실행에서 대조한다. 원본의 관련 출력 범위90%를 확보하기 전 새 회귀나 기준값을 추가하지 않는다.

제품 head `02de253c1bd84ca9607694694abfd7658a1bdb01`의 공유 target Native 빌드는 jobs8·exit0(452.17초)이다. 원본 결재문서5개 전7쪽은 최신 Native에서도 최저99.61781%다. 편람 HWPX의 기존 미달16쪽을 새로 출력한 결과,370쪽96.40865%와373쪽98.59779%는90% 이상이 됐으나 나머지14쪽은 미달이다. 전체 문서는383쪽이며 선택 쪽 비교를 전쪽 통과로 보고하지 않는다.140쪽은 문단17부터 이어받아 원본 소유 경계를 회복했지만26.89414%이고, 셀 안 문단 간격·그림·최종 외곽선의 높이 차이는 남는다. `candidate-02de253c1-handbook-boundaries.json`, `candidate-02de253c1-handbook-hwpx-boundaries/handbook-hwpx/silhouette.tsv`와 `candidate-02de253c1-handbook-boundary-panels/`에 실행·직접 비교 자료를 보존한다. fresh WASM 및 최종 전체 회귀는 아직 미검증이다.

116쪽 원본(section2,문단884/885)의 TAC 표 앞에는 흐름을 점유하지 않는 `IN_FRONT_OF_TEXT` 장식 도형이 있다. 두 host의 저장 높이7041HU는 표6761HU와 바깥여백140+140HU의 합이고, 다음 저장 원점12921−4440=8481HU에는 첫 줄 후행 간격1440HU도 포함된다. 기존 Native의 표 원점 차이는93.9px로 줄 높이만 소비하고19.2px의 간격을 놓쳤다. 장식이 없다는 단일 컨트롤 가정 때문에 `stored_tac::prepare_computed`가 닫힌 원본 저장 줄을 거절하고, 일반 paint의 가시 장식 전용 억제 경로가 후행 간격을 버린 것이 원인이다. 장식은 비가시 metadata가 아니며, 가시성과 흐름 점유를 구별한다.

후속 후보는 편집되지 않은 HWPX의 기존 저장 원점·후속 줄 연결·실측 높이 인증을 유지하면서, 유일한 표와 비TAC 글 앞/뒤 장식만 있는 경우 실제 표 컨트롤 번호로 계획을 만든다. 합성 줄은 기존 단일 컨트롤 조건을 유지하고, 자리차지·어울림·TAC 장식과 다른 표는 이 단축을 쓰지 않는다. 생산 `prepare_computed`의 `origin/end` → `commit_stored_tac_control`의 fit/흐름/inline placement → `layout_table_control`의 같은 pen와 `advance_end` 소비를 연결한다. 장식은 기존 `place_paragraph_controls`에서 계속 발행한다. 새 후보의 직접 출력과 원본 대조군은 아직 미검증이며, 이 단계에 새 시험·fixture·golden을 추가하지 않는다.

140쪽에서 남은 간격 차이는 빈 문단16/18/22의 실제 저장 슬롯1760/1872/1700HU(합71.09333px)와 연결된다. 모두 다음 저장 줄 원점에 정확히 닿지만 현재 tree에는 세 문단의 줄이 없고, 문단17이 셀 위 패딩 직후 시작한다. `cell_units_uncached::preserve_forward_stored_empty_spacer`는 직접 HWPX에 선언 높이800px 이상과 셀 전체의 선형 등식을 요구한다. 해당 표는735.70667px이고 그림 host11/14 뒤에는 바깥 간격300HU가 있어 두 전제를 통과하지 못한다. 반면 분할 컷은 이미 `direct_hwpx_cell_has_declared_stored_frame`으로 같은 원본의 두 프레임을 인증한다. 독립 PDF 텍스트 연산자·원본 저장 줄·현재 tree/hash는 `candidate-02de253c1-book140-blank-trace.json`에 보존한다.

`ca98ce5e8b15d50886fca35a99bd24580eeaf8ab`의 Native 빌드429.72초·exit0, 결재문서5개 전7쪽 최저99.61781%, 기존 의미 검사2PASS를 확인했다. 의미 검사는 검증한 기존 harness와 변하지 않은 모델·parser·검사 source를 재사용하되 출력 CLI를 새 바이너리로 고정했으며 최종 전체 suite 컴파일로 보고하지 않는다.116쪽은39.39997→97.35333%이고 표 원점·뒤 본문 간격이 맞지만, 직접 review에서 화살표 장식의 누락을 검출했다. 앞 기록의 `place_paragraph_controls`가 장식을 발행한다는 설명은 잘못됐다. 표 host의 장식은 일반 `controls/paragraph_flow::place_table_host_shape`가 발행하며, 저장 줄 단축의 조기 반환이 그 경로를 건너뛰었다. 높은 보조값으로 이 누락을 면제하지 않는다. 실행 기록은 `candidate-ca98ce5e8-native-check.json`, `candidate-ca98ce5e8-approval-existing-harness.json`, `candidate-ca98ce5e8-handbook-boundaries.json`과 같은 head의 대표 패널에 있다.

다음 후보는 확정된 단일 저장 TAC 표 앞뒤의 흐름 비점유 장식을 원래 컨트롤 순서대로 같은 `place_table_host_shape`에서 발행한다. 별도 줄 예약을 추가하지 않고 표 계획의 pen을 host 앵커로 전달한다.140쪽의 빈 줄은 기존 source frame 인증과 편집·dirty 제외를 공유하고, 빈 줄의 저장 높이·후행 간격·다음 원점이 정확히 닫힐 때만 물리 전진량을 보존한다. 측정은 그 값을 같은 `CellUnit`에 기록하고 실제 부분 표 배치는 현재 컷이 소유한 그 유닛의 높이를 소비한다. 글자가 없는 compositor 결과를 공간이 없다는 뜻으로 쓰지 않으며, 다른 조각의 빈 줄은 현재 쪽에 다시 소비하지 않는다. 일반 합성·로컬 리셋·중첩 표의 기존 계약은 유지한다. 새 후보의 전체 쪽수·빈 줄 앞뒤 내용·장식·외곽선·정상 대조군 및 Native/fresh WASM은 아직 미검증이며, 새 회귀나 기준값을 추가하지 않는다.

빈 줄 후보 `0ddbb2b420d73390eef068473840e2090be86d68`의 첫 컴파일은 CellUnit 내부 소유 필드를 sibling 모듈에서 직접 읽은 E0616 세 건으로 exit101(100.08초)이었다. 조판 결함의 수정 전 FAIL이나 회귀 실패로 세지 않는다. 후속 `8f5cf20efe4439ed477259873ad7f02ebdf6bf50`은 소유·컷 조회를 생산자 모듈의 `stored_empty_line_advances_for_cut`로 캡슐화했고, 해당 저장 프레임의 가로 글줄만 같은 측정·배치 계약으로 소비한다. 새 컴파일·실제 출력은 진행 중이며 결과를 선행 PASS로 표시하지 않는다. 실패 로그와 새 실행 기록은 `candidate-0ddbb2b42-cli-build.log` 및 `candidate-8f5cf20ef-native-check.json`에 보존한다.

8f의 Native 빌드는567.33초 exit0으로 끝났다. 편람 HWPX는 실제383쪽이며, 기존 미달 경계16쪽을 새로 출력해 비교했다.116쪽98.54465%,140쪽96.66997%,370쪽96.40865%,373쪽98.59779%이고, 나머지12쪽(126/135/152–156/223/287/289/290/375)은90% 미만이다. 선택16쪽의 결과를 전체383쪽 통과나 편람 fresh WASM 통과로 보고하지 않는다. 직접 읽은116쪽에서 화살표 장식은 복원됐으나 선·화살촉 차이가 남고,140쪽에서는 빈 줄과 본문 순서가 맞아졌지만 표의 마지막 하단이 여전히 짧다. 높은 점수로 실제 외곽선 차이를 면제하지 않는다. 정확한 실행·TSV·대표 패널은 ignored `candidate-8f5cf20ef-handbook-boundaries.json` 및 같은 head의 경계 폴더에 보존한다.

사용자가 개별 수정마다 빌드를 반복하지 않도록 요청했으므로, 다음 제품 빌드 전에 남은 원인을 묶어 분석한다. 현재 원문/실행 대조는 다음과 같으며 아직 구현·수용 완료 판정이 아니다.

| 경계 | 독립 원문·출력과 실제 경로 | 현재 판정 |
| --- | --- | --- |
| 135쪽 | section3/문단47의 전체 표다. 앞 안내 줄은vpos4000, textheight1000, spacing600이고 다음 그림 줄은vpos5600이다. 두 줄의 저장 line_height는21126HU로 같다. 문단 전체 그림 높이로 줄 소유를 추측하는 renderer/mod.rs와 table_layout.rs corrected_h, paragraph/format.rs의 같은 높이 두 줄 처리 및 실제 paragraph_layout.rs 배치를 함께 추적한다. | continuation으로 분류하지 않는다. 안내 줄의 실제 슬롯과 그림 소유를 공통 결과로 만들 경계이며 새 코드 검증은 미검증. |
| 152–156쪽 | section4/문단47의 같은 표다. 실제 Print보다 시작이 앞서 제목과 겹치고, 다음 쪽들도 내용 소속이 앞당겨진다. CENTER 셀의 저장 프레임이 여러 문단에 걸쳐 진행하다 되감기지만 direct_hwpx_cell_has_declared_stored_frame의 CENTER 단일-reset 조건이 일부 프레임을 제외한다. | 로컬 vpos=0 문단인 #6035 대조군과 실제 문단 간 좌표 진행을 구별해야 한다. 원점을 숫자로 당기거나 행 높이만 줄이지 않는다. |
| 223쪽 | section7/문단239의 실제 본문 셀은vpos1000에서 시작하고 두 후속 페이지 프레임은vpos200으로 돌아간다. 이 셀의 세 프레임 원문 끝과 선언 높이를 기록했다. 현재 인증의vpos<=0 조건은 양수의 실제 쪽 되감김을 세지 못한다. | 양수 원점의 저장 경계와 문단 로컬 좌표를 구별하는 인증·컷·paint 연결이 필요하다. 임의 reset 허용은 보정 근거가 아니다. |
| 289–290쪽 | section10/문단26 제목 도형은 원문 저장 줄부터2줄이며 curSz.height4066HU다. Print에서는 같은 원문이1줄로 출력된다. 기존 normalized_single_line_inline_textbox_host는 폭 fit과 호스트 높이를 함께 투영하지만 queries/rendering.rs에서 HWP5 프로필만 호출한다. | HWPX의 해당 경로 적용과 같은 폭·높이 인증을 검토한다. 단순 paint collapse로 뒤 표 원점을 남겨두지 않는다. 문단28 바깥 표는6×5이며4×4는 내부 표다. |
| 126쪽 | section2/문단991·994의 TAC 표 앞 장식은 Shape가 아니라 비TAC InFrontOfText 표다. 저장 host 높이는 본체와 위아래 여백의 합이고 첫 host의 다음 저장 원점은 후행880HU까지 닫힌다. | 유일한 흐름 표와 장식 표를 구분하는 경로와 실제 장식 발행을 함께 확인한다.116쪽에서 발견한 장식 누락을 반복하지 않는다. |
| 375쪽 | section12/문단18의 왼쪽 양식은 비TAC InFrontOfText 표이며 메모는 별도 본문41·42다. 표 내부 메모로 분류하지 않는다. 원문에는 그 사이 빈 문단의 양수 저장 슬롯과 간격이 있다. | 본문 빈 줄 점유·실제 후행 원점의 소비를 추가 확인한다. 가시 글자가 없다는 이유로 원문 공간을 없애지 않는다. |

ignored `batch-original-source-native-audit.json`, `batch-original-stored-frame-rewinds.json` 및 `batch-8f-source-diagnostic/provenance.json`에 원문 입력/PDF/실행 파일 해시와 현재 컷 로그를 보존했다. 진단은 기존8f 실행 파일을 사용하며 새 빌드를 하지 않았다. render tree의 기본 진단 프로필을 Print 시각 증거와 구별한다. 같은 미달12쪽의 새 Native raster가 ca98의 raster와 바이트 단위로 같은지도 `batch-8f-ca98-native-raster-identity.json`에서 확인했다. 이전 라벨의 패널을 새로 생성한8f 패널이라고 바꾸어 부르지 않는다. source/회귀/golden 변경이나 새로운 #7445 이관은 이 분석 단계에서 수행하지 않았다.

### 묶음 구현 후보 — 실행 검증 전

135쪽 안내 줄의 공통 생산자는 `renderer::stored_picture_guide_line_height_px`다. 원본/구성 줄 수가 같고, 현재·다음 원본 줄이 비합성·미편집이며, 현재 textheight와 spacing이 다음 원점을 정확히 닫고, 실제 control-to-line 소유로 다음 줄의 TAC 그림이 같은 저장 높이를 소유할 때만 안내 줄의 textheight를 반환한다. 현재 줄의 TAC 그림·도형·표, 같은 text_start, dirty/reflow 줄은 이 단축을 쓰지 않는다. HeightMeasurer의 문단/셀 경로4개와 table_layout의 corrected_h, paragraph/format의 예약, paragraph_layout의 실제 줄 높이가 같은 값을 소비한다. 문단 전체 개체 높이를 앞 줄 소유의 대용으로 쓰지 않으며 원문을 수정하지 않는다. 실제 입력의 적용 여부와 수정 전후 출력은 아직 실행으로 확인하지 않았다.

HWPX CENTER/양수 원점 프레임은 선언 span과 문단 사이 좌표 진행을 추가 인증한다. 진행 후 실제 원점이 작아질 때만 physical reset을 만들고, 큰 그림 줄 상자와 다음 줄의 겹침을 페이지 reset으로 보지 않는다. 단순 문단 로컬 vpos=0에는 진행 증거가 없어 새 인증을 쓰지 않으며 #6035 기존 대조군의 기대값은 바꾸지 않았다. 기존 HWP5 분기는 유지했다.

도형의 기존 폭 fit·자동 높이·호스트 투영은 HWPX에도 같은 인증을 거쳐 적용한다. 본문 빈 원본 글줄은 format 결과의 전량 전진을 사용해 다단의 후행 간격 손실을 막는다.152쪽 제목처럼 다음 비TAC 흐름 표의 원점이 원본 줄 끝·줄간격·현재 뒤 간격·후속 앞 간격을 정확히 닫는 경우에도 같은 전량 전진을 소비한다. `FormattedParagraph`에 흐름 전진 보존 상태를 분리했으며 fit 높이나 paint 원점을 위장해 바꾸지 않았다. 기존 unit test 생성 helper에는 새 상태의 기본값만 넣었고 새 회귀/fixture/golden은 추가하지 않았다.

이 후보는 수정 파일의 rustfmt와 `git diff --check`만 통과했다. 제품 빌드·집중 회귀·Native/fresh WASM·최종 lint는 아직 수행하지 않았다.140쪽의 내용 컷과 별개인 선언 물리 상자 잔여,126쪽 장식 표의 실제 발행 경로를 더 연결한 뒤 동일 묶음의 Native/WASM 산출물로 경계와 대조군을 함께 검증한다. 같은 작업의 수정마다 빌드를 다시 시작하지 않는다.

140쪽의 추가 연결은 기존 `saved_single_cell_opening_frame_height`를 사용한다. 원본 전체 셀의 진행·되감김·선언 span이 인증되고, 실제 첫 컷의 끝과 다음 프레임이 일치할 때 선언 첫 상자의 빈 하단도 예약한다. 마지막 슬롯이 빈 원본 줄이어도 글자 소유와 물리 공간을 구별한다. 기존 정확한 잉크 끝+안 여백 계약은 유지한다. 같은 생산 결과를 `fragment/budget`의 선행 예산, `row_step`의 실제 수용 높이와 row override, `emit`의 기존 `stored_row_box_sum` 및 다음 `start_row_height_override`, `row`의 후속 요구 높이와 부분 표 paint가 소비한다. 이미 소비한 내용 컷으로 과거의 물리 높이를 재추정하지 않는다. 단일 셀에 새 그림·줄·baseline을 추가하지 않았다. 첫 상자보다 원문 잉크가 크거나 dirty/재조판/로컬 리셋·중첩 셀이면 새 인증을 쓰지 않는다.

126쪽의 비TAC 장식 표는 flowWithText=false·글 앞/뒤·caption/각주 없음일 때 유일한 TAC 표와 구분한다. `prepare_computed`의 저장 원점·실측 높이·후속 줄 닫힘 인증을 유지하며, 장식은 일반 `place_decoration_table`에서 원래 순서로 발행한다. 그 경로의 포맷·continuation 조회도 확정된 host pen을 쓴다. 실제 deferred shape paint는 원본 한 줄의 유일한 inline 소유자가 전달한 같은 pen을 소비하므로, 장식이 뒤에 있다고 흐름 끝점으로 앵커를 바꾸지 않는다. 재조판·다줄·복수 inline 소유자는 기존 경로를 쓴다. 남은 경계와 결재 원본 대조군은 이 묶음의 고정된 실행 파일과 fresh WASM으로 함께 비교한다. 현재 기록은 구현 연결이며 실행 통과나 전체383쪽 승인으로 세지 않는다.

묶음 head `89e3fb88dffaaf6185b4971924aff8805f807e3e`의 manifest 준비·전체 fmt는 exit0이다. Native 컴파일은 셀 측정 세 번째 경로의 실제 줄 인덱스li를 i로 연결한 E0425 한 건으로 exit101(77.72초) 종료됐다. 실제 출력·회귀·WASM은 아직 실행되지 않았고 이 실패를 조판 회귀 또는 음성 대조로 세지 않는다. 각 경로의 iterator 인덱스를 대조해 연결 오류를 수정했으며 동일 묶음 전체를 다시 검증한다. 실패 원본과 명령/exit는 ignored `candidate-89e3fb88d-batch-check.json`, `candidate-89e3fb88d-native-check.json`, `candidate-89e3fb88d-cli-build.log`에 보존한다. 공유 target을 지우거나 새로운 target을 만들지 않았다.

묶음 제품 `bc254b8a5`의 Native/fresh WASM 경계 실행은 완료됐다. 두 backend 모두383쪽, 비교한 이전 미달16쪽 중135/154/155/156/223/287/375쪽7개만90% 미만이며 최저66.67882%다. 결재 원본5개 전7쪽은 두 backend 모두최저99.61781%를 유지했다. Native140쪽의 실제 마지막 외곽선은 Print와 맞아졌으나 위7쪽과 최종 전쪽/회귀/lint는 아직 보류다. 상세 실행·직접 판독과 #7611 새 보완의 source 우선/신규 검사 보류 및 순서 지정 rebase 결과는 [#7611 기록](pr_7611_review.md)의 마지막 절에 연결한다. ignored 실행 자료의 source는bc이며 이후 rebase head의 검증으로 승격하지 않는다.

### 남은 경계의 두 번째 묶음 — 2026-10-09, 실행 중

최신 base `79abd9e49675f4286eaed38197f7e837afb744e7` 위의 제품 후보는 `5e31d6db0bc73394d1d05f7e05a2e18b63d512dc`다. #7611 새 source가 기존 #7611 끝과 #7617 시작 사이에 들어가고, 원 053 커밋이 메인터너 보정에 앞서는 순서는 유지했다. 아래는 구현 연결과 이전 후보의 실패 관측이며 새 후보의 통과 판정이 아니다.

| 경계 | 원본·독립 출력·이전 관측 | 공통 생산 결과와 실제 소비 경로 |
| --- | --- | --- |
| 135쪽 | 그림 앞 문단의 before 500HU와 가로 여백이 실제 TextLine 상자에 반영됐지만 폴백 그림은 vpos에서 원점을 다시 만들었다. 그림이 Print보다 약6.67px 위·왼쪽이었다. 최종 원본 빈 줄 끝34626HU+안 여백282HU는 개체 높이34908HU를 정확히 닫는다. | 기존 `stored_inline_content_frame`의 원본·컨트롤 소유·정수 닫힘 인증을 HWPX의 HeightMeasurer와 실제 셀 배치가 함께 소비한다. 완전 셀 인증이 없는 경우도 `stored_picture_guide_line_height_px`가 다음 그림 줄의 소유를 증명한 경로는 이미 발행한 해당 TextLine 상자를 사용한다. 임의 좌표 보정이나 모든 그림의 강제 배치는 하지 않는다. |
| 154–156쪽 | 이전 HWP/HWPX는 같은 내용 컷을 선택했지만 HWPX의 비끝 물리 조각이 더 길어 마지막 잔여가 누적해 짧아졌다. source/PDF 괘선·내용은 `batch-bc254-native-pdf-baseline-audit.json`, `batch-bc254-native-pdf-rule-audit.json`과 `batch-bc254-source-diagnostic/{hwp,hwpx}-row-drift.log`에 있다. | `stored_table_has_physical_text_frames`는 기존 Native 또는 깨끗한 HWPX의 공유 축·진행·실제 되감김·선언 span을 인증한다. `fragment/budget`과 `table_partial`의 그리기는 기존 `rowbreak_fragment_frame_bottom`이 정한 같은 물리 하단을 사용한다. 스캔의 실제 수용 높이와 `stored_row_box_sum`이 다음 잔여를 전달한다. 내용 컷과 빈 물리 밴드를 섞지 않는다. 새 수치 허용치를 넣지 않았다. |
| 154쪽 셀 내부 ColumnDef | row5 본문은 단 설정이1단으로 바뀌며 vpos가0으로 돌아간다. 실제 Print에서는 같은 물리 행 안에 있는 로컬 축 전환이다. 뒤쪽 텍스트의 약8.5px 간격 차이는 별도 관측이며 물리 쪽으로 분류하지 않는다. | 진행 프레임 인증에서 ColumnDef가 동반한 문단 간 로컬 되감김을 페이지 인증으로 세지 않는다. 이 반례의 간격까지 해결됐다고 주장하지 않으며 새 출력에서 다시 확인한다. |
| 223쪽 | body 셀의 원점1000HU와 후속 두 프레임200HU는 선언 높이를 닫아 인증됐지만 local-origin gate가 후속 reset/cut 소비를 껐다. 앞 프레임의 마지막 내용이 다음 쪽으로 넘어가고 본문·뒤 문단이 밀렸다. | 인증된 `direct_hwpx_stored_frame_cell`을 원점 및 실제 reset-before·tail·hard-frame 계산이 공유한다. 양수 origin 수용의 기존 숫자 상한을 높이지 않았다. 원본 소유·누락·중복은 새 출력의 해당 프레임들과 함께 확인한다. |
| 287쪽 | section10의 첫 표 원점3900HU+개체19883HU+바깥 아래566HU=다음 표 원점24349HU다. `dump-pages`는 같은 원점·개체 높이·후속 표 소유를 확인한다. 이전 후보는 닫힌 바깥 상자 예약 뒤 `finish_paragraph_float_flow`에서 host after를 다시 더해 다음 표 글줄을 Print보다 약13.4px 아래에 배치했다. | `empty_float::prepare`는 복구 여부와 관계없이 인증된 높이를 실제 수용 원점에서 투영한 후속 줄 원점을 전달한다. `ParagraphFloatPlacement::paragraph_end`는 그 원점이 점유 끝을 닫을 때 host after를 중복 가산하지 않는다. state의 흐름 마무리와 layout의 마지막 원점 소비가 같은 결과를 쓴다. 인증이 없거나 점유 끝과 다른 successor는 기존 간격을 유지한다. |
| 375쪽 | 원본 overlay 표 host 뒤의 빈 글줄은 LH+LS가 다음 저장 원점을 연속해서 닫는다. HWPX의 section entry가 HWP5 전용 gate 때문에 이 글줄들을 제외해 별도 본문 메모가 약257px 위에 놓였다. | 기존 `stored_overlay_guide_owns_flow`의 깨끗한 연속 원본 인증을 HWPX에도 적용한다. section entry → paragraph의 전체 advance → layout의 overlay host page-base 유지가 같은 사다리를 소비한다. 비연속·합성·편집·dirty 사다리는 원본 공간 인증에서 제외한다. |

수정 파일 rustfmt와 `git diff --check`는 통과했다. 공유 `target/pr-review`·jobs8의 Native 1회와 fresh WASM 1회로 결재 원본5개 전7쪽 및 편람의 이전 미달16쪽을 함께 실행했다. 이 경계 실행은 전383쪽 승인이나 최종 회귀/lint의 대체가 아니다. source SHA·명령·실행 파일·입력/PDF/글꼴 해시는 ignored `candidate-5e31d6db0-{batch,native,wasm}-check.json`과 각 로그에 기록한다. 아직 새 렌더링 회귀·fixture·golden·기준값 변경을 추가하지 않았으며 공개 PR·push·승인은 수행하지 않았다.

#### 두 번째 묶음의 실제 결과와 후속 원인

제품 source `5e31d6db0`의 Native 빌드는 exit0(568.69초), fresh WASM 빌드는 exit0(296.51초)다. 결재 원본5개·전체7쪽 Native는 최저99.61781%로 모두90% 이상이다. 편람 집중16쪽은135쪽99.48977%,155쪽99.30465%,156쪽98.60223%로 개선됐지만154쪽89.76118%는 미달이다. **전체384쪽으로 기준383쪽과 달라 수용 불가**다. 최신135/154/223쪽 review PNG를 직접 열어 그림 원점 개선,154쪽 간격 차이와223쪽의 한 줄만 남은 조각을 확인했다. `candidate-5e31d6db0-handbook-remaining-panels/`의 나머지 패널은 직접 판독 완료로 기록하지 않는다.

`batch2-page-count-audit/{before,after,provenance}.json`의 같은 입력·compat2022 비교에서 구역7만52→53쪽이다. 표 host239의 첫 컷은두후보 모두 `[1,10,1]`, 두 번째는 `[1,43,1]`이다. 새 후보는 다음에 `[1,44,1]`까지 한 줄만 따로 넘겼다. 첫 reset은 unit10(문단8), 다음은 unit44(문단29)이며 유닛·문단·저장 줄 순번을 혼동하지 않는다. `batch2-frame-budget-diagnostic/`의 실제 CLI 로그는 두 번째 조각 budget739.453px, 마지막 줄 전 누적718.880px, 그 줄 높이22.027px를 기록한다. 마지막 spacing452HU(6.02667px)를 제외하면 가시 끝734.880px가 들어가지만, 기존 trim은 원점0만 받아 양수200HU의 인증된 프레임을 놓친다. 한 쪽이 늘어난 뒤의 같은 쪽 번호 비교 점수에는 내용 소유가 밀린 영향이 포함된다.

후속 구현은 인증된 양수 source frame의 끝 spacing을 기존 `stored_page_reset_cut_trailing_trim → row_cut_with_start의 capacity/최종 높이 → cell_cut_visible_height/row_cut_content_height → table_partial` 경로에 전달한다. 일반0원점·중첩·rowspan 제외 조건은 유지한다. 같은 physical-frame 인증을 `native_text_frame_first_fragment_saved_top → block/prepare.source_text_first_frame_height → fragment/budget와 layout partial paint`, `native_text_frame_terminal_successor_gap → fragment/emit와 layout 최종 cursor`에도 연결한다. 이 보완은 실행 검증 전이며 새 head의 쪽수·원본 보존·독립 PDF 비교가 필요하다. 154쪽 ColumnDef를 포함한12문단의 원본 XML/style은 `batch2-column-axis-source.json`에 보존했다. 원문은 문단 앞/뒤 간격0·sameGap0이며 관측8.5px를 구현 상수로 사용하지 않는다.

WASM 비교는 첫 원본 export에서 `setHangul2024Compat` 부재로 중단됐다. 새 package와 `wasm_api.rs`에 export가 없고 Native의 `DocumentCore::set_hangul2024_compat`만 있다. 같은 core에 위임하는 공개 setter를 추가하고 rebuild 뒤 실제 Chrome export로 검증한다. 오래된 cache 또는 문서 회귀로 분류하지 않는다. Windows pdfinfo의 비UTF8 Title은 decoding만 교정했으며 수정 전실패/후통과 및 원본2/2쪽 확인을 `batch2-pdfinfo-{before,after}-fix.log`에 남겼다. Visual Sweep 도구105개는 두 OS 경로 assertion을 native `Path`로 교정한 뒤 모두통과했다(`batch2-visual-sweep-tool-tests-after-path-fix.log`). 원본 렌더링 검사·baseline·90% 기준을 완화하지 않았다. 최신source 전체 회귀·세 Clippy·전체 Native/fresh WASM은 아직 미완료다.

#### 후속 묶음 `3483c17f4`의 실행 결과

제품 source는 `3483c17f43b06516b590c495e4d96bb77c523330`, base는 `79abd9e49675f4286eaed38197f7e837afb744e7`다. 준비된 소유 review worktree의 전체 fmt가 exit0이고, jobs8·공유 target의 Native 빌드는570.17초 exit0, fresh WASM도 exit0이다. Native 실행 파일 SHA256은 `a0af71c560ade8c620f5e1f9cb85326b184c0021a35d6e3acdfbae6a06519b72`다. 실제 Chrome export에서 새 compat setter를 사용했고 결재 원본5개·전체7쪽 Native/fresh WASM 모두 최저99.61781%를 유지했다. 명령·source/입력/PDF/font/package 해시와 결과는 `candidate-3483c17f4-{native,wasm,batch}-check.json`에 보존한다. controller의 exit0은 단계 실행 완료이며 시각 gate 통과를 뜻하지 않는다.

편람은 Native/fresh WASM 모두 전체383쪽으로 복구됐다. 기준 Print에 맞는 `--compat 2024`의 집중16쪽에서 두 backend의 점수는 같고,223쪽99.66604%,287쪽99.03899%,370쪽97.60690%,373쪽99.57694%다. 미달은154쪽89.76118%와375쪽83.30378% 두 곳이다. `candidate-3483c17f4-handbook-compat2024-boundaries.json` 및 `candidate-3483c17f4-wasm-handbook-hwpx-compat2024-boundaries/provenance.json`의383쪽·generation2024·누락0을 확인했다. 앞서 default2022로 실행한16쪽은 별도 진단 기록으로 유지하며 같은 세대의 최종 검증으로 대체하지 않는다. `batch2-layout-generation-count-audit/`의bc254/5e31 두 세대 쪽수 대조에서도 기존383→384 실패가 같았다. 최신 전쪽 계획은 `batch3-pages-compat2024.json`에 있다. 이 집중 실행은 전383쪽 TSV·전체 영향 corpus·최종 회귀/lint의 통과가 아니다.

현재 원인 추적은 두 미달을 함께 다룬다. `batch3-3483-native-pdf-baseline-audit.json`의 독립 Print text operator 대조에서154쪽 row5의 ColumnDef 앞4줄은+4.3px, 그 뒤는−4.2px이며 다른 행은0.0~0.2px다. 임의의 전체 행 이동으로 고칠 수 없는 내부8.5px 전진 누락이다. 375쪽은 section12/host43의TAC3×1 서식이며 row1 선언33754HU(450.053px)가 실제454.7px로 성장해 뒤 문단의 차이4.6px와 연결된다. 원시 행합37927HU는 common.height를 닫는다. 저장33201HU 글줄과 TAC 자식 표 및 그 안의 비TAC 손자 표의 높이 소비를 따라가며, 부모 common.height로 출력을 clamp하지 않는다. 해당375쪽 최신 review PNG도 직접 판독했다. 새 rendering 회귀·baseline은 아직 추가하지 않으며 두 미달의 원본 출력 개선이 먼저다.

#### 남은 두 경계의 독립 대조와 구현 후보 (2026-10-09, 실행 검증 전)

154쪽의 단 설정 원인은 원본 section4/host47의 11×2 표를 한 구역으로 추출한 수동 XML 대조군으로 분리했다. 원본 자체는 변경하지 않았으며, 추출본의 상속된 한컴 메타데이터를 새 한컴 저장으로 주장하지 않는다. 실제 2024 한컴 Print(`13.0.0.3901`, `frame_print_to_pdf_ex_one_up`, `pdf_print_method=0`)에서 단 제어가 있으면 이전 마지막 줄→새 첫 줄의 기준선 전진은28.48px, 없으면20px다. 최초 제어 제거본은 저장 줄의 raw UTF-16 위치를 유지해 첫 줄의 줄바꿈도 달랐다. 후속 제거본은 제어8단위를 줄 시작 위치에서도 제거했고, 첫 줄 텍스트가 원본과 같아도20px였다. 숨은 단 제어 run만20pt로 바꾼 대조군은28.48px를 유지했다. 따라서 셀 전체 이동이나 숨은 제어 글자 크기 보정으로 해결하지 않는다. 기록은 `column-axis-independent-print-comparison.json`, `column-axis-independent-print-comparison-v6.json`이며, 실제 계산 규칙은 아직 미검증이다. 진단 입력의 XML encoding을 `utf8`로 잘못 선언한 두 v5 작업은 변환 전에 실패했고, `utf-8`로 고친 v6의 두 작업은4쪽 Print에 성공했다. 이 실패를 원본 결함이나 #7445 이관 근거로 세지 않는다.

375쪽은 최신 불변 실행 파일 `3483c17f4`의 `dump-pages --compat 2024`와 원시 XML을 대조했다(`batch4-parent-height-diagnostic/`, `batch4-child-measure-diagnostic/`). 외곽 표의 가운데 행은 선언450.1px→측정454.7px이며, 그 안의 11×14 표 마지막 셀은 선언14360HU다. 이 셀의 마지막 빈 호스트 줄은 vpos11432HU·LH900HU이고, 문단 기준 비TAC 손자 표의 몸통2364HU와 바깥 여백282HU는 같은 호스트 원점부터2646HU를 점유한다. 실제 점유 끝은11432+2646=14078HU, 셀 안 여백282HU까지14360HU로 정확히 닫힌다. 기존 압축 사다리 분류의 가산 경로는 앵커 글줄900HU를 먼저 더해12332+2646=14978HU로 계산한다. 이 이중 계상 때문에 자식 표를618HU 더 크게 측정하고, 부모의 선언33754HU를347HU(4.62667px) 초과한다.

구현 후보는 `float_placement::stored_block_cell_flow`가 후속 문단 없는 마지막 block host도 저장 앵커 줄과 비교하도록 하고, HeightMeasurer의 행 요구 높이·MeasuredCell 내용 높이가 그 흐름 계획을 먼저 소비하도록 연결한다. 실제 셀 배치는 이미 `sequential_nested_cell_layout`을 통해 같은 helper의 문단·개체 원점을 소비한다. 정상 저장 사다리, 한 줄의 빈 block host, 문단 기준 TopAndBottom 비TAC 표라는 기존 인증을 유지하고, TAC·Square·caption·dirty/합성 줄은 해당 계획에 포함하지 않는다. 부모 선언 높이에 맞춘 clamp나 픽셀 상수는 추가하지 않는다. 이 후보는 `351de6e06`에 저장했으며 수정 후 실제 출력·대조군·fresh WASM·전체 회귀는 아직 실행하지 않았다.

154쪽의 추가 대조는 `column-axis-independent-print-comparison-v7.json`과 `-v8.json`에 있다. 셀 높이를2000HU 늘리거나 전체 저장 줄을 제거해 한컴이 재조판하게 해도10pt 본문의 기준선 전진은28.48px였고,20pt 본문에서는41.76px였다. 20pt 대조군은 이전 문단의 마지막 `다.` 줄을 사용해야 하며 `0.5`가 들어간 앞줄의 기준선을 마지막 줄로 세지 않는다. 따라서8.48px를 고정 추가하는 보정은 반례를 통과하지 못한다.

[한컴의 단 너비 및 간격 도움말](https://help.hancom.com/hoffice100/ko-KR/Hwp/format/columns/columns(width).htm)은 다단 전환 간격을 구역 설정에서 지정한다고 설명한다. 원본 section4의 `secPr.spaceColumns`는1134HU다. v9 수동 XML 대조군의0/2268 변경은 Print에 반영되지 않았고, 이 PC의 한컴 COM `PageSetup.GetDefault` 및 import 후 HWPML 덤프에서는 모두1134HU로 관측됐다(`column-context-native-com-probe-v2.json`, `*-native-import.hml`). 입력 XML의 숫자만으로 실제 로드된 설정을 주장하지 않는다.

실제 한컴 `PageSetup` 액션의 `SpaceBetweenColumns`를0/2268로 변경·readback·새 파일 저장한 대조군은 `column-context-native-save-v10.json`에 보존했다. 해당 로컬 COM의 저장 제품은2020(`11.0.0.9136`)이므로2024로 추정하지 않고2020 Print로 보냈다. 두 입력 모두4쪽 정상 Print이며 기준선 전진은13.44/43.52px다(`column-axis-independent-print-comparison-v10.json`). 원본10pt 글줄 높이1000HU와 전환 간격0/2268HU의 합13.333/43.573px를 인쇄 좌표 정밀도 안에서 확인한다. 원본2024의1134HU 설정은28.453px로 앞선28.48px 관측과 연결된다. 수동 추출본과 네이티브 재저장 대조군은 원본383쪽의 시각 통과를 대신하지 않는다.

`7c9f55a6992ab0af0db353efceb7e625bb505f27`은 원본 저장 줄을 사용하는 텍스트 전용 셀에서 한 단 설정의 로컬 축을 공통 `StoredSingleColumnCellFlow`로 연결한다. 앞 밴드의 실제 줄 상자 끝에 source 구역 간격을 더하고, 다음 밴드의 저장 줄 위치를 그 원점에 더한다. 원본 row5의 전환 문단은5500+1134=6634HU, 전체 내용 끝은6634+17500=24134HU다. HeightMeasurer의 행/MeasuredCell, Layout의 세로 정렬/실제 문단 원점, CellUnit의 분할 advance가 동일한 줄 원점·점유 끝을 소비한다. DocumentCore의 일반/선택적/지연 측정과 Typeset의 scratch LayoutEngine에 같은 구역 값을 전달하고 값이 바뀌면 유닛 cache를 무효화한다. 구역 값이 없는 독립 helper 호출, 여러 단, 객체 혼재, 명시 쪽/단 나누기, dirty/합성/역행 줄은 기존 경로에 남는다. 새 source를 포함한 전체 재조판·영향 corpus·편집 저장본의 Print 검증과 회귀/lint는 아직 미완료이며, 이 두 보완을 한 번의 Native/fresh WASM 빌드로 먼저 확인한다.

#### 후보 `7c9f55a69`의 실행 결과와 실제 배치 경로 누락

공유 target·build jobs8의 Native 빌드는637.58초 exit0, 잠금 파일0.2.127의 fresh WASM 빌드는368.81초 exit0이었다. 준비 및 전체 fmt 검사도 exit0이다. 원본 결재문서5개·전체7쪽은 Native/fresh WASM 모두 최저99.61781%이며, 첫 원본의5개 그림은 실제 소유 셀 내부에 있다. 기록은 ignored `candidate-7c9f55a69-{native,wasm,batch}-check.json`과 각 명령 로그에 있다. controller의 exit0은 비교 실행의 종료이며 시각 gate 통과가 아니다.

편람 원본 HWPX는 Native/fresh WASM 모두383쪽이다. compat2024로 비교한 기존 미달16쪽 중15쪽은90% 이상이지만,154쪽은89.76118→84.67500%로 악화되어 gate가 미충족이다.375쪽은83.30378→92.52421%다. 두 backend의 점수는 같고 누락은0쪽이다. TSV와 package/source/입력/PDF/font 해시는 `candidate-7c9f55a69-handbook-compat2024-boundaries.json`, `candidate-7c9f55a69-wasm-handbook-hwpx-compat2024-boundaries/provenance.json`에 고정했다. 전383쪽 시각 비교·최종 회귀·세 Clippy·개별7개 PR의 최종 검토는 아직 미완료다. 새 렌더링 회귀·baseline 추가와 공개 PR 생성·push·승인은 수행하지 않았다.

`batch4-7c9-native-pdf-baseline-audit.json`에서154쪽 앞4줄은 Print와0.0px 차이로 맞지만 전환 후 줄은−8.5px다. `batch4-cell-flow-path-diagnostic/provenance.json`은 새 불변 CLI로 실제154쪽 `export-render-tree --compat 2024`를 실행한 exit0 기록이다. 해당 host47/셀(5,1)은 `table_partial.rs`의 paragraph paint를 사용하며 `cut=None`인 온전한 셀이다. cp0/1/2의 실제 문단 원점은306.6/346.6/386.6px로 순차 전진해 단 전환 간격이 없다. 일반 horizontal-cell path에 새 앵커를 연결한 것만으로 실제 소비가 끝났다는 앞 설명은 불충분했다. 공통 결과로 측정·정렬은 바뀌었으나 이 partial-table paint 소비가 빠졌다. 후속 보완은 같은 source 밴드 원점을 이 실제 paint에 연결하고, 컷이 있는 경우에는 현재 조각의 첫 소유 줄과 함께 원점을 해석해야 한다. 출력 전체를 이동하거나 관측8.5px를 상수로 추가하지 않는다.

375쪽은 최신 Native raster와 독립 PDF를 직접 열어 오른쪽 표 하단 및 뒤 본문의 높이 복구를 확인했다. `batch4-post-fix-height-diagnostic/provenance.json`의 same-source `dump-pages --compat 2024`도 exit0이다. 남은 좌측 양식의 약1.9px 위치 차이와 상단 쪽번호 추가 표시는 별도 미해결 관측이며, 높은 실루엣 점수로 누락·중복 계약을 면제하지 않는다. 두 경계의 compare/standalone overlay/review는 ignored `candidate-7c9f55a69-handbook-boundary-panels/`에 보존한다. 사용자가 이번 실행을 끝낸 뒤 최신 `upstream/devel` 동기화와 리베이스를 요청했으므로, 완료 결과를 먼저 커밋하고 그 다음 base 정렬 및 보완을 진행한다. 기존 기록·원본·독립 PDF·공유 cache를 보존한다.

#### 최신 base 위의 두 소비 경로 보완 — 실행 검증 전

최신 base `21ed64071`의 순서 보존 rebase를 끝낸 뒤 `b06e963f9`은 온전한 셀이 partial-table 안에 놓이는 실제 경로에도 `stored_single_column_cell_flow`의 문단/줄 원점을 전달한다. 부분 셀은 현재 컷의 첫 소유 줄을 같은 밴드 좌표계의 조각 원점으로 사용하며, 첫 조각의 leading band와 문단 앞 간격은 한 번만 소비한다. 기존 문단/개체 소유 컷을 바꾸거나 줄을 숨기지 않는다. 그림·중첩 표·dirty/합성 등 공통 생산자가 제외한 경로는 해당 계획을 사용하지 않는다. 실제154쪽의 `cut=None` 경계와 컷이 있는 대조군의 실행 확인은 아직 미검증이다.

375쪽의 상단 쪽번호는 원본 XML과 실제 Print로 별도 분리했다. 원문 전체에는 section11의 EVEN 머리말, ODD 꼬리말, 빈 EVEN 꼬리말만 있으며 ODD/BOTH 머리말은 없다. 홀수 표시 쪽367의 Print text operator에는 하단 쪽번호 하나만 있지만 이전 실제 render tree에는 Header/Footer 양쪽에 `－ 367 －`가 있다. `batch4-hf-parity-source-diagnostic.json`에 입력/PDF/tree hash·원본 applyPageType·Print 기준선681.92px·수정 전 노드를 보존했다. `0a8ac9f82e3123637b5f3741feb9f5d56dd735b7`은 구역 간 kind 상속의 반대 홀짝 fallback을 제거하고 같은 홀짝 슬롯만 소비한다. BOTH 정의는 기존 carry 갱신에서 두 슬롯에 각각 기록되며, 현재 구역의 슬롯별 재선택과 이전 구역의 같은 슬롯 상속은 유지한다. 임의로375쪽의 Header를 숨기는 처리가 아니다. 작성 당시 이 source의 Native/fresh WASM·홀짝 정상 대조군·전체 회귀와 최종 lint는 미검증이었다. 아래 실제 실행 결과에서 완료한 범위만 갱신한다.

#### 최신 후보의 영향 쪽 검증과 홀짝 대조 — 2026-10-10

제품 source `0a8ac9f82e3123637b5f3741feb9f5d56dd735b7`, base `21ed64071f81f35f37618717693bd30f727392c9`를 고정했다. 같은 공유 target과 build jobs8에서 Native CLI 빌드는481.66초 exit0, fresh WASM은129.53초 exit0이며 fmt check도 exit0이다. 이전 source의 산출물을 최신 source의 결과로 재사용하지 않았다. 원본 결재문서5개·전체7쪽은 Native/fresh WASM 모두 최저99.61781%, 미달·누락0쪽이다.

편람 원본 HWPX는 두 경로 모두383쪽이며 기존 미달16쪽의 Native/fresh WASM 최저92.77642%, 미달·누락0쪽이다. 154쪽은 수정 전84.67500%에서98.48451%,375쪽은92.52421%에서92.77642%다. `candidate-0a8ac9f82-handbook-compat2024-boundaries.json` 및 `candidate-0a8ac9f82-wasm-handbook-hwpx-compat2024-boundaries/provenance.json`에 입력·Print PDF·font·package/source hash와 전체16쪽 TSV를 연결했다. 이 결과는383쪽 전수 비교의 완료 판정이 아니다.

같은 후보의154·375쪽 Native/fresh WASM compare·standalone overlay·review4개를 새로 산출하고 모두 직접 열었다(`candidate-0a8ac9f82-handbook-boundary-panels/`). 154쪽은 셀 내부 단 전환 뒤 글줄과 후속 행의 큰 밀림이 해소됐다.375쪽은 오른쪽 서식의 외곽·뒤 본문이 복구되고 상단 중복 쪽번호가 제거됐다. 왼쪽 서식의 약1.9px 위치·획 농도 차이는 남는다. 이 차이를 점수로 면제하거나 전체 일치를 주장하지 않는다.

독립 원문 Print 번호와 실제 두 backend의 render tree를374·375·376쪽에서 대조했다. 표시366·368의 짝수 쪽은 Header 번호만, 표시367의 홀수 쪽은 Footer 번호만 소유한다. 수정 전375쪽의 Header/Footer 중복과 수정 후 Footer 단독을 기록했다(`batch5-hf-parity-check.json`). `displayText`의 실제 자동 번호를 검사했으며 원시`text`의 공백을 번호 부재로 오인하지 않았다. 이 진단은 정식 회귀 검사를 대신하지 않는다.

고정 base를 전달한 manifest48/48 target 검사와 source unit tier4149 tests 검사는 각각 exit0이다(`batch5-base-policy-checks.json`). 원 PR7개는 원격에서 모두 open이며 기록한 contributor head와 동일함을 재확인했다(`batch5-remote-pr-heads.json`). 현재 HWP/HWPX의 Native 전체383쪽 비교를 실행 중이다. 전체 fresh WASM 비교·분할 반례의 정식 회귀·전체 nextest·세 Clippy·실제 출력 backend·각 PR 최종 검토는 아직 미완료이므로 보류 판정을 유지한다. 새 rendering test/baseline과 원격 push·통합 PR은 아직 추가하지 않았다.

#### 242쪽 이월 오류의 공통 끝점 보완 후보 — 2026-10-10

`0a8ac9f82`의 HWP fresh WASM 전383쪽도 종료됐다. 미달은242쪽44.16538% 하나이며, 앞서 완료한 HWP Native44.16228%, HWPX Native/fresh WASM43.99893%와 같은 행 소유 오류다. 실제 전쪽 결과는 `candidate-0a8ac9f82-wasm-handbook-hwp-full/provenance.json`에 보존한다. 4개 조합의382쪽 통과를 전체 gate 통과로 바꾸지 않는다.

같은 원본을 고정된 Native 실행파일 `f97144651`과 `bc254b8a5`로 진단했다. section7 host408의 첫 컷은 원래0..11/이어받기11..13에서0..9/9..13으로 달라졌으며 셀별 줄 높이는 같다. 이전 표405의 컷·이어받기 점유도 같다. 차이는 제목407에서 전체49.3px를 누적한 흐름으로, 다음 표 진입이242.4→275.0px가 됐다. 실제 제목 원점과 표 원점은 두 실행의 paint에서 같지만 이 추가 예약 때문에 두 행이242쪽으로 이월된다. 기록은 `batch6-242-{cut,budget,flow}-history/`에 원본·binary·source hash와 명령·실제 JSON·진단 로그를 연결한다. 전역 내용 중복이 아니라 쪽 소유 오류다.

독립 원본 저장 줄은 제목407의17118HU와1250HU 줄높이·748HU 줄간격·1000HU 뒤 간격으로 다음 host408의20116HU를 정확히 닫는다.152쪽의 제목46도27323+1250+748+1000=30321HU로 후속 host47을 닫는다. 두 제목은 같은 문단 양식이다. 전체 문단 높이를 더하는 기존 boolean은152쪽에는 맞지만 앞 partial-table 뒤 커서의 차이가 있는241쪽에는 과다 예약한다. 이전처럼 무조건 fit 높이만 더하면152쪽의 제목·표 겹침을 다시 만든다.

후속 후보는 `float_placement.rs:stored_paragraph_successor_flow`에서 genuine·비편집·비dirty의 저장 줄과 후속 비TAC 문단 상대 흐름 표가 닫는 콘텐츠 하단/흐름 끝 HU를 함께 만든다. 문서 ID·행 번호·고정 픽셀 보정·clamp를 사용하지 않는다. 재조판 줄 수가 달라진 경로와 다단·합성·저장 줄 리셋은 이 전체 문단 계약 밖이다. page base는 column anchor, lazy base는 body origin, unbased partial-table 저장 사다리는 기존 배치처럼 body-relative 축을 쓴다.

측정 소비는 전체 fitted/atomic-overflow 문단의 공통 끝점 사영 → 실제 cursor 차이 전진 → 다음 표의 budget/cut이며, 콘텐츠 하단도 같은 결과를 기록한다. 절대 끝점과 전체 문단 높이의 차이를 spacing trim 또는 underrun으로 재해석하지 않는다. 실제 배치는 `layout.rs`의 FullParagraph 최종 항목 완료에서 같은 끝점을 같은 축으로 소비하고, 그 뒤 다음 항목 커서를 갱신한다. 빈/partial 문단은 이 전체 문단 계약으로 덮어쓰지 않는다. 이 source 변경의 fresh Native/WASM·전체 페이지 gate·정상 대조군·필수 회귀/lint는 아직 미검증이다. 새 회귀나 baseline은 아직 추가하지 않으며 보류 판정을 유지한다.

#### 공통 끝점 후보의 실행 결과 — 2026-10-10

보완 source는 `711f7f92ee63279754d80e1be9529ae24905f5ad`다. commit 후 upstream/devel을 fetch한 결과 base `21ed64071f81f35f37618717693bd30f727392c9`는 같았고 rebase는 up to date로 종료했다. owned review의 포맷·manifest 준비를 완료하고 공유 target에서 jobs8로 Native CLI635초 exit0, fresh WASM338초 exit0을 확인했다. Native binary SHA-256은 `1bdcf8abb96004b10bb2b0f7086cccc8f558430e05c5bbf7d6d5bd34e7b9c037`다. 소스·입력·독립 PDF·WASM package hash와 실제 명령/로그는 `candidate-711f7f92e-{native,wasm}-check.json`에 있다. 첫 checkout/검증 진입이 보존된 source 복사본 때문에 거부된 경우는 build FAIL이 아니며, 해당 복사본이 commit blob과 같음을 확인한 뒤 owned checkout만 정리하고 검증했다.

HWP/HWPX의 실제 Native page plan은 모두383쪽이다.241쪽 host408은0..11,242쪽은11..13으로 복구됐으며 제목407의 전진은49.3px 고정 가산 대신 실제 끝점까지42.5px다. 다음 표의 진입268.2px·가용465.8px·소비460.1px·endRow11을 실제 로그로 확인했다.152쪽 host47의 시작/끝 컷과153~156쪽의 이어받기 소유도 유지했다(`batch6-711f7f92e-cut-diagnostic/`). 이 진단의 쪽수·컷 복구를 HWP 전쪽 시각 통과로 대신하지 않는다.

원본 결재문서5개 전7쪽은 Native/fresh WASM 모두 최저99.61781%다. 편람 HWPX 영향·정상 대조20쪽은 두 backend 모두 최저92.77642%, 미달·누락0쪽이다.152쪽99.43126%,241쪽98.56603%,242쪽98.37935%,243쪽99.15390%,281쪽99.92654%,375쪽92.77642%다.242쪽의 이전43.99893% 실패와 같은 원본·Print에서 비교했다. 두 backend는 실제 새 Native와 fresh WASM 출력이며 이전 capture를 바꾸어 이름 붙인 것이 아니다. 브라우저 재사용은 canonical 준비·font byte·PNG parity가 확인된 실행 orchestration만 재사용하며 점수·원시 WASM geometry를 바꾸지 않았다.

152/241/242/281/375쪽의 Native/fresh WASM compare·standalone overlay·review10개를 산출하고 모두 직접 열었다(`candidate-711f7f92e-handbook-boundary-panels/`).241쪽의 공개 단계 두 행이 본래 쪽에 남고242쪽은 결과 활용 행으로 시작하며 뒤 본문·표 외곽·하단 번호가 보존된다.152쪽의 제목과 뒤 표는 겹치지 않으며281쪽의 목차 쪽번호와 아래 테두리도 유지한다.375쪽의 왼쪽 서식 약1.9px 차이, 글꼴 획·농도·AA·일부 괘선 두께 차이는 남는다. 선택20쪽을 원본383쪽 전수 통과로 보고하지 않는다. 최신 전체 회귀·lint·다른 PR 경계와 전쪽 시각 검증을 마친 뒤 최종 판정하며 새 rendering test/baseline·원격 push·통합 PR은 아직 보류한다.

### 2026-10-10 통합 제품 source의 전쪽 수치와 직접 판독

제품 source `5538efd3832ab2137e86de5302b4dee7caf392b7`, 고정 base
`21ed64071f81f35f37618717693bd30f727392c9`를 Native·fresh WASM으로 새로 빌드했다.
그 뒤 검사·fixture만 더한 `bea45e807be47d460899a6c62f55e200a0fcd423`는 제품 source/Cargo와 같다.
동일 원본과 대응 2024 Print, 동일 font bytes,96dpi/Print profile에서 다음 전쪽 수치를 확보했다.

| 원본·출력 | 비교/전체 | 최저 실루엣 보조값 | 90% 미만·누락 |
| --- | --- | --- | --- |
| HWPX Native | 383/383 | 90.40200% | 0·0 |
| HWPX fresh WASM | 383/383 | 90.40200% | 0·0 |
| HWP Native | 383/383 | 90.42728% | 0·0 |
| HWP fresh WASM | 383/383 | 90.42728% | 0·0 |

HWPX Native/fresh WASM의 새 전383쪽 raster 중382쪽은 PNG bytes가 같으며1쪽은 다르다.
같은 수치와 대부분의 동일 PNG를 실제 backend·기하 계약 전체의 충족으로 승격하지 않는다.
ignored `candidate-5538efd38-hwpx-full-raster-identity.json`에 동일·상이 쪽을 구별했다.
WASM의 최초 전체 실행은 historical raster parity control의 source SHA를 제품 source SHA와
같아야 한다고 잘못 묶은 orchestration guard에서 중단됐다. source/package·raw geometry·exact
font bytes를 각각 검증하고, 이미 완료된 같은 source의 raw export에서 실제383쪽 raster와 TSV를
완료했다. 중단 record는 `provenance.invalid-control-binding.json`에 보존했다.

새 source의152/241/242/281/375쪽 양 경로 review10개를 직접 열었다.
281쪽 목차의 오른쪽 번호·아래 표 외곽과241/242쪽의 원래 쪽 소속은 유지된다.
추가로 전체 최저 쪽93/130/273의 Native review를 직접 판독했다. 수치90% 이상이어도
그림 좌표 차이가 남는다. 같은96dpi 좌표계의 SVG image와 독립 PDF CTM에서93쪽 두 번째
그림 x는123.28 대138.4107,273쪽 두 번째 그림 y는508.12 대519.4693이다.
상위 SVG transform 여부도 확인했다. 해당 세 쪽의 새 PNG는 f971 및0a의 같은 쪽 PNG와 같지만,
기존 차이라는 분류로 배치 보류를 해제하지 않는다. 실제 소비 경로·원본 선언·기준 출력의
대응을 대조하여 원인과 필요한 보완 범위를 정한다.130쪽과 나머지 낮은 쪽의 괘선·그림 경계도
추가 판독·해석 대상이다. 이 절은 수치 선행 조건과 직접 판독상 잔여 결함을 분리한 진행 기록이다.

실행·source/입력/Print/package hash·전쪽 TSV는 ignored
`output/pr-review/planet6897-20261008/candidate-5538efd38-native-handbook-{hwp,hwpx}-full.json`,
`candidate-5538efd38-wasm-handbook-{hwp,hwpx}-full/provenance.json`에 보존했다.
새 대표 compare·overlay·review는 `candidate-5538efd38-handbook-boundary-panels/`,
낮은 쪽은 `candidate-5538efd38-hwpx-native-full-low-panels/`, 그림 bounds는
`candidate-5538efd38-handbook-low-picture-bounds.json`에 있다.
전체 회귀는 `bea45e807-full-regression.json`/`bea45e807-full.log`에서 종료됐다.
`bea45e807`의 빌드 잡8개·nextest10개, `--no-fail-fast` 실행은10,265건 중10,243통과·22실패·46제외,
nextest exit100이다. 마지막 대형 표 #2063 검사는1,085.469초 뒤 통과했다.
22실패는12개 검사 묶음이며 본문 넘침·텍스트 겹침·쪽수·저장 줄/표 경계·글꼴 메트릭을 포함한다.
같은 source의 Studio 전체 Node 검사는1,827건 중1,813통과·12실패·2제외다.
Vite 모듈 로딩 시간 초과7건과 해당 로딩 실패 뒤 생성자 오류5건을 관측했으며,
동일 source·같은 실패 파일7개를 file concurrency2로 재실행하여54건 모두 통과(exit0,61.06초)했다.
source hash가 같고 실패 파일 전체가 재실행됐음을
`bea45e807-studio-failure-control.json`/`.log`로 고정했다. 최초 로딩 시간 초과의 실행 부하 원인을
의심할 근거이며, 이 집중 대조를 최종 Studio 전체 통과로 보고하지 않는다.
검사명·원본 경로·고정 base 내 검사 존재 여부·assertion·로그 hash는 ignored
`bea45e807-terminal-failure-inventory.json`에 보존했다.

최종 실패 원본의 고정 upstream/devel 시각 근거를 확보하기 전에 baseline을 완화하거나
검사를 임의 제거하지 않는다. 사용자 지시에 따라 같은 원본·독립 Print의 고정 base
Native/fresh WASM 최저값이90% 미만으로 입증된 기존 미달만 기존 이슈 #7445로 이관하며,
90% 이상이면 현재 통합 branch에서 개선한다. 기준 미검증은 이관 근거가 아니다.
원 PR·통합 후보의 머지 보류는 유지한다.

#### 실패 원본의 고정 base 대조와 묶음 보정 — 2026-10-10

고정 base `21ed64071f81f35f37618717693bd30f727392c9`를 실제로 다시 빌드했다.
Native CLI와 fresh WASM의 빌드는 각각 exit0이며, 실행 파일·package·1330개 source 파일의
해시는 ignored `upstream-21ed64071-failure-triage/build.json`에 보존한다. 전자투표 원본
`samples/issue6787/16774617-electronic-ballot-form.hwp`와 독립 Print
`pdf/16774617-electronic-ballot-form-2020.pdf`의 동일 바이트를 비교한 결과, base의 Native와
fresh WASM은 모두 2쪽 최저90.88965%, 미달0이다. base WASM은 새 compat setter가 없으므로,
source에서 확인한 기본2022 계약을 그대로 export하는 비공개 protocol adapter를 사용했다.
2024 WASM 검증으로 보고하지 않는다. 실패한 최초 exporter 실행도 보존했다.

같은 조건의 후보 `5538efd38`은 첫 쪽24.04528%로 카드·사진이 탈락한다. Native base/후보의
1·2쪽 compare/overlay/review를 직접 판독했으며, 2쪽은 같은 배치를 유지한다.
불변 중간 CLI 11개의 실제 render tree에서 `3483c17f4`까지 카드281.6/280.3px와 사진2개가
있지만 `7c9f55a69`부터 카드108.0px와 사진0개를 확인했다. 전자투표의 기존 검사3건은
현재 수정에서 생긴 회귀이며 #7445로 이관하지 않는다.

원본 native parser dump와 진단용 HWPX 직렬화에서 카드 표의 PARA/TOP 오프셋
−20484/−20579HU, 호스트 저장 원점21683HU를 확인했다. 진단용 직렬화를 새 기준 원본으로
사용하지 않는다. `stored_block_cell_flow`는 cursor 아래로 전진하는 상자를 생산하지만
이 부호 있는 앵커까지 받아들였다. 동일 생산자를 소비하는
`HeightMeasurer::cell_stored_block_flow`·`cell_nested_controls_bottom`과
`LayoutEngine::sequential_nested_cell_layout`의 계약에서 위쪽/비TOP 앵커를 제외해
기존 저장 앵커·점유 끝점 경로로 보낸다. 문서 식별자·관측 px 예외를 추가하지 않는다.

다른 두 위치 보정도 같은 묶음에 포함한다. 편람93쪽의 가운데 정렬 인라인 그림 뒤5개
공백은 그림 자체나 앞/사이 간격을 줄이지 않고 기존 후행 공백 측정 결과로 정렬 폭에서
제외한다. 밑줄 공백과 일반 빈 줄 계약은 유지한다. 273쪽의 TAC 도형 줄은 저장 줄 높이가
도형·바깥여백과 일치하고 다음 본문 원점이 저장 줄 끝·후행 간격을 정확히 닫을 때만,
`stored_paragraph_successor_flow`의 같은 끝점을 조판과 마지막 Shape paint가 소비한다.
paint는 이미 전진한 빈 text item 뒤에서 높이를 다시 합하지 않고 원래 문단 진입점으로
투영한다. 이 세 보정의 실행·최종 lint·영향 페이지 시각 결과는 아직 미검증이다.

rowbreak HWPX 18→19쪽과 PrEP HWPX 140→141쪽은 전자투표와 다른 구간에서 시작됐다.
불변 CLI의 동일 원본·2022 출력에서 `02de253c1`은18/140쪽,
`5e31d6db0`은19/141쪽이며, `rowbreak-prep-intermediate-stage-controls/provenance.json`에
입력/실행 파일 해시와 명령을 보존했다. 실제 dump-pages의 첫 구조 차이는 rowbreak14쪽의
호스트16 컷8→7과 추가 이어받기, PrEP105쪽의 호스트336 온전한20행→19행 조각이다.
PrEP의 일부 빈 줄 전진량 변경도 관측했으나 쪽수 원인으로 단정하지 않는다.
두 원본의 고정 base 전쪽 Native 시각 비교는 진행 중이며, 이 쪽수 진단만으로 기존 미달
이관이나 기준값 변경을 결정하지 않는다. 통합·개별 PR의 보류 판정은 유지한다.

#### 묶음 보정 실행 결과와 다음 원인 분석 — 2026-10-10

보정 commit `77d5c12ec9301c530a46d1954e63dfb424a8c23f`에서 review worktree의
파생 suite 준비·전체 fmt check·Native CLI 빌드·locked fresh WASM 빌드를 순차 실행해
모두 exit0을 확인했다. 공유 `target/pr-review`와 빌드 잡8개를 사용했다.
고정 base `21ed64071f81f35f37618717693bd30f727392c9`에 대한 manifest check와
source-side unit tier check도 exit0이다. 세 Clippy·최종 전체 회귀의 완료 증거는 아니다.

| 같은 원본·Print의 보정 경계 | Native | fresh WASM | 미달 |
| --- | --- | --- | --- |
| 전자투표 전체2쪽 | 최저90.88965% | 최저90.88965% | 0 |
| 편람 HWPX 영향10쪽 | 최저90.54980% | 최저90.54980% | 0 |

10쪽은93/94/130/152/154/241/242/273/281/375쪽이다. 이12쪽의 실제 양 경로 raster는
PNG bytes가 모두 같다. Native 대표 review9개를 직접 판독했으며, 전자투표의 카드·사진,
241/242쪽의 행 소속·이어받기,281쪽의 오른쪽 목차 번호·아래 테두리를 확인했다.
93쪽 두 번째 그림 x는138.8px로 Print138.4107px에 가까워지고,273쪽의 캡션 y484.1px와
두 번째 그룹 y519.6px도 Print484.08/519.4693px에 가까워졌다. 작은 획·AA 차이는 남는다.
선택12쪽을 전383쪽·Canvas/Skia 소비 경로·전체 회귀의 통과로 보고하지 않는다.

수치가 통과해도93/130쪽 그림 테두리는 누락된다. 실제 원본 `hp:pic/hp:lineShape`에
93쪽 SOLID/width0,130쪽 SOLID/width34HU가 있지만 `parse_picture`가 이 자식을 적재하지
않는다. 독립 Print93쪽은 두 그림 뒤에0.12pt 검은 stroke를 그린다. 같은 문서273쪽의
그룹 그림에는 lineShape가 없어 정상 대조가 된다. 생산자인 파서·Picture IR에서 기존
`make_picture_border_node`와 각 배치 경로, 출력 backend로 이어지는 경로를 보정 대상으로
추적하며, 관측 좌표·문서 ID를 조건으로 삼지 않는다.130쪽 안쪽 여백과 개체 외곽은
테두리 보정 후 같은 좌표계에서 다시 판독한다.

최초 편람 WASM 실행은 C: 여유 공간0으로 font-embedded SVG 복사 중 ENOSPC exit1이었다.
미완료 SVG 중간 파일만 해시 목록을 남기고 제거해9.79GiB를 확보했다. 실패 로그·raw WASM·
render tree·frozen package는 보존했다. 재실행은 전체383쪽 raw SVG/tree와 동일 입력의
전체 Native font-face 정책을 검증하되 요청한10쪽의 글꼴 포함 복사본만 만든다.
기하·글꼴 bytes·페이지 수·raster·실루엣 계산·gate는 바꾸지 않았으며 실제 재실행 exit0이다.
실패 원문과 전송 adapter의 변경 함수 해시는 ignored provenance에 분리했다.

고정 base Native의 실패 원본 전쪽 비교도 종료됐다. rowbreak HWPX는18쪽 중
4/6/7/8/15/16쪽 미달, 최저32.69054%이며 이6쪽의 review를 직접 판독했다.
사용자가 지정한 기존 이슈 #7445 이관의 base 미달 조건에 해당하지만, 현재 후보의
추가 이어받기와 같은 조건 전쪽 비교·실제 이관은 아직 완료 전이다.
PrEP HWPX는140쪽 전체 최저91.84847%, 미달0이다.27쪽의 기존 차트 색·스타일 차이와
105쪽의20개 예산 행·최종 합계,108쪽의13개 참고문헌을 직접 판독했다.
PrEP 관련 실패5건은 #7445로 이관하지 않고 현재 branch에서 보정한다.
base fresh WASM에2024 setter가 없으므로 이140쪽 결과를 base WASM2024로 보고하지 않는다.

PrEP105쪽의 저장 표 원점과 바로 앞 제목의 공유 끝점은11181HU로 정확히 같다.
독립 Print의 표 윗변243.2533px와 base 표243.5px도 이를 뒷받침한다.
`saved_rowbreak_object_frame`의 HWPX 조건이 `source_top < current_height`여서 같은 원점을
제외한 뒤 host 후행 간격까지 포함한790.2px를 일반 whole-fit 예산에 넣고,20행 중 마지막
행을 다음 쪽으로 보낸다. 같은 저장 프레임의 수용·예약·최종 배치 경로와 multi-page
반례를 대조한 뒤 수정하며, 제목의 유효한 끝점을 되돌려 쪽수만 맞추지 않는다.

실행·최저값·12쪽 raster identity·대표 직접 판독은 ignored
`candidate-77d5c12ec-{native,wasm}-boundary-fidelity.json`,
`candidate-77d5c12ec-boundary-raster-identity.json`,
`candidate-77d5c12ec-{native,wasm}-correction-panels/`에 있다.
base18/140쪽 전수와 대표 직접 판독은
`upstream-21ed64071-rowbreak-prep-native-fidelity.json`,
`upstream-21ed64071-{rowbreak-below90-panels,prep-base-boundary-panels}/`에 있다.
실패22건의 원본별 판정은 `bea45e807-terminal-failure-inventory.json`에서 갱신했다.
최종 전체 회귀·lint·남은 원본의 base 판정·개별 PR 검토와 통합은 계속 보류한다.

### 2026-10-10 최신 보완 source의 경계 비교와 원 PR 추가 입력

제품 source `f4853740ecc9f7cd3f7350d191d727cb7b240856`의 Native와 fresh WASM
빌드는 실제 exit0이다. 최초 WASM 빌드의 ENOSPC 실패 로그를 보존하고, 소스가 동일함을
확인한 뒤 WASM 단계만 재실행했다. 공유 `target/pr-review`와 Native 빌드는 재사용했다.

| 입력과 비교 범위 | Native 최저 | fresh WASM 최저 | 미달 쪽 |
| --- | ---: | ---: | --- |
| 편람 HWPX 93/130/273쪽 | 94.75915% | 94.75915% | 없음 |
| PrEP HWPX 40/105/108쪽 | 91.80848% | 91.80848% | 없음 |
| 원 PR 추가 경계 HWP 전4쪽 | 99.43997% | 99.43997% | 없음 |
| 원 PR 추가 경계 HWPX 전4쪽 | 96.45793% | 96.45793% | 없음 |

실제 생성된14쪽의 Native/fresh WASM raster bytes가 각각 동일함을 확인했다.
Native14개 review를 직접 열었다. 편람93/130쪽은 이전에 빠진 그림 외곽선이 나타나고
원점·내부 여백·후속 캡션이 기준 프레임을 유지한다.273쪽의 두 그림과 캡션 순서도 유지한다.
PrEP105쪽은20행과 마지막 연구비 총액을 모두 같은 쪽에 표시하며,108쪽은 원문의
중복 번호8을 포함한 참고문헌13개를 보존한다. 테두리 AA·그림 안 글자 raster 차이는 남는다.
이 결과는 편람383쪽·PrEP140쪽 전체의 최종 head 검증을 대신하지 않는다.

원 PR의 추가 commit `1ed091daf421f92a9e2536645f9178877d54be40`에서 원본 HWP,
생성기 출력 HWPX, 기존 대응 Print PDF, 생성기와 Rust 검사를 읽기 전용으로 확보했다.
HWP는 한컴2020 저장 LineSeg 경로이며 HWPX는 LineSeg 없는 재조판 경로다.
HWPX 자체도 한컴2020 비동기 Print로 새 PDF를 생성했고 실제 `engine=2020`,
`pdf_print_method=0`, `hancom2020_pdf_driver_one_up`,4쪽을 확인했다.
HWP4쪽은 기여자의 대응 HWP Print와, HWPX4쪽은 이 새 같은 입력의 Print와 비교했다.
M의 앞 간격·대조군, T의 쪽 상단 표와 채움1..39/다음 쪽40..42, L의 다중 줄 host와
뒤 문단을 Native review에서 직접 확인했다. 추가 Rust 검사의 누적 적용·정식 실행·
수정 전 FAIL 증거는 아직 완료하지 않았다. 합성 입력의 통과를 실물 편람의 통과로 바꾸지 않는다.

PrEP40쪽은 점수와 별개로 종료 셀의 아래 밴드가 짧고 뒤 본문이 높다는 결함이 남는다.
실제 진단은 조판 예약106.6667px, 배치의 논리 높이92.0px, 최종 Table bbox95.3px를
구분해 보존했다. 단순 여백 상수 추가로 처리하지 않고 동일 컷의 유닛·점유 높이 소비를
추적한다. base 전140쪽이90% 이상이므로 이 결함은 현재 branch에서 수정하며 #7445로 이관하지 않는다.

ignored 증적은 `candidate-f4853740e-build/build.json`,
`candidate-f4853740e-{native,wasm}-boundary-fidelity.json`,
`candidate-f4853740e-boundary-raster-identity.json`,
`candidate-f4853740e-{native,wasm}-correction-panels/provenance.json`,
`batch10-pr7617-boundary-print.json`, `pr7617-latest-original-head-intake/provenance.json`,
`candidate-f4853740e-prep40-path-diagnostic/`에 있다. WASM 패널의 실제 직접 판독은
해당 provenance에 별도로 남긴다. 전체 회귀·lint·추가 기여 commit 순서 검토와 최종 승인은 보류한다.

### 2026-10-10 순서 재적용과 최신 경계 검증

최신 fetch의 `upstream/devel`은 `21ed64071f81f35f37618717693bd30f727392c9`다.
추가 기여 commit `1ed091daf421f92a9e2536645f9178877d54be40`과
`85ea9a90bd0cd348b820db0eb9cea8d95356a15d`를 기존 기여 코드의 대응 commit
`aeab4110e2903e10b976b4523c3c7903b831b18c` 바로 뒤, 메인터너 보정 앞에 넣었다.
원본·생성기·검사 7개와 PNG 12개는 기여자 author와 cherry-pick 출처를 유지한다.
기여자가 추가한 진단 TSV 2개만 bytes를 유지해 ignored output으로 보존했다.
이후 보정 107개를 순서대로 rebase했고 실제 exit0이다. 결과
`c2d41cb4b70e79e73541949d74bcba858afe36c6`의 제품 source와 Cargo 파일은
검증 제품 `e4087c00a8d5b78534e27cb79f01cdd4bf2b6917`과 bytes가 같다.
검사·자산 추가가 끝난 것이며, 추가 검사 7개의 정식 실행과 수정 전 FAIL 증거는 아직 미검증이다.

| e408 제품의 비교 범위 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: |
| 편람 HWPX 93/130/273/281쪽 | 94.75915% | 94.75915% |
| PrEP HWPX 39/40/105/108쪽 | 99.78379% | 99.78379% |
| 기여자 경계 HWP 전4쪽 | 99.43997% | 99.43997% |
| 같은 입력 Print를 새로 만든 경계 HWPX 전4쪽 | 96.45793% | 96.45793% |
| 저장 프레임 RowBreak 정상 대조군 전7쪽 | 96.75116% | 96.75116% |
| 소비가 끝난 빈 밴드 HWP 정상 대조군 전17쪽 | 97.12321% | 97.12321% |

각 backend 40쪽 TSV가 실제 완료됐다. 두 출력의 38쪽 raster bytes는 같으며,
빈 밴드 대조군 13/17쪽은 작은 글자 영역의 실제 raster 차이가 있어 동일하다고 기록하지 않는다.
해당 WASM 두 review를 직접 열어 표 외곽·뒤 본문·내용 순서를 확인했으며 글자 굵기·AA 차이는 남는다.
같은 대조군 15쪽도 직접 판독해 소비한 행을 다시 그리지 않고 양쪽 열의 소속과 표 하단을 유지함을
확인했다. Native 편람 93/130/273쪽을 최신 패널에서 다시 열어 그림 외곽·캡션을 확인했다.
편람 281쪽은 최신 판독에서 이전의 짧은 표 하단과 목차 오른쪽 번호 배치를 보완한 결과를 확인했다.
이 40쪽은 편람 전383쪽·PrEP 전140쪽이나 전체 회귀의 최종 통과를 대신하지 않는다.

남은 실패 원본 4개의 동일 조건 Native 경계 35쪽도 비교했다. `pr-1674.hwpx` 4쪽은
base92.97642% → 현재27.20138%, `issue1853_caption_precedes_body_split.hwpx` 14쪽은
base96.73474% → 현재32.02164%로 추가 회귀다. 전자는 표 프레임 높이·칸 내용 정렬,
후자는 13쪽 마지막 줄의 14쪽 이월과 뒤 본문 원점 차이를 직접 확인했다. 문서 전체의
base 미달을 이유로 이 추가 회귀를 #7445에 넘기지 않는다. 남은 원본의 전쪽 현재 비교와
fresh WASM, 보정·정식 회귀·lint가 완료되지 않아 이 검토의 최종 판정은 계속 보류다.

ignored 근거는 `batch13-ordered-pr7617-insert.json`,
`candidate-e4087c00a-{native,wasm}-boundary-fidelity.json`,
`candidate-e4087c00a-boundary-raster-identity.json`, `batch14-{native,wasm}-correction-panels/`,
`batch14-current-failure-boundaries/provenance.json`, `batch14-pr1674-print-path/`에 있다.
완료된 WASM 글꼴 전송 SVG는 `candidate-e4087c00a-font-policy-lossless-archive/archive.json`으로
bytes 복원 가능한 상태로 보존했으며, 재사용 시 해당 manifest의 hash와 복원을 확인한다.

`pr-1674` 4쪽의 원점 보완은 아직 실행 검증 전이다. 같은 문단의 앞 표를 이전 쪽에서
소비한 뒤 시작하는 다음 표는 예산의 `anchor_offset_already_spent`에서 1130HU 문단
오프셋을 이미 제외하지만, 별도의 배치 계획이 없으면 paint가 이를 다시 적용한다.
예산에서 결정한 현재 프레임 원점을 `FragmentBudget.fragment_placement` →
`emit.rs`의 `commit_fragment`/`record_paragraph_float_placement` →
`layout_partial_table_item`의 `resolved_table_top` → `layout_partial_table_resolved`의
실제 `table_y`로 전달하는 보완을 작성했다. 독립적인 저장/캡션 원점이 있으면 그 결과를 우선한다.
관측 좌표 clamp, 문서 ID 조건, 행 컷 변경은 추가하지 않는다.

처음 패널에서 보인 차이를 표 전체의 단순 이동으로만 해석하지 않는다. fresh SVG 대조에서
첫 제목의 원점은 base/current가 같았고, 행의 물리 높이·정렬 공간이 더 커진 것을 확인했다.
소비된 문단 오프셋과 실제 프레임 원점을 일치시킨 뒤 표 외곽·칸 내용·뒤 내용까지 다시 대조해야 한다.
새 보완 코드는 기존 e408 실행 증거의 대상이 아니며 아직 빌드·시각·정식 검사가 미검증이다.
`issue1853`의 잘못된 줄 소유와 나머지 실패를 함께 분석·보완한 후 묶어서 검증한다.

### 2026-10-10 캐시·증적 삭제 뒤 재검증과 칸별 분할 예산 보완

사용자가 공간 확보를 위해 `target`과 `output`을 모두 삭제했다. 위 기록의 실행 이력은
남지만 ignored TSV·로그·캡처·실행파일·복원용 압축 payload는 현재 사용할 수 없다.
이를 최신 head의 재현 가능한 통과 증거로 재사용하지 않는다. 커밋된 원본·독립 PDF의
bytes 일치와 현재 source `8cac0e244ee07425dbc33a39cf410028001361d5`를 확인했고,
공유 `target/pr-review`에서 build jobs8·nextest threads10으로 다시 검증한다.
삭제 상태·입력 해시는 새 ignored `recovery-after-user-reset/recovery.json`에 기록한다.

`issue1853` pi105의 오른쪽 칸은 6줄 뒤 같은 문단의 다음 첫 줄까지 앞 프레임에 두고,
그 뒤 줄을 저장 vpos0에서 재개한다. pi311도 좌우 칸의 저장 되감김이 앞 조각의
마지막 줄 소속을 증언한다. 원본의 칸별 안 여백은 서로 다르다. 기존 스캐너가
행 전체의 최대 여백을 모든 칸에서 공통으로 빼면, 짧은 형제 칸의 큰 여백 때문에
실제로 들어가는 긴 칸의 마지막 줄도 먼저 이월된다. 실제 배치는 칸별 내용+자기
여백의 최댓값을 사용하므로 이 공통 차감은 배치의 요구 높이와 일치하지 않는다.

보완은 `row_remaining_visible_cell_padding` → `advance_row_cut_for_physical_band`의
칸별 내용 예산 → 기존 `advance_row_cut_inner`의 컷 → `row_cut_content_height`의
동일 컷 물리 높이 → 스캐너 누적 예약·PartialTable 컷 → partial paint의 같은
내용/여백으로 이어진다. 다른 칸의 여백 차이를 각 칸 예산에 반영하고, 중첩 내용도
선택한 컷의 실제 표시 높이로 다시 수용 여부를 확인한다. 여백이 동일한 행은 기존
mixed-nested reserve 경로를 사용한다. 일반 행의 최초 컷·넘침 재시도와 빈 밴드
probe가 같은 조회를 소비한다. rowspan 블록 스캐너는 별도 블록 원점 계약이므로
이번 ordinary-row 변경을 적용하지 않는다. 문서 ID·좌표 clamp·저장 줄 소속 변경이나
기대값 완화를 추가하지 않는다.

이 보완과 앞선 소비된 문단 오프셋 보완은 아직 실행·시각 검증 전이다. 새 회귀나
baseline은 추가하지 않았다. 같은 원본·Print의 해당 줄·앞뒤 조각·다른 여백의 정상
대조군을 재캡처하고 관련 Native/fresh WASM 90% 이상을 확인한 뒤 정식 관계 검사를
보완한다. 전체 쪽수·전체 TSV·최종 회귀·lint와 나머지 원본의 추가 회귀는 계속 보류다.

### 2026-10-10 재구축 결과와 확정 조각 원점 후속 보완

삭제 뒤 새로 빌드한 source `c99f8bd66f97156fb865a710b598f6e4ca0cc83f`의
Native 빌드와 fresh WASM 빌드는 모두 exit 0이다. Native는 build jobs 8로 실행했고,
WASM은 잠금 파일과 일치하는 wasm-bindgen 0.2.127을 사용했다. 실제 실행파일·package·
source 해시는 새 `candidate-c99f8bd66-build/build.json`에 고정했다. 이 결과는 아래의
후속 code head에 적용하지 않는다.

새 Native 전체 페이지 비교는 6입력 209쪽이며 누락·추가 쪽은 없다.

| 입력 | 비교 쪽 | 90% 미만 쪽 수 | 최저 일치율 |
| --- | ---: | ---: | ---: |
| caption1853 | 52 | 5 | 34.60991% |
| pr1674 | 35 | 2 | 82.86858% |
| scattered | 52 | 40 | 41.11436% |
| sample4514 | 46 | 18 | 3.01807% |
| empty-band-control | 17 | 0 | 97.12321% |
| large-frame-control | 7 | 0 | 96.75116% |

caption1853의13/14/37/38쪽과 pr1674의4쪽은 Native/fresh WASM 모두 각각
97.20426/96.73474/96.08718/95.85736/94.17980%이고 같은 PNG bytes다.
반면 caption1853의45쪽은 두 출력 모두34.60991%라 보류다. 이6쪽만의 WASM 비교를
전체 WASM 검증으로 보고하지 않는다. font data URI와 같은 font bytes의 file URI,
canonical raster와 동일 helper를 사용하는 pooled raster의 새 대조군은 모든 픽셀이
같았다. 메트릭·gate는 canonical 도구를 그대로 사용했다.

직접 판독한 새 PNG는 pr1674의4/5쪽, caption1853의13/44/45쪽,
empty-band-control의17쪽, large-frame-control의7쪽이다. pr1674의4쪽 큰 배치 회귀와
caption1853의13쪽 마지막 줄 소유는 개선됐지만, pr1674의표 아래 괘선과
large-frame-control의선택 상자 세로 위치에는 작은 차이가 남는다. 점수로 이 차이를
면제하지 않는다. 새 TSV·로그·PNG는 ignored `candidate-c99f8bd66-native-full-failure-fidelity/`
및 `candidate-c99f8bd66-wasm-focused-fidelity/`에 있으며 과거 삭제된 자료와 구분한다.

후속 code head `386d8deab86aade6064b607578a54fc49cd463f1`은 다음 네 소비 경계를 함께
보완했다. 첫째, caption1853 pi371처럼 선행 inline 캡션이 흐름을 전진시켜도 문단 기준
float의 실제 원점은 그대로이므로 일반 예산과 물리 프레임 하단 제한이 같은
`fragment_flow_origin`을 사용한다. 둘째, 최초·재시도 ordinary-row 컷에 clamped 내용
예산+여백을 전달하지 않고 실제 남은 물리 밴드를 전달한다. 셋째, 칸별 여백 차이도
재시도 때의 실제 물리 밴드에서 다시 계산해 여백보다 작은 예산에 공간을 만들어내지 않는다.
넷째, `prepare.first_fragment_saved_offset`에서 이미 수용한 원점을 조각 배치 결과로
전달한다. `align_flow_to`로 예산만 옮긴 뒤 paint가 별도 column에서 원점 query를 다시
실행하면 sample4514 pi763처럼 컷은655.5에서, 배치는684.6에서 시작할 수 있었다.
같은 확정 원점 → 예산·컷 → emit의 배치 계획 → `layout_partial_table_item`의
resolved top → 실제 괘선으로 연결하며 기존 확정 배치가 있으면 보존한다.

이 후속 head의 실행 검증은 아직 진행 전이며 기대값·baseline·회귀 테스트를 바꾸지 않았다.
pr1674의빈 글줄 후 lazy 기준이180HU를 재가산하는 현상, scattered의rowspan 컷 예약과
paint의 요구 높이 차이, caption1853의표 뒤 본문 흐름 차이는 여전히 추가 조사 대상이다.
고정 base의 새 재출력 전에는 이 네 원본의 미달을 #7445 이관 근거로 삼지 않는다.

추가 보완 `d970d831558d0cebdaaaf02d9e66f2bed45fd99e`에서 같은 배치 상태의
mutable 바인딩을 유지했다. 앞선 f5b340cb8 빌드는 바인딩 오류로 exit101이었고
해당 오류를 수정한 d970d8315의 Native 빌드는 jobs8, 504.33초, exit0이다.
고정 base21ed64071 대비 manifest·unit-tier 검사는 모두 exit0이다.
fresh WASM과 전체 시각 재검증은 계속 진행 중이다. 현재 진단에서 sample4514 pi763의
예약655.5와 paint의원점은 같아졌고, caption1853 pi371의첫 예산은790.1에서855.7로
수정됐다. 이 좌표 진단만으로 시각 gate·겹침·내용 보존 통과를 선언하지 않는다.

삭제된 HWPX Print 실행 증거는 동일한 커밋 원본으로 한컴2020 비동기 Print를 다시
실행해 확보했다. input SHA-256은
`a6ea57f3006f4ff66299d525899ecdd5b6973a53be8b064ec9489c2461523d2f`이며,
job `3c027504-500b-4010-9e03-fde6485d69e5`의 실제 결과는 engine2020,
`pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 4쪽이다.
download의 크기·SHA와 PDF signature·EOF를 확인했다. 새 PDF는
[같은 HWPX의 독립 Print](../../../pdf/float_push_spacing_before/float_push_spacing_before_boundaries-hwpx-2020.pdf)에
등재했고 SHA-256은 `d92e138dd6e147dd223acd205cad9cc008b51b2a0ec633dcc3b0ad49f57766b9`다.
이전 Downloads 보존본과 전4쪽의 텍스트·MediaBox는 같다. 이후 두 PDF를 같은 96dpi로
각각 새로 raster화해 전4쪽 모든 픽셀의 일치도 확인했다. 이 PDF 대조는 rhwp 출력의
통과 판정과 구분한다. 원 HWP의 대응 PDF를
HWPX 자체의 Print 증거로 대신하지 않는다. sanitized receipt와 bytes 확인은 ignored
`batch16-pr7617-boundary-print.json`, `restored-boundary-print-verification.json`에 남겼다.

### 2026-10-10 확정 문단 끝과 앞 간격의 좌표계 보완

검증 code head `d970d831558d0cebdaaaf02d9e66f2bed45fd99e`의 Native/fresh WASM
빌드는 모두 exit0이다. 원 PR 경계 입력 HWP·HWPX 각각4쪽을 두 backend에서 비교한
총16쪽은 누락·추가 쪽이 없으며 최저는 HWP99.43997%, HWPX96.45793%다.
새 HWPX Print 기준을 사용했고 일부 대표 PNG는 직접 판독했지만 나머지 확대 판독과
아래 후속 source 검증은 남아 있으므로 최종 승인으로 보고하지 않는다.

같은 code head의 Native6입력 전209쪽 비교는 완료됐다. 최저값은 caption1853
49.33810%, pr1674 82.86858%, scattered41.11436%, sample4514 3.01807%,
empty-band-control97.12321%, large-frame-control96.75116%다. 누락·추가 쪽은 없다.
caption1853의44쪽은94.77088%로 개선됐으나45쪽은49.33810%라 보류이며,
후속 전체 WASM 비교와 최종 head의 증거로 대신하지 않는다.

고정 base `21ed64071f81f35f37618717693bd30f727392c9`도 삭제 뒤 Native/fresh WASM을
다시 빌드해 각각 exit0을 확인했다. 동일 원본의 default `layout-anomaly`에서 base는
caption1853 overflow0/textOverlap5, pr1674 overflow6/textOverlap0,
scattered overflow5/textOverlap0, sample4514 overflow0/textOverlap0이다.
d970은 각각1/0,7/0,9/0,0/0이며 추가 하단 넘침은 caption1853의10쪽,
pr1674의5쪽, scattered의36/37/38/45쪽이다. 가로 넘침·기존 하단 넘침과 구분했고,
이 수량 대조만으로 #7445 이관이나 시각 통과를 판정하지 않는다.

보완 source `5176c7bfb721c49c073636a2a234e0b3b8c45277`와
`5e4429302ebbeaf02ea971785543edbfbbd01324`는 다음 세 원인을 함께 수정한다.

- `HeightCursor::source_continuous_base`에서 원본→모델 차이가0인 정상 저장 줄도
  수용하고, 앞 빈 글줄이 소비한 간격을 뒤 문단의 개체 유무로 부정하지 않는다.
- `stored_successor_flow`가 수용한 단 상대 문단 끝 → state 기록 → 단 전환·문단 인덱스
  remap·미주 scratch 계측 → `ColumnContent::stored_paragraph_flow_ends` → 문단의 마지막
  항목 paint로 전달한다. scattered의36쪽은 조판이 lazy base2814로 끝826.6을 수용했으나
  배치가 아직 없는 base를0으로 다시 투영해 끝864.08로 옮겼다. helper 뒤에서 기준을
  다시 고르지 않도록 확정 결과를 공유하며, 실제 Shape 끝을 잘라내지 않는다.
- `vpos_corrected_end_y`의 뒤 문단 앞 간격 사전 차감은 실제 첫 줄의 후방 이동과
  구분한다. 차감 전 저장 첫 줄이 순차 끝 이후이면 pre-spacing 커서만 되감겨도 같은
  첫 줄로 판단한다. 개체 호스트·사전 차감을 생략한 경로에는 적용하지 않는다.

기존 테스트에는 새 필드의 초기화만 추가했고 assertion·baseline·시각 기대값은 바꾸지
않았다. 아직 새 회귀를 추가하지 않는다. 첫5176 빌드는 누락한 미주 scratch 전달 필드를
발견해 소유 프로세스를 중지했으며 성공·제품 결함 재현으로 세지 않는다. 5e442 source의
공유 target 빌드·Native/fresh WASM 시각 검증·필수 lint·전체 회귀는 진행 중/미검증이다.
upstream/devel fetch SHA는 같고 rebase는 최신 상태라 기존 기여 커밋 순서를 바꾸지 않았다.
증적은 ignored `candidate-5e4429302-build/`, `batch17-diagnostic-paths/`,
`batch17-fixed-base-evidence/`, `recovery-base-21ed64071-*-full-failure-fidelity/`,
`restored-boundary-print-pixel-parity/`에 연결한다. base와 현재의 동일 Print·전쪽
Native/fresh WASM 검증이 끝나기 전에는 #7445로 추가 이관하지 않는다.

### 2026-10-10 추가 넘침의 새 실행 결과

`5e4429302ebbeaf02ea971785543edbfbbd01324`의 Native 빌드 jobs8·866.62초와
fresh WASM 빌드478.62초는 모두 exit0이다. source·실행파일·package 해시는
ignored `candidate-5e4429302-build/build.json`에 고정했다. default anomaly 재실행에서
caption1853은 overflow0/textOverlap0, scattered는5/0, sample4514는0/0으로,
추가된 caption10·scattered36/37/38/45의 넘침은 해소됐다. pr1674는7/0이며5쪽의
2.8px 하단 넘침이 남았다. 기존 가로 넘침과 신규 하단 넘침을 구분한다.

새 Native 경계6쪽은 caption10 96.06109%, pr1674 p5 91.06418%, scattered
p36/37/38/45는58.54279/52.35599/49.04563/51.37458%다. caption10을 직접 판독해
표 뒤 본문의 간격이 독립 Print와 연결되는 것을 확인했고, scattered36은 넘침 해소와
별개로 첫 표의 행 소속·뒤 표 시작이 Print와 다르므로 계속 미충족이다.
`batch17-candidate-boundaries/`의 TSV·PNG는 이 code head의 Native 부분 검증이며,
WASM 빌드 성공이나 다른 head의 PNG로 fresh WASM 비교를 대신하지 않는다.

후속 `09d53474a058fd957e98ed4dfd1fea84690d6232`는 재계산 스냅샷이 없는 정상 저장
줄도 원본 좌표로 확인하며 dirty·합성 줄을 제외한다. `aa07ccc9e526f7d08d89da2201c417b47f0bf56d`는
끝 조각 이후 실제 다음 문단에서 원본 빈 줄의 시작이 현재 끝+표 아래 여백으로 닫힐 때만
그 여백을 소비한다. pr1674의38번 빈 문단은27153HU이고 앞 조각의 끝26870HU+
아래 여백283HU와 정확히 닫힌다. 이미 prefill한28~37번 문단을 후속 문단으로 재사용하지
않는지 새 실행에서 확인한다. 확정 끝은 같은 column map으로 배치에 전달하고 실제 표
상자를 축소하지 않는다. assertion·baseline은 유지하며, 이 후속 head의 Native/WASM·
시각·전체 회귀는 아직 미검증이다. 비싼 재빌드 전에 준비한 review worktree에서 Clippy
세 단계와 workspace build·정책 검사를 순차 실행한다. 로그는
`candidate-aa07ccc9e-validation/`에 남긴다.


#### 삭제 후 최신 보완 코드의 lint와 5쪽 초기 진단 — 2026-10-10

code head `aa07ccc9e526f7d08d89da2201c417b47f0bf56d`의 owned review worktree에서 고정 base `21ed64071f81f35f37618717693bd30f727392c9`에 대해 prepare·fmt check·Native/WASM/workspace all-target Clippy·workspace build·integration manifest·source unit-tier 정책 검사를 모두 exit0으로 확인했다. 공유 `target/pr-review`, 빌드 잡8개를 사용했고 tracked source 해시가 실행 전후 동일하다. 명령·exit·시간·로그 해시는 ignored `candidate-aa07ccc9e-validation/provenance.json`에 보존했다. 이 lint 통과를 전체 회귀나 시각 검증 완료로 보고하지 않는다.

동일 code head의 성공한 workspace dev 실행 파일을 별도로 보존해 pr1674를 먼저 진단했다. 35쪽·overflow6/textOverlap0이며 추가됐던5쪽 하단 넘침은 없다. 같은 원본·독립 Print의5쪽 초기 비교는95.17154%다. canonical 원본 SVG 래스터와 동일 글꼴 바이트의 file URI·browser pool 결과가2쪽에서 픽셀·PNG 바이트까지 같음을 확인했다. 새 review PNG를 직접 판독했고 내용 순서와 표 이어받기는 보존되지만 아래 표의 위 괘선은485 대489, 아래 괘선은1027 대1022 raster 행에 남는다. 같은 base 원본의 아래 괘선은1004행이다. 이 수치는 차이 분석이며 회귀 assertion이나 baseline으로 추가하지 않는다.

실제 Print 배치 로그에는 `VPOS_SOURCE_CONTINUOUS: pi=39 ... source_base=0 plain_line_consumed=true`가 있다. 새 terminal source gap 분기의 적용 로그는 이 원본에서 없으므로, 그 분기가 표 여백을 해결했다고 주장하지 않는다. 초기 dev PNG·원본/Print/실행 파일 해시·직접 판독은 ignored `batch18-debug-pr1674-boundary/provenance.json`에 연결했다. 최종 release-test Native/fresh WASM 빌드·영향 페이지 재검증과 전체 회귀는 별도 진행 중이며, 이 초기 한 쪽을 전쪽 통과로 승격하지 않는다.

삭제 후 재구성한 고정 base의6입력 Native 전체209쪽에는 누락·쪽수 차이가 없다. Native 최저값은 caption1853 49.33810%, pr1674 62.93877%, scattered41.11436%, sample4514 2.70444%, 정상 대조군 empty-band96.34172%·large-frame96.75116%다. base WASM package 빌드는 성공했지만 당시 공개 API에 compat setter가 없어 canonical exporter 실행은 중단됐다. 이것은 측정 실패이며90% 미만 결과가 아니다. 재구성 실행의 WASM은 미검증으로 유지한다. canonical `visual_fixture_evidence.md`는 Native만으로 고정 base의 미달이 입증돼도 이관을 허용하므로, 기존-at-base 검사·동일 원본/Print·현재 추가 회귀 해소를 함께 확인한 범위만 사용자 지정 기존 #7445로 이관할 수 있다. 이 단계에서 추가 검사/자료 이관, 새 회귀 추가, 기대값 완화, 원격 push·PR 생성은 하지 않았다.

## 2026-10-10 지연 표의 실제 후속 흐름 보존

`batch48-frozen-next-owner-diagnosis/`의 고정5b04 Native 실행에서 pr-1674 HWPX의 문단28은 먼저3쪽에 배치되고, 문단29 처리 뒤 원래 host27의 두 번째 표가5쪽에서 끝난다. 그런데 마지막 표 여백 조회는 이미 출력된28을 선택한다. 원본38의 빈 줄 시작27153HU는 실제 조각 끝26870HU + 표 바깥여백283HU에 대응한다. 30~37의 원본 빈 줄은 저장 LineSeg를 가진다. 빈 글자라는 이유로 이 줄들을 삭제하거나 가짜 무점유 문단으로 재분류하지 않는다.

지연 큐가 가진 Before/After/SectionEnd 문단 cursor를 마지막 조각의 후속 조회에 전달하고 기존 prefill 소유 정보를 함께 소비하도록 보정했다. 끝 조각의 두 native successor 경로와 빈 끝줄 경로는 같은 `table_flow_successor`를 사용한다. 실제 후속 빈 줄이 아직 없으면 표 소유·끝 높이·쪽·단을 보존하고, 같은 물리 흐름의 원본 저장 줄이 기존 `stored_terminal_empty_line_gap` 등식을 입증할 때만 여백을 소비한다. 사이 문단이 공간을 소비하거나 쪽/단이 바뀌면 폐기하며, 기존 문단 fit·빈 줄 상자·명시적 나눔은 유지한다. 문서 ID·문단38·고정 px 위치를 분기나 assertion으로 넣지 않았다.

이 추가 source는 fmt·공백 확인만 끝났으며 컴파일·독립 Print의 해당5쪽/정상 대조군·Native/fresh WASM·관계형 회귀는 아직 미검증이다. 현재 보정 중인 입력은 최신 사용자 지시대로 #7445에 새로 이관하지 않으며, 전쪽90%와 직접 판독 전에 baseline/기대값을 바꾸지 않는다.

`dc5fcb59f`의 Native Clippy와 Native 빌드는 exit0/438.83초로 끝났다. 실제 실행에서 `TERMINAL_SOURCE_OWNER ... next=30`으로 이미 배치된28을 제외하고, `DEFERRED_TERMINAL_GAP ... next=38 flow=358.267 gap=3.773`으로 원본 빈 줄의 물리 등식이 닫힐 때만283HU를 소비했다.35쪽을 유지하며 해당5쪽 새 review98.41635%를 직접 읽어 위 표의 끝과 아래 표 시작·외곽선 및 내용 순서가 Print와 맞음을 확인했다. 원본 HWPX·PDF의 working bytes와 검토 commit bytes 일치 및 실행 파일·PNG 해시·명령·결과를 `batch52-terminal-flow-proof/provenance.json`에 기록했다. 이는 선택 Native 한 쪽의 결과이며 전체35쪽/fresh WASM 및 뒤에 보정한8af777a20의 최종 출력 통과를 대신하지 않는다.


## 각주 표시의 조각 소유 대조 — 2026-10-11

`8af777a20` Native는 가맹사업 보고서의 전체 page plan을 독립 Print와 같은142쪽으로 복원했다. 그러나 새70쪽80.69136%·71쪽66.59059%는 미달이며 직접 review 판독에서도 남은 표 위치·후속 본문·각주 차이를 확인했다. 표시67은71쪽 본문에도 있지만 각주67의 등록은 첫 표 조각의 앞쪽 lane에서 실행됐다. 원본 첫 표 글의 표시와 실제 lane 수용은 아래 추가 대조에서 구분한다. 원본/PDF는 바꾸지 않고 `batch53-current-frame-and-flow-boundaries`의 새 PNG와 `batch54-current-franchise-routing`의 실제 호출 로그를 보존했다.

소비 경로는 `emit_table_fragment → register_queued_table_footnotes → table_footnote_path_precedes_cut → 실제 compose/cell unit·부모/자식 컷`이다. HWPX만 컷 소유를 검사하던 조건을 제거해 HWP5도 표시가 실제 수용된 조각에서만 새 각주를 등록하고, 아직 수용되지 않은 표시 앞에서 source 순서의 큐를 멈추도록 보정했다. 이미 소유한 각주의 밀린 tail·모든 내용을 소비한 끝 조각·전용 각주 새 쪽의 기존 경로는 유지한다. 새 코드의 컴파일·두 backend 시각·일반/중첩/병합 경계·관계형 검사·전체 회귀는 아직 미검증이다.142쪽만으로 완료 판정하거나143쪽 기대값을 수정하지 않는다.

편람의 최신 Native 실제 흐름은 구역10의 표13 뒤 `cur_h=545.52`이며 원본 `22269 + 18045 + 600 = 40914HU`와 일치한다. 예전9ded 캡처에서 관측한600HU 누락을 최신 코드의 미해결 결함으로 단정하지 않는다. Print profile의 새284쪽 캡처와 확대 판독은 별도로 확인한다.


## 확정 TAC 끝점 이후 paint 회수 경계 — 2026-10-11

새8af Native Print-profile review에서 HWP284쪽은90.42728%지만7번 제목·본문이 여전히 약8px 위이며, HWPX284쪽97.94738%의 같은 제목은 Print와 맞는다. HWPX281쪽99.92654%를 직접 읽어 사용자가 지적한 우측 목차 쪽 번호와 외곽선이 새 출력에서 맞음을 확인했다. HWPX284쪽 우측 바탕쪽 선 등 잔여 차이는 전쪽 검증에서 계속 확인한다. 새 PNG/실행·입력 hash는 `batch57-current-handbook-panels`에 보존했다. 선택 Native 판독이며 두 형식 전쪽/fresh WASM 통과가 아니다.

HWP 실제 paint 로그(`batch61-current-handbook-paint-flow`)는 표13이444.3→692.9로 원본600HU 간격까지 소비하고, 다음 표14가692.9에서 진입한 뒤 앞 표의 괘선684.9로 원점을 되돌리는 경로를 입증한다. 앞 표 괘선은 점유 끝이며 확정 TAC 줄의 뒤 흐름 끝을 대체할 수 없다. 기존 `InlineBoxPlacement.advance_end`를 같은 단의 선행 흐름 끝 조회에 포함해 partial-table paint와 caret/cell 조회가 동일한 확정 끝점을 소비하도록 수정했다. 별도 px 여백이나 문서 조건·이름을 넣지 않고, flow 끝이 없는 인라인 개체·exclusion은 기존과 같이 제외한다. 각주 표시 소유 보정7859f790a의 Native는 실행 중이며 이 추가 source의 재빌드/경계·시각·회귀는 미검증이다.


## 표시 소유와 물리 각주 lane의 추가 대조 — 2026-10-11

`7859f790a` Native Clippy·빌드가 exit0/411.12초로 끝났지만 가맹사업은 여전히142쪽과69쪽78.89077%·70쪽80.69136%·71쪽66.59059%다. 새69·71쪽 review를 직접 읽고 원본 각주67의 표시가69쪽 첫 표 글에도 있음을 확인했다. 소유 검사만으로71쪽에 두어야 한다는 앞선 추정은 성립하지 않으며, 추가 guard를 각주 배치 해결로 표시하지 않는다. 원본 각주67~70의 내용·계층, 실제 큐, Print의65·66/67~70 물리 lane 구분을 `batch60-franchise-marker-diagnosis`에서 대조했다.

단일 단의 HWP5 표 각주 fit만 추정 높이와 footer-band reclaim을 사용해 이미 수용한 표 조각과 새 각주가 같은 물리 영역을 차지하는 상태를 허용했다. HWPX의 기존 경로는 `estimate_footnote_area_height_with_metrics → 실제 본문 영역의 capacity/표 끝 → reserve_painted_footnote_area`로 출력과 동일한 높이를 소비한다. 이 단일 단 공통 경로를 컨테이너 구분 없이 사용하도록 보정했다. 표시 컷 검사는 별개의 소유 계약으로 유지하고, 다단의 기존 lane 정책·순서·저장 note reset/suffix 경로는 그대로다. 표 번호·각주 번호·임의 최소 개수나 고정 px 보정은 추가하지 않았다. 이 추가 source는 아직 재빌드/시각/관계형 회귀 전이며 원본·기준값과 현재 보정 문서 범위를 유지한다.


`edb086255` Native Clippy·빌드 exit0/396.39초 뒤 같은 HWP284쪽 새 Print-profile review를 직접 읽었다.7번 제목·괘선·본문의8px 조기 배치가 해소됐고90.42728%→98.26895%,전체 count383/383이다. HWPX281·284쪽 새 PNG는 직전에 읽은8af 출력과 byte-identical이며 비교/hash 기록을 `batch64-edb-handbook-panels/provenance.json`에 남겼다. 이는 선택 Native 판독이며 전체383쪽·fresh WASM·현재 추가 각주 보정689484afb의 최종 검증은 아니다.

현재689484afb에서 fmt·Native Clippy는 exit0이고 Native 빌드가 공유 cache에서 jobs8로 실행 중이다. 종료 뒤 `batch67-physical-note-lane-verification`이 같은 frozen binary로 가맹사업 전142쪽 TSV를 먼저 산출하고 실제69~72쪽·정상 대조군을 재출력한다. 현재는 실행 대기이며 전쪽 결과를 완료로 표시하지 않는다.8af777a20의 별도 순차 WASM Clippy·workspace build/all-targets Clippy·두 base 고정 정책 검사·fresh WASM 빌드는 exit0으로 끝났지만 이후 source 변경의 최종 gate를 대신하지 않는다.

## 정상 최소 셀 높이와 행 컷의 여백 — 2026-10-11

689484afb Native 빌드는 exit0/382.92초로 완료했다. 같은 frozen 실행 파일의 실제 각주 등록 로그에서67~70이 끝 표 조각의 물리71쪽에 함께 예약됨을 확인했다(`batch69-current-franchise-lane-routing`). 전142쪽 시각 비교는 계속 실행 중이며 등록 로그를 시각 통과로 대신하지 않는다.

원본650의4×2 자식 표에서2행은 선언 높이282HU, 위·아래 안 여백141HU씩이고 오른쪽의 저장5줄은7400HU에서 끝난다. 기존 출력 행 높이100.5px는 줄7400HU + 축소된 여백141HU에 해당한다. 독립 Print의 같은 행 괘선 차이는102.288px이며, 원본 줄과 온전한 여백의7682HU는96DPI에서102.42667px다. 같은 최소 높이를 가진 기존 #7234의 실제 한컴 저장본·PDF 대조도 내용이 들어오면서 여백을 포함한 행으로 성장하는 계약을 입증한다. 입력을 바꾸거나 합성 최소값으로 원본을 대체하지 않았다.

생산·소비 경로는 `Cell::vertical_padding_is_abnormal → resolve_cell_padding_at_height → row_cut_content_height → table_partial의 MeasuredTable 없는 온전한 자식 행 재계산`이다. 선언 높이와 여백 합이 같은 정상 최소값을 비정상으로 처리하던 `>=`를 문서화된 엄격 초과 `>`로 수정했다. 실제 초과 여백의 기존 축소와0 높이 비발동은 유지한다. 측정의 중첩 relaxed-pad 및 전체 행 계산은 별도 소비 경로이므로 같은 결과가 되는지 기존 대조군으로 확인한다. 원본 레코드·기존 paint·Print 괘선은 ignored `batch71-padding-source-proof/provenance.json`에 연결했다.

이 보정의 재빌드·전체 페이지 Native/fresh WASM·관계형 검사·최종 회귀는 미검증이다. 부모 표 원점 등 남은 차이까지 해결했다고 주장하지 않으며 새 회귀·기대값·baseline·이관은 추가하지 않았다.

`005fab000`의 fmt·Native Clippy와 Native 빌드가 exit0/521.69초로 끝났다. 실행 파일 SHA-256은 `aa7fa5201564f394d7ce194ee13e3f9cba0407b61de3b07366d63a47060ba284`다. 실제 새 render tree는 문제 행의100.5→102.4px 성장과 전체142쪽 유지를 보여 준다(`batch73-padding-runtime-check`). 표 원점489.0px는 그대로이므로 행 여백 보존과 표 원점의 남은 차이를 구분한다. 이 수치는 진단 관측값이며 회귀의 절대 px assertion으로 추가하지 않았다.

새 가맹사업70쪽과 기존 최소 높이 표1쪽에서 전체 임베딩 글꼴의 byte-identical file transport·browser pool과 canonical raster가 모든 픽셀과 PNG bytes까지 같음을 다시 입증했다(`batch75-current-font-raster-control/control.json`). 같은 원본 font cmap 검사·글꼴 공급·페이지별 지표·gate를 유지하는 기존 실행 wrapper로 새 전체142쪽 TSV와 최소 높이 두 독립 Print 대조군을 `batch76-current-padding-visual`에서 생성 중이다. 기존 #5751·#7234의 두 suite만 선택한 nextest는 공유 cache의 Native 빌드 종료 뒤 jobs8/threads10으로 실행 중이다(`batch74-padding-focused-regression/focused-regression.log`). 아직 전체47입력·fresh WASM·최종 전체 회귀·PR 완료를 입증하지 않는다.

집중 nextest의 첫 batch74는 기본 checkout의 오래된 generated suite 번호를 사용해0개를 선택했고 exit4로 끝났다. 코드 회귀 FAIL이나 PASS가 아니다. 실제 review worktree에서 재준비한 generated 원본을 조회해 #7234는004, #5751은006으로 확인했다. 번호를 고정하지 않고 해당 worktree에서 원본 파일명으로 suite를 찾는 batch77로 다시 실행해 **6 PASS/430 skipped**, exit0/61.56초(테스트0.176초)를 확인했다. 원본 검사와 기대값은 바꾸지 않았다. 모든 최종 검사는 같은 review worktree에서 prepare한 목록을 사용한다.

## 중첩 빈 호스트의 저장 바깥 프레임 소비 — 2026-10-11

원본650의 빈 호스트12는25920HU, 후속13은61657HU다. 실제 자식 표의 세로 offset0·선언 높이35171HU·위/아래 바깥여백283HU씩은 `61657 − 25920 = 35171 + 283 + 283`을 정확히 만족한다. 독립 Print와005fab000의 실제 행 높이는 이제 맞지만 표 원점은 아직489.0px 대492.10133px다. 원본 레코드와 기존 호출을 ignored `batch78-empty-host-source-proof/provenance.json`에 보존했다.

루트 조판은 기존 `ParagraphFloatPlacement::from_empty_stored_ladder_host`에서 같은 저장 등식·문단 앞 간격·온전한 물리 높이를 검증한다. 중첩 경로는 보이는 텍스트가 있어야 하는 `from_stored_host`만 사용해 빈 호스트의 프레임을 받지 못했다. 중첩 공통 조회에 원본 셀 문단 slice와 cursor를 전달하고 같은 기존 빈 호스트 계약을 fallback으로 소비하도록 보정했다. 전체 셀 paint·분할 셀 paint·mixed 유닛 조회가 같은 원본 후속 문단을 참조한다. 분할 경로가 저장 줄 상단으로 snap한 빈 호스트는 앞 간격 이전의 문단 원점으로 환산한다. 최종 resolved table top·점유 끝을 기존 소비 지점에 전달하므로 바깥여백을 paint에서 별도로 중복 추가하지 않는다.

원본이 아닌 줄·편집/재조판·되감긴 후속 좌표·내용 혼재·캡션·선언과 실제 높이가 다른 표·후속 문단이 없는 경우는 기존 계약이 거부하거나 기존 경로를 유지한다. 다행/재귀 컷의 각 행 유닛 높이와 바깥여백 회계는 기존 경로를 별도로 확인한다. 새 임의 수치·문서 식별자·고정 px 보정은 넣지 않았으며 새 회귀와 baseline을 추가하지 않았다. 이 추가 source의 실제 활성화·앞 간격 및 첫/끝/분할 경계·Native/fresh WASM 전쪽·확대 판독은 아직 미검증이다.

기본 checkout의 `cargo fmt --all`은 오래된 ignored generated suite가 이미 제외된 원본을 참조해 실패했다. tracked 변경은 위 두 source만이며 standalone rustfmt로 포맷했다. 제출용 전체 fmt·Clippy와 정책 검사는 prepare된 review worktree에서 새 source로 수행한다. 이 환경 실패를 formatter 통과나 제품 회귀로 바꾸어 기록하지 않는다.

e2e674040의 prepare·전체 fmt check·Native Clippy·Native 빌드는 exit0으로 종료했다(Native616.39초, SHA-256 `69bd2e13bccfa4cfe61f8bac5b1e8063d04ee1e74efa110c6ab51142dc415fcf`). 실제 새 render tree에서70쪽 표 원점은489.0→492.8px로 바뀌었으며 독립 Print492.10133px에 가깝고,2행102.4px와 전체142쪽을 유지한다(`batch82-empty-host-runtime-check`). 처음 batch80의 진단 프로세스는 빌드 provenance 생성보다 먼저 읽어 FileNotFoundError로 끝났다. 종료를 확인하고 완료된 동일 실행 파일로 진단만 재실행했으며 Native 빌드를 재시작하지 않았다. 이 관측값을 시각 PASS로 대신하지 않는다.

가맹사업689484afb 및005fab000의 전체142쪽 TSV는24쪽이90% 미만이며 최저30.86526%다.005fab000의70쪽은80.69136→87.56941%지만 여전히 미달이다. 미달 목록은1,2,8,9,10,19,38,39,41,42,43,50,52,56,57,67,68,69,70,71,91,95,118,119이며70·71쪽만 해결하면 완료되는 범위가 아니다. 이전 head의 전쪽 원본과 전체24쪽 review를 `batch67-physical-note-lane-verification`, `batch76-current-padding-visual`, `batch81-franchise-all-failed-panels`에 보존했다. 새 e2e674040의 정확한 font transport/pool parity를 다시 통과한 뒤 같은 전142쪽 TSV와 이24쪽·정상 경계 PNG를 `batch83-current-empty-host-visual`에서 생성 중이다.

직접 판독한8·9쪽은 표138의 첫/끝 물리 프레임과 후속 본문이 이르다. 원본 셀 높이10788HU는143.84px인데 Print의 두 조각은59.29467+84.388=143.68267px이며 기존 Native bbox는46.9+71.2px다. 단순 내용 컷이 남은 물리 공간을 누락하는지 생산·예약·paint 경로를 추적해야 하며 선언값 clamp만으로 해결했다고 판단하지 않는다.57쪽은 표557 이어받기에 Print와 다른 source 줄이 들어가고 뒤 행/본문이 이르다.005fab000의71쪽에서 각주67~70은 보존되지만 마지막 괘선·본문·각주 간격이 다르다. 이 세 그룹을 함께 분석하며 현재 보정 문서를 #7445로 이관하거나 검사를 제거하지 않는다.

추가 독립 대조군7216의 긴/짧은 최소 높이 HWPX는 각각 최저45.83553%·37.85254%로 미달이다. 짧은 표1쪽을 직접 읽어 표와 뒤 문단이 Print보다 이른 배치 차이를 확인했다.6개 집중 검사 PASS를 이 대조군의 시각 일치로 승격하지 않는다. 대조군의 새 회귀/기대값은 추가하지 않았고 관련 실제 출력 및 두 backend 확인은 미검증으로 남긴다. 각 이미지의 실제 판독 쪽·hash·관측값은 해당 ignored provenance에 기록했다. 전체47입력·fresh WASM·최종 lint/policy·전체 nextest·개별7 review·PR/merge는 아직 미완료다.

## 저장 줄 컷과 두 물리 상자의 공동 예약 — 2026-10-11

e2e674040의 batch83은 전142쪽 TSV와 선택27쪽의 실제 compare/overlay/review를 모두 산출했다.70쪽은97.97045%로90% 이상이지만71쪽은66.59223%, 전쪽 최저30.86526%/미달23쪽이다.70·71쪽의 새 review를 직접 열어 자식 표 원점/행 높이 개선과 남은 부모 아래 괘선·본문·각주 사이 간격 차이를 구분했다.70쪽의 높은 점수로 부모 괘선 차이나 전쪽 미달을 면제하지 않는다. 이 결과는 e2e674040이며 다음66061ea03의 검증 결과가 아니다.

동일 실행 파일의 `dump-pages --compat 2022 --json`과 원본 dump를 `batch84-franchise-source-cut-audit`에 보존했다. 표138은8쪽 끝 컷2/9쪽 시작 컷2이며 소비는46.69333/70.96px다. 원본 셀10788HU·첫 개체4452HU·후속 문단139의6616HU·바깥여백140HU씩은 `10788 − 4452 + 140 + 140 = 6616`을 정확히 만족한다. 독립 Print의 두 상자는59.29467/84.388px다. 표477도 원본19650HU·첫 개체2443HU·다음 문단478의17773HU·바깥여백283HU씩이 `19650 − 2443 + 283 + 283 = 17773`으로 닫힌다. 독립 Print42쪽의 첫 상자는32.604px/첫 글줄1개,43쪽 종료 상자는229.18933px/남은9줄이다. 원본 같은 문단의 첫 두 줄은0/0이고 뒤 줄1872HU가 다음 글줄 높이1100+간격772HU를 정확히 잇는다. 기존 컷8/2줄은 원본의1/9줄 소속과 다르다. 실제 PDF vector 좌표를 별도 JSON에 보존했으며 이 수치를 회귀의 px assertion으로 사용하지 않는다.

표557의 첫 개체61462HU는 앞 두 온전한 행30731HU씩의 합과 같다. Print55쪽의 두 행은169.25467→578.56667→987.87867px다. 기존 스캐너는 두 행819.49333px 뒤의 잔여20.28px에 다음 행 첫 컷35.22667px를 수용해55쪽에 컷[1,1]을 남겼다. 원본 행3의 문단 내부 양수→0 쪽 경계도 확인했다. 뒤 호스트의 원점이 앞 호스트보다 크다는 이유로 첫 개체의 정확한 행 끝을 무시하던 가정을 제거하고, 유효한 원본 셀의 내부 쪽 경계와 전체 행합이 동일한 첫 프레임을 닫는 경우에도 그 행 끝까지만 스캔한다. dirty·합성·편집/재조판·경쟁 rowspan·각주 예약은 기존 거부 조건을 유지한다.

66061ea03의 단일 셀 변경은 기존 `stored_two_frame_successor_origin_hu`의 독립 종료 등식과 원본 줄 컷 조회를 재사용한다. 같은 문단의 양수→0 또는 첫0/다음0/그 뒤 정확한 줄 전진을 인증하며, 세 번째 프레임·개체 혼재·dirty/합성/재조판·컷별 내용이 선언 상자를 넘는 경우는 거부한다. 생산 결과는 `native_single_cell_source_row_frames`의 컷·앞/뒤 높이이며, `block/prepare → fragment/budget → fragment/scan`이 페이지 예산 안에서 먼저 두 물리 상자를 수용한다. `fragment/emit`은 실제 수용한 앞 상자에서 뒤 상자를 이어받고, `table_partial`은 동일 원본 조회와 수용한 높이·컷이 일치할 때만 조각 내용으로 정렬한다. paint에서 선언 높이를 별도로 늘리거나 내용 유닛을 추가 소비하지 않는다.

66061ea03의 prepare·전체 fmt check·Native Clippy·Native 빌드는 exit0으로 종료했다(빌드579.27초, SHA-256 `64da82068dab30b5982a831e2bdb019679b421289430eac70060b6782e44998a`). `batch86-physical-source-runtime-check`의 실제142쪽에서 표138의 앞/뒤 상자는59.6/84.7px, 표477의 컷은1/9줄과32.8/229.7px, 표557의 첫 조각은 두 온전한 행에서 끝난다. 하지만 표477의 첫 원점617.3px는 독립 Print782.984px와 크게 다르며, 표557의 뒤 상자도854.2/639.5px로 Print866.892/652.08667px에 못 미친다. 올바른 컷만으로 배치가 해결됐다고 판정하지 않는다. RenderTree의 실제 `pi/ci` 계약으로 완성된 원본 JSON을 다시 추출했으며 최초 잘못된 `paraIndex` 선택의0개를 통과로 세지 않았다. 새 세 쪽의 canonical/full-font transport/pooled raster parity는 통과했고, 전142쪽 시각 비교는 `batch87-current-physical-source-visual`에서 실행 중이다. 정상 대조군·fresh WASM·관계형 회귀·최종47입력은 미검증이다.

## 첫 원점·문단 내부 경계·뒤 각주 소유의 추가 보정 — 2026-10-11

`batch88-original-note-frame-audit`는 변경하지 않은 원본 HWP 레코드와66061ea03의 dump를 보존한다. 본문447의 각주27 표시는 원본 UTF-16 위치191이며 쪽 reset은149다. reset 앞 네 줄의 마지막 잉크 끝63300HU가 기존 각주 영역 위에 들어가지만, 실제 컷은2줄이다. `paragraph/scan`의 저장 프레임 조회가 실제 각주를 가진 문단 전체를 거부해 뒤 프레임의 표시가 앞 프레임까지 조기 이월시키는 경로를 확인했다. 컨트롤과 경계 위치를 같은 codepoint 공간으로 변환하고 모든 각주 표시가 경계 뒤이며 앞 줄이 실제 예약된 각주 영역을 침범하지 않을 때만 기존 저장 prefix 소유를 유지한다. 경계 앞 표시·다른 개체·dirty·다단·합성 줄·앵커 불일치는 기존 경로다. 본문 marker와 각주의 실제 소유·내용 보존은 추가 실행으로 확인해야 한다.

표477은 원본 host47462HU·문단 기준 offset1122HU·바깥 위283HU로 첫 괘선 원점을 결정한다. 기존 `native_single_cell_source_row_frames`는 독립 종료 등식과 컷/두 높이를 인증하지만 원점은 반환하지 않았다. 순차 current_height466.3px가 별도로 남아 첫 상자를 너무 위에 그렸다. 같은 인증 결과에 원본 anchor와 table top을 포함하고, `block/prepare`에서 동일 물리 쪽·단일 단·현재 각주/배제 경계에 실제 첫 상자가 들어가는지 확인한다. 수용된 원점은 `first_fragment_saved_offset → budget.align_flow_to`와 동일 `host_placement → budget.fragment_placement → emit.record_paragraph_float_placement → table_partial.resolved_table_top`으로 전달한다. 바깥 위 여백은 기존 first-fragment 변환에서 한 번만 열며, 이월한 다른 물리 프레임에 이전 anchor를 다시 적용하지 않는다.

표557의 행3은 공통 저장 좌표계를 잇는 여러 문단 뒤 같은 문단 안의23220→0HU 경계를 가진다. 기존 원본 행 최소 공간 조회는 문단 사이 reset만 인정해, 내용 컷이 맞아도 CENTER 행의 빈 물리 밴드를 버렸다. 유효한 원본 글줄의 같은 문단 내부 양수→0도 공통 reset 판정에 포함했다. 기존 원본 행합/첫 개체·공유 vpos·동일 높이의 정렬 owner·dirty/경쟁 rowspan/재조판 거부를 유지한다. `stored_full_width_row_declared_height → emit.stored_row_frame`의 실제 수용 높이·누적 상자·남은 높이와 `table_partial`의 동일 행 소유가 소비한다. paint에서 커지는 상자만 추가하지 않았다. 종료 상자·뒤 행/본문·유닛 누락/중복은 새 실행 전 미검증이다.

위 세 보정은 source commit `da1580e817abfcbe24c90b55c83ba7ac8a898da7`으로 묶었다. standalone rustfmt·diff check를 마쳤고, 준비된 review worktree/고정 공유 target에서 전체 fmt·Native Clippy·빌드를 jobs8로 순차 진행한다. 이는 제출 검증 완료가 아니다. 각주31의 원본에는 별도 문단의 비-TAC 표가 있고 독립 Print41쪽에는6행 표가 보이지만 기존 Native에는 누락돼 있다. 원본/Print/실제 트리를 대조한 이 결함은 아직 수정 전이다. 현재 보정 문서의 이관·회귀 제거·기대값 또는 baseline 갱신 없이 모든 잔여 차이를 계속 처리한다. 진행 중인 frozen66061ea03 시각 결과를 da1580e81의 결과로 재사용하지 않는다. batch 검증 단계가 끝난 후 base 동기화/필요한 rebase와 최종47입력·개별7 review·PR 범위를 계속한다.

## 사전 이월과 물리 하단의 소비 경계 보완 — 2026-10-11

66061ea03의 batch87은 전142쪽 TSV와29쪽 review 생성까지 종료했다. 전체21쪽 미달이며9쪽93.37791%,43쪽95.10478%,55쪽98.70162%로 개선됐다. 새8·9·55·56쪽을 직접 읽었다.9쪽의 닫힌 표 상자/세 글줄과55쪽의 두 온전한 행 소속은 Print와 맞지만,8쪽 각주/본문 간격과56쪽 CENTER 행의 이른 배치·닫는 상자가 남는다. 직접 판독은29쪽 중4쪽이며 나머지 선택 쪽·다른47입력·fresh WASM의 최종 검증을 대신하지 않는다. 오래된 batch83의 full-font 중간 SVG142개는 경로·완료 프로세스·TSV/27 PNG 보존을 확인하고 각각 SHA-256을 기록한 뒤 제거했다(9.73GiB, `batch83-svg-removal-ledger.json`). 원본/Print·PNG·TSV·로그·font/export manifests·고정 실행 파일·공유 target은 보존했다.

da1580e81의 prepare·전체 fmt·Native Clippy·Native 빌드는 PASS(624.24초/exit0, 실행 파일 SHA-256 `bf3eab63d6fea070253324d1b0d264455dba159a091d9bccb5f9e5b171a262a6`)다. 그러나 batch90의 실제 쪽수는143으로 늘어 **회귀**다. 본문447의 소속은38쪽0~4/39쪽4~6으로 저장 경계와 맞지만, 표477의 첫 상자는42쪽 대신43쪽151.0px에 이월됐으며 종료 상자도44쪽으로 밀렸다. 원본 행의 공간을 보존한 표557은57쪽873.3px/58쪽647.9px가 되어 분포가 여전히 Print와 다르다. 이 head의 시각 완료·90% 통과·승인으로 판정하지 않으며, 실패한 쪽수로 비용 큰 전쪽 시각을 새로 실행하지 않았다.

표477의 실제 로그는 current632.8px·기존 각주162.7px·보수 가용674.1px에서 첫 행 최소28.7px가 남은22.6px에 안 들어간다고 보고 통째 이월했다. 독립 종료 등식으로 이미 인증한 첫 상자는 원본651.56px부터32.57333px를 쓰므로 실제 각주 경계714.18667px 안에 들어간다. 인증 조회가 `prepare` 끝에만 있어 앞선 첫 행 이월 판정에 쓰이지 않던 호출 경계를 확인했다. `79015cd4d`는 같은 `native_single_cell_source_row_frames` 결과를 사전 이월 전에 만들고, 단일 단/같은 row geometry/표 각주 없음/같은 원본 상자가 실제 각주 및 배제 경계에 들어가는 경우 그 prefix의 물리 쪽 소유를 유지한다. 이후의 anchor·컷·높이 수용과 실제 paint도 동일 조회를 소비한다. 예약할 공간이 부족한 경우나 편집/무효 source는 기존 이월 계약을 유지한다.

표557의57쪽 예약873.08px는 source-owned CENTER 행의 빈 물리 공간을 현재 body 끝까지 수용한 값이다. paint는 기존 `rowbreak_fragment_frame_bottom`의 바깥 아래283HU+100HU 경계를 사용하지만 선언 행 소유가 생긴 뒤에는 후처리 축소를 하지 않아 서로 다른 상자가 누적됐다. 독립 Print56/57쪽의866.892/652.08667px와 원본 행합을 대조해 이 분포 차이를 확인했다. `2b6bf82df`는 이어받는 Native 원본 다행 조각의 budget도 paint와 같은 기존 하단을 소비하게 한다. 실제 수용 상자의 누적→선언 행의 남은 높이→종료 조각으로 이어지는 원장은 유지한다. HWPX와 쪽 중간 첫 조각에 Native inset을 추가하지 않았으며, 새 문서 수치나 px clamp를 넣지 않았다. 새 실행에서 해당 조각의 컷·전체 행합·뒤 본문·확대 판독을 확인하기 전 완료로 판정하지 않는다.

source 보완을 커밋한 뒤 `git fetch upstream devel`과 `git rebase upstream/devel`을 수행했다. 최신 base는 기존 `21ed64071f81f35f37618717693bd30f727392c9`와 같아 브랜치는 이미 최신이며 commit 순서가 바뀌지 않았다. 새2b6bf82df의 prepare/전체 fmt/Native Clippy/빌드 jobs8을 동일 공유 target에서 순차 실행하고, 고정 실행 파일의 실제 전체 쪽수·원점·컷 진단 뒤142/142가 확인될 때만 전쪽 Native 시각을 진행한다(`candidate-2b6bf82df-native-diagnostic-build`, `batch94-source-boundary-runtime-check`, `batch95-source-boundary-visual`). 최종 WASM/workspace lint·정상 대조·관계형 회귀·47입력 양 backend·전체 nextest·개별7 review·PR은 미완료다. 각주31의 표는 아직 수정 전이다. 파서 진단 HWPX는 원본 보존 상태의 별도 projection이며 독립 한컴 기준으로 쓰지 않는다. 최초 행 수만으로 선택한 각주8을31로 착각한 진단 선택은 실제 번호31/6×3/비-TAC를 확인해 수정했다. 원본 레코드 및 독립 Print41쪽은 이 선택과 무관하게 보존했다.

## 각주 블록 표의 측정·배치·본문 예약 공유 — 2026-10-11

직전 code head `2b6bf82dfbb1ef8258119220970aa3f510fd98f1`의 suite 준비·전체 fmt·Native Clippy·Native 빌드는 exit0으로 완료했다(빌드544.20초). frozen binary SHA-256은 `549e8c03fbdc9e7facf682dfccd3e508e3f4c35417e8b0c95e94b2147cce5921`이다. `batch94-source-boundary-runtime-check`의 실제 전체 쪽수는142/Print142다. ¶477의 첫 표는 잘못된43쪽에서 독립 Print와 같은42쪽으로 돌아왔고, ¶557의56·57쪽 물리 프레임은 각각868.2·653.0px로 재분배됐다. 이는 실행 진단값이며 절대 px 회귀 기대값으로 추가하지 않는다. `batch95-source-boundary-visual`의 새 전142쪽 TSV·대표29쪽 review는 진행 중이고, runtime 위치/쪽수로 시각 통과를 대신하지 않는다.

현재 source 보정은 `8a7a2cff8baa5141f35d921a476d1d4528ce6f03`이다. 원본 HWP 각주31에는 본문6줄 뒤의 빈 표 호스트와 비-TAC 6×3 표가 있다. raw source의 표 행합11020HU·바깥여백283HU씩, 저장 호스트7032HU와 독립 Print41쪽의 실제 표를 `batch88-original-note-frame-audit`에서 확인했다. diagnostic HWPX 재직렬화는 parser 조회 자료이며 독립 한컴 기준 PDF로 취급하지 않는다. 앞선 Native는 각주 본문만 그려 표와 해당 점유 높이를 누락했다.

새 공통 계획은 빈 단독 표 호스트의 문단 기준 Top/Inside·TopAndBottom 블록 표에 적용한다. `picture_footnote::footnote_block_table_frame`의 원래 MeasuredTable과 whole-table 행 높이 resolver → `footnote_paragraph_placement`의 표 원점·전진량 → 각주 영역 높이와 `layout_footnote_area`의 `layout_table` 호출이 동일한 결과를 소비한다. 마지막 paint 직전에는 공통 계획의 원점을 전달하고 표 뒤의 y도 그 전진량으로 이동한다. 표 호스트의 빈 글줄 높이를 표 아래에 중복 합산하지 않는다. 첫 문단이 표이면 번호도 같은 호스트에 출력하고, 표와 번호 줄의 점유 끝 중 큰 값을 공통 전진량으로 사용한다.

실제 pagination 소비 경로도 함께 연결했다. `BodyFootnoteMeasure`는 section column spacing·profile·column 환경·render normalization을 실제 layout에 공급한다. `register_body_footnote`의 일반/지연/끝 표 호스트 예약과 `prepare_forced_page_boundary → native_hwp5_existing_body_footnote_area_height` 및 현재 문단의 추가 각주 충돌 조회가 같은 블록 표 내용 높이를 사용한다. 표가 없는 각주의 기존 빠른 예약과 저장 조각 높이는 유지한다. Caption·Square·혼합 텍스트 호스트는 이 블록 계획의 대상이 아니며 기존 경로를 유지한다.

새 source의 fmt·공백 검사는 통과했고, `candidate-8a7a2cff8-native-diagnostic-build`에서 공유 `target/pr-review`/jobs8로 suite 준비→fmt→Native Clippy→Native 빌드를 순차 실행 중이다. `batch97-block-footnote-runtime-check`는 같은 frozen binary를 기다려 전체 쪽수·표 컷·FootnoteArea의 실제 표를 조사한다. observer의 최초 시작은 build provenance 생성 전 FileNotFound로 종료했으며 `initial-start-failure.json`에 보존하고 같은 빌드 관찰만 재개했다. Native 빌드 자체는 재시작하지 않았다.

새 각주 표의 Native/fresh WASM 전쪽90% 및 확대 판독, 첫 표 번호/계속 조각 경계, 정상 대조군은 아직 미검증이다. 일반 여러 표·복합 호스트·캡션·Square·편집 저장본 및 블록 표 분할의 한컴 일치를 이 보정의 통과로 주장하지 않는다. 원본·검사·baseline을 바꾸거나 현재 보정 문서를 #7445로 이관하지 않았다. 전체47입력·7개 PR의 추가 시각/편집 증거·최종 lint/nextest·개별 review/PR/CI/merge/후속 처리는 계속 미완료다.

### 19쪽 기호의 원문·독립 PDF 폭 대조

`batch99-franchise-print-font-readonly`에서 독립 Print의 garbled BaseFont 이름을 원래 CP949로 복구해 본문 `/F1`이 휴먼명조임을 확인했다. 설치된 정확한 family는 `C:/Windows/Fonts/HMKMM.TTF`이며 SHA-256은 `d033df71fa41de48905407f7d39aca74ebd9ae87298c5420513acaa187fbefc6`다. PDF subset의512 UPEM과 대표 글자 `필`·`것`의512 advance/윤곽 bounding box는 설치 face와 같다. 이 소수 글자의 대조를 전체 글꼴 동일성으로 확대하지 않는다. 처음 조사한 HMFMMUEX.TTC는 휴먼모음T·휴먼엑스포였으므로 대상 family 증거에서 제외했다.

독립 Print의 모서리 기호는 U+231C `⌜`·U+231F `⌟`이고 `/F4` Haansoft Batang subset에 있다. 휴먼명조에는 두 glyph가 없으며 PDF fallback의1024 UPEM/1024 advance는 전각이다. PDF text transform(size104×0.12×96/72)으로 약16.64px가 된다. 직전 실제 `2b6bf82df` RenderTree의 같은 원본 ¶252 기호 run은8.3px다. 기호 뒤 내용 위치 차이는 이 자연 전진폭 누락과 함께 조사한다. 기존 `text_measurement`의 metric miss→generic halfwidth 경로 및 실제 glyph fallback/font selection 소비를 추적하되, 기호를 임의 전각 상수로 바꾸거나 현재 보정 문서를 글꼴 예외·#7445 이관으로 처리하지 않는다. 독립 PDF의 fallback 출처와 실제 측정/paint 선택이 공유되는지 아직 미검증이며, 새 글꼴/폭 source 보정이나 회귀 기대값 변경은 이 조사에서 하지 않았다.

`8a7a2cff8`의 suite 준비·전체 fmt·Native Clippy는 exit0이다. Native 빌드가 실제 rustc에서 실행 중이며, batch97의 실제 각주 표/전체 쪽수 진단과 batch98의 새 전쪽 시각 검증은 같은 owned build를 기다린다. batch98은142쪽과41쪽 FootnoteArea의 표가 확인되지 않으면 고비용 시각 산출 전 실패 상태를 기록한다. 전체 제출·통합 판정은 미완료다.

## 각주·미주 출처 오인과 최신 직접 판독 — 2026-10-11

`8a7a2cff8`의 Native 빌드는 exit0/680.89초로 종료했다. frozen binary SHA-256은 `46d0fc13d8fa3457a07fa8536c2412e102eb1a335d08327015fc9380d2b0f54f`이며 actual runtime은142/142쪽, 각주31의6×3 표를41쪽 FootnoteArea에 출력한다. `batch100-note31-direct-review/review/review_041.png`는 새 canonical Native PNG와 같은 PDF SHA의 독립 Print reference PNG를 비교했다. Native를 이전 head 이미지로 대체하지 않았으며 기준 PNG 재사용 출처·hash는 JSON에 구분했다. 실루엣은90.88544%다. 직접 판독에서 표 누락 해소를 확인했지만 표·다음 각주의 원점이 약 한 저장 줄간격 일찍 끝나므로 점수로 면제하지 않았다.

생산/소비 경로는 `rendering::set_endnote_para_sources(본문 문단 수, 실제 미주 출처) → paragraph_layout의 para_index >= base → endnote_line_vpos_base → 같은 미주 successor 없는 마지막 줄의 trailing spacing=0`이다. 각주의 큰 마커 문단 주소는 base 이후이지만 실제 미주 출처 배열에 속하지 않는다. 그럼에도 미주 경로를 타면서 각주31의 본문6줄 뒤272HU 간격이 빠져 표와 각주32가 앞당겨졌다. 독립 Print의 첫 각주 baseline724.16과 fresh Native의 첫 줄713.9+원본 baseline765HU는 맞지만, 표 원점807.8/Print810.633 및 다음 번호 baseline의 차이가 남는다. 이 진단 px는 regression assertion으로 추가하지 않는다.

보정 `48a667bda90a3cee7883cc560d2785a925f68b74`는 미주 여부를 실제 등록된 출처 범위로 조회하게 한다. 저장 미주 vpos·lazy tail·영 여백 특수 분기·blank-spacer·줄 점유/간격의6개 실제 적용 지점이 같은 조회를 소비한다. 미주 자체의 source/행 메트릭은 변경하지 않고 각주를 미주로 오인하는 가정을 제거했다. 실제 미주 대조군 실행은 아직 미검증이다. 블록 표 content-height query도 실제 단일 앵커 계획의 적용 여부와 일치시켜, 다줄 호스트를 표 없는 fast reservation 대신 exact block 경로로 잘못 보내지 않게 했다. 공유 target/jobs8의 suite/fmt/Native Clippy/Native 빌드는 batch102에서 순차 실행 중이며 batch103 runtime·batch104 전쪽 시각이 이어진다. batch104는 이전 batch98 전체 비교가 끝날 때까지 기다려 산출물·I/O를 겹치지 않는다.

직전 `2b6bf82df`의 batch95 전142쪽 비교는1472.84초로 종료했고, 미달은15쪽/최저41.50210%다(1,2,8,10,19,41,50,52,67,68,69,91,95,118,119). 이전21쪽 미달 중38·39·42·56·57·71쪽은 새 수치에서90% 이상이 됐지만 전체 문서 gate는 여전히 보류다. 새68·119쪽 review를 직접 읽어 가운데 정렬·행 프레임·각주 소유 및 표 뒤 본문 위치 차이를 확인했다. 오래된 batch95 중간 SVG142개는 driver 종료·완료 상태·TSV/29 PNG 보존을 확인하고 파일별 SHA ledger를 남긴 뒤 제거해9.73GiB를 확보했다. 공유 Cargo cache, 원본·기준 PDF·PNG·TSV·JSON·로그·frozen binary는 보존한다.

68쪽 각주65는 현재 Native FootnoteArea에 없고, 독립 Print는 첫 줄을68쪽/뒤5줄을69쪽에 둔다. 실제 runtime 표의 원본 소유자는 ¶642다. `batch106-table642-note65-owner-audit`의 diagnostic parser 조회에서 각주65 저장 줄은0,0,1172,2344,3516,4688HU이며 첫 쪽1줄/다음 쪽5줄 계약과 대조한다. ¶650의 각주67~70은 독립 Print와 같이71쪽 소유이며 다른 표다. 이 두 표를 같은 원인으로 합치지 않는다. 원본 raw HWP와 실제 cell-note queue의 cut/소유를 더 확인해야 하며, 아직 추가 source 보정·새 regression/baseline 변경·이관을 하지 않았다. batch101의 최초 XML host 줄 수16에는 중첩 셀 줄이 포함돼 앵커 자체의 줄 수로 쓰지 않는다; 각주31 실제 호스트1줄은 이전 raw audit 및 이번 common plan으로 대조한다.

전체47입력/3,628 Print쪽의 Native/fresh WASM·개별7 PR 추가 시각/편집 증거·최종 lint/nextest·PR/CI/merge/후속 처리는 계속 미완료다. 현재 보정 문서는 #7445로 이관하지 않으며 관측 실패를 검사 제거/기준값 완화로 처리하지 않는다.

## 표 조각 전체의 컷과 각주 표시 소유를 분리 — 2026-10-11

`batch107-raw-note65-66-audit/original-records.json`은 diagnostic 재직렬화 없이 원본 OLE Section0을 직접 읽었다. 각주65의 원본 소유자는 ¶642이며 실제6줄의 vpos는0,0,1172,2344,3516,4688HU다. 뒤 줄은 앞 줄의900HU 높이+272HU 간격으로 이어지고, 저장 textStart0/69/142/216/285/360을 보존한다. 독립 Print는68쪽에 첫 줄,69쪽에 뒤5줄을 출력한다. 각주66은 다른 Body 호스트 ¶644의7줄로, 각주65의 table-cell 출처와 혼동하지 않는다.

실제 기존 source는 이 각주의 저장 재시작을 `native_hwp5_footnote_reset_fragments`에서 이미 인식했다. 원인은 helper 누락이 아니라 소비 지점의 aggregate 조건이다. `emit_table_fragment`는 ¶642의68쪽 조각이 앞 행 중간의 start_cut=[1,1]에서 시작했다는 이유로 fragment_has_intra_row_cut=true를 전달한다. 이 조각의 다른 완전한 행에 각주65 표시가 실제 소비됐는데도 `register_queued_table_footnotes`가 그 bool만으로 prefix 등록을 거절하고, 새 쪽에서는 각주 전체를 원자적으로 넣었다. 최신 이전 head의68쪽 FootnoteArea 부재와69쪽의 전체 각주65를 직접 대조했다.

보정 code head `3159a2238929690c153216400ec3fc39567df8ed`는 그 aggregate bool을 실제 시작 컷으로 바꾼다. 생산·소비 경로는 `accepted start/end row+CellUnit cut → table_footnote_path_precedes_cut의 동일한 부모/자식/마커 경계 조회 → 시작 컷 전에 소비되지 않고 끝 컷 전에 소비된 표시 소유 → 저장 prefix/suffix fit → 실제 FootnoteRef.fragment 등록/다음 물리 쪽 예약 → footnote paint의 같은 줄 범위`다. cut이 다른 행에 있다는 이유로 표시를 버리지 않고, 이미 앞 조각에 소비된 표시·미래 표시·강제로 연 큐 소진 쪽을 새 표시 소유로 인정하지 않는다. 원본/줄 수/숫자/px 조건을 추가하지 않고 기존 actual cut 조회를 두 경계에 공유했다. rowspan 없는 기존 queue 적용 범위·실제 footnote fit·prefix 한 번의 번호와 각 쪽 separator·source 순서/남은 큐 진행은 유지한다.

실제 수정 전 FAIL 근거는68쪽 각주65 누락 및 독립 Print68/69쪽의1/5줄이다. 수정 후 실행·원점·누락/중복·번호/구분선 및 관계형 회귀는 아직 미검증이다. `candidate-3159a2238-native-diagnostic-build`는 실제 이전 build handle을 기다린 뒤 같은 review checkout/cache를 순차 사용한다. suite/fmt/Native Clippy는 exit0이며 새 Native 빌드는 실행 중이다. batch109의 같은 frozen runtime과 batch110의 이전 시각 종료 후 전142쪽 비교를 연결했다. 원본 판독으로 앞으로 개선할 대상의 검사를 제거하거나 기대값을 갱신하지 않았다.

선행 `48a667bda` Native 빌드도 exit0/624.42초로 완료했다. frozen SHA-256은 `ee283fa53eb0e729505fae922ac69e4649a6ed974e5a013fab678c511d29e43b`이고 실제142/142쪽이다. 각주31 표 원점은807.8→811.4px로 이동해 빠진272HU 간격을 반영했고, 각주43의1×1 표는 기존 원점/251.9px 높이를 유지한다. runtime만으로 직접 시각 통과를 선언하지 않는다. batch104 observer는 runtime JSON 쓰기 중 일시적 빈 파일을 읽어 JSONDecodeError로 실제 종료했다. 실패 stdout/stderr와 최초 state를 `initial-observation-failure.json`에 보존하고5초 이내의 같은 파일 재읽기를 추가해 관찰을 재개했다. Native 빌드·runtime은 재실행하지 않았으며 font control/전쪽 raster는 최초 실행으로 이어진다.

`8a7a2cff8`의 batch98 전142쪽 시각은1368.19초로 종료했다.41쪽은 미달 목록에서 빠졌으나 전체14쪽(1,2,8,10,19,50,52,67,68,69,91,95,118,119)/최저41.50210%가 남는다. 새48/3159 source의 전쪽·fresh WASM·정상/실제 미주·nested/partial 마커 경계, 현재 가운데 정렬/표 물리 행 높이 잔여 문제 및 전체47입력/7 PR의 나머지 제출 조건은 아직 미완료다. 개별 source head와 증거를 구분하고 PR/승인/merge는 보류한다.

## 실제 표시 컷 검증과 새 쪽 본문 좌표 — 2026-10-11

`3159a2238`의 Native Clippy·suite/fmt는 exit0, Native 빌드는 exit0/614.16초다. frozen binary SHA-256은 `b84d6be4ee6953cf79d09730ca47c468556a36c97b143a18115b64c824be455e`다. batch109의 실제 runtime은 **143쪽(독립 Print142쪽과 불일치)**이며 batch110은 `page_count_mismatch`로 전쪽 visual을 시작하지 않았다. 이를 통과로 기록하지 않는다.

batch112는 원본 OLE와 실제 RenderTree를 연결했다. 각주65 prefix는68쪽, suffix는69쪽에 있으므로 표시 소유 보정은 실행되지만, 본문¶644와 각주66은70쪽으로 밀렸다. 69쪽 본문 항목은 빈¶643뿐이다. 원본¶643의 저장 VPOS는0, ¶644는2498HU부터 시작한다. 원인은 `terminal fragment → register_queued_table_footnotes → force_new_page + suffix 등록 → finish_paragraph_anchor_state`에서 앞쪽 표 host의 이전 문단 좌표를 새 쪽에 재기록한 것이다. caller가 pending queue를 새 쪽에서 소진하는 경로와 unqueued 저장 각주 경로에는 `request_vpos_reset_after_queued_footnote`가 있지만 이 직접 terminal split 경로에는 없었다. 같은 요청을 추가해 post-flow가 새 쪽의 본문 cursor를 보존하게 했다. 실제 수정 후142쪽·각주65/66·본문 소유 및 정상 대조군은 아직 미검증이다. 더 이상 소비하지 않는 aggregate `fragment_starts_intra_row` field도 제거했다.

batch111은 선행 source `48a667bda`의 fresh canonical Native41쪽과 같은 SHA의 독립 Print 기준 PNG를 비교했고 **98.54779%**다. review_041.png를 직접 판독해 6×3 표와 앞6줄·뒤 각주32 보존, 표 상하단과 뒤 각주 baseline의 이전 줄간격 차이 해소를 확인했다. 문자 outline 차이는 남으며 선택한 한 쪽으로 전쪽/fresh WASM 통과를 선언하지 않는다. 기존90.88544% 그림과 구분된 새 PNG다.

68쪽 표 물리 높이/왼쪽 가운데 정렬은 별도 미해결이다. batch112의 원본 row0 cell 높이는49088/49342HU로 서로 다르고 row1은15117HU다. batch113의 원본 common.height는1710HU이고 왼쪽 문단은 보이는 첫 줄 다음에0→0 저장 재시작을 가진다. 이를 빈 줄 컷으로 추정하지 않는다. 실제 Native68쪽 row0 높이595.0px/row1 201.6px와 Print의 가운데 라벨·괘선을 대조하며, 같은 높이·모든 칸 가운데 정렬을 요구하는 기존 물리 frame helper가 이 원본을 수용하지 않는 경로를 확인했다. 아직 인증되지 않은 숫자를 frame fallback으로 넣지 않았으며 검사/baseline을 변경하지 않았다. 증적은 ignored batch111/112/113에 보존한다.

### 실제 미주 정상 대조군과 원본 물리 행 프레임

batch117은 원본 `samples/endnote-01.hwp` OLE에서6개의 실제 미주 control을 확인한 뒤, 역할 보정 전 `8a7a2cff8`와 보정 후 `3159a2238`의 같은 입력/프로필로 전체 page plan과 모든 RenderTree를 출력했다. 두 실행 명령 모두 exit0이며 **page plan 및 전쪽 RenderTree bytes/hash가 동일**하다. 실제 등록된 미주의 출처를 계속 인정하고 각주를 배제하는 positive control이다. 같은 출력이라는 결과를 독립 Print/fresh WASM 시각 통과나 새 regression expectation으로 승격하지 않는다.

batch118은 parser와 동일한 original CTRL_HEADER 높이 offset20을 직접 읽어 ¶642의 원본 첫 object frame이1710HU(22.8px)임을 확인했다. 원본 row0 최대 cell height49342HU에서 첫 물리 상자를 뺀 잔여는635.093333px다. 독립 Print 벡터에서67쪽 상자886.550667→909.405333,68쪽 row0 136.010667→770.357333, row1→971.737333을 확인했다. 즉 전체 row의 선언 공간과 첫 조각에 수용한 내용 높이를 혼동해서는 안 되며, 첫 상자/남은 상자는 같은 원본 공간을 나눈다. 기존 Native의67쪽26.4px/68쪽595.0px는 그 계약과 다르다. 이 px는 원인 진단 자료이며 고정 회귀 assertion으로 추가하지 않았다. 단순 Center 복원·clamp보다 source frame → accepted physical box → remaining row → 배치 정렬 소비 연결의 보정이 필요하며 아직 구현/시각 수용은 완료되지 않았다.

`9f4671e51`의 새 쪽 vpos 보정은 batch114/115/116에서 검증 중이다. Native Clippy exit0/2분26초, suite/fmt exit0이며 Native 빌드 실제 driver5712/Cargo18792/Rust18332의 실행을 확인했다. runtime observer가 producer의 최초 provenance 생성 전에 시작해 FileNotFoundError로 종료한 흔적을 보존하고, 해당 handle 종료와 output 미생성을 확인한 뒤 observer만 재개했다. Native/Cargo를 재실행하지 않았다. 현재 기록은 전체 회귀·전쪽/fresh WASM 수용을 의미하지 않는다.

## 새 쪽 좌표 보정의 실제142쪽·내용 보존 — 2026-10-11

`9f4671e51a5c71638cfe035c60d66b1d2ed57e7f`의 batch114 Native 빌드는 **exit0/465.12초**로 종료했다. frozen SHA-256은 `9c32d25352bc187b8d9e592ca8840478806176c4a253fd97b85dc378ac2bdce9`다. 같은 바이너리 batch115는 **142/Print142쪽**이며 본문¶643–649 및 표650 시작을69쪽에 배치한다. 이전3159의143쪽/본문¶644가70쪽으로 밀린 회귀를 해소했다. batch120/121은 같은 원본 OLE/hash와 actual RenderTree를 대조했다: 각주65의6줄을68쪽1줄/69쪽5줄로, 각주66의7줄을69쪽에 보존한다. 공백과 원본8-unit 자동번호 control을 제외한 전체 실제 텍스트가 원본과 같고 SHA도 일치한다. 번호65는68쪽 한 번,66은69쪽 한 번이며 각 쪽 구분선은1개다. 이 content/page relation 진단을 새 정식 regression이나 전쪽/fresh WASM 시각 통과로 대신하지 않는다.

선행 source48의 batch104 전체142쪽 Native TSV·대표29 PNG 생성은 완료됐다. **90% 미만9쪽/최저41.50210%**: 1,2,10,19,68,91,95,118,119. 이전8a의14미달 중8·50·52·67·69쪽이90% 이상으로 올라왔다. 8쪽94.37556%,41쪽98.54779%,50쪽92.29192%,69쪽96.45666%다. 8·50쪽 review를 직접 읽어 본문/각주 내용 보존과 footer 배치 개선을 확인했고, 인용 기호 advance·미세 글자 간격과50쪽 표 괘선의 작은 세로 차이는 남겼다. 수치 통과로 그 차이를 면제하지 않는다. batch116은 최신 source9의 font transport/canonical/pool 동등성3쪽을 통과한 뒤 전142쪽 시각 비교를 시작했다. 최신 전체 TSV/대표 직접 판독 및 fresh WASM은 진행/미검증이다.

완료된 batch104 driver 종료·TSV/29 PNG 보존·경로/소유와 파일 hash를 확인하고 오래된 중간 SVG142개만 제거해9.73GiB를 확보했다. `batch104-svg-removal-ledger.json`에 기록하며 모든 PNG/TSV/JSON/log/원본/PDF/frozen binary와 공유 target은 보존했다.

전체 source frame 원인 분석은 batch119를 두 section으로 확장했다. ¶642의 서로 다른 row0 cell heights와 첫1710HU physical frame 외에도 section1 ¶461의107–142쪽 이어받기를 구분했다. 현재 수정 입력의 검사를 이관/삭제하거나 허용치·baseline을 바꾸지 않았다. 최신 upstream/devel fetch는 기존 `21ed64071f81f35f37618717693bd30f727392c9`와 같고 rebase는 up to date였다. batch122에서 원7 PR이 모두 open/unmerged이고 이전 검토 head와 같음을 API로 재확인했다. 개별7 review·47입력/fresh WASM·고비용 전체 회귀·최종 Rust 묶음·통합 PR/CI/merge/후속 처리는 완료되지 않았다.

## 첫 행 저장 물리 프레임 공유 — 2026-10-11

메인터너 code head `a9c25f39b544d372eef9f3c3d67fe066f39c1247`는 첫 object frame이 첫 source row 안에서 닫힐 때, 동기화된 실제 CellUnit reset과 저장 LineSeg 범위가 인증한 `(컷, opening, remaining)`을 같은 query에서 만든다. source cutoff는 실제 유닛의 첫 frame reset이며 raw 행 높이의 최대값은 같은 격자 행의 물리 공간이다. owner들의 높이/정렬이 같아야 한다는 가정으로 이를 거절하지 않는다. 다른 source reset·수정/재조판·중첩/걸침/객체 단위·비영 cell spacing 등 별도 공간 계약은 기존 경로이며 이번 query의 충족으로 보증하지 않는다. 저장 줄 상자 하단과 다음 전진 범위가 첫 object frame을 닫는지 확인하고, 새 측정 내용이 source tail보다 크면 수용하지 않는다. 저장 LineSeg 높이를 실제 글리프 잉크 정확성의 보증으로 쓰지는 않으며 실제 glyph/spacing은 Print 시각 검증에서 확인한다.

생산/소비 연결은 `original common.height + raw first-row max + CellUnit synchronized cuts → saved_first_row_opening_frame → scan_table_fragment의 실제 opening 요구 높이/수용 컷 → emit_table_fragment의 같은 컷/높이 확인 → 실제 accepted box ledger + remaining cursor → RowScanQuery::whole_row_height의 같은 물리 tail 요구 높이 → PartialTable의 start/end override 및 source alignment → accepted source box에 대한 paint-only fold 제외`다. opening이 현재 예산에 안 맞으면 현재 내용/각주가 있는 쪽을 이월하며 새 빈 쪽에서 이월만 반복하지 않는다. 다른 행의 fragment와 row-span block에서는 그 경계의 기존 수용/컷 경로를 유지한다. 이후 컷이 필요할 때 ledger는 실제 수용한 box만 누적한다. row count/특정 수치/파일 ID에 따른 분기를 추가하지 않았다.

독립 원본/Print 근거는 batch112/113/118이며, 원본1710HU opening 및49342−1710HU remaining을 Native의26.4/595.0px 대신 같은 물리 상자로 전달한다. 아래쪽 표 괘선과 뒤 본문까지 확인해야 하고 Center 하나의 복원으로 완료하지 않는다. field/caption/비영 cell spacing 및 multi-reset 소유는 이 추가 query에서 미검증/비적용이며 source 반례를 지침보다 축소해서 검사를 갱신하지 않는다. 이번 코드의 실제 경계·정상 대조군과 전쪽 Native/fresh WASM은 아직 미검증이다. 새 정식 regression/expectation/baseline은 변경하지 않았다.

batch123은 공유 target/jobs8의 suite/fmt와 Native Clippy를 통과했다(Clippy2분31초). 실제 Native 빌드는 driver9928/Cargo16804/Rust15720으로 실행 중이며 batch124 runtime과 batch125 visual이 같은 source를 기다린다. batch125는 실제 source query 적용과142쪽을 확인하기 전 전쪽 raster를 시작하지 않으며 완료된 source9f의 visual과 I/O를 순차 사용한다. 새 canonical/pool font control에는67/68/69쪽 opening/closing/suffix 경계도 포함했다.

선행 source9f의 전체142쪽 Native/29 PNG는9미달·최저44.39375%로 종료했다(1,2,10,19,68,91,95,118,119). 68쪽을 직접 읽어 각주65 prefix 복원은 확인했지만, 왼쪽 label의 Top 정렬·early row border/본문을 실제 결함으로 남겼다. batch127의 원본 parser-view charPr72는 일곱 언어 슬롯 모두 휴먼명조를 지정한다. 19쪽 U+231C/U+231F 문제를 언어 슬롯 교체로 해결할 근거가 없다. 실제 원본/PDF/TTF의 missing glyph와 fallback natural advance를 확인해야 하며 전체 face를 대체하거나 일반1em/PX 폭을 넣지 않았다. Canvas supplemental session은 portable export와 구분되므로 그 측정만 켜서 SVG 수용 근거를 만드는 우회도 하지 않았다.

### 첫 행 프레임의 실행·직접 판독·정상 대조군

같은 `a9c25f39b`의 Native 빌드는 **exit0/553.31초**로 종료했고 frozen binary SHA-256은 `1688695e35cca9d2a920b0d375b36e71677abe0b97bf2d12209c9293402a672e`다. batch124의 실제 출력은142/Print142쪽이며 ¶642의 source query는 `[1,1]` 컷과22.8px opening을 실제 사용한다. 68쪽 row0/row1은635.1/201.6px이고 왼쪽 라벨 baseline은440.5667px/Print440.64px다. 이 좌표는 독립 원인·실행 진단이며 정식 회귀 expectation으로 고정하지 않았다.

batch125의6쪽 canonical/file-transport/browser-pool pixel·PNG byte 대조가 통과했다. 그중 새67/68/69쪽과 SHA가 같은 독립 Print 기준 PNG로 만든 batch126의 비교값은 **93.35654 / 99.67505 / 98.70757%**다. 세 review PNG를 직접 읽어 opening 상자, 왼쪽 가운데 정렬/두 행 경계, 이어받는 본문·표·각주65 prefix/suffix 및66을 확인했다. 작게 보이는 본문 번호도 canonical SVG에서65)임을 확인했다. 괘선 두께·문자 outline·67쪽 인용 부호 전진·69쪽 첫 행 배경색 차이는 남으며, 선택한 세 쪽의 수치로 전142쪽/fresh WASM의 수용을 선언하지 않는다.

batch128은 source9f→a9의 정상6문서312쪽과 실제 미주 문서5쪽에서 전체 page plan 및 **모든 RenderTree bytes/hash 동일**을 확인했다. 실행 명령 모두 exit0다. batch129는 같은 a9 runtime과 원본 OLE 각주65/66을 대조해6/7줄,65의68쪽1줄+69쪽5줄과66의69쪽7줄, 전체 텍스트 hash, 각 번호1회와 각 쪽 separator1개를 다시 확인했다. 출력 동일성/내용 보존을 독립 Print/fresh WASM의 시각 통과로 대신하지 않는다. 증적은 ignored batch124/125/126/128/129에 보존한다.

다음 미달 원인 batch130은 원본 각주8의6×2 TAC 표, 저장12872HU 글줄(`12592+140+140`)과 Print10쪽 괘선822.94→990.596px, 실제 a9 FootnoteArea 안의 Table0개를 연결했다. 일반 본문의 `is_tac_table_inline_in_para`는 단독 전면 폭 표를 블록으로 분류하지만, 현재 각주 block plan은 non-TAC만 처리하므로 높이를 예약하고 표를 출력하지 않는다. 기존31의 non-TAC 계획,43의 실제 인라인 TAC 대조군과 구분해 저장 전용 글줄의 소유·바깥여백·다음 줄간격을 같은 측정/paint 결과로 전달해야 한다. 이 후속 보정은 아직 구현/검증 전이며 원본/회귀 검사 삭제·이관·baseline 완화 없이 진행한다.

### 각주 전용 TAC 글줄의 표 소유 보정 — 검증 전

후속 구현은 표 하나만 소유하는 빈 호스트에서 기존 일반 문단의 inline 분류가 거절한 TAC만 대상으로 한다. `tac_stored_band_is_outer_box`가 원본 한 줄의 표+위/아래 바깥여백 등식을 인증하고 저장 text partition이 변경되지 않았을 때, 그 글줄을 각주 공통 table frame에 포함한다. 작은 실제 인라인 표·앞뒤 텍스트/복수 개체·다중 줄/수정된 저장 밴드는 기존 경로다. source 번호나 문서 ID로 대상을 선택하지 않는다.

생산/소비는 `실제 단독 control+저장 외곽 글줄 → 일반 inline 분류/저장 밴드 인증 → footnote_block_table_frame의 동일 MeasuredTable/TAC 행 resolver → 원본 글줄과 실제 표 외곽 점유의 큰 값 및 같은 저장 줄간격 → body 각주 예약·fragment 높이 → layout_footnote_area의 같은 advance/후속 줄간격 → 같은 resolved x/y의 layout_table`다. TAC의 객체 float offset을 글줄 좌표에 다시 더하지 않고 기존 body의 `tac_table_stored_outer_band_top`을 공유한다. 실제 표가 저장 줄보다 자라면 요구 공간도 커지며, 마지막 선택 줄에는 trailing spacing을 붙이지 않는다.31의 non-TAC 표와43의 실제 inline TAC는 기존 경로를 유지해야 한다. 수정 전 결함은 a9의10쪽 Table0개이며, 새 코드의 빌드·원점/높이·셀/원문 보존·뒤 빈 줄/각주·전체142쪽·정상 대조군 및 Native/fresh WASM은 아직 미검증이다.

보정은 `2b9dd8360`에 커밋했다. 첫 Native Clippy에서 폭 인자를 상위 content-height 호출에 잘못 추가하고 실제 placement 호출에서 빠뜨린2개 E0061을 검출했으며 그 단계 exit101로 Native 빌드/시각 실행은 시작하지 않았다. `24cd3a64c275a0942a0537376ca779c8ca69f0a2`에서 전달 위치를 고쳤고 review worktree의 suite/fmt와 Native Clippy **exit0/1분56초**를 확인했다. 공유 target/jobs8 Native 빌드 및 같은 frozen runtime/시각 검증은 batch135/136/137에서 이어진다. 실패 기록을 통과로 바꾸거나 덮어쓰지 않았다.

선행 a9의 batch125 전체142쪽 TSV/대표29 PNG가 종료했으며 **8쪽 미달·최저55.04285%**다(1,2,10,19,91,95,118,119). 68쪽은 미달 목록에서 해소됐지만 전체 gate는 계속 보류다. batch134는 같은 source SVG와 독립 Print의 font/text transforms를 기록했다. batch138은 실제 설치 `HMKMM.TTF` SHA-256 `d033df71fa41de48905407f7d39aca74ebd9ae87298c5420513acaa187fbefc6`, 원본의 TTF 선언 및 Print 휴먼명조를 확인하고 같은 face에 `trueTypeFaces=[휴먼명조]`를 명시했다. face 대체/기준값 변경 없이91/95쪽은 **99.99808/99.99752%**로 개선됐고 두 review를 직접 읽어 글머리 뒤 본문 원점과 간격 일치를 확인했다. 이는 **명시적 검증 글꼴 환경의 대조 실행**이며 기본 설정의 결함 해소나 새24 source 전체 수용을 의미하지 않는다. batch139의 같은 대조에서도1/2/19쪽은88.96966/87.60402/86.67729%로 남는다. 표지·corner glyph와118/119쪽 배치 문제는 미해결이며 같은 선택 점수로 면제하지 않는다.

`24cd3a64c` Native 빌드는 **exit0/612.39초**, frozen SHA-256은 `b5752c09fbb5756a58015991c059f3d973795179827beacb46da6e90c4b91b11`이다. batch136은142/Print142쪽이며10쪽 각주 영역에6×2 표를 실제 출력한다(상자115.3/823.7,559.2×167.9px). batch137의8쪽 canonical/transport/browser-pool 대조가 통과했고, 그 새10쪽과 같은 독립 Print PNG로 만든 batch141은 **95.23784%**다. 직접 review를 읽어 표/앞 번호·내용/후속 여백·footer를 확인했다. 본문의 인용 기호/간격과 괘선 농도 차이는 남는다. batch144는 **원본12셀의 주소와 전체 텍스트 모두 보존**, a9→24의 전142쪽 RenderTree 중 **10쪽만 변경**을 확인했다. 왼쪽 가로 괘선이 없다는 외형으로 rowspan을 추정하지 않았다(원본 cellSpan은 모두1). batch142의 정상6문서312쪽+실제 미주5쪽도 전쪽 plan/RenderTree bytes 동일이다. 이 관측은 관계형 진단이며 전쪽 Native/fresh WASM을 통과하기 전 정식 새 regression/expectation을 추가하지 않았다.

종료된 a9 batch125 driver의 실제 종료를 확인하고 raw SVG142개만 SHA-256 ledger 후 제거했다(`batch125-svg-removal-ledger.json`,9.73GiB). 나머지672파일의 크기/hash를 재확인했으며 PNG/TSV/JSON/log/원본/기준 PDF/바이너리·공유 target은 보존했다. 새24의 전142쪽 시각은 batch137에서 진행 중이다.

다음 표지 분석 batch140/145는 원본¶4/charPr100의3개 저장 줄·가운데 정렬·자간−17%를 구분했다. 첫 줄의 마지막 공간은 저장 wrap 경계의 원문 separator다. TTF 메트릭으로 계산한 총 전진572.7px/줄폭566.933333px의 선형 압축−0.2306667px가 실제 Native 첫 두 글자의 전진25.2226667px와 정확히 일치한다. separator 전진을 뺀 뒤 음수 tracking의 마지막 전진 상자를 복원한 보수적인 범위565.186667px는 줄폭 안에 든다. 현재 fit은 말미 공간을 포함하고 뒤 정렬은 그 공간을 빼는 소비 차이가 후보 원인이다. 이는 실제 glyph 잉크/Print 일치의 완료 증거가 아니라 다음 보정 가설이며, 장식된 공간·inline 객체 소유·측정/실제 배치의 공통 결과를 대조한 후 구현해야 한다. 최초 whole-paragraph/단일-title-line 혼동 assertion 실패도 ignored 진단에 보존하고 원본3줄 경계로 수정했다.

### 가운데 정렬의 줄 끝 폭 공유 — 실행 검증 전

`71a3b0015eaefd3ac40446843c49e02ca704f57d`는 순수 가운데 정렬 글줄의 장식 없는 말미 공백을 줄 끝 측정에서 제외하고, 마지막 가시 글자의 음수 자간 뒤 전진 점유를 함께 계산한다. 문서 ID·특정 폭·픽셀 위치를 선택 조건으로 쓰지 않는다. 저장 wrap과 문단 끝이 같은 폭 결과를 사용한다. 밑줄·취소선·음영·실제 border/background가 있는 공백은 남기고, 공백뿐인 줄·TAC 개체·탭·겹침·각주 마커·생성된 soft-wrap separator는 각자의 기존 계약으로 보낸다. Right/Justify/Distribute의 분배 규칙은 변경하지 않았다. 기존 여섯 source 단위 검사에는 새 선택 인자 `None`만 전달하며 assertion/기준값을 바꾸지 않았다.

생산/소비는 `원래 ComposedLine의 표시 문자열/실제 run style → CenteredTextLineMeasure의 동일 run 범위 → compute_line_extra_spacing의 수용 폭/문자 수 → 필요한 압축에서 같은 measure.width(extra)의 반복 실측 → 실제 Center 시작점의 같은 measure.width(extra) → emit_line_runs의 같은 extra_char_sp`다. 장식 공백을 제거하지 않고, 개체 소유를 텍스트 공백으로 추정하지 않는다. 말미 음수 자간 복원은 보수적인 glyph advance 상자이며 실제 외형/Print 일치의 보증은 시각 대조에서 별도로 확인한다. 마지막 위치를 clamp하여 제목을 맞추는 별도 보정을 추가하지 않았다.

공유 target/jobs8의 batch146이 같은 source에서 suite/fmt/source-unit-tier 및 고정 base manifest 검사, Native Clippy와 Native 빌드를 진행한다. 이전 각주 source24의 전체142쪽 batch137은 별도로 계속 실행 중이다. 새 표지/장식 공백·개체 대조군/전쪽 Native·fresh WASM 및 관계형 정식 회귀는 아직 미검증이며, 소스 커밋을 시각 통과로 보고하지 않는다. 새 expectation/baseline/test를 추가하지 않았다.

각주 source24의 batch137은 전체142쪽 TSV/대표29 PNG 산출을 마쳤다. **최저55.04285%, 미달7쪽(1,2,19,91,95,118,119)**이며10쪽 누락 표가 미달 목록에서 해소됐다. gate는 여전히 보류다. batch150에서118/119 review를 직접 읽고 실제 같은 중첩 표가 두 쪽에 이어진다는 원본 소유를 확인했다(section1/outer¶461/row4/inner¶32/1×1 cell). section0의 동번호 문단은 다른 입력이므로 그 경로를 거절했다. cell subList는 CENTER, 원래 common.height6488HU, cell height16231HU이고 첫 문단 뒤 다음 문단에서 저장 vertpos가300HU로 다시 시작한다. 독립 Print의 변환한 path로 읽은 첫 조각 높이는86.465334px로6488/75와 대응하지만 Native는72.0px다. 첫 본문 baseline도 Print940.48/Native927.933333px로 앞당겨졌다. 다음 조각 높이는 Print136.969334/Native137.3px로 가깝지만 원점·뒤 본문도 다르다. 이 값은 진단 기록이며 정식 절대 px 기대값이 아니다. 다음 보정은 nested 컷의 요구/수용 높이·원점·여백과 paint가 같은 실제 조각을 소비하는지 추적해야 하며, 문서를 삭제/이관하거나 점수로 면제하지 않는다.

새71 source의 suite/fmt/source-unit-tier/고정 base manifest 및 **Native Clippy exit0/3분44초**를 확인했고 Native 빌드가 진행 중이다. batch147/149 runtime 대조는 그 정확한 frozen binary를 기다리고, batch148의 선택 페이지 직접 비교는 선행 batch137 종료 후 실행한다. 최종3 Clippy·workspace·fresh WASM·전쪽 증적·정식 회귀는 아직 완료하지 않았다.

### 표지 정확 폭과 중첩 프레임의 공통 예약/배치 — 검증 전

71 Native 빌드는 **exit0/479.81초**, frozen SHA-256은 `8cfb871277a5ac76dc1726c1e7ddf8a0227e6f43fbc0e8e6ea7366636776e54b`다. batch147은142쪽을 유지했고 batch148의 기본 글꼴 환경에서1/2쪽은 **89.86984/91.21171%**,19/118/119쪽은86.67729/87.32923/55.04285%다. 1쪽 review를 직접 읽었으며 아직 미달이다. batch149의 정상6문서+실제 미주317쪽은 모두 page plan을 유지했지만, 그중 일부의 Center 좌표가 달라졌다. 동일 RenderTree/무회귀로 보고하지 않으며 해당 영향의 Print 대조가 남는다. 새 helper가 실제 배치의 `estimate_text_width_exact` 대신 반올림 함수로 run 폭과 마지막 글리프를 재면서 압축을 남긴 오류를 발견했다. `ced755949`에서 실제 배치와 같은 소수 폭으로 고쳤다. 71 결과를 ced의 수용 결과로 재사용하지 않는다.

기존24의 실행 진단에서 중첩 셀은9유닛, 첫3유닛 뒤 hard/stored reset1개이며 기존 원장은 첫 내용24+20+24px를 투영했다. 첫 물리 상자6488HU와 글리프 끝이 같지 않다는 이유로 기존 exact-end 프레임 판정이 이를 거절했다. 수정 `49b064f5a8a160abb4ce6a6af539cf4d3589be22`는 원래 저장 partition·단일 행/셀·정확한 authoritative reset 컷·원본 cell 전체/첫 object box·실제 내용의 수용을 확인한 뒤 첫 상자의 빈 정렬 밴드를 보존한다. 내용이 첫 상자를 넘으면 수용하지 않는다. closing은 작게 남은 선언 높이를 강제하지 않고 같은 실제 row-cut 내용/여백 요구 높이를 쓴다. 편집/재조판·TAC·셀 내부 개체/각주·다중 reset·걸침/복수 셀·block 컷은 기존 계약이며 이 추가 query로 검증됐다고 하지 않는다. 문서 ID·고정 px·특정 자간/글꼴로 분기하지 않는다.

생산/소비는 `saved_nested_row_frames의 원본 reset 컷 + opening/closing 실제 상자 → nested_table_mixed_fragment_heights의 동일 유닛에 물리 공간 투영 → 각 source viewport의 outer margin 예약 → mixed_nested_split_from_cut의 같은 visible/flow height와 recursive 컷 → nested_cut_physical_height의 같은 부모 예약 보조값 → table_partial의 같은 start/end height override와 위 여백을 포함한 resolved_table_top → 부분 셀의 원래 Center/Bottom 정렬 보존 → 실제 표 뒤의 같은 flow advance`다. 빈 공간을 새 내용 유닛으로 세지 않으며 paint에서만 상자를 키우거나 origin을 clamp하지 않는다. 바깥여백은 흐름에서 예약하고 표 원점에서는 위 여백만 한 번 소비한다. 원래 내용 순서·누락/중복·첫 예산·이어받기·종료와 정상 대조군의 실제 실행은 아직 확인 전이다.

현재 단계 산출 후 `upstream/devel`을 다시 fetch했으며 SHA는21ed64071로 그대로다. rebase는 up to date로 종료해 이미 정렬된 커밋을 재작성하지 않았다. batch151은 소수 폭/중첩 상자 보정을 함께 Native 빌드하고 batch152/153/154는 같은 frozen head의 실제 쪽/표·선택 PNG·정상 대조군을 검증한다. 선행 source와 새 source를 구분하며 새 회귀/expectation/baseline은 추가하지 않았다.

49 source의 suite/fmt/source-unit-tier/고정 base manifest와 **Native Clippy exit0/2분22초**, Native 빌드 **exit0/434.67초**를 확인했다. frozen SHA-256은 `80a1e4fdc80595e58648e03b0b3a9fd8a1611a0091161ec6f5b084cbde730cff`다. batch152는142쪽과 실제 nested source-frame query를 확인했으며118쪽 표 상자127.7/910.5,543.8×86.7px,119쪽127.7/141.7,543.8×137.3px다. 24→49의 **전142쪽 TextRun/각주 번호의 전체 순서가 동일**하다. batch154 정상317쪽도 page plan을 유지했지만 일부 Center 좌표가 달라져 그 시각 수용을 대신하지 않는다.

batch153의 같은 기본 글꼴/독립 Print 선택 비교는1/2/10/19/118/119쪽 **89.32023/91.07011/95.23784/86.67729/92.20721/95.87528%**다.118/119 review를 직접 읽어 첫 물리 표 상자와 Center 문단의 복원, 이어받기 내용 순서, 후속 본문/마지막 항목과 footer를 확인했다.118쪽의 표 안 글자 위치·획과 일부 본문의 기호/간격 차이는 남아 세부 판독/보정 대상이며 두 점수로 전체 원본을 수용하지 않는다.1/19쪽은 여전히 미달이고91/95는 새 source에서 재비교하지 않았다. 전체 최신 Native/fresh WASM과 실제 예산 반례·정식 회귀·전체 회귀는 미완료다.

batch155는 실제 HMKMM/Windows Batang 프로그램에 U+231C/U+231F가 없고 HCR Batang은0.97em, Haansoft Batang은1.0em임을 확인했다. 같은 canonical font 공급을 유지한 Chrome CDP는19쪽 두 기호를 **HCR Batang/HCRBatang**, 자연 전진16.160187px(@16.6667px)로 실제 출력한다고 보고했다. 새 ID만 부여한 probe PNG는 기존 canonical PNG와 bytes/hash가 동일하다. 기존 배치의 반각8.33px와 paint의 자연 전진이 갈린다는 증거이며, 다음 수정은 actual fallback 프로그램/선택과 메트릭을 공유해야 한다. 특정 기호에1em/고정 px를 넣거나 주 font 전체를 바꾸지 않았다. 초기 probe의 SVGRect.toJSON 미지원 오류는 별도 파일에 보존했고 지원하는 필드를 읽어 다시 실행했으며 렌더링 회귀로 세지 않는다. Print의1/2/19쪽 폰트도 FontFile2가 포함되어 있음을 확인해 비임베드 PDF라는 추정으로 차이를 면제하지 않았다.

종료한24 batch137 driver의 실제 종료를 확인하고 raw SVG142개만 ledger 후 제거했다(**9.73GiB**). PNG/TSV/JSON/log/외부화 SVG 등 나머지 **814파일의 크기/해시를 재확인**했으며 원본/기준 PDF·새 진단·공유 target·공용 font pool을 보존했다. cleanup ledger는 `batch137-svg-removal-ledger.json`이며 증적을 source PR에 stage하지 않았다.

### 음영 없음의 줄 끝 점유와 결측 글리프 체인 — 검증 전

batch156의 PDF FontFile2와 실제 HMKMM 비교에서 표지 F1의 `가·맹·괄호·공백`은 전진폭과 glyph outline 명령이 동일했다. 첫 느슨한 ToUnicode 정규식이 bfrange의 숫자 구간을 닫는 괄호로 오독한 결과는 폐기하고 bfchar를 구분한 `bfchar-corrected-outline-comparison.json`으로 교정했다. F2 HCR Batang은 다른 프로그램이다. 이 자료는 글꼴 예외나 전체 출력 일치의 근거가 아니다. batch157은 최신49의 첫 줄 전진25.222667px와 두 번째 줄 시작147.2004px를 실제 SVG에서 읽어, 독립 Print의 각 낱말 시작점과 여전히 차이가 있음을 확인했다.

줄 끝 helper는 `shade_color != 0xFFFFFF`로 투명 sentinel을 장식으로 분류했다. 원본 `shadeColor=none`과 출력의 `model::color::char_shade` 계약에 따라 같은 공통 판정을 소비하도록 바꿨다. 실제 음영·밑줄·취소선·border/background가 있는 공백은 보존한다. 생산/소비는 기존 가운데 줄 끝 공유 경로 그대로이며 원본 특정 문서나 좌표를 분기로 쓰지 않는다. 일반 작업공간의 오래된 generated suite 때문에 첫 `cargo fmt --all`은 실패했다. 제품 실행 실패로 세지 않고 변경 파일만 rustfmt로 정리했으며 최종 review worktree에서 suite 준비 후 전체 fmt를 다시 확인한다.

결측 기호는 세션 `FontEnvironment.glyphFallbacks`에 호출자가 실제 공급을 확인한 최종 face의 순서를 선언하도록 했다(기본값 없음, IR 저장 없음, 최대256 키/키당16개, 단일 face/중복·자기 참조 거절). 원 face와 source substFont는 보존한다. `스타일 해소의 동일 face 체인 → char_width_decision의 원 face character-miss에서만 다음 선언 face의 실제 DB 폭 → emit_line_runs의 전진 → SVG/Canvas의 같은 체인`을 연결했다. 공백·primary hit·원 HFT 메트릭은 변경하지 않고 fallback에 원 face의 HFT 속성을 옮기지 않는다. SVG는 체인 전체를 한 글꼴 이름으로 임베드하던 문제를 고쳐 각 face를 따로 공급한다. Native Skia도 체인을 단일 이름으로 조회하지 않고 선언 순서를 따르며, 뒤 face의 custom 공급이 앞 face의 system 공급을 앞지르지 않게 했다. 이름 선언 자체는 backend 설치 증명이 아니므로 actual glyph 선택과 font hash를 재검증해야 한다. 고정 기호폭·주 font 전체 치환·기준 PDF 변경은 없다.

현재는 source 변경과 잘못된 선언 거절 검사만 추가한 상태다. 최신 Native/fresh WASM의 전체 페이지·확대 판독, 정상 대조군, Skia 및 최종 lint/회귀는 미검증이다. 새 실물 배치 회귀·기대값·baseline은 추가하지 않았다. 선택쪽 진단을 47입력/175글꼴 영향 범위의 통과로 대신하지 않으며 PR 생성·승인·통합은 보류한다.

추가한 glyph 체인에서 실제 program을 찾더라도 historical 메트릭 DB의7개 구간에는 U+2300..U+23FF 자체가 없다. 따라서 체인만으로 낫표 측정 완료를 주장할 수 없다. batch161은 Print19쪽 FontFile2 `/F4`의 두 기호와 실제 `HBATANG.TTF`(SHA-256 `35f84328500fc2c3ee0b148aa75de0ed384bf9eee8dee9148c01b0a11a27fe05`)가 em1024·advance1024뿐 아니라 outline 명령도 같음을 확인했다. `scripts/extract_verified_symbol_metrics.py`로 이 프로그램의 cmap/hmtx 전체 기술 기호 구간을 추출했다. 기존 generated/overlay와 entry 순서를 바꾸지 않고 별도 `font_metrics_technical_symbols.rs`에서 기존 lookup miss만 보충한다. 특정 문서·두 기호·상수1em을 분기로 사용하지 않고 실제 cmap 결측은0→None으로 남긴다. 추출 TSV 대신 ignored `batch161-print-glyph-program/full-technical-range.json`이 원 프로그램 해시/전체256 전진폭의 재생성 근거다. 신규 파일은 폰트 바이너리·glyph outline을 포함하지 않으며, 후속 source의 실제 선택/Native·WASM 확인 전이다.

b20dd1f76의 suite/fmt·고정base 두 정책 검사·Native Clippy(exit0/85.39초)·Native 빌드(exit0/459.83초)가 완료됐다. frozen binary SHA-256은 `30ec7bf3ff674b08227b805f6e4a565f8d7cb4dbe25820243d67a143891a5778`이다. batch159 기본 폰트의 표지1쪽은89.32023→**99.56972%**,2쪽은91.10305%이며 두 새 review를 직접 읽었다. 표지 제목 두 줄의 압축/시작점 차이가 해소됐지만2쪽 인용 부호는 점수 통과만으로 면제하지 않는다. batch160의 정상7입력317쪽은 전체 page plan이 이전49와 동일했고 tree 변화는 animal5/6·rowbreak14/18·manual2/50·emptyframe2/22·policy26/215, neartop0/1·endnote0/5다. 이는 쪽/내용 구성 유지의 증거이며 독립 Print·WASM 시각 통과가 아니다.

batch165의 unmodified canonical PNG와 byte-identical한 CDP probe에서2쪽 `“”`는 실제 휴먼명조 custom 프로그램이며 자연 전진20px다. SVG의 글자 존재만으로 정상 표시를 선언하지 않았다. batch166의 Print F1과 실제 HMKMM은 이 두 glyph의 outline도 같고 hmtx512/512다. Print의 여는 기호→다음 한글 원점은138.64486→156.715067px로18.070207px 전진(@19.989167px, 원문 음수 자간)하여 반각이라는 앞선 가정을 반증한다. 이 숫자는 독립 진단이며 회귀의 고정 px 기대값이 아니다. 명시적으로 검증한 TrueType 휴먼명조 세션에서 두 이중 따옴표도 기존 가운뎃점과 같이 원 hmtx 값을 사용하도록 보정했다. HFT·단일 따옴표·다른 face는 변경하지 않았다. 앞 commit의 기술 기호 빌드는 별도 frozen 대조군으로 끝까지 실행하며 이후 source와 혼합하지 않는다. 최신 source의 glyph/시각·관계형 회귀·WASM·전체 scope는 미검증이다.

추가 검토에서 **메트릭 결측을 glyph 결측으로 간주하면 안 된다**는 일반 경계를 확인했다. 실제 한컴바탕은 기술 기호를 보유하지만 historical DB가 그 구간을 수집하지 않았던 사례 자체가 반례다. 두 실제 regular 프로그램의 전체 U+2300..U+23FF cmap 유무와 hmtx를 함께 추출하고(`verified-cmap-and-widths.json`), DB 조회 결과와 별개인 지원 판정을 추가했다. 원 face의 확인된 TrueType 프로그램/같은 weight·italic과 cmap 부재가 모두 입증되고, 다음 face의 같은 프로그램 cmap 지원과 실제 폭도 확인된 경우만 폴백 측정을 사용한다. 알 수 없는 구간·다른 weight·미확인 프로그램은 기존 backend 보충 측정에 남긴다. 기본 동작을 특정 문서 속성으로 좁힌 것이 아니라 잘못된 추론을 제거한 것이다. 원 IR/cmap/font bytes를 수정하지 않는다. 이 새 guard와 source의 실제 Native/WASM·정상 대조군 검증은 아직 진행 전이다.

21fc1552c의 batch164는 확인한 폰트 프로필의19쪽을86.62744→**99.98006%**로 복원했다. 실제 CDP는 두 기호 모두 공급한 Haansoft Batang custom 프로그램을 선택했고, probe/canonical PNG가 byte-identical하다. 새 review에서 낫표 뒤 본문·줄 끝·아래 줄이 Print와 같은 위치임을 직접 읽었다. 기본 환경19쪽86.67729%는 그대로이며 프로필 대조를 기본 source의 완료로 바꾸지 않는다. 3f11558f8 Native Clippy/빌드도 exit0(77.52/409.19초)이며 batch168에서 확인한 프로필의2쪽은91.10305→**99.80806%**로 복원되고 실제 review에서 여는/닫는 인용 부호와 이웃 본문 겹침이 해소됐다. 그러나 같은 source의91/95쪽이89.56032/87.00596%로 되돌아갔다. 이중 따옴표 변경의 `glyph_w`를 가운뎃점에도 쓰면서 기존 HFT 보정이 섞인 DB 값을 실제 TTF hmtx로 오인한 **이번 구현의 회귀**다. baseline을 바꾸거나 #7445로 넘기지 않는다.

검증한 두 SFNT에서 가운뎃점·인용 부호의 hmtx도 따로 추출했다(`verified-cmap-widths-and-program-punctuation.json`). 휴먼명조의 실제512/512와 한컴바탕의 서로 다른 구두점 폭을 data로 보존하고, 확인한 휴먼명조 TrueType 선택에서만 이 원 프로그램 값을 소비한다. 기존 HFT 호환 표와 raw SFNT 프로그램을 혼동하지 않으며, 단일 따옴표에는 이 선택을 적용하지 않는다. 수정 후 최신 Native/WASM·전체 scope와 91/95 재복원은 검증 전이다.

ebc453e3e의 Native Clippy/빌드는 exit0(106.34/588.11초)이나 batch173 실제 비교는 확인한 프로필에서도2쪽91.10305·19쪽86.62744·91쪽89.56032·95쪽87.00596%다. 앞선 개선 결과를 이 head의 결과로 재사용하지 않는다. 코드 대조에서 원 HMKMM의 unitsPerEm=512와 기존 HumanMyeongJo overlay의 em_size=1024를 같은 단위로 판정한 오류를 확인했다. `verified_technical_symbol_entry`와 `verified_program_punctuation_width`의512 조건이 실제 DB1024와 맞지 않아 cmap 부재도 알려지지 않고 원 구두점 폭도 사용되지 않았다.

추출기에 명시적으로 확인한 DB em을 전달하고, 원 SFNT 폭/단위와 환산한 DB 폭/단위를 함께 보존하도록 보정했다. 원 프로그램의 cmap 유무는 단위와 독립적으로 유지하며 폭만 같은 비율로 환산한다. 원512/512의 휴먼명조 구두점은 DB1024/1024가 되고, em이 이미 같은 한컴바탕은 변하지 않는다. `프로그램 cmap/hmtx + 확인한 DB face/em → 환산 데이터/지원 판정 → char_width_decision → 공통 글자 전진 → SVG/Canvas/Skia` 소비 연결을 유지한다. 원본·Print·font bytes·historical overlay·기대값은 바꾸지 않았다. 독립 추출 기록은 ignored `batch178-metric-unit-correction/verified-raw-and-normalized-programs.json`이며 새 source 재빌드/Native·fresh WASM·전쪽 판독 전이다. 실패한 head의 fresh WASM/전체 sweep은 선택쪽 gate에서 진행하지 않는다.

**b886ff89f의 실제 검증:** suite 준비·fmt·고정21ed64071 base의 unit-tier/manifest 검사·Native Clippy(exit0/90.05초)·Native 빌드(exit0/507.02초)를 통과했다. frozen CLI SHA-256은 `5c5f43186ffe1e3148fe3d95dfe3eb4b0e87eab302022512391ca50726bb3f9b`다. 실제 SFNT 재조회와 DB 선언을 별도로 대조한 두 프로그램의 기술 기호 각256개·구두점 각5개는 단위 전후 전진 비율이 정확히 보존된다(`batch178-metric-unit-correction/independent-unit-check.json`).

batch180의 확인한 폰트 환경 선택8쪽은1쪽99.56972·2쪽99.80806·10쪽98.19634·19쪽99.98006·91쪽99.99808·95쪽99.99752·118쪽92.20721·119쪽95.87528%, 최저92.20721%다. ebc에서 발생한2·19쪽 개선 상실과91·95쪽 회귀가 이 head의 실제 출력에서 복원됐다. 2·19·91·95쪽 실제 review와 동일 배율 확대본을 직접 읽어 기호와 이웃 본문 겹침이 없고 줄 시작·끝·목록 순서가 보존됨을 확인했다(`batch186-selected-expanded-read/`, `batch188-selected-symbol-expanded-read/`). 획/래스터의 미세 차이와 선택쪽 판독을 전쪽 완료로 바꾸지 않는다. 기본 환경의19쪽86.67729%는 그대로여서 확인한 프로그램 선택과 구분한다.

batch181 정상7입력317쪽의 모든 page plan은 이전49와 같았다. tree 변화는 animal5/6·rowbreak14/18·manual2/50·emptyframe2/22·policy26/215이고 neartop0/1·endnote0/5다. 독립 Print 및 fresh WASM 시각 검증의 대체가 아니다. batch183의 새 폰트 전송/브라우저 재사용 대조3쪽은 실제 canonical PNG와 픽셀·PNG bytes가 같음을 확인했고 전142쪽 Native TSV가 실행 중이다. 같은 source의 WASM lib Clippy/새 package 빌드와 전142쪽 WASM 비교를 순차 연결했다. 최종3Clippy 묶음·workspace·Skia·전체 nextest·47입력/175글꼴 영향·개별7 review/PR/CI/merge/후속은 아직 미완료다. batch187 API에서 원7 PR 모두 open/unmerged이며 검토한 head가 그대로임을 재확인했다.

b886ff89f의 batch183 Native 전체142쪽은 Print142쪽과 같고, 최저23쪽 **91.99624%**, 90% 미만·누락0쪽이다. silhouette-only gate는 `not_evaluated`이며 전체 승인으로 표시하지 않는다. batch195의23·50·70·71·118쪽 review와 batch196의23쪽 확대본을 실제 판독했다. 70·71쪽의 표 분할·뒤 본문·각주67~70 소속은 Print와 같은 경계를 보존하지만, 23·50쪽 작은따옴표 주변 글자 간격 차이가 남아 있어 90% 통과만으로 완료하지 않는다. 118쪽의 표 내부 미세 간격도 최종 직접 판독 대상이다.

batch198에서 독립 Print23쪽 `/F1`의 작은따옴표 두 glyph와 실제 HMKMM의 윤곽·전진폭이 동일함을 확인했다(원512/512). 실제 닫는 따옴표317.60→다음 글자334.24 및 여는 따옴표439.52→다음 글자456.16의 전진16.64는 같은 font size16.64의 전각이다. 처음 진단은 결과 JSON을 저장한 뒤 console CP949 출력에서 실패했으며, 저장한 원문과 실제 원점을 다시 조회해 확인했다. 기존 TrueType 구두점 경로가 이중따옴표만 수용하여 작은따옴표를0.3em으로 좁히던 조건을 같은 확인된 raw hmtx 경로로 보완했다. `verified_program_punctuation_width → measure_char_width_embedded_decision_for_font → char_width_decision → 공통 글자 전진/배치`이며, 원 HFT·다른 face·공백의 계약은 유지한다. 새 source의 실제 재빌드·Native/fresh WASM 전쪽 및 확대 판독 전이며 baseline·새 배치 회귀를 추가하지 않았다.

batch182의 b886 fresh WASM package 빌드 및 WASM lib Clippy는 exit0, batch191 workspace build도 exit0/1081.34초다. all-target Clippy는 테스트 초기화에서 `source_single_cell_row_frames` 누락(E0063)으로 exit101이었다. 테스트의 비적용 저장 프레임을 `None`으로 보완한9541f0270은 런타임 변경이 아니며 최종 lint 묶음에서 재검증한다. root의 이전 generated suite가 없는 원본을 참조하여 root fmt는 실패했으므로 review worktree에서 suite 준비 후 검사한다. 공유 cache를 다른 디렉터리로 바꾸거나 검사 실패를 성공으로 기록하지 않는다.

batch190은 종료·소유·미사용이 확인된 이전 선택 진단의 raw SVG22개1.841GiB만 제거했다. 폰트 원 bytes·compact geometry에서 원 SHA로 복원 가능함을 먼저 입증했고 PNG·로그·원본/PDF·공유 target은 보존했다. batch184의 실제 프로세스는 완료된 batch183 결과를 hash 검증하고 있으며, 원 Native SVG 제거 후 같은 b886 package의 전142쪽 WASM을 이어 실행한다. 이후 source와 이전 package의 증적은 구분한다. 전체47입력/175글꼴 영향·Skia·전체 nextest·7 review·PR/CI/merge/후속은 미완료다.

253784355의 batch199은 suite 준비·fmt·Native Clippy(exit0/265.69초)·WASM Clippy(exit0/167초)를 통과했으나 workspace build는 디스크 여유0 상태에서 `LNK1318 LIMIT(12)`로 exit101이었다. 같은 때 batch184의 WSVG 쓰기는 `OSError: No space left on device`로 exit1 종료하여 전체 TSV가 없다. 이전 Native142쪽의 통과를 WASM 결과로 바꾸거나 환경 실패를 제품 회귀로 세지 않는다. 정확한 compiler/출력 로그와 터미널 종료를 확인했다. 종료된184가 만든 font-policy/merged raw SVG만 파일별 SHA·절대 소유 경로·보호 파일을 대조하며 제거하고, 원 WASM SVG·render tree·PNG·로그·원본/PDF·공유 target은 보존한다(`batch210-failed-wasm-svg-cleanup/`). 공간 회복 뒤 동일 source와 최종 gate의 남은 검증을 다시 실행해야 한다.

batch210은 실제 종료한184의 raw SVG249개 **35.396GiB**를 제거하고 모든 보호 파일의 SHA 불변을 확인했다. 먼저 제거한 raw SVG1개는 별도 회복 ledger로 보존한다. 공유 `target/pr-review`, 샘플/PDF와 원 WASM SVG/tree·PNG·로그·JSON은 삭제하지 않았다. 실행 중 Cargo가 없을 때 최신 upstream/devel **19245412a**로 local devel을 fast-forward하고 누적 branch를 리베이스했다. 현재 검증 head는 **717783327367995f5d49c43a7d965ae3f83c50d0**이다. 오늘할일 add/add와 후속 append 충돌에서 upstream #7712와 이번 기록을 모두 보존했다. `batch221-rebase-source-audit`는 원240개 commit의 저자·cherry-pick 메시지·순서가 그대로이고 Rust/의존성/toolchain/test/script tree가 동일함을 확인했다. 변경은 새 upstream Studio/문서뿐이다.

동일성을 확인한 batch216은 완료한 Native/WASM Clippy의 exact source 증거를 재사용했고 suite/fmt도 다시 통과했다. 공간 회복 뒤 workspace build는 exit0/317.61초, all-target Clippy는 exit0/250.34초, 새19245412a base의 manifest·unit-tier 검사는 exit0/8.48·128.52초다. 기존 LNK1318이 공간 회복 뒤 재현되지 않았음을 확인했으며 release-test Native CLI 빌드는 실행 중이다. 새 source의 최종 전체 회귀와 시각 gate는 미완료다.

통과한 현재 workspace Debug CLI를 별도 freeze한 batch219의23쪽은 **91.99624→98.09390%**다. 새 review에서 작은따옴표와 이웃 글자 간격의 큰 차이가 해소됐음을 직접 읽었다. Debug 선택1쪽이며 release-test/fresh WASM 전쪽 결과로 승격하지 않는다. 획/래스터 미세 차이와 나머지 영향쪽 판독은 최종 출력에서 확인한다.

WASM 임베딩 글꼴의 운반만 바꾸는 격리 대조212/215에서 실제2·19쪽 W 원 geometry/tree를 보존했다. 제거 전 Native font-face CSS의 원 순서/bytes를 사용하고, 같은 글꼴 bytes를 파일 URI로 운반한 SVG는 원 SHA로 정확히 복원된다. 엄격 font family/cmap/status 배열, render tree와 manifest도 같고 canonical raster의 모든 픽셀·PNG bytes가 동일했다. 두 merged SVG의 원187,614,892bytes를 compact86,479bytes와 원 font bytes로 보존했다. 실루엣 계산·glyph/geometry·gate는 바꾸지 않으며 이 계약 대조는 제품 전쪽 검증이 아니다. 다음 실제 fresh WASM에서는 원 URI로 복원한 데이터에 원 엄격 cmap 판독을 적용하고 새 head의 실제 두 쪽 픽셀 대조를 먼저 확인한다. 현재184/199 실패 기록을 성공으로 덮어쓰거나 기존 낮은 문서를 이관하지 않는다.

### 2026-10-11 자산관리규정 중복 보완 제외 후 source 정합

사용자가 자산관리규정은 별도 #7445에서 진행 중이며 아직 PR하지 않았다고 범위를 정정했다.
이번 작업의488/3a9 자산 원점·폭 보완과 그 부작용을 보정하려던 미커밋 후보를 별도 보존하고
누적 PR 변경에서 제외했다. 원7개 PR의 기여/메인터너 보완 및 순서는 그대로다.
생산 Rust·dependency source는717783327367995f5d49c43a7d965ae3f83c50d0과 동일하다.
같은 frozen Native와 실행 중인717 fresh WASM의 검증 범위를 유지하며, 새로운 source의
빌드라고 보고하거나 #7445의 완료 결과를 가져와 전체 통과로 보고하지 않는다.
전체51/175·최종회귀·개별7 review·PR/CI/merge/후속 처리는 미완료다.

### 2026-10-11 독립 Print 전쪽 확인 및 시험지 항목 범위

최신 upstream/devel `90fa2f8955c2dd21e9f0ea8fd0c6ee8dc5996a47`로 리베이스했고,
245개 커밋의 저자·메시지·적용 순서와 upstream #7706 오늘할일을 보존했다(batch268).
Rust/test/script/dependency tree는 리베이스 전과 동일하다.

가맹사업 보고서의 동일 원본·독립 한컴2020 Print PDF는 142쪽이다. source717의
Native/fresh WASM 전142쪽 최저92.20721%, 미달0이며 142개 PNG 모두 bytes가 동일하다
(batch223/224/265). 이는 확인한 Human 글꼴 공급 profile217의 결과로 기본 fallback과 구별한다.
선택18쪽 및 회귀 검사 대상60쪽의 새 review를 직접 판독했다. 60쪽의 표 셀·본문·쪽번호와
67~73쪽의 각주/표 이어받기에서 대규모 누락·순서 역전·용지 밖 출력을 보지 않았다.
50/118쪽 글상자 외곽과 목차 leader의 미세 차이는 최종 확대 판독 항목으로 남긴다.

기존5966/6718 검사의143쪽은 당시 rhwp 실측을 고정한 값으로, 이 독립 Print의142쪽과
충돌한다. 전쪽 시각 선행 조건을 확보한 뒤 두 검사의 엄격 쪽수 기대값만142로 교정했고
본문/용지 포함 관계 검사는 유지한다. 실제 새 검사 실행은 batch270 로그로 확인해야 한다.
TSV·PNG·입력/PDF/package SHA는 ignored batch265 provenance에 연결했으며 이 한 문서의
결과를 전체51/175 또는 최종 회귀 통과로 확대하지 않는다.

새 사용자 첨부91파일 목록을 항목별로 확인해 남은 시험지 전용 Rust 검사16개와 Studio
시험지 실행 진입점1개만 제외했다(batch267/269). 파일 자체와 제품 코드, 혼합 파일의
일반 검사, 원본 샘플은 보존했다. HWP3 일반 검사는 기존 시험지 수집 제외 조건을 유지한다.
E2E helper 파일은 유틸로 기록하고152파일 manifest 및 JS 구문 검사를 통과했다.
53개 이미 부재인 파일을 이번 삭제로 보고하거나 샘플을 삭제하지 않았다.
전체 회귀·세 Clippy·새 base 정책과 개별7 review/통합 PR/CI/merge/후속 처리는 미완료다.

</details>
