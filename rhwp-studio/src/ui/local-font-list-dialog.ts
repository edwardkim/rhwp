import { getLocalFontRecords } from '@/core/local-fonts';
import { t } from '../i18n/index.ts';
import { ModalDialog } from './dialog';

/** 브라우저가 감지한 face와 이름을 검색해 확인한다. */
export class LocalFontListDialog extends ModalDialog {
  constructor() {
    super(t('dialog.localFontList.title'), 760);
  }

  override show(): void {
    super.show();
    const footer = this.dialog.querySelector('.dialog-footer');
    const close = footer?.querySelector<HTMLButtonElement>('.dialog-btn-primary');
    if (close) close.textContent = t('dialog.localFontList.close');
    footer?.querySelector('.dialog-btn:not(.dialog-btn-primary)')?.remove();
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    const search = document.createElement('input');
    search.type = 'search';
    search.className = 'dialog-input local-font-list-search';
    search.placeholder = t('dialog.localFontList.search');
    search.setAttribute('aria-label', search.placeholder);
    const count = document.createElement('p');
    const list = document.createElement('div');
    list.className = 'local-font-list-scroll';
    const table = document.createElement('table');
    table.className = 'local-font-list-table';
    const header = document.createElement('tr');
    for (const label of ['family', 'names', 'style', 'postscript']) {
      const th = document.createElement('th');
      th.textContent = t(`dialog.localFontList.${label}`);
      header.appendChild(th);
    }
    const head = document.createElement('thead');
    head.appendChild(header);
    const rows = document.createElement('tbody');
    table.append(head, rows);
    list.appendChild(table);
    const records = getLocalFontRecords({ includeRegistered: true })
      .sort((a, b) => a.displayName.localeCompare(b.displayName));
    const render = (): void => {
      const query = search.value.trim().toLocaleLowerCase();
      const selected = records.filter(record =>
        [record.family, record.fullName, record.postscriptName, record.style, ...record.aliases]
          .some(value => value.toLocaleLowerCase().includes(query)));
      count.textContent = t('dialog.localFontList.count', { p1: selected.length, p2: records.length });
      rows.replaceChildren();
      for (const record of selected) {
        const row = document.createElement('tr');
        const names = [...new Set([record.displayName, record.fullName, ...record.aliases])]
          .filter(name => name && name !== record.family && name !== record.postscriptName)
          .join(' · ');
        const style = record.detectionSource === 'font-presence-probe'
          ? t('dialog.localFontList.presenceOnly') : record.style;
        for (const value of [record.family, names, style, record.postscriptName]) {
          const cell = document.createElement('td');
          cell.textContent = value || '—';
          row.appendChild(cell);
        }
        rows.appendChild(row);
      }
    };
    search.addEventListener('input', render);
    render();
    body.append(search, count, list);
    return body;
  }

  protected onConfirm(): void {}
}
