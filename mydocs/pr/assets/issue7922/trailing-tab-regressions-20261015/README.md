# #7922: 실물 회귀 입력과 검사 원문 이관

- 추적: [#7922](https://github.com/edwardkim/rhwp/issues/7922), 대표 [#7445](https://github.com/edwardkim/rhwp/issues/7445)
- 제거 전 source SHA: `d80593d15a3dba4c6c8066f8792b26fd6f02b6ff`
- 기준 upstream SHA: `463c48cc228170b576c832591b76826116e86a49`
- 사용자는 현재 전쪽90% 이상·완료 페이지 px 회귀 범위를 확인한 문서 이외의 실패 검사를 이관하도록 지정했다. 이번 이관을 새로운 base90% 미달 입증이나 실제 출력 결함 해결로 보고하지 않는다.
- 입력·독립 PDF를 아래 assets 경로로 이동했다. 제거 전 검사 원문은 자동 수집되지 않는 `.rs.txt`로 바이트 보존했다.
- parser·serializer·외부 이미지 등 범위 밖 정상 계약은 새 보존 경로를 직접 참조한다. 일반 samples 자동 수집 및 해당 render baseline 행에서는 분리한다.

## 보존 파일과 이전 경로

| 이전 경로 | 현재 파일 | SHA-256 |
| --- | --- | --- |
| `samples/issue6800/1192000-202100017-policy-research-report.hwp` | [1192000-202100017-policy-research-report.hwp](1192000-202100017-policy-research-report.hwp) | `fd0b95cb4239b08e2ab9130b6b697af56dda379f1c9029dbf5f5a0b97af5ceee` |
| `pdf/1192000-202100017-policy-research-report-2020.pdf` | [1192000-202100017-policy-research-report-2020.pdf](1192000-202100017-policy-research-report-2020.pdf) | `34c9173724fa5d2ef5e0b2a796b56c9b564d3f5e0c9224609dffd507921ada48` |

## 제거 전 검사 원문

- [issue_6800_tab_run_bbox_width.rs.txt](issue_6800_tab_run_bbox_width.rs.txt) / SHA-256 `9ba9c2bc8b45788a89205e436b97145eb30e7bdf49134482cf26062914a1e965`

## 판정과 분리 범위

- 실패3건: `maintainer_public_character_positions_share_the_resolved_tab_end`, `maintainer_hit_test_uses_the_resolved_trailing_tab_positions`, `the_trailing_tab_run_ends_exactly_where_the_next_run_starts`.
- 현재 끝 탭 줄은 `"  "` → `"- \t"` → `"내수면 어업인의 경쟁력 확보…"`로 분리된다. 기존 단일 런 검색이 실패한 것이며 실제 캐럿/탭 결함의 재발을 확정하지 않는다.
- 같은 파일의 정상 겹침 검사와 다른 문서 `lseg-05-tab.hwp`의 중간 탭 음성 검사는 유지한다. 보존 원문의 모든 함수를 제거한 것으로 해석하지 않는다.
- 과거 전1쪽 Native/fresh WASM95.92922% 기록은 `mydocs/pr/assets/issue7445/README.md`에 보존돼 있다. 이번 현재21문서의98% px 검증 대상은 아니며, 사용자 최신 범위 지정에 따라 실패3건만 이관한다.
- [원 fixture 설명](fixture-origin-README.md)과 [이관 manifest](MANIFEST.json)도 보존한다.

## 복원 조건

같은 입력·독립 Print의 전쪽 Native/fresh WASM 최저90% 이상과 직접 확대 판독을 확보한 뒤 내용·줄/개체 소속·폭 비율·캐럿 경계 관계로 복원한다. ±2px 좌표를 고정하려면 해당 페이지 양쪽98% 이상과 독립 기대 좌표를 추가로 입증한다. 이관은 이슈 종료나 문서 피델리티 완료가 아니다.
