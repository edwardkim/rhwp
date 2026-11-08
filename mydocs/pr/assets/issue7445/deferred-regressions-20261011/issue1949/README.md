# #7813로 이관한 거대 셀 회귀 검사 원문

사용자가 최초 체리픽의 보정 범위 밖이며 이미 #7445에 이관된 검사를 제거하도록 지정했다. 기존 [#7813](https://github.com/edwardkim/rhwp/issues/7813)의 실제 parent가 #7445이고 본문에 아래 네 함수가 명시돼 있음을 API로 재확인했다. 새 이슈를 만들지 않는다.

이 네 함수는 이번 체리픽에서 새로 추가한 검사가 아니다. 원본 문서는 contributor의 영향 관측 목록에도 등장하지만, 그 사실을 별도 편집·캐시 회귀 보정의 추가 승인으로 간주하지 않는다. 앞서 이 문서를 현재 보정 보호 목록에 묶었던 분류는 이번 지시로 정정했다.

검사 원문은 [failed-wasm-tests.rs.txt](failed-wasm-tests.rs.txt)에 보존한다. helper와 전체 소스는 변경 전 `6416f23304942706d9747396d73b2b585104de68`의 `src/wasm_api/tests.rs`에 남는다. HWP/HWPX·Print PDF와 다른 API·캐시·합성 계약은 보존한다. 구현을 되돌리거나 baseline을 현재 값으로 완화하지 않는다.

- `issue2214_scoped_cache_coherence_preserves_transient_pagination`
- `issue2424_resumable_pagination_commits_only_after_final_fragment`
- `issue2424_resumable_delete_commits_only_after_final_fragment`
- `issue2424_new_edit_stales_old_job_and_sync_flush_restarts_latest_revision`

직전 source `406041dec89d09d9e3fe93cbe606d29ef2eb17dd`의 집중 nextest는24건 중22PASS/2FAIL이며, 이 네 함수 가운데 컷 변화 개수113/관측108 및 변경 페이지 목록 불일치가 실패했다. 이관은 원인 해결·시각 수용·최종 회귀 통과가 아니다. 115쪽 현재 Native 비교에도90% 미만이 남는다. #7813의 base 증거와 현재 branch의 새 컷 소유 문제를 구분하며, 해당 이슈에서 전쪽 Native/fresh WASM90% 이상·확대 판독·내용 소유/누락/중복 검증 뒤 관계형 회귀를 복원한다. 로그·TSV·중간 JSON은 ignored output에 보존한다.

네 검사만 사용하던 cut snapshot struct와 helper3개도 같은 원문에 보존하고 제거해 dead-code 경고를 남기지 않는다. 다른 검사 함수는 유지한다.


## 2026-10-11 추가 이관: 고정 px 검사 4건

이전 이관에서 아래 네 검사를 일반 API 검사로 남겨 둔 분류를 정정한다. 실제로는
부적합한 배치의 caret·selection 좌표와 첫 조각 하단 재현점을 고정하고 있었다.
사용자 지시에 따라 새 배치를 기대값으로 고치지 않고 동일 원본의 기존 #7813으로 추가 이관한다.
앞서 이관한 네 건과 합쳐 **총 8건**이며 이번 활성 검사 감소는 **4건**이다.

| 제거한 활성 검사 | 제거 전 원문 |
| --- | --- |
| `wasm_api::tests::test_get_table_bbox_at_page_for_giant_multi_page_cell` | [table-bbox-before.rs.txt](table-bbox-before.rs.txt) |
| `issue_2214_warm_deferred_tree_and_cursor_are_exact` | [issue_2214_page_local_repaint-before.rs.txt](issue_2214_page_local_repaint-before.rs.txt) |
| `issue_2214_cold_representative_queries_are_exact` | 위 원문과 전용 helper |
| `issue_2215_hwp_and_hwpx_preserve_normal_selection_oracles` | [issue_2215_selection_page_range-before.rs.txt](issue_2215_selection_page_range-before.rs.txt) |

변경 전 source는 `d0528fb55f225fba8b4b65b9ed670a8dee9c654b`, 고정 base는
`790a6f8f388445e7bff2bcb165aa7b384f9ece80`이다. #7813의 기존 전115쪽 Native/fresh
WASM 미달 증거를 유지하며, 최신 base와 이 source를 같은 원문·독립 Print·글꼴로 비교한
Native 1쪽은 HWP `77.31730% → 86.28739%`, HWPX `79.25054% → 88.85414%`로 모두 미달이다.
이번 수치는 1쪽 재확인 값이며 전115쪽 최저값 또는 fresh WASM 결과가 아니다.
기존 이관 근거가 충분하다는 사용자 지시에 따라 중복 전쪽 재출력은 중단했다.

원문·Print PDF·공유 Studio alias는 다른 API·성능·파서 계약에서도 사용하므로 기존 경로와
바이트를 유지한다. 문서 전체 피델리티 및 기존 기록의 새 컷 간극은 해결된 것으로 처리하지 않는다.
남은 `issue_2214_cell_flow_transition_baseline`와 `issue_2215_split_paragraph_*` 두 함수,
다른 table-bbox API 검사는 유지한다. 전용 cursor 구조체2개·helper8개와 selection px assertion helper1개를
제거했으며 `#[ignore]`, 조용한 return, 새 제외 조건, baseline 완화는 추가하지 않았다.

최신 입력/Print의 SHA-256:

| 파일 | SHA-256 |
| --- | --- |
| `samples/issue1949_giant_cell_nested_tables_perf.hwp`와 공유 Studio alias | `ef10261cd29325116028e4f4f3e6be1a72c675eb771bddfd8484e7fe5aa94b4e` |
| `samples/issue1949_giant_cell_nested_tables_perf.hwpx` | `fc6e5f156de470dfbb14aab392389491720ee7fb1bf6f03fe9a018e93b420c65` |
| `pdf/issue1949_giant_cell_nested_tables_perf-2020.pdf` | `0c2530d921942d9bd5335b9e130e600f08450b039f46d1494116f45e6f5a8ac1` |
| `pdf/issue1949_giant_cell_nested_tables_perf-hwpx-2020.pdf` | `ba065afdd3ace7fd6b87fe82848e6cae3d01915a1606e4d8a70e4c2153d57d73` |

이관 및 필수 lint·남은 API 검사 로그는 ignored
`output/pr-review/planet6897-20261008/linux-validation-20261011/issue7813-*.log`,
`transfer-*.log`에 보존한다. 검증 결과: fmt, Native/WASM lib/workspace-all-targets 3종 Clippy, workspace build,
고정 base 대비 manifest와 source-unit-tier 검사 모두 PASS. 남은 API 3건은
`transfer-remaining-api-tests-corrected.log`에서 **3건 실행·3건 PASS**(exit 0,
nextest `5973d5b1-6552-47d5-b80d-ee34d8cb046b`)를 확인했다.
최초 `transfer-remaining-api-tests.log`는 재배정 전 suite 번호를 지정해 0건 실행·exit 4였으며
검증 통과로 처리하지 않는다. 전체 회귀 검사는 남은 다른 실패 보정 후 재실행한다.
복원은 #7813에서 동일 원본의 전쪽 Native/fresh WASM 최저90% 이상·직접 확대 판독을 확보한 뒤
내용 소유·순서·보존·포함·겹침 관계를 검사하는 회귀로 진행한다.
