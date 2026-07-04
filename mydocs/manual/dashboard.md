# 코드 품질 대시보드 매뉴얼

## 개요

`scripts/metrics.sh`로 코드 품질 메트릭을 수집하고, `scripts/dashboard.html`로 시각화하는 대시보드 시스템이다.

## 구성 파일

| 파일 | 역할 |
|------|------|
| `scripts/metrics.sh` | 메트릭 수집 스크립트 (Bash) |
| `scripts/dashboard.html` | Chart.js 기반 대시보드 (HTML) |
| `output/metrics.json` | 수집된 메트릭 데이터 (자동 생성, **비커밋** — `output/` 은 gitignore) |
| `output/dashboard.html` | 대시보드 복사본 (자동 생성, 비커밋) |
| `output/metrics_history/` | 실행별 이력 (최근 30개 롤링, 비커밋) — `metrics_history.json` 요약으로 델타 카드/추세 렌더 |
| `mydocs/metrics/{날짜}/` | **커밋되는 스냅샷 보관소** — 의미 있는 시점의 측정 공유·추적 ([README](../metrics/README.md)) |

## 스냅샷 보관 (커밋해 공유)

`output/` 산출물은 gitignore 라 보존되지 않는다. **의미 있는 시점**(리팩토링 Phase 경계,
릴리즈, 코드 리뷰)의 측정은 스냅샷으로 커밋해 컬래버레이터와 공유하고 추세를 추적한다:

```bash
./scripts/metrics.sh --snapshot   # 수집 + mydocs/metrics/{오늘날짜}/ 자동 보관
```

- 매 실행을 커밋하지 않는다 — 시점 선정 기준·목록은 `mydocs/metrics/README.md`.
- 스냅샷 폴더는 `metrics.json` + `dashboard.html` 세트라 그 자리에서 바로 열람 가능.

## 실행 방법

### Docker 환경 (권장)

```bash
docker compose --env-file .env.docker run --rm dev bash /app/scripts/metrics.sh
```

### 로컬 환경

```bash
./scripts/metrics.sh
```

### 대시보드 열기

실행 완료 후 브라우저에서 `output/dashboard.html`을 연다.

```bash
# macOS
open output/dashboard.html

# Linux
xdg-open output/dashboard.html

# WSL
explorer.exe output/dashboard.html
```

> `dashboard.html`은 같은 디렉토리의 `metrics.json`을 `fetch()`로 읽으므로,
> 파일 프로토콜(`file://`)에서는 CORS 제한으로 로딩이 안 될 수 있다.
> 이 경우 로컬 웹서버를 사용한다:
> ```bash
> cd output && python3 -m http.server 8080
> # 브라우저에서 http://localhost:8080/dashboard.html
> ```

## 수집 항목 (5단계)

### 1단계: 파일별 줄 수

- 대상: `src/**/*.rs`, `rhwp-studio/src/**/*.{ts,css}`
- `font_metrics_data.rs`는 대시보드에서 자동 필터링됨 (자동생성 파일)

### 2단계: Clippy 경고

- `cargo clippy` 실행 후 `warning:` 줄 수를 카운트
- 목표: 0개

### 3단계: Cognitive Complexity

- Clippy의 `cognitive_complexity` 린트를 사용
- `clippy.toml`에 임시로 `cognitive-complexity-threshold = 5`를 설정하여 CC >= 5인 함수를 모두 수집
- 수집 후 `clippy.toml`은 원래 상태로 복원됨

### 4단계: 테스트

- `cargo test` 실행 후 passed/failed/ignored 수를 파싱

### 5단계: 커버리지

- `cargo-tarpaulin`이 설치된 경우에만 수집
- 미설치 시 `null`로 표시

## 대시보드 구성

### 상단 카드 (4개)

| 카드 | 기준 | 색상 |
|------|------|------|
| 1,200줄 초과 파일 | 0개 = 초록, 1개 이상 = 빨강 | `font_metrics_data` 제외 |
| Clippy 경고 | 0개 = 초록, 1~49 = 노랑, 50+ = 빨강 | |
| CC > 25 함수 | 0개 = 초록, 1개 이상 = 빨강 | 최대값과 수집 함수 수 표시 |
| 테스트 | 실패 0 = 초록, 1+ = 빨강 | passed/total 표시 |

### 차트 (4개)

#### 파일 크기 분포 (상위 30)

- 가로 막대 차트
- 빨간 파선: 1,200줄 상한선
- 초과 파일은 빨간색 바로 강조

#### Cognitive Complexity Top 22

- 가로 막대 차트
- 노란 파선: 목표 상한(15)
- 빨간 파선: 경고 임계값(25)
- 색상: 파란(≤15), 노란(16~25), 빨간(>25), 진빨간(>100)

#### 테스트 현황

- 도넛 차트
- 통과(초록), 실패(빨강), 무시(회색)

#### 파일 크기 분포 (구간별)

- 세로 막대 히스토그램
- 구간: 0-200, 201-500, 501-800, 801-1200, 1201-2000, 2001-5000, 5001+
- 1,200줄 초과 구간은 빨간색

## metrics.json 스키마

```json
{
  "timestamp": "2026-02-23T...",
  "file_lines": [
    { "file": "src/wasm_api.rs", "lines": 1770 }
  ],
  "clippy": {
    "warnings": 0,
    "autofix": 0
  },
  "cognitive_complexity": [
    { "file": "src/renderer/layout/table_partial.rs", "line": 25, "complexity": 85 }
  ],
  "tests": {
    "passed": 608,
    "failed": 0,
    "ignored": 0
  },
  "coverage": null,
  "thresholds": {
    "max_lines": 1200,
    "max_cognitive_complexity": 15,
    "warn_cognitive_complexity": 25,
    "target_clippy_warnings": 0,
    "target_coverage": 70
  }
}
```

## 임계값 변경

`metrics.sh` 하단의 `thresholds` 블록에서 변경한다:

```bash
"thresholds": {
    "max_lines": 1200,              # 파일 줄 수 상한
    "max_cognitive_complexity": 15,  # CC 목표 (노란 파선)
    "warn_cognitive_complexity": 25, # CC 경고 (빨간 파선)
    "target_clippy_warnings": 0,    # Clippy 목표
    "target_coverage": 70           # 커버리지 목표 (%)
}
```

## 주의사항

- `metrics.sh` 실행 시 `clippy.toml`을 임시 변경 후 복원한다. 기존 `clippy.toml`이 있으면 내용을 백업/복원한다.
- `output/` 디렉토리는 `.gitignore`에 등록되어 있으므로 `metrics.json`과 `dashboard.html`은 Git에 포함되지 않는다.
- `scripts/dashboard.html`이 원본이고, `output/dashboard.html`은 `metrics.sh` 실행 시 자동 복사된다.
