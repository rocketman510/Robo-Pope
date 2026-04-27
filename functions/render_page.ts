import { ButtonBuilder, ButtonStyle, ContainerBuilder } from "discord.js";
import type { Collection } from "mongodb";
import type { BookPrimitive } from "../commands/read";


export async function render_page(book_id: string, start_id: string, max_caharacters: number, primitives: Collection<BookPrimitive>): Promise<ContainerBuilder[]> {
  const error = new ContainerBuilder().setAccentColor(0x242429).addTextDisplayComponents(t => t.setContent("Error Could not find that part of the book"));
  let entry = await primitives.findOne({_id: start_id, book_id: book_id });

  if (entry === null) return [error];

  let text_buffer = "# " + entry.reference.book + " " + entry.reference.chapter + "\n";

  while (text_buffer.length + entry.content.length < max_caharacters) {
    text_buffer += entry.type == "title" ? "### " : ""
    text_buffer += entry.reference_number != 0 ? to_superscript(entry.reference_number) : "";
    text_buffer += entry.content + '\n';

    entry = await primitives.findOne({_id: entry.next, book_id: book_id});
    if (entry === null) return [error];
  }

  const container = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addTextDisplayComponents(t => t.setContent(text_buffer))
    .addActionRowComponents(ar => ar
      .addComponents(new ButtonBuilder().setLabel("Next").setStyle(ButtonStyle.Secondary).setCustomId("rn-" + entry.next))
    )

  return [container]
}

const superscriptMap: Record<string, string> = {
  "0": "⁰",
  "1": "¹",
  "2": "²",
  "3": "³",
  "4": "⁴",
  "5": "⁵",
  "6": "⁶",
  "7": "⁷",
  "8": "⁸",
  "9": "⁹",
};

function to_superscript(num: number): string {
  return num
    .toString()
    .split("")
    .map(d => superscriptMap[d])
    .join("");
}
