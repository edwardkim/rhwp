# 단계1 완료보고서: 재현 샘플·비교 파이프라인 고정

- **타스크**: [#146](https://github.com/edwardkim/rhwp/issues/146)
- **마일스톤**: M100
- **브랜치**: `local/task146`
- **단계**: 1 / 5 (샘플 편입 + 비교 파이프라인 고정)
- **작성일**: 2026-04-23
- **상위 문서**: `mydocs/plans/task_m100_146.md`, `mydocs/plans/task_m100_146_impl.md`

## 1. 작업 내용

### 1.1 샘플 편입

- `text-align.hwp` → `samples/text-align.hwp` 로 복사 편입 (32KB)
- `text-align.pdf` → `output/compare/text-align/text-align.pdf` 로 이동 (비교용, `.gitignore` 대상)
- 작업지시자가 제공한 루트의 `text-align.hwp` 원본 파일은 OS 핸들 잠금으로 자동 삭제 불가 → 샘플 편입본과 중복. 작업지시자 측에서 정리 요청 (또는 작업 종료 후 자연 해제되면 별도 커밋에서 제거).

### 1.2 비교 파이프라인 고정

150dpi 기준 좌우 비교 산출 명령:

```bash
# 1) rhwp → SVG
cargo run --bin rhwp -- export-svg samples/text-align.hwp -o output/svg/text-align/

# 2) PDF → PNG (150dpi, mutool 1.23.0)
mutool convert -O resolution=150 \
  -o output/compare/text-align/pdf-%d.png \
  output/compare/text-align/text-align.pdf

# 3) SVG → PNG (150dpi 대응, Chrome headless scale 1.5625x)
CHROME="/c/Program Files/Google/Chrome/Application/chrome.exe"
WINSVG=$(cygpath -w "$PWD/output/svg/text-align/text-align.svg")
WINOUT=$(cygpath -w "$PWD/output/compare/text-align/svg-chrome150.png")
"$CHROME" --headless --disable-gpu --no-sandbox \
  --force-device-scale-factor=1.5625 \
  --window-size=794,1123 --hide-scrollbars \
  --default-background-color=ffffffff \
  --screenshot="$WINOUT" "file:///${WINSVG//\\//}"
```

**도구 설치**: `winget install --id ArtifexSoftware.mutool` (mutool 1.23.0).

### 1.3 현상 베이스라인 고정

- 현재 SVG 산출물: `output/svg/text-align/text-align.svg` (294 줄)
- PDF 렌더: `output/compare/text-align/pdf-1.png`
- SVG 렌더(150dpi): `output/compare/text-align/svg-chrome150.png`
- 제목 첫 글자들 x 좌표 실측(후보 ① 증빙):
  - □ @ x=75.58 (= 20mm 여백)
  - 국 @ x=94.40 (Δ=18.82px, 폰트 21.33·자간 -8% 적용된 □ 폭만 반영)
  - 어 @ x=114.88 (Δ=20.48px, 국어 연속)
  - 변 @ x=145.17 (Δ=30.29px, 어/변 사이 공백 9.81px 가산 정상)
  - → **□ 직후 공백 9.81px 누락** 을 수치로 확인

### 1.4 코드 변경 없음

본 단계는 현상 재현 환경·명령 고정 단계. 소스 수정 없음.

## 2. 다음 단계

2단계 (② `Alignment::Justify` SVG 반영) 로 이행. 진입점:
- `src/renderer/layout/paragraph_layout.rs:902-1042` — x_start match 문에 Justify 분기 추가 + `text_style.extra_word_spacing` 소비 경로 보완
- 검증: `text-align.hwp` 본문 첫 문단 "…시범" 줄 오른쪽 끝이 body right margin 에 근접, svg_snapshot 대량 diff 영향 정량화

## 3. 커밋 계획

본 단계 산출물:
- `samples/text-align.hwp` (신규 샘플)
- `mydocs/plans/task_m100_146.md` (수행계획서)
- `mydocs/plans/task_m100_146_impl.md` (구현계획서)
- `mydocs/orders/20260423.md` (오늘할일)
- `mydocs/working/task_m100_146_stage1.md` (본 보고서)

단일 커밋으로 묶되 메시지는 `Task #146 단계1: 샘플 편입 + 비교 파이프라인 고정`.
