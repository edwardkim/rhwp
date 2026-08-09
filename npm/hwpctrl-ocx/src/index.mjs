/**
 * `@rhwp/hwpctrl` — 웹한글컨트롤(WebHwpCtrl) API v2.4 호환 층.
 *
 * 계약의 출처는 규격서(`spec/webhwpctrl_api.json`)와 **설치된 한글의 실측**이다. 문서가
 * 모호한 자리(서명과 `Parameters N` 이 어긋나는 18건 등)는 오라클이 답한 대로 맞춘다.
 * 대조 하니스: `tools/hwpctrl_compat/`.
 *
 * ## 이 파일이 지키는 규칙
 *
 * - **반환형을 규격대로 돌려준다.** `PutFieldText`·`RenameField` 는 값을 돌려주지 않는다
 *   (오라클 `null`). "성공했으니 true" 는 규격 위반이고, 기존 studio 층이 그렇게 했다.
 * - **없는 것을 있는 척하지 않는다.** 아직 못 하는 API 는 규격의 실패값(`false`/`''`/`-1`)을
 *   돌려주고 `console.warn` 으로 이유를 남긴다.
 * - **브라우저 제약은 규격이 이미 답을 정해 놓았다**(v2.4 §2.2). `Open` 은 업로드된 File,
 *   `SaveAs` 는 다운로드다. Node 에서 돌릴 때는 호스트가 넣어 준 `onSave` 싱크를 쓴다.
 */

/** 필드 목록 구분자 — 규격 §8.3.9. 마지막 필드에는 붙지 않는다. */
const SEP = String.fromCharCode(2);

/** `SetFieldViewOption` 값. 표시 전용이라 문서를 바꾸지 않는다. */
const FIELD_VIEW_DEFAULT = 0;

/** 확장 컨트롤 하나가 문단 안에서 차지하는 코드 유닛 수 — 한글의 `pos` 는 글자 수가 아니다. */
const CONTROL_CODE_UNITS = 8;

/** `EditMode` 기본값 — 일반 편집(규격 §8.2.4). */
const EDIT_MODE_NORMAL = 1;

/** `SelectionMode` — 규격 §8.2.13. 표 셀 블록은 3 이다(오라클 실측). */
const SELECTION_NONE = 0;
const SELECTION_NORMAL = 1;
const SELECTION_TABLE = 3;
/** `Select`(F3) 로 켜는 선택 모드 — 블록이 있든 없든 이 값이다(오라클 실측 17). */
const SELECTION_EXTEND = 17;
/** 표 셀 블록을 줄·열로 넓힌 상태 — 3 에 확장 플래그가 붙는다(오라클 실측 19). */
const SELECTION_TABLE_EXTEND = 19;
/** 개체를 고른 상태(규격 §8.2.13 의 4). */
const SELECTION_OBJECT = 4;
/** 칸(열) 블록을 넓힌 상태 — 2 에 확장 플래그가 붙는다(오라클 실측 18). */
const SELECTION_COLUMN_EXTEND = 18;

/**
 * `Version` 기본값 — **이 층의 버전**이다(규격 §8.2.14).
 *
 * 설치된 한글의 버전이 아니다. 호스트가 `createHwpCtrl({ version })` 로 바꿀 수 있다.
 */
const PACKAGE_VERSION = '0, 0, 0, 0';

/** "이 문단 끝까지" — 코어가 문단 길이로 자른다. */
const WHOLE_PARAGRAPH = 0xffffffff;

/** 글자 크기 증감 폭 — 한글 실측 700→800→900 (HWPUNIT, 1pt). */
const HEIGHT_STEP = 100;
/** 자간 증감 폭 — 한글 실측 0→1 (%). */
const SPACING_STEP = 1;
/** 장평 증감 폭 — 한글 실측 100→101 (%). */
const RATIO_STEP = 1;
/** 줄 간격 증감 폭 — 한글 실측 160→170 (%). */
const LINE_SPACING_STEP = 10;
/** 여백·들여쓰기 증감 폭 — 한글 실측 0→200 (HWPUNIT). */
const MARGIN_STEP = 200;

/** 자간·장평은 언어 일곱 갈래를 한꺼번에 준다. */
function sevenLangs(value) {
  return [value, value, value, value, value, value, value];
}

/**
 * `MovePos` 의 `moveID` — 규격 §8.3.30 표. 여기 없는 값은 아직 구현하지 않은 것이다.
 *
 * **`ACTIONS` 보다 먼저 선언해야 한다** — 이동 액션이 이 값을 참조한다.
 */
const MOVE = {
  MAIN: 0,
  CUR_LIST: 1,
  TOP_OF_FILE: 2,
  BOTTOM_OF_FILE: 3,
  TOP_OF_LIST: 4,
  BOTTOM_OF_LIST: 5,
  START_OF_PARA: 6,
  END_OF_PARA: 7,
  NEXT_POS: 12,
  PREV_POS: 13,
  NEXT_POS_EX: 14,
  PREV_POS_EX: 15,
  NEXT_CHAR: 16,
  PREV_CHAR: 17,
  START_OF_WORD: 8,
  END_OF_WORD: 9,
  NEXT_WORD: 18,
  PREV_WORD: 19,
  START_OF_LINE: 22,
  END_OF_LINE: 23,
  PARENT_LIST: 24,
  TOP_LEVEL_LIST: 25,
  ROOT_LIST: 26,
  // 규격에 번호가 없는 자리 — 구역 이동은 `MovePos` 가 아니라 액션으로만 걸린다. 표 안에서
  // 쓰려고 여기 두되 규격 번호와 겹치지 않게 100 대를 쓴다.
  PREV_SECTION: 101,
  NEXT_SECTION: 102,
};

/**
 * `Run` 이 다루는 액션 표. 동작은 전부 **한글2022 실측**이다.
 *
 * - `toggle` — 같은 액션을 두 번 걸면 되돌아온다(0→1→0→1).
 * - `char`/`para` — 정해진 값을 놓는다.
 * - `charStep`/`paraStep` — 지금 값에서 `step` 만큼 옮긴다.
 *
 * `item` 은 지금 상태를 읽을 파라미터셋 항목, `prop`·`props` 는 코어 서식 API 의 키다.
 * 색은 CSS 문자열로 준다 — 코어가 한글의 BGR 로 옮긴다(빨강 `#FF0000` → 255).
 */
