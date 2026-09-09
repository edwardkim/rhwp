# Task #1567: HWPX 표 셀 pic 드롭 해소 — 구현 계획서

> 수행계획서 `task_m100_1567.md` 승인. 3단계. HWPX serializer 범위.

## 확정 사실
- 원본 셀 pic 다수가 `binaryItemIDRef=""`(빈 ref) → 파서 `bin_data_id=0`.
- `write_img`(`picture.rs:256`) `resolve_bin_id(0)→None`→Err → `section.rs:701` 조용한 드롭.

## Stage 1 — 근본원인 확정 + 대표 격리
- `bin_data_id==0` ⟺ 빈 binaryItemIDRef 확인(HWPX 1-based, 0=무참조). 비-0 미해결 케이스 유무 점검.
- 대표(36385464·36388571) IR diff 의 pic 드롭이 빈-ref pic 임을 격리 확인.
- `write_img`/`write_picture`/`section.rs:701`·`1173` 정확 경로 매핑.
- 산출: `task_m100_1567_stage1.md` + 커밋(보고서).

## Stage 2 — serializer 수정 (빈 ref 보존)
- `write_img`(picture.rs): `resolve_bin_id(bin_id)` 실패 시 — **`bin_id==0`(빈 ref)**이면
  `binaryItemIDRef=""` 방출(에러 대신). 비-0 미해결은 종전 Err 유지(손실 은폐 방지).
- 그 결과 `write_picture` Ok 반환 → `section.rs:701` 정상 방출(드롭 없음).
- `cargo build` + 대표 케이스 roundtrip: `expected=[pic] actual=[]` 해소(`rhwp hwpx-roundtrip`).
- 단위 테스트: 빈 ref pic 직렬화 시 `binaryItemIDRef=""` 방출 단언.
- 산출: `task_m100_1567_stage2.md` + 소스 커밋.

## Stage 3 — 회귀 + 측정 + 보고
- opengov 스냅샷(#1564): 개선 파일 status 승격 → `tests/fixtures/opengov_snapshot.tsv` 갱신.
- `hwpx_roundtrip_baseline`(samples/hwpx) 회귀 없음.
- fidelity 재측정: `hwpx-roundtrip --batch hwpdocs` 표셀 pic 드롭 건수 감소 정량화(전/후).
- 산출: `task_m100_1567_stage3.md` + `mydocs/report/task_m100_1567_report.md` + 커밋.

## 주의
- 비-0 미해결(진짜 BinDataContent 누락)은 보존 아닌 진단 유지.
- HWP3 분기 금지. F3(#1556) 직접수정 아님(후속효과만 측정).
