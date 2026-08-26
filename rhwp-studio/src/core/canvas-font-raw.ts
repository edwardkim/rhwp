/**
 * Canvas2D font substitution과 설치 글꼴 presence probe가 공유하는 원시 font descriptor.
 *
 * wasm-bridge가 전역 setter를 패치하기 직전에 native descriptor를 등록한다. 로컬 글꼴 probe는
 * 이 descriptor를 직접 호출해 제품의 fallback 치환을 다시 통과하지 않는다.
 */

type CanvasFontContext = Pick<CanvasRenderingContext2D, 'font'>;

type CanvasFontDescriptor = Pick<PropertyDescriptor, 'get' | 'set'>;

let rawCanvasFontDescriptor: CanvasFontDescriptor | null = null;

function findCanvasFontDescriptor(context: CanvasFontContext): CanvasFontDescriptor | null {
  let prototype = Object.getPrototypeOf(context) as object | null;
  while (prototype) {
    const descriptor = Object.getOwnPropertyDescriptor(prototype, 'font');
    if (descriptor?.get && descriptor.set) return descriptor;
    prototype = Object.getPrototypeOf(prototype) as object | null;
  }
  return null;
}

/** 전역 substitution patch를 설치하기 전의 Canvas2D font descriptor를 한 번만 보존한다. */
export function rememberRawCanvasFontDescriptor(descriptor: PropertyDescriptor): void {
  if (rawCanvasFontDescriptor || !descriptor.get || !descriptor.set) return;
  rawCanvasFontDescriptor = { get: descriptor.get, set: descriptor.set };
}

/** 제품의 전역 font substitution setter를 우회해 정확한 CSS font 문자열을 설정한다. */
export function setRawCanvasFont(context: CanvasFontContext, value: string): void {
  const descriptor = rawCanvasFontDescriptor ?? findCanvasFontDescriptor(context);
  if (descriptor?.set) {
    descriptor.set.call(context, value);
    return;
  }
  // 단순 mock 또는 descriptor가 없는 비표준 Canvas 구현의 호환 경로다.
  context.font = value;
}

/** 테스트 전용: 모듈에 보존된 native descriptor를 초기화한다. */
export function resetRawCanvasFontDescriptorForTests(): void {
  rawCanvasFontDescriptor = null;
}
