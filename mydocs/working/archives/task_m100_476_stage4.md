# Task M100 #476 Stage 4 완료 보고서

## 1. 광범위 회귀 검증

### 1-1. 전체 단위/통합 테스트

```
cargo test --release
test result: ok. 1078 passed; 0 failed; 1 ignored;  (lib)
test result: ok. 14 passed; 0 failed;
test result: ok. 25 passed; 0 failed;
... (모든 테스트 슈트 통과)
```

### 1-2. 골든 SVG 스냅샷 테스트

```
cargo test --release --test svg_snapshot

running 6 tests
test table_text_page_0 ... ok
test issue_157_page_1 ... ok
test issue_267_ktx_toc_page ... ok
test form_002_page_0 ... ok
test render_is_deterministic_within_process ... ok
test issue_147_aift_page3 ... ok

test result: ok. 6 passed; 0 failed;
```

시각 회귀 없음. 인라인 도형 포함 페이지(`issue-147`, `issue-267` 등) 모두 정상.

### 1-3. 핵심 회귀 케이스 최종 시각 검증

`samples/21_언어_기출_편집가능본.hwp`:

| 항목 | Before | After (A 수정 적용) |
|------|--------|---------------------|
| 페이지 11 단 1 끝 박스 | 누락 | ✓ 정상 출현 (x=616.1, y=1313.3) |
| 페이지 12 잘못된 박스(y=742) | 출현 | ✓ 제거 |
| 23번 박스(페이지 12) | 출현 (위치 별도 이슈) | 출현 (위치 별도 이슈) |

페이지 11 PDF와의 정성 비교: paragraph 238 본문 중 "[배너지와 뒤플로]" 박스가 단 1 본문 끝부분에 정상 인라인 위치에 출현. PDF와 일치.

## 2. D 차단(Stage 1) 처리 결정

### 2-1. 결정: 유지

`shape_layout.rs:218`의 D 차단 분기는 다음 이유로 **유지**:

1. **회귀 안전장치**: 향후 paginator 라우팅에 미커버 케이스가 발생하면 잘못된 위치에 박스가 그려지는 것을 차단하여 **눈에 띄는 시각 회귀를 방지**.
2. **silent failure 시그널**: `RHWP_DEBUG_LAYOUT=1` 시 stderr 로그로 누락 케이스 즉시 감지 가능.
3. **비용 0**: 통상 케이스(A 수정으로 inline_pos 정상 등록)에서는 차단 분기 진입 자체가 없음 — 성능/렌더링 영향 없음.

본 task의 대표 케이스는 A 수정으로 D 차단을 발동시키지 않음(검증됨). D 차단이 발동된다면 그것은 새로운 회귀 시그널.

## 3. 변경 파일 정리

### 3-1. 본 task 변경

| 파일 | 변경 내용 |
|------|-----------|
| `src/renderer/pagination.rs` | `find_inline_control_target_page` 공용 함수 추가 |
| `src/renderer/pagination/engine.rs` | Paginator의 Shape 분기에서 라우팅 호출 |
| `src/renderer/typeset.rs` | TypesetEngine의 Shape 분기에서 라우팅 호출 |
| `src/renderer/layout/shape_layout.rs` | D 안전장치(treat_as_char + inline_pos=None → return) |
| `src/renderer/layout/paragraph_layout.rs` | 빈 paragraph + 인라인 Shape 케이스 inline_pos 등록 보강 |

### 3-2. 디버그 코드

모든 임시 디버그 로그(`#476 reg`, `#476 paginate Shape` 등) 제거. shape_layout.rs의 `#476 skip`만 `RHWP_DEBUG_LAYOUT=1` 환경변수 시 출력되도록 유지.

## 4. 산출물

- 수행 계획서: `mydocs/plans/task_m100_476.md`
- 구현 계획서: `mydocs/plans/task_m100_476_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_476_stage{1,2,3,4}.md`
- 최종 결과 보고서: `mydocs/report/task_m100_476_report.md` (다음 단계)

## 5. 다음 단계

최종 결과 보고서 작성 + 커밋. 머지는 작업지시자 승인 후 진행.
