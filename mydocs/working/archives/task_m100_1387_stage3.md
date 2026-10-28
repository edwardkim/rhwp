# Task M100 #1387 — 3단계 완료 보고서 (게이트 동승)

- 브랜치: `local/task1387`
- 작성일: 2026-06-12
- 수정 파일: `src/serializer/hwpx/roundtrip.rs` (`TableCaption` 동승 + 테스트 4종)

## 1. 구현 내용

### 1.1 caption 구조 비교 (3.1)

- `IrDifference::TableCaption { section, paragraph, path, detail }` + Display
  (`section[i] paragraph[j]{path} caption: {detail}`).
- `diff_table_caption` 헬퍼: 존재 비대칭(missing/synthetic) + 속성 5종(side/fullSz/
  width/gap/lastWidth) + 문단 수 비교.
- `vert_align` 비교 제외 — HWPX `hp:caption`에 대응 속성이 없는 HWP5 유래 필드
  (1단계 측정 근거), HWP5 출발 플로우 위양성 방지. 사유 doc 주석 명기.

### 1.2 문단 재귀 동승 (3.2)

- Table arm 2곳에 caption 추가:
  - `diff_paragraph_char_shapes` (char_shapes + controls 통합 비교 함수): 구조 비교
    (`TableCaption`) + 내부 문단 재귀 — 경로 `…tbl.caption.p[k]`
  - `diff_paragraph_linesegs`: 내부 문단 lineseg 재귀 (구조 비교는 char_shapes 쪽
    한 곳에서만 수행 — 중복 보고 방지)
- 소비처 3곳(`roundtrip_ir_diff` / baseline 테스트 / 배치 IR_DIFF) 자동 동승
  (#1380/#1388 패턴).

## 2. 단위 테스트 (3.3)

| 테스트 | 검증 |
|--------|------|
| `task1387_caption_loss_in_gate` | 캡션 소실(Some→None) → `TableCaption` "missing" + path 고정 |
| `task1387_caption_attr_mismatch_in_gate` | side/gap 차이 → detail 문자열 고정 |
| `task1387_caption_paragraph_recursed_in_gate` | 캡션 내부 문단 char_shapes 차이 → `tbl.caption.p[0]` 경로 검출 |
| `task1387_ta_pic_001_r_roundtrip_caption_gate_zero` | 캡션 보유 실샘플 roundtrip → 게이트 0 |

`cargo test --lib serializer::hwpx::roundtrip` — **31 passed** (기존 27 + 신규 4).

## 3. baseline 전수 (게이트 동승 후)

`cargo test --test hwpx_roundtrip_baseline` — **4 passed, 0 failed** (36.2s).

- 신규 xfail **0** — 1단계 사전 판정(4절) 적중. 기존 xfail(#1382 1건, #1384 4건)·제외
  (hwpx-01) 변동 없음.
- ta-pic의 autoNum #1382 변위는 텍스트 축 증상이라 게이트 비교 항목(char_shapes/
  controls/linesegs/caption 구조) 밖 — 게이트 0과 모순 없음 (2단계 보고서 3.1 참조).
- `cargo fmt --check` 통과.

## 4. 다음 단계

4단계 — 전수 검증(배치/SVG 캡션 복원 + #1382 잔존 귀속 정량화) + 매뉴얼·최종 보고서
+ CI(release-test) + 그림/도형 캡션 별도 이슈 제안.

승인 요청드립니다.
