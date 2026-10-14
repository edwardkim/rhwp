# Issue #7445 — 최신 devel 리베이스 후 흐름 원점 보정

## 검증 기준과 현재 상태

- upstream/devel 고정 기준: `22317614b03722e95349a1d3d71c86923363c6f5`.
- 수정 전 candidate: `344db09b2d72f1e1292e0bbd59b08c0ab3387f04`.
- 수정 전 전체 nextest: 10219 PASS / 8 FAIL / 45 skip, 361.310초, exit 100.
- 최종 검증 코드: `0ecf268fc4df5921f0a5102385c22e3b6afcaf1d`. 전체 nextest 10227 PASS / 0 FAIL / 45 skip, Native·fresh WASM 각각 43건·1055쪽 모두 90% 이상입니다. 실제 출력 검증과 제한은 마지막 절에 기록했습니다. 아래 선행 기록은 후보별 진행 이력이며 최종 결과를 대신하지 않습니다. PR 생성·push·merge는 새 사용자 승인 전까지 보류합니다.

## 원인과 실제 소비 경로

1. `height_cursor.rs::paragraph_origin_rewinds`가 여러 쪽을 소유한 직전 문단의 첫 저장 줄을 후속 문단과 비교했습니다. 마지막 내부 rewind 뒤의 저장 프레임 원점을 사용하도록 고쳤습니다. 이 판정은 `typeset/section/post_flow.rs`의 앵커 상태와 `typeset/paragraph/stored_lines.rs`의 간격 복원 판정, `layout.rs`의 실제 흐름 배치가 함께 소비합니다. 자산관리 13쪽의 측정 원점과 paint 원점이 갈라져 본문 밖 세 번째 줄을 수용한 결함을 보정합니다.
2. `typeset/paragraph/metrics.rs::flow_advance_height`는 fit 높이에 포함된 앞 간격을 명시적인 앞 간격 trim 이후에도 흐름 전진에 남겼습니다. trim한 몫을 한 번만 빼고 뒤 간격은 후속 문단에 보존합니다. 저장 스냅샷을 다시 구성한 사다리의 trim 차단 조건은 유지합니다.
3. `typeset/paragraph/format.rs`의 저장 span fit은 정규화된 사다리의 수용 원점에 앞 간격을 다시 예약했습니다. 원본 저장 축에는 앞 간격을 예약하고, 정규화 스냅샷이 있는 소비 축에는 같은 간격을 중복 예약하지 않습니다. `total_height`의 실제 흐름 전진은 유지합니다. 고용노동부 문서의 마지막 표가 7쪽으로 밀리는 결함을 보정합니다.
4. `layout/table_partial.rs`는 첫 저장 띠를 Top 원점에 배치한 뒤 `centers_pinned_fragment`에서 다시 가운데로 이동했습니다. 시작 컷 0의 첫 저장 lead가 위치를 소유한 띠에는 두 번째 이동을 적용하지 않습니다. RowBreak 13쪽 제목의 문단 앞 간격·저장 lead를 유지하고, 이어받기와 첫 lead가 없는 가운데 정렬 경로는 유지합니다.

문서 ID·현재 출력 좌표를 조건으로 넣거나 허용치·기존 회귀 기대값을 바꾸지 않았습니다.

## 선행 검증

집중 nextest: **109 PASS / 0 FAIL**, 실행 13.339초. 빌드 jobs 8, 테스트 threads 8. #6031·#6145·#7470와 쪽수 partition 10·본문 넘침 partition 15가 통과했습니다. #6761·#6976·#7063·#7095와 공유 height cursor 대조군도 포함합니다.

현재 정본 Visual Sweep `203b68b239bb77fb435de1b3221fc9f021856937108daaf16989679728b8fe74`, 독립 한컴 PDF, 같은 전체 글꼴 공급·thinlinemode shape·2px 관용·90% 문턱으로 Native 전쪽을 비교했습니다. 캡처 동시성 8이며 로딩 대기 한도만 늘렸습니다. 비교 공식·수용 문턱은 변경하지 않았습니다.

| 원본 | 고정 base Native 최저 | 수정 전 Native 최저 | 보정본 Native 최저 | 페이지 / 미달 |
| --- | ---: | ---: | ---: | --- |
| `samples/rowbreak-problem-pages.hwp` | 93.20187% | 92.75195% | 92.75195% | 18 / 0 |
| `samples/issue6031/3249937_asset_management_rules.hwpx` | 54.60439% | 35.33074% | 91.86922% | 60 / 0 |
| `samples/issue6145/worklife_balance_index_156607916.hwpx` | 98.44841% | 7쪽 / PDF 6쪽 불일치 | 99.32236% | 6 / 0 |

자산관리 13쪽의 마지막 줄 소속과 다음 쪽 이어받기, RowBreak 13쪽 제목·외곽·다음 본문, 고용노동부 6쪽 마지막 표·내용 보존을 review PNG로 직접 판독했습니다. 미세 글꼴 잉크·괘선 차이가 남아 있으므로 픽셀 완전 일치를 주장하지 않습니다.

