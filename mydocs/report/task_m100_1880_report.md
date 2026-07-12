# Task M100 #1880 최종 보고서 — convert-HWP 자리차지 표 host_before 비대칭 해소

- 이슈: #1880
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1880` (base: devel bf5228df)
- 계획서: `mydocs/plans/task_m100_1880.md` / `task_m100_1880_impl.md`
- 작성일: 2026-07-05

## 1. 결론

이슈 #1880(known-limitation 보류 상태)의 재착수 진입점(코멘트 3: 2780073 pi=4
`host_spacing.before` HWPX=6.7px/conv=0.0px)을 따라 조사하여, PI_MOVED 잔존
4건 중 **2건(2780073, 3075729)의 근본 원인을 확정·해소**했다. 이슈 본문의
재현 문서 3075729 는 convert-HWP 렌더가 한컴 oracle(p13)에 재정합되었다.

## 2. 근본 원인

`format_table`(typeset.rs)의 자리차지(비-TAC TopAndBottom) 판정이 **원시
`table.attr` 비트**((attr>>21)&7==1)를 읽었다:

- HWP5 파서(`parser/control.rs:153`): `table.attr` = 원시 attr 전체
- HWPX 파서(`parser/hwpx/section.rs:1831`): bit0 만 미러(의도된 설계)

→ 같은 IR 의 convert-HWP 재파스에서만 자리차지 표의 `host_spacing.before` 가
`spacing_before` 를 잃어(6.7↔0.0px) 표 이월(defer) 가드가 플립, razor-thin
경계에서 쪽 배치가 이동했다. `ir-diff` 는 의미 필드(wrap 열거형)만 비교하므로
이 원시값 차이가 0건으로 보였다 — 코멘트 3 의 "플래그 계열이 아닌 또 다른
구조 비대칭"의 정체.

## 3. 수정

`src/renderer/typeset.rs` `format_table` 1곳: 원시 비트 판정을
`!self.is_hwpx_source.get() && matches!(common.text_wrap, TopAndBottom)` 로 교체.

- native HWP5/HWP3: 비트⇔열거형 전단사(`shape.rs:394`)로 **불변**
- 순수 HWPX: 종전에도 attr=0 으로 분기 미진입 — **불변**
- convert-HWP(#1886 `is_hwpx_variant`): HWPX 렌더와 정합 (유일한 행동 변화,
  #1886 origin 전달의 연장)

`>>21` 원시 소비처는 렌더러 전체에서 이 1곳뿐(전수 확인).

## 4. 검증

**개별 문서** (릴리즈 빌드, 로컬 hwpdocs 코퍼스):

| 문서 | 수정 전 | 수정 후 |
|------|---------|---------|
| 3075729 (oracle p13) | conv p12 ✗ | conv **p13** ✓ |
| 2780073 | host_before 6.7↔0.0, defer 플립 | 트레이스 완전 일치 |
| 2776741 (phantom, oracle 1쪽) | 1쪽/1쪽 | **불변** (#1836 층위 무영향) |

**A/B 하니스** (2,005건 결정적 서브셋, 수정 전 바이너리 stash 재빌드 대조):

| 빌드 | SAME | PI_MOVED | PAGE_DELTA | ERR |
|------|------|----------|------------|-----|
| 수정 전 | 2000 | 4 | 1 | 0 |
| 수정 후 | **2002** | **2** | 1 | 0 |

개선 +2, **신규 divergence 0** (수정 후 비-SAME 은 수정 전의 엄밀한 부분집합).

**회귀 테스트**: `tests/issue_1880_takeplace_host_before.rs` — fixture 2건
(`samples/issue1880_takeplace_{host_before,oracle_p13}.hwpx`) 자기정합 +
oracle 고정 테스트, 2/2 통과.

**전체 스위트**: `cargo test --tests --no-fail-fast` — 스위트 193 통과 /
테스트 2,863 통과 / 신규 실패 0. 유일한 실패 스위트 svg_snapshot 5건은
수정 전 stash 상태에서도 동일 실패(로컬 CRLF 노이즈, #1886 보고와 동일 5건)
임을 A/B 로 확인. clippy 신규 경고 0.

## 5. 잔존 known-limitation (별개 클래스, 수정 전후 상세 동일)

- **2959953** PI_MOVED 5개 pi: 표 트레이스(pi≤23) 양 경로 동일 — 표 경로가
  아닌 문단 flow 층위의 다른 비대칭. 다음 진입점: pi=170(9→8) 인근 문단 흐름
  계측.
- **3171755** PI_MOVED 1개 pi (s0:pi213 20→21).
- **3235145** PAGE_DELTA(3→2): 기존재 별개 클래스.

## 6. 범위 외 관찰 (후속 이슈 후보)

1. **TAC bit0 미러 비대칭**: HWPX 는 `table.attr` bit0 = `treat_as_char &&
   flow_with_text`, HWP5 는 원시 bit0 — `attr & 0x01` 소비처(typeset 4곳,
   table_layout 1곳)에서 TAC+flow_with_text=false 조합 시 갈릴 수 있음.
   본 4건 트레이스는 전부 tac=false 로 무관.
2. **ir-diff 원시 attr 미비교**: 이번 클래스가 ir-diff 0 으로 보였던 원인.
   원시 `table.attr` (또는 재질화 규칙) 비교 추가 검토.

## 7. 산출물

- 커밋: `3010c1f8` (Stage 1: 수정+테스트), `c7d63f0f` (Stage 2 보고서)
- 하니스 TSV·트레이스: 세션 스크래치 `task1880/` (재현 명령은 stage2 보고서)
