---
kind: investigation
status: active
canonical: mydocs/tech/investigations/issue-2392/README.md
last_verified: 2026-07-19
---

# Task M100 #2392 - picture props apply characterization

- 이슈: #2392
- 범위: `rhwp-studio/src/ui/picture-props-dialog.ts`의 `handleOk`
- source 기준: `upstream/devel@1cfb42734f57094be5f3c2c43096ce52d0675ce5`
- 측정 HEAD: 계획 문서만 commit된 `6a1ef541bd51a0f1b32804cf1443089c01561a56`
- 작성일: 2026-07-19

## 1. 결론

`handleOk`는 form 값을 읽는 책임, patch 정책, object type 정책, 5개 mutation target, undo/fallback을
381 LOC와 CC 348 안에서 함께 처리한다. pure patch builder와 target resolver를 분리하는 #2392의 방향은
유효하다. 단, 다음 현재 동작은 구현 전에 명시적으로 보존해야 한다.

- diff-only와 always-send field가 섞여 있다.
- 정상 UI의 unchanged 확인도 image/OLE은 caption, shape/line/group은 shadow key 때문에 보통 non-empty다.
- image scale은 common width/height 계산을 같은 key로 덮어쓴다.
- image target은 `headerFooter`가 `cellPath`보다 우선한다.
- non-empty patch만 mutation과 undo를 발생시키고 정상 경로에서는 마지막에 dialog를 닫는다.
- `insert:picture-props`의 재사용 instance에서는 이전 object type의 분리된 DOM control 참조가 남을 수 있다.
- `line`은 화면에 없는 textbox/fill policy를 실행하므로 fresh instance와 shape/group 이후 재사용 instance의
  patch가 달라질 수 있다.

마지막 항목은 #2392에서 조용히 수정하지 않는다. Stage 2 snapshot이 control group의 존재 여부를 보존하도록
설계하고, 기능 수정은 별도 후속 이슈로 판단한다.

## 2. metrics 기준선

### 2.1 재현성

| 항목 | 값 |
|------|-----|
| included product files | 215 |
| git clean / measured source clean | false / true |
| dirty path | Stage 1 evidence 문서 6개, measured source dirty path 0 |
| ESLint / SonarJS | 10.6.0 / 4.1.0 |
| typescript-eslint parser / TypeScript | 8.63.0 / 6.0.3 |
| Node / platform | v24.15.0 / darwin arm64 |
| metrics script SHA-256 | `5d100c90f47671240f463b0a48fe61d34eb8aedbf8c22bbe333f31241f11d087` |
| metrics lock SHA-256 | `a7ae3c1a0f3c94700cfe29dc9c363657cb1f675c988446d5dc81b7eeecace5dd` |
| Studio lock SHA-256 | `22c6b2690959b0a0e0559830220a81f720c5f9775bb383e9a101a88b145b1881` |
| snapshot SHA-256 | `2adeef7656c037556907daec58e3d8f365306c3cfd4dfdebbb9b3a725438754f` |

snapshot은 `output/frontend-metrics/task2392/pre/metrics.json`에 생성했으며 `/output/` ignore 정책에 따라
commit하지 않는다. 동일 source의 post snapshot과 `--compare` 결과로 #2392 delta를 계산한다.

### 2.2 aggregate

| 범위 | 파일/함수 | Total CC | Top 20 합 | CC>25 개수/합 | CC>100 | Max CC |
|------|-----------|---------:|-----------:|---------------:|-------:|-------:|
| frontend 전체 | 215 / 2,386 reported | 12,369 | 2,660 | 70 / 4,297 | 7 | 453 |
| 대상 파일 | 1 / 35 reported | 647 | 해당 없음 | 2 / 560 | 2 | 348 |

대상 파일은 2,825 physical LOC, 2,562 code LOC다. 고복잡도 두 함수는 `handleOk` 348과
범위 밖 `populateFromProps` 212다.

### 2.3 대상 파일의 모든 reported function

