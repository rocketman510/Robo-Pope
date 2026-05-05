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
      console.log(match[1], match[2] + "001");
      
      await interaction.channel.send({components: [section], flags: MessageFlags.IsComponentsV2})
    }
  },
} as Button;

export function encode(input: number[]): string {//CHAT-GPT
  if (input.length === 0) return "";

  // 1. Sort ascending
  const arr = [...input].sort((a, b) => a - b);

  const bits: number[] = [];

  // helper: push bits into stream
  function push(value: number, bitCount: number) {
    for (let i = bitCount - 1; i >= 0; i--) {
      bits.push((value >> i) & 1);
    }
  }

  // 2. Encode first value as 10-bit base
  const base = arr[0];
  if (base < 0 || base > 1023) {
    throw new Error("Base value out of 10-bit range");
  }
  push(base, 10);

  // 3. Encode deltas (4-bit each)
  for (let i = 1; i < arr.length; i++) {
    const delta = arr[i] - arr[i - 1];

    if (delta < 0) {
      throw new Error("Array must be sorted ascending");
    }
    if (delta > 15) {
      throw new Error(`Delta too large for 4 bits: ${delta}`);
    }

    push(delta, 4);
  }

  // 4. Pack bits into bytes
  const byteLength = Math.ceil(bits.length / 8);
  const bytes = new Uint8Array(byteLength);

  for (let i = 0; i < bits.length; i++) {
    const byteIndex = Math.floor(i / 8);
    const bitIndex = 7 - (i % 8);
    bytes[byteIndex] |= bits[i] << bitIndex;
  }

  // 5. Convert to Base64
  return Buffer.from(bytes).toString("base64");
}

export function decode(str: string): string[] {//CHAT-GPT
  const bytes = Buffer.from(str, "base64");
  const bits: number[] = [];

  for (const byte of bytes) {
    for (let i = 7; i >= 0; i--) {
      bits.push((byte >> i) & 1);
    }
  }

  let idx = 0;

  function read(n: number): number {
    let val = 0;
    for (let i = 0; i < n; i++) {
      val = (val << 1) | bits[idx++];
    }
    return val;
  }

  const result: string[] = [];

  let current = read(10);
  result.push(current.toString().padStart(3, "0"));

  while (idx + 4 <= bits.length) {
    const delta = read(4);
    current += delta;
    result.push(current.toString().padStart(3, "0"));
  }

  return result;
}

export function format_list(input: string[]) {//CHAT-GPT
  if (input.length === 0) return "";

  // Convert to numbers and sort
  const nums = input.map(n => parseInt(n, 10)).sort((a, b) => a - b);

  const result: string[] = [];

  let start = nums[0];
  let prev = nums[0];

  for (let i = 1; i < nums.length; i++) {
    const curr = nums[i];

    if (curr === prev + 1) {
      // շարունակation of range
      prev = curr;
    } else {
      // end of range
      if (start === prev) {
        result.push(`${start}`);
      } else {
        result.push(`${start}-${prev}`);
      }

      start = curr;
      prev = curr;
    }
  }

  // handle last range
  if (start === prev) {
    result.push(`${start}`);
  } else {
    result.push(`${start}-${prev}`);
  }

  return result.join(", ");
}
