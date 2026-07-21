# PR #????: hwpctl SaveAs 성공 시 dirty 상태 정리 — onSave 콜백 추가

## 이슈
- **Issue**: #2661 — rhwp-studio: hwpctl SaveAs()가 dirty 상태를 정리하지 않음

## 분석

`rhwp-studio/src/hwpctl/index.ts`의 `HwpCtrl.SaveAs()`는 blob 다운로드 방식 내보내기를 수행한 후 `documentState.markClean()`을 호출하지 않는다. 이로 인해 SaveAs 성공 후에도 편집 창의 dirty 상태와 자동 백업 draft가 정리되지 않는다.

### 문제의 원인

`HwpCtrl`은 `wasmDoc`만 주입받는 독립 객체로 설계되어 있다. 내부 저장 경로(`file.ts`)는 `services.documentState.markClean('save-as')`를 호출하지만, HwpCtrl은 `documentState`나 `eventBus`에 접근할 방법이 없다.

이슈 본문에서 언급된 대로, 생성자 시그니처를 변경하거나 콜백을 주입하는 방식이 필요하다.

### 해결 방안

세 가지 접근법을 고려했다:

1. **`documentState` 직접 주입**: HwpCtrl이 `DocumentDirtyState`를 직접 알게 하는 방법. 그러나 HwpCtrl의 독립성과 테스트 용이성이 손상됨.
2. **커스텀 이벤트 발생**: SaveAs 성공 시 DOM 커스텀 이벤트를 발생시키는 방법. 그러나 전역 이벤트는 디버깅이 어렵고 타입 안전성이 낮음.
3. **✅ 선택적 콜백 주입 (채택)**: 생성자에 선택적 `onSave: SaveCallback` 파라미터를 추가. HwpCtrl의 독립성을 유지하면서 호출자가 후처리를 원하는 대로 구성 가능.

### 선택한 방식의 장점

- **하위 호환성**: `onSave`는 선택적(optional) 파라미터로, 기존 `new HwpCtrl(wasmDoc)` 호출은 전혀 영향 없음
- **단일 책임**: HwpCtrl은 저장 자체만 담당하고, dirty 상태 관리는 호출자에게 위임
- **테스트 용이성**: 콜백 모킹이 간단함

## 변경

```typescript
// before
export class HwpCtrl {
  wasmDoc: any;
  constructor(wasmDoc: any) {
    this.wasmDoc = wasmDoc;
  }
  SaveAs(filename: string): boolean {
    // ... blob 생성 및 다운로드 ...
    return true;
  }
}

// after
export type SaveCallback = () => void;

export class HwpCtrl {
  wasmDoc: any;
  private onSave: SaveCallback | undefined;

  constructor(wasmDoc: any, onSave?: SaveCallback) {
    this.wasmDoc = wasmDoc;
    this.onSave = onSave;  // 선택적 콜백 저장
  }

  SaveAs(filename: string): boolean {
    // ... blob 생성 및 다운로드 ...
    this.onSave?.();  // 저장 성공 시 콜백 호출
    return true;
  }
}
```

`createHwpCtrl` 팩토리 함수도 확장:

```typescript
export async function createHwpCtrl(options: {
  wasmUrl?: string;
  wasmModule?: any;
  onSave?: SaveCallback;  // 신규
}): Promise<HwpCtrl> {
  // ...
  return new HwpCtrl(wasmDoc, options.onSave);
}
```

## 사용 예

호출자(embed runtime 등)에서 `onSave`로 `documentState.markClean()`을 연결:

```typescript
const ctrl = await createHwpCtrl({
  wasmUrl: '/pkg/rhwp_bg.wasm',
  onSave: () => documentState.markClean('hwpctl-save-as'),
});
```

## 검증

- `onSave` 미지정 시 기존 동작 완전히 동일 (`this.onSave`가 `undefined`이므로 `?.()`는 무시)
- 기존 `new HwpCtrl(wasmDoc)` 생성자 호출 전혀 영향 없음
- TypeScript 타입 체크 통과 (기존 오류만 존재)
- 콜백 등록 시 SaveAs 성공 후 정확히 한 번 호출됨
- SaveAs 예외 발생 시 콜백 호출되지 않음 (catch 블록보다 위에 위치)

## 결과
- **Branch**: `pr/fix-issue-2661-hwpctl-markclean`
- **PR**: https://github.com/edwardkim/rhwp/pull/???? (생성 후 업데이트)
- **Closes**: #2661
