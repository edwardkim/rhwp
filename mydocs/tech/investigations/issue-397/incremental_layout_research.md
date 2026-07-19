---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-397/README.md
last_verified: 2026-07-17
---

# 프로덕션 문서 편집기의 증분 레이아웃(Incremental Layout) 아키텍처 조사

> 조사일: 2026-03-27
> 목적: WASM 기반 HWP 편집기에 적용할 증분 레이아웃 패턴 비교 분석

---

## 1. LibreOffice Writer

### 1.1 프레임 계층 구조 (SwFrame Hierarchy)

Writer의 레이아웃은 **SwFrame** 서브클래스로 구성된 트리 구조이다.

```
SwRootFrame
 └─ SwPageFrame (페이지)
     ├─ SwHeaderFrame / SwFooterFrame
     ├─ SwBodyFrame (본문 영역)
     │   └─ SwSectionFrame (구역)
     │       └─ SwColumnFrame (단)
     │           ├─ SwTextFrame (텍스트 문단) ← SwContentFrame 하위
     │           ├─ SwTabFrame (표)
     │           │   └─ SwRowFrame → SwCellFrame → SwTextFrame...
     │           └─ SwNoTextFrame (그림 등)
     └─ SwFlyFrame (플로팅 개체)
         └─ SwFlyInContentFrame (인라인 개체)
```

- 각 프레임은 **upper/lower/next/prev** 포인터로 연결
- **SwFlowFrame**: 페이지 경계를 넘는 프레임의 분할(flowing) 처리
- 프레임 간 관계를 따라가며 트리 순회 가능

### 1.2 무효화(Invalidation) 메커니즘

각 SwFrame은 세 가지 독립적인 유효성 플래그를 가진다:

| 플래그 | 메서드 | 의미 |
|--------|--------|------|
| `mbFrameAreaSizeValid` | `InvalidateSize_()` | 프레임 크기 재계산 필요 |
| `mbFrameAreaPositionValid` | `InvalidatePos_()` | 프레임 위치 재계산 필요 |
| `mbPrtAreaValid` | `InvalidatePrt_()` | 인쇄 영역(내부 여백 제외) 재계산 필요 |

- **InvalidatePage()**: 현재 프레임이 속한 페이지 전체를 무효화
- **InvalidateContent()**: 페이지의 `m_bInvalidContent` 플래그 설정 → 해당 페이지 콘텐츠 재레이아웃 대상으로 표시
- 문단 편집 시: 해당 SwTextFrame의 `InvalidateSize()` 호출 → 부모 프레임에 전파

### 1.3 2단계 레이아웃 (SwLayAction + SwLayIdle)

Writer는 편집 후 레이아웃을 **두 단계**로 수행한다:

**1단계: SwLayAction (동기 레이아웃)**
- `SwLayAction::InternalAction()`이 핵심
- **현재 화면에 보이는 페이지만** 즉시 재레이아웃
- 각 페이지의 `m_bInvalidContent` 플래그를 확인하여 무효화된 페이지만 처리
- 사용자에게 즉각적인 시각적 응답 제공

**2단계: SwLayIdle (비동기/유휴 레이아웃)**
- 화면 밖 페이지들을 **유휴 시간(idle time)**에 점진적으로 레이아웃
- 사용자 입력이 들어오면 즉시 중단
- 스크롤바 위치 등은 유휴 레이아웃 완료 후 정확해짐

**핵심 설계 원칙:**
- 키 입력마다 전체 문서를 재레이아웃하지 않음
- 화면 밖 내용은 "나중에 필요할 때" 계산
- Collabora Online에서는 타일 렌더링과 비동기 레이아웃을 조율하는 API 추가

### 1.4 표(Table) 재레이아웃

- SwTabFrame → SwRowFrame → SwCellFrame → SwTextFrame 계층
- 셀 내 텍스트 변경 시: SwTextFrame.InvalidateSize() → SwCellFrame → SwRowFrame → SwTabFrame으로 전파
- 표 전체 크기가 변하면 후속 콘텐츠의 위치도 무효화
- 표 크기 불변 시 표 이후 콘텐츠에 영향 없음

### 1.5 장점과 한계

| 장점 | 한계 |
|------|------|
| 성숙한 아키텍처 (20년+) | 코드 복잡도 매우 높음 |
| 세밀한 프레임별 무효화 | 전체 문서 무효화 시 동기 레이아웃이 느림 |
| 유휴 레이아웃으로 UI 반응성 유지 | 플로팅 개체 + 텍스트 흐름 상호작용이 복잡 |
| 페이지 단위 증분 처리 | 레이아웃 버그 디버깅 어려움 |

