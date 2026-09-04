---
kind: working
status: active
issue: 5481
---

# M-fill 누름틀 채움·반복 필드 픽스처 고도화 (#5481)

작업 브랜치: `feat/m-fill-fatten`
대상: `tools/form_fill/`

## 한 줄

devel 의 `fields` / `edit fill-fields` / `batch fill` 계약을 픽스처로 펼친다.
`이름[N]` · `--dry-run` · `--verify` · T07 첫 필드 홍길동 복제 금지(#4781).

## 이슈가 요구한 것

- 기존 CLI 만
- 이름[N] 반복 필드 · dry-run · verify 픽스처
- 첫 필드 홍길동 복제 금지 (#4781)
- 추가 10000–20000줄, 최소 10000
- `cargo fmt --all -- --check`, base `devel`, 한국어 PR

## 하지 말라는 것

- 채움 로직 발명 금지
- 다른 라이브 시트 금지
- gym 금지
- `git add -A` 금지
- 금지 워크트리(`rhwp`, `rhwp-desk*`, `rhwp-handoff`, `rhwp-scaffold-final`, `rhwp-doc-repro`) 사용 금지

## 만진 경로

- `tools/form_fill/` (계약 함수·카탈로그·생성기·픽스처·스키마·표·전사)
- `mydocs/working/m_fill_fixture_fatten.md`

## 만지지 않은 경로

- `src/` · `src/document_core/`
- `gym/`
- `.claude/skills/`
- `scripts/visual_sweep.py`
- 다른 MEGA 시트(`tools/fidelity_compare`, `tools/oracle_public`, inspect)

## 시험

```bash
python tools/form_fill/fatten_form_fill.py
# 61 forms / 30 fill / 17 occurrence / 25 dry-run / 25 verify / 13 batch / 37 honggildong / 23 paths
python tools/form_fill/test_form_fill.py          # 31 passed
python tools/form_fill/test_fatten_form_fill.py   # 14 passed
cargo fmt --all -- --check                        # 통과
node scripts/rust-test-suite-manifest.mjs --check
node scripts/rust-unit-test-tiers.mjs --check
```

## PR 메모

`closes #5481`, `--body-file`, base `devel`, head `kevin9327:feat/m-fill-fatten`.