로컬 실행 정본: `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-failures-20261013/`의 `focused-v2.log`, `v2-native-results.json`, `v2/{rowbreak18,asset60,worklife6}/silhouette.tsv` 및 `review/`입니다. 이 로그·TSV·중간 JSON은 ignored output에 보존하며 커밋하지 않습니다. 선행 시각 producer는 `repair-v2-rhwp`이며 최종 제출용 Native/fresh WASM producer의 검증을 대신하지 않습니다.

## 남은 검증

- 거대 셀 #7114의 저장·재열기 5건은 고정 base에서 모두 통과했습니다. 따라서 현재 실패 2건은 이관하지 않고 구현에서 보정했습니다. 아래 후속 보정 결과를 참조합니다.
- `[2027] 온새미로 1 본교재` 변환 HWP는 devel 기준 baseline 48쪽으로 복구했으나, 해당 변환본의 PDF는 46쪽이어서 새 시각 수용이나 기대값 변경을 주장하지 않습니다. 원본 47쪽 PDF를 다른 변환 입력의 정본으로 대체하지 않습니다.
- fresh WASM, 기존 완료 범위의 전쪽 영향 비교, 최종 전체 nextest는 후속 검증으로 남깁니다. 기존 테스트·baseline을 통과 목적으로 완화하지 않습니다.

## 거대 셀의 새 회귀 보정

고정 base의 같은 검사 원문·원본은 **5 PASS**, 4.124초입니다. 초기 준비 오류는 최종 테스트 결과로 사용하지 않았으며, suite harness를 생성한 뒤 실제 실행했습니다. 현재 branch는 보정 전 3 PASS / 2 FAIL이므로 새 회귀로 판정했습니다. #7813의 기존 원본 시각 미달을 이 실패의 제거 사유로 사용하지 않았습니다.

관측 연결은 `cell_units`의 저장 프레임 시작 표지 → `empty_unit_owns_flow_line_box` → `is_free_leading_spacer` 및 이어받기 컷 선택 → `skip_leading_free_spacers_in_row_cut` → `writeback_reflowed_table_frames`의 저장 컷입니다. 새 프레임을 여는 원본 빈 줄을 앞 프레임의 꼬리 spacer로 처리해, 실제 3쪽 끝 컷 115의 다음 물리 시작이 117로 바뀌었습니다. 저장 과정에서 그 뒤 본문에 새 rewind를 쓰면서 원래 빈 프레임 소유를 잃었고, 재열기의 높이 예약·다음 컷이 달라졌습니다.

`empty_unit_owns_flow_line_box`에서 **실제 저장 프레임 시작 표지가 있고 양수 높이의 원본 줄을 가진 control-free 빈 문단**의 소유를 보존합니다. 앞 프레임의 꼬리 spacer, 합성 보조 유닛, 일반 로컬 재시작을 모두 물리 프레임으로 승격하지 않습니다. 공통 소유 판정이 선택·측정·배치·저장에 전달되며 저장 단계에서만 컷을 따로 추측하지 않습니다.

보정 후 기존 #7114 **5 PASS / 0 FAIL**, 4.551초입니다. 56자·61자·200자 편집 및 55자·1자·무편집 대조군의 재열기 쪽별 본문 구성을 유지했고, 독립 정본 첫 쪽 경계 검사도 그대로 통과했습니다. 검사 제거·115쪽 기대값 변경·허용치 확장은 없습니다.

같은 원본의 미편집 시각 검증은 최신 base Native 전115쪽 최저84.01039%·미달22쪽, 수정 전 head Native 최저83.70496%·미달25쪽입니다. 이 결과는 #7813의 미해결 시각 범위이며 **원본 전쪽 피델리티 완료를 주장하지 않습니다**. 저장·재열기 새 회귀 보정과 기존 시각 미달을 구분합니다. fresh WASM 및 전체 회귀·완료 문서 영향 검증은 계속 진행합니다.

추가 로컬 로그: `giant-base-tests-prepared.log`, `giant-frame-owner-tests.log` 및 `base/giant115/silhouette.tsv`·`head/giant115/silhouette.tsv`. 동일 입력·검사 원문을 대조했으며 원본/PDF와 #7114 활성 검사를 보존합니다.

## 생물독소 44쪽 영향 검증과 보정

43건·backend별1055쪽의 최종 비교를 시작한 뒤 생물독소 Native205쪽 중44쪽이64.28723%로 미달했습니다. 해당 실행은 실패와 PNG·TSV를 보존한 채 보정을 위해 중단했으며 fresh WASM 시각 완료로 보고하지 않습니다. 기존 완료 문서이므로 이관하지 않고 현재 branch에서 보정합니다.

44쪽의 표 첫 줄 `L40-like isoform X1`이43쪽의 같은 셀에 흡수됐습니다. rebase 전에는 43쪽 끝 `endCut=[1,1,1,1,1,1]`, 44쪽 시작 row118의 이어받기였으나, 최신 devel 결합 후에는 row119부터 시작했습니다. 첫 문단의 제목 간격이나 글꼴 크기 변경이 아니라 원본 셀 프레임의 닫힘 판정입니다.

