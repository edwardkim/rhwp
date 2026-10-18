# #7998 P4 여섯 번째 절편 이후 — 병합·지역 격자의 관측 근거

- 기준 checkpoint: `0007064995157631b540a0c8e31380db2caa5e14`.
- 2026-10-18 지시: 잘못 입력한 PDF 안내는 제외하고 다음 절차를 진행한다.
- 이슈 #7998은 open·assignee edwardkim, 관련 열린 PR은 없다. 기존 task branch를 사용한다.
- B17의 통과 판정·빌드·전쪽 증거는 보존한다. 원격 반영과 PR 생성은 이번 지시에 포함하지 않는다.

## B18 첫 경계와 구현 전 확인

P4/B03의 실제 셀 범위를 먼저 다룬다. 현재 `table_detection::grid`는 선이 없는 내부
연결의 사각 face와 분리 대안을 함께 만들고, `detect_ruled_tables`가
`uncertain_connections > 0`인 family를 일괄 보류한다. Builder는 이미 colspan/rowspan을
소비하지만 앞 단계에서 근거를 선택하지 못하면 실제 IR로 연결되지 않는다.

선이 없다는 사실이나 사각 clip 하나만으로 병합을 확정하지 않는다. 같은 외곽·부분 내부
선을 가진 **정상 colspan/rowspan 생성본**과 **선을 감춘 별도 셀**을 독립 한컴 Print로
먼저 대조한다. clip이 어떤 내용과 연결되고 그 경계·공통 글줄/문자의 범위가 각 대안을
어떻게 지지하거나 반박하는지 확인한다. 겉모양만 같은 무태두리 셀은 보류를 유지한다.

입력의 공통 문자 관측 → 후보 face와 clip/내용 근거 → 실제 셀 소유 → Builder의 span·
BorderFill/paint → 편집·양 저장·재열기까지 연결한다. 새 문자 군집화를 별도로 만들지 않는다.
작성 당시 객체 복원과 출력에 일관된 편집 IR의 선택을 구분하며, 대안과 선택 이유를 남긴다.
독립 근거가 부족한 경로는 미검증/보류다. 임의 지역 후보 전체를 채택하거나 병합이 없는
가짜 축·문서 ID 조건으로 예제를 통과시키지 않는다.

로컬 증거는 저장소 밖의 `semantic-stage3/p4-cell-regions/`에 보존하고,
ignored `output/pr-review/issue7998/p4-cell-regions/`에 연결한다. 작은 적용/비적용 경계와
독립 Print를 먼저 확인한 뒤 영향 범위만 Native/fresh Docker WASM·실제 Studio로 검증한다.
전체 CI 성격의 긴 검증은 P5의 별도 게이트다. P4 전체 완료나 PR 준비 완료로 표시하지 않는다.

## 독립 관측과 실제 소비의 보정

`generate.rs`는 공개 scaffold에서 정상 create/insert/merge/border API로 colspan,
rowspan, 숨긴 공유 변의 별도 셀을 만들었다. 수동 LineSeg를 넣지 않았다. 한컴 변환 서비스의
기존 대기 작업이 완료됐으므로 새 세 입력도 engine 2020 Print로 요청해 성공했다.
검증 응답은 `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`,
`hancom_version=11.0.0.9136`이다. 원본·생성기·응답은 로컬 증거에 보존했다.

실제 병합 입력은 병합 영역 전체의 clip, 선만 감춘 네 셀은 각 셀 clip을 사용했다.
clip과 문자의 위치가 겹친다는 사실로 소유를 정하지 않는다. backend의 page/Form interpreter가
q/Q·CTM·BBox를 적용한 상태에서 text-show의 invocation path와 활성 clip chain을 기록한다.
painted span은 clip/graphics-state 경계를 넘어 합쳐지지 않으며 검색 API는 그대로다.
`extract`가 기존 공통 ObservedText를 먼저 만든 뒤 같은 glyph 관측과 사용 관계를 검출에 전달한다.

`table_detection/cell_regions`는 원래 paint 격자 안에서 내용 clip의 완전한 직사각 tiling,
glyph 포함·유일 소유, 실제 분리선과의 비모순을 확인한 해석만 선택한다. 빈/누락 구역,
부분·비사각·겹치는 clip, 실제 선을 가로지르는 넓은 clip, Q 뒤의 죽은 clip은 보류한다.
이 자격은 원래 작성 객체의 유일한 복원 증명이 아니다. 대안과 `authoredStructure=unknown`을
유지하고, 선택된 셀 주소·glyph ID·실제 source order를 진단에 남긴다.

