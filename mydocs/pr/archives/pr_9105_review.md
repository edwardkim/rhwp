# PR #9105 리뷰 — 가져온 번호·글머리표의 글자 모양 참조

## 최종 판정

**메인터너 보정 후 수용 가능.** 작은따옴표·속성 공백의 원문 참조 누락을 보정한 통합 후보에서 실제 가져오기·두 형식 반복 저장과 전체 회귀를 통과했다. 원 contributor head 직접 병합 대신 보정 통합 PR의 최신 CI를 확인한다.

## 접수 정보와 범위

- [원 PR](https://github.com/edwardkim/rhwp/pull/9105), 작성자 `donggyun112`.
- 원 head `559be7a737a19a29cc091d0724a015c357cd6fd6`, base `53774cc1508591b6ac794950f0063452fbaf6e87`.
- reviewer `jangster77`를 API로 확인했고 기존 reviewer는 유지했다. milestone은 `v1.0.0`이다.
- #9103의 HWPX `raw_para_heads`·`raw_para_head` 안 `charPrIDRef`를 대상 문서의 ID로 옮긴다.

## 검토 내용과 필요한 검증

렌더러가 직접 읽는 모델 참조만 아니라 직렬화기가 다시 쓰는 원문 참조도 일치해야 한다.
`import_paragraph_block_native`로 실제 ID 충돌을 만들고, 번호의 굵기·크기와 글머리표의
기울임·크기가 HWP/HWPX 저장→재열기→재저장 뒤에도 같은 참조를 가리키는지 확인한다.
불일치하는 opaque 원문과 지원하지 않는 그림 글머리표는 기존 원자적 거부 계약을 유지해야 한다.

새 mapper가 공백·큰따옴표 리터럴을 탐색하므로 XML이 허용하는 작은따옴표와 속성 주변
공백에서 원문 참조를 놓치는 결함을 실제 가져오기·저장 검사로 확인했다. 유효한 작은따옴표와
속성 주변 공백 두 경우 모두 모델 참조는 대상 ID 11로 바뀌지만 원문에는 원본 ID 7이 남았다.
최초 실행은 13/14건 중 11 PASS·2 FAIL이고 fail-fast로 1건이 미실행이다. 제품 실패와
그에 앞선 진단 코드의 quick-xml API 컴파일 오류를 구분한다. 관련 검사는
`tests/cases/issue_3587_block_import.rs`이며 보정 후 전체 집중 검사를 다시 실행했다.

보정 후보는 quick-xml로 실제 태그·속성의 의미를 읽고, 검증한 `charPrIDRef` 값의 원문 범위만
교체한다. 다른 속성·본문·따옴표·공백은 보존하고 원문/모델 불일치 거부를 유지한다.
최소 글꼴 macOS sandbox의 보정 후 집중 검사는 **14/14 PASS**, 329건 선택 밖,
실제 검사 시간 0.064초였다. 이 결과는 전체 회귀·시각·최종 CI 완료를 뜻하지 않는다.

## 조판 원칙과 증적

문서·글자 모양 ID의 소유와 원본/대상 간 참조 의미를 보존하는 변경이다. 무관한 시각 점수를
개선 근거로 사용하지 않는다. 원문 ID·대상 ID·실제 저장 바이트와 반복 저장 결과를 증적으로 남긴다.
시각 근거가 필요한 산출물이 생기면 유효한 데스크톱 Print를 사용하며 웹한글기안기 PDF를
공식 대조 자료로 수용하지 않는다.

## Merge 후 contributor PR comment 계획

통합 PR·merge SHA·최종 CI와 실제 가져오기/반복 저장 결과를 한국어 존댓말로 알린다.
메인터너 보정이 필요했다면 원 PR의 참조 재배치 성과와 보정한 XML 경계를 구분한다.
#9103의 전체 해결을 확인한 뒤 원 PR·이슈를 종료한다.


## 통합 후보의 최종 로컬 검증 결과

제품 소스 `167921b3c5`와 검증 head `3e9ce11f9c`에서 최소 글꼴 macOS 전체 회귀는
**13,696 PASS·0 FAIL·45 skip**이다. Native Skia·세 Clippy·workspace build·fmt·
suite 정책·unit tier·문서 테스트를 포함한 [Rust 13단계](../assets/all_open_prs_20261116/rust-gates-final.json)는 통과했다.
통합 검토의 공식 데스크톱 Print 66종·74쪽은
[Native](../assets/all_open_prs_20261116/native-claims-final.json)·
[fresh WASM](../assets/all_open_prs_20261116/wasm-claims-final.json) 각각 전쪽 90% 이상이다.
이 집계는 다른 PR의 입력을 이 PR만의 개선으로 계산한 값이 아니다. PR별 주장·반례는 위 접수 범위와
개별 보정 증거로 판정한다. 관계형 검사 보정 뒤 9단계는 이 head에서 다시 실행했고, 제품 및
해당 검사 소스가 바뀌지 않은 Native Skia·직접 PDF·placeholder·doc 4단계는 원 실행 신원을
보존해 재사용했다. [최신 회귀 판정](../assets/all_open_prs_20261116/relation-test-final.json)을 따른다.

[Native/base 비교](../assets/all_open_prs_20261116/native-base-head-comparison.json)와
[WASM/base 비교](../assets/all_open_prs_20261116/wasm-base-head-comparison.json)는 66종·74쪽에서
각각 상승 59쪽·하락 0쪽이다. [공통 보호 8조건·명시적 편람 383쪽](../assets/all_open_prs_20261116/public-gates-final.json)도 통과했다.
[실제 소비](../assets/all_open_prs_20261116/actual-consumers-final.json)는 Native PNG 16개·브라우저 48개를
실행했고 runtime fallback/replay hard 위반은 없다. report-only 최소 잉크량 32 FAIL과 자동 선택
readiness 미실행은 그대로 공개하며 통과로 바꾸지 않았다. [정본 전쪽 보호](../assets/all_open_prs_20261116/protected-final.json)는
Native/fresh WASM 18문서씩 총 3,542행에서 하락·누락 0쪽으로 완료했다. 상승도 0쪽이므로
정본 하한을 변경하지 않았다. [sample10 원본 공급 재현](../assets/all_open_prs_20261116/sample10-original-supply-validation.json)의
base/후보 Native/WASM 네 실행은 각각 전 763쪽 최저 99.57%·보호 하락 0쪽이다.
로컬 필수 검증은 완료했으며 통합 PR 게시 뒤 최신 head CI·병합·후속 처리는 별도 확인한다.

## 실제 공개 가져오기·저장 검사의 수집 확인

[검사별 실행 근거](../assets/all_open_prs_20261116/scope-test-coverage.json)에서
`issue_3587_block_import` 14건이 전체 회귀 안에서 실제 실행·통과한 것을 확인했다.
그중 canonical/작은따옴표/속성 공백 3건은 실제 HWPX 바이트 재파싱 → 공개 가져오기 API →
대상 ID 변경 → HWP/HWPX 저장·재열기·재저장의 원문/모델 참조 일치를 검사한다.
번호 굵기·1500HU 크기, 글머리표 기울임·1200HU 크기와 다른 대상 ID로의 이동을 함께 보호한다.
원본과 맞지 않는 opaque 원문 및 미지원 자원의 원자적 거부 검사도 같은 14건에 포함된다.

원문 속성 보정 SHA는 `5d0bbeef10`이다. 전체 회귀에서도 [실제 가져오기 14건](../assets/all_open_prs_20261116/scope-test-coverage.json)의 PASS를 개별 확인했다.

## 통합 PR #9143의 code head CI

[통합 PR #9143](https://github.com/edwardkim/rhwp/pull/9143)의 code head
`8fdcff552506c7d4a51edf6fcbebe2d63a1a729a`에서 [CI](https://github.com/edwardkim/rhwp/actions/runs/43948529913),
CodeQL·Render Diff·Proptest·Adapter·CI 정책 판정이 모두 완료·성공했다.
원격 기본 회귀 다섯 archive는 **13,511 PASS·0 FAIL·45 skip**이며 로컬 최소 글꼴 macOS의
13,696 PASS와 실행 환경·선택 범위를 구분한다. Native Skia와 frontend package gate도 성공했다.
[정확한 workflow·job·head와 실제 summary](../assets/all_open_prs_20261116/code-ci-final.json)를 보존했다.
문서 전용 trailing의 최신 required check·mergeability를 확인한 뒤 통합하며, 아직 merge나 원 PR 종료가
완료됐다고 기록하지 않는다. contributor 원 head·fork branch는 보존한다.
