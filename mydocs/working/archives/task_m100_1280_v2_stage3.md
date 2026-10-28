# Task #1280 (v2) 3단계 완료보고서 — findPictureAtClick 최상단 선택 + e2e

## 목표

겹침 클릭 시 "보이는 것 = 클릭되는 것"(WYSIWYG)이 되도록, `findPictureAtClick` 1차 패스를
**첫-적중-반환**에서 **`(plane, zOrder, stableIndex)` 최댓값(최상단) 선택**으로 교체한다.
#1171(Pass 0 중첩 picture)·#516(Pass 2 BehindText)은 보존한다.

## 변경 내용

**파일**: `rhwp-studio/src/engine/input-handler-picture.ts`

1. 모듈 헬퍼 3개 추가:
   - `controlTopKey(ctrl)` → `[plane??2, zOrder??0, stableIndex??0]` (Rust `paper_node_sort_key`와
     단일 진실 원천. layer 부재 시 렌더 폴백과 동일).
   - `isAboveControl(a, b)` → 정렬키 사전식 비교(동률이면 false → 기존 emit 순서 유지).
   - `controlToRef(ctrl)` → 적중 컨트롤에서 `PictureObjectRef` 구성(line 끝점 포함).
2. Pass 1 루프: 적중 즉시 반환 대신 `topHit` 를 돌며 `isAboveControl` 로 최상단 갱신,
   루프 후 `controlToRef(topHit)` 반환. line 히트 판정(점-선분 거리/베지어 샘플링)은 그대로 두고
   `return` 대신 `hit` 플래그로 후보화. **Pass 0/Pass 2 무변경.**

핵심: 단일 적중 시 결과 불변(최댓값=그 후보) → 회귀 0. 겹침 시에만 plane 큰 개체 승.

## 검증

```
npx tsc --noEmit                                  # 신규 오류 0 (기존 canvaskit-wasm 3건만)
docker compose --env-file .env.docker run --rm wasm   # WASM 재빌드 (Stage1 Rust 반영)
node e2e/topmost-hittest.test.mjs --mode=headless     # 신규 — PASS
node e2e/textbox-picture-1171.test.mjs --mode=headless # #1171 회귀 — PASS
```

### 신규 e2e: `topmost-hittest.test.mjs`

한컴 권위 샘플 `samples/textbox-under-image.hwp` 로드 → 결과:

```
shapePlane=3(InFrontOfText)  imagePlane=2(Square)
shapeZ=0  imageZ=1   ← z-order는 이미지가 위인데도
overlapHit = shape(ci=2)     ← 겹침 클릭이 최상단 글상자 선택 (plane 우위)
```

- Stage 1 `plane/zOrder/stableIndex` 노출이 WASM 경유로 확인됨.
- **z=1 이미지 위에 z=0 글상자**가 plane(3>2)으로 이겨 클릭에 잡힘 = 한컴 동일.
- (이 샘플은 이미지가 글상자에 완전 포함되어 "이미지-단독 영역"이 없어 단일-적중 회귀 검사는
  자동 생략. 해당 케이스는 #1171 등 기존 e2e가 커버.)

### #1171 회귀: `textbox-picture-1171.test.mjs`

글상자 안 nested picture(cellPath) 클릭이 여전히 picture 반환(Pass 0 보존) + 속성 round-trip PASS.

## 다음 단계

Stage 4 — 정합 회귀 e2e 확장(#516/line/다중구역) + **선택 ref 소비처 lifecycle 검증**
(메모리 룰 `audit-selection-ref-consumers`: 겹침에서 최상단=다른 개체 선택 시 삭제/리사이즈/
오려두기 정상).

## 승인 대기

본 보고서와 소스 커밋 후 승인 요청. 승인 후 Stage 4 진행.
