# Stage 2 완료 보고서 — Task #1258 spike: 드롭 지점 특정 + 구현안 결정

- **이슈**: edwardkim/rhwp#1258
- **단계**: Stage 2 / 5 (spike)
- **코드 변경**: 없음 (정적 분석)

## 1. 결정적 발견 — 단일줄/다줄 비대칭의 정확한 코드 위치

`endnote_line_vpos_base` 는 **다줄 미주 문단에만** 설정된다 (`paragraph_layout.rs:1612` `end > start_line + 1`).
그 결과 미주 문단의 마지막 줄 y 전진 경로가 갈린다:

| prev 종류 | 경로 | 마지막 줄 trailing |
|-----------|------|-------------------|
| **단일줄** | `paragraph_layout.rs:4523` `y += line_height + line_spacing_px` | **포함** (baked between_notes=1984) ✓ |
| **다줄** | `paragraph_layout.rs:4496-4517` (vpos 구동) | 문제경계면 **0** (`:4509`, [Task #1236]) ✗ → gap≈0 |

→ **이 비대칭(4523 포함 vs 4509 제로)이 #1246(문22) gap≈0 의 근본**이며, S8 계열이 이를 하류 보정.

## 2. 더 깊은 발견 — layout↔pagination drift 가 이미 존재

- pagination 은 미주 문단을 **일반 문단으로** 처리 → `para_height = sum(lh+ls)` 에 baked trailing(1984) **포함**.
- render 는 다줄 미주 문제경계에서 trailing **0** (`:4509`).
- → **layout 이 pagination 보다 ~1984HU 짧게 누적**(다줄 미주-final 한정). height_cursor S8 + base-shift 가 이 drift 를 vpos→y 매핑에서 보정.
- `paragraph_layout.rs:4413` 주석("layout trailing 가산 → pagination 과 정합")의 철학과 **정확히 어긋나는 예외**가 미주 문제경계다.

## 3. S8 계열의 실제 역할 (양방향)

| 분기 | prev | 동작 | 이유 |
|------|------|------|------|
| #1246 (`:477-484`) | 다줄 | between_notes 만큼 **가산** | 4509 가 trailing 0 으로 누락 → 복원 |
| #1256/#1261 (`:448-466`) | 단일줄 | y_offset **유지** + base shift | 4523 이 이미 포함 → 이중가산 방지 |

→ 같은 between-notes 를 단일줄은 "넣지 마"·다줄은 "넣어"로 **정반대 보정**. 비대칭의 증거.

## 4. 구현안 결정 — **B 채택** (render 정규화: 4509 정합)

`paragraph_layout.rs:4504-4510` 의 다줄 미주 문제경계 trailing 을 **단일줄(4523)·pagination 과 동일하게**
last-seg trailing 포함으로 정합한다.

| 안 | 평가 |
|----|------|
| **B (채택)** | 4509 를 4523/pagination 에 맞춤. **render 를 pagination 에 정합시키는 방향**(drift 축소) → 가장 자연스럽고 vpos/pagination 절대값 불변 |
| A | typeset vpos 가 full gap encode → vpos 절대값 변동, 페이지 분할 위험. 기각 |
| C | 하이브리드, 두 곳 수정. 기각 |

### B 의 정직한 한계 (중요)

- 4509 에 trailing 을 넣으면 다줄 render y 가 단일줄과 같아져 **#1246 가산은 불필요**해진다.
- 그러나 그 순간 **inter-para vpos_adjust 가 다줄에서도 이중가산 위험**에 노출 → 단일줄용 #1256/#1261
  base-shift 가드를 **다줄로 확장(또는 두 가드 통합)** 해야 한다.
- 즉 결과는 "#1246 분기 순수 삭제"가 아니라 **#1246(가산) → #1256형(이중방지) 가드로 수렴/통합**.
  특례 수는 8→7(또는 통합으로 더 감소) 기대하되, **완전 제거는 아닐 수 있음**.
- 추가 검증 필수: 4509 변경이 다줄 미주-final 의 layout↔pagination drift 를 줄이는 게 맞는지
  (overflow·페이지 분할 무변동) — Stage 3 에서 baseline 대조.

## 5. Stage 3 계획 반영

- (1) `paragraph_layout.rs:4504-4510` 다줄 미주 문제경계 trailing 을 last-seg ls 포함으로 정합.
- (2) S8 은 아직 유지 → 미주 회귀 스위트 + 렌더 y baseline(§Stage1) 대조로 "무회귀 + #1246 가산이 no-op 화" 확인.
- (3) 이중가산이 관찰되면 #1256 가드 다줄 확장 설계.

## 승인 요청

구현안 B 승인 후 Stage 3(정규화 적용) 착수. **한계(§4)** 감안해 방향 재조정 원하시면 지시 바랍니다.
