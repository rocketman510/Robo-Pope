import { ButtonBuilder, ButtonStyle, ContainerBuilder } from "discord.js";
import type { Book } from "../commands/read";

export function render(book: Book, chapter: string, starting_chapter: number) {
  if (!book.books[chapter] || !book.chapters[chapter]) return;

  const books = Object.entries(book.books);

  const container = new ContainerBuilder()
    .addTextDisplayComponents(t => t.setContent("# " + book.books[chapter]))

  let buffer = []
  let next_button_id: string | null;
  let previous_button_id: string | null;

  let index = 0
  for (let i = starting_chapter; i < book.chapters[chapter] + 1 && index < 25; i++) {
    index++
    buffer.push(new ButtonBuilder().setCustomId("rn-" + book._id + "-" + chapter + format(i) + "001").setLabel(i.toString()).setStyle(ButtonStyle.Secondary));

    if (i >= book.chapters[chapter]) {
      next_button_id = books[books.findIndex(([key]) => key === chapter) + 1]?.[0] + "001";
    } else {
      next_button_id = chapter + format(i+1);
    }
  }

  if (starting_chapter <= 1) {
    const previous_book = books[books.findIndex(([key]) => key === chapter) - 1]?.[0];
    if (previous_book !== undefined) {
      const previous_chapters = book.chapters[previous_book]!;
      const whole_pgs = Math.floor(previous_chapters/25);
      if ((previous_chapters - 1) % 25 == 0) {
        previous_button_id = previous_book + format((whole_pgs - 1) * 25);
      } else {
        previous_button_id = previous_book + format(previous_chapters - (previous_chapters - 1) % 25);
      }
    } else {
      previous_button_id = null
    }
  } else {
    previous_button_id = chapter + format(starting_chapter - 25);
  }


  while (0 < buffer.length) {
    const e = buffer.splice(0, 5);
    container.addActionRowComponents(ar => ar.addComponents(e))
  }

  const next_chapter_id = books[books.findIndex(([key]) => key === chapter) + 1]?.[0] ?? null

  const previous_chapter_id = (() => {
    if (starting_chapter == 1) {
      return books[books.findIndex(([key]) => key === chapter) - 1]?.[0] ?? null
    } else {
      return chapter;
    }
  })()

  container.addActionRowComponents(ar => ar.addComponents([
    new ButtonBuilder().setStyle(ButtonStyle.Secondary).setEmoji("<:previous_button_stop:1499162066236211350>").setCustomId("cp-" + book._id + "--" + previous_chapter_id + "001").setDisabled(previous_chapter_id === null),
    new ButtonBuilder().setStyle(ButtonStyle.Secondary).setEmoji("<:previous_button:1499160154828963940>").setCustomId("cp-" + book._id + "-" + previous_button_id).setDisabled(previous_button_id === null),
    new ButtonBuilder().setStyle(ButtonStyle.Secondary).setEmoji("<:next_button:1499159772258242600>").setCustomId("cp-" + book._id + "-" + next_button_id).setDisabled(next_button_id === null),
    new ButtonBuilder().setStyle(ButtonStyle.Secondary).setEmoji("<:next_button_stop:1499162049375240262>").setCustomId("cp-" + book._id + "--" + next_chapter_id + "001").setDisabled(next_chapter_id === null),
  ]))

  return [container]
}

function format(n: number): string {
  const clamped = Math.min(n, 999);
  return clamped.toString().padStart(3, '0');
}

/**
 * Returns the custom_id that corresponds to the screen containing a chapter. Chat GPT wrote idk how it works
 */
export function get_chapter_screen_id(book: Book, chapter: string, chapter_number: number): string | null {
  const chapters_in_book = book.chapters[chapter];
  if (!chapters_in_book) return null;

  const safe_chapter = Math.max(1, Math.min(chapter_number, chapters_in_book));

  const starting_chapter =
    Math.floor((safe_chapter - 1) / 25) * 25 + 1;

  return (
    "cp-" +
    book._id +
    "-" +
    chapter +
    format(starting_chapter)
  );
}
