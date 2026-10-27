# PR #8319 후속 증적의 입력 대응 정정

원 PR의 실제 보고 문서와 공개 API 재현 입력을 혼동한 이미지 연결을 정정한다.
**원 PR에 검증용 HWP/HWPX 원본이 첨부되지 않아 유사 문서를 사용한 사례에는 대체 여부를 표시한다. 파일 없는 단위 계약은 별도로 구분한다.**
대체 자료는 원 보고 문서를 직접 검증한 것으로 표현하지 않는다. 공식 배포처에서 내용 대응을 확인한 자료도 작성자의 원본 바이트와 동일하다는 뜻은 아니다.

39개 유효 재현 입력은 같은 입력·Print·페이지의 Native/fresh WASM 전쪽 TSV와 대표 review/overlay를 새로 산출했다.
이 중 18건은 원 보고 실물 문서 대신 사용한 대체 입력이다. 5건은 파일을 소비하지 않는 단위 계약이고, #8033은 저장본의 쪽수 차이가 남은 추가 진단으로 구분한다.
새로 찾은 공식 문서의 전 페이지 90% 보정은 요청 범위가 아니다. 코드·회귀·baseline·지침은 변경하지 않는다.

| 원 PR | 검증 입력 분류 | 실제 증적 입력 / 보고 문서 |
| --- | --- | --- |
| [#7559](https://github.com/edwardkim/rhwp/pull/7559) | 공개 재현 입력 | mydocs/pr/assets/computed_empty_host_line_20261020/public_api_originals/inputs/offset-hwpx.hwpx |
| [#7572](https://github.com/edwardkim/rhwp/pull/7572) | 공개 재현 입력 | mydocs/pr/assets/s21_20261027/inputs/178.hwpx |
| [#7573](https://github.com/edwardkim/rhwp/pull/7573) | 공개 재현 입력 | mydocs/pr/assets/s21_20261027/inputs/181.hwpx |
| [#7588](https://github.com/edwardkim/rhwp/pull/7588) | 공개 재현 입력 | mydocs/pr/assets/semanticist21_group_geometry_20261021/inputs/hwpx/pr7588-pr8042-ungroup-30.hwpx |
| [#7589](https://github.com/edwardkim/rhwp/pull/7589) | 공개 재현 입력 | mydocs/pr/assets/pr_7589_form_multiline/pr7589-multiline-true.hwpx |
| [#7662](https://github.com/edwardkim/rhwp/pull/7662) | 공개 재현 입력 | mydocs/pr/assets/s21_20261027/inputs/188.hwpx |
| [#7694](https://github.com/edwardkim/rhwp/pull/7694) | 공개 재현 입력 | tests/fixtures/inline-control-move/chart-after.hwp |
| [#7819](https://github.com/edwardkim/rhwp/pull/7819) | 공개 재현 입력 | tests/fixtures/char_overlap_caret_axis/nonbmp-endnote-true.hwp |
| [#7833](https://github.com/edwardkim/rhwp/pull/7833) | 공개 재현 입력 | tests/fixtures/hwpx_cell_frame_print/pr7833-flat-false-tables-false-cut.hwpx |
| [#7904](https://github.com/edwardkim/rhwp/pull/7904) | 공개 재현 입력 | mydocs/pr/assets/s21_20261027/inputs/211.hwpx |
| [#8014](https://github.com/edwardkim/rhwp/pull/8014) | 유사 문서 대체 | mydocs/pr/assets/s21_20261027/inputs/195.hwpx |
| [#8025](https://github.com/edwardkim/rhwp/pull/8025) | 단위 계약 / 파일 없는 단위 계약 | 문서 입력 없음 |
| [#8027](https://github.com/edwardkim/rhwp/pull/8027) | 원문 미첨부 / 파일 없는 단위 계약 | 재정경제부 고용동향 브리핑 HWPX 3쪽 |
| [#8029](https://github.com/edwardkim/rhwp/pull/8029) | 원문 미첨부 / 파일 없는 단위 계약 | 중앙선관위 선거 안내 포스터 |
| [#8030](https://github.com/edwardkim/rhwp/pull/8030) | 원문 미첨부 / 파일 없는 단위 계약 | 표준근로계약서의 따옴표 |
| [#8031](https://github.com/edwardkim/rhwp/pull/8031) | 유사 문서 대체 | tests/fixtures/pr8031_page_number_cell/pr8031-api-defined-page-number-cell.hwpx |
| [#8032](https://github.com/edwardkim/rhwp/pull/8032) | 공개 재현 입력 | tests/fixtures/pr8032_print_alpha/wide.hwpx |
| [#8033](https://github.com/edwardkim/rhwp/pull/8033) | 유사 문서 대체 / 추가 진단 실패 | 서울시교육청 가정통신문의 공익제보센터 안내 표 |
| [#8035](https://github.com/edwardkim/rhwp/pull/8035) | 공개 재현 입력 | mydocs/pr/assets/s21_20261027/inputs/212.hwpx |
| [#8036](https://github.com/edwardkim/rhwp/pull/8036) | 유사 문서 대체 | tests/fixtures/pr8036_stored_tall_line/continuing.hwpx |
| [#8037](https://github.com/edwardkim/rhwp/pull/8037) | 공개 재현 입력 | mydocs/pr/assets/pr8319_evidence_alignment_20261027/inputs/pr8037-body-picture-on-host.hwpx |
| [#8038](https://github.com/edwardkim/rhwp/pull/8038) | 공개 재현 입력 | mydocs/pr/assets/raised_dot_clip_20261025/raised-dot.hwpx |
| [#8039](https://github.com/edwardkim/rhwp/pull/8039) | 유사 문서 대체 | tests/fixtures/hwpx_cell_frame_print/pr8039-control-tail-true.hwpx |
| [#8041](https://github.com/edwardkim/rhwp/pull/8041) | 공개 재현 입력 | mydocs/pr/assets/semanticist21_group_geometry_20261021/inputs/hwpx/pr8041-case-3-base-0-after.hwpx |
| [#8042](https://github.com/edwardkim/rhwp/pull/8042) | 공개 재현 입력 | mydocs/pr/assets/active_field_group_owner_20261021/inputs/ungroup-after.hwp |
| [#8043](https://github.com/edwardkim/rhwp/pull/8043) | 공개 재현 입력 | mydocs/pr/assets/s21_20261027/inputs/197.hwpx |
| [#8045](https://github.com/edwardkim/rhwp/pull/8045) | 유사 문서 대체 | tests/fixtures/pr8045_square_wrap/square-hancom2020.hwpx |
| [#8049](https://github.com/edwardkim/rhwp/pull/8049) | 유사 문서 대체 | mydocs/pr/assets/caption_autonumber_projection_20261021/inputs/ta-pic-caption-tight.hwpx |
| [#8050](https://github.com/edwardkim/rhwp/pull/8050) | 유사 문서 대체 | mydocs/pr/assets/s21_20261027/inputs/198.hwpx |
| [#8051](https://github.com/edwardkim/rhwp/pull/8051) | 유사 문서 대체 | mydocs/pr/assets/s21_20261027/inputs/199.hwpx |
| [#8052](https://github.com/edwardkim/rhwp/pull/8052) | 유사 문서 대체 | mydocs/pr/assets/s21_20261027/inputs/200.hwpx |
| [#8053](https://github.com/edwardkim/rhwp/pull/8053) | 유사 문서 대체 | mydocs/pr/assets/s21_20261027/inputs/202.hwpx |
| [#8054](https://github.com/edwardkim/rhwp/pull/8054) | 유사 문서 대체 | mydocs/pr/assets/s21_20261027/inputs/203.hwpx |
| [#8055](https://github.com/edwardkim/rhwp/pull/8055) | 유사 문서 대체 | mydocs/pr/assets/float_fragment_frames_20261020/inputs/hwp/public-rowbreak-normalized-2020.hwp |
| [#8056](https://github.com/edwardkim/rhwp/pull/8056) | 유사 문서 대체 | mydocs/pr/assets/s21_20261027/inputs/205.hwpx |
| [#8057](https://github.com/edwardkim/rhwp/pull/8057) | 유사 문서 대체 | mydocs/pr/assets/verified_point_suppliers_20261025/inputs/gulim.hwpx |
| [#8058](https://github.com/edwardkim/rhwp/pull/8058) | 단위 계약 / 파일 없는 단위 계약 | 문서 입력 없음 |
| [#8061](https://github.com/edwardkim/rhwp/pull/8061) | 유사 문서 대체 | tests/fixtures/pr8061_cell_picture_layer/picture-z2.hwpx |
| [#8062](https://github.com/edwardkim/rhwp/pull/8062) | 유사 문서 대체 | mydocs/pr/assets/s21_20261027/inputs/206.hwpx |
| [#8065](https://github.com/edwardkim/rhwp/pull/8065) | 유사 문서 대체 | mydocs/pr/assets/s21_20261027/inputs/207.hwpx |
| [#8066](https://github.com/edwardkim/rhwp/pull/8066) | 유사 문서 대체 | mydocs/pr/assets/s21_20261027/inputs/208.hwpx |
| [#8090](https://github.com/edwardkim/rhwp/pull/8090) | 공개 재현 입력 | mydocs/pr/assets/s21_20261027/inputs/209.hwpx |
| [#8093](https://github.com/edwardkim/rhwp/pull/8093) | 공개 재현 입력 | tests/fixtures/pr8093_form_caret/form-slot.hwp |
| [#8094](https://github.com/edwardkim/rhwp/pull/8094) | 공개 재현 입력 | mydocs/pr/assets/pr8319_evidence_alignment_20261027/inputs/pr8094-caption-number-sudal.hwpx |
| [#8096](https://github.com/edwardkim/rhwp/pull/8096) | 공개 재현 입력 | tests/fixtures/pr8096_form_defaults/form-only-defaults-hwpx.hwpx |

[PR별 입력·SHA·페이지 대응 원장](alignment.json), [공식 배포 문서의 출처와 대체 표시](source-identification.md)를 함께 확인한다.
기존 댓글만 수정하며 새 정정 댓글은 만들지 않는다. 과거 원 작성자의 캡처·실행 로그·중간 JSON·비공개 글꼴은 이 제출 자료에 포함하지 않는다.
삭제된 손상 이동 HWP 두 개는 복원하지 않았다.

## 원본과 PDF 동반 보존

[HWP/HWPX와 PDF 대응표](source-input-pdf-pairs.md)에서 공식 배포 문서 16개와 확보한 Print PDF 16개를 함께 확인할 수 있다. 배포 PDF와 Print 출력 계약을 확인한 기준 PDF를 구분한다. 입력·Print PDF 확보는 완료했으며 원 PR의 실제 주장 영역 확인과 필요한 공통 로직 보정·새 head 검증이 끝나기 전까지 이 PR의 최종 수용·머지를 보류한다.
