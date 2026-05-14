import { MessageFlags, type Message, type MessageReaction, type User } from "discord.js";
import { interpolators } from "sharp";
import type { BookPrimitive } from "../commands/read";
import { render_primitives } from "./render_page";

const books = {
  "gen": "Genesis",
  "exo": "Exodus",
  "lev": "Leviticus",
  "num": "Numbers",
  "deu": "Deuteronomy",
  "jos": "Joshua",
  "jdg": "Judges",
  "rut": "Ruth",
  "1sa": "1 Samuel",
  "2sa": "2 Samuel",
  "1ki": "1 Kings",
  "2ki": "2 Kings",
  "1ch": "1 Chronicles",
  "2ch": "2 Chronicles",
  "ezr": "Ezra",
  "neh": "Nehemiah",
  "est": "Esther",
  "job": "Job",
  "psa": "Psalms",
  "pro": "Proverbs",
  "ecc": "Ecclesiastes",
  "sng": "Song of Solomon",
  "isa": "Isaiah",
  "jer": "Jeremiah",
  "lam": "Lamentations",
  "ezk": "Ezekiel",
  "dan": "Daniel",
  "hos": "Hosea",
  "jol": "Joel",
  "amo": "Amos",
  "oba": "Obadiah",
  "jon": "Jonah",
  "mic": "Micah",
  "nam": "Nahum",
  "hab": "Habakkuk",
  "zep": "Zephaniah",
  "hag": "Haggai",
  "zec": "Zechariah",
  "mal": "Malachi",
  "tob": "Tobit",
  "jdt": "Judith",
  "esg": "Additions to Esther",
  "wis": "Wisdom",
  "sir": "Sirach",
  "bar": "Baruch",
  "lje": "Letter of Jeremiah",
  "s3y": "Prayer of Azariah and the Song of the Three Jews",
  "sus": "Susanna",
  "bel": "Bell and the Dragon",
  "1es": "1 Esdras",
  "2es": "2 Esdras",
  "man": "Prayer of Manasseh",
  "1ma": "1 Maccabees",
  "2ma": "2 Maccabees",
  "mat": "Matthew",
  "mrk": "Mark",
  "luk": "Luke",
  "jhn": "John",
  "act": "Acts",
  "rom": "Romans",
  "1co": "1 Corinthians",
  "2co": "2 Corinthians",
  "gal": "Galatians",
  "eph": "Ephesians",
  "php": "Philippians",
  "col": "Colossians",
  "1th": "1 Thessalonians",
  "2th": "2 Thessalonians",
  "1ti": "1 Timothy",
  "2ti": "2 Timothy",
  "tit": "Titus",
  "phm": "Philemon",
  "heb": "Hebrews",
  "jas": "James",
  "1pe": "1 Peter",
  "2pe": "2 Peter",
  "1jn": "1 John",
  "2jn": "2 John",
  "3jn": "3 John",
  "jud": "Jude",
  "rev": "Revelation"
}

export async function handel_bible_mention(message: Message) {
  const regex = /\b([a-z]+)\s+(\d+)[:;,.-\s]+([\d,.-]+)?\b/gi

  const matchs = message.content.matchAll(regex).toArray();

  if (matchs.length == 0) return;

  let is_present = false;

  for (const match of matchs) {
    const key = findKeyFromBooks(match[1] ?? "");
    if (!key) continue;

    const verses = (match[3] ?? "")
      .split(",")
      .flatMap((part): string[] => {
        part = part.trim();

        if (part.includes("-")) {
          const [startStr, endStr] = part.split("-");

          const start = Number(startStr);
          const end = Number(endStr);

          if (Number.isNaN(start) || Number.isNaN(end)) return [];

          return Array.from(
            { length: end - start + 1 },
            (_, i) => String(start + i)
          );
        }

        const num = Number(part);
        return Number.isNaN(num) ? [] : [String(num)];
      });

    console.log(matchs, verses);

    for (const verse of verses) {
      const primitive = await message.client.db.collection<BookPrimitive>("book_primitives").findOne({book_id: "nrsv_ci", _id: key + match[2]!.padStart(3, "0") + verse.padStart(3, "0")});
      if (!!primitive) {
        is_present = true
      }
    }
  }

  if (is_present) {
    message.react('<:view_verse:1503861450454732902>')
  }
}

export async function handel_reaction_bible(message_reaction: MessageReaction, user: User) {
  const client = message_reaction.client;

  if (message_reaction.emoji.toString() == "<:view_verse:1503861450454732902>" && user.id == message_reaction.message.author?.id) {
    const regex = /\b([a-z]+)\s+(\d+)[:;,.-\s]+([\d,.-]+)?\b/gi

    const matchs = (message_reaction.message.content ?? "").matchAll(regex).toArray();

    if (matchs.length == 0) return;

    let buffer = []

    for (const match of matchs) {
      const key = findKeyFromBooks(match[1] ?? "");
      if (!key) continue;

      const verses = (match[3] ?? "")
        .split(",")
        .flatMap((part): string[] => {
          part = part.trim();

          if (part.includes("-")) {
            const [startStr, endStr] = part.split("-");

            const start = Number(startStr);
            const end = Number(endStr);

            if (Number.isNaN(start) || Number.isNaN(end)) return [];

            return Array.from(
              { length: end - start + 1 },
              (_, i) => String(start + i)
            );
          }

          const num = Number(part);
          return Number.isNaN(num) ? [] : [String(num)];
        });


      for (const verse of verses) {
        const regex = new RegExp("^" + key + match[2]!.padStart(3, "0"));
        const primitive = await client.db.collection<BookPrimitive>("book_primitives").findOne({book_id: "nrsv_ci", _id: { $regex: regex }, reference_number: Number(verse) });
        
        if (!!primitive) {
          buffer.push(primitive)
        }
      }
    }

    const container = await render_primitives(buffer);
    message_reaction.message.reply({ components: container, flags: MessageFlags.IsComponentsV2 })
  }
}

function findKeyFromBooks(// CHAT-GPT wrote
  input: string
): string | null {
  const normalized = input.trim().toLowerCase();

  for (const [key, value] of Object.entries(books)) {
    if (
      key.toLowerCase() === normalized ||
      value.toLowerCase() === normalized
    ) {
      return key;
    }
  }

  return null;
}
