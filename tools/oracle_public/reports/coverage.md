# M01-3 오라클 커버리지

`samples/` 의 `.hwp`/`.hwpx` 와 `{stem}-{year}.pdf` (2018/2020/2022/2024) 짝.
같은 상대 하위 경로를 `pdf/` · `pdf-2020/` · `pdf-large/` 에서 찾는다.
`oracle_resolver.py` 가 없어도 이 도구가 같은 규칙으로 직접 맞춘다.
짝 없는 개수는 아래 표의 실측값이다.

- 클레임: `M01-3`
- 생성기: `tools/oracle_public/coverage_report.py`
- 매칭: `stem-{year}.pdf`
- 오라클 루트: `pdf/`, `pdf-2020/`, `pdf-large/`

## 요약

| 항목 | 수 |
| --- | ---: |
| 샘플 (`.hwp`/`.hwpx`) | 694 |
| 짝 있는 샘플 | 389 |
| 짝 없는 샘플 | 305 |
| 오라클 링크 (샘플×PDF) | 409 |
| 커버리지 (짝 있는 샘플 / 전체) | 56.1% |

### 형식별

| 형식 | 샘플 | 짝 있음 | 짝 없음 | 커버리지 |
| --- | ---: | ---: | ---: | ---: |
| `hwp` | 412 | 270 | 142 | 65.5% |
| `hwpx` | 282 | 119 | 163 | 42.2% |

## 한글 버전별 (2018 / 2020 / 2022 / 2024)

한 샘플이 여러 버전 PDF 를 가지면 각 버전에 모두 센다.
분모는 전체 샘플 수다. 2010·2014 접미 PDF 는 이 표에 넣지 않는다.

| 한글 버전 | 링크 수 | 해당 버전이 있는 샘플 | 샘플 대비 |
| --- | ---: | ---: | ---: |
| 2018 | 0 | 0 | 0.0% |
| 2020 | 56 | 53 | 7.6% |
| 2022 | 294 | 294 | 42.4% |
| 2024 | 59 | 57 | 8.2% |

