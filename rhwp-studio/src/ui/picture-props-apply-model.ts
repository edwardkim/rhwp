import type { CellPathLike, PictureProperties, ShapeProperties } from '@/core/types';

export type PicturePropsObjectType = 'image' | 'shape' | 'line' | 'group' | 'ole';

export type PicturePropsPatch = Record<string, unknown>;

export interface PicturePropsApplyTargetContext {
  sec: number;
  para: number;
  ci: number;
  headerFooter?: {
    outerParaIdx: number;
    outerControlIdx: number;
  };
  cellPath?: CellPathLike;
  innerControlIdx: number;
}

export type PicturePropsApplyTarget =
  | {
      kind: 'cell-shape';
      sec: number;
      para: number;
      cellPath: CellPathLike;
      innerControlIdx: number;
    }
  | {
      kind: 'body-shape';
      sec: number;
      para: number;
      ci: number;
    }
  | {
      kind: 'header-footer-picture';
      sec: number;
      outerParaIdx: number;
      outerControlIdx: number;
      para: number;
      ci: number;
    }
  | {
      kind: 'cell-picture';
      sec: number;
      para: number;
      cellPath: CellPathLike;
      innerControlIdx: number;
    }
  | {
      kind: 'body-picture';
      sec: number;
      para: number;
      ci: number;
    };

interface RawRotationControl {
  value: string;
  disabled: boolean;
}

interface RawFlipControl {
  value: boolean;
  disabled: boolean;
}

interface RawBoxValues {
  left: string;
  top: string;
  right: string;
  bottom: string;
}

export interface PicturePropsApplyForm {
  common: {
    sizeProtect: boolean;
    width: string;
    height: string;
    treatAsChar: boolean;
    textWrap: string;
    horzRelTo: string;
    horzAlign: string;
    horzOffset: string;
    vertRelTo: string;
    vertAlign: string;
    vertOffset: string;
    restrictInPage: boolean;
    allowOverlap: boolean;
    description: string;
  };
  transform: {
    rotation?: RawRotationControl;
    horzFlip?: RawFlipControl;
    vertFlip?: RawFlipControl;
  };
  outerMargin: {
    left?: string;
    top?: string;
    right?: string;
    bottom?: string;
  };
  caption: {
    present: boolean;
    activeIndex: number;
    size: string;
    gap: string;
    includeMargin: boolean;
  };
  line: {
    color?: string;
    width?: string;
    type?: string;
    end?: string;
    arrowStart?: string;
    arrowEnd?: string;
    arrowStartSize?: string;
    arrowEndSize?: string;
  };
  shapeTextBox: {
    marginLeft?: string;
    marginTop?: string;
    marginRight?: string;
    marginBottom?: string;
    verticalAlign?: string;
  };
  shapeCorner: {
    customChecked: boolean;
    customValue?: string;
    activeIndex: number;
  };
  shapeFill: {
    solidChecked?: boolean;
    gradientChecked?: boolean;
    solidColors?: { face: string; pattern: string };
    patternType?: string;
    gradientType?: string;
    gradientAngle?: string;
    gradientCenterX?: string;
    gradientCenterY?: string;
    gradientBlur?: string;
    transparency?: string;
  };
  shapeShadow: {
    present: boolean;
    activeIndex: number;
    color: string;
    offsetX: string;
    offsetY: string;
  };
  image: {
    scale?: { x: string; y: string };
    crop?: RawBoxValues;
    padding?: RawBoxValues;
    effectControlsPresent: boolean;
    selectedEffect?: string;
    brightness?: string;
    contrast?: string;
    transparency?: string;
  };
}

const HWP_PER_MM = 7200 / 25.4;

function numberOr(raw: string | undefined, fallback: number): number {
  return parseFloat(raw ?? '') || fallback;
}

function integerOr(raw: string | undefined, fallback: number): number {
  return parseInt(raw ?? '') || fallback;
}

function mmToHwp(raw: string | undefined): number {
  return Math.round(numberOr(raw, 0) * HWP_PER_MM);
}

