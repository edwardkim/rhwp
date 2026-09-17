---
kind: investigation
status: active
---

# #7195 결과보고 — 부분 개선 및 잔여 실패 분리

- Issue: [#7195](https://github.com/edwardkim/rhwp/issues/7195).
- 작성일: 2026-09-17. 브랜치: `task_m100_7195`.
- 검증 제품·테스트 commit: `d54459747d136cf072943d9006659027c5ad274a`.
- 검증 기록 head: `b502715ea`. PR base: `fcbd00e0fabc4b309a887357033f92e2d511cd75`.
- 상태: 구현·조사 결과와 로컬 검사 실행은 정리했으나, **전체 회귀 실패로 제출·merge 준비 완료는 아니다**.
  이번 승인으로 실패를 예외 승인하거나 #7195의 종료 조건을 충족한 것으로 처리하지 않는다.
- 보고서 준비 당시 PR 없음. 이후 실패 공개 목적의 승인으로
  [Draft PR #7222](https://github.com/edwardkim/rhwp/pull/7222)을 `369fb09514`에서 게시했다.
  [접수 기록](../pr/archives/pr_7222_review.md)은 머지 보류이며 Ready·merge·이슈 close는 미실행이다.

## 1. 해결 범위

| 대상 | 적용한 변경과 근거 | 검증·한계 |
| --- | --- | --- |
| #552, #77/#993 | ignore 해제, 입력 누락·파싱 실패의 거짓 통과 제거. 기존 조판 assertion 유지 | [stage1](../working/task_m100_7195_stage1.md): 정상 2 PASS, 입력 오류 음성 대조. 새 한컴 시각 판정과 구분 |
| #2308 중첩 폭 | owner 기반 폭 투영 시 `Cell::effective_padding`으로 table 기본/개별/명시적 0 여백을 반영. 기존 호환 분기 밖으로 적용 범위를 넓히지 않음 | [stage3](../working/task_m100_7195_stage3.md): 좌우 510HU 차감, 38245→37225HU. 저장 원문/일반 표 불변 및 폭·clip 계약 |
| #2308 이어받기 중복 | scalar continuation에서 소비한 source prefix를 다시 출력하는 되감기 제거, 같은 prefix 높이로 원점 보존 | 같은 stage3: clip에 의존하지 않는 줄 소유, 전체 원문 한 번 출력, 다른 가용 높이 반례. 작업지시자 WASM 통과 |
| #2279 저장 프레임·본문 경계 | 줄 상자와 trailing 간격 구분, 동일 문단/빈 문단의 유효 저장 reset 및 단일 행에도 공통 계산. 초과량 자체를 허용량으로 삼던 조건 제거 | stage3의 교체 입력 4–8·11·12쪽, 본문 fit/원문 보존/예산 부족 반례와 작업지시자 시각 통과. 전체 rowspan 조합은 미검증 |
| #2279 계약 | 교체 문서의 실제 문단/표 구조와 독립 저장 메트릭으로 검사. 다른 문단의 LS 유무가 NO_LS 표 분할을 결정하지 않는 합성 대조 | 승인된 계약 정비. 기존 제품 결함의 음성 대조와 테스트 정비 전후 통과를 구분 |
| #3798 계약 | 말미 간격 초과와 실제 글자 상자 초과를 구분. 원문/실제 글자 영역을 보호하는 반례 추가 | 작업지시자 실물 WASM 통과, 합성 계약 3 PASS. 합성 계약을 한컴 출력 증명으로 쓰지 않음 |
| giant HWPX 중첩 원점 | 앞에 가시 글자가 없더라도 빈 줄이 공간을 소유하면 `para_y_before_lines` 보존 | [stage6](../working/task_m100_7195_stage6.md): 실제 원점 계약 수정 전 FAIL/후 PASS, p40만 이동. 한컴 전체 피델리티는 미해결 |

구현 주장·실제 컷/요구 높이/예약/최종 배치의 연결은 stage3의 적용 규칙·호출 경로 표,
[stage5 충돌 해소](../working/task_m100_7195_stage5.md), stage6 원인 계층 기록을 따른다.
모든 중첩·rowspan 경로를 전면 재설계했다는 주장이 아니다.

## 2. 입력 및 시각 판정의 경계

검증에 사용한 아래 현재 파일은 `b502715ea`의 Git blob과 byte-identical임을 다시 확인했다.

| 입력 | SHA-256 |
| --- | --- |
| `samples/86712_regulatory_analysis.hwp` (사용자 교체본) | `ee82c7755617003cb972ba398da9cffadfed24ac0fa068eee1a5347da7658a88` |
| `samples/76076_regulatory_analysis.hwp` | `3308ba8505391bae2d0d62963e9399f4e48cdae574304cc0f89a311c6efbb6b5` |
| `samples/issue3798/page_end_trailing_spill.hwpx` | `bca8c89ca8a6a342531566f9e72c146f7393a40d5c338ec5e6f0523913bb2262` |
| `samples/table_giant_cell_overfill.hwpx` | `5d7eb4a21e46d9ad01a0f631eea2b1f2ec8a71750b4d448868e944e1b95042f4` |
| `pdf/table_giant_cell_overfill-hwpx-2024.pdf` | `7669a8f468fd47e3180a09b5ea60526785e968fcb9ad5f1039d1f585d55d59be` |
| `samples/issue1891/76076_regulatory_analysis-2024.pdf` | `06a389455d6b96e5f6580c9930fd8555256f9c712be85fb3cdaf31fc601a090d` |

86712 교체 전 입력/PDF의 비교를 교체본의 정답 증거로 재사용하지 않는다.
교체본의 4–8·11·12쪽 작업지시자 판정과 후속 byte-identical 검증을 구분한다.
HFT 글꼴 차이는 범위 밖이며, 14→13→14pt 편집 뒤 누락 해소는 [#7202](https://github.com/edwardkim/rhwp/issues/7202)다.

최종 giant 출력은 Native/fresh WASM 39–42쪽을 직접 확인했고 두 backend PNG도 일치했다.
그러나 같은 부속서 내용의 PDF41쪽과 rhwp40쪽은 다르며, p42의 기존 경계 문제도 남았다.
이를 승인·피델리티 통과로 승격하지 않는다. 대표 PR asset 보존과 최종 사람 판정은
실제 PR의 해결/보류 범위가 확정된 뒤 수행한다. 현재 임시 output은 영구 GitHub 증적이 아니다.

## 3. 로컬 검증 결과

[stage6](../working/task_m100_7195_stage6.md), [stage7](../working/task_m100_7195_stage7.md)의
정확한 명령·수치·SHA를 재사용한다. 코드 변경 없이 문서만 준비하므로 전체 회귀를 반복하지 않는다.

| 게이트 | 결과 |
| --- | --- |
| 집중 계약 (최종 제품) | 37 PASS /1 FAIL: giant 49/48쪽 계약 |
| 전체 nextest | **9932 PASS /30 FAIL /47 skipped**, 9962 실행, exit100 |
| fmt, Native/WASM/workspace all-targets Clippy | 모두 PASS |
| workspace build, base 고정 suite/unit 정책 | 모두 PASS |
| Native Skia lib | **3929 PASS /3 FAIL /11 ignored** |
| Native Skia 그림 /PDF | 2 PASS /4 PASS |
| Docker fresh WASM | PASS, stage6 빌드와 실제 Studio 제공 해시 일치 |
| 교체 sample 명시 보안 검사 | 6 PASS, 대상 HWP 1개를 명시하여 세 탐지기 실제 실행 |

Skia 실패3건은 전체 회귀 실패30건의 부분집합으로, 테스트 이름과 assertion 비교값이 동일하다.
33개 독립 실패로 합산하지 않는다. nextest0.9.137/default profile 실행이며 권장0.9.140 및
관측용 JUnit 키 경고는 그대로 기록했다. 도구 업그레이드는 하지 않았다.

추가 보안 검증은 같은 review worktree에서 다음 명령으로 실행했다.

```sh
RHWP_SECURITY_SWEEP_SAMPLES_JSON='["samples/86712_regulatory_analysis.hwp"]' \
CARGO_BUILD_JOBS=4 node scripts/run-rust-test.mjs security_corpus_regression -- \
  --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --test-threads 4
```

run ID `0fb222c4-4e61-4402-8e40-f334a90233b5`, exit0, 6 PASS, 211 필터 제외.
입력은 위 교체본 1개이며 현재 commit/review 바이트 일치를 확인했다.
로그·환경·해시는 로컬 `output/7195/stage8/security.{json,log}`에 보존했다.
이는 명시 입력으로 보완한 별도 보안 검사이며, 전체 회귀30개 실패의 해소가 아니다.

### 실패 증감의 올바른 해석

| 시점 | PASS | FAIL | skipped |
| --- | ---: | ---: | ---: |
| stage4 통합 전 | 9903 | 28 | 47 |
| stage5 devel 통합 | 9931 | 30 | 47 |
| stage6 원점 수정 | 9932 | 30 | 47 |

stage5→stage6의 신규0/해결0은 **직전 통합본 대비** 결론이지, devel에도 30건 전부 존재한다는 뜻이 아니다.
통합 시 추가된2건은 giant HWPX 49/48쪽 차이다. 해당 입력의 고정 devel은48쪽이고 통합본은49쪽이다.
기존28건의 신규 회귀/기존 결함/기대 계약 노후화/교체 입력 영향은 전수 분류 완료하지 않았다.
개별 동일입력 base 대조 없이 전부 기존 결함으로 수용하지 않는다.
PASS +1은 원점 회귀 검사 추가이며 기존 실패가 줄어든 것이 아니다.

## 4. 잔여 30건의 후속 이슈 연결

2026-09-17 조회에서 아래 후속 이슈는 모두 OPEN이었다.
stage5 매핑을 stage6의 개별 테스트 이름30개와 대조했다. generated suite 번호는 재배치되므로 키로 쓰지 않는다.
한 partition이 여러 문서를 검사하면 복수 이슈에 연결되며, 이슈별 건수를 단순 합산하지 않는다.
**이슈 등록은 실패 해소·ignore 승인·merge 예외 승인이 아니다.**

| 실패 테스트 | 후속 이슈 | 검출 시점 |
| --- | --- | --- |
| `body_overflow_baseline::body_overflow_does_not_grow_partition_7` | [#7208](https://github.com/edwardkim/rhwp/issues/7208) | stage4부터 잔존 |
| `issue_1891::issue_1891_hwp5_origin_hwpx_export_reparse_keeps_page_count` | [#7095](https://github.com/edwardkim/rhwp/issues/7095) | stage4부터 잔존 |
| `issue_2097_band_fill::issue_2097_block_band_fill_page_pins` | [#7207](https://github.com/edwardkim/rhwp/issues/7207) | stage4부터 잔존 |
| `issue_2097_squeeze::issue_2097_bottom_squeeze_page_pins` | [#7207](https://github.com/edwardkim/rhwp/issues/7207) | stage4부터 잔존 |
| `issue_3637_split_cell_nested_table_vpos::nested_table_snap_stays_inside_the_split_cell` | [#7206](https://github.com/edwardkim/rhwp/issues/7206) | stage4부터 잔존 |
| `issue_3820_rowbreak_rowspan_band::issue_3820_p4_keeps_saved_rowbreak_body_with_its_first_fragment` | [#7209](https://github.com/edwardkim/rhwp/issues/7209) | stage4부터 잔존 |
| `issue_3930_hwpx_hwp_save_layout::issue_3820_hwp5_qa_rowbreak_tail_reduces_page_count` | [#7009](https://github.com/edwardkim/rhwp/issues/7009) | stage4부터 잔존 |
| `issue_5908_giant_cell_last_fragment_overfill::issue_5908_annex_two_moves_off_the_collapsed_page` | [#7140](https://github.com/edwardkim/rhwp/issues/7140) | stage4부터 잔존 |
| `issue_5908_giant_cell_last_fragment_overfill::issue_5908_giant_cell_last_fragment_keeps_splitting` | [#7140](https://github.com/edwardkim/rhwp/issues/7140) | stage4부터 잔존 |
| `issue_5920_nested_atom_tail_page_fit::issue_5920_page8_holds_both_nested_boxes` | [#7206](https://github.com/edwardkim/rhwp/issues/7206) | stage4부터 잔존 |
| `issue_5920_nested_atom_tail_page_fit::issue_5920_page9_does_not_restart_with_the_conclusion_box` | [#7206](https://github.com/edwardkim/rhwp/issues/7206) | stage4부터 잔존 |
| `issue_6013_empty_para_stored_frame_flag::issue_6013_stored_chunk_last_line_stays_on_page_10` | [#7095](https://github.com/edwardkim/rhwp/issues/7095) | stage4부터 잔존 |
| `issue_6712_cell_visual_row_units::footer_overlay_group_images_are_owned_once_by_the_last_page` | [#7207](https://github.com/edwardkim/rhwp/issues/7207) | stage4부터 잔존 |
| `issue_6712_cell_visual_row_units::stored_square_picture_flow_matches_korean_hancom_oracle` | [#7207](https://github.com/edwardkim/rhwp/issues/7207) | stage4부터 잔존 |
| `issue_6712_title_glyph_fit::original_newsletter_title_has_fitted_glyphs_on_the_same_two_pages` | [#7207](https://github.com/edwardkim/rhwp/issues/7207) | stage4부터 잔존 |
| `issue_7080_area_dot_fullwidth::issue_7080_fullwidth_area_dot_keeps_no_ls_statute_rows_on_hancom_page_108` | [#7095](https://github.com/edwardkim/rhwp/issues/7095) | stage4부터 잔존 |
| `issue_7095_page_spanning_fragment_box::stored_midpage_cut_keeps_its_last_source_unit` | [#7095](https://github.com/edwardkim/rhwp/issues/7095) | stage4부터 잔존 |
| `issue_7140_nested_row_unit_paint_height::hwpx_page_count_matches_the_oracle` | [#7140](https://github.com/edwardkim/rhwp/issues/7140) | stage5 통합 시 추가 |
| `off_canvas_baseline::off_canvas_does_not_grow_partition_4` | [#7206](https://github.com/edwardkim/rhwp/issues/7206) | stage4부터 잔존 |
| `oracle_page_count_baseline::page_counts_do_not_drift_from_hancom_oracle_partition_1` | [#7009](https://github.com/edwardkim/rhwp/issues/7009), [#7095](https://github.com/edwardkim/rhwp/issues/7095) | stage4부터 잔존 |
| `oracle_page_count_baseline::page_counts_do_not_drift_from_hancom_oracle_partition_2` | [#7207](https://github.com/edwardkim/rhwp/issues/7207) | stage4부터 잔존 |
| `oracle_page_count_baseline::page_counts_do_not_drift_from_hancom_oracle_partition_4` | [#7095](https://github.com/edwardkim/rhwp/issues/7095) | stage4부터 잔존 |
| `oracle_page_count_baseline::page_counts_do_not_drift_from_hancom_oracle_partition_5` | [#7140](https://github.com/edwardkim/rhwp/issues/7140) | stage5 통합 시 추가 |
| `overflow_cell_lines_do_not_grow_partition_14` | [#7140](https://github.com/edwardkim/rhwp/issues/7140) | stage4부터 잔존 |
| `overflow_cell_lines_do_not_grow_partition_4` | [#7206](https://github.com/edwardkim/rhwp/issues/7206) | stage4부터 잔존 |
| `text_overlap_baseline::text_overlaps_do_not_grow_partition_14` | [#7140](https://github.com/edwardkim/rhwp/issues/7140) | stage4부터 잔존 |
| `text_overlap_baseline::text_overlaps_do_not_grow_partition_4` | [#7204](https://github.com/edwardkim/rhwp/issues/7204), [#7206](https://github.com/edwardkim/rhwp/issues/7206) | stage4부터 잔존 |
| `wasm_api::tests::issue2214_scoped_cache_coherence_preserves_transient_pagination` | [#7205](https://github.com/edwardkim/rhwp/issues/7205) | stage4부터 잔존 |
| `wasm_api::tests::issue2424_resumable_pagination_commits_only_after_final_fragment` | [#7205](https://github.com/edwardkim/rhwp/issues/7205) | stage4부터 잔존 |
| `wasm_api::tests::test_get_table_bbox_at_page_for_giant_multi_page_cell` | [#7205](https://github.com/edwardkim/rhwp/issues/7205) | stage4부터 잔존 |

## 5. 기준값·ignore 처리

- 이번 브랜치의 baseline/golden TSV 변경 없음.
- 최초 대상5건은 조사·구현·승인된 계약 정비 후 기본 실행으로 복귀했다.
- 별도 [#7009](https://github.com/edwardkim/rhwp/issues/7009)로 넘긴
  `issue_3931_keeps_pr4763_hwp_page_count_contract` 1건만 작업지시자의 명시 승인으로 ignore했다.
  기대384 및 나머지 기하 계약은 유지했다. 코퍼스 page-count partition은 여전히 실패한다.
- 이후 검출된30건은 새 ignore를 추가하지 않았으며 partition 전체를 제외하지 않는다.
- #7195의 완료 조건 중 전체 검증 통과는 미충족이다. 따라서 자동 종료 문구를 쓰지 않는다.

## 6. 제출 처리 제안

1. 이번 범위의 개선과 남은 결함을 위와 같이 분리하여 기록한다.
2. 전체 회귀/Native Skia 실패를 보류 사유로 유지하고, 정상 Open/merge 후보로 준비 완료라고 선언하지 않는다.
3. 조기 검토가 필요하면 실패와 후속 이슈를 명시한 **Draft PR 공유** 여부를 작업지시자가 결정한다.
   Draft도 별도 push·생성 승인 후에만 만든다. CI 실패를 우회하는 merge는 제안하지 않는다.
4. 후속 수정 또는 구체적인 테스트 계약/제외 변경은 개별 근거·승인을 통해 처리하며 현재 상태를 자동 통과시키지 않는다.
5. 원격 PR 번호를 미리 만들지 않는다. 실제 PR 생성 후 self-review와 최종 asset/CI를 연결한다.

PR 본문 초안과 실행 전 조건은 [PR 준비 문서](../plans/task_m100_7195_pr.md)에 둔다.