첫 실행에서 Builder의 uniform-border 경로가 별도 셀을 보류하고 일부 글줄을 다시 추측하는
소비 결함을 검출했다. 선택된 무paint 경계는 기존 `source_paint_family → fixed_lines →
per-cell paint/native border plan`으로 연결했다. 없는 공유 선을 Solid로 만들지 않는다.
실제 Native API의 글자·행열 삽입/삭제·양 저장·재열기에서 주소/span·단일 내용 소유·셀 포함은
통과했다. 아직 최종 시각 판정으로 바꾸지 않는다.

확대 판독에서는 `ArialMT`를 HWP family로 그대로 저장하여 측정과 출력이 다른 글꼴을
소비하는 결함도 확인했다. 공급한 Arial 프로그램의 OpenType nameID 6=`ArialMT`,
nameID 1=`Arial`을 독립 대조했다(`arial-identity-evidence.json`). PDF import의 정확한
PostScript→family 쌍만 연결하며 임의 suffix 제거·문서 조건을 추가하지 않는다. 원래 span 이름은
관측에 보존하고 IR/측정/출력은 family를 공유한다. 최종 Native/fresh Docker WASM 검증은 진행 중이다.

Form의 자원 별칭 `F1`을 caller 자원으로 나중에 해소하는 별도 결함도 실제로 재현했다.
같은 이름을 caller가 Times-Roman, Form이 ArialMT로 사용한 반례를 추가하고, buffered/direct
painted span이 활성 FontInfo를 참조하는 순간 BaseFont를 보존하도록 고쳤다. 동일한
Form source의 Native raw는 87.18431% → 98.08227%이며, 텍스트의 위치를 숨기거나 clamp하지 않았다.
일반 검색의 기존 경로는 유지한다.

새 source 12개 × PDF/HWP/HWPX와 B17 공통 범위의 Native 결과는 237건·270쪽,
최저 97.45634%이며 쪽수 차이·90% 미달·오류가 없다. 공통 201건·234쪽 PNG는 모두 동일하다.
실제 글자·행열 편집 저장본 18쪽은 독립 한컴 Print 대비 최저 94.26318%다.
`native/results.json`, `native-base-comparison.json`, `native-extra/results.json`을 참조한다.
fresh WASM과 직접 판정은 아직 진행 중이며 이 Native 결과로 대신하지 않는다.

최초 삼각 clip 진단본은 실제 글자를 자르는 기존 text-clip 미지원 차이가 86.64081%로 남아
`diagnostic/nonrectangular-triangle.pdf`에 원본·실패 출력을 보존했다. 새 회귀 등록을 보류한다.
현재 비사각 자격 반례는 글자를 가리지 않는 별도 bevel 입력이다. 그 통과를 삼각 진단본의
출력 일치로 바꾸어 보고하지 않는다. 일반 비사각 text paint 복원은 이번 셀 선택의 완료 주장이 아니다.

Docker 첫 호출은 worktree 이름의 project/network를 만들다 주소 풀 부족으로 빌드 전에 종료됐다.
기존 `-p rhwp` network·named volume을 명시해 표준 서비스로 재개했다. 이때 생성된 새 project
volume 네 개는 생성 시각과 container 부재를 확인하고 제거했다. 공유 rhwp volume은 유지했다.

## 최종 producer와 검증 결과

제품 소스는 `source-runtime.json`의 2,446개 입력으로 고정했다. SHA-256은
`adff5a43f2b12de61c0b8101f9a9014e4f18b81edbfe82354fa8308e065da55c`다.
새 fixture·회귀·문서 등록 뒤에도 제품 입력의 hash가 같음을 확인했으므로 빌드·전쪽 캡처는
다시 실행하지 않았다. 아래 source SHA는 빌드 시점의 제품 입력 집합인 이 manifest를 뜻한다.

- Native Skia: `native-build-form-scope.log` exit 0, `rhwp-candidate` SHA-256
  `c27200ca2691fa646a07f1da55fe429638f393a8ce171dd3677b6a3a3e013d02`.
- fresh Docker WASM: `docker-wasm-final.log` exit 0, 19,251,626 bytes, SHA-256
  `1b4ff7797cfd6e37d1319d8f3f4d6f3009843a29cfecd7b91ac13687ee6d6c3c`.
  `pkg/` 전체를 보존했고 실제 Studio가 받은 바이트·SHA와 일치한다(`studio-result.json`).
- 고정 host `target/pr-review`와 기존 `rhwp` Docker named volume을 재사용했다.
  기본 7700의 JS/WASM은 최초 hash와 같다. 검증 Studio 7793만 갱신했다.