원본 section4/paragraph263/row118/col2는 서로 다른 textStart0·33의 두 글줄이 vpos0에서 각각 시작합니다. 원본 셀 높이1417HU는 첫 잉크·유효 패딩 합1182HU와 그 줄의 뒤 간격까지의 전진1722HU 사이에 있습니다. 프레임이 그 공백 밴드 안에서 닫히는데 기존 판정은 전진 끝과 선언 높이의 정확한 등가만 인정해 유효한 첫 프레임을 기각했습니다.

`stored_plain_zero_origin_line_boundary`가 선언 셀 상자의 닫힘을 마지막 잉크부터 후행 전진까지의 범위로 확인하도록 고쳤습니다. `stored_plain_restart_cut`는 이 정확한 컷에서 입증한 닫힘을 source 경계의 근거로 사용합니다. 같은 원점을 쓰는 다른 텍스트 범위를 실제 원본 줄·가로 구역으로 구분하는 선행 검사, 편집·합성·TAC·개체 경로의 제외는 유지합니다. 느슨한 로컬 restart를 모두 쪽 경계로 승격하거나 source allowance로 다음 조각의 내용을 흡수하지 않습니다. shared cut은 행 스캔과 partial paint가 함께 소비합니다.

Native44쪽 보정 후 **99.99362%**입니다. [수정 후 review PNG](../pr/assets/issue7445/completed-20261009/rebase-repair-20261013/biotoxin-p044-native-review.png)에서 이어받기 첫 줄·표 외곽·뒤 본문의 보존을 직접 판독했습니다. 현재 한 쪽 선행 진단이며 전205쪽·fresh WASM 통과를 대신하지 않습니다.

집중 nextest **13 PASS / 0 FAIL**, 5.082초: #7114 저장·재열기5건, #6761 zero-origin4건, 생물독소 기존 내용 소유4건입니다. 기대값·검사 수·허용치를 바꾸지 않았습니다. 로그 정본은 `cell-frame-focused.log`, `cell-frame-diagnostic/biotoxin205/silhouette.tsv`입니다. 수정한 source에서 전체 회귀와43건 Native/fresh WASM을 다시 실행합니다.

## 전체 재검증에서 발견한 snapshot / 재구성 구분 보정

`73d19a54d296115f715ced4d8ae5d1a86da9e4c2`의 전체 nextest는 **10225 PASS / 2 FAIL / 45 skip**, 424.725초, exit 100입니다. 선행 실패 8건은 모두 통과했지만 `issue_2158_hwpx_vpos_reset_preserve::hwp3_sample16_hwpx_matches_hwp5_and_hangul`와 `text_overlap_baseline::text_overlaps_do_not_grow_partition_5`가 새로 실패했습니다. 둘 다 sample16 HWPX의 같은 방화벽 문단을 23쪽 대신 22쪽의 본문 바닥 밖에 수용한 결함입니다. 활성 검사·기대값·겹침 baseline은 보존했습니다.

원본 vpos 스냅샷이 존재하는 문단도 현재 줄 위치가 원본과 같을 수 있습니다. `format.rs`가 스냅샷 존재만으로 앞 간격을 제외한 판단을 고쳤습니다. `metrics.rs::flow_advance_height`가 이미 사용하는 **원본 줄 수 / 실제 vpos 변환 비교**와 같은 기준으로, 실제 재구성한 사다리만 수용한 소비 원점에서 fit합니다. 보존된 저장 축에는 문단 앞 간격을 계속 예약합니다. 원본·포맷 이름이나 특정 문서 ID로 분기하지 않습니다.

대조 실험에서 모든 경로에 앞 간격을 예약하면 sample16은 복구되지만 고용노동부 문서가 다시 7쪽으로 밀렸습니다. 포맷 출처의 기존 HWP3 플래그만 사용해도 sample16 HWPX의 문제가 남았습니다. 두 실험은 최종 구현으로 채택하지 않았으며 로그를 보존했습니다. 실제로 재구성된 축을 구분한 보정 후 **6 PASS / 0 FAIL**, 0.967초입니다. sample16 내용 소유·겹침 partition, 고용노동부 안 여백 2건, 자산관리 꼬리 본문과 온새미로 대조군을 함께 확인했습니다.

보정 전 source의 생물독소 전 205쪽 Native/fresh WASM은 각각 최저 **92.75882%**, 미달 0쪽입니다. 규제분석 Native 전 17쪽 최저 **94.34255%**, 미달 0쪽이며 WASM도 완료했습니다. 이 결과를 새 source의 전쪽 증거로 대체하지 않습니다. `final-v3-whole` 실행은 실패와 완료된 PNG·TSV를 보존한 채 sample16 보정을 위해 중단했습니다. 최종 커밋의 fresh WASM·전쪽 비교·전체 회귀·필수 lint/Skia는 다시 실행합니다.

