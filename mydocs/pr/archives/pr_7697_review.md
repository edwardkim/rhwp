---
kind: review
status: completed
last_verified: 2026-10-09
---

# PR #7697 self-review — renderer 공통 소비 계약과 검증 절차

## 최종 판정

**승인 — #7688의 crop·화살촉·글꼴 소비 보정 범위에서 로컬 검증, 직접 시각 판독과 정확한 code head CI가 완료됐다.**
2026-10-09 작업지시자의 시각 통과 및 self-review 진행 승인을 반영한다.
이 판정은 GitHub의 작성자 본인 approve event가 아닌 저장소 self-review 기록이다.
문서-only trailing head의 CI·fast-pass와 merge 직전 head/mergeability 확인, 작업지시자의 병합 승인은 남은 조건이다.
#7688 전체 backend parity를 해결한 것으로 확대하지 않으며 이슈는 열린 상태로 유지한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#7697](https://github.com/edwardkim/rhwp/pull/7697) / edwardkim self-review / devel |
| 검증한 code head | `7e71c6638a9bb3791fbacee8b37a2c5e2ced2d5a` |
| 제품 검증 기준 | base `6f66932a73ee6fd05a74a83bdc438c9499a89a12`; 최종 로컬 기록 `33991c311a81aef43392f9f9940f5ab805972d8d` |
| 현재 base 확인 | `79abd9e49675f4286eaed38197f7e837afb744e7`의 canonical 절차를 읽었다. 문서 기록 때문에 source를 merge/rebase하지 않는다. |
| 관련 이슈 | [#7688](https://github.com/edwardkim/rhwp/issues/7688), `Refs`만 사용; 종료하지 않음 |
| 작성 시점 상태 | Open, draft=false, mergeable=true / clean; reviewer 지정 없음 |

base route: `collaborator_self_merge.md`.
modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
`review_only_fast_pass.md`, `rework_and_exceptions.md`, `review_template.md`.
대형 PR은 단계별 코드·반례·시각 검토를 마친 뒤 처리하며 즉시 admin merge하지 않는다.
기존 단계별 근거와 실제 호출 경로는 [최종 보고서](../../report/task_m100_7688_report.md) 및 아래 단계 기록에 연결한다.

## 변경과 조판 원칙 검토

| 항목 | 실제 호출·독립 근거와 반례 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | [4단계](../../working/task_m100_7688_stage4.md): HWPUNIT의 등방 단위와 독립 Print의 두 그림 선택 행을 확인했다. imgDim 없는 zero-origin을 무조건 전체 그림으로 해석하던 가정을 제거했다. imgDim·scan·한 축 crop 대조군은 보존했다. 문서 ID 예외나 새 좌표 clamp를 추가하지 않았다. | 충족 |
| 공통 결과의 실제 소비 | crop/imgDim → ImageNode/PaintOp → Rust `compute_image_crop_src` → SVG viewBox·WebCanvas/Skia source rect. Studio의 동일 단위 계약은 `imageCropScale` → `imageCropSourceRect` → PageRenderer/CanvasKit이다. [5단계](../../working/task_m100_7688_stage5.md)의 `arrow.rs` 형상은 SVG·WebCanvas·Skia·paint JSON `arrowHeads` → CanvasKit path로 소비된다. 화살촉을 backend에서 재추측하지 않는다. [6단계](../../working/task_m100_7688_stage6.md)의 font 요구 → bytes 준비 → 실제 style/glyph 선택도 확인했다. | 충족 |
| 분할·이어받기 계약 | pagination·rowspan·내용 컷·예약 높이를 변경하지 않는다. | 비해당 |
| 줄 소속과 점유 높이 | 저장 LineSeg·재조판 줄 구성·쪽/셀 소유를 변경하지 않는다. 기존 IR의 원점·bbox·transform/clip·plane을 paint에서 소비하는 수정이다. | 비해당 |
| 사례와 증거의 독립성 | 원본 및 Hancom Print를 대응시켰고 crop·실제 Native 화살촉·font preparation/style 회귀는 동일 검사로 수정 전 FAIL / 후 PASS를 확인했다. 합성 face 공급 검사와 실물 조판 일치를 구분했다. [7단계](../../working/task_m100_7688_stage7.md)에 최종 source·출력 신원 및 반례가 있다. | 충족 |
| 기준값 변경 | crop의 잘못 연결된 두 unit 기대값과 #7333의 과거 marker 표현을 독립 crop/끝점 방향·소유 관계로 정정했다. [8단계](../../working/task_m100_7688_stage8.md)의 새 sample 5행은 각 5건이며 base/head anomaly path·bbox·겹침 폭/높이가 동일했다. 기존 baseline 행·래칫 검출 함수·허용치는 유지했다. 작은 `item id` 글상자 줄바꿈 차이는 해결했다고 주장하지 않는다. | 충족 |
| 회귀 추가·변경의 선행 조건 | 같은 원본·Print PDF의 Native/fresh WASM 모든 관련 페이지가 90% 이상인 뒤 검사/등록을 진행했다. 실물 화면 절대 좌표·전체 SVG hash 대신 내용 순서·선 끝점 소유·방향·intrinsic 형상·source 선택을 검사한다. 추가 대조군 5개 최저 99.84531%, #7333 전체 50쪽 최저 95.41718%. | 충족 |
| 90% 통과 후 직접·확대 판독 | crop의 전체 70행/아래 약 58행, 네 화살촉과 짧은/굵은 선, 수식 glyph·본문/탭 간격, #7333 영향 19·31·41·43쪽을 독립 Print와 직접 대조했다. [7단계](../../working/task_m100_7688_stage7.md)의 실제 Native/Studio 캡처와 확대 자료에 연결한다. 수식/탭 raster의 남은 차이 및 glyph/font 한계를 아래에 구분했다. 사용자가 같은 비교 자료에 시각 통과를 판정했다. | 충족 |
| 주장과 검증 범위 | 13문서 전체 64쪽 Native/fresh WASM TSV와 대표 실제 NativePNG/두 Studio backend를 확인했다. GPU·전체 style/곡선 조합은 미검증이며 이번 완료 범위에 포함하지 않는다. | 충족 |

## 검증 입력과 결과

원본/PDF의 저장소 경로·SHA-256·정상 출력 출처는 [13쌍 입력 원장](../../../samples/issue7688/README.md)에 있다.
Hancom 2020 `11.0.0.9136`, Print method 0, one-up 기준이며 #7333의 기존 정상 PDF는 재사용했다.
실제 실행 명령·argv·로그는 [7단계](../../working/task_m100_7688_stage7.md), [8단계](../../working/task_m100_7688_stage8.md),
[보고서](../../report/task_m100_7688_report.md)에 연결한다. ignored 증적은
`output/pr-review/renderer-backend-audit-20261009/final/` 및 `final/ci-fix/`에 보존한다.

| 검증 | 실제 결과 |
| --- | --- |
| Rust 필수 lint·정책 | fmt, Native/WASM32/workspace all-target Clippy, workspace build, 추가 Skia Clippy 및 고정 base 정책 PASS |
| 최종 전체 로컬 nextest | 10,553 PASS / 0 FAIL / 50 skipped |
| Native Skia/workspace lib 및 출력 | 4,109 PASS / 13 ignored; placeholder 2 PASS, 직접 PDF 4 PASS |
| CI 보정 뒤 코퍼스 래칫 | 84 PASS / 0 FAIL / filter skip 10,519; text-overlap 16 partition 전부 PASS |
| Studio | 1,825 PASS / 0 FAIL / 2 skipped; noEmit/build PASS, E2E manifest 151/151 PASS |
| 실제 브라우저 font 회귀 | 준비·Regular/Bold face 소비 및 자원 차단 fallback PASS; 수정 전 결함 FAIL 확인 |
| Render Diff 후속 로컬 확인 | Canvas 3쪽 PASS, 직접 PDF 3/3 PASS; 보조 보고서 4 warnings / 0 errors |

Native SHA-256 `c5ad2a3d672bcf500b86252bafda95894bfb8cd5c2539e89be61fd774def09ea`.
Docker WASM `cc5e9c9d86d2d3ac117a9184510bca661fb8cfaa6aef660707de2c9063575de2`,
JS `70cde06a369fa7fd4fc8bc8f3d6acaee158596ba1a116a2c72159002b0b5654e`.
pkg/public·실제 초기화 응답의 동일성을 확인했다. 제품 source와 검증한 21개 파일이 같아,
CI 보정 이후·문서 기록을 위해 동일 WASM 및 전체 Rust 회귀를 중복 실행하지 않았다.

### 정확한 code head의 GitHub Actions

위 로컬 결과와 별도로, 아래 모든 실행을 `7e71c6638a9bb3791fbacee8b37a2c5e2ced2d5a`에서 확인했다.

| workflow | 실행 결과 |
| --- | --- |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/37919787831) | success; Build & Test・Rust archive A/B/C/D・lint・Native・frontend package success |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/37919787396) | success |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/37919787642) | 4개 언어 Analyze 및 GHAS check success |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/37919787703) | success |
| [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/37919787724) | success |
| [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/37919786112) | success |

