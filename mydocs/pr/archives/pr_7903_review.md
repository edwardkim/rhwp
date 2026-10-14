---
kind: report
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-14
---

# PR #7903 리뷰 — Visual Sweep 직접 PNG와 검증 계약 보완

## 최종 판정

**승인 — 메인터너 보정 후 승인.** 직접 PNG 도구·CLI 자기서술·페이지별 회귀 지침 범위의
로컬 검증과 정확한 code head CI가 완료됐고 코드 PR을 squash merge했다.
#7902의 화학제품 쪽번호·행 나눔 HWPX·giant overfill 및 전체19문서 보정은 후속 범위이며
해결·종료로 기록하지 않는다. 사용자 선택 옵션2에 따라 이 review와 오늘할일은 merge 후
별도 문서 PR로 준비하고, 생성·push 전 사용자 승인을 받는다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#7903](https://github.com/edwardkim/rhwp/pull/7903) / jangster77 / devel |
| 검증한 code head | `fbea4a697c5ec5d04fb36d6c377830a47b557b33` |
| 원 검증 base | `b3eaf7b10b15a2d40810d42acdcf4de6c1ba026b` |
| 렌더링 producer source | `d56b6c1106bbe0292c1c11e3bfaba2286cdfb43c` |
| merge commit | `116b2b6ae0c1ec655c2db9878b1c8cf1bdd67cc4` / 2026-10-14 22:12 KST |
| 관련 이슈 | [#7902](https://github.com/edwardkim/rhwp/issues/7902) 참조, close 없음 |
| 판정 시 원격 상태 | 실제 CI·정책 성공, 대기/실패0, mergeable true·clean, merge 완료 |
| 변경 규모 | 44파일, 코드·도구·문서·대표 PNG |

merge commit과 검증 code head의 tree는 모두 `d697a7636a141c0cec6a938d1fbc9b4945315ca5`였다.
CI 추정이나 다른 SHA의 결과를 사용하지 않았다.

## 변경과 실제 소비 경로

- 기본 PNG: Native Skia `export-png --profile print`와 fresh WASM Print Canvas.
  SVG 진단은 명시적인 `--rasterizer svg`로 유지한다.
- Native `export_png` → `NativePngExporter` → 동일 Core의 profile별 page layer tree →
  `SkiaLayerRenderer::render_png_with_options`. 명령별 글꼴 snapshot을 재사용하고
  같은 문서의 선택 쪽 render tree를 함께 산출한다. 기본 screen PNG 계약은 유지했다.
- WASM은 작업자당 브라우저·탭·문서를 재사용하고 실제 글꼴·그림 준비 뒤 Print Canvas를
  캡처한다. `--page-workers N`, backend·producer SHA 및 resume 정합성을 검사한다.
- 글꼴 공급 도구의 cmap 보완은 잘못된 U+FFFF notdef 종단만 교정하며 원본 글꼴을 보존한다.
  HTTP 읽기 실패는 고정500 응답, 상세는 로컬 stderr이며 allowlist·SHA 검사는 유지했다.
- PNG의 MCP feature 스키마·업무 프로필·필드 사전·공통 출처 지도·실제 호출 레시피와
  스킬 카탈로그/fixture를 일치시켰다. JSON은 표준1.0·false 출처 표지·빈 경로 목록이며
  산출 PNG와 별도 render tree의 문서 내용은 재소비 시 미신뢰 데이터다.
- 완료된 페이지의 Native/fresh WASM 각각98% 이상·직접 판독 조건에서만 ±2px 좌표
  회귀를 허용한다. 관계 검사·전쪽90% 최소 gate·98% 유지 조건을 함께 적용한다.

### 조판 원칙 판정

| 항목 | 판정·근거 |
| --- | --- |
| 독립 기준과 일반성 | 충족: 같은 원본·독립 한컴 Print PDF, 문서 ID/좌표 예외 없음 |
| 공통 측정·배치 결과 | 조판 알고리즘 변경 비해당: 기존 Core layer tree를 두 출력이 소비 |
| 높이·컷·소유 유닛 | 분할/이어받기 변경 비해당: layout/typeset의 컷·예약 높이를 바꾸지 않음 |
| 저장 LineSeg 유효성 | 변경 비해당: 저장 정보 수용·재조판 계약을 수정하지 않음 |
| 실제 출력 | 충족: 대상 두 문서 전쪽26비교, 미달/누락/쪽수 차이0 및 대표 영역 직접 판독 |
| 신규 검사 의미 | CLI/MCP·JSON 공급/출처 계약 검사, 배치 px/golden 고정 없음 |

Studio·확장과 공통 WASM 렌더러 소스는 변경하지 않았다. Windows 실행 성능,
모든 내장 글꼴 조합 및 #7902 전체 문서의 최신 전쪽 출력은 미검증이다.

## 검증 입력의 commit 포함

아래 원본·기준 PDF는 merge commit `116b2b6ae`의 실제 object와 실행 파일의 SHA-256이
일치했다. 기존 저장소 경로를 사용했으며 새 사본·글꼴 파일을 공개하지 않았다.

| 입력 / 독립 한컴 Print PDF | SHA-256 |
| --- | --- |
| `samples/table-vpos-01.hwp` | `2e6c5bb4bf29f60d97d332414a0ded023b894e59ae49b5f9d6bb11476d39d766` |
| `pdf/table-vpos-01-hwp-2020.pdf` | `3439059430495acc264bb6924195d5a8065265e405fb6e68cd05869f506185cf` |
| `samples/issue2004_cell_image_stack.hwp` | `c633fd086323075ab25777d8ee10357869d037bd17c9e74d444d5b8297c40d50` |
| `pdf/issue2004_cell_image_stack-hwp-2020.pdf` | `b5e2f962620d8d7cdd2ec8b10571c53fbe959eb9ce7b3428ff09a595f12fee90` |

## 로컬·원격 검증

공용 `/home/tsjang/rhwp/target/pr-review`, Cargo jobs8, 순차 검증이다.

```bash
CARGO_BUILD_JOBS=8 cargo nextest run --cargo-profile release-test \
  --target-dir /home/tsjang/rhwp/target/pr-review \
  --tests --test-threads 12 --no-fail-fast
```

- **최종10,285개 통과 / 실패0 / 제외45**, exit0. 컴파일5분13초·검사1129.782초.
- fmt, Native Skia·기본·WASM·workspace all-targets Clippy, workspace build,
  manifest의 고정 base 비교 및 문서 링크/공백 검사 통과.
- Native PNG 계약2개·Native 출처10개·스킬 계약12개 통과. 실제 PNG 표지 검사는
  수정 전 코드와 동일한 고정 CLI에서 표지 누락으로 FAIL, 보완 CLI에서 PASS였다.
- 수정 전 전체는5개 실패였다. MCP 스키마·프로필·사전·출처 선언/레시피를 보완한 첫
  전체 실행에서 기존5개는 PASS, 추가 스킬 연결 누락2개가 FAIL했다. 자료 보완 후 최종
  전체가 PASS했다. 기존 검사는 유지했고 baseline·허용치를 완화하지 않았다.
- CI classifier 동일8명령 Node219·Python38, HTTP 실제 요청4개, Node 정책177,
  Python workflow66·Visual Sweep135와 actionlint 통과 기록을 연결했다.
- 실제 최신 head [CI](https://github.com/edwardkim/rhwp/actions/runs/38566510885),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38566510871),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38566510761),
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/38566510858),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/38566510757),
  [Skill router](https://github.com/edwardkim/rhwp/actions/runs/38566510860)와 CI Impact Policy 성공.
  Archive A/B/C/D·Native Skia·Lint의 실제 완료를 확인했다.

원시 로그와 producer/fingerprint는 ignored
`output/pr-review/visual-sweep-direct-png-20261014/`의
`final-complete-validation-result.json`, `final-full-regression.log`,
`pr7903-final-ci-and-merge-preflight.json`에 있다. 상세 구현·과거 보완은
[단계 기록](../../working/task_m100_7902_stage1.md)을 따른다.

## 전쪽 TSV와 직접 판독

같은 Print profile·2022 compat·96dpi·입력/기준 PDF·글꼴 공급에서 Native13쪽과 fresh
WASM13쪽을 확인했다. 렌더링 source는 d56b6c110이다. 이후 자기서술 보완은 paint/layout을
바꾸지 않아 해당 증거를 재사용했다. HTTP 보안 보완 source b50405589에서는 fresh WASM
13쪽을 재출력해 이전 PNG와 전쪽 바이트 동일함도 확인했다. 최신 metadata head에서 새
paint 캡처를 수행했다고 기록하지 않는다.

| 문서 | 쪽수 | Native 최저 | fresh WASM 최저 | 미달/누락/쪽수 차이 |
| --- | ---: | ---: | ---: | --- |
| table-vpos-01 HWP | 5/5 | 98.84071% | 98.50488% | 0 |
| issue2004_cell_image_stack HWP | 8/8 | 95.75413% | 96.55826% | 0 |

[Native/fresh WASM TSV 원본](https://gist.github.com/jangster77/3c3ebc08de3f4d618045feec75b742f2).
대표 cellimg1쪽의 로고·제목·표 외곽·본문 줄바꿈·간격을 실제 PDF와 비교했다.
누락·큰 위치 이동·겹침은 보이지 않았고 글꼴 획/래스터 차이는 남는다. 점수는 실루엣
보조값이며 전체 fidelity와 동일시하지 않는다.

![Native review](https://raw.githubusercontent.com/edwardkim/rhwp/116b2b6ae0c1ec655c2db9878b1c8cf1bdd67cc4/mydocs/pr/assets/issue7902-visual-sweep-png/native-cellimg-review-001.png)

![Native overlay](https://raw.githubusercontent.com/edwardkim/rhwp/116b2b6ae0c1ec655c2db9878b1c8cf1bdd67cc4/mydocs/pr/assets/issue7902-visual-sweep-png/native-cellimg-overlay-001.png)

![fresh WASM review](https://raw.githubusercontent.com/edwardkim/rhwp/116b2b6ae0c1ec655c2db9878b1c8cf1bdd67cc4/mydocs/pr/assets/issue7902-visual-sweep-png/wasm-cellimg-review-001.png)

![fresh WASM overlay](https://raw.githubusercontent.com/edwardkim/rhwp/116b2b6ae0c1ec655c2db9878b1c8cf1bdd67cc4/mydocs/pr/assets/issue7902-visual-sweep-png/wasm-cellimg-overlay-001.png)

## 실제 파일 성능 비교

동일5쪽·source에서 TSV 완료까지의 단회 관측이며 빌드는 제외했다. 다른 검증도 실행 중이었다.
Native SVG/Chrome1작업자40.27초·208.27MiB → PNG4작업자11.62초·1.84MiB로71.1% 단축,
fresh WASM SVG4작업자64.42초·470MiB → PNG4작업자36.48초·17.59MiB로43.4% 단축됐다.
Native PNG1/4의 PNG·TSV는 바이트 동일했다. SVG/PNG paint 동일이나 이 측정을
upstream/devel 대비 무회귀 TSV 비교라고 주장하지 않는다.

## 문서 지침·merge 후속

후속 문서 PR에서 [devel 대비 페이지별 하락 재검토](../../manual/verification/visual_verification_governance.md#upstream-devel-대비-페이지별-실루엣-하락-재검토)를
신설한다. Native/fresh WASM 각각 같은 조건의 최신 devel 전쪽 TSV와 후보를 비교해
0.01%p 이상 하락한 쪽이 하나라도 있으면90% 이상이어도 재검토한다. #7902 후속 작업에는
이 비교를 실제 적용하며, 위 동일 source의 backend 성능 관측으로 대체하지 않는다.
거버넌스에 남아 있던100% 문구도 완료 페이지98% 이상으로 교정한다.

코드 PR merge는 완료됐다. 문서 PR 생성·push는 사용자 승인 대기이며, 승인 뒤 실제
문서-only fast-pass와 required checks를 확인한다. 운영 기록 반영 후 최종 devel sync,
merge SHA 이미지의 한국어 comment, duration 갱신 결과 및 소유가 확인된 작업 전용
branch 정리를 수행한다. #7902는 열린 상태로 유지한다.
