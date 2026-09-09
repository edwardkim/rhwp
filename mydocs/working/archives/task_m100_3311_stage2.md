---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3311.md
last_verified: 2026-08-01
---

# Task #3311 Stage 2 보고 — 회귀 가드 신설

## 산출물

`tests/issue_3311_malformed_cfb_no_panic.rs` — "손상 CFB 입력은 패닉이 아니라 `Err`"
계약을 고정한다. 공개 API(`HwpDocument::from_bytes`)만 사용.

케이스 구성 (177건):

- **`reporter_exact`** — 리포터 보고 값을 상수로 명시
  (FAT 824 entries / first DIFAT 128 / poison sector 1,851,072,928 / len 3,072).
- 합성 스윕 150건 — 길이 6종 × FAT 카운트 5종(0·1·824·65535·u32::MAX) × 손상 섹터
  id 5종의 조합.
- 실 샘플(`hwpers_test4_complex_table.hwp`) 뮤테이션 21건 — 헤더 7필드(섹터 크기
  지수·FAT 카운트·first dir·first mini FAT·mini FAT 카운트·first DIFAT·DIFAT 카운트)
  × 손상 값 3종. "정상 문서에서 한 필드만 깨진" 경로를 덮는다.
- 절단 스윕 5건 — 실 샘플의 1/2·1/3·1/4·1/8·1/16.

자기 방어 단언 2개: ①케이스 수 급감 감지(샘플 경로 변경 등으로 커버리지가 조용히
줄어드는 것 차단) ②전부 정상 개봉되면 실패("더 이상 malformed 가 아니다").

## red-check — 정식 가드로 재확인

`git worktree` 로 리포터 커밋(`8d3bfa4b`) 체크아웃 후 **이 테스트 파일 그대로** 실행:

```
thread 'malformed_cfb_returns_err_instead_of_panicking' panicked at
  src/parser/cfb_reader.rs:407:43:
range end index 8606 out of range for slice of length 512
test result: FAILED
```

리포터 보고(`cfb_reader.rs:407:43`, "range end index ... out of range for slice of
length ...")와 **동일 지점·동일 형태**다. 현행 devel 에서는 ok.

worktree 는 실행 후 제거했고(상주 stash 6개 무손상), 조사용 임시 probe 파일도 삭제했다.

## 실행 비용

<0.01s — 스위트 부담 없음.