## 짝 없는 샘플 (305건)

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
| 9 | `samples/3-09월_교육_통합_2023.hwp` | hwp | no_oracle_pdf |
| 10 | `samples/3-09월_교육_통합_2023.hwpx` | hwpx | no_oracle_pdf |
| 11 | `samples/3-09월_교육_통합_2024-격자기준종이.hwp` | hwp | no_oracle_pdf |
| 12 | `samples/3-09월_교육_통합_2024-격자기준쪽.hwp` | hwp | no_oracle_pdf |
| 13 | `samples/76076_regulatory_analysis.hwp` | hwp | no_oracle_pdf |
| 14 | `samples/80250_regulatory_analysis.hwp` | hwp | no_oracle_pdf |
| 15 | `samples/86712_regulatory_analysis.hwp` | hwp | no_oracle_pdf |
| 16 | `samples/basic/calendar_monthly.hwp` | hwp | no_oracle_pdf |
| 17 | `samples/basic/issue1994_behindtext_table_20200830.hwp` | hwp | no_oracle_pdf |
| 18 | `samples/basic/pau-004.hwp` | hwp | no_oracle_pdf |
| 19 | `samples/byeolpyo1.hwp` | hwp | no_oracle_pdf |
| 20 | `samples/byeolpyo4.hwp` | hwp | no_oracle_pdf |
| 21 | `samples/el-school-001.hwp` | hwp | no_oracle_pdf |
| 22 | `samples/eq-002.hwp` | hwp | no_oracle_pdf |
| 23 | `samples/exam-kor-1p.hwp` | hwp | no_oracle_pdf |
| 24 | `samples/exam-kor-2p.hwp` | hwp | no_oracle_pdf |
| 25 | `samples/exam-kor-3p.hwp` | hwp | no_oracle_pdf |
| 26 | `samples/exam-kor-4p.hwp` | hwp | no_oracle_pdf |
| 27 | `samples/exam_social-p1.hwp` | hwp | no_oracle_pdf |
| 28 | `samples/footnote-tbox-01.hwp` | hwp | no_oracle_pdf |
| 29 | `samples/form-02.hwp` | hwp | no_oracle_pdf |
| 30 | `samples/hcar-001.hwp` | hwp | no_oracle_pdf |
| 31 | `samples/honbo-save.hwp` | hwp | no_oracle_pdf |
| 32 | `samples/hwp-3.0-HWPML.hwp` | hwp | no_oracle_pdf |
| 33 | `samples/hwp3-curve.hwp` | hwp | no_oracle_pdf |
| 34 | `samples/hwp3-ellipse-empty-textbox.hwp` | hwp | no_oracle_pdf |
| 35 | `samples/hwp3-empty-cell.hwp` | hwp | no_oracle_pdf |
| 36 | `samples/hwp3-pagedef-1915.hwp` | hwp | no_oracle_pdf |
| 37 | `samples/HWP3-password-123456.hwp` | hwp | no_oracle_pdf |
| 38 | `samples/hwp3-sample10-hwp5.hwp` | hwp | no_oracle_pdf |
| 39 | `samples/hwp3-sample10-hwpx.hwpx` | hwpx | no_oracle_pdf |
| 40 | `samples/hwp3-sample10.hwp` | hwp | no_oracle_pdf |
| 41 | `samples/hwp3-sample11-hwp5.hwp` | hwp | no_oracle_pdf |
| 42 | `samples/hwp3-sample11-hwpx.hwpx` | hwpx | no_oracle_pdf |
| 43 | `samples/hwp3-sample13.hwp` | hwp | no_oracle_pdf |
| 44 | `samples/hwp3-sample14.hwp` | hwp | no_oracle_pdf |
| 45 | `samples/hwp3-sample16-hwp5-2024-password-123456.hwp` | hwp | no_oracle_pdf |
| 46 | `samples/hwp3-sample19-hwpx.hwpx` | hwpx | no_oracle_pdf |
| 47 | `samples/hwp3-sample19.hwp` | hwp | no_oracle_pdf |
| 48 | `samples/hwp3-sample5-hwp5-v2018.hwp` | hwp | no_oracle_pdf |
| 49 | `samples/hwp3-sample5-hwp5-v2024.hwp` | hwp | no_oracle_pdf |
| 50 | `samples/hwp3-table-caption.hwp` | hwp | no_oracle_pdf |
| 51 | `samples/hwp3-table-cell-order.hwp` | hwp | no_oracle_pdf |
| 52 | `samples/hwp3-table-cell-overlap.hwp` | hwp | no_oracle_pdf |
| 53 | `samples/hwp3-table-grid-gap.hwp` | hwp | no_oracle_pdf |
| 54 | `samples/HWP5-nopassword-123456.hwp` | hwp | no_oracle_pdf |
| 55 | `samples/HWP5-nopassword-123456.hwpx` | hwpx | no_oracle_pdf |
| 56 | `samples/HWP5-password-123456.hwpx` | hwpx | no_oracle_pdf |
| 57 | `samples/hwp5-tbl-attr-1916.hwp` | hwp | no_oracle_pdf |
| 58 | `samples/hwp_table_test_saved.hwp` | hwp | no_oracle_pdf |
| 59 | `samples/hwpers_test4_complex_table.hwp` | hwp | no_oracle_pdf |
| 60 | `samples/hwpx/143E433F503322BD33.hwpx` | hwpx | no_oracle_pdf |
| 61 | `samples/hwpx/2026_oss_rst.hwpx` | hwpx | no_oracle_pdf |
| 62 | `samples/hwpx/[2027] 온새미로 1 본교재.hwpx` | hwpx | no_oracle_pdf |
| 63 | `samples/hwpx/basic-table-01.hwpx` | hwpx | no_oracle_pdf |
| 64 | `samples/hwpx/business_overview.hwpx` | hwpx | no_oracle_pdf |
| 65 | `samples/hwpx/el-school-001.hwpx` | hwpx | no_oracle_pdf |
| 66 | `samples/hwpx/eq-002.hwpx` | hwpx | no_oracle_pdf |
| 67 | `samples/hwpx/exam-kor-1p.hwpx` | hwpx | no_oracle_pdf |
| 68 | `samples/hwpx/exam-kor-2p.hwpx` | hwpx | no_oracle_pdf |
| 69 | `samples/hwpx/exam-kor-3p.hwpx` | hwpx | no_oracle_pdf |
| 70 | `samples/hwpx/exam-kor-4p.hwpx` | hwpx | no_oracle_pdf |
| 71 | `samples/hwpx/exam_kor.hwpx` | hwpx | no_oracle_pdf |
| 72 | `samples/hwpx/exam_social-p1.hwpx` | hwpx | no_oracle_pdf |
| 73 | `samples/hwpx/exam_social.hwpx` | hwpx | no_oracle_pdf |
| 74 | `samples/hwpx/expense_report.hwpx` | hwpx | no_oracle_pdf |
| 75 | `samples/hwpx/field-multipara-clickhere.hwpx` | hwpx | no_oracle_pdf |
| 76 | `samples/hwpx/footnote-01.hwpx` | hwpx | no_oracle_pdf |
| 77 | `samples/hwpx/footnote-tbox-01.hwpx` | hwpx | no_oracle_pdf |
| 78 | `samples/hwpx/form-01.hwpx` | hwpx | no_oracle_pdf |
| 79 | `samples/hwpx/form-02.hwpx` | hwpx | no_oracle_pdf |
| 80 | `samples/hwpx/hancom-hwp/2024년 1분기 해외직접투자 보도자료 ff.hwp` | hwp | no_oracle_pdf |
| 81 | `samples/hwpx/hancom-hwp/2024년 2분기 해외직접투자 보도자료ff.hwp` | hwp | no_oracle_pdf |
| 82 | `samples/hwpx/hancom-hwp/2024년 연간 해외직접투자 보도자료 _ ff.hwp` | hwp | no_oracle_pdf |
| 83 | `samples/hwpx/hancom-hwp/2025년 1분기 해외직접투자 보도자료f.hwp` | hwp | no_oracle_pdf |
| 84 | `samples/hwpx/hancom-hwp/2025년 2분기 해외직접투자 (최종).hwp` | hwp | no_oracle_pdf |
| 85 | `samples/hwpx/hancom-hwp/[2027] 온새미로 1 본교재.hwp` | hwp | no_oracle_pdf |
| 86 | `samples/hwpx/hancom-hwp/activity_report.hwp` | hwp | no_oracle_pdf |
| 87 | `samples/hwpx/hancom-hwp/blank_hwpx.hwp` | hwp | no_oracle_pdf |
| 88 | `samples/hwpx/hancom-hwp/business_overview.hwp` | hwp | no_oracle_pdf |
| 89 | `samples/hwpx/hancom-hwp/expense_report.hwp` | hwp | no_oracle_pdf |
| 90 | `samples/hwpx/hancom-hwp/form-002.hwp` | hwp | no_oracle_pdf |
| 91 | `samples/hwpx/hancom-hwp/hang_job_01.hwp` | hwp | no_oracle_pdf |
| 92 | `samples/hwpx/hancom-hwp/hwpx-01.hwp` | hwp | no_oracle_pdf |
| 93 | `samples/hwpx/hancom-hwp/hwpx-02.hwp` | hwp | no_oracle_pdf |
| 94 | `samples/hwpx/hancom-hwp/hwpx-h-01.hwp` | hwp | no_oracle_pdf |
| 95 | `samples/hwpx/hancom-hwp/hwpx-h-02.hwp` | hwp | no_oracle_pdf |
| 96 | `samples/hwpx/hancom-hwp/hwpx-h-03.hwp` | hwp | no_oracle_pdf |
| 97 | `samples/hwpx/hancom-hwp/hy-001.hwp` | hwp | no_oracle_pdf |
| 98 | `samples/hwpx/hancom-hwp/hy-002.hwp` | hwp | no_oracle_pdf |
| 99 | `samples/hwpx/hancom-hwp/issue_157.hwp` | hwp | no_oracle_pdf |
| 100 | `samples/hwpx/hancom-hwp/issue_241.hwp` | hwp | no_oracle_pdf |
| 101 | `samples/hwpx/hancom-hwp/mel-001.hwp` | hwp | no_oracle_pdf |
| 102 | `samples/hwpx/hancom-hwp/service_agreement.hwp` | hwp | no_oracle_pdf |
| 103 | `samples/hwpx/hancom-hwp/tb-org-02.hwp` | hwp | no_oracle_pdf |
| 104 | `samples/hwpx/hancom-hwp/tbox-v-flow-01.hwp` | hwp | no_oracle_pdf |
| 105 | `samples/hwpx/hcar-001.hwpx` | hwpx | no_oracle_pdf |
| 106 | `samples/hwpx/hwpx-centered-cell-vpos-after-tac-shape.hwpx` | hwpx | no_oracle_pdf |
| 107 | `samples/hwpx/hy-001.hwpx` | hwpx | no_oracle_pdf |
| 108 | `samples/hwpx/hy-002.hwpx` | hwpx | no_oracle_pdf |
| 109 | `samples/hwpx/issue1535_coanchored_float_exclusion.hwpx` | hwpx | no_oracle_pdf |
| 110 | `samples/hwpx/issue1948_cross_para_fieldend.hwpx` | hwpx | no_oracle_pdf |
| 111 | `samples/hwpx/issue2019_floating_form_74312.hwpx` | hwpx | no_oracle_pdf |
| 112 | `samples/hwpx/issue2439_page_local_float_exclusion.hwpx` | hwpx | no_oracle_pdf |
| 113 | `samples/hwpx/issue_1133.hwpx` | hwpx | no_oracle_pdf |
| 114 | `samples/hwpx/k-water-rfp.hwpx` | hwpx | no_oracle_pdf |
| 115 | `samples/hwpx/landscape-001.hwpx` | hwpx | no_oracle_pdf |
| 116 | `samples/hwpx/local-font-nanumsquare-bold.hwpx` | hwpx | no_oracle_pdf |
| 117 | `samples/hwpx/math-001.hwpx` | hwpx | no_oracle_pdf |
| 118 | `samples/hwpx/opengov/36382399_결재문서본문_일반지출결의서_기간제 소블록 세척 작업용품 구매.hwpx` | hwpx | no_oracle_pdf |
| 119 | `samples/hwpx/opengov/36382669_결재문서본문_’26년 제3차 강사선정 심의회 운영결과.hwpx` | hwpx | no_oracle_pdf |
| 120 | `samples/hwpx/opengov/36383351_결재문서본문_[관악산] 산악구조대 구급의약품 폐기 계획 보고.hwpx` | hwpx | no_oracle_pdf |
| 121 | `samples/hwpx/opengov/36383351_결재문서본문_관악산산악구조대급식의약품폐기.hwpx` | hwpx | no_oracle_pdf |
| 122 | `samples/hwpx/opengov/36384285_결재문서본문_[여의도] 2026년 6월 기본업무수행 급식비 지급.hwpx` | hwpx | no_oracle_pdf |
| 123 | `samples/hwpx/opengov/36384689_결재문서본문_화재발생종합보고서(제2026-298호).hwpx` | hwpx | no_oracle_pdf |
| 124 | `samples/hwpx/opengov/36385226_결재문서본문_제2처리장 슬러지인발용 에어리프트 브로워 2호기 소모품 교체 보고.hwpx` | hwpx | no_oracle_pdf |
| 125 | `samples/hwpx/opengov/36385445_결재문서본문_화재발생종합보고서(제2026-189호, 2026. 5. 14.).hwpx` | hwpx | no_oracle_pdf |
| 126 | `samples/hwpx/opengov/36385464_결재문서본문_근무변경(반포수난구조대, 2026-06).hwpx` | hwpx | no_oracle_pdf |
| 127 | `samples/hwpx/opengov/36386761_백제학연구총서위탁판매의뢰목록.hwpx` | hwpx | no_oracle_pdf |
| 128 | `samples/hwpx/opengov/36387103_결재문서본문_수질검사 신청 민원 처리 결과 안내.hwpx` | hwpx | no_oracle_pdf |
| 129 | `samples/hwpx/opengov/36387725_footer_page_bottom.hwpx` | hwpx | no_oracle_pdf |
| 130 | `samples/hwpx/opengov/36388571_결재문서본문_2026년 6월 생일 축하품 수령증 제출(119항공대).hwpx` | hwpx | no_oracle_pdf |
| 131 | `samples/hwpx/opengov/36388711_사회보장제도 신설 협의요청서(청년오피스)_260624.hwpx` | hwpx | no_oracle_pdf |
| 132 | `samples/hwpx/opengov/36388853_결재문서본문_일상감사 의견서 송부.hwpx` | hwpx | no_oracle_pdf |
| 133 | `samples/hwpx/opengov/36389298_결재문서본문_물품검사(수)조서(라벨테이프 등).hwpx` | hwpx | no_oracle_pdf |
| 134 | `samples/hwpx/opengov/36389301_결재문서본문_직장훈련계획_덧말.hwpx` | hwpx | no_oracle_pdf |
| 135 | `samples/hwpx/opengov/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177).hwpx` | hwpx | no_oracle_pdf |
| 136 | `samples/hwpx/opengov/36392900_결재문서본문_일일굴착복구공사현황보고.hwpx` | hwpx | no_oracle_pdf |
| 137 | `samples/hwpx/opengov/36395270_footer_overpush.hwpx` | hwpx | no_oracle_pdf |
| 138 | `samples/hwpx/opengov/36398366_결재문서본문_PC 셧다운 제외 및 초과근무 인정 요청(데이터전략과).hwpx` | hwpx | no_oracle_pdf |
| 139 | `samples/hwpx/pagenation-001.hwpx` | hwpx | no_oracle_pdf |
| 140 | `samples/hwpx/para-001.hwpx` | hwpx | no_oracle_pdf |
| 141 | `samples/hwpx/para-unit-01.hwpx` | hwpx | no_oracle_pdf |
| 142 | `samples/hwpx/pr-1674.hwpx` | hwpx | no_oracle_pdf |
| 143 | `samples/hwpx/shape-001.hwpx` | hwpx | no_oracle_pdf |
| 144 | `samples/hwpx/ta-pic-001-r.hwpx` | hwpx | no_oracle_pdf |
| 145 | `samples/hwpx/tb-img-03.hwpx` | hwpx | no_oracle_pdf |
| 146 | `samples/hwpx/tb-org-02.hwpx` | hwpx | no_oracle_pdf |
| 147 | `samples/hwpx/water-mark.hwpx` | hwpx | no_oracle_pdf |
| 148 | `samples/issue-986-receipt.hwp` | hwp | no_oracle_pdf |
| 149 | `samples/issue1549_empty_host_float_clamp.hwpx` | hwpx | no_oracle_pdf |
| 150 | `samples/issue1549_multipositive_float_tables.hwpx` | hwpx | no_oracle_pdf |
| 151 | `samples/issue1639_empty_host_negative_offset_float.hwpx` | hwpx | no_oracle_pdf |
| 152 | `samples/issue1639_empty_host_positive_only_float.hwpx` | hwpx | no_oracle_pdf |
| 153 | `samples/issue1842_cell_tac_group_lineheight.hwp` | hwp | no_oracle_pdf |
| 154 | `samples/issue1858_paper_anchor_float_stack.hwpx` | hwpx | no_oracle_pdf |
| 155 | `samples/issue1880_anchor_stack_sb_convert.hwpx` | hwpx | no_oracle_pdf |
| 156 | `samples/issue1880_takeplace_host_before.hwpx` | hwpx | no_oracle_pdf |
| 157 | `samples/issue1880_takeplace_oracle_p13.hwpx` | hwpx | no_oracle_pdf |
| 158 | `samples/issue1891/76076_regulatory_analysis.hwpx` | hwpx | no_oracle_pdf |
| 159 | `samples/issue1891/80168_regulatory_analysis.hwpx` | hwpx | no_oracle_pdf |
| 160 | `samples/issue1891/80250_regulatory_analysis.hwpx` | hwpx | no_oracle_pdf |
| 161 | `samples/issue1891/86712_regulatory_analysis.hwpx` | hwpx | no_oracle_pdf |
| 162 | `samples/issue1891_external_bindata_link.hwpx` | hwpx | no_oracle_pdf |
| 163 | `samples/issue1892_hwp3_drawing_group_roundtrip.hwp` | hwp | no_oracle_pdf |
| 164 | `samples/issue1892_hwp3_tab_roundtrip.hwp` | hwp | no_oracle_pdf |
| 165 | `samples/issue1937_rowbreak_footnote_overpagination.hwp` | hwp | no_oracle_pdf |
| 166 | `samples/issue2439/issue2439_repeat_table_overlap.hwp` | hwp | no_oracle_pdf |
| 167 | `samples/issue2439_zero_offset_coanchored_float_exclusion.hwp` | hwp | no_oracle_pdf |
| 168 | `samples/issue2527_empty_linesegs.hwpx` | hwpx | no_oracle_pdf |
| 169 | `samples/issue3637/regulatory_impact_nested_table_escape.hwpx` | hwpx | no_oracle_pdf |
| 170 | `samples/issue3738/tac_sibling_shape_line_advance.hwpx` | hwpx | no_oracle_pdf |
| 171 | `samples/issue3751/vpos_reset_midparagraph_fit.hwpx` | hwpx | no_oracle_pdf |
| 172 | `samples/issue3798/page_end_trailing_spill.hwpx` | hwpx | no_oracle_pdf |
| 173 | `samples/issue3834/flow_with_text_zero.hwpx` | hwpx | no_oracle_pdf |
| 174 | `samples/issue4090/156492236_규제샌드박스_min.hwpx` | hwpx | no_oracle_pdf |
| 175 | `samples/issue4490/148720174_111014(인력기획과)민간경력자_5급_일괄채용_필기_합격자_발표.hwp` | hwp | no_oracle_pdf |
| 176 | `samples/issue4491/30213_1.혼합단지등 제도개선 방안.hwp` | hwp | no_oracle_pdf |
| 177 | `samples/issue4657/distribute-alignment-sample.hwpx` | hwpx | no_oracle_pdf |
| 178 | `samples/issue4690/30098_indent_over_stored_cs.hwp` | hwp | no_oracle_pdf |
| 179 | `samples/issue_1133.hwp` | hwp | no_oracle_pdf |
| 180 | `samples/issue_2148_degenerate_cell_vpos.hwpx` | hwpx | no_oracle_pdf |
| 181 | `samples/issues/2809/jubo_20260104.hwp` | hwp | no_oracle_pdf |
| 182 | `samples/landscape-001.hwp` | hwp | no_oracle_pdf |
| 183 | `samples/math-001.hwp` | hwp | no_oracle_pdf |
| 184 | `samples/para-001.hwp` | hwp | no_oracle_pdf |
| 185 | `samples/para-unit-01.hwp` | hwp | no_oracle_pdf |
| 186 | `samples/pic-in-table-with-toggle.hwp` | hwp | no_oracle_pdf |
| 187 | `samples/pic2-2018.hwp` | hwp | no_oracle_pdf |
| 188 | `samples/pr4093/outline_navigation_panel_demo.hwpx` | hwpx | no_oracle_pdf |
| 189 | `samples/pr4093/outline_navigation_table_cell_number.hwpx` | hwpx | no_oracle_pdf |
| 190 | `samples/render-p35-font-native-bitmap.hwpx` | hwpx | no_oracle_pdf |
| 191 | `samples/shape-001.hwp` | hwp | no_oracle_pdf |
| 192 | `samples/ta-pic-001-r.hwp` | hwp | no_oracle_pdf |
| 193 | `samples/ta-pic-cell-center-pos-bottom.hwpx` | hwpx | no_oracle_pdf |
| 194 | `samples/ta-pic-cell-top-pos-center.hwpx` | hwpx | no_oracle_pdf |
| 195 | `samples/tac-host-spacing.hwpx` | hwpx | no_oracle_pdf |
| 196 | `samples/tac-verify/scenario-a-after.hwp` | hwp | no_oracle_pdf |
| 197 | `samples/tac-verify/scenario-a-before.hwp` | hwp | no_oracle_pdf |
| 198 | `samples/tac-verify/scenario-b-after.hwp` | hwp | no_oracle_pdf |
| 199 | `samples/tac-verify/scenario-b-before.hwp` | hwp | no_oracle_pdf |
| 200 | `samples/tac-verify/scenario-c-after.hwp` | hwp | no_oracle_pdf |
| 201 | `samples/tac-verify/scenario-c-before.hwp` | hwp | no_oracle_pdf |
| 202 | `samples/tac-verify/scenario-d-after.hwp` | hwp | no_oracle_pdf |
| 203 | `samples/tac-verify/scenario-d-before.hwp` | hwp | no_oracle_pdf |
| 204 | `samples/task1700/byeolpyo1_uujeong_wrap_singlepage.hwp` | hwp | no_oracle_pdf |
| 205 | `samples/task1700/myeonjeok_wrap_10page.hwp` | hwp | no_oracle_pdf |
| 206 | `samples/task1705/wrap_empty_para_anchor_page.hwp` | hwp | no_oracle_pdf |
| 207 | `samples/task1706/empty_para_before_pagebreak.hwpx` | hwpx | no_oracle_pdf |
| 208 | `samples/task1706/empty_para_between_tac_tables.hwp` | hwp | no_oracle_pdf |
| 209 | `samples/task1716/table_scattered_header_rowbreak.hwpx` | hwpx | no_oracle_pdf |
| 210 | `samples/task1718/table_giant_cell_overfill.hwp` | hwp | no_oracle_pdf |
| 211 | `samples/task1725/text_footnote_tail_overpagination.hwp` | hwp | no_oracle_pdf |
| 212 | `samples/task1725/text_footnote_tail_overpagination.hwpx` | hwpx | no_oracle_pdf |
| 213 | `samples/task1745/table_text_anchor_wrap.hwp` | hwp | no_oracle_pdf |
| 214 | `samples/task1749/saved_bounds_cumulative_page_break.hwp` | hwp | no_oracle_pdf |
| 215 | `samples/task1749/saved_bounds_cumulative_page_break.hwpx` | hwpx | no_oracle_pdf |
| 216 | `samples/task1749/saved_bounds_cumulative_vpos.hwp` | hwp | no_oracle_pdf |
| 217 | `samples/task1749/saved_bounds_cumulative_vpos.hwpx` | hwpx | no_oracle_pdf |
| 218 | `samples/task1750/split_guard_spacing_before.hwp` | hwp | no_oracle_pdf |
| 219 | `samples/task1750/split_guard_spacing_before.hwpx` | hwpx | no_oracle_pdf |
| 220 | `samples/task1753/deferred_takeplace_fill_ahead.hwp` | hwp | no_oracle_pdf |
| 221 | `samples/task1753/deferred_takeplace_fill_ahead.hwpx` | hwpx | no_oracle_pdf |
| 222 | `samples/task1763/cell_trailing_ls_expand.hwp` | hwp | no_oracle_pdf |
| 223 | `samples/task1763/cell_trailing_ls_expand.hwpx` | hwpx | no_oracle_pdf |
| 224 | `samples/task1765/merged_cell_trailing_ls.hwp` | hwp | no_oracle_pdf |
| 225 | `samples/task1765/merged_cell_trailing_ls.hwpx` | hwpx | no_oracle_pdf |
| 226 | `samples/task1768/distribution_doc.hwpx` | hwpx | no_oracle_pdf |
| 227 | `samples/task1771/nested_group_vectors.hwpx` | hwpx | no_oracle_pdf |
| 228 | `samples/task1772/table_outer_margin_common_sync.hwpx` | hwpx | no_oracle_pdf |
| 229 | `samples/task1789/exclusion_probe_line_spacing.hwpx` | hwpx | no_oracle_pdf |
| 230 | `samples/task2070/hy_ladder3.hwpx` | hwpx | no_oracle_pdf |
| 231 | `samples/task2070/hy_ladder4.hwpx` | hwpx | no_oracle_pdf |
| 232 | `samples/task2097/17809123_jawonbongsa.hwpx` | hwpx | no_oracle_pdf |
| 233 | `samples/task2097/18095317_eogu_geumji.hwp` | hwp | no_oracle_pdf |
| 234 | `samples/task2097/21217935_simsa_jipyo.hwp` | hwp | no_oracle_pdf |
| 235 | `samples/task2097/3023771_wichokjang.hwpx` | hwpx | no_oracle_pdf |
| 236 | `samples/task2097/3248363_upmu_bunjang.hwpx` | hwpx | no_oracle_pdf |
| 237 | `samples/task2097/rowbreak_midpage_declared_fits.hwpx` | hwpx | no_oracle_pdf |
| 238 | `samples/task2105/rowbreak_table_declared_fits.hwpx` | hwpx | no_oracle_pdf |
| 239 | `samples/task2137/156637323_unification_lecture.hwpx` | hwpx | no_oracle_pdf |
| 240 | `samples/task2156/width_ladder.hwpx` | hwpx | no_oracle_pdf |
| 241 | `samples/task2169/anchor_ladder.hwpx` | hwpx | no_oracle_pdf |
| 242 | `samples/task2169/empty_ladder.hwpx` | hwpx | no_oracle_pdf |
| 243 | `samples/task2279/36353832_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 244 | `samples/task2279/36358528_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 245 | `samples/task2279/36365360_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 246 | `samples/task2279/36376848_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 247 | `samples/task2279/36378128_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 248 | `samples/task2279/36378481_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 249 | `samples/task2279/36394733_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 250 | `samples/task2279/36395825_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 251 | `samples/task2279/36398724_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 252 | `samples/task2279/36404953_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 253 | `samples/task2279/36406174_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 254 | `samples/task2279/36407074_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 255 | `samples/task2279/36410902_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 256 | `samples/task2279/36417406_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 257 | `samples/task2279/36423194_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 258 | `samples/task2279/36423558_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 259 | `samples/task2279/36425476_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 260 | `samples/task2279/36433160_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 261 | `samples/task2279/36434078_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 262 | `samples/task2279/36437313_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 263 | `samples/task2279/36446358_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 264 | `samples/task2279/36455850_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 265 | `samples/task2279/36456688_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 266 | `samples/task2279/36477251_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 267 | `samples/task2279/36477373_gyeoljae.hwpx` | hwpx | no_oracle_pdf |
| 268 | `samples/task2311/156744475_nano_plan_poster.hwpx` | hwpx | no_oracle_pdf |
| 269 | `samples/task2319/20544835_jinan_apt_form.hwp` | hwp | no_oracle_pdf |
| 270 | `samples/task2322/19439117_gokseong_voucher_form.hwp` | hwp | no_oracle_pdf |
| 271 | `samples/task2322/20862337_cheongyang_voucher_form.hwp` | hwp | no_oracle_pdf |
| 272 | `samples/task2430/1382000_domestic_violence_survey.hwp` | hwp | no_oracle_pdf |
| 273 | `samples/task3307/issue3307_outline_number.hwpx` | hwpx | no_oracle_pdf |
| 274 | `samples/tb-img-03.hwp` | hwp | no_oracle_pdf |
| 275 | `samples/test-image.hwp` | hwp | no_oracle_pdf |
| 276 | `samples/test-image.hwpx` | hwpx | no_oracle_pdf |
| 277 | `samples/test-image2.hwp` | hwp | no_oracle_pdf |
| 278 | `samples/textbox-under-image.hwp` | hwp | no_oracle_pdf |
| 279 | `samples/unicode/각 항목에 명시되어 있는_유니코드.hwp` | hwp | no_oracle_pdf |
| 280 | `samples/valign_fixtures/cell_vbottom_nested_overcount.hwpx` | hwpx | no_oracle_pdf |
| 281 | `samples/valign_fixtures/cell_vcenter_multi_nested_overcount.hwpx` | hwpx | no_oracle_pdf |
| 282 | `samples/valign_fixtures/cell_vcenter_nested_undercount.hwpx` | hwpx | no_oracle_pdf |
| 283 | `samples/valign_fixtures/centered_cell_nested_table.hwpx` | hwpx | no_oracle_pdf |
| 284 | `samples/water-mark.hwp` | hwp | no_oracle_pdf |
| 285 | `samples/대각선샘플3.hwp` | hwp | no_oracle_pdf |
| 286 | `samples/대각선샘플3.hwpx` | hwpx | no_oracle_pdf |
| 287 | `samples/대각선샘플4.hwp` | hwp | no_oracle_pdf |
| 288 | `samples/대각선샘플4.hwpx` | hwpx | no_oracle_pdf |
| 289 | `samples/대각선샘플5.hwp` | hwp | no_oracle_pdf |
| 290 | `samples/대각선샘플5.hwpx` | hwpx | no_oracle_pdf |
| 291 | `samples/셀보호.hwp` | hwp | no_oracle_pdf |
| 292 | `samples/셀보호.hwpx` | hwpx | no_oracle_pdf |
| 293 | `samples/수식-문자처럼취급-아님.hwp` | hwp | no_oracle_pdf |
| 294 | `samples/수식-문자처럼취급-아님.hwpx` | hwpx | no_oracle_pdf |
| 295 | `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp` | hwp | no_oracle_pdf |
| 296 | `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx` | hwpx | no_oracle_pdf |
| 297 | `samples/종이기준.hwp` | hwp | no_oracle_pdf |
| 298 | `samples/종이기준.hwpx` | hwpx | no_oracle_pdf |
| 299 | `samples/쪽기준.hwp` | hwp | no_oracle_pdf |
| 300 | `samples/쪽기준.hwpx` | hwpx | no_oracle_pdf |
| 301 | `samples/투명도0-50-2nd그림글차처럼off.hwp` | hwp | no_oracle_pdf |
| 302 | `samples/투명도0-50-2nd그림글차처럼off.hwpx` | hwpx | no_oracle_pdf |
| 303 | `samples/투명도0-50.hwp` | hwp | no_oracle_pdf |
| 304 | `samples/투명도0-50.hwpx` | hwpx | no_oracle_pdf |
| 305 | `samples/한글문서파일형식_5.0_revision1.3.hwp` | hwp | no_oracle_pdf |
