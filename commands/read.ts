import { SlashCommandBuilder, ChatInputCommandInteraction, type ApplicationCommandOptionChoiceData, MessageFlags, ContainerBuilder } from "discord.js";
import type { Command } from "../deploy";
import { readdirSync, readFileSync } from "fs";
import path from "path";

type BookPart = {
  name: string | null;
  id: number;
  type: "chapter" | "section" | "paragraph";
  children: (BookPart | Sentence)[];
}

type Sentence = {
  name: string | null;
  id: number;
  type: "sentence" | "verse";
  content: string;
  foot_note: string | null;
}

type Book = {
  name: string;
  id: string;
  children: BookPart[];
};

export default {
  data: new SlashCommandBuilder()
    .setName('read')
    .setDescription('Read form a selection of books.')
    .addStringOption(
      option => option
        .setName('book')
        .setDescription('The book to read')
        .addChoices(...get_choices('./books/'))
        .setRequired(true)
    ),
  async execute(interaction: ChatInputCommandInteraction) {
    const book_id = interaction.options.getString('book');
    const book_raw = readFileSync("./books/" + book_id + ".json", "utf-8");
    if (book_raw === null) return interaction.reply({ content: "No Book of that name", flags: MessageFlags.Ephemeral });
    const book_data = JSON.parse(book_raw) as Book;

    const components = render_chapter(book_data, 0, 0, [0, 32])

    if (components === typeof "") return await interaction.reply({content: components, flags: [MessageFlags.Ephemeral]})

    for (const [i, component] of components.entries()) {
      console.log(component, i);
      if (i == 0) {
        await interaction.reply({ components: component, flags: [MessageFlags.IsComponentsV2] })
      } else {
        await interaction.followUp({components: component, flags: [MessageFlags.IsComponentsV2] })
      }
    }

  },
} as Command;

function get_choices(dir_path: string): ApplicationCommandOptionChoiceData<string>[] {
  let buffer = []

  const dir = readdirSync(dir_path);

  for (const file of dir) {
    const file_name = path.parse(file).name;
    const file_ext = path.parse(file).ext;

    if (file_name.startsWith('.')) continue;
    if (file_ext != ".json") continue;

    const file_data = readFileSync(dir_path + file, "utf-8");
    const file_json = JSON.parse(file_data);

    buffer.push({name: file_json.name, value: file_json.id})
  }

  return buffer
}

function render_chapter(book_data: Book, book_id: number, chapter: number, range: [number, number]): [ContainerBuilder[]] | string {
  let buffer: any = [];

  const chapters = book_data.children.find(i => i.id === book_id);
  if (chapters === undefined) return "No Chapters";
  if (chapters.children === undefined) return "No chapter data";
  const chapter_data: BookPart | undefined  = chapters.children.find((i): i is BookPart => i.id === chapter && !is_sentence(i));
  if (chapter_data === undefined) return "No chapter data";

  let sentence_buffer: {type: string, content: string}[] = (() => !!chapter_data.name ? [{type: chapter_data.type, content: chapters.name + " " + chapter_data.name}]:[])();
  sentence_buffer.push(...get_children(chapter_data));

  let text_display_buffer = ""

  for (const [index, sentence] of sentence_buffer.entries()) {
    const type = sentence.type;
    const content = sentence.content;
    if (text_display_buffer.length + (type == "sentence" || type == "verse" ? content.length + 5 : content.length + 6) > 4000) {
      buffer.push([new ContainerBuilder()
        .setAccentColor(0x242429)
        .addTextDisplayComponents(o => o.setContent(text_display_buffer))
      ])
      text_display_buffer = ""
    }

    if (type == "sentence" || type == "verse") {
      text_display_buffer += "  " + to_superscript(get_position(sentence_buffer, index) + 1) + content + "\n"
    } else if (type == "chapter") {
      text_display_buffer += "# " + content + "\n"
    } else {
      text_display_buffer += "### " + content + "\n"
    }
  }

  if (text_display_buffer.length > 0) {
    buffer.push([new ContainerBuilder()
      .setAccentColor(0x242429)
      .addTextDisplayComponents(o => o.setContent(text_display_buffer))
    ])
  }

  return buffer;
}

function get_children(children_data: BookPart): {type: string, content: string}[] {
  let buffer: {type: string, content: string}[]  = []

  for (const [index, child] of children_data.children.entries()) {
    if (is_sentence(child)) {
      buffer.push({ type: child.type, content: child.content })
    } else {
      if (!!child.name) buffer.push({ type: child.type, content: child.name });
      buffer.push(...get_children(child))
    }
  }

  return buffer;
}

function is_sentence(child: BookPart | Sentence): child is Sentence {
  return child.type === "sentence" || child.type === "verse";
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

function get_position(items: {type: string, content: string}[], index: number): number {
  const targetType = items[index]?.type;
  if (!targetType) return -1;

  let count = 0;

  for (let i = 0; i < items.length; i++) {
    if (items[i]!.type === targetType) {
      if (i === index) return count;
      count++;
    }
  }

  return -1; // index not found (shouldn't happen if valid input)
}
