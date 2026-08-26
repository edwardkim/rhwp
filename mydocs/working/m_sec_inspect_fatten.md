---
kind: working
status: active
issue: 5476
---

# M-sec inspect 3축 봉투·픽스처 고도화 (#5476)

작업 브랜치: `feat/m-sec-inspect-fatten`
대상: `tests/fixtures/inspect_msec/` · `tools/inspect_msec/` · `mydocs/working/inspect_msec/`

## 한 줄

기존 inspect 3축 계약을 봉투·예외·토큰 행렬로 두껍게 고정한다. 탐지 규칙은 그대로다.

## 이슈가 요구한 것 / 하지 말라는 것

- 요구: hidden-text / injection / unicode 계약 픽스처, 예외 봉투, 작업 문서
- 금지: 새 탐지 로직, DocumentCore 발명, visual_sweep/canvaskit/serializer/pdf/equation
- 금지 좌석: layout-anomaly, oracle, render_backend, proptest, fidelity_compare,
  hwp5-inventory, page-count, gym

## 만진 경로 / 만지지 않은 경로

- 만짐: `tools/inspect_msec/`, `tests/fixtures/inspect_msec/`, `mydocs/working/inspect_msec/`,
  `mydocs/working/m_sec_inspect_fatten.md`, `tests/cases/inspect_msec_fatten.rs`
- 안 만짐: `src/`, `gym/`, `scripts/visual_sweep.py`, 다른 MEGA 좌석

## 건수

- 성공 봉투 194
- 예외 봉투 22
- hidden-text 48
- injection 102
- unicode 44

## 시험

```bash
python tools/inspect_msec/gen_msec_fixtures.py
python tools/inspect_msec/test_msec_fixtures.py
cargo fmt --all -- --check
```

## PR 메모

closes #5476. `--body-file`. base `devel`. 한국어.
