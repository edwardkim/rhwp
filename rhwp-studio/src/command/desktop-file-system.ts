/**
 * 데스크톱(Tauri/WebView2) 파일 시스템 어댑터.
 *
 * WebView2는 File System Access API의 picker를 구현하지 않는다.
 * `window.showOpenFilePicker`/`showSaveFilePicker`는 함수로 **존재하지만** 호출해도
 * 다이얼로그가 뜨지 않고 Promise가 영원히 pending으로 남는다. Blink는 첫 호출 시
 * 창에 "picker active" 플래그를 세우고 브라우저 프로세스 응답에서 해제하는데,
 * 그 응답이 오지 않으므로 플래그가 영구히 남아 두 번째 호출부터
 * `NotAllowedError: File picker already active.`로 거부된다.
 *
 * 그래서 데스크톱에서는 picker 존재 여부로 능력을 판정할 수 없다. 이 모듈은
 * Tauri dialog 플러그인의 네이티브 다이얼로그와 fs 플러그인의 경로 기반 읽기/쓰기로
 * {@link FileSystemWindowLike}/{@link FileSystemFileHandleLike} 계약을 그대로 구현해,
 * 상위 저장/열기 로직(`file-system-access.ts`, `commands/file.ts`)이 웹과 동일한
 * 코드 경로를 쓰도록 한다.
 *
 * 경로 접근 권한: dialog 플러그인이 사용자가 고른 경로를 fs 런타임 스코프에 등록한다
 * (`tauri-plugin-dialog`의 `allow_file`). 따라서 정적 fs 스코프를 넓히지 않으며,
 * 이 세션에서 사용자가 직접 고른 파일만 읽고 쓸 수 있다. 이 스코프는 세션 한정이라
 * 이전 실행에서 저장된 경로는 되살릴 수 없다 — 최근 문서는 데스크톱에서 메타-only로
 * 남고(`recent-store.ts`) 재선택을 요구한다.
 */

import type { FilePickerType } from './save-format.ts';
import type {
  FileSystemFileHandleLike,
  FileSystemPermissionState,
  FileSystemWindowLike,
  FileSystemWritableFileStreamLike,
} from './file-system-access.ts';

/** 어댑터가 의존하는 Tauri 플러그인 표면 — 테스트에서 대체 가능하게 좁혀 둔다. */
export interface TauriFileSystemBridge {
  /** dialog 플러그인 `open` — 취소 시 null. */
  openDialog(options: {
    multiple: false;
    directory: false;
    filters?: { name: string; extensions: string[] }[];
  }): Promise<string | null>;
  /** dialog 플러그인 `save` — 취소 시 null. */
  saveDialog(options: {
    defaultPath?: string;
    filters?: { name: string; extensions: string[] }[];
  }): Promise<string | null>;
  readFile(path: string): Promise<Uint8Array>;
  writeFile(path: string, data: Uint8Array): Promise<void>;
}

/** Tauri 런타임(데스크톱 셸) 안에서 실행 중인지 여부. */
export function isDesktopRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * 사용자 취소를 웹 picker와 동일한 형태로 전달한다.
 * 상위 로직은 AbortError를 "사용자 취소"로 해석해 폴백 다운로드를 건너뛴다
 * (`commands/file.ts`의 `isUserCancelError`).
 */
function abortError(): DOMException {
  return new DOMException('파일 선택이 취소되었습니다.', 'AbortError');
}

function baseName(path: string): string {
  const match = /[^\\/]+$/.exec(path);
  return match ? match[0] : path;
}

/** picker `types`(MIME→확장자 맵)를 dialog 플러그인 filter로 옮긴다. */
export function dialogFiltersFromPickerTypes(
  types: FilePickerType[] | undefined,
): { name: string; extensions: string[] }[] | undefined {
  if (!types?.length) return undefined;
  return types.map((type) => {
    const extensions = [...new Set(
      Object.values(type.accept)
        .flat()
        .map((ext) => ext.replace(/^\./, '').toLowerCase()),
    )];
    return { name: type.description, extensions };
  }).filter((filter) => filter.extensions.length > 0);
}