| Line | Function | Kind | CC | LOC |
|-----:|----------|------|---:|----:|
| 235 | `open` | method | 9 | 53 |
| 297 | `build` | method | 1 | 67 |
| 358 | anonymous #6 | arrow | 1 | 3 |
| 371 | `rebuildTabs` | method | 6 | 40 |
| 404 | anonymous #11 | arrow | 1 | 6 |
| 463 | anonymous #14 | arrow | 2 | 12 |
| 475 | anonymous #15 | arrow | 2 | 12 |
| 820 | `buildLinePanel` | method | 1 | 155 |
| 909 | anonymous #29 | arrow | 1 | 4 |
| 1194 | `updateFillVisibility` | arrow | 2 | 10 |
| 1368 | anonymous #44 | arrow | 1 | 27 |
| 1424 | anonymous #47 | arrow | 1 | 26 |
| 1481 | `buildPicturePanel` | method | 1 | 265 |
| 1528 | anonymous #50 | arrow | 1 | 6 |
| 1557 | anonymous #51 | arrow | 1 | 13 |
| 1575 | anonymous #52 | arrow | 1 | 5 |
| 1580 | anonymous #53 | arrow | 1 | 5 |
| 1719 | anonymous #57 | arrow | 1 | 6 |
| 1751 | `buildReflectionPanel` | method | 1 | 52 |
| 1808 | `buildGlowPanel` | method | 1 | 59 |
| 1872 | `buildSoftEdgePanel` | method | 1 | 42 |
| 1930 | `handleOk` | method | 348 | 381 |
| 2266 | `applyProps` | arrow | 7 | 26 |
| 2316 | `populateFromProps` | method | 212 | 286 |
| 2583 | anonymous #76 | arrow | 1 | 3 |
| 2603 | `updateSizeProtectControls` | method | 6 | 14 |
| 2620 | anonymous #79 | arrow | 1 | 3 |
| 2626 | `updateOverlapOption` | method | 2 | 8 |
| 2635 | `selectWrap` | method | 3 | 9 |
| 2645 | `getSelectedWrap` | method | 1 | 9 |
| 2656 | `captionGridIndex` | method | 11 | 8 |
| 2666 | `gridIndexToCaption` | method | 13 | 8 |
| 2730 | anonymous #85 | arrow | 1 | 3 |
| 2782 | `numberInput` | method | 3 | 9 |
| 2800 | `selectEl` | method | 1 | 11 |

`applyProps`는 `handleOk` 내부 중첩 함수이지만 별도 entry로도 보고된다. post 비교에서는 stable function id를
사용해 단순 이동과 실제 순감소를 구분한다.

## 3. 현재 apply 순서

1. `props`가 없으면 dialog를 닫고 반환한다.
2. common size, position, description patch를 계산한다.
3. shape/line/group/OLE 또는 image 전용 patch를 계산한다.
4. patch가 비어 있지 않을 때 object type과 context로 mutation setter 하나를 고른다.
5. InputHandler가 있으면 `objectProps` snapshot으로, 없으면 직접 setter와 `document-changed`로 적용한다.
6. 정상 반환 경로에서 dialog를 닫는다.

setter 또는 `executeOperation`이 throw하면 현재 코드는 마지막 `hide()`에 도달하지 않는다. 이번 리팩터링은
오류 처리 의미를 추가하지 않는다.

## 4. field policy

### 4.1 common

| 책임군 | patch key | 입력·변환 | 전송 조건과 baseline |
|--------|-----------|-----------|----------------------|
| size lock | `sizeProtect` | checkbox boolean | `(props.sizeProtect ?? false)`와 다를 때 |
| size | `width`, `height` | `parseFloat(value) || 0`, mm -> rounded HWPUNIT | size lock false이고 현재 값과 다를 때 |
| anchor | `treatAsChar` | checkbox boolean | `props.treatAsChar`와 다를 때 |
| wrap | `textWrap` | selected wrap, `TakePlace`면 `TopAndBottom` | treat-as-char false이고 다를 때 |
| horizontal | `horzRelTo` | select string | treat-as-char false, `TakePlace`가 아니며 다를 때 |
| horizontal | `horzAlign`, `horzOffset` | string, mm -> HWPUNIT | treat-as-char false이고 다를 때 |
| vertical | `vertRelTo`, `vertAlign`, `vertOffset` | string, string, mm -> HWPUNIT | treat-as-char false이고 다를 때 |
| placement | `restrictInPage` | checkbox | `(props.restrictInPage ?? true)`와 다를 때 |
| placement | `allowOverlap` | checkbox | `(props.allowOverlap ?? false)`와 다를 때 |
| metadata | `description` | hidden input string | `props.description`과 다를 때 |

활성 wrap button이 없으면 `getSelectedWrap()`은 `props.textWrap ?? 'Square'`를 반환한다. 이 규칙이
`Through` 무변경 확인을 보존한다.

### 4.2 shape, line, group, OLE

