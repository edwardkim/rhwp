# PR #1039 검토 — HWPX slash/backSlash 형태 enum 파싱 분리 (closes #1038)

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #1039 |
| 제목 | HWPX slash/backSlash 형태 enum 파싱 분리 (#1038) |
| 작성자 | planet6897 (Jaeuk Ryu) — 누적 컨트리뷰터 (PR #221 OLE/Chart/EMF + PR #587 HWP 5.0 spec swap 등 굵직한 기여) |
| base ← head | `devel` ← `planet6897:task1038-hwpx-slash-diagonal` |
| 라벨 | bug |
| 연결 이슈 | `closes #1038` (OPEN, planet6897 본인 작성, **assignee 없음**) |
| mergeable | MERGEABLE / BEHIND (rebase 필요) |
| CI | Build & Test ✅ / Analyze rust·js·py ✅ / CodeQL ✅ / WASM skip / Canvas visual diff 없음 (parser-only) |
| 변경 | 6 파일 +345 / -29 — 소스 1 (`header.rs` +128/-29 포함 테스트 +93), 문서 5 |
| 본질 commit | **단일 commit `3d5c0ead`** |
| 생성 | 2026-05-20 14:52 |

## 2. 배경 (이슈 #1038)

`samples/2. 인공지능(AI) 기반 재정통합시스템 구축 용역 제안요청서.hwpx`
p4 헤딩 "Ⅰ 사업안내" (1×3 표, treat_as_char) 세 셀에 한컴 PDF 에는
없는 검정 대각선 (좌하→우상) 이 그려짐.

### Root cause

HWPX 의 셀 대각선은 **독립된 두 요소** 로 표현:

| 요소 | 의미 |
|------|------|
| `<hh:slash>` / `<hh:backSlash>` `type` | 대각선 **방향/형태** enum (NONE/CENTER/CENTER_BELOW/CENTER_ABOVE/...) |
| `<hh:diagonal>` `type/width/color` | **선 종류·굵기·색** (HWP5 DiagonalLine 1:1) |

세 셀의 borderFill (341/342/343) 은 `<hh:slash type="CENTER"/>` 만
있고 `<hh:diagonal>` 미보유. 한컴 의미상 "방향만 정의, 선 자체는 없음"
→ 대각선 미표시가 정답.

기존 파서 (`src/parser/hwpx/header.rs` slash/backSlash 핸들러) 가 slash
의 `type="CENTER"` 를 **선 종류 파서** `parse_border_line_type_code()`
에 넘김 → "CENTER" 는 선 종류 enum 에 없어 `_ => Solid(1)` 폴백 →
`diagonal_type=1` 설정 → 렌더 트리거 (`border_rendering.rs`,
`diagonal_type != 0`) ON → 기본 검정 실선 그려짐.

## 3. 변경 내용 (`src/parser/hwpx/header.rs` 단일 파일)

### 3.1 핵심 분리

**slash/backSlash 핸들러**:
- 기존: `type` → `parse_border_line_type_code()` → `set_diagonal_attr_bits` + `diagonal_type` 할당 + `width`/`color` 분기
- 신규: `type` → `parse_slash_shape_code()` → 방향 비트(attr) 만 설정. `diagonal_type`/`width`/`color` 분기 모두 제거.

**`<hh:diagonal>` 요소**: 선 종류·굵기·색 단독 책임 (기존 처리 보존).

### 3.2 신규 헬퍼 `parse_slash_shape_code`

```rust
NONE         → 0
CENTER       → 0b010
CENTER_BELOW → 0b011
CENTER_ABOVE → 0b110
기타/ALL     → 0b111
```

HWP5 spec 의 BORDER_FILL attr 비트 필드 (slash: bits 2~4, backSlash:
bits 5~7) 와 1:1 매핑. enum → 3비트 코드 변환.

### 3.3 `set_diagonal_attr_bits` 단순화

- 기존: `line_type != 0` 일 때만 고정 `0b010` 기록 (3비트 정보 → 1비트 축소)
- 신규: `code & 0x07` 그대로 기록 (방향 정보 완전 보존)

### 3.4 신규 테스트 4건

1. `test_parse_slash_shape_code` — enum 5종 (NONE/CENTER/CENTER_BELOW/CENTER_ABOVE/기타) 매핑
2. `test_set_diagonal_attr_bits` — 비트 설정·보존·클리어 단위
3. **`test_slash_center_without_diagonal_no_line`** — **#1038 회귀 가드** (slash CENTER + diagonal 없음 → `diagonal_type=0` 보장)
4. **`test_diagonal_element_sets_line_independent_of_slash`** — slash NONE + diagonal SOLID → 선 정상 (비회귀 보장)

## 4. 검토 항목

### 4.1 설계 적합성 — 메모리 룰 정합 ✅

- **`feedback_hancom_compat_specific_over_general`**: 방향/형태(slash) vs
  선 종류(diagonal) **책임 분리** = 구조 가드. 측정 의존 분기 없음.
  HWPX spec 의 두 요소 1:1 매핑.
- **`feedback_small_batch_release_strategy`**: 단일 commit + 단일 파일
  (소스) + +128/-29 (테스트 93 포함, 실 변경 +35/-29). 파서 책임
  분리에 한정.
- **scope 정직**: PR 본문 "렌더러/모델/HWP5·HWP3 경로 무수정" 명시.
  실제 변경 파일 검증 — `src/parser/hwpx/header.rs` 단일.
- **회귀 가드 동봉**: #1038 정합 테스트 + 비회귀 (`tac-img-02.hwpx`
  유사 시나리오) 테스트 2건.

### 4.2 코드 품질 ✅

- **주석 명료**: 핸들러 진입부 + helper 함수 모두 책임 + spec 매핑 명시
- **테스트 helper** (`slash_code`, `parse_single_border_fill`) 도입으로
  테스트 가독성 양호
- **3비트 정보 완전 보존** — 기존 1비트 축소 (`line_type != 0 ?
  0b010 : 0`) 의 정보 손실 해소. backSlash 분리 + CENTER_BELOW/ABOVE
  구분 가능
- 큰 지적 사항 없음

### 4.3 검증 충실성 — 작업지시자 시각 판정 게이트 ⚠️

PR body 검증 결과:
- cargo test 전체 통과 (실패 0) + 신규 단위/회귀 4건 ✅
- 문제 샘플 p4 헤딩: **검정 대각선 3→0** (정량 측정)
- `tac-img-02.hwpx`: 대각선 정상 유지 (비회귀)
- "한컴 PDF 정합" 주장 — 메모리 룰 `feedback_pdf_not_authoritative` 관련

본 PR 의 검증 근거는 **결정적 측정** ("3→0" + 회귀 가드 테스트) 이라
PR #950 같은 자가검증 패턴과 결이 다름. 그러나 메모리 룰
`feedback_visual_judgment_authority` 정합으로 **작업지시자 메인테이너
hands-on 점검 게이트** 는 일반적으로 필요. 본 환경에서 SVG 생성으로
정량 측정 보조 가능 (sample p4 헤딩 셀의 대각선 line 노드 개수
before/after 측정).

### 4.4 잔존 / scope 외

- 본 PR 은 borderFill XML 파싱 한정. 렌더러 (`border_rendering.rs` —
  `diagonal_type != 0` 트리거) 는 무수정. parser 가 정답을 주면
  렌더는 자동 정합.
- HWP5/HWP3 경로 무수정 — HWPX 만 영향. spec 차이 (HWP5 attr 비트가
  이미 정답을 담음) 로 인한 자연스러운 분리.
- 이슈 #1038 assignee 누락 — PR #1031 / PR #950 과 동일 패턴
  (본인 작성 + 본인 PR). 메모리 룰 `feedback_assign_issue_before_work`
  안내 후보, merge blocker 아님.

## 5. 처리 절차 (간소화 4단계)

1. ✅ PR 정보 확인 (본 문서 §1~2)
2. → 본 검토 문서 작성 + 작업지시자 승인 요청 (현 단계)
3. (불요 예상) 코드 품질 매우 양호, 본 PR 수정요청 항목 없음
4. 검증 (로컬 빌드/테스트 + 작업지시자 시각 판정) → `pr_1039_report.md`

## 6. 1차 판단 (작업지시자 승인 전 잠정)

| 영역 | 평가 |
|------|------|
| 설계 방향 | ✅ 적합 — 책임 분리 (방향/형태 vs 선 종류), 메모리 룰 정합 |
| CI / 결정적 검증 | ✅ 통과 (전부 pass + 신규 테스트 4건) |
| 코드 품질 | ✅ 양호 — 주석/테스트 helper/spec 매핑 명료. 지적 사항 없음 |
| scope | ✅ parser 단일 파일, 렌더/HWP5/HWP3 무수정 |
| 회귀 가드 | ✅ #1038 + 비회귀 테스트 2건 동봉 |
| 시각 검증 | ⚠️ 작업지시자 메인테이너 hands-on 점검 필요 (sample p4 헤딩 셀 대각선 0 확인) |
| 이슈 연결 | #1038 assignee 누락 (안내 후보, merge blocker 아님) |

**잠정 결론**: 코드·설계·결정적 검증 모두 양호. PR 본문이 매우
명료하고 책임 분리가 spec 정합. **머지 전 1개 게이트**: 작업지시자의
샘플 p4 헤딩 셀 시각 판정 (3→0 정량 측정 사실 확인). 본 환경에서
SVG 생성으로 정량 측정 보조 가능.

> 본 문서는 검토 계획 + 항목 통합. 작업지시자 승인/피드백 후
> 검증 단계 → `pr_1039_report.md` 로 최종 판단 기록.
