# M01-3 오라클 커버리지

`samples/` 의 `.hwp`/`.hwpx` 와 `{stem}-{year}.pdf` (2018/2020/2022/2024) 짝.
같은 상대 하위 경로를 단일 오라클 루트 `pdf/` 에서 찾는다.
`oracle_resolver.py` 가 없어도 이 도구가 같은 규칙으로 직접 맞춘다.
짝 없는 개수는 아래 표의 실측값이다.

- 클레임: `M01-3`
- 생성기: `tools/oracle_public/coverage_report.py`
- 매칭: `stem-{year}.pdf`
- 오라클 루트: `pdf/`

## 요약

| 항목 | 수 |
| --- | ---: |
| 샘플 (`.hwp`/`.hwpx`) | 979 |
| 짝 있는 샘플 | 565 |
| 짝 없는 샘플 | 414 |
| 오라클 링크 (샘플×PDF) | 1030 |
| 커버리지 (짝 있는 샘플 / 전체) | 57.7% |

### 형식별

| 형식 | 샘플 | 짝 있음 | 짝 없음 | 커버리지 |
| --- | ---: | ---: | ---: | ---: |
| `hwp` | 537 | 350 | 187 | 65.2% |
| `hwpx` | 442 | 215 | 227 | 48.6% |

## 한글 버전별 (2018 / 2020 / 2022 / 2024)

한 샘플이 여러 버전 PDF 를 가지면 각 버전에 모두 센다.
분모는 전체 샘플 수다. 2010·2014 접미 PDF 는 이 표에 넣지 않는다.

| 한글 버전 | 링크 수 | 해당 버전이 있는 샘플 | 샘플 대비 |
| --- | ---: | ---: | ---: |
| 2018 | 0 | 0 | 0.0% |
| 2020 | 539 | 491 | 50.2% |
| 2022 | 364 | 364 | 37.2% |
| 2024 | 127 | 84 | 8.6% |

## 짝 없는 샘플 (414건)