/** 경로 기반 파일 핸들 — 웹 `FileSystemFileHandle`과 같은 계약을 만족한다. */
export class DesktopFileHandle implements FileSystemFileHandleLike {
  readonly kind = 'file' as const;
  readonly path: string;
  readonly name: string;
  // 파라미터 프로퍼티는 node --test 의 type-strip 실행에서 지원되지 않아 명시 필드로 둔다.
  private readonly bridge: TauriFileSystemBridge;

  constructor(path: string, bridge: TauriFileSystemBridge) {
    this.path = path;
    this.name = baseName(path);
    this.bridge = bridge;
  }

  async getFile(): Promise<File> {
    const bytes = await this.bridge.readFile(this.path);
    return new File([bytes as BlobPart], this.name);
  }

  /**
   * 웹 writable stream과 달리 부분 쓰기를 파일에 바로 흘리지 않는다.
   * close() 시점에 한 번에 쓴다 — 쓰기 도중 실패해도 원본이 잘린 채 남지 않는다.
   */
  async createWritable(): Promise<FileSystemWritableFileStreamLike> {
    const chunks: Blob[] = [];
    const { bridge, path } = this;
    return {
      async write(data: Blob): Promise<void> {
        chunks.push(data);
      },
      async close(): Promise<void> {
        const merged = new Blob(chunks);
        await bridge.writeFile(path, new Uint8Array(await merged.arrayBuffer()));
      },
    };
  }

  async isSameEntry(other: FileSystemFileHandleLike): Promise<boolean> {
    return other instanceof DesktopFileHandle && other.path === this.path;
  }

  /** 경로는 사용자가 다이얼로그로 직접 고른 것이므로 별도 권한 프롬프트가 없다. */
  async queryPermission(): Promise<FileSystemPermissionState> {
    return 'granted';
  }

  async requestPermission(): Promise<FileSystemPermissionState> {
    return 'granted';
  }
}

/** 실제 Tauri 플러그인을 지연 로드하는 기본 bridge — 웹 번들에는 로드되지 않는다. */
const tauriPluginBridge: TauriFileSystemBridge = {
  async openDialog(options) {
    const { open } = await import('@tauri-apps/plugin-dialog');
    return open(options);
  },
  async saveDialog(options) {
    const { save } = await import('@tauri-apps/plugin-dialog');
    return save(options);
  },
  async readFile(path) {
    const { readFile } = await import('@tauri-apps/plugin-fs');
    return readFile(path);
  },
  async writeFile(path, data) {
    const { writeFile } = await import('@tauri-apps/plugin-fs');
    await writeFile(path, data);
  },
};

/** 데스크톱용 `FileSystemWindowLike` 구현체를 만든다. */
export function createDesktopFileSystemWindow(
  bridge: TauriFileSystemBridge = tauriPluginBridge,
): FileSystemWindowLike {
  return {
    async showOpenFilePicker(options) {
      const path = await bridge.openDialog({
        multiple: false,
        directory: false,
        filters: dialogFiltersFromPickerTypes(options?.types),
      });
      if (path === null) throw abortError();
      return [new DesktopFileHandle(path, bridge)];
    },
    async showSaveFilePicker(options) {
      const path = await bridge.saveDialog({
        defaultPath: options?.suggestedName,
        filters: dialogFiltersFromPickerTypes(options?.types),
      });
      if (path === null) throw abortError();
      return new DesktopFileHandle(path, bridge);
    },
  };
}

let desktopWindow: FileSystemWindowLike | null = null;

/**
 * 파일 열기/저장이 사용할 picker 표면을 고른다.
 * 데스크톱(Tauri)에서는 네이티브 다이얼로그 어댑터, 그 외에는 브라우저 `window`.
 */
export function getFileSystemWindow(): FileSystemWindowLike {
  if (!isDesktopRuntime()) return window as FileSystemWindowLike;
  desktopWindow ??= createDesktopFileSystemWindow();
  return desktopWindow;
}
