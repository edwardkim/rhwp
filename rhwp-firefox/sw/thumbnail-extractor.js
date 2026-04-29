// HWP 파일에서 PrvImage 썸네일을 경량 추출 (WASM 불필요)
//
// CFB(OLE2 Compound File) 컨테이너에서 /PrvImage 스트림만 추출한다.
// 전체 HWP 파싱 없이 썸네일만 빠르게 얻을 수 있다.

import { resolveDocumentUrl } from './document-url-resolver.js';

const THUMBNAIL_CACHE = new Map();
const CACHE_MAX_SIZE = 100;

/**
 * URL에서 HWP 파일을 fetch하여 PrvImage 썸네일을 추출한다.
 * @param {string} url - HWP 파일 URL
 * @returns {Promise<{dataUri: string, width: number, height: number} | null>}
 */
export async function extractThumbnailFromUrl(url) {
  // 캐시 확인
  if (THUMBNAIL_CACHE.has(url)) {
    return THUMBNAIL_CACHE.get(url);
  }

  try {
    const response = await fetch(resolveDocumentUrl(url));
    if (!response.ok) return null;
    const buffer = await response.arrayBuffer();
    const data = new Uint8Array(buffer);

    // HWP(CFB) 또는 HWPX(ZIP) 감지
    const isZip = data.length >= 4 && data[0] === 0x50 && data[1] === 0x4B;
    const result = isZip
      ? await extractPrvImageFromZipAsync(data)
      : extractPrvImage(data);
    if (result) {
      // 캐시 저장 (LRU)
      if (THUMBNAIL_CACHE.size >= CACHE_MAX_SIZE) {
        const firstKey = THUMBNAIL_CACHE.keys().next().value;
        THUMBNAIL_CACHE.delete(firstKey);
      }
      THUMBNAIL_CACHE.set(url, result);
    }
    return result;
  } catch {
    return null;
  }
}

/**
 * CFB 바이너리에서 /PrvImage 스트림을 추출한다.
 *
 * CFB 구조:
 * - 헤더 512바이트 (매직: D0 CF 11 E0 A1 B1 1A E1)
 * - 디렉토리 엔트리에서 "PrvImage" 이름을 찾아 스트림 위치/크기 파악
 * - 해당 섹터 체인을 따라 데이터 읽기
 *
 * 디렉토리 섹터도 FAT 체인으로 연결될 수 있으므로 체인 전체를 순회한다.
 */
