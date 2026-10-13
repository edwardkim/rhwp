# Issue #7445 — 최신 devel 리베이스 후 흐름 원점 보정

## 검증 기준과 현재 상태

- upstream/devel 고정 기준: `22317614b03722e95349a1d3d71c86923363c6f5`.
- 수정 전 candidate: `344db09b2d72f1e1292e0bbd59b08c0ab3387f04`.
- 수정 전 전체 nextest: 10219 PASS / 8 FAIL / 45 skip, 361.310초, exit 100.
- 이 단계는 실패 6건의 구현 보정입니다. fresh WASM·최종 전체 회귀 및 기존 완료 문서의 영향 검증은 아직 완료하지 않았습니다. PR 생성·merge를 승인하거나 재개하는 기록이 아닙니다.

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
