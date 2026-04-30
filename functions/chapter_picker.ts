import { ButtonBuilder, ButtonStyle, ContainerBuilder } from "discord.js";
import type { Book } from "../commands/read";

export function render(book: Book, chapter: string, starting_chapter: number) {
  if (!book.books[chapter] || !book.chapters[chapter]) return;

  const books = Object.entries(book.books);

  const container = new ContainerBuilder()
    .addTextDisplayComponents(t => t.setContent("# " + book.books[chapter]))

  let buffer = []
  let next_button: string;

  let index = 0
  for (let i = starting_chapter; i < book.chapters[chapter] + 1 && index < 25; i++) {
    index++
    buffer.push(new ButtonBuilder().setCustomId("rn-" + book._id + "-" + chapter + format(i) + "001").setLabel(i.toString()).setStyle(ButtonStyle.Secondary));

    if (i >= book.chapters[chapter]) {
      next_button = books[books.findIndex(([key]) => key === chapter) + 1]![0] + "001";
    } else {
      next_button = chapter + format(i+1);
    }
  }

  while (0 < buffer.length) {
    const e = buffer.splice(0, 5);
    container.addActionRowComponents(ar => ar.addComponents(e))
  }

  container.addActionRowComponents(ar => ar.addComponents([
    new ButtonBuilder().setStyle(ButtonStyle.Secondary).setEmoji("<:previous_button:1499160154828963940>").setCustomId("cp-1"),
    new ButtonBuilder().setStyle(ButtonStyle.Secondary).setEmoji("<:next_button:1499159772258242600>").setCustomId("cp-" + book._id + "-" + next_button)
  ]))

  return [container]
}

function format(n: number): string {
  const clamped = Math.min(n, 999);
  return clamped.toString().padStart(3, '0');
}