function extractPrvImage(data) {
  // CFB 매직 넘버 확인
  if (data.length < 512) return null;
  if (data[0] !== 0xD0 || data[1] !== 0xCF || data[2] !== 0x11 || data[3] !== 0xE0) return null;

  // CFB 헤더에서 섹터 크기 읽기
  const sectorSizePow = data[30] | (data[31] << 8);
  const sectorSize = 1 << sectorSizePow; // 보통 512

  const miniSectorSizePow = data[32] | (data[33] << 8);
  const miniSectorSize = 1 << miniSectorSizePow; // 보통 64

  const miniStreamCutoff = readU32LE(data, 56); // 보통 4096
  const miniFatStart     = readU32LE(data, 60);

  // FAT 테이블 구성
  const fatEntries = buildFatTable(data, sectorSize);

  // Root Entry에서 Mini Stream 위치/크기 읽기 (offset 48: dirStartSector)
  const dirStartSector = readU32LE(data, 48);
  const rootOffset = (dirStartSector + 1) * sectorSize;
  const miniStreamStart = readU32LE(data, rootOffset + 116);
  const miniStreamSize  = readU32LE(data, rootOffset + 120);

  // Mini FAT 구성
  const miniFatEntries = buildMiniFatTable(data, sectorSize, miniFatStart, fatEntries);

  // Mini Stream 데이터 (Root Entry FAT 체인)
  const miniStreamData = readStreamFromFAT(data, miniStreamStart, miniStreamSize, sectorSize, fatEntries);

  // 디렉토리 섹터 FAT 체인을 따라 전체 순회
  const entriesPerSector = sectorSize / 128;
  let dirSector = dirStartSector;

  while (dirSector < 0xFFFFFFFE) {
    const dirOffset = (dirSector + 1) * sectorSize;

    for (let i = 0; i < entriesPerSector; i++) {
      const entryOffset = dirOffset + i * 128;
      if (entryOffset + 128 > data.length) break;

      // 엔트리 이름 읽기 (UTF-16LE)
      const nameLen = readU16LE(data, entryOffset + 64);
      if (nameLen === 0 || nameLen > 64) continue;

      const name = readUTF16LE(data, entryOffset, nameLen);
      if (name !== 'PrvImage') continue;

      const startSector = readU32LE(data, entryOffset + 116);
      const streamSize  = readU32LE(data, entryOffset + 120);

      if (streamSize === 0 || streamSize > 10 * 1024 * 1024) continue;

      let streamData;
      if (streamSize < miniStreamCutoff && miniStreamData) {
        // Mini Stream에서 읽기
        streamData = readStreamFromMini(miniStreamData, startSector, streamSize, miniSectorSize, miniFatEntries);
      } else {
        // 일반 FAT 체인에서 읽기
        streamData = readStreamFromFAT(data, startSector, streamSize, sectorSize, fatEntries);
      }
      if (!streamData) continue;

      return parseImageData(streamData);
    }

    // 다음 디렉토리 섹터로 이동
    dirSector = dirSector < fatEntries.length ? fatEntries[dirSector] : 0xFFFFFFFE;
  }

  return null;
}

/**
 * CFB Mini FAT 테이블을 구성하여 반환한다.
 *
 * Mini FAT 섹터들은 일반 FAT 체인으로 연결된다.
 */
function buildMiniFatTable(data, sectorSize, miniFatStart, fatEntries) {
  const miniFatEntries = [];
  let sector = miniFatStart;
  for (let safety = 0; safety < 10000; safety++) {
    if (sector >= 0xFFFFFFFE) break;
    const offset = (sector + 1) * sectorSize;
    const entriesPerSector = sectorSize / 4;
    for (let j = 0; j < entriesPerSector; j++) {
      const off = offset + j * 4;
      if (off + 4 > data.length) break;
      miniFatEntries.push(readU32LE(data, off));
    }
    sector = sector < fatEntries.length ? fatEntries[sector] : 0xFFFFFFFE;
  }
  return miniFatEntries;
}

/**
 * Mini Stream에서 Mini FAT 체인을 따라 데이터를 읽는다.
 *
 * @param {Uint8Array} miniStream - Root Entry의 Mini Stream 전체 데이터
 * @param {number} startSector - Mini Stream 내 시작 미니 섹터 번호
 * @param {number} streamSize - 읽을 바이트 수
 * @param {number} miniSectorSize - 미니 섹터 크기 (보통 64)
 * @param {number[]} miniFatEntries - Mini FAT 테이블
 */
function readStreamFromMini(miniStream, startSector, streamSize, miniSectorSize, miniFatEntries) {
  const result = new Uint8Array(streamSize);
  let sector = startSector;
  let bytesRead = 0;

  for (let safety = 0; safety < 10000 && bytesRead < streamSize; safety++) {
    if (sector >= 0xFFFFFFFE) break;
    const offset = sector * miniSectorSize;
    const copyLen = Math.min(miniSectorSize, streamSize - bytesRead);
    if (offset + copyLen > miniStream.length) break;
    result.set(miniStream.subarray(offset, offset + copyLen), bytesRead);
    bytesRead += copyLen;
    sector = sector < miniFatEntries.length ? miniFatEntries[sector] : 0xFFFFFFFE;
  }

  return bytesRead >= streamSize ? result : null;
}