const ACTIONS = {
  // 글자 모양 토글
  CharShapeBold: { kind: 'toggle', item: 'Bold', prop: 'bold' },
  CharShapeItalic: { kind: 'toggle', item: 'Italic', prop: 'italic' },
  CharShapeUnderline: { kind: 'toggle', item: 'UnderlineType', prop: 'underline' },
  CharShapeSuperscript: { kind: 'toggle', item: 'SuperScript', prop: 'superscript' },
  CharShapeSubscript: { kind: 'toggle', item: 'SubScript', prop: 'subscript' },
  CharShapeCenterline: { kind: 'toggle', item: 'StrikeOutType', prop: 'strikethrough' },
  CharShapeOutline: { kind: 'toggle', item: 'OutlineType', prop: 'outlineType', numeric: true },
  CharShapeShadow: { kind: 'toggle', item: 'ShadowType', prop: 'shadowType', numeric: true },
  CharShapeEmboss: { kind: 'toggle', item: 'Emboss', prop: 'emboss' },
  CharShapeEngrave: { kind: 'toggle', item: 'Engrave', prop: 'engrave' },

  // 글자 색 (오라클 실측 BGR: 검정 0 · 파랑 16711680 · 빨강 255 · 초록 32768 · 청록 8421376)
  CharShapeTextColorBlack: { kind: 'char', props: { textColor: '#000000' } },
  CharShapeTextColorBlue: { kind: 'char', props: { textColor: '#0000FF' } },
  CharShapeTextColorRed: { kind: 'char', props: { textColor: '#FF0000' } },
  CharShapeTextColorGreen: { kind: 'char', props: { textColor: '#008000' } },
  CharShapeTextColorBluish: { kind: 'char', props: { textColor: '#008080' } },
  // 자주 안 보이는 셋도 실측값이다 (BGR 6697881 · 16777215 · 65535).
  CharShapeTextColorViolet: { kind: 'char', props: { textColor: '#993366' } },
  CharShapeTextColorWhite: { kind: 'char', props: { textColor: '#FFFFFF' } },
  CharShapeTextColorYellow: { kind: 'char', props: { textColor: '#FFFF00' } },

  // 크기·자간 증감
  CharShapeHeightIncrease: { kind: 'charStep', item: 'Height', prop: 'fontSize', step: HEIGHT_STEP },
  CharShapeHeightDecrease: {
    kind: 'charStep',
    item: 'Height',
    prop: 'fontSize',
    step: -HEIGHT_STEP,
  },
  CharShapeSpacingIncrease: {
    kind: 'charStep',
    item: 'SpacingHangul',
    prop: 'spacings',
    step: SPACING_STEP,
    perLang: true,
  },
  CharShapeSpacingDecrease: {
    kind: 'charStep',
    item: 'SpacingHangul',
    prop: 'spacings',
    step: -SPACING_STEP,
    perLang: true,
  },

  // 장평 증감 — 항목은 `RatioHangul`, 폭은 1 이다(100 → 101 → 102 → 101 실측).
  CharShapeWidthIncrease: {
    kind: 'charStep',
    item: 'RatioHangul',
    prop: 'ratios',
    step: RATIO_STEP,
    perLang: true,
  },
  CharShapeWidthDecrease: {
    kind: 'charStep',
    item: 'RatioHangul',
    prop: 'ratios',
    step: -RATIO_STEP,
    perLang: true,
  },

  // 위 첨자 → 아래 첨자 → 없음 을 돌린다(실측). 따로 있는 `CharShapeSuperscript`·
  // `CharShapeSubscript` 는 각각의 토글이라 이것과 다르다.
  CharShapeSuperSubscript: { kind: 'charCycle' },

  // 글자 모양 되돌리기 — 속성과 색만 지운다. **크기는 그대로 둔다**(실측: 800 유지).
  CharShapeNormal: {
    kind: 'char',
    props: {
      bold: false,
      italic: false,
      underline: false,
      strikethrough: false,
      superscript: false,
      subscript: false,
      outlineType: 0,
      shadowType: 0,
      textColor: '#000000',
    },
  },

  // 문단 정렬 (오라클 실측 AlignType: 양쪽혼합 0 · 왼쪽 1 · 오른쪽 2 · 가운데 3 · 배분 4 · 나눔 5)
  ParagraphShapeAlignJustify: { kind: 'para', props: { alignment: 'justify' } },
  ParagraphShapeAlignLeft: { kind: 'para', props: { alignment: 'left' } },
  ParagraphShapeAlignRight: { kind: 'para', props: { alignment: 'right' } },
  ParagraphShapeAlignCenter: { kind: 'para', props: { alignment: 'center' } },
  ParagraphShapeAlignDistribute: { kind: 'para', props: { alignment: 'distribute' } },
  ParagraphShapeAlignDivision: { kind: 'para', props: { alignment: 'division' } },

  // 줄 간격 증감
  ParagraphShapeIncreaseLineSpacing: {
    kind: 'paraStep',
    parts: [{ item: 'LineSpacing', prop: 'lineSpacing', step: LINE_SPACING_STEP }],
  },
  ParagraphShapeDecreaseLineSpacing: {
    kind: 'paraStep',
    parts: [{ item: 'LineSpacing', prop: 'lineSpacing', step: -LINE_SPACING_STEP }],
  },

  // 여백·들여쓰기 증감. **오른쪽 여백만 부호가 반대다** — 늘리기가 저장값을 -200 으로
  // 옮긴다(실측). 좌우를 함께 옮기는 `IncreaseMargin` 은 둘 다 +200 이라 또 다르다.
  ParagraphShapeIncreaseLeftMargin: {
    kind: 'paraStep',
    parts: [{ item: 'LeftMargin', prop: 'marginLeft', step: MARGIN_STEP }],
  },
  ParagraphShapeDecreaseLeftMargin: {
    kind: 'paraStep',
    parts: [{ item: 'LeftMargin', prop: 'marginLeft', step: -MARGIN_STEP }],
  },
  ParagraphShapeIncreaseRightMargin: {
    kind: 'paraStep',
    parts: [{ item: 'RightMargin', prop: 'marginRight', step: -MARGIN_STEP }],
  },
  ParagraphShapeDecreaseRightMargin: {
    kind: 'paraStep',
    parts: [{ item: 'RightMargin', prop: 'marginRight', step: MARGIN_STEP }],
  },
  ParagraphShapeIncreaseMargin: {
    kind: 'paraStep',
    parts: [
      { item: 'LeftMargin', prop: 'marginLeft', step: MARGIN_STEP },
      { item: 'RightMargin', prop: 'marginRight', step: MARGIN_STEP },
    ],
  },
  ParagraphShapeDecreaseMargin: {
    kind: 'paraStep',
    parts: [
      { item: 'LeftMargin', prop: 'marginLeft', step: -MARGIN_STEP },
      { item: 'RightMargin', prop: 'marginRight', step: -MARGIN_STEP },
    ],
  },
  ParagraphShapeIndentPositive: {
    kind: 'paraStep',
    parts: [{ item: 'Indentation', prop: 'indent', step: MARGIN_STEP }],
  },
  ParagraphShapeIndentNegative: {
    kind: 'paraStep',
    parts: [{ item: 'Indentation', prop: 'indent', step: -MARGIN_STEP }],
  },

  // 문단 보호 토글
  ParagraphShapeProtect: { kind: 'paraToggle', item: 'KeepLinesTogether', prop: 'keepLines' },
  ParagraphShapeWithNext: { kind: 'paraToggle', item: 'KeepWithNext', prop: 'keepWithNext' },

  // 커서 이동 — `MovePos` 표(규격 §8.3.30)와 1:1 이다.
  MoveDocBegin: { kind: 'move', moveID: MOVE.TOP_OF_FILE },
  MoveDocEnd: { kind: 'move', moveID: MOVE.BOTTOM_OF_FILE },
  MoveListBegin: { kind: 'move', moveID: MOVE.TOP_OF_LIST },
  MoveListEnd: { kind: 'move', moveID: MOVE.BOTTOM_OF_LIST },
  MoveParaBegin: { kind: 'move', moveID: MOVE.START_OF_PARA },
  MoveParaEnd: { kind: 'move', moveID: MOVE.END_OF_PARA },
  // 구역 이동은 **앞뒤 구역의 첫 문단 처음**으로 간다 — 지금 구역의 처음이 아니다(판별 실측:
  // 마지막 구역 한가운데에서 위로 가면 그 구역 처음이 아니라 앞 구역 처음이 나온다).
  // 끝 구역에서 아래로, 첫 구역에서 위로는 제자리가 아니라 **그 구역의 처음**으로 물린다.
  MoveSectionUp: { kind: 'move', moveID: MOVE.PREV_SECTION },
  MoveSectionDown: { kind: 'move', moveID: MOVE.NEXT_SECTION },

  MoveParentList: { kind: 'move', moveID: MOVE.PARENT_LIST },
  MoveTopLevelList: { kind: 'move', moveID: MOVE.TOP_LEVEL_LIST },
  MoveRootList: { kind: 'move', moveID: MOVE.ROOT_LIST },
  // 최상위 리스트의 처음·끝. 실측상 루트 리스트의 처음·끝과 같다 — 표가 본문에 놓이므로
  // 셀 안에서 올라가도 최상위는 본문이다.
  MoveTopLevelBegin: { kind: 'move', moveID: MOVE.TOP_OF_FILE },
  MoveTopLevelEnd: { kind: 'move', moveID: MOVE.BOTTOM_OF_FILE },

  // 한 칸 이동. 문단 안에서는 네 가지가 같게 움직인다 — 다른 점은 리스트를 넘나드는지인데,
  // 그 경계 넘기는 아직 구현하지 않았다(문단 끝에서 멈춘다).
  MoveNextChar: { kind: 'move', moveID: MOVE.NEXT_CHAR },
  MovePrevChar: { kind: 'move', moveID: MOVE.PREV_CHAR },
  MoveRight: { kind: 'move', moveID: MOVE.NEXT_CHAR },
  MoveLeft: { kind: 'move', moveID: MOVE.PREV_CHAR },
  MoveNextPos: { kind: 'move', moveID: MOVE.NEXT_POS },
  MovePrevPos: { kind: 'move', moveID: MOVE.PREV_POS },
  MoveNextPosEx: { kind: 'move', moveID: MOVE.NEXT_POS_EX },
  MovePrevPosEx: { kind: 'move', moveID: MOVE.PREV_POS_EX },

  // 줄의 처음·끝. 줄 시작 자리는 파일이 코드 유닛으로 들고 있다(`LineSeg::text_start`).
  // 줄을 **위아래로** 옮기는 이동(`MoveLineUp`·`MoveDown` 등)은 리스트를 넘나드는 기하
  // 탐색이라(실측: 셀 10 → 15 → 20) 아직 다루지 않는다.
  MoveLineBegin: { kind: 'move', moveID: MOVE.START_OF_LINE },
  MoveLineEnd: { kind: 'move', moveID: MOVE.END_OF_LINE },

  // 단어 이동. 단어는 공백으로 나뉜 덩어리이고 누름틀이 그 자체로 경계를 만든다.
  MoveNextWord: { kind: 'move', moveID: MOVE.NEXT_WORD },
  MovePrevWord: { kind: 'move', moveID: MOVE.PREV_WORD },
  MoveWordBegin: { kind: 'move', moveID: MOVE.START_OF_WORD },
  MoveWordEnd: { kind: 'move', moveID: MOVE.END_OF_WORD },

  // 문단 단위 이동. `MovePos` 표에 없는 동작이라 액션에서 직접 다룬다(실측 규칙은 `#moveParagraph`).
  MoveNextParaBegin: { kind: 'movePara', to: 'nextBegin' },
  MovePrevParaBegin: { kind: 'movePara', to: 'prevBegin' },
  MovePrevParaEnd: { kind: 'movePara', to: 'prevEnd' },

  // ── 선택 확장 이동 ──
  //
  // 같은 자리로 가되 **닻에서 여기까지를 블록으로 잡는다**. 닻은 첫 확장 때의 캐럿이고 보통
  // 이동이나 `SetPos` 가 놓는다. 되돌아와 닻과 겹치면 블록이 풀린다(오라클 `result:false`)
  // — 닻은 그대로라 더 가면 반대쪽으로 다시 잡힌다.
  //
  // **블록은 리스트를 넘지 못한다.** 그래서 문서 처음·끝으로 가는 확장은 셀 안에서 그 셀의
  // 처음·끝에 멈춘다(실측: 셀에서 `MoveSelDocEnd` → 셀 끝, `MoveDocEnd` → 본문 끝).
  MoveSelNextChar: { kind: 'move', moveID: MOVE.NEXT_CHAR, sel: true },
  MoveSelPrevChar: { kind: 'move', moveID: MOVE.PREV_CHAR, sel: true },
  MoveSelRight: { kind: 'move', moveID: MOVE.NEXT_CHAR, sel: true },
  MoveSelLeft: { kind: 'move', moveID: MOVE.PREV_CHAR, sel: true },
  MoveSelNextPos: { kind: 'move', moveID: MOVE.NEXT_POS, sel: true },
  MoveSelPrevPos: { kind: 'move', moveID: MOVE.PREV_POS, sel: true },
  MoveSelNextWord: { kind: 'move', moveID: MOVE.NEXT_WORD, sel: true },
  MoveSelPrevWord: { kind: 'move', moveID: MOVE.PREV_WORD, sel: true },
  MoveSelWordBegin: { kind: 'move', moveID: MOVE.START_OF_WORD, sel: true },
  MoveSelWordEnd: { kind: 'move', moveID: MOVE.END_OF_WORD, sel: true },
  MoveSelListBegin: { kind: 'move', moveID: MOVE.TOP_OF_LIST, sel: true },
  MoveSelListEnd: { kind: 'move', moveID: MOVE.BOTTOM_OF_LIST, sel: true },
  MoveSelParaBegin: { kind: 'move', moveID: MOVE.START_OF_PARA, sel: true },
  MoveSelParaEnd: { kind: 'move', moveID: MOVE.END_OF_PARA, sel: true },
  MoveSelDocBegin: { kind: 'move', moveID: MOVE.TOP_OF_LIST, sel: true },
  MoveSelDocEnd: { kind: 'move', moveID: MOVE.BOTTOM_OF_LIST, sel: true },
  MoveSelTopLevelBegin: { kind: 'move', moveID: MOVE.TOP_OF_LIST, sel: true },
  MoveSelTopLevelEnd: { kind: 'move', moveID: MOVE.BOTTOM_OF_LIST, sel: true },
  MoveSelNextParaBegin: { kind: 'movePara', to: 'nextBegin', sel: true },
  MoveSelPrevParaBegin: { kind: 'movePara', to: 'prevBegin', sel: true },
  MoveSelPrevParaEnd: { kind: 'movePara', to: 'prevEnd', sel: true },

  // ── 개체 ──
  //
  // 개체를 고르면 `SelectionMode` 가 4 가 되고 캐럿이 `(문단, 8 × 컨트롤 번호)` 에 선다.
  // 앞뒤 이동은 **문서 순서로 돌아간다**(끝에서 처음으로 감김 — 실측: 두 개체가 번갈아 나온다).
  // `ShapeObjTextBoxEdit` 는 그 개체가 담은 글 리스트 안으로 들어간다(모드 0).
  // 캐럿 자리부터의 **다음 개체**를 고른다. 대상은 본문 층의 잠기지 않은 개체이고, 더 없으면
  // 고르기를 푼다 — 두 표본 열셋으로 확인했다(§4.44).
  SelectCtrlFront: { kind: 'selectCtrl' },

  ShapeObjNextObject: { kind: 'objectMove', step: 1 },
  ShapeObjPrevObject: { kind: 'objectMove', step: -1 },
  ShapeObjTextBoxEdit: { kind: 'objectTextEdit' },

  // 잠금은 고른 개체 하나에 걸고, 풀기는 본문 전체를 푼다. 둘 다 **고르기를 놓는다**(모드 0)
  // — 캐럿은 그 개체 자리에 남는다(실측: 0/0/16 그대로).
  ShapeObjLock: { kind: 'objectLock', locked: true },
  ShapeObjUnlockAll: { kind: 'objectLock', locked: false, all: true },

  // ── 나누기 ──
  //
  // `BreakPara` 는 문단을 가르고 캐럿이 새 문단의 처음으로 간다(6/0/1 → 6/1/0).
  // `BreakLine` 은 문단을 안 가르고 **한 칸짜리 줄바꿈 글자**를 끼운다(길이 +1, 캐럿 +1).
  // 나머지 넷도 **문단을 가른다** — 다른 것은 새 문단이 지는 표식뿐이다(§4.45). 처음엔 빈
  // 문단에서 재다가 앞의 둘이 아무 일도 안 한다고 볼 뻔했다. 자를 빈 곳에 대면 눈금이 없다.
  BreakPara: { kind: 'breakPara' },
  BreakLine: { kind: 'insert', text: '\n' },
  BreakPage: { kind: 'break', breakKind: 'page' },
  BreakColumn: { kind: 'break', breakKind: 'column' },
  BreakColDef: { kind: 'break', breakKind: 'colDef' },
  BreakSection: { kind: 'break', breakKind: 'section' },

  // ── 빈칸 끼우기 ──
  //
  // 셋 다 스트림에서 **한 칸**을 차지하는데 글자가 저마다 다르다(실측: 문단 길이 +1).
  // `InsertTab` 은 탭 글자 하나를 끼우는데 스트림에서는 **8칸**이다(실측: 캐럿 3 → 11) —
  // 코어의 좌표 셈이 그 8칸을 세게 고쳤다(`tab_padding`).
  // 쪽 번호 셋은 사슬에 `atno` 하나를 더하고 스트림에서 8칸을 먹는다 — 컨트롤 아이디로는
  // 셋이 안 갈린다(갈래는 번호 종류에만 있다).
  InsertPageNum: { kind: 'autoNumber', numberKind: 'page' },
  InsertCpNo: { kind: 'autoNumber', numberKind: 'current' },
  InsertTpNo: { kind: 'autoNumber', numberKind: 'total' },

  InsertTab: { kind: 'insert', text: '\t' },
  InsertSpace: { kind: 'insert', text: ' ' },
  InsertNonBreakingSpace: { kind: 'insert', text: '\u001E' },
  InsertFixedWidthSpace: { kind: 'insert', text: '\u001F' },

  // ── 지우기 ──
  //
  // 블록이 있으면 넷 다 블록을 지운다. 없으면 저마다 다른 범위다(전부 실측).
  // `DeleteLine`·`DeleteLineEnd` 는 여기 없다 — "줄"은 조판이 정하는 것이라 파일만
  // 보고는 알 수 없다(§4.18 과 같은 이유).
  // 블록을 지운다. 블록이 없을 때는 안 재 봤다 — 시나리오도 블록이 있는 경우만 건다.
  Erase: { kind: 'delete', to: 'blockOnly' },
  Delete: { kind: 'delete', to: 'nextChar' },
  DeleteBack: { kind: 'delete', to: 'prevChar' },
  DeleteWord: { kind: 'delete', to: 'nextWord' },
  DeleteWordBack: { kind: 'delete', to: 'prevWord' },

  // ── 표 셀 이동 ──
  //
  // 좌우는 **문서 순서로 한 칸**이라 줄을 넘어간다(Tab 과 같다). 위아래는 같은 열의 이웃 줄.
  // `TableColBegin`·`TableColEnd` 는 이름과 달리 **그 줄의 첫 칸·끝 칸**이다(실측 12 → 11·13).
  TableRightCell: { kind: 'tableMove', to: 'next' },
  TableLeftCell: { kind: 'tableMove', to: 'prev' },
  TableLowerCell: { kind: 'tableMove', to: 'down' },
  TableUpperCell: { kind: 'tableMove', to: 'up' },
  TableColBegin: { kind: 'tableMove', to: 'rowBegin' },
  TableColEnd: { kind: 'tableMove', to: 'rowEnd' },

  // ── 표 셀 블록 ──
  //
  // 셀 블록은 글자 범위가 아니라서 `GetSelectedPos` 가 `result:false` 다. 관측되는 것은
  // `SelectionMode` 와 캐럿뿐이다 — 한 칸은 3, 줄·열로 넓히면 19(3 + 확장 플래그)다.
  // ── 표 고치기 ──
  //
  // 끼울 때 캐럿은 **자기 칸을 따라간다**(줄을 위에 끼우면 그 칸이 한 줄 내려가고 캐럿도 같이).
  // 지울 때는 자기 칸이 사라지므로 갈 곳이 정해져 있다 — 줄을 지우면 그 자리 줄의 **첫 칸**,
  // 열을 지우면 **첫 줄**의 그 자리 열이다(둘 다 표 밖으로는 안 나가게 잘린다).
  // `TableAppendRow` 는 이름과 달리 **표 끝에 붙이는 것이 아니라** 지금 줄 바로 아래에
  // 끼우고 캐럿을 그 줄의 같은 칸으로 옮긴다(실측 8 → 11, 9 → 12).
  // `TableSubtractRow` 는 `TableDeleteRow` 와 같은 동작이다(지문·캐럿 모두 일치).
  TableAppendRow: { kind: 'tableEdit', op: 'appendRow' },
  TableSubtractRow: { kind: 'tableEdit', op: 'deleteRow' },
  TableSplitCellRow2: { kind: 'tableEdit', op: 'splitRow2' },
  TableSplitCellCol2: { kind: 'tableEdit', op: 'splitCol2' },
  TableMergeCell: { kind: 'tableMerge' },

  TableInsertUpperRow: { kind: 'tableEdit', op: 'insertRowAbove' },
  TableInsertLowerRow: { kind: 'tableEdit', op: 'insertRowBelow' },
  TableInsertLeftColumn: { kind: 'tableEdit', op: 'insertColLeft' },
  TableInsertRightColumn: { kind: 'tableEdit', op: 'insertColRight' },
  TableDeleteRow: { kind: 'tableEdit', op: 'deleteRow' },
  TableDeleteColumn: { kind: 'tableEdit', op: 'deleteCol' },

  // 오른쪽 칸으로 가되 **마지막 칸이면 줄을 하나 붙이고** 그 첫 칸으로 간다(실측 442 → 443,
  // 경계 442 → 445). 가운데서는 `TableRightCell` 과 같다(8 → 9).
  TableRightCellAppend: { kind: 'tableMove', to: 'nextOrAppend' },

  // 셀 블록을 넓히는 모드. `Extend` 는 사다리다 — 처음엔 모드만 켜고(19) 다시 걸면 표의
  // **마지막 칸**까지 넓힌다. `ExtendAbs` 는 켜기만 하고 되풀이해도 그대로다.
  TableCellBlockExtend: { kind: 'tableBlockExtend', abs: false },
  TableCellBlockExtendAbs: { kind: 'tableBlockExtend', abs: true },

  TableCellBlock: { kind: 'tableBlock', span: 'cell' },
  TableCellBlockRow: { kind: 'tableBlock', span: 'row' },
  TableCellBlockCol: { kind: 'tableBlock', span: 'col' },

  // ── 블록 잡기 ──
  //
  // `SelectColumn` 은 **칸 블록**이다 — `SelectionMode` 가 18(칸 2 + 확장 16)이 되고 캐럿은
  // 제자리다. 덮는 범위는 관측되지 않는다(셀 블록과 마찬가지로 글자 범위가 아니다).
  SelectColumn: { kind: 'selectColumn' },
  SelectAll: { kind: 'selectAll' },
  Select: { kind: 'select' },
  Cancel: { kind: 'cancel' },
};

