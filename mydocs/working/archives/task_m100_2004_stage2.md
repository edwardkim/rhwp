# task_m100_2004 Stage 2 완료보고서 — 인라인(tac=true) 이미지 스택 페이지네이션

- 이슈 #2004, 브랜치 `fix/2004-image-stack-pagination`

## 근본원인 (계측 확정)

1430000 pi=2607 = tac(글자처럼) 그림 21장(각 227mm=860px 전면)만 있는 빈-텍스트 문단. 21장 전부 **동일 char_start=0**.

- **formatter 결함**: `tac_control_indices_for_line` 이 각 줄의 char-range `[start, next_start)=[0,0)` 로 tac 컨트롤을 찾는데, 모든 줄 char_start=0 이라 중간 줄들은 빈 범위 → 컨트롤 없음 → `empty_tac_guide_line`(0 높이)로 붕괴. 결과 `fmt_line_heights=[860, 0×19, 860]`(첫·마지막 줄만 높이). advances_sum≈1742px(≈2쪽) → 1회만 분할.
- **pagination 결함**: 줄별 fit 검사(`cumulative+content_h > avail`)가 overflow 를 잡아도, `hwp_authoritative`(다음 줄 vpos==0 이고 현재 줄 bottom=hwpunit(vpos+lh) 가 본문 안이면 현재 쪽 유지)가 발동. 21장 전부 vpos=0(각자 쪽 상단)이라 bottom=860≤877 → 모든 줄에서 hwp_authoritative=true → 분할 억제 → 한 쪽 붕괴.

## 수정 (2개, 좁은 게이트)

`src/renderer/typeset.rs`:
1. **formatter** — `stacked_tac_picture_heights`: tac-그림-only + 그림수==줄수 + 전부 동일 char_start + 모든 높이>8px 일 때 각 줄에 순서대로(index) 그림 높이 직접 부여. → `fmt_line_heights=[860×21]`.
2. **pagination** — `is_tac_picture_stack`(tac-그림-only + 줄수≥2 + 모든 line_height>본문×0.5) 일 때 `hwp_authoritative` 강제 off → 줄별 fit 분할(쪽당 1장) 복원.

두 게이트 모두 `para_is_treat_as_char_picture_only` 필수 → 일반 인라인 그림/텍스트 문단 불영향.

## 검증

**오라클 대조 (한글 2022)**:
| doc | base | 수정후 | 오라클 | 비고 |
|---|---:|---:|---:|---|
| 1430000 (인라인) | 384 | **403** | 404 | **−20→−1** (pi=2607 → 21 fragment, 쪽당 1장) |
| 1613000 (부동) | 171 | 171 | 268 | 불변 (부동 변종=Stage 3) |
| 1790387 (표) | 130 | 130 | 146 | 불변 |
| 1220000 (표) | 125 | 125 | 134 | 불변 (baseline 오기 126→125 정정: 이전 측정은 stale binary) |

**게이트 발동 확인**: 1430000 pi=2607 만 TACSTACK 발동, 1220000 은 발동 0 → 무회귀 계측 확인.
**clean-base 격리**: stash 후 origin/devel 측정 = 1220000 125·1430000 384 → 내 변경이 1430000 만 +19 함을 격리 확증.
**테스트**: `hwpx_roundtrip_baseline` 4/4, renderer lib 테스트 무회귀.

## 잔여
- 1430000 −1: 미세 off-by-one(trailing spacing 등), 범위 밖 허용.
- 부동 변종(−97/#1994)은 Stage 3.

## 교훈 (중요)
lib/bin 동명 충돌로 `target/release/rhwp.exe` 가 특정 시점(01:12) 이후 갱신 정지 → 빌드 성공해도 stale 바이너리 실행. **매 rebuild 전 `rm -f target/release/rhwp.exe` 필수**. 초기 baseline 126 도 이 함정 산물이었음.
