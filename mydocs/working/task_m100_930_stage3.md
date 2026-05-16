# Task #930 Stage 3 완료보고서 — 글꼴 축소 로직 정정 구현

## 1. 수정 파일

`src/renderer/layout/shape_layout.rs` — `layout_textbox_content()` 글꼴 축소 로직

## 2. 변경 내용

### 2.1 발동 조건 (결함 A 해소)

```rust
// 변경 전
let max_ratio = sw_ratio.max(sh_ratio);
if max_ratio > 1.5 { ... }

// 변경 후
let max_ratio = sw_ratio.max(sh_ratio);
let min_ratio = sw_ratio.min(sh_ratio);
if min_ratio > 1.5 { ... }
```

두 축 모두 1.5배 초과(등방 확대)일 때만 글꼴 축소를 발동한다. `table-in-tbox.hwp` 2페이지처럼 한 축만 강하게 늘어난 이방 확대 글상자(sx≈1.07, sy≈8.2 → min_ratio=1.068)는 발동에서 제외된다.

### 2.2 축소 계수 (결함 B 해소)

```rust
// 변경 전
let inv = (1.0 / max_ratio).min(1.0);

// 변경 후
let inv = (2.0 / max_ratio).min(1.0);
```

Stage 2 에서 확정한 PDF 정합 계수. `shortcut.hwp` 자동번호: `inv = 2.0/2.6775 = 0.747`.

### 2.3 주석 갱신

`[Task #874 #3]` 주석을 `[Task #874 #3 / #930]`로 갱신하여 발동 조건·축소 계수 변경 사유와 두 케이스(이방 제외, 등방 2배 계수)를 명시했다. 경험적 보정임과 재검증 필요성을 기록했다.

### 2.4 임시 디버그 출력 제거

Stage 1·2 측정용 `[TASK930]` `eprintln!` 블록을 제거했다.

## 3. 검증 결과

### 3.1 table-in-tbox.hwp 2페이지 (결함 A)

글상자 본문 char `font-size` (SVG 출력): 변경 전 `2.44`/`2.28` → 변경 후 `22.67`/`20`/`18.67` 등 정상 본문 크기. 글상자 본문이 정상 렌더된다.

### 3.2 shortcut.hwp 1페이지 (결함 B)

자동번호 "1" 글리프 높이 (96dpi flood-fill 측정):

| | 높이 | 폭 |
|---|------|-----|
| 변경 전 | 93px | 45px |
| 변경 후 | **187px** | 90px |
| PDF (한글 2022) | **187px** | 95px |

높이 PDF 정확 일치. 폭 90 vs 95 는 글꼴 형상 차이(허용 범위).

### 3.3 회귀 테스트

`cargo test --release --lib`: **1258 passed; 0 failed** (회귀 0).

## 4. 잔존 사항

- `shortcut.hwp` 자동번호 "1" 세로 위치가 PDF 대비 약 17px 아래에 있다(우리 y557–743 vs PDF y540–726). 글꼴 *크기*는 정합하며, 위치 오프셋은 #930(글상자 matrix 글꼴 스케일) 범위 밖이다. Stage 4 에서 별도 후속 여부를 판단한다.
