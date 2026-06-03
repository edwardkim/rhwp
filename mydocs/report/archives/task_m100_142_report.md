# 최종 보고서: 수식 레이아웃 보정 (#142, #159)

- **타스크**: [#142](https://github.com/edwardkim/rhwp/issues/142), [#159](https://github.com/edwardkim/rhwp/issues/159)
- **마일스톤**: M100
- **브랜치**: `local/task142`
- **작성일**: 2026-04-17

## 1. 해결된 문제 (15건)

| 문제 | 수정 내용 |
|------|----------|
| 수식-텍스트 겹침 (TAC 너비) | composer.rs 레이아웃 산출값 → HWP 저장값 복원 |
| `f'(1)` prime 폭 과대 | Unicode 수학 기호별 폭 세분화 |
| `cos(π/2+θ)=-1/5` 분수 오류 | OVER 중위 연산자 재설계 |
| `lim` 폰트 크기 비정상 | font_size_from_box 버그 수정 → fs 직접 사용 |
| 수식 y축 정렬 불일치 | layout_box.baseline 기준 배치 |
| 분수 baseline Row 겹침 | axis_height(0.25em) 기반 baseline |
| 분수선 SVG y좌표 | baseline - axis_height |
| sqrt {n} of {x} 중괄호 | parse_sqrt에서 {n} of 패턴 지원 |
| log₂ 밑 첨자 떨어짐 | try_parse_scripts에서 Thin 공백 건너뛰기 |
| 적분 ∫ 배치 | MathSymbol BIG_OP_SCALE + 적분 전용 SubSup (KaTeX 참조) |
| sin/cos/tan 첨자 | Thin 공백 건너뛰기로 함수 첨자 정상 파싱 |
| bar{rm AD it} `}` 출력 | parse_single_or_group에서 RBrace 처리 |
| ③+수식0 겹침 | inline_tabs 동기화 (estimate_text_width + compute_char_positions) |
| CASES && 이중 탭 | CASES/EQALIGN 파서에서 && → Tab 처리 |
| rm P it left(...) | FontStyle 뒤 구조 명령어 body 먹힘 방지 |

## 2. 추가 개선

| 항목 | 내용 |
|------|------|
| 수식 폰트 | Latin Modern Math font-family 적용 (#141) |
| 분수 여백 | FRAC_LINE_PAD 0.15 → 0.2 |
| 함수명 뒤 Thin 공백 | 파서에서 함수 뒤 ` 소비 |
| 원문자 전각 | is_fullwidth_symbol 확장 (①~⑤ 등) |
| 탭 /2.0 | 한컴 격자 비교로 올바른 변환 확인 |
| Function 여백 | 0.1 → 0.02 (미세 조정) |
| 격자 오버레이 | --show-grid 옵션 (#145) |

## 3. 미해결 (별도 이슈 분리)

| 이슈 | 문제 |
|------|------|
| [#174](https://github.com/edwardkim/rhwp/issues/174) | 수식 TAC 높이 줄 반영 — 큰 수식+텍스트 겹침 (16,20페이지) |
| [#175](https://github.com/edwardkim/rhwp/issues/175) | CASES+EQALIGN 한글 혼합 오버래핑 (8페이지) |

## 4. 시각 검증 결과 (exam_math.hwp 20페이지)

| 페이지 | 상태 |
|--------|------|
| 1~7, 9~15, 17~19 | ✅ 통과 (15/20) |
| 6 | ✅ CASES && 해결 |
| 9 | ✅ rm P it left 해결 |
| 8 | ⚠ 미해결 → #175 |
| 16, 20 | ⚠ 미해결 → #174 |

## 5. 수정 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/equation/parser.rs` | OVER 재설계, sqrt, bar, nolimits, 함수 Thin 공백, CASES &&, FontStyle body 방지 |
| `src/renderer/equation/layout.rs` | axis_height, FRAC_LINE_PAD, 적분 전용 레이아웃, Unicode 폭, is_integral_symbol |
| `src/renderer/equation/svg_render.rs` | font-family, font_size_from_box→fs, 분수선 y, 적분 기호 크기 |
| `src/renderer/layout/text_measurement.rs` | inline_tabs in estimate_text_width, is_fullwidth_symbol 확장 |
| `src/renderer/layout/paragraph_layout.rs` | inline_tabs 정렬 동기화, baseline 정렬 |
| `src/renderer/composer.rs` | 수식 TAC 너비 HWP 저장값 |
| `src/renderer/style_resolver.rs` | 탭 /2.0 주석 정리 |
| `src/renderer/svg.rs` | 수식 bbox 스케일링, 폰트 임베딩 |
| `src/main.rs` | dump 수식 정보, --show-grid |

## 6. 테스트

- `cargo test` 790개 전체 통과
- 트러블슈팅 문서: `mydocs/troubleshootings/tab_tac_overlap_142_159.md`
