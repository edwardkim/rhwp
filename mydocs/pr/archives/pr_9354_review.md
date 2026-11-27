# PR #9354 각주·미주·머리말·꼬리말 누름틀 검토

**접수 당시 판정은 보완 필요였다.** 같은 자리에 있는 두 누름틀 중 고른 필드 하나만 입력을 받는 것을 네 영역에서 확인했다. 머리말·꼬리말의 문단 끝 인라인 표 뒤에 입력할 때는 반환한 글자 위치가 실제 삽입 위치와 어긋난다.

## 검토 범위

기여자 `semanticist21`, 리뷰어 `edwardkim`, 목표 브랜치 `devel`. 2026-11-24에 원 PR `fc4e2cf07a1439760d2cfb67c8230cb3140e2a78`와 기준 `2c0c1d205a54b8d5bd262241bac1391cffe0d801`을 확인했다. 단독 병합 tree는 `d27e2cfc7c90f8b3dd85b482d822e577da712943`이다. 원 PR의 Build & Test와 CI Impact Policy는 성공했다. 연결된 자동 종료 이슈는 없다. [원 PR](https://github.com/edwardkim/rhwp/pull/9354)

## 검증과 재현

- 기여자의 머리말 회귀가 통과했다. 메인테이너는 두 필드가 같은 위치에서 시작하는 HWPX를 별도로 열고, #9380에서 쓰는 `setActiveFieldAt`으로 둘째를 골라 한 글자를 넣었다. 머리말·꼬리말·각주·미주 모두 선택한 필드만 확장됐고, HWP·HWPX로 다시 연 8개 결과에서 필드별 범위가 유지됐다. [영역별 검사](../assets/semanticist21-9337-9354-9377/field-ownership.tsv)
- ‘앞뒤’ 사이에 두 누름틀이 있고 ‘뒤’ 다음에 인라인 표가 있는 머리말과 꼬리말에서 표 뒤 논리 위치 3으로 `X`를 입력했다. 실제 모델 텍스트는 ‘앞뒤X’로 `X`는 인덱스 2다. API는 다음 글자 위치 4를 돌려주었다. 그 반환값을 사용해 방금 친 한 글자를 지워도 ‘앞뒤X’가 남았다. 끝에 표가 없는 머리말에서 같은 검사는 ‘앞뒤’로 복원됐다.
- `insert_area_text`가 입력을 처리한 **뒤**의 글자 수로 요청 위치를 제한한다. 머리말·꼬리말 호출자는 반환값을 실제 삽입 위치로 사용해 종료 위치와 편집 이벤트를 만든다. 기존 #3216의 회귀는 문단 문자열 인덱스와 뒤쪽 인라인 개체를 포함한 논리 캐럿 위치가 다를 수 있음을 검사한다.

보완 범위는 원래 선택한 필드의 소유 관계를 유지하면서 실제 삽입 위치를 반환하고, 머리말·꼬리말의 끝 개체 뒤 입력과 복원을 확인하는 것이다.

### 입력 보완 뒤 발견한 인쇄 차이

머리말과 꼬리말에서 인라인 표 뒤의 논리 위치 3에 한 글자를 넣으면, 보완한 API는 글자 위치 2에 삽입하고 끝 위치 3을 반환한다. 실행 취소로 원문이 복원된다. 각주·미주를 포함해 선택한 누름틀 하나에만 글자가 들어갔고 HWP·HWPX 재열기에서도 범위를 유지했다.

한컴 2024에서 같은 입력을 인쇄하면 너비 41,952 HU의 인라인 표가 다음 줄을 차지하고, 앞줄의 ‘앞’과 ‘뒤’는 양끝에 놓인다. PDF의 두 글자 시작점은 85.03 pt와 500.25 pt다. Native 출력에서는 두 글자가 붙는다. 한컴에서 HWP로 저장하고 다시 HWPX로 변환한 문서에도 두 줄이 기록된다. [한컴과 Native 글자 위치 비교](../assets/semanticist21-9337-9354-9377/header-inline-print-native.png)

| 머리말·꼬리말 대조군 | 당시 devel Native | 입력만 보완한 Native |
|---|---:|---:|
| 누름틀 두 개와 줄 너비 표가 있는 머리말 | 66.93% | 66.93% |
| 같은 구조의 꼬리말 | 67.37% | 67.37% |
| 누름틀만 제거한 머리말 | 67.11% | 67.11% |
| 표를 제거한 머리말 | 100.00% | 100.00% |
| 표를 6,000 HU로 줄인 머리말과 꼬리말 | 각각 100.00% | 각각 100.00% |

2026-11-24에 독립 빌드한 devel `2c0c1d205a`와 보완 후보의 96 DPI 출력 PNG를 직접 대조했다. 위 여섯 입력 모두 두 빌드의 PNG SHA-256이 같다. 미달한 것은 줄 전체를 차지하는 표 앞 문장의 배치다. 누름틀 선택과 캐럿 반환은 별도로 검증했다.

## 메인테이너 보완 결과

WASM API는 삽입하기 전의 실제 글자 위치에 입력한 유니코드 스칼라 개수를 더해 끝 위치를 돌려준다. Studio가 그 반환값으로 입력 이력을 만들도록 수정했다. 각주·미주·머리말·꼬리말에서 같은 위치에 있는 두 누름틀 중 선택한 하나만 입력을 소유했고, 두 저장 형식에서 원문과 각 필드의 범위를 확인했다. [영역별 재열기 결과](../assets/semanticist21-9337-9354-9377/corrected-fields.tsv)

줄 배치에는 본문과 같은 인라인 표의 측정 결과를 쓴다. 문단의 어절·글자 단위 설정에서 허용한 줄 끝을 찾고, 한 줄 전체에 담길 수 없는 인라인 표 앞에서는 그 규칙에 따라 글자와 표를 배치한다. 너비가 없는 누름틀 경계와 숨은 안내문 길이를 포함한 저장 줄은 이 계산에서 처리하며, 동적 쪽 번호가 있는 머리말은 기존 치환 경로를 쓴다. 한컴에서 다시 저장한 HWP와 HWPX, 누름틀을 뺀 문서, 너비가 6,000 HU인 표와 표를 제거한 문서를 같은 독립 Print 절차로 확인했다. [입력·인쇄본·생산 버전](../assets/semanticist21-9337-9354-9377/sources-and-print.tsv)

넓은 표·좁은 표, 표·누름틀 제거 대조군, 한컴의 HWP·HWPX 재저장본을 포함한 10개 문서는 모두 1쪽이었다. 96 DPI로 한컴 인쇄와 비교한 결과 Native와 fresh WASM 모두 전쪽 100.00%였다. 넓은 표 앞의 ‘앞’과 ‘뒤’는 양쪽 끝에 놓이고 좁은 표는 글자 바로 뒤의 같은 줄에 배치된다. 실제 Chrome에서 머리말·꼬리말 4개 문서를 열어 직접 입력과 한글 IME 조합 입력을 실행했고, 실행 취소와 다시 실행 뒤 문자열과 캐럿을 함께 확인했다. [쪽별 실루엣](../assets/semanticist21-9337-9354-9377/focus-pages.tsv) · [머리말 확대](../assets/semanticist21-9337-9354-9377/trailing-header-comparison.png) · [꼬리말 확대](../assets/semanticist21-9337-9354-9377/trailing-footer-comparison.png) · [키보드·IME 회귀](../../../rhwp-studio/e2e/header-footer-inline-text-history.test.mjs)

### 본문의 어절 나눔과 줄 너비 표의 대조

줄 전체 너비의 표를 다음 줄에 배치하는 보완이 일반 본문의 ‘천원)’까지 한 줄 위로 옮기는 것을 기존 인쇄 회귀 `verified_overflow_spaces_and_styled_break_preserve_print_rows`에서 발견했다. 원 검사와 한컴 인쇄 PDF는 커밋 `72a9824975`에서 추적했다. 첫 보완 소스의 원본 1쪽 실루엣은 Native 9.14%, WASM 9.25%였다.

