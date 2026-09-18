---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7251 검토

## 최종 판정

**메인터너 보정 후 수용 가능** — 통합 로컬 필수 검증 완료. 원 PR 직접 merge가 아닌 보정 통합 head를 수용한다.

2026-09-18 보정에서 overflow 판정과 실제 절삭이 같은 `leader_fill_spans`를 사용하도록
통일했다. run 내부의 점 구간도 필요한 만큼만 줄이고 앞 제목·뒤 쪽 번호와 원문을 보존한다.
저장 줄이 없거나 편집으로 저장 분할이 무효인 문단에는 이 절삭을 적용하지 않는다.

- `Ⅰ. 사업개요 TOC` + 점 150개 + `42`의 동일 run 및 style 경계 반례를 추가했다.
  수정 전 5개 중 1개 실패, 수정 후 **5/5 통과**. 짧은 일반 말줄임표는 보존한다.
- 일반 재래핑 음성 대조 #2525 **1/1**, #2291 **2/2 통과**.
- Native Skia 포함 CLI와 fresh WASM 빌드 통과. 최종 통합 게이트 결과는 아래 최종 통합 검증 절을 따른다.
- 기존 입력을 engine 2020으로 재변환하고 status `succeeded` 확인 후 기존 PDF를 교체했다.
  입력 HWP SHA-256 `e9bc5e78b412876ad3a810d01ba3b5ca5077e921f13cd15dde755c8f537bbeae`는 불변이다.
  새 PDF SHA-256 `b7c3d5056f09cf351e5e7c7c94844dc4700e2882bf67f2432183639a213cc7fa`,
  12쪽, 322062 bytes, 생성 KST 2026-09-18 15:26:31,
  Creator `Hwp 2020 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`.
  MCP job `ce550d36-4d00-411e-a78a-85528856184b`; 정상 점 글리프를 직접 확인했다.

새 PDF의 점 리더는 `Haansoft Batang`, rhwp 기본 환경은 문서의 `바탕체`를 사용한다.
명시적 글꼴 대체 진단에서 점 간격이 바뀌는 것을 확인했지만, 문서 전체 글꼴 대체를 제품
수정으로 적용하지 않았다. 기준 PDF의 동일 줄 안에서 제목은 `BatangChe`, 점만
`Haansoft Batang`인 것을 실제 span으로 확인했다(첫 줄 115점). 내장 `BatangChe`에는
U+2024 폭이 없어 일반 폭 fallback을 쓰는 반면 PDF 점 전진은 약 2.798pt다.
따라서 남은 점 밀도·기준선은 글리프 fallback의 별도 차이이고, 이번 절삭의 보존/넘침
계약이나 정상 PDF 확보 실패로 분류하지 않는다. 문서 전체 글꼴을 바꾸어 비교를 맞추지 않았다.

승인 범위는 **유효한 저장 한 줄 유지, 칸 내부 리더 표시, 제목·쪽 번호·원문 보존**이다.
기본 환경의 점 간격·기준선까지 한컴과 일치하거나 #6802 전체를 해결했다고 주장하지 않는다.
원 PR의 내부 run 반례 누락과 결함 있는 PDF라는 두 보류 사유는 `a165bd4bf`에서 해소했다.
아래 종전 PDF 비교는 **수정 전 기록**이며 최종 full gate는 공동 기록에서 갱신한다.

## 보정 후 Visual Sweep

새 PDF로 Native/fresh WASM p1·p2를 다시 캡처했다. CLI SVG의 Chrome raster 비교이며
Native Skia raster 전체 일치를 뜻하지 않는다. 새 PDF를 사용한 전 12쪽 layout ledger도 산출했다.

