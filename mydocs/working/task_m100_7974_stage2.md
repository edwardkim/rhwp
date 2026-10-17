# #7974 — 전체 검증과 로컬 PR 준비

2026-10-17 사용자가 **“전체 검증·PR 준비 승인”**으로 시각 통과 후 실행안을 승인했다.
원격 push·PR 생성·ready 전환·merge·이슈 종료는 이번 승인에 포함하지 않는다.

- 기본 경로: `collaborator_self_merge`.
- 추가 적용: `intake_and_review`, `local_validation`, `visual_fixture_evidence`.
- 읽은 권위 문서: `pr_review_workflow.md`, `pr_review/README.md`와 위 네 자식 문서.
- 착수 시 제품 commit: `6b072955e9b83d8f7729133eda5e93536ee453f5`.
- 시작 HEAD: `88535330fdce00379c0f6284d182f2a116d4efb3` (제품 검증 후 문서만 추가).
- 새 fetch의 upstream/devel: `515bd72dcde0eecdeece333666b675354b7fa91d`.
  HEAD가 이를 포함하며 추가 통합은 필요 없다.
- 공유 target: `/home/edward/mygithub/rhwp/target/pr-review`; Cargo 작업은 순차 실행한다.
- 기존 Native/Docker WASM·집중 검사·전쪽 시각 TSV와 사용자 판정은 stage1의 동일 소스 증거를 재사용한다.
  새 제품 소스 변경이 없으므로 동일 WASM을 반복 빌드하지 않는다.

## 착수 시 실행 범위

아래 착수 기록 이후 exact-base 비교에서 축소 회귀를 발견해 보정했다. 최종 source와
재사용·새 실행의 구분은 뒤의 보정본 기록을 따른다.

`output/pr-review/backlog-7949/validation-next/plan-ko.md`의 순서로 미완료 lint·전체 Rust·Studio·
공개 PDF API/실제 Worker·npm 소비 경계를 검증한다. 개별 명령·시각·종료값·로그는 같은 경로의
`validation-results.json`에 저장하며, 미실행 검사를 통과로 기록하지 않는다.

exact-base Native/fresh WASM의 공통 정상 대조군 전쪽 하락 비교도 남아 있다. 새 지원 입력은 base에서
거절된 기존 증거와 독립 PDF 비교를 연결하고 기준 TSV가 있는 것으로 보고하지 않는다.

기본 Studio 7700, 다른 작업의 WIP와 공유 target은 보존한다. task Studio 7795의 생성 glue는
로컬 실행 산출물로 남기고 source commit에는 포함하지 않는다.

## exact-base 비교에서 검출한 회귀와 보정

Native·WASM32·workspace all-target Clippy와 workspace build는 통과했다. 이후 release lib 컴파일 중
exact-base fresh WASM의 공통 대조군 27쪽 비교에서 개인 정상 문서의 9쪽이 0.01%p 이상 하락했다.
최대 하락은 9쪽의 99.85616 → 98.40687% (1.44929%p)다. 90% 통과·앞선 사용자 시각 판정을
하락 기준의 예외로 해석하지 않았다. 후속 통합 검사를 실행하기 전에 이 작업 소유의 전체 검증
runner를 중단했으며, 아직 끝나지 않은 release lib 검사도 통과로 세지 않았다.

하락 영역은 source보다 작은 출력에 배치된 이미지다. 기존 코드가 확대의 Interpolate 선택을
축소의 coverage/prefilter까지 강제한 가정을 제거한다. PDF 32000-1 §8.9.5.3의 확대 정의와 원본
PDF의 실제 축소 출력을 근거로, 원색 선언은 PNG에 계속 보존하되 최종 draw의 source rectangle과
device rectangle을 비교한다. 축소에서는 기존 backend coverage/profile을 유지한다.
Native·Canvas의 실제 변환 행렬과 crop을 소비하고, CanvasKit도 draw 시점의 행렬·source 창을
사용한다. DOM은 실제 source 창·화면 배율·devicePixelRatio를, SVG는 intrinsic 96-DPI frame을
사용한다. 문서 ID·목표 점수·배율 보정 상수는 넣지 않는다.