로컬 정본은 `final-v3-full-nextest.log`, `fit-before-band-focused.log`, `fit-spacing-consumer-focused.log`, `fit-rebuilt-ladder-focused.log`, `final-v3-whole/results.json`입니다. 수정 방향이 잘못된 대조 결과도 실패 상태로 남기고, 테스트 통과나 전쪽 완료로 보고하지 않습니다.

## 표 앞 제목의 흐름 간격 복구

`dc0010320a3c2b959a2b91ddf64dbf96e30768aa`의 Native 비교에서 sample16 HWPX 45쪽81.96319%, 46쪽89.76911%, 49쪽86.25559%와 개인정보 42쪽30.65766%가 확인됐습니다. 평균이나 회귀 검사 통과로 이를 수용하지 않았습니다. `final-v4-full-nextest`는 실제 컴파일 중인 작업의 소유 프로세스만 중단했고 exit130을 보존했습니다. 최종 회귀 summary가 없는 실행입니다.

sample16의 표 앞 제목 `pi731`은 수정 전 흐름 전진24.9px에서17.3px로 바뀌었습니다. 앞 간격7.6px를 추가 차감한 영향이 뒤 표 원점에 남았습니다. 해당 문단의 fit은 원본 앞 간격을 예약하며, 이 반환 경로의 전진에도 그 몫이 필요합니다. `metrics.rs`의 추가 앞 간격 차감을 제거해 기존 수용 높이와 뒤 간격의 합을 유지합니다. 앞 간격의 소실을 허용치나 특정 표 좌표로 덮지 않습니다. 사다리 재구성 판정·dirty/합성 가드·원본 프레임 원점 보정은 유지합니다.

보정 후 기존 검사 **19 PASS / 0 FAIL**, 16.760초입니다. 원래 실패8건의 입력·관계와 sample16 내용 소유·겹침, #7114 5개 편집/재열기, #7470 원본 컷과 제목 소유 대조군을 포함합니다. 테스트나 baseline 변경은 없습니다. Native 선택 쪽의 재비교는45쪽**99.02821%**,46쪽**99.36060%**,49쪽**98.88525%**이며, 전64쪽·fresh WASM 완료를 대신하지 않습니다.

개인정보42쪽은 초기 리베이스 head344db09b2와 모든 후속 후보에서 같은 첫 줄 중복이 재현됐습니다. 리베이스 전 완료 producer에서는 이어받기 첫 본문이 `동일한 방법 및 절차`이지만 초기 리베이스부터 이미 앞 쪽의 `ㅇ (비용)` 줄이 다시 표시됩니다. 이 결과를 제목 간격 수정 때문으로 분류하지 않습니다. 페이지41/42의 row/startCut/endCut은 같지만, 새 straddle 셀의 accepted content cursor·paint 소유 창을 추가로 추적하고 있습니다. 기존 완료 문서이므로 이관하거나 수용하지 않습니다.

정본은 `fit-preserved-before-focused.log`, `fit-preserved-before-producer.json`, `preserved-band-diagnostic/sample16-hwpx/silhouette.tsv`, `sample16-{before,current}-adv.log`, `pii42-attribution/`, `final-v4-full-nextest-stopped.json`입니다. 새로운 전체 회귀와43건 Native/fresh WASM, 필수 lint/Skia 게이트는 아직 완료하지 않았습니다.


## 개인정보 이어받기의 문단 사이 원본 프레임 복구

독립 Print PDF와 리베이스 전 정상 producer를 다시 대조했습니다. 개인정보 41쪽의 실제 종료 컷은 마지막 두 칸이 각각1유닛이며, 초기 리베이스 이후에는0유닛으로 바뀌었습니다. 앞 기록의 “row/startCut/endCut은 같다”는 비교는 초기 리베이스 head와 후속 후보 사이에만 해당합니다. 정상 producer까지 포함한 비교에서는 같지 않습니다.

왼쪽 분류명은 문단0의 `11.비용편익`과 문단1의 `분석`이 각각 원점0이고, 다음 문단 `(정성분석)`이1300+780HU만큼 전진합니다. 오른쪽 본문도 첫 두 원본 줄이원점0이며 다음 줄은1100+660HU만큼 전진합니다. 앞 쪽의 한 줄과 뒤 쪽의 정상 사다리를 인증하는 동일한 근거입니다. 새 셀 유닛 producer는 문단 안 재시작만 프레임으로 인정하여 분류명에 `hard=true/frame=false`를 남겼습니다.

`is_stored_frame_rewind`의 문단 사이 판정에도 같은 원본 전진 인증을 적용합니다. 원본 태그·동일 수평 영역·원점0·뒤 줄 전진·비편집/비재흐름/일반 텍스트·RowBreak 조건을 확인하며, 문서 ID나 현재 px 위치를 조건으로 쓰지 않습니다. 같은 전진 helper를 기존 행 컷 인증과 문단 안 프레임 인증도 사용합니다. 중간 후보의 행 컷 생성기 변경과 임시 진단은 제거했습니다.

