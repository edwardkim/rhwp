/**
 * 한글·오피스 클립보드의 그림을 붙여넣기용 HTML 에 실어 준다.
 *
 * 한글(HWP)은 클립보드 HTML 에 그림을 `<img src="file:///C:\…\clip_image001.png">` 로만 적는다.
 * 브라우저는 그 로컬 경로를 열 수 없어 그림이 통째로 사라진다(실사용 신고 2026-09-03).
 * 같은 클립보드의 `text/rtf` 에는 같은 그림이 순서대로 들어 있으므로, 거기서 픽셀을 꺼내
 * `data:` URI 로 바꿔 HTML 의 `<img src>` 를 채운다.
 *
 * RTF 그림 형태
 *  - `\pngblip` / `\jpegblip` : 그대로 쓴다.
 *  - `\wmetafile8` / `\emfblip` : EMF 안의 DIB(EMR_STRETCHDIBITS·EMR_SETDIBITSTODEVICE)를
 *    BMP 로 재조립한다 — 한글이 넣는 EMF 는 원본 래스터를 그대로 품고 있어 화질 손실이 없다.
 *    벡터 도형만 있는 EMF 는 래스터가 없으므로 건너뛴다(그 그림만 빠지고 나머지는 붙는다).
 */

const EMR_SETDIBITSTODEVICE = 80;
const EMR_STRETCHDIBITS = 81;
const EMR_EOF = 14;

export interface RtfPicture {
  bytes: Uint8Array;
  /** 알아낸 MIME. BMP 는 브라우저 canvas 로 PNG 로 바꿔 쓴다. */
  mime: string;
}

function hexToBytes(hex: string): Uint8Array {
  const len = hex.length >> 1;
  const out = new Uint8Array(len);
  for (let i = 0; i < len; i++) out[i] = parseInt(hex.substr(i * 2, 2), 16);
  return out;
}

interface EmfDib {
  bmp: Uint8Array;
  x: number; y: number; w: number; h: number;
}

function dibToBmpFile(bmi: Uint8Array, bits: Uint8Array): Uint8Array {
  const header = new Uint8Array(14);
  const hv = new DataView(header.buffer);
  header[0] = 0x42; header[1] = 0x4d; // 'BM'
  hv.setUint32(2, 14 + bmi.byteLength + bits.byteLength, true);
  hv.setUint32(10, 14 + bmi.byteLength, true);
  const out = new Uint8Array(header.byteLength + bmi.byteLength + bits.byteLength);
  out.set(header, 0);
  out.set(bmi, header.byteLength);
  out.set(bits, header.byteLength + bmi.byteLength);
  return out;
}

/**
 * EMF 안의 비트맵 레코드를 **전부** 목적 사각형과 함께 뽑는다.
 * 한글의 묶음 그림(로고+글자 등)은 한 EMF 안에 조각 여러 개로 들어온다 —
 * 가장 큰 조각만 쓰면 그림이 '반쪽'이 된다. 조각을 모두 모아 좌표대로 합성한다.
 */
export function extractEmfDibs(emf: Uint8Array): { dibs: EmfDib[]; bounds: { x: number; y: number; w: number; h: number } | null } {
  const dibs: EmfDib[] = [];
  if (emf.byteLength < 44) return { dibs, bounds: null };
  const view = new DataView(emf.buffer, emf.byteOffset, emf.byteLength);
  // EMF 헤더의 rclBounds(device units)
  const bx = view.getInt32(8, true), by = view.getInt32(12, true);
  const bx2 = view.getInt32(16, true), by2 = view.getInt32(20, true);
  const bounds = bx2 >= bx && by2 >= by
    ? { x: bx, y: by, w: bx2 - bx + 1, h: by2 - by + 1 }
    : null;
  let offset = 0;
  while (offset + 8 <= emf.byteLength) {
    const type = view.getUint32(offset, true);
    const size = view.getUint32(offset + 4, true);
    if (size < 8 || offset + size > emf.byteLength) break;
    if (type === EMR_STRETCHDIBITS || type === EMR_SETDIBITSTODEVICE) {
      const xDest = view.getInt32(offset + 24, true);
      const yDest = view.getInt32(offset + 28, true);
      const offBmi = view.getUint32(offset + 48, true);
      const cbBmi = view.getUint32(offset + 52, true);
      const offBits = view.getUint32(offset + 56, true);
      const cbBits = view.getUint32(offset + 60, true);
      // STRETCHDIBITS 만 cxDest/cyDest 를 갖는다. SETDIBITSTODEVICE 는 원본 크기 그대로다.
      const cxDest = type === EMR_STRETCHDIBITS && offset + 80 <= emf.byteLength
        ? view.getInt32(offset + 72, true) : 0;
      const cyDest = type === EMR_STRETCHDIBITS && offset + 80 <= emf.byteLength
        ? view.getInt32(offset + 76, true) : 0;
      if (cbBmi >= 12 && cbBits > 0
        && offset + offBmi + cbBmi <= emf.byteLength
        && offset + offBits + cbBits <= emf.byteLength) {
        const bmi = emf.subarray(offset + offBmi, offset + offBmi + cbBmi);
        const bits = emf.subarray(offset + offBits, offset + offBits + cbBits);
        const bmiView = new DataView(bmi.buffer, bmi.byteOffset, bmi.byteLength);
        const srcW = Math.abs(bmiView.getInt32(4, true));
        const srcH = Math.abs(bmiView.getInt32(8, true));
        if (srcW > 0 && srcH > 0) {
          dibs.push({
            bmp: dibToBmpFile(bmi, bits),
            x: xDest, y: yDest,
            w: cxDest > 0 ? cxDest : srcW,
            h: cyDest > 0 ? cyDest : srcH,
          });
        }
      }
    }
    offset += size;
    if (type === EMR_EOF) break;
  }
  return { dibs, bounds };
}

