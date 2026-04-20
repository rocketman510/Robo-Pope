import { SlashCommandBuilder, ChatInputCommandInteraction, PermissionFlagsBits, ContainerBuilder, MessageFlags, ButtonBuilder, ButtonStyle, Message, ChannelType, type TextBasedChannel, ComponentBuilder } from "discord.js";
import type { Command } from "../deploy";
import { channel, type Channel } from "node:diagnostics_channel";
import { log } from "node:console";

export default {
  data: new SlashCommandBuilder()
    .setName('delete')
    .setDescription('A powerful tool to mass delete messages.')
    .addSubcommand((subcommand) => 
      subcommand
        .setName('user')
        .setDescription('Delete messages form a user.')
        .addUserOption((option) => option.setName('target').setDescription('The user to target').setRequired(true))
        .addIntegerOption((option) => option.setName('number_of_messages').setDescription('The number of messages to delete').setRequired(true).setMaxValue(100).setMinValue(1))
        .addStringOption((option) => option.setName('channels').setDescription('For all channels or only for the current channel').setRequired(true).addChoices({ name: 'All channels', value: 'all_channels' }, { name: 'Current channel', value: 'current_channel' }))
        .addStringOption((option) => option.setName('after').setDescription('Filter for messages that were sent after a time. Format as: MM/DD/YY HH:MM AM or PM TIMEZONE').setRequired(false))
        .addStringOption((option) => option.setName('before').setDescription('Filter for messages that were sent before a time. Format as: MM/DD/YY HH:MM AM or PM TIMEZONE').setRequired(false))
        .addBooleanOption((option) => option.setName('attachments').setDescription('Filter for messages that have Attachments').setRequired(false))
        .addStringOption((option) => option.setName('regex').setDescription('Filter messages by regex').setRequired(false))
    )
    .addSubcommand((subcommand) => 
      subcommand
        .setName('channel')
        .setDescription('Delete messages form this channel.')
        .addIntegerOption((option) => option.setName('number_of_messages').setDescription('The number of messages to delete').setRequired(true).setMaxValue(100).setMinValue(1))
        .addStringOption((option) => option.setName('after').setDescription('Filter for messages that were sent after a time. Format as: MM/DD/YY HH:MM AM or PM TIMEZONE').setRequired(false))
        .addStringOption((option) => option.setName('before').setDescription('Filter for messages that were sent before a time. Format as: MM/DD/YY HH:MM AM or PM TIMEZONE').setRequired(false))
        .addRoleOption((option) => option.setName('role').setDescription('Filter by role').setRequired(false))
        .addBooleanOption((option) => option.setName('attachments').setDescription('Filter for messages that have Attachments').setRequired(false))
        .addStringOption((option) => option.setName('regex').setDescription('Filter messages by regex').setRequired(false))
    )
    .addSubcommand((subcommand) => 
      subcommand
        .setName('server')
        .setDescription('Delete messages form all channels.')
        .addIntegerOption((option) => option.setName('number_of_messages').setDescription('The number of messages to delete THIS IS PER-CHANNEL and not the total.').setRequired(true).setMaxValue(100).setMinValue(1))
        .addStringOption((option) => option.setName('after').setDescription('Filter for messages that were sent after a time. Format as: MM/DD/YY HH:MM AM or PM TIMEZONE').setRequired(false))
        .addStringOption((option) => option.setName('before').setDescription('Filter for messages that were sent before a time. Format as: MM/DD/YY HH:MM AM or PM TIMEZONE').setRequired(false))
        .addRoleOption((option) => option.setName('role').setDescription('Filter by role').setRequired(false))
        .addBooleanOption((option) => option.setName('attachments').setDescription('Filter for messages that have Attachments').setRequired(false))
        .addStringOption((option) => option.setName('regex').setDescription('Filter messages by regex').setRequired(false))
    )
    .addSubcommand((subcommand) => 
      subcommand
        .setName('help')
        .setDescription('Explain how to use the command.')
    )
    .setDefaultMemberPermissions(PermissionFlagsBits.ManageMessages),
  async execute(interaction: ChatInputCommandInteraction) {
    if (interaction.options.getSubcommand() == 'help') await help(interaction);
    if (interaction.options.getSubcommand() == 'user') await user(interaction);
},
} as Command;

