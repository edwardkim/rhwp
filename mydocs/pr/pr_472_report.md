# PR #472 처리 보고서

**제목**: Task #470 + #471 + #473: cross-column vpos-reset/박스 stroke_sig + 그림 crop scale 정정
**작성자**: planet6897 (Jaeook Ryu / Jaeuk Ryu)
**처리 결과**: cherry-pick 머지 (Task #470 + #471) + Task #473 제외 (본 #477 와 충돌)

## 처리 요약

PR #472 의 67 commits 중 본질 신규 3 Task. 작업지시자 옵션 A 선택 — Task #470 + #471 cherry-pick, Task #473 제외.

| Task | 처리 | commit |
|------|------|--------|
| **Task #470** 다단 cross-paragraph vpos-reset 검출 완화 | cherry-pick (be17702) | 7e632e1 |
| **Task #471** cross-column 검출 stroke_sig 비교 | cherry-pick (b242862) | 64cb32d |
| **Task #473** 그림 crop 변환 scale 정정 | **제외** | 8f5a079 (본 #477 와 충돌) |

## Task #473 제외 사유

본 작업 사이클의 **Task #477 (commit `6057bd3`, 2026-04-30)** 가 동일 영역 (`compute_image_crop_src`) 정정. 두 정정 비교:

| 정정 방식 | Task #477 (이미 머지) | Task #473 (제외) |
|----------|---------------------|----------------|
| 룰 | 항상 75 HU/px (단일 룰) | 75 ± 5% 안일 때만 orig 사용, 아니면 75 fallback |
| 분기 | 없음 | 2 케이스 분기 |
| 작업지시자 통찰 | "이건 휴리스틱이 아닙니다. 룰입니다." | (분기 룰) |

**메인테이너 정정 (Task #477) 이 더 단순한 룰** — 작업지시자 통찰 "이건 룰이다" 정합. Task #473 의 정정 의도는 이미 본 #477 로 해결됨.

## 검증 게이트

| 검증 | 결과 |
|------|------|
| `cargo test --lib` | **1080 passed** (1078 + 신규 통합 테스트 2) ✅ |
| `cargo test --test svg_snapshot` | 6/6 ✅ |
| `cargo test --test issue_418` | 1/1 ✅ |
| `cargo clippy --lib -- -D warnings` | 0건 ✅ |
| WASM 빌드 | 4,204,787 bytes ✅ |

## 본 #431 (synam-001 15페이지) 자동 정정 점검

PR #424 (Task #412) 의 vpos 보정 변경이 본 #431 결함의 회귀 origin. Task #470 가 cross-paragraph vpos-reset 영역의 후속 정정이라 본 #431 자동 정정 가능성 점검.

```bash
cargo run --release --quiet --bin rhwp -- dump-pages samples/synam-001.hwp -p 14
```

**결과** (PR #472 cherry-pick 후):
- 15페이지: `PartialTable rows=6..7 cont=true used=0.0px` — 표 fragment 만 (빈 페이지)
- 16페이지: 동일 fragment (또 빈 페이지)

**판정**: Task #470 + #471 만으로는 본 #431 결함 자동 해결 안 됨. Task #470 는 다른 영역 (`cv != 0` 케이스 vs synam-001 15페이지의 다른 메커니즘). 본 #431 별도 정정 필요.

## 컨트리뷰터 댓글 안내

PR #472 댓글에서 다음 사항 안내:
1. Task #470 + #471 cherry-pick 머지 완료
2. Task #473 은 본 작업 사이클 Task #477 (75 HU/px 단일 룰) 와 동일 영역 정정 — 작업지시자 통찰 ("이건 룰이다") 정합. Task #473 의 분기 정정은 흡수 안 됨
3. 위키 [HWP 그림 Crop Scale 룰](https://github.com/edwardkim/rhwp/wiki/HWP-%EA%B7%B8%EB%A6%BC-Crop-Scale-Rule) 참조
4. PR #472 close

## 다음 단계

- PR #472 close + cherry-pick 머지
- 본 #431 결함 별도 정정 시작
