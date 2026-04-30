import { ButtonBuilder, ButtonStyle, ContainerBuilder } from "discord.js";
import type { Collection } from "mongodb";
import type { BookPrimitive, Book } from "../commands/read";
import { log } from "node:console";


export async function render_page(book_id: string, start_id: string, max_caharacters: number, primitives: Collection<BookPrimitive>, book: Collection<Book>): Promise<ContainerBuilder[]> {
  const error = new ContainerBuilder().setAccentColor(0x242429).addTextDisplayComponents(t => t.setContent("Error Could not find that part of the book"));
  let entry = await primitives.findOne({_id: start_id, book_id: book_id });

  if (entry === null) return [error];

  const this_book_start = get_previous_chapter(entry);
  let text_buffer: string[] = ["# " + entry.reference.book + " " + entry.reference.chapter + "\n"];

  while (true) {
    let buffer = entry.type == "title" ? "### " : ""
    buffer += entry.reference_number != 0 ? to_superscript(entry.reference_number) : "";
    buffer += entry.content + '\n';

    text_buffer.push(buffer)

    if (make_string(text_buffer).length + buffer.length > max_caharacters) break;
    if (entry.next.slice(0, 6) != entry._id.slice(0, 6) && entry.next != "" && entry._id != "") break;

    const pre_entry: BookPrimitive | null = await primitives.findOne({_id: entry.next, book_id: book_id});
    if (pre_entry === null) break;
    entry = pre_entry;
  }

  const previous_id = await find_previous_page_start(book_id, start_id, max_caharacters, primitives);

  const chapter_text = text_buffer.shift() ?? "IDK";

  const next_chapter = await get_next_chapter(start_id, book_id, book);

  const container = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addSectionComponents(t => t
      .addTextDisplayComponents(t => t.setContent(chapter_text))
      .setButtonAccessory(new ButtonBuilder().setEmoji("<:back:1499176748909330482>").setLabel("Back").setCustomId("test").setStyle(ButtonStyle.Secondary))
    )
    .addTextDisplayComponents(t => t.setContent(make_string(text_buffer)))
    .addActionRowComponents(ar => ar
      .addComponents(new ButtonBuilder().setEmoji("<:previous_button:1499160154828963940>").setStyle(ButtonStyle.Secondary).setCustomId("rn-" + entry.book_id + "-" + previous_id).setDisabled(previous_id == ""))
      .addComponents(new ButtonBuilder().setEmoji("<:share_to_channel:1499153256935592067>").setStyle(ButtonStyle.Secondary).setCustomId("rs-" + entry.book_id + "-" + start_id).setDisabled(start_id == ""))
      .addComponents(new ButtonBuilder().setEmoji("<:next_button:1499159772258242600>").setStyle(ButtonStyle.Secondary).setCustomId("rn-" + entry.book_id + "-" + entry.next).setDisabled(entry.next == ""))
    )
    .addActionRowComponents(ar => ar
      .addComponents(new ButtonBuilder().setEmoji("<:previous_button_stop:1499162066236211350>").setStyle(ButtonStyle.Secondary).setCustomId("rn--" + entry.book_id + "-" + this_book_start).setDisabled(previous_id == ""))
      .addComponents(new ButtonBuilder().setEmoji("<:highlighter:1499170569818734642>").setStyle(ButtonStyle.Secondary).setCustomId("2").setDisabled(start_id == ""))
      .addComponents(new ButtonBuilder().setEmoji("<:next_button_stop:1499162049375240262>").setStyle(ButtonStyle.Secondary).setCustomId("rn--" + entry.book_id + "-" + next_chapter).setDisabled(entry.next == ""))
    )

  return [container]
}

export async function find_previous_page_start(book_id: string, start_id: string, max_characters: number, primitives: Collection<BookPrimitive>): Promise<string> {// AI WROTE THIS IDK WHAT IT DOSE
  let entry = await primitives.findOne({_id: start_id, book_id: book_id});
  if (entry === null) return "";

  entry = await primitives.findOne({_id: entry.previous, book_id: book_id})
  if (entry === null) return "";

  const chapter_prefix = entry._id.slice(0, 6);
  const chapter_start_id = chapter_prefix + "001";
  
  let chapterEntry = await primitives.findOne({_id: chapter_start_id, book_id: book_id});
  if (chapterEntry === null) return "";

  let chunks = [chapter_start_id];
  let total = chapterEntry.reference.chapter.length + chapterEntry.reference.book.length + 5;

  while (true) {
    const contentLength = chapterEntry.content.length;
    const refLength = chapterEntry.reference_number != 0 ? chapterEntry.reference_number.toString().length : 0;
    const typeLength = chapterEntry.type == "title" ? 4 : 0;
    const entrySize = contentLength + refLength + typeLength + 2;

    total += entrySize;

    if (total > max_characters) {
      const nextEntry = await primitives.findOne({_id: chapterEntry.next, book_id: book_id});
      if (nextEntry === null || nextEntry._id.slice(0, 6) !== chapter_prefix) {
        break;
      }
      
      chunks.push(nextEntry._id);
      total = nextEntry.reference.chapter.length + nextEntry.reference.book.length + 4;
      chapterEntry = nextEntry;
    } else {
      const nextEntry = await primitives.findOne({_id: chapterEntry.next, book_id: book_id});
      if (nextEntry === null || nextEntry._id.slice(0, 6) !== chapter_prefix) {
        break;
      }
      
      chapterEntry = nextEntry;
    }
  }

  const previousEntryNumber = Number(entry._id.slice(3, 9));
  
  for (let i = chunks.length - 1; i >= 0; i--) {
    const chunkNumber = Number(chunks[i].slice(3, 9));
    
    if (chunkNumber <= previousEntryNumber) {
      return chunks[i]!;
    }
  }

  return chunks[0]!;
}

async function get_next_chapter(start_id: string, book_id: string, book_db: Collection<Book>) {
  const book = await book_db.findOne({_id: book_id});
  if (book === null) return "";

  const this_book_id = start_id.slice(0,3);

  const entries = Object.entries(book.books);

  const this_book_index = entries.findIndex(([key]) => key === this_book_id);
  
  const next = this_book_index !== -1 ? entries[this_book_index + 1] : undefined;

  return next?.[0]! + "001001";
}

function get_previous_chapter(primitive: BookPrimitive) {
  console.log(primitive.previous.slice(0,3) + "001001", primitive);
  
  return primitive.previous.slice(0,3) + "001001"
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