선택 Native42쪽은30.65766%에서 **99.99358%**로 복구됐습니다. 본문 첫 줄은 `동일한 방법 및 절차`, 왼쪽 남은 문단은 `분석`·`(정성분석)`이며, 앞 쪽 소유의 `11.비용편익`·`ㅇ (비용)`을 반복하지 않습니다. review PNG에서 본문·분류명·하단 표를 직접 대조했습니다. 이 선택 쪽 결과는 전66쪽/fresh WASM/최종 전체 회귀 완료를 대신하지 않습니다. 회귀 검사·기대값·baseline을 변경하지 않았습니다.

선행 정본: ignored `pii-cross-paragraph-diagnostic/pii66-declared/silhouette.tsv`, `render_tree/`, `review/review_042.png`. 최종 producer를 새 커밋에서 빌드하고43건 Native/fresh WASM 전쪽과 전체 회귀·필수 게이트를 재실행합니다. PR/merge는 계속 보류합니다.


## 35쪽의 인증된 TAC 축과 합성 행 반례

커밋6504bf9ae의 전체 nextest는 **10226 PASS / 1 FAIL / 45 skip**, 실행408.967초, exit100입니다. `issue_6712_cell_visual_row_units::equal_vpos_without_different_columns_is_not_a_wrap_fragment`가 실패했습니다. 최종 통과로 보고하지 않습니다. Native43건·1055쪽은 완료했고, HWP판·2010 alias·2022판의35쪽3건만90% 미만입니다. 개인정보 전66쪽 Native/fresh WASM은 각각 최저92.53112%, 미달0입니다. 이 후보의 이후 WASM 전체·실제 출력 작업은 미달 보정을 위해 중단했습니다.

합성 행의 선언 높이는 전체 줄들을 포함하며, 첫 두 원점0 줄 뒤의 정상 전진만으로 물리 프레임을 인증하면 안 됩니다. 새 전진 인증은 선언 셀이 첫 줄의 advance와 유효 패딩 안에서 끝나는 경우로 제한합니다. 개인정보의 선언450HU와 다음 프레임의 원본 전진은 이 근거를 만족하지만, #6712의 전체 행 선언7200HU는 첫 줄1200HU 안에서 닫히지 않습니다. 기존 반례나 회귀 기대값을 변경하지 않습니다.

HWP35쪽은 초기 리베이스부터 차이가 있었습니다. 직전TAC 표 문단602의 원본 줄8032HU와 수용 pen107.0933px·끝140.8px는 같은 축입니다. 다음 원본 표 문단603의 줄11128HU(148.3733px)에서 그 문단의 앞 간격7.5733px를 빼면 실제 수용 끝140.8px와 일치합니다. 후처리가 모든TAC 뒤의 page base를 지우면서 다음 표의 원본 pen을 전달하지 못했습니다. 정상 producer에서는 이 축으로 전달한 inline placement가225.9px에 표를 두지만, 초기 리베이스부터218.3px에 놓였습니다.

`stored_tac_successor_keeps_axis`는 원본 단일TAC carrier의 선언 외곽과 저장 줄 높이, 비편집·비재구성 원본, host pen과 placement, 다음TAC 줄의 앞 간격 및 실제 수용 끝의 동일성을 함께 인증합니다. 이 경우에만 fit 축을 유지해 다음 표의 확정 inline placement를 paint에 전달합니다. `advance_end`만으로 축을 유지하지 않으며, 일반TAC·부동 개체·재구성·불일치 원본의 축 종료는 유지합니다. 개별 문서 ID나 현재 px 위치를 조건으로 쓰지 않습니다.

보정 후 집중 회귀는 **44 PASS / 0 FAIL**, 6.839초입니다. #6712의 합성 반례와 실제 그림 줄, #6761 원점0 원본/변형, 생물독소 소유, #7114 편집/재열기, 원래 실패 범위와 본문/겹침 partition 대조군을 포함합니다. 선택 Native35쪽은HWP·2010 alias **94.35054%**,2022판 **96.61619%**입니다. 개인정보42쪽 **99.99358%**, HWPX45·46·49쪽99.02821%·99.36060%·98.88525%도 유지했습니다. 선택 결과는 새source의 전쪽/fresh WASM/전체 회귀 완료를 대신하지 않습니다.

정본은 ignored `final-v6-full-nextest.log`, `final-v6-whole/results.json`, `final-v6-stopped-processes.json`, `s16p35-attribution/`, `v7-diagnostic/`, `v7-source-axis-focused.log`입니다. 새 커밋에서 전체 전쪽 및 필수 검증을 다시 실행하며, PR·merge는 보류합니다.


## 최종 전쪽·회귀·실제 출력 검증 완료 — 2026-10-14