/** EMF 의 비트맵 조각을 좌표대로 한 장의 PNG Blob 으로 합성한다. 비트맵이 없으면 null. */
async function emfToPngBlob(emf: Uint8Array): Promise<Blob | null> {
  const { dibs, bounds } = extractEmfDibs(emf);
  if (dibs.length === 0) return null;
  const bitmaps: { bmp: ImageBitmap; d: EmfDib }[] = [];
  try {
    for (const d of dibs) {
      const copy = new Uint8Array(d.bmp.byteLength);
      copy.set(d.bmp);
      try {
        bitmaps.push({ bmp: await createImageBitmap(new Blob([copy.buffer as ArrayBuffer], { type: 'image/bmp' })), d });
      } catch (error) {
        console.warn('[paste] EMF 조각 디코드 실패(건너뜀):', error);
      }
    }
    if (bitmaps.length === 0) return null;
    // 조각이 하나뿐이면 원본 픽셀 그대로 쓴다(재표본 없음).
    if (bitmaps.length === 1 && dibs.length === 1) {
      const only = bitmaps[0];
      const canvas = document.createElement('canvas');
      canvas.width = only.bmp.width; canvas.height = only.bmp.height;
      canvas.getContext('2d')?.drawImage(only.bmp, 0, 0);
      return await new Promise<Blob | null>(r => canvas.toBlob(b => r(b), 'image/png'));
    }
    // 여러 조각: 목적 사각형 합집합(헤더 bounds 가 있으면 그것)을 캔버스로 삼는다.
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const { d } of bitmaps) {
      minX = Math.min(minX, d.x); minY = Math.min(minY, d.y);
      maxX = Math.max(maxX, d.x + d.w); maxY = Math.max(maxY, d.y + d.h);
    }
    // 헤더 rclBounds 는 그림이 없는 여백까지 포함해 붙여넣은 그림이 헐렁해진다 — 조각 합집합만 쓴다.
    void bounds;
    const spanW = Math.max(1, Math.round(maxX - minX));
    const spanH = Math.max(1, Math.round(maxY - minY));
    // 조각의 원본 해상도를 잃지 않도록 배율을 잡는다(최대 4배, 4000px 상한).
    const natural = bitmaps.reduce((acc, { bmp, d }) =>
      Math.max(acc, d.w > 0 ? bmp.width / d.w : 1), 1);
    const scale = Math.min(4, Math.max(1, natural), 4000 / Math.max(spanW, spanH));
    const canvas = document.createElement('canvas');
    canvas.width = Math.max(1, Math.round(spanW * scale));
    canvas.height = Math.max(1, Math.round(spanH * scale));
    const ctx = canvas.getContext('2d');
    if (!ctx) return null;
    for (const { bmp, d } of bitmaps) {
      ctx.drawImage(bmp, (d.x - minX) * scale, (d.y - minY) * scale, d.w * scale, d.h * scale);
    }
    console.log(`[paste] EMF 조각 ${bitmaps.length}개 합성 → ${canvas.width}×${canvas.height}`);
    return await new Promise<Blob | null>(r => canvas.toBlob(b => r(b), 'image/png'));
  } finally {
    for (const { bmp } of bitmaps) bmp.close?.();
  }
}

