import { SlashCommandBuilder, ChatInputCommandInteraction, PermissionFlagsBits } from "discord.js";
import type { Command } from "../deploy";

export default {
  data: new SlashCommandBuilder()
    .setName('delete')
    .setDescription('A powerful tool to mass delete messages.')
    .addSubcommand((subcommand) => 
      subcommand
        .setName('user')
        .setDescription('Delete messages form a user.')
        .addUserOption((option) => option.setName('target').setDescription('The user to target').setRequired(true))
        .addIntegerOption((option) => option.setName('number_of_messages').setDescription('The number of messages to delete').setRequired(true))
        .addStringOption((option) => option.setName('after').setDescription('Filter for messages that were sent after a time. Format as: MM/DD/YY HH:MM').setRequired(false))
        .addStringOption((option) => option.setName('before').setDescription('Filter for messages that were sent before a time. Format as: MM/DD/YY HH:MM').setRequired(false))
        .addStringOption((option) => option.setName('regex').setDescription('Filter messages by regex').setRequired(false))
    )
    .addSubcommand((subcommand) => 
      subcommand
        .setName('channel')
        .setDescription('Delete messages form this channel.')
        .addIntegerOption((option) => option.setName('number_of_messages').setDescription('The number of messages to delete').setRequired(true))
        .addStringOption((option) => option.setName('after').setDescription('Filter for messages that were sent after a time. Format as: MM/DD/YY HH:MM').setRequired(false))
        .addStringOption((option) => option.setName('before').setDescription('Filter for messages that were sent before a time. Format as: MM/DD/YY HH:MM').setRequired(false))
        .addRoleOption((option) => option.setName('role').setDescription('Filter by role').setRequired(false))
        .addStringOption((option) => option.setName('regex').setDescription('Filter messages by regex').setRequired(false))
    )
    .addSubcommand((subcommand) => 
      subcommand
        .setName('server')
        .setDescription('Delete messages form all channels.')
        .addIntegerOption((option) => option.setName('number_of_messages').setDescription('The number of messages to delete').setRequired(true))
        .addStringOption((option) => option.setName('after').setDescription('Filter for messages that were sent after a time. Format as: MM/DD/YY HH:MM').setRequired(false))
        .addStringOption((option) => option.setName('before').setDescription('Filter for messages that were sent before a time. Format as: MM/DD/YY HH:MM').setRequired(false))
        .addRoleOption((option) => option.setName('role').setDescription('Filter by role').setRequired(false))
        .addStringOption((option) => option.setName('regex').setDescription('Filter messages by regex').setRequired(false))
    )
    .addSubcommand((subcommand) => 
      subcommand
        .setName('help')
        .setDescription('Explain how to use the command.')
    )
    .setDefaultMemberPermissions(PermissionFlagsBits.ManageMessages),
  async execute(interaction: ChatInputCommandInteraction) {
  },
} as Command;
