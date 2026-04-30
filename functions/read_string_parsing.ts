import type { Book } from "../commands/read";
const regex = /^(\w+)[\s-_|\\/~:;]*(\d+)?[\s-_|\\/~:;]*(?:[\s-_|\\/~:;](\d+))?$/i

export function parse(string: string, document: Book) {
  const matches = string.match(regex);
  if (matches === null) return Object.keys(document.books)[0];

  let book: string | undefined = matches[1];

  if (book !== undefined && (book in document.books || Object.values(document.books).includes(book))) {
    book = Object.entries(document.books).find(([k]) => k == book)?.[0] || book
  } else {
    book = Object.keys(document.books)[0]
  }

  let chapter = format(Math.min(Number(matches[2] ?? 0), document.chapters[book!] ?? 1));
  let primitive = Number(matches[3]);

  return book + (chapter != "000" ? chapter : "");
}

function format(n: number): string {
  const clamped = Math.min(n, 999);
  return clamped.toString().padStart(3, '0');
}

