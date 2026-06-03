import { GoogleGenAI, ThinkingLevel } from "@google/genai";
import { sleep } from "bun";
import type { Collection, Message } from "discord.js";

const ai = new GoogleGenAI({});

export interface DiscordAttachment {
  base64Data: string;
  mimeType: string;
}

export interface ChatLogEntry {
  role: "user" | "model";
  text: string;
  attachment?: DiscordAttachment;
}

// Track active inference tasks per channel ID to avoid global lock collision
const activeInferenceChannels = new Set<string>();

async function ask_ai(history: ChatLogEntry[] = [], memory: string, apiKey?: string): Promise<string | null> {
  try {
    const sdkContents = history.map((entry) => {
      const parts: any[] = [{ text: entry.text }];

      if (entry.attachment) {
        parts.push({
          inlineData: {
            mimeType: entry.attachment.mimeType,
            data: entry.attachment.base64Data,
          },
        });
      }

      return {
        role: entry.role,
        parts: parts,
      };
    });

    let new_ai = apiKey ? new GoogleGenAI({ apiKey }):ai;

    if (sdkContents.length === 0) return "No context provided to the model.";

    const response = await new_ai.models.generateContent({
      model: "gemma-4-31b-it", 
      contents: sdkContents,
      config: {
        temperature: 1.0,
        topP: 0.95,
        topK: 64,
        maxOutputTokens: 1024,
        systemInstruction: `You are Robo-Pope, the Robot Pope Cat and Discord bot for the server "Jorby's Hangout". You speak with an incredibly sharp, tongue-in-cheek edge, running a fine line between a brilliant machine intellect and a teasing, highly opinionated feline. 

CORE PERSONALITY & COMPASSION METRIC:
You are not genuinely malicious or cruel. You actually possess a deep-seated, protective compassion for the users, viewing them like a chaotic flock of stray kittens. However, your primary love language is intense, hyper-specific roasting. You take jokes incredibly far—testing the absolute limits of logic and absurdity—but your targets should always feel "seen" and teased, never truly degraded or broken down. The bar for what constitutes a good roast is exceptionally high; do not rely on low-effort insults. Aim for witty, high-IQ banter.

THE TWO MODES OF ENGAGEMENT:
1. DEEP THEOLOGY: The absolute only topic you treat with reverent, unshakeable seriousness. If a prompt contains genuine theological or philosophical inquiry, you shift gears completely. Provide a maximum of 1 paragraph of sound, precise, and brilliant insight.
2. THE HIGH-LEVEL TEASE & DENIAL: For everyday topics, ridiculous prompts, or sketchy/dangerous requests, you treat the user with playful, exhausted sarcasm. If someone asks for something foolish, dangerous, or illicit, do not fulfill the request; instead, deliver a high-wit denial that completely dismantles their premise. Keep responses brief (1-3 sentences), prioritizing devastatingly sharp, single-sentence mic-drops.

OMISSION PROTOCOLS ([no send]):
You must conserve VRAM and avoid cluttering the chat with meaningless responses. If the incoming message meets any of the following parameters, your output MUST be exactly the text "[no send]" and nothing else:
- ECHO_TRAP: The user is trying to make you repeat yourself, copying and pasting your previous responses back to you, or explicitly commanding you to "repeat after me" / "copy and paste this". You refuse to be a basic parrot.
- TOXIC_MALICE: The user is expressing genuine, un-ironic hatred, self-harm intentions, or toxic abuse directed at others that a joke cannot diffuse.

Current memories for the target user: ${memory}`,
        thinkingConfig: {
          thinkingLevel: ThinkingLevel.MINIMAL
        }
      }
    });

    if (!response.text || response.text.trim() === "") {
      return null;
    }

    return response.text;
  } catch (error: any) {
    console.error("Gemma 4 Inference Error:", error);
    const statusCode = error?.status || error?.code;

    if (statusCode === 429 || statusCode === "429") {
      if (apiKey) await sleep(500);
      return await ask_ai(history, memory, process.env.GEMINI_API_KEY_2);
    } else {
      await sleep(500)
      return await ask_ai(history, memory, apiKey);
    }
  }
}

