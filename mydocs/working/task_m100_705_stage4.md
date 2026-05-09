# Task #705 Stage 4 — 결함 #3 정정 (dump 인프라, main.rs)

## 산출물

- `src/main.rs` — 셀 안 controls 매칭에 `Control::PageHide(ph)` 분기 추가 (+5 line)
- `mydocs/working/task_m100_705_stage4.md` (본 보고서)

## 코드 변경 (main.rs:1665-1670 추가)

```rust
Control::PageHide(ph) => {
    println!("{}    ctrl[{}] PageHide: header={} footer={} master={} border={} fill={} page_num={}",
        indent, ci,
        ph.hide_header, ph.hide_footer, ph.hide_master_page,
        ph.hide_border, ph.hide_fill, ph.hide_page_num);
}
```

기존 `Control::Picture(p) => ...` + `Control::Shape(s) => ...` 매칭 직후, `_ => {}` 직전에 삽입.

## 검증 — 메인테이너 권위 측정 dump 표시

### 명령

```
rhwp dump samples/aift.hwp -s 0 -p 1
```

### 출력 (해당 부분)

```
[0]   셀[167] r=34,c=0 rs=1,cs=27 h=12161 w=47290 pad=(141,141,113,113) aim=false bf=68
       paras=10 text="|관련 법령 및 규정과 모든 의무사항을 준수하면서 이 과
                      ||       년        월        일|...
[0]       ctrl[0] PageHide: header=true footer=true master=true border=true fill=true page_num=true
```

→ **메인테이너 권위 측정 데이터와 정확 일치**:
- 위치: `s0/p[1]/Table[0]/셀[167]` ✓
- text: "       년        월        일" 포함 ✓
- PageHide 6 필드 모두 `true` ✓

## 회귀 sweep

```
cargo test --release --lib
test result: ok. 1123 passed; 0 failed; 1 ignored
```

→ **0 fail** (main.rs 의 dump 분기 추가는 출력 영역만 영향, 빌드/테스트 무영향).

## 디버깅 인프라 효용

본 정정 전:
```
[0]       (셀[167] 의 PageHide 컨트롤은 _ => {} 으로 무시되어 dump 출력에 미표시)
```

본 정정 후:
```
[0]       ctrl[0] PageHide: header=true footer=true master=true border=true fill=true page_num=true
```

→ 향후 PageHide 결함 디버깅 시 셀 안 PageHide 즉시 시각 확인 가능. PR #640 (Task #637) 의 H2 가설 기각이 본 결함 #1 + #3 (디버깅 한계) 으로 인한 잘못된 측정이었음을 향후 재발 방지.

## Stage 5 진입 결정

**Stage 5 (회귀 검증 + 최종 보고)** 진입 가능:

1. `cargo test --release --lib` 1123 passed 재확인
2. `cargo clippy --release` warning 0 확인
3. aift.hwp 페이지 카운트 무변화 확인 (77)
4. Stage 0 의 174 영향 샘플 (6건) 추가 검증 — 페이지 카운트 + page_hide 매핑
5. SVG smoke check (aift p1, p2, p4, p5) 표시/미표시 정합 시각 확인
6. 최종 보고서 작성

## 관련

- 수행 계획서: `mydocs/plans/task_m100_705.md`
- 구현 계획서: `mydocs/plans/task_m100_705_impl.md`
- Stage 0~3 보고서
- 본 보고서: `mydocs/working/task_m100_705_stage4.md`
