# rhwp 리팩토링 마스터 플랜 (2026, 0.8/v1.0) — 초안 v1.1

- 이슈: #1883 (계획 수립) / umbrella: #1582 / 작성일: 2026-07-04 (v1.1: 3단계 원칙 명시)
- 상태: **초안 — 컬래버레이터(@jangster77, @postmelee) 리뷰 대기**
- 거버넌스: **SOLID + 복잡도** (복잡도 공식 측정 = 코드 품질 대시보드 `scripts/metrics.sh`,
  임계값: 파일 1,200줄 / CC 목표 ≤15·경고 >25. SOLID 채점: `manual/solid_scoring_guide.md`)

## 0. 대원칙 — 한 번에 하지 않는다: 리팩토링 3단계 (작업지시자 확정)

선행 리팩토링 경험(3회 모두 분할 진행: Task 146 → Task 149 → 4차 후속 정리)에 근거해,
**빅뱅 리팩토링 대신 3개의 순차 리팩토링 단계**로 나눈다. 본 계획의 실행부가 그 3단계다:

| 리팩토링 단계 | = 본 계획 Phase | 성격 |
|---|---|---|
| **1차** | Phase 1 (Provenance/Profile) | 저위험 — behavior 불변 소유권 분리 |
| **2차** | Phase 2 (복잡도 해체) | 중위험 — 함수 단위 롤링 |
| **3차** | Phase 3 (Document 축 분리) | 고위험 — roundtrip 영향 |

**단계-관문(stage-gate) 규칙:**
1. 각 단계는 **완료 → 게이트 전수 통과 → 대시보드 스냅샷(`--snapshot`) → 중간 재평가 →
   작업지시자 승인** 후에만 다음 단계에 착수한다. 단계 간 병행·중첩 금지.
2. 각 단계는 **독립적으로 종료 가능한 지점**이어야 한다 — 다음 단계를 하지 않아도 그
   자체로 devel 이 안정 상태(필요 시 릴리즈 가능).
3. 중간 재평가에서 위험이 크다고 판단되면 **다음 단계를 축소·연기·중단**할 수 있다 —
   3차(Document 축)는 특히 2차 결과를 보고 범위를 재결정한다.
4. Phase 0(baseline freeze)은 1차의 전제 조건, Phase 4(6차 리뷰)는 3단계 전체의 마감이다.

## 1. 현황 재진단 요약 (상세: `mydocs/tech/task_m100_1883_diagnosis.md`)

- 마지막 리팩토링(4차 리뷰, 2026-03-23) 이후 3.5개월간 **src ×2.7 성장** (133k→356k줄).
- 대시보드(2026-07-04): 1,200줄 초과 .rs **70개**, **CC>25 함수 80개**(최대 288), clippy 0,
  테스트 2,820 pass — **동작은 건강하나 구조가 비대**.
- 최우선 핫스팟(두 측정 교차 검증): `typeset_section_with_variant`(7,059줄·CC 282),
  `layout_composed_paragraph`(3,771줄·CC 288).
- #1582 감사 지적(Document 이질 축 필드, DocumentCore 표면 277 pub fn, lib.rs 표면) 전부 실측 일치.
- ③차 정리 지점(`layout_column_item` 827→505줄)은 개선 유지 — 단계적 정리의 유효성 전례.

## 2. 범위와 단계 (위험도 오름차순)

### Phase 0 — Baseline Freeze (실행 관문)
행동 고정 자산을 스냅샷·게이트화한다. 코드 변경 없음.
- public Rust API 표면 목록화(`lib.rs` 18 pub mod 기준) + WASM JSON schema 스냅샷
- CLI output 계약(39개 명령 대표 출력) 고정
- 대표 샘플 행동 고정: HWP5/HWPX/HWP3/embedded media — 페이지 오라클·golden SVG·
  roundtrip baseline·**OVR baseline**(개체 geometry) 일괄 저장
- 대시보드 지표 스냅샷(본 진단) = 개선 측정의 영점

### Phase 1 — SourceProvenance + LayoutCompatibilityProfile (SOLID, #1582 합의 1단계)
- `is_hwp3_variant`/`is_hwpx_source` 계열 판단의 **소유권만** 내부 policy 로 분리.
  기존 API 는 shim 유지 — **observable behavior 불변**.
- 효과: layout/typeset 의 소스-포맷 분기가 profile 조회로 수렴 → Phase 2 해체의 전제.
- 게이트: 전체 테스트 + roundtrip/golden/OVR 무변동 (bit-identical 기대).

### Phase 2 — 복잡도 해체 (대시보드 빨간불 해소)
Phase 1 완료 후, CC·크기 상위부터 단계 해체. 각 함수는 **①추출 대상 식별 → ②순수 함수
추출(동작 불변) → ③게이트 통과** 사이클로 1 PR = 1 함수(또는 1 클러스터).
- 2a: `typeset_section_with_variant` (7,059·CC 282) — 배치 단계별(분할/앵커/flow) 추출
- 2b: `layout_composed_paragraph` (3,771·CC 288)
- 2c: 표 계열 4개 (`layout_table_cells`/`layout_partial_table`/`typeset_block_table`/
  `build_single_column`, 각 1,2~1,6천줄)
