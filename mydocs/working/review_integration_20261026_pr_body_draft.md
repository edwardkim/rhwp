# 통합 PR 본문 준비안

> 로컬 검증과 개별 리뷰를 기록한 제출용 준비안이다. 원격 CI·머지 결과는 별도로 확인한다.

## 제목안

문서 조판·그림·PDF 처리와 성능 개선 통합

## 본문안

planet6897의 문서 조판, 그림 저장, PDF 가져오기 및 성능 개선 PR 20건을
최신 `devel` 위에 통합한다. 표·각주와 저장 프레임의 경계, 가는 획, 그림
호스트 재조판, PDF 본문 흐름 및 Studio 설정을 함께 검토하고 통합 과정에서
드러난 호환성 문제를 보정한다.

대상: #8115, #8128, #8133, #8142, #8184, #8202, #8205, #8206, #8209,
#8210, #8213, #8215, #8217, #8219, #8221, #8227, #8229, #8237, #8265,
#8283. 원 head와 최초 적용은 [기존 체리픽 원장](assets/review_integration_20261026/cherry_pick_ledger.json), 리베이스 후 현재 적용 SHA는 [대응 원장](assets/review_integration_20261026/cherry_pick_current_ledger_20261028.json)에 기록한다.
#8268과 #8161은 이 통합 대상이 아니다.

Closes #7723

2026-10-29 현재 원 PR 20건의 head와 본문은 기존 검토 시점과 같다. 개별 리뷰 40개는
링크 검사 뒤 변경되지 않았다. [원 PR·리뷰 대조](assets/review_integration_20261026/validation/post-rebase-8328/source-pr-head-body-20261029.json).
이후 원 PR 20건의 현재 상태·head·본문을 다시 조회했으며 모두 OPEN이고 위 대조와 head·본문이 같다.
[재조회 결과와 시각](assets/review_integration_20261026/validation/post-rebase-8328/source-pr-head-live-recheck.json).
개별 리뷰는 이후 수정했으므로 위의 리뷰 파일 불변 기록은 해당 대조 시점에만 적용한다.

## 제출 전 검증 기록

