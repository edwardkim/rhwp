# 중첩 표 뒤 빈 저장 줄의 정렬 간격 복구

기존 가맹사업 보고서 `samples/issue5966/1130000-202100008_franchise_review_report.hwp`와
독립 한컴 2020 Print `pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue5966_franchise_review_report_1130000-202100008_franchise_review_report-2020.pdf`의 전 142쪽을 대조했다.
고정 base는 `605ccfd92f2f12f18e14182548d729f9929fda00`, 후보는 `85ee912697e698c3e52bed5dd6c83af6b99f1014`에
61단계 공통 정렬 보정을 적용한 실행이다. 조건은 Print/compat 2022, 96DPI, raw 2px 실루엣,
기존 명시적 Human TrueType·Haansoft glyph fallback 및 PDFium 153이다.

- Native SHA-256: `381c6bcd30d2017ae428b34029aa20d56d9ef55e044bddd0892f9c09583f5e84`.
- fresh WASM SHA-256: `ef42405714824cb41e77c2ac3eb0463300ca85b14d1281347215ee1404d6fa9e`.
- Mac locked web/no-opt 로컬 대체 빌드이며 Docker 최적화 통과가 아니다.

| 출력 | 쪽수 | 전쪽 최저 raw | 미달 | 70쪽 raw |
| --- | --- | ---: | --- | ---: |
| 변경 전 Native | 142/142 | 89.20533% | 70쪽 | 89.20533% |
| 변경 전 fresh WASM | 142/142 | 89.17954% | 70쪽 | 89.17954% |
| 변경 후 Native | 142/142 | 92.75649% | 없음 | 100.00000% |
| 변경 후 fresh WASM | 142/142 | 92.75394% | 없음 | 100.00000% |

양쪽 backend의 70쪽 review·overlay를 직접 판독했다. Native 전 142쪽에서70쪽 PNG만 바뀌고
TextRun의 내용·순서는 모두 같다. 68쪽은 두 backend 모두 이전 직접 판독 PNG와 SHA가 같고
각각 100%다. 100%는 RGB byte 동일성이나 모든 글꼴 획 차이의 해소를 뜻하지 않는다.

고정 base의 같은 전쪽 Native/fresh WASM TSV도 보존했다. Native 최저 88.83113%(68·70쪽 미달),
WASM 최저 32.67053%(50·51·52·53·54·68쪽 미달)다. 각 backend의 같은 조건 base와 후보를
페이지별로 대조한 0.1%p 이상 하락은 0쪽이다. 최대 하락은 Native 0.06510%p / WASM 0.01892%p다.
이전 Native를 WASM의 기준으로 대신 쓰지 않았다.

## 직접 판독 자료

| 출력 | review | overlay |
| --- | --- | --- |
| 변경 전 Native | [review](before-native-p070-review.png) | [overlay](before-native-p070-overlay.png) |
| 변경 후 Native | [review](after-native-p070-review.png) | [overlay](after-native-p070-overlay.png) |
| 변경 후 fresh WASM | [review](after-wasm-p070-review.png) | [overlay](after-wasm-p070-overlay.png) |

전체 TSV를 원본 그대로 복사했다. 68·70쪽의 관계·독립 PDF 기준선 보호 검사는 전쪽 선행 조건과
직접 판독 이후 추가했다. 다른 기존 실패 15건과 최신 전체 회귀·최종 필수 게이트·실제 소비 경로는
아직 남아 있으므로 PR 완료 증거가 아니다. 세부 실행·producer·비교·로그는 ignored
`rebase-8155-20261022/stage61-empty-closing-line-alignment/`에 보존한다.

## 회귀 검사 검증

새 두 검사는 같은 source test binary를 변경 전 frozen Native로 직접 실행해 각각 실제 FAIL을
확인했다. 앞·뒤 띠 관계가 깨졌다는 실패이며 CLI 오류를 실패 재현으로 세지 않았다.
수정 후 현재 Native의 해당 모듈 8건은 nextest에서 8 PASS, 2.784초다.
첫 저장 앞 띠·현재 조각의 마지막 빈 줄 상자 보존을 관계로 검사하며, 같은 두 쪽의 독립 PDF
기준선은 ±2 CSSpx로 보호한다. 새 검사의 CLI TextLine 원점·높이는 실제 Native PNG 생산자가
기록한 같은 셀의 RenderTree와 모두 일치한다.
