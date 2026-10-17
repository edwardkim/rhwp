import type { CanvasKit, Image } from 'canvaskit-wasm';
import { decodedImageMatchesEncodedHeader, type EncodedImageHeader } from './image-header.ts';

interface BrowserImageEnvironment {
  createImage(): HTMLImageElement;
  createObjectURL(blob: Blob): string;
  revokeObjectURL(url: string): void;
}

export function decodeBrowserImage(
  kit: CanvasKit,
  bytes: Uint8Array,
  header: EncodedImageHeader,
  environment: BrowserImageEnvironment = {
    createImage: () => new globalThis.Image(),
    createObjectURL: blob => URL.createObjectURL(blob),
    revokeObjectURL: url => URL.revokeObjectURL(url),
  },
): { promise: Promise<Image | null>; cancel(): void } {
  let cancel = () => {};
  const promise = new Promise<Image | null>(resolve => {
    let image: HTMLImageElement | undefined;
    let url: string | undefined;
    let settled = false;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const finish = (result: Image | null) => {
      if (settled) { result?.delete(); return; }
      settled = true;
      clearTimeout(timer);
      if (image) {
        image.onload = null;
        image.onerror = null;
        try { image.src = ''; } catch { /* 이미 분리한 load를 되살리지 않는다. */ }
      }
      if (url) {
        try { environment.revokeObjectURL(url); } catch { /* 정리는 best effort다. */ }
      }
      resolve(result);
    };
    cancel = () => finish(null);
    try {
      const mime = header.format === 'svg' ? 'image/svg+xml' : `image/${header.format}`;
      url = environment.createObjectURL(new Blob([bytes.slice().buffer], { type: mime }));
      image = environment.createImage();
      image.onload = () => {
        let decoded: Image | null = null;
        try {
          decoded = kit.MakeImageFromCanvasImageSource(image!);
          if (decoded && !decodedImageMatchesEncodedHeader(header, decoded.width(), decoded.height())) {
            decoded.delete();
            decoded = null;
          }
        } catch { decoded?.delete(); decoded = null; }
        finish(decoded);
      };
      image.onerror = () => finish(null);
      timer = setTimeout(() => finish(null), 10_000);
      image.src = url;
    } catch { finish(null); }
  });
  return { promise, cancel: () => cancel() };
}
