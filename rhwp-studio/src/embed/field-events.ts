/**
 * 누름틀 진입·이탈 이벤트(`field-focus-events-v1`) — 부모 페이지에 보내는 값.
 *
 * studio 안의 `field-info-changed` 는 캐럿이 누름틀 안에서 한 칸 움직일 때마다 다시 나온다(상태
 * 표시줄 갱신용). 부모 페이지에는 **어느 누름틀에 있는지가 바뀔 때만** 보낸다. 필드 아이디만으로는
 * 바인딩을 찾을 수 없으므로 이름·안내문을 함께 싣는다.
 */

/** `field-info-changed` 의 값. 누름틀 밖이면 `null`. */
export interface FieldInfoChange {
  fieldId: number;
  fieldType: string;
  guideName?: string;
}

export type RhwpFieldChangedEventV1 =
  | { schemaVersion: 1; inField: false }
  | {
    schemaVersion: 1;
    inField: true;
    fieldId: number;
    fieldType: string;
    name: string;
    guide: string;
  };

/**
 * `field-info-changed` 값을 받아 보낼 이벤트를 만든다. 직전과 같은 누름틀이면 `null`.
 *
 * @param lookupName 필드 아이디로 이름을 찾는다. 못 찾으면 빈 문자열.
 */
export function createFieldChangeForwarder(
  lookupName: (fieldId: number) => string,
): (info: unknown) => RhwpFieldChangedEventV1 | null {
  let lastFieldId: number | null = null;
  return (info) => {
    const fi = info as FieldInfoChange | null | undefined;
    if (!fi || typeof fi.fieldId !== 'number') {
      if (lastFieldId === null) return null;
      lastFieldId = null;
      return { schemaVersion: 1, inField: false };
    }
    if (fi.fieldId === lastFieldId) return null;
    lastFieldId = fi.fieldId;
    return {
      schemaVersion: 1,
      inField: true,
      fieldId: fi.fieldId,
      fieldType: fi.fieldType ?? '',
      name: lookupName(fi.fieldId),
      guide: fi.guideName ?? '',
    };
  };
}
