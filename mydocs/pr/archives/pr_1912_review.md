# PR #1912 검토 — Issue #1898: tac 그림 문단 뒤 lazy vpos 재역산의 trailing_ls 이중 가산 수정

- 작성일: 2026-07-05 / 검토자: Claude (메인테이너 대행 검토)
- PR: planet6897 → devel / base 이후 devel 진행분과 **충돌 없음(MERGEABLE)**
- 연결 이슈: #1898 (기전 1) — 본문 `closes #1898`
- 컨트리뷰터 사이클: planet6897 누적 100건(머지 46 / 클로즈 50 / 오픈 4) — 베테랑, 첫 PR 아님

## 1. PR 요약

tac(글자처럼) 인라인 그림 문단 **뒤** 렌더 줄 전진이 layout·한컴 대비 line gap 1회분(+11.7px)
과대해지는 렌더-레이아웃 자기 불일치의 **하류 보정**. tac 그림 `PageItem::Shape`가 vpos base를
리셋한 직후, 다음 문단의 `HeightCursor::vpos_adjust` lazy 재역산이 spacing_before로 인코딩된
gap을 불연속으로 오판해 +trailing_ls bridge(880HU)를 이중 가산하는 것을, **직전 문단이
실텍스트 + tac Picture 호스트일 때 gap ≤ spacing_before를 연속으로 판정**해 차단한다.

변경: `src/renderer/height_cursor.rs` +26줄 (vpos_continuous 판정 1곳) +
신규 게이트 `tests/issue_1898_tac_image_line_advance.rs` (36388711 p9 세 전이 33.1±3px + 대조군).

## 2. 코드 검토

- 판정 한정이 잘 좁혀져 있다: `skip_spacing_before_prededuct`(HWP3-origin 사전 차감 생략
  경로) 제외 + `prev_has_text` + tac Picture 호스트 한정 — 일반 lazy 재역산의 bridge는 종전
  유지. 컨트리뷰터가 1차 blanket 접근의 반증(rowbreak 쪽나눔 2건 회귀)을 스스로 기록하고
  좁힌 과정이 본문에 남아 있다.
- sb의 px→HWPUNIT 환산(`* 7200 / dpi`)과 `.max(0)` 방어는 적절.
- 한계: `Control::Picture`만 검사 — tac **Shape/Equation** 호스트는 미커버
  (수식은 항상 tac인 프로젝트 특성상 좁음). 아래 #1919 관계 참조.

## 3. ⚠ #1919와의 관계 — 동일 결함의 상·하류 이중 수정 (supersede 의심)

동일 컨트리뷰터가 약 2시간 뒤 올린 **#1919가 같은 결함(#1898 기전 1, 44.8→33.1px)을 상류에서
수정**한다. 두 PR 모두 상호 참조·코멘트 없음.

| | #1912 (16:09) | #1919 (18:06) |
|---|---|---|
| 개입 지점 | **하류** — vpos_adjust 재역산에서 bridge 생략 | **상류** — tac 개체의 vpos 기준점 초기화 자체 면제 |
| 커버 개체 | Picture만 | **Picture/Shape/Equation** |
| 판별 정제 | rowbreak 반증으로 한정 | issue_1116 반례(tac-전용 문단)로 한정, 핀 13/13 유지 |
| 파일 | height_cursor.rs | layout.rs + paragraph_layout.rs(계측만) |
| 파일 겹침 | (없음 — 텍스트 충돌 0) | |

**기능 관계 — 실측으로 확정 (2026-07-05, supersede 아님·상호 보완)**: p9 SVG 글리프 y
시퀀스 실측 결과, 결함 전이 3곳(87→88, 95→96, 96→97) 중

| 빌드 | 87→88 | 95→96 | 96→97 |
|---|---|---|---|
| devel (before) | 44.8 | 44.8 | 44.8 |
| devel+#1912 | **33.0** | **33.1** | **33.0** |
| devel+#1919 | **44.8 잔존** | 33.1 | 33.0 |

