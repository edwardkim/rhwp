# Task M100 / 이슈 #5882 처리 결과

**제목**: fix(renderer): 미리보기 없는 OLE 의 진단용 자리표시가 사용자 산출물에 그려지는 것을 고친다 — 3067979 조명시설편 45쪽 정합 (#5882)

- 이슈: https://github.com/edwardkim/rhwp/issues/5882
- 대상 문서: `3067979_[붙임 2] 도로안전시설 설치 및 관리 지침- 조명시설편….hwpx` 인쇄 45쪽
  (samples 미수집 — 이슈 본문의 구조 분석을 그대로 재현한 변형 문서로 검증, 아래 3절)
- 브랜치: `fix/ole-placeholder-5882` (base `upstream/devel` = `ad28677080`)

---

## 1. 결함

미리보기·차트·이미지 폴백이 전부 실패한 OLE(3067979 의 ole23 — 유효 CFB 헤더 뒤
디렉터리 엔트리 0개인 빈 CFB)에 대해 rhwp 는 회색(#F0F0F0) 점선 상자와
`OLE 개체 (BinData #N)` 라벨을 **본문 렌더 트리에 넣어** export-svg/pdf 산출물까지
그렸다. 한글 2022 는 같은 자리에 아무것도 그리지 않는다(빈 자리).

## 2. 원인

`src/renderer/layout/shape_layout.rs` 의 OLE 폴백 말단에서 사유 라벨 유무와 무관하게
placeholder 노드를 push 했다. 개발·진단용 표시가 사용자 산출물로 나가면 문서에 없던
이물이 된다.

## 3. 수정

`chart_error_label` 이 없을 때(#5882 대상 — 내용 없는 OLE 의 무표식 회색 자리표시)는
기본으로 렌더 트리에 넣지 않고, `RHWP_DIAG_OLE_PLACEHOLDER` 환경 변수로 종전 진단
표시를 복원한다. [#5582] 차트 해석 실패 사유 라벨은 대상에서 제외하고 종전대로 그린다.

## 4. 검증

### 재현 문서 구성

원본 샘플 미수집으로 이슈가 기술한 ole23 과 같은 조건을 `samples/한셀OLE.hwpx` 변형으로
만들었다: `BinData/ole1.ole` 을 **유효 헤더 + FAT 1섹터 + 디렉터리 0엔트리(1,536바이트)의
빈 CFB** 로 교체. 모든 미리보기 폴백이 실패해 종전에는 자리표시가 그려졌다.

| | export-svg 산출 |
|---|---|
| 수정 전 (base exe) | `<rect … fill="#f0f0f0" stroke="#707070" stroke-dasharray="6 3"/>` + `OLE 개체 (BinData #1)` 라벨 |
| 수정 후 (new exe) | 해당 요소 없음 — 빈 자리 |

![깨진 OLE 전후](https://raw.githubusercontent.com/kevin9327/rhwp/fix/ole-placeholder-5882/mydocs/report/edit_demo_5882/broken_ole_before_after.png)

확대(좌 전/우 후):

![확대 전후](https://raw.githubusercontent.com/kevin9327/rhwp/fix/ole-placeholder-5882/mydocs/report/edit_demo_5882/broken_ole_closeup_before_after.png)

정상 OLE(`한셀OLE.hwpx`, 미리보기 WMF 보유)는 종전대로 렌더된다 — 자리표시 억제가
정상 경로를 건드리지 않는다는 회귀 가드를 `tests/cases/issue_5882_ole_placeholder_not_drawn.rs`
에 함께 넣었다.

### 게이트 · 테스트

| 항목 | 결과 |
|---|---|
| 코퍼스 쪽수 게이트 A/B (`tools/render_page_gate.py`, 259문서) | base exe vs new exe **변화 0행** — 매치 249 (96.1%) 동일 |
| `cargo test --profile release-test --lib -p rhwp` | **3889 passed / 0 failed** |
| regression_suite 004·007·009·012·013·021 | 전부 passed (신규 `issue_5882_*` 3케이스 포함, suite_012) |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | 차이 없음 |
| `node scripts/rust-unit-test-tiers.mjs --check --base-ref upstream/devel` | 4221 tests 정합 |
| `node scripts/rust-test-suite-manifest.mjs --check --base-ref upstream/devel` | 894 sources 정합 |

## 5. 핀 갱신

없음 — 쪽수·레이아웃 불변(렌더 트리에서 진단 노드 하나가 빠지는 것뿐), 골든 SVG 중
자리표시를 긍정 단정하는 테스트도 없었다(issue_1156 의 placeholder 조회는 차트 미렌더
폴백 경로라 미영향).
