# 각주 표지가 있는 셀의 저장 앞 간격 복구

같은 원본 `samples/issue5966/1130000-202100008_franchise_review_report.hwp`와 독립 한컴 2020
Print `pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue5966_franchise_review_report_1130000-202100008_franchise_review_report-2020.pdf`의 142쪽을 사용했다.
고정 base는 `605ccfd92f2f12f18e14182548d729f9929fda00`, 후보는 `a2534770631aa2351ad34749504ae3472888f8e1`에
공통 앞 간격 보정을 적용한 실행이다. 조건은 Print/compat 2022, 96DPI, raw 2px 실루엣이다.
이전과 같은 명시적 Human TrueType·Haansoft glyph fallback 환경 및 PDFium 153을 사용했다.

- Native SHA-256: `0ee95375f663b3bd62c8e474ced27b452e5fac0b3fb920995f5ff496f345614a`.
- fresh WASM SHA-256: `6e77d989658fc411514c205b1ddfbd6f3caffafdfb6769ce5d95e7c2a7bf3177`.
- Mac locked web/no-opt 로컬 대체 빌드이며 Docker 최적화 통과가 아니다.

| 출력 | 전쪽 수 | 최저 raw | 90% 미만 | 68쪽 raw |
| --- | --- | ---: | --- | ---: |
| 고정 base Native | 142/142 | 88.83113% | 68·70쪽 | 88.83113% |
| 보정 후 Native | 142/142 | 89.20533% | 70쪽 | 100.00000% |
| 보정 후 fresh WASM | 142/142 | 89.17954% | 70쪽 | 100.00000% |

변경 전 후보는 집중 3쪽만 산출했으며 68쪽은 87.55884%다. 그 TSV를 전쪽 증거로 쓰지 않는다.
보정 후 양쪽 backend의 68쪽 review·overlay를 직접 판독했다. 밑줄 본문과 각주 표지의 소속·순서가
보존되며 저장 앞 간격이 실제 본문에 복원됐다. 100%는 RGB byte 동일성의 뜻이 아니다.
70쪽과 전체 회귀 15건 실패가 남아 있으므로 이 자료는 PR 제출 완료 증거가 아니다.
회귀 기대값이나 문턱은 바꾸지 않았고 새 검사는 전쪽 선행 조건 완료 후 검토한다.

## 직접 판독 자료

| 출력 | review | overlay |
| --- | --- | --- |
| 변경 전 Native | [review](before-native-p068-review.png) | [overlay](before-native-p068-overlay.png) |
| 변경 후 Native | [review](after-native-p068-review.png) | [overlay](after-native-p068-overlay.png) |
| 변경 후 fresh WASM | [review](after-wasm-p068-review.png) | [overlay](after-wasm-p068-overlay.png) |

원본 TSV를 그대로 복사했다. 설치 글꼴·source·입력·PDF의 hash, 실행 명령, 로그, 실행 신원 및
70쪽 잔여 진단은 ignored `rebase-8155-20261022/stage60-centered-cell-audit/`에 보존한다.