/**
 * `CreateSet` 이 받아 주는 파라미터셋 이름 — **실측으로 확인한 것만** 담는다.
 *
 * 한글은 아는 이름이면 그 이름을 단 셋을, 모르는 이름이면 빈 이름을 준다. 규격 전체 목록이
 * 아니므로 여기 없는 이름을 한글이 받아 줄 수 있다 — 확인하면 그때 넣는다.
 */
const KNOWN_SET_IDS = new Set([
  'CharShape',
  'ParaShape',
  'SecDef',
  'Table',
  'TableCreation',
  'InsertText',
  'FindReplace',
  'Style',
  'CellBorderFill',
  'ShapeObject',
]);

/** 개체 갈래 → 컨트롤 네 글자 코드. `CurSelectedCtrl` 이 사슬에서 짚을 때 쓴다. */
const CTRL_ID_BY_KIND = { shape: 'gso', picture: 'gso', equation: 'eqed', table: 'tbl' };

/**
 * 규격 §8.4 — 컨트롤 하나. 문서 순서 사슬의 마디다.
 *
 * `CtrlCh` 는 그 컨트롤이 스트림에서 갖는 글자 코드다 — 구역·단 정의 같은 표식은 2, 표·그리기
 * 같은 개체는 11(오라클 실측). `UserDesc` 는 사람이 읽는 이름이고 그리기는 갈래마다 다르다
 * ("사각형"·"타원").
 */
class CtrlCode {
  #at;
  #index;
  #chain;

  constructor(at, index, chain) {
    this.#at = at;
    this.#index = index;
    this.#chain = chain;
  }

  get CtrlID() {
    return this.#at.ctrlId;
  }

  get CtrlCh() {
    return this.#at.ctrlCh;
  }

  get UserDesc() {
    return this.#at.userDesc;
  }

  get Next() {
    return this.#chain()[this.#index + 1] ?? null;
  }

  get Prev() {
    return this.#index === 0 ? null : (this.#chain()[this.#index - 1] ?? null);
  }

  /**
   * 규격 §8.4 — 이 컨트롤이 매달린 자리. `List`·`Para`·`Pos` 를 담은 파라미터셋이다.
   *
   * `Pos` 는 그 컨트롤이 **스트림에서 서 있는 자리**다(본문 첫 문단에 셋이 있으면 0·8·16,
   * 셀 안의 표는 그 문단의 글자 자리 그대로).
   */
  GetAnchorPos() {
    return new ParameterSet('AnchorPos', {
      List: this.#at.list,
      Para: this.#at.para,
      Pos: this.#at.pos,
    });
  }

