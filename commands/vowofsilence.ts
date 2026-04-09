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
    if (!user) return fail(interaction, "Failed to get the user");
    const guild = interaction.guild;
    if (!guild) return fail(interaction, "Failed to get guild");
    const member = guild.members.cache.get(user.id);
    if (!member) return fail(interaction, "Failed to get the member");
    const time_string = interaction.options.getString('time');
    if (!time_string) return fail(interaction, "Invalid Time string");

    const match_from_back = time_string?.matchAll(/(\d+)\s*(?:[mM][iI][lL][lL][iI][sS][eE][cC][oO][nN][dD][sS]?|[mM][sS]|[mM][iI][lL][lL][sS]?)(?![a-zA-Z])|(\d+)\s*(?:[sS][eE][cC][oO][nN][dD][sS]?|[sS]|[sS][eE][cC][sS]?)(?![a-zA-Z])|(\d+)\s*(?:[mM][iI][nN][uU][tT][eE][sS]?|m|[mM][iI][nN][sS]?)(?![a-zA-Z])|(\d+)\s*(?:[hH][oO][uU][rR][sS]?|[hH])(?![a-zA-Z])|(\d+)\s*(?:[dD][aA][yY][sS]?|[dD])(?![a-zA-Z])|(\d+)\s*(?:[wW][eE][eE][kK][sS]?|[wW])(?![a-zA-Z])|(\d+)\s*(?:[mM][oO][nN][tT][hH][sS]?|M)(?![a-zA-Z])|(\d+)\s*(?:[yY][eE][aA][rR][sS]?|[yY])(?![a-zA-Z])/g);
    const match_from_front = time_string?.matchAll(/(?:[mM][iI][lL][lL][iI][sS][eE][cC][oO][nN][dD][sS]?|[mM][sS]|[mM][iI][lL][lL][sS]?)(?![a-zA-Z]):\s*(\d+)|(?:[sS][eE][cC][oO][nN][dD][sS]?|[sS]|[sS][eE][cC][sS]?)(?![a-zA-Z]):\s*(\d+)|(?:[mM][iI][nN][uU][tT][eE][sS]?|m|[mM][iI][nN][sS]?)(?![a-zA-Z]):\s*(\d+)|(?:[hH][oO][uU][rR][sS]?|[hH])(?![a-zA-Z]):\s*(\d+)|(?:[dD][aA][yY][sS]?|[dD])(?![a-zA-Z]):\s*(\d+)|(?:[wW][eE][eE][kK][sS]?|[wW])(?![a-zA-Z]):\s*(\d+)|(?:[mM][oO][nN][tT][hH][sS]?|M)(?![a-zA-Z]):\s*(\d+)|(?:[yY][eE][aA][rR][sS]?|[yY])(?![a-zA-Z]):\s*(\d+)/g);

    let time: number | null = 0;

    for (const matchs of [...match_from_back.toArray(), ...match_from_front.toArray()]) {
      for (const [i, value] of matchs.entries()) {
        if (!value) continue;
        const unit = parseInt(value, 10)
        const durations = [1, 1000, 60_000, 3600000, 86400000, 604800000, 2592000000, 31536000000];
        time = unit * (durations[i - 2] || 1); // = durations[0] when i === 1
      }
    }

    if (time === 0) {
      time = null
    } else {
      time = Math.min(time, max_timeout_time)
    }

    if (!member.moderatable) return await interaction.reply({ content: `You can not Timeout this user`, flags: MessageFlags.Ephemeral });
    const reason = interaction.options.getString('reason') ?? undefined;

    const is_error = await member.timeout(time, reason)
      .then(() => false)
      .catch(
        (e) => {
          interaction.reply({ content: `Error could not timeout <@${user.id}>\nError: \`${e}\``, flags: MessageFlags.Ephemeral});
          return true
        }
      )
    if (is_error) return;

    const timeout_timestamp = Math.floor(member.communicationDisabledUntilTimestamp!/1000);

    if (time === null) {
      await interaction.reply({ content: `<@${user.id}> can speak now.`})
    } else  {
      let response = `<@${user.id}> has decided to take a vow of silence until <t:${timeout_timestamp}:D>\n${reason ? ' for ' + reason : ''}`;
      if (time > max_timeout_time) response += "\n-# :warning: You can not timeout a user for more than \`28 Days\`"
      await interaction.reply({ content: response })
    }
  },
} as Command;
