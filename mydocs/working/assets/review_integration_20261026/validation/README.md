# 리베이스 후 통합 후보의 최종 로컬 검증 증거

**2026-10-27 #8319 반영 후 현재 상태:** 최신 기준 `af6bb05b2` 위 리베이스를 완료했다.
아래 수치는 이전 `faa885e8b` 기준 후보의 완료 증거이며 현재 후보의 재검증 결과로
사용하지 않는다. [현재 리베이스·충돌 해소·검증 상태](post-rebase-8319/README.md)를 따른다.

Native producer source는 `ae8c413f84a01a4cc4591357056183709f21b150`, 최적화 WASM 빌드 head는 `523af99acf2c53265e390a4e982eb07dd1071099`,
전체 Rust 검증 head는 `2e902abacf328bccdd7780606666ac3b1c5c1ce9`, 기준 devel은 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`다.
증거만 추가한 커밋과 실제 source를 구분하고 소스 바이트 동일성을 검증했다. 원 PR CI는 이 통합 후보의 CI를 대신하지 않는다.

| 경로 | 입력 조합 | 전쪽 비교 횟수 | 최저 raw | 90% 미만 |
| --- | ---: | ---: | ---: | ---: |
| native | 67 | 956 | 90.81277% | 0 |
| wasm | 67 | 956 | 90.94873% | 0 |

61개 HWP/HWPX 조합과 PDF 세 종류의 위치 고정·편집 우선 6개 조합이다. 쪽수는 문서별 전체 페이지의 합산 비교 횟수이며 하나의 문서 쪽수가 아니다. 각 후보와 독립 PDF의 쪽수·누락 검사도 통과했다.

- [전쪽 입력·명령·SHA·TSV·대표 PNG](visual-validation.json), [글꼴·래스터 출처](font-and-raster-provenance.json), [설치된 원 글꼴 경로와 SHA](font-program-source-locations.json)를 연결한다. `tsv/native/`와 `tsv/wasm/`는 각각 원본 TSV 67개다.
- [기준 대비 비교](base-head-comparison.json)와 [쪽별 십진수 차이](base-head-page-deltas.tsv)는 Native·WASM 각각 64개 조합·910개 기준 쪽이다. 같은 PDF·실제 글꼴 목록 128쌍을 대조했다.
  native: 양의 하락 0쪽, 2%p 이상 0쪽, 최대 하락 0.00000%p.
  wasm: 양의 하락 0쪽, 2%p 이상 0쪽, 최대 하락 0.00000%p.
  기준의 공식 사양 83쪽/71쪽과 cellnotes 12쪽/11쪽 오류는 기준의 실패로 보존했다. 대응 기준 쪽의 별도 진단과 추가 쪽 해시를 공개하며 후보 쪽수를 임의로 자르지 않았다. 새 editable 3모드는 구 버전에 없어 독립 PDF·실제 본문 소유 검사로 별도 검증했다.
- [입력 커밋 감사](tracked-input-audit.json): 정식 경로 125개의 파일 바이트와 커밋 blob이 일치한다. 실제 CLI/브라우저 PDF import 중간본 12개는 생성 결과이며 테스트 fixture로 커밋하지 않는다.
- [전체 Rust 실행 원장](final-rust-validation.json): fmt·세 Clippy·workspace build·고정 base 정책·focused 5개 그룹·전체 nextest·Native Skia lib·Native 4개 모듈을 순차 완료했다. 전체 nextest 11,637 PASS·기존 skip 45개, Native lib 합계 4,040 PASS·기존 ignore 12개다.
- [전체 검사에 포함된 관련 17개](final-contract-subset.json)는 그림 fit 6개·no-LineSeg 4개·source-frame 소유 7개다. 별도 실행 횟수로 더하지 않는다. 새 base의 overlay 회귀 15개도 focused와 전체 수집에서 확인했다.
- [최적화 WASM 빌드](final-wasm-build.json), [생산 소스 동일성](production-source-continuity.json)으로 Native·WASM·Rust head를 연결한다. pkg/public JS·WASM 해시와 WebAssembly.compile을 실제 확인했다.
- [Studio·Python·정책 검사](non-rust-validation.json)는 변경되지 않은 source의 Studio 2,607 PASS·기존 skip 2개 및 Python 192 PASS, workflow 196 PASS, font 29 PASS 등 완료된 실행을 구분한다. 새 WASM의 [브라우저 실행 원장](browser-validation.json)에 AI 제공자 E2E·PDF 설정과 가져오기·제한된 CanvasKit 선 연산을 기록했다. AI는 모의 API이며 실제 키를 사용하지 않았다.
- [실제 PDF UI 영수증](pdf-layout-ui-review.json)은 2010 16쪽·2020 15쪽·mac 15쪽의 두 모드와 비 PDF 대조군이다. editable 실제 본문 소유를 확인했다. 대형 span 배열은 개수·해시로 식별하며 golden으로 사용하지 않는다.
- [현재 CanvasKit 선 연산](post-rebase-8116/canvaskit-lines.json)은 원본 sungeo 34쪽의 계층·clip을 보존한 선/배경 연산이다. 두 모드 × 두 배율에서 실제 Skia 획 최소 1 장치 화소와 네 점선의 세 지점 화소를 검사했다. [전체 UI의 Canvas2D 전환](sungeo-canvaskit-preflight.json)과 구분하며 전체 문서 CanvasKit 통과로 세지 않는다.
- [정식 비교·overlay](review-panels.json)는 현재 Native/WASM PNG와 독립 PDF의 병치다. 수식·획·중첩 표·여러 줄 그림·중국어 글줄·온새미로 42쪽·재정동향·PDF 편집 모드 및 새 base의 표/도형 호스트를 포함한다.

## 새 devel #8116과 기존 개선 보존

- [리베이스 전후 커밋 매핑](rebase-8116-commit-map.json)은 117개 동일 패치와 한 결합 패치, 원 저자 보존을 기록한다. `line_breaking.rs`에서 overlay 표/도형 조건과 그림 호스트 조건을 모두 보존했다.
- [새 base 기준 PDF 출처](rebase8116-reference-provenance.json)는 공개 원본 4개와 기여자가 제공한 Hancom WebHWP Print PDF 4개의 바이트를 고정한다. 새 MCP Print로 부르지 않으며 파일 형식 번호로 Office 연도를 추정하지 않는다.
- [Native 전쪽 비교](post-rebase-8116/native-comparison.json), [WASM 전쪽 비교](post-rebase-8116/wasm-comparison.json)로 새 4개 문서·15쪽과 기존 조합을 함께 비교했다.
- [기존 Native 출력 연속성](post-rebase-8116/native-prior-output-continuity.json)은 기존 63개·941쪽의 PNG가 리베이스 이전과 바이트 단위로 동일함을 확인한다.
- [새 base 변경 영역 Native 직접 판독](post-rebase-8116/native-direct-reading.json), [현재 WASM 직접 판독](post-rebase-8116/wasm-direct-reading.json)을 연결한다. 표·도형·그림의 소유·겹침·행 종료를 판독했다. 기준에도 있는 장식 색상 영역 크기·글꼴 굵기·일부 셀 줄바꿈 차이를 해결 완료로 바꾸어 보고하지 않는다. 새 네 문서의 기준/후보 출력 연속성도 원장으로 확인했다.
- 기존 수정 영역의 [Native 판독](v14-native-reading-notes.json)·[WASM 판독](v14-wasm-reading-notes.json)은 당시 source와 당시 실제 결과의 기록이다. 현재 동일 출력 증거 또는 새 확대 판독과 함께 사용하며 과거 실행을 현재 재실행으로 세지 않는다.

## 독립 근거와 남은 차이

- [공식 사양 20·31쪽 잔여 차이](official-spec-known-residuals.json), [실패한 새 Print와 공식 PDF 재사용](official-spec-reference-reuse.json)를 분리했다. 기존 글꼴·작은 표식·도식 빗금·간격·번호 차이를 이번 프레임 수정의 해결로 주장하지 않는다. 공식 PDF·사양·정오표는 수정하지 않았다.
- [독립 여백 편집 Print](civil-form-independent-margin-edit.json)는 원본 top margin만 바꾼 출력이다. 기존 0 이동 기대값을 정정한 독립 근거다.
- [source-frame 수정 전후](source-frame-before-after-proof.json), [7번째 회귀·정렬 보정](v12-focused-proofs.json)은 실제 assertion 실패와 성공을 구분한다. 컴파일 실패·0개 수집을 결함 재현으로 세지 않는다.

## 실패·중단 및 과거 실행

- [잘못된 캐시 재사용 무효화](post-rebase-8116/invalid-cached-wasm-attempt.json)는 첫 후보 WASM이 기준 artifact와 같고 실제 중국어 출력도 보정 전이었던 시도다. 현재 소스 재컴파일·바이너리 차이·실제 보정 출력 확인 후 전쪽·브라우저·Rust를 새로 완료했으며 이전 결과를 최종 성공으로 세지 않는다.
- [초기 전체 실행](rust-before-safety-corrections.json): 8,854 PASS·실제 assertion 실패 7개·SIGINT 3개·미실행 2,757개·기존 skip 45개. 전체 통과로 세지 않는다.
- [그림 fit 보정 전 전체 실행](rust-before-inline-picture-fit-correction.json): 11,619 PASS·실제 실패 3개·기존 skip 45개. 이후 공통 물리 행 상자 기반 수용을 보정했다.
- [v13 WASM 의도적 중단](v13-wasm-corrective-stop.json)은 정적 검토의 불필요한 부동 도형 점유 합산을 제거하기 위한 중단이며 최종 빌드 실패가 아니다.
- 리베이스 전 v14의 63개·941쪽, nextest 11,622 PASS 결과는 완료된 과거 결과다. 이 문서의 리베이스 후 67개 결과와 구분한다.

원 PR 20건의 review·impl과 보정 SHA는 [통합 계획](../../../review_integration_20261026_document_updates_plan.md)에 연결한다. 로컬 준비를 완료한 뒤 작업지시자의 PR 생성 승인을 기다린다. 원격 push·PR 생성·CI·머지·원 PR comment/close를 완료로 세지 않는다. 임시 실행 스크립트·생성 fixture/golden·개인 글꼴 파일을 포함하지 않는다. [원시 로그 감사](temporary-log-audit.json)는 최종 트리에서 제거한 임시 로그의 요약·해시·실제 결과를 보존한다.
