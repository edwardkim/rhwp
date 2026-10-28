# #7998 stage29 — 공백 메트릭·실제 셀 테두리와 저장 배치

2026-10-20 승인된 다음 절편과 **“이번 절편에서 테두리 저장 배치도 수정”**을 처리했다.
기준은 `6563a5662b30bbeb62b899ae6994634641c551b8`다. 메인테이너가 지적한
`horizontal-wide`·`horizontal-noop`의 선택 가능한 사각형을 실제 셀 BorderFill로 옮겼다.
새 wide·noop·row·column 네 Studio 탭은 **시각·편집 구조 통과** 판정을 받았다.
추가 반쪽 clip의 기존 선 농도 차이는 별도 **B25/이번 범위 예외**로 수용했다.
전체 R1/R2/P4/P5 완료나 PR 준비 완료가 아니며 다음 절편·원격 작업은 실행하지 않았다.

## 규칙·원인과 실제 소비

| 구현 주장 | 생산 → 공통 소비와 최종 경로 | 적용·반례 및 독립 기대 |
|---|---|---|
| 공백 전진폭 | `structured_lowering::observed_line_text` → `fit_spacing/fit_whitespace_advance`의 실제 `estimate_text_width_exact` → CharShape ratio/spacing → Paragraph/LineSeg → Native/WASM·양 저장 | 공백만 법정 ratio 50–200, spacing −50–50의 합성으로 표현한다. ArialMT의 독립 Widths와 TJ/Tw 15·2.88pt를 검사한다. 실제 ink glyph/합성 범위 밖 27pt는 보류하며 clamp·반복 공백으로 맞추지 않는다. |
| 셀 위 사각형 제거 | `native_borders::plan`의 원본 `Rectangle`/경계 관측 → 변별 실제 pen·coverage/소유 closure → `lower`가 같은 셀 BorderFill을 설정하고 해당 source control만 소비 → 실제 셀 테두리 paint/행열 template/양 저장 | 원래 네 `m/l/S` 경로와 같은 닫힌 `re S`도 표준 실선으로 편집한다. 각 변의 실제 ink 영역을 검사해 내부 글자를 가림으로 오인하지 않는다. fill·alpha·dash·곡선·잘린 clip·가림·다른 owner는 단순 bbox로 승격하지 않는다. 기존 표준 HWP pen bucket 투영은 유지한다. |
| 비표준 source frame의 저장 원점 | 실제 `plan.cells` 벡터와 행열 band → `prepend_cell_paint`의 공통 slot rebuilder → 문자/글자모양/range/LineSeg·field control index → 한컴 첫 줄 anchor와 Native `para_y_before_compose` | 셀에 실제 배정한 paint만 글자 앞에 둔다. 일반 내용 그림·중첩 표·page/outside paint를 bbox로 이동하지 않는다. compound source의 실제 행열 편집을 별도로 검사한다. |
| 앞 삽입·새 빈 셀의 frame 소속 | 명시적 `CommonObjAttr.cell_frame_paint` → 실제 `insert_text_at_caret`·`replace_text_keeping_slots` → 첫 slot 유지 → RBPO v6의 owner/크기/digest/고유 ID 검증 → 양 저장 재열기 | 테두리 편집 side tag와 frame 역할은 별개다. 반투명 닫힌 그림의 frame-only는 변 편집 권한을 만들지 않는다. 기존 v1–v5 읽기는 유지하고 geometry만으로 새 역할을 추정하지 않는다. |
| 실제 여러 줄 입력 | PDF paragraph 생성의 정상 BREAK → 저장 LineSeg와 실제 재조판 → Native/fresh WASM·한컴 Print | 기존 SQUEEZE 강제는 한컴에서 긴 새 문장을 한 줄에 겹쳤다. observed source 줄은 LineSeg에 남기고 편집 문단은 BREAK를 사용한다. source 줄을 지우거나 빈 문단·높이/원점 보정으로 맞추지 않는다. |

새 분기·메트릭에는 문서 ID나 특정 sample 수치를 사용하지 않는다. 원본 관측 좌표와
최종 측정의 적용 단위는 pt/HU/96DPI를 명시한다. native BorderFill은 기존 일반 셀
배치를 소비하며 이번 변경은 pagination/continuation 컷·rowspan 분할 예약을 바꾸지 않는다.
미지원 일반 B24 격자와 다쪽 continuation 증명을 완료했다고 확대하지 않는다.
field-aware 분기의 frame 제외도 구현했으나 **실제 Field와 frame을 함께 둔 별도 실행은
미검증**이다. PDF frontend는 Field를 발행하지 않는다. 제품 6개 소스는 최종 빌드 뒤 바꾸지 않았다.

