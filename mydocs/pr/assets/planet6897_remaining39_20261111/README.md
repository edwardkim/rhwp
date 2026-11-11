# planet6897 원 PR 39개 체리픽 통합 검토

**로컬 검토 완료: 메인터너 보정 후 원 PR 주장 페이지와 기존 보호 문서 기준을 충족합니다.**

## 제품 코드와 범위

- 브랜치: `review/integration-20261111-planet6897-open39`.
- 실제 새 빌드·검증 제품 코드: `3575d814f44053483bb51f54b01dcf8044d0b575`. 증적·문서 커밋의 head와 구분합니다.
- 고정 base: `4c876654d86d071d342fc78706c51cfff6ddbde4`.
- 동결한 원 PR 39개에서 적층 중복을 제외한 102개 커밋을 실제 체리픽했습니다. 이후 공통 경로의 메인터너 보정과 최신 devel 리베이스를 반영했습니다. 원 contributor의 브랜치와 head는 변경하지 않았습니다.
- 사용자 지시에 따라 원 PR의 기능·주장 페이지와 기존 보호 문서로 범위를 제한합니다. 원 PR 범위 밖의 기존 미달을 전부 개선했다고 주장하지 않습니다. [범위 기록](maintainer-repair/final-scope.md).
- [리베이스 이력·102개 체리픽의 SHA 대응](maintainer-repair/rebase-reuse.json)과 [최신 원 PR 39개 head 대조](maintainer-repair/latest-head-3575d814/source-head-check.json)를 확인했습니다.

## 최신 HEAD에서 완료한 검증

- Mac 최소 글꼴 전체 nextest: **13,312 PASS / 0 FAIL / 45 skip**, 452.943초. [결과](maintainer-repair/latest-head-3575d814/full-nextest.json).
- 원 PR별 42개 모듈·77개 함수가 최신 전체 회귀에서 실제 PASS입니다. [함수별 실행 대조](maintainer-repair/latest-head-3575d814/individual-tests.json).
- fmt·Native/WASM/all-target Clippy·workspace build·Skia lib 4,034 PASS / 12 ignore·PNG 2 PASS·PDF 35 PASS·manifest/unit tiers 등 필수 게이트 12개 모두 통과했습니다. [게이트 기록](maintainer-repair/latest-head-3575d814/rust-gates.json).
- 원 PR 주장 페이지: Native와 fresh WASM 각각 **44입력 조건·88쪽**, 총 176개 페이지 증적입니다. 양쪽 최저 **90.79%**, 90% 미만·측정 누락 0쪽입니다. 해당 입력의 전체 쪽수도 독립 Print PDF와 일치합니다. [증적·TSV 색인](maintainer-repair/latest-head-3575d814/claims/index.json).
- 보호 문서: Native와 fresh WASM 각각 **18조건·1,771쪽**, 총 **36조건·3,542쪽**에서 정본 TSV 공식 두 자리 표시값 하락 **0쪽**입니다. 필수 편람 HWPX Native 383쪽 `--handbook-gate`도 통과했습니다. [보호 비교와 원본 TSV](maintainer-repair/latest-head-3575d814/protected-comparison.json).
- 반복 저장은 4입력의 편집·저장·재열기·재저장·재열기, **총 8단계**를 실행해 종료 코드·verify·쪽수·삽입 문자열 1회 보존을 확인했습니다. [단계별 결과](maintainer-repair/latest-head-3575d814/repeated-save.json).
- 리베이스로 새로 들어온 행 나눔·선택 기능의 7개 검사도 실제 PASS입니다. [기능 보존 결과](maintainer-repair/latest-head-3575d814/incoming-feature-tests.json).
- 원 #8930의 실패에 대응하는 Linux Canvas CI 대조군 6개를 같은 기본 픽셀·채우기 순서 기준으로 통과했습니다. 최신 Mac WASM을 실제 초기화한 응답 해시까지 연결했습니다. [실제 소비 결과](maintainer-repair/latest-head-3575d814/linux-canvas.json).

## 출력·글꼴·직접 판독의 구분

