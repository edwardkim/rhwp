export type SaveFormat = 'hml' | 'hwp' | 'hwpx';

export interface FilePickerType {
  description: string;
  accept: Record<string, string[]>;
}

export interface SaveFormatDetails {
  extension: '.hml' | '.hwp' | '.hwpx';
  mimeType: 'application/xml' | 'application/x-hwp' | 'application/hwp+zip';
  pickerType: FilePickerType;
}

export const SAVE_FORMAT_DETAILS: Record<SaveFormat, SaveFormatDetails> = {
  hml: {
    extension: '.hml',
    mimeType: 'application/xml',
    pickerType: {
      description: 'HML 문서',
      accept: { 'application/xml': ['.hml'] },
    },
  },
  hwp: {
    extension: '.hwp',
    mimeType: 'application/x-hwp',
    pickerType: {
      description: 'HWP 문서',
      accept: { 'application/x-hwp': ['.hwp'] },
    },
  },
  hwpx: {
    extension: '.hwpx',
    mimeType: 'application/hwp+zip',
    pickerType: {
      description: 'HWPX 문서',
      accept: { 'application/hwp+zip': ['.hwpx'] },
    },
  },
};
