# #7445 잔여 11건 처리

대상은 #7813, #7821, #7822, #7830, #7831, #7834, #7863, #7864, #7922, #7955, #8100이다.
비교 빌드의 기준 커밋은 `ca96e9f342a541da8aee40f96495180b6b2ee863`이다.

## 1단계 끝 탭 커서와 장식 표의 문단 여백

- #7922의 저장 끝 탭 앞 공백이 다음 가시 런의 원점까지 남은 폭을 넘었다. `TextRunNode::resolve_trailing_tab_end`에서 탭 앞 말미 공백의 전진을 배분하고 같은 경계를 bbox와 문자 위치 배열에 사용한다. 최소 글꼴 환경에서 다음 런 직전의 클릭은 문자 4, 다음 런 안의 클릭은 문자 5가 됐다. 두 커서는 경계 248.6px를 공유하며 같은 셀 22의 문단 1에 속한다. 기준 후보에서는 다음 런 안을 클릭해도 앞 문자 4와 그 오른쪽 커서가 반환됐다.
- #7955의 장식 표는 문단 기준 가로 오프셋을 사용한다. `layout_column_shapes_pass`가 해당 문단 스타일의 가로 여백을 전쪽 표와 분할 표의 공통 배치 호출에 전달한다. 한컴 Print 1쪽의 두 장식 표 경계가 약 9px 오른쪽으로 이동해야 한다는 근거와 일치한다.

| 이슈 | 비교 쪽수 | 기준 Native 최저 | 단계별 후보 Native 최저 | 전쪽 TSV |
| --- | ---: | ---: | ---: | --- |
| #7922 | 1 | 95.37% | 95.37% | [기준](stage1/7922-base-native.tsv) · [후보](stage1/7922-candidate-native.tsv) · [최소 글꼴 클릭 결과](stage1/7922-minfont-hit.tsv) |
| #7955 | 3 | 95.87% | 99.93% | [기준](stage1/7955-base-native.tsv) · [후보](stage1/7955-candidate-native.tsv) |

측정은 2026-11-28 KST, 96dpi, 색상 경계 보정 전 2px 실루엣 기준이다. 측정 명령은
`venv/bin/python scripts/visual_sweep.py --silhouette-only --hwp <원본> --pdf <한컴 Print> --rhwp-bin <고정 후보 실행 파일> --compat <값> --dpi 96`이며 #7922는 2022, #7955는 2024 프로필을 사용했다.

| 이슈 | 비교 입력과 SHA-256 | 한컴 Print과 SHA-256 |
| --- | --- | --- |
| #7922 | [원본](../issue7922/trailing-tab-regressions-20261015/1192000-202100017-policy-research-report.hwp) `fd0b95cb4239b08e2ab9130b6b697af56dda379f1c9029dbf5f5a0b97af5ceee` | [PDF](../issue7922/trailing-tab-regressions-20261015/1192000-202100017-policy-research-report-2020.pdf) `34c9173724fa5d2ef5e0b2a796b56c9b564d3f5e0c9224609dffd507921ada48` |
| #7955 | [원본](../../../../tests/fixtures/issue_7047/housing-lease-standard-form.hwp) `fe295c12c5f9bd9e7e73a540c7c7ecd044f18ed58ea9826c0de5d55ba2710439` | [PDF](../../../../pdf/housing-lease-standard-form-2020.pdf) `f73f17d57b566d35f1641eef4ba1ab580aaf13725493b29e5ec88e7496e64e39` |

최종 커밋의 fresh WASM 측정, 복원할 관계형 회귀, 전체 회귀와 보호 문서 검증은 후속 단계에서 수행한다.

## 제출 형식 검사

2026-11-28 KST에 이번 분기의 변경 소스 14개에 Rust 서식을 적용했다.
`cargo fmt --all -- --check`를 통과했다. 위 단계의 측정은 각각 기록한 후보에
해당하며, 새 검증은 서식 커밋에서 빌드한 Native와 fresh WASM으로 실행한다.
