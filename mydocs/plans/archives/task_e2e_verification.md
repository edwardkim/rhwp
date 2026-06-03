# 인라인 표 조판 자동 검증 체계 설계

## 목표

빈 문서에서 프로그래밍으로 문서를 생성하고, 렌더링 결과가 기대값과 일치하는지 **자동으로 검증**하는 전체 사이클 구현.

## 전체 사이클

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  1. 시나리오  │────▶│  2. 기대값    │────▶│  3. 실행     │────▶│  4. 비교     │
│  정의        │     │  생성        │     │  + 측정      │     │  + 판정      │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

### 1단계: 시나리오 정의

문서 구조를 선언적으로 기술한다. 한컴에서의 입력 순서를 그대로 반영.

```javascript
const scenario = {
  name: 'TC #20: 인라인 TAC 표 기본',
  steps: [
    { type: 'text', value: 'TC #20' },
    { type: 'enter' },
    { type: 'text', value: 'tacglkj 표 3 배치 시작   ' },
    { type: 'inlineTable', rows: 2, cols: 2, colWidths: [6777, 6777],
      cells: ['1', '2', '3 tacglkj', '4 tacglkj'] },
    { type: 'text', value: '   4 tacglkj 표 다음' },
    { type: 'enter' },
    { type: 'text', value: 'tacglkj 가나 옮' },
  ],
};
```

### 2단계: 기대값 생성

두 가지 방법으로 기대값을 확보한다:

**방법 A: 한컴 원본 기준** (골든 테스트)
- 동일한 내용의 HWP 파일을 한컴에서 작성
- 해당 파일을 로드하여 렌더 트리 좌표를 추출 → 기대값으로 저장
- 한컴의 조판 결과가 정답

**방법 B: 규칙 기반 계산**
- 시나리오에서 기대되는 배치 규칙을 명시
- 렌더 트리에서 규칙 충족 여부를 검증

```javascript
const expectations = {
  // 구조 검증
  pageCount: 1,
  paragraphs: [
    { index: 0, text: 'TC #20' },
    { index: 1, textContains: ['배치 시작', '표 다음'],
      controls: [{ type: 'Table', rows: 2, cols: 2, treatAsChar: true }] },
    { index: 2, text: 'tacglkj 가나 옮' },
  ],
  // 배치 규칙 검증
  layout: [
    // 규칙 1: 표가 텍스트 사이에 인라인 배치 (x좌표 순서)
    { rule: 'inline-order', paraIndex: 1,
      order: ['text:배치 시작', 'table:0', 'text:표 다음'] },
    // 규칙 2: 표 하단 ≈ 호스트 텍스트 베이스라인 + outer_margin_bottom
    { rule: 'table-baseline-align', paraIndex: 1, controlIndex: 0,
      tolerance: 2.0 }, // px
    // 규칙 3: 표 앞뒤 공백이 렌더링됨
    { rule: 'space-before-table', paraIndex: 1, controlIndex: 0,
      minGap: 5.0 }, // px (공백 3개 ≈ 10px)
    { rule: 'space-after-table', paraIndex: 1, controlIndex: 0,
      minGap: 5.0 },
    // 규칙 4: Enter 후 표 위치 불변
    { rule: 'stable-after-enter', paraIndex: 1,
      compareSteps: ['after-table', 'after-enter'] },
  ],
};
```

### 3단계: 실행 + 측정

시나리오를 E2E로 실행하고, 각 단계에서 렌더 트리 좌표를 수집한다.

```javascript
// 시나리오 실행기
async function executeScenario(page, scenario) {
  const snapshots = {}; // 단계별 렌더 트리 스냅샷

  for (const step of scenario.steps) {
    switch (step.type) {
      case 'text':
        // 키보드 입력
        break;
      case 'enter':
        // Enter 키
        break;
      case 'inlineTable':
        // createTableEx API + 셀 입력 + 커서 이동
        break;
    }
    // 각 단계 후 렌더 트리 스냅샷 저장
    snapshots[step.label] = await captureRenderTree(page);
  }
  return snapshots;
}

// 렌더 트리에서 검증에 필요한 좌표 추출
async function captureRenderTree(page) {
  return page.evaluate(() => {
    const tree = JSON.parse(window.__wasm.doc.getPageRenderTree(0));
    // 재귀적으로 Table, TextRun 노드의 bbox 수집
    return extractLayoutInfo(tree);
  });
}
```

### 4단계: 비교 + 판정

기대값과 실제 측정값을 비교하여 통과/실패를 판정한다.

```javascript
function verifyLayout(snapshots, expectations) {
  const results = [];

  for (const rule of expectations.layout) {
    switch (rule.rule) {
      case 'inline-order':
        // 표 앞 텍스트 maxX < 표 minX < 표 maxX < 뒤 텍스트 minX
        results.push(verifyInlineOrder(snapshots, rule));
        break;
      case 'table-baseline-align':
        // |표 하단 - (텍스트 baseline + om_bottom)| < tolerance
        results.push(verifyBaselineAlign(snapshots, rule));
        break;
      case 'space-before-table':
        // 표 시작 x - 앞 텍스트 끝 x > minGap
        results.push(verifySpaceGap(snapshots, rule, 'before'));
        break;
      case 'stable-after-enter':
        // 두 스냅샷 간 표 bbox 좌표 차이 < 1px
        results.push(verifyStability(snapshots, rule));
        break;
    }
  }

  return results;
}
```

## 확장 시나리오

이 체계가 구축되면 다음 시나리오들을 자동 검증할 수 있다:

| 시나리오 | 검증 항목 |
|---------|----------|
| 인라인 표 1개 (기본) | x순서, 세로 정렬, 공백 |
| 인라인 표 2개 연속 | 두 표 사이 간격, x순서 |
| 인라인 표 + 줄바꿈 | 표가 줄 넘어갈 때 다음 줄 배치 |
| 인라인 표 다양한 크기 | 큰 표/작은 표의 세로 정렬 |
| 인라인 표 + Enter 분할 | 표 포함 문단 분할 후 레이아웃 |
| 인라인 표 + 텍스트 삭제 | 표 앞뒤 텍스트 삭제 후 재배치 |
| 한컴 원본 비교 (골든) | 좌표 차이 < 허용 범위 |

## 구현 우선순위

1. **시나리오 실행기** — 선언적 시나리오 → E2E 실행
2. **렌더 트리 측정기** — getPageRenderTree → 검증용 좌표 추출
3. **규칙 검증기** — 기대값 규칙별 비교 로직
4. **골든 테스트** — 한컴 원본 로드 → 좌표 저장 → 비교
5. **보고서** — 통과/실패 + 좌표 차이 시각화
