import Quill from 'quill-next';
import type { EmitterSource } from 'quill-next';
import { htmlToMarkdown, markdownToHtml } from './markdown-codec';

const TOOLBAR = [
  ['bold', 'italic'],
  [{ header: [2, 3, false] }],
  [{ list: 'ordered' }, { list: 'bullet' }],
  ['blockquote', 'link'],
];

type ChangeHandler = (markdown: string) => void;

export type QuillHostOptions = {
  ariaLabel: string;
  onChange: ChangeHandler;
  onBlur: () => void;
};

export class QuillMarkdownHost {
  #quill: Quill;
  #onChange: ChangeHandler;
  #onBlur: () => void;
  #lastMarkdown = '';
  #loading = false;

  constructor(mount: HTMLElement, options: QuillHostOptions) {
    this.#onChange = options.onChange;
    this.#onBlur = options.onBlur;
    this.#quill = new Quill(mount, {
      theme: 'snow',
      modules: { toolbar: TOOLBAR },
    });
    this.#quill.root.setAttribute('role', 'textbox');
    this.#quill.root.setAttribute('aria-label', options.ariaLabel);
    this.#quill.on('text-change', this.#handleTextChange);
    this.#quill.root.addEventListener('blur', this.#handleBlur);
  }

  setMarkdown(markdown: string): void {
    if (markdown === this.#lastMarkdown) return;
    this.#loading = true;
    const html = markdownToHtml(markdown);
    this.#quill.setContents([], Quill.sources.SILENT);
    if (html !== '') {
      this.#quill.clipboard.dangerouslyPasteHTML(html, Quill.sources.SILENT);
    }
    this.#lastMarkdown = markdown;
    this.#loading = false;
  }

  getMarkdown(): string {
    const markdown = htmlToMarkdown(this.#quill.getSemanticHTML());
    this.#lastMarkdown = markdown;
    return markdown;
  }

  destroy(): void {
    this.#quill.root.removeEventListener('blur', this.#handleBlur);
    this.#quill.off('text-change', this.#handleTextChange);
    this.#quill.destroy();
  }

  #handleTextChange = (_delta: unknown, _old: unknown, source: EmitterSource): void => {
    if (this.#loading || source !== Quill.sources.USER) return;
    this.#onChange(this.getMarkdown());
  };

  #handleBlur = (): void => {
    this.#onBlur();
  };
}
