---
kind: verification
status: completed
canonical: mydocs/manual/verification/visual_sweep_guide.md
last_verified: 2026-08-02
---

# Task #3738 Stage 19 visual sweep — HWP p25 그림 25 복원

## 범위와 기준

- HWP: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 동일 문서 HWPX: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx`
- 한컴 2020 기준 PDF: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 명령:

  ```bash
  python3 scripts/visual_sweep.py \
    --key issue3738-stage19-hwp-p023-p025-after \
    --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
    --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
    --pages 23,24,25 --dpi 144 \
    --rhwp-bin target/review-planet6897-20260802/release-test/rhwp \
    --out /private/tmp/rhwp-stage19-p023-p025-after
  ```

선택 raster/PDF/compare/overlay/review 범위는 p23–25다. SVG와 render tree는 문서 전체 220쪽을
export했지만 `requested_pages=completed_pages=[23,24,25]`, `missing_pages=[]`, `run_state=complete`다.

## p25 판정

수정 전 p25의 그림 25는 render tree에서 `Image y=-88.3px`로 page frame 밖에 나가 완전히 보이지 않았다.
수정 후 그림 25를 소유한 `pi=357` 1×1 RowBreak 표는 `y=243.2px`이고 내부 Image는
`(142.0,244.1,475.8×261.7)`로 표 frame 안에 있다. 3-way review에서 rhwp와 기준 PDF 모두 그림 25와
그림 26 두 개가 순서대로 보인다.

자동 지표는 p25 pixel match `92.08907%`, ink match `69.93313%`, 구조 flag `0건`이다. 이 수치는
폰트 raster 차이를 포함하므로 전체 문서 fidelity의 절대 판정으로 쓰지 않으며, 여기서 확정하는 범위는
그림 25의 page-local visibility 복원뿐이다. p23–24는 유지보수자가 기준 PDF와 직접 대조해 정상으로
판정한 무회귀 페이지다.

## focused 검증

- `cargo fmt --check` 통과
- `native_hwp5_same_page_stale_empty_rowbreak_picture_resets_offset` unit regression 1/1 통과
- `issue_3738_rowbreak_table_footnote_fragment` focused fixture 묶음 9/9 통과
- `cargo build --profile release-test --bin rhwp` 통과
- 사용자가 수행한 WASM 빌드는 재실행하지 않았다.

unit test를 실행하면서 `BlockTableContinuationPreparedState` test fixture가 추가된
`queue_table_footnotes`·`table_footnotes` 필드를 초기화하지 않아 컴파일되지 않는 기존 유지보수 결함을 발견했다.
두 필드에 중립값(`false`, 빈 벡터)만 명시해 test contract를 현재 구조체 정의와 동기화했다.

## 장기 증적과 입력 해시

- [p25 3-way review](../pr/assets/pr_3740_issue3738_stage19/hwp_p025_review_after.png)
- HWP SHA-256: `50094a3db2b2003b293c5cbf43014d001aa97929acb488cef0cb7ea0e16b3113`
- HWPX SHA-256: `8ae9dc95643d0902fcced2af73badd732aea86c1cc5b875ef7b1272bccba862c`
- PDF SHA-256: `7879ffee6313575132187c44c0090cd2e62c32c12c29b7eabd989181acf27b3a`
- review PNG SHA-256: `9dfb22e8569703f33692a6b75ab71d4625bc1f3ccbdab66f9bfeac0625dd4bcc`

원본 HWP/HWPX/PDF는 위 저장소 경로에 보관한다. p43, p44–45, p52–53, p66–67, p83–85,
p90, p94, p106, p107–108을 포함한 잔여 결함은 [Stage 19 조사 기록](task_m100_3738_stage19.md)에
명시적으로 이월되어 있으며, 이 결과는 그것들의 해결이나 220/215쪽 차이 해소를 주장하지 않는다.
