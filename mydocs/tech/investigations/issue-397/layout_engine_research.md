---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-397/README.md
last_verified: 2026-07-17
---

# 레이아웃 엔진 설계 연구: MS Word, LibreOffice, Chromium LayoutNG, Typst

## 1. 연구 배경

### 1.1 현재 문제

rhwp의 기존 3단계 파이프라인에서 **측정-배치 불일치** 버그가 발생한다:

```
height_measurer.measure()  →  paginator.paginate()  →  layout.render()
     (측정)                      (배분)                   (배치)
```

- **pagination**은 측정된 높이로 "이 문단이 현재 페이지에 들어가는가"를 판단
- **layout**은 실제 spacing_before/after, host_spacing, line_spacing 등을 적용하며 배치
- 두 단계의 계산이 미묘하게 다르면 **문단이 페이지 경계를 넘어 크롭**됨

실제 사례 (k-water-rfp.hwp):
- page 14에서 pagination은 para 195가 들어간다고 판단 (current_h + para_h - trailing_ls ≤ available)
- layout에서 실제 배치 시 y_offset이 col_bottom을 14.2px 초과
- 원인: 표 문단(para 192, 194)의 host_spacing이 pagination 높이에 부정확하게 반영

### 1.2 근본 원인

**측정과 배치를 분리하면 반드시 불일치가 발생한다.**

이는 TeX의 "measure first, place later" 모델의 본질적 한계이며, MS Word, LibreOffice, Chromium 등 모든 주요 워드프로세서/레이아웃 엔진이 이 문제를 경험하고 해결한 문제다.

---

## 2. 선구자 엔진 분석

### 2.1 Chromium LayoutNG — Break Token 패턴

**가장 체계적이고 문서화가 잘 된 접근 방식.**

#### 핵심 아키텍처: Input/Output 튜플

```
Layout(BlockNode, ConstraintSpace) → (PhysicalFragment, Option<BreakToken>)
```

| 요소 | 역할 |
|------|------|
| BlockNode | 조판 대상 (스타일 + 자식 목록) |
| ConstraintSpace | 가용 공간, fragmentainer 높이, 현재 블록 오프셋 |
| PhysicalFragment | 배치 결과 (좌표, 자식 fragment) |
| BreakToken | 다음 fragmentainer에서 재개할 정보 |

#### Break Token 시스템

- `NGBlockBreakToken`에 레이아웃 재개에 필요한 모든 정보 저장
- Break token은 레이아웃 트리를 미러링하는 트리 구조
- 두 가지 분할 시나리오:
  - **내부 분할**: fragment + break token 반환
  - **이전 분할**: fragment 없음, 부모가 "break-before" 토큰 생성

#### 분할 기회 점수 (Break Opportunity Scoring)

CSS Fragmentation Level 3 기반 4단계 평가:

1. 모든 break 속성이 허용
2. 공통 조상에 break-inside: avoid 없음
3. orphan/widow 제약 충족
4. 모든 조상이 break-inside: auto

최적 분할점이 아닌 곳에서 공간이 소진되면 **2-pass 최적화**: 1차에서 최적 분할점 탐색, 2차에서 정확히 그 지점까지 레이아웃.

#### 표 분할 (Chrome 106에서 구현)

- 블록 분할과 동일한 Break Token 메커니즘 사용
- 행 → 셀 → 셀 내 블록 순서로 재귀적 분할

**참고**: https://developer.chrome.com/docs/chromium/renderingng-fragmentation

---

### 2.2 LibreOffice Writer — Master/Follow Chain

#### Frame 계층 구조

```
SwFrame (기본)
├── SwLayoutFrame (자식 포함)
│   ├── SwPageFrame (페이지)
│   ├── SwTabFrame (표) ← SwFlowFrame mixin
│   ├── SwRowFrame (행)
│   └── SwCellFrame (셀)
└── SwContentFrame (내용)
    └── SwTextFrame (텍스트) ← SwFlowFrame mixin
```

#### SwFlowFrame — 분할/이동 관리 Mixin

