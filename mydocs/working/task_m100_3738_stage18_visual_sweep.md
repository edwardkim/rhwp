---
kind: verification
status: completed
canonical: mydocs/manual/verification/visual_sweep_guide.md
last_verified: 2026-08-02
---

# Task #3738 Stage 18 visual sweep — HWP p37 TAC 그림 중복

## 기준과 범위

- HWP: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 동일 문서 HWPX: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx`
- 한컴 2020 기준 PDF: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 명령: `python3 scripts/visual_sweep.py --key issue3738-stage18-hwp-p037 --hwp <HWP> --pdf <PDF> --pages 37 --dpi 144 --rhwp-bin target/review-planet6897-20260802/release-test/rhwp --out /private/tmp/rhwp-stage18-p037-sweep`

선택 raster/PDF/compare/overlay/review 범위는 37쪽 하나다. SVG와 render tree는 문서 전체 220쪽을 export했으나
`requested_pages=completed_pages=[37]`, `missing_pages=[]`, `run_state=complete`다.

## 판정

수정 전 p37에는 그림 37이 좌측 정상 위치와 오른쪽 하단 밖의 fallback 위치에 두 번 보여 그림이 셋처럼 보였다.
수정 후 render tree의 p37 Image는 그림 37 `(94.5,706.9,254.0×221.0)`과 그림 38
`(413.6,728.0,247.8×211.0)` 두 개뿐이다. 3-way review에서 PDF와 rhwp 모두 그림 37·38이 한 쌍으로
남고, 이전의 오른쪽 하단 세 번째 그림은 사라진 것을 확인했다.

자동 지표는 pixel match `93.29936%`, ink match `49.04027%`, 구조 flag `0건`이다. 폰트 raster와 기존
텍스트·표 metric 차이가 overlay ink 수치를 낮추므로 이 수치만으로 전체 페이지 정합을 주장하지 않는다. 여기서의
판정 범위는 p37의 중복 그림 제거뿐이다.

## 검증

`CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3738_rowbreak_table_footnote_fragment -- --nocapture`는 8/8 통과했다.
`cargo fmt --check`와 `git diff --check`도 통과했다. 사용자가 별도로 완료한 WASM 빌드는 재실행하지 않았다.

## 장기 증적과 입력 해시

- [p37 3-way review](../pr/assets/pr_3740_issue3738_stage18/hwp_p037_review_after.png)
- HWP SHA-256: `50094a3db2b2003b293c5cbf43014d001aa97929acb488cef0cb7ea0e16b3113`
- HWPX SHA-256: `8ae9dc95643d0902fcced2af73badd732aea86c1cc5b875ef7b1272bccba862c`
- PDF SHA-256: `7879ffee6313575132187c44c0090cd2e62c32c12c29b7eabd989181acf27b3a`
- review PNG SHA-256: `c0f2c7c15e46952d4a04bc4f2640923ebac30d29679ca19775ad6544ab0f2239`

원본 HWP/HWPX/PDF는 위 저장소 경로에 보관한다. 이 증적은 37쪽 그림 중복의 해결 근거이며, 전체 220/215쪽
pagination 차이나 다른 이월 페이지의 완료를 뜻하지 않는다.