- 기준 `db5cd5bcc60a69e42f419f18a7f57a33b6fb7fdc`(#8328) 반영 후
  변경 범위와 새 기준의 Native·fresh WASM 결과를 아래와 같이 대조했다.
  페이지별 하락 기준은 0.1%p다.
- #8319 반영 범위 Native는 197개 입력·256쪽에서 실행 오류, 0.1%p 이상 하락과
  90% 미만이 없었다. [전쪽 비교 원장](assets/review_integration_20261026/validation/post-rebase-8328/incoming-native-comparison.json).
- #8328 기준 대비 fresh WASM은 55개 입력·329쪽에서 0.1%p 이상 하락이 없었다.
  기준에서도 90% 미만인 여섯 쪽은 기준과 후보의 동일 결과로 별도 기록한다.
  [전쪽 비교 원장](assets/review_integration_20261026/validation/post-rebase-8328/fixed-wasm-matrix.json).
- 표 뒤 문단 보정 당시 전체 Rust 회귀검사는 11,898개 통과·45개 skip으로 종료했다. 이후 원 PR 추가 체리픽과 회귀 assertion 변경이 있으므로 이 실행을 최종 head의 전체 회귀 결과로 표기하지 않는다. 작업지시자 지시에 따라 전체 회귀는 다시 실행하지 않으며 필요한 남은 개별 검사를 진행한다.
- 후속 보정 후 관련 선택 회귀 24개가 모두 통과했다(필터에서 제외된 794개는 재실행하지 않음). [검사명·명령과 로그 해시](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-after-targeted-nextest.json).
- #8184 후속 보정 전 측정에서 #8188 세 문서의 Native 및 fresh WASM 총 384쪽은 실제 이미지의 양방향
  반경 2px 미대응 픽셀이 0이다. [전쪽 픽셀 계약](assets/review_integration_20261026/validation/post-rebase-8328/sample16-fixed-wasm/combined-pixel-contract.json).
- #7723의 2010 이름 원본은 기본 HWP와 바이트가 같고, 기존 2024 MCP PDF 기준 Native
  전 64쪽에서 raw 100%를 확인했다. 이후 재출력한 새 MCP PDF는 같은 결과가 아니며
  원 PR HWPX의 22쪽 재출력에서도 차이가 관찰됐다. 작업지시자는 PR 검증 기준으로 기존
  MCP PDF를 선택했다. 최종 Native와 fresh WASM은 기존 PDF 래스터 기준 64쪽 모두 raw 100%다.
  Native는 기존 글꼴 검색 조건에서 과거 100% PNG와 전쪽 해시가 같다. 현재 방식으로 PDF를
  재래스터화하면 Native 세 쪽이 99.98~99.99%다. [Native 전쪽 근거](assets/review_integration_20261026/validation/post-rebase-8328/issue7723/selected-reference-latest-native.json),
  [WASM 전쪽 근거](assets/review_integration_20261026/validation/post-rebase-8328/issue7723/selected-reference-latest-wasm.json).
  새 PDF의 미달은 [22쪽 진단](assets/review_integration_20261026/validation/post-rebase-8328/issue7723/page22-font-line-diagnostic.json)에 별도로 기록한다.
- 과거 WASM TSV 일부의 원래 실행 환경은 재현하지 못했다. 대신 아래 #8319 보존 비교는
  동일한 기존 PDF 래스터와 전쪽 렌더 PNG 해시를 경로별로 대조한 결과다.
- 과거 TSV와 차이가 난 12쪽은 과거와 SHA가 동일한 WASM과 최신 후보를 같은 현재 글꼴·PDF
  래스터 조건으로 직접 비교했다. 최대 하락은 0.02042%p이며 0.1%p 이상 하락은 없다.
  이는 과거 TSV 점수 자체를 복원한 결과는 아니다. [12쪽 대조](assets/review_integration_20261026/validation/post-rebase-8328/historical-wasm-exact-to-current-12pages.json).
- 가정통신문 Native 두 쪽은 과거 글꼴 조건을 프로세스 단위로 복원하고 기존 PDF 래스터를
  고정했을 때 과거와 현재 렌더 PNG 및 점수가 같다. 현재 시스템 글꼴 조건의 점수는 별도로
  기록한다. [동일 조건 대조](assets/review_integration_20261026/validation/post-rebase-8328/newsletter-historical-font-recovery.json).
- 리베이스 시 두 #8149 후속 체리픽을 기존 묶음 뒤로 이동했다. 충돌은 없었고 리베이스 전후 최종 트리는 동일했다. [순서와 원 PR 대조](assets/review_integration_20261026/validation/post-rebase-8328/source-pr-paired-range-diff-review.md). 이후 문서만 추가 커밋했다.
- 새 #8184 보정 이후 sample16 세 문서는 Native와 fresh WASM 각각 64쪽 모두 기존의 동일 PDF 래스터와 raw 100%다. 현재 방식으로 PDF를 다시 래스터화하면 일부 99.98%로 측정되어 두 결과를 별도 TSV로 보존한다. [입력·PDF·쪽별 해시 및 TSV](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-after-sample16/evidence.json).
- 새 #8184 보정 이후 36개 표 조각 입력의 Native·fresh WASM 각 64쪽은 모두 raw 100%다. [입력·PDF 및 쪽별 결과](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-float-fragments/evidence.json). 공식 사양서 71쪽도 같은 기존 PDF 래스터 기준 두 경로 모두 보정 전과 전쪽 수치가 같으며 0.1%p 이상 하락이 없다. [전쪽 비교](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-spec-all/evidence.json).
- 후속 체리픽 뒤 기존 통합 범위 67개 입력·956쪽은 Native·fresh WASM의 전쪽 렌더 PNG가 보정 전과 각각 동일하다. 같은 기존 PDF 래스터를 기준으로 두 경로 모두 0.1%p 이상 하락은 없고 90% 미만도 없다. [Native 66건](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-original-native-png-comparison.json), [WASM 66건](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-original-wasm-png-comparison.json), [공식 사양서 71쪽](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-spec-all/evidence.json). 동일 PNG에 관한 수치는 기존 래스터에서 추론한 것으로, 새 래스터의 직접 측정과 구분한다.
- 20건의 개별 리뷰에는 원 PR의 해당 입력·쪽과 과거 실행 시점을 구분해 기록했다.
- #8219의 사용자 지정 `hwp3-sample4.hwp`는 기존 base와 후보 PNG가 양 경로에서 전쪽 같지만,
  1쪽은 90% 미만이다. 작업지시자는 글꼴 제약을 확인한 뒤 이 쪽의 추가 보정 없이 진행하도록 했다.
  [해당 리뷰](../pr/archives/pr_8219_review.md).

제출 직전 최신 `upstream/devel`, 변경된 head와 필수 검사 결과를 다시 확인한다.
원격 CI와 머지는 아직 수행하지 않았다.

- #8319 보존 대상 197개 입력·256쪽은 Native와 fresh WASM 렌더 PNG가 후속 보정 전과 각 경로에서 전쪽 동일했다. 기존 PDF 래스터 기준 최저는 Native 93.22230% / WASM 93.61735%이며, 0.1%p 이상 하락과 90% 미만이 없다. 새 PDF 래스터의 직접 측정과 구분한다. [Native 전쪽](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-incoming-native-same-raster-comparison.json), [WASM 전쪽](assets/review_integration_20261026/validation/post-rebase-8328/pr8184-incoming-wasm-same-raster-comparison.json).

- 시스템 글꼴 디렉터리를 가리고 Fontconfig 검색 결과를 0개로 제한한 Linux 격리 환경에서 이 브랜치의 회귀 파일 26개를 선택 실행해 110개 모두 통과했다. 저장소에 포함된 글꼴은 검사 자원으로 사용할 수 있으며, Windows·macOS나 전체 회귀의 대리 결과는 아니다. [파일별 검사와 로그 해시](assets/review_integration_20261026/validation/post-rebase-8328/fontless-branch-regression.json).
- 2026-10-29 Rust 형식 검사, Native·WASM·workspace 전체 대상 Clippy, workspace 빌드,
  테스트 모음과 단위 테스트 정책 검사가 통과했다. 전체 회귀는 지시에 따라 재실행하지 않았다.
  [명령과 로그 해시](assets/review_integration_20261026/validation/post-rebase-8328/pr-local-rust-gates-20261029.json).
- Studio TypeScript 검사와 AI 제공자·PDF 배치·CanvasKit 변환 관련 선택 검사 41개가 통과했다.
  [대상과 로그 해시](assets/review_integration_20261026/validation/post-rebase-8328/studio-targeted-20261029.json).
  이는 전체 Studio 검사나 최신 WASM 브라우저 검사 전체의 대체 결과가 아니다.
- 최신 WASM을 실제로 받은 격리 브라우저에서 AI 제공자 검사 34개와 2010 시험 PDF의
  기본·편집 배치 및 페이지 수 확인 6개가 통과했다. [바이트 해시·실행 근거](assets/review_integration_20261026/validation/post-rebase-8328/studio-fresh-browser-20261029.json).
- Fontconfig 검색 결과를 0개로 제한한 환경에서 최신 WASM을 사용한 CanvasKit 점선 네 종류와
  투명 테두리 검사가 통과했다. 글자 연산은 검사 범위에서 제외했으며 시스템 글꼴 디렉터리를
  숨긴 실험은 아니다. [실행 근거](assets/review_integration_20261026/validation/post-rebase-8328/studio-fresh-browser-20261029.json).
- 최신 Visual Sweep의 선택 검사 13개와 CI workflow `actionlint`가 통과했다. [명령과 로그 해시](assets/review_integration_20261026/validation/post-rebase-8328/prepr-visual-ci-checks.json).
- 새로 설치한 글꼴은 내부 검증에만 사용한다. base 대비 변경 파일과 변경된 HWPX·ZIP·EPUB에서
  글꼴 확장자 파일은 발견되지 않았다. 이 검사는 PDF 글꼴 자원을 검사하지 않았다. [범위와 결과](assets/review_integration_20261026/validation/post-rebase-8328/font-distribution-audit.json).