/** 이미지 바이트에서 픽셀 크기를 읽는다(PNG IHDR·JPEG SOF·BMP 헤더). 모르면 null. */
export function imagePixelSize(bytes: Uint8Array): { w: number; h: number } | null {
  const v = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  if (bytes.byteLength > 24 && bytes[0] === 0x89 && bytes[1] === 0x50) {
    return { w: v.getUint32(16), h: v.getUint32(20) };            // PNG IHDR
  }
  if (bytes.byteLength > 4 && bytes[0] === 0x42 && bytes[1] === 0x4d) {
    return { w: Math.abs(v.getInt32(18, true)), h: Math.abs(v.getInt32(22, true)) }; // BMP
  }
  if (bytes.byteLength > 4 && bytes[0] === 0xff && bytes[1] === 0xd8) {  // JPEG
    let i = 2;
    while (i + 9 < bytes.byteLength) {
      if (bytes[i] !== 0xff) { i++; continue; }
      const marker = bytes[i + 1];
      const len = v.getUint16(i + 2);
      if (marker >= 0xc0 && marker <= 0xcf && marker !== 0xc4 && marker !== 0xc8 && marker !== 0xcc) {
        return { h: v.getUint16(i + 5), w: v.getUint16(i + 7) };
      }
      i += 2 + len;
    }
  }
  return null;
}

/** RTF 에서 그림을 문서 순서대로 뽑는다. */
export function extractRtfPictures(rtf: string): RtfPicture[] {
  const pictures: RtfPicture[] = [];
  const re = /\\pict([\s\S]*?)\}/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(rtf)) !== null) {
    const body = m[1];
    const isPng = /\\pngblip\b/.test(body);
    const isJpeg = /\\jpegblip\b/.test(body);
    const isMeta = /\\(?:wmetafile\d*|emfblip)\b/.test(body);
    // 제어 단어(\picw3439 등)를 지우고 남는 16진수만 payload 다.
    const hex = body.replace(/\\[a-zA-Z]+-?\d*\s?/g, '').replace(/[^0-9a-fA-F]/g, '');
    if (hex.length < 32) continue;
    const bytes = hexToBytes(hex.slice(0, hex.length - (hex.length % 2)));
    if (isPng) pictures.push({ bytes, mime: 'image/png' });
    else if (isJpeg) pictures.push({ bytes, mime: 'image/jpeg' });
    else if (isMeta) pictures.push({ bytes, mime: 'image/emf' });
    else pictures.push({ bytes: new Uint8Array(0), mime: '' });
  }
  return pictures;
}

async function bytesToDataUrl(bytes: Uint8Array, mime: string): Promise<string | null> {
  if (!bytes.byteLength || !mime) return null;
  // TS lib.dom 의 BlobPart 는 ArrayBuffer 뒷받침만 받는다(SharedArrayBuffer 배제) — 사본으로 맞춘다.
  const copy = new Uint8Array(bytes.byteLength);
  copy.set(bytes);
  const blob = new Blob([copy.buffer as ArrayBuffer], { type: mime });
  // 코어는 png/jpeg 를 그대로 받는다. BMP 는 canvas 로 PNG 로 바꿔 크기를 줄인다.
  const source = mime === 'image/emf'
    ? await emfToPngBlob(bytes)
    : mime === 'image/bmp' ? await bmpBlobToPng(blob) : blob;
  if (!source) return null;
  return await new Promise<string | null>((resolve) => {
    const reader = new FileReader();
    reader.onload = () => resolve(typeof reader.result === 'string' ? reader.result : null);
    reader.onerror = () => resolve(null);
    reader.readAsDataURL(source);
  });
}

async function bmpBlobToPng(blob: Blob): Promise<Blob | null> {
  try {
    const bitmap = await createImageBitmap(blob);
    const canvas = document.createElement('canvas');
    canvas.width = bitmap.width;
    canvas.height = bitmap.height;
    const ctx = canvas.getContext('2d');
    if (!ctx) return null;
    ctx.drawImage(bitmap, 0, 0);
    bitmap.close?.();
    return await new Promise<Blob | null>((resolve) => canvas.toBlob(b => resolve(b), 'image/png'));
  } catch (error) {
    console.warn('[paste] BMP → PNG 변환 실패:', error);
    return null;
  }
}

/**
 * HTML 의 `<img src>` 중 브라우저가 열 수 없는 것(file:// 등)을 RTF 그림의 data URI 로 바꾼다.
 * 이미 `data:` 인 것은 그대로 두고, 대응하는 그림이 없으면 그 `<img>` 만 남겨 둔다.
 */