- 2d: `parse_paragraph_list`(HWP3)·`vpos_adjust`·`paginate_with_measured_opts`
- 게이트: 함수별 PR 마다 전체 테스트 + 페이지 오라클 + OVR + (시각 영향 시) 시각 판정
  ([visual_verification_governance](../manual/visual_verification_governance.md) 준수).

### Phase 3 — Document 축 분리 (고위험, 후순위)
- `PackagePreservation`(extra_streams/hwpx_aux_entries), `EmbeddedMediaProjection`,
  `VisualScene` 분리 — roundtrip 회귀 위험이 커서 Phase 1·2 안정화 후 별도 판단.
- `DocumentCore` session aggregate 축소(277 pub fn → 도메인/세션 분리)도 이 단계.

### Phase 4 — 재평가 (6차 코드 리뷰)
- SOLID 5원칙 점수표 + 대시보드 재측정으로 성과 검증. 잔여 부채 목록화 → 차기 사이클.

## 3. Baseline Freeze 항목 (Phase 0 상세)

| 자산 | 고정 방법 |
|---|---|
| public Rust API | `lib.rs`/각 pub mod 표면 목록 문서화 + cargo-public-api 검토(도입 여부 리뷰 안건) |
| WASM JSON | wasm_api 반환 JSON 스키마 스냅샷 테스트 |
| CLI output | 대표 명령 출력 golden (기존 svg_snapshot 패턴 확장) |
| 렌더 행동 | golden SVG(8) + 페이지 오라클 + hwpx/hwp5 roundtrip baseline + OVR baseline + fidelity 하니스 TSV |

## 4. Feature Freeze (작업지시자 결정 사항)

- **전면 장기 freeze 는 비현실적** (현 PR 유입: 일 수십 커밋). 제안: **Phase 단위 짧은 freeze** —
  Phase 1(1~2주 예상)과 Phase 2 의 각 함수 해체 PR 구간만 해당 영역(renderer/layout·typeset)
  freeze, 그 외 영역(확장·studio·도구·문서)은 정상 유입.
- freeze 공지: CONTRIBUTING 공지 + 해당 기간 렌더링 PR 은 hold 라벨.
- 대안(전면 freeze, 릴리즈 직후 집중 실행)과의 선택은 리뷰 안건.

## 5. 회귀 게이트 전략

[visual_verification_governance.md](../manual/visual_verification_governance.md) 를 그대로
적용한다. Phase 별 필수 게이트:

| Phase | 게이트 |
|---|---|
| 0 | (자산 생성 자체) |
| 1 | 전체 테스트 + roundtrip/golden/OVR **무변동** (bit-identical 기대) |
| 2 | 함수별 PR: 전체 테스트 + 페이지 오라클 + OVR + 시각 판정(시각 영향 시) |
| 3 | 위 전부 + roundtrip fidelity 하니스 + big-corpus render-diff(컨트리뷰터 협조) |

## 6. 성공 기준 (정량 — 리뷰에서 목표치 조정 가능)

| 지표 | 현재 (영점) | Phase 2 완료 목표 | v1.0 목표 |
|---|---|---|---|
| CC>25 함수 | 80개 (최대 288) | 상위 10 해소, **최대 CC < 100** | **0개** |
| 1,200줄 초과 .rs | 70개 | 로직 파일 상위 7 해소 | ≤ 20개 (데이터/테스트 제외 0) |
| SOLID (6차 리뷰) | 4차 8.9 | — | **≥ 9.0** |
| 행동 회귀 | — | 전 Phase **0건** (게이트 강제) | 0건 |

## 7. 일정 (0.8/v1.0 결합)

- **0.8 릴리즈 전**: Phase 0(freeze 자산) — 릴리즈 검증 자산과 겸용이라 선행 이득.
- **0.8 릴리즈 직후**: Phase 1 (저위험, 1~2주) → Phase 2 를 함수 단위로 롤링(각 PR 독립 머지,
  중단 가능 지점 유지).
- **v1.0 전**: Phase 3 판단 + Phase 4(6차 리뷰).
- 구체 일자는 0.8 릴리즈 일정 확정 후 리뷰에서 조정.

## 8. 리뷰 요청 안건 (컬래버레이터)

1. Phase 순서·분할 타당성 (특히 Phase 1 → 2 의존 관계)
2. Feature freeze 방식: Phase 단위 영역 freeze vs 전면 freeze
3. 성공 기준 목표치(§6) 적정성
4. cargo-public-api 등 API 표면 고정 도구 도입 여부
5. Phase 2 함수 해체의 PR 단위(1 함수 vs 1 클러스터)와 담당 분배
