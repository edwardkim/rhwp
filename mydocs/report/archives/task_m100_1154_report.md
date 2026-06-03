# Task #1154 최종 결과 보고서 — 중복 Pic 컨트롤 스케일 미스매치 잔상 제거

## 1. 이슈 / 목표

- GitHub Issue: [#1154](https://github.com/edwardkim/rhwp/issues/1154)
- 브랜치: `local/task1154` (베이스 `local/devel`)
- 목표: 같은 `bin_data_id` 의 두 Pic 컨트롤이 세로로 인접 겹쳐 그려져 미세한
  스케일 차이로 나타나는 이중 라인 잔상을 제거. 한컴 정합도 유지.

## 2. 결론

- exam_eng.hwp page 2 의 박스 효과 잔상 제거 완료. 한컴 권위 PDF 와 시각 정합 ✓
- Stage 1 식별 17 영향 sample 중 exam_eng 만 fix 적용, 나머지 16 sample (의도적
  효과 케이스) 페이지 단위 100% 불변
- 일반 회귀 7 sample (페어 없음) 페이지 단위 100% 불변
- 단위 테스트 1318 + 신규 11 = clip algorithm 검증 통과
- WASM 빌드 정상 통과

## 3. 채택한 접근 (옵션 3 — Clip below to top of above)

같은 bin_id, 같은 x/width 의 두 Pic 페어 (A=아래, B=위) 에서 A 의 bbox.height
와 crop.bottom 을 B.y 까지 정확히 축소. B 가 100% 위에 덮으므로 표시 결과는
한컴과 동일하면서 안티엘리어싱 차이로 인한 잔상은 사라진다.

### 알고리즘 strict 5 조건

```
모두 만족 시 clip 적용:
1. A.bin_data_id == B.bin_data_id
2. |A.x - B.x| <= 1.0           (수평 위치 동일)
3. |A.width - B.width| <= 1.0    (수평 폭 동일)
4. A 가 트리 순서상 먼저 + A.y < B.y  (A 가 위, z 작음)
5. A.y + A.height > B.y          (세로 겹침)
```

조건 2/3 의 strict 가드가 의도적 그림자/2중 노출 케이스 (대각선 오프셋,
다른 너비) 를 모두 우회 — 회귀 보호.

### 구현 — IR 단계 후처리, 중앙 1곳

`PageRenderTree::clip_overlapping_same_bin_images()` 메서드를 만들고,
`DocumentCore::build_page_tree()` 의 종단(`extra_mps` 머지 후, `Ok(tree)`
직전) 에서 1 회 호출. 모든 렌더러 (SVG/HTML/Canvas/WebCanvas/SvgLayer/
CanvasLayer/WebCanvasLayer) 가 이 빌드 결과를 소비하므로 호출 누락 위험 없음.
LayerBuilder (`src/paint/builder.rs:86`) 가 `node.bbox` 와 `image.clone()`
(crop 4 필드 포함) 을 그대로 PaintOp::Image 에 복사하므로 layer 기반 백엔드도
자동 영향.

## 4. Stage 별 산출물

| Stage | 핵심 산출물 | 커밋 |
|---|---|---|
| 1 — 진단 정밀화 + baseline | `mydocs/working/task_m100_1154_stage1.md`, 17 sample baseline SVG | `8a2ea7df` |
| 2 — Algorithm + 단위 테스트 11 개 | `src/renderer/render_tree.rs` (+438), `task_m100_1154_stage2.md` | `2e2ff0b3` |
| 3 — 렌더러 통합 + 시각 검증 | `src/document_core/queries/rendering.rs` (+4), `task_m100_1154_stage3.md` | `c533dca1` |
| 4 — 회귀 sweep + WASM + 최종 보고 | `task_m100_1154_report.md` | (본 커밋) |

## 5. 회귀 검증 결과 종합

### 5.1 단위 테스트 / 정적 검사

```
cargo test --release --lib
  test result: ok. 1318 passed; 0 failed; 6 ignored

cargo clippy --release --lib -- -D warnings
  Finished — no warnings

cargo fmt — applied to rendering.rs only
```

`cargo clippy --release --all-targets` 는 tests/ 영역에 56 errors 가 있으나
모두 pre-existing (Stage 3 이전 HEAD 에서도 동일). 본 task 와 무관.

### 5.2 Stage 1 식별 17 영향 sample sweep (same-commit 공정 비교)

`RHWP_TASK1154_NOCLIP=1` 임시 가드로 no-clip 산출 → guard 해제 후 with-clip
산출 → 페이지 단위 `diff -rq`:

| Sample | 페이지 수 | differ |
|---|---|---|
| 3-10월_교육_통합_2022.hwp | 16 | 0 |
| 3-10월_교육_통합_2022.hwpx | 16 | 0 |
| KTX.hwp | 27 | 0 |
| BlogForm_BookReview.hwp | 1 | 0 |
| BlogForm_MovieReview.hwp | 1 | 0 |
| BlogForm_Recipe.hwp | 1 | 0 |
| **exam_eng.hwp** | **8** | **1 (page 2)** |
| exam_social.hwp | 4 | 0 |
| hwpctl_ParameterSetID_Item_v1.2.hwp | 75 | 0 |
| hwpspec.hwp | 175 | 0 |
| pic-in-head-01.hwp | 22 | 0 |
| pic-in-table-01.hwp | 22 | 0 |
| pic2-2018.hwp | 3 | 0 |
| pic2.hwp | 3 | 0 |
| pic2.hwpx | 3 | 0 |
| test-image.hwp | 1 | 0 |
| test-image.hwpx | 1 | 0 |

→ **17 sample 합계 379 페이지 중 exam_eng page 2 한 장만 변경**. strict 5 조건이
정확히 대상 케이스만 매칭.

### 5.3 일반 회귀 sweep (페어 없는 문서)

| Sample | 페이지 수 | differ |
|---|---|---|
| exam_kor.hwp | 20 | 0 |
| exam_math.hwp | 20 | 0 |
| exam_math_8.hwp | 1 | 0 |
| biz_plan.hwp | 6 | 0 |
| 통합재정통계(2010.11월).hwp | 1 | 0 |
| 통합재정통계(2011.10월).hwp | 1 | 0 |
| 통합재정통계(2014.8월).hwp | 1 | 0 |

→ 페어가 없는 문서에서 100% 동일. 회귀 위험 0.

### 5.4 exam_eng page 2 — 정확한 변경 확인

LOWER (z=2, drawn first):
- before: `y=243.59, height=256.09, viewBox="0 0 2532 1612.77"`
- after:  `y=243.59, height=**219.59**, viewBox="0 0 2532 **1382.87**"`
- 검증: `B.y(=463.17) − A.y(=243.59) = 219.58` ✓
- crop bottom: `1612.77 × (219.59 / 256.09) ≈ 1382.94` (실제 1382.87, 반올림 오차)

UPPER (z=3, drawn after): 완전 불변.

→ 알고리즘이 정확히 LOWER 의 visible 영역만 잘라내고 UPPER 는 그대로 둔다.

### 5.5 WASM 빌드

```
docker compose --env-file .env.docker run --rm wasm
  Finished `release` profile [optimized] target(s) in 51.28s
  [INFO]: Optimizing wasm binaries with `wasm-opt`...
  [INFO]: :-) Done in 1m 31s
  [INFO]: :-) Your wasm pkg is ready to publish at /app/pkg.
```

## 6. 변경 코드 요약

```
src/renderer/render_tree.rs                 | +438  (Stage 2 commit 2e2ff0b3)
src/document_core/queries/rendering.rs      | +4    (Stage 3 commit c533dca1)
mydocs/plans/task_m100_1154.md              | (수행 계획서, Stage 0)
mydocs/plans/task_m100_1154_impl.md         | (구현 계획서, Stage 0)
mydocs/working/task_m100_1154_stage1.md     | (진단 + baseline)
mydocs/working/task_m100_1154_stage2.md     | (algorithm + 단위 테스트)
mydocs/working/task_m100_1154_stage3.md     | (렌더러 통합)
mydocs/report/task_m100_1154_report.md      | (본 보고서)
```

## 7. Out of Scope (의식적 미해결)

- 동일 bin_id 가 아닌 다른 이미지 페어 → 처리하지 않음 (현재 동작 유지)
- 옵션 1 (두 Pic 통합) / 옵션 2 (공통 세로 스케일 강제) — 한컴 알고리즘이
  비결정적이라 적용 시 회귀 위험 큼. 본 task 의 옵션 3 만 적용
- ImageFillMode != FitToSize 분기 영향은 없음 — bbox/crop 만 조정하므로 호환

## 8. 다음 단계

- `local/task1154` → `local/devel` merge (작업지시자 승인 후)
- orders 갱신 — orders 는 작업지시자 관할이므로 본 task 에서 직접 편집하지
  않음. merge 시 작업지시자가 갱신.
- 이슈 #1154 close — 작업지시자 승인 후 `closes #1154` 메시지로 처리

## 9. 작업 시간 산정

| Stage | 예상 | 실제 |
|---|---|---|
| 1 — 진단 + baseline | 1-2h | ~2h |
| 2 — Algorithm + 테스트 | 1-2h | ~2h |
| 3 — 통합 + 검증 | 1-2h | ~1.5h |
| 4 — 회귀 sweep + WASM + 보고 | 1-2h | ~1.5h |
| **합계** | **4-8h** | **~7h** |

승인 요청.