검증한 코드 head는 `0ecf268fc4df5921f0a5102385c22e3b6afcaf1d`, 고정 base는 `22317614b03722e95349a1d3d71c86923363c6f5`입니다. 이후 문서·증적 커밋은 생산 소스와 회귀 검사 트리를 변경하지 않습니다. 원래 실패와 중간 후보의 실패·중단 기록을 보존했습니다. 이번 리베이스 보정에서 회귀 함수·assertion·기대값·baseline을 제거하거나 완화하지 않았습니다.

| 최종 검증 | 결과 |
| --- | --- |
| 전체 nextest | **10227 PASS / 0 FAIL / 45 skip**, 380.281초, exit 0 |
| Native 전쪽 Visual Sweep | **43건·1055쪽**, 최저 **90.06028%**, 미달·누락 0쪽 |
| fresh WASM 전쪽 Visual Sweep | **43건·1055쪽**, 최저 **90.06028%**, 미달·누락 0쪽 |
| fmt·Native/WASM/all-target Clippy·workspace build·manifest·unit tiers·Skia 빌드 | 필수 게이트 11개 전부 exit 0 |
| Native Skia lib | rhwp 3864 PASS와 지원 crate 182 PASS, 합계 **4046 PASS / 12 ignored** |
| Skia 집중 PNG / PDF | **2 PASS / 4 PASS**, 실패 0 |
| 실제 Skia 화면·인쇄 | 12개, 최저 **98.35333%**, 미달 0 |
| 실제 Studio Canvas2D 화면·인쇄 | 16개, 최저 **98.01524%**, 미달 0 |
| 실제 CanvasKit compat / default, software surface | 각 4개, 각각 최저 **99.62391%**, 미달 0 |

빌드 jobs 8, 테스트 threads 8, 전쪽 캡처 동시성 8로 실행했습니다. Cargo 작업은 순차 실행했습니다. WASM은 Mac 로컬 `--no-opt` 대체 빌드이며 Docker 최적화 빌드 통과로 보고하지 않습니다. 루트 pkg·Studio/public의 JS/WASM hash와 실제 Studio 초기화 응답의 hash를 대조했습니다. 비교기 hash `203b68b239bb77fb435de1b3221fc9f021856937108daaf16989679728b8fe74`, 2px 관용·90% 문턱·독립 PDF를 유지했습니다. 완전한 준비 SVG와 글꼴 감사가 같은 경우에만 기존 PNG를 재사용하고 모든 페이지 TSV를 다시 산출했습니다. 재사용 여부와 준비 SVG hash는 각 실행 manifest에 있습니다.

전체 회귀의 최종 로그는 `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-failures-20261013/final-v7-full-nextest.log`입니다. 전쪽 2110행의 통합 TSV는 같은 경로의 `final-v7-all-pages.tsv`, 각 원본 TSV와 실행 신원은 `final-v7-whole/`에 있습니다. 실제 출력 36개의 독립 PDF 비교는 `final-v7-actual/independent-print-summary.json`과 그 아래 review PNG로 보존했습니다. `.log`는 ignored output에만 있습니다.

### 직접 판독과 실제 backend의 제한

실제 화면·인쇄 36개를 모두 열고, 변경한 6개 입력과 CanvasKit 대조군 2개는 독립 Print review 패널도 확대 확인했습니다. 개인정보 42쪽은 첫 본문 `동일한 방법 및 절차`와 남은 분류명 `분석`·`(정성분석)`을 보존하며 앞 쪽의 비용 첫 줄을 반복하지 않습니다. sample16의 제목·표 간격, 생물독소의 `L40-like isoform X1` 이어받기와 뒤 본문, 자산관리 꼬리 본문, rowbreak 제목의 원본 lead, 고용노동부 마지막 표의 행·본문 포함을 확인했습니다. 미세한 글리프 래스터·굵기, 제품 편집 가이드, 일부 그림의 채움 색 차이는 남아 있으며 100% 픽셀 동일성을 주장하지 않습니다.

CanvasKit 요청의 개인정보 입력은 문서 preflight의 기존 작업 한도 50000을 넘었습니다. portable WASM 진단에서 54쪽까지 스캔한 뒤 pageIndex 54에 `workLimitExceeded`, `complete=false`가 확인됐습니다. 고용노동부 입력은 기본 Studio 환경에서 `한양신명조` family가 준비되지 않아 Canvas2D로 대체됐습니다. 두 캡처는 대체 출력의 증거이며 해당 원본의 실제 CanvasKit 통과 증거가 아닙니다. 대체 결과·선택 이유를 보존하고 한도·글꼴 준비 guard를 우회하지 않았습니다. 실제 CanvasKit 검증은 지원되는 low-slack·저장 줄 결측 입력에서 compat/default와 화면/인쇄 모두 수행했습니다.

실제 Skia CLI는 명시적 `--font-environment` 옵션을 지원하지 않으므로 지원되는 Native 글꼴 치환 환경과 기준 compat를 사용했습니다. Studio는 oracle 환경을 주입하지 않은 기본 제품 환경입니다. 전쪽 Visual Sweep의 명시적 기준 환경과 이 실제 출력 환경을 구분해 JSON에 기록했습니다. 최초 하네스의 지원하지 않는 옵션 및 host 브라우저 모드 실패도 output에 보존했고, Mac headless Chrome과 올바른 옵션으로 다시 캡처했습니다. 이 실행 오류를 제품 회귀나 통과로 섞지 않았습니다.

