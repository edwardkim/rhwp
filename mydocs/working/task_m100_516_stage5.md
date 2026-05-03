# Task #516 Stage 5 완료 보고서 — 시각 판정 + 본 task 마무리 결정

## 시각 판정 결과 (작업지시자)

| 결함 | 결과 |
|------|------|
| 1 — 엠블럼 (JPEG) 흰색 배경 투명 처리 | ❌ **실패** |
| 2 — BehindText 그림 위 텍스트 클릭 안 됨 | ❌ **실패** |
| 3 — 워터마크 효과 (multiply blend) | ❌ **미적용** |
| 부분 정합 — 회색조 적용 | ✅ 성공 |

## 본질 분석 (Stage 4 의 한계)

세 결함의 공통 근본은 **rhwp-studio 의 web 렌더가 단일 Canvas 2D 평면**:

- 결함 1 (배경 투명): JPEG 알파 채널 부재 + 단일 평면에서 blend 불가
- 결함 2 (hit-test): 단일 Canvas 의 픽셀 hit-test 한계 (BehindText z-order 의미 손실)
- 결함 3 (multiply blend): blend mode 적용을 위한 별도 layer 필요

본 task 의 Stage 4 정정 (CSS filter 적용) 은 **회색조/밝기/대비** 만 처리 가능하고, 위 3 결함은 **다층 레이어 인프라** 도입 없이는 본질 정정 불가.

## 다층 레이어 인프라 도입 결정 (Discussion #529)

작업지시자 결정으로 **다층 레이어 아키텍처** 를 별도 task 로 분리. 후보 3 안 (Multi-Canvas / WebGPU / HTML Hybrid) 비교 보고서 작성:

- **로컬 보고서**: `mydocs/tech/multi_layer_rendering_strategy.md`
- **GitHub Discussion**: [#529](https://github.com/edwardkim/rhwp/discussions/529) — Ideas 카테고리
- **목적**: 후보 결정 과정 기록 (의사결정 권위 자료)

## 본 task #516 의 마무리 정책

**Stage 4 까지의 결과로 본 task 마무리** (수행 계획서의 옵션 1):

| 영역 | 본 task 처리 | 비고 |
|------|--------------|------|
| 1. Web Canvas effect 적용 | ✅ Stage 4 (CSS filter, 회색조/bc) | 부분 정합 (한컴 정답지와 시각 차이는 다층 레이어 task 에서 multiply blend 로 해결) |
| 2. brightness/contrast 적용 | ✅ Stage 4 | 동일 |
| 3. effect+bc 합성 | ✅ Stage 4 | 동일 |
| 4. PCX 알파 + 워터마크 결합 | ✅ Task #514 의 PCX 알파 + 본 task CSS filter 결합 검증 | 학교 로고 (PCX) 는 정상 (배경 투명) |
| 5. dump `[image_attr]` 메타 | ✅ Stage 3 | 완료 |
| 6. AI JSON `watermark.preset` 필드 | ✅ Stage 3 | 완료 |
| **결함 1, 2, 3 (엠블럼 투명/hit-test/multiply blend)** | ❌ → **별도 task 분리** | 다층 레이어 인프라 도입 후 재진행 |

## 다층 레이어 task 의 후속 영역

다층 레이어 인프라 도입 task 가 본 task 의 결함 1, 2, 3 을 자연스럽게 해결할 영역:

- BehindText / InFrontOfText 그림을 별도 layer 로 분리
- multiply blend mode 적용 (워터마크 본질)
- pointer-events 정책으로 hit-test 정확성
- 그림 효과 (`<img>` 또는 GPU shader 기반) 의 더 정확한 적용 — CSS filter 의 SVG feComponentTransfer 매핑 차이 해소

## 검증 게이트 (최종)

| 게이트 | 결과 |
|--------|------|
| `cargo build --lib` | ✅ Finished |
| `cargo build --release` | ✅ Finished |
| `cargo test --lib` | ✅ **1110 passed** (회귀 0) |
| `cargo test --test issue_516` (신규) | ✅ **5 passed** |
| `cargo test --test issue_418/501/514` (회귀 0) | ✅ 1 + 1 + 3 |
| `cargo test --test svg_snapshot` | ✅ **6/6 passed** |
| `paint::json::tests` (PR #510 호환) | ✅ 4/4 passed |
| `cargo clippy --lib` | ✅ 0 건 |
| `cargo clippy --test issue_516` | ✅ 0 건 |
| WASM 빌드 | ✅ 4,453,541 bytes (+1,337 vs Task #514) |
| `rhwp-studio/public/` 동기화 | ✅ |
| **작업지시자 시각 판정** | ⚠️ **부분 정합** (회색조 ✅, 워터마크 본질 ❌ → 다층 레이어 task) |

## 산출물 (본 task 전체)

- `mydocs/plans/task_m100_516.md` (수행 계획서)
- `mydocs/plans/task_m100_516_impl.md` (구현 계획서)
- `mydocs/working/task_m100_516_stage{1,3,4,5}.md` (단계별 보고서)
- `mydocs/tech/multi_layer_rendering_strategy.md` (다층 레이어 후보 3 안 기술 조사)
- GitHub Discussion [#529](https://github.com/edwardkim/rhwp/discussions/529) (Ideas 카테고리)

## 다음 단계

Stage 5 완료 보고서 승인 후:
1. 최종 보고서 (`mydocs/report/task_m100_516_report.md`) 작성
2. orders 갱신 (본 task 완료 + 다층 레이어 task 후속)
3. local/task516 → local/devel merge → devel push
4. 이슈 #516 close (commit 의 devel 머지 검증 후)
5. 신규 이슈 등록: 다층 레이어 인프라 (Discussion #529 결정 후 후보 확정 시)
