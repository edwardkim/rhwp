# 구현계획서 — Task #987

## 이슈

**#987** HWP3 쪽 테두리: `border_type=4` 이중선 매핑 + 위치 본문 기준 정합 (sample16)

브랜치: `local/task987`

## 사실 정리 (조사 완료)

### attr 비트 의미 (HWPX 인코딩 기준, `section.rs:553`)

| 비트 | 의미 |
|------|------|
| 0x01 | textBorder=PAPER (set=종이 기준, clear=본문 기준) |
| 0x02 | headerInside |
| 0x04 | footerInside |
| 0x08/0x10 | fillArea PAGE/BORDER |

→ **bit0 = 1 → paper, bit0 = 0 → body** 가 본래 스펙 의미.

### 현재 상태

- HWP3 파서 (`mod.rs:2818`): `attr: 1` (paper) 명시 설정
- HWP3 선 종류 (`mod.rs:2802`): `border_type=4` → `_ => Solid` fallback
- layout.rs (`:944`): `paper_based = true` **전역 하드코딩** — attr 무시
  - #952 에서 "모든 sample paper 정합" 으로 판정하여 attr 결정 로직 제거

### 충돌 지점

layout.rs 가 attr 을 무시하므로, HWP3 파서가 attr=0 (body) 으로 바꿔도
paper 로 렌더됨. attr 존중을 복원하면 #952 가 paper 로 맞춰둔
HWPX/HWP5 sample 이 회귀할 수 있음 (회귀 history 있는 민감 지점).

## 구현 단계 (4단계)

### Stage 1 — 선 종류 이중선 매핑 (격리 안전)

`src/parser/hwp3/mod.rs:2798-2803` `border_type` → `BorderLineType`:
- `4 => BorderLineType::Double` 추가
- 기존 `1=>Solid, 2=>Dash, 3=>Dot` 유지, `_ =>` 는 Solid 유지

HWP3 파서 단독 변경, 회귀 면 없음. 이중선만 우선 검증 가능.

**검증**: `cargo test` + sample16 export-svg 시각 (이중선 확인)

### Stage 2 — layout.rs paper/body 결정 로직 복원 (attr 존중)

`src/renderer/layout.rs:944`:
- `let paper_based = true;` → `let paper_based = (pbf.attr & 0x01) != 0;` 복원
- 즉 attr bit0 존중 (HWPX/HWP5 의 textBorder=PAPER 의미 그대로)
- 기존 디버그 출력 (`RHWP_DEBUG_PAGE_BORDER`) 유지

**리스크**: #952 가 전역 true 로 맞춰둔 HWPX/HWP5 sample 회귀 가능.
→ Stage 3 에서 HWP3 attr 을 body 로, Stage 4 에서 전 sample 회귀 검증.

### Stage 3 — HWP3 attr body 기준 설정

`src/parser/hwp3/mod.rs:2818`:
- `attr: 1` → `attr: 0` (sample16 한컴 정답지 = body 기준)
- CLAUDE.md "HWP3 전용 로직은 파서 안에서만" 준수 — layout.rs 는
  포맷 중립 attr 해석만, HWP3 고유 판단은 파서에서 attr 로 표현

**검증**: sample16 export-svg 시각 (body 기준 박스 + 이중선)

### Stage 4 — 전 sample 회귀 검증 + 보고

- `cargo test` 전체 (page border 회귀 테스트 포함)
- `cargo clippy -- -D warnings`
- HWPX/HWP5 시험지 sample export-svg 회귀 확인
  (#920/#952 에서 문제됐던 시험지 계열)
- 작업지시자 시각 판정 요청
- 회귀 발견 시: Stage 2 복원 범위를 HWP3 한정으로 축소하는 대안 검토
  (예: PageBorderFill 에 format-origin 힌트 필드 추가 — fallback 안)

## 회귀 시 대안 (Stage 4 에서 필요 시)

attr bit0 복원으로 HWPX/HWP5 가 회귀하면:
- layout.rs 에서 attr bit0 을 직접 쓰지 않고, HWP3 파서가 별도 신호
  (예: 사용 안 하는 attr 상위 비트 또는 신규 bool 필드) 로 body 의도 전달
- HWPX/HWP5 는 `paper_based=true` 유지, HWP3 만 body
- 이 경우 구현계획서 v2 작성 후 재승인

## 산출물

- Stage 별: `mydocs/working/task_m100_987_stage{N}.md`
- 최종: `mydocs/report/task_m100_987_report.md`

## 승인 요청

위 4단계 구현 방향으로 진행해도 될지 승인 요청드립니다.
Stage 2 가 회귀 history 민감 지점이라, Stage 4 회귀 검증을
필수 게이트로 두고 회귀 시 대안(HWP3 한정 격리)으로 전환하는 안전장치를 포함했습니다.