/**
 * [Task #6758] 크기 칸의 표시값 — **이 모듈이 서식의 단일 소유자다.**
 *
 * 아래 `addChangedSize` 는 "사용자가 이 칸을 건드렸는가"를 표시값과 견줘 판정한다. 그래서
 * 다이얼로그가 칸을 채우는 서식과 여기 서식이 반드시 같아야 한다 — 갈라지면 판정이 늘
 * "바뀌었다"가 되어 #6758(무변경 확인이 치수를 200 으로 부풀림)이 되살아난다.
 * 두 벌을 두고 가드로 묶는 대신, 다이얼로그가 이 함수를 가져다 쓴다.
 */
export function displayedMm(hwp: number): string {
  return (hwp / HWP_PER_MM).toFixed(2);
}

/**
 * [Task #6758] 크기 전용 — 사용자가 칸을 건드렸을 때만 패치에 싣는다.
 *
 * 종전에는 되돌린 HWPUNIT 과 모델 값을 비교했다(`addChanged`). 그런데 mm 2자리 표시는
 * 저장 단위를 잃는다 — 높이 1 HWPUNIT 은 `"0.00"` 으로 보이고 되돌리면 `0` 이라 모델의
 * `1` 과 달라져, **사용자가 아무것도 안 고쳐도 변경으로 판정돼** 패치에 실렸다. 그 `0` 이
 * 엔진의 최소 크기 클램프(`MIN_SHAPE_SIZE = 200`)에 걸려 가는 선이 200배로 두꺼워졌다.
 *
 * 한글 2024 는 같은 표시 정밀도를 쓰면서도 확인에서 치수를 그대로 둔다(#6758 실측).
 * 입력값이 표시값과 같으면 사용자가 건드리지 않은 것이므로 보내지 않는다.
 *
 * 비교는 **표시 정밀도로 정규화해서** 한다 — 문자열을 그대로 견주면 같은 값의 다른 표기
 * (`"10"` 과 `"10.00"`)가 변경으로 잡힌다.
 */
function untouchedMm(raw: string | undefined, current: number): boolean {
  return numberOr(raw, 0).toFixed(2) === displayedMm(current);
}

function addChangedSize(
  patch: PicturePropsPatch,
  key: string,
  raw: string | undefined,
  current: number,
): void {
  if (untouchedMm(raw, current)) return;
  patch[key] = Math.max(0, mmToHwp(raw));
}

/**
 * [Task #6769] 위치 오프셋 전용 — 판정은 크기와 같고, 0 클램프만 두지 않는다.
 *
 * 오프셋도 같은 mm 2자리를 왕복하므로 무변경 확인에서 값이 흔들린다. 크기와 달리
 * 엔진 클램프가 없어 눈에 보이는 파손은 없지만, `horizontal_offset`/`vertical_offset`
 * 은 `shape_transform_fingerprint`(`object_ops/common.rs`)의 구성 요소다 — 1 HWPUNIT
 * 만 흔들려도 지문이 달라져 #6740 이 지키려던 한컴 원본 `raw_rendering` 이 지워진다.
 *
 * 음수 오프셋이 정당하므로 크기와 달리 `Math.max(0, ...)` 을 쓰지 않는다.
 */
function addChangedOffset(
  patch: PicturePropsPatch,
  key: string,
  raw: string | undefined,
  current: number,
): void {
  if (untouchedMm(raw, current)) return;
  patch[key] = mmToHwp(raw);
}

function hexToColorRef(hex: string): number {
  const value = hex.replace('#', '');
  const red = parseInt(value.substring(0, 2), 16);
  const green = parseInt(value.substring(2, 4), 16);
  const blue = parseInt(value.substring(4, 6), 16);
  return (blue << 16) | (green << 8) | red;
}

function addChanged(
  patch: PicturePropsPatch,
  key: string,
  next: unknown,
  current: unknown,
): void {
  if (next !== current) patch[key] = next;
}

function addAlways(patch: PicturePropsPatch, key: string, value: unknown): void {
  patch[key] = value;
}

