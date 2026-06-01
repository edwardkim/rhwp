# Task #1154 구현 계획서 — 중복 Pic 컨트롤 스케일 미스매치 잔상 제거

## 0. 수행계획서 / 이슈

- 수행 계획서: [`task_m100_1154.md`](./task_m100_1154.md)
- GitHub Issue: [#1154](https://github.com/edwardkim/rhwp/issues/1154)
- 브랜치: `local/task1154` (분기 베이스 `local/devel`)

## 1. 방향 결정 (수행계획서 Stage 2 의 사전 결론)

수행계획서의 세 후보 중 **옵션 3 — 겹침 제거 (Clip below pic to top of above pic)** 채택.

### 근거

| 옵션 | 구현 난이도 | 회귀 위험 | 한컴 정합도 |
|---|---|---|---|
| 1. 두 Pic 통합 | 중 (페어 매칭 + crop union + 스케일 결정 로직) | 중 (스케일 결정 정책 다양) | 비결정적 (한컴 알고리즘 미지) |
| 2. 공통 세로 스케일 강제 | 중 (그룹 검출 + 스케일 통일) | 상 (다른 의도 케이스 깨질 수 있음) | 비결정적 |
| **3. 겹침 제거 (Clip below)** | **하 (단순 bbox/crop 조정)** | **하 (단일 Pic 케이스 불변)** | **고 (z-on-top 만 보임 = 작가 의도와 일치)** |

옵션 3 은 z-order 가 높은 Pic 이 아래 Pic 의 겹치는 영역을 100% 가린다는 작가 의도를 그대로 구현. 단일 Pic 분기와 동일 bin_id 가 아닌 케이스에는 영향 없음.

### 정확한 동작

기준 페어 (A, B): `bin_data_id` 가 같고, A.y < B.y 이며 (A.y + A.height) > B.y (세로 겹침).
- A 가 아래 깔리는(z 가 작은) 그림: 위쪽 부분만 보임
- B 가 위에 덮는(z 가 큰) 그림: 그대로 유지
- **수정**: A 의 bbox.height 를 (B.y − A.y) 로 줄이고, A 의 crop.bottom 을 동일 비율로 축소

같은 이미지를 z 가 큰 B 가 위에서 100% 가리므로 표시 결과는 한컴과 동일하면서 스케일 미스매치 잔상이 생기지 않는다.

## 2. 영향 코드

핵심:
- `src/renderer/svg.rs:1235-1382` — `render_image_node` (FitToSize/crop 분기)
- `src/renderer/render_tree.rs:711-803` — `ImageNode` 정의

선택지:
- (A) **SVG 렌더 직전 pre-pass** — `render_tree` 노드 walk 전에 페이지 단위 ImageNode 수집 → 겹침 페어 검출 → bbox/crop 조정. 변경 범위 최소.
- (B) 레이아웃 단계 (`picture_footnote.rs`) — 페어를 검출하여 ImageNode 생성 시점에 bbox/crop 미리 조정. 다른 렌더러(web_canvas, skia)도 자동 혜택.

**(B) 채택**: 모든 렌더러(svg/web_canvas/skia) 가 동일 ImageNode 를 그리므로 한 곳에서 조정하면 일관됨. 또한 픽셀-단계가 아닌 IR-단계 조정이라 자연스러움.

다만 `picture_footnote.rs` 의 `layout_body_picture` 가 그림 하나씩 처리하므로, 페어 검출은 그림들이 모두 페이지 트리에 들어간 후의 후처리(pre-render pass) 가 더 자연스러움.

→ 최종: **SVG/web_canvas/skia 렌더링 직전 공통 후처리** — `PageRenderTree` 트리를 walk 하여 ImageNode 들을 수집 / 겹침 검출 / bbox·crop 조정.

수정 위치 후보:
- `src/renderer/render_tree.rs` — `PageRenderTree::resolve_image_overlap()` 신규 메서드 (트리 자체를 in-place 수정)
- 각 렌더러(svg/canvas/web_canvas) 가 페이지 렌더 시작 전에 1 번 호출

## 3. Stage 분할 (4 단계)

### Stage 1 — 진단 정밀화 + baseline 기록

목표: 패턴 파악 + 회귀 안전성 확인.

작업:
1. `samples/` 와 `samples/hwpx/` 전 샘플에 대해 "동일 `bin_data_id` Pic 컨트롤이 동일 paragraph 또는 인접 paragraph 에 2개 이상" 패턴 검출 스크립트 작성 (e.g., `rhwp dump | grep` 또는 임시 Rust 바이너리).
2. 검출된 케이스의 SVG 출력 baseline 생성:
   - `output/svg/task1154_baseline/<sample>/` 디렉토리
   - 영향 페이지만 SVG 추출
3. 검출 결과를 `mydocs/working/task_m100_1154_stage1.md` 에 정리:
   - 영향 sample 목록 + 케이스 수
   - exam_eng.hwp 의 Pic[0]/Pic[1] geometry 정밀 기록
4. 권위 PDF 비교 — exam_eng-2022.pdf 페이지 2 의 박스 외관과 우리 출력의 잔상 차이를 PNG 비교 산출물로 저장.

산출물: `mydocs/working/task_m100_1154_stage1.md` + baseline SVG 디렉토리.

### Stage 2 — Fix 함수 설계 + 단위 테스트 우선 작성

목표: 구현 전에 함수 시그니처 / 단위 테스트 / 알고리즘 동결.

작업:
1. **함수 시그니처 결정**:
   ```rust
   // src/renderer/render_tree.rs 에 PageRenderTree 메서드
   impl PageRenderTree {
       /// 동일 bin_data_id 를 가진 ImageNode 가 세로로 겹칠 때,
       /// 아래에 깔리는 (트리 순서상 먼저 그려지는) ImageNode 의 bbox/crop 을
       /// 위에 덮는 ImageNode 의 top 까지 축소한다.
       pub fn clip_overlapping_same_bin_images(&mut self);
   }
   ```
2. **알고리즘**:
   - DFS 로 모든 `ImageNode` 와 그 bbox 를 수집 (트리 순서 보존)
   - 트리 순서 = SVG paint 순서 = z-order
   - 정렬: bin_data_id 별로 그룹화
   - 각 그룹 내에서 트리 순서가 빠른 = z 가 작음 = 아래에 깔림
   - 페어 검출: 두 이미지의 bbox 가 가로 겹침(x 범위 교집합) + 세로 겹침(y 범위 교집합) 이면 페어로 판정
   - 페어 (A=아래, B=위) 에 대해 A 의 bbox.height 와 crop 을 다음과 같이 조정:
     - `new_height = B.y - A.y`
     - `ratio = new_height / A.height`
     - `crop.bottom_new = crop.top + (crop.bottom - crop.top) * ratio`
     - A.height = new_height
     - A.crop = (crop.left, crop.top, crop.right, crop.bottom_new)
3. **단위 테스트** (`src/renderer/render_tree.rs` 의 tests 모듈에 추가):
   - test_overlap_same_bin_clips_lower
   - test_overlap_different_bin_no_clip
   - test_non_overlap_no_clip
   - test_horizontal_no_overlap_no_clip
   - test_multiple_pairs_all_clipped
4. **알고리즘 회피 조건**:
   - bin_data_id 가 다르면 무시
   - crop 이 None 인 경우: crop 새로 생성 또는 무시 (검토 필요)
   - 단일 이미지 케이스 (페어 없음): 무시 — 불변

산출물: `mydocs/working/task_m100_1154_stage2.md` + 단위 테스트만 추가된 commit.

### Stage 3 — 구현 + exam_eng.hwp 시각 검증

목표: 알고리즘 구현 후 단위 테스트 / exam_eng.hwp / 권위 PDF 정합 확인.

작업:
1. `PageRenderTree::clip_overlapping_same_bin_images` 구현.
2. 호출 지점 (각 렌더러 페이지 렌더링 진입부):
   - `src/renderer/svg.rs` — `render_page` 또는 SVG 시작점
   - `src/renderer/web_canvas.rs` — Canvas 렌더 진입부
   - `src/renderer/skia/` — Skia 렌더 진입부
   - **(설계 결정 필요)** 트리 후처리는 PageRenderTree 빌드 직후 1 번만 — `PageRenderTree::build_complete` 같은 finalize 시점.
3. Stage 2 단위 테스트 통과 확인.
4. exam_eng.hwp 페이지 2 SVG 재생성 → 박스 하단 단일 라인 확인 (Stage 1 의 baseline PNG 와 시각 비교).
5. 작업지시자 시각 판정 요청용 비교 자료 준비.

산출물: 코드 변경 + `mydocs/working/task_m100_1154_stage3.md` + SVG 비교 산출물.

### Stage 4 — 회귀 검증 + 최종 보고

목표: 다른 샘플 영향 없음 / 전체 테스트 통과 / WASM 정상.

작업:
1. `cargo test --release --lib` 전체 (baseline 대비 변동 확인).
2. `cargo clippy --release -- -D warnings`.
3. `cargo fmt` (변경 파일만).
4. Stage 1 에서 식별된 모든 영향 sample SVG 재생성 → baseline 과 시각 비교.
5. 일반 회귀: exam_kor, exam_math, biz_plan, 통합재정통계, 시험지 일반, HWPX 변환본 sweep.
6. Docker WASM 빌드 (`docker compose --env-file .env.docker run --rm wasm`).
7. 최종 보고서 `mydocs/report/task_m100_1154_report.md`.
8. `mydocs/orders/` 의 오늘 할일 갱신 (해당하는 경우).

산출물: `mydocs/report/task_m100_1154_report.md` + 회귀 검증 결과.

## 4. 위험 / 완화

| 위험 | 완화 |
|---|---|
| 동일 bin_id 페어 사용 다른 의도 케이스 (의도적 부분 가리기) | Stage 1 에서 영향 sample 전수 조사 → Stage 4 시각 회귀로 검증 |
| crop 이 없는 그림의 페어 | 단위 테스트로 동작 정의 — crop 없으면 bbox 만 축소하고 crop 은 그대로 |
| 음수 폭 / height 0 edge case | 페어 조건 검사 시 `B.y > A.y` 와 `A.height > 0` 보장 |
| 페어 3 개 이상 (A < B < C) | A 는 B 의 top 까지, B 는 C 의 top 까지 (각 페어 독립적) |

## 5. Out of Scope

- 동일 bin_id 가 아닌 (다른 이미지) 그림 겹침은 처리하지 않음 (현재 동작 유지)
- ImageFillMode == TileAll/TileHorz/TileVert/배치 모드 분기는 변경 없음 (Issue 의 FitToSize/crop 케이스에 한정)
- 한컴 정합도 100% 달성 (스케일 통일 옵션 1/2) 은 후속 task 로 분리

## 6. 예상 작업량

각 Stage 약 1-2 시간, 총 6-8 시간 (1 일 이내).

진행 보고는 각 Stage 완료 시 작업지시자 승인 후 다음 Stage.