| 쪽 | Native | fresh WASM |
| --- | --- | --- |
| 1 | [review](../assets/pr7251_review/maintainer_20260918/native_review_001.png) · [overlay](../assets/pr7251_review/maintainer_20260918/native_overlay_001.png) | [review](../assets/pr7251_review/maintainer_20260918/wasm_review_001.png) · [overlay](../assets/pr7251_review/maintainer_20260918/wasm_overlay_001.png) |
| 2 | [review](../assets/pr7251_review/maintainer_20260918/native_review_002.png) · [overlay](../assets/pr7251_review/maintainer_20260918/native_overlay_002.png) | [review](../assets/pr7251_review/maintainer_20260918/wasm_review_002.png) · [overlay](../assets/pr7251_review/maintainer_20260918/wasm_overlay_002.png) |

## 최종 통합 검증

보정 제품 코드 `88f2f00da8412c769f34ef6bc3b72bc13402557b`. contributor 원 head는 아래 provenance에 별도 기록했다.
[공동 최종 검증](pr_7244_review_impl.md#최종-통합-검증)의 전체 nextest, Native Skia 3종,
fmt·Clippy 3종·workspace build·정책 검사, Studio 타입·단위·production build를 통과했다.
원 PR head CI를 재사용한 통과 주장과 구분한다. 해당 입력은 최종 Native/fresh WASM으로
다시 Visual Sweep했고 compare·standalone overlay·review와 남은 차이를 확인했다.
렌더 변경이 없는 진단·scaffold ID·래칫 자체에는 별도 시각 통과를 주장하지 않는다.

작업지시자의 PR 생성·CI 모니터링·merge·후속 처리 승인을 받았다. 통합 원격 최종 head의
CI·mergeability 확인은 아직 남아 있으며 완료 전 merge하지 않는다.

## Metadata·체리픽 provenance

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7251](https://github.com/edwardkim/rhwp/pull/7251) — 수정: 차례 칸의 점 채움 줄을 저장대로 한 줄로 두고 넘치는 점만 끊는다 (#6802) |
| 작성자·reviewer | planet6897 / jangster77 (검토 전 지정) |
| 원 base·head | `devel` / `40fd97a62606c0525f50c37f6f5b18310d04d5c7` |
| 규모 | 4 files, +331 / -5 |
| 조회 당시 mergeability | `MERGEABLE` / `CLEAN` — 참고 snapshot |
| 통합 base | `18a9fa85e955c220e5eb4d0143dc918a4de6be73` |
| 로컬 branch | `codex/planet-review-20260918` |
| 최초 검토 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
| 형식·주석·PDF 보존 | `30b9cca953848cb03dd16fcff6c7a548007ce623` — 실행 의미 변경 없음 |

| 적용 source commit | 로컬 commit |
| --- | --- |
| `40fd97a62606c0525f50c37f6f5b18310d04d5c7` | `a63d7a351b868b49c5244c9814d8792ae9f8aee0` |

원 head CI는 성공/skip/neutral 상태이며 통합 head CI를 대신하지 않는다. 재조회에서 원 head가 동일함을 확인했다. [CI 1](https://github.com/edwardkim/rhwp/actions/runs/35282349689) · [CI 2](https://github.com/edwardkim/rhwp/actions/runs/35282349658) · [CI 3](https://github.com/edwardkim/rhwp/actions/runs/35282349156) · [CI 4](https://github.com/edwardkim/rhwp/actions/runs/35282349696) · [CI 5](https://github.com/edwardkim/rhwp/actions/runs/35282349657) · [CI 6](https://github.com/edwardkim/rhwp/actions/runs/35282349274)

## 범위·조판 계약 검토

관련 이슈: [#6802](https://github.com/edwardkim/rhwp/issues/6802). 차례 리더 때문에 저장 줄을 재래핑하지 않고 넘는 리더 display_text를 줄인다.

저장 줄 수용 → 재래핑 생략 → display_text 절삭 → paint 흐름이다. 원 text는 남지만 화면에서 문자를 제거하는 분기이므로 제목·쪽번호 보존과 실제 점 채움 기준이 필요하다. 현재 3건의 대상 표본 통과만으로 모든 리더 위치를 승인하지 않는다.

주요 소비 경로: [src/renderer/composer.rs](../../../src/renderer/composer.rs).

파일명·문서 ID에 따른 제품 분기를 추가하지 않았다. 저장 정보/재조판·음성 대조·최종 paint 적용 범위의 미검증은 위 판정에 명시했다. 분할·이어받기가 범위에 없는 PR에는 해당 체크를 적용하지 않았다.

## 최초 검토 검증과 한계

- 통합 제품의 `issue_6802_cell_leader_fill_stored_line`: **3 tests run: 3 passed, 161 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 최초 검토 당시에는 전체 게이트가 미실행이었다. 아래 과거 기록을 최종 상태로 해석하지 않으며, 최신 결과는 최종 통합 검증 절과 공동 원장을 따른다.

2쪽의 5개 차례 항목은 단일 줄로 정리됐고 겹침은 사라졌다. PDF의 사각형 리더와 rhwp의 점은 명백히 달라, 자동 flag 0을 시각 일치로 읽지 않았다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## 최초 검토 Visual Sweep 증적

CLI Native SVG와 fresh WASM SVG를 각각 Chrome webfont 경로로 캡처했다. 이것은 Native Skia raster나 Studio CanvasKit 화면 캡처가 아니다. `fidelity_compare --text-only --export-all-svg --layout-ledger` 전쪽 원장을 산출해 후보를 확인했다. 아래 자동 flag와 ink-match는 보조 지표이며 시각 승인 그 자체가 아니다.

| 입력 | 쪽 | Native flag / 평균 ink-match | WASM flag / 평균 ink-match |
| --- | --- | --- | --- |
| toc_leader | 1, 2 | 0 / 12.38% | 0 / 12.21% |

| 증적 | Native | fresh WASM |
| --- | --- | --- |
| toc_leader p1 | [compare](../assets/pr7251_review/native_toc_leader_compare_001.png) · [overlay](../assets/pr7251_review/native_toc_leader_overlay_001.png) · [review](../assets/pr7251_review/native_toc_leader_review_001.png) | [compare](../assets/pr7251_review/wasm_toc_leader_compare_001.png) · [overlay](../assets/pr7251_review/wasm_toc_leader_overlay_001.png) · [review](../assets/pr7251_review/wasm_toc_leader_review_001.png) |
| toc_leader p2 | [compare](../assets/pr7251_review/native_toc_leader_compare_002.png) · [overlay](../assets/pr7251_review/native_toc_leader_overlay_002.png) · [review](../assets/pr7251_review/native_toc_leader_review_002.png) | [compare](../assets/pr7251_review/wasm_toc_leader_compare_002.png) · [overlay](../assets/pr7251_review/wasm_toc_leader_overlay_002.png) · [review](../assets/pr7251_review/wasm_toc_leader_review_002.png) |

## Merge 후 contributor PR comment 계획

승인된 통합 PR의 최종 head CI와 실제 merge를 확인한 뒤 원 PR·관련 이슈에 한국어로
merge SHA·CI URL·수정 계약·실제 검증 범위·남은 차이를 기록하고 기여에 감사한다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.

- `mydocs/pr/assets/pr7251_review/maintainer_20260918/`: `native_review_001.png`, `native_overlay_001.png`, `wasm_review_001.png`, `wasm_overlay_001.png`, `native_review_002.png`, `native_overlay_002.png`, `wasm_review_002.png`, `wasm_overlay_002.png`를 실제 이미지로 포함한다.

이미지는 `https://raw.githubusercontent.com/edwardkim/rhwp/<실제-merge-SHA>/<위-경로>`로 표시한다.
UTF-8 파일과 `--body-file`로 게시하고 한국어·실제 head·이미지 URL을 API와 HTTP로 재확인한다.

## 이슈·다음 단계

#6802 전체 종료 금지. 이번 표시 절삭의 정확한 경계를 확인한 후 판단한다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.
