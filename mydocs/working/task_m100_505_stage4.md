# Task M100 #505 Stage 4 완료 보고서 (시각 검증)

## 작업 내용

미적분 기출문제_03.미분계수와 도함수1-1.hwp p5/6/7 을 정정 후 재출력하여 시각 비교.

```bash
./target/release/rhwp.exe export-svg "...미적분03.hwp" -p 4 -o output/diag_505_after/ --debug-overlay
./target/release/rhwp.exe export-svg "...미적분03.hwp" -p 5 -o output/diag_505_after/ --debug-overlay
./target/release/rhwp.exe export-svg "...미적분03.hwp" -p 6 -o output/diag_505_after/ --debug-overlay
```

Chrome headless 로 SVG → PNG 변환:

```bash
chrome --headless --disable-gpu --screenshot=... --window-size=900,1300 file:///<svg>
```

## 시각 검증 결과 (페이지 5 — pi=165 CASES 결함 위치)

### BEFORE (정정 전)

`(i), (ii)에서  g(x) = | 12r²  (0≤x≤2) | x  (x<0 또는 x>2)` —
**`{1}/{2} x²` 가 `12r²` 로 squashed 표시.** 분수가 분수가 아니라 두 숫자 `1`, `2` 가 가로로 붙고 `x²` 의 `x` 가 'r' 글리프처럼 왜곡.

### AFTER (정정 후)

`(i), (ii)에서  g(x) = | (1/2) x²  (0≤x≤2) | x  (x<0 또는 x>2)` —
**분수 `(1/2)` 정상 분자/분모/분수막대 표시**, `x²` 위첨자 정상, 두 행 간 간격 적절.

스크린샷:
- 정정 전: `output/diag_175_visual/png/미적분 기출문제_03.미분계수와 도함수1-1_005.png`
- 정정 후: `output/diag_505_after/png/미적분 기출문제_03.미분계수와 도함수1-1_005.png`

## SVG transform 정량 비교 (페이지 5 의 35개 수식)

| 메트릭 | BEFORE | AFTER | 평가 |
|---------|--------|-------|------|
| scale_y max | 1.6370 | **1.1818** | ✓ 1.20 이하 |
| scale_y mean | 1.0417 | 1.0266 | 정상 |
| scale_y min | 0.9613 | 0.9613 | 변동 없음 |
| scale_x max | 1.4232 | 1.4232 | 변동 없음 |
| scale_x min | 0.7191 | 0.7832 | 개선 (\n 폐기 효과) |
| 극단 그룹 (\|scale_y - 1\| > 0.30) | **1건** | **0건** ★ |

## 마무리

- 임시 진단 모듈 `src/renderer/equation/layout_probe_505.rs` 의 모듈 등록 (`mod.rs`) 제거. 파일 자체는 untracked 잔존 (작업지시자가 정리).
- 신규 영구 회귀 테스트 `tests/issue_505.rs` 4건이 본 진단 영역을 영구 보호.

## 검증 종합

- ✓ pi=165 scale_y 1.64 → 1.08 (수락 기준 ≤ 1.20)
- ✓ 시각 squashing 해소
- ✓ PR #396 회귀 0건
- ✓ issue_418/501 회귀 0건
- ✓ cargo test --lib 1104 통과 (probe 2건 삭제 후 1102 통과 — 정상)
- ✓ clippy 0건 (본 변경 영역)
- ✗ svg_snapshot 5/6 실패 — **본 정정과 무관한 사전 CRLF/LF 회귀** (main 브랜치 동일)
