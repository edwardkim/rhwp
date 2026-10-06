/**
 * 도형 선택 드롭다운 — 도구상자 "도형" 버튼 클릭 시 표시
 * 한컴 스타일 층구조: 그리기 개체 / 연결선
 */

import { t } from '../i18n/index.ts';
export type ShapeType = 'line' | 'rectangle' | 'ellipse' | 'polygon' | 'arc'
  | 'connector-straight' | 'connector-stroke' | 'connector-arc'
  | 'connector-straight-arrow' | 'connector-stroke-arrow' | 'connector-arc-arrow';

export interface ShapePickerOptions {
  onSelect: (type: ShapeType) => void;
}

interface ShapeGroup {
  title: string;
  columns: number;
  items: { type: ShapeType; label: string; icon: string }[];
}

const GROUPS: ShapeGroup[] = [
  {
    title: t('dialog.shapePicker.tooltip'),
    columns: 5,
    items: [
      { type: 'line',      label: t('dialog.shapePicker.label'),   icon: '╲' },
      { type: 'rectangle', label: t('dialog.shapePicker.label.x397378'), icon: '▭' },
      { type: 'ellipse',   label: t('dialog.shapePicker.label.x1d636c'),   icon: '⬭' },
      { type: 'polygon',   label: t('dialog.shapePicker.label.x496f50'), icon: '△' },
      { type: 'arc',       label: t('dialog.shapePicker.label.x86220f'),     icon: '⌒' },
    ],
  },
  {
    title: t('dialog.shapePicker.tooltip.x7e90bf'),
    columns: 3,
    items: [
      { type: 'connector-straight',       label: t('dialog.shapePicker.label'),         icon: '─' },
      { type: 'connector-straight-arrow', label: t('dialog.shapePicker.label.xf3879e'), icon: '→' },
      { type: 'connector-stroke',         label: t('dialog.shapePicker.label.x9d1e71'),         icon: '⌐' },
      { type: 'connector-stroke-arrow',   label: t('dialog.shapePicker.label.x558836'), icon: '⮎' },
      { type: 'connector-arc',            label: t('dialog.shapePicker.label.xd79077'),         icon: '∼' },
      { type: 'connector-arc-arrow',      label: t('dialog.shapePicker.label.x75a223'), icon: '↝' },
    ],
  },
];

let currentPicker: HTMLDivElement | null = null;

function closePicker(): void {
  if (currentPicker) {
    currentPicker.remove();
    currentPicker = null;
  }
  document.removeEventListener('mousedown', onOutsideClick, true);
}

function onOutsideClick(e: MouseEvent): void {
  if (currentPicker && !currentPicker.contains(e.target as Node)) {
    closePicker();
  }
}

export function showShapePicker(anchorEl: HTMLElement, opts: ShapePickerOptions): void {
  if (currentPicker) { closePicker(); return; }

  const panel = document.createElement('div');
  panel.className = 'shape-picker';

  for (const group of GROUPS) {
    // 그룹 제목
    const title = document.createElement('div');
    title.className = 'shape-picker-title';
    title.textContent = group.title;
    panel.appendChild(title);

    // 아이콘 그리드
    const grid = document.createElement('div');
    grid.className = 'shape-picker-grid';
    grid.style.gridTemplateColumns = `repeat(${group.columns}, 1fr)`;

    for (const shape of group.items) {
      const btn = document.createElement('button');
      btn.className = 'shape-picker-btn';
      btn.title = shape.label;
      const icon = document.createElement('span');
      icon.className = 'shape-picker-icon';
      icon.textContent = shape.icon;
      const label = document.createElement('span');
      label.className = 'shape-picker-label';
      label.textContent = shape.label;
      btn.appendChild(icon);
      btn.appendChild(label);
      btn.addEventListener('click', () => {
        closePicker();
        opts.onSelect(shape.type);
      });
      grid.appendChild(btn);
    }

    panel.appendChild(grid);
  }

  // 위치 계산
  const rect = anchorEl.getBoundingClientRect();
  panel.style.position = 'fixed';
  panel.style.left = `${rect.left}px`;
  panel.style.top = `${rect.bottom + 2}px`;

  document.body.appendChild(panel);
  currentPicker = panel;

  setTimeout(() => {
    document.addEventListener('mousedown', onOutsideClick, true);
  }, 0);
}