최종 [검증 JSON](../pr/assets/issue7445/completed-20261009/rebase-repair-20261013/final_validation.json)에 입력·독립 PDF·코드·비교기·Native/WASM producer hash, 실제 backend/profile, 필수 검증 명령과 결과를 기록했습니다.

- [sample16 HWP 35쪽 Native review](../pr/assets/issue7445/completed-20261009/rebase-repair-20261013/sample16-p035-native-review.png): **94.35054%**.
- [개인정보 42쪽 Native review](../pr/assets/issue7445/completed-20261009/rebase-repair-20261013/pii-p042-native-review.png): **99.99358%**.
- [개인정보 42쪽 실제 Studio 인쇄 review](../pr/assets/issue7445/completed-20261009/rebase-repair-20261013/pii-p042-studio-print-review.png): **98.01524%**.
- [low-slack 실제 CanvasKit 인쇄 review](../pr/assets/issue7445/completed-20261009/rebase-repair-20261013/low-slack-p001-canvaskit-print-review.png): **99.89896%**.

GitHub 게시·push·PR 생성·merge는 수행하지 않았습니다. 기존 오늘할일과 닫힌 PR #7871 검토 문서의 미커밋 초안은 그대로 보존합니다. 사용자의 새 승인 후 PR 준비 단계를 진행합니다.

## PR #7882 병합 후 재동기화 — 2026-10-14

PR #7882의 실제 병합 SHA `86a3c88fc89de9a2306a10bdda9a31023f0e24a5`가 `upstream/devel`에 포함됨을 확인했습니다. 기존 head `95ae552f30cc5d97f2217eb01638db6f4c373a4b`의 170개 커밋을 이 base로 모두 리베이스했으며, 리베이스 직후 head는 `9b107d1230b92622ad5194bb72b83c9ea251b0e0`입니다. 이전 절의 통과 결과는 이전 코드의 증거이며 새 head의 통과로 재인용하지 않습니다.

백업 ref `refs/codex-backups/7445-completed-before-7882-rebase-20261014`와 보존 stash `a8288afd56f1bcc5709e5839231b51fe3916993f`를 유지합니다. 오늘할일·PR #7871 검토 초안은 복원 후 SHA-256으로 원래 바이트와 같음을 확인했으며 이번 보정 커밋에서 제외합니다. 원본·PDF·다른 작업트리·공유 target은 보존했습니다.

### 충돌 해소와 API 연결

upstream의 물리 행 높이·이월 원장을 유지하면서 이 브랜치의 저장 프레임 원점·제목 여백·원본 글꼴 경로를 연결했습니다. 중복 원점 판정은 `stored_page_origin`으로, 글꼴 파일 탐색은 typed filename을 사용하는 `svg/font_file_lookup`으로 합쳤습니다. 모든 검색 루트에서 원 face를 먼저 확인하는 upstream 계약과 일반 대체 글꼴을 허용하지 않는 기존 엄격 경로를 함께 보존했습니다.

| 경계 | 연결 및 보존 근거 |
| --- | --- |
| 부분 표 | 내용 컷과 물리 예약 높이를 구분합니다. 일반 source 컷의 `physical_height`는 `None`이며 기존 별도 행 높이를 이중 계상하지 않습니다. spacer recut은 수용한 `fragment_row_heights`를 실제 배치에 전달합니다. |
| 저장 본문 프레임 | 자격 판정 중 재귀 적용을 막는 query는 `false`, 실제 paint query는 입증된 `source_body_frame`을 전달합니다. straddle의 시작·끝 컷 및 block 소속을 새 공통 query에 전달합니다. |
| 온전한 표 | 전체 행 paint 합과 선언 object 높이가 일치하는 경우에만 전체 표의 물리 높이로 재예약하는 upstream 판정을 복구했습니다. 첫 조각의 선언 높이를 전체 높이로 오인하지 않습니다. |
| 인라인 원점 | 계산된 x와 저장 horizontal pen의 소속을 구분합니다. 원점 증거가 없는 신규 producer는 임의의 후속 본문 owner를 지정하지 않습니다. |
| 기존 이관 | 거대 표·2025 편람의 이미 이관된 출력 검사를 재도입하지 않고 원본·증적·범위 밖 IR 검사를 보존했습니다. API v2.4의 upstream #7731 이관과 독립 규제영향분석 대조군 검사를 유지했습니다. |
| 기존 완료 문서 | KPS의 upstream 행 보존 검사와 이 브랜치 source cell 소유 검사를 함께 유지했습니다. 저장 원점 되감기 서식의 이미 검증한 overlap 기준을 임의 제거하지 않고 새 출력에서 재검증합니다. |

