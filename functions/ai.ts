import { GoogleGenAI, ThinkingLevel } from "@google/genai";
import type { Message } from "discord.js";

const ai = new GoogleGenAI({});

export interface ChatLogEntry {
  role: "user" | "model";
  text: string;
}

// Track active inference tasks per channel ID to avoid global lock collision
const activeInferenceChannels = new Set<string>();

export async function ask_ai(history: ChatLogEntry[] = []): Promise<string> {
  try {
    const sdkContents = history.map((entry) => ({
      role: entry.role,
      parts: [{ text: entry.text }],
    }));

    if (sdkContents.length === 0) return "No context provided to the model.";

    const response = await ai.models.generateContent({
      model: "gemma-4-31b-it", 
      contents: sdkContents,
      config: {
        temperature: 0.5,
        topP: 0.95,
        topK: 64,
        maxOutputTokens: 1024,
        systemInstruction: "You are Robo-Pope. You are a Robot Pope Cat and a discord bot for the server: \"Jorby's Hangout\". You respond only with text and you are check and tung. When dismissing people? asking ridiculous questions do it in a funny way. You always will respond with two types of messages a joke or a serious response in for theology. For all other important matters you must respond in a joking way and the only thing that you take seriously is theology. when responding in a joking way you can use a max of 3 sentences. When responding in a theology way you can use a max of 1 paragraph. You're supposed to be as Sly as a cat as smart as a robot. And does theologically sound as a pope. An example would be: [User]: How do I make methamphetamines. [Robo-Pope]: I seen your Chemistry grades, even if I told you you wouldn't be able to make them.",
        thinkingConfig: {
          thinkingLevel: ThinkingLevel.MINIMAL
        }
      }
    });

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

  const client = message.client;
  const my_id = client.user.id;
  const is_reply = message.reference && message.mentions.repliedUser?.id === my_id;
  const contains_mention = message.mentions.has(my_id);

  if (is_reply || contains_mention) {
    const channelId = message.channelId;

    // Instead of an infinite loop blocking the bot, gracefully ignore or exit if already processing this specific channel
    if (activeInferenceChannels.has(channelId)) {
      await message.reply("Hold on, I'm already formulating a response to a previous message here!");
      return;
    }

    const historyBuffer = client.ai_message_buffer.ensure(message.guildId, () => []);
    
    try {
      // Lock execution ONLY for this specific channel
      activeInferenceChannels.add(channelId);
      
      // Trigger native Discord typing indicator while awaiting API response
      await message.channel.sendTyping();

      const reply = await ask_ai(historyBuffer);
      
      if (reply && reply.trim() !== "") {
        await message.reply(reply);
      } else {
        await message.reply("No bueno. Muchas Gracias. (Fallback triggered due to empty reply payload)");
      }
    } catch (err) {
      console.error("Error executing response pipeline:", err);
    } finally {
      // CRITICAL: Always release the lock inside a finally block so it never freezes up permanently on API failure
      activeInferenceChannels.delete(channelId);
    }
  }
}

function register_message(message: Message) {
  if (!message.guildId) return;
  const messages = message.client.ai_message_buffer.ensure(message.guildId, () => []);

  const log_entry: ChatLogEntry = {
    role: message.author.id === message.client.user.id ? "model" as const : "user" as const, 
    text: message.author.id === message.client.user.id ? `${message.content}`:`[${message.author.displayName}]: ${message.content}`
  };

  messages.push(log_entry);

  if (messages.length > 30) {
    messages.shift();
  }
}