- 87→88의 base 리셋 원인은 tac 그림이 아니라 **직전 표(성과지표)** — #1919의 "tac Shape
  초기화 면제"가 닿지 않고, 재역산 지점에서 막는 #1912만 잡는다. **#1919의 자기 핀
  (`issue_1898.rs`)은 95~97만 커버해 이 잔존을 검출하지 못한다.**
- 역으로 tac **Shape/Equation** 호스트가 일으키는 리셋은 Picture 한정인 #1912가 못 잡고
  #1919만 예방한다 (수식은 항상 tac인 프로젝트 특성상 실재 경로).
- 즉 **어느 쪽도 상위집합이 아님**: #1912 = "리셋 원인 무관, Picture 호스트 직후" /
  #1919 = "tac 개체 자신이 일으키는 리셋 예방, 3종 개체". 겹침은 Picture 자기-리셋
  사례뿐(무해 — #1919가 진입을 막아 #1912 가드는 불활성).

**검토자 권고 (정정)**: **둘 다 머지 (시간순 #1912 → #1919)** — 관측 결함(3전이)의 완전
해소는 #1912가 담당하고, #1919는 Shape/Equation 축 예방 + 계측 개선을 보탠다. 결합 상태
게이트(devel+4PR)로 검증. 겹침 가드는 vpos_adjust가 리팩토링 Phase 2d 해체 대상이므로
그 시점에 재설계로 흡수. 컨트리뷰터에게는 머지 코멘트로 87→88 실측(자기 핀 미커버)과
보완 관계를 공유한다.

## 4. 게이트 결과 (devel `bf5228df` + PR 테스트 머지, manifest `task_m100_1904_baseline_manifest.md` 준수)

| 게이트 | 결과 |
|---|---|
| GitHub CI | 전 체크 pass |
| cargo fmt --check | 통과 |
| cargo clippy --profile release-test --all-targets | 경고 0 |
| cargo test --profile release-test --tests | **2,870 통과 / 실패 0** (신규 게이트 포함) |
| OVR baseline 5샘플 (--no-hwp, ±2px) | **5/5 개체 회귀 0건** (baseline 00014ecf) |
| 36388711 p9 전진 실측 | 신규 테스트 3전이 33.1±3px + 대조군 26.4px 통과 (스위트 내) |

## 5. 시각 판정 자료

- 대상: 36388711 p9 불릿 리스트 구간, "사업 목표 달성도" 문단 앵커로 내용 정렬 크롭
  (오라클과의 같은 y밴드 비교는 기전 2 페이지 경계 드리프트로 불가 — 본 PR 범위 밖).
- 산출물 (`output/poc/pr1912/`):
  - `pr1912_p9_3way.png` — BEFORE(44.8px/줄, 리스트가 아래로 밀림) / AFTER(33.1px/줄) /
    ORACLE(32.9px/줄) side-by-side. AFTER의 수직 리듬·"9. 참고자료" 도달 위치가 오라클과 정합.
  - `pr1912_p9_after_vs_oracle_ovl.png` / `..._before_vs_oracle_ovl.png` — OVL 정합
    (R=오라클, G=B=rhwp; 검정=일치/빨강=rhwp만/청록=오라클만). after는 줄 단위 겹침 유지
    (잔여 프린지 = 폰트 메트릭 차이 + 줄피치 0.2px/줄 누적), before는 리스트 하부로 갈수록 발산.
- 오라클: `pdf/36388711_...-2024.pdf` — **Hwp 2024 편집기** 직접 출력(Creator: Hwp 2024
  13.0.0.3622). 정답지 규약(2020/2022)의 명시 등급 밖이므로 **보조 자료**로 취급.
  이슈 본문 실측(한글 2022, 32.9px/줄)이 수치 근거.
- 최종 시각 판정 권위는 작업지시자(한컴 편집기 환경)에 있음.

## 6. 판단 (작업지시자 승인 대기)

- 검증 결과 수정 자체는 건전. 단 **§3의 #1919 중복 관계로 merge/close 판단은 두 PR을
  함께 보고 결정할 사안** — 작업지시자 지시 대기.