  /**
   * 규격 §8.4 — 이 컨트롤의 속성 파라미터셋.
   *
   * `Lock` 이 특히 뜻이 있다 — **잠긴 개체는 `SelectCtrlFront` 가 건너뛴다**(실측).
   * `attr` 비트를 풀어야 하는 항목(`TextWrap`·`VertRelTo` …)은 아직 안 넣는다.
   */
  get Properties() {
    return new ParameterSet('Ctrl', this.#at.props ?? {});
  }

  /** 이 컨트롤이 문서 어디에 있는지 — `DeleteCtrl` 이 쓰는 내부 값이다(규격 API 아님). */
  get location() {
    return { list: this.#at.list, para: this.#at.para, controlIndex: this.#at.controlIndex };
  }
}

function parseJson(raw, fallback) {
  try {
    return JSON.parse(raw);
  } catch {
    return fallback;
  }
}

/** `name`, `name{{3}}` 두 표기를 (이름, 순번)으로 가른다. */
function splitOccurrence(token) {
  const m = /^(.*?)\{\{(\d+)\}\}$/.exec(token);
  if (m) return { name: m[1], occurrence: Number(m[2]) };
  return { name: token, occurrence: 0 };
}

/**
 * 파일에 적힌 캐럿 리스트 번호를 **실행 중 번호**로 옮긴다.
 *
 * 문서가 저장한 번호는 서브리스트를 1부터 세고, 실행 중 한글은 2부터 센다(1번 자리를 하나
 * 비워 둔다 — 무엇인지는 아직 모른다). 영수증 서식은 파일에 291, 한글이 답한 값은 292 였다.
 */
function storedListToRuntime(list) {
  return list >= 1 ? list + 1 : 0;
}

/**
 * 필드를 담은 **컨테이너**(표·글상자, 셀 번호는 뺀다)의 식별자.
 * OCX 의 필드 순회가 이 단위로 묶인다 — `ocxFieldOrder` 주석 참고.
 */
function containerKey(location) {
  const parts = [`s${location?.sectionIndex ?? 0}`, `p${location?.paraIndex ?? 0}`];
  for (const entry of location?.path ?? []) {
    parts.push(`c${entry.controlIndex}`);
  }
  return parts.join('/');
}

/**
 * 문서 순서 목록을 **OCX 순회 순서**로 다시 세운다.
 *
 * 한글2022 실측(165개 서식 전수 재구성으로 확인): 필드를 담은 컨테이너가 처음 나온 순서대로
 * 돌되, 컨테이너 하나 안에서는 **셀 구역 이름을 모두 낸 뒤 누름틀을 낸다**. rhwp 의 문서
 * 순서는 셀을 훑으며 둘을 섞어 내므로 그대로 쓰면 순서가 어긋난다(집합은 같다).
 */
function ocxFieldOrder(fields) {
  const groups = new Map();
  for (const field of fields) {
    const key = containerKey(field.location);
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(field);
  }
  const ordered = [];
  for (const group of groups.values()) {
    ordered.push(...group.filter((f) => f.cellField), ...group.filter((f) => !f.cellField));
  }
  return ordered;
}

/**
 * 규격 §9 의 ParameterSet 객체.
 *
 * 한글은 서식·개체 속성을 값이 아니라 **이름표 붙은 항목 묶음**으로 주고받는다. 항목 이름과
 * 단위는 규격의 ParameterSet 표(`spec/parameter_sets.json`)를 따른다 — 예를 들어 `Height` 는
 * HWPUNIT(1/100 pt), `AlignType` 은 0~5 코드값이다.
 *
 * 없는 항목을 물으면 `undefined` 를 돌려준다. 0 으로 채우지 않는다 — "모른다"와 "0이다"를
 * 뭉개면 서식이 틀린 것을 못 잡는다.
 */
export class ParameterSet {
  #setId;
  #items;

  constructor(setId, items = {}) {
    this.#setId = String(setId ?? '');
    this.#items = { ...items };
  }

  /** 규격 §9 — 이 셋의 ID. */
  get SetID() {
    return this.#setId;
  }

  /** 규격 §9 — 담긴 항목 수. */
  get Count() {
    return Object.keys(this.#items).length;
  }

  /** 규격 §9 — 항목 값. 없으면 `undefined`. */
  Item(name) {
    return this.#items[name];
  }

  /** 규격 §9 — 항목 값 설정. */
  SetItem(name, value) {
    this.#items[name] = value;
  }

  /** 규격 §9 — 항목이 있는가. */
  ItemExist(name) {
    return Object.prototype.hasOwnProperty.call(this.#items, name);
  }

  /** 규격 §9 — 항목 제거. */
  RemoveItem(name) {
    delete this.#items[name];
  }

  /** 규격 §9 — 전부 제거. */
  RemoveAll() {
    this.#items = {};
  }

  /** 규격 §9 — 같은 내용의 새 셋. */
  Clone() {
    return new ParameterSet(this.#setId, this.#items);
  }

  /** 내부: 담긴 항목 전부. 호스트와 이 층이 쓴다. */
  toObject() {
    return { ...this.#items };
  }
}

/**
 * 규격 §9 의 Action 객체 — `CreateAction` 이 준다.
 *
 * 반환 갈래가 메서드마다 다르다(실측): `Run` 은 **1**, `Execute` 는 **true**, `GetDefault`
 * 는 **1**. 하나로 뭉개면 안 된다. `SetID` 는 셋을 쓰는 액션만 이름을 주고 아니면 빈 문자열이다.
 */
export class HwpAction {
  #host;
  #actID;

  constructor(host, actID) {
    this.#host = host;
    this.#actID = actID;
  }

  get ActID() {
    return this.#actID;
  }

  /** 이 액션이 쓰는 파라미터셋 이름. 안 쓰면 빈 문자열이다. */
  get SetID() {
    return KNOWN_SET_IDS.has(this.#actID) ? this.#actID : '';
  }

  /** 이 액션이 쓸 빈 셋. 셋을 안 쓰는 액션이면 이름 없는 셋이다. */
  get CreateSet() {
    return new ParameterSet(this.SetID);
  }

  /** 셋에 이 액션의 기본값을 채운다. 아직 채울 값이 없어 셋은 그대로 둔다. */
  GetDefault() {
    return 1;
  }

  /**
   * 셋을 실어 액션을 건다. **셋은 필수다** — 없이 부르면 한글이 "필수 매개 변수입니다" 로
   * 죽는다(실측). 셋에 담긴 값은 아직 안 읽는다; 읽으려면 항목별 실측이 먼저다.
   */
  Execute(set) {
    if (set == null) throw new TypeError('Execute 는 파라미터셋이 필요하다');
    this.#host.Run(this.#actID);
    return true;
  }

  Run() {
    this.#host.Run(this.#actID);
    return 1;
  }
}

export class HwpCtrl {
  #wasm;
  #doc;
  #onSave;
  #cursor = { list: 0, para: 0, pos: 0 };
  /** 리스트 표 캐시 — 문서를 새로 열 때 버린다. */
  #listModel = null;
  #sections = null;
  #fieldViewOption = FIELD_VIEW_DEFAULT;
  #modified = false;
  #editMode = EDIT_MODE_NORMAL;
  #selectionMode = SELECTION_NONE;
  /** 글자 블록의 범위 `{start, end}` (둘 다 커서 좌표). 셀 블록·블록 없음이면 null. */
  #selection = null;

  /** 선택 확장 이동의 닻. 블록이 풀려도 남아서 반대쪽으로 다시 잡히게 한다. */
  #selAnchor = null;

  /** `Select`(F3) 로 켠 선택 모드. 켜져 있으면 보통 이동과 `SetPos` 도 블록을 늘린다. */
  #selectMode = false;

  /** 표 셀 블록이 덮은 격자 범위 — 오라클이 안 보여 주는 값이라 이 층이 들고 있는다. */
  #tableBlock = null;

  /** 고른 개체 `{para, controlIndex, listId}` — 개체 이동과 글상자 진입이 딛는다. */
  #selectedObject = null;

  /** 컨트롤 사슬 — 문서가 바뀌면 버린다. */
  #ctrls = null;

  /** 잠긴 액션 이름들 — `LockCommand` 가 넣고 `Run` 이 본다. */
  #lockedCommands = new Set();
  #version = PACKAGE_VERSION;
  #listeners = new Map();

  constructor({ wasm, doc, onSave, version } = {}) {
    this.#wasm = wasm;
    this.#doc = doc ?? (wasm ? wasm.HwpDocument.createEmpty() : null);
    this.#onSave = onSave;
    if (typeof version === 'string') this.#version = version;
  }

  /** 내부: 현재 문서. 하니스와 호스트가 쓴다. */
  getWasmDoc() {
    return this.#doc;
  }

  // ── 문서 관리 (규격 §8.3.1, 8.3.22, 8.3.33, 8.3.39, 8.3.50~52) ──

  /**
   * 문서 열기. 규격 §8.3.33 — 반환값은 **인자가 제대로 들어왔는지**에 대한 답이고,
   * 실제 성공 여부는 콜백 인자로 온다.
   *
   * 브라우저에서는 업로드된 `File`, Node 에서는 바이트 배열을 받는다.
   */
  Open(source, format, arg, callback, callbackUserData) {
    if (source == null) {
      callback?.(false, callbackUserData);
      return false;
    }
    try {
      const bytes = this.#toBytes(source);
      if (!bytes) {
        // File 은 비동기로만 읽을 수 있다 — 규격이 콜백을 둔 이유다.
        source
          .arrayBuffer()
          .then((buf) => {
            this.#doc = new this.#wasm.HwpDocument(new Uint8Array(buf));
            this.#resetForNewDocument();
            callback?.(true, callbackUserData);
          })
          .catch((e) => {
            console.warn('[hwpctrl] Open 실패:', e);
            callback?.(false, callbackUserData);
          });
        return true;
      }
      this.#doc = new this.#wasm.HwpDocument(bytes);
      this.#resetForNewDocument();
      callback?.(true, callbackUserData);
      return true;
    } catch (e) {
      console.warn('[hwpctrl] Open 실패:', e);
      callback?.(false, callbackUserData);
      return false;
    }
  }

  /** 규격 §8.3.50 — `Open` 의 간소화판. */
  OpenDocument(path, format, callback) {
    return this.Open(path, format, '', callback);
  }

  /**
   * 규격 §8.3.39 — 브라우저에서는 **다운로드**다(v2.4 §2.2). 파일 이름만 지정할 수 있다.
   * Node 에서는 호스트가 넣어 준 `onSave(bytes, fileName)` 싱크로 흘린다.
   */
  SaveAs(fileName, format, arg, callback, callbackUserData) {
    try {
      const bytes = this.#exportBytes(format, fileName);
      if (!bytes) return false;
      if (this.#onSave) {
        this.#onSave(bytes, fileName);
      } else if (typeof document !== 'undefined') {
        this.#download(bytes, fileName);
      } else {
        console.warn('[hwpctrl] SaveAs: 저장 싱크가 없다 (onSave 미지정)');
        return false;
      }
      callback?.(true, callbackUserData);
      return true;
    } catch (e) {
      console.warn('[hwpctrl] SaveAs 실패:', e);
      callback?.(false, callbackUserData);
      return false;
    }
  }

  /** 규격 §8.3.51 — `SaveAs` 의 간소화판. */
  SaveDocument(fileName, format, callback) {
    return this.SaveAs(fileName, format, '', callback);
  }

  /** 규격 §8.3.1 — 문서를 닫고 빈 문서로 만든다. */
  Clear(option) {
    try {
      this.#doc = this.#wasm.HwpDocument.createEmpty();
      this.#resetForNewDocument();
    } catch (e) {
      console.warn('[hwpctrl] Clear 실패:', e);
    }
  }

  /** 규격 §8.3.22 — 문서 끼워넣기. 아직 구현하지 않았다. */
  Insert(path, format, arg, callback, callbackUserData) {
    console.warn('[hwpctrl] Insert: 미구현 (문서 끼워넣기)');
    callback?.(false, callbackUserData);
    return false;
  }

  /** 규격 §8.3.52 — `Insert` 의 간소화판. */
  InsertDocument(path, callback) {
    return this.Insert(path, '', '', callback);
  }

  /** 규격 §8.3.66 — 브라우저 인쇄 대화상자. */
  PrintDocument() {
    if (typeof window !== 'undefined' && typeof window.print === 'function') {
      window.print();
      return;
    }
    console.warn('[hwpctrl] PrintDocument: 브라우저 밖에서는 할 일이 없다');
  }

  // ── 필드 (규격 §8.3.3, 8.3.7~10, 8.3.29, 8.3.34, 8.3.36, 8.3.41~42) ──

  /**
   * 규격 §8.3.9 — 필드 이름을 `0x02` 로 이어 붙인 **문자열**을 돌려준다.
   *
   * - `number` 가 1 이면 이름 뒤에 `{{순번}}` 을 붙인다. 순번은 **돌려주는 목록 안에서** 센다.
   * - `option` 은 낼 종류를 고르는 비트다(한글2022 실측). 0 은 전부 —
   *   `1`=셀 필드만(151) · `2`=누름틀만(14) · `3`=둘 다(165) · `4`=빈 목록.
   *   비트가 하나도 안 서면 아무것도 내지 않는다는 뜻이다.
   *
   * `number` 에 2 를 주면 **오라클이 죽는다**(`com_error` RPC 실패). 시나리오에 넣지 말 것.
   */
  GetFieldList(number = 0, option = 0) {
    const wantCell = option === 0 || (option & 1) !== 0;
    const wantClickHere = option === 0 || (option & 2) !== 0;
    const picked = this.#fields().filter((f) => (f.cellField ? wantCell : wantClickHere));
    const seen = new Map();
    return picked
      .map((f) => {
        const n = seen.get(f.name) ?? 0;
        seen.set(f.name, n + 1);
        return number === 1 ? `${f.name}{{${n}}}` : f.name;
      })
      .join(SEP);
  }

  /** 규격 §8.3.7 — 존재 여부. 순번 접미사(`이름#0`)는 오라클이 받지 않는다. */
  FieldExist(field) {
    if (typeof field !== 'string' || !field) return false;
    return this.#fields().some((f) => f.name === field);
  }

  /** 규격 §8.3.10 — 여러 필드를 `0x02` 로 묶어 물으면 같은 순서로 돌려준다. */
  GetFieldText(fieldlist) {
    if (typeof fieldlist !== 'string' || !fieldlist) return '';
    return fieldlist
      .split(SEP)
      .map((token) => this.#fieldValue(token))
      .join(SEP);
  }

  /**
   * 규격 §8.3.34 — **반환값이 없다.** 현재 필드 내용은 지워지고 새 값이 들어간다.
   * 필드 개수와 텍스트 개수는 같아야 하며, 없는 필드는 무시한다.
   */
  PutFieldText(fieldlist, textlist) {
    if (typeof fieldlist !== 'string' || !fieldlist) return;
    const names = fieldlist.split(SEP);
    const values = typeof textlist === 'string' ? textlist.split(SEP) : [];
    names.forEach((token, idx) => {
      const value = values[idx] ?? '';
      const { name } = splitOccurrence(token);
      try {
        const raw = this.#doc.setFieldValueByName(name, value);
        const parsed = parseJson(raw, { ok: false });
        if (parsed.ok) this.#modified = true;
        else console.warn(`[hwpctrl] PutFieldText("${name}") 실패`);
      } catch (e) {
        // 없는 필드는 무시한다 — 규격 §8.3.34 Remarks.
        console.warn(`[hwpctrl] PutFieldText("${name}"): ${e}`);
      }
    });
  }

  /**
   * 규격 §8.3.8 — 캐럿이 든 필드의 이름. 없으면 빈 문자열.
   *
   * 셀 필드와 누름틀이 한 셀에 같이 있으면 **누름틀이 이긴다**(실측: `night+yn` 셀의
   * 8 위치는 `night_yn` 누름틀 안이다). 셀 이름은 그 셀 어디에 있든 답하는 바닥값이다.
   */
  GetCurFieldName(option = 0) {
    const { list, para, pos } = this.#cursor;
    const fields = this.#fields();
    // 범위는 누름틀 **시작 컨트롤부터** 센다 — 한글은 코드 바로 앞자리(안내문 시작)도
    // 그 필드 안으로 본다. `startPos` 는 컨트롤 8칸 **뒤**(텍스트 시작)를 가리킨다.
    const inside = fields.find(
      (f) =>
        !f.cellField &&
        f.listId === list &&
        f.paraInList === para &&
        pos >= Math.max(0, f.startPos - CONTROL_CODE_UNITS) &&
        pos <= f.endPos,
    );
    if (inside) return inside.name;
    const cell = fields.find((f) => f.cellField && f.listId === list);
    return cell ? cell.name : '';
  }

  /** 규격 §8.3.41 — 캐럿 위치의 필드 이름을 바꾼다(없으면 만든다). */
  SetCurFieldName(fieldname, option, direction, memo) {
    const current = this.GetCurFieldName(0);
    if (current) return this.#renameField(current, fieldname);
    return this.CreateField(direction ?? '', memo ?? '', fieldname);
  }

  /**
   * 규격 §8.3.3 — 캐럿 위치에 누름틀을 만든다.
   *
   * 커서 좌표(list/para/pos) 그대로 넘긴다. 코어가 리스트를 풀고 코드 유닛을 글자 번호로
   * 옮긴다 — 여기서 옮기면 `char_offsets` 없이 짐작하게 된다.
   */
  CreateField(direction, memo, name) {
    try {
      const raw = this.#doc.insertClickHereFieldAtCursor(
        this.#cursor.list,
        this.#cursor.para,
        this.#cursor.pos,
        direction ?? '',
        memo ?? '',
        name ?? '',
        true,
      );
      this.#listModel = null; // 컨트롤이 늘면 뒤 리스트 번호가 밀린다
      this.#sections = null;
      const created = parseJson(raw, { ok: false });
      if (created.ok !== true) return false;
      this.#modified = true;
      // 캐럿은 **만든 누름틀 안으로** 들어간다 — 오라클도 바로 뒤 `GetCurFieldName` 에서
      // 새 이름을 답한다.
      const field = this.#fields().find((f) => f.fieldId === created.fieldId);
      if (field) {
        this.#cursor = { list: field.listId, para: field.paraInList, pos: field.startPos };
      }
      return true;
    } catch (e) {
      console.warn('[hwpctrl] CreateField 실패:', e);
      return false;
    }
  }

  /**
   * 규격 §8.3.36 — **반환값이 없다.**
   *
   * 인자에 `0x02` 리스트를 줘도 오라클은 **첫 짝만** 바꾼다(한글2022 실측:
   * `RenameField("med_str_dt\x02med_end_dt", "시작일\x02종료일")` 뒤 `med_end_dt` 가
   * 그대로 남는다). 규격 문구를 믿고 짝을 맞춰 돌면 오라클보다 더 많이 바꾼다.
   */
  RenameField(oldname, newname) {
    this.#renameField(oldname, newname);
  }

  /**
   * 규격 §8.3.29 — 필드 속성 비트를 지우고(remove) 더한다(add).
   * 음수는 오류를 뜻한다. 아직 편집 가능 비트만 다룬다.
   */
  ModifyFieldProperties(field, remove, add) {
    const target = this.#fields().find((f) => f.name === field);
    if (!target) return -1;
    if (!remove && !add) return 1; // 조회만 — 오라클 실측 반환값
    try {
      const raw = this.#doc.updateClickHereProps(
        target.fieldId,
        target.guide ?? '',
        target.memo ?? '',
        target.name,
        (target.editableInForm && !remove) || Boolean(add),
      );
      return parseJson(raw, { ok: false }).ok === true ? 1 : -1;
    } catch (e) {
      console.warn('[hwpctrl] ModifyFieldProperties 실패:', e);
      return -1;
    }
  }

  /** 규격 §8.3.42 — 표시 옵션. 설정된 값을 그대로 돌려준다(오라클 실측). */
  SetFieldViewOption(option) {
    if (typeof option !== 'number') return 0;
    this.#fieldViewOption = option;
    return option;
  }

  // ── 서식 (규격 §8.2.2, §8.2.11, §8.3.5) ──

  /**
   * 규격 §8.3.5 — 빈 ParameterSet 을 만든다.
   *
   * 규격의 셋 ID 인지 여기서 따지지 않는다. 값을 쓰는 쪽(`CharShape = set` 등)이 아는 항목만
   * 집어 간다.
   */
  CreateSet(setId) {
    return new ParameterSet(setId);
  }

  /**
   * 규격 §8.2.2 — 캐럿 자리의 글자 모양.
   *
   * 항목 이름·단위는 한글 것이다(`Height` HWPUNIT, `FaceNameHangul` 글꼴 이름 …).
   * 아직 못 채우는 항목(`FontType*`·`SmallCaps`·`BorderFill`)은 **담지 않는다**.
   */
  get CharShape() {
    const raw = this.#doc?.getCharShapeSet?.(
      this.#cursor.list,
      this.#cursor.para,
      this.#cursor.pos,
    );
    return new ParameterSet('CharShape', parseJson(raw ?? '', {}) ?? {});
  }

  /** 규격 §8.2.11 — 캐럿이 놓인 문단의 문단 모양. */
  get ParaShape() {
    const raw = this.#doc?.getParaShapeSet?.(this.#cursor.list, this.#cursor.para);
    return new ParameterSet('ParaShape', parseJson(raw ?? '', {}) ?? {});
  }

  // ── 문서 속성 (규격 §8.2) ──

  /** 규격 §8.2.7 — 아무 내용도 없는 빈 문서인가. 읽기 전용. */
  get IsEmpty() {
    try {
      return this.#doc.isEmptyDocument();
    } catch {
      return true;
    }
  }

  /**
   * 규격 §8.2.8 — 연 뒤 문서가 바뀌었는가. 문서를 바꾸는 호출이 성공하면 선다.
   *
   * **오라클과 값을 맞추지 않는다.** 한글의 이 값은 문서 상태가 아니라 편집 엔진의 실행취소
   * 경계를 따라간다 — 커서를 옮긴 뒤의 첫 `PutFieldText` 는 값이 분명히 들어갔는데도(바로
   * 읽으면 새 값이 나온다) false 였고, 두 번째 쓰기에서야 true 가 됐다. 그 시차까지 흉내내면
   * 남의 구현 사정을 계약으로 굳히는 셈이다.
   */
  get IsModified() {
    return this.#modified;
  }

  /**
   * 규격 §8.2.14 — **웹한글컨트롤 자신의 버전**이다.
   *
   * 규격이 못박는다: "웹한글컨트롤은 한글 설치와 관계없이 사용되므로 웹한글의 버전을
   * 리턴한다." 그래서 설치된 한글의 버전(COM 오라클이 답하는 값)과 같을 수 없다 —
   * 이 항목만은 오라클이 판정자가 아니다. 호스트가 값을 정할 수 있다.
   */
  get Version() {
    return this.#version;
  }

  /**
   * 규격 §8.2.4 — 편집 모드. 0=읽기 전용 · 1=일반 · 2=양식 모드 · 16=배포용(지정 불가).
   *
   * 값을 지니기만 한다. **양식 모드의 편집 제약은 아직 걸지 않는다** — 2 로 두어도
   * 편집 불가 필드가 막히지 않는다.
   */
  get EditMode() {
    return this.#editMode;
  }

  set EditMode(mode) {
    if (mode === 16) {
      console.warn('[hwpctrl] EditMode 16(배포용)은 규격상 지정할 수 없다');
      return;
    }
    if (mode === 0 || mode === 1 || mode === 2) this.#editMode = mode;
  }

  /**
   * 규격 §8.2.13 — 블록 지정 상태. 읽기 전용.
   *
   * 0=없음 · 1=일반 · 2=칸 · 3=표 셀 블록 · 4=개체. 지금 블록을 만드는 길은
   * `MoveToField(select=true)` 뿐이다.
   */
  get SelectionMode() {
    // `Select`(F3) 로 켠 선택 모드는 블록이 있든 없든 17 이다(실측).
    if (this.#selectMode) return SELECTION_EXTEND;
    return this.#selectionMode;
  }

  /**
   * 규격 §8.3 — 액션 하나를 잠그거나 푼다. 잠긴 액션은 `Run` 이 **아무 일도 하지 않는다**
   * (오라클 실측: 잠근 채 `MoveNextChar` 를 걸면 캐럿이 그대로다).
   */
  LockCommand(actionID, lock) {
    if (lock) this.#lockedCommands.add(actionID);
    else this.#lockedCommands.delete(actionID);
  }

  /** 규격 §8.3 — 그 액션이 잠겨 있는가. 잠근 것만 참이다(다른 액션은 영향 없다). */
  IsCommandLock(actionID) {
    return this.#lockedCommands.has(actionID);
  }

  /** 규격 §8.4 — 문서가 담은 첫 컨트롤. `Next` 로 사슬을 탄다. */
  get HeadCtrl() {
    return this.#ctrlChain()[0] ?? null;
  }

  /** 규격 §8.4 — 문서가 담은 마지막 컨트롤. */
  get LastCtrl() {
    const chain = this.#ctrlChain();
    return chain[chain.length - 1] ?? null;
  }

  /** 규격 §8.4 — 지금 고른 개체의 컨트롤. 고른 것이 없으면 `null`. */
  get CurSelectedCtrl() {
    const obj = this.#selectedObject;
    if (!obj) return null;
    const chain = this.#ctrlChain();
    // 자리로 짚는 것이 먼저다 — `SelectCtrlFront` 는 종류를 안 남긴다. 개체 목록으로 고른
    // 경우에만 종류로 되짚는다(그 길은 자리 대신 종류를 준다).
    return (
      chain.find(
        (c) => c.location.para === obj.para && c.location.controlIndex === obj.controlIndex,
      ) ??
      chain.find((c) => c.CtrlID === CTRL_ID_BY_KIND[obj.kind]) ??
      null
    );
  }

  /** 어느 리스트든 그것을 담은 **본문 문단** 번호로 올라간다. 본문이면 그대로다. */
  #bodyParaOf(list, para) {
    if (list === 0) return para;
    const model = this.#cursorModel();
    let entry = model.byId.get(list);
    let guard = 0;
    while (entry && entry.hostListId !== 0 && guard < 64) {
      entry = model.byId.get(entry.hostListId);
      guard += 1;
    }
    return entry ? entry.hostPara : 0;
  }

  /** 구역마다 첫 본문 문단 번호. 나누기가 구역을 늘리면 리스트 표와 함께 다시 읽는다. */
  #sectionStarts() {
    if (this.#sections) return this.#sections;
    const raw = parseJson(this.#doc?.getSectionStarts?.() ?? '', null);
    this.#sections = Array.isArray(raw) ? raw : [0];
    return this.#sections;
  }

  /** 컨트롤 사슬 — 코어가 문서 순서로 준다. 문서가 바뀌면 다시 만든다. */
  #ctrlChain() {
    if (this.#ctrls) return this.#ctrls;
    const raw = parseJson(this.#doc?.getControls?.() ?? '', null);
    const items = Array.isArray(raw) ? raw : [];
    this.#ctrls = items.map((it, i) => new CtrlCode(it, i, () => this.#ctrls));
    return this.#ctrls;
  }

  /**
   * 규격 §8.2 — 캐럿이 든 필드의 상태.
   *
   * 실측값 셋: 필드 밖 0 · 셀 필드 안 17 · 누름틀 안 18. 0x10 이 "필드 안"이고 아래 두 비트가
   * 갈래다.
   */
  get CurFieldState() {
    const { list, para, pos } = this.#cursor;
    return this.#doc?.getCurFieldState?.(list, para, pos) ?? 0;
  }

  /**
   * 규격 §8.3 — 캐럿 위치를 파라미터셋으로. `GetPos` 와 같은 값을 `List`·`Para`·`Pos` 로 준다.
   */
  GetPosBySet() {
    const { list, para, pos } = this.#cursor;
    return new ParameterSet('Pos', { List: list, Para: para, Pos: pos });
  }

  /** 규격 §8.3 — 파라미터셋으로 캐럿을 옮긴다. `SetPos` 와 같은 자를 쓴다. */
  SetPosBySet(set) {
    const at = set?.toObject ? set.toObject() : (set ?? {});
    return this.SetPos(at.List ?? 0, at.Para ?? 0, at.Pos ?? 0);
  }

  /**
   * 규격 §8.3 — 이름으로 빈 파라미터셋을 만든다.
   *
   * **아는 이름이면 그 이름을, 모르면 빈 이름을 단 셋**을 준다(실측: `CharShape`·`Table` 따위는
   * 그대로, 없는 이름은 `""`). 아래 목록은 **실측으로 확인한 것만** 담는다 — 규격 전체 목록이
   * 아니다. 확인한 적 없는 이름을 넣으면 "모른다"가 사라진다.
   */
  CreateSet(setId) {
    return new ParameterSet(KNOWN_SET_IDS.has(setId) ? setId : '', {});
  }

  /**
   * 규격 §8.3 — 액션 하나를 객체로 만든다.
   *
   * `Run` 을 바로 부르는 길과 같은 일을 하되, 파라미터셋을 실어 `Execute` 할 수 있다.
   * 실측으로 확인한 것: `ActID` 는 준 이름 그대로, `SetID` 는 셋을 쓰는 액션이면 그 이름이고
   * 안 쓰면 **빈 문자열**이다(`MoveDocEnd` → `""`). `Run` 은 **1**, `Execute` 는 **true** 를
   * 돌려주고 — 둘의 반환 갈래가 다르다 — `GetDefault(셋)` 은 **1** 이다.
   */
  CreateAction(actionID) {
    const id = String(actionID ?? '');
    // 액션이 아닌 이름(예: `MovePos` 는 메서드다)에는 객체를 안 준다 — 오라클이 `null` 이다.
    if (!(id in ACTIONS) && !KNOWN_SET_IDS.has(id)) return null;
    return new HwpAction(this, id);
  }

  /**
   * 규격 §8.3 — 컨트롤 하나를 지운다. 사슬에서 얻은 `Ctrl` 을 그대로 넘긴다.
   *
   * 지우면 사슬이 다시 매겨지므로 캐시를 버린다.
   */
  DeleteCtrl(ctrl) {
    const at = ctrl?.location;
    if (!at) return false;
    let ok = false;
    try {
      const raw = this.#doc.deleteControlAt(at.list, at.para, at.controlIndex);
      ok = parseJson(raw, { ok: false }).ok !== false;
    } catch (e) {
      console.warn('[hwpctrl] DeleteCtrl 실패:', e);
      return false;
    }
    if (!ok) return false;
    this.#ctrls = null;
    this.#listModel = null;
    this.#sections = null;
    this.#modified = true;
    return true;
  }

  // ── 커서·문서 정보 ──

  /** 규격 §8.2.10 — 전체 쪽수. */
  PageCount() {
    try {
      return this.#doc.pageCount();
    } catch {
      return 0;
    }
  }

  /** 규격 §8.3.12 — 웹은 객체를 돌려준다. */
  GetPos() {
    return { ...this.#cursor };
  }

  /**
   * 규격 §8.3.43 — 캐럿을 리스트 좌표로 옮긴다.
   *
   * 없는 리스트를 주면 한글은 **문서의 시작으로 떨어뜨린다**(실측: 마지막 리스트 다음
   * 번호와 400 둘 다 루트로 갔다). 그래도 반환은 true 다.
   */
  SetPos(list, para, pos) {
    // 선택 모드(F3)에서는 자리를 옮겨도 블록이 안 풀린다 — 닻에서 새 자리까지로 늘어난다.
    const anchor = this.#selectMode ? this.#selAnchor : null;
    this.#clearSelection();
    this.#selectMode = anchor != null;
    this.#selAnchor = anchor;
    if (!this.#cursorExists(list, para)) {
      this.#cursor = this.#topOfFile();
      return true;
    }
    // 문단 밖을 찍으면 문단 안으로 자른다 — 한글도 그렇다(59칸 문단에 60을 주면 59,
    // 앞머리 자리차지가 있는 문단에 0 을 주면 그 뒤 자리로 민다).
    const bounds = this.#paraBounds(list, para);
    this.#cursor = { list, para, pos: Math.min(Math.max(pos, bounds.start), bounds.end) };
    if (anchor) this.#applyExtendedSelection(anchor);
    return true;
  }

  /**
   * 규격 §8.3.30 — 캐럿 이동. `moveID` 는 §8.3.30 표를 따른다.
   *
   * 위치 기반 이동만 구현했다(오라클로 계약을 확인한 것들). 글자·줄·단어 단위 이동은
   * 규격의 실패값 `false` 를 돌려주고 이유를 남긴다 — 못 하는 것을 하는 척하지 않는다.
   */
  MovePos(moveID = MOVE.CUR_LIST, para = 0, pos = 0) {
    const model = this.#cursorModel();
    this.#clearSelection(); // 규격 §8.3.30 — 위치 이동 시 셀렉션은 무조건 풀린다
    switch (moveID) {
      case MOVE.MAIN: // 루트 리스트의 특정 위치
        this.#cursor = this.#cursorExists(0, para)
          ? { list: 0, para, pos }
          : this.#topOfFile();
        return true;
      case MOVE.CUR_LIST: // 현재 리스트의 특정 위치
        this.#cursor = this.#cursorExists(this.#cursor.list, para)
          ? { list: this.#cursor.list, para, pos }
          : this.#topOfFile();
        return true;
      case MOVE.TOP_OF_FILE:
        this.#cursor = this.#topOfFile();
        return true;
      case MOVE.BOTTOM_OF_FILE:
        this.#cursor = { list: 0, para: model.root.endPara, pos: model.root.endPos };
        return true;
      case MOVE.TOP_OF_LIST: {
        // 리스트의 첫 문단 — 본문은 앞머리 자리차지 컨트롤을 건너뛴 자리라 0 이 아니다.
        const list = this.#cursor.list;
        this.#cursor = { list, para: 0, pos: this.#paraBounds(list, 0).start };
        return true;
      }
      case MOVE.BOTTOM_OF_LIST: {
        const list = this.#cursor.list;
        const last = Math.max(0, this.#listParaCount(list) - 1);
        this.#cursor = { list, para: last, pos: this.#paraBounds(list, last).end };
        return true;
      }
      case MOVE.NEXT_CHAR:
      case MOVE.NEXT_POS:
      case MOVE.NEXT_POS_EX:
        this.#cursor = { ...this.#cursor, pos: this.#stepCaret(1) };
        return true;
      case MOVE.PREV_CHAR:
      case MOVE.PREV_POS:
      case MOVE.PREV_POS_EX:
        this.#cursor = { ...this.#cursor, pos: this.#stepCaret(-1) };
        return true;
      case MOVE.NEXT_WORD:
      case MOVE.PREV_WORD:
      case MOVE.START_OF_WORD:
      case MOVE.END_OF_WORD: {
        const starts = this.#wordStarts();
        const pos = this.#cursor.pos;
        let next;
        if (moveID === MOVE.NEXT_WORD) {
          next = starts.find((s) => s > pos) ?? starts[starts.length - 1];
        } else if (moveID === MOVE.PREV_WORD) {
          next = starts.filter((s) => s < pos).pop() ?? starts[0];
        } else if (moveID === MOVE.START_OF_WORD) {
          // 지금 단어의 처음 — 자기 자리가 단어 시작이면 제자리다.
          next = starts.filter((s) => s <= pos).pop() ?? starts[0];
        } else {
          // 지금 단어의 끝 — **다음 공백 글자의 자리**다(실측: 4 → 6, 1 → 2). 마지막
          // 단어에서는 문단 끝이다(16 → 17).
          const raw = this.#doc?.getWordEnd?.(this.#cursor.list, this.#cursor.para, pos);
          const parsed = parseJson(raw ?? '', null);
          next = typeof parsed === 'number' ? parsed : pos;
        }
        this.#cursor = { ...this.#cursor, pos: next };
        return true;
      }
      case MOVE.START_OF_LINE:
      case MOVE.END_OF_LINE: {
        const starts = parseJson(
          this.#doc?.getLineStarts?.(this.#cursor.list, this.#cursor.para) ?? '',
          null,
        );
        const bounds = this.#paraBounds(this.#cursor.list, this.#cursor.para);
        const lines = Array.isArray(starts) && starts.length ? starts : [bounds.start];
        const pos = this.#cursor.pos;
        this.#cursor = {
          ...this.#cursor,
          pos:
            moveID === MOVE.START_OF_LINE
              ? (lines.filter((s) => s <= pos).pop() ?? lines[0])
              : (lines.find((s) => s > pos) ?? bounds.end),
        };
        return true;
      }
      case MOVE.START_OF_PARA:
        this.#cursor = {
          ...this.#cursor,
          pos: this.#paraBounds(this.#cursor.list, this.#cursor.para).start,
        };
        return true;
      case MOVE.END_OF_PARA:
        this.#cursor = {
          ...this.#cursor,
          pos: this.#paraBounds(this.#cursor.list, this.#cursor.para).end,
        };
        return true;
      case MOVE.PREV_SECTION:
      case MOVE.NEXT_SECTION: {
        // 셀 안에서 걸면 **본문으로 나간다**(실측: 셀에서 위로 가면 본문 첫 구역 처음).
        // 어느 구역에 있었는지는 그 리스트를 담은 본문 문단으로 친다.
        const starts = this.#sectionStarts();
        if (!starts.length) return true;
        const bodyPara = this.#bodyParaOf(this.#cursor.list, this.#cursor.para);
        let here = 0;
        for (let i = 0; i < starts.length; i += 1) if (starts[i] <= bodyPara) here = i;
        const step = moveID === MOVE.NEXT_SECTION ? 1 : -1;
        const target = Math.min(starts.length - 1, Math.max(0, here + step));
        const para = starts[target];
        this.#cursor = { list: 0, para, pos: this.#paraBounds(0, para).start };
        return true;
      }
      case MOVE.PARENT_LIST:
      case MOVE.TOP_LEVEL_LIST:
      case MOVE.ROOT_LIST: {
        // 올라간 뒤 위치는 그 서브리스트를 담은 컨트롤 자리다 — 컨트롤 하나가 8 코드 유닛.
        let entry = model.byId.get(this.#cursor.list);
        if (!entry) return true; // 이미 루트면 제자리 (규격 §8.3.30 moveRootList 주석)
        if (moveID !== MOVE.PARENT_LIST) {
          while (entry.hostListId !== 0) entry = model.byId.get(entry.hostListId) ?? entry;
        }
        this.#cursor = {
          list: entry.hostListId,
          para: entry.hostPara,
          pos: entry.controlIndex * CONTROL_CODE_UNITS,
        };
        return true;
      }
      default:
        console.warn(`[hwpctrl] MovePos(moveID=${moveID})는 아직 구현하지 않았다`);
        return false;
    }
  }

  /**
   * 규격 §8.3.31 — 필드로 캐럿을 옮긴다.
   *
   * `start` 가 참이면 필드의 처음, 거짓이면 끝이다. 셀 필드는 셀 리스트의 첫 문단 0 위치다
   * (오라클 실측: 셀 필드 이동 뒤 `GetPos` 가 항상 `{셀 리스트, 0, 0}`).
   */
  MoveToField(field, text, start, select) {
    const { name, occurrence } = splitOccurrence(String(field ?? ''));
    const target = this.#fields().filter((f) => f.name === name)[occurrence];
    if (!target) return false;
    this.#cursor = {
      list: target.listId ?? 0,
      para: target.paraInList ?? 0,
      pos: (start ?? true) ? (target.startPos ?? 0) : (target.endPos ?? 0),
    };
    // 블록 상태는 세 갈래다(한글2022 실측): 셀 필드는 내용과 무관하게 표 셀 블록(3),
    // 누름틀은 내용이 있으면 일반 블록(1), 비어 있으면 잡을 게 없어 블록 없음(0).
    if (!select) {
      this.#clearSelection();
    } else if (target.cellField) {
      // 셀 블록은 **글자 범위가 아니다** — 오라클의 `GetSelectedPos` 가 실패로 답한다.
      this.#selectionMode = SELECTION_TABLE;
      this.#selection = null;
    } else if (target.value) {
      this.#selectionMode = SELECTION_NORMAL;
      this.#selection = {
        start: { list: target.listId, para: target.paraInList, pos: target.startPos },
        end: { list: target.listId, para: target.paraInList, pos: target.endPos },
      };
    } else {
      this.#clearSelection();
    }
    return true;
  }

  // ── 블록(선택 영역) — 규격 §8.3.14, §8.3.40 ──

  /**
   * 규격 §8.3.40 — 현재 리스트 안에서 글자 블록을 잡는다. `epos` 가 가리키는 글자는 **뺀다**.
   *
   * 인자에 리스트 아이디가 없다 — 블록은 **한 리스트 안에서만** 만들어진다.
   */
  SelectText(spara, spos, epara, epos) {
    const list = this.#cursor.list;
    if (!this.#cursorExists(list, spara) || !this.#cursorExists(list, epara)) return false;
    this.#selectionMode = SELECTION_NORMAL;
    this.#selection = {
      start: { list, para: spara, pos: spos },
      end: { list, para: epara, pos: epos },
    };
    this.#cursor = { list, para: epara, pos: epos };
    return true;
  }

  /**
   * 규격 §8.3.14 — 블록의 시작·끝 위치.
   *
   * 규격의 속성 목록에는 `result` 가 없지만 컨트롤은 그것을 함께 준다(오라클 실측). 셀 블록은
   * **글자 범위가 아니라서** `result:false` 에 전부 0 이다 — 그 구분이 실제 정보다.
   */
  GetSelectedPos() {
    const sel = this.#selection;
    if (!sel) {
      return { result: false, slist: 0, spara: 0, spos: 0, elist: 0, epara: 0, epos: 0 };
    }
    return {
      result: true,
      slist: sel.start.list,
      spara: sel.start.para,
      spos: sel.start.pos,
      elist: sel.end.list,
      epara: sel.end.para,
      epos: sel.end.pos,
    };
  }

  /**
   * 규격 §8.3.38 — 액션을 실행한다. **반환값이 없다**(오라클 `null`).
   *
   * 지금 다루는 것은 글자 모양 토글뿐이다(`CharShapeBold`·`Italic`·`Underline`). 한글에서
   * 이 액션들은 **토글**이다 — 같은 액션을 두 번 걸면 되돌아온다(실측 0→1→0→1).
   *
   * 블록이 없으면 한글은 "다음에 칠 글자"의 서식을 바꾼다. 이 층은 그 대기 서식을 모델링하지
   * 않아서 **아무 일도 하지 않고** 이유를 남긴다.
   */
  Run(actionID, callback, callbackUserData) {
    // 잠긴 액션은 아무 일도 하지 않는다(실측).
    if (this.#lockedCommands.has(actionID)) {
      callback?.(null, false, callbackUserData);
      return;
    }
    const action = ACTIONS[actionID];
    if (!action) {
      console.warn(`[hwpctrl] Run("${actionID}")는 아직 구현하지 않았다`);
      callback?.(null, false, callbackUserData);
      return;
    }
    if (action.kind === 'move' || action.kind === 'movePara') {
      const moved = this.#runMoveAction(action);
      callback?.(null, moved, callbackUserData);
      return;
    }
    if (action.kind === 'selectColumn') {
      this.#selectionMode = SELECTION_COLUMN_EXTEND;
      this.#selection = null;
      this.#selAnchor = null;
      callback?.(null, true, callbackUserData);
      return;
    }
    if (action.kind === 'selectAll' || action.kind === 'select' || action.kind === 'cancel') {
      const done = this.#runBlockAction(action.kind);
      callback?.(null, done, callbackUserData);
      return;
    }
    if (
      action.kind === 'tableMove' ||
      action.kind === 'tableBlock' ||
      action.kind === 'tableBlockExtend'
    ) {
      const done = this.#runTableAction(action);
      if (done && action.kind === 'tableMove') this.#modified = this.#modified || false;
      callback?.(null, done, callbackUserData);
      return;
    }
    if (action.kind === 'tableEdit' || action.kind === 'tableMerge') {
      const done =
        action.kind === 'tableMerge'
          ? this.#runTableMerge(actionID)
          : this.#runTableEdit(actionID, action);
      if (done) this.#modified = true;
      callback?.(null, done, callbackUserData);
      return;
    }
    if (action.kind === 'selectCtrl') {
      const done = this.#runSelectCtrl();
      callback?.(null, done, callbackUserData);
      return;
    }
    if (action.kind === 'autoNumber') {
      const { list, para, pos } = this.#cursor;
      let ok = false;
      try {
        const raw = this.#doc.insertAutoNumberAtCursor(list, para, pos, action.numberKind);
        ok = parseJson(raw, { ok: false }).ok !== false;
      } catch (e) {
        console.warn(`[hwpctrl] Run("${actionID}") 실패:`, e);
      }
      if (ok) {
        this.#ctrls = null;
        this.#modified = true;
        this.#clearSelection();
        this.#cursor = { list, para, pos: pos + CONTROL_CODE_UNITS };
      }
      callback?.(null, ok, callbackUserData);
      return;
    }
    if (action.kind === 'objectLock') {
      const done = this.#runObjectLock(action);
      if (done) this.#modified = true;
      callback?.(null, done, callbackUserData);
      return;
    }
    if (action.kind === 'objectMove' || action.kind === 'objectTextEdit') {
      const done = this.#runObjectAction(action);
      callback?.(null, done, callbackUserData);
      return;
    }
    if (action.kind === 'breakPara' || action.kind === 'break') {
      const done = this.#runBreakPara(actionID, action.breakKind);
      if (done) this.#modified = true;
      callback?.(null, done, callbackUserData);
      return;
    }
    if (action.kind === 'insert') {
      const done = this.#runInsertAction(actionID, action);
      if (done) this.#modified = true;
      callback?.(null, done, callbackUserData);
      return;
    }
    if (action.kind === 'delete') {
      const done = this.#runDeleteAction(actionID, action);
      if (done) this.#modified = true;
      callback?.(null, done, callbackUserData);
      return;
    }
    const ok = action.kind.startsWith('para')
      ? this.#runParaAction(actionID, action)
      : this.#runCharAction(actionID, action);
    if (ok) this.#modified = true;
    callback?.(null, ok, callbackUserData);
  }

  /** 규격 §8.3.67 — 이벤트 등록. 발화는 아직 없다. */
  AddEventListener(eventType, listener) {
    if (!this.#listeners.has(eventType)) this.#listeners.set(eventType, []);
    this.#listeners.get(eventType).push(listener);
  }

  // ── 내부 ──

  /**
   * 문서를 연 직후의 캐럿. 한글은 **문서에 저장된 캐럿 자리**에서 시작한다
   * (영수증 서식은 `list=292`, 즉 마지막으로 편집한 셀이었다).
   */
  #resetForNewDocument() {
    this.#listModel = null;
    this.#sections = null;
    this.#ctrls = null;
    this.#modified = false;
    this.#clearSelection();
    const stored = parseJson(this.#doc?.getStoredCaret?.() ?? '', null);
    this.#cursor =
      stored && typeof stored.list === 'number'
        ? { list: storedListToRuntime(stored.list), para: stored.para, pos: stored.pos }
        : { list: 0, para: 0, pos: 0 };
  }

  /** 리스트 표는 문서가 바뀌지 않는 한 그대로다 — 호출마다 다시 만들지 않는다. */
  #cursorModel() {
    if (this.#listModel) return this.#listModel;
    const raw = parseJson(this.#doc?.getCursorModel?.() ?? '', null);
    const model = raw ?? { listCount: 1, root: { paraCount: 1, topPos: 0, endPara: 0, endPos: 0 }, lists: [] };
    model.byId = new Map((model.lists ?? []).map((l) => [l.listId, l]));
    this.#listModel = model;
    return model;
  }

  #topOfFile() {
    return { list: 0, para: 0, pos: this.#cursorModel().root.topPos ?? 0 };
  }

  #clearSelection() {
    this.#selectionMode = SELECTION_NONE;
    this.#selection = null;
    this.#selAnchor = null;
    this.#tableBlock = null;
    this.#selectedObject = null;
  }

  /**
   * 이동 액션 한 번. 보통 이동은 블록을 풀고, 선택 확장 이동은 **닻에서 여기까지**를 잡는다.
   *
   * 닻은 `MovePos` 가 부르는 `#clearSelection` 이 지우므로 **부르기 전에 챙긴다**.
   */
  /**
   * 블록을 잡는 세 액션.
   *
   * - `SelectAll` 리스트 전체. 시작은 캐럿의 처음이 아니라 **블록의 처음**이다 — 본문
   *   첫 문단은 앞머리 개체를 담을 수 있어서 72 가 아니라 16 이다(코어 `selectStart`).
   * - `Select`(F3) 선택 모드를 켠다. 켜져 있으면 한 단계 넓힌다 — 블록이 없으면 지금 단어,
   *   있으면 리스트 전체. 모드가 켜진 동안 보통 이동과 `SetPos` 도 블록을 늘린다.
   * - `Cancel` 모드도 블록도 끈다. 캐럿은 있던 자리에 남는다.
   */
  #runBlockAction(kind) {
    if (kind === 'cancel') {
      this.#selectMode = false;
      this.#clearSelection();
      return true;
    }
    const { list } = this.#cursor;
    if (kind === 'selectAll') {
      this.#selectWholeList(list);
      return true;
    }
    // select
    if (!this.#selectMode) {
      this.#selectMode = true;
      this.#selAnchor = { ...this.#cursor };
      return true;
    }
    if (this.#selection) {
      this.#selectWholeList(list);
      this.#selAnchor = { ...this.#selection.start };
      return true;
    }
    const starts = this.#wordStarts();
    const pos = this.#cursor.pos;
    const from = starts.filter((s) => s <= pos).pop() ?? starts[0];
    const to = starts.find((s) => s > pos) ?? this.#paraBounds(list, this.#cursor.para).end;
    this.#selAnchor = { list, para: this.#cursor.para, pos: from };
    this.#selectionMode = SELECTION_NORMAL;
    this.#selection = {
      start: { list, para: this.#cursor.para, pos: from },
      end: { list, para: this.#cursor.para, pos: to },
    };
    this.#cursor = { list, para: this.#cursor.para, pos: to };
    return true;
  }

  /**
   * 지우기 액션 — 블록이 있으면 블록을, 없으면 캐럿에서 정해진 데까지 지운다(전부 실측).
   *
   * | | 지우는 범위 | 캐럿 |
   * | --- | --- | --- |
   * | `Delete` | 다음 한 글자 | 제자리 |
   * | `DeleteBack` | 앞의 한 글자 | 지운 만큼 뒤로 |
   * | `DeleteWord` | 지금 단어의 끝까지 | 제자리 |
   * | `DeleteWordBack` | 앞 단어의 처음까지 | 그 처음 |
   *
   * 문단 끝에서 `Delete` 는 아무 일도 하지 않는다 — 다음 문단을 끌어올리지 않는다(실측).
   */
  #runDeleteAction(actionID, action) {
    const { list, para } = this.#cursor;
    const block = this.#selection;
    let from;
    let to;
    if (block && block.start.list === list && block.start.para === block.end.para) {
      [from, to] = [block.start.pos, block.end.pos];
    } else {
      [from, to] = this.#deleteRange(action.to);
    }
    if (from >= to) return false;
    let ok = false;
    try {
      const raw = this.#doc.deleteAtCursor(list, para, from, to);
      ok = parseJson(raw, { ok: false }).ok !== false;
    } catch (e) {
      console.warn(`[hwpctrl] Run("${actionID}") 실패:`, e);
      return false;
    }
    if (!ok) return false;
    this.#clearSelection();
    this.#cursor = { list, para, pos: from };
    return true;
  }

  /**
   * 개체 사이를 오가고, 개체가 담은 글 안으로 들어간다.
   *
   * 개체를 고르면 `SelectionMode` 4 에 캐럿이 `(문단, 8 × 컨트롤 번호)` 다. 앞뒤 이동은
   * 문서 순서로 돌아간다(끝에서 처음으로 감긴다). 고른 개체가 없으면 첫 개체부터다.
   */
  #runObjectAction(action) {
    const objects = parseJson(this.#doc?.getObjects?.() ?? '', null);
    if (!Array.isArray(objects) || !objects.length) return false;

    if (action.kind === 'objectTextEdit') {
      const here = this.#selectedObject;
      if (!here || here.listId == null) return false;
      this.#selectionMode = SELECTION_NONE;
      this.#selection = null;
      this.#cursor = { list: here.listId, para: 0, pos: 0 };
      return true;
    }

    const at = this.#selectedObject
      ? objects.findIndex(
          (o) =>
            o.para === this.#selectedObject.para &&
            o.controlIndex === this.#selectedObject.controlIndex,
        )
      : -1;
    const next = objects[(at + action.step + objects.length * 2) % objects.length];
    this.#selectObject(next);
    return true;
  }

  /**
   * `SelectCtrlFront` — 개체를 하나씩 앞으로 고른다.
   *
   * - 고른 개체가 없으면 **캐럿 자리부터**(같은 자리 포함) 첫 개체.
   * - 고른 개체가 있으면 그 **다음** 개체.
   * - 더 없으면 **고르기를 푼다**(모드 0). 캐럿은 그대로.
   *
   * 대상은 본문 층의 개체 중 **잠기지 않은 것**이다. 잠긴 것을 건너뛰는 것은 실측이고
   * (표 열둘 중 잠긴 셋만 빠진다), 처음에 "글 앞으로 놓인 개체도 빠진다"고 본 것은 **오독**
   * 이었다 — 그 개체는 캐럿(문단 시작으로 밀린 자리)보다 앞에 있어서 안 걸린 것뿐이다.
   */
  #runSelectCtrl() {
    const eligible = this.#ctrlChain().filter(
      (c) => c.location.list === 0 && c.CtrlCh === 11 && !c.Properties.toObject().Lock,
    );
    const anchorOf = (c) => c.GetAnchorPos().toObject();
    const here = this.#selectedObject;
    let target;
    if (here) {
      const at = eligible.findIndex(
        (c) => c.location.para === here.para && c.location.controlIndex === here.controlIndex,
      );
      target = at >= 0 ? eligible[at + 1] : undefined;
    } else {
      const { para, pos } = this.#cursor;
      target = eligible.find((c) => {
        const a = anchorOf(c);
        return a.Para > para || (a.Para === para && a.Pos >= pos);
      });
    }
    if (!target) {
      // 더 고를 것이 없으면 푼다 — 캐럿은 그대로다.
      this.#selectedObject = null;
      this.#selectionMode = SELECTION_NONE;
      this.#selection = null;
      return true;
    }
    const at = anchorOf(target);
    this.#selectedObject = { ...target.location, kind: null };
    this.#selectionMode = SELECTION_OBJECT;
    this.#selection = null;
    this.#selAnchor = null;
    this.#cursor = { list: 0, para: at.Para, pos: at.Pos };
    return true;
  }

  /**
   * `ShapeObjLock`(고른 개체 잠그기) · `ShapeObjUnlockAll`(본문 전체 풀기).
   *
   * 둘 다 끝나면 **고르기가 풀린다**(모드 0). 캐럿은 그 개체 자리에 그대로 남는다 — 실측이다.
   * 잠그기는 고른 개체가 있어야 한다. 풀기는 고른 것이 없어도 된다.
   */
  #runObjectLock(action) {
    const ALL = 0xffffffff;
    let para = ALL;
    let ctrl = ALL;
    if (!action.all) {
      const here = this.#selectedObject;
      if (!here) return false;
      para = here.para;
      ctrl = here.controlIndex;
    }
    try {
      const raw = this.#doc.setControlLock(para, ctrl, action.locked);
      if (parseJson(raw, { ok: false }).ok === false) return false;
    } catch (e) {
      console.warn('[hwpctrl] 개체 잠금 실패:', e);
      return false;
    }
    this.#ctrls = null; // 잠금 값이 바뀌었으니 사슬을 다시 읽는다.
    this.#selectedObject = null;
    this.#selectionMode = SELECTION_NONE;
    this.#selection = null;
    return true;
  }

  /** 개체 하나를 고른 상태로 만든다 — 모드 4, 캐럿은 그 개체의 자리. */
  #selectObject(obj) {
    this.#selectedObject = obj;
    this.#selectionMode = SELECTION_OBJECT;
    this.#selection = null;
    this.#selAnchor = null;
    this.#tableBlock = null;
    this.#cursor = { list: 0, para: obj.para, pos: obj.controlIndex * CONTROL_CODE_UNITS };
  }

  /**
   * 문단을 캐럿 자리에서 가른다. 캐럿은 새 문단의 처음으로 간다.
   *
   * `breakKind` 가 있으면 새 문단이 나누기 표식까지 진다 — 그러면 그 문단의 시작 자리가
   * 표식만큼 뒤로 밀리는데(`colDef` 8, `section` 16) 캐럿은 그 밀린 시작에 선다.
   */
  #runBreakPara(actionID, breakKind) {
    const { list, para, pos } = this.#cursor;
    let landed = null;
    try {
      const raw = breakKind
        ? this.#doc.breakAtCursor(list, para, pos, breakKind)
        : this.#doc.splitParaAtCursor(list, para, pos);
      const res = parseJson(raw, { ok: false });
      if (res.ok === false) return false;
      // 나누기는 캐럿 자리를 코어가 함께 준다 — 규칙이 한 곳에만 있게 한다.
      if (res.para != null) landed = { para: res.para, pos: res.pos };
    } catch (e) {
      console.warn(`[hwpctrl] Run("${actionID}") 실패:`, e);
      return false;
    }
    this.#listModel = null; // 문단이 늘었다 — 리스트 표의 문단 수가 달라진다
    this.#sections = null; // 구역도 늘 수 있다(BreakSection)
    this.#ctrls = null; // 표식 컨트롤이 늘 수 있다
    this.#clearSelection();
    this.#cursor = landed
      ? { list, para: landed.para, pos: landed.pos }
      : { list, para: para + 1, pos: this.#paraBounds(list, para + 1).start };
    return true;
  }

  /**
   * 빈칸 하나를 캐럿 자리에 끼운다. 캐럿은 끼운 만큼 뒤로 간다(한 칸).
   *
   * 블록이 있으면 한글은 블록을 지우고 끼우겠지만 여기서는 아직 그 경우를 다루지 않는다.
   */
  #runInsertAction(actionID, action) {
    const { list, para, pos } = this.#cursor;
    let ok = false;
    try {
      const raw = this.#doc.insertTextAtCursor(list, para, pos, action.text);
      ok = parseJson(raw, { ok: false }).ok !== false;
    } catch (e) {
      console.warn(`[hwpctrl] Run("${actionID}") 실패:`, e);
      return false;
    }
    if (!ok) return false;
    this.#clearSelection();
    // 글자 수가 아니라 **스트림 칸 수**만큼 민다 — 탭 하나가 8칸이다(실측: 3 → 11).
    const units = [...action.text].reduce((n, ch) => n + (ch === '\t' ? CONTROL_CODE_UNITS : 1), 0);
    this.#cursor = { list, para, pos: pos + units };
    return true;
  }