| 책임군 | 적용 type | patch key | 정책 |
|--------|-----------|-----------|------|
| outer margin | OLE | `outerMarginLeft/Right/Top/Bottom` | mm -> HWPUNIT, `(props.* ?? 0)`과 diff |
| caption | OLE | `hasCaption`과 caption detail | control이 있으면 `hasCaption` always-send, true일 때 detail always-send |
| text box margin | shape/line/group | `tbMarginLeft/Right/Top/Bottom` | optional control value가 없으면 0, `(shapeProps.* ?? 0)`과 diff |
| text box align | shape/line/group | `tbVerticalAlign` | active button이 없으면 `Top`, `(shapeProps.* ?? 'Top')`과 diff |
| transform | shape/line/group | `rotationAngle`, `horzFlip`, `vertFlip` | control 존재·활성 조건에서 default 0/false와 diff |
| border | 전 type | `borderColor`, `borderWidth`, `lineType`, `lineEndShape` | control 존재 시 default 0/0/1/0과 diff |
| arrow | OLE 제외 | `arrowStart/End`, `arrowStartSize/EndSize` | control 존재 시 default 0과 diff |
| corner | OLE 제외 | `roundRate` | custom integer 또는 preset 0/20/50, default 0과 diff |
| fill kind | OLE 제외 | `fillType` | radio 선택이 없으면 `none`, default `none`과 diff |
| solid fill | OLE 제외 | `fillBgColor`, `fillPatColor`, `fillPatType` | solid이면 색상 always-send, pattern control 존재 시 `parseInt || -1` |
| gradient | OLE 제외 | gradient 5개 key | gradient이면 존재하는 control 값을 always-send, type default 1, 나머지 0 |
| fill alpha | OLE 제외 | `fillAlpha` | solid/gradient와 control 존재 시 percent * 255 / 100 반올림, always-send |
| shadow | OLE 제외 | shadow type/color/offset | panel 존재 시 type always-send; 0이면 offset 0 always-send, 양수면 color/offset always-send |

OLE는 outer margin, caption, border 4개 key만 전용 branch에서 계산하고 transform, arrow, corner, fill,
shadow를 억제한다. shape/group은 필요한 탭을 모두 생성한다. line은 non-OLE branch 전체를 실행하지만
textbox와 fill 탭을 생성하지 않는다.

### 4.3 image

| 책임군 | patch key | 정책 |
|--------|-----------|------|
| transform | `rotationAngle`, `horzFlip`, `vertFlip` | control 존재·활성 조건에서 default 0/false와 diff |
| outer margin | 4개 outer margin key | mm -> HWPUNIT, default 0과 diff |
| caption | `hasCaption`과 caption detail | `hasCaption` always-send, true일 때 direction/align/width/gap/include-margin always-send |
| border | `borderColor`, `borderWidth` | default 0과 diff |
| scale | `width`, `height` | size lock false, scale control 존재, originalWidth > 0일 때 original size * percent 반올림 |
| crop | 4개 crop key | left control 존재를 group presence로 사용, mm -> HWPUNIT, default 0과 diff |
| padding | 4개 padding key | left control 존재를 group presence로 사용, mm -> HWPUNIT, default 0과 diff |
| effect | `effect` | selected `Original`을 `RealPic`으로 변환, `(effect ?? 'RealPic')`과 diff |
| tone | `brightness`, `contrast` | `parseInt || 0`, default 0과 diff |
| transparency | `transparency` | `parseInt || 0`을 0..100으로 clamp, default 0과 diff |

image scale은 common size 뒤에 실행되어 `width`와 `height`를 덮어쓸 수 있다. `originalHeight > 0`을 별도로
검사하지 않으며 `originalWidth > 0`만 gate다. 이 순서와 gate를 그대로 fixture로 고정한다.

## 5. mutation target matrix

| Case | 판정 입력 | setter와 인자 |
|------|-----------|---------------|
| shape cell | shape/line/group/OLE + `cellPath` | `setCellShapePropertiesByPath(sec, para, cellPath, innerControlIdx, patch)` |
| shape body | shape/line/group/OLE, no cellPath | `setShapeProperties(sec, para, ci, patch)` |
| image header/footer | image + `headerFooter` | `setHeaderFooterPictureProperties(sec, outerParaIdx, outerControlIdx, para, ci, patch)` |
| image cell | image + no header/footer + `cellPath` | `setCellPicturePropertiesByPath(sec, para, cellPath, innerControlIdx, patch)` |
| image body | image + no markers | `setPictureProperties(sec, para, ci, patch)` |
| image both markers | image + `headerFooter` + `cellPath` | header/footer target 우선, cell marker는 사용하지 않음 |

shape 계열은 `headerFooter` marker를 판정하지 않는다. target resolver는 `cellPath` object identity와 모든 index를
변형 없이 반환해야 한다. 실제 5개 WASM setter는 기존 dialog에 남기고 mutation routing baseline 5회를
유지한다.

## 6. undo, fallback, close matrix

| 조건 | setter | undo/history | event | close |
|------|--------|--------------|-------|-------|
| `props` 없음 | 0 | 없음 | 없음 | 즉시 hide |
| empty patch | 0 | 없음 | 없음 | hide |
| non-empty + InputHandler | target setter 1회 | `kind: snapshot`, `operationType: objectProps`; callback은 cursor position 반환 | 명시 emit 없음 | 정상 완료 후 hide |
| non-empty + no InputHandler | target setter 1회 | 없음 | `document-changed` 1회 | 정상 완료 후 hide |
| mutation/operation throw | 완료 전 중단 | 호출 지점까지의 현재 의미 | 보장 없음 | 현재 hide 보장 없음 |

