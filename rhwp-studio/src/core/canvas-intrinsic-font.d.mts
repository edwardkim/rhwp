export function intrinsicCanvasFontAlias(family: string): string;
export function intrinsicCanvasSourceFamily(family: string): string | null;
export function fontProgramOwnsFamily(bytes: ArrayBuffer | ArrayBufferView, family: string): boolean;
export function installIntrinsicCanvasFont(family: string, bytes: ArrayBuffer, descriptors: FontFaceDescriptors): Promise<FontFace | null>;