function captionFromGrid(index: number): { direction: string; vertAlign: string } {
  const column = index % 3;
  const row = Math.floor(index / 3);
  if (column === 0) {
    return { direction: 'Left', vertAlign: row === 0 ? 'Top' : row === 1 ? 'Center' : 'Bottom' };
  }
  if (column === 2) {
    return { direction: 'Right', vertAlign: row === 0 ? 'Top' : row === 1 ? 'Center' : 'Bottom' };
  }
  return { direction: row <= 1 ? 'Top' : 'Bottom', vertAlign: 'Top' };
}

function appendCommonSize(
  patch: PicturePropsPatch,
  props: PictureProperties,
  form: PicturePropsApplyForm['common'],
): void {
  addChanged(patch, 'sizeProtect', form.sizeProtect, props.sizeProtect ?? false);
  if (form.sizeProtect) return;
  addChangedSize(patch, 'width', form.width, props.width);
  addChangedSize(patch, 'height', form.height, props.height);
}

function appendCommonPosition(
  patch: PicturePropsPatch,
  props: PictureProperties,
  form: PicturePropsApplyForm['common'],
): void {
  addChanged(patch, 'treatAsChar', form.treatAsChar, props.treatAsChar);
  if (form.treatAsChar) return;

  const textWrap = form.horzRelTo === 'TakePlace' ? 'TopAndBottom' : form.textWrap;
  addChanged(patch, 'textWrap', textWrap, props.textWrap);
  if (form.horzRelTo !== 'TakePlace') {
    addChanged(patch, 'horzRelTo', form.horzRelTo, props.horzRelTo);
  }
  addChanged(patch, 'horzAlign', form.horzAlign, props.horzAlign);
  addChangedOffset(patch, 'horzOffset', form.horzOffset, props.horzOffset);
  addChanged(patch, 'vertRelTo', form.vertRelTo, props.vertRelTo);
  addChanged(patch, 'vertAlign', form.vertAlign, props.vertAlign);
  addChangedOffset(patch, 'vertOffset', form.vertOffset, props.vertOffset);
  addChanged(patch, 'restrictInPage', form.restrictInPage, props.restrictInPage ?? true);
  addChanged(patch, 'allowOverlap', form.allowOverlap, props.allowOverlap ?? false);
}

function appendTransform(
  patch: PicturePropsPatch,
  props: Pick<PictureProperties, 'rotationAngle' | 'horzFlip' | 'vertFlip'> | ShapeProperties,
  form: PicturePropsApplyForm['transform'],
): void {
  if (form.rotation && !form.rotation.disabled) {
    addChanged(patch, 'rotationAngle', integerOr(form.rotation.value, 0), props.rotationAngle ?? 0);
  }
  if (form.horzFlip && !form.horzFlip.disabled) {
    addChanged(patch, 'horzFlip', form.horzFlip.value, Boolean(props.horzFlip));
  }
  if (form.vertFlip && !form.vertFlip.disabled) {
    addChanged(patch, 'vertFlip', form.vertFlip.value, Boolean(props.vertFlip));
  }
}

function appendOuterMargin(
  patch: PicturePropsPatch,
  props: PictureProperties,
  form: PicturePropsApplyForm['outerMargin'],
): void {
  if (form.left !== undefined) addChanged(patch, 'outerMarginLeft', mmToHwp(form.left), props.outerMarginLeft ?? 0);
  if (form.right !== undefined) addChanged(patch, 'outerMarginRight', mmToHwp(form.right), props.outerMarginRight ?? 0);
  if (form.top !== undefined) addChanged(patch, 'outerMarginTop', mmToHwp(form.top), props.outerMarginTop ?? 0);
  if (form.bottom !== undefined) addChanged(patch, 'outerMarginBottom', mmToHwp(form.bottom), props.outerMarginBottom ?? 0);
}

function appendCaption(
  patch: PicturePropsPatch,
  form: PicturePropsApplyForm['caption'],
): void {
  if (!form.present) return;
  const hasCaption = form.activeIndex >= 0 && form.activeIndex !== 4;
  addAlways(patch, 'hasCaption', hasCaption);
  if (!hasCaption) return;

  const caption = captionFromGrid(form.activeIndex);
  addAlways(patch, 'captionDirection', caption.direction);
  addAlways(patch, 'captionVertAlign', caption.vertAlign);
  addAlways(patch, 'captionWidth', mmToHwp(form.size));
  addAlways(patch, 'captionSpacing', mmToHwp(form.gap));
  addAlways(patch, 'captionIncludeMargin', form.includeMargin);
}