## 시각 증적과 남은 차이

13문서·전체 64쪽의 Native/fresh WASM 각각 최저 **93.49689%**다.
90% 미만·누락·측정 불가·쪽수 불일치가 없고 현행 gate의 26개 판정도 통과했다.
대표 6문서 전체 8쪽 실제 Native PNG 최저 96.18681%, Studio 첫쪽 6개는
Canvas2D 95.53281%, CanvasKit software 94.58234%다. 표 2·3쪽은 Studio 캡처 범위가 아니다.
#7333 영향 4쪽 실제 Native PNG 99.77949%, Canvas2D 99.52841%, CanvasKit 99.07962%다.
[28개 대표 PNG와 hash](../assets/issue_7688_renderer_backends/README.md)는 PR 본문에서 head 고정 raw 이미지로 표시했다.
[전체 Native TSV 첨부](https://github.com/user-attachments/files/33244345/issue7688-native-tsv.zip)는 원본 TSV 13개/64행이며,
fresh WASM TSV·두 경로 명령·provenance는 보고서의 ignored 증적 위치에 있다.
기존 이미지의 overlay는 RHWP-only 빨강 / PDF-only 파랑이다. 새 도구의 색상 규칙과 혼동하지 않는다.

엄격 backend parity는 report-only 24건 중 **16 PASS / 8 FAIL**(탭/수식 각 4건)이다.
그 차이의 실제 크기를 전후 대조했으며 허용치를 완화하지 않았다. 위 실루엣 통과로 완전한 pixel parity를 주장하지 않는다.
작은 글상자 줄바꿈/clip, font/raster 차이도 남는다. GPU readiness evaluated=0,
Windows/macOS·실제 GPU·모든 italic 원 face·모든 화살촉 모양/curve/arc·전체 크기가 완전히 불명인 crop은 미검증이다.
검증 절차와 후속 소비 경계는 [backend 검증 가이드](../../manual/verification/renderer_backend_verification.md)에 연결했다.

## Merge 후 contributor PR comment 계획

병합 승인을 받은 뒤 최종 merge SHA·위 code CI 및 trailing CI URL, 해결 범위와 #7688의 남은 범위를
이 self PR에 한국어 존댓말로 안내한다. [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md)을 연결하고,
위 28개 중 대표 review/overlay를 같은 asset의 **merge SHA 고정 raw URL**로 다시 표시한다.
문안은 UTF-8 Markdown 파일의 `--body-file`로 게시한 뒤 API로 원문·URL·한글/BOM 여부를 재조회한다.
현재는 계획만 기록하며 merge·issue close·post-merge comment를 완료로 쓰지 않는다.
