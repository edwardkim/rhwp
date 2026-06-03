# Stage 1 완료 보고서 — Task M100 #394

## 작업 내용

`rhwp-studio/src/engine/input-handler.ts` 의 호출 지점 2 곳을 주석 처리.

### 변경 1 — `updateCaretAndScroll` (이전 line 1503)

```diff
     this.updateSelection();
     this.emitCursorFormatState();
-    this.checkTransparentBordersTransition();
+    // [Task #394] 셀 진입 자동 ON 로직 비활성화 — 한컴 출력 정합성을 위해 OFF 기본값 유지.
+    // 되돌리려면 아래 호출 + line ~1520 의 동일 호출 + 메서드 본체 / 상태 변수 / 이벤트 핸들러
+    // 의 주석을 동시에 풀면 이전 동작 복원.
+    // this.checkTransparentBordersTransition();
     this.updateFieldMarkers();
```

### 변경 2 — `updateCaretNoScroll` (이전 line 1520)

```diff
     this.updateSelection();
     this.emitCursorFormatState();
-    this.checkTransparentBordersTransition();
+    // [Task #394] 셀 진입 자동 ON 로직 비활성화 — 위 updateCaretAndScroll 의 코멘트 참고.
+    // this.checkTransparentBordersTransition();
   }
```

## 검증

| 항목 | 결과 |
|------|------|
| TS 빌드 (`npx tsc --noEmit`) | ✅ 통과 (출력 없음 = 에러 없음) |
| 호출 지점 위치 확인 (`grep`) | ✅ line 1503-1506, 1523-1524 모두 주석 적용 |
| 메서드 본체 (`checkTransparentBordersTransition`) | 그대로 보존 (line 1774) |

## 영향

- 호출 지점이 주석 처리되어 셀 진입 / 탈출 시점에 더 이상 자동 토글 안 일어남
- 메서드 본체는 미호출 상태로 보존 — 컴파일 통과 (TS 의 미사용 private 메서드는 `noUnusedLocals` 미설정 상태에서 에러 안 남)
- 기존 사용자 토글 (`Alt+V → T`, 메뉴 [보기] → [투명 선]) 은 영향 없음

## 다음 단계

Stage 2 — 메서드 본체 + 상태 변수 + 이벤트 핸들러 주석 처리.

## 산출물

- 변경 파일: `rhwp-studio/src/engine/input-handler.ts`
- 본 보고서: `mydocs/working/task_m100_394_stage1.md`
