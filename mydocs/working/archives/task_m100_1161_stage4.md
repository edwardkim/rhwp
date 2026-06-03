# 단계별 완료 보고서 — Task M100-1161 Stage 4

## 목표

프런트(rhwp-studio)가 다단계 cellPath 를 선택 ref 에 보유하고, 복사/오려두기 전 경로가
cellPathJson 을 native 로 전달하도록 배선. 타입 보정.

## 데이터 흐름 (확정)

p16 중첩 picture 컨트롤 레이아웃 실측:
- `paraIdx == parentParaIdx == 58`(외부 표 소유 문단), `controlIdx == 0`(셀 문단 안 picture index)
- `cellPath = [{controlIndex:0,cellIndex:5,cellParaIndex:0},{controlIndex:0,cellIndex:1,cellParaIndex:0}]`

→ `ref.ppi`(=paraIdx=58)=parent_para, `ref.ci`(=controlIdx=0)=control_idx 가 **이미 정확**.
   유일한 누락은 cellPath. 즉 복사 호출에 cellPathJson 만 추가하면 됨.

## 변경 사항

### 1. 선택 ref 가 cellPath 보유
- `findPictureAtClick`(`input-handler-picture.ts`): 반환 타입 + bbox/BehindText hit 2곳에 `cellPath: ctrl.cellPath` 추가.
- `cursor.ts`: `selectedPictureRef` 타입 + `enterPictureObjectSelectionDirect`(param/저장) + `getSelectedPictureRef` 타입에 `cellPath` 추가.
- `input-handler-mouse.ts`: 이미지 선택 2곳(헤더 510 / 본문 831)에서 `picHit.cellPath` 전달.
  (글상자/선 경계 선택 등 shape 분기는 #1171/#919 영역 → 미변경, 계약 C.)

### 2. 복사 native 인자 배선
- `wasm-bridge.ts`: `copyControl`/`exportControlHtml`/`getControlImageData`/`getControlImageMime` 4 래퍼에
  **마지막 선택 인자** `cellPathJson = ''` 추가(기존 호출 하위호환), `this.doc.*(sec, para, cellPathJson, ci)` 호출.
- `input-handler-keyboard.ts`: 헬퍼 `pictureCellPathJson(ref)`(셀 밖이면 '') 신설.
  `writeImageToClipboard` 에 cellPathJson 인자 추가. onKeyDown Ctrl+C/Ctrl+X, onCopy(=onCut 위임) 의
  copyControl/exportControlHtml/writeImageToClipboard 에 cellPathJson 전달.
- `input-handler.ts`: performCopy(=performCut 위임) picture 분기 배선. `getSelectedPictureRef` 재선언 타입에
  누락된 `outerTableControlIdx` + `cellPath` 보정.

### 범위 밖(명시)
- **표** 복사(getSelectedTableRef) 분기는 #1161(picture) 범위 밖 — 미변경(중첩 표 cut 은 기존에 length>1 제외).
- **삭제(cut 의 delete)**: `deletePictureControl(sec,ppi,ci)` 는 cellPath 미지원 → 중첩 셀 picture *삭제*는
  별도 후속(본 이슈는 복사). cut 의 *복사* 부분은 정상화됨.

## 검증

| 항목 | 결과 |
|------|------|
| `tsc --noEmit` (cursor/input-handler/picture/mouse 편집분) | ✅ 오류 0 |
| `tsc` wasm-bridge 4 래퍼 | ⏳ 바인딩 재생성 전: 구 .d.ts(3-arg)로 TS2554 4건 → **WASM 재빌드 후 해소** |
| `canvaskit-wasm` 오류 3건 | 기존부터 존재(본 변경 무관) |
| WASM 재빌드 | (Stage 5) pkg/*.d.ts 4-arg 재생성 → tsc clean |

## 계약 C — 표-중첩 picture 선택 가능 여부

- 레이아웃이 중첩 picture 의 bbox + cellPath 를 방출하므로 `findPictureAtClick` 가 클릭 hit 시
  cellPath 동반 ref 를 생성. hit-test 우선순위(Shape 가로채기)는 미변경(#1171 영역).
- 실제 인터랙티브 선택→Ctrl+C→Ctrl+V 는 Stage 5 작업지시자 시각 판정에서 확인.

## 다음 단계

Stage 5 — WASM 재빌드(tsc clean 확인) + 통합 검증 + 작업지시자 시각 판정 + 최종 보고서(#1171 회귀 절).
