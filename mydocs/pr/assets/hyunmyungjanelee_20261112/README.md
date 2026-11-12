# hyunmyungJaneLee 6개 PR 통합 검토 증적

현재 후보 소스는 `8fe7d1fb8bc5024d0377ff7aa37d98c78e8f071a`, 검증 base는
`80b1c354c29b7a02fb2b3c14a274da7f5b653ada`다. Native 공개 검증은 34종·46쪽 전체 90% 이상이며,
같은 입력·PDF의 devel 대비 2%p 초과 하락은 없다. 단 구역 간격 보정의 최소 글꼴 집중 검사 4건은
모두 통과했다. 최소 글꼴 Native 집중 검사 126건·공개 fresh WASM 34종 46쪽과 정본 Native 18종 1,771쪽도 완료했다.
최소 글꼴 [전체 회귀](final-print-8fe7d1fb/full-nextest-validation.json)는 **13,356 PASS / 0 FAIL / 45 skipped**(774.340초), fmt는 PASS다. Native·WASM·workspace all-target Clippy와 workspace build도 모두 PASS다. 정본 fresh WASM 18종·1,771쪽도 공식 표시값 하락·누락·추가 0쪽이다. 추가 Native Skia·suite/단위 정책과 실제 Studio 소비 경로는 현재 이 소스로 검증 중이다. 이전 head의 통과 결과로 이 후보의 완료를 대신하지 않는다.

## 기준 PDF와 검증 입력

원 PR 6개는 본문에 웹한글기안기 PDF를 기준이라고 명시했다. 그 PDF와 Cairo 기반의 이전 점수는
현재 판정에서 제외했다. 동일 입력의 **Windows 데스크톱 한컴 Print PDF 34종**을 보존한다.
MCP Print는 33종이며, 실제 출력 제품별로 한컴 2020 11.0.0.9136이 25종,
한컴 2024 13.0.0.3901이 8종이다. 나머지 1종은 사용자가 제공하고 Print 경로라고 확인한
한컴 2024 출력이다. 저장 제품 연도를 Print 제품 연도로 바꾸어 쓰지 않는다.

- [공식 Print 영수증](mcp-print-reference/receipts.json): 입력·이전 PDF·현재 PDF 해시, MCP 작업과 실제 제품·버전·출력 방식, 수동 Print 확인.
- [체리픽 적용 기록](cherry-pick-result.json) · [충돌 해소 기록](conflicts.json): 원 PR head와 실제 누적 적용 순서.
- [정본 Native 보호](final-print-8fe7d1fb/protected-native-validation.json) · [정본 fresh WASM 보호](final-print-8fe7d1fb/protected-wasm-validation.json): 각 18종·1,771쪽, 공식 표시값 하락 0쪽.
- [현재 fresh WASM 전쪽 비교](final-print-8fe7d1fb/wasm-comparison.tsv) · [빌드·실제 초기화 응답](final-print-8fe7d1fb/wasm-validation.json) · [대표 review·overlay 직접 판독](final-print-8fe7d1fb/wasm-direct-reading.json).
- [현재 Native 전쪽 비교](final-print-8fe7d1fb/native-comparison.tsv) · [생산 SHA·소스 동일성·PNG 해시](final-print-8fe7d1fb/native-validation.json).

기존 base 출력에 없던 `page-boundary-text`의 2쪽은 0점으로 채우지 않고 구조 복원으로 구분했다.
원본·PDF·판정 문턱을 바꾸어 점수를 맞추지 않았다.

## PR별 공개 검증 범위

| PR | 입력 | 전체 쪽수 | Native 최저 | fresh WASM | 개별 리뷰 |
| --- | ---: | ---: | ---: | --- | --- |
| #8895 | 2 | 4 | 98.00% | 97.94% | [리뷰](../../archives/pr_8895_review.md) |
| #8897 | 22 | 28 | 97.18% | 97.18% | [리뷰](../../archives/pr_8897_review.md) |
| #8900 | 4 | 4 | 95.91% | 95.08% | [리뷰](../../archives/pr_8900_review.md) |
| #8916 | 1 | 1 | 99.75% | 99.72% | [리뷰](../../archives/pr_8916_review.md) |
| #8918 | 1 | 1 | 99.54% | 99.50% | [리뷰](../../archives/pr_8918_review.md) |
| #8920 | 4 | 8 | 100.00% | 100.00% | [리뷰](../../archives/pr_8920_review.md) |

## 메인터너 보정과 수용 경계

- #8897: [그림 전용 칸 높이](picture-cell-repair/cause.md)는 사용자 지시에 따라 공개 합성 6종으로 대조했다. 미공개 원본 3건을 직접 검증했다고 쓰지 않는다. [단 구역 후행 간격](stacked-zone-spacing-repair/cause.md)은 실제 3 PASS / 1 FAIL → 4 PASS, Native 99.06% → 100.00%로 복원했다.
- [명시적 글리프 공급 순서](canvas-fallback-selection/cause.md): 보호 sample16의 원 글꼴 윤곽을 설치 별칭이 가로채던 공통 Canvas 경로를 보정했다. 새 head의 정본 Native/fresh WASM 전쪽에서 확인한다.
- #8920: [저장 줄 없는 도형 문단의 줄 상자](partial-table-line-repair/cause.md)를 복구하여 일반·병합 칸의 위·가운데 정렬 4종 Native 8쪽은 모두 100%다.
- #8895의 기존 도장 차이는 원 메인터너의 제한된 수용 범위를 유지한다. 제목 이월 개선이 도장 이월까지 해결했다고 쓰지 않는다.
- #8900은 공개 4종의 값축 시작값·눈금 간격 범위다. 차트 전체 피델리티나 모든 데이터 조합의 해결로 확대하지 않으며 #8678은 OPEN을 유지한다.
- #8916은 MCP 2020의 문서 열기 거절과 사용자 제공 2024 Print를 구분한다. #8918의 범위 밖 F0FA를 해결했다고 쓰지 않는다.

## 최종 검증과 증적 역할

정본 보호는 [공식 보호 TSV](../visual-protected-baseline/final.tsv)의 두 자리 표시값을 페이지별로
대조한다. 18종·1,771쪽의 Native와 fresh WASM을 각각 확인하고, HWPX 편람의 Native
`--handbook-gate`도 실제 실행한다. 일반 공개 검증의 90%·devel 대비 2%p 기준과 이 특별 보호
기준은 서로 대체하지 않는다.

최소 글꼴 회귀는 Mac 사용자 글꼴을 변경하지 않는 sandbox에서 실행한다. 현재 head에서
CoreText 528 face가 모두 OS 경로임을 확인했고, 사용자 글꼴 목록 1,701 face는 검증 전후 동일하다.
이 격리 사실을 브라우저의 OS CSS 글꼴 격리라고 확대하지 않는다. Mac fresh WASM은 locked wrapper의
`--target web --out-dir pkg --no-opt` 로컬 대체 빌드로 기록하며 Docker 최적화 성공으로 쓰지 않는다.

`final-print-8fe7d1fb/`가 현재 후보의 공개 비교 자료다. 그 밖의 과거 head·Cairo 진단 자료는 원인
추적 이력이며 현재 수용 근거가 아니다. 각 입력의 같은 페이지 raw·독립 PDF PNG·review·overlay를
함께 보존한다. 최종 CI·통합 PR·원 PR 및 이슈 후속 처리는 검증 완료 뒤 확정한다.
