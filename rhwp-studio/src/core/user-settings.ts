/**
 * 사용자 환경설정 저장/로드 서비스
 *
 * localStorage 기반, 단일 키(rhwp-settings)에 JSON으로 저장.
 * 섹션별 확장 가능한 구조.
 */

import type { PageArrangement } from '../view/page-arrangement.ts';
import {
  DEFAULT_PAGE_MOVEMENT,
  resolvePageViewSettings,
  type PageMovementSettings,
} from '../view/page-movement.ts';

/** 대표 글꼴 세트 (7개 언어별 글꼴) */
export interface FontSet {
  name: string;
  korean: string;
  english: string;
  chinese: string;
  japanese: string;
  other: string;
  symbol: string;
  user: string;
}

/** 글꼴 환경 설정 */
export interface FontSettings {
  /** 사용자 정의 대표 글꼴 세트 */
  fontSets: FontSet[];
  /** 최근 사용 글꼴 표시 여부 */
  showRecentFonts: boolean;
  /** 최근 사용 글꼴 표시 개수 (1~5) */
  recentFontCount: number;
}

/** 앱 UI 테마 설정값 */
export type ThemeMode = 'system' | 'light' | 'dark';

/** 앱 UI 스킨 설정값 — 'flat'/'oldschool' 은 옵트인 스킨(theme-*.css) */
export type ThemeSkin = 'default' | 'flat' | 'oldschool';

/** 정규화가 참조하는 스킨 전체 목록 (theme-init.js 의 목록과 함께 갱신한다) */
export const THEME_SKINS: readonly ThemeSkin[] = ['default', 'flat', 'oldschool'];

/** 앱 UI 테마 설정 */
export interface ThemeSettings {
  /** 사용자가 선택한 테마 모드 */
  mode: ThemeMode;
  /** 사용자가 선택한 스킨 */
  skin: ThemeSkin;
  /** 스킨을 한 번이라도 직접 선택했는지 — false 면 첫 실행 스킨 선택을 안내한다 */
  skinChosen: boolean;
}

/** 대화상자 UI 설정 */
export interface DialogSettings {
  /** 개체 속성 기본 탭에서 너비/높이 입력 비율을 유지할지 여부 */
  picturePropsKeepRatio: boolean;
  /** PDF 저장 전에 브라우저 인쇄 대상 선택 방법을 안내할지 여부 */
  showPdfPrintGuidance: boolean;
}

/** 보기 표시 설정 */
export interface ViewSettings {
  /** 문단부호 표시 여부 */
  showParagraphMarks: boolean;
  /** 조판부호 표시 여부 */
  showControlCodes: boolean;
  /** 짤림보기(잘림 보기) 켜짐 여부. true = 편집용지 경계 밖 오버플로 내용을 보임(잘림 미적용). */
  clipView: boolean;
  /** 기본 도구 상자(아이콘 도구 모음) 표시 여부 */
  toolbarBasic: boolean;
  /** 서식 도구 상자(서식 도구 모음) 표시 여부 */
  toolbarFormat: boolean;
  /** 배율과 독립적으로 유지하는 페이지 화면 배치 */
  pageArrangement: PageArrangement;
  /** 쪽을 세로/가로 어느 방향으로 이어 볼지와 휠 변환 설정 */
  pageMovement: PageMovementSettings;
}

/** 복구용 자동저장 설정 */
export interface AutosaveSettings {
  /** 복구용 자동저장 사용 여부 */
  recoveryEnabled: boolean;
  /** 복구용 자동저장 간격(분) */
  recoveryIntervalMinutes: number;
  /** 입력이 멈췄을 때 자동저장 사용 여부 */
  idleSaveEnabled: boolean;
  /** 입력이 멈춘 뒤 자동저장까지 기다릴 시간(초) */
  idleDelaySeconds: number;
}

/** 전체 설정 구조 */
export interface AppSettings {
  version: number;
  font: FontSettings;
  theme: ThemeSettings;
  dialog: DialogSettings;
  view: ViewSettings;
  autosave: AutosaveSettings;
}

/** 언어 인덱스 상수 (HWP 7개 언어) */
export const LANG = {
  KOREAN: 0,
  ENGLISH: 1,
  CHINESE: 2,
  JAPANESE: 3,
  OTHER: 4,
  SYMBOL: 5,
  USER: 6,
} as const;

/** 언어 인덱스 → 한국어 라벨 */
export const LANG_LABELS = ['한글', '영문', '한자', '일어', '외국어', '기호', '사용자'] as const;

