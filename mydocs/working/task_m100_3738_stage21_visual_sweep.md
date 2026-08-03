---
kind: verification
status: completed
canonical: mydocs/manual/verification/visual_sweep_guide.md
last_verified: 2026-08-02
---

# Task #3738 Stage 21 visual sweep — HWP p154–155·p157–158 RowBreak 표 복원

## 범위와 기준

- HWP: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 동일 문서 HWPX: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx`
- 한컴 2020 기준 PDF: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 명령:

  ```bash
  python3 scripts/visual_sweep.py \
    --key issue3738-stage21-hwp-p154-p155-p157-p158 \
    --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
    --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
    --pages 154,155,157,158 --dpi 144 \
    --rhwp-bin target/review-planet6897-20260802/release-test/rhwp \
    --out /private/tmp/rhwp-stage21-after-sweep
  ```

선택 raster/PDF/compare/overlay/review 범위는 p154, p155, p157, p158이다. SVG와 render tree는 수정 후
native HWP 219쪽 전체를 export했고, `requested_pages=completed_pages=[154,155,157,158]`,
`missing_pages=[]`, `run_state=complete`다.

## 직접 판정

| 쪽 | 기준 PDF와의 physical owner 판정 | 결과 |
| --- | --- | --- |
| 154 | pi=1682 표의 마지막 셀 문단과 각주 210이 같은 쪽에서 끝난다. | 일치 |
| 155 | 불필요한 pi=1682 tail 없이 `(3) 평가 절차`로 시작한다. 단, 기준 PDF에는 없는 분홍 흐름도 그림이 표 우하단에서 본문·각주 위로 겹친다. | **page boundary 해결, 그림 겹침 잔여** |
| 157 | 표 37의 `BTS Guideline`~`OPTN policy` 첫 fragment가 footer 안에 남는다. | 일치 |
| 158 | `BC Canada` continuation 뒤에 `신체 검진은 체중` 본문이 이어지고 표가 page frame 밖으로 새지 않는다. | 일치 |

자동 구조 지표는 네 쪽 모두 flag 0건이고 render-tree frame-tail-overflow 후보도 0건이다. 그러나 이 자동
판정은 p155의 분홍 흐름도 오배치를 잡지 못했다. overlay pixel match는 p154 90.83726%, p155 90.83666%,
p157 94.55227%, p158 95.61487%다. Hancom/로컬 글꼴 raster 차이를 포함하므로 이 수치는 전체 fidelity의
절대 pass/fail가 아니라 physical owner·clip 복원의 보조 근거로만 사용한다.

## focused 검증

- `cargo fmt --check` 통과
- `cargo test --profile release-test --test issue_3738_rowbreak_table_footnote_fragment` 11/11 통과
- `cargo build --profile release-test --bin rhwp` 통과
- 사용자가 이미 수행한 WASM 빌드는 재실행하지 않았다.

## 장기 증적과 입력 해시

- [p154 3-way review](../pr/assets/pr_3740_issue3738_stage21/hwp_p154_review_after.png)
- [p155 3-way review](../pr/assets/pr_3740_issue3738_stage21/hwp_p155_review_after.png)
- [p157 3-way review](../pr/assets/pr_3740_issue3738_stage21/hwp_p157_review_after.png)
- [p158 3-way review](../pr/assets/pr_3740_issue3738_stage21/hwp_p158_review_after.png)
- [run manifest](../pr/assets/pr_3740_issue3738_stage21/run_manifest.json), [구조 지표](../pr/assets/pr_3740_issue3738_stage21/metrics.json), [overlay 지표](../pr/assets/pr_3740_issue3738_stage21/overlay_metrics.json)
- HWP SHA-256: `50094a3db2b2003b293c5cbf43014d001aa97929acb488cef0cb7ea0e16b3113`
- HWPX SHA-256: `8ae9dc95643d0902fcced2af73badd732aea86c1cc5b875ef7b1272bccba862c`
- PDF SHA-256: `7879ffee6313575132187c44c0090cd2e62c32c12c29b7eabd989181acf27b3a`
- review PNG SHA-256: p154 `cd2d130e035813bbf5f08895229563699a7462171834bb22dc570e8d5099f69e`,
  p155 `eaa89c5d75478d75eeaef5efb445138b06e466a0a05da7a8e9ee225e458b9223`,
  p157 `e355936b1f5f109bdde5410464c74270c05a26e766b9599cb566c439165cb451`,
  p158 `88aedb4e82c7afef7f8b47e91397487982e0bb2d1064efa752059c854477090b`
- 구현 revision: `edeb2396e2d64207a53b019916963d513556da92`
- run manifest SHA-256: `f6b7ac7eb5cea54e3f4263244135547c58584b9380a8a941a251827232023bd5`

원본 HWP/HWPX/PDF는 위 저장소 경로에 보관한다. 이 Stage는 220→219의 단일 page-map 분기만 해소했다.
p155 분홍 흐름도 오배치, 기준 PDF 215쪽 대비 native HWP 219쪽의 잔여 차이, p43·54·67·85·106 등의 독립
후보는 다음 Stage에서 다시 원인 분석한다.
