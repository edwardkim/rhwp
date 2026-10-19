# #7998 행·열 편집의 표준 경계 상속 — 첫 경계

- 시작 SHA: `08c9e701c`, 2026-10-19 다음 절편 진행 승인.
- 위반: 셀 템플릿은 BorderFill만 상속하므로 보존한 PDF 경계 그림은 새 행·열에 생기지 않는다.
  표 크기/셀 좌표 변경 뒤 페이지 조각의 오래된 owner도 편집 권한을 잃는다.
- 이번 작은 경계는 **완전한 표준 실선과 동등한 source**의 구조 편집이다. 원본 PDF의 pen과
  HWP 표준 굵기가 1 HWPUNIT 양자화 안에서 같은지, 실제 crop들의 전 구간/양쪽 반쪽이
  함께 완전한 선을 만드는지 확인한 뒤 표준 BorderFill로 옮긴다. 근접한 임의 굵기·alpha·
  위상 점선·채움 띠/가림은 같은 스타일이라고 추정하지 않는다. 이 범위는 아직 남는다.
- 입력은 stage20의 같은 2×2 부모/자식·본문 PDF를 독립적으로 재생성하고 부모 pen만
  HWP 0.15mm의 PDF point 값으로 작성한다. 삽입 공간을 확보한 600×720pt 용지와 아래 본문
  위치를 명시한다. 구현에서 특정 문서/행/숫자를 분기하지 않는다.
  `before.log`에서 새 행의 실제 BorderFill 부재 FAIL을 확인했다. 빌드 오류는 재현으로 세지 않는다.
- 실제 소비: PNG의 인증된 PdfStroke + 현재 physical crop/owner → 공통 edge ink 증명 →
  표준 BorderFill → 기존 Table::insert_row/insert_column/delete_row/delete_column → 새 셀의
  Cell::new_from_template 및 기존 renderer/serializer. 셀/문단/자식 내용은 복제하지 않는다.
- 기존 실선/점선 외형과 편집 의미의 동등성을 구분한다. 이번에 증명하지 못하는 tagged
  source는 구조 편집 전 오류로 거절한다. 준비/실제 model edit가 실패하면 원문·스타일·
  이벤트를 변경하지 않는다. 일반 native 표의 기존 동작은 그대로 사용한다.
- 표준 승격은 원자적 candidate parent와 새 BorderFill 목록에서 계획한다. 실제 Studio의
  기존 snapshot 명령이 원래 paint/형식까지 undo/redo한다. parser의 원문 import는 바꾸지 않는다.
- 합성 진단은 ignored `output/pr-review/issue7998/p4-border-structure/`에 둔다. 새 정식 회귀는
  Native/fresh WASM과 독립 한컴 편집 Print의 90%·직접 판독 뒤 추가한다.
- 전체 회귀/lint·Worker/package·devel 통합은 P5에 남긴다. 원격 작업은 하지 않는다.

## 선행 진단

- stage20 producer에서 새 행의 테두리 부재 FAIL. 같은 입력에서 4방향 삽입·2방향 삭제,
  양 저장·원본 snapshot 복원은 첫 보정 Native에서 통과했다.
- 개별 crop의 역투영은 정수 HWPUNIT 때문에 양쪽으로 오차를 가질 수 있다. 조각의
  계산값끼리 비교하는 잘못된 가정을 제거하고, 각각 독립 표준 pen과 1HU 안에서 대조한다.
  coverage의 좌표 양자화도 1HU로 제한하며 그림의 위치/폭을 clamp하지 않는다.
- 기존 62개 집중 함수는 첫 보정에서 통과했다. source PDF의 Native PNG는 stage20과
  SHA-256이 같았다. 이 결과는 후속 색상 보정의 최종 증거와 구분한다.