async function help(interaction:ChatInputCommandInteraction) {
  const component = new ContainerBuilder()
    .setAccentColor(0xE64D3E)
    .addTextDisplayComponents((td) => td.setContent(`# HOW TO USE DELETE COMMAND
The delete command is separated into 3 subcommands:
- User
- Channel
- Server
All of these have the same 5 filters:
- number\_of\_messages
- after
- before
- attachments
- regex
Both *after* and *before* take the same input of either: a point in time, or a message ID.
## Time Formatting
You can format the time by:
- Unix Timestamp:
  - \`UT: 1000213380\`
  - \`1000213380ut\`
- Time Formatting
  - Formatted as: **MM/DD/YYYY HH:MM AM or PM TIMEZONE**
  - \`1/6/26 12:45 pm pst\`
  - \`01 03 2026 12:45am utc\`
  - \`12,29,2020 18:45 gmt\`
For Time Formatting if no timezone is provided the default is PST (Pacific Standard Time) and if no time is provided then the default is 0:00 or 12:00 am both being the same time. formatting is lose for the date however the time must be: HH:MM and all of the units must be in the order: **Date, Time, Timezone**
## Message ID's
Message ID's are a precise tool for identifying a message to get a message ID you need to go into setting of your discord account enable developer mode. This setting can be found by opening settings and going to: **Developer -> Developer Mode.** Now when you right click a message you can copy the message's ID. The bot can accept this in the *before* and *after* fields but you must format it as: \`id: (MESSAGE ID HERE)\` eg: \`id: 1494863721342828704\`.
## Fields
The fields of the commands

### number\_of\_messages Field
The number of messages is how many messages the bot will delete the max is 100 and the min is 1.
### after Field
The after field will delete all qualifying messages that happen after the time or message ID that is provided in the field. This field takes the same patern defined above **Time Formatting** and **Message ID's**.
### before Field
The before field will delete all qualifying messages that happen before the time or message ID that is provided in the field. This field takes the same patern defined above **Time Formatting** and **Message ID's**.`))
      .addTextDisplayComponents((td) => td.setContent(`### attachments Field
This filters only messages that have a attachment.
### regex Field
This filters for messages that pass the regex pastern that is provided in the Field. This is a powerful tool that enables a mod to remove messages by there content.`))
      .addSectionComponents((s) => s
        .addTextDisplayComponents((td) => td.setContent('For more Information and testing go to:'))
        .setButtonAccessory(new ButtonBuilder().setURL('https://regexr.com/').setLabel('regexr').setStyle(ButtonStyle.Link))
      )
      .addTextDisplayComponents((td) => td.setContent(`When inputting regex you must use the full formatting: \`/(REGEX)/(FLAGS)\` eg: \`/clare/gi\`.
### Regex Basics
Regular expressions (regex) are patterns used to match text. They are useful for filtering messages based on their content.

**Core Syntax:**
- \`.\` → Matches any single character  
- \`*\` → Matches 0 or more of the previous pattern  
- \`+\` → Matches 1 or more of the previous pattern  
- \`?\` → Makes the previous pattern optional  

**Character Matching:**
- \`[abc]\` → Matches **a**, **b**, or **c**  
- \`[^abc]\` → Matches anything *except* **a**, **b**, or **c**  
- \`[a-z]\` → Matches any lowercase letter  
- \`[A-Z]\` → Matches any uppercase letter  
- \`[0-9]\` → Matches any digit  

**Special Sequences:**
- \`\\d\` → Any digit (same as \`[0-9]\`)  
- \`\\w\` → Any word character (letters, numbers, underscore)  
- \`\\s\` → Any whitespace (spaces, tabs)  

**Anchors:**
- \`^\` → Start of the message  
- \`$\` → End of the message  

**Flags:**
Flags modify how the regex behaves. Add them at the end of the pattern:
- \`/pattern/i\` → Case-insensitive (matches "Hello" and "hello")  
- \`/pattern/g\` → Global (matches all occurrences, not just first)  
- \`/pattern/m\` → Multiline (anchors work per line instead of whole message)  

:warning: Small mistakes in regex can match far more messages than intended.

## User Subcommand
The user subcommand is to delete all messages from 1 user across the whole server or in the current channel

## Channel Subcommand
The channel subcommand is used to delete messages in the specified channel regardless of user.

## Server Subcommand
The server subcommand is used to delete messages in everything channel of the server.`))
    interaction.reply({ components: [ component ], flags: [ MessageFlags.Ephemeral, MessageFlags.IsComponentsV2 ]})
}