근거: [PDF 32000-1:2008 §8.9.5.3](https://opensource.adobe.com/dc-acrobat-sdk-docs/standards/pdfstandards/pdf/PDF32000_2008.pdf).
작은 checkerboard 축소 입력을 별도로 생성하여 수정 전/후를 확인한다. 이 절은 보정 방향과
검증 계획이며 새 Native/fresh WASM 검증 완료를 뜻하지 않는다. 새 코드에 영향을 받는 빌드·
시각·전체 회귀는 새 결과로 갱신하고 기존 stage1 결과와 구분한다.

작은 축소 대조군의 기준 PDF는 RGB 127 회색, 수정 전 실제 WASM은 RGB 0 검정이다.
양쪽 실루엣은 100%여서 색상·픽셀 직접 판독을 별도로 기록한다. 새 회귀를 점수만으로 통과시키지 않는다.
원래 후보의 Native 대조도 27쪽 중 8쪽 하락(최대 1.44636%p)을 확인했다.

보정 중 WASM 보조 함수를 trait impl에 잘못 배치한 컴파일 오류를 자체 impl으로 옮겼다.
또한 새 base worktree를 빌드한 뒤 공유 target이 오래된 mtime의 candidate vendor 산출물을
최신으로 보아 재사용한 경우를 실제 새 마스크 열기 smoke에서 검출했다. 잘못된 producer로
Native 시각 검증을 완료하지 않았다. `shared-target-refresh.json`의 정확한 9개 소스/manifest
mtime만 갱신했고 전후 bytes SHA는 동일하다. 공유 cache 전체를 삭제하지 않았다.
fingerprint 로그에서 candidate의 `path_color.rs` 변경을 인식해 vendor와 의존 root를 다시
컴파일하는지 확인한다. source 전환 후에는 producer 해시뿐 아니라 새 기능의 실제 열기 smoke로
소비하는 라이브러리도 확인한다. 첫 컴파일 실패·중단 로그와 잘못된 producer 실행은 보존한다.

## 보정본의 완료된 선행 검증과 두 번째 사용자 판정

- Rust/WASM 제품 source: `b2e0f32ba`와 동일한 Rust 입력. 후속 변경은 TS의 잘못된 선언 fallback
  보존과 정식 회귀·공개 입력·기록이다. Native producer와 실제 새 마스크 열기 smoke를 연결했다.
- 새 Docker WASM SHA-256: `50ead3ca44c8f807f44aec6e575b4d61426b1b8828bd182d1864fcd440770f57`.
  실제 Studio Canvas2D·CanvasKit 초기화 응답 bytes도 이 해시다. DOM flow도 실제 decode·pixelated를 확인했다.
- 작은 공개 입력 22쪽 + 개인 정상 문서 19쪽의 Native/fresh WASM 전쪽 raw TSV를 새로 산출했다.
  공개 최저는 기존 linear control의 Native 92.92381%, WASM 92.64496%다. 16-bit은 새 출력과 MuPDF가
  양쪽 100%다. 정상 19쪽 최저는 Native 99.12816%, WASM 99.54466%다.
- exact-base 공통 27쪽의 양쪽 0.01%p 이상 하락은 각각 0쪽이다. 추가 축소 입력 한 쪽도 base/current
  Native와 WASM의 픽셀이 각각 같다. 축소 checker의 PDF gray 127 / Native 99 / WASM 77은 기존 필터
  차이로 남으며 새 정확한 색상 golden으로 고정하지 않는다. 원색을 하나 버린 이전 WASM gray 0과 구분한다.
- 실제 Native-skia 기능을 켠 정식 #7974 집중 **8건 통과**. 기존 sampling/save 검사에 source coverage
  소실 반례를 보완했다. HWP/HWPX 저장·재열기까지 포함한다. layout 좌표나 회색 kernel 값은 고정하지 않는다.
- 포맷 후 source byte weight가 바뀌어 파생 suite가 달라진 첫 집중 실행은 **0건**이므로 통과가 아니다.
  포맷 완료 후 prepare/check를 다시 수행해 실제 module 배정과 필터를 맞추고 위 8건을 실행했다.
- Studio 전체 **2,315 통과·조건부 2 생략**, 뒤 소비 경로 보완 **6건 통과**, editor **32건 통과**,
  frontend binding/embed·declaration·package dry-run 통과. 실제 PDF API/Worker·오류 복구 검사도 새 WASM으로 통과했다.
- 전체 Rust 회귀와 새 code의 lint는 남아 있다. 실행 로그는 `validation-final/`에 이어 저장한다.

2026-10-17 사용자는 보정 비교 화면 `E:\tmp\issue7974-final-review-20261017\index.html`의
확대 경계·추가 축소 대조군·기존 문서 9쪽과 남은 기존 tone 차이를 제시받고 **“시각 판정 통과”**로
회신했다. 이 판정을 원격 push·PR 등록·merge 승인으로 확대하지 않는다.

## PR 준비에 남은 판정

- 새 지원 입력의 기준 TSV 부재는 2026-10-17 사용자가 **“신규 지원 예외·공통 대조군 비교 승인”**으로
  판정했다. devel이 거절하는 13개만 동일 입력의 미지원 오류 → 정상 열기·독립 PDF 전쪽 90% 이상으로
  평가한다. 양쪽에서 열리는 정상 대조군의 Native/fresh WASM 페이지별 하락 비교는 그대로 수행한다.
  원래 90%·쪽수·직접 판독 기준이나 공통 대조군의 하락 허용치를 완화한 승인이 아니다.
- 개인 정상 PDF는 승인된 후속 계획 4절의 개인정보 보존 범위에 따라 저장소 밖에만 보관한다.
  50 MiB 미만이지만 이를 공개 fixture로 옮기지 않으며, 공개 재현 입력 22개·저장 입력 42개와
  구분한다. 공개 본문에는 해당 정상 대조군의 집계만 기록한다.
- PR 번호별 self-review·오늘할일은 PR 등록 뒤 정확한 code head CI 성공 후 작성한다.
- 검증과 본문·대표 이미지·생성 명령까지 로컬로 준비한 다음 원격 반영 승인을 요청한다.

## 최종 source와 증거 연결

코드·정식 검사 후보는 `ce8997a4dc4c900602e65ba8f662e8bd98b719c9`다. 변경된 Rust/WASM runtime
16개 파일이 Docker 빌드 source `b2e0f32ba`와 bytes가 같으며 pkg와 Studio/public의 실제
glue/WASM도 일치한다. `validation-next/final-artifact-reuse-proof.json`에 해시를 보존했다.
TS guard 이후 TypeScript 검사와 실제 Studio 소비 경로도 통과했다. 문서·대표 PNG를 추가한다는
이유로 같은 WASM을 다시 빌드하지 않는다.

최종 SVG 실제 export·Chrome raster 공개 22쪽도 최저 92.64496%다. 확대 source/crop 창은
PNG 선언 → 최종 draw의 device 변환에서 소비한다. Native는 `skia/image_conv.rs`의
`draw_image_rect_with_sampling_options`, WebCanvas는 `image_sampling_scope`의 `get_transform`,
CanvasKit은 draw 행렬·source rect, DOM은 실제 natural/crop 창·displayScale·devicePixelRatio를
사용한다. SVG는 intrinsic 96-DPI frame이며 외부 viewer의 임의 zoom은 미검증이다.
줄/쪽·bbox·높이 계산은 변경하지 않으므로 pagination·rowspan의 컷 계약은 비해당이다.

공통 27쪽에 추가 축소 1쪽을 합친 28쪽의 exact-base 하락은 Native/WASM 각각 0쪽이다.
`validation-next/final-visual-summary.json`은 전체 41쪽의 누락·90% 미만 0쪽과 직접 판독을
연결한다. 색상 키의 숨김 칸·원색, stencil의 현재 색/투명 칸, 저해상도 확대 경계의 비교를
다시 직접 확인했고 대표 PNG는 이 최종 캡처에서 복사했다. checker의 기존 tone 차이와
시각 통과 판정은 위 두 번째 사용자 판정을 따른다.

전체 Rust/lint의 최종 실행은 `validation-final/validation-results.json`에 기록 중이다.
완료한 frontend/실제 Worker 로그와 새 source의 추가 소비 경로 검사를 연결하며, 초기 중단된
release lib 실행을 통과로 세지 않는다. 최종 결과가 모두 확정된 뒤 PR 준비 완료로 기록한다.

### 전체 통합 실행에서 확인한 기존 거절 기대값

첫 release lib 검사는 3,986건 통과·기존 12건 생략으로 완료됐다. 전체 nextest는
10,816건 중 10,815건 통과·1건 실패·45건 생략이었다. 실패한
`issue_7911_pdf_resource_limits::limited_import_preserves_supported_masks_and_rejects_unsupported_masks`는
이번에 지원한 색상 키 배열을 여전히 `PDF_IMAGE_UNSUPPORTED`로 기대했다.

`bf00da2dc5fbf5a5462f3f94b7c44eccfd39bb7b`는 이 검사만 보정했다. 원색 검정의 Mask 일치와
빨강 대조군을 각각 alpha 0/255로 검사하고, inline/XObject stencil의 set/clear bit를 투명/검정
칠로 검사한다. 이는 입력 sample 계약이며 실물 배치 px나 그림 golden이 아니다.
기존 DCT SMask 미지원·잘못된 Decode/길이·매달린 참조·예약 한도는 유지한다. 뒤쪽 미지원 입력의
전체 import 원자성 검사는 이제 정상 지원하는 색상 키 대신 기존 미지원 DCT SMask로 보호한다.
정식 #7911 모듈 **123건 통과**로 변경 경계와 나머지 자원 계약을 함께 확인했다.

제품 source·fixture·sampling이 바뀌지 않아 기존 Native/fresh WASM·직접 시각 판정과 성공한
release lib 검사를 재사용한다. `src`·`crates`·vendor의 Rust/manifest/lock 변경이 없음을 확인하고,
기존 전체 실행과 실패 로그는 `validation-final/attempt1/`에 별도로 보존했다. 새 test head의 lint·
전체 통합 재실행과 미완료 Native Skia·doc 검사는 같은 공용 target에서 순차 실행한다.

집중 검사 뒤 단일 항목 loop의 Clippy 지적을 같은 동작의 블록으로 정리하고 포맷했다.
최종 검사 코드 head는 `0e8ad837cc6d4ce86b21427e44e9c7204d56040d`다. 이 변경도 테스트 함수의 블록/포맷만이며
제품 Rust/TS·입력·시각 출력은 그대로다. 해당 lint 실패는 `validation-final/attempt2/`에
보존했고 새 lint 묶음은 통과했다.

## 최종 결과 — 로컬 PR 준비 완료

검증한 code/test head는 `0e8ad837cc6d4ce86b21427e44e9c7204d56040d`다.
후속 기록·대표 PNG만 추가하며 제품·테스트·입력의 bytes는 바꾸지 않는다.

| 최종 게이트 | 결과 |
| --- | --- |
| fmt, manifest/base SHA, unit-tier | 통과 |
| Native/WASM32 Clippy, workspace build/all-target Clippy | 모두 통과 |
| release lib | 3,986 통과·12 생략; 변경 없는 lib source의 성공 결과 재사용 |
| release-test 전체 nextest | 10,816 통과·45 생략 |
| Native Skia lib | 4,043 통과·12 생략 |
| Native Skia 그림 placeholder / direct PDF | 2 / 14 통과 |
| doc | 8 통과·3 생략 |
| Studio 전체 / 소비 경로 후속 / editor / frontend | 2,315 통과·2 생략 / 6 / 32 / 3 통과 |
| Studio TypeScript, editor 선언, package dry-run | 통과 |
| 실제 PDF API/Worker·오류 복구 | 115 assertions·Worker 7개·12쪽 render/save 통과 |

모든 명령·종료값은 `validation-final/validation-results.json`, 집계는
`validation-final/final-result-summary.json`에 있다. 필터로 선택한 Native 검사에서는
선택 밖의 함수를 실행한 것으로 세지 않았다. 기존 90% 기준·golden·허용치는 완화하지 않았다.
프로토콜 검사 `schemaMinorVersion` 25→26만 additive field와 함께 변경했다.

전쪽 raw TSV는 Native/WASM 각각 공개 22쪽·정상 19쪽에서 미달·누락 0쪽이다.
공통 28쪽의 devel 대비 하락도 양쪽 0쪽이다. Native/fresh WASM의 같은 최종 PNG에서
canonical `make_compares`·`make_overlay_compares`·`make_review_panels`로 대표 패널을 만들었다.
`final-review-panels/manifest.json`에 source/reference hash와 metric gate를 기록했으며 제품을
다시 렌더하지 않았다. silhouette-only의 `not_evaluated` 원본은 변경하지 않고 metric gate와
사용자 직접 판정을 별도로 연결했다.

재현용 공개 PDF 22개·HWP/HWPX 42개와 inventories는 이미 commit했다. 최종 대표 review·
standalone overlay·compare PNG는 `mydocs/pr/assets/issue_7974/`에 보존하고 한국어 PR 본문에
실제 Markdown 이미지로 넣는다. private 정상 문서·폰트·로그·TSV·중간 JSON·생성 glue/WASM·
pkg·환경 파일은 source commit에 넣지 않는다.

사용자 확인 화면은 `E:\tmp\issue7974-final-review-20261017\index.html`이며 같은 캡처의
공식 비교 패널 링크와 사용자 통과 판정도 연결했다. Studio 7795는 유지한다. 비교용으로만 만든
`/tmp/rhwp-7974-base-20261017`은 base producer·evidence를 보존하고 활성 cwd가 없음을 확인한 뒤
제거했다. 기본 작업공간·7700·기존 WIP·공유 target은 보존했다.

원격 devel은 마지막 fetch에서도 기준 `515bd72dcde0eecdeece333666b675354b7fa91d`와 같다.
한국어 PR 본문·대표 이미지·생성 명령까지 로컬로 준비했으며 원격 push·PR 생성은 아직 하지 않았다.
등록 뒤 정확한 code head CI 성공 시 self-review·오늘할일을 준비하고 이후 merge 조건을 확인한다.
이 기록은 PR ready·merge·이슈 종료 승인이나 남은 백로그 완료를 뜻하지 않는다.