- 추가 유색 대조군이 잘못된 byte swap을 검출했다. `PdfStroke.color`는 이미 ColorRef다.
  공통 값을 그대로 소비하도록 수정하고 이전 유색 출력은 `before-color/`에 보존한다.
  중간 WASM 빌드는 종료했고 최종 source로 새로 빌드한다. source SHA가 바뀐 중간
  capture를 최종으로 표시하지 않는다. Native/Print의 검정 입력 바이트가 같으면 해당
  독립 Print는 재사용하되 최종 Native/fresh WASM의 actual export는 구분한다.
- 검정 입력 Helvetica의 한컴 대체와 Arial을 명시한 파란 선 대조군을 구분한다.
  유색 입력은 원본/양 저장의 실제 색상값과 출력으로 검증한다.

## 최종 source 검증 — 시각·편집 구조 통과

2026-10-19 메인테이너가 표준 실선 경계의 행·열 상속 범위를 **“시각·편집 구조 통과”**로
수용했다. 아래 R2의 기존 문자 역삭제 원점 차이와 비표준 source 상속은 이번 수용 범위에
포함하지 않으며, 이전 판정과 빌드·캡처를 반복하지 않는다.

| 실행 경계 | 최종 결과와 증적 |
|---|---|
| 원본 PDF 2개 + 독립 한컴 편집 Print 14개 | Native/fresh Docker WASM **각 16입력·16쪽**, 최저 **96.57867% / 96.62863%**. 미달·측정 불가·누락·쪽수 차이 0. 모든 `document_page_counts`를 확인했다. `--silhouette-only`의 gate는 `not_evaluated`이며 수치를 자동 승인으로 해석하지 않는다. `final-validation-summary.json`. |
| 구조 편집 저장본만의 한컴 Print | 4방향 삽입·원래 두 번째 행/열 삭제·유색 행 삽입의 양 형식, 총 14개. Native 최저 **98.66183%**, WASM **97.15880%**. 파란 편집본 두 형식은 Native **100%**, WASM **98.79160%**. |
| 실제 Studio | 최종 WASM에서 7동작, 문자 삽입·역삭제 뒤 SnapshotCommand의 undo/redo, 양 저장·재열기 통과. 최초 구조 편집 출력 14개는 Native API와 바이트가 같아 같은 입력의 Native/WASM/Print 증거를 재사용한다. 14쪽을 별도 합산하지 않는다. `studio-result.json`, `actual-identical-inputs.json`. |
| Native focused | 최종 source의 기존 정식 함수 **62 passed / 0 failed**. `existing-guards-final.log`. 신규 정식 `regression_suite_026`의 10함수도 **10 passed / 0 failed**다(`formal-tests.log`). 총 **72개 고유 함수**이며 진단/반복 실행은 중복 합산하지 않는다. |
| 변경 전 FAIL | stage20 producer에서 같은 입력의 새 행에 실제 BorderFill이 없음(`before.log`). 색상 probe는 byte swap을 별도로 검출했고 최종 `color-fixed.log`에서 독립 PDF의 #336699/ColorRef 계약을 확인했다. 빌드·도구 사용 오류를 결함 재현으로 세지 않는다. |
| 정상 원본 보존 | 검정 PDF를 여는 Native PNG는 stage20과 SHA-256이 같다. 마지막 색상 수정 뒤 검정 저장본 12개도 byte-identical(`unchanged-black-edits.json`)이므로 해당 독립 Print는 재사용했다. 최종 Native/Canvas 캡처는 새 producer로 산출했다. |

Native 후보 SHA-256:
`486257c6f18f106754bbc6941d26834fc6689ffe3c1bfba8c33a8139963190ab`.
Fresh Docker WASM은 **19,592,260바이트**, SHA-256
`3a218322f4eca7bf36ca5b6c7abe89932fcc2111f64835bd989ed16de836a427`.
JS glue는 `5671802fbb89ed060c3bab7a6f458a78833a7e5ca3e7df8e1699d45098cddeb6`로
기존과 같다. runtime 4파일의 SHA는 `source-runtime.json`에 고정했다. 추적 중인 Studio
JS/types는 변경하지 않았다. 사용자 CDP의 새 4탭도 같은 WASM과 1쪽 표시를 확인했다.

