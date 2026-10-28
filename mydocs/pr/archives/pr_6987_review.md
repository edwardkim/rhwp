# PR #6987 리뷰 — 연속 줌 정착과 페이지 교체

- 검토일: 2026-09-10
- PR: https://github.com/edwardkim/rhwp/pull/6987
- 관련 이슈: #6040 (열림, 자동 종료 참조 없음)
- 작성자/실행 계정: `postmelee`; collaborator self-review 경로
- 대상 head: `e3f5dfadeefd28bdafe27e61ae20cc0ce04e13c2`
- 기준 devel: `37bd46a72f9fd9ffd709e35244df79c00e789780` — 대상 head의 직접 부모
- 작성 시점 참고값: Open, non-draft, MERGEABLE. 최신 head와 CI는 merge 직전 다시 확인한다.
- 범위: 1 commit, Studio 제품 코드·테스트 및 #6040 계획·계측·결과 문서 53개 파일. Rust/Cargo, sample, golden, CI workflow 변경 없음.

## 경로와 검토 범위

기본 경로는 `collaborator_self_merge`, 보조 경로는 `intake_and_review`, `local_validation`,
`visual_fixture_evidence`, `rework_and_exceptions`다. 각 문서와 PR review 라우터·선택표를 읽었다.
검토는 독립 임시 review worktree에서 수행했다. 기록 push 단계에서 `review_only_fast_pass`도 읽었으며,
제품 코드·테스트를 바꾸지 않고 검증한 code head 뒤에 이 기록과 대표 PNG만 별도 commit으로 추가한다.

연속 휠 입력의 quiet/animation 수렴, generation별 작업 취소, strict 갱신, renderer/document 전환,
지연 이미지 작업 소유권, 페이지별 preview 교체, visible DPR 보호와 offscreen 예산 반환을 검토했다.
관련 이슈의 기존 CSS preview·authoritative geometry·pointer/중앙 앵커·live 열 전환 보존 요구와 대조했다.
개발 계측 코드와 결과보고서의 수치 해석도 확인했다.

## 발견 사항

검토한 코드와 실행 시나리오에서 수정 요청으로 제시할 재현 가능한 결함을 찾지 못했다.
기존 bitmap 유지 중의 stale zoom 검사와 이미지 microtask 시작/완료 가드는 추가 회귀 테스트로 확인했다.
물리 scale이 같아도 active surface의 rendered zoom이 다르면 visible/retained 작업을 다시 예약하는 경로가 있다.
모든 가능한 캐시·입력 순서의 무결함을 보증하는 결과는 아니다.

## 완료한 검증

| 검사 | 결과 |
| --- | --- |
| `git merge-tree --write-tree upstream/devel <head>` | 충돌 없음; tree `57abb84ccf982b397745f9af3beb6e5b7a6968a6` |
| `git diff --check HEAD^ HEAD` | 통과 |
| Studio `npx tsc --noEmit` | exit 0 |
| Studio `npm test` (sandbox 밖) | exit 0; 1,645개 중 1,643 pass / 2 skip / 0 fail |
| `zoom-dialog-transaction.test.mjs --mode=headless` | exit 0; 37개 확인 항목 PASS |
| 실제 Canvas2D 문서 smoke | exit 0; 아래 최종 상태와 화면 직접 확인 |
| 실제 CanvasKit 요청 문서 smoke | exit 0; 같은 시나리오의 최종 상태와 화면 직접 확인 |

브라우저는 macOS Chrome 152, 1280×720 CSS px, DPR 2, `samples/exam_kor.hwp` 20쪽을 사용했다.
100→34→50→100→200% smooth zoom에서 보이는 페이지의 rendered zoom과 최종 요청이 일치하고,
표시 폭이 논리 페이지 폭에 일치했다(34%의 CSS 반올림 오차 약 0.01px).
200%에서 클릭 없이 6쪽으로 이동한 후 DPR 2 복원을 확인했다.
Ctrl-wheel 20건(16ms 간격, 축소 10건·확대 10건) 뒤 양쪽 backend 모두 약 200%로 수렴했으며,
pending=false, visible/prefetch queue=0, 예약 frame/idle/scroll-settle 없음이었다.
화면에서는 본문·표제·눈금자가 표시되고 최종 캔버스가 공백으로 남지 않는 것을 확인했다.
이 검사는 핀치 왕복의 앵커 오차를 정량 측정하지 않았다.