## 실패 증거와 철회한 중간 결과

- `final-verified/`: stage28과 R2-only가 만든 일반 간격 HWPX는 바이트 동일하지만,
  한컴은 글자 뒤의 frame을 마지막 줄에 붙여 Print/Native **89.93255%**였다. 수동
  slot-only 진단본은 원래 그림·크기·문단을 유지하고 첫 slot으로 옮겨 차이를 분리했다.
  수동 진단 통과를 제품 완료로 대신하지 않는다.
- `anchor-marker-experiment/`: 편집 side tag만 frame으로 간주한 접근은 집중 4건 실패로
  철회했다. 최종 구현은 생산자가 배정한 실제 paint와 명시적 frame 역할을 사용한다.
- `anchor-integrated/`: prepend만으로는 앞 문장 삽입 때 frame control 위치가 **0→53**으로
  이동했다. 최종 실제 편집은 PDF/HWP/HWPX 모두 **0 유지**이며 내용/소속을 함께 검사한다.
- `anchor-persistent/`: frame 역할만 고친 새 빈 셀의 긴 입력은 한컴 SQUEEZE로 겹쳐
  **79.37960%**였다. 같은 저장본의 줄 나눔 속성만 BREAK로 바꾼 독립 Print로 원인을
  분리한 뒤 제품 producer를 고쳤다. 원본과 실패 Print를 보존한다.
- `anchor-wrap/`: 숫자상 75입력·86쪽 통과 뒤 메인테이너가 사각형 overlay를
  **편집 구조 미달**로 판정했다. 최종 통과로 재사용하지 않는다. 같은 다섯 원본 첫
  셀의 control 수는 **2→0**이며 `overlay-before-after.json`에 정확한 이전/현재 입력과
  실제 모델을 연결했다. 새 행·열의 모든 셀도 control 0개·표준 테두리 속성이다.
- `before-joint-metrics/`: 고정 stage28 CLI에서 세 공백 양성 입력의 실제 문단 연결이
  보류되는 FAIL을 확인했다. 새 Rust 검사 전체를 옛 rlib로 재컴파일했다는 주장은 아니다.
  source slot·Frame·BREAK·사각형 개체 검출도 각각 실제 이전 실패와 현재 검사를 구분한다.

## 최종 제품과 검증 범위

최종 증거는 ignored `output/pr-review/issue7998/p4-horizontal-metrics/anchor-native-borders/`
→ `/mnt/e/tmp/issue7998-stage29-evidence/anchor-native-borders/`다. 다른 중간 후보의
TSV·Print·runtime을 최종 제품 검증으로 대신하지 않는다.

- Native: `6ada0e34735386beccf1d9ce9f7a15724680242bd3a0e80fd5f7cdd8d353367b`.
- fresh Docker WASM: `4a37b3aae7cb021f96a49925f848e25f291c1bafb23939060dd5989c23fa8ef0`.
- JS glue: `5671802fbb89ed060c3bab7a6f458a78833a7e5ca3e7df8e1699d45098cddeb6` — 공개 API/glue는 이전과 같다.
- `source-runtime.json`에 제품 6개 source의 SHA를 고정했다. 공유
  `/home/edward/mygithub/rhwp/target/pr-review`를 재사용해 jobs 14·release-test Native와
  canonical Docker WASM(codegen-units 16)을 빌드했다. pkg/public/실제 Studio 로드 SHA를 대조했다.
- 각 case의 **원본/기준 PDF SHA·정확한 명령·전체 TSV·쪽수/누락**은
  `{native,wasm}/results.json`과 `visual/<key>/{silhouette.tsv,silhouette_manifest.json}`에 있다.
  compat 2022·96DPI·Arial 공급 조건이다. silhouette-only의 `not_evaluated`를 사람 승인으로
  부르지 않으며 전체 raw·직접 판독·시각 예외를 구분한다.

| 최종 범위 | Native | fresh WASM |
|---|---:|---:|
| 전체 측정 입력·쪽 | 87개·98쪽 | 87개·98쪽 |
| 승인된 B25 예외 외 86개·97쪽 최저 | 96.47823% | 96.47194% |
| B25 반쪽 clip 1쪽 원시 미달 | 76.04946% | 76.21202% |
| 누락·전체 쪽수 차이 | 0 | 0 |

전체 원시 최저는 B25이며 **raw gate 미달을 통과로 바꾸지 않는다**. 메인테이너의 명시적
예외는 이번 사각형 제거 범위에만 적용한다. 반쪽 clip은 stage28에서도 Native 74.63931%·
fresh WASM 74.87277%였다. 기존 그림 선이 너무 옅은 출력 결함이며 표준 테두리 승격을
거절한 사실과 다르다. 입력·실패 TSV·이전/현재/기준 PNG를 보존하고 정상 rendering
fixture로 등록하지 않는다. [B25](../plans/task_m100_7998_backlog.md)에 후속 조건을 남겼다.