| # | 경로 | 형식 | 이유 |
| ---: | --- | --- | --- |
| 1 | `samples/143E433F503322BD33.hwp` | hwp | no_oracle_pdf |
| 2 | `samples/156457624_210622 7월부터 해외직구 구매대행업체 등록제 시행.hwp` | hwp | no_oracle_pdf |
| 3 | `samples/156636617_240617 2024년 5월 월간 수출입 현황(확정치).hwp` | hwp | no_oracle_pdf |
| 4 | `samples/20250130-hongbo-no.hwp` | hwp | no_oracle_pdf |
| 5 | `samples/20250130-hongbo_saved.hwp` | hwp | no_oracle_pdf |
| 6 | `samples/2026_oss_rst.hwp` | hwp | no_oracle_pdf |
| 7 | `samples/21868765_별표2_보건소_분장사무.hwp` | hwp | no_oracle_pdf |
| 8 | `samples/253E164F57A1BC6934-empty.hwp` | hwp | no_oracle_pdf |
| 9 | `samples/3-09월_교육_통합_2024-격자기준종이.hwp` | hwp | no_oracle_pdf |
| 10 | `samples/3-09월_교육_통합_2024-격자기준쪽.hwp` | hwp | no_oracle_pdf |
| 11 | `samples/76076_regulatory_analysis.hwp` | hwp | no_oracle_pdf |
| 12 | `samples/80250_regulatory_analysis.hwp` | hwp | no_oracle_pdf |
| 13 | `samples/basic/calendar_monthly.hwp` | hwp | no_oracle_pdf |
| 14 | `samples/basic/issue1994_behindtext_table_20200830.hwp` | hwp | no_oracle_pdf |
| 15 | `samples/basic/pau-004.hwp` | hwp | no_oracle_pdf |
| 16 | `samples/byeolpyo1.hwp` | hwp | no_oracle_pdf |
| 17 | `samples/byeolpyo4.hwp` | hwp | no_oracle_pdf |
| 18 | `samples/el-school-001.hwp` | hwp | no_oracle_pdf |
| 19 | `samples/eq-002.hwp` | hwp | no_oracle_pdf |
| 20 | `samples/exam-kor-1p.hwp` | hwp | no_oracle_pdf |
| 21 | `samples/exam-kor-2p.hwp` | hwp | no_oracle_pdf |
| 22 | `samples/exam-kor-3p.hwp` | hwp | no_oracle_pdf |
| 23 | `samples/exam-kor-4p.hwp` | hwp | no_oracle_pdf |
| 24 | `samples/exam_social-p1.hwp` | hwp | no_oracle_pdf |
| 25 | `samples/footnote-tbox-01.hwp` | hwp | no_oracle_pdf |
| 26 | `samples/form-02.hwp` | hwp | no_oracle_pdf |
| 27 | `samples/hcar-001.hwp` | hwp | no_oracle_pdf |
| 28 | `samples/honbo-save.hwp` | hwp | no_oracle_pdf |
| 29 | `samples/hwp-3.0-HWPML.hwp` | hwp | no_oracle_pdf |
| 30 | `samples/hwp3-curve.hwp` | hwp | no_oracle_pdf |
| 31 | `samples/hwp3-ellipse-empty-textbox.hwp` | hwp | no_oracle_pdf |
| 32 | `samples/hwp3-empty-cell.hwp` | hwp | no_oracle_pdf |
| 33 | `samples/hwp3-pagedef-1915.hwp` | hwp | no_oracle_pdf |
| 34 | `samples/HWP3-password-123456.hwp` | hwp | no_oracle_pdf |
| 35 | `samples/hwp3-sample10-hwp5.hwp` | hwp | no_oracle_pdf |
| 36 | `samples/hwp3-sample10-hwpx.hwpx` | hwpx | no_oracle_pdf |
| 37 | `samples/hwp3-sample10.hwp` | hwp | no_oracle_pdf |
| 38 | `samples/hwp3-sample11-hwp5.hwp` | hwp | no_oracle_pdf |
| 39 | `samples/hwp3-sample13.hwp` | hwp | no_oracle_pdf |
| 40 | `samples/hwp3-sample16-hwp5-2024-password-123456.hwp` | hwp | no_oracle_pdf |
| 41 | `samples/hwp3-sample19-hwpx.hwpx` | hwpx | no_oracle_pdf |
| 42 | `samples/hwp3-sample19.hwp` | hwp | no_oracle_pdf |
| 43 | `samples/hwp3-sample5-hwp5-v2018.hwp` | hwp | no_oracle_pdf |
| 44 | `samples/hwp3-sample5-hwp5-v2024.hwp` | hwp | no_oracle_pdf |
| 45 | `samples/hwp3-table-caption.hwp` | hwp | no_oracle_pdf |
| 46 | `samples/hwp3-table-cell-order.hwp` | hwp | no_oracle_pdf |
| 47 | `samples/hwp3-table-cell-overlap.hwp` | hwp | no_oracle_pdf |
| 48 | `samples/hwp3-table-grid-gap.hwp` | hwp | no_oracle_pdf |
| 49 | `samples/HWP5-nopassword-123456.hwp` | hwp | no_oracle_pdf |
| 50 | `samples/HWP5-nopassword-123456.hwpx` | hwpx | no_oracle_pdf |
| 51 | `samples/HWP5-password-123456.hwpx` | hwpx | no_oracle_pdf |
| 52 | `samples/hwp5-tbl-attr-1916.hwp` | hwp | no_oracle_pdf |
| 53 | `samples/hwp_table_test_saved.hwp` | hwp | no_oracle_pdf |
| 54 | `samples/hwpers_test4_complex_table.hwp` | hwp | no_oracle_pdf |
| 55 | `samples/hwpx/143E433F503322BD33.hwpx` | hwpx | no_oracle_pdf |
| 56 | `samples/hwpx/2026_oss_rst.hwpx` | hwpx | no_oracle_pdf |
| 57 | `samples/hwpx/basic-table-01.hwpx` | hwpx | no_oracle_pdf |
| 58 | `samples/hwpx/business_overview.hwpx` | hwpx | no_oracle_pdf |
| 59 | `samples/hwpx/el-school-001.hwpx` | hwpx | no_oracle_pdf |
| 60 | `samples/hwpx/eq-002.hwpx` | hwpx | no_oracle_pdf |
| 61 | `samples/hwpx/exam-kor-1p.hwpx` | hwpx | no_oracle_pdf |
| 62 | `samples/hwpx/exam-kor-2p.hwpx` | hwpx | no_oracle_pdf |
| 63 | `samples/hwpx/exam-kor-3p.hwpx` | hwpx | no_oracle_pdf |
| 64 | `samples/hwpx/exam-kor-4p.hwpx` | hwpx | no_oracle_pdf |
| 65 | `samples/hwpx/exam_social-p1.hwpx` | hwpx | no_oracle_pdf |
| 66 | `samples/hwpx/expense_report.hwpx` | hwpx | no_oracle_pdf |
| 67 | `samples/hwpx/field-multipara-clickhere.hwpx` | hwpx | no_oracle_pdf |
| 68 | `samples/hwpx/footnote-tbox-01.hwpx` | hwpx | no_oracle_pdf |
| 69 | `samples/hwpx/form-01.hwpx` | hwpx | no_oracle_pdf |
| 70 | `samples/hwpx/form-02.hwpx` | hwpx | no_oracle_pdf |
| 71 | `samples/hwpx/hancom-hwp/activity_report.hwp` | hwp | no_oracle_pdf |
| 72 | `samples/hwpx/hancom-hwp/business_overview.hwp` | hwp | no_oracle_pdf |
| 73 | `samples/hwpx/hancom-hwp/expense_report.hwp` | hwp | no_oracle_pdf |
| 74 | `samples/hwpx/hancom-hwp/hang_job_01.hwp` | hwp | no_oracle_pdf |
| 75 | `samples/hwpx/hancom-hwp/hy-001.hwp` | hwp | no_oracle_pdf |
| 76 | `samples/hwpx/hancom-hwp/hy-002.hwp` | hwp | no_oracle_pdf |
| 77 | `samples/hwpx/hancom-hwp/service_agreement.hwp` | hwp | no_oracle_pdf |
| 78 | `samples/hwpx/hancom-hwp/tb-org-02.hwp` | hwp | no_oracle_pdf |
| 79 | `samples/hwpx/hcar-001.hwpx` | hwpx | no_oracle_pdf |
| 80 | `samples/hwpx/hwpx-centered-cell-vpos-after-tac-shape.hwpx` | hwpx | no_oracle_pdf |
| 81 | `samples/hwpx/hy-001.hwpx` | hwpx | no_oracle_pdf |
| 82 | `samples/hwpx/hy-002.hwpx` | hwpx | no_oracle_pdf |
| 83 | `samples/hwpx/issue1535_coanchored_float_exclusion.hwpx` | hwpx | no_oracle_pdf |
| 84 | `samples/hwpx/issue2439_page_local_float_exclusion.hwpx` | hwpx | no_oracle_pdf |
| 85 | `samples/hwpx/issue_1133.hwpx` | hwpx | no_oracle_pdf |
| 86 | `samples/hwpx/landscape-001.hwpx` | hwpx | no_oracle_pdf |
| 87 | `samples/hwpx/local-font-nanumsquare-bold.hwpx` | hwpx | no_oracle_pdf |
| 88 | `samples/hwpx/math-001.hwpx` | hwpx | no_oracle_pdf |
| 89 | `samples/hwpx/opengov/36382399_결재문서본문_일반지출결의서_기간제 소블록 세척 작업용품 구매.hwpx` | hwpx | no_oracle_pdf |
| 90 | `samples/hwpx/opengov/36382669_결재문서본문_’26년 제3차 강사선정 심의회 운영결과.hwpx` | hwpx | no_oracle_pdf |
| 91 | `samples/hwpx/opengov/36383351_결재문서본문_[관악산] 산악구조대 구급의약품 폐기 계획 보고.hwpx` | hwpx | no_oracle_pdf |
| 92 | `samples/hwpx/opengov/36383351_결재문서본문_관악산산악구조대급식의약품폐기.hwpx` | hwpx | no_oracle_pdf |
| 93 | `samples/hwpx/opengov/36384285_결재문서본문_[여의도] 2026년 6월 기본업무수행 급식비 지급.hwpx` | hwpx | no_oracle_pdf |
| 94 | `samples/hwpx/opengov/36385226_결재문서본문_제2처리장 슬러지인발용 에어리프트 브로워 2호기 소모품 교체 보고.hwpx` | hwpx | no_oracle_pdf |
| 95 | `samples/hwpx/opengov/36385464_결재문서본문_근무변경(반포수난구조대, 2026-06).hwpx` | hwpx | no_oracle_pdf |
| 96 | `samples/hwpx/opengov/36386761_백제학연구총서위탁판매의뢰목록.hwpx` | hwpx | no_oracle_pdf |
| 97 | `samples/hwpx/opengov/36387103_결재문서본문_수질검사 신청 민원 처리 결과 안내.hwpx` | hwpx | no_oracle_pdf |
| 98 | `samples/hwpx/opengov/36387725_footer_page_bottom.hwpx` | hwpx | no_oracle_pdf |
| 99 | `samples/hwpx/opengov/36388571_결재문서본문_2026년 6월 생일 축하품 수령증 제출(119항공대).hwpx` | hwpx | no_oracle_pdf |
| 100 | `samples/hwpx/opengov/36388853_결재문서본문_일상감사 의견서 송부.hwpx` | hwpx | no_oracle_pdf |
| 101 | `samples/hwpx/opengov/36389298_결재문서본문_물품검사(수)조서(라벨테이프 등).hwpx` | hwpx | no_oracle_pdf |
| 102 | `samples/hwpx/opengov/36389301_결재문서본문_직장훈련계획_덧말.hwpx` | hwpx | no_oracle_pdf |
| 103 | `samples/hwpx/opengov/36392900_결재문서본문_일일굴착복구공사현황보고.hwpx` | hwpx | no_oracle_pdf |
| 104 | `samples/hwpx/opengov/36395270_footer_overpush.hwpx` | hwpx | no_oracle_pdf |
| 105 | `samples/hwpx/pagenation-001.hwpx` | hwpx | no_oracle_pdf |
| 106 | `samples/hwpx/para-001.hwpx` | hwpx | no_oracle_pdf |
| 107 | `samples/hwpx/para-unit-01.hwpx` | hwpx | no_oracle_pdf |
| 108 | `samples/hwpx/shape-001.hwpx` | hwpx | no_oracle_pdf |
| 109 | `samples/hwpx/ta-pic-001-r.hwpx` | hwpx | no_oracle_pdf |
| 110 | `samples/hwpx/tb-img-03.hwpx` | hwpx | no_oracle_pdf |
| 111 | `samples/hwpx/tb-org-02.hwpx` | hwpx | no_oracle_pdf |
| 112 | `samples/hwpx/water-mark.hwpx` | hwpx | no_oracle_pdf |
| 113 | `samples/issue-6271-rowbreak-float-tail-line.hwp` | hwp | no_oracle_pdf |
| 114 | `samples/issue-986-receipt.hwp` | hwp | no_oracle_pdf |
| 115 | `samples/issue1549_empty_host_float_clamp.hwpx` | hwpx | no_oracle_pdf |
| 116 | `samples/issue1549_multipositive_float_tables.hwpx` | hwpx | no_oracle_pdf |
| 117 | `samples/issue1639_empty_host_negative_offset_float.hwpx` | hwpx | no_oracle_pdf |
| 118 | `samples/issue1639_empty_host_positive_only_float.hwpx` | hwpx | no_oracle_pdf |
| 119 | `samples/issue1842_cell_tac_group_lineheight.hwp` | hwp | no_oracle_pdf |
| 120 | `samples/issue1858_paper_anchor_float_stack.hwpx` | hwpx | no_oracle_pdf |
| 121 | `samples/issue1880_anchor_stack_sb_convert.hwpx` | hwpx | no_oracle_pdf |
| 122 | `samples/issue1880_takeplace_host_before.hwpx` | hwpx | no_oracle_pdf |
| 123 | `samples/issue1880_takeplace_oracle_p13.hwpx` | hwpx | no_oracle_pdf |
| 124 | `samples/issue1891/76076_regulatory_analysis.hwpx` | hwpx | no_oracle_pdf |
| 125 | `samples/issue1891/80250_regulatory_analysis.hwpx` | hwpx | no_oracle_pdf |
| 126 | `samples/issue1891_external_bindata_link.hwpx` | hwpx | no_oracle_pdf |
| 127 | `samples/issue1892_hwp3_drawing_group_roundtrip.hwp` | hwp | no_oracle_pdf |
| 128 | `samples/issue1892_hwp3_tab_roundtrip.hwp` | hwp | no_oracle_pdf |
| 129 | `samples/issue1937_rowbreak_footnote_overpagination.hwp` | hwp | no_oracle_pdf |
| 130 | `samples/issue2439/issue2439_repeat_table_overlap.hwp` | hwp | no_oracle_pdf |
| 131 | `samples/issue2439_zero_offset_coanchored_float_exclusion.hwp` | hwp | no_oracle_pdf |
| 132 | `samples/issue2527_empty_linesegs.hwpx` | hwpx | no_oracle_pdf |
| 133 | `samples/issue3738/tac_sibling_shape_line_advance.hwpx` | hwpx | no_oracle_pdf |
| 134 | `samples/issue3751/vpos_reset_midparagraph_fit.hwpx` | hwpx | no_oracle_pdf |
| 135 | `samples/issue3798/page_end_trailing_spill.hwpx` | hwpx | no_oracle_pdf |
| 136 | `samples/issue3834/flow_with_text_zero.hwpx` | hwpx | no_oracle_pdf |
| 137 | `samples/issue4090/156492236_규제샌드박스_min.hwpx` | hwpx | no_oracle_pdf |
| 138 | `samples/issue4490/148720174_111014(인력기획과)민간경력자_5급_일괄채용_필기_합격자_발표.hwp` | hwp | no_oracle_pdf |
| 139 | `samples/issue4491/30213_1.혼합단지등 제도개선 방안.hwp` | hwp | no_oracle_pdf |
| 140 | `samples/issue4599/36374873_night_guard_log.hwpx` | hwpx | no_oracle_pdf |
| 141 | `samples/issue4657/distribute-alignment-sample.hwpx` | hwpx | no_oracle_pdf |
| 142 | `samples/issue4690/30098_indent_over_stored_cs.hwp` | hwp | no_oracle_pdf |
| 143 | `samples/issue5162_field_wraps_table.hwpx` | hwpx | no_oracle_pdf |
| 144 | `samples/issue5169_viewtext_changetracking.hwp` | hwp | no_oracle_pdf |
| 145 | `samples/issue5524_hangul2024_compat_letterhead.hwp` | hwp | no_oracle_pdf |
| 146 | `samples/issue5543_carried_anchor_ladder.hwpx` | hwpx | no_oracle_pdf |
| 147 | `samples/issue5584/3232693_employment_support_criteria.hwpx` | hwpx | no_oracle_pdf |
| 148 | `samples/issue5584/float_host_title_above_table.hwpx` | hwpx | no_oracle_pdf |
| 149 | `samples/issue5590_per_row_column_widths.hwpx` | hwpx | no_oracle_pdf |
| 150 | `samples/issue5593_cell_center_front_object.hwpx` | hwpx | no_oracle_pdf |
| 151 | `samples/issue5595_rotated_picture_topbottom.hwpx` | hwpx | no_oracle_pdf |
| 152 | `samples/issue5599_oracle/3191107_leave_request_form.hwpx` | hwpx | no_oracle_pdf |
| 153 | `samples/issue5601/2537593_supply_agreement_form.hwp` | hwp | no_oracle_pdf |
| 154 | `samples/issue5637/2817919_emfplus_ole_preview.hwpx` | hwpx | no_oracle_pdf |
| 155 | `samples/issue5679/10857_delegation_rules.hwp` | hwp | no_oracle_pdf |
| 156 | `samples/issue5699/16758113_pruning_forms.hwp` | hwp | no_oracle_pdf |
| 157 | `samples/issue5699/20099369_yeongwol_forms.hwp` | hwp | no_oracle_pdf |
| 158 | `samples/issue5699/37787_regulatory_impact.hwp` | hwp | no_oracle_pdf |
| 159 | `samples/issue5701/1270000-202200012_slice_p76_rewound_host.hwp` | hwp | no_oracle_pdf |
| 160 | `samples/issue5701/1530000-200800002_slice_p139_tac_reset_tail.hwp` | hwp | no_oracle_pdf |
| 161 | `samples/issue5712/3184241_medical_exam_equipment.hwpx` | hwpx | no_oracle_pdf |
| 162 | `samples/issue5714/1490000-200800034_vietnam_labor_report.hwp` | hwp | no_oracle_pdf |
| 163 | `samples/issue5715/float_chart_ghost_ladder_gap.hwp` | hwp | no_oracle_pdf |
| 164 | `samples/issue5720/2734559_mixed_column_grid.hwpx` | hwpx | no_oracle_pdf |
| 165 | `samples/issue5721/2568129_textbox_float_tables.hwp` | hwp | no_oracle_pdf |
| 166 | `samples/issue5723/coanchored_square_pair_center_slack.hwpx` | hwpx | no_oracle_pdf |
| 167 | `samples/issue5724/2689441_wmf_contents_ole.hwp` | hwp | no_oracle_pdf |
| 168 | `samples/issue5725/2921145_equation_ole.hwpx` | hwpx | no_oracle_pdf |
| 169 | `samples/issue5727/156732636_inline_logo_cell.hwp` | hwp | no_oracle_pdf |
| 170 | `samples/issue5729/stacked_tac_band_om_top.hwpx` | hwpx | no_oracle_pdf |
| 171 | `samples/issue5730/underline_probe.hwp` | hwp | no_oracle_pdf |
| 172 | `samples/issue5731/cell_second_float_flow_anchor.hwpx` | hwpx | no_oracle_pdf |
| 173 | `samples/issue5734/cell_float_stack_stored_vpos.hwpx` | hwpx | no_oracle_pdf |
| 174 | `samples/issue5747/mismatched_manifest_refs.hwpx` | hwpx | no_oracle_pdf |
| 175 | `samples/issue5748/tac_shrink_row_floor.hwpx` | hwpx | no_oracle_pdf |
| 176 | `samples/issue5755/rewind_overflow_page_break.hwpx` | hwpx | no_oracle_pdf |
| 177 | `samples/issue5756/156732409_superscript_advance.hwp` | hwp | no_oracle_pdf |
| 178 | `samples/issue5757/nontac_declared_height_shrink.hwpx` | hwpx | no_oracle_pdf |
| 179 | `samples/issue5780/flow_image_page_background.hwpx` | hwpx | no_oracle_pdf |
| 180 | `samples/issue5782/2181727_press_guard_test_methods.hwp` | hwp | no_oracle_pdf |
| 181 | `samples/issue5785/medal_cells_ws_host_inline.hwpx` | hwpx | no_oracle_pdf |
| 182 | `samples/issue5789/tac_line_shape_baseline.hwpx` | hwpx | no_oracle_pdf |
| 183 | `samples/issue5793/pua_f0827_double_rule.hwp` | hwp | no_oracle_pdf |
| 184 | `samples/issue5797_shape_selfclosing_child.hwpx` | hwpx | no_oracle_pdf |
| 185 | `samples/issue5800-hancom-symbol.hwp` | hwp | no_oracle_pdf |
| 186 | `samples/issue5802/hf_cross_section_inherit.hwp` | hwp | no_oracle_pdf |
| 187 | `samples/issue5808/square_group_left_outer_margin.hwpx` | hwpx | no_oracle_pdf |
| 188 | `samples/issue5820/156560092_ecard_meeting_press.hwpx` | hwpx | no_oracle_pdf |
| 189 | `samples/issue5822/tac_chart_stored_frame_fit.hwpx` | hwpx | no_oracle_pdf |
| 190 | `samples/issue5825/shrunk_row_degenerate_baseline.hwpx` | hwpx | no_oracle_pdf |
| 191 | `samples/issue5828/landscape_rowbreak_bleed.hwp` | hwp | no_oracle_pdf |
| 192 | `samples/issue5833/cell_multi_para_float_pics.hwp` | hwp | no_oracle_pdf |
| 193 | `samples/issue5847/x2x_lineseg_vertpos_cumulative.hwpx` | hwpx | no_oracle_pdf |
| 194 | `samples/issue5866/memo_field_hwp5.hwp` | hwp | no_oracle_pdf |
| 195 | `samples/issue5870/empty_host_float_flow_advance.hwp` | hwp | no_oracle_pdf |
| 196 | `samples/issue5871/ws_host_float_double_charge.hwp` | hwp | no_oracle_pdf |
| 197 | `samples/issue5872/toc_midline_right_tab.hwpx` | hwpx | no_oracle_pdf |
| 198 | `samples/issue5875/nested_table_text_caption.hwp` | hwp | no_oracle_pdf |
| 199 | `samples/issue5877/fragment_ghost_vrules.hwp` | hwp | no_oracle_pdf |
| 200 | `samples/issue5885/3171199_design_capability_criteria.hwp` | hwp | no_oracle_pdf |
| 201 | `samples/issue5929/table_below_square_pic.hwpx` | hwpx | no_oracle_pdf |
| 202 | `samples/issue5941/1130000-200900012_anchor_delay_ulp.hwp` | hwp | no_oracle_pdf |
| 203 | `samples/issue5941/1490000-201600081_roadmap_research.hwp` | hwp | no_oracle_pdf |
| 204 | `samples/issue5941/3240179_efficiency_test_orgs.hwpx` | hwpx | no_oracle_pdf |
| 205 | `samples/issue5966/1130000-202100008_franchise_review_report.hwp` | hwp | no_oracle_pdf |
| 206 | `samples/issue6023/30269_reform_recommendation.hwp` | hwp | no_oracle_pdf |
| 207 | `samples/issue6025/3232693_employment_support_criteria.hwpx` | hwpx | no_oracle_pdf |
| 208 | `samples/issue6028/2307287_construction_machinery_spec.hwp` | hwp | no_oracle_pdf |
| 209 | `samples/issue6029/3200477_icao_procedure.hwpx` | hwpx | no_oracle_pdf |
| 210 | `samples/issue6030/2386771_agritech_review_form.hwp` | hwp | no_oracle_pdf |
| 211 | `samples/issue6031/3249937_asset_management_rules.hwpx` | hwpx | no_oracle_pdf |
| 212 | `samples/issue6032/2912695_civil_petition_form.hwp` | hwp | no_oracle_pdf |
| 213 | `samples/issue6034/2912735_court_report_form.hwp` | hwp | no_oracle_pdf |
| 214 | `samples/issue6035/2804253_cosmetics_gmp_checklist.hwpx` | hwpx | no_oracle_pdf |
| 215 | `samples/issue6035/cgmp_evaluation_table.hwpx` | hwpx | no_oracle_pdf |
| 216 | `samples/issue6036/156509073_police_press_release.hwpx` | hwpx | no_oracle_pdf |
| 217 | `samples/issue6044/156513948.hwpx` | hwpx | no_oracle_pdf |
| 218 | `samples/issue6057/29494.hwp` | hwp | no_oracle_pdf |
| 219 | `samples/issue6060/30307_local_service_reform.hwp` | hwp | no_oracle_pdf |
| 220 | `samples/issue6086/30098_resident_registration_reform.hwp` | hwp | no_oracle_pdf |
| 221 | `samples/issue6095/3090867_icepack_levy_criteria.hwpx` | hwpx | no_oracle_pdf |
| 222 | `samples/issue6099/2197981_scanned_form.hwp` | hwp | no_oracle_pdf |
| 223 | `samples/issue6101/36361137_firefighter_training_plan.hwpx` | hwpx | no_oracle_pdf |
| 224 | `samples/issue6101/36501883_approval_doc_body.hwpx` | hwpx | no_oracle_pdf |
| 225 | `samples/issue6102/36310257_overtime_report.hwpx` | hwpx | no_oracle_pdf |
| 226 | `samples/issue6102/36360328_vehicle_inspection_expense.hwpx` | hwpx | no_oracle_pdf |
| 227 | `samples/issue6102/36444579_traffic_fine_exemption.hwpx` | hwpx | no_oracle_pdf |
| 228 | `samples/issue6110/39819_press_release_header_slice.hwp` | hwp | no_oracle_pdf |
| 229 | `samples/issue6111/56345_regulatory_impact_analysis.hwp` | hwp | no_oracle_pdf |
| 230 | `samples/issue6117/52690_higher_education_decree.hwp` | hwp | no_oracle_pdf |
| 231 | `samples/issue6121/156531618_police_press_header.hwpx` | hwpx | no_oracle_pdf |
| 232 | `samples/issue6122/2181727_press_guard_test_method.hwp` | hwp | no_oracle_pdf |
| 233 | `samples/issue6123/3112461_railway_emc_criteria.hwpx` | hwpx | no_oracle_pdf |
| 234 | `samples/issue6124/2737927_housing_evaluation_guideline.hwpx` | hwpx | no_oracle_pdf |
| 235 | `samples/issue6126/3171199_design_capability_criteria.hwp` | hwp | no_oracle_pdf |
| 236 | `samples/issue6127/2599643_vessel_pass_application.hwp` | hwp | no_oracle_pdf |
| 237 | `samples/issue6128/156653004_privacy_day_ceremony.hwpx` | hwpx | no_oracle_pdf |
| 238 | `samples/issue6132/156482639_startup_ir_contest.hwp` | hwp | no_oracle_pdf |
| 239 | `samples/issue6133/156483831_poster_title_above_offset_float.hwp` | hwp | no_oracle_pdf |
| 240 | `samples/issue6134/156731730_contact_table_logo_overlay.hwpx` | hwpx | no_oracle_pdf |
| 241 | `samples/issue6135/156544683_title_row_underfit.hwp` | hwp | no_oracle_pdf |
| 242 | `samples/issue6140/156462405_smart_expo.hwp` | hwp | no_oracle_pdf |
| 243 | `samples/issue6143/156555538_securities_settlement_review.hwpx` | hwpx | no_oracle_pdf |
| 244 | `samples/issue6145/worklife_balance_index_156607916.hwpx` | hwpx | no_oracle_pdf |
| 245 | `samples/issue6146/156583583_press_release_logo_band.hwpx` | hwpx | no_oracle_pdf |
| 246 | `samples/issue6147/156741101_press_release_band.hwpx` | hwpx | no_oracle_pdf |
| 247 | `samples/issue6167/leading_space_tac_table.hwpx` | hwpx | no_oracle_pdf |
| 248 | `samples/issue6172/2599643_port_call_form.hwp` | hwp | no_oracle_pdf |
| 249 | `samples/issue6173/textbox_right_align_logos.hwpx` | hwpx | no_oracle_pdf |
| 250 | `samples/issue6174/156661338_police_press_release.hwpx` | hwpx | no_oracle_pdf |
| 251 | `samples/issue6175/seed_expo_square_float_body.hwpx` | hwpx | no_oracle_pdf |
| 252 | `samples/issue6179/right_tab_footer_logo.hwpx` | hwpx | no_oracle_pdf |
| 253 | `samples/issue6180/156745974_tac_object_line_spacing.hwpx` | hwpx | no_oracle_pdf |
| 254 | `samples/issue6181/156562368_inline_tac_table_line_advance.hwpx` | hwpx | no_oracle_pdf |
| 255 | `samples/issue6184/156489124_tail_line_before_deferred_table.hwp` | hwp | no_oracle_pdf |
| 256 | `samples/issue6185/156570535_logo_box_self_displacement.hwpx` | hwpx | no_oracle_pdf |
| 257 | `samples/issue6186/156755659_defense_press_release.hwpx` | hwpx | no_oracle_pdf |
| 258 | `samples/issue6186/156755659_footer_vertalign_bottom.hwpx` | hwpx | no_oracle_pdf |
| 259 | `samples/issue6190/center_align_first_line_indent.hwp` | hwp | no_oracle_pdf |
| 260 | `samples/issue6192/cell_behind_text_para_anchor.hwpx` | hwpx | no_oracle_pdf |
| 261 | `samples/issue6194/156494392_agri_press_release.hwpx` | hwpx | no_oracle_pdf |
| 262 | `samples/issue6196/cell_char_spacing_fit.hwp` | hwp | no_oracle_pdf |
| 263 | `samples/issue6204/square_picture_band_host.hwp` | hwp | no_oracle_pdf |
| 264 | `samples/issue6208/print_method_nup.hwp` | hwp | no_oracle_pdf |
| 265 | `samples/issue6264/1977964_env_satellite_report_form.hwp` | hwp | no_oracle_pdf |
| 266 | `samples/issue6266/seizure_list_form_button.hwp` | hwp | no_oracle_pdf |
| 267 | `samples/issue6267/kdt_result_para_float_table.hwpx` | hwpx | no_oracle_pdf |
| 268 | `samples/issue6269/156739836_public_sector_jobs_stats.hwpx` | hwpx | no_oracle_pdf |
| 269 | `samples/issue6280/156742029_prosecutor_transfer_list.hwp` | hwp | no_oracle_pdf |
| 270 | `samples/issue6284/child_policy_top_caption_charts.hwpx` | hwpx | no_oracle_pdf |
| 271 | `samples/issue6298/copay_cap_tac_table_leading.hwpx` | hwpx | no_oracle_pdf |
| 272 | `samples/issue6299/forest_press_wrap_seg_pairs.hwpx` | hwpx | no_oracle_pdf |
| 273 | `samples/issue6300/trade_report_forced_break_object.hwp` | hwp | no_oracle_pdf |
| 274 | `samples/issue6310/press_release_cell_logo.hwpx` | hwpx | no_oracle_pdf |
| 275 | `samples/issue6312/fiscal_trend_float_table_anchor.hwpx` | hwpx | no_oracle_pdf |
| 276 | `samples/issue6313/microbe_bank_cell_picture.hwpx` | hwpx | no_oracle_pdf |
| 277 | `samples/issue6442/access_pass_form.hwp` | hwp | no_oracle_pdf |
| 278 | `samples/issue6443/research_project_design_form.hwpx` | hwpx | no_oracle_pdf |
| 279 | `samples/issue6448/tac_cell_leftover_fits.hwpx` | hwpx | no_oracle_pdf |
| 280 | `samples/issue6451/underline_run_fragments.hwpx` | hwpx | no_oracle_pdf |
| 281 | `samples/issue6465/press_release_footer_logos.hwpx` | hwpx | no_oracle_pdf |
| 282 | `samples/issue6469/wmf_fill_shapes.hwpx` | hwpx | no_oracle_pdf |
| 283 | `samples/issue6524/30098_float_host_split_lineseg.hwp` | hwp | no_oracle_pdf |
| 284 | `samples/issue6535/36339092_low_slack_absorb_block.hwpx` | hwpx | no_oracle_pdf |
| 285 | `samples/issue6535/36399617_page_anchored_block_reset.hwpx` | hwpx | no_oracle_pdf |
| 286 | `samples/issue6535/36404612_page_anchored_footer_block.hwpx` | hwpx | no_oracle_pdf |
| 287 | `samples/issue6542/156678235_mid_para_vpos_rewind.hwp` | hwp | no_oracle_pdf |
| 288 | `samples/issue6549/16418295_square_rowbreak_table.hwp` | hwp | no_oracle_pdf |
| 289 | `samples/issue6551/113424_evaluation_guideline.hwpx` | hwpx | no_oracle_pdf |
| 290 | `samples/issue6575/156489219_satellite_pm_release.hwp` | hwp | no_oracle_pdf |
| 291 | `samples/issue6575/tac_picture_line_top.hwpx` | hwpx | no_oracle_pdf |
| 292 | `samples/issue6598/2744465_fingerprint_appraisal.hwp` | hwp | no_oracle_pdf |
| 293 | `samples/issue6601/36331407_side_by_side_tac_tables.hwpx` | hwpx | no_oracle_pdf |
| 294 | `samples/issue6619_cellzone_border.hwpx` | hwpx | no_oracle_pdf |
| 295 | `samples/issue_1133.hwp` | hwp | no_oracle_pdf |
| 296 | `samples/issue_2148_degenerate_cell_vpos.hwpx` | hwpx | no_oracle_pdf |
| 297 | `samples/issues/2809/jubo_20260104.hwp` | hwp | no_oracle_pdf |
| 298 | `samples/landscape-001.hwp` | hwp | no_oracle_pdf |
| 299 | `samples/math-001.hwp` | hwp | no_oracle_pdf |
| 300 | `samples/para-001.hwp` | hwp | no_oracle_pdf |
| 301 | `samples/para-unit-01.hwp` | hwp | no_oracle_pdf |
| 302 | `samples/pic-in-table-with-toggle.hwp` | hwp | no_oracle_pdf |
| 303 | `samples/pr4093/outline_navigation_panel_demo.hwpx` | hwpx | no_oracle_pdf |
| 304 | `samples/pr4093/outline_navigation_table_cell_number.hwpx` | hwpx | no_oracle_pdf |
| 305 | `samples/pr5935/test-2010.hwp` | hwp | no_oracle_pdf |
| 306 | `samples/pr5935/test-2018.hwp` | hwp | no_oracle_pdf |
| 307 | `samples/pr5935/test-2018.hwpx` | hwpx | no_oracle_pdf |
| 308 | `samples/pr5935/test-2022.hwp` | hwp | no_oracle_pdf |
| 309 | `samples/pr5935/test-2022.hwpx` | hwpx | no_oracle_pdf |
| 310 | `samples/pr5935/test-2024.hwp` | hwp | no_oracle_pdf |
| 311 | `samples/pr5935/test-2024.hwpx` | hwpx | no_oracle_pdf |
| 312 | `samples/render-p35-font-native-bitmap.hwpx` | hwpx | no_oracle_pdf |
| 313 | `samples/shape-001.hwp` | hwp | no_oracle_pdf |
| 314 | `samples/ta-pic-001-r.hwp` | hwp | no_oracle_pdf |
| 315 | `samples/ta-pic-cell-center-pos-bottom.hwpx` | hwpx | no_oracle_pdf |
| 316 | `samples/ta-pic-cell-top-pos-center.hwpx` | hwpx | no_oracle_pdf |
| 317 | `samples/tac-verify/scenario-a-after.hwp` | hwp | no_oracle_pdf |
| 318 | `samples/tac-verify/scenario-a-before.hwp` | hwp | no_oracle_pdf |
| 319 | `samples/tac-verify/scenario-b-after.hwp` | hwp | no_oracle_pdf |
| 320 | `samples/tac-verify/scenario-b-before.hwp` | hwp | no_oracle_pdf |
| 321 | `samples/tac-verify/scenario-c-after.hwp` | hwp | no_oracle_pdf |
| 322 | `samples/tac-verify/scenario-c-before.hwp` | hwp | no_oracle_pdf |
| 323 | `samples/tac-verify/scenario-d-after.hwp` | hwp | no_oracle_pdf |
| 324 | `samples/tac-verify/scenario-d-before.hwp` | hwp | no_oracle_pdf |
| 325 | `samples/task1700/byeolpyo1_uujeong_wrap_singlepage.hwp` | hwp | no_oracle_pdf |
| 326 | `samples/task1700/myeonjeok_wrap_10page.hwp` | hwp | no_oracle_pdf |
| 327 | `samples/task1705/wrap_empty_para_anchor_page.hwp` | hwp | no_oracle_pdf |
| 328 | `samples/task1706/empty_para_before_pagebreak.hwpx` | hwpx | no_oracle_pdf |
| 329 | `samples/task1706/empty_para_between_tac_tables.hwp` | hwp | no_oracle_pdf |
| 330 | `samples/task1749/saved_bounds_cumulative_page_break.hwp` | hwp | no_oracle_pdf |
| 331 | `samples/task1749/saved_bounds_cumulative_page_break.hwpx` | hwpx | no_oracle_pdf |
| 332 | `samples/task1749/saved_bounds_cumulative_vpos.hwp` | hwp | no_oracle_pdf |
| 333 | `samples/task1749/saved_bounds_cumulative_vpos.hwpx` | hwpx | no_oracle_pdf |
| 334 | `samples/task1750/split_guard_spacing_before.hwp` | hwp | no_oracle_pdf |
| 335 | `samples/task1750/split_guard_spacing_before.hwpx` | hwpx | no_oracle_pdf |
| 336 | `samples/task1753/deferred_takeplace_fill_ahead.hwp` | hwp | no_oracle_pdf |
| 337 | `samples/task1753/deferred_takeplace_fill_ahead.hwpx` | hwpx | no_oracle_pdf |
| 338 | `samples/task1763/cell_trailing_ls_expand.hwp` | hwp | no_oracle_pdf |
| 339 | `samples/task1763/cell_trailing_ls_expand.hwpx` | hwpx | no_oracle_pdf |
| 340 | `samples/task1765/merged_cell_trailing_ls.hwp` | hwp | no_oracle_pdf |
| 341 | `samples/task1765/merged_cell_trailing_ls.hwpx` | hwpx | no_oracle_pdf |
| 342 | `samples/task1768/distribution_doc.hwpx` | hwpx | no_oracle_pdf |
| 343 | `samples/task2070/hy_ladder3.hwpx` | hwpx | no_oracle_pdf |
| 344 | `samples/task2070/hy_ladder4.hwpx` | hwpx | no_oracle_pdf |
| 345 | `samples/task2097/17809123_jawonbongsa.hwpx` | hwpx | no_oracle_pdf |
| 346 | `samples/task2097/18095317_eogu_geumji.hwp` | hwp | no_oracle_pdf |
| 347 | `samples/task2097/21217935_simsa_jipyo.hwp` | hwp | no_oracle_pdf |
| 348 | `samples/task2097/3023771_wichokjang.hwpx` | hwpx | no_oracle_pdf |
| 349 | `samples/task2097/3248363_upmu_bunjang.hwpx` | hwpx | no_oracle_pdf |
| 350 | `samples/task2097/rowbreak_midpage_declared_fits.hwpx` | hwpx | no_oracle_pdf |
| 351 | `samples/task2105/rowbreak_table_declared_fits.hwpx` | hwpx | no_oracle_pdf |
| 352 | `samples/task2137/156637323_unification_lecture.hwpx` | hwpx | no_oracle_pdf |
| 353 | `samples/task2156/width_ladder.hwpx` | hwpx | no_oracle_pdf |
| 354 | `samples/task2169/anchor_ladder.hwpx` | hwpx | no_oracle_pdf |
| 355 | `samples/task2169/empty_ladder.hwpx` | hwpx | no_oracle_pdf |
| 356 | `samples/task2279/36353832_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 357 | `samples/task2279/36358528_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 358 | `samples/task2279/36365360_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 359 | `samples/task2279/36376848_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 360 | `samples/task2279/36378128_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 361 | `samples/task2279/36378481_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 362 | `samples/task2279/36394733_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 363 | `samples/task2279/36395825_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 364 | `samples/task2279/36398724_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 365 | `samples/task2279/36404953_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 366 | `samples/task2279/36406174_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 367 | `samples/task2279/36407074_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 368 | `samples/task2279/36410902_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 369 | `samples/task2279/36417406_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 370 | `samples/task2279/36423194_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 371 | `samples/task2279/36423558_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 372 | `samples/task2279/36425476_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 373 | `samples/task2279/36433160_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 374 | `samples/task2279/36434078_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 375 | `samples/task2279/36437313_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 376 | `samples/task2279/36446358_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 377 | `samples/task2279/36455850_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 378 | `samples/task2279/36456688_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 379 | `samples/task2279/36477251_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 380 | `samples/task2279/36477373_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 381 | `samples/task2311/156744475_nano_plan_poster.hwpx` | hwpx | no_oracle_pdf |
| 382 | `samples/task2319/20544835_jinan_apt_form.hwp` | hwp | no_oracle_pdf |
| 383 | `samples/task2322/19439117_gokseong_voucher_form.hwp` | hwp | no_oracle_pdf |
| 384 | `samples/task2322/20862337_cheongyang_voucher_form.hwp` | hwp | no_oracle_pdf |
| 385 | `samples/task2430/1382000_domestic_violence_survey.hwp` | hwp | no_oracle_pdf |
| 386 | `samples/task3307/issue3307_outline_number.hwpx` | hwpx | no_oracle_pdf |
| 387 | `samples/tb-img-03.hwp` | hwp | no_oracle_pdf |
| 388 | `samples/test-image.hwp` | hwp | no_oracle_pdf |
| 389 | `samples/test-image.hwpx` | hwpx | no_oracle_pdf |
| 390 | `samples/test-image2.hwp` | hwp | no_oracle_pdf |
| 391 | `samples/textbox-under-image.hwp` | hwp | no_oracle_pdf |
| 392 | `samples/unicode/각 항목에 명시되어 있는_유니코드.hwp` | hwp | no_oracle_pdf |
| 393 | `samples/valign_fixtures/cell_vbottom_nested_overcount.hwpx` | hwpx | no_oracle_pdf |
| 394 | `samples/valign_fixtures/cell_vcenter_multi_nested_overcount.hwpx` | hwpx | no_oracle_pdf |
| 395 | `samples/valign_fixtures/cell_vcenter_nested_undercount.hwpx` | hwpx | no_oracle_pdf |
| 396 | `samples/valign_fixtures/centered_cell_nested_table.hwpx` | hwpx | no_oracle_pdf |
| 397 | `samples/water-mark.hwp` | hwp | no_oracle_pdf |
| 398 | `samples/대각선샘플3.hwp` | hwp | no_oracle_pdf |
| 399 | `samples/대각선샘플3.hwpx` | hwpx | no_oracle_pdf |
| 400 | `samples/대각선샘플4.hwp` | hwp | no_oracle_pdf |
| 401 | `samples/대각선샘플4.hwpx` | hwpx | no_oracle_pdf |
| 402 | `samples/대각선샘플5.hwp` | hwp | no_oracle_pdf |
| 403 | `samples/대각선샘플5.hwpx` | hwpx | no_oracle_pdf |
| 404 | `samples/셀보호.hwp` | hwp | no_oracle_pdf |
| 405 | `samples/셀보호.hwpx` | hwpx | no_oracle_pdf |
| 406 | `samples/종이기준.hwp` | hwp | no_oracle_pdf |
| 407 | `samples/종이기준.hwpx` | hwpx | no_oracle_pdf |
| 408 | `samples/쪽기준.hwp` | hwp | no_oracle_pdf |
| 409 | `samples/쪽기준.hwpx` | hwpx | no_oracle_pdf |
| 410 | `samples/투명도0-50-2nd그림글차처럼off.hwp` | hwp | no_oracle_pdf |
| 411 | `samples/투명도0-50-2nd그림글차처럼off.hwpx` | hwpx | no_oracle_pdf |
| 412 | `samples/투명도0-50.hwp` | hwp | no_oracle_pdf |
| 413 | `samples/투명도0-50.hwpx` | hwpx | no_oracle_pdf |
| 414 | `samples/한글문서파일형식_5.0_revision1.3.hwp` | hwp | no_oracle_pdf |
