---
kind: canonical
status: active
canonical: mydocs/tech/hancom_model/README.md
last_verified: 2026-07-19
---

# 한컴 레이아웃 형식 모델 (Hancom Formal-Model Index)

한컴 워드프로세서 레이아웃 시맨틱의 **형식 모델(formal model)** 권위 인덱스다. 이 모델은
**현재 증거에 대해 수학적으로 완결**되어 있다 — 알려진 모든 관측량과 레인 간 handoff 가
타입으로 정의되고, 아직 재현되지 않은 동작은 명시적 **부분 함수 / 반례(countermodel) /
동적 파라미터 / 증명 의무(proof obligation)** 로 남겨 둔다. 이것은 **완전한 네이티브
시맨틱은 아니다**: 모델이나 형식 정의가 완결됐다고 해서 해당 동작이 증명된 것은 아니다.

> 근거는 한컴 편집기 출력 PDF(정답지)와 라인세그 라운드트립 등 **관측 가능한 행동**이다.
> 각 레인 문서의 방정식·증거가 그 레인의 권위이며, 통합 모델은 이를 대체하지 않고 합성한다.

## 권위 문서 (Authorities)

| 범위 | 권위 문서 | 소유 내용 |
| --- | --- | --- |
| 레인 간 합성 | [통합 레이아웃 시맨틱](unified_layout_semantics.md) | 공유 상태, provenance, EP/OP/FL 인터페이스, 전이 시스템, 정리 의존성, projection 경계 |
| 종결점·라인 종결 | [엔드포인트 시맨틱](endpoint_line_closure_semantics.md) | 커서/스팬 대수, wrapper–selector–measure–scale–finalizer 관계, fit, descriptor commit, 종결점 반례 |
| 객체/컨트롤 배치 | [객체 배치 시맨틱](object_placement_semantics.md) | source span, 필터 해소, outward dispatch, 배치 클래스, 앵커/소유자 분리, placement provider 매핑 |
| 흐름 소유자·페이지네이션 | [흐름 시맨틱](flow_pagination_semantics.md) | 방향 지시자, 후속 선택, 프레임/페이지/소유자 합성, 표/머리꼬리/노트 관계, 페이지네이션 반례 |
| 라인 레이아웃 실측 | [라인 레이아웃 findings](line_layout_findings.md) | per-glyph advance·`.hft` 폭 보정, 누적 펜, 고정점 행렬, grid/원고지, 반각 셀 fit-render 분리, 라운드트립 정합률 |

## 남은 모델 파라미터 (Open/Dynamic Parameters)

아래는 정적으로 확정된 형식 구조 위에 남은 **동적 파라미터**(런타임 동작 관측을 요하는
미확정 값)다 — 모델의 구멍이 명시적으로 표시된 지점이다.

- **EP(엔드포인트):** 수용-run 셀렉터가 참조하는 dispatch 멤버십, 선택 receiver 클래스/유효성,
  수치 파이프라인 출력 단위, state-byte·모드·retry/fallback·구성 가능 집합의 행동 의미,
  생성기→descriptor 결합 수치 행 1건.
- **OP(객체 배치):** family 별 resolver 도메인·source span, 실제 outward/placement 호출,
  특수 반환 의미, numeric-class 의미, 앵커·배제·조각·중첩 소유자·페이지 소유자 효과.
- **FL(흐름):** 커밋 셀렉터가 선택하는 가상 타깃·상태 쓰기, 터미널 후속 정체성,
  섹션/프레임 생성, 머리꼬리 선택·예약, 표 조각화, 부동/노트 상호작용.
- **XL(교차):** 후속-폭 carry vs re-finalize vs replay, 소유자 admission 전 객체 배제,
  앵커/예약 통합, 정규 라인-로컬 재귀.

이 파라미터들은 인과 join·안정 정체성·재현으로 **관측**될 때에만 확정으로 승격된다. 정적
구조·동일 형태·시각 출력만으로는 대체되지 않는다.