---

## 2. Typst (Rust 기반 조판 시스템)

### 2.1 전체 아키텍처

Typst의 컴파일 파이프라인:

```
소스 텍스트 → [파싱] → AST → [평가] → 콘텐츠 → [레이아웃] → 페이지들
```

- 각 단계가 독립적인 캐시 가능
- **레이아웃이 가장 비용이 큰 단계** → 캐싱 최적화의 핵심 대상

### 2.2 Comemo: 제약 기반 메모이제이션

Typst는 자체 개발한 **comemo** 라이브러리로 증분 계산을 구현한다.

**핵심 개념:**

```rust
#[memoize]     // 함수 결과를 캐시
#[track]       // impl 블록의 메서드 호출을 추적
```

**작동 원리:**

1. `#[memoize]` 함수 호출 시 캐시에서 호환되는 항목 검색
2. 캐시 항목은 결과값 + **제약 조건(constraints)** 으로 구성
3. `#[track]`된 인자의 메서드 호출이 제약 조건을 자동 생성
4. 새 호출의 인자가 기존 제약 조건을 **만족**하면 캐시 히트 → 재사용

**제약 조건 예시:**
- "이 페이지에 최소 4cm 남아 있는가?" (= 정확한 크기가 아닌 충분 조건)
- 인자가 정확히 같을 필요 없이 "동일하게 사용되면" 재사용 가능

### 2.3 레이아웃 캐싱 전략

**공간 제약(Spatial Constraints) 기반 캐싱:**

- 레이아웃 함수가 **region**(가용 공간)을 인자로 받음
- 전체 region을 비교하지 않고, 실제 관찰한 부분만 제약으로 기록
- 예: "너비가 500pt인가?" + "높이가 최소 200pt인가?" → 이 조건 만족하는 다른 region에서도 캐시 재사용

**요소(Element) 단위 캐싱:**
- 레이아웃 캐싱 단위는 **개별 요소(element)**
- 문단, 표, 그림 등 각 요소의 레이아웃 결과가 독립적으로 캐시됨
- 요소의 입력(콘텐츠 + 스타일 + 가용 공간)이 제약 조건을 만족하면 재사용

### 2.4 증분 파싱

- 소스 변경 시 영향받는 AST 노드만 재파싱
- 마크업 언어의 컨텍스트 민감성(off-side rule 등)을 수용하는 증분 파서
- 재귀 하강 파서에 적합한 설계

### 2.5 멀티스레딩

- 명시적 페이지 브레이크 경계에서 병렬 레이아웃 가능
- 일반적인 하드웨어에서 2~3배 속도 향상

### 2.6 Salsa 대신 Comemo를 선택한 이유

- Salsa: 쿼리 기반 증분 계산 (rustc, rust-analyzer 사용)
- Comemo: 제약 기반 메모이제이션 (더 세밀한 접근 추적)
- 초기에는 수작업 레이아웃 제약을 사용했으나 버그 빈발 → comemo로 자동화
- 레이아웃에서는 "입력이 정확히 같은가?"보다 **"입력이 동등하게 사용되는가?"** 가 더 적합

### 2.7 장점과 한계

| 장점 | 한계 |
|------|------|
| Rust 네이티브, WASM 호환 | 배치 컴파일러 모델 (실시간 편집기가 아님) |
| 세밀한 제약 기반 캐시 재사용 | 페이지 넘침(overflow) 시 캐시 미스 가능 |
| 자동화된 제약 추적 (comemo) | 대화형 편집의 지연 시간 최적화는 별도 필요 |
| 요소 단위 독립 캐싱 | 표처럼 상호 의존적 레이아웃은 캐시 효율 저하 |
| 증분 파싱 내장 | 현재 WYSIWYG 편집기가 아님 |

---

## 3. 웹 기반 편집기 (Google Docs / ProseMirror / Slate)

### 3.1 Google Docs (Canvas 렌더링)

**2021년 DOM → Canvas 전환의 이유:**
- 워드프로세서는 극도로 정밀한 레이아웃 요구사항이 있음
- DOM은 이러한 요구사항에 맞게 설계되지 않음
- 커서 위치의 텍스트 줄만 업데이트하는 "치트(cheat)" 기법을 DOM에서 구현하기 어려움
- Canvas 기반으로 전환하여 자체 레이아웃 엔진 운영

**증분 렌더링 접근:**
- 키 입력 시 **삽입점이 있는 줄(line)만** 즉시 다시 그림
- 나머지 영역은 필요할 때(스크롤 등) 업데이트
- 자체 레이아웃 엔진이 문단 단위로 캐시 관리 (추정)
- 플랫폼 독립적인 렌더링 보장

