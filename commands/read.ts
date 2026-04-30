import { SlashCommandBuilder, ChatInputCommandInteraction, type ApplicationCommandOptionChoiceData, MessageFlags } from "discord.js";
import type { Command } from "../deploy";
import { readdirSync, readFileSync } from "fs";
import path from "path";
import { render_page } from "../functions/render_page";
import { render } from "../functions/chapter_picker";

export type BookPrimitive = {
  _id: string;
  book_id: string;
  absolute_index: number;
  content: string;
  type: string;
  reference_number: number;
  is_image: boolean;
  image_path: string;
  foot_note: string[];
  next: string;
  previous: string;
  reference: {
    book: string;
    chapter: string;
    primitive: string;
    section: string;
  };
};

export type Book = {
  _id: string;
  title: string;
  translation: string;
  language: string;
  thumbnail: string;
  author: string;
  books: Record<string, string>;
  chapters: Record<string, number>;
}


export default {
  data: new SlashCommandBuilder()
    .setName('read')
    .setDescription('Read form a selection of books.')
    .addStringOption(
      option => option
        .setName('document')
        .setDescription('The document to read')
        .addChoices(...get_choices('./books/'))
        .setRequired(true)
    )
    .addStringOption(
      option => option
        .setName('book')
        .setDescription('This is the book you would like to read')
        .setRequired(false)
    ),
  async execute(interaction: ChatInputCommandInteraction) {
    const db = interaction.client.db;

    const books = db.collection<Book>('books');
    const primitives = db.collection<BookPrimitive>('book_primitives');

    const document_id = interaction.options.getString("document")
    const book_id = interaction.options.getString("book")

    if (!book_id) return;
    if (!document_id) return;
    const this_book = await books.findOne({_id: document_id})
    if (!this_book) return;

    //const container = await render_page("nrsv_ci", "gen001001", 1500, primitives, books);
    const container = render(this_book, book_id, 1);

    interaction.reply({components: container, flags: [MessageFlags.Ephemeral, MessageFlags.IsComponentsV2]})
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

    buffer.push({name: file_json.metadata.title, value: file_json.metadata._id})
  }

  return buffer
}
