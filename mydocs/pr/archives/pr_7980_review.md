# PR #7980 리뷰 — PDF 원색 마스크와 확대 sampling 보존

## 최종 판정

**승인 — #7974의 색상 키 Mask·독립 stencil·확대 sampling 보존 범위에서 self-review를 완료했다.**
검토한 code head는 `dcd70ea411a70fe7b45ddc89d8156dac51f8798f`다. 새 코드 보정은 없으며,
이 기록 이후 문서-only trailing head의 실제 CI·mergeability 확인과 작업지시자 병합 승인이 남는다.
별도 GitHub APPROVE event나 병합·이슈 종료를 수행했다는 의미가 아니다.

## 접수 정보

| 항목 | 확인한 값 |
| --- | --- |
| PR / 작성자 / base | [#7980](https://github.com/edwardkim/rhwp/pull/7980) / edwardkim / devel |
| 해결 이슈 | [#7974](https://github.com/edwardkim/rhwp/issues/7974), PR 본문의 `Closes #7974` |
| PR code head | `dcd70ea411a70fe7b45ddc89d8156dac51f8798f` |
| 로컬 전체 검증 code/test | `0e8ad837cc6d4ce86b21427e44e9c7204d56040d`; 이후 제품·검사·입력 변경 없음 |
| 시각 기준 producer의 base | `515bd72dcde0eecdeece333666b675354b7fa91d` |
| 최신 fetch devel | `42dade44e3a603015cecb7e9731928e3c3c8bd05` (#7981) |
| 규모 | 원 code diff 113파일, 2,051줄 추가 / 57줄 삭제; 공개 입력·inventory·문서·PNG 포함 |
| 역할 / 참고 상태 | 작성자 self-review, reviewer 지정 안 함. 작성 시 OPEN/non-draft, MERGEABLE/CLEAN |

기본 경로는 `collaborator_self_merge`, 보조 경로는 `intake_and_review`, `local_validation`,
`visual_fixture_evidence`, `review_only_fast_pass`, 대형 PR의 `rework_and_exceptions`다.
`pr_review_workflow.md`·선택표·위 자식 문서와 review template을 읽었다.
2026-10-17 작업지시자가 self-review 진행을 승인했다. 실제 source·test·fixture·workflow는 바꾸지 않는다.

## 변경 경로와 코드 검토

실제 diff와 아래 소비 경로를 대조했다. 각 기능의 구현·실행 근거는
[최종 검증 기록](../../working/task_m100_7974_stage2.md)과
[초기 수정 전후 기록](../../working/task_m100_7974_stage1.md)에 연결한다.

| 구현 주장 | 실제 적용 경로와 독립 기대값 / 확인한 반례 |
| --- | --- |
| 색상 키는 원래 성분 샘플에서 판정 | `image_mask_limited::Mask::read` → `ColourKey::read` → decoded image에 alpha 적용. PDF 1.7 §4.8.5의 Decode 이전 범위를 사용한다. Indexed의 같은 RGB/다른 index, 같은 8-bit 값으로 반올림되는 16-bit 값, packed 행 패딩·RGB 모든 성분·SMask 우선순위를 검사한다. |
| stencil은 호출의 칠 색·alpha 사용 | page/Form/inline의 `process_image_colour_operator` → `ImageClipTracker`의 저장·복원 상태 → handle의 `stencil_colour` → `stencil` 및 opacity 적용. 같은 Form 두 번 호출의 빨강/초록, 반전 Decode·9-bit 행·중간 alpha가 정식 검사에 있다. |
| 선언은 저장 후에도 전달 | bounded `PdfImage.interpolate` → `png_limited::encode`의 `rhWP` → BinData → HWP/HWPX → PNG resolver. 버전·길이·CRC를 확인하는 고정 prefix만 읽으며, 그림 효과로 재인코딩할 때도 선언을 보존한다. Paint additive minor 26과 full/narrow image transport가 `imageInterpolation`을 전달한다. |
| 확대만 명시적 sampler 적용 | PDF 32000-1 §8.9.5.3을 근거로 최종 source/crop 창과 device 크기를 비교한다. Native는 실제 canvas 행렬·source rect, WebCanvas는 `get_transform`과 cropped/ordinary draw, CanvasKit은 최종 draw 행렬, DOM은 natural/crop 창·displayScale·DPR을 소비한다. 값이 없거나 잘못되면 기존 profile을 유지한다. |
| 축소 coverage 보존 | 단순 nearest 강제가 기존 문서 9쪽과 checker에서 잉크를 잃는 반례를 재현해 `f0865c21e`에서 제거했다. 정식 검사는 저장·재열기 후 중간 잉크의 존재를 확인하고 kernel별 회색값이나 문서 좌표를 고정하지 않는다. |
| 별도 backend의 적용 범위 | Native Skia·실제 WASM Canvas2D·CanvasKit software·DOM·SVG를 구분한다. WebCanvas smoothing은 scope가 끝나면 복원되고, binary effect와 bitmap glyph의 우선순위는 유지한다. SVG 판정은 intrinsic 96-DPI frame이며 외부 viewer 임의 zoom은 미검증이다. |

새 조건은 샘플 ID·목표 점수·페이지 수를 참조하지 않는다. 줄 구성·bbox·높이·쪽 나눔을 바꾸지 않으며
잘못된 좌표를 clamp하거나 그림을 숨기는 보정도 없다. 마스크 성분 비교와 최종 draw의 sampling 선택을
구분했으며, 공통 helper 이후 backend별 기존 효과 우선순위와 축소 fallback도 확인했다.

## 검증 입력과 증거의 동일성

**충족 — 공개 입력은 검토 code commit에 포함되어 있고 실제 실행 bytes와 같다.**
[공개 PDF inventory](../../../pdf/issue7974/inputs.json)의 22개와
[저장본 inventory](../../../pdf/issue7974/saved/inputs.json)의 HWP/HWPX 42개를 실제 파일·Git blob·
SHA-256·바이트 수로 대조했다. inventory SHA-256은 각각
`6e04f34f509ae9f6735982fbd46ddf889623a363de544321c62eecdd3682b013`,
`513d37976d11e99a033ee1ee99cebd4f7d4c3c017306beef0ab1494c9a53570e`다.
파일별 source/generator/hash는 해당 inventory를 정본으로 사용한다.

PDF는 자체 작성한 원색·비트·사전 계약 입력이며, 독립 Poppler 96-DPI 출력을 기준으로 비교했다.
`gray16-key`는 raw 32767/32768을 Poppler가 같은 8-bit 값으로 축소하는 문제를 분리하여
PDF 원색 계약과 MuPDF를 기준으로 Native/WASM 양쪽 100%를 확인했다.
이를 실제 한컴 문서·Print 출력의 증거로 바꾸어 주장하지 않는다.

비공개 정상 대조군 19쪽과 글꼴은 사용자가 승인한 후속 계획의 개인정보 보존 예외에 따라 로컬에만
보관한다. 이 예외는 공개 input 64개의 커밋 확인이나 전쪽 시각 기준을 면제하지 않는다.
공개 기록에는 집계만 남기고 식별 파일·PNG·글꼴을 추가하지 않았다.

Docker WASM SHA-256은 `50ead3ca44c8f807f44aec6e575b4d61426b1b8828bd182d1864fcd440770f57`다.
실제 Studio 초기화 응답과 pkg/public의 동일성을 확인한 기존 증거를 재사용했다.
검토 head의 runtime 16파일이 빌드 source `b2e0f32ba8481de1adf2eb80dc621d552863ef9f`와 같음을
다시 대조했다. 동일 빌드를 반복한 결과나 새 head 재캡처로 기록하지 않는다.

## 시각 검증과 직접 판독

| 범위 | 전체 쪽수 | Native 최저 raw | fresh Docker WASM 최저 raw | 누락 / 90% 미만 |
| --- | ---: | ---: | ---: | --- |
| 공개 source PDF | 22 | 92.92381% | 92.64496% | 양쪽 0 / 0 |
| 비공개 정상 대조군 | 19 | 99.12816% | 99.54466% | 양쪽 0 / 0 |

공개 source 22개는 각 1쪽이며 `linear-control` 외 21개는 Native/WASM 모두 100%다.
공통 지원 28쪽의 고정 base 대비 0.01%p 이상 하락은 양쪽 0쪽이다. 추가 축소 checker는
base와 현재 출력 픽셀이 backend별로 동일하다. 새 지원 13개는 기준 devel에서 거절되므로
작업지시자가 승인한 **신규 지원 예외·공통 대조군 비교**를 적용했다. 기준 TSV가 있었다고 쓰지 않는다.
실제 SVG export를 Chrome으로 raster한 공개 22쪽도 최저 92.64496%다.

동일 final PNG의 색상 키·stencil·확대 경계·축소 대조군 review를 이번 self-review에서도 열었다.
숨겨야 하는 칸, 빨강/초록/파랑, stencil의 마젠타와 투명 영역, 확대 사각 경계가 기준과 대응한다.
Native/WASM review·overlay 라벨도 판독 가능하다. 축소의 PDF gray 127 / Native 99 / WASM 77은
기존 backend kernel 차이이며 새 검정 0의 coverage 소실과 구분한다. 사용자는 해당 차이를 포함한
최종 화면과 정상 대조군 9쪽을 확인하고 **시각 판정 통과**를 주었다.

[색상 키 review](../assets/issue_7974/colour-key-review.png),
[stencil review](../assets/issue_7974/standalone-stencil-review.png),
[Native 확대 review](../assets/issue_7974/low-resolution-mask-native-review.png),
[Native overlay](../assets/issue_7974/low-resolution-mask-native-overlay.png),
[WASM 확대 review](../assets/issue_7974/low-resolution-mask-wasm-review.png),
[WASM overlay](../assets/issue_7974/low-resolution-mask-wasm-overlay.png),
[축소 review](../assets/issue_7974/minification-control-review.png)를 사용했다.
등록 당시 PR 본문의 실제 Markdown 이미지 8개는 검증한 code SHA로 고정했고 원격 bytes도 일치했다.
후행 본문 갱신에서는 같은 Git blob을 후행 head SHA로 고정하며 원 실행 code SHA와 구분한다.

## 최신 devel 전진과 검증 재사용

#7981의 `42dade44e3`은 Cargo의 기본 활성 `wasm-api` 선택과 `src/lib.rs`의 module/export/start gate를
추가했다. 기본 feature를 전개한 lib token은 기존 base와 같고, 빈 opt-out feature와 CLI required-feature를
제외한 manifest도 같다. 렌더러·PDF backend·model·paint·글꼴·lockfile은 바뀌지 않았다.
[해당 PR의 기본 빌드 동등성 기록](https://github.com/edwardkim/rhwp/blob/42dade44e3a603015cecb7e9731928e3c3c8bd05/mydocs/pr/archives/pr_7876_review.md)에는
독립 feature-only package 7파일의 WASM 포함 bytes 동일성과 실제 기본 브라우저 12쪽 동등성도 있다.

이에 따라 기존 기본 모드 TSV의 producer 출처 `515bd72dcd`를 보존해 재사용한다.
`42dade44e3`에서 새로 렌더한 TSV나 새 통합 실행이라고 바꾸어 쓰지 않는다. source branch에
devel을 반복 merge/rebase하지 않았고 최신 base와의 자동 merge tree는 충돌 없이 계산됐다.
후행 문서 push 전에는 최신 base/head로 merge tree·링크·오늘할일 기록 보존을 다시 확인한다.

## 로컬 검사와 정확한 code-head GitHub CI

로컬 실행 명령·실패와 보정·source 재사용 범위는 [stage2](../../working/task_m100_7974_stage2.md)를 따른다.
이번 검토에서는 source·검사를 바꾸지 않아 이를 반복 실행하지 않았다.

| 로컬 게이트 | 완료한 실제 결과 |
| --- | --- |
| fmt / manifest / unit-tier / Native·WASM32·workspace all-target lint / build | 모두 통과 |
| release lib / 전체 nextest | 3,986 PASS·12 ignore / 10,816 PASS·45 skip |
| Native Skia lib / 그림 placeholder / direct PDF | 4,043 PASS·12 ignore / 2 PASS / 14 PASS |
| doctest | 8 PASS·3 ignore |
| Studio / 소비 경로 후속 / editor / frontend | 2,315 PASS·2 skip / 6 / 32 / 3 PASS |
| TypeScript / 선언 / package dry-run | 통과 |
| 실제 PDF API·Worker | 115 assertion·실제 Worker 7개·12쪽 render/save 및 오류 복구 통과 |

code SHA `dcd70ea411a70fe7b45ddc89d8156dac51f8798f`에 대해 다음 Actions가 모두 completed/success였다.

- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/39030056524)
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39030056525)
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39030056496)
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39030056512)
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39030056511)
- [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39030056489)

