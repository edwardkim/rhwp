# Task #1560: 한글 페이지 충실도 오라클 — 구현 계획서

> 수행계획서 `task_m100_1560.md`(승인). 3단계. 소스 변경 없음(도구/문서).
> 컨벤션: `tools/verify_hwpx.py`(argparse + pyhwpx + 종료코드).

## Stage 1 — 핵심 도구
- `tools/verify_hangul_pages.py`:
  - 입력 `--batch <orig_dir> <rt_dir>` (재귀, stem/상대경로 매칭, rt 접미사 `.rt.hwpx`/`.rt.hwp`).
  - 각 쌍 한글 `PageCount` 비교 → `OK`/`COLLAPSE`/`EXPAND`/`MISSING`/`ERR`.
  - TSV(`verdict/orig_pg/rt_pg/rel`) + 요약(붕괴율) + **COLLAPSE>0 시 exit 1**.
  - pyhwpx 미설치/비-Windows 명확 안내 후 종료(크래시 금지).
- 검증: fidelity3 rt 로 36382669 OK(8→8), 36387103 COLLAPSE(2→1) 검출.
- 산출: `task_m100_1560_stage1.md` + 커밋.

## Stage 2 — 견고성·재현성·입력 확장
- `--inventory <tsv>`(hwpx/hwp5-roundtrip 산출) + `--rt-root` 모드, `--status` 필터.
- `--sample N [--seed S]`, `--pdf`(PDF 페이지수 교차검증, 선택).
- 파일별 try/except 격리(한글 행/오류 스킵), `clear(option=1)` 정리.
- 재현성: 출력 헤더에 git HEAD + (지정 시) 바이너리 빌드시각 기록.
- 산출: `task_m100_1560_stage2.md` + 커밋.

## Stage 3 — 매뉴얼 + 검증 + 임시 스크립트 대체 + 최종 보고
- `mydocs/manual/hangul_page_oracle.md`(사용법·등급·한계).
- 알려진 케이스 재현: v3 표본(~2% 붕괴, 36387103) 동일 재현 + 종료코드 확인.
- `output/poc/fidelity*/t3_*.py` 대체 안내(도구로 일원화).
- `mydocs/report/task_m100_1560_report.md` + 커밋.

## 주의
- 한글 COM 불안정 대응(예외 격리, 필요시 재기동). 다이얼로그 차단 전제(FilePathCheckerModule).
- 대량 시 `--sample` 권장. rhwp 소스 무변경.