최초 라이브러리 check의 27개 연결 오류와 다음 check의 남은 2개 오류를 수정했습니다. 세 번째 `cargo check --locked --lib`는 exit 0입니다. 테스트 빌드에서는 OLE 차트 보조 함수 호출 6개가 과거 미완성 검사 본문 재도입으로 남은 것을 확인했습니다. `tests/issue_1251_ole_chart_contents.rs`를 최신 upstream의 완전한 검사로 복구했으며, 전각 공백 보존 기대값도 upstream 그대로 유지했습니다. 다음 테스트 전체 컴파일은 2분 5초에 exit 0으로 완료했습니다. 집중 검사는 51 PASS / 0 FAIL, 2.500초로 완료했습니다. 이 결과는 선택 범위의 구조 계약이며, 새 전쪽 시각·전체 회귀 완료로 표시하지 않습니다.

새 전쪽 비교 준비 범위는 기존 43조건 1055쪽에 저장 원점 되감기 서식 4쪽, KPS 77쪽, `56345_regulatory_impact_analysis.hwp` 21쪽을 더한 **46조건 1157쪽**입니다. 현행 Visual Sweep hash `993d97fd6249fa7763d0abd72870dc9f5de86a32480157047417ee90936116ad`, 기존 입력/PDF·DPI·compat·글꼴 조건을 사용하며 2px 관용·90% 기준을 유지합니다. 빌드·테스트·캡처 동시성은 각각 8이며 Cargo는 순차 실행합니다.

새 실행 기록은 `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7882-20261014/`에 보존합니다. 지침·회귀 기대값은 이번 API 연결 단계에서 변경하지 않았습니다. GitHub 게시·push·PR·merge는 새 승인을 받기 전 수행하지 않습니다.


## PR #7882 이후 입력 위치·수집 경로 보정 — 2026-10-14

API 연결 head `4fb5584c4179436861c1928c708342f3b0b0cca3`의 전체 nextest는
**10259 PASS / 33 FAIL / 45 skip**, 363.833초, exit 100입니다. 이전 head의 전체 통과를
이 head의 결과로 재사용하지 않습니다. Native 전쪽에서는 개인정보 3·31·35·56·57쪽의
미달을 확인하여 전쪽 실행을 중단하고 산출물을 보존했습니다.

이번 입력/검사 경로 보정은 다음 세 가지입니다.

- 문단 끝 공백을 자동 번호 자리로 옮기는 추가 추정 분기를 제거했습니다. 실제 컨트롤의
  원시 위치와 upstream의 말미 fallback을 유지합니다. 기존 자동 번호 4건과 Enter undo
  검사를 기대값 변경 없이 통과했습니다.
- IR 검사에서 이미 보존한 KTX·aift 공유 입력 경로와 upstream의 보존 입력 경로를 동일한
  resolver로 찾습니다. 원본 파일이 없는 것으로 잘못 보고하던 `baseline_samples_exist`가 통과했습니다.
- 본문 넘침 수집기가 이미 제외하던 전용 장시간/기존 이관 입력을 baseline 로더에서도 같은
  범위로 제외합니다. 기존 제외 목록과 raw TSV의 숫자는 수정하지 않았습니다.
  본문 넘침 partition **16개 전부 PASS**입니다. 원본·PDF를 삭제하지 않았습니다.

진단 후보 v7의 기존 집중 검사 결과는 **59건 중 55 PASS / 4 FAIL**, 349.656초입니다.
이 실행은 dev 프로필이며 최종 release-test 전체 검증의 대체 자료가 아닙니다.
남은 실패는 고용지원 표 하단 여백, 개인정보 소제목 위여백, 각주 HWPX의 추가 쪽과 그
oracle 쪽수 검사입니다. 가스 라벨 소속·가맹사업 정렬·로드맵 내용 소속은 이후 조판 보정
후 집중 검사에서 통과했으나, 이 입력/수집 경로 커밋에는 그 조판 소스를 포함하지 않습니다.

로그 정본은 ignored `rebase-7882-20261014/full-nextest-01.log`,
`v5-focused-dev-02.log`, `v7-focused-dev.log`입니다. v7 Native 캡처에서 개인정보
31·35·56쪽은 여전히 미달입니다. 새 전쪽 Native/fresh WASM·실제 소비 경로·전체 회귀는
조판 보정 뒤 재실행해야 합니다. 회귀 assertion·숫자 기대값을 완화하지 않았으며 PR은 보류합니다.

고정 upstream 비교용 코드 archive를 별도로 빌드하는 과정에서 공유 target의 base CLI를
후보 바이너리로 잘못 복사한 초기 v4 진단을 발견했습니다. 해당 `v4-focused-native/`는
`INVALID-CANDIDATE.txt`로 무효 표시했습니다. 실제 root CLI 재빌드 후 동결한 바이너리의
`v4-focused-native-rebuilt/` 및 이후 후보별 새 캡처만 사용합니다. 최종 검증에서는 제출할
head·생산 소스 hash·바이너리 hash·프로필을 다시 고정합니다.
