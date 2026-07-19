---
kind: decision
status: historical
canonical: mydocs/tech/rendering_engine_design.md
last_verified: 2026-07-17
---

# 단일 패스 레이아웃 엔진 설계서

## 프로젝트 전환점

**벡터 드로잉 뷰어 → 워드프로세서 레이아웃 엔진**

rhwp는 HWP 파일을 SVG/HTML로 출력하는 벡터 드로잉 방식으로 출발했다.
이 방식은 "문서의 모양을 그리는 것"에 초점이 맞춰져 있어, 사전에 모든 요소의
높이를 측정하고, 페이지에 분배하고, 좌표를 계산하는 3단계 파이프라인을 사용했다.

그러나 이 접근은 근본적 한계에 도달했다:
- 사전 측정값과 실제 배치의 괴리로 overflow, 빈 페이지, 크롭 문제 반복
- 케이스별 패치가 다른 케이스를 깨뜨리는 악순환 (7일 이상)
- 표 > 셀 > 중첩 표 등 복잡한 구조에서 해결 불가

이제 MS Word / LibreOffice Writer와 동일한 **워드프로세서 방식**으로 전환한다.
"문서의 모양을 그리는 것"이 아니라, **"문서를 조판하는 것"**으로 패러다임을 바꾼다.

## 현재 아키텍처 vs 목표 아키텍처

### 현재: 벡터 드로잉 방식 (3단계 분리)

```
height_measurer → pagination engine → layout engine → SVG/HTML
  (사전 측정)       (페이지 분배)        (좌표 배치)     (출력)
```

- 각 단계가 독립적으로 동작
- 이전 단계의 결과를 다음 단계가 신뢰해야 함
- 피드백 루프 없음 → 오차 누적

### 목표: 워드프로세서 방식 (단일 패스)

```
layout engine (조판) → 렌더 트리 → SVG/HTML
  ├─ 요소 format (크기 계산)        (출력)
  ├─ 페이지에 들어가는가? (판단)
  ├─ 배치 확정 또는 다음 페이지 이동
  └─ 다음 페이지 높이 리셋
```

- 측정과 판단과 배치가 하나의 흐름
- 실제 크기로 판단하므로 괴리 없음
- overflow 발생 불가 — 안 들어가면 즉시 다음 페이지

## 핵심 알고리즘

### 원칙

1. **배치하면서 판단한다** — 사전 측정 없이, format() 시점에 크기를 알고 즉시 판단
2. **안 들어가면 넘긴다** — overflow 대신, 다음 페이지로 이동
3. **다음 페이지는 리셋한다** — 이전 페이지의 오차가 전파되지 않음
4. **모든 요소에 동일 규칙** — 문단, 표, 중첩 표, 이미지 모두 같은 패턴

### 기본 흐름

```
for each paragraph in section:
    frame = create_frame(paragraph)
    height = frame.format()  // 실제 크기 계산 (텍스트 줄, 표 높이 등)

    if page.remaining >= height:
        page.place(frame, height)
    else if frame.is_splittable():
        (master, follow) = frame.split(page.remaining)
        page.place(master)
        page = new_page()  // 높이 리셋
        // follow를 다음 페이지에서 계속 처리
        // follow가 다시 안 들어가면 또 분할 → 재귀적 처리
    else:
        page = new_page()  // 높이 리셋
        page.place(frame, height)
```

### 표 분할

```
table_frame.format():
    for each row in table:
        row_height = row.format()  // 셀 내용 포함 실제 높이

        if page.remaining >= row_height:
            page.place(row)
        else if row.is_splittable():
            // 셀 내용이 여러 줄 → 행 내부 분할
            (master_row, follow_row) = row.split(page.remaining)
            page.place(master_row)
            page = new_page()
            // follow_row + 나머지 행 계속
        else:
            page = new_page()
            page.place(row)
```

### 중첩 표

```
cell.format():
    total = 0
    for each content in cell.contents:
        if content is Table:
            h = content.format()  // 재귀 — 동일한 알고리즘
        else:
            h = content.format()  // 문단 높이
        total += h
    return total
```

중첩 깊이에 관계없이 동일 규칙. 사전 측정 불필요.

## 현재 코드 구조와의 관계

### 역할 재편

| 현재 모듈 | 현재 역할 | 전환 후 |
|-----------|----------|---------|
| `height_measurer.rs` | 사전 높이 측정 | **제거** — format()이 대체 |
| `pagination/engine.rs` | 페이지 분배 결정 | **제거** — layout이 직접 판단 |
| `pagination/state.rs` | 페이지 상태 관리 | **진화** → PageState |
| `layout.rs` | 좌표 배치 + 렌더 트리 | **확장** — 조판 엔진의 중심 |
| `composer.rs` | TextRun 구성 | **유지** — format()에서 호출 |
| `style_resolver.rs` | 스타일 해석 | **유지** |
| `render_tree.rs` | 렌더 트리 구조 | **유지** |
| `svg.rs` / `html.rs` | 출력 | **유지** |

### 인터페이스 변경

현재:
```rust
// 1단계: 측정
let measured = height_measurer.measure_section(&paragraphs);
// 2단계: 분배
let pagination = engine.paginate_with_measured(&paragraphs, &measured, ...);
// 3단계: 배치 (페이지별)
let tree = layout_engine.build_page_tree(page_num);
```

전환 후:
```rust
// 단일 호출: 조판 (모든 페이지 한 번에)
let result = layout_engine.typeset_section(&paragraphs, &page_def, &styles);
// result에 각 페이지의 렌더 트리가 포함됨
let tree = result.page_tree(page_num);
```

## 구현 전략

### 점진적 전환 (기존 테스트 보호)

기존 684개 테스트를 보호하면서 단계적으로 전환한다.

**Phase 1: 문단 조판**
- 비-표 문단의 format() + fits() + place() 구현
- 기존 pagination의 문단 처리와 동일 결과를 내는지 검증
- height_measurer의 문단 측정을 대체

**Phase 2: 표 조판**
- 표 format() + 행 단위 fits() + split() 구현
- 기존 split_table_rows, find_break_row를 대체
- 중첩 표의 재귀적 format() 처리

**Phase 3: 통합 및 정리**
- 기존 height_measurer, pagination engine 제거
- build_page_tree를 새 조판 결과에서 생성하도록 전환
- 자가 검증(LayoutOverflow) 0건 달성

### 각 Phase 검증 기준

1. 684개 기존 테스트 PASS
2. 자가 검증(LayoutOverflow) 0건
3. kps-ai.hwp, hwpp-001.hwp 등 주요 문서 시각적 정확도 유지
4. WASM 빌드 성공

## 기대 효과

1. **높이 계산 불일치 근본 해소** — 측정과 배치가 동일 시점에서 수행
2. **overflow 원천 차단** — 안 들어가면 넘기므로 overflow 자체가 발생하지 않음
3. **빈 페이지 문제 해소** — 실제 콘텐츠 기준으로 페이지를 구성
4. **코드 단순화** — 3단계 파이프라인 → 단일 조판 엔진
5. **향후 편집 기능의 기반** — 워드프로세서 수준의 레이아웃 엔진을 보유하게 됨