function appendBorder(
  patch: PicturePropsPatch,
  props: Pick<PictureProperties, 'borderColor' | 'borderWidth'> | ShapeProperties,
  form: PicturePropsApplyForm['line'],
): void {
  if (form.color !== undefined) addChanged(patch, 'borderColor', hexToColorRef(form.color), props.borderColor ?? 0);
  if (form.width !== undefined) addChanged(patch, 'borderWidth', mmToHwp(form.width), props.borderWidth ?? 0);
}

function appendShapeLine(
  patch: PicturePropsPatch,
  props: ShapeProperties,
  form: PicturePropsApplyForm['line'],
  includeArrows: boolean,
): void {
  appendBorder(patch, props, form);
  if (form.type !== undefined) addChanged(patch, 'lineType', integerOr(form.type, 0), props.lineType ?? 1);
  if (form.end !== undefined) addChanged(patch, 'lineEndShape', integerOr(form.end, 0), props.lineEndShape ?? 0);
  if (!includeArrows) return;
  if (form.arrowStart !== undefined) addChanged(patch, 'arrowStart', integerOr(form.arrowStart, 0), props.arrowStart ?? 0);
  if (form.arrowEnd !== undefined) addChanged(patch, 'arrowEnd', integerOr(form.arrowEnd, 0), props.arrowEnd ?? 0);
  if (form.arrowStartSize !== undefined) addChanged(patch, 'arrowStartSize', integerOr(form.arrowStartSize, 0), props.arrowStartSize ?? 0);
  if (form.arrowEndSize !== undefined) addChanged(patch, 'arrowEndSize', integerOr(form.arrowEndSize, 0), props.arrowEndSize ?? 0);
}

function appendShapeTextBox(
  patch: PicturePropsPatch,
  props: ShapeProperties,
  form: PicturePropsApplyForm['shapeTextBox'],
): void {
  addChanged(patch, 'tbMarginLeft', mmToHwp(form.marginLeft), props.tbMarginLeft ?? 0);
  addChanged(patch, 'tbMarginRight', mmToHwp(form.marginRight), props.tbMarginRight ?? 0);
  addChanged(patch, 'tbMarginTop', mmToHwp(form.marginTop), props.tbMarginTop ?? 0);
  addChanged(patch, 'tbMarginBottom', mmToHwp(form.marginBottom), props.tbMarginBottom ?? 0);
  addChanged(patch, 'tbVerticalAlign', form.verticalAlign ?? 'Top', props.tbVerticalAlign ?? 'Top');
}

function appendShapeCorner(
  patch: PicturePropsPatch,
  props: ShapeProperties,
  form: PicturePropsApplyForm['shapeCorner'],
): void {
  let roundRate = 0;
  if (form.customChecked && form.customValue !== undefined) {
    roundRate = integerOr(form.customValue, 0);
  } else if (form.activeIndex === 1) {
    roundRate = 20;
  } else if (form.activeIndex === 2) {
    roundRate = 50;
  }
  addChanged(patch, 'roundRate', roundRate, props.roundRate ?? 0);
}

function shapeFillType(form: PicturePropsApplyForm['shapeFill']): string {
  if (form.solidChecked) return 'solid';
  if (form.gradientChecked) return 'gradient';
  return 'none';
}

function appendSolidFill(
  patch: PicturePropsPatch,
  form: PicturePropsApplyForm['shapeFill'],
): void {
  if (!form.solidColors) return;
  addAlways(patch, 'fillBgColor', hexToColorRef(form.solidColors.face));
  addAlways(patch, 'fillPatColor', hexToColorRef(form.solidColors.pattern));
  if (form.patternType !== undefined) {
    addAlways(patch, 'fillPatType', integerOr(form.patternType, -1));
  }
}

