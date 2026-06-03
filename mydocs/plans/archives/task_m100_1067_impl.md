# Task M100-1067 — 도형 회전 렌더링 (구현 계획서)

- 이슈: [#1067](https://github.com/edwardkim/rhwp/issues/1067)
- 마일스톤: v1.0.0 (M100)
- 브랜치: `local/task1067`
- 일시: 2026-05-22
- 수행 계획서: [`task_m100_1067.md`](task_m100_1067.md)

## 1. 본질 식별 (사전 정밀 분석)

### 1.1 Rust paint pipeline ✓

`src/paint/json.rs:596-622::PaintOp::Path` 가 이미 `transform` 필드를 JSON 에 emit:

```rust
PaintOp::Path { bbox, path } => {
    // ... commands, style, gradient, lineStyle ...
    buf.push_str(",\"transform\":");
    write_transform(buf, path.transform);
    buf.push('}');
}
```

`write_transform` (line 1792):
```rust
fn write_transform(buf: &mut String, transform: ShapeTransform) {
    let _ = write!(buf, "{{\"rotation\":{:.3},\"horzFlip\":{},\"vertFlip\":{}}}",
        transform.rotation, transform.horz_flip, transform.vert_flip);
}
```

→ Rust 측은 정상. JSON 으로 `{"rotation": ..., "horzFlip": ..., "vertFlip": ...}` 형식
보내고 있음.

### 1.2 rhwp-studio LayerPathOp ✗

`rhwp-studio/src/core/types.ts:857-863`:
```typescript
export interface LayerPathOp {
  type: 'path';
  bbox: LayerBounds;
  commands?: LayerPathCommand[];
  style?: LayerShapeStyle;
  lineStyle?: LayerLineStyle;
  // transform 필드 누락 ← 결함
}
```

→ TypeScript 인터페이스에 `transform` 필드 없음. JSON 의 transform 정보 무시됨.

### 1.3 canvaskit-renderer `renderPath` + `drawStyledPath` ✗

`rhwp-studio/src/view/canvaskit-renderer.ts:329-343`:
```typescript
private renderPath(canvas: SkCanvas, op: LayerPathOp): void {
    const path = new this.canvasKit.Path() as MutablePath;
    // ... commands 적용 ...
    this.drawStyledPath(canvas, path, style);  // ← canvas.save/rotate/restore 없음
}

private drawStyledPath(canvas: SkCanvas, path: Path, style: LayerShapeStyle): void {
    // fill/stroke 만 — 회전/flip 미적용
}
```

비교 — `renderTextRun` (line 410-416) 는 정확:
```typescript
const rotation = op.rotation ?? 0;
canvas.save();
if (rotation !== 0) {
    canvas.rotate(rotation, x, y);
}
canvas.drawText(op.text, x, y, paint, font);
canvas.restore();
```

→ 같은 패턴을 polygon path 에 적용해야 함. flip 도 `canvas.scale(-1, 1)` (h-flip) /
`canvas.scale(1, -1)` (v-flip) 으로 적용.

### 1.4 본질 확정

| 영역 | 상태 |
|------|------|
| Rust IR + JSON emit | ✓ 정확 |
| rhwp-studio `LayerPathOp` interface | ✗ `transform` 필드 누락 |
| rhwp-studio `renderPath` | ✗ transform 미사용 |
| rhwp-studio `drawStyledPath` | ✗ canvas.save/rotate/scale/restore 없음 |

## 2. 단계 구성 (3 단계)

### Stage 1 — LayerPathOp 인터페이스 + renderPath 정정

**파일**:
- `rhwp-studio/src/core/types.ts:857-863` — `LayerPathOp` 에 `transform?: { rotation?: number; horzFlip?: boolean; vertFlip?: boolean }` 필드 추가
- `rhwp-studio/src/view/canvaskit-renderer.ts:329` — `renderPath` 에서 transform 정보 추출 + canvas.save/rotate/scale/restore wrap

**구현 패턴** (renderTextRun 정합):
```typescript
private renderPath(canvas: SkCanvas, op: LayerPathOp): void {
    const path = new this.canvasKit.Path() as MutablePath;
    let currentX = op.bbox.x;
    let currentY = op.bbox.y;
    for (const command of op.commands ?? []) {
        [currentX, currentY] = this.applyPathCommand(path, command, currentX, currentY);
    }
    const style = op.style ?? { ... };
    const tr = op.transform;
    const needsTransform = tr && ((tr.rotation ?? 0) !== 0 || tr.horzFlip || tr.vertFlip);
    if (needsTransform) {
        const cx = op.bbox.x + (op.bbox.width ?? 0) / 2;
        const cy = op.bbox.y + (op.bbox.height ?? 0) / 2;
        canvas.save();
        if (tr.horzFlip || tr.vertFlip) {
            canvas.translate(cx, cy);
            canvas.scale(tr.horzFlip ? -1 : 1, tr.vertFlip ? -1 : 1);
            canvas.translate(-cx, -cy);
        }
        if ((tr.rotation ?? 0) !== 0) {
            canvas.rotate(tr.rotation, cx, cy);
        }
    }
    this.drawStyledPath(canvas, path, style);
    if (needsTransform) {
        canvas.restore();
    }
    path.delete?.();
}
```

회전 중심: bbox 의 center (CanvasKit 의 `rotate(degrees, x, y)` 는 (x,y) 기준 회전).
HWPX `<hp:rotationInfo centerX/Y>` 는 도형 로컬 좌표 기준 — 정확한 변환은 Stage 2 에서
다듬음 가능. 우선 bbox center 로 정상 동작 확인.

### Stage 2 — 회귀 가드 + 시각 판정

**테스트**:
- TypeScript 단위 테스트 (rhwp-studio e2e 또는 jest) — 회전 transform 적용 검증
- 또는 시각 회귀: shape-001 fixture 의 SVG snapshot 비교 (Rust 측)

**WASM 빌드 + 시각 판정**:
- Docker WASM 빌드 → rhwp-studio/public 동기화
- 작업지시자 한컴 한글 2020 시각 판정 + rhwp-studio 시각 판정

### Stage 3 — 최종 보고서 + 트러블슈팅 + commit/merge/push

## 3. 위험 분석

| 위험 | 영향 | 완화 |
|------|------|------|
| 회전 중심 좌표 부정확 | 도형 위치 어긋남 | bbox center 기본 사용 + HWPX centerX/Y 정밀 매핑은 Stage 2 보강 |
| flip 우선순위 + 회전 조합 순서 | 잘못된 행렬 | scale-then-rotate vs rotate-then-scale 순서 검증 |
| 다른 PaintOp (Image, Line, Rect) 의 transform 누락 회귀 | 별도 영역 | 본 task scope 외 — 별도 task 처리 |
| TypeScript interface 변경의 다른 영역 영향 | 컴파일 fail | 호환적 (`transform?: ...` optional) |

## 4. 산출물

- `rhwp-studio/src/core/types.ts` (LayerPathOp transform 필드)
- `rhwp-studio/src/view/canvaskit-renderer.ts` (renderPath transform 적용)
- 회귀 가드 (Stage 2)
- WASM 산출물 + rhwp-studio/public 동기화
- 단계별 보고서: `mydocs/working/task_m100_1067_stage{1..3}.md`
- 최종 보고서: `mydocs/report/task_m100_1067_report.md`
- 트러블슈팅 (필요 시)

## 5. 작업지시자 승인 요청

1. 본 구현 계획 (3 단계) 승인 여부
2. Stage 1 의 정정 패턴 (renderTextRun 정합) 권장 수용 여부
3. 회전 중심 좌표 — bbox center 기본 사용 후 정밀 매핑 Stage 2 보강 권장 수용 여부
