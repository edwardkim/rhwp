# 구현 계획서 — Task M100 #394

## 이슈

[#394](https://github.com/edwardkim/rhwp/issues/394) — rhwp-studio 표 셀 진입 시 투명선 자동 ON 로직 제거 (기본값 OFF 유지)

## 단일 진실 소스 — `rhwp-studio/src/engine/input-handler.ts`

영향 라인 (현재 기준):

| 라인 | 코드 | 역할 |
|------|------|------|
| 220 | `private wasInCell = false;` | 자동 토글 상태 변수 |
| 221 | `private manualTransparentBorders = false;` | 자동 토글 상태 변수 |
| 222 | `private autoTransparentBorders = false;` | 자동 토글 상태 변수 |
| 219 | `// 투명선 자동 활성화 상태` | 위 변수들의 그룹 코멘트 |
| 390-393 | `eventBus.on('transparent-borders-changed', ...)` | manual 추적 핸들러 |
| 1503 | `this.checkTransparentBordersTransition();` | 호출 지점 (`updateCaretAndScroll`) |
| 1520 | `this.checkTransparentBordersTransition();` | 호출 지점 (`updateCaretNoScroll`) |
| 1769-1794 | `private checkTransparentBordersTransition()` 메서드 본체 | 자동 토글 로직 |

## TS 컴파일 검증

`rhwp-studio/tsconfig.json` 설정:
- `strict: true`
- `noUnusedLocals` / `noUnusedParameters` **미설정** → 미사용 private 필드 / 메서드는 컴파일 에러 안 남

→ 본체를 주석 처리해도 빌드 통과 가능. 다만 본체는 큰 블록이므로 **`/* ... */` 블록 주석 으로 묶어 보존** 하면 가독성 / 보존성 양호.

## Stage 1 — 호출 지점 주석 처리 (line 1503, 1520)

### 변경 1 — `updateCaretAndScroll` 내부 (line 1503)

```typescript
this.updateSelection();
this.emitCursorFormatState();
// [Task #394] 셀 진입 자동 ON 로직 비활성화 — 한컴 출력 정합성을 위해 OFF 기본값 유지.
// 되돌리려면 아래 호출 + line 1520 의 동일 호출 + 메서드 본체 / 상태 변수 / 이벤트 핸들러
// 의 주석을 동시에 풀면 이전 동작 복원.
// this.checkTransparentBordersTransition();
this.updateFieldMarkers();
```

### 변경 2 — `updateCaretNoScroll` 내부 (line 1520)

```typescript
this.updateSelection();
this.emitCursorFormatState();
// [Task #394] 셀 진입 자동 ON 로직 비활성화 — 위 updateCaretAndScroll 의 코멘트 참고.
// this.checkTransparentBordersTransition();
```

### Stage 1 검증

- `rhwp-studio` TS 빌드 (`npm run build` 또는 `npx tsc --noEmit`) 통과
- 메서드 본체는 아직 보존 — `private` 미사용 필드 컴파일 에러 없음 확인

## Stage 2 — 메서드 본체 + 상태 변수 + 이벤트 핸들러 주석 처리

### 변경 3 — 메서드 본체 (line 1769-1794)

블록 주석으로 보존:

```typescript
/* [Task #394] 셀 진입 자동 ON 로직 비활성화 — 호출 지점 (updateCaretAndScroll, updateCaretNoScroll)
   의 호출도 같이 주석 처리됨. 되돌리려면 본 블록 주석 + 호출 지점 주석 + 상태 변수 / 이벤트 핸들러
   주석을 동시에 풀면 이전 동작 복원.

  // 셀 진입/탈출 시 투명선 자동 ON/OFF
  private checkTransparentBordersTransition(): void {
    const nowInCell = this.cursor.isInCell() && !this.cursor.isInTextBox();
    if (nowInCell && !this.wasInCell) {
      // 셀 밖 → 셀 진입: 자동 ON
      if (!this.manualTransparentBorders) {
        this.autoTransparentBorders = true;
        this.wasm.setShowTransparentBorders(true);
        document.querySelectorAll('[data-cmd="view:border-transparent"]').forEach(el => {
          el.classList.add('active');
        });
        this.eventBus.emit('document-changed');
      }
    } else if (!nowInCell && this.wasInCell) {
      // 셀 안 → 셀 탈출: 자동으로 켜진 경우에만 OFF
      if (this.autoTransparentBorders && !this.manualTransparentBorders) {
        this.autoTransparentBorders = false;
        this.wasm.setShowTransparentBorders(false);
        document.querySelectorAll('[data-cmd="view:border-transparent"]').forEach(el => {
          el.classList.remove('active');
        });
        this.eventBus.emit('document-changed');
      }
    }
    this.wasInCell = nowInCell;
  }
*/
```

### 변경 4 — 상태 변수 (line 219-222)

```typescript
// [Task #394] 셀 진입 자동 ON 로직 비활성화 — checkTransparentBordersTransition 와 동시 주석 처리.
// 되돌리려면 아래 3 개 변수 + 호출 지점 + 메서드 본체 + 이벤트 핸들러의 주석을 동시에 해제.
// // 투명선 자동 활성화 상태
// private wasInCell = false;
// private manualTransparentBorders = false;
// private autoTransparentBorders = false;
```

### 변경 5 — 이벤트 핸들러 (line 390-393)

```typescript
// [Task #394] 셀 진입 자동 ON 로직 비활성화 — manual 추적 불필요.
// transparent-borders-changed 이벤트 자체는 view.ts 에서 emit 되므로 보존됨 (다른 구독자가 사용 가능).
// // 투명선 수동 토글 상태 추적
// eventBus.on('transparent-borders-changed', (show) => {
//   this.manualTransparentBorders = show as boolean;
// });
```

### Stage 2 검증

- `rhwp-studio` TS 빌드 통과
- `transparent-borders-changed` 이벤트가 다른 곳에서 emit / on 되는지 검색 — view.ts 의 emit 만 있고 다른 subscribe 없는지 확인 (있어도 영향 없음, 본 핸들러만 정리)

## Stage 3 — 빌드 / 회귀 테스트 / 시각 검증

### 자동 검증

| 검증 항목 | 명령 | 기대 결과 |
|---|---|---|
| TS 빌드 | `cd rhwp-studio && npx tsc --noEmit` | 통과 |
| Vite 빌드 | `cd rhwp-studio && npm run build` 또는 `npx vite build` | 통과 |
| WASM 회귀 | `cargo test --lib` (변경 없음) | 변동 없음 |
| svg_snapshot | `cargo test --test svg_snapshot` | 변동 없음 |
| E2E | `cd rhwp-studio && npm run e2e` | 회귀 없음 |

WASM 빌드는 **본 task 에서 변경 없음** — 재빌드 불필요. 다만 rhwp-studio 가 `pkg/` 를 alias 로 참조하므로 기존 `pkg/` 만 있으면 됨.

### 시각 / 수동 검증 (작업지시자 환경)

| 시나리오 | 기대 동작 |
|---|---|
| 문서 로드 직후 | 투명선 OFF (변경 전과 동일) |
| 표 셀 클릭 | 투명선 자동 ON 안 일어남 |
| 표 셀 안에서 키보드 이동 | 변화 없음 |
| 표 셀 밖으로 이동 | 변화 없음 |
| `Alt+V → T` 단축키 | 투명선 수동 ON / 다시 누르면 OFF (기존 동작 유지) |
| 메뉴 [보기] → [투명 선] | 토글 동작 유지 |
| 사용자가 manual ON 한 상태에서 셀 진입 / 탈출 | manual ON 그대로 유지 (자동 OFF 안 일어남) |

### 시각 검증 산출물

dev server 기동 (`cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700`) → 작업지시자 환경에서 확인.

## 위험 / 대응

### 1. `transparent-borders-changed` 이벤트의 다른 구독자

검증 — Stage 2 에서 grep:
```bash
grep -rn "transparent-borders-changed" rhwp-studio/src 2>/dev/null
```

본 핸들러 (line 391) 외 다른 구독자가 있다면 그것은 보존. emit 측 (view.ts) 은 변경 안 함 — 사용자 토글 시 여전히 emit 되어 다른 구독자가 동작 가능.

### 2. TS 미사용 경고

`tsconfig.json` 에 `noUnusedLocals` / `noUnusedParameters` 없음 → 미사용 private 필드 / 메서드 에러 안 남. 추가 조치 불필요.

### 3. dead code 가독성

코멘트 분량이 많아짐. `[Task #394]` 마커로 추적 가능 + 향후 정리 task 시 grep 으로 일괄 처리.

### 4. 사용자 학습

자동 ON 이 사라지면서 `Alt+V → T` 단축키를 모르는 사용자는 셀 경계 가이드를 못 받을 수 있음. 본 task 범위 외 — 추후 도구 상자 / 메뉴 노출 강화는 별도 task.

## 산출물

| 단계 | 산출물 |
|---|---|
| Stage 1 | `mydocs/working/task_m100_394_stage1.md` |
| Stage 2 | `mydocs/working/task_m100_394_stage2.md` |
| Stage 3 | `mydocs/working/task_m100_394_stage3.md` |
| 최종 | `mydocs/report/task_m100_394_report.md` |

## 다음 단계

본 구현 계획서 승인 → Stage 1 진행.
