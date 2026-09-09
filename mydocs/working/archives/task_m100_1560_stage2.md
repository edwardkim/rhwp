# Task #1560 Stage 2 완료보고서 — 입력 확장 + 재현성

## 변경 (`tools/verify_hangul_pages.py`)
- `--inventory <tsv> --orig-root <dir> --rt-root <dir>`: roundtrip `inventory.tsv`
  의 `sample`/`status` 컬럼 기준으로 쌍 수집. `--status IR_DIFF,PASS` 필터.
- `--sample N [--seed S]`: 재현 가능한 무작위 표본(시드 고정).
- `--pdf`: 한글 PDF 내보내기 후 PyMuPDF 페이지수 교차검증(렌더 기준 강화).
- 재현성: 콘솔/TSV 헤더에 **git HEAD** 기록(stale-binary 함정 방지, v1 F2' 재발 봉인).
- `--batch`(Stage 1)와 상호배타 그룹.

## 검증
```
python tools/verify_hangul_pages.py --inventory output/poc/fidelity3/hwpx/inventory.tsv \
  --orig-root <hwpdocs> --rt-root output/poc/fidelity3/hwpx \
  --status IR_DIFF,PASS --sample 45 --seed 42 -o ...

# git HEAD=b086bd5a | 대상 45건
=== 45건 / OK=44 COLLAPSE=1 (붕괴율 2%) ===  종료 코드 1
```
v3 임시 스크립트(`t3_v3.py`) 결과(1/45=2%)를 **정확히 재현**. 인벤토리·표본·종료코드·HEAD 기록 동작 확인.

## 다음
Stage 3 — 매뉴얼 + 임시 스크립트 대체 안내 + 최종 보고.
