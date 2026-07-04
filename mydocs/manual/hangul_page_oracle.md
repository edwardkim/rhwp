# 한글 페이지 충실도 오라클 가이드 (Task #1560)

`tools/verify_hangul_pages.py` — 원본 ↔ 저장본(rt)을 한컴오피스로 열어 **PageCount** 를
비교하여, IR 게이트가 못 잡는 **한글 전용 페이지 붕괴**를 검출하는 정식 도구.

## 1. 왜 필요한가
무손실 검증(`hwpx-roundtrip`/`hwp5-roundtrip`)의 IR diff 와 rhwp 자체 페이지수로는
**한글에서만 나타나는 페이지 붕괴**가 검출되지 않는다(예: #1557 secCnt — IR diff=0 PASS
인데 한글 8→1). CLAUDE.md 권위 등급상 Windows+한컴에디터가 1차 정답지이므로, 이 오라클을
재현 가능·게이트 가능한 도구로 정식화했다. (임시 스크립트 `output/poc/fidelity*/t3_*.py` 대체)

## 2. 요구사항
- Windows + 한컴오피스 2010+, `pip install pyhwpx`.
- `--pdf` 옵션 사용 시 `pip install pymupdf`.
- 다이얼로그 차단 전제(FilePathCheckerModule 등록 — `hwp_com_automation` 메모리 룰).

## 3. 사용법

### 배치 모드 (원본 폴더 ↔ rt 폴더)
```bash
python tools/verify_hangul_pages.py --batch <원본_폴더> <rt_폴더> -o out.tsv
```
원본을 재귀 스캔하여 상대경로로 rt(`{stem}.rt.hwpx`/`.rt.hwp`)를 매칭.

### 인벤토리 모드 (roundtrip 산출 TSV 기준)
```bash
rhwp hwpx-roundtrip --batch <원본_폴더> -o out/rt        # 1) rt + inventory.tsv 생성
python tools/verify_hangul_pages.py \                     # 2) 한글 페이지 비교
    --inventory out/rt/inventory.tsv \
    --orig-root <원본_폴더> --rt-root out/rt \
    --status IR_DIFF,PASS --sample 40 --seed 42 [--pdf] -o out/hangul_pages.tsv
```

### 옵션
| 옵션 | 설명 |
|------|------|
| `--batch ORIG RT` / `--inventory TSV` | 입력(상호배타). 인벤토리는 `--orig-root`·`--rt-root` 필요 |
| `--status A,B` | 인벤토리 상태 필터(예: `IR_DIFF,PASS`) |
| `--sample N --seed S` | 재현 가능한 무작위 표본(0=전수) |
| `--pdf` | 한글 PDF 내보내기 후 PyMuPDF 페이지수 교차검증 |
| `-o TSV` | 결과 저장(헤더에 `git_head` 기록) |
| `--visible` | 한글 창 표시(디버깅) |

## 4. 판정·종료코드
- `OK`(원본=rt) / `COLLAPSE`(rt<원본) / `EXPAND`(rt>원본) / `ERR`(파일별 오류, 격리).
- **COLLAPSE ≥ 1 이면 종료 코드 1**(게이트). pyhwpx/PyMuPDF 미설치·입력 오류는 2.
- 콘솔·TSV 헤더에 **git HEAD** 기록 → stale-binary 측정 오보(v1 F2') 재발 봉인.

## 5. 한계
- **한글 의존**: Windows+한컴 전용. Linux CI 게이트 불가 — 한컴 보유 컨트리뷰터의 **로컬 오라클**.
- 한글 COM 불안정(행/팝업) 가능 — 파일별 예외 격리로 전체 중단은 방지하나, 개별 ERR 은 수동 확인.
- 대량은 `--sample` 권장(한글 1건당 수 초). 전수는 시간 소요.
- **페이지수 비교**이며 시각 픽셀 충실도는 검사하지 않음(T4 시각 diff 는 후속 별도).

## 6. 관련
- 수행/구현 계획: `mydocs/plans/task_m100_1560{,_impl}.md`
- 단계 보고서: `mydocs/working/task_m100_1560_stage{1..3}.md`
- 최종 보고서: `mydocs/report/task_m100_1560_report.md`
- 배경(붕괴 발견): `output/poc/fidelity3/report.md`, #1557(secCnt)
- 컨벤션: `tools/verify_hwpx.py`(단일파일 한글 검증)