### 3.2 ProseMirror

**트랜잭션(Transaction) 기반 업데이트:**

```
사용자 입력 → Transaction 생성 → 새 State 계산 → View.updateState()
```

**DOM 업데이트 최적화:**
- 이전 문서와 새 문서를 비교하여 **변경되지 않은 노드의 DOM은 유지**
- `changedRanges`: 트랜잭션이 영향을 미친 범위를 추적
- 브라우저가 이미 적용한 DOM 변경(타이핑)은 ProseMirror가 다시 변경하지 않음
- **Decoration**: 효율적으로 비교/업데이트되는 영속적(persistent) 데이터 구조

**핵심 설계:**
- 불변(immutable) 문서 모델 → 이전/이후 비교 용이
- 노드 단위 비교: 변경된 노드만 DOM 업데이트
- 동기적(synchronous) 업데이트: 트랜잭션 적용 즉시 DOM 반영

### 3.3 Slate

- React 기반의 편집기 프레임워크
- Immutable 데이터 모델 사용
- React의 재조정(reconciliation) 메커니즘에 의존하여 변경된 노드만 리렌더링
- contenteditable 위에 구축

### 3.4 웹 편집기의 공통 패턴

| 패턴 | 설명 |
|------|------|
| 불변 문서 모델 | 이전/이후 스냅샷 비교로 변경 범위 파악 |
| 노드 단위 비교 | 변경되지 않은 노드는 리렌더링 생략 |
| 트랜잭션/연산 기반 | 변경 사항을 명시적 객체로 표현 |
| 레이아웃 위임 | 대부분 브라우저 레이아웃 엔진에 의존 (Google Docs 제외) |

---

## 4. xi-editor (Rust 기반 텍스트 편집기)

### 4.1 Rope 기반 아키텍처

xi-editor는 Raph Levien이 Google에서 개발한 Rust 기반 편집기로, 증분 처리의 원칙을 체계적으로 정리했다.

**핵심 철학:** "가능한 모든 처리를 증분적으로"
- 변경 사항을 **명시적 delta**로 표현
- delta가 렌더링 파이프라인을 통과하며 문서의 극히 일부만 영향

### 4.2 증분 줄바꿈 (Incremental Word Wrapping)

**문단 독립성 원칙:**
- 모든 문단(하드 브레이크로 구분)은 독립적으로 줄바꿈 계산 가능
- 변경된 문단만 재계산, 나머지는 캐시 재사용

**줄바꿈 결과 저장:**
- 줄바꿈 위치(breaks)를 B-tree rope 구조에 저장
- 문단 변경 시: 해당 문단의 줄바꿈 범위만 교체
- 전체 문서의 줄바꿈 목록을 재생성하지 않음

**글자 너비 측정 캐시:**
- 글자 너비 측정은 비용이 크므로 별도 캐시 운영
- 캐시 히트율이 성능에 결정적 영향

### 4.3 최소 무효화 (Minimal Invalidation)

- 캐시에 **프론티어(frontier)** 집합을 유지
- 유효한 캐시 항목이 프론티어에 없으면, 다음 항목도 유효함 (불변량)
- 변경 시 프론티어만 업데이트하여 무효화 범위를 최소화

### 4.4 장점과 한계

| 장점 | 한계 |
|------|------|
| 매우 큰 파일도 즉각 반응 | 페이지네이션 미지원 (코드 에디터) |
| 문단 단위 독립 캐싱 | 표, 플로팅 개체 등 복잡한 레이아웃 미지원 |
| delta 기반 전파로 최소 영향 | 프로젝트 중단됨 (2020) |
| Rust 네이티브 | WYSIWYG 문서 편집기가 아님 |

---

## 5. 패턴 비교 종합

### 5.1 무효화 단위(Granularity) 비교

| 시스템 | 무효화 단위 | 캐시 단위 | 페이지네이션 |
|--------|------------|----------|------------|
| LibreOffice | 프레임(문단/표/셀) | 없음 (재계산) | 페이지별 유휴 레이아웃 |
| Typst | 요소(element) | 요소별 제약 기반 | 페이지 경계에서 병렬화 |
| ProseMirror | 노드(node) | DOM 노드 재사용 | 해당 없음 |
| Google Docs | 줄(line)/문단 | Canvas 타일 (추정) | 자체 엔진 |
| xi-editor | 문단(paragraph) | B-tree rope | 해당 없음 |

### 5.2 증분 레이아웃 전략 비교

