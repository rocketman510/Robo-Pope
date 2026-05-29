import { ButtonBuilder, ButtonStyle, ContainerBuilder, StringSelectMenuBuilder, StringSelectMenuOptionBuilder, type Interaction } from "discord.js";
import type { Collection } from "mongodb";
import type { BookPrimitive, Book } from "../commands/read";
import { get_chapter_screen_id } from "./chapter_picker";
import { encode } from "../button/rs";
import type { HighlighterSetting } from "../button/rh";

export async function render_page(book_id: string, start_id: string, max_caharacters: number, interaction: Interaction, primitives: Collection<BookPrimitive>, documents: Collection<Book>, highlighter_settings: Collection<HighlighterSetting>): Promise<ContainerBuilder[]> {
  const error = new ContainerBuilder().setAccentColor(0x242429).addTextDisplayComponents(t => t.setContent("Error Could not find that part of the book"));
  let entry = await primitives.findOne({_id: start_id, book_id: book_id });

  let components_accumulator = 2 + 4 + 1

  if (entry === null) return [error];

  const previous_id = await find_previous_page_start(book_id, start_id, max_caharacters, primitives);

  const next_chapter = await get_next_chapter(start_id, book_id, documents);

  const book = await documents.findOne({_id: book_id})
  const back_custom_id = book ? get_chapter_screen_id(book, start_id.slice(0,3), Number(start_id.slice(3,6))) : null;

  const this_book_start = get_previous_chapter(entry);

  let container_buffer: ContainerBuilder[] = [];

  let container = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addSectionComponents(t => t
      .addTextDisplayComponents(t => t.setContent("# " + entry.reference.book + " " + entry.reference.chapter + "\n"))
      .setButtonAccessory(new ButtonBuilder().setEmoji("<:back:1499176748909330482>").setLabel("Back").setCustomId(back_custom_id ?? "").setStyle(ButtonStyle.Secondary).setDisabled(back_custom_id === null))
    )

  const colors = [
    0,
    0xF9E2AF,
    0xF58AA6,
    0xCAA6F7,
    0xA6E3A1,
    0x8BDBEB,
  ]

  let last_color = 0

  const user_highlights = await highlighter_settings.find({ 
    book_id: book_id, 
    user_id: interaction.user.id 
  }).toArray();

  const highlight_map = new Map(user_highlights.map(h => [h._id, h.color]));

  while (components_accumulator < 37) {
    components_accumulator += entry.type == "title" ? 1 : 3;

    const highlight_color = highlight_map.get(entry._id) ?? 0;
    const content = (entry.type == "title" ? "### ":"") + (entry.reference_number == 0 ? "":to_superscript(entry.reference_number)) + entry.content;

    if (highlight_color != last_color) {
      container_buffer.push(container);
      container = new ContainerBuilder().addTextDisplayComponents(t => t.setContent(content));

      if (highlight_color != 0) {
        container.setAccentColor(colors[highlight_color]);
      }
    } else {
      container.addTextDisplayComponents(t => t.setContent(content));
    }

    // Follow the pointer to the next primitive
    const pre_entry: BookPrimitive | null = await primitives.findOne({_id: entry.next, book_id: book_id});
    if (pre_entry === null) break;
    entry = pre_entry;
    last_color = highlight_color;
  }

  if (last_color != 0) {
    container_buffer.push(container);
    container = new ContainerBuilder();
  }

  container.addActionRowComponents(ar => ar
      .addComponents(new ButtonBuilder().setEmoji("<:previous_button:1499160154828963940>").setStyle(ButtonStyle.Secondary).setCustomId("rn-" + entry.book_id + "-" + previous_id).setDisabled(previous_id == ""))
      .addComponents(new ButtonBuilder().setEmoji("<:share_to_channel:1499153256935592067>").setStyle(ButtonStyle.Secondary).setCustomId("rp-" + entry.book_id + "-" + start_id).setDisabled(start_id == ""))
      .addComponents(new ButtonBuilder().setEmoji("<:next_button:1499159772258242600>").setStyle(ButtonStyle.Secondary).setCustomId("rn-" + entry.book_id + "-" + entry.next).setDisabled(entry.next == ""))
    )
    .addActionRowComponents(ar => ar
      .addComponents(new ButtonBuilder().setEmoji("<:previous_button_stop:1499162066236211350>").setStyle(ButtonStyle.Secondary).setCustomId("rn--" + entry.book_id + "-" + this_book_start).setDisabled(previous_id == ""))
      .addComponents(new ButtonBuilder().setEmoji("<:highlighter:1499170569818734642>").setStyle(ButtonStyle.Secondary).setCustomId("rh-" + entry.book_id + "-" + start_id).setDisabled(start_id == ""))
      .addComponents(new ButtonBuilder().setEmoji("<:next_button_stop:1499162049375240262>").setStyle(ButtonStyle.Secondary).setCustomId("rn--" + entry.book_id + "-" + next_chapter).setDisabled(next_chapter == ""))
    )

  container_buffer.push(container)

  return container_buffer
}

