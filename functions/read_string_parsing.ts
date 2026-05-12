import type { Book } from "../commands/read";

const regex =
  /^(\w+)[\s\-_|\\/~:;]*(\d+)?[\s\-_|\\/~:;]*(?:[\s\-_|\\/~:;](\d+))?$/i;

export function parse(string: string, document: Book) {
  const matches = string.match(regex);

  if (matches === null) {
    return Object.keys(document.books)[0];
  }

  const input = matches[1]?.toLowerCase();

  let book =
    Object.entries(document.books).find(([key, value]) =>
      key.toLowerCase() === input ||
      value.toLowerCase() === input
    )?.[0];

  if (!book) {
    book = Object.keys(document.books)[0];
  }

  const chapter = format(
    Math.min(
      Number(matches[2] ?? 0),
      document.chapters[book] ?? 1
    )
  );

  return book + (chapter !== "000" ? chapter : "");
}

function format(n: number): string {
  const clamped = Math.min(n, 999);
  return clamped.toString().padStart(3, "0");
}