같은 문서에서 HWPX의 `breakNonLatinWord`만 바꾼 두 대조군을 한컴 2024로 직접 인쇄했다. 본문에서는 설정에 따라 ‘천원)’과 다음 표가 한 줄 차이로 놓였다. 머리말에서는 두 설정 모두 ‘앞’과 ‘뒤’가 줄 양끝에 놓였다. `a1bc2e5fe6`에서 글줄 구성과 인라인 배치가 같은 줄 끝 판정을 쓰도록 보완했다. 기존 본문 회귀를 그대로 실행해 통과했다.

최종 Native와 fresh WASM으로 두 경로에서 각각 본문 원본 9쪽과 한컴 2024에서 다시 인쇄한 네 설정 대조군 20쪽을 확인했다. 전쪽 실루엣은 모두 100.00%였다. ‘천원)’의 소속 줄, 바로 뒤 표의 윗변, 머리말의 글자 양끝을 같은 배율에서 대조했다. [입력·인쇄본](../assets/semanticist21-9337-9354-9377/additional-print-sources.tsv) · [쪽별 측정](../assets/semanticist21-9337-9354-9377/additional-pages.tsv) · [본문 줄 확대](../assets/semanticist21-9337-9354-9377/body-overflow-comparison.png)

## Merge 후 contributor PR comment 계획

