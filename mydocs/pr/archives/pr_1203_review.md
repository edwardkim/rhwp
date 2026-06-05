# PR #1203 검토 — HWPX curve 도형 `<hp:seg>` 파싱 (외곽선/태그 박스 렌더링)

- **작성일**: 2026-06-01
- **PR**: #1203 (OPEN)
- **컨트리뷰터**: @planet6897 (핵심 컨트리뷰터 — #1202/#1182/#1164/#1148/#1095 머지)
- **연결 이슈**: #1200 (`closes #1200`)
- **base/head**: `devel` ← `fix/1200-curve-seg-outline` (cross-repo)
- **규모**: 7 파일, +259/−0 (소스 `section.rs` 단 1파일 +52(분기+테스트), 나머지 docs ×6)
- **mergeable**: **CONFLICTING / DIRTY** → 로컬 충돌 해소 필요 (아래 4절)
- **CI**: Build & Test / Analyze ×3 / CodeQL 전부 SUCCESS. WASM skip.
- **라벨**: enhancement / 마일스톤 v1.0.0

## 1. 문제와 원인 (PR 본문 검증 — 코드 확인 완료)

`samples/3-09월_교육_통합_2022.hwpx` 9쪽 우측 단 "다른 풀이"가 한글 2022 PDF 에는 태그 박스
(`〉` 꺾쇠) 안에 표시되나, rhwp 는 **텍스트만 있고 박스 외곽선이 없음**.

근본 원인: "다른 풀이"는 `<hp:curve id="0">` 도형의 drawText 이고, 박스는 그 curve 외곽선.
이 curve 의 geometry 가 **`<hp:seg type x1 y1 x2 y2>` 417개**로 인코딩됨(`<hc:pt>` 0개).
파서 `parse_shape_object()`(`section.rs`)는 가변 꼭짓점을 **`b"pt"`(`<hc:pt>`) 분기에서만** 수집
→ `polygon_points` empty → `CurveShape.points` empty → `curve_to_path_commands_scaled()` 빈 path
→ **외곽선 미렌더**. drawText 텍스트는 별도 경로라 텍스트만 보였음.

**코드 확인**: `section.rs:3218` `b"pt"` 분기만 존재(seg 없음), `:3340` curve 가 `polygon_points`
를 `points` 로 사용 → 진단 정확. 렌더 인프라(stroke→SVG/Skia)는 기존 완비 → 점만 채우면 외곽선 생성.

## 2. 수정 내용 검토

`parse_shape_object()` `b"pt"` 옆에 `b"seg"` 분기 추가(`section.rs:3234`):
- x1/y1/x2/y2 파싱. `polygon_points` 비어 있으면 첫 seg `(x1,y1)` 1회 push + 각 seg `(x2,y2)` push
  → chain 폴리라인.
- `segment_types` 비움 → 렌더러 LineTo (seg 는 제어점 아닌 sampled 꼭짓점).
- **순수 추가**(+분기 1, 모델/렌더러/HWP3/공통 무변경). `<hp:seg>` 는 curve 에만 존재 →
  타 도형/기존 `<hc:pt>` 경로 불변.
- 회귀 테스트 1건: seg 3개 chain → `points = [(10,10),(90,10),(90,90),(10,10)]` 검증.

## 3. 위험 평가

- **낮음.** 파서 단일 파일, 순수 추가. seg 없으면 기존 동작 유지(무회귀). curve 전용.

## 4. 충돌 해소 (CONFLICTING)

PR 이 #1202(prefixChar) 머지 **전** 분기되어, `section.rs` tests 모듈에 PR 의 curve seg 테스트와
devel 의 prefixChar 테스트 2건이 **같은 라인 영역에 추가** → 텍스트 인접 충돌(의미 충돌 아님).
**본문 분기(`b"seg"`)는 충돌 없이 자동 머지됨.** tests 모듈만 충돌.

해소: 두 PR 의 테스트 **전부 보존**(prefixChar 2건 + curve seg 1건), 각 테스트를 온전히 분리.
- 잔여 충돌 마커 0, `b"seg"` 본문 분기 머지 확인, 테스트 3건 모두 존재 확인.

## 5. 검증 결과 (로컬 머지 시뮬 `pr1203-verify`)

| 단계 | 명령 | 결과 |
|------|------|------|
| 충돌 해소 | tests 모듈 3 테스트 보존 | ✅ 마커 0 |
| fmt | `cargo fmt --all --check` | ✅ clean |
| build | `cargo build` | ✅ Finished |
| 전체 테스트 | `cargo test --tests` | ✅ **failed 0** |
| curve seg 테스트 | `test_parse_curve_seg_populates_points` | ✅ ok |
| prefixChar 테스트 2건(#1202 보존) | | ✅ ok |
| **9쪽 외곽선 (정량)** | 긴 path d_len | ✅ **보정 후 15582**(417-seg), devel `b"seg"` 0개라 미존재 |
| 긴 path 속성 | | ✅ `fill="none" stroke="#000000" stroke-width≈1.5` (외곽선) |
| "다른 풀이" 텍스트 | 개별 글자 다/른/풀/이 | ✅ 존재(텍스트는 기존부터 렌더, 박스만 신규) |
| 11쪽 교차 확인 | 긴 path 0개 | ✅ PR 비고대로 11쪽은 본문 리터럴(도형 아님), 무관 |

- 산출물: `output/poc/pr1203/3-09월_교육_통합_2022_009.svg`(+_011.svg).
- 정답지: `pdf/3-09월_교육_통합_2022.pdf` 9쪽 (한글 2022).

## 6. 판단 — 머지 권고 (시각 판정 게이트)

진단 정확 + 파서 단일 파일 순수 추가 + 회귀 테스트 + 정량(외곽선 path 신규) + CI green.
충돌은 텍스트 인접 충돌로 두 테스트 보존하여 해소(의미 충돌 없음).
단, 외곽선 시각 영역이므로 **작업지시자 직접 시각 판정**(9쪽 "다른 풀이" 태그 박스 ↔ PDF)을 게이트로.
승인 + 시각 판정 통과 시 메인테이너 로컬 `--no-ff` 머지(충돌 해소 포함) + push.

## 7. 비고

- `<hp:seg>` CURVE 타입의 진짜 베지어 정밀화는 범위 외(점-대-점 폴리라인으로 시각 정합).
- @planet6897 #1208(수식 토큰) 동시 OPEN — 본 PR 과 독립(curve 파서 단일).
