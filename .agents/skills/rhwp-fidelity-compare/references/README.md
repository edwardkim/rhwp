# 레퍼런스 목차 — rhwp-fidelity-compare

이 폴더는 실 에이전트가 한컴 공식 PDF 와 `rhwp export-svg` 를 대조할 때
여는 장이다. gym 이 아니다. 새 CLI 가 없다.

정본은 `tools/fidelity_compare/README.md` 와
`mydocs/manual/verification/visual_verification_governance.md` 이다.
이 장들은 그 정본을 에이전트 정지 규칙·예외 경로·레시피로 접는다.

| 장 | 주제 | 정지 |
| --- | --- | --- |
| 00_tree.md | 판단 트리 | F01–F16 |
| 01_when_to_use.md | 독립 PDF 있을 때만 | F01, F17 |
| 02_setup_venv.md | venv · pypdf · pillow | F09, F15 |
| 03_windows.md | `venv\Scripts\python.exe` | F15 |
| 04_page_sheets.md | `cmp-pNNN.png` | F03, F10 |
| 05_pixel_ranking.md | 최악 쪽 우선 | F03, F05 |
| 06_text_report.md | 소실/과잉/치환 | F02 |
| 07_font_style.md | `--font-style` | F04 |
| 08_local_face_aliases.md | local() 별칭 | F04, F14 |
| 09_tofu.md | PUA · U+FFFD · □ | F04, F14 |
| 10_font_path_dir.md | `RHWP_FONT_PATH_DIR` | F04 |
| 11_provenance.md | 도구·버전·경로·글꼴 | F05, F17 |
| 12_visual_verdict.md | 유지자 판정 | F05 |
| 13_missing_chrome.md | Chrome 부재 | F10 |
| 14_missing_venv.md | venv 부재 | F09 |
| 15_page_count_mismatch.md | 쪽수 | F11 |
| 16_encrypted_pdf.md | 암호화 PDF | F13 |
| 17_tofu_harness.md | 하네스 두부 오염 | F14 |
| 18_registered_keys.md | plan/manual/… | F03, F17 |
| 19_direct_pair.md | `--source` 쌍 | F12 |
| 20_outputs.md | TSV/시트 카탈로그 | F02–F12 |
| 21_vs_visual_regression.md | 축 분리 | F01, F07 |
| 22_vs_bug_hunter.md | 여정과 다름 | F08 |
| 23_journeys.md | 실사용 여정 | — |
| 24_pitfalls.md | 함정 | — |
| 25_worked_traces.md | 재현 트레이스 | — |
| 26_handoff.md | 이웃 스킬 | F01, F07, F08 |
| 27_exception_catalog.md | 예외 카탈로그 | F09–F15 |

픽스처는 `../fixtures/`. 예제는 `../examples/`.
`_gen_pack.py` 는 fixtures 만 방출한다. 이 마크다운은 손으로 유지한다.