/**
 * CFB FAT 테이블을 구성하여 반환한다.
 *
 * 헤더 offset 76~507의 DIFAT 엔트리(최대 109개)에서 FAT 섹터 목록을 읽고,
 * 각 FAT 섹터를 순회하여 sector → nextSector 매핑 배열을 반환한다.
 */
function buildFatTable(data, sectorSize) {
  const fatEntries = [];
  for (let i = 0; i < 109; i++) {
    const fatSect = readU32LE(data, 76 + i * 4);
    if (fatSect === 0xFFFFFFFE || fatSect === 0xFFFFFFFF) break;
    const fatOffset = (fatSect + 1) * sectorSize;
    const entriesPerSector = sectorSize / 4;
    for (let j = 0; j < entriesPerSector; j++) {
      const off = fatOffset + j * 4;
      if (off + 4 > data.length) break;
      fatEntries.push(readU32LE(data, off));
    }
  }
  return fatEntries;
}

/**
 * FAT 체인을 따라 스트림 데이터를 읽는다.
 *
 * @param {Uint8Array} data - CFB 전체 바이너리
 * @param {number} startSector - 스트림 시작 섹터 번호
 * @param {number} streamSize - 읽을 바이트 수
 * @param {number} sectorSize - 섹터 크기 (바이트)
 * @param {number[]} [fatEntries] - 미리 구성된 FAT 테이블 (없으면 내부 구성)
 */
function readStreamFromFAT(data, startSector, streamSize, sectorSize, fatEntries) {
  if (!fatEntries) {
    fatEntries = buildFatTable(data, sectorSize);
  }

  // 섹터 체인을 따라 데이터 수집
  const result = new Uint8Array(streamSize);
  let sector = startSector;
  let bytesRead = 0;

  for (let safety = 0; safety < 10000 && bytesRead < streamSize; safety++) {
    if (sector >= 0xFFFFFFFE) break;
    const offset = (sector + 1) * sectorSize;
    const copyLen = Math.min(sectorSize, streamSize - bytesRead);
    if (offset + copyLen > data.length) break;
    result.set(data.subarray(offset, offset + copyLen), bytesRead);
    bytesRead += copyLen;

    // 다음 섹터
    if (sector < fatEntries.length) {
      sector = fatEntries[sector];
    } else {
      break;
    }
  }

  return bytesRead >= streamSize ? result : null;
}

/**
 * 이미지 데이터에서 포맷 감지 + dataUri 생성
 */
function parseImageData(data) {
  let mime, width = 0, height = 0;

  if (data.length >= 8 && data[0] === 0x89 && data[1] === 0x50 && data[2] === 0x4E && data[3] === 0x47) {
    // PNG
    mime = 'image/png';
    if (data.length >= 24) {
      width = (data[16] << 24) | (data[17] << 16) | (data[18] << 8) | data[19];
      height = (data[20] << 24) | (data[21] << 16) | (data[22] << 8) | data[23];
    }
  } else if (data.length >= 2 && data[0] === 0x42 && data[1] === 0x4D) {
    // BMP
    mime = 'image/bmp';
    if (data.length >= 26) {
      width = readU32LE(data, 18);
      height = Math.abs(readI32LE(data, 22));
    }
  } else if (data.length >= 3 && data[0] === 0x47 && data[1] === 0x49 && data[2] === 0x46) {
    // GIF
    mime = 'image/gif';
    if (data.length >= 10) {
      width = readU16LE(data, 6);
      height = readU16LE(data, 8);
    }
  } else {
    return null;
  }

  // Base64 인코딩
  let binary = '';
  for (let i = 0; i < data.length; i++) {
    binary += String.fromCharCode(data[i]);
  }
  const base64 = btoa(binary);
  const dataUri = `data:${mime};base64,${base64}`;

  return { dataUri, width, height, mime };
}

/**
 * HWPX(ZIP) 컨테이너에서 Preview/PrvImage.* 추출
 *
 * ZIP End of Central Directory → Central Directory → 로컬 파일 헤더 → 데이터
 */