/** 언어 인덱스 → FontSet 키 매핑 */
const LANG_KEYS: (keyof Omit<FontSet, 'name'>)[] = [
  'korean', 'english', 'chinese', 'japanese', 'other', 'symbol', 'user',
];

/** 내장 기본 대표 글꼴 (편집/삭제 불가) */
export const BUILTIN_FONT_SETS: readonly FontSet[] = [
  {
    name: '함초롬',
    korean: '함초롬바탕', english: '함초롬바탕', chinese: '함초롬바탕',
    japanese: '함초롬바탕', other: '함초롬바탕', symbol: '함초롬바탕', user: '함초롬바탕',
  },
  {
    name: '함초롬돋움',
    korean: '함초롬돋움', english: '함초롬돋움', chinese: '함초롬돋움',
    japanese: '함초롬돋움', other: '함초롬돋움', symbol: '함초롬돋움', user: '함초롬돋움',
  },
  {
    name: '맑은 고딕',
    korean: '맑은 고딕', english: '맑은 고딕', chinese: '맑은 고딕',
    japanese: '맑은 고딕', other: '맑은 고딕', symbol: '맑은 고딕', user: '맑은 고딕',
  },
  {
    name: '바탕',
    korean: '바탕', english: '바탕', chinese: '바탕',
    japanese: '바탕', other: '바탕', symbol: '바탕', user: '바탕',
  },
];

const STORAGE_KEY = 'rhwp-settings';

function defaultSettings(): AppSettings {
  return {
    version: 1,
    font: {
      fontSets: [],
      showRecentFonts: true,
      recentFontCount: 3,
    },
    theme: {
      mode: 'system',
      skin: 'default',
      skinChosen: false,
    },
    dialog: {
      picturePropsKeepRatio: true,
      showPdfPrintGuidance: true,
    },
    view: {
      showParagraphMarks: false,
      showControlCodes: false,
      clipView: true,
      toolbarBasic: false,
      toolbarFormat: true,
      pageArrangement: { kind: 'auto' },
      pageMovement: { ...DEFAULT_PAGE_MOVEMENT },
    },
    autosave: {
      recoveryEnabled: true,
      recoveryIntervalMinutes: 10,
      idleSaveEnabled: true,
      idleDelaySeconds: 10,
    },
  };
}

/** 첫 실행 스킨 안내를 보여줄지 판단한다 (한 번도 직접 선택하지 않은 경우만). */
export function shouldShowSkinOnboarding(theme: Pick<ThemeSettings, 'skinChosen'>): boolean {
  return !theme.skinChosen;
}

function normalizeThemeMode(value: unknown): ThemeMode {
  return value === 'light' || value === 'dark' || value === 'system' ? value : 'system';
}

function normalizeThemeSkin(value: unknown): ThemeSkin {
  return THEME_SKINS.includes(value as ThemeSkin) ? (value as ThemeSkin) : 'default';
}

/**
 * 저장된 테마 설정을 정규화한다.
 *
 * `skinChosen` 이 없던 저장값(스킨 도입 초기 사용자)은 기본 스킨이 아니라는 사실로
 * '직접 선택함'을 복원해 첫 실행 안내를 건너뛴다.
 */
export function normalizeThemeSettings(parsed: Partial<ThemeSettings> | undefined): ThemeSettings {
  const skin = normalizeThemeSkin(parsed?.skin);
  return {
    mode: normalizeThemeMode(parsed?.mode),
    skin,
    skinChosen: normalizeBoolean(parsed?.skinChosen, skin !== 'default'),
  };
}

function normalizeBoolean(value: unknown, fallback: boolean): boolean {
  return typeof value === 'boolean' ? value : fallback;
}

function normalizeNumber(value: unknown, fallback: number, min: number, max: number): number {
  const number = typeof value === 'number' ? value : Number(value);
  if (!Number.isFinite(number)) return fallback;
  return Math.min(max, Math.max(min, Math.round(number)));
}

/** 사용자 환경설정 서비스 (싱글턴) */
class UserSettingsService {
  private data: AppSettings;

  constructor() {
    this.data = this.load();
  }