`CommandServices`는 `insert:picture-props`와 `format:object-properties` 두 생성 경로에서 전달된다. fallback은
tests나 services 미주입 소비자를 위한 현재 계약으로 남긴다. 정상 build의 모든 object type에는 caption 또는
shadow always-send group이 있으므로 unchanged 확인도 보통 non-empty다. empty patch는 group이 없는 방어적
snapshot에서 보존할 safety contract다.

## 7. control lifecycle과 line 위험

`build()`는 instance당 한 번만 overlay를 만들고, 각 `open()`은 `rebuildTabs()`로 패널 DOM을 교체한다.
`rebuildTabs()`가 공통으로 초기화하는 것은 tabs, panels, size-lock controls뿐이다. panel builder 내부에서 일부
배열을 새로 만들지만 생성되지 않은 탭의 element field를 `undefined`로 되돌리지는 않는다.

| type | 생성 탭 | apply에서 읽는 탭별 group |
|------|---------|----------------------------|
| image | 기본, 여백/캡션, 선, 그림, 그림자, 반사, 네온, soft edge | common, image, shadow는 image apply에서 미사용 |
| shape/group | 기본, 여백/캡션, 선, 채우기, 글상자, 그림자 | common, textbox, transform, line, fill, shadow |
| line | 기본, 여백/캡션, 선, 그림자 | common, **미생성 textbox/fill**, transform, line, shadow |
| OLE | 기본, 여백/캡션, 선 | common, OLE margin/caption/line; 나머지는 type guard로 억제 |

영향은 다음과 같다.

1. 새 instance에서 line을 열면 textbox element는 `undefined`, align array는 빈 배열, fill element는
   `undefined`다. apply는 이를 0, `Top`, `none`으로 해석해 현재 ShapeProperties와 다르면 patch를 만든다.
2. singleton instance에서 shape/group을 먼저 연 뒤 line을 열면 이전 패널의 detached textbox/fill element
   참조가 남는다. `populateFromProps()`는 그 참조에 현재 line 값을 채우고 apply가 다시 읽는다.
3. 툴바 `format:object-properties`는 호출마다 새 dialog를 만들지만 context menu의
   `insert:picture-props`는 module-level singleton을 재사용한다. 따라서 진입 경로와 과거 open 순서가 결과에
   영향을 줄 수 있다.

이는 동일 입력이 동일 patch를 만들지 못하는 잠재 결함이다. #2392는 현재 control presence와 raw 값을
snapshot에 표현해 parity를 유지한다. 다음 후속 이슈 후보는 Stage 2 전에 생성하지 않는다.

- 제목 초안: `[프론트][개체 속성] line dialog의 미생성 textbox/fill patch와 stale control 참조 제거`
- 범위 초안: type 전환 fixture, panel ref reset, line 허용 field 결정, 한컴/실물 문서 확인

## 8. 현재 검증 범위와 공백

| 근거 | 보장 | 공백 |
|------|------|------|
| `picture-props-undo.test.ts` | `handleOk` 안 snapshot 문자열, 두 생성 경로 services 주입 | helper 분리 후 구조에 과결합, fallback/empty/target 미검증 |
| `wrap-through-preserve.test.ts` | no active wrap에서 원래 값 보존 | 전체 patch deep equality 미검증 |
| `mutation-routing-guard.test.ts` | dialog의 direct mutation call 5회 상한 | 어떤 target이 선택되는지는 미검증 |
| `undo-contracts.test.mjs` case 2/2b/5 | fresh image body apply, history 1건, undo, 실제 Ctrl+Z, Through | shape/line/group/OLE, cell/HF, fallback, field matrix 미검증 |

Stage 2의 pure model test는 field fixture와 6개 target fixture를 data-driven으로 추가한다. Stage 3에서 source
guard를 `applyPropertyPatch`의 snapshot/fallback 의미로 옮긴다. 기존 E2E는 수정하지 않고 회귀 gate로
계속 사용한다.

초기 구현 계획의 "unchanged image/shape 입력은 empty patch" fixture는 현재 always-send 동작과 모순되어
Stage 1에서 보정했다. 정상 unchanged fixture는 mandatory key를 exact 비교하고, 별도의 방어적 snapshot만
empty patch를 기대한다.

## 9. Stage 2 진입 조건

- [x] pre metrics와 모든 대상 파일 function entry 고정
- [x] common, shape/OLE, image field policy 고정
- [x] 5-target와 header/footer 우선순위 고정
- [x] empty/snapshot/fallback/close 의미 고정
- [x] stale control 위험과 current entry-point 차이 기록
- [x] baseline unit/build/headless E2E 통과
- [ ] 작업지시자의 Stage 2 승인