표와 문단 모두에 적용되는 핵심 인터페이스:

| 메서드 | 역할 |
|--------|------|
| `MoveFwd()` | 다음 페이지로 이동 |
| `MoveBwd()` | 이전 페이지로 이동 |
| `IsKeep()` | keep-together 제약 확인 |
| `CheckKeep()` | 문단/행 응집 강제 |
| `MoveSubTree()` | 프레임 서브트리 재배치 |

#### Master/Follow Chain

- 표가 페이지를 넘으면 **Follow** 프레임 생성
- Master ←→ Follow 양방향 링크 (`m_pFollow`, `m_pPrecede`)
- 머리행 반복: Follow 생성 시 Master의 머리행을 복제하여 삽입
- **FollowFlowLine**: Master와 Follow 표의 경계 행 관리

#### MakeAll() 루프

```
SwTabFrame::MakeAll() {
    loop {
        Format();          // 높이 계산
        if fits_on_page() { break; }
        split_or_move();   // 분할 또는 다음 페이지 이동
        nUnSplitted--;     // 진동 방지 (최대 5회)
    }
}
```

- **진동 방지**: 분할 → 재배치 → 다시 분할이 무한 반복되는 것을 방지
- `nUnSplitted = 5` 카운터로 제한

**참고**: https://wiki.openoffice.org/wiki/Writer/Core_And_Layout

---

### 2.3 MS Word / OOXML — 행 단위 제어

#### 핵심 속성

| 속성 | 효과 |
|------|------|
| `cantSplit` | 행 분할 금지 → 행 전체를 다음 페이지로 |
| `tblHeader` | 머리행 반복 (자동 페이지 나눔에서만 작동) |
| `trHeight` + `hRule` | 행 높이 제어 (exact/atLeast/auto) |

#### 행 분할 규칙

1. **cantSplit = false** (기본값): 셀 내 문단을 줄 단위로 분할 가능
2. **cantSplit = true**: 행 전체가 한 페이지에 있어야 함. 안 맞으면 다음 페이지로 이동
3. 셀 내용이 한 페이지를 넘으면: cantSplit 무시하고 강제 분할 (monolithic overflow 방지)

#### 머리행 반복

- `tblHeader` 설정된 행은 각 페이지 상단에 자동 반복
- 연속된 첫 n행만 머리행 가능 (중간 행은 불가)
- 수동 페이지 나눔 시에는 반복하지 않음 (표를 두 개로 분리)

---

### 2.4 Typst — Place-First 모델

#### 핵심 개념: Regions

```
fn layout(content, regions) → Vec<Fragment>
```

- **Region**: 요소가 배치될 수 있는 공간 형태
- 콘텐츠를 먼저 실체화(realize)한 후, 배치하면서 동적으로 region 조정
- **TeX과의 결정적 차이**: TeX는 표를 먼저 완성한 후 페이지 나눔 결정 → 셀 내부 분할 불가

#### TeX vs Typst 비교

| 측면 | TeX (Measure First) | Typst (Place First) |
|------|---------------------|---------------------|
| 표 셀 페이지 분할 | 불가 | 가능 |
| orphan/widow 제어 | 우수 | 현재 제한적 |
| 레이아웃 최적화 | 전역 최적화 가능 | 지역(greedy) 최적화 |

**참고**: https://laurmaedje.github.io/posts/layout-models/

---

## 3. 공통 설계 원칙

### 3.1 단일 패스 조판 (Format While Placing)

모든 현대 엔진이 수렴한 핵심 원칙:

> **측정과 배치를 분리하지 않는다. 배치하면서 측정한다.**

```
// 안 좋은 패턴 (현재 rhwp)
let height = measure(paragraph);
let fits = height <= available;
place(paragraph, y);  // 실제 높이가 height와 다를 수 있음

// 좋은 패턴 (MS Word, LibreOffice, Chromium)
let (fragment, break_token) = format_and_place(paragraph, available_space);
// fragment.height는 실제 배치된 높이와 정확히 일치
```

### 3.2 Constraint Space → Fragment + BreakToken