35 check는 31 success·4 skipped, 실패·진행 중 0이었다. `Build & Test`와 언어별 Analyze·GHAS CodeQL을
포함한다. 독립 WASM Build job의 skipped를 로컬 fresh Docker WASM이나 frontend gate의 실행과 혼동하지 않는다.
전체 통합의 첫 1FAIL은 신규 지원을 여전히 거절하던 기존 기대값이었다. 색상 키·stencil의 raw alpha
성공을 검사하도록 바꾸면서 DCT SMask 거절·손상 Decode/길이·예산·오류 복구·전체 import 원자성은 유지했다.
이후 #7911 모듈 123건과 최종 전체 회귀가 통과했다. 실패 로그는 제거하지 않았다.

## 조판 원칙 준수 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | PDF 원색 Mask/현재 칠 색/확대 계약과 실제 sample·device 크기. 문서별 예외 없음 |
| 측정·배치 일관성 | 충족 | 기존 geometry를 유지하며 실제 draw의 source/crop·device 좌표에서 sampler 선택 |
| 최종 출력 소비 계약 | 충족 | 실제 Native·WASM Canvas2D·CanvasKit software·DOM·intrinsic SVG 증거; 외부 SVG 임의 zoom과 GPU 일반성은 미검증 |
| 분할·이어받기 / 줄 소속·점유 높이 | 비해당 | 컷·행·쪽·문단 높이·배치 geometry를 바꾸지 않음 |
| 사례와 증거의 독립성 | 충족 | 원본 sample/사전·Poppler/MuPDF와 후보를 대조. 기존 public 흐름의 미지원 13개·확대 흐림·축소 coverage 반례를 보존. 새 formal binary 자체를 base에서 실행한 증거로 확대하지 않음 |
| baseline·golden·허용치 변경 | 비해당 | 완화 없음. schema minor 25→26은 additive transport 계약 갱신 |
| 회귀 추가·변경 선행 조건 | 충족 | 관련 source·저장 입력의 Native/fresh WASM 전쪽 90% 이상 확인 후 정식 검사 반영. sample alpha·소유·내용 보존·coverage를 검사하며 문서 절대 px/golden은 추가하지 않음 |
| 90% 이후 확대 판독 | 충족 | 마스크 칸·원색·동일 index/RGB 구분·확대 경계·축소 잉크와 사용자 판정 연결. 수식 배치 변경은 비해당 |
| 주장과 검증 범위 | 충족 | runtime·producer·검사 head를 분리했고 실행 검출 회귀/기존 tone 차이/아래 미검증을 구분 |

