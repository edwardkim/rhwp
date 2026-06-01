# PR #1164 검토 — #1154 잔상 통합 fix (clip + overlay crop + flow prefetch)

## 1. PR 정보

| 항목 | 내용 |
|------|------|
| 번호 | #1164 |
| 제목 | fix(render): #1154 잔상 통합 fix — clip + overlay crop + flow image prefetch |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) — 핵심 컨트리뷰터 |
| base ← head | `devel` ← `planet6897:pr/task-1154` (cross-repo) |
| 상태 | OPEN, MERGEABLE (BEHIND) |
| 변경 | +1663 / -10, 15 파일 (코드 3 + 문서 12) |
| 라벨 | enhancement / 마일스톤 v1.0.0 |
| 연결 | #1154 (잔상) |
| CI | 전부 pass (Build&Test, Analyze rust/js/py, Canvas visual diff, CodeQL) |

## 2. 컨트리뷰터 이력

@planet6897: #1148/#1095 merged, #1149/#1153 등 다수. OLE/Chart, HWP 5.0 spec, 다단, 그림 crop(#430/#434) 영역 핵심 기여자. `image_crop_scale_rule.md` 트러블슈팅도 이 컨트리뷰터 작업 이력.

## 3. 변경 내용 (3 독립 원인 통합)

| # | 원인 | 처리 |
|---|------|------|
| 1 | 동일 bin_id Pic 수직 인접 → 세로 스케일 미스매치 리샘플링 잔상 (SVG/Canvas/CanvasKit 공통) | `render_tree.rs` `clip_overlapping_same_bin_images()` + 11 단위 테스트, `rendering.rs:3101` build_page_tree 끝에 호출 |
| 2 | rhwp-studio overlay `<img>` 가 crop 무시 → bbox stretch | `rendering.rs` overlay JSON crop 출력 + `page-renderer.ts` wrapper div + overflow:hidden (75 HU/px 룰) |
| 3 | 큰 PNG/JPEG 비동기 디코드가 scheduleReRender 초과 → 첫 렌더 누락 | `page-renderer.ts` delays 확장 + `prefetchFlowImages` |

## 4. 코드 검토

### `clip_overlapping_same_bin_images` (render_tree.rs:1092)
- 적용 조건 strict 가드: 같은 bin_id + x/width 동일(1px tol) + A 가 위 + 세로 겹침. 조건 2/3 이 의도적 시각 효과(대각선 오프셋 등) 보호.
- LOWER 노드별 최소 height 채택(여러 UPPER 겹침 대응), crop 도 비율 조정.
- 11 단위 테스트 동반.
- ✅ 로직 타당, 가드 충실.

### overlay crop (page-renderer.ts)
- `<img>` 직접 source rect 불가 → wrapper div + overflow:hidden. crop_hu/75 → px source rect. `image_crop_scale_rule.md` 75 HU/px 룰 정합.

## 5. 충돌·회귀 점검 (devel 최신 = #1167 까지 머지된 상태)

- BEHIND 원인: devel 에 #1163/#1167 가 들어가며 head 가 뒤처짐.
- 코드 충돌: **없음** — `render_tree.rs` 를 PR(clip 함수 추가)과 devel(#1167 node_z_plane/text_wrap)이 **다른 영역** 수정 → auto-merge clean (충돌마커 0). orders 만 union.
- 의미 정합: #1167 plane 정렬 + #1164 clip 이 같은 ImageNode 를 다루나 독립 동작 — 빌드 + 전체 테스트로 확인.

## 6. 검증

| 항목 | 결과 |
|------|------|
| auto-merge 빌드 | ✅ Finished |
| `cargo test --tests` 전수 | ✅ 실패 없음 (clip 11 테스트 포함) |
| native-skia skia | (확인 중) |
| #1167(plane)/svg_snapshot 공존 | ✅ 회귀 없음 |

PR 자체 검증: 379 페이지 중 exam_eng page 2 한 장만 변경(나머지 100% 불변), 한컴 PDF 시각 일치.

## 7. 판단 (잠정)

3 원인 모두 근거 명확 + strict 가드 + 11 단위 테스트 + 한컴 PDF 시각 정합. CI pass. devel 충돌 없음. **merge 권장**.

- 처리: orders union 해결 후 devel 직접 머지 (fork push 권한 없음, BEHIND).
- 작업지시자 시각 판정 권장: rhwp-studio overlay crop + flow large PNG 는 web 경로라 자동 테스트 밖 — 시각 게이트 유효.

> 승인 시 검증 마무리(native-skia) + 시각 판정 → merge → `pr_1164_report.md`.

---

## 8. 처리 결과 (보고)

- **MERGED**: devel `969cd022` (이슈 #1154 close)
- orders union 해결, render_tree.rs auto-merge clean (#1167 plane 과 다른 영역)
- 검증: cargo test --tests 전수 + render_tree::tests 15 (clip 11) + native-skia 32 + fmt 통과
- PR 코멘트 등록 + 이슈 #1154 close
- 원인 #2(overlay crop)·#3(flow prefetch)는 rhwp-studio web 경로 — CI Canvas visual diff pass + PR 첨부 한컴 PDF 시각 일치로 검증