```
layout(node, constraint_space) → (fragment, Option<break_token>)
```

- **constraint_space**: 가용 높이, 너비, 현재 오프셋
- **fragment**: 실제 배치 결과
- **break_token**: 다음 페이지에서 재개할 정보 (없으면 완전 배치)

### 3.3 분할 시 정보 보존

BreakToken에 포함되어야 할 정보:

| 대상 | 보존 정보 |
|------|----------|
| 문단 | 시작 줄 번호, spacing_before 억제 여부 |
| 표 | 시작 행 번호, 머리행 반복 여부 |
| 행 (인트라-로우) | 각 셀의 시작 줄 번호 |
| 중첩 표 | 재귀적 break token 트리 |

### 3.4 Spacing 상호작용 규칙

1. **페이지 상단 spacing_before 억제**: 페이지/fragmentainer 첫 요소의 spacing_before는 0
2. **trailing line_spacing**: 페이지 마지막 문단의 trailing line_spacing은 무시
3. **표 host_spacing**: 표를 소유한 문단의 spacing_before/after는 표 전후에 적용
4. **셀 내 spacing**: 셀 padding 이후 시작, 셀 경계에서 종료

---

## 4. rhwp TypesetEngine Phase 2 적용 방안

### 4.1 Break Token 도입

```rust
/// 조판 분할 지점 — 다음 페이지에서 재개할 정보
enum TypesetBreakToken {
    /// 문단 줄 분할
    Paragraph {
        para_index: usize,
        start_line: usize,
        suppress_spacing_before: bool,
    },
    /// 표 행 분할
    Table {
        para_index: usize,
        control_index: usize,
        start_row: usize,
        header_rows: Vec<usize>,    // 반복할 머리행 인덱스
        cell_breaks: Option<Vec<CellBreakToken>>,  // 인트라-로우 분할 시
    },
}

/// 셀 내부 분할 정보
struct CellBreakToken {
    cell_index: usize,
    start_line: usize,  // 셀 내 문단의 시작 줄
}
```

### 4.2 표 조판 흐름

```rust
fn typeset_table(
    &self,
    table: &Table,
    constraint: ConstraintSpace,
    break_token: Option<&TypesetBreakToken>,
) -> (TableFragment, Option<TypesetBreakToken>) {
    // 1. 머리행 배치 (break_token에서 header_rows 참조)
    // 2. 시작 행 결정 (break_token.start_row 또는 0)
    // 3. 각 행에 대해:
    //    a. 행 높이 계산 (셀 내용 format)
    //    b. 가용 공간에 들어가는가?
    //       - YES: 행 배치, 커서 전진
    //       - NO + 분할 가능: 인트라-로우 분할, BreakToken 반환
    //       - NO + 분할 불가: 행 전체를 다음 페이지로, BreakToken 반환
    //       - NO + 아무것도 없음: monolithic overflow (강제 배치)
    // 4. 모든 행 배치 완료: (fragment, None) 반환
}
```

### 4.3 기존 Paginator와의 호환

Phase 2 완료 시:
- TypesetEngine이 표 포함 구역도 정확히 처리
- TYPESET_VERIFY에서 표 구역 차이 0건 목표
- Phase 3에서 Paginator 표 로직 제거, TypesetEngine이 유일한 경로

---

## 5. 참고 자료

| 출처 | URL |
|------|-----|
| Chromium LayoutNG Block Fragmentation | https://developer.chrome.com/docs/chromium/renderingng-fragmentation |
| CSS Fragmentation Level 3 | https://www.w3.org/TR/css-break-3/ |
| LibreOffice Writer Core And Layout | https://wiki.openoffice.org/wiki/Writer/Core_And_Layout |
| LibreOffice New Table Model | https://wiki.openoffice.org/wiki/Writer/New_Table_Model |
| OOXML Table Row Properties | http://officeopenxml.com/WPtableRowProperties.php |
| Typst Layout Models | https://laurmaedje.github.io/posts/layout-models/ |
| Univer Typesetting Design | https://docs.univer.ai/blog/doc-typesetting-design |