- 최신 HEAD의 Native와 fresh WASM을 새로 빌드해 출력했습니다. WASM은 Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화 결과가 아닙니다. [빌드와 root/Studio 해시](maintainer-repair/latest-head-3575d814/producer.json).
- [실제 초기화 JS/WASM 응답](maintainer-repair/latest-head-3575d814/app-initialization.json)을 확인했습니다. WASM은 원본과 바이트가 같고, JS의 실제 응답은 Vite가 WASM URL 한 줄을 재작성한 차이를 별도로 기록했습니다.
- 최소 글꼴 회귀·공개 글꼴 주장 페이지 조건과, 기승인 원본 글꼴 조건의 보호 문서 검증을 구분합니다. 개인 글꼴을 삭제하거나 비활성화하지 않았으며 비공개 글꼴 바이너리를 Git에 넣지 않습니다.
- 새 주장 페이지 출력의 대표 79건은 과거 직접 판독한 raw/PDF와 SHA-256이 동일해 판독 근거를 재사용했습니다. 새 캡처 시각과 과거 판독 시각을 구분합니다. 모든 176쪽을 새로 직접 판독했다는 뜻이 아닙니다. [판독 재사용 근거](maintainer-repair/latest-head-3575d814/claims/direct-reading.json).
- 최신 최저 표본 #8729 3쪽의 review·overlay도 다시 직접 대조했습니다. 작은 글리프·전진 차이는 남으므로 픽셀 동일을 주장하지 않습니다. [추가 판독](maintainer-repair/latest-head-3575d814/claims/new-direct-reading.json).
- 각 리뷰에는 같은 입력·페이지의 Native/fresh WASM raw·PDF PNG·review·overlay, 실제 TSV 점수와 merge 후 원 PR 댓글 계획을 넣었습니다. [제품·입력·PDF 생산 근거](maintainer-repair/latest-head-3575d814/claim-producers.json).

## 개별 PR 리뷰

