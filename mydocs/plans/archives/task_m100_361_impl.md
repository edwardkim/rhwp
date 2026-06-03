# Task #361 구현계획서 — TypesetEngine page_num + vpos reset 결함 정정

## 사전 조사 (수행계획 후 추가)

### finalize_pages 의 page_num 갱신 — 정상 코드 존재

`src/renderer/typeset.rs:1764` 의 `finalize_pages`:
```rust
let mut page_num: u32 = 1;
for page in pages.iter_mut() {
    // ...
    page.page_number = page_num;
    // ...
    page_num += 1;
}
```

코드상으로는 1, 2, 3, ... 으로 갱신되어야 함. dump-pages 가 모두 page_num=1 인 것은 다른 결함.

### dump 의 page_number 출력
`src/document_core/queries/rendering.rs:1286` — `page.page_number` 직접 출력.

### 가설 H1: finalize_pages 가 호출 안됨 / 일찍 break
검증: typeset_section 의 line 488 호출이 실제 실행되는지 진단.

### 가설 H2: page_num 이 finalize 후 어딘가에서 1 로 재설정
검증: rendering.rs / pagination 후처리 코드 grep.

### 가설 H3: section 내 모든 페이지가 같은 page_num 을 받는 다른 origin
검증: typeset 결과의 PageContent 가 1개씩 만들어지지 않고 1개를 공유하는지 (Rc/Clone 결함).

## 단계 (4 단계, 각 단계는 보고서 + 승인 게이트)

### Stage 1 — 결함 origin 정량 분석

**Task 1.1 — page_num 결함 origin 식별**
- `typeset_section` 의 `finalize_pages` 호출이 실행되는지 디버그 출력 추가 (임시)
- finalize_pages 의 `page_num += 1` 진행 추적
- `dump_page_items` 가 출력하는 `page.page_number` 가 어디서 1 로 고정되는지

**Task 1.2 — vpos 누적 origin 식별**
- `dump_page_items` 의 `vpos=` 표시는 어디 데이터에서 가져오는지
- `compute_hwp_used_height` 함수의 `hwp_used` 계산 로직 (페이지 마다 reset 되는지)
- LINE_SEG.vpos 가 페이지 경계에서 어떻게 처리되는지 (typeset 단계? layout 단계?)

**Task 1.3 — section 1 의 page_num 시작값**
- v0.7.3 / Paginator 에서는 k-water-rfp p3 (section 1 의 첫 페이지) page_num=1 부터 시작
- TypesetEngine 에서는 어떻게 갱신되는지

**산출물**: `mydocs/working/task_m100_361_stage1.md`
- 진단 결과 (정확한 결함 위치 + 누적 메커니즘)
- 수정 방향 가설

### Stage 2 — 수정 방안 설계

**Task 2.1 — page_num 수정안 확정**
- finalize_pages 가 호출되지 않거나 결과가 무시되면 → 호출 보장 또는 결과 보존 수정
- finalize_pages 자체 결함이면 → 직접 수정

**Task 2.2 — vpos / hwp_used 수정안 확정**
- typeset 결과의 PageContent 안 vpos 가 누적인지 reset 인지 결정
- dump 출력에서 vpos 변환이 필요한지 결정 (페이지 시작 vpos 기준 상대값)

**Task 2.3 — page_num 사용처 영향 분석**
```
grep -rn "page_number\b" src/ | grep -v test
```
- 머리말꼬리말 렌더링
- page-number 컨트롤 (PageNumberPos)
- 머리말꼬리말 apply (Even/Odd)
- dump / debug 출력
모든 사용처가 정상 page_num 을 받는지 점검.

**산출물**: `mydocs/working/task_m100_361_stage2.md`
- 수정 위치 + 변경 내용 명세
- 영향 범위 + 회귀 검증 항목

### Stage 3 — 코드 수정 + 자동 회귀

**Task 3.1 — 수정 적용**
- Stage 2 확정된 수정만 적용 (최소 변경)

**Task 3.2 — 자동 회귀 시퀀스**
1. `cargo build --release`
2. `cargo test --lib` (1008 passed 확인)
3. `cargo test --test svg_snapshot` (6/6 또는 영향 시 UPDATE_GOLDEN 검토)
4. `cargo test --test issue_301`
5. `cargo clippy --lib -- -D warnings`
6. `cargo check --target wasm32-unknown-unknown --lib`

**Task 3.3 — 페이지네이션 회귀 검증**
- v0.7.3 와 page_num 비교 (kps-ai p1~p11, k-water-rfp p3~p10)
- LAYOUT_OVERFLOW 회귀 0 (Task #359 효과 유지: k-water-rfp 0 건 유지)
- 7 핵심 샘플 + form-002 + hwp-multi-001 페이지 수 + page_num

**Task 3.4 — dump-pages 출력 비교**
- 페이지마다 vpos 가 0 부터 시작하는지
- hwp_used 가 페이지 콘텐츠 높이만 표시 (≈ used_height)

**산출물**: `mydocs/working/task_m100_361_stage3.md`
- 수정 내용 (diff)
- 자동 검증 결과
- 페이지네이션 회귀 비교표

### Stage 4 — WASM 빌드 + 시각 검증 + 최종 보고서

**Task 4.1 — WASM Docker 빌드**
```
docker compose --env-file .env.docker run --rm wasm
```

**Task 4.2 — 시각 판정 케이스**
- k-water-rfp 의 머리말꼬리말 페이지 번호 (p1=1, p2=2, ...)
- kps-ai p1~p11 페이지 번호 정상 표시
- 7 핵심 샘플 + form-002 회귀 0
- Task #359 효과 유지 (k-water-rfp p3 LAYOUT_OVERFLOW 0)

**Task 4.3 — 최종 보고서**
- `mydocs/report/task_m100_361_report.md`

**Task 4.4 — 트러블슈팅**
- `mydocs/troubleshootings/typeset_page_num_vpos_reset.md`

**Task 4.5 — orders 갱신**
- `mydocs/orders/20260427.md` Task #361 추가

**Task 4.6 — 타스크 브랜치 커밋 + local/devel merge**
- 작업지시자 시각 판정 통과 후
- 이슈 클로즈 (작업지시자 승인 후)

## 회귀 방지 체크리스트 (트러블슈팅 등록 항목 사전 메모)

- TypesetEngine 의 page_num 갱신 위치는 finalize_pages
- finalize_pages 호출 보장 (typeset_section 의 마지막에)
- page_num 사용처는 grep -rn "page_number" 로 사전 점검
- vpos 누적 vs reset 의 책임 위치 명확히

## 위험 요소 + 대응

| 위험 | 대응 |
|------|------|
| Stage 1 에서 단일 origin 이 아닐 가능성 | 정량 진단 후 우선순위 정렬, 큰 결함부터 수정 |
| page_num 사용처 광범위 + 영향 누락 | grep + 회귀 테스트로 전수 점검 |
| Task #359 의 fit drift 수정 효과 손상 | Stage 3 의 LAYOUT_OVERFLOW 회귀 검증 |
| 머리말꼬리말 렌더링 회귀 | Stage 4 시각 판정 |

## 참고

- 수행계획서: `mydocs/plans/task_m100_361.md`
- 이슈: [#361](https://github.com/edwardkim/rhwp/issues/361)
- 회귀 도입 commit: `edddebd Task #313`
