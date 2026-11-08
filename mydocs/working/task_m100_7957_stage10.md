# Task #7957 10단계 — 연속 텍스트 보정본 검증과 색상 회귀

runtime source는 `443a7c0acd1bec14cf9af7906c3b14b642d705ce`로 고정했다. 입력 보존 commit
`1c5f5ce5d`는 실행 코드·글꼴 규칙·생성 bindings를 바꾸지 않는다. 이 단계의 소스가 바뀌면
이전 producer의 검증을 재인용하지 않는다.

## 실패 원인과 보정 계약

9단계의 단어 간격 검사 FAIL은 같은 줄의 연속 Tj text object를 각각의 paint 소속으로
분리한 데서 발생했다. 기존 문자열·줄·상대 너비·두 저장 형식의 기대값을 유지한 채
연속 text group을 도입했고 같은 정식 검사가 PASS했다. path/image mark와 Form 진입·종료는
group을 나눈다. 최종 개체 z-order는 group 번호가 아니라 첫 원본 operator의 계층적 순서다.
가려진 텍스트를 삭제하지 않으며, Form·그림·도형을 넘어 단어를 합치지 않는다.

실제 검출은 `paint-order-20261017/word-gaps.log`(FAIL),
`word-gaps-group-fix-checked.log`(PASS)에 보존한다. 별도의 가림/Form 관계 검사도 새 runtime에서
PASS했다. 색상 실루엣이 같은 내부의 잘못된 글자색을 잡지 못하는 경계는 9단계의 직접 PNG
판독과 구분한다.

## producer와 입력

공유 target은 기본 작업공간의 `target/pr-review` 하나다. 위 집중 nextest가 생성한 실제
Native Skia CLI를 별도로 보존했으며 불필요한 Native 재빌드는 하지 않았다.
CLI SHA-256은 `c9670eaf073a643854a28dca7fb2d04bdcabedc33faa3f4d5dea2efb4cfcd5fc`다.
Docker 빌드는 표준 `-p rhwp`, jobs 8, release codegen-units 16을 사용한다.
모든 로그·전쪽 TSV·producer·입력 목록은 ignored
`output/pr-review/issue7957/paint-group-20261017/`에 둔다.

사용한 실물·16 MiB 생성 입력은 Git blob과 실행 파일의 해시를 대조한다. 원본
`bounded-import-path.pdf`의 기존 offset 0 xref 항목에는 qpdf warning(exit 3)이 있으며,
정규화·복구한 다른 파일로 바꾸지 않았다. 나머지 신규 보존 입력의 qpdf 검사는 exit 0이다.
64 MiB 입력은 50 MiB 이상 로컬 보관 규정에 따라 저장소 밖의
`/home/edward/rhwp-local-pdf/issue7962/`에 보존한다. Windows font bytes와 환경 비밀은 커밋하지 않는다.

## 자원 정책의 독립 검증

Native 공개 CLI에서 16 MiB 직전·정확한 경계·직후와 64 MiB 직전·정확한 경계는 성공했다.
64 MiB 초과는 `PDF_RESOURCE_LIMIT`로 거절됐고 후속 정상 요청도 성공했다.
`size-policy.json`과 실제 stdout/stderr를 보존한다. 이 결과는 16 MiB 생성본의 글꼴 일치나
전체 parser 할당 증명을 뜻하지 않는다. 사용자가 승인한 자원 전용 범위와 Helvetica 후속
백로그 B1을 유지하며 `parserAllocationBoundsProven=false`다.

## 최종 검증 상태

Native 360쪽을 산출했다. 자원 전용 1쪽을 제외한 359쪽은 raw 90% 이상이고 쪽수 누락이 없다.
신규 원본 19쪽의 최저값은 99.31446%, 기존 operators 12쪽은 90.51927%다.
고정 devel 대비 공통 대조군의 0.01 percentage point 이상 하락은 없다.
fresh Docker WASM·확대 판독·필수 회귀·lint는 실행 종료와 반환 코드를 확인한 뒤 기록한다.
현재 상태를 PR 등록 완료·CI 성공·merge·이슈 종료로 표시하지 않는다.

### fresh WASM과 직접 판독 완료

Docker 빌드가 exit 0으로 끝났고 pkg와 Studio public의 JS/WASM bytes가 동일했다.
WASM SHA-256은 `276acabfcc76fa13a190492f30672db46c4f04f0b0b8b8d7debda76845d9bae5`다.
CDP에서 실제 HTTP 응답 bytes도 이 해시와 일치했으며 Studio는 원본 19쪽을 표시했다.

