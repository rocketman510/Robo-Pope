import { SlashCommandBuilder, ChatInputCommandInteraction, PermissionFlagsBits, MessageFlags } from "discord.js";
import type { Command } from "../deploy";

async function fail(interaction: ChatInputCommandInteraction, message: string) {
  console.error(message);
  await interaction.reply({content: message, flags: MessageFlags.Ephemeral});
}

export default {
  data: new SlashCommandBuilder()
    .setName('vowofsilence')
    .setDescription('Mute someone')
    .addUserOption(
      option => option
      .setName('user')
      .setDescription('The user to Silence')
      .setRequired(true)
    )
    .addStringOption(
      option => option
      .setName('time')
      .setDescription('Set the time by its unit. ? for help')
      .setRequired(true)
    )
    .addStringOption(
      option => option
      .setName('reason')
      .setDescription('The reason the user was silenced')
      .setRequired(false)
    )
    .setDefaultMemberPermissions(PermissionFlagsBits.ModerateMembers),
  async execute(interaction: ChatInputCommandInteraction) {
    const max_timeout_time = 2_419_200_000;
    const user = interaction.options.getUser('user');
    if (!user) {fail(interaction, "Failed to get the user"); return;}
    const guild = interaction.guild;
    if (!guild) {fail(interaction, "Failed to get guild"); return;}
    const member = guild.members.cache.get(user.id);
    if (!member) {fail(interaction, "Failed to get the member"); return;}
    const time_string = interaction.options.getString('time');
    if (!time_string) return fail(interaction, "Invalid Time string");

    const match_form_back = time_string?.matchAll(/(\d+)\s*(?:[mM][iI][lL][lL][iI][sS][eE][cC][oO][nN][dD][sS]?|[mM][sS]|[mM][iI][lL][lL][sS]?)(?![a-zA-Z])|(\d+)\s*(?:[sS][eE][cC][oO][nN][dD][sS]?|[sS]|[sS][eE][cC][sS]?)(?![a-zA-Z])|(\d+)\s*(?:[mM][iI][nN][uU][tT][eE][sS]?|m|[mM][iI][nN][sS]?)(?![a-zA-Z])|(\d+)\s*(?:[hH][oO][uU][rR][sS]?|[hH])(?![a-zA-Z])|(\d+)\s*(?:[dD][aA][yY][sS]?|[dD])(?![a-zA-Z])|(\d+)\s*(?:[wW][eE][eE][kK][sS]?|[wW])(?![a-zA-Z])|(\d+)\s*(?:[mM][oO][nN][tT][hH][sS]?|M)(?![a-zA-Z])|(\d+)\s*(?:[yY][eE][aA][rR][sS]?|[yY])(?![a-zA-Z])/g);
    const match_form_front = time_string?.matchAll(/(?:[mM][iI][lL][lL][iI][sS][eE][cC][oO][nN][dD][sS]?|[mM][sS]|[mM][iI][lL][lL][sS]?)(?![a-zA-Z]):\s*(\d+)|(?:[sS][eE][cC][oO][nN][dD][sS]?|[sS]|[sS][eE][cC][sS]?)(?![a-zA-Z]):\s*(\d+)|(?:[mM][iI][nN][uU][tT][eE][sS]?|m|[mM][iI][nN][sS]?)(?![a-zA-Z]):\s*(\d+)|(?:[hH][oO][uU][rR][sS]?|[hH])(?![a-zA-Z]):\s*(\d+)|(?:[dD][aA][yY][sS]?|[dD])(?![a-zA-Z]):\s*(\d+)|(?:[wW][eE][eE][kK][sS]?|[wW])(?![a-zA-Z]):\s*(\d+)|(?:[mM][oO][nN][tT][hH][sS]?|M)(?![a-zA-Z]):\s*(\d+)|(?:[yY][eE][aA][rR][sS]?|[yY])(?![a-zA-Z]):\s*(\d+)/g);

    let time: number | null = 0

    for (const matchs of match_form_back.toArray()) {
      for (const [i, value] of matchs.entries()) {
        if (!value) continue;
        const unit = parseInt(value, 10)
        if (i == 1) time = unit
        if (i == 2) time = unit*1000
        if (i == 3) time = unit*6000
        if (i == 4) time = unit*3600000
        if (i == 5) time = unit*86400000
        if (i == 6) time = unit*604800000
        if (i == 7) time = unit*2592000000
        if (i == 8) time = unit*31536000000
      }
    }

    if (time == 0) {
      time = null
    } else {
      time = Math.min(time, 2_419_200_000)
    }

    if (!member.moderatable) return await interaction.reply({ content: `You can not Timeout this user`, flags: MessageFlags.Ephemeral })

    const is_error = await member.timeout(time)
      .then(() => {return false;})
      .catch(
        (e) => {
          interaction.reply({ content: `Error could not timeout <@${user.id}>\nError: \`${e}\``, flags: MessageFlags.Ephemeral});
          return true
        }
      )
    if (is_error) return;

    const timeout_timestamp = Math.floor(member.communicationDisabledUntilTimestamp!/1000)

    if (time === null) {
      await interaction.reply({ content: `<@${user.id}> can speek now.`})
    } else if (time > max_timeout_time) {
      await interaction.reply({ content: `<@${user.id}> has decided to take a vow of silence until <t:${timeout_timestamp}:D>\n-# :warning: You can not time out a user for more that \`28 Days\``})
    } else {
      await interaction.reply({ content: `<@${user.id}> has decided to take a vow of silence until <t:${timeout_timestamp}:D>\n` })
    }
  },
} as Command;
