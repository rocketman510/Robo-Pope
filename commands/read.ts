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

  const sentence_buffer = get_children(chapter_data);

  let text_display_buffer = ""

  for (const sentence of sentence_buffer) {
    const type = sentence.type;
    const content = sentence.content;
    if (text_display_buffer.length + (type == "sentence" || type == "verse" ? content.length + 1 : content.length + 6) > 4000) {
      buffer.push([new ContainerBuilder()
        .setAccentColor(0x242429)
        .addTextDisplayComponents(o => o.setContent(text_display_buffer))
      ])
      text_display_buffer = ""
    }

    if (type == "sentence" || type == "verse") {
      text_display_buffer += content + " "
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
