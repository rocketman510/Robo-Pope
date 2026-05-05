import { ButtonBuilder, ButtonInteraction, ButtonStyle, MessageFlags, SectionBuilder } from "discord.js";
import type { Button } from "../deploy";
import type { Book, BookPrimitive } from "../commands/read";
import { render_page } from "../functions/render_page";

export default {
  data: "rs",
  async execute(interaction: ButtonInteraction) {
    if (!interaction.channel?.isSendable()) return;
    const match = interaction.customId.match(/^\w.(?:-+)([\w]*)(?:-+)([\w]*)(?:-+)?([A-Za-z0-9+/=]*)?/)
    
    if (!match || !match[1] || !match[2]) return;

    const books = interaction.client.db.collection<Book>("books");
    const book_primitives = interaction.client.db.collection<BookPrimitive>("book_primitives");
    
    const document = await interaction.client.db.collection<Book>("books").findOne({_id: match[1]});
    const book = match[2].slice(0,3).toUpperCase();
    const chapter = Number(match[2].slice(3,6));

    if (!match[3]) {
      const section = new SectionBuilder()
        .addTextDisplayComponents(t => t.setContent(`<@${interaction.user.id}> shared a chapter of ${document?.title} - ${book} ${chapter}`))
        .setButtonAccessory(new ButtonBuilder().setCustomId("rn-" + match[1] + "-" + match[2] + "001").setStyle(ButtonStyle.Primary).setLabel("Read"));

      await interaction.update({components: await render_page(match[1], match[2] + "001", 3000, book_primitives, books)})
      await interaction.channel.send({components: [section], flags: MessageFlags.IsComponentsV2})
    } else {
      const section = new SectionBuilder()
        .addTextDisplayComponents(t => t.setContent(`<@${interaction.user.id}> shared a part of ${document?.title} - ${book} ${chapter}:${format_list(decode(match[3]!))}`))
        .setButtonAccessory(new ButtonBuilder().setCustomId("rv-" + match[1] + "-" + match[2] + "-" + match[3]).setStyle(ButtonStyle.Primary).setLabel("Read"));

      await interaction.update({components: await render_page(match[1], match[2] + "001", 3000, book_primitives, books)})
      await interaction.channel.send({components: [section], flags: MessageFlags.IsComponentsV2})
    }
  },
} as Button;


/*
 * CHAT GPT KEEK AWAY
 * */

const MAX_VALUE = 999;
const VALUE_BITS = 10;   // 0..1023, enough for 000..999
const COUNT_BITS = 16;   // up to 65535 items
const DELTA_BITS = 4;    // 0..15
const DELTA_MAX = 15;
const ESCAPE_FLAG = 1;
const DIRECT_FLAG = 0;

class BitWriter {
  private bits: number[] = [];

  write(value: number, bitCount: number) {
    if (bitCount <= 0) return;
    if (!Number.isInteger(value) || value < 0) {
      throw new Error(`Invalid bit value: ${value}`);
    }

    for (let i = bitCount - 1; i >= 0; i--) {
      this.bits.push((value >> i) & 1);
    }
  }

  toBase64(): string {
    const byteLength = Math.ceil(this.bits.length / 8);
    const bytes = new Uint8Array(byteLength);

    for (let i = 0; i < this.bits.length; i++) {
      const byteIndex = Math.floor(i / 8);
      const bitIndex = 7 - (i % 8);
      bytes[byteIndex] |= this.bits[i] << bitIndex;
    }

    return Buffer.from(bytes).toString("base64");
  }
}

class BitReader {
  private idx = 0;

  constructor(private readonly bits: number[]) {}

  read(bitCount: number): number {
    if (this.idx + bitCount > this.bits.length) {
      throw new Error("Unexpected end of data while decoding");
    }

    let value = 0;
    for (let i = 0; i < bitCount; i++) {
      value = (value << 1) | this.bits[this.idx++];
    }
    return value;
  }

  remaining(): number {
    return this.bits.length - this.idx;
  }
}

function base64ToBits(str: string): number[] {
  const bytes = Buffer.from(str, "base64");
  const bits: number[] = [];

  for (const byte of bytes) {
    for (let i = 7; i >= 0; i--) {
      bits.push((byte >> i) & 1);
    }
  }

  return bits;
}

function validateValue(n: number) {
  if (!Number.isInteger(n)) {
    throw new Error(`All values must be integers, got: ${n}`);
  }
  if (n < 0 || n > MAX_VALUE) {
    throw new Error(`Value out of range 0..${MAX_VALUE}: ${n}`);
  }
}

export function encode(input: number[]): string {
  if (input.length === 0) return "";

  const arr = [...input].sort((a, b) => a - b);
  arr.forEach(validateValue);

  const writer = new BitWriter();

  // Store item count so decode knows exactly when to stop.
  if (arr.length > 0xFFFF) {
    throw new Error("Too many items to encode");
  }
  writer.write(arr.length, COUNT_BITS);

  // First value as absolute 10-bit base.
  writer.write(arr[0], VALUE_BITS);

  // Subsequent values: either small delta or escape to absolute value.
  for (let i = 1; i < arr.length; i++) {
    const current = arr[i];
    const prev = arr[i - 1];

    if (current < prev) {
      throw new Error("Array must be sorted ascending");
    }

    const delta = current - prev;

    if (delta <= DELTA_MAX) {
      // Direct delta: 0..15
      writer.write(DIRECT_FLAG, 1);
      writer.write(delta, DELTA_BITS);
    } else {
      // Escape: write absolute value instead of delta.
      writer.write(ESCAPE_FLAG, 1);
      writer.write(current, VALUE_BITS);
    }
  }

  return writer.toBase64();
}

export function decode(str: string): string[] {
  if (str === "") return "";

  const bits = base64ToBits(str);
  const reader = new BitReader(bits);

  const count = reader.read(COUNT_BITS);
  if (count === 0) return [];

  const result: string[] = [];

  let current = reader.read(VALUE_BITS);
  result.push(current.toString().padStart(3, "0"));

  for (let i = 1; i < count; i++) {
    const flag = reader.read(1);

    if (flag === DIRECT_FLAG) {
      const delta = reader.read(DELTA_BITS);
      current += delta;
    } else {
      current = reader.read(VALUE_BITS);
    }

    result.push(current.toString().padStart(3, "0"));
  }

  return result;
}

export function format_list(input: string[]): string {
  if (input.length === 0) return "";

  const nums = input
    .map((s) => {
      const n = Number(s);
      if (!Number.isInteger(n)) {
        throw new Error(`Invalid number string: ${s}`);
      }
      return n;
    })
    .sort((a, b) => a - b);

  const result: string[] = [];

  let start = nums[0];
  let prev = nums[0];

  for (let i = 1; i < nums.length; i++) {
    const curr = nums[i];

    if (curr === prev + 1) {
      prev = curr;
    } else {
      result.push(start === prev ? `${start}` : `${start}-${prev}`);
      start = curr;
      prev = curr;
    }
  }

  result.push(start === prev ? `${start}` : `${start}-${prev}`);

  return result.join(", ");
}