  /** 지우기 액션 하나가 덮는 범위. 캐럿을 기준으로 앞뒤 눈금을 찾는다. */
  #deleteRange(to) {
    const pos = this.#cursor.pos;
    if (to === 'blockOnly') return [pos, pos]; // 블록이 없으면 지울 것이 없다
    if (to === 'nextChar') return [pos, this.#stepCaret(1)];
    if (to === 'prevChar') return [this.#stepCaret(-1), pos];
    if (to === 'nextWord') {
      const starts = this.#wordStarts();
      const next = starts.find((s) => s > pos);
      return [pos, next ?? this.#paraBounds(this.#cursor.list, this.#cursor.para).end];
    }
    // prevWord
    const starts = this.#wordStarts();
    const prev = starts.filter((s) => s < pos).pop();
    return [prev ?? this.#paraBounds(this.#cursor.list, this.#cursor.para).start, pos];
  }

  /**
   * 표 셀 이동과 셀 블록. 캐럿이 셀 안에 없으면 아무 일도 하지 않는다.
   *
   * 이동은 전부 실측이다 — 좌우는 문서 순서로 한 칸(줄을 넘어간다), 위아래는 같은 열의 이웃
   * 줄, `TableColBegin`·`TableColEnd` 는 그 줄의 첫 칸·끝 칸이다. 표 끝에서는 제자리.
   *
   * 블록은 캐럿이 가는 자리와 `SelectionMode` 만 관측된다(`GetSelectedPos` 는 `result:false`).
   * 한 칸이면 3, 줄·열로 넓히면 19 이고 캐럿은 그 줄·열의 **마지막 칸**에 선다.
   */
  #runTableAction(action) {
    const here = this.#cellOf(this.#cursor.list);
    if (!here) return false;
    const siblings = this.#cellsOfSameTable(here);
    const at = (row, col) =>
      siblings.find((c) => c.row === row && c.col === col) ?? null;

    if (action.kind === 'tableBlockExtend') {
      const alreadyExtending = this.#selectionMode === SELECTION_TABLE_EXTEND;
      this.#selectionMode = SELECTION_TABLE_EXTEND;
      this.#selection = null;
      this.#selAnchor = null;
      // `Extend` 를 이미 켠 채 다시 걸면 표 끝까지 넓어진다. `ExtendAbs` 는 켜기만 한다.
      const to = !action.abs && alreadyExtending ? siblings[siblings.length - 1] : here;
      this.#tableBlock = { from: here, to };
      this.#cursor = { list: to.listId, para: 0, pos: 0 };
      return true;
    }

    if (action.kind === 'tableMove') {
      if (action.to === 'nextOrAppend' && siblings.indexOf(here) === siblings.length - 1) {
        // 마지막 칸이면 줄을 붙이고 그 첫 칸으로 간다.
        return this.#runTableEdit('TableRightCellAppend', { op: 'appendRowAtEnd' });
      }
      const target = this.#tableMoveTarget(action.to, here, siblings, at);
      if (!target || target === here) return true; // 표 가장자리 — 제자리
      this.#clearSelection();
      // **앞으로 가면 그 칸의 처음, 뒤로 가면 그 칸의 끝**이다(Tab·Shift+Tab 과 같다).
      // 실측: 9 → 오른쪽 10/0/0(끝은 24), 9 → 왼쪽 8/0/0·7/0/19, 9 → 위 6/0/2(끝이 2).
      const back = siblings.indexOf(target) < siblings.indexOf(here);
      const para = back ? target.paraCount - 1 : 0;
      const pos = back ? this.#paraBounds(target.listId, para).end : 0;
      this.#cursor = { list: target.listId, para, pos };
      return true;
    }

    // tableBlock — 줄 블록은 그 줄 전체, 열 블록은 그 열 전체다(캐럿 칸부터가 아니다).
    let first = here;
    let last = here;
    if (action.span === 'row') {
      const inRow = siblings.filter((c) => c.row === here.row);
      [first, last] = [inRow[0] ?? here, inRow[inRow.length - 1] ?? here];
    } else if (action.span === 'col') {
      const inCol = siblings.filter((c) => c.col === here.col);
      [first, last] = [inCol[0] ?? here, inCol[inCol.length - 1] ?? here];
    }
    this.#selectionMode = action.span === 'cell' ? SELECTION_TABLE : SELECTION_TABLE_EXTEND;
    this.#selection = null;
    this.#selAnchor = null;
    // 블록이 덮은 격자 범위는 여기서만 안다 — 오라클도 `GetSelectedPos` 로는 안 보여 준다.
    // `TableMergeCell` 이 이 값을 쓴다.
    this.#tableBlock = { from: first, to: last };
    this.#cursor = { list: last.listId, para: 0, pos: 0 };
    return true;
  }

  /**
   * 셀 블록을 하나로 합친다. 캐럿은 합쳐진 칸(블록의 첫 칸)에 선다.
   *
   * 블록이 없으면 아무 일도 하지 않는다 — 한 칸만 잡고 합칠 것은 없다.
   */
  #runTableMerge(actionID) {
    const block = this.#tableBlock;
    if (!block || block.from === block.to) return false;
    const first = block.from;
    const table = {
      hostListId: first.hostListId,
      sectionIndex: first.sectionIndex,
      hostPara: first.hostPara,
      controlIndex: first.controlIndex,
    };
    let ok = false;
    try {
      const raw = this.#doc.tableMergeAtCursor(first.listId, block.to.row, block.to.col);
      ok = parseJson(raw, { ok: false }).ok !== false;
    } catch (e) {
      console.warn(`[hwpctrl] Run("${actionID}") 실패:`, e);
      return false;
    }
    if (!ok) return false;
    this.#listModel = null;
    this.#sections = null;
    this.#clearSelection();
    this.#tableBlock = null;
    const target = (this.#cursorModel().lists ?? []).find(
      (l) => this.#sameTable(l, table) && l.row === first.row && l.col === first.col,
    );
    if (target) this.#cursor = { list: target.listId, para: 0, pos: 0 };
    return true;
  }

  /**
   * 표에 줄·열을 끼우거나 지운다. 캐럿이 어디에 서는지는 전부 실측이다.
   *
   * | 액션 | 캐럿 |
   * | --- | --- |
   * | `TableInsert*` | **자기 칸을 따라간다**(위·왼쪽에 끼우면 그만큼 밀린 자리) |
   * | `TableDeleteRow` | 지운 줄 자리의 **첫 칸** |
   * | `TableDeleteColumn` | **첫 줄**의, 지운 열 자리 |
   *
   * 표가 바뀌면 리스트 번호가 통째로 다시 매겨지므로 **모델을 버리고 다시 읽는다**.
   */
  #runTableEdit(actionID, action) {
    const here = this.#cellOf(this.#cursor.list);
    if (!here) return false;
    const table = {
      hostListId: here.hostListId,
      sectionIndex: here.sectionIndex,
      hostPara: here.hostPara,
      controlIndex: here.controlIndex,
    };
    const want = this.#caretAfterTableEdit(action.op, here);
    let ok = false;
    try {
      const raw = this.#doc.tableEditAtCursor(this.#cursor.list, action.op);
      ok = parseJson(raw, { ok: false }).ok !== false;
    } catch (e) {
      console.warn(`[hwpctrl] Run("${actionID}") 실패:`, e);
      return false;
    }
    if (!ok) return false;

    this.#listModel = null; // 격자가 바뀌었다 — 리스트 표를 다시 만든다
    this.#sections = null;
    this.#clearSelection();
    const cells = (this.#cursorModel().lists ?? []).filter(
      (l) => this.#sameTable(l, table),
    );
    const lastRow = Math.max(...cells.map((c) => c.row));
    const lastCol = Math.max(...cells.map((c) => c.col));
    const row = Math.min(want.row, lastRow);
    const col = Math.min(want.col, lastCol);
    const target = cells.find((c) => c.row === row && c.col === col) ?? cells[0];
    if (target) this.#cursor = { list: target.listId, para: 0, pos: 0 };
    return true;
  }