- 동일 실제 저장/편집 **HWP/HWPX 42개 각각**의 독립 한컴 2020 Print를 확보했다.
  원본 용지·100%·method 0·한 쪽씩·11.0.0.9136이다. 20개는 이전 출력과 입력 바이트/
  SHA까지 같아 Print를 재사용했고, 바뀐 22개만 새로 제출했다. `reused-print-identity.json`과
  job별 immutable 입력/요청·PDF가 출처다. 자동 Print 성공을 사람이 42개를 열어 확인한
  결과로 바꾸어 보고하지 않는다. 이전에 장애가 났던 서비스 입력은 중복 제출하지 않았다.
- 실제 Studio에서 세 공백 입력의 중간 삽입·직접 역삭제·Snapshot undo/redo, compound 및
  표준 테두리의 행열 삽입, 반투명 frame의 앞 문장 입력 4줄·새 빈 셀 입력 3줄과 양 저장
  재열기를 실행했다. 결과 저장 바이트가 위 Native Print 입력과 같고 JS 오류는 0이다.
  `studio-*-result.json`·`verify_*.log`에 연결했다. 7700과 기존 사용자 탭은 보존했다.
- stage28 대비 같은 원본 source **56쪽**은 각 backend의 전체 TSV/PNG로 비교했다.
  Native PNG 38쪽이 같으며 견적서의
  raw −0.00342%p는 compare/이전 PNG에 남겼다. 보고서 5·6·7쪽과 새 간격 문단은 개선됐다.
  fresh WASM의 변화와 하락은 `wasm-base-comparison.json`을 그대로 참조한다. 최신 devel
  통합 전 P5 비교와는 구분한다.

## 직접 판독·정식 검사와 입력 등록

2026-10-20 메인테이너는 `issue7998-native-border-{wide,noop,row,column}` 새 네 탭의
**시각·편집 구조 통과**를 판정했다. 정확한 입력·WASM SHA·범위와 B25 예외는
`maintainer-disposition.json`에 있다. 비교 화면은
`E:\tmp\issue7998-native-cell-borders-review\index.html`이다. 같은 위치·배율의
Native/fresh WASM compare·standalone overlay·review, 전체 TSV와 직접 열 파일을 제공한다.

직접 판독에서는 두 source 줄/문자 사이 간격·Noop 표 외곽·빈 새 행/열의 테두리·
내용 순서/캐럿 진입·앞/빈 셀의 긴 입력 줄바꿈·뒤 본문을 대조했다. 새 일반 사각형은
선택 가능한 셀 overlay가 없으며, source/Noop과 실제 행열 편집 모두 표준 셀 속성이다.
Native의 글꼴 smoothing·표준 pen 투영/가는 선 농도, 한컴 Print의 후속 본문 기준선
미세 차이는 compare에 그대로 표시한다. 반투명 frame·가림·compound는 별도 반례이고
일반 사각형과 같은 편집 속성이라고 주장하지 않는다. B25는 직접 미달로 구분한다.

등록 대상 독립 공개 PDF **14개**는 Native/fresh WASM 원본과 관련 실제 양 저장/편집본의
전쪽 90% 이상·직접 판독 뒤 등록한다. `pdf/issue7998/horizontal-metrics/generate.py`는
같은 PDF 바이트를 별도 폴더에서 재현하며 글꼴 프로그램/견적서/실행 로그를 포함하지 않는다.
현재 report 원본 `pdf/pr7704/6778-square-2020.pdf`는 사용자 `content_operators-002.pdf`와
SHA가 같은 **한 원본**이다. 두 독립 문서로 중복 집계하지 않는다.

새 rendering 회귀는 기존 자동 수집 `issue_7998_pdf_table_inference` module에 추가한다.
공백 한 문단/두 줄·원문 유일 소유·최종 전진폭/셀 내부 포함·뒤 본문, 개체 0개/실제 테두리,
양 저장, 실제 행열·문자 입력의 frame 소속을 검사한다. ±2px는 양쪽 raw 98% 이상인
해당 완료 쪽의 독립 ArialMT 전진폭 관계에만 사용하며 전체 SVG 해시/실물 절대 px pin을
추가하지 않는다. v2–v5 wire compatibility는 기존 source를 사용한 codec 계약 검사로
구분하며 이를 새 한컴 배치 증거라고 부르지 않는다.

