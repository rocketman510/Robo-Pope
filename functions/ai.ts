import { GoogleGenAI } from "@google/genai";
import type { Message } from "discord.js";

const ai = new GoogleGenAI({});

export interface ChatLogEntry {
  role: "user" | "model";
  text: string;
}

export async function ask_ai(history: ChatLogEntry[] = []): Promise<string> {
  try {
    // 1. Transform your flat history array into the strict format the GenAI SDK expects
    const sdkContents = history.map((entry) => ({
      role: entry.role,
      parts: [{ text: entry.text }],
    }));

    // If the history is empty, don't ping the API with an empty payload
    if (sdkContents.length === 0) {
      return "No context provided to the model.";
    }

    const response = await ai.models.generateContent({
      model: "gemma-4-31b-it", 
      contents: sdkContents, // Pass the corrected structure here
      config: {
        temperature: 1.0,
        topP: 0.95,
        topK: 64,
        maxOutputTokens: 4096
      }
    });

    console.log("Model Raw Response:", response.text);

    // 2. Fallback check: ensure response is neither null, undefined, nor a blank string
    if (!response.text || response.text.trim() === "") {
      return "No bueno. (The model returned an empty string)";
    }

    return response.text;
  } catch (error) {
    console.error("Gemma 4 Inference Error:", error);
    return "No bueno Muchas Gracias.";
  }
}

export async function handle_message(message: Message) {
  if (!message.guildId) return;
  register_message(message);

  const my_id = message.client.user.id;
  const is_reply = message.reference && message.mentions.repliedUser?.id === my_id;
  const contains_mention = message.mentions.has(my_id);

  if (is_reply || contains_mention) {
    const historyBuffer = message.client.ai_message_buffer.ensure(message.guildId, () => []);
    
    const reply = await ask_ai(historyBuffer);
    console.log("reply: ", reply);
    
    if (reply && reply.trim() !== "") {
      await message.reply(reply);
    } else {
      await message.reply("No bueno. Muchas Gracias. (Fallback triggered due to empty reply payload)");
    }
  }
}

function register_message(message: Message) {
  if (!message.guildId) return;
  const messages = message.client.ai_message_buffer.ensure(message.guildId, () => []);

  const log_entry: ChatLogEntry = {
    role: message.author.id === message.client.user.id ? "model" as const : "user" as const, 
    text: `[${message.author.displayName}]: ${message.content}`
  };

  messages.push(log_entry);

  // 4. Optional: Keep history bounded (e.g., keep the last 30 messages) so memory doesn't bloat
  if (messages.length > 30) {
    messages.shift();
  }
}
