# Stage 2 완료 보고서 — Task M100 #394

## 작업 내용

`rhwp-studio/src/engine/input-handler.ts` 의 메서드 본체 + 상태 변수 + 이벤트 핸들러를 주석 처리.

### 변경 3 — 메서드 본체 (line 1773-1798 → 1777-1808)

`/* ... */` 블록 주석으로 보존:

```diff
-  /** 셀 진입/탈출 시 투명선 자동 ON/OFF */
-  private checkTransparentBordersTransition(): void {
-    const nowInCell = this.cursor.isInCell() && !this.cursor.isInTextBox();
-    ...
-    this.wasInCell = nowInCell;
-  }
+  /* [Task #394] 셀 진입 자동 ON 로직 비활성화 — 호출 지점 (updateCaretAndScroll, updateCaretNoScroll)
+     의 호출도 같이 주석 처리됨. 되돌리려면 본 블록 주석 + 호출 지점 주석 + 상태 변수 / 이벤트 핸들러
+     주석을 동시에 풀면 이전 동작 복원.
+
+  // 셀 진입/탈출 시 투명선 자동 ON/OFF
+  private checkTransparentBordersTransition(): void {
+    ...
+    this.wasInCell = nowInCell;
+  }
+  */
```

### 변경 4 — 상태 변수 (line 219-222 → 219-224)

`//` 라인 주석으로 보존:

```diff
-  // 투명선 자동 활성화 상태
-  private wasInCell = false;
-  private manualTransparentBorders = false;
-  private autoTransparentBorders = false;
+  // [Task #394] 셀 진입 자동 ON 로직 비활성화 — checkTransparentBordersTransition 와 동시 주석 처리.
+  // 되돌리려면 아래 3 개 변수 + 호출 지점 + 메서드 본체 + 이벤트 핸들러의 주석을 동시에 해제.
+  // // 투명선 자동 활성화 상태
+  // private wasInCell = false;
+  // private manualTransparentBorders = false;
+  // private autoTransparentBorders = false;
```

### 변경 5 — 이벤트 핸들러 (line 390-393 → 392-397)

```diff
-    // 투명선 수동 토글 상태 추적
-    eventBus.on('transparent-borders-changed', (show) => {
-      this.manualTransparentBorders = show as boolean;
-    });
+    // [Task #394] 셀 진입 자동 ON 로직 비활성화 — manual 추적 불필요.
+    // transparent-borders-changed 이벤트 자체는 view.ts 에서 emit 되므로 보존됨 (다른 구독자가 사용 가능).
+    // // 투명선 수동 토글 상태 추적
+    // eventBus.on('transparent-borders-changed', (show) => {
+    //   this.manualTransparentBorders = show as boolean;
+    // });
```

## 검증

| 항목 | 결과 |
|------|------|
| TS 빌드 (`npx tsc --noEmit`) | ✅ 통과 |
| `transparent-borders-changed` 다른 구독자 점검 | view.ts:121 emit, input-handler.ts:391 (본 핸들러, 주석 처리됨) — **다른 구독자 없음** ✅ |
| 활성 코드의 `checkTransparentBordersTransition` 참조 | 0 건 (모두 주석 안에 있음) ✅ |
| 활성 코드의 상태 변수 (`wasInCell` / `manualTransparentBorders` / `autoTransparentBorders`) 참조 | 0 건 ✅ |
| `[Task #394]` 마커 위치 | line 219, 392, 1507, 1527, 1777 (총 5 영역) ✅ |

## 영향

- **활성 코드에서 자동 토글 로직 완전 비활성화** — 셀 진입 / 탈출 시 투명선 동작 변화 없음
- **모든 코드 보존** — 향후 정책 복귀 시 5 개 영역의 주석을 동시에 해제하면 복원 가능
- **emit 측 (view.ts) 보존** — 사용자 토글 시 여전히 `transparent-borders-changed` 이벤트 발행 (다른 향후 구독자 사용 가능)
- **TS 빌드 영향 없음** — `noUnusedLocals` 미설정이라 미참조 주석 안 코드는 영향 없음

## 다음 단계

Stage 3 — 빌드 / 회귀 테스트 / 시각 검증.

## 산출물

- 변경 파일: `rhwp-studio/src/engine/input-handler.ts`
- 본 보고서: `mydocs/working/task_m100_394_stage2.md`