| 원 PR | 검토 문서 | 최신 대응 회귀 |
| --- | --- | ---: |
| [#8689](https://github.com/edwardkim/rhwp/pull/8689) | [리뷰](../../archives/pr_8689_review.md) | 1 PASS |
| [#8708](https://github.com/edwardkim/rhwp/pull/8708) | [리뷰](../../archives/pr_8708_review.md) | 1 PASS |
| [#8710](https://github.com/edwardkim/rhwp/pull/8710) | [리뷰](../../archives/pr_8710_review.md) | 2 PASS |
| [#8717](https://github.com/edwardkim/rhwp/pull/8717) | [리뷰](../../archives/pr_8717_review.md) | 9 PASS |
| [#8719](https://github.com/edwardkim/rhwp/pull/8719) | [리뷰](../../archives/pr_8719_review.md) | 1 PASS |
| [#8729](https://github.com/edwardkim/rhwp/pull/8729) | [리뷰](../../archives/pr_8729_review.md) | 4 PASS |
| [#8736](https://github.com/edwardkim/rhwp/pull/8736) | [리뷰](../../archives/pr_8736_review.md) | 1 PASS |
| [#8740](https://github.com/edwardkim/rhwp/pull/8740) | [리뷰](../../archives/pr_8740_review.md) | 1 PASS |
| [#8744](https://github.com/edwardkim/rhwp/pull/8744) | [리뷰](../../archives/pr_8744_review.md) | 1 PASS |
| [#8746](https://github.com/edwardkim/rhwp/pull/8746) | [리뷰](../../archives/pr_8746_review.md) | 2 PASS |
| [#8751](https://github.com/edwardkim/rhwp/pull/8751) | [리뷰](../../archives/pr_8751_review.md) | 3 PASS |
| [#8758](https://github.com/edwardkim/rhwp/pull/8758) | [리뷰](../../archives/pr_8758_review.md) | 2 PASS |
| [#8772](https://github.com/edwardkim/rhwp/pull/8772) | [리뷰](../../archives/pr_8772_review.md) | 2 PASS |
| [#8775](https://github.com/edwardkim/rhwp/pull/8775) | [리뷰](../../archives/pr_8775_review.md) | 2 PASS |
| [#8785](https://github.com/edwardkim/rhwp/pull/8785) | [리뷰](../../archives/pr_8785_review.md) | 2 PASS |
| [#8788](https://github.com/edwardkim/rhwp/pull/8788) | [리뷰](../../archives/pr_8788_review.md) | 1 PASS |
| [#8792](https://github.com/edwardkim/rhwp/pull/8792) | [리뷰](../../archives/pr_8792_review.md) | 1 PASS |
| [#8794](https://github.com/edwardkim/rhwp/pull/8794) | [리뷰](../../archives/pr_8794_review.md) | 4 PASS |
| [#8803](https://github.com/edwardkim/rhwp/pull/8803) | [리뷰](../../archives/pr_8803_review.md) | 1 PASS |
| [#8806](https://github.com/edwardkim/rhwp/pull/8806) | [리뷰](../../archives/pr_8806_review.md) | 1 PASS |
| [#8830](https://github.com/edwardkim/rhwp/pull/8830) | [리뷰](../../archives/pr_8830_review.md) | 1 PASS |
| [#8833](https://github.com/edwardkim/rhwp/pull/8833) | [리뷰](../../archives/pr_8833_review.md) | 4 PASS |
| [#8835](https://github.com/edwardkim/rhwp/pull/8835) | [리뷰](../../archives/pr_8835_review.md) | 3 PASS |
| [#8838](https://github.com/edwardkim/rhwp/pull/8838) | [리뷰](../../archives/pr_8838_review.md) | 1 PASS |
| [#8846](https://github.com/edwardkim/rhwp/pull/8846) | [리뷰](../../archives/pr_8846_review.md) | 1 PASS |
| [#8848](https://github.com/edwardkim/rhwp/pull/8848) | [리뷰](../../archives/pr_8848_review.md) | 1 PASS |
| [#8849](https://github.com/edwardkim/rhwp/pull/8849) | [리뷰](../../archives/pr_8849_review.md) | 1 PASS |
| [#8861](https://github.com/edwardkim/rhwp/pull/8861) | [리뷰](../../archives/pr_8861_review.md) | 3 PASS |
| [#8862](https://github.com/edwardkim/rhwp/pull/8862) | [리뷰](../../archives/pr_8862_review.md) | 1 PASS |
| [#8876](https://github.com/edwardkim/rhwp/pull/8876) | [리뷰](../../archives/pr_8876_review.md) | 1 PASS |
| [#8879](https://github.com/edwardkim/rhwp/pull/8879) | [리뷰](../../archives/pr_8879_review.md) | 1 PASS |
| [#8881](https://github.com/edwardkim/rhwp/pull/8881) | [리뷰](../../archives/pr_8881_review.md) | 3 PASS |
| [#8901](https://github.com/edwardkim/rhwp/pull/8901) | [리뷰](../../archives/pr_8901_review.md) | 2 PASS |
| [#8903](https://github.com/edwardkim/rhwp/pull/8903) | [리뷰](../../archives/pr_8903_review.md) | 4 PASS |
| [#8909](https://github.com/edwardkim/rhwp/pull/8909) | [리뷰](../../archives/pr_8909_review.md) | 3 PASS |
| [#8921](https://github.com/edwardkim/rhwp/pull/8921) | [리뷰](../../archives/pr_8921_review.md) | 2 PASS |
| [#8928](https://github.com/edwardkim/rhwp/pull/8928) | [리뷰](../../archives/pr_8928_review.md) | 1 PASS |
| [#8929](https://github.com/edwardkim/rhwp/pull/8929) | [리뷰](../../archives/pr_8929_review.md) | 1 PASS |
| [#8930](https://github.com/edwardkim/rhwp/pull/8930) | [리뷰](../../archives/pr_8930_review.md) | 1 PASS |

## 이력과 원격 처리

기존 `visual-final.json`, `protected-final.json`, 초기 review와 `maintainer-repair/final-*` 기록은 보정·리베이스 전 이력입니다. 최신 판정은 위 `latest-head-3575d814/`의 새 실행만 사용합니다. 기준 PDF·비교 문턱·보호 TSV를 변경해 통과시키지 않았습니다.

초기 HFT 표본의 Studio CanvasKit glyphMapping 계약 실패는 당시 같은 base에도 재현된 별도 조건입니다. 이번 Linux 6개 대조군을 그 HFT 표본의 재통과로 표기하지 않습니다. 이번 검토는 동결한 원 PR 주장 페이지의 Native/fresh WASM과 보호 문서 범위이며, [초기 base/후보 소비 경로 대조](validation/actual-browser-base-comparison.json)는 이력으로 남깁니다.

로컬 완료와 GitHub CI·승인·병합은 별개입니다. 원격 처리는 실제 실행한 PR·CI·merge SHA와 댓글·close 결과를 확인해 기록합니다. 원 PR이 참조하는 이슈는 해결 범위를 확인한 뒤 완전히 해결된 경우에만 닫습니다.

## 제출 후 발견한 CI 파일 목록 보정

PR #8958의 파일 4,142개가 REST API에서 3,000개로 잘려 CI preflight가 코드 변경을 문서 전용으로 오판했다. 수동 Full 실행은 필수 정책의 PR 이벤트 근거가 아니므로, CI·CodeQL·Render Diff·Adapter·Proptest 5개 workflow의 8개 수집 지점에서 전체 변경 수와 목록 완전성을 검사한다. [보정·실패→통과·기존 계약 결과](ci-file-inventory-validation.json)에 새 행동 검사 37개, 관련 Python 243개와 Node 166개, actionlint 5개를 연결했다. 제품 렌더러·입력·보호 기준은 바꾸지 않았다. CI 변경에 따른 최종 CI 기록·오늘할일은 code head의 PR CI와 병합 완료 후 옵션 2로 반영한다.


같은 lint가 Ubuntu 패키지 DNS 오류로 두 번 중단된 뒤 실제 의존성을 확인했다. PNG 계약은 PDFium을 사용하므로 이 단계의 불필요한 Poppler 설치와 오래된 Poppler 존재 조건을 제거했다. Poppler 네 도구 호출을 차단한 환경에서도 **254 PASS / 0 FAIL / 0 skip**, 실제 차단 도구 호출 0회를 확인했다. [의존성 감사와 실행 증거](ci-pdfium-bootstrap-validation.json)를 따르며, 일반 Visual Sweep의 메타데이터 도구 요구나 기준 래스터·검사 자체는 바꾸지 않았다.