  private load(): AppSettings {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) return defaultSettings();
      const parsed = JSON.parse(raw) as Partial<AppSettings>;
      // 기본값 병합
      const defaults = defaultSettings();
      const dialog: Partial<DialogSettings> = parsed.dialog ?? {};
      const view: Partial<ViewSettings> = parsed.view ?? {};
      const autosave: Partial<AutosaveSettings> = parsed.autosave ?? {};
      const pageView = resolvePageViewSettings(view.pageArrangement, view.pageMovement);
      return {
        version: parsed.version ?? defaults.version,
        font: {
          ...defaults.font,
          ...(parsed.font ?? {}),
        },
        theme: normalizeThemeSettings(parsed.theme),
        dialog: {
          ...defaults.dialog,
          ...dialog,
          picturePropsKeepRatio: normalizeBoolean(
            dialog.picturePropsKeepRatio,
            defaults.dialog.picturePropsKeepRatio,
          ),
          showPdfPrintGuidance: normalizeBoolean(
            dialog.showPdfPrintGuidance,
            defaults.dialog.showPdfPrintGuidance,
          ),
        },
        view: {
          ...defaults.view,
          ...view,
          showParagraphMarks: normalizeBoolean(
            view.showParagraphMarks,
            defaults.view.showParagraphMarks,
          ),
          showControlCodes: normalizeBoolean(
            view.showControlCodes,
            defaults.view.showControlCodes,
          ),
          clipView: normalizeBoolean(
            view.clipView,
            defaults.view.clipView,
          ),
          toolbarBasic: normalizeBoolean(
            view.toolbarBasic,
            defaults.view.toolbarBasic,
          ),
          toolbarFormat: normalizeBoolean(
            view.toolbarFormat,
            defaults.view.toolbarFormat,
          ),
          pageArrangement: pageView.arrangement,
          pageMovement: pageView.movement,
        },
        autosave: {
          ...defaults.autosave,
          ...autosave,
          recoveryEnabled: normalizeBoolean(
            autosave.recoveryEnabled,
            defaults.autosave.recoveryEnabled,
          ),
          recoveryIntervalMinutes: normalizeNumber(
            autosave.recoveryIntervalMinutes,
            defaults.autosave.recoveryIntervalMinutes,
            1,
            120,
          ),
          idleSaveEnabled: normalizeBoolean(
            autosave.idleSaveEnabled,
            defaults.autosave.idleSaveEnabled,
          ),
          idleDelaySeconds: normalizeNumber(
            autosave.idleDelaySeconds,
            defaults.autosave.idleDelaySeconds,
            5,
            600,
          ),
        },
      };
    } catch {
      return defaultSettings();
    }
  }

  save(): void {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(this.data));
  }

  /** 전체 설정 반환 */
  getAll(): AppSettings {
    return this.data;
  }

  /** 글꼴 설정 반환 */
  getFontSettings(): FontSettings {
    return this.data.font;
  }

  /** 글꼴 설정 업데이트 */
  updateFontSettings(partial: Partial<FontSettings>): void {
    Object.assign(this.data.font, partial);
    this.save();
  }

  /** 테마 설정 반환 */
  getThemeSettings(): ThemeSettings {
    return this.data.theme;
  }

  /** 테마 모드 설정 */
  setThemeMode(mode: ThemeMode): void {
    this.data.theme.mode = normalizeThemeMode(mode);
    this.save();
  }

  /** 스킨 설정 — 직접 선택이므로 첫 실행 안내 플래그도 함께 확정한다 */
  setThemeSkin(skin: ThemeSkin): void {
    this.data.theme.skin = normalizeThemeSkin(skin);
    this.data.theme.skinChosen = true;
    this.save();
  }

  /** 스킨 선택을 확정 처리한다 (첫 실행 안내를 닫기만 한 경우 포함) */
  markSkinChosen(): void {
    if (this.data.theme.skinChosen) return;
    this.data.theme.skinChosen = true;
    this.save();
  }

  /** 대화상자 UI 설정 반환 */
  getDialogSettings(): DialogSettings {
    return this.data.dialog;
  }

  /** 개체 속성 기본 탭 비율 유지 설정 반환 */
  getPicturePropsKeepRatio(): boolean {
    return this.data.dialog.picturePropsKeepRatio;
  }

  /** 개체 속성 기본 탭 비율 유지 설정 */
  setPicturePropsKeepRatio(value: boolean): void {
    this.data.dialog.picturePropsKeepRatio = value;
    this.save();
  }

  /** PDF 저장 전 브라우저 인쇄 대상 안내 표시 설정 반환 */
  getShowPdfPrintGuidance(): boolean {
    return this.data.dialog.showPdfPrintGuidance;
  }

  /** PDF 저장 전 브라우저 인쇄 대상 안내 표시 설정 */
  setShowPdfPrintGuidance(value: boolean): void {
    this.data.dialog.showPdfPrintGuidance = value;
    this.save();
  }

  /** 보기 표시 설정 반환 */
  getViewSettings(): ViewSettings {
    return this.data.view;
  }

  /** 문단부호 표시 설정 */
  setShowParagraphMarks(value: boolean): void {
    this.data.view.showParagraphMarks = value;
    this.save();
  }

  /** 조판부호 표시 설정 */
  setShowControlCodes(value: boolean): void {
    this.data.view.showControlCodes = value;
    this.save();
  }

  /** 짤림보기(잘림 보기) 켜짐 설정. true = 오버플로 내용 표시(잘림 미적용). */
  setClipView(value: boolean): void {
    this.data.view.clipView = value;
    this.save();
  }

  /** 기본 도구 상자 표시 설정 */
  setToolbarBasic(value: boolean): void {
    this.data.view.toolbarBasic = value;
    this.save();
  }

  /** 서식 도구 상자 표시 설정 */
  setToolbarFormat(value: boolean): void {
    this.data.view.toolbarFormat = value;
    this.save();
  }

  /** 페이지 화면 배치 설정 */
  setPageArrangement(value: PageArrangement): void {
    this.data.view.pageArrangement = resolvePageViewSettings(
      value,
      this.data.view.pageMovement,
    ).arrangement;
    this.save();
  }

  /** 페이지를 세로/가로 어느 방향으로 이어 볼지 저장한다. */
  setPageMovement(value: PageMovementSettings): void {
    const pageView = resolvePageViewSettings(this.data.view.pageArrangement, value);
    this.data.view.pageArrangement = pageView.arrangement;
    this.data.view.pageMovement = pageView.movement;
    this.save();
  }

  /** 복구용 자동저장 설정 반환 */
  getAutosaveSettings(): AutosaveSettings {
    return this.data.autosave;
  }

  /** 복구용 자동저장 설정 */
  updateAutosaveSettings(partial: Partial<AutosaveSettings>): void {
    this.data.autosave = {
      ...this.data.autosave,
      ...partial,
      recoveryEnabled: normalizeBoolean(
        partial.recoveryEnabled,
        this.data.autosave.recoveryEnabled,
      ),
      recoveryIntervalMinutes: normalizeNumber(
        partial.recoveryIntervalMinutes,
        this.data.autosave.recoveryIntervalMinutes,
        1,
        120,
      ),
      idleSaveEnabled: normalizeBoolean(
        partial.idleSaveEnabled,
        this.data.autosave.idleSaveEnabled,
      ),
      idleDelaySeconds: normalizeNumber(
        partial.idleDelaySeconds,
        this.data.autosave.idleDelaySeconds,
        5,
        600,
      ),
    };
    this.save();
  }

  /** 모든 대표 글꼴 세트 반환 (내장 + 사용자) */
  getAllFontSets(): FontSet[] {
    return [...BUILTIN_FONT_SETS, ...this.data.font.fontSets];
  }

  /** 사용자 정의 대표 글꼴 세트만 반환 */
  getUserFontSets(): FontSet[] {
    return this.data.font.fontSets;
  }

  /** 대표 글꼴 세트 추가 */
  addFontSet(fs: FontSet): boolean {
    const allNames = this.getAllFontSets().map(s => s.name);
    if (allNames.includes(fs.name)) return false; // 중복 이름 불가
    this.data.font.fontSets.push(fs);
    this.save();
    return true;
  }

  /** 대표 글꼴 세트 수정 (사용자 정의만) */
  updateFontSet(index: number, fs: FontSet): boolean {
    if (index < 0 || index >= this.data.font.fontSets.length) return false;
    this.data.font.fontSets[index] = fs;
    this.save();
    return true;
  }

  /** 대표 글꼴 세트 삭제 (사용자 정의만) */
  removeFontSet(index: number): boolean {
    if (index < 0 || index >= this.data.font.fontSets.length) return false;
    this.data.font.fontSets.splice(index, 1);
    this.save();
    return true;
  }

  /** FontSet의 언어 인덱스로 글꼴 이름 조회 */
  static getFontByLang(fs: FontSet, langIndex: number): string {
    return fs[LANG_KEYS[langIndex] ?? 'korean'] ?? fs.korean;
  }

  /** FontSet에 언어 인덱스로 글꼴 이름 설정 */
  static setFontByLang(fs: FontSet, langIndex: number, fontName: string): void {
    const key = LANG_KEYS[langIndex];
    if (key) (fs as any)[key] = fontName;
  }
}

/** 싱글턴 인스턴스 */
export const userSettings = new UserSettingsService();
