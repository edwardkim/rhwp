# Task #937 최종 결과 보고서

## 이슈

[Issue #937](https://github.com/edwardkim/rhwp/issues/937) — 복학원서 서명란 `(인)` 기호 렌더링 불일치

- 마일스톤: M100 — v1.0.0 조판 엔진
- 브랜치: `local/task937`
- 기준 브랜치: `upstream/devel` `b8710d92`
- 원본 문서: `samples/복학원서.hwp`
- 권위 자료: `pdf/복학원서-2022.pdf`

## 완료 커밋

| 커밋 | 내용 |
|------|------|
| `67b6d626` | Stage 1: 서명란 PUA 재현 테스트 |
| `9d016d73` | Stage 2: `U+F012B` 표시 문자열 치환 |
| `47fe0146` | Stage 3: 복학원서 SVG 회귀 검증 |
| `86414ada` | Stage 3 피드백: `U+F081C` filler 렌더 숨김 |

## 결함 본질

### 1. 서명란 `(인)` 누락

`samples/복학원서.hwp`의 서명란 `(Signature)` 앞 기호는 원본 IR에서 한컴 PUA `U+F012B` 1글자로 저장된다. 기존 렌더러는 이 값을 표시 문자열로 확장하지 않아 rhwp-studio/SVG에서 깨진 PUA 글리프가 그대로 보였다.

정정 방향:

- IR의 원문 `U+F012B`는 유지한다.
- 렌더/측정 경로에서만 `U+F012B -> "(인)"`으로 확장한다.
- 다문자 치환으로 인한 폭 계산 부작용을 막기 위해 `display_text`와 `effective_text_for_metrics()` 경로를 사용한다.

### 2. 하단 주석 앞 깨진 글리프

작업지시자 시각 검증에서 하단 왼쪽 주석 앞 깨진 글리프가 추가 확인되었다. 기준 PDF를 렌더링해 확인한 결과 해당 위치는 `(인)`이 아니라 `※` 주석 시작부였다.

현재 SVG에는 정상 `※` 옆에 HWP TAC filler `U+F081C`가 텍스트 노드로 중복 출력되고 있었다. `U+F081C`는 레이아웃 측정에서는 0폭 filler로 보존해야 하지만, 실제 렌더 출력에는 글리프로 표시되면 안 된다.

정정 방향:

- 측정 경로에서는 `U+F081C` 원문을 유지해 기존 0폭 규칙을 적용한다.
- SVG/Canvas/HTML/Skia 렌더 출력 직전 `expand_pua_render_text()`에서 `U+F081C`를 숨긴다.

## 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/composer.rs` | `U+F012B -> "(인)"` 표시 문자열 치환, `U+F081C` 렌더 출력 숨김, 측정용 effective text 보정 |
| `src/renderer/layout/paragraph_layout.rs` | right-tab/line 폭 측정에 `effective_text_for_metrics()` 적용 |
| `src/renderer/svg.rs` | 텍스트 출력에 `expand_pua_render_text()` 적용 |
| `src/renderer/web_canvas.rs` | Canvas 텍스트 출력에 동일 치환 적용 |
| `src/renderer/html.rs` | HTML 텍스트 출력에 동일 치환 적용 |
| `src/renderer/canvas.rs` | native canvas 텍스트 출력에 동일 치환 적용 |
| `src/renderer/skia/text_replay.rs` | Skia replay 텍스트 출력에 동일 치환 적용 |
| `tests/issue_937.rs` | 복학원서 IR/SVG 회귀 테스트 추가 |
| `tests/golden_svg/issue-677/bokhakwonseo-page1.svg` | 서명란 `(인)` 반영, 하단 `U+F081C` filler 출력 제거 |

## 검증 결과

```bash
cargo test --test issue_937
cargo test --test svg_snapshot issue_677_bokhakwonseo_page1
cargo test --test issue_826
cargo test --lib effective_text_for_metrics
cargo check --features native-skia --lib
cargo run --bin rhwp -- export-svg samples/복학원서.hwp -o output/svg/task937
docker-compose --env-file .env.docker run --rm wasm
```

결과:

- `issue_937` — 4개 통과
- `issue_677_bokhakwonseo_page1` — 통과
- `issue_826` — 4개 통과
- `effective_text_for_metrics` — 4개 통과
- `native-skia` feature check — 통과
- SVG 산출물 생성 — 통과
- WASM 패키지 재생성 — 통과

`cargo test --lib effective_text_for_metrics`의 기존 warning 6건은 이번 작업과 무관한 기존 경고이며 신규 경고는 확인되지 않았다.

`export-svg` 실행 시 기존 복학원서 문단 16의 `LAYOUT_OVERFLOW` 2.5px 로그가 유지된다. 이는 Task #677 이후 허용 영역으로 남은 기존 상태이며, 이번 PUA 치환 작업으로 새로 증가하지 않았다.

## rhwp-studio 확인

WASM 재빌드 후 `rhwp-studio`를 재시작했다.

```text
http://127.0.0.1:7700/
```

Headless Chrome으로 Studio에서 `복학원서.hwp`를 로드하고 `renderPageSvg(0)` 결과를 확인했다.

- `(인)(Signature)` 포함
- 원본 `U+F012B` 출력 없음
- 원본 `U+F081C` 출력 없음
- 깨진 filler 글리프(`󰠜`) 출력 없음
- 하단 주석 `※` 2개 유지
- 빨간 `㊞` 도장 유지

## 단계별 보고서

| Stage | 보고서 |
|-------|--------|
| Stage 1 | `mydocs/working/task_m100_937_stage1.md` |
| Stage 2 | `mydocs/working/task_m100_937_stage2.md` |
| Stage 3 | `mydocs/working/task_m100_937_stage3.md` |

## 결론

Task #937의 본질 결함인 서명란 `(인)` 렌더링 불일치를 정정했다. 작업지시자 시각 검증에서 추가 확인된 하단 왼쪽 깨짐은 `(인)`이 아니라 `U+F081C` TAC filler 출력 문제였고, 기준 PDF에 맞게 실제 렌더 출력에서 제거했다.

현재 `local/task937` 브랜치는 최종 보고서와 오늘할일 갱신 후 PR 준비 가능한 상태다.