  /** 표를 고친 **뒤** 캐럿이 서야 할 격자 자리. */
  #caretAfterTableEdit(op, here) {
    if (op === 'insertRowAbove') return { row: here.row + 1, col: here.col };
    if (op === 'insertColLeft') return { row: here.row, col: here.col + 1 };
    // 줄 덧붙임은 끼우기와 자리는 같고 **캐럿만 새 줄로** 간다(같은 칸).
    if (op === 'appendRow') return { row: here.row + 1, col: here.col };
    // 마지막 칸에서 `TableRightCellAppend` — 새 줄의 **첫 칸**으로 간다.
    if (op === 'appendRowAtEnd') return { row: here.row + 1, col: 0 };
    if (op === 'insertRowBelow' || op === 'insertColRight' || op.startsWith('split')) {
      return { row: here.row, col: here.col };
    }
    if (op === 'deleteRow') return { row: here.row, col: 0 };
    return { row: 0, col: here.col }; // deleteCol
  }

  /** 표 이동 하나가 가리키는 셀. 갈 곳이 없으면 `null`(제자리). */
  #tableMoveTarget(to, here, siblings, at) {
    if (to === 'next' || to === 'prev' || to === 'nextOrAppend') {
      const step = to === 'prev' ? -1 : 1;
      const idx = siblings.indexOf(here) + step;
      return siblings[idx] ?? null;
    }
    if (to === 'down') return at(here.row + here.rowSpan, here.col);
    if (to === 'up') return here.row === 0 ? null : at(here.row - 1, here.col);
    const inRow = siblings.filter((c) => c.row === here.row);
    return (to === 'rowBegin' ? inRow[0] : inRow[inRow.length - 1]) ?? null;
  }

  /** 그 리스트가 표 셀이면 격자 정보를, 아니면 `null`. */
  #cellOf(list) {
    const entry = this.#cursorModel().byId.get(list);
    return entry && entry.isCell && typeof entry.row === 'number' ? entry : null;
  }

  /**
   * 같은 표에 속한 셀들 — 문서 순서 그대로.
   *
   * `hostPara` 는 **구역 안 번호**라 구역이 여럿이면 다른 구역의 표와 겹친다. `sectionIndex`
   * 까지 봐야 갈린다.
   */
  #cellsOfSameTable(cell) {
    return (this.#cursorModel().lists ?? []).filter((l) => this.#sameTable(l, cell));
  }

  /** 두 리스트가 같은 표의 셀인가. */
  #sameTable(a, b) {
    return (
      a.isCell &&
      typeof a.row === 'number' &&
      a.hostListId === b.hostListId &&
      a.sectionIndex === b.sectionIndex &&
      a.hostPara === b.hostPara &&
      a.controlIndex === b.controlIndex
    );
  }

  /** 리스트 하나를 통째로 블록으로 잡고 캐럿을 그 끝에 놓는다. */
  #selectWholeList(list) {
    const last = this.#listParaCount(list) - 1;
    const head = this.#paraBounds(list, 0);
    const tail = this.#paraBounds(list, last);
    this.#selectionMode = SELECTION_NORMAL;
    this.#selection = {
      start: { list, para: 0, pos: head.selectStart },
      end: { list, para: last, pos: tail.end },
    };
    this.#cursor = { list, para: last, pos: tail.end };
  }

  #runMoveAction(action) {
    // 선택 모드(F3)가 켜져 있으면 보통 이동도 블록을 늘린다.
    const extending = action.sel || this.#selectMode;
    const anchor = extending ? (this.#selAnchor ?? { ...this.#cursor }) : null;
    const wasSelectMode = this.#selectMode;
    const moved =
      action.kind === 'movePara'
        ? this.#moveParagraph(action.to)
        : this.MovePos(action.moveID, 0, 0);
    if (!extending) return moved;

    this.#selectMode = wasSelectMode;
    this.#applyExtendedSelection(anchor);
    return moved;
  }

  /** 닻에서 지금 캐럿까지를 블록으로 만든다. 겹치거나 리스트를 넘으면 블록이 없다. */
  #applyExtendedSelection(anchor) {
    const cur = this.#cursor;
    this.#selAnchor = anchor;
    if (cur.list !== anchor.list || (cur.para === anchor.para && cur.pos === anchor.pos)) {
      this.#selectionMode = SELECTION_NONE;
      this.#selection = null;
      return;
    }
    const ordered =
      anchor.para < cur.para || (anchor.para === cur.para && anchor.pos < cur.pos)
        ? [anchor, cur]
        : [cur, anchor];
    this.#selectionMode = SELECTION_NORMAL;
    this.#selection = { start: { ...ordered[0] }, end: { ...ordered[1] } };
  }

  /**
   * 문단 단위 이동 — 전부 실측이다(문단 4개짜리 셀).
   *
   * - `nextBegin` 다음 문단의 처음. 마지막 문단에서는 **아예 안 움직인다**(3/1 → 3/1).
   * - `prevBegin` **지금 문단의 처음**, 이미 거기면 앞 문단의 처음(2/1 → 2/0 → 1/0).
   * - `prevEnd` 앞 문단의 끝(2/1 도 2/0 도 1/1). 첫 문단에서는 그 문단의 처음.
   */
  #moveParagraph(to) {
    const { list, para, pos } = this.#cursor;
    this.#clearSelection();
    const count = this.#listParaCount(list);
    const at = (p) => this.#paraBounds(list, p);
    if (to === 'nextBegin') {
      // 마지막 문단에서는 **제자리다** — 그 문단의 처음으로 끌어내리지 않는다(3/1 → 3/1).
      if (para + 1 >= count) return true;
      this.#cursor = { list, para: para + 1, pos: at(para + 1).start };
      return true;
    }
    if (to === 'prevBegin') {
      const here = at(para).start;
      if (pos > here) {
        this.#cursor = { list, para, pos: here };
        return true;
      }
      const prev = Math.max(para - 1, 0);
      this.#cursor = { list, para: prev, pos: at(prev).start };
      return true;
    }
    // prevEnd
    if (para === 0) {
      this.#cursor = { list, para: 0, pos: at(0).start };
      return true;
    }
    this.#cursor = { list, para: para - 1, pos: at(para - 1).end };
    return true;
  }

  /**
   * 캐럿을 한 눈금 옮긴 위치. 눈금은 코어가 준다 — 글자마다 하나, 누름틀은 시작 코드 앞과
   * 내용 시작에 하나씩, 문단 끝에 하나. 끝에서 더 가려 하면 제자리다.
   */
  #stepCaret(direction) {
    const stops = parseJson(
      this.#doc?.getCaretStops?.(this.#cursor.list, this.#cursor.para) ?? '',
      null,
    );
    if (!Array.isArray(stops) || !stops.length) return this.#cursor.pos;
    const pos = this.#cursor.pos;
    if (direction > 0) {
      return stops.find((s) => s > pos) ?? stops[stops.length - 1];
    }
    const before = stops.filter((s) => s < pos);
    return before.length ? before[before.length - 1] : stops[0];
  }

  /** 단어가 시작하는 자리들. 코어가 스트림 기준으로 셈해 준다. */
  #wordStarts() {
    const raw = this.#doc?.getWordStarts?.(this.#cursor.list, this.#cursor.para);
    const parsed = parseJson(raw ?? '', null);
    return Array.isArray(parsed) && parsed.length ? parsed : [this.#cursor.pos];
  }


  /** 문단 하나의 캐럿 경계. 코어가 앞머리 자리차지 컨트롤까지 셈해 준다. */
  #paraBounds(list, para) {
    const raw = this.#doc?.getParaBounds?.(list, para);
    const parsed = parseJson(raw ?? '', null);
    return {
      start: parsed?.start ?? 0,
      end: parsed?.end ?? 0,
      selectStart: parsed?.selectStart ?? 0,
    };
  }

  /** 그 리스트가 담은 문단 수. */
  #listParaCount(list) {
    const model = this.#cursorModel();
    if (list === 0) return model.root.paraCount ?? 1;
    return model.byId.get(list)?.paraCount ?? 1;
  }

  /** 글자 모양 액션 — 블록이 있어야 한다. */
  #runCharAction(actionID, action) {
    const ranges = this.#selectedRanges();
    if (!ranges.length) {
      console.warn(`[hwpctrl] Run("${actionID}"): 블록이 없다 — 대기 서식은 아직 다루지 않는다`);
      return false;
    }
    const props = this.#charActionProps(action);
    if (!props) return false;
    const json = JSON.stringify(props);
    let ok = true;
    for (const range of ranges) {
      try {
        const raw = this.#doc.applyCharFormatAtCursor(
          range.list,
          range.para,
          range.start,
          range.end,
          json,
        );
        ok = ok && parseJson(raw, { ok: false }).ok !== false;
      } catch (e) {
        console.warn(`[hwpctrl] Run("${actionID}") 실패:`, e);
        ok = false;
      }
    }
    return ok;
  }

  /** 액션 하나가 코어에 넘길 서식 속성. 토글·증감은 지금 값을 읽어서 정한다. */
  #charActionProps(action) {
    if (action.kind === 'char') return action.props;
    if (action.kind === 'charCycle') {
      // 없음 → 위 첨자 → 아래 첨자 → 없음 (실측).
      if (this.CharShape.Item('SuperScript')) return { superscript: false, subscript: true };
      if (this.CharShape.Item('SubScript')) return { superscript: false, subscript: false };
      return { superscript: true, subscript: false };
    }
    const current = this.CharShape.Item(action.item);
    if (action.kind === 'toggle') {
      const next = current ? 0 : 1;
      return { [action.prop]: action.numeric ? next : next === 1 };
    }
    // charStep
    const base = typeof current === 'number' ? current : 0;
    const next = base + action.step;
    return { [action.prop]: action.perLang ? sevenLangs(next) : next };
  }

  /**
   * 문단 모양 액션 — 블록이 덮는 문단들에 건다. 블록이 없으면 캐럿이 있는 문단 하나다
   * (편집기의 상식이지만 오라클로 재지는 않았다 — 시나리오는 블록 있는 경우만 고정한다).
   */
  #runParaAction(actionID, action) {
    const targets = this.#selectedParagraphs();
    const json = JSON.stringify(this.#paraActionProps(action));
    let ok = true;
    for (const target of targets) {
      try {
        const raw = this.#doc.applyParaFormatAtCursor(target.list, target.para, json);
        ok = ok && parseJson(raw, { ok: false }).ok !== false;
      } catch (e) {
        console.warn(`[hwpctrl] Run("${actionID}") 실패:`, e);
        ok = false;
      }
    }
    return ok;
  }

  /** 문단 액션 하나가 코어에 넘길 속성. 증감·토글은 지금 값을 읽어서 정한다. */
  #paraActionProps(action) {
    if (action.kind === 'para') return action.props;
    const shape = this.ParaShape;
    if (action.kind === 'paraToggle') {
      return { [action.prop]: !shape.Item(action.item) };
    }
    const props = {};
    for (const part of action.parts) {
      props[part.prop] = (shape.Item(part.item) ?? 0) + part.step;
    }
    return props;
  }

  /** 문단 서식이 걸릴 문단들. */
  #selectedParagraphs() {
    const ranges = this.#selectedRanges();
    if (ranges.length) {
      return ranges.map((r) => ({ list: r.list, para: r.para }));
    }
    return [{ list: this.#cursor.list, para: this.#cursor.para }];
  }

  /**
   * 서식을 걸 자리들. 글자 블록은 그 범위 하나, 셀 블록은 **그 셀의 모든 문단**이다
   * (오라클 실측: 셀 블록에 `CharShapeItalic` 을 걸면 셀 글자가 기울어진다).
   */
  #selectedRanges() {
    if (this.#selectionMode === SELECTION_TABLE) {
      const entry = this.#cursorModel().byId.get(this.#cursor.list);
      if (!entry) return [];
      return Array.from({ length: entry.paraCount }, (_, para) => ({
        list: this.#cursor.list,
        para,
        start: 0,
        end: WHOLE_PARAGRAPH,
      }));
    }
    const sel = this.#selection;
    if (!sel || sel.start.list !== sel.end.list) return [];
    if (sel.start.para !== sel.end.para) {
      console.warn('[hwpctrl] 여러 문단에 걸친 블록은 아직 다루지 않는다');
      return [];
    }
    return [
      { list: sel.start.list, para: sel.start.para, start: sel.start.pos, end: sel.end.pos },
    ];
  }

  /**
   * 그 자리가 문서에 실제로 있는가.
   *
   * 없는 리스트도, 없는 문단도 한글은 같은 곳으로 떨군다 — **문서의 시작**(실측: 마지막
   * 리스트 다음 번호·400·문단 9 가 모두 `{0, 0, 문서시작}`). 반환은 그래도 `true` 다.
   */
  #cursorExists(list, para) {
    if (typeof list !== 'number' || typeof para !== 'number' || list < 0 || para < 0) return false;
    const model = this.#cursorModel();
    if (list === 0) return para < (model.root.paraCount ?? 0);
    const entry = model.byId.get(list);
    return Boolean(entry) && para < entry.paraCount;
  }

  /**
   * 문서의 필드 전부 — **OCX 순회 순서로** 돌려준다.
   *
   * 이 층의 모든 순번(`{{n}}`)이 같은 순서를 딛고 서야 한다. `GetFieldList` 만 따로 정렬하면
   * 목록이 말한 순번과 값을 쓰는 자리의 순번이 서로 다른 필드를 가리킨다.
   */
  #fields() {
    try {
      const parsed = parseJson(this.#doc.getFieldList(), []);
      const list = Array.isArray(parsed) ? parsed : (parsed.fields ?? []);
      return ocxFieldOrder(list);
    } catch {
      return [];
    }
  }

  #fieldValue(token) {
    const { name } = splitOccurrence(token);
    try {
      const parsed = parseJson(this.#doc.getFieldValueByName(name), null);
      return parsed?.ok ? parsed.value : '';
    } catch {
      return '';
    }
  }

  /**
   * 이름 변경의 실제 몸통. `renameField` 는 누름틀과 셀 필드를 함께 다룬다 —
   * `updateClickHereProps` 로는 셀 필드가 `{"ok":false}` 로 막혔다.
   *
   * 같은 이름이 여러 번 나오면 오라클은 **전부** 바꾼다(`pt_no` ×2 문서에서 한 번의 호출
   * 뒤 `FieldExist("pt_no")` 가 false). 코어가 그 규칙을 지킨다.
   */
  #renameField(oldname, newname) {
    // 리스트를 줘도 첫 짝만 쓴다 — 오라클 실측(§8.3.36 주석).
    const from = String(oldname ?? '').split(SEP)[0];
    const to = String(newname ?? '').split(SEP)[0];
    if (!from) return false;
    try {
      const raw = this.#doc.renameField(from, to);
      const ok = parseJson(raw, { ok: false }).ok === true;
      if (ok) this.#modified = true;
      return ok;
    } catch (e) {
      console.warn('[hwpctrl] RenameField 실패:', e);
      return false;
    }
  }

  #toBytes(source) {
    if (source instanceof Uint8Array) return source;
    if (source instanceof ArrayBuffer) return new Uint8Array(source);
    return null; // File — 비동기 경로로 넘긴다
  }

  #exportBytes(format, fileName) {
    const wanted = String(format ?? '').toLowerCase();
    const ext = String(fileName ?? '').toLowerCase();
    if (wanted === 'hwpx' || ext.endsWith('.hwpx')) return this.#doc.exportHwpx();
    if (wanted === 'hml' || ext.endsWith('.hml')) return this.#doc.exportHml();
    return this.#doc.exportHwp();
  }

  #download(bytes, fileName) {
    const blob = new Blob([bytes], { type: 'application/x-hwp' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = fileName || 'document.hwp';
    a.click();
    URL.revokeObjectURL(url);
  }
}

/** 하니스·호스트 공통 진입점. */
export function createHwpCtrl(options = {}) {
  return new HwpCtrl(options);
}