원 기여 커밋을 포함한 통합 PR의 병합 SHA를 확인한 뒤, #9354에 감사 댓글을 게시한다. 머리말의 줄 너비 표 앞 1쪽을 한컴 인쇄에 대조한 아래 두 출력과 쪽별 TSV를 병합 SHA로 고정해 첨부한다. 표가 다음 줄로 넘어갔을 때 앞줄의 두 글자가 양끝에 놓이는지 비교한다. 네 영역에서 입력을 소유한 필드, 표 뒤의 한글 입력·실행 취소와 일반 본문 어절 나눔 결과도 알린다.

- Native Skia의 [전쪽 TSV](../assets/semanticist21-9337-9354-9377/page-tsv/target/native/trailing-header.tsv) · [인쇄 비교](../assets/semanticist21-9337-9354-9377/print-review/trailing-header-p001-native-review.png) · [겹쳐 보기](../assets/semanticist21-9337-9354-9377/print-review/trailing-header-p001-native-overlay.png)
- 브라우저 Canvas2D의 [전쪽 TSV](../assets/semanticist21-9337-9354-9377/page-tsv/target/wasm/trailing-header.tsv) · [인쇄 비교](../assets/semanticist21-9337-9354-9377/print-review/trailing-header-p001-wasm-review.png) · [겹쳐 보기](../assets/semanticist21-9337-9354-9377/print-review/trailing-header-p001-wasm-overlay.png)

## 최신 devel 병합 후 원 기여와의 최종 대조

원 PR 최종 커밋 `fc4e2cf07a1439760d2cfb67c8230cb3140e2a78`에서 Native와 브라우저용 WASM을 각각 빌드했다. 원 PR 본문과 변경 파일에 시각 TSV가 없어 같은 원본·한컴 인쇄본·글꼴·96 DPI로 측정했다. 기준 `e961a78771`을 병합한 후보 `ff9194594e`과 쪽별로 대조한 20행에서 12행이 개선됐고 8행은 같은 일치율을 유지했다. 누락과 하락은 0행이었다. [원 커밋·기준·후보의 쪽별 측정과 PNG 해시](../assets/semanticist21-9337-9354-9377/original-candidate.tsv) · [전체 검증 결과](../assets/semanticist21-9337-9354-9377/local-checks.json)

같은 Chrome 156에서 두 WASM 파일을 각각 불러와 실제 편집을 비교했다. 머리말과 꼬리말의 끝 표 뒤에 X를 넣었을 때 원 기여의 반환 위치는 4였고, 그 위치로 취소하면 X가 남았다. 후보는 3을 반환했고 두 영역 모두 원래 ‘앞뒤’가 복원됐다. [두 실행 파일의 해시와 편집 결과](../assets/semanticist21-9337-9354-9377/local-checks.json)
