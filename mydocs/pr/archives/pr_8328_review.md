# PR #8328 리뷰 — 원문 대조와 조판 보정

## 최종 판정

**메인터너 보정 후 수용 가능 — 원문 대조와 조판 보정 범위의 로컬 검증 완료.** 원 PR 원본을 확보하지 못한 경우의 대체 문서와 기존 90% 미만 페이지는 아래에 구분했다. 코드 head CI는 성공했으며 문서 trailing head의 CI와 merge 상태는 별도로 확인한다.

## 접수와 변경 범위

- [PR #8328](https://github.com/edwardkim/rhwp/pull/8328), 작성자 `jangster77`, base `devel`.
- 고정 base `af6bb05b26887135fb9da0fa4e54ca7205564fca`, 검증한 코드 head `0ba3d04fe94118fdebe26c94d6d692e10dca4079`.
- 통합 PR #8319의 45개 원 PR에서 주장한 문서와 이미지 대응을 재확인했다. 원 PR에 입력이 없는 경우 대체 입력임을 표시하고, 확보한 공식 문서와 한컴 Print의 같은 주장 위치를 직접 비교했다.
- EMF 차트의 복합 다각형·패턴 브러시·채움 경로와 저장 줄 개체 소유·표 뒤 글줄·위 캡션 프레임을 공통 처리로 보정했다. 원문 16건의 대응, SHA 및 Print 출처는 [원문·PDF 원장](../assets/pr8319_evidence_alignment_20261027/source-input-pdf-pairs.json), 원 PR별 검토는 [PR 본문](https://github.com/edwardkim/rhwp/pull/8328)에 기록했다.

## 검증과 시각 대조

- `cargo fmt --all -- --check`, Native·WASM32·workspace all-target Clippy, workspace build와 suite 정책 검사를 통과했다. 전체 nextest는 11,841개 통과, 45개 건너뜀이다. Native Skia 라이브러리 본체 3,858개, 이미지 누락 표시 2개, 직접 PDF 출력 14개가 통과했다.
- 최적화 포함 fresh WASM을 빌드해 Studio public 산출물과 해시를 대조했다. 같은 입력과 한컴 Print의 Native/fresh WASM 전체 페이지 TSV 및 주장 위치의 review·overlay를 제출했다. 기존 대조군 39건의 최신 결과는 [원장](../assets/pr8319_evidence_alignment_20261027/current-evidence/index.json)에서 확인한다.
- 고정 base 또는 이전 증적과 비교 가능한 같은 물리 쪽에서 raw 0.1%p 초과 하락은 없었다. #8036은 base 12쪽과 후보·Print 13쪽으로 공통 12쪽만 점수 비교했고 추가된 13쪽은 별도 판독했다.
- 사용자가 새로 확보한 원문의 전쪽 90% 보정 의무를 면제했다. #8039 WASM 표지 89.61973%, #8045 Native/WASM 최저 71.03970%/71.10912%, #8049 26.81625%/26.63057%, #8056 65.65455%/66.39946%, #8062 76.80043%/76.70998%는 기존 차이로 남으며 통과로 기록하지 않는다.
- #8014의 모든 페이지는 양쪽 90% 이상이며 14·15쪽은 양쪽 98% 이상이다. #8039는 양쪽 90% 이상인 5·6쪽의 그림·후속 줄 관계를 검사하며, 표지 미달로 전체 쪽수는 회귀에 고정하지 않았다. 98% 이상인 독립 Print 좌표에는 ±2px 검사를 사용하고 관계 검사도 유지한다.
- #8014, #8030, #8031, #8049, #8053, #8055, #8056, #8061, #8065는 확보한 대체 문서의 통합 전 Native에서 원 PR의 수정 전 증상이 재현되지 않았다. 이 비교를 해당 증상의 수정 전후 재현으로 주장하지 않는다.
- 기존 대조군 #7694의 일시적인 Native 저하는 공유 Cargo 캐시에 통합 전 차트 crate가 연결된 산출물의 오류였다. 후보 crate를 다시 컴파일해 Native 99.71479%, fresh WASM 99.73289%를 확인하고 해당 생산자로 모든 Native 대조를 다시 실행했다.

## GitHub CI와 후속 처리

- 정확한 코드 head의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/40578110078)는 2차 시도에서 성공했고 [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/40588390920)도 성공했다. 첫 시도의 shard B는 checkout에서 25분 이상 정체해 취소했으며, 2차 shard B는 checkout과 회귀 검사를 모두 통과했다.
- 이 문서와 오늘할일은 위 CI 성공 후 추가하는 문서-only trailing 기록이다. trailing head의 CI·mergeability를 다시 확인한 다음 merge한다.
- merge 후 45개 원 PR의 기존 댓글을 각각 수정하고, 실제 사용한 원문 또는 대체 입력과 같은 페이지의 review·overlay·TSV를 merge SHA로 고정해 게시 후 재조회한다. 새 정정 댓글은 추가하지 않는다.

## 이전 문서 후보의 검토 이력

아래는 원문 대조 및 추가 코드 보정 전의 기록이다. 당시 보류 판정과 문서-only CI는 위의 현재 판정·코드 CI와 구분한다.

### 최종 판정

**머지 보류 — 작업지시자가 요청한 원본·대응 PDF 동반 보존을 보완 중이다.**

### 검토 범위

통합 PR #8319의 잘못된 대표 이미지 연결을 실제 검증 입력·Print·페이지로 정정했다. 원 PR 보고 문서 사례 22건은 유사 문서 대체 19건과 파일 없는 단위 계약 3건으로 구분했다. 대체 자료를 원 보고 실물 문서의 직접 검증으로 표현하지 않는다.

39개 유효 재현 입력의 Native/fresh WASM 전쪽 TSV와 같은 페이지 review/overlay를 연결했다. #8033의 저장본 추가 진단 실패는 성공 자료로 사용하지 않는다. 공식 배포처에서 내용 대응을 확인한 문서도 작성자 파일과 바이트 동일성을 뜻하지 않는다. 새로 찾은 문서의 전 페이지 90% 보정은 작업지시 범위가 아니다.

### 실제 검증

- 문서 후보: `ad321308c3e3acfb6ca0bd8f27525a20dbea3e9d`.
- base: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [후보 CI](https://github.com/edwardkim/rhwp/actions/runs/40513485940): completed / success.
- preflight: `fast_pass=true`, `all-review-only-no-code-impact`, candidate는 위 base다. heavy Rust/Studio worker의 skip은 문서-only 판정이며 새 전체 회귀 실행 성공으로 표현하지 않는다.
- 45개 대응 원장, 대체 여부, 실제 입력·PDF·PNG 해시와 전체 TSV 페이지·수치 검증 통과.
- 전체 TSV 78개를 포함한 raw 자산 88개 재다운로드: 커밋된 원본과 바이트 동일.
- Markdown 97개 내부 링크·공백 검사 통과. source/test/fixture 등록/baseline/workflow/지침 변경 없음 확인.
- 최신 base와 `git merge-tree --write-tree` 종료 코드 0.

[최종 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json), [공식 출처·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md), [CI 영수증](../assets/pr8319_evidence_alignment_20261027/pr8328-candidate-ci.json)을 함께 확인한다.

### 후속 처리

이 기록은 후보 CI 성공 후의 review-only trailing 기록이다. trailing 최신 head의 필수 check 성공을 별도로 확인한 뒤 머지한다. 이미 완료된 #8319의 기록 정정이므로 오늘할일·이슈 종료·새 contributor 댓글을 중복하지 않는다.

자료가 devel에 반영된 뒤 기존 45개 원 PR 댓글과 #8319 본문·기존 결과 댓글을 직접 수정한다. 새 정정 댓글은 만들지 않는다. 실제 merge SHA로 입력·PDF·TSV·review/overlay 링크를 고정하고 게시 내용을 재조회한다. contributor fork·원본 branch는 보존한다.

### 추가 요청 반영

HWP/HWPX 16개는 후보에 이미 포함됐다. 누락했던 대응 PDF는 현재 16개 모두 Print 출력본으로 확보해 정식 경로에 보존했다. 원 PR의 실제 주장 영역 확인과 필요한 구현 보정은 계속 진행한다. [원본·PDF 대응표](../assets/pr8319_evidence_alignment_20261027/source-input-pdf-pairs.md)를 따른다. 실제 주장 영역의 수정 전후 검증과 새 head의 필수 검증 완료 후 재판정한다.

기여자 지침·PR 템플릿·접수 절차에는 조판 변경 PR의 실제 검증 HWP/HWPX 첨부를 필수로 추가했다. 입력이 없으면 보류하며 스크린샷·PDF·TSV·생성 코드만으로 대신하지 않는다. 기존 파일의 SHA 고정 경로 재사용은 허용하고 실제 대체 입력의 동일 결함 검증과 대체 표시를 요구한다.
