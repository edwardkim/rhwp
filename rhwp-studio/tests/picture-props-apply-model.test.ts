import test from 'node:test';
import assert from 'node:assert/strict';
import type { CellPathLike, PictureProperties, ShapeProperties } from '../src/core/types.ts';
import {
  buildPicturePropsPatch,
  resolvePicturePropsApplyTarget,
  type PicturePropsApplyForm,
  type PicturePropsObjectType,
  type PicturePropsPatch,
  type PicturePropsApplyTarget,
  type PicturePropsApplyTargetContext,
} from '../src/ui/picture-props-apply-model.ts';

function pictureProps(overrides: Partial<PictureProperties> = {}): PictureProperties {
  return {
    width: 2835,
    height: 5669,
    treatAsChar: false,
    vertRelTo: 'Page',
    vertAlign: 'Top',
    horzRelTo: 'Column',
    horzAlign: 'Left',
    vertOffset: 567,
    horzOffset: 283,
    textWrap: 'Square',
    restrictInPage: true,
    allowOverlap: false,
    sizeProtect: false,
    brightness: 0,
    contrast: 0,
    effect: 'RealPic',
    transparency: 0,
    description: 'description',
    rotationAngle: 0,
    horzFlip: false,
    vertFlip: false,
    originalWidth: 1000,
    originalHeight: 800,
    cropLeft: 0,
    cropTop: 0,
    cropRight: 0,
    cropBottom: 0,
    paddingLeft: 0,
    paddingTop: 0,
    paddingRight: 0,
    paddingBottom: 0,
    outerMarginLeft: 0,
    outerMarginTop: 0,
    outerMarginRight: 0,
    outerMarginBottom: 0,
    borderColor: 0,
    borderWidth: 0,
    hasCaption: false,
    captionDirection: 'Bottom',
    captionVertAlign: 'Top',
    captionWidth: 0,
    captionSpacing: 0,
    captionMaxWidth: 0,
    captionIncludeMargin: false,
    ...overrides,
  };
}

function shapeProps(overrides: Partial<ShapeProperties> = {}): ShapeProperties {
  return {
    width: 2835,
    height: 5669,
    treatAsChar: false,
    vertRelTo: 'Page',
    vertAlign: 'Top',
    horzRelTo: 'Column',
    horzAlign: 'Left',
    vertOffset: 567,
    horzOffset: 283,
    textWrap: 'Square',
    sizeProtect: false,
    description: 'description',
    ...overrides,
  };
}

function applyForm(): PicturePropsApplyForm {
  return {
    common: {
      sizeProtect: false,
      width: '10',
      height: '20',
      treatAsChar: false,
      textWrap: 'Square',
      horzRelTo: 'Column',
      horzAlign: 'Left',
      horzOffset: '1',
      vertRelTo: 'Page',
      vertAlign: 'Top',
      vertOffset: '2',
      restrictInPage: true,
      allowOverlap: false,
      description: 'description',
    },
    transform: {},
    outerMargin: {},
    caption: {
      present: false,
      activeIndex: -1,
      size: '',
      gap: '',
      includeMargin: false,
    },
    line: {},
    shapeTextBox: {},
    shapeCorner: {
      customChecked: false,
      activeIndex: -1,
    },
    shapeFill: {},
    shapeShadow: {
      present: false,
      activeIndex: -1,
      color: '',
      offsetX: '',
      offsetY: '',
    },
    image: {
      effectControlsPresent: false,
    },
  };
}

interface PatchFixture {
  name: string;
  objectType: PicturePropsObjectType;
  props?: PictureProperties;
  shapeProps?: ShapeProperties | null;
  update?: (form: PicturePropsApplyForm, props: PictureProperties, shape: ShapeProperties | null) => void;
  expected: PicturePropsPatch;
}