export async function inlineOfficeClipboardImages(html: string, rtf: string): Promise<string> {
  if (!rtf || !/\\pict/.test(rtf)) return html;
  const pictures = extractRtfPictures(rtf);
  if (pictures.length === 0) return html;

  const imgTags = html.match(/<img\b[^>]*>/gi) ?? [];
  if (imgTags.length === 0) return html;

  // 한글은 그림을 pict 순서대로, HTML `<img>` 도 같은 순서로 낸다 —
  // **순서 매칭**이 정답이다(크기·비로 짝지으면 같은 비의 작은 로고가 큰 도식 자리를 뺏는다, 실측).
  // 묶음 개체는 한 pict 안에 조각 여러 개로 오므로 좌표대로 한 장으로 합성한다.
  const urls: (string | null)[] = [];
  let pictIndex = 0;
  let replaced = 0;
  for (const tag of imgTags) {
    if (/src\s*=\s*["']data:/i.test(tag)) { urls.push(null); continue; }
    while (pictIndex < pictures.length && !pictures[pictIndex].bytes.byteLength) pictIndex++;
    const pic = pictures[pictIndex++];
    if (!pic) { urls.push(null); continue; }
    const url = await bytesToDataUrl(pic.bytes, pic.mime);
    urls.push(url);
    if (url) replaced++;
  }
  let index = 0;

  const out = html.replace(/<img\b[^>]*>/gi, (tag) => {
    const url = urls[index++];
    if (!url) return tag;
    replaced++;
    return /src\s*=\s*["'][^"']*["']/i.test(tag)
      ? tag.replace(/src\s*=\s*["'][^"']*["']/i, `src="${url}"`)
      : tag.replace(/<img\b/i, `<img src="${url}"`);
  });
  console.log(`[paste] 클립보드 그림 ${replaced}/${imgTags.length}개 삽입(pict ${pictures.length})`);
  return liftImagesToBlockLevel(out);
}

/**
 * 코어 HTML 가져오기는 최상위 `<img>` 만 그림으로 만든다 — 한글·워드는 `<p><span><img></span></p>`
 * 처럼 문단 안에 넣으므로 그대로 두면 그림이 통째로 버려진다. 표 셀(`<td>`) 안은 그대로 두고,
 * 문단 안 그림만 그 문단 뒤 최상위로 끌어올린다(문서 순서 유지).
 */
export function liftImagesToBlockLevel(html: string): string {
  if (!/<img\b/i.test(html) || typeof DOMParser === 'undefined') return html;
  try {
    const doc = new DOMParser().parseFromString(html, 'text/html');
    const body = doc.body;
    if (!body) return html;
    let moved = 0;
    for (const img of Array.from(body.querySelectorAll('img'))) {
      // 셀 안 그림도 같은 규칙으로 올린다 — 셀 내용도 같은 파서(parse_html_to_paragraphs)를 타므로
      // `<td>` 직속으로 올려 두면 셀 안 그림으로 들어간다(실측 2026-09-03).
      const block = img.closest('p, div');
      if (!block || block === body || block.parentElement === null) continue;
      if (block.parentElement.tagName === 'TR' || block.parentElement.tagName === 'TABLE') continue;
      // 문단의 정렬·여백을 그림에 실어 보낸다 — 코어는 <img> 의 style 로 문단서식을 만든다.
      const blockStyle = block.getAttribute('style') ?? '';
      const keep = ['text-align', 'margin-left', 'margin-right', 'text-indent', 'line-height']
        .map(prop => {
          const m = new RegExp(`(?:^|;)\\s*${prop}\\s*:\\s*([^;]+)`, 'i').exec(blockStyle);
          return m ? `${prop}:${m[1].trim()}` : null;
        })
        .filter(Boolean)
        .join(';');
      if (keep) img.setAttribute('style', `${img.getAttribute('style') ?? ''};${keep}`.replace(/^;/, ''));
      block.parentElement.insertBefore(img, block.nextSibling);
      moved++;
    }
    if (moved === 0) return html;
    console.log(`[paste] 문단 안 그림 ${moved}개를 문단 밖으로 끌어올림`);
    return body.innerHTML;
  } catch (error) {
    console.warn('[paste] 그림 위치 정리 실패 — 원본 HTML 사용:', error);
    return html;
  }
}

/** RTF 로 채워야 할 그림이 있는 붙여넣기인지. */
export function needsRtfImageInlining(html: string, rtf: string): boolean {
  if (!html || !rtf || !/\\pict/.test(rtf)) return false;
  return /<img\b[^>]*src\s*=\s*["'](?!data:)/i.test(html);
}