| 범위 | 결과·증거 |
|---|---|
| 원본 79개 × PDF/HWP/HWPX, 전쪽 | 각 backend **237건·270쪽**. 최저 Native **97.45634%**, fresh WASM **97.41574%**. 90% 미달·쪽수 차이·누락·오류 0. 양 저장 재열기의 같은 backend PNG는 PDF 직접 출력과 모두 동일. `native/results.json`, `wasm/results.json`, `final-validation-summary.json`. |
| B17 공통 범위 | 각 backend **201건·234쪽 PNG 전부 동일**. `native-base-comparison.json`, `wasm-base-comparison.json`. 최신 upstream/devel 최종 비교는 P5다. |
| 새 셀 선택·보류 경계 12개 × 세 형식 | 각 backend **36쪽**, 최저 **98.08227% / 98.27632%**. 정상 두 병합은 3셀, 숨긴 공유 변은 4셀로 실제 IR에 생성한다. 나머지 미충족 clip/내용 근거는 사유와 대안을 유지한다. |
| 동일한 편집·저장본의 독립 Print | Native/실제 WASM 저장본 각 **18쪽**, 최저 **94.26318% / 94.15814%**. 정상 열기·두 저장 형식 및 문자/행열 편집 후 출력을 대조했다. `native-extra/results.json`, `wasm-extra/results.json`, `independent-print-contracts.json`. |
| 실제 Native/Studio API | 5개 정상·Form 입력의 주소/span, 내용 순서·중복/누락·셀 포함, 문자/행열 삽입·삭제·양 저장 재열기 통과. 7개 반례는 보류. `cases-final.log`, `studio-result.json`. 실제 로드 hash와 page error 0을 확인했다. |
| 기존 정식 집중 회귀 | 9개 case **41건 통과**. 각 기존 suite의 Native 실행과 결과를 `focused-results.json`·로그에 보존했다. |
| 새 정식 회귀 | `issue_7998_pdf_cell_regions` **5건 통과**, 자동 수집·`run-rust-test.mjs`·nextest 경로 사용. 새 호출을 CI에 추가하지 않았다. `new-regression-final.log`, `manifest-prepare-final.log`. |
| 형식 | `cargo fmt --all -- --check`와 `git diff --check` 통과. 제품 전체 회귀·lint·Worker/package 최종 게이트는 P5다. |

기존 producer의 성공한 PDF import → HWPX를 같은 API 검사로 읽으면 실제 Table이 없어
실패했고, 현재 producer는 같은 소유/span·내용 검사와 편집·저장을 통과했다
(`before-api.log`, `before-after-api.json`). 폰트/paint 소비의 실패 producer와 출력도
`diagnostic/`에 보존했다. 빌드 실패를 원인 FAIL로 세지 않았다.

새 회귀 첫 작성에서 아래쪽 선을 raw BorderFill의 Solid로만 가정한 항목이 실패했다.
실제 선은 셀 소유 Rectangle paint로 보존되는 경로였으므로 제품을 바꾸지 않았다.
독립 Print의 기대는 **위쪽 공유 잉크의 부재, 아래쪽 선의 실제 셀 소속·행 전체 포함**이다.
이를 최종 트리와 양 저장 재열기 관계로 보완했다. 검사 기대를 코드 helper 값이나 SVG hash로
정하지 않았고, 이미 맞는 원본 선을 native BorderFill로 강제하지 않았다.

새 검사와 fixture는 관련 Native/fresh WASM 전쪽 90% 이상 및 에이전트의 동일 배율 확대
판독 뒤 등록했다. source·기준 입력·hash·기대 구조는
[`pdf/issue7998/cell-regions.md`](../../pdf/issue7998/cell-regions.md)에 연결했다.
절대 px 핀은 추가하지 않았다. 기존 대상의 source/정상 관계 검사는 유지했다.

## 메인테이너 검토 화면과 남은 경계

검토 폴더는 `E:\tmp\issue7998-cell-region-review\index.html`이며, 폴더 안의 PDF·PNG·
실제 WASM 저장본·원문 TSV·실제 저장 셀 주소를 상대 경로로 연결하고 존재 검사를 마쳤다.
Studio 7793의 `?review=issue7998-cell-colspan`, `?review=issue7998-cell-rowspan`,
`?review=issue7998-cell-hidden-separate` 새 탭에 PDF를 열어 두었다. 1쪽 및 실제 셀 주소를
조회했다(`opened-review-tabs.json`). 메인테이너의 **시각·편집 구조 판정은 대기 중**이다.

현재 표는 PDF 페이지 내부의 page-anchored `TablePageBreak::None` 경로다. HWP 일반 표의
분할·이어받기 호출을 변경한 결과로 주장하지 않는다. paint 축 자체가 부족한 지역 가설,
L자/부분 격자·빈 구역의 의미 복원과 비표준 source paint의 일반 스타일 교체/행열 상속,
기존 본문 메트릭 및 P5 최신 devel·통합/전체 게이트는 유지한다. 새 text-clip 진단은 B19에
보존했다. P4 전체 완료·PR 준비 완료·이슈 종료로 승격하지 않으며 원격 반영은 하지 않았다.