const fixtures: PatchFixture[] = [
  {
    name: 'defensive image snapshot with no optional controls produces an empty patch',
    objectType: 'image',
    expected: {},
  },
  {
    name: 'common size, placement, flags, and description use current conversions and defaults',
    objectType: 'image',
    update(form) {
      Object.assign(form.common, {
        width: '12.5',
        height: '0',
        textWrap: 'Tight',
        horzRelTo: 'Page',
        horzAlign: 'Center',
        horzOffset: '3',
        vertRelTo: 'Para',
        vertAlign: 'Bottom',
        vertOffset: '4',
        restrictInPage: false,
        allowOverlap: true,
        description: 'changed',
      });
    },
    expected: {
      width: 3543,
      height: 0,
      textWrap: 'Tight',
      horzRelTo: 'Page',
      horzAlign: 'Center',
      horzOffset: 850,
      vertRelTo: 'Para',
      vertAlign: 'Bottom',
      vertOffset: 1134,
      restrictInPage: false,
      allowOverlap: true,
      description: 'changed',
    },
  },
  {
    name: 'size protection suppresses common size and image scale updates',
    objectType: 'image',
    update(form) {
      form.common.sizeProtect = true;
      form.common.width = '99';
      form.image.scale = { x: '50', y: '50' };
    },
    expected: { sizeProtect: true },
  },
  {
    name: 'TakePlace forces TopAndBottom and omits horzRelTo',
    objectType: 'image',
    update(form) {
      form.common.textWrap = 'Through';
      form.common.horzRelTo = 'TakePlace';
    },
    expected: { textWrap: 'TopAndBottom' },
  },
  {
    name: 'treat-as-character skips all placement fields',
    objectType: 'image',
    update(form) {
      Object.assign(form.common, {
        treatAsChar: true,
        textWrap: 'Tight',
        horzRelTo: 'Page',
        horzAlign: 'Right',
        horzOffset: '9',
        vertRelTo: 'Para',
        vertAlign: 'Bottom',
        vertOffset: '9',
        restrictInPage: false,
        allowOverlap: true,
      });
    },
    expected: { treatAsChar: true },
  },
  {
    name: 'disabled transform controls preserve their properties',
    objectType: 'image',
    update(form) {
      form.transform = {
        rotation: { value: '45', disabled: true },
        horzFlip: { value: true, disabled: true },
        vertFlip: { value: true, disabled: true },
      };
    },
    expected: {},
  },
  {
    name: 'enabled image transform controls create changed-only fields',
    objectType: 'image',
    update(form) {
      form.transform = {
        rotation: { value: '45', disabled: false },
        horzFlip: { value: true, disabled: false },
        vertFlip: { value: false, disabled: false },
      };
    },
    expected: { rotationAngle: 45, horzFlip: true },
  },
  {
    name: 'OLE keeps margin, caption, and line scope without arrow fields',
    objectType: 'ole',
    shapeProps: shapeProps(),
    update(form) {
      form.outerMargin = { left: '1', top: '2', right: '3', bottom: '4' };
      form.caption = {
        present: true,
        activeIndex: 3,
        size: '5',
        gap: '6',
        includeMargin: true,
      };
      form.line = {
        color: '#112233',
        width: '1',
        type: '2',
        end: '1',
        arrowStart: '3',
        arrowEnd: '4',
      };
    },
    expected: {
      outerMarginLeft: 283,
      outerMarginRight: 850,
      outerMarginTop: 567,
      outerMarginBottom: 1134,
      hasCaption: true,
      captionDirection: 'Left',
      captionVertAlign: 'Center',
      captionWidth: 1417,
      captionSpacing: 1701,
      captionIncludeMargin: true,
      borderColor: 3351057,
      borderWidth: 283,
      lineType: 2,
      lineEndShape: 1,
    },
  },
  {
    name: 'line-style absent textbox controls retain zero and Top normalization',
    objectType: 'line',
    shapeProps: shapeProps({
      tbMarginLeft: 10,
      tbMarginRight: 20,
      tbMarginTop: 30,
      tbMarginBottom: 40,
      tbVerticalAlign: 'Bottom',
      fillType: 'solid',
      roundRate: 20,
    }),
    expected: {
      tbMarginLeft: 0,
      tbMarginRight: 0,
      tbMarginTop: 0,
      tbMarginBottom: 0,
      tbVerticalAlign: 'Top',
      roundRate: 0,
      fillType: 'none',
    },
  },
  {
    name: 'normal shape controls retain always-send shadow keys when values are unchanged',
    objectType: 'shape',
    shapeProps: shapeProps(),
    update(form) {
      form.shapeShadow = {
        present: true,
        activeIndex: 0,
        color: '#000000',
        offsetX: '0',
        offsetY: '0',
      };
    },
    expected: { shadowType: 0, shadowOffsetX: 0, shadowOffsetY: 0 },
  },
  {
    name: 'line snapshot preserves detached shape controls from a reused dialog instance',
    objectType: 'line',
    shapeProps: shapeProps(),
    update(form) {
      form.shapeTextBox = {
        marginLeft: '1',
        marginTop: '2',
        marginRight: '3',
        marginBottom: '4',
        verticalAlign: 'Bottom',
      };
      form.shapeCorner = {
        customChecked: true,
        customValue: '35',
        activeIndex: 0,
      };
      form.shapeFill = {
        solidChecked: true,
        solidColors: { face: '#010203', pattern: '#040506' },
        patternType: '2',
        transparency: '10',
      };
    },
    expected: {
      tbMarginLeft: 283,
      tbMarginRight: 850,
      tbMarginTop: 567,
      tbMarginBottom: 1134,
      tbVerticalAlign: 'Bottom',
      roundRate: 35,
      fillType: 'solid',
      fillBgColor: 197121,
      fillPatColor: 394500,
      fillPatType: 2,
      fillAlpha: 26,
    },
  },
  {
    name: 'non-OLE line controls include arrow and size fields',
    objectType: 'shape',
    shapeProps: shapeProps(),
    update(form) {
      form.line = {
        type: '0',
        end: '2',
        arrowStart: '1',
        arrowEnd: '2',
        arrowStartSize: '3',
        arrowEndSize: '4',
      };
    },
    expected: {
      lineType: 0,
      lineEndShape: 2,
      arrowStart: 1,
      arrowEnd: 2,
      arrowStartSize: 3,
      arrowEndSize: 4,
    },
  },
  {
    name: 'solid fill always sends colors, pattern fallback, and alpha',
    objectType: 'shape',
    shapeProps: shapeProps(),
    update(form) {
      form.shapeFill = {
        solidChecked: true,
        gradientChecked: false,
        solidColors: { face: '#ff0000', pattern: '#00ff00' },
        patternType: '0',
        transparency: '50',
      };
    },
    expected: {
      fillType: 'solid',
      fillBgColor: 255,
      fillPatColor: 65280,
      fillPatType: -1,
      fillAlpha: 128,
    },
  },
  {
    name: 'gradient fill preserves per-control fallback and always-send policy',
    objectType: 'group',
    shapeProps: shapeProps({ fillType: 'gradient' }),
    update(form) {
      form.shapeFill = {
        gradientChecked: true,
        gradientType: '0',
        gradientAngle: '-15',
        gradientCenterX: '25',
        gradientCenterY: '0',
        gradientBlur: '8',
        transparency: '20',
      };
    },
    expected: {
      gradientType: 1,
      gradientAngle: -15,
      gradientCenterX: 25,
      gradientCenterY: 0,
      gradientBlur: 8,
      fillAlpha: 51,
    },
  },
  {
    name: 'disabled shadow always sends type zero and zero offsets',
    objectType: 'shape',
    shapeProps: shapeProps(),
    update(form) {
      form.shapeShadow = {
        present: true,
        activeIndex: 0,
        color: '#123456',
        offsetX: '3',
        offsetY: '4',
      };
    },
    expected: { shadowType: 0, shadowOffsetX: 0, shadowOffsetY: 0 },
  },
  {
    name: 'enabled shadow always sends color and converted offsets',
    objectType: 'shape',
    shapeProps: shapeProps(),
    update(form) {
      form.shapeShadow = {
        present: true,
        activeIndex: 2,
        color: '#123456',
        offsetX: '-1',
        offsetY: '2',
      };
    },
    expected: {
      shadowType: 2,
      shadowColor: 5649426,
      shadowOffsetX: -283,
      shadowOffsetY: 567,
    },
  },
  {
    name: 'caption center always sends hasCaption false without detail fields',
    objectType: 'image',
    update(form) {
      form.caption = {
        present: true,
        activeIndex: 4,
        size: '5',
        gap: '6',
        includeMargin: true,
      };
    },
    expected: { hasCaption: false },
  },
  {
    name: 'image scale overwrites common width and height patch values',
    objectType: 'image',
    update(form) {
      form.common.width = '99';
      form.common.height = '99';
      form.image.scale = { x: '50', y: '25' };
    },
    expected: { width: 500, height: 200 },
  },
  {
    name: 'image geometry, effects, border, and clamped transparency preserve field policy',
    objectType: 'image',
    update(form) {
      form.line = { color: '#abcdef', width: '0.5' };
      form.image = {
        crop: { left: '1', top: '2', right: '3', bottom: '4' },
        padding: { left: '4', top: '3', right: '2', bottom: '1' },
        effectControlsPresent: true,
        selectedEffect: 'GrayScale',
        brightness: '-20',
        contrast: '15',
        transparency: '150',
      };
    },
    expected: {
      borderColor: 15715755,
      borderWidth: 142,
      cropLeft: 283,
      cropTop: 567,
      cropRight: 850,
      cropBottom: 1134,
      paddingLeft: 1134,
      paddingTop: 850,
      paddingRight: 567,
      paddingBottom: 283,
      effect: 'GrayScale',
      brightness: -20,
      contrast: 15,
      transparency: 100,
    },
  },
  {
    name: 'Original picture effect normalizes to existing RealPic without a diff',
    objectType: 'image',
    update(form) {
      form.image.effectControlsPresent = true;
      form.image.selectedEffect = 'Original';
    },
    expected: {},
  },
];