## 남은 범위와 후속 처리

[기존 백로그](../../plans/task_m100_7949_backlog.md)와
[승인된 후속 계획](../../plans/task_m100_7949_backlog_followup.md)의 나머지 항목은 완료하지 않았다.
RGB와 alpha가 모두 변하면서 다른 sampling을 요구하는 별도 plane, graphics-state soft mask/blend·부분 text alpha,
추가 raw-mask codec adapter, 의미 표·배경 그룹, MOF 글꼴·간격은 후속 범위다.
`parserAllocationBoundsProven=false`를 유지하며 전체 할당 감사나 비신뢰 입력 배포 준비 완료를 주장하지 않는다.

다음 실행 순서는 [후행 기록·병합 계획](pr_7980_review_impl.md)을 따른다. self-review의 승인 판정과
후행 CI 성공·작업지시자 병합 승인·실제 이슈 종료는 구분한다.

## Merge 후 contributor PR comment 계획

자체 PR이므로 별도 외부 contributor의 원 PR은 없다. 병합 후속 처리 승인과 merge SHA가 확정되면
#7980·#7974에 실제 merge/CI 결과, [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md),
41쪽의 수치·직접 판정·남은 범위를 한국어 존댓말로 기록한다. 대표 이미지는
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_7974/colour-key-review.png`
등 같은 asset의 merge SHA 고정 URL을 사용한다. 게시 시 UTF-8 Markdown `--body-file`을 사용하고
API로 본문·이미지 URL을 재조회한다. 이번 self-review 단계에서는 미리 댓글을 게시하거나 이슈를 닫지 않는다.