| 전략 | 사용처 | 원리 | rhwp 적용 가능성 |
|------|--------|------|-----------------|
| **프레임별 dirty 플래그** | LibreOffice | 각 프레임에 size/pos/prt 유효성 플래그. 무효화된 프레임만 재계산 | **높음** - 현재 rhwp의 문단/표 구조와 유사 |
| **유휴 레이아웃** | LibreOffice | 화면 밖 페이지는 유휴 시간에 처리 | **높음** - WASM에서 requestIdleCallback 활용 가능 |
| **제약 기반 메모이제이션** | Typst | 실제 사용된 입력 조건만 추적하여 캐시 재사용 극대화 | **중간** - 구현 복잡도 높으나 캐시 효율 우수 |
| **불변 문서 + diff** | ProseMirror | 이전/이후 상태 비교로 변경 범위 파악 | **중간** - 편집 모델에는 유용하나 레이아웃 캐시와는 별개 |
| **문단 독립 캐싱** | xi-editor | 문단 간 의존성 없이 독립적으로 줄바꿈 계산 | **높음** - HWP 문단은 대체로 독립적 |
| **delta 전파** | xi-editor | 변경을 명시적 delta로 표현하여 파이프라인 통과 | **높음** - 편집 연산을 delta로 표현하면 자연스럽게 적용 |

### 5.3 페이지네이션과 증분 레이아웃의 상호작용

이것이 가장 어려운 문제이다. 코드 에디터와 달리 HWP 편집기는 **페이지 단위 배치**가 필수적이다.

**문제 시나리오:**
1. 페이지 3의 문단에 텍스트 추가 → 문단 높이 증가
2. 페이지 3의 콘텐츠가 넘침 → 일부가 페이지 4로 이동
3. 페이지 4도 넘침 → 페이지 5로 이동... (cascade)
4. 최악의 경우 문서 끝까지 전파

**각 시스템의 해결 방식:**

| 시스템 | 해결 방식 |
|--------|----------|
| LibreOffice | 보이는 페이지만 즉시 처리 + 나머지는 유휴 레이아웃 |
| Typst | 제약 기반 캐시로 높이 불변 요소는 재사용. 페이지 경계 변경 시 해당 페이지부터 재레이아웃 |
| Google Docs | 자체 엔진이 줄 단위로 처리 (상세 미공개) |

**rhwp에 적용 가능한 전략:**
- 변경된 문단부터 **페이지 경계가 안정화될 때까지** 재레이아웃
- 페이지 N의 마지막 항목이 변경 전과 동일한 위치면 → 페이지 N+1 이후는 재사용
- "안정화 지점(stabilization point)" 감지가 핵심

### 5.4 표(Table) 편집 시 최소 재레이아웃

**전파 경로:**
```
셀 내 텍스트 변경
 → 셀 내 문단 재레이아웃
 → 셀 높이 변경?
    ├─ 아니오 → 종료 (셀 내부만 재렌더링)
    └─ 예 → 행 높이 재계산
        → 표 전체 높이 변경?
           ├─ 아니오 → 해당 행 이후 셀들만 위치 재계산
           └─ 예 → 표 이후 콘텐츠 위치 재계산
               → 페이지 넘침 확인
```

**최적화 포인트:**
- 셀 높이 불변 시: 셀 내부만 재레이아웃 (가장 흔한 경우)
- 행 높이 불변 시: 표 이후 콘텐츠 영향 없음
- 표 높이 변경 시: LibreOffice처럼 표 이후 프레임부터 무효화

---

## 6. rhwp WASM 편집기를 위한 권장 아키텍처

### 6.1 핵심 설계 원칙

1. **문단 단위 레이아웃 캐시** (xi-editor + Typst 방식)
   - 각 문단의 레이아웃 결과(줄 목록, 높이)를 캐시
   - 문단 내용이나 스타일 변경 시에만 해당 문단 재계산
   - 현재 rhwp의 `paragraph_layout` 결과를 캐시하면 자연스러움

2. **dirty 플래그 전파** (LibreOffice 방식)
   - 문단/표/셀 각각에 `layout_valid` 플래그
   - 편집 시: 해당 항목 무효화 → 필요한 만큼 부모로 전파
   - 재레이아웃 시: 무효화된 항목만 처리

3. **페이지네이션 안정화 지점 감지**
   - 변경된 문단이 속한 페이지부터 재페이지네이션
   - 각 페이지의 마지막 항목 위치를 이전과 비교
   - 동일하면 이후 페이지는 캐시 재사용

4. **유휴 레이아웃** (LibreOffice 방식)
   - 화면에 보이는 페이지만 즉시 레이아웃
   - 나머지는 `requestIdleCallback` 또는 `setTimeout`으로 비동기 처리
   - WASM 환경에서 메인 스레드 블로킹 방지

