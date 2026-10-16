# Task #7949·#7950 — 실제 표 글자 보존과 최신 devel 통합

- 작성일: 2026-10-17 (Asia/Seoul)
- 앞 단계: [도형 채움·보이는 paint](task_m100_7949_stage6.md)
- 표 보정 제품 source: `5c9242b3bb4b4248af2ee6e7846dfadc9b7bcb0d`
- 회귀·CI source: `2ba7411b4`
- 통합 source: `337cde2865bf6c926fdcd53018d420fb19207b24`
- 통합 devel: `552514f423ec91f1b7171f6e6b0882a4efe034f2` (#7951)
- 상태: 최신 devel 통합 후 전쪽 시각·전체 회귀·필수 lint 통과. 원격 push·PR 등록 승인 대기

## 실제 회귀와 수정 근거

MOF는 배경색을 복구한 뒤 추론 표 글자가 그 배경에 가려졌다. 실루엣의 foreground 판정은
배경색과 검은 글자를 모두 내용으로 보므로 97% 이상도 글자 가시성을 입증하지 못했다.
이는 기존 차이로 이관하지 않고 이번 paint 변경의 회귀로 보정했다. 사용자 판정이 끝난
19쪽 PowerPoint 문서는 별도이며 그 판정을 MOF의 검사 대신 사용하지 않았다.

`structured_lowering.rs`의 `table_paint_layer`는 후보가 실제 소비할 원본 control의 마지막
paint layer와 남아 있는 개체의 물리 영역을 대조한다. 표가 본문 paint plane으로 이동할 때
앞면 positional 배경과 순서를 유지할 수 없으면 **표 추론만 보류**한다. 기존 원본 control과
글자는 그대로 남고 임시 style 할당은 되돌린다. 특정 PDF·색·좌표값 분기는 없다.

| 생산 → 소비 | 최종 동작·반례 |
| --- | --- |
| 원본 operator 순서 → shape/picture `CommonObjAttr` → 표 후보의 consumed control 집합 | 후보가 실제 제거할 개체에서만 마지막 source layer를 구한다 |
| 후보 bbox·남은 positional frame → `table_paint_layer` → `builder.build` 결과 | 겹치는 선행 paint가 있으면 표 승격 보류. 표 밖의 장식은 승격을 막지 않음 |
| 성공 후보 `table.common.z_order` / 실패 후보 원본 controls → 기존 Native Skia·WASM Canvas paint | 겹치는 배경이 있는 표는 원본 positional 경로로 글자 표시 보존. 일반 grid는 semantic table 유지 |

table z-order만 올리는 초기 진단으로는 plane 차이가 해결되지 않았다. 최종 보정은 공통
renderer나 표 분할/높이를 바꾸지 않는다. MOF는 semantic table 3개→1개로 줄고 2개는
positional로 유지된다. 의미 승격 개선은 [백로그](../plans/task_m100_7949_backlog.md)에 남겼다.

## 회귀의 독립 기대값과 수정 전후

자체 작성 PDF 3개는 MediaBox 300×200pt, grid 축과 각 셀의 H1/H2/B1/B2 문자를 원본으로
정의한다. 그림 배경이 grid 전/후에 있는 두 경우와 표 밖 장식만 있는 일반 표를 구분했다.
원본 PDF 자체의 Poppler 출력이 독립 기준이다. 전쪽 Native/fresh WASM 90% 이상과 확대
판독 후 `tests/cases/issue_7950_pdf_paint.rs`의 실제 CLI→PNG 검사 1건을 추가했다.
문자 위치는 생산된 IR 값으로 다시 계산하지 않고 원본 grid 내부의 글자 잉크와 채움 색을
검사한다. 글리프 모양의 완전 일치나 전체 이미지 hash를 기대값으로 고정하지 않았다.

- 같은 최종 검사 실행 파일 + `d37cc9257` producer: background-after-grid 셀의 글자 잉크
  누락으로 FAIL. 빌드 실패가 아니다.
- 같은 검사 + `5c9242b3b` producer: 3개 입력 모두 PASS. 일반 표는 semantic table 1개 유지.
- 기존 paint 검사 3건도 PASS. 각 검사 입력 bytes는 독립 sweep PDF와 같다.
- 증거: ignored `output/pr-review/issue7949/table-paint-png-contract-before-after.json`,
  `table-paint-png-contract-before.log`, `table-paint-png-contract-after.log`,
  `table-test-generated-inputs/proof.json`.

## 통합 전 제품 검증

`5c9242b3b`의 기본 Native + native-skia와 표준 Docker 기본 WASM을 실제 생성했다.
Native binary SHA-256은 `84a2cbf4e756f0709225ba021b95e9fa4c6b575e135ed6eda76647d88a43bae3`,
WASM은 `754a3495fcb314ffadc9e3a709dc4b304bc587d099827b2ed1e9df55274aa541`이다.
최신 회귀·CI source까지 제품 파일 불변을 확인했다. 실제 Studio는 같은 WASM을 fetch했고
Canvas2D로 PDF 19쪽을 표시했다. 입력 폰트 경로와 해시는 기존 font manifest에 고정했다.

| 원본 PDF 전체 | 쪽수 | Native 최저 | Docker WASM 최저 |
| --- | --- | --- | --- |
| 사용자 type002 | 19 | 99.40149%, 11쪽 | 99.40886%, 11쪽 |
| 사용자 MOF | 1 | 97.47152% | 97.52634% |
| 공개 nested-table 대조군 | 12 | 90.14609%, 11쪽 | 90.16832%, 11쪽 |
| 합성 paint·표 경계 6개 | 각 1 | 98.68496% 이상 | 97.75313% 이상 |
| 공개 Mask·고해상도 SMask·Matte 3개 | 각 1 | 100% | 100% |

41쪽씩에 미달·누락·쪽수 차이가 없다. MOF의 표 머리·마지막 행, 공개 대조군 7·8쪽의
표 글자와 배경, 합성 마스크의 투명 방향·alpha 단계·Matte 색을 실제 PNG에서 대조했다.
19쪽 양쪽 결과는 사용자 승인본 `d37cc9257`의 각 페이지와 RGB가 동일하다. 이를
`table-paint-final-accepted-visual-preservation.json`에 기록해 기존 판정을 유지한다.
2px 실루엣 100%를 보간·색상·글꼴까지 완전 동일하다는 뜻으로 쓰지 않는다.

- 전체 Rust nextest: **10,666 PASS**, 45 skip. Skia lib·placeholder·직접 PDF·새 PNG 검사 통과.
- fmt, Native Clippy, WASM32 Clippy, workspace build/all-target Clippy 모두 통과.
- 실제 기본/명시적 off WASM·Worker: 85 assertion·5 Worker 통과. 자원 회귀 113건 포함.
- CI workflow 계약 35건 통과. 새 표 PNG 명령은 Native Skia release-test/release 분기에만 추가.
  O3 job command 변경이며 event·permission·runner·fork 신뢰 경계·check 이름은 바꾸지 않았다.
  release-test 명령을 로컬 실행했다. release profile과 GitHub exact-head CI는 원격 반영 뒤 확인한다.
- 이 단계 로그·TSV는 `output/pr-review/issue7949/table-paint-final-*`, `public-mask-*`에 있다.

## 최신 devel 통합과 제출 경계

제출 준비 중 원격이 `0953598d8e`에서 `552514f423`으로 바뀌었다. #7951의 표 composer·
height 측정 변경을 충돌 없이 병합했다. renderer 제품 소스가 바뀌었으므로 기존 결과를
통합 head의 새 실행으로 표시하지 않고 Native/Docker WASM·전체 회귀·영향 출력부터
재검증했다. 통합 증거는 `output/pr-review/issue7949/integration-20261017/`에 보존한다.

통합 후 실제 결과:

- 기본 Native + native-skia producer SHA-256:
  `7def5bd1f2cad4d65633f477918953cf49772f79175a099d1682dd4d52bee055`.
- 표준 Docker 기본 WASM SHA-256:
  `6dc88d238132563e40d332c53898e96ce1272df997d099923e9ec69adf9ee35b`.
  pkg/public/Studio 실제 fetch가 같고 JS는 `894c403edf1c7746a29680cfb79e3ec9233739903194d4c0ef66362ae3212eb5`다.
- 전쪽 Native/WASM **41쪽씩 모두 90% 이상**, 누락·쪽수 차이 없음. 위 표의 값과 같다.
  재현·대조군 32쪽은 양쪽 모두 `5c9242b3b` 출력과 RGB가 동일하다. 19쪽 사용자의 판정을 유지한다.
- `cargo nextest run --locked --cargo-profile release-test --tests --features native-skia
  --test-threads 8 --no-fail-fast`: **10,680 PASS, 45 skip**, 실제 검사 407.745초.
  컴파일을 포함한 명령은 944.94초였다. 느린 baseline partition의 종료까지 확인했다.
- Skia lib·placeholder·직접 PDF·새 PNG CI 명령도 통과했다. 원본·baseline·golden을 바꾸지 않았다.
- 실제 새 WASM API/Worker 85 assertion·5 Worker, Studio PDF 19쪽·Canvas2D·오류 없음 통과.
  명시적 off API 대조는 해당 경로가 바뀌지 않아 선행 Docker off bundle을 재사용했다.
  최신 renderer의 off 시각 검증이라고 표시하지 않는다.
- 새 base에 대한 manifest·unit tier 검사, workflow 계약 35건·YAML 구문 검사 통과.
  actionlint는 설치되어 있지 않아 그 실행을 주장하지 않는다.
- 통합 source에서 fmt·Native Clippy·WASM32 Clippy·workspace build·workspace all-target
  Clippy를 다시 실행해 모두 통과했다. `lint-results.json`과 각 명령 로그에 기록했다.
  Cargo target은 기본 작업공간의 공유 `target/pr-review` 한 곳을 사용했다.
- 제출 제품·회귀·workflow source는 통합 SHA에서 변하지 않았다. 보고·공개 시각 자산만 후행한다.
  `validation-source-invariance.json`, `visual-base-merge-identity.json`, `rust-results.json`,
  `wasm-ready.json`, `browser-results.json`, `policy-results.json`, `lint-results.json`에 연결했다.

통합본의 공개 입력·같은 배율 compare/standalone overlay는
[시각 증거](../pr/assets/issue7949_7950_20261017/README.md)에 있다. 직접 작성한 입력만
Git에 포함하며 개인 PDF·폰트·TSV·JSON·생성 suite는 포함하지 않는다.
전쪽 비교는 `E:\tmp\issue7949-type002-review-20261016\index.html`과
`E:\tmp\issue7949-pdf-review-20261016\index.html`에 새 통합 결과로 갱신했다.
Studio는 `http://localhost:7798/?review=issue7949-type002-paint-337cde286&lang=ko`다.

앞선 workspace build의 디스크 부족은 소스 회귀가 아니었다. 모든 Cargo 사용 종료와 참조
없음을 확인한 오래된 `debug/incremental` 하위 25개만 정리한 뒤 재실행해 통과했다.
공유 target의 실행 파일·deps·fingerprint, 원본·PDF·글꼴·진행 중 작업은 보존했다.
정확한 목록과 전후 공간은 `disk-cleanup-ended-incremental*.json`에 기록했다.

새 사용자 PDF 원본은 공개 권한이 확인되지 않아 Git에 포함하지 않는다. 공개 PR 그림은
자체 작성 합성 PDF로 준비한다. 전체 parser 할당 증명은 여전히 false이며 추가 codec·
전체 paint 지원·배포를 완료했다고 주장하지 않는다. 원격 push·PR·CI·merge는 아직 수행하지 않았다.