for (const fixture of fixtures) {
  test(fixture.name, () => {
    const shape = fixture.shapeProps === undefined ? null : fixture.shapeProps;
    const props = fixture.props ?? (
      shape ? shape as unknown as PictureProperties : pictureProps()
    );
    const form = applyForm();
    fixture.update?.(form, props, shape);

    assert.deepEqual(
      buildPicturePropsPatch(fixture.objectType, props, shape, form),
      fixture.expected,
    );
  });
}

interface TargetFixture {
  name: string;
  objectType: PicturePropsObjectType;
  context: PicturePropsApplyTargetContext;
  expected: PicturePropsApplyTarget;
}

const cellPath: CellPathLike = [];
const location = { sec: 1, para: 2, ci: 3, innerControlIdx: 4 };
const headerFooter = { outerParaIdx: 5, outerControlIdx: 6 };

const targetFixtures: TargetFixture[] = [
  {
    name: 'shape in a table cell resolves to cell-shape',
    objectType: 'shape',
    context: { ...location, cellPath },
    expected: { kind: 'cell-shape', sec: 1, para: 2, cellPath, innerControlIdx: 4 },
  },
  {
    name: 'OLE in the body resolves through the shape body setter',
    objectType: 'ole',
    context: location,
    expected: { kind: 'body-shape', sec: 1, para: 2, ci: 3 },
  },
  {
    name: 'header-footer image preserves the five lookup indexes',
    objectType: 'image',
    context: { ...location, headerFooter },
    expected: {
      kind: 'header-footer-picture',
      sec: 1,
      outerParaIdx: 5,
      outerControlIdx: 6,
      para: 2,
      ci: 3,
    },
  },
  {
    name: 'image in a table cell resolves to cell-picture',
    objectType: 'image',
    context: { ...location, cellPath },
    expected: { kind: 'cell-picture', sec: 1, para: 2, cellPath, innerControlIdx: 4 },
  },
  {
    name: 'body image resolves to body-picture',
    objectType: 'image',
    context: location,
    expected: { kind: 'body-picture', sec: 1, para: 2, ci: 3 },
  },
  {
    name: 'header-footer marker takes priority over an image cell path',
    objectType: 'image',
    context: { ...location, headerFooter, cellPath },
    expected: {
      kind: 'header-footer-picture',
      sec: 1,
      outerParaIdx: 5,
      outerControlIdx: 6,
      para: 2,
      ci: 3,
    },
  },
];

for (const fixture of targetFixtures) {
  test(fixture.name, () => {
    const actual = resolvePicturePropsApplyTarget(fixture.objectType, fixture.context);
    assert.deepEqual(actual, fixture.expected);
    if (actual.kind === 'cell-shape' || actual.kind === 'cell-picture') {
      assert.equal(actual.cellPath, cellPath, 'cell path identity must be preserved');
    }
  });
}
