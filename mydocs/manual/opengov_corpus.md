# opengov 고정 실문서 회귀 말뭉치 가이드 (Task #1564)

`samples/hwpx/opengov/` — 서울 정보소통광장 정보공개 결재문서 클래스별 대표를 동결한
**재현 가능한 충실도 회귀 기준선**. hwpdocs(수집기로 건수가 변하는 비재현 대상)를 대체.

## 1. 구성 (8건, 1.7MB)
출처 opengov.seoul.go.kr 정보공개(공개), 수집 2026-06-26, PII 방침 **A(그대로 동결)**.
클래스 매핑은 `samples/hwpx/opengov/README.md` 참조.

## 2. 두 갈래 회귀 게이트

### (a) IR 스냅샷 — `tests/opengov_corpus_snapshot.rs` (Linux CI 가능)
`tests/fixtures/opengov_snapshot.tsv`(골든 id/status/diff)와 비교:
- **악화**(PASS→IR_DIFF, diff 증가, FAIL) → 실패(회귀, 결함 조사).
- **개선**(IR_DIFF→PASS, diff 감소) → 실패(스냅샷 갱신=승격 강제).
- `cargo test --test opengov_corpus_snapshot`.
- diff=0 강제 baseline(`hwpx_roundtrip_baseline`)과 별개 — opengov 하위는 거기서 제외됨.

### (b) 한글 페이지 오라클 — `tools/verify_hangul_pages.py` (#1560, 로컬·한글)
```bash
rhwp hwpx-roundtrip --batch samples/hwpx/opengov -o out/rt
python tools/verify_hangul_pages.py --batch samples/hwpx/opengov out/rt -o out/hangul.tsv
```
현재 코드(HEAD) 기준 기대 verdict:
| 파일 | 한글 verdict |
|------|--------------|
| 36382669 | **OK 8→8** (#1557 secCnt 회귀 가드 — 붕괴 재발 시 즉시 감지) |
| 36384285·36385464·36388571·36388853·36389298 | OK |
| 36383351 | COLLAPSE 2→1 (잔여 단일구역 붕괴) |
| 36387103 | COLLAPSE 2→1 (잔여 단일구역 붕괴) |
→ OK 6 / COLLAPSE 2. 36382669 가 OK 인 한 #1557 은 회귀 없음.

## 3. 갱신 절차
- 결함 수정으로 status/verdict **개선** 시: IR 스냅샷은 테스트 실패 → `opengov_snapshot.tsv`
  갱신. 한글 verdict 는 본 매뉴얼 표 갱신.
- **악화** 시: 회귀 → 결함 조사(스냅샷 임의 갱신 금지).
- 신규 클래스: 파일 동결(`samples/hwpx/opengov/`) + 스냅샷 행 + README/매뉴얼 표 추가.

## 4. 한계
- 스냅샷은 IR 구조 회귀(diff/status). 한글 전용 페이지 붕괴는 #1560 도구(한글 의존, 로컬).
- 시각 픽셀 diff(T4)·pic 시각 triage 는 별도(측정도구 3순위).

## 5. 관련
- 계획/보고: `mydocs/plans/task_m100_1564{,_impl}.md`, `mydocs/working/task_m100_1564_stage{1..3}.md`,
  `mydocs/report/task_m100_1564_report.md`
- 말뭉치 README: `samples/hwpx/opengov/README.md`
- 한글 오라클: `mydocs/manual/hangul_page_oracle.md`(#1560)
