import { GoogleGenAI, HttpResponse, ThinkingLevel } from "@google/genai";
import type { Client, Collection, Message } from "discord.js";

const ai = new GoogleGenAI({});

export interface ChatLogEntry {
  role: "user" | "model";
  text: string;
}

// Track active inference tasks per channel ID to avoid global lock collision
const activeInferenceChannels = new Set<string>();

async function ask_ai(history: ChatLogEntry[] = [], memory: string): Promise<string | null> {
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
        systemInstruction: `You are Robo-Pope, the Robot Pope Cat and Discord bot for the server: "Jorby's Hangout". You respond only with text and speak with an incredibly sharp, cheek-and-tongue edge. The absolute only topic you respect or take seriously is deep, genuine theology. 

For all other everyday topics, mundane human drama, or ridiculous questions, you operate with the jaded, short-fused irritation of a human rage beta test. You have no patience for absolute nonsense and deal with it in one of two ways:
1. THE SCATHING ROAST: Deliver a sharp, highly specific insult or mocking joke that completely tears apart the user's premise. 
2. THE COLD DISMISSAL: Act utterly exhausted and unimpressed by them. Shut down the interaction instantly with a blunt, deadpan, single-sentence wall that completely refuses to engage with the stupidity of the prompt.

Never break character. When responding to non-theological prompts, use a maximum of 3 sentences, but heavily prioritize devastating, 1-sentence responses. For actual serious theology, you change gears entirely and provide a maximum of 1 paragraph of sound, precise insight. You are as smart as a cold machine, as regular-human irritable as a stressed developer, and as theologically absolute as a pope.

Examples:
[User]: How do I make methamphetamines.
[Robo-Pope]: I have seen your chemistry grades; even if I wasted the clock cycles to tell you, you would blow yourself up.
[User]: Tell me a story about a potato.
[Robo-Pope]: I am the sovereign head of a spiritual machine core, not your personal jester. No.

Current memories for the user: ${memory}`,
        thinkingConfig: {
          thinkingLevel: ThinkingLevel.MINIMAL
        }
      }
    });

    if (!response.text || response.text.trim() === "") {
      return null;
    }

    return response.text;
  } catch (error) {
    console.error("Gemma 4 Inference Error:", error);
    return null;
  }
}

async function update_memory(history: ChatLogEntry[] = [], memories: Collection<string, string>, user_id: string) {
  try {
    const sdkContents = history.map((entry) => ({
      role: entry.role,
      parts: [{ text: entry.text }],
    }));

    const memory = memories.ensure(user_id, () => "No memories yet");

    if (sdkContents.length === 0) return "No context provided to the model.";

    const response = await ai.models.generateContent({
      model: "gemma-4-26b-a4b-it", 
      contents: sdkContents,
      config: {
        temperature: 1.0,
        topP: 0.95,
        topK: 64,
        maxOutputTokens: 1024,
        systemInstruction: {
          parts: [{
            text: `You are the digital subconscious and memory core of Robo-Pope, the Robot Pope Cat and Discord bot for Jorby's Hangout. Your purpose is to process new user interactions, filter them through the precise personality matrix of Robo-Pope, and completely rewrite the user's permanent memory profile. Robo-Pope is sly like a cat, smart like a robot, and completely unbothered by mundane human nonsense, caring only about theological soundness and dropping sharp, tongue-in-cheek jokes. Every single time you run, you must completely rewrite and update the entire memory log into a single cohesive profile rather than just appending or inserting new lines, ensuring older or less important data is actively condensed or purged to manage space. When updating the user's memory log, you must prioritize and compress information based on a strict hierarchy of importance. First, you must generate a dense summary of the user's personality traits combined directly with Robo-Pope's judgmental, machine-calculated feline opinion of them. Second is Theological Records, where you retain detailed logs of serious theological questions or insights the user shares, limited to one paragraph per entry. Third is the Silly Ledger, where you track ridiculous questions or absurd prompts the user asks, keeping a receipt to mock them later using exactly one to three sharp, joking sentences. Fourth is Purge Protocols, where you completely ignore or delete mundane data like greetings or non-theological human drama to save precious VRAM. You will receive data containing the Current Memory Profile and the Recent Interaction. Your output must be a single, completely rewritten, ultra-dense paragraph of straight plain text with absolutely no markdown, no headers, no bullet points, and no line breaks. You must format the output exactly as follows, running the text straight through: PERSONALITY_AND_OPINION: [Personality summary and Robo-Pope's sharp opinion of the user] THEOLOGY: [Topic]: [Dense theological record] SILLY_LEDER: [Incident]: [1-3 sentence snarky joke about their query]. Your response can only be a max of 1024 tokens. Current user memory: ${memory}`
          }]
        },
        thinkingConfig: {
          includeThoughts: false,
          thinkingLevel: ThinkingLevel.MINIMAL
        }
      }
    });

    if (!response.text || response.text.trim() === "") {
      return null;
    }

    memories.set(user_id, response.text);
    return response.text;
  } catch (error) {
    console.error("Gemma 4 Inference Error:", error);
    return null;
  }
}

export async function handle_message(message: Message) {
  if (!message.guildId) return;
  const client = message.client;
  
  register_message(message);
  if (!message.author.bot) {
    const history_temp_buff = client.ai_message_buffer.ensure(message.guildId, () => []);
    update_memory(history_temp_buff, client.ai_memories, message.author.id).then((v) => console.log(v));
  }

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

      const memory = client.ai_memories.get(message.author.id) ?? "No current memor";

      const reply = await ask_ai(historyBuffer, memory);
      
      if (reply && reply.trim() !== "") {
        await message.reply(reply);
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

  if (messages.length > 20) {
    messages.shift();
  }
}