기존 module의 변경은 새 의미와 실제 저장 구조에 맞춘 것이다. source slot이 앞에 추가된
LineSeg는 slot 길이를 함께 반영한다. 새 공백 수용 본문은 한 문단의 원문 순서로 검사한다.
이미 stage19에서 보정된 부분 배경의 옛 15400/8400 면적 pin은 원래 cell fill/소유 관계로
바꾼다. 프로토콜 거절은 실제 v6 flag/record stride와 frame 권한 초기화까지 검사한다.
미지원 hidden/L자 구조 편집은 원문 변경 전에 거절함을 검사하며 과거 성공을 강제하지 않는다.
정확한 내용·source 속성 계약은 유지하고 기준값/90% 관용치를 완화하지 않는다.

최종 정식 집중 검사·자동 수집 확인·변경 파일 rustfmt 결과는 아래 마감 기록에 연결한다.
기존 `cargo fmt --all -- --check`의 `commands/mod.rs` 모듈 순서 문제는 HEAD에도 존재한다.
범위 밖 파일은 바꾸지 않으며 이를 전체 fmt 성공으로 보고하지 않는다.

## 다음 경계

보고서 셀의 메트릭 보류 줄은 **17→2**, 견적서는 0줄이다. 남은 6쪽의 ink glyph 장평
한도·7쪽의 합성 한도 밖 공백과 본문의 별도 제안 수를 더해 미완료 문단 수로 부르지 않는다.
직접 문자 역삭제의 저장 줄 원점은 다음 R2 경계이며 B24/R1 일반화·P5도 남는다.
RBPO는 기존 4MiB와 레코드 예산을 유지하며 보고서 3,033,388바이트/43,904레코드,
견적서 17,124바이트다. 작은 반례의 통과를 전체 allocation proof로 확대하지 않고
`parser_allocation_bounds_proven=false`를 유지한다. 전체 suite·전체 lint·Worker/package·
최신 devel 병합과 최종 PR 게이트는 이번 절편에서 실행하지 않았다.


## 마감 — 2026-10-20 로컬 절편 수용

- 실제 등록한 기존 20개 case module을 최종 코드의 Native rlib에 연결한 집중
  **155건 통과**(`focused-registered.log`). 새 9개 함수는 같은 자동 수집 module에 있으며
  임시 focused harness나 개별 CI 호출을 저장소에 추가하지 않았다. 이 실행은 전체
  regression suite가 아니다. 테스트 컴파일 경고도 0이다.
- 자동 suite를 canonical `--prepare`로 갱신한 뒤 `--check`를 통과했다. 기존 28 suite와
  20 exception/48 target 경로를 유지하며 policy·Cargo target·CI workflow는 바꾸지 않았다.
- 최종 공개 fixture 14개의 PDF 바이트 재현과 양 backend 90% 이상은
  `generator-byte-identity.json`·`fixture-qualification.json`에 source/runtime SHA로 연결했다.
  B25는 전쪽 원시 실패/예외 기록에 남고 새 fixture에는 포함되지 않는다. 낮은 쪽을
  지워 전체 gate가 통과한 것처럼 보고하지 않는다.
- 새 경계·문자/구조 편집의 형식별 TSV와 대표 비교 화면을 직접 확인했다. 최종 gallery의
  로컬 링크도 확인했다. 실제 메인테이너의 사각형 제거 수용과 B25 예외를 각각 기록했다.
- 마지막 집중 검사 연결 중 다른 PR #7930의 동시 Cargo 빌드가 공유 target의 같은 이름
  backend를 교체했다. 누락 rlib/기존 API 불일치는 제품 회귀와 구분했다. 그 빌드는
  중단하지 않았고 target을 삭제/초기화하지 않았다. 원본 Native/WASM은 E 드라이브에
  이미 고정돼 있어 전쪽 캡처에 영향이 없다. 제품 소스는 그대로 두고 로컬 vendor
  `src/lib.rs`의 시각만 갱신해 원래 root native-skia Cargo 경로로 올바른 의존성을
  재생성했다. 패키지 단독 선택은 다른 dependency metadata가 되어 검증에 쓰지 않았다.
  복구 rlib는 원본 rlib와 바이트 동일하다고 주장하지 않는다. 등록 검사 연결 전후
  rlib/backend 지문이 유지되고 제품 6개 source·served WASM은 기존 증거와 같음을
  `registered-validation.json`·`shared-build-collision.json`에 남겼다. 이미 끝난 시각 캡처는
  재실행하지 않았다.
- 변경 Rust 파일의 rustfmt·`git diff --check` 결과와 로컬 커밋을 이번 절편에 묶는다.
  원격 push·PR 생성/변경·GitHub 게시·다음 절편은 수행하지 않았다. R2 두 줄·역삭제
  원점, B24와 P5의 남은 완료 조건은 위 계획/백로그에 유지한다.
