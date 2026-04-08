import { SlashCommandBuilder, ChatInputCommandInteraction, MessageFlags, InteractionResponse } from "discord.js";
import type { Command } from "../deploy";
import { sleep } from "bun";

export default {
    data: new SlashCommandBuilder()
        .setName('spam')
        .setDescription('spam someone\'s DM\'s.')
        .addUserOption(
          option => option
          .setName('user')
          .setDescription('The user to Spam')
          .setRequired(true)
        )
        .addStringOption((option) => option.setName('message').setDescription('The message to DM them').setRequired(true))
        .addStringOption((option) => option.setName('number').setDescription('Number of message (-1 for nonstop)').setRequired(true)),
    async execute(interaction: ChatInputCommandInteraction) {
      console.log(interaction.user.username);
      const authorized = 'rocketman510 jorby_official'.includes(interaction.user.username);
      if (!authorized) return interaction.reply({content: "you have no perms to run this D:", flags: MessageFlags.Ephemeral});

      let user = interaction.options.getUser('user');
      let number = parseInt(interaction.options.getString('number') || "", 10);

      if (isNaN(number)) {
        interaction.reply({content:"Invalid Number", flags:MessageFlags.Ephemeral});
      }

      interaction.reply({content:"Started to Spam", flags:MessageFlags.Ephemeral});

      console.log(interaction.client.shouldStopSpam);

      let i = 0;
      while (!interaction.client.shouldStopSpam && i !== number) {
        i++
        console.log(interaction.client.shouldStopSpam);

        await sleep(1000)
        await user?.send(interaction.options.getString('message')!);
      }

      if (interaction.client.shouldStopSpam) {
        interaction.client.shouldStopSpam = false;
      }

    },
} as Command;