async function extractPrvImageFromZipAsync(data) {
  // End of Central Directory 찾기 (ZIP 파일 끝에서 역방향 탐색)
  let eocdOffset = -1;
  for (let i = data.length - 22; i >= 0 && i >= data.length - 65558; i--) {
    if (data[i] === 0x50 && data[i+1] === 0x4B && data[i+2] === 0x05 && data[i+3] === 0x06) {
      eocdOffset = i;
      break;
    }
  }
  if (eocdOffset < 0) return null;

  const cdOffset = readU32LE(data, eocdOffset + 16);
  const cdEntries = readU16LE(data, eocdOffset + 10);

  // Central Directory 순회
  let offset = cdOffset;
  for (let i = 0; i < cdEntries && offset + 46 < data.length; i++) {
    // Central Directory 시그니처 확인
    if (data[offset] !== 0x50 || data[offset+1] !== 0x4B || data[offset+2] !== 0x01 || data[offset+3] !== 0x02) break;

    const compMethod = readU16LE(data, offset + 10);
    const compSize = readU32LE(data, offset + 20);
    const uncompSize = readU32LE(data, offset + 24);
    const nameLen = readU16LE(data, offset + 28);
    const extraLen = readU16LE(data, offset + 30);
    const commentLen = readU16LE(data, offset + 32);
    const localHeaderOffset = readU32LE(data, offset + 42);

    // 파일 이름 읽기
    const nameBytes = data.subarray(offset + 46, offset + 46 + nameLen);
    const name = new TextDecoder().decode(nameBytes);

    // Preview/PrvImage 확인
    if (name.startsWith('Preview/PrvImage')) {
      // 로컬 파일 헤더에서 실제 데이터 위치 계산
      if (localHeaderOffset + 30 >= data.length) break;
      const localNameLen = readU16LE(data, localHeaderOffset + 26);
      const localExtraLen = readU16LE(data, localHeaderOffset + 28);
      const dataStart = localHeaderOffset + 30 + localNameLen + localExtraLen;

      if (compMethod === 0) {
        // 비압축 (stored)
        const imageData = data.subarray(dataStart, dataStart + uncompSize);
        return parseImageData(imageData);
      } else if (compMethod === 8) {
        // deflate 압축 — DecompressionStream (비동기)
        try {
          const compressed = data.slice(dataStart, dataStart + compSize);
          const ds = new DecompressionStream('raw');
          const writer = ds.writable.getWriter();
          writer.write(compressed);
          writer.close();
          const reader = ds.readable.getReader();
          const chunks = [];
          while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            chunks.push(value);
          }
          const totalLen = chunks.reduce((s, c) => s + c.length, 0);
          const decompressed = new Uint8Array(totalLen);
          let offset = 0;
          for (const chunk of chunks) {
            decompressed.set(chunk, offset);
            offset += chunk.length;
          }
          return parseImageData(decompressed);
        } catch {
          return null;
        }
      }
    }

    offset += 46 + nameLen + extraLen + commentLen;
  }

  return null;
}

// ─── 바이너리 헬퍼 ───

function readU16LE(data, offset) {
  return data[offset] | (data[offset + 1] << 8);
}

function readU32LE(data, offset) {
  return (data[offset] | (data[offset + 1] << 8) | (data[offset + 2] << 16) | (data[offset + 3] << 24)) >>> 0;
}

function readI32LE(data, offset) {
  return data[offset] | (data[offset + 1] << 8) | (data[offset + 2] << 16) | (data[offset + 3] << 24);
}

function readUTF16LE(data, offset, byteLen) {
  let str = '';
  for (let i = 0; i < byteLen - 2; i += 2) {
    const code = data[offset + i] | (data[offset + i + 1] << 8);
    if (code === 0) break;
    str += String.fromCharCode(code);
  }
  return str;
}
