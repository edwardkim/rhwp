---
kind: investigation
status: historical
canonical: mydocs/pr/archives/pr_7628_review.md
last_verified: 2026-10-11
---

# PR #7628 리뷰 — 인라인 표·탭·정렬과 첫 TAC 원점

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 기여와 누적 메인터너 보정을 포함한 통합 후보의 판정이다.
최종 통합은 아래에 기록한 남은 검증 조건을 완료한 뒤 진행한다.

결재란 원본 프레임의 행 높이·그림 포함·캡션과 뒤 본문 순서를 보정했다. 원본5개 전7쪽의 Native/fresh WASM 문서별 최저99.61781/99.91520/99.70523/99.73514/99.99166%와 정식 관계형 회귀를 검토 기록에 연결했다.

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
| 원 PR·작성자·base | [#7628](https://github.com/edwardkim/rhwp/pull/7628) / planet6897 / devel |
| 원 head | `9086277bc82e8f05e84d74eb4a0656a2f76a1e81` |
| 최초 적용 commit | `61b9248234514576b82b04e48d54660f4b62be64` |
| 이 PR의 적용 뒤 head | `5c5edd08459ac57aac9798122062071cd5b02a3e`; 기능 commit 5개, 원 저자·cherry-pick -x 보존 |
| 누적 검증 source head | `90e2c6fd2c614ba0ea7667355c4ed2f3617f6563` |
| 최신 base 정렬 뒤 code head | `ae786f75b2cee72bd5576f6473bd51d9a8216f7c`; 제품/검증 입력 tree 동일 확인 |
| 검토 branch·reviewer | `review/planet6897-20261008` / jangster77; 원 PR reviewer assign 선행 |
| 원 PR 참고 상태 | draft=False, MERGEABLE / CLEAN; 작성 시점 snapshot이며 merge 직전 판정을 대신하지 않음 |
| 경로 | collaborator 외부 PR의 주 작업공간 누적 cherry-pick 9.1.1; intake/template/local_validation/visual_fixture_evidence 및 대형 PR 보조 절차 |

관련 이슈와 해결 표현은 원 PR 본문의 범위를 유지한다. 이번 보류 기록에 새로운 closes 표현을 추가하지 않는다. 원 PR 제공 before/after와 corpus 수치는 참고이며 통합 head의 독립 실행과 구별한다.

## 구현 주장·소비 경로·검증 판정

`layout/paragraph_layout.rs`의 저장 탭 폭·탭 간격 → inline_x/줄 나눔 → 말미 공백을 제외한 정렬 폭과 실제 TextRun 원점을 대조했다. 같은 경로는 누름틀 마커를 건너뛴 저장 줄 소비도 바꾼다. `layout/table_layout.rs`의 끊긴 셀 사다리 정렬 높이와 저장 줄 없는 첫 문단 TAC 원점은 별도 소비 경로다. 공통 값 뒤 최종 앵커 선택·정렬 보정을 다시 확인해야 한다.

**미검증:** 비공개 결재란/양식 원본은 로컬에서 찾았지만 대응 Print PDF가 없었다. 공개 변환·합성 fixture의 계약 검사 PASS와 작성자 제공 이미지를 원본의 독립 출력 일치로 바꾸어 보고하지 않는다. 탭·후행 공백·Field marker·첫 문단 위 간격 각각의 정상 대조군/비적용 경계에 같은 편집을 한 저장본의 Print PDF가 필요하다. 검증 범위 최저 90%를 입증하지 못해 머지 보류한다.

공통 빌드·회귀·글꼴 등록·383쪽 시각 결과와 입력/기준 해시·증적 보존·merge 후 댓글 계획은 [#7611 공통 통합 검증](pr_7611_review.md#공통-통합-검증과-증거-보존)에 연결한다. 이 PR의 조건부 수용이 아니라 미충족·미검증 해소 후 재검토 계획이다.


## 원본·독립 Print PDF 재확보와 최신 Native 비교

처음 기록의 “대응 Print PDF 없음”은 접수 당시의 상태다. 원본 5개를 지정된 다운로드 자료에서 찾았으며 모두 실제 마지막 저장 product가 Hancom2020/11.0.0.8227이었다. 해당 버전 계약대로 HWP2020 MCP의 비동기 Print method0/one-up으로 PDF를 새로 만들었고, 실제 runtime11.0.0.9136, 완료 상태·byte 수·SHA·PDF signature/EOF·쪽수를 확인했다. 기존3개 sample은 그대로 보존했다. 아래의 바이트 대조로 이 sample들이 대응 원본과 동일함을 확인했으며, 파일명만으로 수동 수정본으로 분류하지 않는다. 서비스 주소·token은 공개하지 않는다.

| 원본 | 독립 Print PDF | 쪽수 | Native 전쪽 최저 | 판정 |
| --- | --- | --- | --- | --- |
| [36308670](../../../samples/issue7344/original/36308670_seoul_approval.hwpx) | [Print PDF](../../../pdf/issue7344/36308670-2020.pdf) | 2 | 87.25354% | 미충족 — 2쪽 표 높이/뒤 내용 차이 |
| [36312980](../../../samples/issue7344/original/36312980_seoul_approval.hwpx) | [Print PDF](../../../pdf/issue7344/36312980-2020.pdf) | 1 | 99.91520% | Native 점수 충족; fresh WASM·최종 직접 판독 필요 |
| [36414761](../../../samples/issue7344/original/36414761_seoul_approval.hwpx) | [Print PDF](../../../pdf/issue7344/36414761-2020.pdf) | 2 | 99.70523% | Native 점수 충족; fresh WASM·최종 직접 판독 필요 |
| [36455985](../../../samples/issue7344/original/36455985_seoul_approval.hwpx) | [Print PDF](../../../pdf/issue7344/36455985-2020.pdf) | 1 | 99.73514% | Native 점수 충족; fresh WASM·최종 직접 판독 필요 |
| [36494836](../../../samples/issue7344/original/36494836_seoul_approval.hwpx) | [Print PDF](../../../pdf/issue7344/36494836-2020.pdf) | 1 | 99.99166% | Native 점수 충족; fresh WASM·최종 직접 판독 필요 |

실행 source는 `f9714465180585f3ed6eb56d9b3df4e17793f4c6`이며 전7쪽을 비교했다. 36308670의2쪽 review/overlay를 직접 판독해 첫 이어받은 행이 짧아 사진 행과 뒤 `끝.`이 위로 당겨진 것을 확인했다. pypdf로 독립 PDF의 실제 괘선 좌표를 읽었으며 사진 행 시작은195.46667px, Native cell 시작은180.5px였다. 단순 글꼴 농도 차이로 분류하지 않는다. Native 전체 score는 이 표와 같고, fresh WASM 원본7쪽 검증은 아직 남는다. `candidate-f97144651-pr7628-original-native.json`, `candidate-f97144651-pr7628-original-panels/`, `pr7628-36308670-pdf-p2-paths.json`을 ignored output에 보존한다.

base74의 같은 원본·새 Print PDF Native 전2쪽 최저42.02958%를 별도로 얻었지만, 임베딩 face와 직접 판독/fresh WASM 및 실제 실패 검사와의 연결을 확인하기 전에는 새로운 #7445 이관을 확정하지 않는다. base74→5168의 원본 read/export 소스 동등성 추론은 새5168 실행과 구별한다. 현재 브랜치87.25354%만 보고 이관하지 않으며, 정상 테스트·원본·PDF는 보존한다. base에서90% 이상인 실패는 현재 branch 개선 대상이고 정확히90%도 포함한다. 다른4개 원본의 Native 개선은 첫 원본의 남은 배치 차이나 실제로 편집한 입력의 독립 Print 검증을 대신하지 않는다. 새 회귀·golden·baseline은 이 자료 보존 commit에서 추가/완화하지 않았다.

원본/PDF의 정확한 SHA-256과 보존 경로는 아래 표이며, 실제 검증 파일과 바이트 동일함을 확인했다.

| 문서 | 원본 SHA-256 | Print PDF SHA-256 |
| --- | --- | --- |
| 36308670 | `d65b785ece5319287c6fdd4b78742b25a856a86b40b0e6b8b14371dd3cbfc134` | `94ff472c75bc2f5e4a211ae743c841f13da8df4b49296da861c5e14ed9603e5c` |
| 36312980 | `57e3acec05d3c856e931b5c4146fae6ecd3f96c6356fc7d6d99c501a83a34e21` | `ef35e530d654dcbfc9c4f008e89b2280d7a297b0e7bbcb2f39efef344ea4532b` |
| 36414761 | `b6dd9fef43c34a099384cb2e0a0795642dc268a3a50f54affc713f4814717541` | `77b776755b080f5f8621f53910636e2922fc2d407cbf37afdc8e5750d575bbc3` |
| 36455985 | `9518b5ee4ca2c625b9b6ac8504def1915472bd7be60023eec9b79a4f38f045a2` | `6b6dc687b810ab6a1f6def894334fcf9e2bde1084d1a7ad6bf38b1416fc2b8a7` |
| 36494836 | `4103c9e87b43bd783043c9d757d83179363e9ca5e50cc6c50bcea7220ac86358` | `485e04ee4d309235cdbb0bcb345432c879391d850e1b4831a3d22136a2dbabde` |

## HWPX 저장 행의 물리 높이 보정 후보

리베이스 뒤 `2288ccb2cdbe78c2781d73025932a9277738bdff`에서 남은 차이를 추적했다. 실제 Hancom 저장 원본 36308670의 XML `pageBreak="CELL"` 행은 두 Center 정렬 소유 셀이 같은 선언 높이 35618HU를 갖고, 본문 셀의 저장 문단 vpos가 쪽 경계에서 다시 시작한다. 독립 Print PDF의 앞/뒤 행 조각 높이 합은 약 474.36px로 선언 높이 474.91px에 대응한다. 기존 Native 조각 합은 약 459.8px이며 뒤 사진 행이 약 15px 위로 당겨졌다. 이 수치는 진단 관측값이며 회귀 기대값으로 고정하지 않는다.

후보는 저장본·비편집·비재조판 HWPX CELL 경로에서 연속 열을 완전히 덮는 비병합 소유 셀, 동일 선언 높이, 비Top 정렬, 유효한 저장 문단 프레임 재시작과 비투영 내용 조건을 확인한다. XML CELL은 `parser/hwpx/section.rs`에서 IR `RowBreak`로 변환되므로 이 내부 속성으로 분기한다. 기존 HWP5 RowBreak 수용 조건과 편집/재조판 경로는 유지한다. 특정 문서 식별자나 위치 상수를 추가하지 않았다.

실제 소비 경로는 `layout/table_layout.rs::stored_full_width_row_declared_height` → `typeset/table/continuation/fragment/emit.rs`의 수용 예산 및 `stored_row_box_sum` 누적 → 같은 emit의 실제 수용 높이를 뺀 `start_row_height_override` → `scan/runner.rs`·행 스캔의 잔여 요구 높이 → `layout/table_partial.rs`의 행 높이와 저장 프레임 정렬 원점이다. 내용 컷과 이미 배치한 물리 공간을 구별하며, 측정 이후 paint가 다른 높이를 추정하는 처리를 추가하지 않는다.

**미검증:** 이 후보의 컴파일·작은 경계 실행·새 Native/fresh WASM 출력은 아직 실행 전이다. 앞/뒤 조각의 내용 소속·누락·중복, 마지막 유닛 뒤 빈 밴드와 뒤 행/문단, 정상 대조군을 새 source head에서 확인한다. 이전 f971의 시각·집중 검사 통과를 이 후보의 통과로 재사용하지 않으며 새 회귀·baseline 변경도 아직 추가하지 않는다.

기존 `36312980_broken_cell_ladder_center.hwpx`, `36455985_no_lineseg_cell_spacing_before.hwpx`, `36494836_field_marker_tac_lines.hwpx`의 SHA-256은 각각 위 대응 원본과 정확히 같다. 새로 확보한 이 세 경로의 Print PDF는 동일 입력의 반복 출력이며, 별도의 편집·합성 입력 coverage로 세지 않는다. 과거의 변형본 분류는 이 바이트 대조로 정정했다.

후보 commit `46de329e53084e5c50b62555e256cf7e2f8f7e8a` 뒤 `upstream/devel`을 다시 fetch했다. base는 `6f66932a73ee6fd05a74a83bdc438c9499a89a12`로 유지되어 rebase는 up-to-date로 끝났고 원 기여/보정 순서는 바뀌지 않았다. 동일 후보를 검토 worktree에 checkout해 파생 suite 준비와 `cargo fmt --all -- --check`를 exit0으로 확인했다. 주 작업공간의 오래된 ignored generated suite를 대상으로 실패했던 앞선 포맷 실행은 통과 근거에 포함하지 않는다. Native 빌드와 시각 확인은 이 기록 작성 시 진행 중이며 이전 종료값 없는 workspace 검사는 중단 기록으로 보존했다.

이후 후보46de의 Native 빌드는447.50초/exit0으로 완료했고 원본5개 전7쪽과 동일 바이트 샘플3개의 반복 Print 전3쪽을 비교했다. 정상4개는 위 점수를 유지했으나36308670의2쪽은87.25354%로 바뀌지 않았다. 실제 scan은 `end_cut=[1,16]`, `frame=None`, `override=None`였다. 후보의 enum 조건이 XML 이름을 내부 enum 이름으로 잘못 해석해 경로가 비적용이었던 것을 파서의 CELL→RowBreak 매핑으로 확인하고 수정했다. 새 후보는 아직 재빌드·시각 실행 전이며, 이 관측을 개선 통과나 정식 회귀의 수정 전 FAIL로 세지 않는다.

다음 후보 `1e58eafab9232c7224aabe142305f6f9a8497bc3`의 Native 빌드는375.08초/exit0, 원본5개 전7쪽 비교도 완료했다. 정상4개 점수는 유지했으나36308670의2쪽은85.33707%로 미달했다. review PNG를 직접 열었고, 실제 scan의 첫 행 예약397.72px와 최종 cell 상자388.1px가 달랐다. 이어받기77.2px를 합해도 선언474.91px가 보존되지 않았다. `layout/table_partial.rs`의 배치 뒤 줄간격 접기는 HWP5 프레임만 보호하고 있었으며, 공유 helper 호출 뒤 다른 분기가 높이를 덮어쓴 사례로 판정했다.

추가 후보는 같은 유효한 HWPX 행 소유 조건으로 이미 계산된 `saved_first_fragment_source_frame`을 행 스캔 예산에 적용한다. source의 첫 개체 프레임은512.92px이고 앞 온전한 행을 제외하면 약394.53px이다. 실제 쪽의 더 큰 잔여 공간으로 이 프레임을 늘리면 다음 조각의 잔여가 줄어드므로 이 source 상자를 먼저 예약한다. 최종 접기에서도 같은 행 소유 조건을 소비해 예약된 물리 상자를 유지한다. 기존 HWP5 접기 보호 조건은 보존한다. 새 수치 상수·문서 ID·기준값 변경은 없다. 이 추가 후보의 컴파일·Native/fresh WASM·경계 검사는 아직 미검증이며,1e58의 실패와 로그/PNG를 보존한다.

`931b711e7e14a031560e44bf5b3ce93068829235`도 Native 빌드382.11초/exit0과 원본5개 전7쪽 비교를 끝냈다. 정상4개는 같았고, 첫 원본의2쪽은83.40316%로 미달했다. 실제 이어받은 행은80.4px로 Print80.23px와 가까워졌으나, review PNG 직접 판독에서 떠 있는 그림들이 아래로 이동했다. PDF ContentStream의 이미지 CTM을 별도로 읽었다. 첫 그림은 PDF y202.81867px / Native216.7px, 세 번째는202.02px /216.8px였다. 그림 높이는 원본 선언과 대응한다. 원래 상자 위치가 틀린 상태에서 그림 오차 일부가 상쇄됐으며, 행 높이만 맞춘 것으로 전체 개선을 선언하지 않는다.

원인 경로는 빈 문단의 LineSeg 높이를 조판한 뒤 Square 그림의 paragraph-relative 앵커로 `para_y`(줄 끝)를 선택하는 분기다. Source 그림은 `vertRelTo=PARA`, Top, 음수/0 오프셋이며 동일 빈 줄 상자와 겹친다. 올바른 Center 정렬은 빈 줄과 그림들의 점유 구간 합집합을 사용하고, 음수 상단까지 포함해 문단 원점을 옮긴다. 그 뒤 각 그림은 같은 원점에 자기 signed offset을 적용한다. 기존 닫힌 빈 줄/그림 프레임 결과를 확장해 측정(`height_measurer`), CellUnit, 전체/부분 셀 정렬, 마지막 picture 원점이 함께 소비하도록 하는 후속 후보를 작성했다. Top 정렬, 가시 글자, 여러 문단, inline/다른 control, 편집/재조판, 합성 LineSeg, 회전/캡션 그림은 이 새 원본 프레임의 적용 범위가 아니다.

후속 후보는 아직 컴파일·Native/fresh WASM·정상 대조군/반례 실행 전이다. 정식 회귀 추가는 관련 전쪽 양 경로90% 확인 뒤에만 진행하며, SourceHeight 보존뿐 아니라 각 그림의 셀 내부 포함과 signed offset 관계, caption/뒤 문장, 누락·중복을 독립 PDF와 대조한다. 합집합 계산을 테스트 기대값으로 재인용하지 않는다. 로그/TSV/PDF 이미지 CTM과 실패 PNG는 ignored output에 보존한다.

`796c5a08656b1aa2eccbaac6d361f3168621be9e`의 Native 빌드358.22초 exit0, 원본5개 전7쪽 비교를 완료했다. 최저값은 순서대로99.06735/99.91520/99.70523/99.73514/99.99166%이다. 모든 review PNG를 직접 읽었으며, 첫 원본의 행 테두리와 앞 네 그림은 Print와 가까워졌다. 그러나 점수만으로 전체 사진 배치 개선을 선언하지 않는다. 마지막 그림은 PDF y301.11067px에 비해 실제 Print SVG y308.60667px이고 그림 하단이 셀 하단을 벗어나는 차이가 남는다. 원본의 같은 빈 문단에는 그림 앞에 `ColumnDef`가 있다. 구조 컨트롤을 그림 점유 프레임의 실패 조건으로 취급했던 가정을 수정하여, 공간을 차지하지 않는 구역·단 정의는 합집합에서 제외하고 실제 그림이 하나 이상 존재하는지 별도로 확인한다. 이는 기존 object-only anchor의 구조 정보 처리와 같은 계약이다. 이 후속 후보의 빌드·두 출력·그림 포함 검증은 아직 미검증이다. 796의 fresh WASM 실행은 해당 코드 SHA에 고정하여 보존하며 새 코드의 증거로 대체하지 않는다.

796의 fresh WASM 빌드와 원본5개 전7쪽 비교도 exit0이며 Native와7개 raster의 SHA-256이 모두 같다. 사진의 남은 배치 결함도 그대로이므로 수용 판정은 보류한다. 후속 `2c1d2f2aa4e4c1f085bf48fa8479dd9310288dfb`는 Native 빌드381.94초 exit0이지만 마지막 그림 위치가 바뀌지 않았다. `parser/hwpx/section.rs`가 `colPr`를 HWP5 헤더 축의 `ColumnBreakType::MultiColumn`으로 합성하므로, control 루프 이전의 break-type guard에서도 이 원본 앵커가 거절됐다. 원본 XML의 pageBreak/columnBreak는 모두0이며 저장 줄은 하나다. 후속 보정은 단 구성 정보인 MultiColumn을 같은 원본 그림 프레임에서 허용하고, 명시적 Page/Column/Section 경계와 기존 편집·합성·여러 문단 제외 조건을 유지한다. 아직 후속 실행 전이며 2c의 score와 포함 실패를 보존한다. 해당 후보의 WASM·새 회귀 테스트는 추가 실행하지 않았다.

## 원본 프레임 보정의 시각 확인과 의미 회귀

제품 source `9830085e79f4d60ece53145cc3b0b67d4e9d6475`, base `6f66932a73ee6fd05a74a83bdc438c9499a89a12`의 Native 빌드534.52초 exit0 및 fresh WASM 빌드61초 exit0을 확인했다. `wasm-bindgen`은 잠금 파일과 일치하는0.2.127을 사용했다. 공유 `target/pr-review`, 빌드 잡8, 원본5개 전7쪽,96dpi/Print 프로필, 동일 입력·독립 Print PDF와 동일 글꼴 바이트로 비교했다. Native/fresh WASM의 문서별 최저값은 각각 같은99.61781/99.91520/99.70523/99.73514/99.99166%이며 미달·누락은0이다. 두 backend의 전7쪽 raster SHA-256도 모두 같다. Native 전7쪽과 fresh WASM의 첫 원본2쪽을 직접 판독했고, 같은 바이트의 정상 대조군 출력도 연결했다. TSV만으로 통합 PR 전체의 시각 gate를 통과했다고 보고하지 않는다.

마지막 그림의 실제 SVG y는301.42px로 PDF301.11067px에 가까워졌고,5개 그림 모두 실제 소유 셀의 세로 범위 안에 있다. 독립 Print와 같은 이어받기 행, 사진 순서,3개 caption과 표 뒤 `끝.`을 확인했다. 다른4개 원본과 첫 원본1쪽의 새 raster는796과 정확히 같아 정상 대조군 배치를 유지했다. 남은 차이는 작은 괘선·글꼴 AA·이미지 raster 차이다. 36414761의 기울임 본문이 셀 오른쪽에서 잘리는 부분은 독립 Print에도 같은 원문 출력으로 존재하며, 새 보정의 해결 주장에 포함하지 않는다.

증적은 ignored `output/pr-review/planet6897-20261008/candidate-9830085e7-{native,wasm}-check.json`, 각각의 `pr7628-{native,wasm}/`, compare/overlay/review 및 raster identity에 보존했다. 제품 입력/기준 해시는 위 원본 표와 같다. 대표 [Native review](../assets/issue_7344_inline_table_cell_alignment/36308670-original-p2-native-review.png)·[overlay](../assets/issue_7344_inline_table_cell_alignment/36308670-original-p2-native-overlay.png), [fresh WASM review](../assets/issue_7344_inline_table_cell_alignment/36308670-original-p2-wasm-review.png)·[overlay](../assets/issue_7344_inline_table_cell_alignment/36308670-original-p2-wasm-overlay.png)를 보존한다. 최종 제출 전 PR 본문의 실제 이미지 URL은 제출 head SHA로 고정한다.

관련 원본의 양 경로 전쪽90% 선행 조건을 확인한 뒤 `tests/cases/issue_7628_original_approval_frame.rs`를 추가했다. 기대값은 원본 선언 행 높이, 독립 Print의 문단16 경계·2쪽 소유·5개 그림·caption/뒤 본문 순서다. 절대 출력 좌표나 SVG hash를 기대값으로 쓰지 않는다. bbox 소수1자리 직렬화의 반올림 오차만 허용한다. 본문 소속/순서/누락·중복, 두 조각의 물리 높이 합, 그림의 실제 셀 내부 포함과 뒤 caption/본문 관계를 검사한다. 기존 CLI 출력의 진단에서 f971은 선언 행 높이 미보존과5개 그림 이탈,796/2c는 마지막 그림 이탈을 확인했다. 아직 정식 Rust harness의 수정 전 FAIL/수정 후 PASS를 실행한 것은 아니며, 다음 집중 실행으로 확인한다. 누적7개 PR의 나머지 시각·회귀·lint·편집 경로와 편람 HWPX 미달은 별도 미검증/미충족으로 유지한다.

테스트 추가 head `a3761ba6f30ae3919943de0bef6be6ac753d3502`의 집중 실행을 완료했다. fmt exit0, `run-rust-test.mjs issue_7628_original_approval_frame`으로 nextest를 build-jobs8/test-threads10에서 실행하여2 PASS/227 skipped, 실행0.946초, build 포함644.94초였다. 정확한 nextest metadata로 같은 컴파일된 `regression_suite_028`을 식별해 같은 Rust 검사를 직접 실행했다. f971 실행 파일은2 FAIL(exit101, 선언 행 공간 손실과 사진의 셀 밖 이탈),2c는1 PASS/1 FAIL(exit101, 마지막 사진 이탈),983은2 PASS(exit0), 마지막 정상 재실행도2 PASS(exit0)다. 입력·소스 기대값·harness는 같고 실제 출력을 생성하는 CLI만 바꿨다. Nextest가 Cargo 실행 파일 환경을 공급해 처음의 이전 CLI 지정은 적용되지 않고 PASS였으므로 그 실행은 음성 대조 증거로 세지 않는다. `candidate-a3761ba6f-focused-approval.json`, `candidate-a3761ba6f-approval-direct-controls.json`과 각 로그에 보존했다. 후속 편람 HWPX source 수정 `02de253c1`의 검증에 이 통과를 전용하지 않는다.

제품 source `8f5cf20efe4439ed477259873ad7f02ebdf6bf50`의 Native와 fresh WASM 빌드 및 원본5개 전7쪽 비교가 완료됐다. 각 문서의 양 경로 최저값은99.61781/99.91520/99.70523/99.73514/99.99166%이고 미달은 없다. 실행 파일과 package, source/입력/PDF 해시는 ignored `candidate-8f5cf20ef-native-check.json`, `candidate-8f5cf20ef-wasm-check.json`에 고정했다. 앞서983/a376에서 실행한 의미 검사의 수정 전후 결과와 이번 source의 시각 결과를 구별한다. 이번 제품 source의 정식 전체 회귀·최종 lint·누적 PR 전체 승인을 완료했다는 뜻은 아니다.
### 2026-10-10 PrEP 실패 원본의 base 판정과 저장 프레임 보완

사용자가 지정한 고정 upstream/devel `21ed64071f81f35f37618717693bd30f727392c9`에서
같은 `samples/issue2006/1790387_prep_final_report.hwpx`와 대응 독립2024 Print를 전140쪽
Native로 비교했다. 최저91.84847%, 미달0이며105쪽 예산 표20행과 마지막 합계,
108쪽 참고문헌13개를 대표 review에서 직접 확인했다. 최저27쪽의 차트 색·스타일 차이는
기존 차이로 기록하며 점수를 모든 모양의 일치로 승격하지 않는다. base fresh WASM2024는
setter 부재로 미검증이다. PrEP 관련 기존 실패5건은 #7445 이관 대상이 아니며 현재 통합
branch에서 수정한다. 근거는 ignored `upstream-21ed64071-rowbreak-prep-native-fidelity.json`,
`upstream-21ed64071-prep-base-boundary-panels/provenance.json`에 있다.

불변 중간 CLI의 동일 원본 출력에서 `02de253c1`은140쪽, `5e31d6db0` 이후141쪽이다.
첫 구조 차이는105쪽 section3 host336의 온전한20행이19행 조각이 되는 경계다.
직전 제목335의 원본 저장 원점7841HU + 줄1300 + 간격1040 + 문단 뒤1000 =
다음 표 원점11181HU다. Print 표 윗변243.2533px와 base243.5px도 같은 원점을 확인한다.
유효한 제목의 공유 끝점을 되돌리지 않는다.

`query_whole_table_fit`의 `saved_rowbreak_object_frame`은 원래 다행·비TAC·빈 단일
host·각주 없음·비양수 앵커·다음 문단의 새 쪽·실측 한 쪽 이내·셀 저장 원점 reset 없음·
단일 실제 저장 줄 조건으로 닫힌 source frame을 검증한다. HWPX 수용 조건만
`source_top < current_height`에서 `<=`로 바꿔, 바로 앞 내용이 표의 저장 원점에서 끝나는
정상 경계도 같은 frame으로 수용한다. 프레임 자체의 높이·컷·행별 요구량은 완화하지 않는다.

소비 경로는 `whole_fit.rs`의 frame → `entry.rs`의 `align_flow_to(source_top)`와
`whole_placement_height = source_bottom - source_top` → `legacy_whole_fits`와 최종
`resolved_host_placement.occupied_bottom` 예산 → `place_table_with_text`다.
source frame을 거부하던 실행에서는790.2px 일반 요구량을 검사하고, 행 스캐너가 실제
19행733px만 수용한 뒤 마지막54.8px 행을 이월했다. 이번 수용 조건은 선언 frame이
첫 조각만 뜻하는 다쪽 표·내부 reset·각주 예산을 그대로 제외하며,
`samples/issue7336/stored_frame_page_larger_rowbreak.hwpx`를 정상 비적용 대조로 재확인한다.
제품 source `f4853740ecc9f7cd3f7350d191d727cb7b240856`의 실제 Native 쪽 수는
140쪽으로 복구됐다. Native/fresh WASM의40/105/108쪽 비교는 양 경로 모두
최저91.80848%, 미달0이며 실제 raster bytes도 각각 같다. Native105쪽 review에서
20행의 본문과 마지막 연구비 총액, 표 외곽과 쪽번호를 직접 확인했고108쪽에서 원본
참고문헌13개의 소속·순서를 확인했다. 증적은 ignored
`candidate-f4853740e-native-geometry-controls/provenance.json`,
`candidate-f4853740e-{native,wasm}-boundary-fidelity.json`,
`candidate-f4853740e-native-correction-panels/provenance.json`에 있다.
이3쪽 비교는 전140쪽 최종 검증·정식 focused 검사·최종 회귀 통과를 대신하지 않는다.

40쪽 종료 조각은 실제 조판에서106.6667px를 예약하나 paint가92.0px로 다시 계산해
최종 Table bbox95.3px와 짧은 아래 밴드가 나타난다. 원본 셀의 유효 여백과 저장 종료
원점8000HU를 보존하면서 컷·점유 높이의 측정/배치 소비를 추적하고 있다.
독립 Print와 Native review의 같은 아래 밴드를 직접 대조했으며, 점수91.80848%로
이 결함을 면제하지 않는다. base90% 이상인 원본이므로 #7445로 이관하지 않는다.

40쪽의 실제 호출 진단에서는 `row_cut_content_height`가 두 경로 모두 같은 시작 컷32와
종료 유닛36, 내용84px, 유효 위/아래 여백각11.3333px, 합106.6667px를 반환했다.
그 뒤 `table_partial.rs`가 `start_row_height_override`의 남은 물리 밴드92px로
이 결과를 덮어썼다. 최종 bbox가95.3px가 되는 별도 내용 확장도 예약 높이를 복구하지 못했다.
따라서 helper가 서로 다른 값으로 측정했다는 이전 가설은 철회한다.

소비 계약은 `RowScanQuery::whole_row_height`의 `max(현재 컷의 내용+여백, 이어받은
물리 높이)` → 스캐너의106.6667px 수용·누적 예약 → 종료 PartialTable의 시작 컷·남은
물리 최소 높이 → paint의 같은 컷 높이 순서다. paint도 동일하게 큰 값을 보존하도록
수정했다. 행을 다시 자른 경우 뒤의 `end_row_height_override`가 실제 수용한 마지막
컷 높이를 정확히 적용하므로 남은 전체 밴드를 현재 조각에 강제로 얹지 않는다.
내용을 이미 모두 소비한 빈 밴드는 현재 컷 높이0과 남은 양수 높이를 비교해 계속 보존한다.
새 문서 ID·원점 상수·커서 clamp를 추가하지 않았다.

진단 전용 worktree의 로그9줄만 추가한 Native 실행은 빌드exit0와 원본 출력exit0이며,
같은 내용을 재계산한 뒤 덮어쓰는 지점을 확인한 증거다. 해당 로그 변경은 제품 수정에
포함하지 않는다. ignored `candidate-f4853740e-prep40-instrumented-path/`에 patch·
실행파일/source 해시·명령·실제 로그를 보존한다. 제품 보정의 원본39→40쪽, 첫/끝 컷·
빈 밴드·재분할 대조군·뒤 본문·최신 전쪽 Native/fresh WASM과 정식 회귀는 아직 미검증이다.

제품 `6fc92ab335d02efd8b5c3707d97c7e7b1cf68ec9`의 Native 빌드exit0에서
40쪽 표 높이106.7px, 마지막 셀 줄 뒤 여백11.4px, 뒤 본문220.2px를 확인했다.
원문 하단850HU 여백과 뒤 본문의 표 밖 배치는 복구됐으나, 이 관측은 전쪽 시각 통과가 아니다.
동시에 실물 정상 대조군 `samples/issue6929/148776468_search_ad_terms_press_release.hwp`
15쪽에서 이미 소비한 행의 빈 밴드가17.4px에서196.3px로 커지는 추가 회귀를 검출했다.
명시적 per-row 컷의 내용 높이가0일 때 paint의 생산 단계가 선언 행 높이를 남기던 것이 원인이다.
컷이 있으면0도 현재 내용 높이로 반영하고, 그 뒤 같은 물리 최소 높이 비교를 적용하도록 보완한다.
소비가 끝난 행의 내용을 다시 예약하거나 새 행에 남은 밴드를 넘기지 않는 계약이며,
마지막 컷 높이를 소유하는 `end_row_height_override`는 그대로 적용한다. 두 실행의 실제
render tree와 입력·실행파일 해시는 ignored `candidate-{f4853740e,6fc92ab33}-empty-band-control/`
및 `candidate-6fc92ab33-build/build.json`에 보존했다. 보완 후 재실행·재분할·시각·정식 검사는
아직 미검증이다. 이 추가 회귀를 #7445에 이관하거나 기존 기대값을 완화하지 않는다.

보완 제품 `e4087c00a8d5b78534e27cb79f01cdd4bf2b6917`의 Native/fresh WASM 빌드는
각각 exit0이다. 최신 Native에서 40쪽의 106.7px 표·11.4px 하단 여백·뒤 본문 220.2px를
유지하고, 위 15쪽 대조군의 빈 밴드 17.4px와 표 높이 905.0px를 복구했다. 소비한 행의
글자는 재출력되지 않는다. 진단 증적은 ignored `candidate-e4087c00a-build/build.json`에 있다.

변경되지 않은 승인 원본 검사 2개도 실제 이 CLI로 실행해 2 PASS를 확인했다. 이전에
컴파일된 assertion harness의 저장 메타데이터 파서를 사용했으며, 최신 라이브러리의
전체 nextest 통과로 보고하지 않는다. source·실행파일·harness 해시와 정확한 선택 검사·
실행 로그는 `candidate-e4087c00a-approval-existing-cli-tests.json`에 있다. 과거 suite028은
해시와 선택 검사가 바뀌어 실행하지 않았고, 실제 해당 검사 2개가 있는 suite027을 확인했다.

편람 281쪽을 포함한 Native/fresh WASM 각40쪽 비교는 실제 완료됐으며 최저94.75915%다.
PrEP39/40/105/108쪽은 두 경로 모두 최저99.78379%, 미달0이다. Native PrEP40쪽 review를
직접 열어 종료 셀의 아래 여백·표 외곽·뒤 본문 순서를 확인했다. 전체 범위·정상 대조군과
기여 commit 순서의 상세 근거는 [#7617 최신 기록](pr_7617_review.md#2026-10-10-순서-재적용과-최신-경계-검증)에
연결한다. 전383/140쪽 최종 시각·전체 회귀·lint는 아직 미검증이다. 빌드나 기존 CLI 검사
2개와 이 경계 결과로 누적7개 PR의 통합 승인을 선언하지 않는다.

</details>
