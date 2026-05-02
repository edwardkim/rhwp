# Task #520 Stage 3 — 검증 + 회귀 확인

## 1. 빌드/테스트/clippy

- `cargo build --release` ✅ (warnings 0)
- `cargo test --release` ✅ (1111 passed, 0 failed, 1 ignored)
- `cargo clippy --release -- -D warnings` ✅

## 2. 본 케이스 검증 — exam_science.hwp 페이지 3

수정 전: ㉠ 사각형이 y=249.68 px 에 그려져 [탐구 과정 및 결과] 라인을 침범.

수정 후 SVG (`/tmp/svg_t520_fix/exam_science_003.svg`) 시각 확인:

```
[가설]
◦ 분자당 구성 원자 수가 3인 분자의 분자 모양은 모두
[㉠]    이다.                          ← ㉠ 박스가 같은 줄에 정상 배치
[탐구 과정 및 결과]                    ← 깨끗 (겹침 없음)
(가) 분자당 구성 원자 수가 3인 분자를 찾고, 각 분자의 분자
    모양을 조사하였다.
...
```

PDF (`samples/exam_science.pdf` 페이지 2 좌측 7번 박스) 와 시각적으로 일치.

## 3. 회귀 샘플

| 샘플 | 결과 |
|------|------|
| `samples/tac-img-02.hwpx` (TAC 이미지) | 73 페이지 SVG 정상 출력, 충돌 없음 |
| `samples/table-vpos-01.hwpx` (표 + vpos) | 5 페이지 정상 출력 |
| `samples/exam_science.hwp` 페이지 4, 5 (인접 페이지) | 시각 검사: 박스 안 다단 콘텐츠 정상, 도형 위치 어긋남 없음 |

## 4. 영향 범위

수정 위치 2 곳 (`src/renderer/layout/table_layout.rs`):
- 1547-1549 (Picture 인라인 분기): `tac_img_y` 산출 시 `seg.vpos` → `seg.vpos - first_seg.vpos`
- 1631-1633 (Shape 인라인 분기): 동일

영향 받는 케이스: **셀 내부의 두 번째+ paragraph 에 line_segs.len() ≥ 2 인 인라인 TAC shape/picture 가 있는 경우** (즉 ls[0].vpos > 0). ls[0].vpos = 0 인 경우 (셀 첫 paragraph 또는 단일 줄) 두 공식 결과가 동일하므로 회귀 없음.

## 5. 결론

Stage 2 의 수정은 정확하고 영향 범위가 좁다. 모든 테스트/clippy 통과, 본 케이스 PDF 일치, 회귀 샘플 정상.