WASM은 source PR 개발 작업공간의 동일 진단 빌드를 사본으로 재사용했다.
SHA-256: `6ee336d08591621b5e5da087014f5a3eebc999325eba726c1ddab8a2d943e719`.
최초 browser 실행은 macOS Chrome 경로 미지정, 다음 실행은 symlink된 WASM의 Vite 403으로 실패했다.
Chrome 경로 지정과 review worktree 내 WASM 사본 배치 후 재실행한 결과만 위 통과 수치에 포함했다.
제품 코드를 고쳐서 검증 환경 문제를 우회하지 않았다.

Rust 변경이 없어 Rust 전체 회귀·Clippy·WASM 재빌드는 생략했다. 로컬 production build는 재실행하지 않았고,
동일 head의 [Frontend package gates 및 Build & Test](https://github.com/edwardkim/rhwp/actions/runs/34453113248) 성공을 확인했다.
동일 head의 [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/34453113051),
[CodeQL 분석](https://github.com/edwardkim/rhwp/actions/runs/34453113154),
[adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34453113200),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/34453113288)도 성공했다.
최종 `gh pr checks`는 exit 0, 실패·대기 없음이었다. 영향 범위 밖 Rust/WASM job의 skipped를 실행 통과로 세지 않았다.

## 직접 시각 확인과 한계

Canvas 출력 경로 변경이므로 실제 브라우저 검증을 수행했다. 한컴 PDF fidelity 변경 주장은 없어
PDF/SVG visual sweep은 수행하지 않았으며 pixel_match·visual_accuracy_proxy_percent·자동 flagged 수는 산출하지 않았다.
대표 화면은 [Canvas2D 6쪽 읽기 200%](../assets/pr6987/canvas2d-reading-200.png)와
[CanvasKit 역방향 핀치 후 200%](../assets/pr6987/canvaskit-reverse-pinch.png)다. 두 PNG를 직접 열어 확인했다.
브라우저 글꼴 차이 및 OS raster 차이에 대한 backend 간 픽셀 동일성을 주장하지 않는다.

SHA-256 출처 고정:

- 원본 `samples/exam_kor.hwp`: `0315576fb25dd29ad3b6b188ee2539d0e8d31c15b74847be801c2186a97aac69`
- Canvas2D 대표 PNG: `89244debb6ec329b3b86e3706ffcfa67d535f06c209271ec9e8fe30e528051c8`
- CanvasKit 대표 PNG: `890c1dbc691fce075cf69e370687746c9963741e272cb5358891c9108a6169f8`

Firefox/Safari 실제 트랙패드, 정량 앵커 오차, 500% 메모리 peak 및 제출 head의 시간 성능 A/B는 이번 리뷰에서
독립 재측정하지 않았다. PR 본문의 개발 단계 성능 수치는 원 작성자 증적으로 구분한다.
visible raw DPR 보호에 따른 메모리 증가와 이미 시작한 대형 raster를 선점하지 못하는 한계는 PR에 명시되어 있다.

## Merge 후 contributor PR comment 계획

절차 정본: [Visual Sweep GitHub comment 규약](../../manual/verification/visual_sweep_guide.md#github-merge-comment).
comment를 별도로 승인받으면 위 실제 UI 검증 범위와 미측정 항목을 그대로 기록한다.
PDF visual sweep 수치나 전체 성능 개선율은 작성하지 않는다.
대표 PNG가 merge commit에 포함된 뒤에만 다음 형식의 URL을 사용한다.

`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr6987/canvas2d-reading-200.png`

`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr6987/canvaskit-reverse-pinch.png`

게시 시 UTF-8 본문 파일을 `--body-file`로 전달하고 API로 Markdown·이미지를 다시 확인한다.
2026-09-10 사용자가 이 기록과 대표 PNG를 현재 PR에 push하도록 승인했다.
후속 head의 CI는 사용자가 직접 확인해 다시 알려주기로 했으며, 그 전에는 merge하지 않는다.

## 최종 판정

- 판정: 승인
- 의미: 검토한 head에 대해 코드 수정 요청 사항 없음. 위 로컬·CI·브라우저 검증 범위의 수용 판단이다.
- merge 전 조건: self-review 기록·대표 asset의 PR 포함 확인, 사용자의 후속 head CI 확인, merge 직전 최신 head·mergeability 재확인 및 사용자 merge 승인.
- 원격 조치 범위: 승인받은 review-only commit push까지만 진행한다. GitHub review/comment, metadata 변경, merge, issue close는 이 단계에서 수행하지 않는다.