대표 compare·standalone overlay·review는 최종 PNG를 입력으로 canonical `visual_sweep`
합성 함수를 실행해 `reviews/`에 둔다. 전체 TSV·입력/양 저장·한컴 Print·선 접점 확대는
`/mnt/e/tmp/issue7998-border-structure-review/index.html`에서 상대 경로로 이동할 수 있다.
새 셀의 네 변, 부모/자식 접점과 앞뒤 문장을 같은 배율로 직접 대조했다. 가는 선의
PDF/Skia/Canvas 농도와 검정 Helvetica의 한컴 대체 글꼴 차이는 화면에 표시한다.
원본·기준 PDF는 `pdf/issue7998/structure-borders/`의 재생성 script/provenance에 연결한다.
SVG는 이번에 별도 캡처하지 않았다. 실제 변경 소비의 Native/Canvas와 한컴 저장 경로를 검증한다.

### R2의 기존 문자 역삭제 원점 — 이번 범위와 구분

실제 SnapshotCommand의 redo 비교 기준은 **undo 직전의 현재 문서**다. 최초 구조 편집
출력과 무조건 같은 바이트여야 한다는 초기 진단 가정은 잘못됐다. 원본 복원과 현재 상태의
redo는 통과했다. 단, 첫 문장에 X를 삽입하고 직접 역삭제하면 저장 LineSeg가
`horzpos=1000→0`, `baseline=950→850`으로 재구성되고 실제 글자의 위치도 달라진다.
`row-below-text-normalization.json`과 `normalization-native/`의 최종 출력에서 확인했다.
문단·표만 순회한 첫 진단도 바깥 글상자 본문을 놓쳤으므로 실제 Shape text box까지 순회해
내용 보존을 검사했다. 구현의 검사를 피하려고 기대한 본문을 제거하지 않았다.

이 원점 차이는 **stage20 WASM** SHA-256
`eb2a741437f4ba3a2131012be95a1cb1893dc5f2be83ad1fdf38c3cb3c15972c`에서 같은 표준 HWPX를
로드해 구조 API를 전혀 호출하지 않고 문자 삽입/삭제만 실행해도 같은 값으로 재현된다.
`normalization/old-wasm.json`과 `row-below-old-wasm.hwpx`가 근거다. 이번 구조 helper가 만든
회귀로 분류하지 않는다. 기존 계획의 R2(B10–B13) 입력 속성/저장 행과 재조판의 수평 메트릭
계약에서 처리하며, 문자 역삭제의 배치 보존까지 통과했다고 보고하지 않는다. 이 추가
진단은 위 16쪽 수용 범위와 별도로 표시했고 해당 R2의 최종 한컴/양 backend gate는 남아 있다.

전체 회귀/lint·Worker/package·최신 devel 통합은 P5다. 임의 굵기·phase/alpha·채움 띠의
일반 상속과 병합/지역 축 등 R1의 남은 경계도 유지한다. P4 전체·PR 준비 완료 또는 비신뢰
입력 배포 준비 완료를 주장하지 않으며 `parser_allocation_bounds_proven=false`다.
원격 push·댓글·PR 변경은 수행하지 않았다.

정식 수집 실행:

```bash
CARGO_BUILD_JOBS=14 cargo test --locked --profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia \
  --test regression_suite_026 issue_7998_pdf_border_structure -- --test-threads=4
```

자동 수집이 배정한 suite를 사용했고 CI에 개별 호출을 추가하지 않았다. 새 회귀의 독립 입력과
Print는 위 전쪽 Native/fresh WASM 90%와 선 접점·본문 직접 판독 후 등록했다. 기존 기대값은
변경하지 않았다. `source-runtime.json`의 제품 SHA가 같으므로 입력/테스트/보고서 추가 뒤
같은 제품을 재빌드하거나 전체 검증을 반복하지 않는다. 메인테이너의 직접 판정을 반영해
이번 표준 실선 상속 절편을 마무리한다. 다음 작은 경계는 비표준 source의 행·열 상속이며,
R2와 P5의 남은 범위는 유지한다.