### 6.2 구현 단계 제안

**Phase 1: 문단 레이아웃 캐시**
- `ParagraphLayout` 결과에 해시 키 부여 (텍스트 + 스타일 + 가용 너비)
- 캐시 히트 시 줄바꿈/높이 계산 생략
- 가장 큰 성능 개선 예상

**Phase 2: dirty 플래그 기반 증분 페이지네이션**
- 편집된 문단에 dirty 표시
- 해당 문단부터 페이지 끝까지 재배치
- 페이지 경계 안정화 시 중단

**Phase 3: 표 셀 최적화**
- 셀 높이 불변 시 표 재레이아웃 생략
- 셀 내부 레이아웃과 표 레이아웃 분리

**Phase 4: 유휴 레이아웃**
- 화면 밖 페이지의 지연 레이아웃
- 스크롤바/페이지 카운트는 추정치 표시 후 점진적 교정

### 6.3 데이터 구조 설계 방향

```
PageLayoutCache {
    pages: Vec<CachedPage>,       // 페이지별 레이아웃 캐시
    dirty_from: Option<PageIdx>,  // 이 페이지부터 재레이아웃 필요
}

CachedPage {
    items: Vec<PageItem>,         // 문단/표 배치 결과
    valid: bool,                  // 유효성 플래그
    last_item_end_y: f64,         // 안정화 비교용
}

ParagraphLayoutCache {
    // 키: (paragraph_content_hash, para_shape_id, available_width)
    // 값: ParagraphLayout (줄 목록, 총 높이)
    cache: HashMap<ParagraphCacheKey, ParagraphLayout>,
}
```

### 6.4 WASM 환경 고려사항

| 고려사항 | 대응 |
|---------|------|
| 단일 스레드 | 유휴 레이아웃을 마이크로태스크로 분할 |
| 메모리 제한 | LRU 캐시로 메모리 사용량 제한 |
| JS-WASM 경계 비용 | 레이아웃 결과를 WASM 내부에서 최대한 처리 |
| 글자 너비 측정 | Canvas measureText 호출 캐시 (xi-editor 방식) |

---

## 7. 참고 자료

- [LibreOffice SwFrame Class Reference](https://docs.libreoffice.org/sw/html/classSwFrame.html)
- [LibreOffice SwRootFrame Class Reference](https://docs.libreoffice.org/sw/html/classSwRootFrame.html)
- [LibreOffice SwPageFrame Class Reference](https://docs.libreoffice.org/sw/html/classSwPageFrame.html)
- [LibreOffice Writer sw module](https://docs.libreoffice.org/sw.html)
- [Collabora Online Issue #9735 - Layout Invalidation](https://github.com/CollaboraOnline/online/issues/9735)
- [Typst comemo - Incremental computation through constrained memoization](https://github.com/typst/comemo)
- [What If LaTeX Had Instant Preview? (Comemo 블로그)](https://laurmaedje.github.io/posts/comemo/)
- [Fast Typesetting with Incremental Compilation (논문)](https://www.researchgate.net/publication/364622490_Fast_Typesetting_with_Incremental_Compilation)
- [Typst Architecture (docs/dev/architecture.md)](https://github.com/typst/typst/blob/main/docs/dev/architecture.md)
- [TeX and Typst: Layout Models](https://laurmaedje.github.io/posts/layout-models/)
- [Typst 0.12 Release (멀티스레딩, 캐싱 개선)](https://typst.app/blog/2024/typst-0.12/)
- [Why Typst uses comemo instead of salsa](https://forum.typst.app/t/why-does-typst-implements-its-own-incremental-computation-comemo-instead-of-using-salsa/4014)
- [ProseMirror Reference Manual](https://prosemirror.net/docs/ref/)
- [ProseMirror Guide](https://prosemirror.net/docs/guide/)
- [Why I rebuilt ProseMirror's renderer in React](https://smoores.dev/post/why_i_rebuilt_prosemirror_view/)
- [Google Docs Canvas Rendering Announcement](https://workspaceupdates.googleblog.com/2021/05/Google-Docs-Canvas-Based-Rendering-Update.html)
- [xi-editor Rope Science Part 5 - Incremental Word Wrapping](https://xi-editor.io/docs/rope_science_05.html)
- [xi-editor Rope Science Part 12 - Minimal Invalidation](https://abishov.com/xi-editor/docs/rope_science_12.html)
- [xi-editor Retrospective (Raph Levien)](https://raphlinus.github.io/xi/2020/06/27/xi-retrospective.html)
