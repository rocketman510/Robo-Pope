import { SlashCommandBuilder, ChatInputCommandInteraction, MessageFlags, SectionBuilder, ButtonStyle, ActionRowBuilder, ButtonBuilder } from "discord.js";
import { execSync } from 'child_process';
import type { Command } from "../deploy";


export default {
  data: new SlashCommandBuilder()
    .setName('update')
    .setDescription('This command updates the bot to the latest version.'),
  async execute(interaction: ChatInputCommandInteraction) {
    console.log(interaction.user.username);
    let developers = process.env.DEVELOPER_IDS || '[]';


    if (!developers.includes(interaction.user.id)) return interaction.reply({ content: 'You are not a developer, so you cannot use this command.', flags: MessageFlags.Ephemeral });

    const version = await getVersion();
    const latest = await getLatestVersion();
    console.log(version, latest);

    const message = getLatestTagMessage();

    const updateSection = new SectionBuilder()
      .addTextDisplayComponents(
        (textDisplay) => textDisplay.setContent(`The Bot has an update.\nCurrent: \`${version}\` => \`${latest}\``,),
        (textDisplay) => textDisplay.setContent(`\`\`\`diff\n${message}\`\`\``),
      )
      .setButtonAccessory(
        (button) => button.setCustomId('update').setLabel('Update the Bot').setStyle(ButtonStyle.Success),
      );

    const update_message_button = new ButtonBuilder()
      .setLabel('Send Message')
      .setStyle(ButtonStyle.Primary)
      .setCustomId('send_update_message');

    const action_row = new ActionRowBuilder()
      .addComponents(update_message_button)

    if (version === latest) {
      interaction.reply({content: `The Bot is up-to-date. Version: \`${version}\``, components: [action_row], flags: MessageFlags.Ephemeral})
    } else {
      interaction.reply({components: [updateSection], flags: MessageFlags.Ephemeral | MessageFlags.IsComponentsV2})
    }
  },
} as Command;

async function getLatestVersion() {
  const owner = "rocketman510";
  const repo = "Robo-Pope";
  const token = process.env.GITHUB_TOKEN;

  if (!token) throw new Error("No GITHUB_TOKEN");
  const res = await fetch(`https://api.github.com/repos/${owner}/${repo}/releases/latest`, {
    headers: {
      Accept: "application/vnd.github+json",
      Authorization: `Bearer ${token}`,
      "X-GitHub-Api-Version": "2022-11-28"
    }
  });

  if (!res.ok) {
    if (res.status === 404) return null;
    throw new Error(`GitHub API error: ${res.status}`);
  }

  const data = await res.json();
  return data.tag_name;
}

async function getVersion() {
  return execSync('git describe --tags --exact-match 2>/dev/null || git rev-parse --short HEAD')
    .toString()
    .trim();
}

function getLatestTagMessage(): string {
  const latestTag = execSync('git describe --tags --abbrev=0').toString().trim();
  const message = execSync(`git tag -l --format='%(contents)' ${latestTag}`).toString().trim();
  return message || 'No tag message';
}
