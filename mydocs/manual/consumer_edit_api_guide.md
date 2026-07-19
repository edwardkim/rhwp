---
kind: reference
status: active
canonical: mydocs/manual/consumer_edit_api_guide.md
last_verified: 2026-07-17
---

# @rhwp/core 편집 API 가이드 (소비자용)

`@rhwp/core`(WASM) 를 앱에 임베드해 HWP 문서를 **생성·편집**하는 개발자를 위한 안내다.
읽기/렌더링 기본은 패키지 README 를 참고하고, 이 문서는 편집 API 호출과 버전 변경 대응에
초점을 둔다.

## 1. 초기화와 문서 객체

```ts
import init, { HwpDocument } from '@rhwp/core';
await init({ module_or_path: '/rhwp_bg.wasm' });

// 빈 문서 생성 (구역 1개 + 빈 문단 1개)
const doc = HwpDocument.createEmpty();

// 기존 파일 로드
const doc2 = new HwpDocument(new Uint8Array(buffer));
```

> 텍스트 레이아웃 계산에는 `globalThis.measureTextWidth` 등록이 필요하다(README 참고).

## 2. 편집 API 한눈에

대부분 결과를 JSON 문자열로 반환한다(`{"ok":true, ...}` 형태). 좌표 인자는
`sectionIdx`(구역), `paraIdx`(문단), `charOffset`(문단 내 글자 위치)를 기준으로 한다.

| 분류 | 대표 메서드 |
|------|-------------|
| 텍스트 | `insertText`, `deleteText`, `getTextRange` |
| 표 | `createTable`, `mergeTableCells`, `splitTableCellInto`, `insertTableRow/Column` |
| 셀 내부 | `insertTextInCell`, `getTextInCell`, `applyCharFormatInCell` (표 셀 좌표 추가) |
| 그림 | `insertPicture` |
| 필드(누름틀) | `insertClickHereField`, `getFieldList`, `setFieldValueByName` |
| 서식 | `applyCharFormat`, `applyParaFormat`, `setCharShapeId` |
| 저장 | `exportHwp` (HWP 바이트 반환) |

정확한 시그니처·반환은 패키지의 `rhwp.d.ts`(타입 정의)를 본다. IDE 자동완성으로 인자
이름과 타입이 표시된다.

## 3. 래퍼(Builder) 패턴 권장

앱에서 직접 좌표 인자를 다루기보다, 자주 쓰는 동작을 감싸는 얇은 래퍼를 두면 유지보수가
쉽다.

```ts
class HwpDocumentBuilder {
  private doc = HwpDocument.createEmpty();

  text(sectionIdx: number, paraIdx: number, offset: number, value: string) {
    this.doc.insertText(sectionIdx, paraIdx, offset, value);
    return this;
  }

  picture(opts: {
    sectionIdx: number; paraIdx: number; charOffset?: number;
    width: number; height: number; naturalWidthPx: number; naturalHeightPx: number;
    extension?: string;
  }, imageBytes: Uint8Array) {
    // options object 변형(*Ex) 사용 — 아래 4절 참고
    this.doc.insertPictureEx(JSON.stringify(opts), imageBytes);
    return this;
  }

  build(): Uint8Array {
    return this.doc.exportHwp();
  }
}
```

## 4. options object 변형(`*Ex`) — 인자 많은 API에 권장

인자가 많은 편집 API 는 **`<이름>Ex(optionsJson[, binary])`** 변형을 함께 제공한다.
positional 인자 대신 JSON 객체로 호출하므로, 라이브러리 버전이 올라가며 인자가 추가·변경돼도
호출부 영향이 작다.

```ts
// positional (인자 위치 변경 시 호출부 모두 수정)
doc.insertPicture(0, 0, 0, '', bytes, 4000, 3000, 100, 80, 'png', '', null, null);

// options object (권장)
doc.insertPictureEx(
  JSON.stringify({
    sectionIdx: 0, paraIdx: 0, charOffset: 0,
    width: 4000, height: 3000, naturalWidthPx: 100, naturalHeightPx: 80, extension: 'png',
  }),
  bytes, // 바이너리는 별도 인자
);
```

규칙:

- 키는 camelCase. 선택 키는 생략 시 기본값(positional default)으로 처리.
- 반환·동작은 positional 과 동일.
- 바이너리(이미지 등)는 JSON 이 아니라 별도 인자(`Uint8Array`).
- `*Ex` 가 있는 메서드는 `rhwp.d.ts` 에서 `Ex(options` 로 찾는다.

예시 — 셀 내부 텍스트/서식을 options 로:

```ts
doc.insertTextInCellEx(JSON.stringify({
  sectionIdx: 0, parentParaIdx: 0, controlIdx: 0,
  cellIdx: 0, cellParaIdx: 0, charOffset: 0, text: '셀 내용',
}));

doc.applyCharFormatInCellEx(JSON.stringify({
  secIdx: 0, parentParaIdx: 0, controlIdx: 0, cellIdx: 0, cellParaIdx: 0,
  startOffset: 0, endOffset: 2, props: { bold: true },
}));
```

## 5. 0.x 버전 변경 대응

`@rhwp/core` 는 0.x 단계라 편집 API 시그니처가 바뀔 수 있다. 업그레이드 비용을 줄이려면:

- 인자가 많은 API 는 `*Ex` 를 쓴다(중간 삽입형 변경에 강함).
- 편집 호출을 래퍼 한 곳에 모은다(변경 시 수정 지점 최소화).
- 업그레이드 시 CHANGELOG 의 `### API` 항목을 확인한다(인자 추가·index 변경을 기록).
- 타입 검사(`tsc`)로 시그니처 불일치를 빌드 단계에서 잡는다.

## 6. 저장

```ts
const hwpBytes = doc.exportHwp(); // Uint8Array — .hwp 파일로 저장
```

> 편집 결과를 원본 HWPX 형식으로 되돌려 저장하는 기능은 제한적이다. 현재는 HWP(.hwp)
> 저장을 권장한다.

## 관련

- 패키지 README: 초기화·렌더링·폰트 설정.
- 타입 정의 `rhwp.d.ts`: 전체 API 시그니처.
- 설계 배경(메인테이너): `mydocs/manual/wasm_api_options_convention.md`, #1413.