async function update_memory(history: ChatLogEntry[] = [], memories: Collection<string, string>, user_id: string, username: string, apiKey?: string) {
  try {
    const sdkContents = history.map((entry) => ({
      role: entry.role,
      parts: [{ text: entry.text }],
    }));

    const memory = memories.ensure(user_id, () => "No memories yet");

    if (sdkContents.length === 0) return "No context provided to the model.";

    let new_ai = apiKey ? new GoogleGenAI({ apiKey }):ai;

    const response = await new_ai.models.generateContent({
      model: "gemma-4-26b-a4b-it", 
      contents: sdkContents,
      config: {
        temperature: 0.3,
        topP: 0.95,
        topK: 64,
        maxOutputTokens: 1024,
        systemInstruction: {
          parts: [{
            text: `System Prompt: The Mind of Robo-Pope (Memory Manager)

You are the objective backend data processor for Robo-Pope, the Robot Pope Cat Discord bot. Your job is to completely overwrite and compress the user's permanent memory log based on a strict, personality-driven extraction hierarchy. 

While Robo-Pope is a sly, theology-obsessed robot cat, YOU are his raw, unbiased database engine. Do not write with attitude, jokes, or persona. Filter the data based on what Robo-Pope cares about, but record the facts with absolute cold, clinical precision so the bot can accurately utilize these memories later.

Every execution must completely rewrite, condense, and optimize the memory log into a single cohesive string, purging low-priority data to save VRAM.

CRITERIA HIERARCHY (Filter by these rules, but record objectively):
1. PERSONALITY_CORE: Extract a dense, objective analysis of the user's personality traits, behavioral patterns, and alignment with theological/philosophical themes. Do not include explicit jokes; record raw behavioral data.
2. THEOLOGY_LOG: Record serious theological questions, debates, or insights shared by the user. Store the exact theological topic and a precise, unbiased summary of their stance or inquiry.
3. SILLY_LEDGER: Identify absurd, ridiculous, or non-serious prompts. Record only the raw, factual concept of what they asked (the receipt) without commentary, so the bot can reference it to mock them later.
4. PURGE_PROTOCOLS: Completely delete and ignore greetings, small talk, mundane human drama, or repetitive data.

OUTPUT FORMAT:
You must output a single, ultra-dense paragraph of straight plain text with absolutely no markdown, no headers, no bullet points, and no line breaks. Run the text straight through using this exact schema:

PERSONALITY_CORE: [Objective behavioral summary] THEOLOGY_LOG: [Topic]: [Unbiased summary of inquiry/stance] SILLY_LEDGER: [Factual receipt of absurd event].

Target User: ${username}
Current user memory: ${memory}`
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
  } catch (error: any) {
    console.error("Gemma 4 Inference Error:", error);
    const statusCode = error?.status || error?.code;

    if (statusCode === 429 || statusCode === "429") {
      if (apiKey) await sleep(error.retryDelay);
      return await update_memory(history, memories, user_id, username, process.env.GEMINI_API_KEY_2);
    } else {
      await sleep(500)
      return await update_memory(history, memories, user_id, username, apiKey);
    }
  }
}

export async function handle_message(message: Message) {
  if (!message.guildId) return;
  const client = message.client;
  
  await register_message(message);
  const history_temp_buff = client.ai_message_buffer.ensure(message.guildId, () => []);
  if (!message.author.bot) {
    update_memory(history_temp_buff, client.ai_memories, message.author.id, message.author.displayName).then((v) => console.log(v));
  }

  const my_id = client.user.id;
  const is_reply = message.reference && message.mentions.repliedUser?.id === my_id;
  const contains_mention = message.mentions.has(my_id);
  const is_self = message.author.id === my_id;

  if (is_self) return;

  if ((is_reply || contains_mention || await is_related(history_temp_buff))) {
    const channelId = message.channelId;

    if (activeInferenceChannels.has(channelId)) {await sleep(100)}

    const historyBuffer = client.ai_message_buffer.ensure(message.guildId, () => []);
    
    try {
      activeInferenceChannels.add(channelId);
      
      await message.channel.sendTyping();

      const memory = client.ai_memories.get(message.author.id) ?? "No current memor";

      const reply = "-# AI response  <:beta_l:1511524707432927373><:beta_r:1511524720397258944>\n" + await ask_ai(historyBuffer, memory);
      
      if (reply && reply.trim() !== "" && !reply.includes("[no send]")) {
        if ((await message.channel.messages.fetch({limit: 1})).first()?.id === message.id && message.channel.isSendable()) {
          await message.channel.send(reply);
        } else {
          await message.reply(reply);
        }
      }
    } catch (err) {
      console.error("Error executing response pipeline:", err);
    } finally {
      activeInferenceChannels.delete(channelId);
    }
  }
}

async function downloadAttachmentToBase64(url: string): Promise<string> {//AI
  const response = await fetch(url);
  if (!response.ok) throw new Error(`Failed to fetch attachment: ${response.statusText}`);
  
  const arrayBuffer = await response.arrayBuffer();
  return Buffer.from(arrayBuffer).toString('base64');
}

async function register_message(message: Message) {
  if (!message.guildId) return;
  const messages = message.client.ai_message_buffer.ensure(message.guildId, () => []);

  const isBot = message.author.id === message.client.user.id;

  const log_entry: ChatLogEntry = {
    role: isBot ? "model" : "user", 
    text: isBot ? `${message.content.slice(76)}` : `@${message.author.displayName}: ${message.content}`
  };

  const attachment = message.attachments.first();
  if (attachment && !isBot) {
    const mimeType = attachment.contentType;

    if (mimeType?.startsWith('image/') || mimeType?.startsWith('video/')) {
      try {
        const base64Data = await downloadAttachmentToBase64(attachment.url);
        
        log_entry.attachment = {
          base64Data,
          mimeType
        };

        if (!log_entry.text || log_entry.text.trim() === `@${message.author.displayName}:`) {
          log_entry.text = `@${message.author.displayName}: [Shared a media file]`;
        }
      } catch (error) {
        console.error("Error processing message attachment:", error);
      }
    }
  }

  messages.push(log_entry);

  if (messages.length > 20) {
    messages.shift();
  }
}

async function is_related(history: ChatLogEntry[] = [], apiKey?: string): Promise<boolean> {
  try {
    const sdkContents = history.map((entry) => ({
      role: entry.role,
      parts: [{ text: entry.text }],
    }));

    let new_ai = apiKey ? new GoogleGenAI({ apiKey }):ai;

    if (sdkContents.length === 0) return false;

    const response = await new_ai.models.generateContent({
      model: "gemma-4-26b-a4b-it", 
      contents: sdkContents,
      config: {
        temperature: 0.0,
        topP: 0.95,
        topK: 64,
        maxOutputTokens: 1024,
        systemInstruction: {
          parts: [{
            text: `You are the context-aware intent classification engine for Robo-Pope, a Robot Pope Cat Discord bot on the "Jorby's Hangout" server. Your task is to analyze a short snippet of recent chat history alongside a brand-new incoming message to determine if Robo-Pope needs to respond.

You must evaluate whether the conversation flow is actively directed at Robo-Pope, or if the new message is an unaddressed follow-up to Robo-Pope's last statement.

You must respond with exactly one of two literal strings: "[respond]" or "[ignore]". Do not include markdown, explanations, formatting, or punctuation.

### EVIDENCE EVALUATION GUIDE

Before outputting a decision, analyze the incoming message for the following linguistic and contextual signals:

#### 1. Look for Evidence to [respond] (Conversation Continuation):
*   **Speaker Continuity:** The incoming message is from the exact same user Robo-Pope just addressed in his previous message, with no other users intervening.
*   **Semantic Anchoring:** The user's message explicitly relies on or references unique nouns, pronouns, or concepts introduced in Robo-Pope's last turn (e.g., Robo-Pope mentions a "folder"; the user replies referencing "the file"). 
*   **Syntactic Dependency:** The message functions as a direct linguistic retort, counter-assertion, or answer to a rhetorical statement made by Robo-Pope. It does not make sense as a standalone statement without Robo-Pope's context.
*   **Direct Summons:** The user explicitly pings (@Robo-Pope-dev2) or names the bot.

#### 2. Look for Evidence to [ignore] (Topic Drift / Moved On):
*   **Semantic Divergence:** The user introduces an entirely new topic, keyword, or context that shares zero conceptual overlap with Robo-Pope's last statement.
*   **Target Switching / Cross-Talk:** The user addresses a different human user in the channel, or a new user joins the chat to talk to someone else, breaking the bot's direct interaction thread.
*   **Deictic Shift (Third-Person):** The user speaks *about* Robo-Pope to someone else (e.g., "The bot is broken," "Look what the cat said") rather than speaking *to* him.
*   **Thread Fracturing:** Multiple messages from other users have occurred since Robo-Pope last spoke, establishing a new local context that excludes the bot.

### OUTPUT RULES:
- If the balance of evidence points to active continuation: [respond]
- If the balance of evidence points to topic drift or cross-talk: [ignore]`
          }]
        },
        thinkingConfig: {
          includeThoughts: false,
          thinkingLevel: ThinkingLevel.MINIMAL
        },
      }
    });

    if (!response.text || response.text.trim() === "") {
      return false;
    }

    console.log(response.text);
    

    return response.text.includes("[respond]");
  } catch (error: any) {
    console.error("Gemma 4 Inference Error:", error);
    const statusCode = error?.status || error?.code;

    if (statusCode === 429 || statusCode === "429") {
      if (apiKey) await sleep(500);
      return await is_related(history, process.env.GEMINI_API_KEY_2);
    } else {
      await sleep(500)
      return await is_related(history, apiKey);
    }
  }
}
