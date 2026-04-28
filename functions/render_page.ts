import { ButtonBuilder, ButtonStyle, ContainerBuilder } from "discord.js";
import type { Collection } from "mongodb";
import type { BookPrimitive } from "../commands/read";


export async function render_page(book_id: string, start_id: string, max_caharacters: number, primitives: Collection<BookPrimitive>): Promise<ContainerBuilder[]> {
  const error = new ContainerBuilder().setAccentColor(0x242429).addTextDisplayComponents(t => t.setContent("Error Could not find that part of the book"));
  let entry = await primitives.findOne({_id: start_id, book_id: book_id });

  console.log(start_id, book_id);
  
  console.log(entry);
  

  if (entry === null) return [error];

  let text_buffer: string[] = ["# " + entry.reference.book + " " + entry.reference.chapter + "\n"];

  while (true) {
    let buffer = entry.type == "title" ? "### " : ""
    buffer += entry.reference_number != 0 ? to_superscript(entry.reference_number) : "";
    buffer += entry.content + '\n';

    entry = await primitives.findOne({_id: entry.next, book_id: book_id});
    if (entry === null) break;

    if (make_string(text_buffer).length + buffer.length > max_caharacters) break;
    if (entry.next.slice(0, 6) != entry.previous.slice(0, 6) && entry.next != "" && entry.previous != "") break;

    text_buffer.push(buffer)
  }

  const container = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addTextDisplayComponents(t => t.setContent(make_string(text_buffer)))
    .addActionRowComponents(ar => ar
      .addComponents(new ButtonBuilder().setLabel("Next").setStyle(ButtonStyle.Secondary).setCustomId("rn-" + entry.book_id + "-" + entry.next))
    )

  return [container]
}

function make_string(arry:string[]): string {
  let result = ""
  for (const string of arry) {
    result += string;
  }
  return result;
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