function appendGradientFill(
  patch: PicturePropsPatch,
  form: PicturePropsApplyForm['shapeFill'],
): void {
  if (form.gradientType !== undefined) addAlways(patch, 'gradientType', integerOr(form.gradientType, 1));
  if (form.gradientAngle !== undefined) addAlways(patch, 'gradientAngle', integerOr(form.gradientAngle, 0));
  if (form.gradientCenterX !== undefined) addAlways(patch, 'gradientCenterX', integerOr(form.gradientCenterX, 0));
  if (form.gradientCenterY !== undefined) addAlways(patch, 'gradientCenterY', integerOr(form.gradientCenterY, 0));
  if (form.gradientBlur !== undefined) addAlways(patch, 'gradientBlur', integerOr(form.gradientBlur, 0));
}

function appendShapeFill(
  patch: PicturePropsPatch,
  props: ShapeProperties,
  form: PicturePropsApplyForm['shapeFill'],
): void {
  const fillType = shapeFillType(form);
  addChanged(patch, 'fillType', fillType, props.fillType ?? 'none');
  if (fillType === 'solid') appendSolidFill(patch, form);
  if (fillType === 'gradient') appendGradientFill(patch, form);
  if (form.transparency !== undefined && (fillType === 'solid' || fillType === 'gradient')) {
    addAlways(patch, 'fillAlpha', Math.round(integerOr(form.transparency, 0) * 255 / 100));
  }
}

function appendShapeShadow(
  patch: PicturePropsPatch,
  form: PicturePropsApplyForm['shapeShadow'],
): void {
  if (!form.present) return;
  const shadowType = form.activeIndex > 0 ? form.activeIndex : 0;
  addAlways(patch, 'shadowType', shadowType);
  if (shadowType > 0) {
    addAlways(patch, 'shadowColor', hexToColorRef(form.color));
    addAlways(patch, 'shadowOffsetX', mmToHwp(form.offsetX));
    addAlways(patch, 'shadowOffsetY', mmToHwp(form.offsetY));
  } else {
    addAlways(patch, 'shadowOffsetX', 0);
    addAlways(patch, 'shadowOffsetY', 0);
  }
}

function appendOlePatch(
  patch: PicturePropsPatch,
  props: PictureProperties,
  shapeProps: ShapeProperties,
  form: PicturePropsApplyForm,
): void {
  appendOuterMargin(patch, props, form.outerMargin);
  appendCaption(patch, form.caption);
  appendShapeLine(patch, shapeProps, form.line, false);
}

function appendNonOleShapePatch(
  patch: PicturePropsPatch,
  shapeProps: ShapeProperties,
  form: PicturePropsApplyForm,
): void {
  appendShapeTextBox(patch, shapeProps, form.shapeTextBox);
  appendTransform(patch, shapeProps, form.transform);
  appendShapeLine(patch, shapeProps, form.line, true);
  appendShapeCorner(patch, shapeProps, form.shapeCorner);
  appendShapeFill(patch, shapeProps, form.shapeFill);
  appendShapeShadow(patch, form.shapeShadow);
}

function appendImageScale(
  patch: PicturePropsPatch,
  props: PictureProperties,
  form: PicturePropsApplyForm,
): void {
  if (form.common.sizeProtect || !form.image.scale || !(props.originalWidth > 0)) return;
  const scaleX = Math.max(1, Math.min(1000, numberOr(form.image.scale.x, 100)));
  const scaleY = Math.max(1, Math.min(1000, numberOr(form.image.scale.y, 100)));
  const width = Math.round(props.originalWidth * scaleX / 100);
  const height = Math.round(props.originalHeight * scaleY / 100);
  addChanged(patch, 'width', width, props.width);
  addChanged(patch, 'height', height, props.height);
}

function appendImageBox(
  patch: PicturePropsPatch,
  values: RawBoxValues | undefined,
  keys: readonly [string, string, string, string],
  current: readonly [number, number, number, number],
): void {
  if (!values) return;
  addChanged(patch, keys[0], Math.max(0, mmToHwp(values.left)), current[0]);
  addChanged(patch, keys[1], Math.max(0, mmToHwp(values.top)), current[1]);
  addChanged(patch, keys[2], Math.max(0, mmToHwp(values.right)), current[2]);
  addChanged(patch, keys[3], Math.max(0, mmToHwp(values.bottom)), current[3]);
}