export async function find_previous_page_start(book_id: string, start_id: string, max_caharacters: number, primitives: Collection<BookPrimitive>): Promise<string> {// AI WROTE THIS IDK WHAT IT DOSE
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

    if (total > max_caharacters) {
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

  if (next === undefined) return "";

  return next[0] + "001001";
}

function get_previous_chapter(primitive: BookPrimitive) {
  return primitive.previous.slice(0,3) + "001001"
}

function make_string(arry: string[] | { value: string; id: string }[]): string {
  return arry
    .map(item => (typeof item === "string" ? item : item.value))
    .join("");
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
  "a": "ᵃ",
  "b": "ᵇ",
  "c": "ᶜ",
  "d": "ᵈ",
  "e": "ᵉ",
  "f": "ᶠ",
  "g": "ᵍ",
  "h": "ʰ",
  "i": "ⁱ",
  "j": "ʲ",
  "k": "ᵏ",
  "l": "ˡ",
  "m": "ᵐ",
  "n": "ⁿ",
  "o": "ᵒ",
  "p": "ᵖ",
  "r": "ʳ",
  "s": "ˢ",
  "t": "ᵗ",
  "u": "ᵘ",
  "v": "ᵛ",
  "w": "ʷ",
  "x": "ˣ",
  "y": "ʸ",
  "z": "ᶻ",
};

export function to_superscript(input: string | number): string {
  return input
    .toString()
    .split("")
    .map(char => superscriptMap[char] ?? char)
    .join("");
}

export async function render_primitives(primitives: BookPrimitive[]): Promise<ContainerBuilder[]> {
  let last_chapter = "# " + primitives[0]?.reference.book + " " + primitives[0]?.reference.chapter;

  let text = last_chapter;
  for (const primitive of primitives) {
    const this_chapter = "# " + primitive?.reference.book + " " + primitive?.reference.chapter;
    let pre_text = "";

    if (this_chapter != last_chapter) {
      last_chapter = this_chapter
      pre_text += "\n" + this_chapter
    }

    pre_text += "\n\n"
    pre_text += primitive.reference_number == 0 ? "" : to_superscript(primitive.reference_number)
    pre_text += primitive.content
    pre_text += primitive.foot_note.length > 0 ? "\n-# " + primitive.foot_note.join(", ") : ""

    if (text.length + pre_text.length >= 4000) {
      break
    } else {
      text += pre_text;
    }
  }

  const container = new ContainerBuilder()
    .addTextDisplayComponents(t => t.setContent(text))
  
  return [container]
}

export async function render_share(book_id: string, start_id: string, max_caharacters: number, primitives: Collection<BookPrimitive>, settings: boolean[]): Promise<ContainerBuilder[]> {
  const error = new ContainerBuilder().setAccentColor(0x242429).addTextDisplayComponents(t => t.setContent("Error Could not find that part of the book"));
  let entry = await primitives.findOne({_id: start_id, book_id: book_id });

  let components_accumulator = 2 + 4 + 1

  if (entry === null) return [error];

  let text_buffer: {value: string, id: string}[] = [{value: "# " + entry.reference.book + " " + entry.reference.chapter + "\n", id: ""}];

  while (true) {
    components_accumulator += entry.type == "title" ? 1 : 3;
    if (components_accumulator >= 40) break;

    let buffer = entry.type == "title" ? "### " : ""
    buffer += entry.reference_number != 0 ? to_superscript(entry.reference_number) : "";
    buffer += entry.content + '\n';

    text_buffer.push({value: buffer, id: entry._id})

    if (make_string(text_buffer).length + buffer.length > max_caharacters) break;
    if (entry.next.slice(0, 6) != entry._id.slice(0, 6) && entry.next != "" && entry._id != "") break;

    const pre_entry: BookPrimitive | null = await primitives.findOne({_id: entry.next, book_id: book_id});
    if (pre_entry === null) break;
    entry = pre_entry;
  }

  const chapter_text = text_buffer.shift() ?? {value: "IDK", id: ""};
  const chapter_is_selected = settings.shift() ?? false;

  let index = 0
  for (const {value} of text_buffer) {// fulls up settings
    if (value.startsWith("#")) {
      continue
    };
    if (settings[index] === undefined) {
      settings[index] = false
    };
    index++;
  }

  let chapter_settings = [...settings]
  chapter_settings.unshift(!chapter_is_selected);

  const container = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addSectionComponents(t => t
      .addTextDisplayComponents(t => t.setContent(chapter_text.value))
      .setButtonAccessory(new ButtonBuilder().setEmoji(get_selection_box(chapter_is_selected)).setLabel("Chapter").setCustomId("rp-" + book_id + "-" + start_id + "-" + boolArrayToBase64(chapter_settings)).setStyle(ButtonStyle.Secondary))
    )

  let share_accumulator: number[] = []
  index = 0
  for (const { value, id } of text_buffer) {
    if (value.startsWith("#")) {
      container.addTextDisplayComponents(t => t.setContent(value))
      continue
    };
    let next_settings = [...settings];
    next_settings[index] = !next_settings[index]
    next_settings.unshift(chapter_is_selected)

    container.addSectionComponents(e => e
      .addTextDisplayComponents(t => t.setContent(value))
      .setButtonAccessory(new ButtonBuilder().setEmoji(get_selection_box(settings[index] ?? false)).setCustomId("rp-" + book_id + "-" + start_id + "-" + boolArrayToBase64(next_settings)).setStyle(ButtonStyle.Secondary).setDisabled(chapter_is_selected))
    )

    if (settings[index]) {
      share_accumulator.push(Number(id.slice(6,9)))
    }
    index++;
  }

  let share_custionId;
  if (chapter_is_selected) {
    share_custionId = "rs-" + book_id + "-" + start_id.slice(0,6);
  } else {
    share_custionId = "rs-" + book_id + "-" + start_id.slice(0,6) + "-" + encode(share_accumulator);
  }

  container.addActionRowComponents(ar => ar.addComponents(
    new ButtonBuilder().setCustomId(share_custionId).setStyle(ButtonStyle.Success).setLabel("Share").setEmoji("<:share_to_channel_white:1500941470696472836>"),
    new ButtonBuilder().setCustomId("rn-" + book_id + "-" + start_id).setStyle(ButtonStyle.Danger).setLabel("Cancel")
  ))

  return [container]
}

function get_selection_box(bool: boolean): string {
  return bool ? "<:checkbox_filled:1500939873438404779>" : "<:checkbox_empty:1500939839313281234>";
}

export function boolArrayToBase64(bits: boolean[]): string {// CHAT-GPT WROTE THIS IDK WHAT ID DOSE
  const fixed = new Array(16).fill(false);

  for (let i = 0; i < Math.min(bits.length, 16); i++) {
    fixed[i] = bits[i];
  }

  const bytes = new Uint8Array(2); // 16 bits = 2 bytes

  for (let i = 0; i < 16; i++) {
    if (fixed[i]) {
      bytes[i >> 3] |= 1 << (7 - (i % 8));
    }
  }

  return Buffer.from(bytes).toString("base64");
}

export function base64ToBoolArray(base64: string): boolean[] {// CHAT-GPT WROTE THIS IDK WHAT ID DOSE
  const bytes = new Uint8Array(Buffer.from(base64, "base64"));

  const bits: boolean[] = new Array(16).fill(false);

  for (let i = 0; i < 16; i++) {
    const byte = bytes[i >> 3];
    bits[i] = (byte & (1 << (7 - (i % 8)))) !== 0;
  }

  return bits;
}

export async function render_highlighting(book_id: string, start_id: string, primitives: Collection<BookPrimitive>, interaction: Interaction, highlighter_settings: Collection<HighlighterSetting>) {
  const error = new ContainerBuilder().setAccentColor(0x242429).addTextDisplayComponents(t => t.setContent("Error Could not find that part of the book"));

  let entry: BookPrimitive | null = await primitives.findOne({_id: start_id, book_id});
  if (!entry) return [error];

  let components_accumulator = 3 + 1 + 2 // Title, Container, Drop Down

  const highlighter_color = interaction.client.highlight_color.ensure(interaction.user.id, () => 1)

  const container = new ContainerBuilder()
    .addSectionComponents(s => s
      .addTextDisplayComponents(t => t.setContent("# " + entry!.reference.book + " " + entry!.reference.chapter))
      .setButtonAccessory(new ButtonBuilder().setCustomId("rn-" + book_id + "-" + start_id).setEmoji("<:back:1499176748909330482>").setLabel("Back").setStyle(ButtonStyle.Secondary))
    )

  let index = 0
  while (components_accumulator < 37) {
    const emojis = [
      "<:colorpicker_empty:1502124148913344593>",
      "<:colorpicker_yellow:1502124141380108330>",
      "<:colorpicker_red:1502124118948970608>",
      "<:colorpicker_purple:1502124109461454858>",
      "<:colorpicker_green:1502124134472089710>",
      "<:colorpicker_blue:1502124127325130862>",
      "",
    ]
    emojis[7] = emojis[highlighter_color] ?? ""

    const emoji = (await highlighter_settings.findOne({ _id: entry._id, book_id: entry.book_id, user_id: interaction.user.id }))?.color ?? 0;

    container.addSectionComponents(s => s
      .addTextDisplayComponents(t => t.setContent((entry!.type == "title" ? "### ":"") + entry!.content))
      .setButtonAccessory(new ButtonBuilder().setCustomId("rh-" + entry!.book_id + "-" + start_id + "-" + entry!._id.slice(6,9)).setEmoji(emojis[emoji] ?? "<:colorpicker_empty:1502124148913344593>").setStyle(ButtonStyle.Secondary))
    )

    const pre_entry: BookPrimitive | null = await primitives.findOne({_id: entry.next, book_id});
    if (pre_entry === null) break;
    entry = pre_entry;
    components_accumulator += 3;
    index++;
  }

  const drop_down = new StringSelectMenuBuilder()
    .setCustomId('colorpicker')
    .addOptions(
      new StringSelectMenuOptionBuilder()
        .setLabel('Yellow')
        .setEmoji('<:colorpicker_yellow:1502124141380108330>')
        .setValue('yellow')
        .setDefault(highlighter_color == 1),
      new StringSelectMenuOptionBuilder()
        .setLabel('Red')
        .setEmoji('<:colorpicker_red:1502124118948970608>')
        .setValue('red')
        .setDefault(highlighter_color == 2),
      new StringSelectMenuOptionBuilder()
        .setLabel('Purple')
        .setEmoji('<:colorpicker_purple:1502124109461454858>')
        .setValue('purple')
        .setDefault(highlighter_color == 3),
      new StringSelectMenuOptionBuilder()
        .setLabel('Green')
        .setEmoji('<:colorpicker_green:1502124134472089710>')
        .setValue('green')
        .setDefault(highlighter_color == 4),
      new StringSelectMenuOptionBuilder()
        .setLabel('Blue')
        .setEmoji('<:colorpicker_blue:1502124127325130862>')
        .setValue('blue')
        .setDefault(highlighter_color == 5),
    )

  container.addActionRowComponents(ar => ar.addComponents(drop_down))

  return [container]
}
