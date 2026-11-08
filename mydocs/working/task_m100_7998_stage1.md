# #7998 1차 구현과 관측 검증

- Issue: [#7998](https://github.com/edwardkim/rhwp/issues/7998), 담당 edwardkim
- 기준 devel: `7d01ef610f378157b79d36ef23e8ed4a3c8ab051`
- 제품 코드: `1d54f8a09d3242d17a54df4d6b5b3a8a7859b4e4`
- 계획: [수행·구현계획](../plans/task_m100_7998_impl.md), [세부 후속 일감](../plans/task_m100_7998_backlog.md)
- 상태: 제품 관측→후보→기존 recognition/lowering→공통 IR/positional fallback→진단/로컬 검토 화면을 연결했다.
  전쪽 Native/fresh Docker WASM 비교와 영향 영역 직접 판독을 마쳤다. 기존 시각 미달이 남아
  `re_review_required`이며 PR 준비 완료·전체 표 복원 완료가 아니다.
- 새 이슈 등록만 원격에 반영했다. source push·PR 생성·추가 GitHub 댓글은 수행하지 않았다.

## 1. 구현과 소비 경로

backend의 실제 clip 종료 지점에서 paint 없는 `W … n`도 별도 장부에 남긴다.
`extract.rs`가 문자/그림·path·clip을 관측하고, `table_inference.rs`가 backend 문자 ID·T0·
구역·지역/전역 격자·내용 소속·공유 변의 근거를 연결한다. `table_detection.rs`는
기존 recognition과 TableBuilder가 소비하는 geometry snapshot을 만든다. 확정하지 못한 후보는
원래 paint와 위치를 보존한다. 단순히 진단 Python/POC만 수정한 것이 아니다.

`PdfImportOptions.include_table_inference`, WASM의 `includeTableInference`, CLI의
`import-pdf ... --json --table-inference`로 상세 장부를 요청한다. 기본 metadata는
후보 수·검출/생성 보류 사유의 요약이다. 상세 자료에는 원본 문자가 있으므로 로컬에서 보관한다.
검토 도구 `scripts/pdf_table_review.py`는 원본 PDF를 배경으로 각 레이어와 후보를 조회한다.
추론 자체에 OCR이나 래스터 특징을 사용하지 않는다.

현재 최종 IR은 paint 기반 `tableDetection.tables`를 기존 recognition/TableBuilder로 소비한다.
새 clip·문자 간격·전역 비교 후보와 T0/T1은 진단 가설이며, 선택한 가설을 Builder 입력으로
승격하는 부분은 B03에 남았다. 정상 닫힌 paint 격자의 끝까지 연결된 실행과 새 지역 가설의
최종 소비를 혼동하지 않는다. 따라서 피드백 M0–M3 전체를 완료했다고 판정하지 않는다.
문자 ID는 backend가 추출한 Unicode span/character의 페이지 내 식별자다. 원본 PDF의
font GID·개별 operator 식별자를 완전히 보존했다는 의미가 아니며, backend가 넣은 의미 공백과
근사 glyph box의 구분·출처 세분화는 B05에 남는다.

## 2. 실제 입력에서 확인한 원인과 변화

| 대상 | 수정 전 실행·원인 | 1차 구현의 관측 |
|---|---|---|
| 원래 PDF 5쪽 | backend는 5,184개 stroke를 보존하지만 검출기는 약 0.48pt 조각을 `길이 > 0.5pt` 필터에서 제외해 28개 edge만 소비했다. 이 입력에서 512개 cap이 직접 원인은 아니었다. | 5,184개 원본 edge와 35개 정리된 경계를 구분한다. 위 6×3·아래 5×4 후보를 구성하며 raw 지지율과 점선 carrier를 구분한다. |
| 원래 PDF 1쪽 | 의미 표 0개. 단일 행 후보를 제품 metadata에서 확인할 수 없었다. | 클립/내용 기반 단일 행 후보와 전역 비교의 축 출처·충돌을 조회한다. 후보의 셀 수를 작성 당시 정답으로 확정하지 않는다. |
| 원래 PDF 3쪽 | 의미 표 1개. 원본 단서와 별개로 paint 없는 변을 자동 연결했다. | 연결된 7 face와 분리 대안을 남기고, 병합 근거가 미확정인 경우 의미 표 생성은 보류한다. 따라서 이 입력의 의미 표 수는 현재 1→0이다. 출력 보존 및 실제 편집 기능 범위를 시각 검증과 별도로 판단한다. |
| Excel 견적서 | 의미 표 0개·lowering 보류 1개. | stroke 55개와 fill 띠 69개를 별도 출처로 보존하고 클립·지역/평면 후보·공유 경계 대안을 조회한다. 아직 작성 당시 표나 중첩 객체를 복원했다고 주장하지 않는다. |
| 정상 합성 대조군 | 2×2의 닫힌 경계와 A/B/C/D를 원본 PDF stream에 독립 선언했다. | 의미 표 1개·보류 0개. 원본 PDF를 기준으로 실제 출력 비교를 이어간다. |

견적서가 Excel 작성이라는 사용자 확인은 출처로 기록했다. 과거 실험의 106/17셀을
판정 상수로 사용하지 않았고 원본 실험 JSON도 변경하지 않았다.

## 3. 집중 검증과 미검증

정식 자동 수집 `regression_suite_009`에서 새 검사 5건이 통과했다.
paint 없는 clip, 흰 글자 보존, 문자 ID·T0 단일 소속/결정성·선택 상세 진단,
짧은 선분 보존·반복 carrier와 실제 paint 구분·중복 paint의 축 불변성 및 정상 의미 표의
HWP/HWPX 저장·재열기 후 네 셀 내용 단일 소유를 확인했다. 마지막 관계형 검사는 정상 대조군의
선행 시각 조건과 직접 판독을 확인한 뒤 추가했으며 절대 px·SVG hash를 고정하지 않았다.
원본 PDF의 xref/stream 바이트는 해당 파일의 `-text -diff` 속성으로 보존한다.
이 결과는 조판 일치나 모든 구조 가설의 정답을 입증하지 않는다.
새 API가 없는 base 전체에 이 검사를 그대로 실행한 FAIL을 원인 재현이라고 주장하지 않는다.
원래 길이 필터의 실행 증거는 수정 전 실제 WASM/추가 관측과 동일 입력의 edge 수 변화다.

기존 PDF frontend 15건과 자원 계약 123건도 통과하여 focused Rust 검사는 총 143건이다.
실제 fresh WASM public API/Worker 검사는 123 assertion·실제 Worker 7개, 12쪽 render/save,
잘못된 옵션·취소·오류 복구·active document 격리를 통과했다. 요약 및 opt-in 상세 진단은
같은 실제 Worker 실행에서 확인했다. 테스트를 추가하면서 suite가 021에서 009로 이동했으며,
이전 shard를 지정해 0건 실행한 로그는 통과 건수에 포함하지 않는다.

Rust 기본 check·진단 CLI build와 Native Skia build를 통과했다. 표준 Docker WASM은
고정 제품 소스에서 8분 9초에 완료했으며, 기존 `rhwp_wasm-target` volume과 공용
`/home/edward/mygithub/rhwp/target/pr-review`를 재사용했다. WASM SHA-256은
`c9d30c4d6bc27aad9c7da3f74a61cccddc66816b42b53d4d91ddbd2931c0aab2`다.
pkg/public의 JS·WASM·type 네 파일이 각각 일치한다. 추적 중인 public JS의 5줄 변경은
새 boolean 옵션용 wasm-bindgen 생성 binding으로 이번 코드에 필요하며 임의 수정이 아니다.
이후 변경은 검사·fixture·기록뿐이며 제품 Rust source hash가 같아 빌드와 캡처를 재사용했다.

Studio 7793의 실제 browser에서 이 WASM URL/바이트/SHA와 두 원본의 12·1쪽,
Canvas 생성 및 page error 0건을 확인했다. primary Studio 7700은 그대로 보존했다.
전체 lint·전체 integration·PR 전 검증은 아직 실행하지 않았다.

## 4. 전쪽 시각 비교의 현재 경계

정식 비교 조건은 원본 PDF, Print, compat 2022, 96 DPI, Native Skia/Canvas PNG와
기준 `pdftoppm -thinlinemode shape`다. baseline producer는 정확한 devel source에서 따로
보존했으며 이전 source의 바이너리를 덮어쓴 뒤 기준 출력으로 바꾸어 보고하지 않는다.

| 대상 | devel Native → 후보 Native | devel WASM → fresh 후보 WASM |
|---|---|---|
| 원래 PDF 3쪽 / 문서 최저 | 89.22072 → 89.65203% | 90.33455 → 90.76350% |
| 견적서 1쪽 | 72.76381 → 72.76381% | 72.20651 → 72.20651% |
| 독립 정상 2×2 대조군 1쪽 | 100.00000 → 100.00000% | 99.53052 → 99.53052% |

실물 13쪽의 같은 backend 비교에서 하락은 없고 3쪽만 각각 0.43131·0.42895%p 개선됐다.
나머지 12쪽 PNG는 backend별로 기준과 바이트가 같다. 이 결과가 의미 표 1→0의 편집 범위
축소까지 무회귀라고 뜻하지 않는다. 보류 변경의 수용 여부는 별도 범위 판정이다.
독립 입력의 전체 쪽수는 각각 12·1쪽이며 가져온 Core도 12·1쪽이다.

견적서의 가는 fill 띠는 0.12pt다. 동일한 baseline Native PNG를 진단용
`pdftoppm -thinlinemode none` 출력과 비교하면 99.85062%다. **정식 TSV의 미달을
이 진단값으로 교체하지 않았다.** 원본 두께를 clamp하거나 선을 늘려 점수를 맞추지 않는다.
Windows 원본 글꼴의 실제 파일·PostScript 이름을 확인해 로컬 공급한 3쪽 진단은
기존 Native 점수와 같았다. 이것만으로 실제 resolved face나 글꼴 예외까지 입증하지는 않는다.

Native/WASM의 3·5쪽, 견적서 1쪽과 정상 대조군은 compare·standalone overlay·review를
생성했다. 3쪽은 문단/하단 표의 큰 배치보다 글자 형태·간격 차이가 보이며 Native의 본문 face
차이를 직접 확인했다. 정확한 resolved face와 원인 해결은 미완료다. 5쪽은 위·아래 표 외곽,
점선과 내용·사진 소속을 대조했으며 후보 검출 증가를 의미 표 생성 완료로 보고하지 않는다.
견적서는 셀 경계의 위치와 가는 괘선의 출력/래스터 농도 차이를 구분했다. 정상 대조군은
Native/WASM 네 셀의 경계·내용이 보존됐다. 메인테이너의 직접 시각 판정은 아직 요청 전이다.
미달을 임의 면제하지 않으며 `re_review_required`를 유지하고 PR을 만들지 않는다.

## 5. 증거와 검토 경로

- 원본/실행/로그/TSV: `/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/`
- 구조 가설 화면: `E:\tmp\issue7998-table-review\index.html`
- source: `candidate-source.json`, `producers/base-native.json`, `producers/base-wasm.json`, `producers/candidate-native.json`
- 수정 전·후 구조 관측: `baseline-summary.json`, `*-observed.json`, `*-final.json`
- 비교: `visual/{base-native,base-wasm,candidate-native,candidate-wasm}/<문서>/silhouette.tsv`
- 전체 비교/픽셀 동일성: `visual-comparison.json`, 대표 compare/overlay/review: `representatives.json`
- 로그: `inference-contracts-final.log`, `frontend-contracts.log`, `resource-contracts.log`,
  `worker-contract.log`, `docker-wasm-build.log`, `studio-browser.json`
- 전쪽 비교: `E:\tmp\issue7998-table-review\visual\index.html`
- 실제 새 WASM Studio: `http://localhost:7793/?review=issue7998-table-inference&lang=ko`
- 진단용 별도 조건: `line-rasterization/`, `font-diagnosis/`. 정식 gate와 혼합하지 않는다.

새 워크트리에서 초기 WASM 캡처는 Node 의존성 및 Chrome 경로가 없어 시작하지 못했다.
기존 Studio node_modules를 symlink로 재사용하고 설치된 Chrome 152 경로를 명시해 재개했다.
환경 실패를 제품 실패나 완료된 시각 검증으로 집계하지 않았다.

## 6. 다음 단계의 실행안과 승인 경계

먼저 기존 출력 차이의 이번 보정/후속 분리 범위를 판정한다. 현재 근거만으로 PR 생성 조건을
충족했다고 하지 않는다. B01–B06의 세부 구현은 첫 절편의 보존·보류 계약과 구분해 진행한다.

전체 검증 승인을 받으면 다음을 순서대로 실행한다. 공용 target, jobs 14를 시작값으로 사용하고
현재 실행 중인 Cargo 작업을 먼저 확인한다. 진행 시 base 변경이 있으면 통합하고 실제 영향
범위만 재검증한다. 현재와 제품 코드가 같으면 기존 Docker WASM·Native·전쪽 캡처·Worker
결과를 재사용하고 같은 빌드를 다시 하지 않는다. 코드 보정이 있으면 영향 증거를 갱신한다.

1. `local_validation.md` 4.3의 Rust lint 묶음: fmt check, root Clippy, WASM32 lib Clippy,
   workspace build/전체 target Clippy, 준비된 manifest의 `--check --base-ref <고정 base SHA>`.
2. release lib 및 `cargo nextest run --locked --cargo-profile release-test --tests
   --test-threads 14 --no-fail-fast --target-dir /home/edward/mygithub/rhwp/target/pr-review`.
3. Native Skia 3종: `--features native-skia --lib`, `run-rust-test.mjs
   issue_2225_missing_picture_placeholder`, `run-rust-test.mjs render_p37_direct_pdf_export`.
   각 focused helper에 release-test profile·동일 target·native-skia를 전달한다.
4. doc test, Studio TypeScript/npm test 및 변경된 public API/package 소비 검증.
   이미 통과한 실제 Worker 검증은 제품 코드·옵션 계약이 그대로면 근거를 재사용한다.
5. 결과·남은 백로그·의미 표 보류 범위를 반영한 로컬 한국어 PR 본문을 준비한다.
   source push·PR 등록은 해당 단계의 별도 승인 대상이다.

긴 전체 검증은 `docs_and_git_workflow.md` 297–300행의 별도 승인 전에는 실행하지 않았다.
AGENTS.md의 페이지별 90% 조건과 함께, 이번 결과/범위에 대한 다음 판정을 요청한다.
