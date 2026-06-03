# 최종 결과보고서 — Task M100 #1043

## 이슈
edwardkim/rhwp#1043 — 중첩 표 외곽선 미표시 (1×1 wrapper 외곽 테두리 lookup off-by-one)

## 원인
`src/renderer/layout/table_layout.rs::layout_table` 의 1×1 wrapper 분기에서 외곽 테두리
borderFill 을 조회할 때 `cell.border_fill_id`(1-based `borderFillIDRef`)를 0-based `border_styles`
Vec 인덱스로 **그대로** 사용했다. 같은 파일의 다른 모든 lookup(일반 셀/표/zone)은 `.saturating_sub(1)`
로 변환하는데 이 분기만 누락되어, 한 칸 어긋난 borderFill(테두리 NONE)을 읽어 `any_border=false`
→ wrapper 외곽 실선 테두리가 통째로 누락되었다.

## 수정
1×1 wrapper 외곽 테두리 lookup 한 줄을 다른 경로와 동일하게 0-based 변환:
```rust
// before
styles.border_styles.get(cell.border_fill_id as usize)
// after
styles.border_styles.get((cell.border_fill_id as usize).saturating_sub(1))
```
좌표/렌더 로직 변경 없음. 본 수정으로 모든 borderFill lookup 이 `-1` 로 일관된다.

## 단계 요약
| 단계 | 내용 | 커밋 |
|------|------|------|
| Stage 1 | 코드 정정(1줄) + 근거 주석 | `ff252c71` |
| Stage 2 | 회귀 가드 추가. 초안의 비공개 픽스처 의존을 **tracked 샘플로 교체** | `da3fd68a` (41acfcdc amend) |
| Stage 3 | 전체 검증 + 시각 확인 + 최종 보고서 | (본 커밋) |

### Stage 2 픽스처 교체 경위
초안 회귀 테스트는 `samples/2. 인공지능…hwpx` 를 썼으나 이 파일은 **비공개 문서(git 미추적,
커밋 금지)** 라 테스트만 커밋되면 CI/타 환경에서 `cargo test` 가 깨진다. 버그/정정 두 빌드로 263개
tracked 비공개제외 샘플 전 페이지 SVG 를 렌더·diff 하여, 정정으로 외곽선이 달라지는 샘플 2건
(`exam_social.hwp`, `k-water-rfp.hwp`) 을 추출했다. 강신호인 **`samples/k-water-rfp.hwp`
p19(index 18)** 를 채택했다.

## 회귀 테스트 (`tests/issue_nested_table_border.rs`)
- 기존: `nested_table_border_exam_social_p1_q4_outline_present` (HWP5, 유지).
- 신규: **`nested_table_border_kwater_rfp_p19_outer_outline_present`** (HWP5).
  내부 표 외곽 격자는 점선(`stroke-dasharray`), wrapper 외곽 테두리는 같은 y 에 겹치는 실선이다.
  가드는 좌표 hardcode 없이 "전폭(>500px) 수평선 중 점선과 y 가 일치(±1px)하는 실선이 ≥1 존재"를
  확인한다 ("외곽 박스 = 내부 표 외곽" 관계). 무관한 다른 표 실선·페이지네이션 시프트에 무영향.
- 두 테스트 모두 **git-tracked 픽스처** 기반.

### 양방향 검증
| 코드 상태 | `kwater_rfp_p19` | `exam_social` |
|-----------|------------------|----------------|
| 정정 적용 | 통과 — 점선과 겹치는 전폭 실선 2건(상 y=583.3, 하 y=993.9, w=621) | 통과 |
| 버그 임시 복원 | 실패 — 겹치는 전폭 실선 0건 | 통과 |

## 검증 결과
- `cargo test` 전체 **0 failed** (모든 suite 통과).
- `cargo test --test issue_nested_table_border`: 2 passed.
- `cargo fmt --check tests/issue_nested_table_border.rs`: clean (수정 파일 한정).
- closes 번호 검증: `gh issue view 1043` → OPEN, 제목 정확히 일치 → `closes #1043` 안전.

### 시각 확인
- `export-svg samples/k-water-rfp.hwp -p 18` 렌더 결과 wrapper 외곽 박스 4변 실선
  (상 y=583.3 / 하 y=993.9 / 좌 x=80 / 우 x=701.2) 복원 확인. 래스터 PNG 에서 외곽 박스 가시.
- 권위 PDF(`pdf/k-water-rfp-2022.pdf`, 27쪽)는 rhwp 렌더(29쪽)와 페이지네이션이 어긋나(해당 구간
  PDF 는 예정공정표/기타사항/첨부 텍스트) 이 박스의 1:1 PDF 오버레이는 성립하지 않았다. 외곽선
  존재/누락 판정은 SVG 선 분석 + 양방향 회귀 테스트로 확정했다.

## 영향 범위 / 리스크
- 수정: `src/renderer/layout/table_layout.rs` 단일 lookup 1줄.
- `border_fill_id==0` 은 `saturating_sub(1)=0` 이나 본 분기는 `has_outer_padding && any_border`
  가드 안이라 무테두리 wrapper 에 영향 없음.
- 기존 exam_social 중첩 표: 정정 후에도 통과(무영향) 재확인.

## 후속
- orders 갱신은 작업지시자 관할(미편집).
- 커밋 후 `local/devel` merge 시점은 작업지시자 결정.
