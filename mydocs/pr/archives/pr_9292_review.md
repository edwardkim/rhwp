# semanticist21의 편집 기능 PR 17건 검토

2026년 11월 21~22일, 리뷰어가 `edwardkim`인 원 PR 17건을 검토했다. 기여자가 갱신한 #9220과 #9273을 포함해 원 head 17개를 병합 이력에 보존했다. 최신 기준은 #9277과 #9283을 포함한 `d7b5286cbf6401678a70eed3d66236a68dd2a72c`이다. 공통 제품 소스 `521f69d19da9176e7947ceb318cf9838bec5ca75`에서 원 사례와 새 기준의 대조군을 다시 실행했다. [원 PR별 head와 리뷰어](../assets/semanticist21-20261121/reviewed-heads.tsv)

17건 모두 통합 후보로 수용할 수 있다. 원 PR별 판정과 보완한 내용은 아래 검토 문서 첫머리에 기록했다.

## 기능별 검증

| 원 PR | 메인테이너가 확인한 동작 |
| --- | --- |
| [#9211](pr_9211_review.md) | 머리말 영역 밖으로 넘친 표의 마지막 칸을 눌러 머리말 셀의 경로를 얻었다. |
| [#9215](pr_9215_review.md) | 감추기를 잘못된 셀로 옮기면 오류가 나고 기존 머리말 감추기와 속성이 보존된다. |
| [#9220](pr_9220_review.md) | 캡션의 줄바꿈·아래 여백·셀·머리말·캡션 속 그림을 여덟 개 저장본으로 검증했다. |
| [#9222](pr_9222_review.md) | 존재하지 않는 그림 자원 ID 177을 거절하고 HWP·HWPX로 저장해 다시 열어도 잘못된 참조가 생기지 않는다. |
| [#9224](pr_9224_review.md) | 제목을 고쳐도 확장 XML과 CDATA가 남고, 잘못된 날짜는 다른 필드가 바뀌기 전에 거절된다. |
| [#9229](pr_9229_review.md) | 공개 TypeScript 선언에서 새 API 17개를 호출했다. 셀·글상자 안 직선의 끝점은 HWP·HWPX로 두 번씩 저장해 다시 연 뒤에도 보존된다. |
| [#9230](pr_9230_review.md) | 캡션 번호 자리표시자를 지우고 HWP·HWPX로 두 번씩 저장한 후에도 뒤 문구가 남는다. |
| [#9231](pr_9231_review.md) | 그림을 끈 편집 화면에는 자리표시선이 표시되고, 같은 상태에서 인쇄하면 원래 그림이 나온다. |
| [#9234](pr_9234_review.md) | 너비가 다른 세 단의 본문과 각주 참조가 처음 열기·편집·두 형식 다시 열기에서 한컴과 같은 줄에 남는다. |
| [#9235](pr_9235_review.md) | 활성 필드 앞 감추기를 지운 뒤에도 글자가 같은 필드에 입력된다. |
| [#9238](pr_9238_review.md) | 편집 화면에서 두 글자 사이의 조판 부호를 켜고 끈 두 상태의 Print SVG가 바이트 단위로 일치한다. |
| [#9241](pr_9241_review.md) | 그룹 도형 크기를 바꾼 HWP와 HWPX를 다시 열어 한컴 인쇄본과 대조했다. |
| [#9246](pr_9246_review.md) | 다단 각주의 다음 단 이월과 오른쪽 단의 가용 폭을 원본·편집본의 모든 쪽에서 대조했다. |
| [#9265](pr_9265_review.md) | 본문의 인접한 두 수식 사이에 붙여넣은 글자가 두 수식 사이에 남는다. |
| [#9270](pr_9270_review.md) | 셀 HTML 붙여넣기에서도 같은 캐럿 경계와 수식·글자 순서가 보존된다. |
| [#9271](pr_9271_review.md) | 글자가 없는 펼침과 포갬 상태를 각각 HWPX로 저장해 두 인쇄 결과를 확인했다. |
| [#9273](pr_9273_review.md) | 70줄로 편집한 중첩 표가 한컴과 같은 21·58·70줄에서 끊긴다. CBTA 문서 59쪽의 뒤 문단도 354.0px에 위치한다. |

## 원 사례와 편집·저장 후 재열기

각 입력과 한컴 Print PDF의 SHA-256을 대조한 후 같은 원본을 Native Skia와 새로 빌드한 WASM Canvas에서 열었다. 일치율은 96 DPI에서 2px 이내의 내용을 비교한 색상 경계 보정 전 실루엣 값이다. 저장된 파일을 열기 전의 실시간 편집 상태도 별도로 측정했다. [입력·독립 PDF·렌더링 이미지의 쪽별 근거 114행](../assets/semanticist21-20261121/validation/final/feature-pages.tsv)

| 검증 대상 | 백엔드별 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| 다단 각주 원본과 편집·저장본 | 24 | 99.25640% | 99.25632% |
| 그림 캡션 | 8 | 99.56569% | 99.56616% |
| 중첩 표 편집과 저장 재열기 | 21 | 98.06894% | 98.06533% |
| 크기를 바꾼 그룹 도형 | 2 | 100.00000% | 100.00000% |
| 빈 글자 겹침 | 2 | 100.00000% | 100.00000% |

#9234의 단별 각주 원본에서 왼쪽과 오른쪽의 마지막 줄 기준선이 같아졌다. 오른쪽 두 각주 사이에는 원본에 저장된 283 HWPUNIT 간격이 있다. 같은 조건에서 Native와 WASM 모두 이 원본의 1쪽 일치율이 97.47%에서 100.00%가 됐다. [각주별 좌표와 같은 영역의 확대 비교](../assets/semanticist21-20261121/footnotes/README.md)

Native 실행 파일의 SHA-256은 `d3fcb8f43cecae77530c38ef29968d9d120fad7751604b348d898f4ce0b056b4`, 브라우저가 실제로 받은 최적화 WASM은 `4327fe4dfdee45a5fdb64b9638ffcc5ee10e4904f5fdd5cfc60b60d7e4567bcb`다. 편람 HWPX Native 383쪽의 최저는 99.97%이고, 382쪽은 100.00%였다.

## 보호 문서의 기존 미달 53행

보호 4,046행 전부 기존 공식 하한에 도달했다. 하락 0건, 누락 0건, 상승 75건이다. 형식 사양서 10·71쪽은 원문의 아래아가 복원됐고, 11쪽의 현대어 ‘한글’은 보존됐다. [전쪽 판정과 확대 근거](../assets/semanticist21-20261121/validation/final/README.md)

형식 사양서의 Native 3행과 WASM 11행, 화학물질 표시 기준의 WASM 39행이 처음 측정에서 공식 하한보다 0.01~0.07%p 낮았다. 독립 인쇄본과 글꼴 윤곽을 대조해 원본 글리프의 전진·잉크 위치, WMF 인쇄 메트릭, 저장 탭과 캡션 공백, 종이의 인쇄 좌표를 보완했다. 형식 사양서 71쪽과 화학 문서 103쪽을 두 백엔드에서 다시 렌더링한 결과 53행 모두 하한을 충족했다. [보완 전후 348행과 이미지 해시](../assets/semanticist21-20261121/validation/final/devel-protected-pairs.tsv) · [원인별 독립 근거](../assets/semanticist21-20261121/validation/README.md)

## 최신 devel의 기능 보존

#9277에서 반영한 문서 640쪽, CBTA 204쪽, #9283의 조판 수정 215쪽을 각각 최신 devel과 후보에서 다시 열었다. Native와 fresh WASM에서 각각 1,059쪽의 렌더 트리가 모두 같고, 새로 들어온 표 분할·그림 배치·글꼴 선택 기능이 보존됐다. 일반 대조군의 독립 Print 대비 최저는 98.21527%와 98.21930%다. #9209의 17~23쪽에는 기존부터 한컴 Print와 표의 왼쪽 괘선이 약 1.9px 어긋난 부분이 있다. 이번 보완으로 바뀐 텍스트의 인쇄 배율은 한컴 PDF의 행렬과 일치한다. [쪽별 대조와 실제 그리기 명령](../assets/semanticist21-20261121/validation/final/rebase-inspection/README.md)

## 원 PR에서 언급한 이슈

2026년 11월 22일 0시 31분(KST)에 GitHub에서 다시 확인한 원 PR 17건의 자동 종료 연결은 0건이었다. 본문이 참조한 이슈 중 수식 선택, 묶음 개체 setter, 긴 각주 겹침 건은 이미 종료됐다. [#595](https://github.com/edwardkim/rhwp/issues/595) · [#6740](https://github.com/edwardkim/rhwp/issues/6740) · [#9014](https://github.com/edwardkim/rhwp/issues/9014)

#9241의 대조군과 관련된 개체 속성 저장·Undo 항등성 이슈는 계속 열려 있다. [#6806](https://github.com/edwardkim/rhwp/issues/6806)

## 최종 로컬 검사

최소 글꼴 환경의 Rust 전체 회귀 13,946건, Native 공식 검사 4,071건과 Studio 2,822건을 통과했다. 편람 HWPX Native 383쪽의 최저 raw는 99.97%이고, 382쪽은 100.00%다. [검사별 실행 결과와 검증한 바이너리](../assets/semanticist21-20261121/validation/final/checks.json)

## 중첩 표의 저장 형식별 검증

원본 HWP와 한컴에서 다시 저장한 HWPX에 각각 30줄·70줄을 입력했다. 편집 직후와 HWP·HWPX 재열기의 전 42쪽씩을 한컴 Print와 비교한 최저값은 Native 98.00640%, fresh WASM 97.86924%다. 3쪽은 58번째 줄과 아래 괘선을 포함하며 4쪽은 59번째 줄에서 시작한다. [전쪽 수치와 실제 파일](../assets/semanticist21-20261121/nested-row-split/edited/README.md) · [200개 캐럿과 저장 원점](../assets/semanticist21-20261121/nested-row-split/edited/caret-and-reopen.tsv)

## 각주와 편람의 최종 대조

원 HWP와 각각 HWP·HWPX로 저장해 다시 연 문서를 한컴에서 인쇄했다. 세 상태 모두 2쪽 첫 단에서 세 번째 문단과 참조 번호가 함께 보이고, 이월된 각주 본문이 원래 순서대로 이어졌다. Native·fresh WASM 각 6쪽 모두 raw 100.00000%였다. [각주 본문과 참조](../assets/semanticist21-20261121/footnotes/right-column-body.tsv)

편람 43쪽의 옛한글에서 문서가 지정한 한컴바탕의 글리프와 자간을 적용했다. 한컴의 내장 글꼴과 실제 HWPX Canvas 윤곽 5개·선분과 곡선 84개가 일치했다. HWP와 HWPX 모두 43쪽의 Native·fresh WASM raw는 100.00000%다. [해당 글리프의 독립 인쇄 근거](../assets/semanticist21-20261121/validation/final/handbook-old-hangul.json)

## 보호 하한 갱신 후 확인

최신 devel 보호표의 4,046행 중 실측값이 오른 75행을 새 하한에 반영했다. 갱신한 보호표로 편람 383쪽과 공개 문서 4종의 Native·WASM 8개 조건을 다시 렌더링해 통과했다. 최종 테스트 묶음을 다시 생성한 뒤 Rust 회귀 13,946건을 재실행했다. 묶음을 바꾸기 전후로 실행한 테스트 이름과 횟수가 모두 일치했다. 기본 실행에서 제외된 45건은 별도 집계했다. [이전값과 새 값 및 재검증 결과](../assets/visual-protected-baseline/updates/semanticist21-20261121.json)

## 병합 결과

2026년 11월 22일 02시 15분(KST)에 [통합 PR #9292](https://github.com/edwardkim/rhwp/pull/9292)의 검증 커밋 `f0f9fc76b4306aef9571467e1a3453eee6e68ff4`을 `devel`에 병합했다. 원 PR의 기여 커밋 17개가 모두 병합 커밋의 조상이다. 병합 직후 GitHub에서 원 PR 17건의 MERGED 상태를 확인했다. [병합 커밋 `0adeca439f`](https://github.com/edwardkim/rhwp/commit/0adeca439f1481e7245a344b7713d7d563df2ace) · [셀프 리뷰](https://github.com/edwardkim/rhwp/pull/9292#pullrequestreview-6006870656)

필수 빌드와 테스트, 보안·렌더·호환성 검사 및 정책 감사가 성공했다. [CI](https://github.com/edwardkim/rhwp/actions/runs/44925877146) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44925877063) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44925877031) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44925877097) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44925877084) · [CI 정책](https://github.com/edwardkim/rhwp/actions/runs/44928812387)

병합 뒤에는 테스트 실행 시간 자료를 확인했다. GitHub가 이 PR의 B~E 작업에서 실행 시간 자료 4개를 수집해 [통계 커밋 `b701464878`](https://github.com/edwardkim/rhwp/commit/b7014648781e3f384cc0fde80bec54c621e34645)에 게시했다. [메타데이터 실행](https://github.com/edwardkim/rhwp/actions/runs/44929059485)

본문이 참조한 이슈 3건은 이미 닫혀 있다. [#595](https://github.com/edwardkim/rhwp/issues/595) · [#6740](https://github.com/edwardkim/rhwp/issues/6740) · [#9014](https://github.com/edwardkim/rhwp/issues/9014)

개체 속성 저장과 Undo의 잔여 범위를 추적하는 이슈는 계속 열려 있다. [#6806](https://github.com/edwardkim/rhwp/issues/6806)