async function user(interaction:ChatInputCommandInteraction) {
  const client = interaction.client;
  const guild = interaction.guild;
  const options = interaction.options
  const user = options.getUser('target');
  const all_channels = interaction.options.getString('channels') == 'all_channels';

  await interaction.deferReply({flags: MessageFlags.Ephemeral})

  if (!user) return;
  if (!guild) return;
  if (!interaction.channel) return;

  if (all_channels) {
    for (const [_, channel] of await guild.channels.fetch()) {
      if (!channel || !channel.isTextBased()) continue;
      let messages: Message[] = await dyn_fetch(channel, interaction.options.getInteger('number_of_messages') || 1, (message) => message.author.id == (interaction.options.getUser('target') || {id:0}).id)

      for (const message of messages) {
        await message.delete()
      }
    }
  } else {
    let messages: Message[] = await dyn_fetch(interaction.channel, interaction.options.getInteger('number_of_messages') || 1, (message) => message.author.id == (interaction.options.getUser('target') || {id:0}).id)

    for (const [index, message] of messages.entries()) {
      await message.delete();
      const progres_percent = Math.round(((index + 1) / messages.length) * 100);
      interaction.editReply({ components: [progress_message('Delete Progres', `${progres_percent}% • ${index+1}/${messages.length}`, index + 1, messages.length)], flags: [MessageFlags.IsComponentsV2] });
    }
  }
}

function progress_message(title:string, description: string | null, value: number, max: number): ContainerBuilder {
  const progres_bar_length = Math.round((value/max)*15);
  let progres_bar = progres_bar_length >= 1 ? '<:progress_bar_start_full:1495868845989040268>':'<:progress_bar_start_empty:1495868840339439836>'
  const middle_full = Math.max(0, progres_bar_length - 2);
  const middle_empty = Math.max(0, 13 - middle_full);
  progres_bar += '<:progress_bar_full:1495868805589504261>'.repeat(middle_full) + '<:progress_bar_empty:1495868817648255107>'.repeat(middle_empty)
  progres_bar += progres_bar_length >= 15 ? '<:progress_bar_end_full:1495868834215755988>':'<:progress_bar_end_empty:1495868823075688670>'

  const component = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addTextDisplayComponents((e) => e.setContent(`## ${title}${description == null ? '':'\n-# ' + description}\n${progres_bar}`))

  return component
}

async function dyn_fetch(channel: TextBasedChannel, max: number, predicate: (message: Message) => boolean): Promise<Message[]> {
  let messages: Message[] = []
  let working_messages = await channel.messages.fetch({limit: 100 });

  while (messages.length != max) {
    for (const [_, message] of working_messages) {
      if (predicate(message)) {
        messages.push(message);
        if (messages.length == max) break;
      }
    }
    if (working_messages.last() == undefined) break;
    working_messages = await channel.messages.fetch({ limit: 100, before: working_messages.last()!.id })
  }

  return messages
}