전쪽 360쪽을 양쪽에서 측정했다. 자원 전용 1쪽을 제외한 14개 문서·359쪽은 양쪽 모두
raw 90% 이상이고 원본 PDF와 쪽수가 일치한다. 전 범위 최저값은 Native 90.14458%,
WASM 90.16683%다. 신규 19쪽은 Native 99.31446%, WASM 99.40231%이며, operators는
Native 90.51927%, WASM 90.17588%다. 공통 지원 대조군은 Decimal 비교에서 0.01%p 이상
하락이 없다. 16 MiB 생성본은 Native 73.54167% / WASM 78.19253%로 별도 보존하며,
해당 silhouette 명령의 exit 1을 지우거나 시각 통과로 바꾸지 않는다.

원본 6쪽의 원형·세로 연결선과 가려진 영문, 7쪽의 굵은 face·그림 경계, operators 12쪽
표제·본문 간격을 같은 영역의 Native/PDF/WASM으로 직접 비교했다. 알려진 Type3 원 glyph
차이는 전 단계의 미완료 범위로 유지하고, 완전한 glyph 복원을 주장하지 않는다.
수식·분할·쪽수가 원래 다른 입력을 높은 평균으로 면제하지 않았다.

새 CLI 검사 두 개는 실제 이전 CLI로 실행하면 각각 붉은 stroke 누락,
`rectangle/image/FIRST/SECOND`의 잘못된 순서로 FAIL(exit 101)한다. 같은 검사와 같은
fixture가 현재 CLI에서는 PASS한다. 이전 CLI를 실행한 뒤 현재 reader로 그 HWPX를
판독하는 방법이며, 현재 library를 이전 코드라고 가장하지 않았다. binary·fixture hash와
실패 assertion은 `negative-control-results.json`, `negative-*.log`, `cli-positive.json`에 있다.
단어 간격 검사도 기존 FAIL/PASS 근거를 유지한다. 기준값이나 기존 기대값을 완화하지 않았다.

공개 WASM API/Worker는 92 assertion, 실제 Worker 6개를 통과했다. 원본 19쪽과 33회 반복
배치 및 affine clip의 연결, 기존 12쪽의 전쪽 렌더·양식별 저장·재열기·그림 보존과 실패 후
복구를 검사했다. 이 구조 검사를 시각 판정 대신 사용하지 않았다.

비교 화면은 `/mnt/e/tmp/issue7966-paint-group-review-20261017/index.html`이며,
Git에 포함한 대표 PNG는 `mydocs/pr/assets/issue7957-20261017/`다. 직전 bba967e7의
결과·비교 화면과 혼동하지 않도록 별도 경로에 보존했다.

### 확대 판독에서 확인한 글자색 회귀 — 완료 보류

직접 확대 판독에서 `bounded-import-path.pdf` 1쪽의 `Website:`는 검정,
`https://ostday.kr`는 파랑이어야 하지만 연속 text group이 두 paint style을 합친 것을
확인했다. bba967e7의 URI는 파랑이었고 443a7c0ac에서는 검정이었다. 독립 Poppler SVG의
원본 색은 RGB `(5,99,193)`(`#0563c1`)이다. 높은 실루엣 점수로 면제하지 않는다.

이 판독을 필수 전체 검증 시작 전에 끝내지 못한 것은 작업 순서의 오류였다.
소유한 nextest만 SIGINT로 중단했고 8,290/10,618건 중 8,274 PASS, 실행 중이던 16건은
SIGINT 종료였다. 이는 새 구현 결함 16건의 재현이 아니며 전체 검증 통과로 세지 않는다.
로그와 `website-before-current.png`, 독립 PDF의 `independent-page1.svg`를 보존했다.

그리기 순서 group만 같다는 이유로 style이 다른 span을 하나의 CharShape에 넣어서는
안 된다. `TextSpan`의 font·size·weight·italic·color·spacing·scale·rise·방향을 보존하는
동일 paint style에서만 기존 단어 간격 복원을 허용한다. 검색용 reading-order 경로의
기존 glue 정책은 바꾸지 않는다. 별도 style 분기를 문서·URI·좌표로 한정하지 않는다.
이 보정과 URI의 IR/SVG·두 저장 형식 및 기존 가림·동일 style 단어 간격 반례를 확인한 뒤
새 Native/fresh Docker WASM 전쪽 판독을 먼저 끝내고 전체 회귀를 다시 실행한다.
실행 자료는 ignored `paint-style-20261017/`로 구분한다. 이 단계의 TSV·PNG를 새 source의
최종 결과로 바꾸어 쓰지 않는다.