function appendImageEffects(
  patch: PicturePropsPatch,
  props: PictureProperties,
  form: PicturePropsApplyForm['image'],
): void {
  if (form.effectControlsPresent && form.selectedEffect !== undefined) {
    const effect = form.selectedEffect === 'Original' ? 'RealPic' : form.selectedEffect;
    addChanged(patch, 'effect', effect, props.effect ?? 'RealPic');
  }
  if (form.brightness !== undefined) {
    const brightness = Math.max(-100, Math.min(100, integerOr(form.brightness, 0)));
    addChanged(patch, 'brightness', brightness, props.brightness ?? 0);
  }
  if (form.contrast !== undefined) {
    const contrast = Math.max(-100, Math.min(100, integerOr(form.contrast, 0)));
    addChanged(patch, 'contrast', contrast, props.contrast ?? 0);
  }
  if (form.transparency !== undefined) {
    const transparency = Math.max(0, Math.min(100, integerOr(form.transparency, 0)));
    addChanged(patch, 'transparency', transparency, props.transparency ?? 0);
  }
}

function appendImagePatch(
  patch: PicturePropsPatch,
  props: PictureProperties,
  form: PicturePropsApplyForm,
): void {
  appendTransform(patch, props, form.transform);
  appendOuterMargin(patch, props, form.outerMargin);
  appendCaption(patch, form.caption);
  appendBorder(patch, props, form.line);
  appendImageScale(patch, props, form);
  appendImageBox(
    patch,
    form.image.crop,
    ['cropLeft', 'cropTop', 'cropRight', 'cropBottom'],
    [props.cropLeft ?? 0, props.cropTop ?? 0, props.cropRight ?? 0, props.cropBottom ?? 0],
  );
  appendImageBox(
    patch,
    form.image.padding,
    ['paddingLeft', 'paddingTop', 'paddingRight', 'paddingBottom'],
    [props.paddingLeft ?? 0, props.paddingTop ?? 0, props.paddingRight ?? 0, props.paddingBottom ?? 0],
  );
  appendImageEffects(patch, props, form.image);
}

export function buildPicturePropsPatch(
  objectType: PicturePropsObjectType,
  props: PictureProperties,
  shapeProps: ShapeProperties | null,
  form: PicturePropsApplyForm,
): PicturePropsPatch {
  const patch: PicturePropsPatch = {};
  appendCommonSize(patch, props, form.common);
  appendCommonPosition(patch, props, form.common);
  addChanged(patch, 'description', form.common.description, props.description);

  if (objectType === 'image') {
    appendImagePatch(patch, props, form);
  } else if (shapeProps) {
    if (objectType === 'ole') appendOlePatch(patch, props, shapeProps, form);
    else appendNonOleShapePatch(patch, shapeProps, form);
  }
  return patch;
}

export function resolvePicturePropsApplyTarget(
  objectType: PicturePropsObjectType,
  context: PicturePropsApplyTargetContext,
): PicturePropsApplyTarget {
  if (objectType !== 'image') {
    if (context.cellPath) {
      return {
        kind: 'cell-shape',
        sec: context.sec,
        para: context.para,
        cellPath: context.cellPath,
        innerControlIdx: context.innerControlIdx,
      };
    }
    return {
      kind: 'body-shape',
      sec: context.sec,
      para: context.para,
      ci: context.ci,
    };
  }

  if (context.headerFooter) {
    return {
      kind: 'header-footer-picture',
      sec: context.sec,
      outerParaIdx: context.headerFooter.outerParaIdx,
      outerControlIdx: context.headerFooter.outerControlIdx,
      para: context.para,
      ci: context.ci,
    };
  }
  if (context.cellPath) {
    return {
      kind: 'cell-picture',
      sec: context.sec,
      para: context.para,
      cellPath: context.cellPath,
      innerControlIdx: context.innerControlIdx,
    };
  }
  return {
    kind: 'body-picture',
    sec: context.sec,
    para: context.para,
    ci: context.ci,
  };
}
