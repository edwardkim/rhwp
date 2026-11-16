# hyunmyungJaneLee 6개 PR 통합 검토 증적

시각·기존 회귀 검증 source는 `30fa7739f0340f2a248b22542c7dd36d9354f056`, base는
`1a3ed4a7cd76701085a24ac81d30bc5fb31942c1`(#8956)다. 원 PR은 #8895·#8897·#8900·#8916·#8918·#8920으로 고정한다.
[44개 커밋의 리베이스 대응](final-rebase-30fa7739/rebase-commit-map.json)에서 patch-id가 모두 동일함을 확인했다.

현재 완료한 결과는 최소 글꼴 Native 집중 141 PASS, 실제 브라우저 ByPath API 11 PASS,
최소 글꼴 전체 회귀 **13,365 PASS / 0 FAIL / 45 skipped**(실행 468.702초)다.
Native/fresh WASM 공개 입력 34종·46쪽을 각각 새로 출력했고 90% 미만·새 하락은 없다.
raw·PDF·review·overlay 368개가 이전 직접 판독 증적과 바이트까지 같아 중복 PNG를 만들지 않았다.
Studio 2,784 PASS / 2 skipped·production build·실행형 renderer 계약도 통과했다. Native Skia lib 3,852 PASS / 12 ignored와 보조 lib 182 PASS, placeholder 2 PASS, direct PDF 36 PASS 및 필수 16단계 게이트를 모두 통과했다. 새 #8956 base Native 4건·후보 4건과 실제 브라우저 API 11건도 통과했다. [게이트 원장](final-rebase-30fa7739/gates-validation.json)을 보존한다.

현재 #9017을 반영한 후보는 `73d06ed0b18df7c222a850bb5d9340c9b7de2bcc`, base는 `a85d3265dd10b316d1a0ae029ca3f67245b38b77`다. 기존 전쪽 TSV·PNG·독립 Print·변경 없는 Studio 전체 검사를 재사용한다. 44개 patch ID·13개 핵심 경로·기존 입력/PDF 동일성과 [재사용 경계](final-rebase-30fa7739/rebase-a85d3265/README.md)를 확인했고, 새 붙여넣기 기능과 Rust 필수 검사는 별도로 실행한다. 기존 source의 실행 기록을 새 head 완료 결과로 표시하지 않는다. 현재 제품 source `73d06ed0b1`의 추가 필수 실행을 완료했다. 최소 글꼴 Native 집중 164건·전체 13,380건이 통과했고, 새 base/후보 HTML 23건·실제 브라우저 6사례/24회 저장/API11건도 통과했다. [현재 실행 원장](final-rebase-30fa7739/rebase-a85d3265/gates-validation.json)을 기존 producer의 시각 증적과 구분한다.

## 기준 PDF와 입력

원 PR 6개는 웹한글기안기 PDF를 기준이라고 명시했다. 이 검토는 해당 PDF와 Cairo 점수를 제외하고,
동일 원본의 **Windows 데스크톱 한컴 Print PDF 34종**을 사용한다. MCP Print 33종의 실제 제품은
한컴 2020 11.0.0.9136이 25종, 한컴 2024 13.0.0.3901이 8종이다. 나머지 1종은 사용자가
Print 출력임을 확인한 한컴 2024 PDF다. 저장 제품과 Print 제품을 구분하며 공식 경로의 PDF를 커밋했다.
[입력·PDF SHA와 Print 영수증](mcp-print-reference/receipts.json)을 보존한다.

## PR별 공개 검증

| PR | 입력 | 전체 쪽수 | Native 최저 | fresh WASM 최저 | 개별 리뷰 |
| --- | ---: | ---: | ---: | ---: | --- |
| #8895 | 2 | 4 | 98.00% | 97.94% | [개별 리뷰](../../archives/pr_8895_review.md) |
| #8897 | 22 | 28 | 97.18% | 97.18% | [개별 리뷰](../../archives/pr_8897_review.md) |
| #8900 | 4 | 4 | 95.91% | 95.08% | [개별 리뷰](../../archives/pr_8900_review.md) |
| #8916 | 1 | 1 | 99.75% | 99.72% | [개별 리뷰](../../archives/pr_8916_review.md) |
| #8918 | 1 | 1 | 99.76% | 99.73% | [개별 리뷰](../../archives/pr_8918_review.md) |
| #8920 | 4 | 8 | 100.00% | 100.00% | [개별 리뷰](../../archives/pr_8920_review.md) |

- [현재 Native 전쪽 비교 TSV](final-rebase-30fa7739/native-comparison.tsv) · [검증·이미지 해시](final-rebase-30fa7739/native-validation.json)
- [현재 fresh WASM 전쪽 비교 TSV](final-rebase-30fa7739/wasm-comparison.tsv) · [검증·이미지 해시](final-rebase-30fa7739/wasm-validation.json)
- [실제 초기화 응답과 ByPath API](final-rebase-30fa7739/browser-runtime-result.json) · [최소 글꼴 환경](final-rebase-30fa7739/minimal-font-verification.json)
- [현재 Native 편람 383쪽 gate](final-rebase-30fa7739/native-handbook-gate.json): 최저 표시값 99.96%, 공식 표시값 하락 0쪽.
- [Native 보호 18종·1771쪽](final-rebase-30fa7739/protected-native-validation.json) · [WASM 보호 18종·1771쪽](final-rebase-30fa7739/protected-wasm-validation.json): 현재 canonical로 전쪽 TSV를 다시 계산해 최저 98.36%, 하락·누락·추가 0쪽.

정본 재계산은 원 PNG의 SHA와 producer를 보존한다. Native 편람은 새 source에서 직접 출력했고,
나머지 정본 PNG의 render producer는 d66568ef다. renderer·입력·글꼴·exporter 소비 경로 동일성과
새 공개 캡처 368개 동일성을 근거로 재사용했다. [리베이스 소비 경로 감사](final-rebase-30fa7739/rebase-read-path-audit.md)를
연결하며, 이전 PNG를 새 source의 재캡처라고 표시하지 않는다. 기존 base에 없던 page-boundary-text의
2쪽은 0점으로 채우지 않고 내용·쪽 소속의 복원으로 구분했다.

## 실제 소비 경로와 지원 경계

실제 Studio 48개 강제 진단 캡처를 보존했다. 현재 hard-gate 집계는 0건이지만, 이 값만으로 내용
보존을 판정하지 않았다. CanvasKit 지원 캡처는 28개이며, 기존 미지원 RawSvg를 강제한 차트
4개는 차트가 없으므로 **CanvasKit 지원 통과에 포함하지 않는다**. 정상 auto 선택은 사전 검사로
Canvas2D에 전환한다. 양 모드 screen/print 4캡처에서 차트를 보존했고 두 Print는 각각 95.08%다.
기존 preflight·지원 판정·문턱을 바꾸지 않았다. [실제 소비 요약](final-rebase-30fa7739/actual-backends/summary.json),
[정상 auto 전환](final-rebase-30fa7739/actual-backends/chart-auto/validation.json),
[기존 지원 계약과 실행 회귀](final-rebase-30fa7739/actual-backends/chart-capability-scope.json)를 확인한다.
백엔드 간 report-only 픽셀 비교 32건 중 16건은 기존 문턱을 넘으며 픽셀 동등성을 주장하지 않는다.

## 메인터너 보정과 남은 범위

- #8897: 미공개 원본 3건은 확보하지 못했다. 사용자 지시로 공개 합성 6종을 만들고 각 원본의
  독립 desktop Print를 확보해 그림 전용 칸의 첫 opening·continuation·종료와 뒤 본문을 보정했다.
  미공개 원본을 직접 검증했다고 쓰지 않는다. 쌓인 단 구역의 줄간격 중복도 보정했다.
- #8920: 저장 줄 없는 빈 도형 문단의 줄 상자와 최종 셀 원점을 연결해 일반/병합 칸의 위/가운데
  정렬 4종·8쪽을 Native/fresh WASM 100.00%로 확인했다. 실제 분할 행의 별도 잔여까지 해결했다고 쓰지 않는다.
- #8916: 크기 0 원문을 잉크 없는 명령으로 보존하고 CanvasKit geometry 거부를 보정했다.
- #8918: 원문자·흰 원·체크의 원 전진과 원 프로그램을 유지했다. 사용자 요청 F0FA는 독립 Print의
  Wingdings bdash2와 385/1000 전진을 확인하고 U+1F8AD·기존 공개 Noto 프로그램으로 보완했다.
  private 글꼴 추출물은 Git에 넣지 않는다.
- #8895의 기존 도장 차이 등 별개 증상은 제목 글줄 이월 해결과 구분한다.
- #8900은 공개 값축 4종의 범위다. #8678의 다른 값 조합·작은 차트·눈금 잔여는 미완료이므로 OPEN을 유지한다.

이전 8fe7d1fb·d66568ef의 기록은 원인과 실행 출처 이력이다. 현재 검증은 final-rebase-30fa7739의
source·명령·입력/PNG hash와 각 결과를 기준으로 읽는다. Mac fresh WASM은 locked wrapper의
--target web --out-dir pkg --no-opt 로컬 대체 빌드이며 Docker 최적화 통과로 표시하지 않는다.
최종 로컬 게이트를 완료했고 개별 6개 리뷰와 PR 제출 자료를 확정했다. 원격 CI·병합·후속 처리 결과는 실제 완료 뒤 연결한다.


## 통합 PR #9025 code CI

code head `8d70de44b12e24ca8297f84e90337a2eb8e2977f`의 Full CI·CodeQL·Render Diff·Adapter·Proptest와 최종 정책 감사를 모두 통과했다. Archive A–E는 13,195 PASS / 0 FAIL / 45 skipped다. [원격 code CI 영수증](remote/pr9025-code-ci.json)에 정확한 head·base, 39개 check, workflow와 archive별 실제 summary를 보존한다. 이번 기록은 제품·검사·CI 정책을 바꾸지 않는 문서-only trailing이며, 새 trailing head의 필수 검사·병합은 별도로 확인한다. 원 PR 6개 리뷰를 유지하고 통합 번호만의 별도 리뷰 문서는 만들지 않는다.
